# k380-fn-lock rust实现

切换罗技 K380 蓝牙键盘的按键模式：F1–F12 功能键模式 或 媒体键模式。

## 用法

```bash
# 切换为 F1–F12 功能键模式
setFnKeys

# 切换为媒体键（音量、播放等）模式
setMediaKeys

# macOS：等待键盘连接，并在每次重新连接时恢复设置
setFnKeys --watch
```

macOS 的 `--watch` 由 IOHIDManager 连接事件驱动，不定时枚举设备。
命令行模式会尝试 K380 的各个 HID 接口；Linux/Windows 保持原有的接口筛选与一次性写入。
当前报文针对 VID `046d`、PID `b342` 的 K380，仍使用该型号已知的 HID++ feature index `0x0b`。

## 编译

```bash
cargo build --release
```

产物位于 `target/release/`。

## macOS 登录后自动设置

先将编译好的 `setFnKeys` 放到 `~/.local/bin/`。在
`~/Library/LaunchAgents/com.kongdd.k380-fn.plist` 创建以下文件，
将 `YOUR_USERNAME` 换成实际用户名（launchd 不展开 `~`）：

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
  "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key><string>com.kongdd.k380-fn</string>
  <key>ProgramArguments</key>
  <array>
    <string>/Users/YOUR_USERNAME/.local/bin/setFnKeys</string>
    <string>--watch</string>
  </array>
  <key>RunAtLoad</key><true/>
  <key>KeepAlive</key><true/>
</dict>
</plist>
```

```bash
launchctl bootstrap "gui/$(id -u)" \
  "$HOME/Library/LaunchAgents/com.kongdd.k380-fn.plist"
launchctl print "gui/$(id -u)/com.kongdd.k380-fn"
```

服务随用户登录启动；键盘当时尚未连接也会在连接时设置。
`setMediaKeys --watch` 同样可用，二者只需运行一个。
如 macOS 要求输入监控权限，请为该程序授权后重新启动服务。

## Linux 额外配置

**安装 hidapi 依赖：**

```bash
# Ubuntu/Debian
sudo apt install libhidapi-dev

# Arch Linux
sudo pacman -S hidapi
```

**添加 udev 规则（避免每次都需要 sudo）：**

```bash
echo 'SUBSYSTEM=="hidraw", ATTRS{idVendor}=="046d", ATTRS{idProduct}=="b342", MODE="0666"' \
  | sudo tee /etc/udev/rules.d/99-k380.rules
sudo udevadm control --reload-rules && sudo udevadm trigger
```

## 设备信息

| 参数       | 值                   |
| ---------- | -------------------- |
| Vendor ID  | `0x046d`（Logitech） |
| Product ID | `0xb342`             |


## References

- <https://github.com/dheygere/k380-fn-lock-for-windows>

- [hidapi](https://crates.io/crates/hidapi)
