//! K380 hotplug handling using the system run loop; no timer or extra crate.
use std::ffi::{c_char, c_void};
use std::ptr;

type Ref = *const c_void;
type Device = *mut c_void;
type DeviceCallback = extern "C" fn(*mut c_void, i32, *mut c_void, Device);

#[link(name = "IOKit", kind = "framework")]
extern "C" {
    fn IOHIDManagerCreate(allocator: Ref, options: u32) -> *mut c_void;
    fn IOHIDManagerSetDeviceMatching(manager: *mut c_void, matching: Ref);
    fn IOHIDManagerRegisterDeviceMatchingCallback(
        manager: *mut c_void,
        callback: DeviceCallback,
        context: *mut c_void,
    );
    fn IOHIDManagerScheduleWithRunLoop(manager: *mut c_void, run_loop: Ref, mode: Ref);
    fn IOHIDManagerUnscheduleFromRunLoop(manager: *mut c_void, run_loop: Ref, mode: Ref);
    fn IOHIDManagerOpen(manager: *mut c_void, options: u32) -> i32;
    fn IOHIDManagerClose(manager: *mut c_void, options: u32) -> i32;
    fn IOHIDDeviceOpen(device: Device, options: u32) -> i32;
    fn IOHIDDeviceClose(device: Device, options: u32) -> i32;
    fn IOHIDDeviceSetReport(
        device: Device,
        report_type: i32,
        report_id: isize,
        report: *const u8,
        length: isize,
    ) -> i32;
}

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
    static kCFTypeDictionaryKeyCallBacks: u8;
    static kCFTypeDictionaryValueCallBacks: u8;
    static kCFRunLoopDefaultMode: Ref;
    fn CFDictionaryCreateMutable(
        allocator: Ref,
        capacity: isize,
        key_callbacks: Ref,
        value_callbacks: Ref,
    ) -> *mut c_void;
    fn CFDictionarySetValue(dictionary: *mut c_void, key: Ref, value: Ref);
    fn CFStringCreateWithCString(allocator: Ref, string: *const c_char, encoding: u32) -> Ref;
    fn CFNumberCreate(allocator: Ref, number_type: isize, value: *const i32) -> Ref;
    fn CFRelease(object: Ref);
    fn CFRunLoopGetCurrent() -> Ref;
    fn CFRunLoopRun();
}

struct Context {
    report: [u8; 7],
}

// IOHIDReportType.output == 1. The report ID is also passed separately.
extern "C" fn matched(context: *mut c_void, _: i32, _: *mut c_void, device: Device) {
    let report = unsafe { &(*(context as *const Context)).report };
    unsafe {
        let opened = IOHIDDeviceOpen(device, 0);
        if opened != 0 {
            eprintln!("K380: 无法打开 HID 设备: {opened:#x}");
            return;
        }

        // macOS HID interfaces differ in whether SetReport includes the ID in
        // the buffer. Try the full report first, then its six-byte payload.
        let mut result = IOHIDDeviceSetReport(device, 1, 0x10, report.as_ptr(), 7);
        if result != 0 {
            result = IOHIDDeviceSetReport(device, 1, 0x10, report[1..].as_ptr(), 6);
        }
        if result != 0 {
            eprintln!("K380: 无法发送 Fn 设置: {result:#x}");
        }
        IOHIDDeviceClose(device, 0);
    }
}

// CFDictionary owns these keys and numbers after SetValue (type callbacks).
unsafe fn matching_dictionary() -> Result<*mut c_void, Box<dyn std::error::Error>> {
    let dict = CFDictionaryCreateMutable(
        ptr::null(),
        2,
        ptr::addr_of!(kCFTypeDictionaryKeyCallBacks).cast(),
        ptr::addr_of!(kCFTypeDictionaryValueCallBacks).cast(),
    );
    if dict.is_null() {
        return Err("无法创建 HID 匹配条件".into());
    }
    for (key, value) in [
        (b"VendorID\0".as_ptr(), 0x046d),
        (b"ProductID\0".as_ptr(), 0xb342),
    ] {
        let key = CFStringCreateWithCString(ptr::null(), key.cast(), 0x0800_0100);
        let number = CFNumberCreate(ptr::null(), 3, &value); // kCFNumberSInt32Type
        if key.is_null() || number.is_null() {
            if !key.is_null() {
                CFRelease(key);
            }
            if !number.is_null() {
                CFRelease(number);
            }
            CFRelease(dict);
            return Err("无法创建 HID 匹配条件".into());
        }
        CFDictionarySetValue(dict, key, number);
        CFRelease(key);
        CFRelease(number);
    }
    Ok(dict)
}

pub fn watch(report: &[u8; 7]) -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        let manager = IOHIDManagerCreate(ptr::null(), 0);
        if manager.is_null() {
            return Err("无法创建 HID manager".into());
        }
        let matching = match matching_dictionary() {
            Ok(matching) => matching,
            Err(error) => {
                CFRelease(manager);
                return Err(error);
            }
        };
        IOHIDManagerSetDeviceMatching(manager, matching);
        CFRelease(matching);

        let mut context = Context { report: *report };
        IOHIDManagerRegisterDeviceMatchingCallback(
            manager,
            matched,
            (&mut context as *mut Context).cast(),
        );
        let run_loop = CFRunLoopGetCurrent();
        IOHIDManagerScheduleWithRunLoop(manager, run_loop, kCFRunLoopDefaultMode);
        let result = IOHIDManagerOpen(manager, 0);
        if result != 0 {
            IOHIDManagerUnscheduleFromRunLoop(manager, run_loop, kCFRunLoopDefaultMode);
            CFRelease(manager);
            return Err(format!("无法打开 HID manager: {result:#x}").into());
        }
        // Initial enumeration also invokes matched; reconnections invoke it
        // again. The run loop sleeps between device events.
        CFRunLoopRun();
        IOHIDManagerUnscheduleFromRunLoop(manager, run_loop, kCFRunLoopDefaultMode);
        IOHIDManagerClose(manager, 0);
        CFRelease(manager);
        Ok(())
    }
}
