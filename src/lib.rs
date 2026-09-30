use hidapi::HidApi;

#[cfg(target_os = "macos")]
mod karabiner;

const K380_VID: u16 = 0x046d;
const K380_PID: u16 = 0xb342;
// 选择厂商自定义 HID 接口发送设置命令，而非普通键盘接口。
const TARGET_USAGE: u16 = 1;
const TARGET_USAGE_PAGE: u16 = 65280;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// true：默认 F1–F12；false：默认媒体键。
pub fn k380_set_fn_keys(fn_keys: bool) -> Result<()> {
    let result = set_fn_keys(fn_keys);
    #[cfg(target_os = "macos")]
    if is_exclusive_error(&result) {
        eprintln!("设备被独占，临时释放 Karabiner 对 K380 的接管…");
        let mut release = karabiner::Release::new()?;
        let mut result = result;
        // 等待 Karabiner 异步释放设备，最多重试 5 秒。
        for _ in 0..20 {
            std::thread::sleep(std::time::Duration::from_millis(250));
            result = set_fn_keys(fn_keys);
            if !is_exclusive_error(&result) {
                break;
            }
        }
        // 无论设置成功与否，都恢复 Karabiner 对设备的接管。
        release.restore()?;
        return result;
    }
    result
}

#[cfg(target_os = "macos")]
// IOKit 的 kIOReturnExclusiveAccess：设备已被其他程序独占。
fn is_exclusive_error(result: &Result<()>) -> bool {
    result
        .as_ref()
        .is_err_and(|e| e.to_string().contains("0xE00002C5"))
}

fn set_fn_keys(fn_keys: bool) -> Result<()> {
    let api = HidApi::new()?;
    #[cfg(target_os = "macos")]
    // 共享打开，避免影响其他程序使用键盘。
    api.set_open_exclusive(false);

    let device_info = api
        .device_list()
        .find(|d| {
            d.vendor_id() == K380_VID
                && d.product_id() == K380_PID
                && d.usage() == TARGET_USAGE
                && d.usage_page() == TARGET_USAGE_PAGE
        })
        .ok_or("K380 设备未找到，请确认键盘已连接且已配对")?;

    let device = device_info.open_device(&api)?;

    // K380 模式设置报文：第 5 字节为 0（功能键）或 1（媒体键）。
    let seq = [0x10, 0xff, 0x0b, 0x1e, u8::from(!fn_keys), 0, 0];
    let written = device.write(&seq)?;
    if written != seq.len() {
        return Err(format!("只写入了 {written} / {} 字节", seq.len()).into());
    }
    let mode = if fn_keys {
        "F1–F12 功能键"
    } else {
        "媒体键"
    };
    println!("K380：默认使用{mode}");
    Ok(())
}
