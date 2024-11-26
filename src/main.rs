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
                println!("  -E, --error    Enable error mode");
                return;
            }
            flags::Flag::Verbose => {
                let mut store = state.lock();
                store.verbose_enabled = true;
            }
            flags::Flag::Error => {
                let mut store = state.lock();
                store.error_enabled = true;
            }
        }
    }

    if !check_nx_available() {
        eprintln!(
            "nx is NOT installed globally. Currently nx is required to be installed globally."
        );
        eprintln!("Please install nx globally and try again.");
        // pkg manager instructions
        eprintln!("For npm:");
        eprintln!("  npm install -g nx");
        eprintln!("For the package manager with the cat:");
        eprintln!("  yarn global add nx");
        eprintln!("For pnpm:");
        eprintln!("  pnpm add -g nx");
        return;
    }
    let search_path = args.get(1).map(String::as_str).unwrap_or(".");

    let _ = ui::terminal::setup();
    let _ = ui::terminal::run_app(search_path.to_string());
    let _ = ui::terminal::cleanup();
}

fn check_nx_available() -> bool {
    let state = state::State::global();
    let store = state.lock();
    if store.error_enabled {
        return false;
    }
    if let Ok(output) = std::process::Command::new("which").arg("nx").output() {
        return output.status.success();
    }

    if let Ok(output) = std::process::Command::new("where").arg("nx").output() {
        return output.status.success();
    }

    false
}
