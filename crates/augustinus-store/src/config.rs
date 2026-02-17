use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppConfig {
    pub language: Language,
    pub shell: String,
    pub git_repo: Option<String>,
}

impl AppConfig {
    pub fn to_toml_string(&self) -> String {
        toml::to_string_pretty(self).expect("AppConfig serializes")
    }

    pub fn from_toml_str(input: &str) -> Result<Self> {
        let parsed: Self = toml::from_str(input).context("parse config toml")?;
        Ok(parsed)
    }

    pub fn load_or_none() -> Result<Option<Self>> {
        let path = config_path()?;
        if !path.exists() {
            return Ok(None);
        }
        let contents = fs::read_to_string(&path)
            .with_context(|| format!("read config at {}", path.display()))?;
        let parsed = Self::from_toml_str(&contents)?;
        Ok(Some(parsed))
    }

    pub fn save(&self) -> Result<PathBuf> {
        let path = config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create config dir {}", parent.display()))?;
        }
        fs::write(&path, self.to_toml_string())
            .with_context(|| format!("write config at {}", path.display()))?;
        Ok(path)
    }

    pub fn path() -> Result<PathBuf> {
        config_path()
    }
}

fn config_path() -> Result<PathBuf> {
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        return Ok(Path::new(&xdg).join("augustinus").join("config.toml"));
    }

    let home = std::env::var_os("HOME").context("HOME not set")?;
    Ok(Path::new(&home)
        .join(".config")
        .join("augustinus")
        .join("config.toml"))
}

pub use augustinus_i18n::Language;
