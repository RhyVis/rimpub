use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Result, anyhow};
use clap::Args;
use ignore::{DirEntry, WalkBuilder};
use log::{debug, info, warn};

use crate::{
    cli::{Config, PROJECT_CONFIG_FILE_NAME, ProjectConf},
    util::{confirm, decode_out},
};

#[derive(Debug, Args)]
pub struct PublishArgs {
    /// Alternate target_dir used to copy files
    #[arg(long)]
    pub target_dir: Option<String>,
}

pub const PUBLISH_IGNORE_FILE_NAME: &str = ".rimpub.ignore";

impl PublishArgs {
    pub fn run(&self) -> Result<()> {
        let config_global = Config::get_clone();
        let working_directory = std::env::current_dir()?;
        info!("Working directory: {}", working_directory.display());

        if let Some(sln) = find_sln_file(&working_directory)? {
            info!("Found solution file, executing build: {}", sln.display());
            execute_dotnet_build(&sln)?;
        }

        let (mut config, _) = ProjectConf::load_current()?;
        config.resolve_name();
        debug!("Loaded project configuration: {:?}", config);

        info!("Working project: {}", config.name);

        let target_base = self
            .target_dir
            .as_ref()
            .map(PathBuf::from)
            .or_else(|| config_global.get_path_mods().ok().flatten())
            .ok_or_else(|| anyhow!("Cannot determine target directory from config or args"))?;
        let target_path = target_base.join(&config.name);

        info!("Target directory: {}", target_path.display());

        if target_path.exists() {
            if !config_global.no_ask {
                if !confirm(&format!(
                    "Target directory '{}' already exists. Do you want to delete it and continue? (y/N): ",
                    target_path.display()
                )) {
                    info!("Operation cancelled by user");
                    return Ok(());
                }
            }

            info!(
                "Clearing existing target directory: {}",
                target_path.display()
            );
            fs::remove_dir_all(&target_path).map_err(|err| {
                let msg = format!("Failed to remove existing target directory: {}", err);
                warn!("{}", msg);
                anyhow!("{}", msg)
            })?;
        }

        fs::create_dir_all(&target_path).map_err(|e| {
            let msg = format!("Failed to create target directory: {}", e);
            warn!("{}", msg);
            anyhow!("{}", msg)
        })?;

        let mut builder = WalkBuilder::new(&working_directory);
        builder
            .git_ignore(true)
            .git_exclude(true)
            .git_global(true)
            .add_custom_ignore_filename(PUBLISH_IGNORE_FILE_NAME)
            .filter_entry(|entry| {
                let path = entry.path();
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                if name == ".gitignore"
                    || name == ".git"
                    || name == PUBLISH_IGNORE_FILE_NAME
                    || name == PROJECT_CONFIG_FILE_NAME
                {
                    false
                } else {
                    true
                }
            });

        let walker = builder.build();
        let mut any_err = false;
        for result in walker {
            match result {
                Ok(entry) => {
                    if let Err(e) = copy_entry(&entry, &working_directory, &target_path) {
                        warn!("Failed to copy {}: {}", entry.path().display(), e);
                        any_err = true;
                    }
                },
                Err(e) => {
                    warn!("Error reading file: {}", e);
                    any_err = true;
                },
            }
        }

        if any_err {
            warn!("Error encountered during processing.")
        } else {
            info!("Successfully processed {}", config.name)
        }

        Ok(())
    }
}

fn copy_entry(entry: &DirEntry, source_root: &Path, target_root: &Path) -> Result<()> {
    let source_path = entry.path();

    if source_path == source_root {
        return Ok(());
    }

    let relative_path = source_path.strip_prefix(source_root)?;
    let target_path = target_root.join(relative_path);

    if entry.file_type().map_or(false, |ft| ft.is_dir()) {
        if !target_path.exists() {
            fs::create_dir_all(&target_path)?;
            debug!("Created directory: {}", relative_path.display());
        }
    } else if entry.file_type().map_or(false, |ft| ft.is_file()) {
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(source_path, &target_path)?;
        debug!("Copied file: {}", relative_path.display());
    }

    Ok(())
}

fn find_sln_file(working_directory: &Path) -> Result<Option<PathBuf>> {
    Ok(fs::read_dir(working_directory)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.is_file() && path.extension().map_or(false, |ext| ext == "sln")))
}

fn execute_dotnet_build(sln_file: &Path) -> Result<()> {
    let output = Command::new("dotnet")
        .arg("build")
        .arg(sln_file)
        .arg("--configuration")
        .arg("Release")
        .current_dir(sln_file.parent().unwrap_or(sln_file))
        .output()
        .map_err(|e| anyhow!("Failed to execute dotnet build:\n{}", e))?;

    if output.status.success() {
        info!("Project build completed successfully");
        debug!("Build output:\n{}", decode_out(&output.stdout));
    } else {
        let stderr = decode_out(&output.stderr);
        warn!("Project build failed: {}", stderr);
        return Err(anyhow!("Project build failed:\n{}", stderr));
    }

    Ok(())
}
