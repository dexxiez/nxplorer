mod detection;
mod flags;
mod state;
mod ui;
mod utils;

use std::env;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Vec<String> = env::args().collect();
    let parsed_flags = flags::parse_args(&args);
    let state = state::State::global();
    for flag in parsed_flags {
        match flag {
            flags::Flag::Version => {
                println!("{} {}", NAME, VERSION);
                return;
            }
            flags::Flag::Debug => {
                println!("Debug mode enabled, which does nothing lmao gottemmm");
                return;
            }
            flags::Flag::Help => {
                println!("Usage: {} [options]", NAME);
                println!();
                println!("Options:");
                println!("  -h, --help     Print this help menu");
                println!("  -v, --version  Print the version");
                println!("  -d, --debug    Enable debug mode");
                println!("  -V, --verbose  Enable verbose mode");
                return;
            }
            flags::Flag::Verbose => {
                let mut store = state.lock();
                store.verbose_enabled = true;
            }
        }
    }
    let search_path = args.get(1).map(String::as_str).unwrap_or(".");

    let _ = ui::terminal::setup();
    let _ = ui::terminal::run_app(search_path.to_string());
    let _ = ui::terminal::cleanup();
}
