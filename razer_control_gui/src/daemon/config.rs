use serde::{Deserialize, Serialize};
use std::{fs, fs::File, io, env};
use std::io::prelude::*;

use crate::comms::{CurvePoint, FanMode};

const SETTINGS_FILE: &str = "/.local/share/razercontrol/daemon.json";
const EFFECTS_FILE: &str = "/.local/share/razercontrol/effects.json";

pub fn default_cpu_curve() -> Vec<CurvePoint> {
    vec![
        CurvePoint { temp_c: 40, rpm: 3500 },
        CurvePoint { temp_c: 55, rpm: 3800 },
        CurvePoint { temp_c: 70, rpm: 4400 },
        CurvePoint { temp_c: 85, rpm: 5000 },
    ]
}

pub fn default_gpu_curve() -> Vec<CurvePoint> {
    vec![
        CurvePoint { temp_c: 45, rpm: 3500 },
        CurvePoint { temp_c: 60, rpm: 3800 },
        CurvePoint { temp_c: 75, rpm: 4400 },
        CurvePoint { temp_c: 85, rpm: 5000 },
    ]
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct FanCurve {
    #[serde(default)]
    pub cpu: Vec<CurvePoint>,
    #[serde(default)]
    pub gpu: Vec<CurvePoint>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PowerConfig {
    pub power_mode: u8,
    pub cpu_boost: u8,
    pub gpu_boost: u8,
    pub fan_rpm: i32,
    pub brightness: u8,
    pub logo_state: u8,
    pub screensaver: bool, // turno of keyboard light if screen is blank
    pub idle: u32,
    #[serde(default)]
    pub fan_mode: FanMode,
    #[serde(default)]
    pub fan_curve: FanCurve,
}

impl PowerConfig {
    pub fn new() -> PowerConfig {
        return PowerConfig{
            power_mode: 0,
            cpu_boost: 1,
            gpu_boost: 0,
            fan_rpm: 0,
            brightness: 128,
            logo_state: 0,
            screensaver: false,
            idle: 0,
            fan_mode: FanMode::Curve,
            fan_curve: FanCurve {
                cpu: default_cpu_curve(),
                gpu: default_gpu_curve(),
            },
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Configuration {
    pub power: [PowerConfig; 2],
    pub sync: bool, // sync light settings between ac and battery
    pub no_light: f64, // no light bellow this percentage of battery
    pub standard_effect: u8,
    pub standard_effect_params: Vec<u8>,
}

impl Configuration {
    pub fn new() -> Configuration {
        return Configuration {
            power: [PowerConfig::new(), PowerConfig::new()],
            sync: false,
            no_light: 0.0,
            standard_effect: 0, // off
            standard_effect_params: vec![]
        };
    }

    pub fn write_to_file(&mut self) -> io::Result<()> {
        let j: String = serde_json::to_string_pretty(&self)?;
        File::create(get_home_directory() + SETTINGS_FILE)?.write_all(j.as_bytes())?;
        Ok(())
    }

    pub fn read_from_config() -> io::Result<Configuration> {
        let str = fs::read_to_string(get_home_directory() + SETTINGS_FILE)?;
        let mut res: Configuration = serde_json::from_str(str.as_str())?;
        for slot in res.power.iter_mut() {
            if slot.fan_curve.cpu.is_empty() {
                slot.fan_curve.cpu = default_cpu_curve();
            }
            if slot.fan_curve.gpu.is_empty() {
                slot.fan_curve.gpu = default_gpu_curve();
            }
        }
        Ok(res)
    }

    pub fn write_effects_save(json: serde_json::Value) -> io::Result<()> {
        let j: String = serde_json::to_string_pretty(&json)?;
        File::create(get_home_directory() + EFFECTS_FILE)?.write_all(j.as_bytes())?;
        Ok(())
    }

    pub fn read_effects_file() -> io::Result<serde_json::Value> {
        let str = fs::read_to_string(get_home_directory() + EFFECTS_FILE)?;
        let res: serde_json::Value = serde_json::from_str(str.as_str())?;
        Ok(res)
    }
}

fn get_home_directory() -> String {
    env::var("HOME").expect("The \"HOME\" environment variable must be set to a valid directory")
}
