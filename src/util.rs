use std::{
    io::{Write, stdin, stdout},
    path::PathBuf,
};

use anyhow::{Result, anyhow};
use log::{debug, warn};

pub fn get_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Failed to get home directory")
        .join(".rimpub")
}

pub fn read_steam_install_path() -> Result<Option<PathBuf>> {
    #[cfg(target_os = "windows")]
    {
        use winreg::{RegKey, enums::HKEY_LOCAL_MACHINE};

        const STEAM_REG_PATH: &str = r"SOFTWARE\WOW6432Node\Valve\Steam";
        const STEAM_REG_PATH_32: &str = r"SOFTWARE\Valve\Steam";

        const KEY_INSTALL_DIR: &str = "InstallPath";

        debug!("Reading Steam install path from registry");

        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let steam_key = hklm
            .open_subkey(STEAM_REG_PATH)
            .or_else(|_| hklm.open_subkey(STEAM_REG_PATH_32))
            .map_err(|e| anyhow!("Failed to open Steam registry key: {}", e))?;

        let install_path: String = steam_key
            .get_value(KEY_INSTALL_DIR)
            .map_err(|e| anyhow!("Failed to read 'InstallPath' from Steam registry: {}", e))?;
        let path = PathBuf::from(install_path);

        if !path.exists() || !path.is_dir() {
            warn!(
                "Steam install path does not exist or is not a directory: {}",
                path.display()
            );
            return Ok(None);
        }

        Ok(Some(path))
    }

    #[cfg(not(target_os = "windows"))]
    {
        debug!("Reading Steam install path is not supported on this OS");
        Ok(None)
    }
}

pub fn decode_out(bytes: &[u8]) -> String {
    #[cfg(target_os = "windows")]
    {
        use encoding_rs::GBK;

        let (decoded, _, had_errors) = GBK.decode(bytes);
        if had_errors {
            log::warn!("Failed to decode bytes using GBK");
        }
        decoded.into_owned()
    }

    #[cfg(not(target_os = "windows"))]
    {
        String::from_utf8_lossy(bytes).into_owned()
    }
}

pub fn confirm(msg: &str) -> bool {
    print!("{} (y/n): ", msg);
    stdout().flush().expect("Failed to flush stdout");

    let mut input = String::new();
    stdin().read_line(&mut input).expect("Failed to read line");

    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_read_steam_install_path() {
        let path = read_steam_install_path();
        assert!(path.is_ok(), "Should be able to read Steam install path");
        dbg!("Steam install path: {:?}", path.unwrap());
    }
}
