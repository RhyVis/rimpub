use std::{fs, path::Path};

use anyhow::Result;
use clap::{Args, Subcommand};
use log::{debug, info, warn};

use super::{PROJECT_CONFIG_FILE_NAME, PUBLISH_IGNORE_FILE_NAME, ProjectConf};

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
        let working_dir = std::env::current_dir()?;
        match self {
            GenerateCommand::ConfigFile => {
                info!("Generating configuration file...");
                gen_config_file(&working_dir)?;
            },
            GenerateCommand::IgnoreFile => {
                info!("Generating ignore file...");
                gen_ignore_file(&working_dir)?;
            },
        }
        Ok(())
    }
}

fn gen_config_file(working_dir: &Path) -> Result<()> {
    let config_path = working_dir.join(PROJECT_CONFIG_FILE_NAME);
    if config_path.exists() {
        warn!(
            "Configuration file already exists at {}",
            config_path.display()
        );
        return Ok(());
    }
    debug!("Generating configuration file at {}", config_path.display());
    fs::write(
        config_path,
        toml::to_string_pretty(&ProjectConf::default())?,
    )?;
    Ok(())
}

fn gen_ignore_file(working_dir: &Path) -> Result<()> {
    let ignore_path = working_dir.join(PUBLISH_IGNORE_FILE_NAME);
    if ignore_path.exists() {
        warn!("Ignore file already exists at {}", ignore_path.display());
        return Ok(());
    }
    debug!("Generating ignore file at {}", ignore_path.display());
    fs::write(ignore_path, "# Add files or directories to ignore here\n")?;
    Ok(())
}

impl GenerateArgs {
    pub fn run(&self) -> Result<()> {
        match self.command {
            Some(ref command) => command.run(),
            None => {
                info!("No subcommand provided, generating both config and ignore files");
                let working_dir = std::env::current_dir()?;
                gen_config_file(&working_dir)?;
                gen_ignore_file(&working_dir)?;
                Ok(())
            },
        }
    }
}
