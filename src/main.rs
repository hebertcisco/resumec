use clap::Parser;
use resumec::{Cli, MachineResult, print_json, run_cli};

fn main() {
    let json_output = std::env::args().any(|arg| arg == "--json-output");
    let cli = Cli::parse();
    if let Err(error) = run_cli(cli) {
        if json_output {
            let _ = print_json(&MachineResult::<serde_json::Value>::error(error.issues()));
        } else {
            eprintln!("error: {error}");
        }
        std::process::exit(error.exit_code());
    }
}
