use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::core::general_config::{
    CALIBRATE_PEAK_HOLD_FRAMES, FUEL_CUT_WINDOW, RPM_REVERSAL_FLOOR, RPM_REVERSAL_RANGE_FRAC,
    SAMPLE_HZ,
};
use crate::core::props::{CalibrationState, ChartFrame, GearValue, HudTelemetryProps};
use crate::core::ring_buffer::RingBuffer;
use crate::core::telemetry::{CarId, TelemetrySample};

static SHIFT_LIGHTS_CALIBRATED: AtomicBool = AtomicBool::new(false);
static ELECTRIC_CAR: AtomicBool = AtomicBool::new(false);
static RESET_REQUEST: AtomicBool = AtomicBool::new(false);
static CURRENT_CAR_ID: Mutex<Option<CarId>> = Mutex::new(None);

pub(crate) fn shift_lights_calibrated() -> bool {
    SHIFT_LIGHTS_CALIBRATED.load(Ordering::Relaxed)
}

fn set_shift_lights_calibrated(calibrated: bool) {
    SHIFT_LIGHTS_CALIBRATED.store(calibrated, Ordering::Relaxed);
}

pub(crate) fn electric_car() -> bool {
    ELECTRIC_CAR.load(Ordering::Relaxed)
}

fn set_electric_car(electric: bool) {
    ELECTRIC_CAR.store(electric, Ordering::Relaxed);
}

pub(crate) fn current_car_id() -> Option<CarId> {
    *CURRENT_CAR_ID.lock().unwrap_or_else(|err| err.into_inner())
}

fn set_current_car_id(id: Option<CarId>) {
    *CURRENT_CAR_ID.lock().unwrap_or_else(|err| err.into_inner()) = id;
}

pub(crate) fn request_reset() {
    set_shift_lights_calibrated(false);
    RESET_REQUEST.store(true, Ordering::Relaxed);
}

pub(crate) fn take_reset_request() -> bool {
    RESET_REQUEST.swap(false, Ordering::Relaxed)
}

pub(crate) trait CarMemory {
    fn remember(&self) -> bool;
    fn fuel_cut_rpm(&self, id: CarId) -> Option<f32>;
    fn save(&self, id: CarId, rpm: f32);
}

pub(crate) struct CalibrationParams {
    pub(crate) strict: bool,
    pub(crate) calibrate_frames: usize,
}

pub(crate) struct TelemetryEngine {
    rpm: f32,
    max_rpm: f32,
    torque: f32,
    power: f32,
    accel: u8,
    handbrake: u8,
    current_gear: Option<u8>,
    has_data: bool,
    car_id: Option<CarId>,
    fuel_cut_rpm: Option<f32>,
    calibrate_frames: usize,
    calibrate_peak_hold: usize,
    calibrate_rpm_max: f32,
    rpm_history: RingBuffer,
    torque_history: RingBuffer,
    power_history: RingBuffer,
}

impl TelemetryEngine {
    pub(crate) fn new() -> Self {
        Self {
            rpm: 0.0,
            max_rpm: 0.0,
            torque: 0.0,
            power: 0.0,
            accel: 0,
            handbrake: 0,
            current_gear: None,
            has_data: false,
            car_id: None,
            fuel_cut_rpm: None,
            calibrate_frames: 0,
            calibrate_peak_hold: 0,
            calibrate_rpm_max: f32::MIN,
            rpm_history: RingBuffer::new(),
            torque_history: RingBuffer::new(),
            power_history: RingBuffer::new(),
        }
    }

    pub(crate) fn apply_sample(
        &mut self,
        sample: TelemetrySample,
        params: &CalibrationParams,
        memory: &impl CarMemory,
    ) {
        if let Some(car_id) = sample.car_id() {
            if self.car_id != Some(car_id) {
                if self.car_id.is_some() {
                    self.reset_for_new_car();
                }
                self.car_id = Some(car_id);
                set_current_car_id(Some(car_id));
                self.restore_stored_calibration(car_id, memory);
            }
        }
        self.rpm = sample.rpm;
        self.max_rpm = sample.max_rpm;
        self.power = sample.power;
        self.torque = sample.torque;
        self.accel = sample.accel;
        self.handbrake = sample.handbrake;
        self.current_gear = Some(sample.current_gear);
        self.has_data = true;
        self.rpm_history.push(sample.rpm);
        self.power_history.push(sample.power);
        self.torque_history.push(sample.torque);
        if sample.car_id().is_some() && sample.num_cylinders == 0 {
            set_electric_car(true);
            self.apply_electric_limiter(sample.max_rpm);
        } else {
            if sample.car_id().is_some() {
                set_electric_car(false);
            }
            self.calibrate_fuel_cut(params, memory);
        }
    }

    pub(crate) fn clear_runtime_calibration(&mut self) {
        self.fuel_cut_rpm = None;
        set_shift_lights_calibrated(false);
        self.reset_calibration();
    }

    pub(crate) fn restore_if_remembered(&mut self, memory: &impl CarMemory) {
        if !memory.remember() {
            return;
        }
        if let Some(car_id) = self.car_id {
            self.restore_stored_calibration(car_id, memory);
        }
    }

    pub(crate) fn props(
        &self,
        now_ms: u128,
        strict_calibrate: bool,
        calibrate_hint_visible: bool,
        listen_addr: String,
        calibrate_frames_target: usize,
    ) -> HudTelemetryProps {
        let calibration = match self.fuel_cut_rpm {
            Some(limit) if limit > 0.0 => CalibrationState::Ready {
                limit_rpm: limit,
                rpm_ratio: self.rpm / limit,
            },
            _ => {
                let (progress, elapsed_s, duration_s) =
                    uncalibrated_timing(self.calibrate_frames, calibrate_frames_target);
                CalibrationState::Uncalibrated {
                    progress,
                    elapsed_s,
                    duration_s,
                }
            }
        };
        HudTelemetryProps {
            has_data: self.has_data,
            rpm: self.rpm,
            max_rpm: self.max_rpm,
            calibration,
            gear: GearValue::from_raw(self.current_gear),
            power_w: self.power,
            torque_nm: self.torque,
            now_ms,
            strict_calibrate,
            calibrate_hint_visible,
            listen_addr,
        }
    }

    pub(crate) fn charts(&self) -> ChartFrame {
        ChartFrame {
            rpm: self.rpm_history.samples(),
            torque: self.torque_history.samples(),
            power: self.power_history.samples(),
            limit_rpm: self.fuel_cut_rpm,
            rpm_now: self.rpm,
            torque_now: self.torque,
            power_now: self.power,
            max_rpm: self.max_rpm,
            has_data: self.has_data,
        }
    }

    fn apply_electric_limiter(&mut self, max_rpm: f32) {
        if self.fuel_cut_rpm == Some(max_rpm) {
            return;
        }
        self.fuel_cut_rpm = Some(max_rpm);
        set_shift_lights_calibrated(true);
        self.reset_calibration();
    }

    fn is_fuel_cut(&self) -> bool {
        if self.rpm_history.len() < FUEL_CUT_WINDOW {
            return false;
        }
        let rpm: Vec<f32> = self.rpm_history.last_n(FUEL_CUT_WINDOW).collect();
        sign_cross(self.power_history.last_n(FUEL_CUT_WINDOW))
            && sign_cross(self.torque_history.last_n(FUEL_CUT_WINDOW))
            && rpm_chatter(&rpm)
    }

    fn calibrate_fuel_cut(&mut self, params: &CalibrationParams, memory: &impl CarMemory) {
        if self.fuel_cut_rpm.is_some() {
            return;
        }
        let holding = if params.strict {
            self.handbrake > 0 && self.accel > 0 && self.is_fuel_cut()
        } else {
            self.accel > 0 && self.is_fuel_cut()
        };
        if !holding {
            self.reset_calibration();
            return;
        }

        self.calibrate_frames += 1;
        if self.rpm > self.calibrate_rpm_max {
            self.calibrate_rpm_max = self.rpm;
            self.calibrate_peak_hold = 0;
        } else {
            self.calibrate_peak_hold += 1;
        }
        if self.calibrate_frames < params.calibrate_frames
            || self.calibrate_peak_hold < CALIBRATE_PEAK_HOLD_FRAMES
        {
            return;
        }

        self.fuel_cut_rpm = Some(self.calibrate_rpm_max);
        set_shift_lights_calibrated(true);
        if let Some(car_id) = self.car_id {
            if memory.remember() {
                memory.save(car_id, self.calibrate_rpm_max);
            }
        }
        self.reset_calibration();
    }

    fn restore_stored_calibration(&mut self, car_id: CarId, memory: &impl CarMemory) {
        if self.fuel_cut_rpm.is_some() {
            return;
        }
        if !memory.remember() {
            return;
        }
        let Some(rpm) = memory.fuel_cut_rpm(car_id) else {
            return;
        };
        self.fuel_cut_rpm = Some(rpm);
        set_shift_lights_calibrated(true);
        self.reset_calibration();
    }

    fn reset_calibration(&mut self) {
        self.calibrate_frames = 0;
        self.calibrate_peak_hold = 0;
        self.calibrate_rpm_max = f32::MIN;
    }

    fn reset_for_new_car(&mut self) {
        self.fuel_cut_rpm = None;
        set_shift_lights_calibrated(false);
        set_electric_car(false);
        self.reset_calibration();
        self.rpm_history.clear();
        self.power_history.clear();
        self.torque_history.clear();
        self.current_gear = None;
    }
}

fn uncalibrated_timing(frames: usize, calibrate_frames_target: usize) -> (f32, f32, f32) {
    if frames == 0 {
        return (0.0, 0.0, 0.0);
    }
    let elapsed = frames as f32 / SAMPLE_HZ as f32;
    let duration =
        (calibrate_frames_target + CALIBRATE_PEAK_HOLD_FRAMES) as f32 / SAMPLE_HZ as f32;
    let progress = if duration > 0.0 {
        (elapsed / duration).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (progress, elapsed, duration)
}

fn sign_cross(samples: impl Iterator<Item = f32>) -> bool {
    let mut pos = false;
    let mut neg = false;
    for value in samples {
        pos |= value > 0.0;
        neg |= value < 0.0;
        if pos && neg {
            return true;
        }
    }
    false
}

fn rpm_chatter(samples: &[f32]) -> bool {
    if samples.len() < 3 {
        return false;
    }
    let mut rpm_min = f32::MAX;
    let mut rpm_max = f32::MIN;
    for &value in samples {
        rpm_min = rpm_min.min(value);
        rpm_max = rpm_max.max(value);
    }
    let range = rpm_max - rpm_min;
    let deadzone = RPM_REVERSAL_FLOOR.max(range * RPM_REVERSAL_RANGE_FRAC);

    let mut last_extreme = samples[0];
    let mut last_sign = 0i8;
    let mut reversals = 0;
    let mut path = 0.0;
    for pair in samples.windows(2) {
        path += (pair[1] - pair[0]).abs();
        let delta = pair[1] - last_extreme;
        let sign = if delta > deadzone {
            1
        } else if delta < -deadzone {
            -1
        } else {
            0
        };
        if sign != 0 {
            if last_sign != 0 && sign != last_sign {
                reversals += 1;
            }
            last_sign = sign;
            last_extreme = pair[1];
        }
    }
    let net = (samples[samples.len() - 1] - samples[0]).abs();
    reversals >= 2 && path > 0.0 && net * 2.0 <= path
}
