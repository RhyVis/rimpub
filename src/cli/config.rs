use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    sync::{OnceLock, RwLock},
};

use anyhow::{Result, anyhow};
use clap::{Args, Subcommand};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};

use crate::util::{get_dir, read_steam_install_path};

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

const PATH_SEG_RIMWORLD: &str = "steamapps/common/RimWorld";
const PATH_SEG_MODS: &str = "Mods";

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

    pub fn get_clone() -> Self {
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
        let dir = get_dir();
        if !dir.exists() {
            // First run init process
            std::fs::create_dir_all(&dir)
                .map_err(|e| anyhow!("Failed to create config directory: {}", e))?;
            return Self::default_make();
        }

        let path_config_file = get_dir().join(CONFIG_FILE_NAME);
        Ok(match fs::read_to_string(&path_config_file) {
            Ok(content) => toml::from_str::<Config>(&content)?,
            Err(error) => match error.kind() {
                ErrorKind::NotFound => {
                    warn!(
                        "Application directory exists but config file not found, recreating default config"
                    );
                    Self::default_make()?
                },
                _ => return Err(error.into()),
            },
        })
    }

    fn default_make() -> Result<Self> {
        info!("Creating default config file");
        let mut default = Self::default();
        default.path_mods = read_steam_install_path()
            .unwrap_or_else(|_| {
                warn!("Failed to read Steam install path, 'path_mods' will not be set");
                None
            })
            .and_then(|path| {
                path.join(PATH_SEG_RIMWORLD)
                    .join(PATH_SEG_MODS)
                    .canonicalize()
                    .inspect_err(|e| warn!("Failed to canonicalize 'path_mods': {}", e))
                    .ok()
            });
        if let Some(path) = &default.path_mods {
            debug!("Default 'path_mods' set to {}", path.display());
        }
        default.save(true)?;
        Ok(default)
    }

    fn save(&self, mark_generated: bool) -> Result<()> {
        fs::write(
            get_dir().join(CONFIG_FILE_NAME),
            if mark_generated {
                format!(
                    "# This file was generated by rimpub, do not edit manually\n\n{}",
                    toml::to_string_pretty(self)?
                )
            } else {
                toml::to_string_pretty(self)?
            },
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
        config.save(true)
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
                    info!("Config object: {:?}", Config::get_clone());
                }
            },
            ConfigCommand::Set(ref args) => {
                let key = &args.key;
                let value = &args.value;
                Config::set(key, value)?;
            },
            ConfigCommand::Check => {
                let mut any_err = false;
                let config = Config::get_clone();
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
