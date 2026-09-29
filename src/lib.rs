use hidapi::HidApi;

#[cfg(target_os = "macos")]
mod karabiner;

const K380_VID: u16 = 0x046d;
const K380_PID: u16 = 0xb342;
const TARGET_USAGE: u16 = 1;
const TARGET_USAGE_PAGE: u16 = 65280;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn k380_set_fn_keys(fn_keys: bool) -> Result<()> {
    let result = set_fn_keys(fn_keys);
    #[cfg(target_os = "macos")]
    if is_exclusive_error(&result) {
        eprintln!("设备被独占，临时释放 Karabiner 对 K380 的接管…");
        let mut release = karabiner::Release::new()?;
        let mut result = result;
        for _ in 0..20 {
            std::thread::sleep(std::time::Duration::from_millis(250));
            result = set_fn_keys(fn_keys);
            if !is_exclusive_error(&result) {
                break;
            }
        }
        release.restore()?;
        return result;
    }
    result
}

#[cfg(target_os = "macos")]
fn is_exclusive_error(result: &Result<()>) -> bool {
    result
        .as_ref()
        .is_err_and(|e| e.to_string().contains("0xE00002C5"))
}

fn set_fn_keys(fn_keys: bool) -> Result<()> {
    let api = HidApi::new()?;
    #[cfg(target_os = "macos")]
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

    let seq = [0x10, 0xff, 0x0b, 0x1e, u8::from(!fn_keys), 0, 0];
    let written = device.write(&seq)?;
    if written != seq.len() {
        return Err(format!("只写入了 {written} / {} 字节", seq.len()).into());
    }
    let mode = if fn_keys { "function" } else { "media" };
    println!("Set {mode} keys as default");
    Ok(())
}
