use hidapi::HidApi;

#[cfg(target_os = "macos")]
mod macos;

const K380_VID: u16 = 0x046d;
const K380_PID: u16 = 0xb342;
#[cfg(not(target_os = "macos"))]
const TARGET_USAGE: u16 = 1;
#[cfg(not(target_os = "macos"))]
const TARGET_USAGE_PAGE: u16 = 0xff00;

const K380_SEQ_FKEYS_ON: [u8; 7] = [0x10, 0xff, 0x0b, 0x1e, 0x00, 0x00, 0x00];
const K380_SEQ_FKEYS_OFF: [u8; 7] = [0x10, 0xff, 0x0b, 0x1e, 0x01, 0x00, 0x00];

pub fn k380_set_fn_keys(fn_keys: bool) -> Result<(), Box<dyn std::error::Error>> {
    let api = HidApi::new()?;

    let seq = if fn_keys {
        &K380_SEQ_FKEYS_ON
    } else {
        &K380_SEQ_FKEYS_OFF
    };

    #[cfg(target_os = "macos")]
    {
        // A K380 exposes several HID interfaces on macOS. Try each interface,
        // since usage_page/usage are not a reliable way to identify HID++.
        let mut last_error = None;
        let mut found = false;
        let mut sent = false;
        for info in api
            .device_list()
            .filter(|d| d.vendor_id() == K380_VID && d.product_id() == K380_PID)
        {
            found = true;
            match info
                .open_device(&api)
                .and_then(|device| device.send_output_report(seq))
            {
                Ok(()) => sent = true,
                Err(error) => last_error = Some(error),
            }
        }
        if sent {
            return Ok(());
        }
        if !found {
            return Err("K380 设备未找到，请确认键盘已连接且已配对".into());
        }
        return Err(last_error.expect("at least one device was tried").into());
    }

    #[cfg(not(target_os = "macos"))]
    {
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

        let written = device.write(seq)?;
        if written != seq.len() {
            return Err(format!("只写入了 {} / {} 字节", written, seq.len()).into());
        }

        Ok(())
    }
}

pub fn run(fn_keys: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    match (args.next().as_deref(), args.next()) {
        (None, None) => k380_set_fn_keys(fn_keys),
        (Some("--watch"), None) => {
            #[cfg(target_os = "macos")]
            {
                watch_fn_keys(fn_keys)
            }
            #[cfg(not(target_os = "macos"))]
            {
                Err("--watch 目前仅支持 macOS".into())
            }
        }
        _ => Err("用法: setFnKeys [--watch] 或 setMediaKeys [--watch]".into()),
    }
}

#[cfg(target_os = "macos")]
pub fn watch_fn_keys(fn_keys: bool) -> Result<(), Box<dyn std::error::Error>> {
    macos::watch(if fn_keys {
        &K380_SEQ_FKEYS_ON
    } else {
        &K380_SEQ_FKEYS_OFF
    })
}
