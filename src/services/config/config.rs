use color_eyre::eyre::Result;
use config::{Config as ConfigLoader, File as ConfigFile, FileFormat};
use directories::ProjectDirs;
use std::{
    fs::{self},
    path::PathBuf,
    str,
};

use crate::services::config::config_data::Config;

#[derive(Debug)]
pub struct ConfigService {
    pub config: Option<Config>,
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

    pub fn save(cfg: &Config) -> Result<()> {
        let path = Self::config_path()?;
        let toml = toml::to_string_pretty(cfg)?;
        fs::write(path, toml)?;
        Ok(())
    }

    // INFO: ADD set_<CONFIG>() for every field
    pub fn set_credentials(server: &str, username: &str) -> Result<()> {
        let mut cfg = Self::load()?;
        cfg.credentials.server = server.to_string();
        cfg.credentials.username = username.to_string();
        Self::save(&cfg)
    }
}
