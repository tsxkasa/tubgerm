use color_eyre::eyre::Result;
use config::{Config as ConfigLoader, File as ConfigFile, FileFormat};
use directories::ProjectDirs;
use std::{
    fs::{self},
    path::PathBuf,
    str,
};

use crate::services::config::data::Config;

#[derive(Debug)]
pub struct ConfigService {
    config: Config,
    path: PathBuf,
}

impl ConfigService {
    fn config_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("", "", "tubgerm")
            .ok_or_else(|| color_eyre::eyre::eyre!("Could not determine config directory"))?;

        let dir = proj_dirs.config_dir();
        fs::create_dir_all(dir)?;

        let mut path = dir.to_path_buf();
        path.push("config.toml");

        Ok(path)
    }

    pub fn new() -> Result<Self> {
        let path = Self::config_path()?;
        let config = if path.exists() {
            let settings = ConfigLoader::builder()
                .add_source(ConfigFile::from(path.clone()).format(FileFormat::Toml))
                .build()?;
            settings.try_deserialize()?
        } else {
            Config::default()
        };

        Ok(Self { config, path })
    }

    pub fn load() -> Result<Config> {
        let path = Self::config_path()?;

        if !path.exists() {
            return Ok(Config::default());
        }

        let settings = ConfigLoader::builder()
            .add_source(ConfigFile::from(path).format(FileFormat::Toml))
            .build()?;

        Ok(settings.try_deserialize()?)
    }

    pub fn save(&self) -> Result<()> {
        let toml = toml::to_string_pretty(&self.config)?;
        fs::write(&self.path, toml)?;
        Ok(())
    }

    // INFO: ADD set_<CONFIG>() for every field
    pub fn set_credentials(&mut self, server: &str, username: &str) -> Result<()> {
        self.config.credentials.server = server.to_string();
        self.config.credentials.username = username.to_string();
        Ok(())
    }
}
