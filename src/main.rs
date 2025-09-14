mod analyzer;
mod cli;
mod formatters;
mod languages;
mod traversal;
mod types;

fn main() {
    // Delegate to CLI runner; errors are printed nicely inside.
    if let Err(err) = cli::run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
