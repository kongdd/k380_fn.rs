#!/bin/sh
set -eu

if [ "$(uname -s)" != Darwin ]; then
    echo '此脚本仅支持 macOS。' >&2
    exit 1
fi

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
label=local.k380.set-fn-keys
agent="$HOME/Library/LaunchAgents/$label.plist"
domain="gui/$(id -u)"

# 固定安装路径，与 LaunchAgent 中的命令保持一致。
cargo install --locked --path "$root" --root "$HOME/.cargo"
sh "$root/launch/sign.sh"
mkdir -p "$HOME/Library/LaunchAgents" "$HOME/Library/Logs"
launchctl bootout "$domain/$label" 2>/dev/null || :
cp "$root/launch/$label.plist" "$agent"
launchctl bootstrap "$domain" "$agent"

printf '%s\n' '已启用登录自动运行，并立即执行。' "日志：$HOME/Library/Logs/k380-fn.log"
