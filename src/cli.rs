use std::path::PathBuf;
use clap::{Parser, Subcommand};
use crate::project::root::Root;
use crate::project::error::Result;
use crate::project::root::checkout::CheckoutMode;

#[derive(Parser)]
#[command(name = "Gust")]
#[command(about = "A rust-based version control system")]
pub struct Cli {
    // Tell clap that this is a subcommand, don't try to parse the text into this enum-type
    #[clap(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Init,
    // Struct-variant commands will still take positional arguments. They can't be tuple-variants because clap doesn't know the arg names
    Add {
        paths: Vec<PathBuf>
    },
    Rm {
        paths: Vec<PathBuf>
    },
    Commit {
        #[arg(short, long, default_value = "")]
        message: String
    },
    Status,
    Log,
    Branch {
        branch_name: Option<String>
    },
    Checkout {
        name: String,
        #[arg(long, short, value_enum)]
        mode: Option<CheckoutMode>,
    },
    Stash {
        #[clap(subcommand)]
        command: Option<StashCommand>
    }
}

#[derive(Subcommand)]
enum StashCommand {
    Save {
        #[arg(short, long)]
        message: Option<String>
    },
    Apply {
        stash_id: u32
    },
    Drop {
        stash_id: u32
    },
    Pop {
        stash_id: u32
    }
}

impl Commands {
    pub fn run(&self) -> Result<()> {
        match self {
            Commands::Init => Root::create_project(),
            other => {
                let mut project = Root::new()?;
                match other {
                    Commands::Add { paths } => project.add(paths),
                    Commands::Rm { paths } => project.remove(paths),
                    Commands::Commit { message } => project.commit(message.clone()),
                    Commands::Status => project.status(),
                    Commands::Log => project.log(),
                    Commands::Branch { branch_name } => project.branch(branch_name),
                    Commands::Checkout { mode, name } => project.checkout(mode, name),
                    Commands::Stash { command } => match command {
                        None => project.save_stash(None),
                        Some(stash_command) => match stash_command {
                            StashCommand::Save { message } => project.save_stash(message.clone()),
                            StashCommand::Apply { stash_id } => project.apply_stash(stash_id.clone()),
                            StashCommand::Drop { stash_id } => project.drop_stash(stash_id.clone()),
                            StashCommand::Pop { stash_id } => project.pop_stash(stash_id.clone()),
                        }
                    },
                    _ => unreachable!(), // Panics if it reaches this
                }
            }
        }
    }
}