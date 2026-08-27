use std::sync::atomic::{AtomicU8, Ordering};

use gpui::{App, Entity, colors::DefaultAppearance};
use gpui_component::{input::InputState, slider::SliderState};

use crate::hud::{ShiftLightsDirection, ShiftLightsPosition};
use crate::settings_section::SettingsSection;

const OPEN_IDLE: u8 = 0;
const OPEN_DEFAULT: u8 = 1;
const OPEN_SHIFT_LIGHTS: u8 = 2;

static OPEN_REQUEST: AtomicU8 = AtomicU8::new(OPEN_IDLE);

mod about;
mod actions;
mod controls;
mod hud_display;
mod layout;
mod overview;
mod pages;
mod settings_window;
mod state;

pub(crate) struct Settings {
    appearance_override: Option<DefaultAppearance>,
    selected_section: SettingsSection,
    last_charts_visible: bool,
    last_only_show_in_game: bool,
    last_calibrate_hint_visible: bool,
    last_shift_lights_position: ShiftLightsPosition,
    last_shift_lights_direction: ShiftLightsDirection,
    last_shift_lights_calibrated: bool,
    host_input: Entity<InputState>,
    port_input: Entity<InputState>,
    calibrate_ms_input: Entity<InputState>,
    shift_lights_lit_opacity_slider: Entity<SliderState>,
    shift_lights_dim_opacity_slider: Entity<SliderState>,
    shift_lights_offset_input: Entity<InputState>,
    shift_lights_thickness_input: Entity<InputState>,
    shift_lights_gap_input: Entity<InputState>,
    shift_lights_width_slider: Entity<SliderState>,
    shift_lights_blink_slider: Entity<SliderState>,
    gear_display_x_slider: Entity<SliderState>,
    gear_display_y_slider: Entity<SliderState>,
    gear_display_size_input: Entity<InputState>,
    gear_display_lit_opacity_slider: Entity<SliderState>,
    gear_display_dim_opacity_slider: Entity<SliderState>,
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
