use crate::resources::GameConfig;
use std::fs;
use std::path::PathBuf;

pub fn get_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|mut path| {
        path.push("minion");
        fs::create_dir_all(&path).ok()?;
        path.push("config.toml");
        Some(path)
    }).flatten()
}

pub fn load_config() -> GameConfig {
    if let Some(config_path) = get_config_path() {
        if let Ok(contents) = fs::read_to_string(&config_path) {
            if let Ok(config) = toml::from_str::<GameConfig>(&contents) {
                return config;
            }
        }
    }
    GameConfig::default()
}

pub fn save_config(config: &GameConfig) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(config_path) = get_config_path() {
        let contents = toml::to_string_pretty(config)?;
        fs::write(config_path, contents)?;
    }
    Ok(())
}