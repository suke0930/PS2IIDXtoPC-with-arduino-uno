mod cli;
mod env;
mod launcher;
mod mapping;
mod outputs;
mod serial;
mod types;

fn main() {
    if let Err(e) = cli::run_cli() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
