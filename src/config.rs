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
            Ok(content) => serde_json::from_str(&content).unwrap(),
            Err(err) => {
                warn!("Failed to load global config {:?}: {}", path, err);
                Config {
                    power_supplies: Vec::new(),
                }
            }
        };

        Self { path, data }
    }

    pub fn name(&self) -> &str {
        self.path.file_stem().unwrap().to_str().unwrap()
    }

    pub fn rename(&mut self, new_name: &str) {
        let new_path = self
            .path
            .clone()
            .with_file_name(new_name)
            .with_extension("json");
        std::fs::rename(&self.path, &new_path).unwrap();
        self.path = new_path;
    }

    pub fn power_supply(&mut self, id: &str) -> &mut PowerSupplyConfig {
        self.data
            .power_supplies
            .iter_mut()
            .find(|config| config.id == id)
            .unwrap()
    }

    pub fn power_supply_channel(&mut self, id: &str, ch: u8) -> &mut ChannelConfig {
        &mut self.power_supply(id).channels[ch as usize]
    }

    pub fn save(&mut self) {
        std::fs::write(
            &self.path,
            serde_json::to_string_pretty(&self.data).unwrap(),
        )
        .unwrap();
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct MultiOn {
    pub enabled: bool,
    pub delay_ms: u32,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct ChannelConfig {
    pub name: String,
    pub voltage: f32,
    pub current: f32,
    #[serde(default)]
    pub multi_on: MultiOn,
    #[serde(default = "one")]
    pub vrange: u8,
    #[serde(default = "def_true")]
    pub auto_vrange: bool,
    pub overvoltage_trip: Option<f32>,
    pub overcurrent_trip: Option<f32>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PowerSupplyConfig {
    pub vid: u16,
    pub pid: u16,
    pub serial_number: Option<String>,
    pub id: String,
    pub name: String,
    pub channels: Vec<ChannelConfig>,
    #[serde(default = "zero")]
    pub voltage_tracking: u8,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    #[serde(default)]
    pub power_supplies: Vec<PowerSupplyConfig>,
}

fn one() -> u8 {
    1
}

fn zero() -> u8 {
    0
}

fn def_true() -> bool {
    true
}

impl Default for MultiOn {
    fn default() -> Self {
        Self {
            enabled: true,
            delay_ms: 0,
        }
    }
}
