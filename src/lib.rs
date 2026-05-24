use hidapi::HidApi;

const K380_VID: u16 = 0x046d;
const K380_PID: u16 = 0xb342;
const TARGET_USAGE: u16 = 1;
const TARGET_USAGE_PAGE: u16 = 65280;

const K380_SEQ_FKEYS_ON:  [u8; 7] = [0x10, 0xff, 0x0b, 0x1e, 0x00, 0x00, 0x00];
const K380_SEQ_FKEYS_OFF: [u8; 7] = [0x10, 0xff, 0x0b, 0x1e, 0x01, 0x00, 0x00];

pub fn k380_set_fn_keys(fn_keys: bool) -> Result<(), Box<dyn std::error::Error>> {
    let api = HidApi::new()?;

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

    let (seq, label) = if fn_keys {
        (&K380_SEQ_FKEYS_ON, "Set function keys as default")
    } else {
        (&K380_SEQ_FKEYS_OFF, "Set media keys as default")
    };

    println!("{}", label);

    let written = device.write(seq)?;
    if written != seq.len() {
        eprintln!("警告：只写入了 {} / {} 字节", written, seq.len());
    }

    Ok(())
}
