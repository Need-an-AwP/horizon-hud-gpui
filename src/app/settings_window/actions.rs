use gpui_kit::{Context, Window};

use crate::core::calibration;
use crate::core::general_config::{
    CALIBRATE_MS_MAX, CALIBRATE_MS_MIN, DEFAULT_CALIBRATE_MS, DEFAULT_FORWARD_HOST,
    DEFAULT_FORWARD_PORT, DEFAULT_LISTEN_HOST, DEFAULT_LISTEN_PORT,
};
use crate::core::telemetry;
use crate::session;

use super::Settings;

impl Settings {
    pub(super) fn set_charts_visible(&mut self, visible: bool, cx: &mut Context<Self>) {
        session::update_general(|general| general.charts_visible = visible);
        self.last_charts_visible = visible;
        cx.notify();
    }

    pub(super) fn reset_charts_visible(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.set_charts_visible(false, cx);
    }

    pub(super) fn set_only_show_in_game(&mut self, visible: bool, cx: &mut Context<Self>) {
        session::update_general(|general| general.only_show_in_game = visible);
        self.last_only_show_in_game = visible;
        cx.notify();
    }

    pub(super) fn reset_only_show_in_game(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.set_only_show_in_game(true, cx);
    }

    pub(super) fn set_calibrate_hint_visible(&mut self, visible: bool, cx: &mut Context<Self>) {
        session::update_general(|general| general.calibrate_hint_visible = visible);
        self.last_calibrate_hint_visible = visible;
        cx.notify();
    }

    pub(super) fn reset_calibrate_hint_visible(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.set_calibrate_hint_visible(false, cx);
    }

    pub(super) fn set_strict_calibrate_conditions(&mut self, strict: bool, cx: &mut Context<Self>) {
        session::update_general(|general| general.strict_calibrate_conditions = strict);
        self.last_strict_calibrate_conditions = strict;
        cx.notify();
    }

    pub(super) fn reset_strict_calibrate_conditions(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.set_strict_calibrate_conditions(false, cx);
    }

    pub(super) fn set_remember_calibrated_cars(&mut self, remember: bool, cx: &mut Context<Self>) {
        session::update_general(|general| general.remember_calibrated_cars = remember);
        self.last_remember_calibrated_cars = remember;
        self.last_has_saved_calibration = session::current_car_has_saved_calibration();
        cx.notify();
    }

    pub(super) fn reset_remember_calibrated_cars(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.set_remember_calibrated_cars(true, cx);
    }

    pub(super) fn reset_current_car_calibration(&mut self, cx: &mut Context<Self>) {
        session::reset_current_car_calibration();
        self.last_shift_lights_calibrated = calibration::shift_lights_calibrated();
        self.last_has_saved_calibration = session::current_car_has_saved_calibration();
        cx.notify();
    }

    pub(super) fn apply_listen_addr(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let host = self.host_input.read(cx).value().to_string();
        let port = self.port_input.read(cx).value().to_string();
        match telemetry::parse_listen_addr(&host, &port) {
            Ok(addr) => match telemetry::apply_listen_addr(addr) {
                Ok(()) => {
                    let (host, port) = telemetry::listen_host_port();
                    session::update_general(|general| {
                        general.listen_host = host.clone();
                        general.listen_port = port;
                    });
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
        let (host, port) = telemetry::listen_host_port();
        session::update_general(|general| {
            general.listen_host = host;
            general.listen_port = port;
        });
        cx.notify();
    }

    pub(super) fn set_forward_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        session::update_general(|general| general.forward_enabled = enabled);
        cx.notify();
    }

    pub(super) fn apply_forward_addr(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let host = self.forward_host_input.read(cx).value().to_string();
        let port = self.forward_port_input.read(cx).value().to_string();
        match telemetry::parse_socket_addr(&host, &port) {
            Ok(addr) => match telemetry::apply_forward_addr(addr) {
                Ok(()) => {
                    let (host, port) = telemetry::forward_host_port();
                    session::update_general(|general| {
                        general.forward_host = host.clone();
                        general.forward_port = port;
                    });
                    self.forward_host_input.update(cx, |input, cx| {
                        input.set_value(host, window, cx);
                    });
                    self.forward_port_input.update(cx, |input, cx| {
                        input.set_value(port.to_string(), window, cx);
                    });
                    self.forward_addr_warning = None;
                }
                Err(err) => self.reset_forward_addr_invalid(window, cx, &err),
            },
            Err(err) => self.reset_forward_addr_invalid(window, cx, &err),
        }
        cx.notify();
    }

    fn forward_addr_warning_message(error: &str) -> &'static str {
        if error.contains("端口") {
            "端口无效，已恢复为默认值。"
        } else if error.contains("相同") {
            "转发地址不能与监听地址相同，已恢复为默认值。"
        } else {
            "转发地址无效，已恢复为默认值。"
        }
    }

    fn reset_forward_addr_invalid(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        error: &str,
    ) {
        let warning = Self::forward_addr_warning_message(error);
        self.reset_forward_addr(window, cx);
        self.forward_addr_warning = Some(warning);
    }

    pub(super) fn reset_forward_addr(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.forward_host_input.update(cx, |input, cx| {
            input.set_value(DEFAULT_FORWARD_HOST, window, cx);
        });
        self.forward_port_input.update(cx, |input, cx| {
            input.set_value(DEFAULT_FORWARD_PORT.to_string(), window, cx);
        });
        telemetry::apply_default_forward();
        session::update_general(|general| {
            general.forward_enabled = false;
            general.forward_host = DEFAULT_FORWARD_HOST.to_string();
            general.forward_port = DEFAULT_FORWARD_PORT;
        });
        cx.notify();
    }

    pub(super) fn apply_calibrate_ms(&mut self, cx: &mut Context<Self>) {
        let value = self.calibrate_ms_input.read(cx).value().to_string();
        if let Ok(ms) = value.trim().parse::<usize>()
            && (CALIBRATE_MS_MIN..=CALIBRATE_MS_MAX).contains(&ms)
        {
            session::update_general(|general| general.calibrate_ms = ms);
        }
        cx.notify();
    }

    pub(super) fn reset_calibrate_ms(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.calibrate_ms_input.update(cx, |input, cx| {
            input.set_value(DEFAULT_CALIBRATE_MS.to_string(), window, cx);
        });
        session::update_general(|general| general.calibrate_ms = DEFAULT_CALIBRATE_MS);
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
        session::reset_to_defaults();
        self.sync_controls_from_runtime(window, cx);
        self.shift_lights
            .sync(&session::shift_lights(), window, cx);
        self.gear_display
            .sync(&session::gear_display(), window, cx);
        self.listen_addr_warning = None;
        self.forward_addr_warning = None;
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
        let (forward_host, forward_port) = telemetry::forward_host_port();
        self.forward_host_input.update(cx, |input, cx| {
            input.set_value(forward_host, window, cx);
        });
        self.forward_port_input.update(cx, |input, cx| {
            input.set_value(forward_port.to_string(), window, cx);
        });
        let general = session::general();
        self.calibrate_ms_input.update(cx, |input, cx| {
            input.set_value(general.calibrate_ms.to_string(), window, cx);
        });
        self.last_charts_visible = general.charts_visible;
        self.last_only_show_in_game = general.only_show_in_game;
        self.last_calibrate_hint_visible = general.calibrate_hint_visible;
        self.last_strict_calibrate_conditions = general.strict_calibrate_conditions;
        self.last_remember_calibrated_cars = general.remember_calibrated_cars;
        self.last_shift_lights_calibrated = calibration::shift_lights_calibrated();
        self.last_electric_car = calibration::electric_car();
        self.last_has_saved_calibration = session::current_car_has_saved_calibration();
    }
}
