fn main() {
    let args: Vec<String> = std::env::args().collect();
    std::process::exit(ap_probe::run_cli(&args));
}
