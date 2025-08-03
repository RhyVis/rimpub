use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::{OnceLock, RwLock},
};

use anyhow::{Result, anyhow};
use clap::{Args, Subcommand};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};

use crate::util::get_dir_store;

#[derive(Debug, Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommand,
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    /// Get a configuration value
    Get(ConfigGetArgs),
    /// Set a configuration value
    Set(ConfigSetArgs),
    /// Checks if the current config is valid
    Check,
}

#[derive(Debug, Args)]
pub struct ConfigGetArgs {
    /// The key of the configuration to get,
    /// if not provided then all config will be printed
    pub key: Option<String>,
}

#[derive(Debug, Args)]
pub struct ConfigSetArgs {
    /// The key of the configuration to set
    pub key: String,
    /// The value to set for the configuration key
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub path_mods: Option<PathBuf>,

    #[serde(default)]
    pub no_ask: bool,
}

static CONFIG: OnceLock<RwLock<Config>> = OnceLock::new();
const CONFIG_FILE_NAME: &str = "Config.toml";
const FIELD_PATH_MODS: &str = "path_mods";
const FIELD_NO_ASK: &str = "no_ask";

impl Config {
    pub fn init() -> Result<()> {
        CONFIG
            .set(RwLock::new(
                Self::load().map_err(|e| anyhow!("Failed to load config file: {e}"))?,
            ))
            .expect("But why?");
        Ok(())
    }

    pub fn get(key: &str) -> Option<String> {
        let config = CONFIG.get().expect("Config has not initialized");
        let config = config.read().expect("Config not readable");
        match key.to_lowercase().as_str() {
            FIELD_PATH_MODS => config
                .path_mods
                .clone()
                .map(|p| p.to_string_lossy().to_string())
                .or_else(|| {
                    warn!("'path_mods' not set");
                    None
                }),
            FIELD_NO_ASK => Some(config.no_ask.to_string()),
            _ => {
                warn!("Unexpected key {key} provided");
                None
            },
        }
    }

    pub fn get_obj() -> Self {
        let config = CONFIG.get().expect("Config has not initialized");
        let config = config.read().expect("Config not readable");
        config.clone()
    }

    pub fn set(key: &str, value: &str) -> Result<()> {
        match key.to_lowercase().as_str() {
            FIELD_PATH_MODS => Self::write(|c| {
                let value = PathBuf::from(value.trim());
                c.path_mods = Some(value);
                info!(
                    "Set 'path_mods' to {}",
                    c.path_mods.as_ref().unwrap().display()
                );
                Ok(())
            }),
            FIELD_NO_ASK => Self::write(|c| {
                c.no_ask = value.parse().map_err(|_| {
                    anyhow!(
                        "Invalid value for 'no_ask': expected a boolean, got '{}'",
                        value
                    )
                })?;
                info!("Set 'no_ask' to {}", c.no_ask);
                Ok(())
            }),
            _ => {
                error!("Unexpected key {key} provided");
                Err(anyhow!("Unexpected key {key} provided"))
            },
        }
    }

    fn load() -> Result<Self> {
        let config_file = get_dir_store().join(CONFIG_FILE_NAME);
        Ok(match fs::read_to_string(&config_file) {
            Ok(content) => toml::from_str::<Config>(&content)?,
            Err(error) => match error.kind() {
                ErrorKind::NotFound => {
                    let default_config = Config::default();
                    fs::write(config_file, toml::to_string_pretty(&default_config)?)?;
                    default_config
                },
                _ => return Err(error.into()),
            },
        })
    }

    fn save(&self) -> Result<()> {
        fs::write(
            get_dir_store().join("Config.toml"),
            toml::to_string_pretty(self)?,
        )?;
        Ok(())
    }

    fn write(action: impl FnOnce(&mut Config) -> Result<()>) -> Result<()> {
        let mut config = CONFIG
            .get()
            .expect("Config has not initialized")
            .write()
            .expect("Config not writable");
        action(&mut config)?;
        config.save()
    }
}

impl ConfigArgs {
    pub fn run(&self) -> Result<()> {
        match self.command {
            ConfigCommand::Get(ref args) => {
                let key = &args.key;
                if let Some(key) = key {
                    let value = Config::get(key);
                    if let Some(value) = value {
                        info!("'{}' = {}", key, value);
                    }
                } else {
                    info!("Config object: {:?}", Config::get_obj());
                }
            },
            ConfigCommand::Set(ref args) => {
                let key = &args.key;
                let value = &args.value;
                Config::set(key, value)?;
            },
            ConfigCommand::Check => {
                let mut any_err = false;
                let config = Config::get_obj();
                let path_mods = config.path_mods;
                if let Some(path_mods) = path_mods {
                    if !fs::exists(Path::new(&path_mods)).map_err(|e| {
                        anyhow!(
                            "Failed to read configured 'path_mods': {} - {}",
                            path_mods.display(),
                            e
                        )
                    })? {
                        warn!("'path_mods': {} does not exist", path_mods.display());
                        any_err = true;
                    }
                } else {
                    warn!("'path_mods' not configured");
                    any_err = true;
                }
                if !any_err {
                    info!("Config ready")
                } else {
                    warn!("Config check failed")
                }
            },
        }
        Ok(())
    }
}
