use anyhow::{Result, anyhow};
use clap::{Parser, Subcommand};
use log::warn;

pub use self::{config::*, generate::*, publish::*};

mod config;
mod generate;
mod publish;

#[derive(Debug, Parser)]
#[command(name = "rimpub", version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Increase logging verbosity (-v for debug, -vv for trace)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
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
    /// Generate files for the mod.
    #[command(aliases = ["gen", "g"])]
    Generate(GenerateArgs),
}

impl Cli {
    pub fn run(&self) -> Result<()> {
        let log_level = match self.verbose {
            0 => log::LevelFilter::Info,
            1 => log::LevelFilter::Debug,
            2 => log::LevelFilter::Trace,
            _ => {
                println!("Man don't be too crazy");
                log::LevelFilter::Trace
            },
        };
        log::set_max_level(log_level);

        match self.command {
            Some(ref command) => match command {
                Command::Config(args) => args.run(),
                Command::Publish(args) => args.run(),
                Command::Generate(args) => args.run(),
            },
            None => {
                warn!("Choose an option, referring to '--help' for more info");
                Ok(())
            },
        }
        .map_err(|e| anyhow!("Unexpected error during exec: {e}"))
    }
}
