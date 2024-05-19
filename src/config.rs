use std::path::PathBuf;

use log::warn;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug, Clone)]
pub struct AppConfig {
    path: PathBuf,
    pub data: Config,
}

impl AppConfig {
    pub fn load_from_file(path: PathBuf) -> Self {
        let data = match std::fs::read_to_string(&path) {
            Ok(content) => toml::from_str(&content).unwrap(),
            Err(err) => {
                warn!("Failed to load global config {:?}: {}", path, err);
                Config {
                    power_supply: Vec::new(),
                }
            }
        };

        Self { path, data }
    }

    pub fn power_supply(&mut self, id: &str) -> &mut PowerSupplyConfig {
        self.data
            .power_supply
            .iter_mut()
            .find(|config| config.id == id)
            .unwrap()
    }

    pub fn power_supply_channel(&mut self, id: &str, ch: u8) -> &mut ChannelConfig {
        &mut self.power_supply(id).channels[ch as usize]
    }

    pub fn save(&mut self) {
        std::fs::write(&self.path, toml::to_string(&self.data).unwrap()).unwrap();
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct ChannelConfig {
    pub name: String,
    pub voltage: f32,
    pub current: f32,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PowerSupplyConfig {
    pub vid: u16,
    pub pid: u16,
    pub serial_number: Option<String>,
    pub id: String,
    pub name: String,
    pub channels: Vec<ChannelConfig>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    pub power_supply: Vec<PowerSupplyConfig>,
}
