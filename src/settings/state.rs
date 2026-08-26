use gpui::{AppContext, Context, Window, colors::Colors, colors::DefaultAppearance};
use gpui_component::{
    Icon, IconName, Theme, ThemeMode,
    input::InputState,
    slider::{SliderEvent, SliderState, SliderValue},
};

use crate::config::{
    DEFAULT_CALIBRATE_MS, DEFAULT_LISTEN_HOST, DEFAULT_LISTEN_PORT, DEFAULT_SHIFT_LIGHTS_GAP_PX,
    DEFAULT_SHIFT_LIGHTS_OFFSET_PX, DEFAULT_SHIFT_LIGHTS_THICKNESS_PX,
};
use crate::hud::{self};
use crate::telemetry;

use super::{Settings, SettingsSection};

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
        hud::set_force_hud_visible(Self::should_force_hud(
            initial_section,
            window.is_window_active(),
        ));
        cx.on_release(|_, _| {
            hud::set_force_hud_visible(false);
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
        let calibrate_ms_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(DEFAULT_CALIBRATE_MS.to_string())
                .default_value(hud::calibrate_ms().to_string())
        });
        let shift_lights_lit_opacity_slider = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(1.0)
                .step(0.05)
                .default_value(hud::shift_lights_lit_opacity())
        });
        let shift_lights_dim_opacity_slider = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(1.0)
                .step(0.05)
                .default_value(hud::shift_lights_dim_opacity())
        });
        cx.subscribe(
            &shift_lights_lit_opacity_slider,
            |_this, _slider, event: &SliderEvent, cx| {
                if let SliderEvent::Change(SliderValue::Single(opacity)) = event {
                    let _ = hud::set_shift_lights_lit_opacity(*opacity);
                    cx.notify();
                }
            },
        )
        .detach();
        cx.subscribe(
            &shift_lights_dim_opacity_slider,
            |_this, _slider, event: &SliderEvent, cx| {
                if let SliderEvent::Change(SliderValue::Single(opacity)) = event {
                    let _ = hud::set_shift_lights_dim_opacity(*opacity);
                    cx.notify();
                }
            },
        )
        .detach();
        let shift_lights_offset_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(DEFAULT_SHIFT_LIGHTS_OFFSET_PX.to_string())
                .default_value(hud::shift_lights_offset_px().to_string())
        });
        let shift_lights_thickness_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(DEFAULT_SHIFT_LIGHTS_THICKNESS_PX.to_string())
                .default_value(hud::shift_lights_thickness_px().to_string())
        });
        let shift_lights_gap_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(DEFAULT_SHIFT_LIGHTS_GAP_PX.to_string())
                .default_value(hud::shift_lights_gap_px().to_string())
        });
        let shift_lights_width_slider = cx.new(|_| {
            SliderState::new()
                .min(1.0)
                .max(100.0)
                .step(1.0)
                .default_value(hud::shift_lights_width_percent() as f32)
        });
        cx.subscribe(
            &shift_lights_width_slider,
            |_this, _slider, event: &SliderEvent, cx| {
                if let SliderEvent::Change(SliderValue::Single(width)) = event {
                    hud::set_shift_lights_width_percent(*width as usize)
                        .expect("shift lights width slider value must be valid");
                    cx.notify();
                }
            },
        )
        .detach();
        let shift_lights_blink_slider = cx.new(|_| {
            SliderState::new()
                .min(crate::config::SHIFT_LIGHTS_BLINK_PERCENT_MIN)
                .max(crate::config::SHIFT_LIGHTS_BLINK_PERCENT_MAX)
                .step(0.1)
                .default_value(hud::shift_lights_blink_percent())
        });
        cx.subscribe(
            &shift_lights_blink_slider,
            |_this, _slider, event: &SliderEvent, cx| {
                if let SliderEvent::Change(SliderValue::Single(percent)) = event {
                    hud::set_shift_lights_blink_percent(*percent)
                        .expect("shift lights blink slider value must be valid");
                    cx.notify();
                }
            },
        )
        .detach();
        Self {
            appearance_override: None,
            selected_section: initial_section,
            last_charts_visible: hud::charts_visible(),
            last_only_show_in_game: hud::only_show_in_game(),
            last_calibrate_hint_visible: hud::calibrate_hint_visible(),
            last_shift_lights_position: hud::shift_lights_position(),
            last_shift_lights_direction: hud::shift_lights_direction(),
            last_shift_lights_calibrated: hud::shift_lights_calibrated(),
            host_input,
            port_input,
            calibrate_ms_input,
            shift_lights_lit_opacity_slider,
            shift_lights_dim_opacity_slider,
            shift_lights_offset_input,
            shift_lights_thickness_input,
            shift_lights_gap_input,
            shift_lights_width_slider,
            shift_lights_blink_slider,
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
            self.selected_section = section;
            self.sync_force_hud_visible(window);
            cx.notify();
        }
    }

    fn should_force_hud(section: SettingsSection, window_active: bool) -> bool {
        window_active && matches!(section, SettingsSection::Hud | SettingsSection::ShiftLights)
    }

    fn sync_force_hud_visible(&self, window: &Window) {
        hud::set_force_hud_visible(Self::should_force_hud(
            self.selected_section,
            window.is_window_active(),
        ));
    }

    pub(super) fn sync_hud_settings(&mut self, cx: &mut Context<Self>) {
        let charts_visible = hud::charts_visible();
        let only_show_in_game = hud::only_show_in_game();
        let calibrate_hint_visible = hud::calibrate_hint_visible();
        let position = hud::shift_lights_position();
        let direction = hud::shift_lights_direction();
        let calibrated = hud::shift_lights_calibrated();
        if self.last_charts_visible != charts_visible
            || self.last_only_show_in_game != only_show_in_game
            || self.last_calibrate_hint_visible != calibrate_hint_visible
            || self.last_shift_lights_position != position
            || self.last_shift_lights_direction != direction
            || self.last_shift_lights_calibrated != calibrated
        {
            self.last_charts_visible = charts_visible;
            self.last_only_show_in_game = only_show_in_game;
            self.last_calibrate_hint_visible = calibrate_hint_visible;
            self.last_shift_lights_position = position;
            self.last_shift_lights_direction = direction;
            self.last_shift_lights_calibrated = calibrated;
            cx.notify();
        }
    }
}
