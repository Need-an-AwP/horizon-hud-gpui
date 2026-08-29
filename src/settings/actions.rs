use gpui::{Context, Window};

use crate::config::{
    DEFAULT_CALIBRATE_MS, DEFAULT_LISTEN_HOST, DEFAULT_LISTEN_PORT,
    DEFAULT_SHIFT_LIGHTS_BLINK_PERCENT, DEFAULT_SHIFT_LIGHTS_DIM_OPACITY,
    DEFAULT_SHIFT_LIGHTS_GAP_PX, DEFAULT_SHIFT_LIGHTS_LIT_OPACITY, DEFAULT_SHIFT_LIGHTS_OFFSET_PX,
    DEFAULT_SHIFT_LIGHTS_THICKNESS_PX,
};
use crate::hud::{self, ShiftLightsDirection, ShiftLightsPosition};
use crate::telemetry;
use crate::user_config;

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

    pub(super) fn set_gear_display_visible(&mut self, visible: bool, cx: &mut Context<Self>) {
        hud::set_gear_display_visible(visible);
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

    pub(super) fn apply_listen_addr(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let host = self.host_input.read(cx).value().to_string();
        let port = self.port_input.read(cx).value().to_string();
        match telemetry::parse_listen_addr(&host, &port) {
            Ok(addr) => match telemetry::apply_listen_addr(addr) {
                Ok(()) => {
                    let (host, port) = telemetry::listen_host_port();
                    self.host_input.update(cx, |input, cx| {
                        input.set_value(host, window, cx);
                    });
                    self.port_input.update(cx, |input, cx| {
                        input.set_value(port.to_string(), window, cx);
                    });
                    self.listen_addr_warning = None;
                }
                Err(err) => self.reset_listen_addr_invalid(window, cx, &err),
            },
            Err(err) => self.reset_listen_addr_invalid(window, cx, &err),
        }
        cx.notify();
    }

    fn listen_addr_warning_message(error: &str) -> &'static str {
        if error.contains("端口") {
            "端口无效，已恢复为默认值。"
        } else if error.contains("无法绑定") {
            "无法绑定该地址，已恢复为默认值。"
        } else {
            "监听地址无效，已恢复为默认值。"
        }
    }

    fn reset_listen_addr_invalid(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        error: &str,
    ) {
        let warning = Self::listen_addr_warning_message(error);
        self.reset_listen_addr(window, cx);
        self.listen_addr_warning = Some(warning);
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

    pub(super) fn apply_gear_display_size(&mut self, cx: &mut Context<Self>) {
        let value = self.gear_display_size_input.read(cx).value().to_string();
        if let Ok(size) = value.trim().parse::<usize>() {
            let _ = hud::set_gear_display_size_px(size);
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

    pub(super) fn arm_reset_config(&mut self, cx: &mut Context<Self>) {
        self.reset_config_confirming = true;
        cx.notify();
    }

    pub(super) fn cancel_reset_config(&mut self, cx: &mut Context<Self>) {
        self.reset_config_confirming = false;
        cx.notify();
    }

    pub(super) fn confirm_reset_config(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.reset_config_confirming = false;
        self.reset_all_config(window, cx);
    }

    fn reset_all_config(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        user_config::reset_to_defaults();
        self.sync_controls_from_runtime(window, cx);
        self.listen_addr_warning = None;
        cx.notify();
    }

    fn sync_controls_from_runtime(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let (host, port) = telemetry::listen_host_port();
        self.host_input.update(cx, |input, cx| {
            input.set_value(host, window, cx);
        });
        self.port_input.update(cx, |input, cx| {
            input.set_value(port.to_string(), window, cx);
        });
        self.calibrate_ms_input.update(cx, |input, cx| {
            input.set_value(hud::calibrate_ms().to_string(), window, cx);
        });
        self.shift_lights_offset_input.update(cx, |input, cx| {
            input.set_value(hud::shift_lights_offset_px().to_string(), window, cx);
        });
        self.shift_lights_thickness_input.update(cx, |input, cx| {
            input.set_value(hud::shift_lights_thickness_px().to_string(), window, cx);
        });
        self.shift_lights_gap_input.update(cx, |input, cx| {
            input.set_value(hud::shift_lights_gap_px().to_string(), window, cx);
        });
        self.gear_display_size_input.update(cx, |input, cx| {
            input.set_value(hud::gear_display_size_px().to_string(), window, cx);
        });
        self.shift_lights_lit_opacity_slider
            .update(cx, |slider, cx| {
                slider.set_value(hud::shift_lights_lit_opacity(), window, cx);
            });
        self.shift_lights_dim_opacity_slider
            .update(cx, |slider, cx| {
                slider.set_value(hud::shift_lights_dim_opacity(), window, cx);
            });
        self.shift_lights_width_slider.update(cx, |slider, cx| {
            slider.set_value(hud::shift_lights_width_percent() as f32, window, cx);
        });
        self.shift_lights_blink_slider.update(cx, |slider, cx| {
            slider.set_value(hud::shift_lights_blink_percent(), window, cx);
        });
        let (gear_x, gear_y) = hud::gear_display_position_ratio();
        self.gear_display_x_slider.update(cx, |slider, cx| {
            slider.set_value(gear_x * 100.0, window, cx);
        });
        self.gear_display_y_slider.update(cx, |slider, cx| {
            slider.set_value(gear_y * 100.0, window, cx);
        });
        self.gear_display_lit_opacity_slider
            .update(cx, |slider, cx| {
                slider.set_value(hud::gear_display_lit_opacity(), window, cx);
            });
        self.gear_display_dim_opacity_slider
            .update(cx, |slider, cx| {
                slider.set_value(hud::gear_display_dim_opacity(), window, cx);
            });
        self.last_charts_visible = hud::charts_visible();
        self.last_only_show_in_game = hud::only_show_in_game();
        self.last_calibrate_hint_visible = hud::calibrate_hint_visible();
        self.last_shift_lights_position = hud::shift_lights_position();
        self.last_shift_lights_direction = hud::shift_lights_direction();
    }
}
