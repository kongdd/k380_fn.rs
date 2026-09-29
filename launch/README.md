# 登录时运行（macOS）

先在项目根目录运行 `cargo install --path .`，确保 `~/.cargo/bin/setFnKeys` 存在。

```sh
cp launch/local.k380.set-fn-keys.plist ~/Library/LaunchAgents/
launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/local.k380.set-fn-keys.plist
```

每次登录运行；失败时每隔 30 秒重试，最多尝试 3 次。日志：`~/Library/Logs/k380-fn.log`。

更新已有配置时，先执行 `launchctl bootout gui/$(id -u)/local.k380.set-fn-keys`，再复制并执行上述 `bootstrap` 命令。
