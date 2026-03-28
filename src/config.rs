use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read {path}: {source}")]
    Io { path: String, source: std::io::Error },
    #[error("failed to parse pyproject.toml: {0}")]
    Parse(#[from] toml::de::Error),
}

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub select: Vec<String>,
    #[serde(default)]
    pub ignore: Vec<String>,
    #[serde(default)]
    pub rules: HashMap<String, toml::Value>,
    #[serde(default)]
    pub plugins: Vec<PluginConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PluginConfig {
    pub name: String,
    pub executable: String,
    #[serde(default)]
    pub config: toml::Value,
}

#[derive(Debug, Deserialize)]
struct PyProjectToml {
    tool: Option<PyProjectTool>,
}

#[derive(Debug, Deserialize)]
struct PyProjectTool {
    ruffian: Option<Config>,
}

pub fn load(dir: &Path) -> Result<Config, ConfigError> {
    let path = dir.join("pyproject.toml");
    if !path.exists() {
        return Ok(Config::default());
    }
    let raw = std::fs::read_to_string(&path).map_err(|source| ConfigError::Io {
        path: path.display().to_string(),
        source,
    })?;
    let doc: PyProjectToml = toml::from_str(&raw)?;
    Ok(doc.tool.and_then(|t| t.ruffian).unwrap_or_default())
}
