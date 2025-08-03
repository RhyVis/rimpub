mod cli;
mod logger;
mod util;

use anyhow::Result;
use clap::Parser;
use cli::*;

fn main() -> Result<()> {
    logger::init().expect("Failed to setup logger");
    Config::init()?;
    Cli::parse().run()
}
