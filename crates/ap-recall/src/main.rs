fn main() {
    let args: Vec<String> = std::env::args().collect();
    std::process::exit(ap_recall::run_cli(&args));
}
