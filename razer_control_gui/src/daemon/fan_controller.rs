use std::sync::Mutex;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use lazy_static::lazy_static;
use log::{debug, info};

use crate::comms::{CurvePoint, FanMode};
use crate::config::FanCurve;
use crate::thermal;

/// How often the controller re-evaluates the curve.
const TICK: Duration = Duration::from_millis(1000);
/// Hardware RPM granularity (the device protocol stores RPM/100 in a u8).
const RPM_BUCKET: u16 = 100;
/// Cap on per-tick change to avoid abrupt fan jolts.
const RAMP_LIMIT: u16 = 300;

#[derive(Default, Copy, Clone, Debug)]
pub struct TempSnapshot {
    pub cpu: Option<f32>,
    pub gpu: Option<f32>,
    pub target_rpm: u16,
}

lazy_static! {
    /// Latest reading from the controller loop. Read by `GetTemps` dispatch.
    pub static ref TEMP_CACHE: Mutex<TempSnapshot> = Mutex::new(TempSnapshot::default());
    /// Thermal sensors probed at daemon startup.
    pub static ref THERMAL: Mutex<thermal::ThermalMonitor> =
        Mutex::new(thermal::ThermalMonitor::probe());
    /// Last RPM the controller wrote, used for ramp-limiting.
    static ref LAST_WRITTEN: Mutex<Option<u16>> = Mutex::new(None);
}

pub fn start_fan_controller_task() -> JoinHandle<()> {
    info!("fan_controller: starting (tick = {} ms)", TICK.as_millis());
    thread::spawn(|| loop {
        tick();
        thread::sleep(TICK);
    })
}

fn tick() {
    // Phase 1: snapshot state under the device-manager lock, then drop it.
    let Some(snapshot) = snapshot_state() else { return };

    // Phase 2: read sensors (no locks held).
    let (cpu_temp, gpu_temp) = {
        let monitor = match THERMAL.lock() {
            Ok(g) => g,
            Err(_) => return,
        };
        (monitor.read_cpu(), monitor.read_gpu())
    };

    // Phase 3: evaluate curve and decide on a target RPM.
    let target = match snapshot.mode {
        FanMode::Curve => compute_target(&snapshot.curve, cpu_temp, gpu_temp, snapshot.fan_range),
        _ => 0,
    };

    // Phase 4: update the temp cache so `GetTemps` sees fresh data regardless of mode.
    if let Ok(mut cache) = TEMP_CACHE.lock() {
        cache.cpu = cpu_temp;
        cache.gpu = gpu_temp;
        cache.target_rpm = target;
    }

    if snapshot.mode != FanMode::Curve {
        return;
    }

    // Phase 5: ramp-limit and bucket to hardware granularity.
    let limited = ramp_limit(target);
    let bucketed = bucket(limited);

    // Phase 6: write under the lock, but re-check the mode in case a CLI flipped it
    // mid-tick.
    let mut dev_manager = match crate::DEV_MANAGER.lock() {
        Ok(g) => g,
        Err(_) => return,
    };
    let still_curve = dev_manager
        .get_active_ac()
        .map(|ac| dev_manager.get_fan_mode(ac) == FanMode::Curve)
        .unwrap_or(false);
    if !still_curve {
        return;
    }

    let mut last = LAST_WRITTEN.lock().unwrap();
    if Some(bucketed) == *last {
        return;
    }
    if dev_manager.apply_curve_rpm(bucketed) {
        debug!(
            "fan_controller: target={} bucketed={} cpu={:?} gpu={:?}",
            target, bucketed, cpu_temp, gpu_temp
        );
        *last = Some(bucketed);
    }
}

struct StateSnapshot {
    mode: FanMode,
    curve: FanCurve,
    fan_range: (u16, u16),
}

fn snapshot_state() -> Option<StateSnapshot> {
    let mut dev_manager = crate::DEV_MANAGER.lock().ok()?;
    let ac = dev_manager.get_active_ac()?;
    let mode = dev_manager.get_fan_mode(ac);
    let cpu = dev_manager.get_fan_curve(ac, crate::comms::Sensor::Cpu);
    let gpu = dev_manager.get_fan_curve(ac, crate::comms::Sensor::Gpu);
    let fan_range = dev_manager.get_fan_range()?;
    Some(StateSnapshot {
        mode,
        curve: FanCurve { cpu, gpu },
        fan_range,
    })
}

fn compute_target(
    curve: &FanCurve,
    cpu_temp: Option<f32>,
    gpu_temp: Option<f32>,
    fan_range: (u16, u16),
) -> u16 {
    let cpu_target = cpu_temp
        .map(|t| thermal::eval(&curve.cpu, t, fan_range))
        .unwrap_or(fan_range.0);
    let gpu_target = gpu_temp
        .map(|t| thermal::eval(&curve.gpu, t, fan_range))
        .unwrap_or(fan_range.0);
    cpu_target.max(gpu_target)
}

fn ramp_limit(target: u16) -> u16 {
    let last = LAST_WRITTEN.lock().unwrap();
    match *last {
        None => target,
        Some(prev) => {
            if target > prev {
                target.min(prev.saturating_add(RAMP_LIMIT))
            } else {
                target.max(prev.saturating_sub(RAMP_LIMIT))
            }
        }
    }
}

fn bucket(rpm: u16) -> u16 {
    ((rpm + RPM_BUCKET / 2) / RPM_BUCKET) * RPM_BUCKET
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bucket_rounds_to_nearest_hundred() {
        assert_eq!(bucket(0), 0);
        assert_eq!(bucket(49), 0);
        assert_eq!(bucket(50), 100);
        assert_eq!(bucket(3549), 3500);
        assert_eq!(bucket(3550), 3600);
        assert_eq!(bucket(4499), 4500);
    }
}
