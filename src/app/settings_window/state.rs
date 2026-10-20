use gpui_kit::{AppContext, Context, Entity, Window, colors::DefaultAppearance};

use super::colors::Colors;
use gpui_kit::component::{
    Icon, IconName, Theme, ThemeMode,
    input::{InputEvent, InputState},
};

use crate::core::calibration;
use crate::core::general_config::{
    DEFAULT_CALIBRATE_MS, DEFAULT_FORWARD_HOST, DEFAULT_FORWARD_PORT, DEFAULT_LISTEN_HOST,
    DEFAULT_LISTEN_PORT,
};
use crate::core::telemetry;
use crate::session;

use super::{Settings, section::SettingsSection};

fn steam_launch_command() -> String {
    let path = std::env::current_exe()
        .ok()
        .map(|path| {
            let text = path.to_string_lossy();
            text.strip_prefix(r"\\?\").unwrap_or(&text).to_string()
        })
        .unwrap_or_default();
    format!("\"{path}\" --launch %command%")
}

impl Settings {
    pub(super) fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
        initial_section: SettingsSection,
    ) -> Self {
        cx.observe_window_appearance(window, |this, window, cx| {
            if this.appearance_override.is_none() {
                Theme::sync_system_appearance(Some(window), cx);
                cx.notify();
            }
        })
        .detach();
        Theme::sync_system_appearance(Some(window), cx);
        cx.observe_window_activation(window, |this, window, _cx| {
            this.sync_force_hud_visible(window);
        })
        .detach();
        session::set_force_hud_visible(Self::should_force_hud(
            initial_section,
            window.is_window_active(),
        ));
        cx.on_release(|_, _| {
            session::set_force_hud_visible(false);
        })
        .detach();
        let (host, port) = telemetry::listen_host_port();
        let host_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(DEFAULT_LISTEN_HOST)
                .default_value(host)
        });
        let port_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(DEFAULT_LISTEN_PORT.to_string())
                .default_value(port.to_string())
        });
        let (forward_host, forward_port) = telemetry::forward_host_port();
        let forward_host_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(DEFAULT_FORWARD_HOST)
                .default_value(forward_host)
        });
        let forward_port_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(DEFAULT_FORWARD_PORT.to_string())
                .default_value(forward_port.to_string())
        });
        let general = session::general();
        let calibrate_ms_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(DEFAULT_CALIBRATE_MS.to_string())
                .default_value(general.calibrate_ms.to_string())
        });
        let launch_command_input = cx.new(|cx| {
            InputState::new(window, cx).default_value(steam_launch_command())
        });
        let mut subscribe_input =
            |input: &Entity<InputState>,
             on_apply: fn(&mut Self, &mut Window, &mut Context<Self>)| {
                cx.subscribe_in(
                    input,
                    window,
                    move |this, _, event: &InputEvent, window, cx| match event {
                        InputEvent::Change => cx.notify(),
                        InputEvent::PressEnter { .. } => on_apply(this, window, cx),
                        _ => {}
                    },
                )
                .detach();
            };
        subscribe_input(&host_input, |this, window, cx| {
            this.apply_listen_addr(window, cx);
        });
        subscribe_input(&port_input, |this, window, cx| {
            this.apply_listen_addr(window, cx);
        });
        subscribe_input(&forward_host_input, |this, window, cx| {
            this.apply_forward_addr(window, cx);
        });
        subscribe_input(&forward_port_input, |this, window, cx| {
            this.apply_forward_addr(window, cx);
        });
        subscribe_input(&calibrate_ms_input, |this, _, cx| {
            this.apply_calibrate_ms(cx);
        });
        let shift_lights =
            crate::components::shift_lights::ShiftLightsControls::new(
                &session::shift_lights(),
                window,
                cx,
            );
        let gear_display = crate::components::gear_display::GearDisplayControls::new(
            &session::gear_display(),
            window,
            cx,
        );
        Self {
            root_focus: cx.focus_handle().tab_stop(false),
            appearance_override: None,
            selected_section: initial_section,
            last_charts_visible: general.charts_visible,
            last_only_show_in_game: general.only_show_in_game,
            last_calibrate_hint_visible: general.calibrate_hint_visible,
            last_strict_calibrate_conditions: general.strict_calibrate_conditions,
            last_remember_calibrated_cars: general.remember_calibrated_cars,
            last_shift_lights_calibrated: calibration::shift_lights_calibrated(),
            last_electric_car: calibration::electric_car(),
            last_has_saved_calibration: session::current_car_has_saved_calibration(),
            config_reset_hovered: false,
            config_open_hovered: false,
            reset_config_confirming: false,
            listen_addr_warning: None,
            host_input,
            port_input,
            forward_addr_warning: None,
            forward_host_input,
            forward_port_input,
            calibrate_ms_input,
            launch_command_input,
            shift_lights,
            gear_display,
        }
    }

    pub(super) fn effective_appearance(&self, window: &Window) -> DefaultAppearance {
        self.appearance_override
            .unwrap_or_else(|| window.appearance().into())
    }

    pub(super) fn colors(&self, window: &Window) -> Colors {
        if let Some(appearance) = self.appearance_override {
            match appearance {
                DefaultAppearance::Light => Colors::light(),
                DefaultAppearance::Dark => Colors::dark(),
            }
        } else {
            Colors::for_appearance(window)
        }
    }

    pub(super) fn theme_toggle_icon(&self, window: &Window) -> Icon {
        let icon = match self.effective_appearance(window) {
            DefaultAppearance::Light => IconName::Moon,
            DefaultAppearance::Dark => IconName::Sun,
        };
        Icon::new(icon)
    }

    pub(super) fn sync_component_theme(&self, window: &mut Window, cx: &mut Context<Self>) {
        let mode = match self.effective_appearance(window) {
            DefaultAppearance::Light => ThemeMode::Light,
            DefaultAppearance::Dark => ThemeMode::Dark,
        };
        Theme::change(mode, Some(window), cx);
    }

    pub(super) fn toggle_appearance(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let next = match self.effective_appearance(window) {
            DefaultAppearance::Light => DefaultAppearance::Dark,
            DefaultAppearance::Dark => DefaultAppearance::Light,
        };
        self.appearance_override = Some(next);
        self.sync_component_theme(window, cx);
        cx.notify();
    }

    pub(super) fn select_section(
        &mut self,
        section: SettingsSection,
        window: &Window,
        cx: &mut Context<Self>,
    ) {
        if self.selected_section != section {
            if self.selected_section == SettingsSection::Overview {
                self.config_reset_hovered = false;
                self.config_open_hovered = false;
                self.reset_config_confirming = false;
            }
            self.selected_section = section;
            self.sync_force_hud_visible(window);
            cx.notify();
        }
    }

    fn should_force_hud(section: SettingsSection, window_active: bool) -> bool {
        window_active
            && matches!(
                section,
                SettingsSection::Hud | SettingsSection::ShiftLights | SettingsSection::GearDisplay
            )
    }

    fn sync_force_hud_visible(&self, window: &Window) {
        session::set_force_hud_visible(Self::should_force_hud(
            self.selected_section,
            window.is_window_active(),
        ));
    }

    pub(super) fn sync_hud_settings(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        let general = session::general();
        let calibrated = calibration::shift_lights_calibrated();
        let electric = calibration::electric_car();
        let has_saved_calibration = session::current_car_has_saved_calibration();
        if self.last_charts_visible != general.charts_visible
            || self.last_only_show_in_game != general.only_show_in_game
            || self.last_calibrate_hint_visible != general.calibrate_hint_visible
            || self.last_strict_calibrate_conditions != general.strict_calibrate_conditions
            || self.last_remember_calibrated_cars != general.remember_calibrated_cars
            || self.last_shift_lights_calibrated != calibrated
            || self.last_electric_car != electric
            || self.last_has_saved_calibration != has_saved_calibration
        {
            self.last_charts_visible = general.charts_visible;
            self.last_only_show_in_game = general.only_show_in_game;
            self.last_calibrate_hint_visible = general.calibrate_hint_visible;
            self.last_strict_calibrate_conditions = general.strict_calibrate_conditions;
            self.last_remember_calibrated_cars = general.remember_calibrated_cars;
            self.last_shift_lights_calibrated = calibrated;
            self.last_electric_car = electric;
            self.last_has_saved_calibration = has_saved_calibration;
            cx.notify();
        }
    }
}
