use std::path::PathBuf;
use clap::{Parser, Subcommand};
use crate::project::Project;
use crate::error::Result;

#[derive(Parser)]
#[command(name = "Gust")]
#[command(about = "A rust-based version control system")]
pub struct Cli {
    // Tell clap that this is a subcommand, don't try to parse the text into this enum-type
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Init,
    // Struct-variant commands will still take positional arguments. They can't be tuple-variants because clap doesn't know the arg names
    Add {
        paths: Vec<PathBuf>
    },
    Status
}

impl Commands {
    pub fn run(&self) -> Result<()> {
        match self {
            Commands::Init => Project::create_project(),
            other => {
                let mut project = Project::new()?;
                match other {
                    Commands::Add { paths } => project.add(paths),
                    Commands::Status => project.status(),
                    _ => unreachable!() // Panics if it reaches this
                }
            }
        }
    }
}