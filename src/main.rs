fn main() {
    if let Err(error) = k380_fn_lock::run(true) {
        eprintln!("错误: {error}");
        std::process::exit(1);
    }
}
