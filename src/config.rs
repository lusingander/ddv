use std::env;

use serde::Deserialize;
use smart_default::SmartDefault;
use umbra::optional;

const CONFIG_PATH_ENV_VAR: &str = "DDV_CONFIG";

impl Config {
    pub fn load() -> Config {
        match env::var(CONFIG_PATH_ENV_VAR) {
            Ok(path) => {
                let content = std::fs::read_to_string(path).unwrap();
                let config: OptionalConfig = toml::from_str(&content).unwrap();
                config.into()
            }
            Err(_) => Config::default(),
        }
    }
}

#[optional(derives = [Deserialize])]
#[derive(Debug, Clone, SmartDefault)]
pub struct Config {
    #[nested]
    pub ui: UiConfig,
}

#[optional(derives = [Deserialize])]
#[derive(Debug, Clone, SmartDefault)]
pub struct UiConfig {
    #[nested]
    pub table_list: UiTableListConfig,
    #[nested]
    pub table: UiTableConfig,
}

#[optional(derives = [Deserialize])]
#[derive(Debug, Clone, SmartDefault)]
pub struct UiTableListConfig {
    #[default = 30]
    pub list_width: u16,
}

#[optional(derives = [Deserialize])]
#[derive(Debug, Clone, SmartDefault)]
pub struct UiTableConfig {
    #[default = 30]
    pub max_attribute_width: usize,
    #[default = 35]
    pub max_expand_width: u16,
    #[default = 6]
    pub max_expand_height: u16,
}
