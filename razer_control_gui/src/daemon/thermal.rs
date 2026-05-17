use std::fs;
use std::path::PathBuf;

use log::{info, warn};
use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
use nvml_wrapper::Nvml;

use crate::comms::CurvePoint;

const HWMON_ROOT: &str = "/sys/class/hwmon";

const CPU_HWMON_NAMES: &[&str] = &["k10temp", "zenpower", "coretemp", "k8temp"];
const GPU_HWMON_NAMES: &[&str] = &["amdgpu", "i915", "xe", "nouveau"];

pub struct ThermalMonitor {
    cpu_hwmon: Option<PathBuf>,
    gpu_hwmon: Option<PathBuf>,
    nvml: Option<Nvml>,
}

impl ThermalMonitor {
    pub fn probe() -> Self {
        let mut cpu_hwmon: Option<PathBuf> = None;
        let mut gpu_hwmon: Option<PathBuf> = None;

        if let Ok(entries) = fs::read_dir(HWMON_ROOT) {
            for entry in entries.flatten() {
                let dir = entry.path();
                let name_path = dir.join("name");
                let Ok(name) = fs::read_to_string(&name_path) else { continue };
                let name = name.trim();
                let temp_path = dir.join("temp1_input");
                if !temp_path.exists() {
                    continue;
                }
                if cpu_hwmon.is_none() && CPU_HWMON_NAMES.iter().any(|n| *n == name) {
                    info!("thermal: CPU sensor via hwmon '{}' at {}", name, temp_path.display());
                    cpu_hwmon = Some(temp_path.clone());
                }
                if gpu_hwmon.is_none() && GPU_HWMON_NAMES.iter().any(|n| *n == name) {
                    info!("thermal: GPU sensor via hwmon '{}' at {}", name, temp_path.display());
                    gpu_hwmon = Some(temp_path);
                }
            }
        }

        let nvml = match Nvml::init() {
            Ok(n) => {
                info!("thermal: NVML initialised (GPU temps via NVIDIA driver)");
                Some(n)
            }
            Err(e) => {
                warn!("thermal: NVML unavailable ({}); falling back to hwmon for GPU", e);
                None
            }
        };

        if cpu_hwmon.is_none() {
            warn!("thermal: no CPU hwmon sensor detected (looked for {:?})", CPU_HWMON_NAMES);
        }
        if gpu_hwmon.is_none() && nvml.is_none() {
            warn!("thermal: no GPU sensor detected (NVML failed and no hwmon match)");
        }

        ThermalMonitor { cpu_hwmon, gpu_hwmon, nvml }
    }

    pub fn read_cpu(&self) -> Option<f32> {
        read_hwmon_temp(self.cpu_hwmon.as_ref()?)
    }

    pub fn read_gpu(&self) -> Option<f32> {
        if let Some(nvml) = &self.nvml {
            if let Ok(dev) = nvml.device_by_index(0) {
                if let Ok(t) = dev.temperature(TemperatureSensor::Gpu) {
                    return Some(t as f32);
                }
            }
        }
        read_hwmon_temp(self.gpu_hwmon.as_ref()?)
    }
}

fn read_hwmon_temp(path: &PathBuf) -> Option<f32> {
    let s = fs::read_to_string(path).ok()?;
    let milli: i32 = s.trim().parse().ok()?;
    Some(milli as f32 / 1000.0)
}

/// Piecewise-linear curve evaluator. Returns RPM target for the given temperature,
/// clamped to the device's `(min, max)` fan range.
///
/// Below the first point → first point's RPM. Above the last → last point's RPM.
/// Empty curve → `clamp.0`.
pub fn eval(curve: &[CurvePoint], temp_c: f32, clamp: (u16, u16)) -> u16 {
    let (lo, hi) = clamp;
    let clip = |r: u16| -> u16 { r.max(lo).min(hi) };

    if curve.is_empty() {
        return lo;
    }
    let first = curve[0];
    if temp_c <= first.temp_c as f32 {
        return clip(first.rpm);
    }
    let last = curve[curve.len() - 1];
    if temp_c >= last.temp_c as f32 {
        return clip(last.rpm);
    }
    for window in curve.windows(2) {
        let a = window[0];
        let b = window[1];
        if (a.temp_c as f32) <= temp_c && temp_c <= (b.temp_c as f32) {
            let dt = (b.temp_c as f32) - (a.temp_c as f32);
            if dt <= 0.0 {
                return clip(b.rpm);
            }
            let t = ((temp_c - a.temp_c as f32) / dt).clamp(0.0, 1.0);
            let rpm = (a.rpm as f32) + t * ((b.rpm as f32) - (a.rpm as f32));
            return clip(rpm.round() as u16);
        }
    }
    clip(last.rpm)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comms::CurvePoint;

    fn pt(t: u8, r: u16) -> CurvePoint { CurvePoint { temp_c: t, rpm: r } }

    #[test]
    fn empty_curve_returns_min_clamp() {
        assert_eq!(eval(&[], 50.0, (3500, 5000)), 3500);
    }

    #[test]
    fn below_first_point_returns_first_rpm() {
        let curve = vec![pt(50, 3500), pt(80, 5000)];
        assert_eq!(eval(&curve, 30.0, (3500, 5000)), 3500);
    }

    #[test]
    fn above_last_point_returns_last_rpm() {
        let curve = vec![pt(50, 3500), pt(80, 5000)];
        assert_eq!(eval(&curve, 90.0, (3500, 5000)), 5000);
    }

    #[test]
    fn midpoint_interpolates_linearly() {
        let curve = vec![pt(50, 3500), pt(80, 5000)];
        // Halfway between 50 and 80 -> 3500 + 0.5*(5000-3500) = 4250
        assert_eq!(eval(&curve, 65.0, (3500, 5000)), 4250);
    }

    #[test]
    fn multi_segment_picks_correct_bracket() {
        let curve = vec![pt(40, 3500), pt(55, 3800), pt(70, 4400), pt(85, 5000)];
        // Between 55 (3800) and 70 (4400). At 62.5: halfway -> 4100
        assert_eq!(eval(&curve, 62.5, (3500, 5000)), 4100);
    }

    #[test]
    fn clamp_caps_high_rpm_from_curve() {
        let curve = vec![pt(50, 3500), pt(80, 9000)];
        assert_eq!(eval(&curve, 80.0, (3500, 5000)), 5000);
    }

    #[test]
    fn clamp_lifts_low_rpm_from_curve() {
        let curve = vec![pt(50, 1000), pt(80, 5000)];
        assert_eq!(eval(&curve, 50.0, (3500, 5000)), 3500);
    }

    #[test]
    fn single_point_curve_returns_that_rpm() {
        let curve = vec![pt(60, 4000)];
        assert_eq!(eval(&curve, 30.0, (3500, 5000)), 4000);
        assert_eq!(eval(&curve, 90.0, (3500, 5000)), 4000);
    }
}
