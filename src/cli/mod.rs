use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};

pub use self::{config::*, publish::*};

mod config;
mod publish;

#[derive(Debug, Parser)]
#[command(name = "rimpub", version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Publish the current folder as a RimWorld mod to configured path,
    /// usually the RimWorld local mods folder.
    #[command(aliases = ["pub", "p"])]
    Publish(PublishArgs),
    /// Configure the mod publishing settings.
    #[command(aliases = ["cfg", "c"])]
    Config(ConfigArgs),
}

impl Cli {
    pub fn run(&self) -> Result<()> {
        match self.command {
            Command::Config(ref args) => args.run(),
            Command::Publish(ref args) => args.run(),
        }
        .map_err(|e| anyhow!("Unexpected error during exec: {e}"))
    }
}
