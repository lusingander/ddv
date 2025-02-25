use serde::Deserialize;
use smart_default::SmartDefault;
use umbra::optional;

impl Config {
    pub fn load() -> Config {
        Config::default()
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
}

#[optional(derives = [Deserialize])]
#[derive(Debug, Clone, SmartDefault)]
pub struct UiTableListConfig {
    #[default = 30]
    pub list_width: u16,
}
