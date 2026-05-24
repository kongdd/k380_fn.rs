# k380-fn-lock rust实现

切换罗技 K380 蓝牙键盘的按键模式：F1–F12 功能键模式 或 媒体键模式。

## 用法

```bash
# 切换为 F1–F12 功能键模式
setFnKeys

# 切换为媒体键（音量、播放等）模式
setMediaKeys
```

## 编译

```bash
cargo build --release
```

产物位于 `target/release/`。

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

## 依赖

- [hidapi](https://crates.io/crates/hidapi)
