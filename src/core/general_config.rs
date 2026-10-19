use serde::{Deserialize, Serialize};

// Forza Horizon dash packet (324 bytes)
pub(crate) const PACKET_SIZE: usize = 324;
pub(crate) const DEFAULT_LISTEN_HOST: &str = "127.0.0.1";
pub(crate) const DEFAULT_LISTEN_PORT: u16 = 9999;
pub(crate) const DEFAULT_FORWARD_HOST: &str = "127.0.0.1";
pub(crate) const DEFAULT_FORWARD_PORT: u16 = 10000;
pub(crate) const ENGINE_MAX_RPM_OFFSET: usize = 8;
pub(crate) const CURRENT_ENGINE_RPM_OFFSET: usize = 16;
pub(crate) const POWER_OFFSET: usize = 260;
pub(crate) const TORQUE_OFFSET: usize = 264;
pub(crate) const ACCEL_OFFSET: usize = 315;
pub(crate) const HANDBRAKE_OFFSET: usize = 318;
pub(crate) const CURRENT_GEAR_OFFSET: usize = 319;
pub(crate) const IS_RACE_ON_OFFSET: usize = 0;
pub(crate) const CAR_ORDINAL_OFFSET: usize = 212;
pub(crate) const CAR_CLASS_OFFSET: usize = 216;
pub(crate) const CAR_PERFORMANCE_INDEX_OFFSET: usize = 220;
pub(crate) const NUM_CYLINDERS_OFFSET: usize = 228;

// Sampling
pub(crate) const SAMPLE_HZ: usize = 60;
pub(crate) const HISTORY_LEN: usize = SAMPLE_HZ * 5;

// Fuel-cut detection / calibration
pub(crate) const FUEL_CUT_WINDOW: usize = SAMPLE_HZ * 700 / 1000;
pub(crate) const RPM_REVERSAL_FLOOR: f32 = 40.0;
pub(crate) const RPM_REVERSAL_RANGE_FRAC: f32 = 0.10;
pub(crate) const DEFAULT_CALIBRATE_MS: usize = 500;
pub(crate) const CALIBRATE_MS_MIN: usize = 100;
pub(crate) const CALIBRATE_MS_MAX: usize = 5000;
pub(crate) const CALIBRATE_PEAK_HOLD_MS: usize = 400;
pub(crate) const CALIBRATE_PEAK_HOLD_FRAMES: usize = SAMPLE_HZ * CALIBRATE_PEAK_HOLD_MS / 1000;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct GeneralConfig {
    pub(crate) only_show_in_game: bool,
    pub(crate) charts_visible: bool,
    pub(crate) calibrate_hint_visible: bool,
    pub(crate) strict_calibrate_conditions: bool,
    pub(crate) remember_calibrated_cars: bool,
    pub(crate) calibrate_ms: usize,
    pub(crate) listen_host: String,
    pub(crate) listen_port: u16,
    pub(crate) forward_enabled: bool,
    pub(crate) forward_host: String,
    pub(crate) forward_port: u16,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            only_show_in_game: true,
            charts_visible: false,
            calibrate_hint_visible: false,
            strict_calibrate_conditions: false,
            remember_calibrated_cars: true,
            calibrate_ms: DEFAULT_CALIBRATE_MS,
            listen_host: DEFAULT_LISTEN_HOST.to_string(),
            listen_port: DEFAULT_LISTEN_PORT,
            forward_enabled: false,
            forward_host: DEFAULT_FORWARD_HOST.to_string(),
            forward_port: DEFAULT_FORWARD_PORT,
        }
    }
}

pub(crate) fn calibrate_frames(calibrate_ms: usize) -> usize {
    (SAMPLE_HZ * calibrate_ms / 1000).max(1)
}
