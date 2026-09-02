use gpui::{AppContext, Context, Entity, Window, colors::DefaultAppearance};

use super::colors::Colors;
use gpui_component::{
    Icon, IconName, Theme, ThemeMode,
    input::{InputEvent, InputState},
    slider::{SliderEvent, SliderState, SliderValue},
};

use crate::config::{
    DEFAULT_CALIBRATE_MS, DEFAULT_FORWARD_HOST, DEFAULT_FORWARD_PORT, DEFAULT_LISTEN_HOST,
    DEFAULT_LISTEN_PORT, DEFAULT_SHIFT_LIGHTS_GAP_PX, DEFAULT_SHIFT_LIGHTS_OFFSET_PX,
    DEFAULT_SHIFT_LIGHTS_THICKNESS_PX,
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
        let (gear_display_x, gear_display_y) = hud::gear_display_position_ratio();
        let gear_display_x_slider = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(100.0)
                .step(0.5)
                .default_value(gear_display_x * 100.0)
        });
        let gear_display_y_slider = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(100.0)
                .step(0.5)
                .default_value(gear_display_y * 100.0)
        });
        for slider in [&gear_display_x_slider, &gear_display_y_slider] {
            cx.subscribe(slider, |this, _slider, _event: &SliderEvent, cx| {
                let x = match this.gear_display_x_slider.read(cx).value() {
                    SliderValue::Single(value) => value,
                    SliderValue::Range(start, _) => start,
                };
                let y = match this.gear_display_y_slider.read(cx).value() {
                    SliderValue::Single(value) => value,
                    SliderValue::Range(start, _) => start,
                };
                let _ = hud::set_gear_display_position_ratio(x / 100.0, y / 100.0);
                cx.notify();
            })
            .detach();
        }
        let gear_display_size_input = cx.new(|cx| {
            InputState::new(window, cx).default_value(hud::gear_display_size_px().to_string())
        });
        let gear_display_lit_opacity_slider = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(1.0)
                .step(0.05)
                .default_value(hud::gear_display_lit_opacity())
        });
        cx.subscribe(
            &gear_display_lit_opacity_slider,
            |_this, _slider, event: &SliderEvent, cx| {
                if let SliderEvent::Change(SliderValue::Single(opacity)) = event {
                    let _ = hud::set_gear_display_lit_opacity(*opacity);
                    cx.notify();
                }
            },
        )
        .detach();
        let gear_display_dim_opacity_slider = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(1.0)
                .step(0.05)
                .default_value(hud::gear_display_dim_opacity())
        });
        cx.subscribe(
            &gear_display_dim_opacity_slider,
            |_this, _slider, event: &SliderEvent, cx| {
                if let SliderEvent::Change(SliderValue::Single(opacity)) = event {
                    let _ = hud::set_gear_display_dim_opacity(*opacity);
                    cx.notify();
                }
            },
        )
        .detach();
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
        subscribe_input(&shift_lights_offset_input, |this, _, cx| {
            this.apply_shift_lights_offset(cx);
        });
        subscribe_input(&shift_lights_thickness_input, |this, _, cx| {
            this.apply_shift_lights_thickness(cx);
        });
        subscribe_input(&shift_lights_gap_input, |this, _, cx| {
            this.apply_shift_lights_gap(cx);
        });
        subscribe_input(&gear_display_size_input, |this, _, cx| {
            this.apply_gear_display_size(cx);
        });
        Self {
            appearance_override: None,
            selected_section: initial_section,
            last_charts_visible: hud::charts_visible(),
            last_only_show_in_game: hud::only_show_in_game(),
            last_calibrate_hint_visible: hud::calibrate_hint_visible(),
            last_strict_calibrate_conditions: hud::strict_calibrate_conditions(),
            last_remember_calibrated_cars: hud::remember_calibrated_cars(),
            last_shift_lights_position: hud::shift_lights_position(),
            last_shift_lights_direction: hud::shift_lights_direction(),
            last_shift_lights_calibrated: hud::shift_lights_calibrated(),
            last_electric_car: hud::electric_car(),
            last_has_saved_calibration: hud::current_car_has_saved_calibration(),
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
            shift_lights_lit_opacity_slider,
            shift_lights_dim_opacity_slider,
            shift_lights_offset_input,
            shift_lights_thickness_input,
            shift_lights_gap_input,
            shift_lights_width_slider,
            shift_lights_blink_slider,
            gear_display_x_slider,
            gear_display_y_slider,
            gear_display_size_input,
            gear_display_lit_opacity_slider,
            gear_display_dim_opacity_slider,
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
        hud::set_force_hud_visible(Self::should_force_hud(
            self.selected_section,
            window.is_window_active(),
        ));
    }

    pub(super) fn sync_hud_settings(&mut self, cx: &mut Context<Self>) {
        let charts_visible = hud::charts_visible();
        let only_show_in_game = hud::only_show_in_game();
        let calibrate_hint_visible = hud::calibrate_hint_visible();
        let strict_calibrate_conditions = hud::strict_calibrate_conditions();
        let remember_calibrated_cars = hud::remember_calibrated_cars();
        let position = hud::shift_lights_position();
        let direction = hud::shift_lights_direction();
        let calibrated = hud::shift_lights_calibrated();
        let electric = hud::electric_car();
        let has_saved_calibration = hud::current_car_has_saved_calibration();
        if self.last_charts_visible != charts_visible
            || self.last_only_show_in_game != only_show_in_game
            || self.last_calibrate_hint_visible != calibrate_hint_visible
            || self.last_strict_calibrate_conditions != strict_calibrate_conditions
            || self.last_remember_calibrated_cars != remember_calibrated_cars
            || self.last_shift_lights_position != position
            || self.last_shift_lights_direction != direction
            || self.last_shift_lights_calibrated != calibrated
            || self.last_electric_car != electric
            || self.last_has_saved_calibration != has_saved_calibration
        {
            self.last_charts_visible = charts_visible;
            self.last_only_show_in_game = only_show_in_game;
            self.last_calibrate_hint_visible = calibrate_hint_visible;
            self.last_strict_calibrate_conditions = strict_calibrate_conditions;
            self.last_remember_calibrated_cars = remember_calibrated_cars;
            self.last_shift_lights_position = position;
            self.last_shift_lights_direction = direction;
            self.last_shift_lights_calibrated = calibrated;
            self.last_electric_car = electric;
            self.last_has_saved_calibration = has_saved_calibration;
            cx.notify();
        }
    }
}
