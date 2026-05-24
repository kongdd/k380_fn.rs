// use k380_fn_lock::k380_set_fn_keys;
fn main() {
    if let Err(e) = k380_fn_lock::k380_set_fn_keys(false) {
        eprintln!("错误: {}", e);
        std::process::exit(1);
    }
}
