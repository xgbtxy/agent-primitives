fn main() {
    let args: Vec<String> = std::env::args().collect();
    std::process::exit(ap_baton::run_cli(&args));
}
