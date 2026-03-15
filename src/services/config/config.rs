use color_eyre::eyre::Error;
use config::{Config as ConfigLoader, File as ConfigFile, FileFormat};
use directories::ProjectDirs;
use std::{
    fs::{self},
    path::PathBuf,
    str,
};

use crate::services::config::config_data::Config;

#[derive(Debug)]
pub struct ConfigService;

impl ConfigService {
    fn config_path() -> Result<PathBuf, Error> {
        let proj_dirs = ProjectDirs::from("", "", "tubgerm")
            .ok_or_else(|| color_eyre::eyre::eyre!("Could not determine config directory"))?;

        let dir = proj_dirs.config_dir();
        fs::create_dir_all(dir)?;

        let mut path = dir.to_path_buf();
        path.push("config.toml");

        Ok(path)
    }

    pub fn load() -> Result<Config, Error> {
        let path = Self::config_path()?;

        if !path.exists() {
            return Ok(Config::default());
        }

        let settings = ConfigLoader::builder()
            .add_source(ConfigFile::from(path).format(FileFormat::Toml))
            .build()?;

        Ok(settings.try_deserialize()?)
    }

    pub fn save(cfg: &Config) -> Result<(), Error> {
        let path = Self::config_path()?;
        let toml = toml::to_string_pretty(cfg)?;
        fs::write(path, toml)?;
        Ok(())
    }

    // INFO: ADD set_<CONFIG>() for every field

    pub fn set_username(key: &str) -> Result<(), Error> {
        let mut cfg = Self::load()?;
        cfg.credentials.username = key.to_string();
        Self::save(&cfg)?;
        Ok(())
    }

    pub fn set_server(key: &str) -> Result<(), Error> {
        let mut cfg = Self::load()?;
        cfg.credentials.server = key.to_string();
        Self::save(&cfg)?;
        Ok(())
    }
}
