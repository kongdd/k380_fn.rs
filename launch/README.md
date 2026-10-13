# 登录时运行（macOS）

在项目根目录执行（需已安装 Rust）：

```sh
sh launch/install.sh
```

脚本安装并签名程序、创建所需目录并注册当前用户的 LaunchAgent；立即执行一次，此后每次登录自动运行，无需 sudo。更新时再次执行即可。

## 固定签名与权限

首次安装自动创建有效期 20 年的本地代码签名证书 `K380 Local Code Signing`，证书与私钥保存在当前用户的默认钥匙串中；信任仅限代码签名。macOS 可能要求确认信任或允许 `codesign` 访问私钥。

首次切换到此证书后，需在「系统设置 → 隐私与安全 → 输入监控」中移除旧 `setFnKeys`，重新添加 `~/.cargo/bin/setFnKeys` 并启用。以后通过安装脚本更新时，复用同一证书、程序标识及签名要求，通常无需重新授权。

已有程序可单独签名，不重新编译：

```sh
sh launch/sign.sh
```

请保留钥匙串中的证书与私钥。更换证书、证书过期、移动程序或直接覆盖为未签名版本，可能需要重新授权。直接使用 `cargo install` 更新后，应再运行签名脚本。

## 重试与日志

首次执行失败后，每隔 30 秒重试，最多重试 3 次（共最多执行 4 次）；成功即退出，不持续运行。日志：`~/Library/Logs/k380-fn.log`。

## 停用

```sh
launchctl bootout gui/$(id -u)/local.k380.set-fn-keys
rm ~/Library/LaunchAgents/local.k380.set-fn-keys.plist
```
