// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--selftest") {
        let report = args
            .windows(2)
            .find(|w| w[0] == "--report")
            .map(|w| w[1].as_str());
        let code = match boostx_lib::selftest::run_selftest(report.map(std::path::Path::new)) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("selftest error: {e}");
                2
            }
        };
        std::process::exit(code);
    }
    boostx_lib::run()
}
