fn main() {
    if let Err(e) = k380_fn_lock::run(false) {
        eprintln!("错误: {}", e);
        std::process::exit(1);
    }
}
