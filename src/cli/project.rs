use std::fs;

use anyhow::{Result, anyhow};
use log::{debug, warn};
use serde::{Deserialize, Serialize};

pub const PROJECT_CONFIG_FILE_NAME: &str = ".rimpub.toml";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectConf {
    #[serde(default)]
    pub name: String,
}

impl ProjectConf {
    pub(super) fn resolve_name(&mut self) {
        if self.name.is_empty() {
            debug!("No 'name' provided in configuration, using folder name instead");
            if let Some(dir_name) = std::env::current_dir()
                .ok()
                .and_then(|path| path.file_name().map(|s| s.to_string_lossy().to_string()))
            {
                self.name = dir_name;
            } else {
                warn!("Failed to resolve project name, using default empty name");
            }
        }
    }

    pub(super) fn load_current() -> Result<(Self, bool)> {
        let working_dir = std::env::current_dir()?;
        let config_path = working_dir.join(PROJECT_CONFIG_FILE_NAME);

        Ok(if config_path.exists() {
            debug!("Reading config file: {}", config_path.display());
            let config_contents = fs::read_to_string(config_path)?;
            (
                toml::de::from_str(&config_contents).map_err(|e| {
                    let msg = format!("Failed to parse {}: {}", PROJECT_CONFIG_FILE_NAME, e);
                    warn!("{}", msg);
                    anyhow!("{}", msg)
                })?,
                true,
            )
        } else {
            debug!("No config file found, using default configuration");
            (ProjectConf::default(), false)
        })
    }
}
