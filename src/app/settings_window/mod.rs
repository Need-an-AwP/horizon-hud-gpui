use std::sync::atomic::{AtomicU8, Ordering};

use gpui::{App, Entity, colors::DefaultAppearance};
use gpui_component::input::InputState;

use section::SettingsSection;

const OPEN_IDLE: u8 = 0;
const OPEN_DEFAULT: u8 = 1;
const OPEN_SHIFT_LIGHTS: u8 = 2;

static OPEN_REQUEST: AtomicU8 = AtomicU8::new(OPEN_IDLE);

mod about;
mod actions;
pub(crate) mod colors;
mod controls;
mod hud_display;
mod layout;
mod overview;
mod pages;
mod section;
mod settings_window;
mod state;

pub(crate) struct Settings {
    appearance_override: Option<DefaultAppearance>,
    selected_section: SettingsSection,
    last_charts_visible: bool,
    last_only_show_in_game: bool,
    last_calibrate_hint_visible: bool,
    last_strict_calibrate_conditions: bool,
    last_remember_calibrated_cars: bool,
    last_shift_lights_calibrated: bool,
    last_electric_car: bool,
    last_has_saved_calibration: bool,
    config_reset_hovered: bool,
    config_open_hovered: bool,
    reset_config_confirming: bool,
    listen_addr_warning: Option<&'static str>,
    host_input: Entity<InputState>,
    port_input: Entity<InputState>,
    forward_addr_warning: Option<&'static str>,
    forward_host_input: Entity<InputState>,
    forward_port_input: Entity<InputState>,
    calibrate_ms_input: Entity<InputState>,
    pub(crate) shift_lights: crate::components::shift_lights::ShiftLightsControls,
    pub(crate) gear_display: crate::components::gear_display::GearDisplayControls,
}

pub(crate) fn request_open() {
    OPEN_REQUEST.store(OPEN_DEFAULT, Ordering::Relaxed);
}

pub(crate) fn request_open_shift_lights() {
    OPEN_REQUEST.store(OPEN_SHIFT_LIGHTS, Ordering::Relaxed);
}

pub(crate) fn poll_open(cx: &mut App) {
    match OPEN_REQUEST.swap(OPEN_IDLE, Ordering::Relaxed) {
        OPEN_SHIFT_LIGHTS => {
            settings_window::open(cx, Some(SettingsSection::ShiftLights));
        }
        OPEN_DEFAULT => settings_window::open(cx, None),
        _ => {}
    }
}

pub(crate) fn poll_sync(cx: &mut App) {
    settings_window::sync(cx);
}
