// Forza Horizon dash packet (324 bytes)
pub(crate) const PACKET_SIZE: usize = 324;
pub(crate) const DEFAULT_LISTEN_HOST: &str = "127.0.0.1";
pub(crate) const DEFAULT_LISTEN_PORT: u16 = 9999;
pub(crate) const ENGINE_MAX_RPM_OFFSET: usize = 8;
pub(crate) const CURRENT_ENGINE_RPM_OFFSET: usize = 16;
pub(crate) const POWER_OFFSET: usize = 260;
pub(crate) const TORQUE_OFFSET: usize = 264;
pub(crate) const ACCEL_OFFSET: usize = 315;
pub(crate) const HANDBRAKE_OFFSET: usize = 318;
pub(crate) const IS_RACE_ON_OFFSET: usize = 0;
pub(crate) const CAR_ORDINAL_OFFSET: usize = 212;
pub(crate) const CAR_CLASS_OFFSET: usize = 216;
pub(crate) const CAR_PERFORMANCE_INDEX_OFFSET: usize = 220;
pub(crate) const NUM_CYLINDERS_OFFSET: usize = 228;

// Sampling
pub(crate) const SAMPLE_HZ: usize = 60;
pub(crate) const HISTORY_LEN: usize = SAMPLE_HZ * 5;

// Shift lights
pub(crate) const DEFAULT_SHIFT_LIGHTS_LIT_OPACITY: f32 = 0.7;
pub(crate) const DEFAULT_SHIFT_LIGHTS_DIM_OPACITY: f32 = 0.5;
pub(crate) const DEFAULT_SHIFT_LIGHTS_OFFSET_PX: usize = 0;
pub(crate) const DEFAULT_SHIFT_LIGHTS_THICKNESS_PX: usize = 40;
pub(crate) const DEFAULT_SHIFT_LIGHTS_GAP_PX: usize = 8;
pub(crate) const DEFAULT_SHIFT_LIGHTS_WIDTH_PERCENT: usize = 75;
pub(crate) const DEFAULT_SHIFT_LIGHTS_BLINK_PERCENT: f32 = 95.0;
pub(crate) const SHIFT_LIGHTS_BLINK_PERCENT_MIN: f32 = 80.0;
pub(crate) const SHIFT_LIGHTS_BLINK_PERCENT_MAX: f32 = 100.0;

// Fuel-cut detection / calibration
pub(crate) const FUEL_CUT_WINDOW: usize = SAMPLE_HZ * 700 / 1000;
pub(crate) const RPM_REVERSAL_FLOOR: f32 = 40.0;
pub(crate) const RPM_REVERSAL_RANGE_FRAC: f32 = 0.10;
pub(crate) const DEFAULT_CALIBRATE_MS: usize = 500;
pub(crate) const CALIBRATE_MS_MIN: usize = 100;
pub(crate) const CALIBRATE_MS_MAX: usize = 5000;
pub(crate) const CALIBRATE_PEAK_HOLD_MS: usize = 400;
pub(crate) const CALIBRATE_PEAK_HOLD_FRAMES: usize = SAMPLE_HZ * CALIBRATE_PEAK_HOLD_MS / 1000;
