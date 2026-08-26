use gpui::{Context, Window};

use crate::config::{
    DEFAULT_CALIBRATE_MS, DEFAULT_LISTEN_HOST, DEFAULT_LISTEN_PORT,
    DEFAULT_SHIFT_LIGHTS_BLINK_PERCENT, DEFAULT_SHIFT_LIGHTS_DIM_OPACITY,
    DEFAULT_SHIFT_LIGHTS_GAP_PX, DEFAULT_SHIFT_LIGHTS_LIT_OPACITY, DEFAULT_SHIFT_LIGHTS_OFFSET_PX,
    DEFAULT_SHIFT_LIGHTS_THICKNESS_PX,
};
use crate::hud::{self, ShiftLightsDirection, ShiftLightsPosition};
use crate::telemetry;

use super::Settings;

impl Settings {
    pub(super) fn set_charts_visible(&mut self, visible: bool, cx: &mut Context<Self>) {
        hud::set_charts_visible(visible);
        self.last_charts_visible = visible;
        cx.notify();
    }

    pub(super) fn set_only_show_in_game(&mut self, visible: bool, cx: &mut Context<Self>) {
        hud::set_only_show_in_game(visible);
        self.last_only_show_in_game = visible;
        cx.notify();
    }

    pub(super) fn set_calibrate_hint_visible(&mut self, visible: bool, cx: &mut Context<Self>) {
        hud::set_calibrate_hint_visible(visible);
        self.last_calibrate_hint_visible = visible;
        cx.notify();
    }

    pub(super) fn set_shift_lights_position(
        &mut self,
        position: ShiftLightsPosition,
        cx: &mut Context<Self>,
    ) {
        hud::set_shift_lights_position(position);
        self.last_shift_lights_position = position;
        self.last_shift_lights_direction = hud::shift_lights_direction();
        cx.notify();
    }

    pub(super) fn set_shift_lights_direction(
        &mut self,
        direction: ShiftLightsDirection,
        cx: &mut Context<Self>,
    ) {
        hud::set_shift_lights_direction(direction);
        self.last_shift_lights_direction = hud::shift_lights_direction();
        cx.notify();
    }

    pub(super) fn apply_listen_addr(&mut self, cx: &mut Context<Self>) {
        let host = self.host_input.read(cx).value().to_string();
        let port = self.port_input.read(cx).value().to_string();
        if let Ok(addr) = telemetry::parse_listen_addr(&host, &port) {
            let _ = telemetry::apply_listen_addr(addr);
        }
        cx.notify();
    }

    pub(super) fn reset_listen_addr(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.host_input.update(cx, |input, cx| {
            input.set_value(DEFAULT_LISTEN_HOST, window, cx);
        });
        self.port_input.update(cx, |input, cx| {
            input.set_value(DEFAULT_LISTEN_PORT.to_string(), window, cx);
        });
        let _ = telemetry::apply_default_listen_addr();
        cx.notify();
    }

    pub(super) fn apply_calibrate_ms(&mut self, cx: &mut Context<Self>) {
        let value = self.calibrate_ms_input.read(cx).value().to_string();
        if let Ok(ms) = value.trim().parse::<usize>() {
            let _ = hud::set_calibrate_ms(ms);
        }
        cx.notify();
    }

    pub(super) fn reset_calibrate_ms(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.calibrate_ms_input.update(cx, |input, cx| {
            input.set_value(DEFAULT_CALIBRATE_MS.to_string(), window, cx);
        });
        let _ = hud::set_calibrate_ms(DEFAULT_CALIBRATE_MS);
        cx.notify();
    }

    pub(super) fn reset_shift_lights_lit_opacity(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.shift_lights_lit_opacity_slider
            .update(cx, |slider, cx| {
                slider.set_value(DEFAULT_SHIFT_LIGHTS_LIT_OPACITY, window, cx);
            });
        let _ = hud::set_shift_lights_lit_opacity(DEFAULT_SHIFT_LIGHTS_LIT_OPACITY);
        cx.notify();
    }

    pub(super) fn reset_shift_lights_dim_opacity(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.shift_lights_dim_opacity_slider
            .update(cx, |slider, cx| {
                slider.set_value(DEFAULT_SHIFT_LIGHTS_DIM_OPACITY, window, cx);
            });
        let _ = hud::set_shift_lights_dim_opacity(DEFAULT_SHIFT_LIGHTS_DIM_OPACITY);
        cx.notify();
    }

    pub(super) fn apply_shift_lights_offset(&mut self, cx: &mut Context<Self>) {
        let value = self.shift_lights_offset_input.read(cx).value().to_string();
        match value.trim().parse::<usize>() {
            Ok(offset) => hud::set_shift_lights_offset_px(offset),
            Err(_) => {}
        }
        cx.notify();
    }

    pub(super) fn reset_shift_lights_offset(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.shift_lights_offset_input.update(cx, |input, cx| {
            input.set_value(DEFAULT_SHIFT_LIGHTS_OFFSET_PX.to_string(), window, cx);
        });
        hud::set_shift_lights_offset_px(DEFAULT_SHIFT_LIGHTS_OFFSET_PX);
        cx.notify();
    }

    pub(super) fn apply_shift_lights_thickness(&mut self, cx: &mut Context<Self>) {
        let value = self
            .shift_lights_thickness_input
            .read(cx)
            .value()
            .to_string();
        if let Ok(thickness) = value.trim().parse::<usize>() {
            let _ = hud::set_shift_lights_thickness_px(thickness);
        }
        cx.notify();
    }

    pub(super) fn reset_shift_lights_thickness(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.shift_lights_thickness_input.update(cx, |input, cx| {
            input.set_value(DEFAULT_SHIFT_LIGHTS_THICKNESS_PX.to_string(), window, cx);
        });
        let _ = hud::set_shift_lights_thickness_px(DEFAULT_SHIFT_LIGHTS_THICKNESS_PX);
        cx.notify();
    }

    pub(super) fn apply_shift_lights_gap(&mut self, cx: &mut Context<Self>) {
        let value = self.shift_lights_gap_input.read(cx).value().to_string();
        match value.trim().parse::<usize>() {
            Ok(gap) => hud::set_shift_lights_gap_px(gap),
            Err(_) => {}
        }
        cx.notify();
    }

    pub(super) fn reset_shift_lights_gap(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.shift_lights_gap_input.update(cx, |input, cx| {
            input.set_value(DEFAULT_SHIFT_LIGHTS_GAP_PX.to_string(), window, cx);
        });
        hud::set_shift_lights_gap_px(DEFAULT_SHIFT_LIGHTS_GAP_PX);
        cx.notify();
    }

    pub(super) fn reset_shift_lights_width(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.shift_lights_width_slider.update(cx, |slider, cx| {
            slider.set_value(
                crate::config::DEFAULT_SHIFT_LIGHTS_WIDTH_PERCENT as f32,
                window,
                cx,
            );
        });
        cx.notify();
    }

    pub(super) fn reset_shift_lights_blink(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.shift_lights_blink_slider.update(cx, |slider, cx| {
            slider.set_value(DEFAULT_SHIFT_LIGHTS_BLINK_PERCENT, window, cx);
        });
        cx.notify();
    }
}
