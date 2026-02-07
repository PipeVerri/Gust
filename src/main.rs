use clap::Parser;

mod project;
mod cli;
mod error;

fn main() {
    let cli = cli::Cli::parse();
    let result = cli.command.run();
    if let Err(e) = result {
        eprintln!("{}", e);
    }
}