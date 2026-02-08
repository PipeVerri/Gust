use clap::Parser;

mod project;
mod cli;

fn main() {
    let cli = cli::Cli::parse();
    let result = cli.command.run();
    if let Err(e) = result {
        eprintln!("{}", e);
    }
}