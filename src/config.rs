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
}
