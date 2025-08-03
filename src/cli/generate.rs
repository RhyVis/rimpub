use std::{fs, path::Path};

use anyhow::Result;
use clap::{Args, Subcommand};
use log::{info, warn};

use super::{PUBLISH_CONFIG_FILE_NAME, PUBLISH_IGNORE_FILE_NAME, PublishConf};

#[derive(Debug, Args)]
pub struct GenerateArgs {
    #[command(subcommand)]
    pub command: Option<GenerateCommand>,
}

#[derive(Debug, Subcommand)]
pub enum GenerateCommand {
    /// Generate a configuration file for the mod.
    ConfigFile,
    /// Generate an ignore file for the mod.
    IgnoreFile,
}

impl GenerateCommand {
    fn run(&self) -> Result<()> {
        match self {
            GenerateCommand::ConfigFile => {
                info!("Generating configuration file...");
                gen_config_file(&std::env::current_dir()?)?;
            },
            GenerateCommand::IgnoreFile => {
                info!("Generating ignore file...");
                gen_ignore_file(&std::env::current_dir()?)?;
            },
        }
        Ok(())
    }
}

fn gen_config_file(working_dir: &Path) -> Result<()> {
    let config_path = working_dir.join(PUBLISH_CONFIG_FILE_NAME);
    if config_path.exists() {
        warn!(
            "Configuration file already exists at {}",
            config_path.display()
        );
        return Ok(());
    }
    fs::write(
        config_path,
        toml::to_string_pretty(&PublishConf::default())?,
    )?;
    Ok(())
}

fn gen_ignore_file(working_dir: &Path) -> Result<()> {
    let ignore_path = working_dir.join(PUBLISH_IGNORE_FILE_NAME);
    if ignore_path.exists() {
        warn!("Ignore file already exists at {}", ignore_path.display());
        return Ok(());
    }
    fs::write(ignore_path, "# Add files or directories to ignore here\n")?;
    Ok(())
}

impl GenerateArgs {
    pub fn run(&self) -> Result<()> {
        match self.command {
            Some(ref command) => command.run(),
            None => {
                let working_dir = std::env::current_dir()?;
                gen_config_file(&working_dir)?;
                gen_ignore_file(&working_dir)?;
                Ok(())
            },
        }
    }
}
