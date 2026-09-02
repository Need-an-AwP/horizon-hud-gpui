use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, AtomicUsize, Ordering};
use std::time::Duration;

use gpui::{Context, Timer};
use serde::{Deserialize, Serialize};

use crate::config::{
    CALIBRATE_MS_MAX, CALIBRATE_MS_MIN, CALIBRATE_PEAK_HOLD_FRAMES, DEFAULT_CALIBRATE_MS,
    DEFAULT_GEAR_DISPLAY_DIM_OPACITY, DEFAULT_GEAR_DISPLAY_LIT_OPACITY,
    DEFAULT_GEAR_DISPLAY_SIZE_PX, DEFAULT_GEAR_DISPLAY_X_RATIO, DEFAULT_GEAR_DISPLAY_Y_RATIO,
    DEFAULT_SHIFT_LIGHTS_BLINK_PERCENT, DEFAULT_SHIFT_LIGHTS_DIM_OPACITY,
    DEFAULT_SHIFT_LIGHTS_GAP_PX, DEFAULT_SHIFT_LIGHTS_LIT_OPACITY, DEFAULT_SHIFT_LIGHTS_OFFSET_PX,
    DEFAULT_SHIFT_LIGHTS_THICKNESS_PX, DEFAULT_SHIFT_LIGHTS_WIDTH_PERCENT, FUEL_CUT_WINDOW,
    RPM_REVERSAL_FLOOR, RPM_REVERSAL_RANGE_FRAC, SAMPLE_HZ, SHIFT_LIGHTS_BLINK_PERCENT_MAX,
    SHIFT_LIGHTS_BLINK_PERCENT_MIN,
};
use crate::ring_buffer::RingBuffer;
use crate::telemetry::{CarId, TelemetrySample, spawn_udp_listener};

static SHOW_CHARTS: AtomicBool = AtomicBool::new(false);
static SHOW_CALIBRATE_HINT: AtomicBool = AtomicBool::new(false);
static SHOW_ONLY_IN_GAME: AtomicBool = AtomicBool::new(true);
static SHIFT_LIGHTS_POSITION: AtomicU8 = AtomicU8::new(ShiftLightsPosition::Top as u8);
static SHIFT_LIGHTS_DIRECTION: AtomicU8 = AtomicU8::new(0);
static SHIFT_LIGHTS_LIT_OPACITY: AtomicU32 =
    AtomicU32::new(DEFAULT_SHIFT_LIGHTS_LIT_OPACITY.to_bits());
static SHIFT_LIGHTS_DIM_OPACITY: AtomicU32 =
    AtomicU32::new(DEFAULT_SHIFT_LIGHTS_DIM_OPACITY.to_bits());
static SHIFT_LIGHTS_OFFSET_PX: AtomicUsize = AtomicUsize::new(DEFAULT_SHIFT_LIGHTS_OFFSET_PX);
static SHIFT_LIGHTS_THICKNESS_PX: AtomicUsize = AtomicUsize::new(DEFAULT_SHIFT_LIGHTS_THICKNESS_PX);
static SHIFT_LIGHTS_GAP_PX: AtomicUsize = AtomicUsize::new(DEFAULT_SHIFT_LIGHTS_GAP_PX);
static SHIFT_LIGHTS_WIDTH_PERCENT: AtomicUsize =
    AtomicUsize::new(DEFAULT_SHIFT_LIGHTS_WIDTH_PERCENT);
static SHIFT_LIGHTS_BLINK_PERCENT: AtomicU32 =
    AtomicU32::new(DEFAULT_SHIFT_LIGHTS_BLINK_PERCENT.to_bits());
static CALIBRATE_PROGRESS_DIRECTION: AtomicU8 = AtomicU8::new(2);
static CALIBRATE_MS: AtomicUsize = AtomicUsize::new(DEFAULT_CALIBRATE_MS);
static STRICT_CALIBRATE_CONDITIONS: AtomicBool = AtomicBool::new(false);
static REMEMBER_CALIBRATED_CARS: AtomicBool = AtomicBool::new(true);
static FORCE_HUD_VISIBLE: AtomicBool = AtomicBool::new(false);
static SHIFT_LIGHTS_CALIBRATED: AtomicBool = AtomicBool::new(false);
static ELECTRIC_CAR: AtomicBool = AtomicBool::new(false);
static RESET_CURRENT_CALIBRATION: AtomicBool = AtomicBool::new(false);
static CURRENT_CAR_ID: Mutex<Option<CarId>> = Mutex::new(None);
static GEAR_DISPLAY_X_RATIO: AtomicU32 = AtomicU32::new(DEFAULT_GEAR_DISPLAY_X_RATIO.to_bits());
static GEAR_DISPLAY_Y_RATIO: AtomicU32 = AtomicU32::new(DEFAULT_GEAR_DISPLAY_Y_RATIO.to_bits());
static GEAR_DISPLAY_SIZE_PX: AtomicUsize = AtomicUsize::new(DEFAULT_GEAR_DISPLAY_SIZE_PX);
static GEAR_DISPLAY_VISIBLE: AtomicBool = AtomicBool::new(true);
static GEAR_DISPLAY_LIT_OPACITY: AtomicU32 =
    AtomicU32::new(DEFAULT_GEAR_DISPLAY_LIT_OPACITY.to_bits());
static GEAR_DISPLAY_DIM_OPACITY: AtomicU32 =
    AtomicU32::new(DEFAULT_GEAR_DISPLAY_DIM_OPACITY.to_bits());

fn persist_if(changed: bool) {
    if changed {
        crate::user_config::persist();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum CalibrateProgressDirection {
    LeftToRight,
    RightToLeft,
    BottomToTop,
}

impl CalibrateProgressDirection {
    #[allow(dead_code)]
    fn to_u8(self) -> u8 {
        match self {
            Self::LeftToRight => 0,
            Self::RightToLeft => 1,
            Self::BottomToTop => 2,
        }
    }

    fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::RightToLeft,
            2 => Self::BottomToTop,
            _ => Self::LeftToRight,
        }
    }
}

pub(crate) fn charts_visible() -> bool {
    SHOW_CHARTS.load(Ordering::Relaxed)
}

pub(crate) fn set_charts_visible(visible: bool) {
    persist_if(SHOW_CHARTS.swap(visible, Ordering::Relaxed) != visible);
}

pub(crate) fn only_show_in_game() -> bool {
    SHOW_ONLY_IN_GAME.load(Ordering::Relaxed)
}

pub(crate) fn set_only_show_in_game(visible: bool) {
    persist_if(SHOW_ONLY_IN_GAME.swap(visible, Ordering::Relaxed) != visible);
}

pub(crate) fn calibrate_hint_visible() -> bool {
    SHOW_CALIBRATE_HINT.load(Ordering::Relaxed)
}

pub(crate) fn set_calibrate_hint_visible(visible: bool) {
    persist_if(SHOW_CALIBRATE_HINT.swap(visible, Ordering::Relaxed) != visible);
}

pub(crate) fn strict_calibrate_conditions() -> bool {
    STRICT_CALIBRATE_CONDITIONS.load(Ordering::Relaxed)
}

pub(crate) fn set_strict_calibrate_conditions(strict: bool) {
    persist_if(STRICT_CALIBRATE_CONDITIONS.swap(strict, Ordering::Relaxed) != strict);
}

pub(crate) fn remember_calibrated_cars() -> bool {
    REMEMBER_CALIBRATED_CARS.load(Ordering::Relaxed)
}

pub(crate) fn set_remember_calibrated_cars(remember: bool) {
    persist_if(REMEMBER_CALIBRATED_CARS.swap(remember, Ordering::Relaxed) != remember);
}

pub(crate) fn force_hud_visible() -> bool {
    FORCE_HUD_VISIBLE.load(Ordering::Relaxed)
}

pub(crate) fn set_force_hud_visible(force: bool) {
    FORCE_HUD_VISIBLE.store(force, Ordering::Relaxed);
}

pub(crate) fn shift_lights_calibrated() -> bool {
    SHIFT_LIGHTS_CALIBRATED.load(Ordering::Relaxed)
}

fn set_shift_lights_calibrated(calibrated: bool) {
    SHIFT_LIGHTS_CALIBRATED.store(calibrated, Ordering::Relaxed);
}

fn current_car_id() -> Option<CarId> {
    *CURRENT_CAR_ID.lock().unwrap_or_else(|err| err.into_inner())
}

fn set_current_car_id(id: Option<CarId>) {
    *CURRENT_CAR_ID.lock().unwrap_or_else(|err| err.into_inner()) = id;
}

pub(crate) fn current_car_has_saved_calibration() -> bool {
    current_car_id()
        .and_then(crate::user_config::fuel_cut_rpm_for)
        .is_some()
}

pub(crate) fn reset_current_car_calibration() {
    if let Some(id) = current_car_id() {
        crate::user_config::remove_calibrated_car(id);
    }
    set_shift_lights_calibrated(false);
    RESET_CURRENT_CALIBRATION.store(true, Ordering::Relaxed);
}

fn take_reset_current_calibration() -> bool {
    RESET_CURRENT_CALIBRATION.swap(false, Ordering::Relaxed)
}

pub(crate) fn electric_car() -> bool {
    ELECTRIC_CAR.load(Ordering::Relaxed)
}

fn set_electric_car(electric: bool) {
    ELECTRIC_CAR.store(electric, Ordering::Relaxed);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ShiftLightsPosition {
    Bottom,
    Right,
    Top,
    Left,
}

impl ShiftLightsPosition {
    fn to_u8(self) -> u8 {
        match self {
            Self::Bottom => 0,
            Self::Right => 1,
            Self::Top => 2,
            Self::Left => 3,
        }
    }

    fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::Right,
            2 => Self::Top,
            3 => Self::Left,
            _ => Self::Bottom,
        }
    }

    pub(crate) fn is_vertical(self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ShiftLightsDirection {
    LeftToRight,
    RightToLeft,
    BottomToTop,
    TopToBottom,
}

impl ShiftLightsDirection {
    fn to_u8(self) -> u8 {
        match self {
            Self::LeftToRight => 0,
            Self::RightToLeft => 1,
            Self::BottomToTop => 2,
            Self::TopToBottom => 3,
        }
    }

    fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::RightToLeft,
            2 => Self::BottomToTop,
            3 => Self::TopToBottom,
            _ => Self::LeftToRight,
        }
    }

    pub(crate) fn default_for(position: ShiftLightsPosition) -> Self {
        if position.is_vertical() {
            Self::BottomToTop
        } else {
            Self::LeftToRight
        }
    }

    pub(crate) fn is_compatible_with(self, position: ShiftLightsPosition) -> bool {
        matches!(
            (position.is_vertical(), self),
            (false, Self::LeftToRight | Self::RightToLeft)
                | (true, Self::BottomToTop | Self::TopToBottom)
        )
    }

    pub(crate) fn reverses_visual_order(self) -> bool {
        matches!(self, Self::RightToLeft | Self::BottomToTop)
    }
}

pub(crate) fn shift_lights_position() -> ShiftLightsPosition {
    ShiftLightsPosition::from_u8(SHIFT_LIGHTS_POSITION.load(Ordering::Relaxed))
}

pub(crate) fn set_shift_lights_position(position: ShiftLightsPosition) {
    let changed =
        SHIFT_LIGHTS_POSITION.swap(position.to_u8(), Ordering::Relaxed) != position.to_u8();
    set_shift_lights_direction(ShiftLightsDirection::default_for(position));
    persist_if(changed);
}

pub(crate) fn shift_lights_direction() -> ShiftLightsDirection {
    ShiftLightsDirection::from_u8(SHIFT_LIGHTS_DIRECTION.load(Ordering::Relaxed))
}

pub(crate) fn set_shift_lights_direction(direction: ShiftLightsDirection) {
    if direction.is_compatible_with(shift_lights_position()) {
        persist_if(
            SHIFT_LIGHTS_DIRECTION.swap(direction.to_u8(), Ordering::Relaxed) != direction.to_u8(),
        );
    }
}

pub(crate) fn shift_lights_lit_opacity() -> f32 {
    f32::from_bits(SHIFT_LIGHTS_LIT_OPACITY.load(Ordering::Relaxed))
}

pub(crate) fn shift_lights_dim_opacity() -> f32 {
    f32::from_bits(SHIFT_LIGHTS_DIM_OPACITY.load(Ordering::Relaxed))
}

pub(crate) fn set_shift_lights_lit_opacity(opacity: f32) -> Result<f32, String> {
    set_shift_lights_opacity(&SHIFT_LIGHTS_LIT_OPACITY, opacity)
}

pub(crate) fn set_shift_lights_dim_opacity(opacity: f32) -> Result<f32, String> {
    set_shift_lights_opacity(&SHIFT_LIGHTS_DIM_OPACITY, opacity)
}

pub(crate) fn shift_lights_offset_px() -> usize {
    SHIFT_LIGHTS_OFFSET_PX.load(Ordering::Relaxed)
}

pub(crate) fn set_shift_lights_offset_px(offset: usize) {
    persist_if(SHIFT_LIGHTS_OFFSET_PX.swap(offset, Ordering::Relaxed) != offset);
}

pub(crate) fn shift_lights_thickness_px() -> usize {
    SHIFT_LIGHTS_THICKNESS_PX.load(Ordering::Relaxed)
}

pub(crate) fn set_shift_lights_thickness_px(thickness: usize) -> Result<usize, String> {
    if thickness == 0 {
        return Err("灯条厚度必须是大于 0 的整数。".into());
    }
    persist_if(SHIFT_LIGHTS_THICKNESS_PX.swap(thickness, Ordering::Relaxed) != thickness);
    Ok(thickness)
}

pub(crate) fn shift_lights_gap_px() -> usize {
    SHIFT_LIGHTS_GAP_PX.load(Ordering::Relaxed)
}

pub(crate) fn set_shift_lights_gap_px(gap: usize) {
    persist_if(SHIFT_LIGHTS_GAP_PX.swap(gap, Ordering::Relaxed) != gap);
}

pub(crate) fn shift_lights_width_percent() -> usize {
    SHIFT_LIGHTS_WIDTH_PERCENT.load(Ordering::Relaxed)
}

pub(crate) fn set_shift_lights_width_percent(width: usize) -> Result<usize, String> {
    if !(1..=100).contains(&width) {
        return Err("整体宽度必须是 1 到 100 之间的整数百分比。".into());
    }
    persist_if(SHIFT_LIGHTS_WIDTH_PERCENT.swap(width, Ordering::Relaxed) != width);
    Ok(width)
}

pub(crate) fn shift_lights_blink_percent() -> f32 {
    f32::from_bits(SHIFT_LIGHTS_BLINK_PERCENT.load(Ordering::Relaxed))
}

pub(crate) fn set_shift_lights_blink_percent(percent: f32) -> Result<f32, String> {
    if !percent.is_finite()
        || !(SHIFT_LIGHTS_BLINK_PERCENT_MIN..=SHIFT_LIGHTS_BLINK_PERCENT_MAX).contains(&percent)
    {
        return Err(format!(
            "闪烁阈值必须是 {SHIFT_LIGHTS_BLINK_PERCENT_MIN:.0} 到 {SHIFT_LIGHTS_BLINK_PERCENT_MAX:.0} 之间的百分比。"
        ));
    }
    persist_if(
        SHIFT_LIGHTS_BLINK_PERCENT.swap(percent.to_bits(), Ordering::Relaxed) != percent.to_bits(),
    );
    Ok(percent)
}

fn set_shift_lights_opacity(target: &AtomicU32, opacity: f32) -> Result<f32, String> {
    if !opacity.is_finite() || !(0.0..=1.0).contains(&opacity) {
        return Err("透明度必须是 0 到 1 之间的数值。".into());
    }
    persist_if(target.swap(opacity.to_bits(), Ordering::Relaxed) != opacity.to_bits());
    Ok(opacity)
}

pub(crate) fn gear_display_position_ratio() -> (f32, f32) {
    (
        f32::from_bits(GEAR_DISPLAY_X_RATIO.load(Ordering::Relaxed)),
        f32::from_bits(GEAR_DISPLAY_Y_RATIO.load(Ordering::Relaxed)),
    )
}

pub(crate) fn set_gear_display_position_ratio(x: f32, y: f32) -> Result<(), String> {
    if !x.is_finite() || !y.is_finite() || !(0.0..=1.0).contains(&x) || !(0.0..=1.0).contains(&y) {
        return Err("挡位显示位置必须是 0 到 1 之间的比例。".into());
    }
    let x_changed = GEAR_DISPLAY_X_RATIO.swap(x.to_bits(), Ordering::Relaxed) != x.to_bits();
    let y_changed = GEAR_DISPLAY_Y_RATIO.swap(y.to_bits(), Ordering::Relaxed) != y.to_bits();
    persist_if(x_changed || y_changed);
    Ok(())
}

pub(crate) fn gear_display_size_px() -> usize {
    GEAR_DISPLAY_SIZE_PX.load(Ordering::Relaxed)
}

pub(crate) fn set_gear_display_size_px(size: usize) -> Result<usize, String> {
    if size == 0 {
        return Err("挡位显示大小必须是大于 0 的整数。".into());
    }
    persist_if(GEAR_DISPLAY_SIZE_PX.swap(size, Ordering::Relaxed) != size);
    Ok(size)
}

pub(crate) fn gear_display_visible() -> bool {
    GEAR_DISPLAY_VISIBLE.load(Ordering::Relaxed)
}

pub(crate) fn set_gear_display_visible(visible: bool) {
    persist_if(GEAR_DISPLAY_VISIBLE.swap(visible, Ordering::Relaxed) != visible);
}

pub(crate) fn gear_display_lit_opacity() -> f32 {
    f32::from_bits(GEAR_DISPLAY_LIT_OPACITY.load(Ordering::Relaxed))
}

pub(crate) fn set_gear_display_lit_opacity(opacity: f32) -> Result<f32, String> {
    set_shift_lights_opacity(&GEAR_DISPLAY_LIT_OPACITY, opacity)
}

pub(crate) fn gear_display_dim_opacity() -> f32 {
    f32::from_bits(GEAR_DISPLAY_DIM_OPACITY.load(Ordering::Relaxed))
}

pub(crate) fn set_gear_display_dim_opacity(opacity: f32) -> Result<f32, String> {
    set_shift_lights_opacity(&GEAR_DISPLAY_DIM_OPACITY, opacity)
}

pub(crate) fn calibrate_progress_direction() -> CalibrateProgressDirection {
    CalibrateProgressDirection::from_u8(CALIBRATE_PROGRESS_DIRECTION.load(Ordering::Relaxed))
}

#[allow(dead_code)]
pub(crate) fn set_calibrate_progress_direction(direction: CalibrateProgressDirection) {
    CALIBRATE_PROGRESS_DIRECTION.store(direction.to_u8(), Ordering::Relaxed);
}

pub(crate) fn calibrate_ms() -> usize {
    CALIBRATE_MS.load(Ordering::Relaxed)
}

pub(crate) fn calibrate_frames() -> usize {
    (SAMPLE_HZ * calibrate_ms() / 1000).max(1)
}

pub(crate) fn set_calibrate_ms(ms: usize) -> Result<usize, String> {
    if !(CALIBRATE_MS_MIN..=CALIBRATE_MS_MAX).contains(&ms) {
        return Err(format!(
            "校准时长需在 {CALIBRATE_MS_MIN}–{CALIBRATE_MS_MAX} 毫秒之间。"
        ));
    }
    persist_if(CALIBRATE_MS.swap(ms, Ordering::Relaxed) != ms);
    Ok(ms)
}

pub(crate) struct RpmHud {
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

impl RpmHud {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        start_sample_pump(cx);
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

    pub(crate) fn has_data(&self) -> bool {
        self.has_data
    }

    pub(crate) fn rpm(&self) -> f32 {
        self.rpm
    }

    pub(crate) fn max_rpm(&self) -> f32 {
        self.max_rpm
    }

    pub(crate) fn torque(&self) -> f32 {
        self.torque
    }

    pub(crate) fn power(&self) -> f32 {
        self.power
    }

    pub(crate) fn gear_display(&self) -> String {
        match self.current_gear {
            None => "--".into(),
            Some(0) => "r".into(),
            Some(gear) if gear >= 11 => "n".into(),
            Some(gear) => gear.to_string(),
        }
    }

    pub(crate) fn gear_display_position_ratio(&self) -> (f32, f32) {
        gear_display_position_ratio()
    }

    pub(crate) fn gear_display_size_px(&self) -> usize {
        gear_display_size_px()
    }

    pub(crate) fn gear_display_visible(&self) -> bool {
        gear_display_visible()
    }

    pub(crate) fn gear_display_lit_opacity(&self) -> f32 {
        gear_display_lit_opacity()
    }

    pub(crate) fn gear_display_dim_opacity(&self) -> f32 {
        gear_display_dim_opacity()
    }

    pub(crate) fn fuel_cut_rpm(&self) -> Option<f32> {
        self.fuel_cut_rpm
    }

    pub(crate) fn charts_visible(&self) -> bool {
        charts_visible()
    }

    pub(crate) fn calibrate_hint_visible(&self) -> bool {
        calibrate_hint_visible()
    }

    pub(crate) fn strict_calibrate_conditions(&self) -> bool {
        strict_calibrate_conditions()
    }

    pub(crate) fn shift_lights_position(&self) -> ShiftLightsPosition {
        shift_lights_position()
    }

    pub(crate) fn shift_lights_direction(&self) -> ShiftLightsDirection {
        shift_lights_direction()
    }

    pub(crate) fn shift_lights_lit_opacity(&self) -> f32 {
        shift_lights_lit_opacity()
    }

    pub(crate) fn shift_lights_dim_opacity(&self) -> f32 {
        shift_lights_dim_opacity()
    }

    pub(crate) fn shift_lights_offset_px(&self) -> usize {
        shift_lights_offset_px()
    }

    pub(crate) fn shift_lights_thickness_px(&self) -> usize {
        shift_lights_thickness_px()
    }

    pub(crate) fn shift_lights_gap_px(&self) -> usize {
        shift_lights_gap_px()
    }

    pub(crate) fn shift_lights_width_percent(&self) -> usize {
        shift_lights_width_percent()
    }

    pub(crate) fn shift_lights_blink_percent(&self) -> f32 {
        shift_lights_blink_percent()
    }

    pub(crate) fn calibrate_progress_direction(&self) -> CalibrateProgressDirection {
        calibrate_progress_direction()
    }

    pub(crate) fn calibrate_progress(&self) -> Option<(f32, f32)> {
        if self.fuel_cut_rpm.is_some() || self.calibrate_frames == 0 {
            return None;
        }
        Some((
            self.calibrate_frames as f32 / SAMPLE_HZ as f32,
            (calibrate_frames() + CALIBRATE_PEAK_HOLD_FRAMES) as f32 / SAMPLE_HZ as f32,
        ))
    }

    pub(crate) fn rpm_samples(&self) -> Vec<f32> {
        self.rpm_history.samples()
    }

    pub(crate) fn torque_samples(&self) -> Vec<f32> {
        self.torque_history.samples()
    }

    pub(crate) fn power_samples(&self) -> Vec<f32> {
        self.power_history.samples()
    }

    fn apply(&mut self, sample: TelemetrySample) {
        if let Some(car_id) = sample.car_id() {
            if self.car_id != Some(car_id) {
                if self.car_id.is_some() {
                    self.reset_for_new_car();
                }
                self.car_id = Some(car_id);
                set_current_car_id(Some(car_id));
                self.restore_stored_calibration(car_id);
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
            self.calibrate_fuel_cut();
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

    fn calibrate_fuel_cut(&mut self) {
        if self.fuel_cut_rpm.is_some() {
            return;
        }
        let holding = if strict_calibrate_conditions() {
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
        if self.calibrate_frames < calibrate_frames()
            || self.calibrate_peak_hold < CALIBRATE_PEAK_HOLD_FRAMES
        {
            return;
        }

        self.fuel_cut_rpm = Some(self.calibrate_rpm_max);
        set_shift_lights_calibrated(true);
        if let Some(car_id) = self.car_id {
            crate::user_config::upsert_calibrated_car(car_id, self.calibrate_rpm_max);
        }
        self.reset_calibration();
    }

    fn restore_stored_calibration(&mut self, car_id: CarId) {
        if self.fuel_cut_rpm.is_some() {
            return;
        }
        let Some(rpm) = crate::user_config::fuel_cut_rpm_for(car_id) else {
            return;
        };
        self.fuel_cut_rpm = Some(rpm);
        set_shift_lights_calibrated(true);
        self.reset_calibration();
    }

    fn clear_runtime_calibration(&mut self) {
        self.fuel_cut_rpm = None;
        set_shift_lights_calibrated(false);
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

fn start_sample_pump(cx: &mut Context<RpmHud>) {
    let queue = spawn_udp_listener();
    cx.spawn(async move |this, cx| {
        let mut last_charts = charts_visible();
        let mut last_calibrate_hint_visible = calibrate_hint_visible();
        let mut last_strict_calibrate_conditions = strict_calibrate_conditions();
        let mut last_remember_calibrated_cars = remember_calibrated_cars();
        let mut last_shift_lights_position = shift_lights_position();
        let mut last_shift_lights_direction = shift_lights_direction();
        let mut last_shift_lights_lit_opacity = shift_lights_lit_opacity();
        let mut last_shift_lights_dim_opacity = shift_lights_dim_opacity();
        let mut last_shift_lights_offset_px = shift_lights_offset_px();
        let mut last_shift_lights_thickness_px = shift_lights_thickness_px();
        let mut last_shift_lights_gap_px = shift_lights_gap_px();
        let mut last_shift_lights_width_percent = shift_lights_width_percent();
        let mut last_shift_lights_blink_percent = shift_lights_blink_percent();
        let mut last_calibrate_direction = calibrate_progress_direction();
        let mut last_listen_generation = crate::telemetry::listen_generation();
        let mut last_calibrate_ms = calibrate_ms();
        let mut last_gear_display_position = gear_display_position_ratio();
        let mut last_gear_display_size = gear_display_size_px();
        let mut last_gear_display_visible = gear_display_visible();
        let mut last_gear_display_lit_opacity = gear_display_lit_opacity();
        let mut last_gear_display_dim_opacity = gear_display_dim_opacity();
        loop {
            let samples = {
                let mut queue = queue.lock().unwrap();
                std::mem::take(&mut *queue)
            };
            let reset_calibration = take_reset_current_calibration();
            let show_charts = charts_visible();
            let calibrate_hint = calibrate_hint_visible();
            let strict_calibrate = strict_calibrate_conditions();
            let remember_cars = remember_calibrated_cars();
            let lights_position = shift_lights_position();
            let lights_direction = shift_lights_direction();
            let lights_lit_opacity = shift_lights_lit_opacity();
            let lights_dim_opacity = shift_lights_dim_opacity();
            let lights_offset_px = shift_lights_offset_px();
            let lights_thickness_px = shift_lights_thickness_px();
            let lights_gap_px = shift_lights_gap_px();
            let lights_width_percent = shift_lights_width_percent();
            let lights_blink_percent = shift_lights_blink_percent();
            let calibrate_direction = calibrate_progress_direction();
            let listen_generation = crate::telemetry::listen_generation();
            let current_calibrate_ms = calibrate_ms();
            let gear_display_position = gear_display_position_ratio();
            let gear_display_size = gear_display_size_px();
            let gear_display_visible = gear_display_visible();
            let gear_display_lit_opacity = gear_display_lit_opacity();
            let gear_display_dim_opacity = gear_display_dim_opacity();
            let remember_just_enabled = remember_cars && !last_remember_calibrated_cars;
            if !samples.is_empty()
                || reset_calibration
                || show_charts != last_charts
                || calibrate_hint != last_calibrate_hint_visible
                || strict_calibrate != last_strict_calibrate_conditions
                || remember_cars != last_remember_calibrated_cars
                || lights_position != last_shift_lights_position
                || lights_direction != last_shift_lights_direction
                || lights_lit_opacity != last_shift_lights_lit_opacity
                || lights_dim_opacity != last_shift_lights_dim_opacity
                || lights_offset_px != last_shift_lights_offset_px
                || lights_thickness_px != last_shift_lights_thickness_px
                || lights_gap_px != last_shift_lights_gap_px
                || lights_width_percent != last_shift_lights_width_percent
                || lights_blink_percent != last_shift_lights_blink_percent
                || calibrate_direction != last_calibrate_direction
                || listen_generation != last_listen_generation
                || current_calibrate_ms != last_calibrate_ms
                || gear_display_position != last_gear_display_position
                || gear_display_size != last_gear_display_size
                || gear_display_visible != last_gear_display_visible
                || gear_display_lit_opacity != last_gear_display_lit_opacity
                || gear_display_dim_opacity != last_gear_display_dim_opacity
            {
                last_charts = show_charts;
                last_calibrate_hint_visible = calibrate_hint;
                last_strict_calibrate_conditions = strict_calibrate;
                last_remember_calibrated_cars = remember_cars;
                last_shift_lights_position = lights_position;
                last_shift_lights_direction = lights_direction;
                last_shift_lights_lit_opacity = lights_lit_opacity;
                last_shift_lights_dim_opacity = lights_dim_opacity;
                last_shift_lights_offset_px = lights_offset_px;
                last_shift_lights_thickness_px = lights_thickness_px;
                last_shift_lights_gap_px = lights_gap_px;
                last_shift_lights_width_percent = lights_width_percent;
                last_shift_lights_blink_percent = lights_blink_percent;
                last_calibrate_direction = calibrate_direction;
                last_listen_generation = listen_generation;
                last_calibrate_ms = current_calibrate_ms;
                last_gear_display_position = gear_display_position;
                last_gear_display_size = gear_display_size;
                last_gear_display_visible = gear_display_visible;
                last_gear_display_lit_opacity = gear_display_lit_opacity;
                last_gear_display_dim_opacity = gear_display_dim_opacity;
                this.update(cx, |this, cx| {
                    if reset_calibration {
                        this.clear_runtime_calibration();
                    }
                    if remember_just_enabled {
                        if let Some(car_id) = this.car_id {
                            this.restore_stored_calibration(car_id);
                        }
                    }
                    for sample in samples {
                        this.apply(sample);
                    }
                    cx.notify();
                })
                .ok();
            }
            Timer::after(Duration::from_secs_f64(1.0 / SAMPLE_HZ as f64)).await;
        }
    })
    .detach();
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
