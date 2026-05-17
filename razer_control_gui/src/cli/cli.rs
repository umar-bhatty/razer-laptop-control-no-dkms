#[path = "../comms.rs"]
mod comms;
use clap::{error::ErrorKind, CommandFactory, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(version="0.5.0", about="razer laptop configuration for linux", name="razer-cli")]
struct Cli {
    #[command(subcommand)]
    args: Args,
}

#[derive(Subcommand)]
enum Args {
    /// Read the current configuration of the device for some attribute
    Read {
        #[command(subcommand)]
        attr: ReadAttr,
    },
    /// Write a new configuration to the device for some attribute
    Write {
        #[command(subcommand)]
        attr: WriteAttr,
    },
    /// Write a standard effect
    StandardEffect {
        #[command(subcommand)]
        effect: StandardEffect,
    },
    /// Write a custom effect
    Effect {
        #[command(subcommand)]
        effect: Effect,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum OnOff {
    On,
    Off,
}

impl OnOff {
    pub fn is_on(&self) -> bool {
        matches!(self, Self::On)
    }
}

#[derive(Subcommand)]
enum ReadAttr {
    /// Read the current fan speed
    Fan(AcStateParam),
    /// Read the current power mode
    Power(AcStateParam),
    /// Read the current brightness
    Brightness(AcStateParam),
    /// Read the current logo mode
    Logo(AcStateParam),
    /// Read the current sync mode
    Sync,
    /// Read the current bho mode
    Bho,
    /// Read the current fan mode (firmware/curve/manual)
    FanMode(AcStateParam),
    /// Read the fan curve points for a sensor
    Curve(CurveReadParams),
    /// Read the latest CPU/GPU temperatures the daemon has sampled
    Temps,
}

#[derive(Subcommand)]
enum WriteAttr {
    /// Set the fan speed
    Fan(FanParams),
    /// Set the power mode
    Power(PowerParams),
    /// Set the brightness of the keyboard
    Brightness(BrightnessParams),
    /// Set the logo mode
    Logo(LogoParams),
    /// Set sync
    Sync(SyncParams),
    /// Set battery health optimization
    Bho(BhoParams),
    /// Set the fan mode (firmware/curve/manual)
    FanMode(FanModeParams),
    /// Set the fan curve points for a sensor: "<temp>:<rpm>,<temp>:<rpm>,..."
    Curve(CurveWriteParams),
}

#[derive(Parser)]
struct PowerParams {
    /// battery/plugged in
    ac_state: AcState,
    /// power mode (0, 1, 2, 3 or 4)
    pwr: u8,
    /// cpu boost (0, 1, 2 or 3)
    cpu_mode: Option<u8>,
    /// gpu boost (0, 1 or 2)
    gpu_mode: Option<u8>,
}

#[derive(Parser)]
struct FanParams {
    /// battery/plugged in
    ac_state: AcState,
    /// fan speed in RPM
    speed: i32,
}

#[derive(Parser)]
struct BrightnessParams {
    /// battery/plugged in
    ac_state: AcState,
    /// brightness
    brightness: i32,
}

#[derive(Parser)]
struct LogoParams {
    /// battery/plugged in
    ac_state: AcState,
    /// logo mode (0, 1 or 2)
    logo_state: i32,
}

#[derive(Parser)]
struct SyncParams {
    sync_state: OnOff,
}

#[derive(Parser)]
struct BhoParams {
    state: OnOff,
    /// charging threshold
    threshold: Option<u8>,
}

#[derive(ValueEnum, Clone)]
enum AcState {
    /// battery
    Bat,
    /// plugged in
    Ac,
}

#[derive(Parser, Clone)]
struct AcStateParam {
    /// battery/plugged in
    ac_state: AcState,
}

#[derive(ValueEnum, Clone, Copy)]
enum FanModeArg {
    /// EC firmware drives the fan (legacy behavior)
    Firmware,
    /// Daemon polls temperatures and drives the fan via a curve
    Curve,
    /// Daemon writes a fixed RPM and leaves it
    Manual,
}

impl From<FanModeArg> for comms::FanMode {
    fn from(m: FanModeArg) -> Self {
        match m {
            FanModeArg::Firmware => comms::FanMode::Firmware,
            FanModeArg::Curve => comms::FanMode::Curve,
            FanModeArg::Manual => comms::FanMode::Manual,
        }
    }
}

impl FanModeArg {
    fn describe(m: comms::FanMode) -> &'static str {
        match m {
            comms::FanMode::Firmware => "firmware",
            comms::FanMode::Curve => "curve",
            comms::FanMode::Manual => "manual",
        }
    }
}

#[derive(ValueEnum, Clone, Copy)]
enum SensorArg {
    Cpu,
    Gpu,
}

impl From<SensorArg> for comms::Sensor {
    fn from(s: SensorArg) -> Self {
        match s {
            SensorArg::Cpu => comms::Sensor::Cpu,
            SensorArg::Gpu => comms::Sensor::Gpu,
        }
    }
}

#[derive(Parser)]
struct FanModeParams {
    /// battery/plugged in
    ac_state: AcState,
    /// firmware | curve | manual
    mode: FanModeArg,
}

#[derive(Parser)]
struct CurveReadParams {
    /// battery/plugged in
    ac_state: AcState,
    /// cpu | gpu
    sensor: SensorArg,
}

#[derive(Parser)]
struct CurveWriteParams {
    /// battery/plugged in
    ac_state: AcState,
    /// cpu | gpu
    sensor: SensorArg,
    /// Curve points: "<temp_c>:<rpm>,<temp_c>:<rpm>,..." e.g. "40:3500,70:4400,85:5000"
    points: String,
}

#[derive(Subcommand)]
enum StandardEffect {
    Off,
    Wave(WaveParams),
    Reactive(ReactiveParams),
    Breathing(BreathingParams),
    Spectrum,
    Static(StaticParams),
    Starlight(StarlightParams),
}

#[derive(Parser)]
struct WaveParams {
    /// direction (0 or 1)
    direction: u8,
}

#[derive(Parser)]
struct ReactiveParams {
    /// speed (0-255)
    speed: u8,
    /// red (0-255)
    red: u8,
    /// green (0-255)
    green: u8,
    /// blue (0-255)
    blue: u8,
}

#[derive(Parser)]
struct BreathingParams {
    /// kind (0-2)
    kind: u8,
    /// red1 (0-255)
    red1: u8,
    /// green1 (0-255)
    green1: u8,
    /// blue1 (0-255)
    blue1: u8,
    /// red2 (0-255)
    red2: u8,
    /// green2 (0-255)
    green2: u8,
    /// blue2 (0-255)
    blue2: u8,
}

#[derive(Parser)]
struct StarlightParams {
    /// kind (0-2)
    kind: u8,
    /// speed (0-255)
    speed: u8,
    /// red1 (0-255)
    red1: u8,
    /// green1 (0-255)
    green1: u8,
    /// blue1 (0-255)
    blue1: u8,
    /// red2 (0-255)
    red2: u8,
    /// green2 (0-255)
    green2: u8,
    /// blue2 (0-255)
    blue2: u8,
}

#[derive(Subcommand)]
enum Effect {
    Static(StaticParams),
    StaticGradient(StaticGradientParams),
    WaveGradient(WaveGradientParams),
    BreathingSingle(BreathingSingleParams),
}

#[derive(Parser)]
struct StaticParams {
    /// red (0-255)
    red: u8,
    /// green (0-255)
    green: u8,
    /// blue (0-255)
    blue: u8,
}

#[derive(Parser)]
struct StaticGradientParams {
    /// red1 (0-255)
    red1: u8,
    /// green1 (0-255)
    green1: u8,
    /// blue1 (0-255)
    blue1: u8,
    /// red2 (0-255)
    red2: u8,
    /// green2 (0-255)
    green2: u8,
    /// blue2 (0-255)
    blue2: u8,
}

#[derive(Parser)]
struct WaveGradientParams {
    /// red1 (0-255)
    red1: u8,
    /// green1 (0-255)
    green1: u8,
    /// blue1 (0-255)
    blue1: u8,
    /// red2 (0-255)
    red2: u8,
    /// green2 (0-255)
    green2: u8,
    /// blue2 (0-255)
    blue2: u8,
}

#[derive(Parser)]
struct BreathingSingleParams {
    /// red (0-255)
    red: u8,
    /// green (0-255)
    green: u8,
    /// blue (0-255)
    blue: u8,
    /// duration (0-255)
    duration: u8,
}

fn main() {
    if std::fs::metadata(comms::SOCKET_PATH).is_err() {
        eprintln!("Error. Socket doesn't exit. Is daemon running?");
        std::process::exit(1);
    }

    let cli = Cli::parse();

    match cli.args {
        Args::Read { attr } => match attr {
            ReadAttr::Fan(AcStateParam { ac_state }) => read_fan_rpm(ac_state as usize),
            ReadAttr::Power(AcStateParam { ac_state }) => read_power_mode(ac_state as usize),
            ReadAttr::Brightness(AcStateParam { ac_state }) => read_brightness(ac_state as usize),
            ReadAttr::Logo(AcStateParam { ac_state }) => read_logo_mode(ac_state as usize),
            ReadAttr::Sync => read_sync(),
            ReadAttr::Bho => read_bho(),
            ReadAttr::FanMode(AcStateParam { ac_state }) => read_fan_mode(ac_state as usize),
            ReadAttr::Curve(CurveReadParams { ac_state, sensor }) => {
                read_fan_curve(ac_state as usize, sensor.into())
            }
            ReadAttr::Temps => read_temps(),
        },
        Args::Write { attr } => match attr {
            WriteAttr::Fan(FanParams { ac_state, speed }) => {
                write_fan_speed(ac_state as usize, speed)
            }
            WriteAttr::Power(PowerParams {
                ac_state,
                pwr,
                cpu_mode,
                gpu_mode,
            }) => write_pwr_mode(ac_state as usize, pwr, cpu_mode, gpu_mode),
            WriteAttr::Brightness(BrightnessParams {
                ac_state,
                brightness,
            }) => write_brightness(ac_state as usize, brightness as u8),
            WriteAttr::Sync(SyncParams { sync_state }) => write_sync(sync_state.is_on()),
            WriteAttr::Logo(LogoParams {
                ac_state,
                logo_state,
            }) => write_logo_mode(ac_state as usize, logo_state as u8),
            WriteAttr::Bho(BhoParams { state, threshold }) => {
                validate_and_write_bho(threshold, state)
            }
            WriteAttr::FanMode(FanModeParams { ac_state, mode }) => {
                write_fan_mode(ac_state as usize, mode.into())
            }
            WriteAttr::Curve(CurveWriteParams { ac_state, sensor, points }) => {
                write_fan_curve(ac_state as usize, sensor.into(), &points)
            }
        },
        Args::Effect { effect } => match effect {
            Effect::Static(params) => send_effect(
                "static".to_string(),
                vec![params.red, params.green, params.blue],
            ),
            Effect::StaticGradient(params) => send_effect(
                "static_gradient".to_string(),
                vec![
                    params.red1,
                    params.green1,
                    params.blue1,
                    params.red2,
                    params.green2,
                    params.blue2,
                ],
            ),
            Effect::WaveGradient(params) => send_effect(
                "wave_gradient".to_string(),
                vec![
                    params.red1,
                    params.green1,
                    params.blue1,
                    params.red2,
                    params.green2,
                    params.blue2,
                ],
            ),
            Effect::BreathingSingle(params) => send_effect(
                "breathing_single".to_string(),
                vec![params.red, params.green, params.blue, params.duration],
            ),
        },
        Args::StandardEffect { effect } => match effect {
            StandardEffect::Off => send_standard_effect("off".to_string(), vec![]),
            StandardEffect::Spectrum => send_standard_effect("spectrum".to_string(), vec![]),
            StandardEffect::Breathing(params) => send_standard_effect(
                "breathing".to_string(),
                vec![
                    params.kind,
                    params.red1,
                    params.green1,
                    params.blue1,
                    params.red2,
                    params.green2,
                    params.blue2,
                ],
            ),
            StandardEffect::Reactive(params) => send_standard_effect(
                "reactive".to_string(),
                vec![params.speed, params.red, params.green, params.blue],
            ),
            StandardEffect::Starlight(params) => send_standard_effect(
                "starlight".to_string(),
                vec![
                    params.kind,
                    params.speed,
                    params.red1,
                    params.green1,
                    params.blue1,
                    params.red2,
                    params.green2,
                    params.blue2,
                ],
            ),
            StandardEffect::Static(params) => send_standard_effect(
                "static".to_string(),
                vec![params.red, params.green, params.blue],
            ),
            StandardEffect::Wave(params) => {
                send_standard_effect("wave".to_string(), vec![params.direction])
            }
        },
    }
}

fn validate_and_write_bho(threshold: Option<u8>, state: OnOff) {
    match threshold {
        Some(threshold) => {
            if !valid_bho_threshold(threshold) {
                Cli::command()
                    .error(
                        ErrorKind::InvalidValue,
                        "Threshold must be multiple of 5 between 50 and 80",
                    )
                    .exit()
            }
            write_bho(state.is_on(), threshold)
        }
        None => {
            if state.is_on() {
                Cli::command()
                    .error(
                        ErrorKind::MissingRequiredArgument,
                        "Threshold is required when BHO is on",
                    )
                    .exit()
            }
            write_bho(state.is_on(), 80)
        }
    }
}

fn read_bho() {
    send_data(comms::DaemonCommand::GetBatteryHealthOptimizer()).map_or_else(
        || eprintln!("Unknown error occured when getting bho"),
        |result| {
            if let comms::DaemonResponse::GetBatteryHealthOptimizer { is_on, threshold } = result {
                match is_on {
                    true => {
                        println!(
                            "Battery health optimization is on with a threshold of {}",
                            threshold
                        );
                    }
                    false => {
                        eprintln!("Battery health optimization is off");
                    }
                }
            }
        },
    );
}

fn write_bho(on: bool, threshold: u8) {
    if !on {
        bho_toggle_off();
        return;
    }

    bho_toggle_on(threshold);
}

fn bho_toggle_on(threshold: u8) {
    if !valid_bho_threshold(threshold) {
        eprintln!("Threshold value must be a multiple of five between 50 and 80");
        return;
    }

    send_data(comms::DaemonCommand::SetBatteryHealthOptimizer {
        is_on: true,
        threshold: threshold,
    })
    .map_or_else(
        || eprintln!("Unknown error occured when toggling bho"),
        |result| {
            if let comms::DaemonResponse::SetBatteryHealthOptimizer { result } = result {
                match result {
                    true => {
                        println!(
                            "Battery health optimization is on with a threshold of {}",
                            threshold
                        );
                    }
                    false => {
                        eprintln!("Failed to turn on bho with threshold of {}", threshold);
                    }
                }
            }
        },
    );
}

fn valid_bho_threshold(threshold: u8) -> bool {
    if threshold % 5 != 0 {
        return false;
    }

    if threshold < 50 || threshold > 80 {
        return false;
    }

    return true;
}

fn bho_toggle_off() {
    send_data(comms::DaemonCommand::SetBatteryHealthOptimizer {
        is_on: false,
        threshold: 80,
    })
    .map_or_else(
        || eprintln!("Unknown error occured when toggling bho"),
        |result| {
            if let comms::DaemonResponse::SetBatteryHealthOptimizer { result } = result {
                match result {
                    true => {
                        println!("Successfully turned off bho");
                    }
                    false => {
                        eprintln!("Failed to turn off bho");
                    }
                }
            }
        },
    );
}

fn send_standard_effect(name: String, params: Vec<u8>) {
    match send_data(comms::DaemonCommand::SetStandardEffect { name, params }) {
        Some(comms::DaemonResponse::SetStandardEffect { result }) => {
            if result {
                println!("Effect set OK!");
            } else {
                eprintln!("Effect set FAIL!");
            }
        },
        Some(_) => eprintln!("Unexpected response from daemon!"),
        None => eprintln!("Unknown daemon error!"),
    }
}

fn send_effect(name: String, params: Vec<u8>) {
    match send_data(comms::DaemonCommand::SetEffect { name, params }) {
        Some(comms::DaemonResponse::SetEffect { result }) => {
            if result {
                println!("Effect set OK!");
            } else {
                eprintln!("Effect set FAIL!");
            }
        },
        Some(_) => eprintln!("Unexpected response from daemon!"),
        None => eprintln!("Unknown daemon error!"),
    }
}

fn send_data(opt: comms::DaemonCommand) -> Option<comms::DaemonResponse> {
    match comms::bind() {
        Some(socket) => comms::send_to_daemon(opt, socket),
        None => {
            eprintln!("Error. Cannot bind to socket");
            None
        },
    }
}

fn read_fan_rpm(ac: usize) {
    match send_data(comms::DaemonCommand::GetFanSpeed { ac }) {
        Some(comms::DaemonResponse::GetFanSpeed { rpm }) => {
            let rpm_desc: String = match rpm {
                f if f < 0 => String::from("Unknown"),
                0 => String::from("Auto (0)"),
                _ => format!("{} RPM", rpm),
            };
            println!("Current fan setting: {}", rpm_desc);
        },
        Some(_) => eprintln!("Daemon responded with invalid data!"),
        None => eprintln!("Unknown daemon error!"),
    }
}

fn read_logo_mode(ac: usize) {
    match send_data(comms::DaemonCommand::GetLogoLedState { ac }) {
        Some(comms::DaemonResponse::GetLogoLedState { logo_state }) => {
            let logo_state_desc: &str = match logo_state {
                0 => "Off",
                1 => "On",
                2 => "Breathing",
                _ => "Unknown",
            };
            println!("Current logo setting: {}", logo_state_desc);
        },
        Some(_) => eprintln!("Daemon responded with invalid data!"),
        None => eprintln!("Unknown daemon error!"),
    }
}

fn read_power_mode(ac: usize) {
    if let Some(resp) = send_data(comms::DaemonCommand::GetPwrLevel { ac }) {
        if let comms::DaemonResponse::GetPwrLevel { pwr } = resp {
            let power_desc: &str = match pwr {
                0 => "Balanced",
                1 => "Gaming",
                2 => "Creator",
                3 => "Silent",
                4 => "Custom",
                _ => "Unknown",
            };
            println!("Current power setting: {}", power_desc);
            if pwr == 4 {
                if let Some(resp) = send_data(comms::DaemonCommand::GetCPUBoost { ac }) {
                    if let comms::DaemonResponse::GetCPUBoost { cpu } = resp {
                        let cpu_boost_desc: &str = match cpu {
                            0 => "Low",
                            1 => "Medium",
                            2 => "High",
                            3 => "Boost",
                            _ => "Unknown",
                        };
                        println!("Current CPU setting: {}", cpu_boost_desc);
                    };
                }
                if let Some(resp) = send_data(comms::DaemonCommand::GetGPUBoost { ac }) {
                    if let comms::DaemonResponse::GetGPUBoost { gpu } = resp {
                        let gpu_boost_desc: &str = match gpu {
                            0 => "Low",
                            1 => "Medium",
                            2 => "High",
                            _ => "Unknown",
                        };
                        println!("Current GPU setting: {}", gpu_boost_desc);
                    };
                }
            }
        } else {
            eprintln!("Daemon responded with invalid data!");
        }
    }
}

fn write_pwr_mode(ac: usize, pwr_mode: u8, cpu_mode: Option<u8>, gpu_mode: Option<u8>) {
    if pwr_mode > 4 {
        Cli::command()
            .error(ErrorKind::InvalidValue, "Power mode must be 0, 1, 2, 3 or 4")
            .exit()
    }

    let cm = if pwr_mode == 4 {
        cpu_mode.expect("CPU mode must be provided when power mode is 4")
    } else {
        cpu_mode.unwrap_or(0)
    };

    if cm > 3 {
        Cli::command()
            .error(ErrorKind::InvalidValue, "CPU mode must be between 0 and 3")
            .exit()
    }

    let gm = if pwr_mode == 4 {
        gpu_mode.expect("GPU mode must be provided when power mode is 4")
    } else {
        gpu_mode.unwrap_or(0)
    };

    if gm > 2 {
        Cli::command()
            .error(ErrorKind::InvalidValue, "GPU mode must be between 0 and 2")
            .exit()
    }

    match send_data(comms::DaemonCommand::SetPowerMode {
        ac,
        pwr: pwr_mode,
        cpu: cm,
        gpu: gm,
    }) {
        Some(_) => read_power_mode(ac),
        None => {
            Cli::command()
                .error(
                    ErrorKind::DisplayHelp,
                    "An error occurred while sending the command to the daemon",
                )
                .exit()
        },
    }
}

fn read_brightness(ac: usize) {
    match send_data(comms::DaemonCommand::GetBrightness { ac }) {
        Some(comms::DaemonResponse::GetBrightness { result }) => {
            println!("Current brightness: {}", result);
        },
        Some(_) => eprintln!("Daemon responded with invalid data!"),
        None => eprintln!("Unknown daemon error!"),
    }
}

fn read_sync() {
    match send_data(comms::DaemonCommand::GetSync()) {
        Some(comms::DaemonResponse::GetSync { sync }) => {
            println!("Current sync: {:?}", sync);
        },
        Some(_) => eprintln!("Daemon responded with invalid data!"),
        None => eprintln!("Unknown daemon error!"),
    }
}

fn write_brightness(ac: usize, val: u8) {
    match send_data(comms::DaemonCommand::SetBrightness { ac, val }) {
        Some(_) => read_brightness(ac),
        None => eprintln!("Unknown error!"),
    }
}

fn write_fan_speed(ac: usize, x: i32) {
    match send_data(comms::DaemonCommand::SetFanSpeed { ac, rpm: x }) {
        Some(_) => read_fan_rpm(ac),
        None => eprintln!("Unknown error!"),
    }
}

fn write_logo_mode(ac: usize, x: u8) {
    match send_data(comms::DaemonCommand::SetLogoLedState { ac, logo_state: x }) {
        Some(_) => read_logo_mode(ac),
        None => eprintln!("Unknown error!"),
    }
}

fn write_sync(sync: bool) {
    match send_data(comms::DaemonCommand::SetSync { sync }) {
        Some(_) => read_sync(),
        None => eprintln!("Unknown error!"),
    }
}

fn read_fan_mode(ac: usize) {
    match send_data(comms::DaemonCommand::GetFanMode { ac }) {
        Some(comms::DaemonResponse::GetFanMode { mode }) => {
            println!("Current fan mode: {}", FanModeArg::describe(mode));
        }
        Some(_) => eprintln!("Daemon responded with invalid data!"),
        None => eprintln!("Unknown daemon error!"),
    }
}

fn write_fan_mode(ac: usize, mode: comms::FanMode) {
    match send_data(comms::DaemonCommand::SetFanMode { ac, mode }) {
        Some(_) => read_fan_mode(ac),
        None => eprintln!("Unknown error!"),
    }
}

fn read_fan_curve(ac: usize, sensor: comms::Sensor) {
    match send_data(comms::DaemonCommand::GetFanCurve { ac, sensor }) {
        Some(comms::DaemonResponse::GetFanCurve { points }) => {
            let label = match sensor {
                comms::Sensor::Cpu => "CPU",
                comms::Sensor::Gpu => "GPU",
            };
            let rendered: Vec<String> = points
                .iter()
                .map(|p| format!("{}:{}", p.temp_c, p.rpm))
                .collect();
            println!("{} curve: {}", label, rendered.join(","));
        }
        Some(_) => eprintln!("Daemon responded with invalid data!"),
        None => eprintln!("Unknown daemon error!"),
    }
}

fn write_fan_curve(ac: usize, sensor: comms::Sensor, raw: &str) {
    let points = match parse_curve(raw) {
        Ok(p) => p,
        Err(e) => {
            Cli::command()
                .error(ErrorKind::InvalidValue, format!("invalid curve: {e}"))
                .exit();
        }
    };
    match send_data(comms::DaemonCommand::SetFanCurve { ac, sensor, points }) {
        Some(_) => read_fan_curve(ac, sensor),
        None => eprintln!("Unknown error!"),
    }
}

fn read_temps() {
    match send_data(comms::DaemonCommand::GetTemps) {
        Some(comms::DaemonResponse::GetTemps {
            cpu, gpu, target_rpm, current_rpm,
            pkg_watts, gpu_watts, pkg_pl1_w, pkg_pl2_w, gpu_tgp_w,
        }) => {
            let temp = |t: Option<f32>| -> String {
                t.map(|v| format!("{:.1}°C", v)).unwrap_or_else(|| "n/a".to_string())
            };
            let cur = current_rpm
                .map(|r| format!("{} RPM", r))
                .unwrap_or_else(|| "n/a".to_string());
            let w = |v: Option<f32>| -> String {
                v.map(|x| format!("{:.1}W", x)).unwrap_or_else(|| "n/a".to_string())
            };
            let limit_pair = |a: Option<u32>, b: Option<u32>| -> String {
                match (a, b) {
                    (Some(a), Some(b)) => format!("{}/{}W", a, b),
                    _ => "n/a".to_string(),
                }
            };
            let tgp = gpu_tgp_w.map(|v| format!("{}W", v)).unwrap_or_else(|| "n/a".to_string());
            println!(
                "CPU: {}   GPU: {}   Current: {}   Target: {} RPM",
                temp(cpu), temp(gpu), cur, target_rpm
            );
            println!(
                "Pkg: {} (PL1/PL2 {})   GPU: {} (TGP {})",
                w(pkg_watts), limit_pair(pkg_pl1_w, pkg_pl2_w),
                w(gpu_watts), tgp
            );
        }
        Some(_) => eprintln!("Daemon responded with invalid data!"),
        None => eprintln!("Unknown daemon error!"),
    }
}

fn parse_curve(raw: &str) -> Result<Vec<comms::CurvePoint>, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err("curve cannot be empty".to_string());
    }
    let mut out = Vec::new();
    let mut prev_temp: Option<u8> = None;
    for (i, segment) in trimmed.split(',').enumerate() {
        let segment = segment.trim();
        let (t, r) = segment
            .split_once(':')
            .ok_or_else(|| format!("point #{} missing ':' (got '{}')", i + 1, segment))?;
        let temp_c: u8 = t.trim().parse()
            .map_err(|_| format!("point #{} temperature is not an integer 0-255 (got '{}')", i + 1, t))?;
        let rpm: u16 = r.trim().parse()
            .map_err(|_| format!("point #{} rpm is not an integer 0-65535 (got '{}')", i + 1, r))?;
        if let Some(prev) = prev_temp {
            if temp_c <= prev {
                return Err(format!(
                    "point #{} temperature ({}) must be strictly greater than previous ({})",
                    i + 1, temp_c, prev
                ));
            }
        }
        prev_temp = Some(temp_c);
        out.push(comms::CurvePoint { temp_c, rpm });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_well_formed_curve() {
        let pts = parse_curve("40:3500,55:3800,70:4400,85:5000").unwrap();
        assert_eq!(pts.len(), 4);
        assert_eq!(pts[0].temp_c, 40);
        assert_eq!(pts[0].rpm, 3500);
        assert_eq!(pts[3].temp_c, 85);
        assert_eq!(pts[3].rpm, 5000);
    }

    #[test]
    fn tolerates_whitespace() {
        let pts = parse_curve(" 40 : 3500 , 70 : 5000 ").unwrap();
        assert_eq!(pts.len(), 2);
        assert_eq!(pts[1].temp_c, 70);
    }

    #[test]
    fn rejects_empty() {
        assert!(parse_curve("").is_err());
        assert!(parse_curve("   ").is_err());
    }

    #[test]
    fn rejects_non_monotonic_temperatures() {
        assert!(parse_curve("70:3500,55:4000").is_err());
        assert!(parse_curve("70:3500,70:4000").is_err());
    }

    #[test]
    fn rejects_malformed_segments() {
        assert!(parse_curve("40-3500").is_err());
        assert!(parse_curve("40:").is_err());
        assert!(parse_curve(":3500").is_err());
        assert!(parse_curve("abc:3500").is_err());
    }
}
