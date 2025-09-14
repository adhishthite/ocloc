mod analyzer;
mod cli;
mod formatters;
mod languages;
mod traversal;
mod types;
mod types_diff;
mod vcs;

fn main() {
    // Delegate to CLI runner; errors are printed nicely inside.
    if let Err(err) = cli::run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
