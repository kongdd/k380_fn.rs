#!/bin/sh
set -eu

if [ "$(uname -s)" != Darwin ]; then
    echo '此脚本仅支持 macOS。' >&2
    exit 1
fi

name='K380 Local Code Signing'
keychain=$(security default-keychain -d user | awk -F '"' '{print $2}')
[ -n "$keychain" ] || { echo '未找到用户钥匙串。' >&2; exit 1; }

for binary in setFnKeys setMediaKeys; do
    [ -x "$HOME/.cargo/bin/$binary" ] || {
        echo "未安装 $binary，请先运行安装脚本。" >&2
        exit 1
    }
done

signing_identity() {
    security find-identity -v -p codesigning "$keychain" \
        | awk -v name="$name" 'index($0, "\"" name "\"") {print $2; exit}'
}

identity=$(signing_identity)
if [ -z "$identity" ]; then
    umask 077
    tmp=$(mktemp -d)
    trap 'rm -rf "$tmp"' EXIT
    trap 'exit 1' HUP INT TERM

    # 证书与私钥仅创建一次；长期存放于用户钥匙串。
    if ! security find-certificate -c "$name" "$keychain" >/dev/null 2>&1; then
        printf '%s\n' '[req]' 'distinguished_name=dn' 'x509_extensions=signing' \
            '[dn]' '[signing]' 'basicConstraints=critical,CA:FALSE' \
            'keyUsage=critical,digitalSignature' 'extendedKeyUsage=codeSigning' > "$tmp/openssl.cnf"
        openssl req -x509 -newkey rsa:2048 -sha256 -days 7300 -nodes \
            -subj "/CN=$name" -config "$tmp/openssl.cnf" \
            -keyout "$tmp/key.pem" -out "$tmp/cert.pem"
        # 临时目录仅当前用户可访问；PKCS#12 使用 macOS 兼容格式。
        openssl pkcs12 -export -inkey "$tmp/key.pem" -in "$tmp/cert.pem" \
            -name "$name" -keypbe PBE-SHA1-3DES -certpbe PBE-SHA1-3DES -macalg sha1 \
            -passout pass: -out "$tmp/signing.p12"
        security import "$tmp/signing.p12" -k "$keychain" -P '' -T /usr/bin/codesign
    fi

    # 只信任代码签名，不授予 SSL 等其他用途的信任。
    security find-certificate -c "$name" -p "$keychain" > "$tmp/cert.pem"
    security add-trusted-cert -r trustRoot -p codeSign -k "$keychain" "$tmp/cert.pem"
    identity=$(signing_identity)
    [ -n "$identity" ] || { echo '签名证书不可用，请检查钥匙串中的证书与私钥。' >&2; exit 1; }
fi

for binary in setFnKeys setMediaKeys; do
    identifier="local.k380.$binary"
    # 固定证书和标识，而非二进制哈希，供 TCC 跨版本验证身份。
    codesign --force --sign "$identity" --keychain "$keychain" --timestamp=none \
        --identifier "$identifier" \
        --requirements "=designated => identifier \"$identifier\" and certificate leaf = H\"$identity\"" \
        "$HOME/.cargo/bin/$binary"
    codesign --verify --strict "$HOME/.cargo/bin/$binary"
done
printf '%s\n' '已使用固定证书签名。'
