use gpui_kit::{
    AnyElement, Context, Entity, FontWeight, Window, div, prelude::*, px, relative,
};
use gpui_kit::component::{
    Sizable, Size,
    input::{Input, InputEvent, InputState},
    slider::{SliderEvent, SliderState, SliderValue},
};
use serde::{Deserialize, Serialize};

use crate::app::settings_window::Settings;
use crate::app::settings_window::colors::Colors;
use crate::components::shared::color::with_opacity;
use crate::components::shared::settings_ui::{
    input_apply_control, input_differs, setting_row_with_reset, slider_row,
};
use crate::core::props::{GearValue, HudTelemetryProps};
use crate::session;

const DEFAULT_SIZE_PX: usize = 96;
const DEFAULT_X_RATIO: f32 = 0.5;
const DEFAULT_Y_RATIO: f32 = 0.075;
const DEFAULT_OPACITY: f32 = 1.0;
const DEFAULT_WEIGHT: f32 = 700.0;
const WEIGHT_MIN: f32 = 300.0;
const WEIGHT_MAX: f32 = 700.0;
const FORWARD_COLOR: u32 = 0xffffff;
const REVERSE_COLOR: u32 = 0xffcc22;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NumericConfig {
    #[serde(default = "default_x_ratio")]
    pub x_ratio: f32,
    #[serde(default = "default_y_ratio")]
    pub y_ratio: f32,
    #[serde(default = "default_size_px")]
    pub size_px: usize,
    #[serde(default = "default_opacity")]
    pub opacity: f32,
    #[serde(default = "default_weight")]
    pub weight: f32,
}

fn default_x_ratio() -> f32 {
    DEFAULT_X_RATIO
}

fn default_y_ratio() -> f32 {
    DEFAULT_Y_RATIO
}

fn default_size_px() -> usize {
    DEFAULT_SIZE_PX
}

fn default_opacity() -> f32 {
    DEFAULT_OPACITY
}

fn default_weight() -> f32 {
    DEFAULT_WEIGHT
}

impl Default for NumericConfig {
    fn default() -> Self {
        Self {
            x_ratio: DEFAULT_X_RATIO,
            y_ratio: DEFAULT_Y_RATIO,
            size_px: DEFAULT_SIZE_PX,
            opacity: DEFAULT_OPACITY,
            weight: DEFAULT_WEIGHT,
        }
    }
}

pub struct SettingsControls {
    pub x: Entity<SliderState>,
    pub y: Entity<SliderState>,
    pub size: Entity<InputState>,
    pub opacity: Entity<SliderState>,
    pub weight: Entity<SliderState>,
}

impl SettingsControls {
    pub fn new(cfg: &NumericConfig, window: &mut Window, cx: &mut Context<Settings>) -> Self {
        let x = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(100.0)
                .step(0.5)
                .default_value(cfg.x_ratio * 100.0)
        });
        let y = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(100.0)
                .step(0.5)
                .default_value(cfg.y_ratio * 100.0)
        });
        let size = cx.new(|cx| InputState::new(window, cx).default_value(cfg.size_px.to_string()));
        let opacity = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(1.0)
                .step(0.05)
                .default_value(cfg.opacity)
        });
        let weight = cx.new(|_| {
            SliderState::new()
                .max(WEIGHT_MAX)
                .min(WEIGHT_MIN)
                .step(10.0)
                .default_value(cfg.weight)
        });

        for slider in [&x, &y] {
            cx.subscribe(slider, |this: &mut Settings, _, _: &SliderEvent, cx| {
                let x_percent = slider_percent(this.gear_display.numeric.x.read(cx).value());
                let y_percent = slider_percent(this.gear_display.numeric.y.read(cx).value());
                session::update_gear_display(|settings| {
                    settings.numeric.x_ratio = (x_percent / 100.0).clamp(0.0, 1.0);
                    settings.numeric.y_ratio = (y_percent / 100.0).clamp(0.0, 1.0);
                });
                cx.notify();
            })
            .detach();
        }
        cx.subscribe(&weight, |_this, _, event: &SliderEvent, cx| {
            if let SliderEvent::Change(SliderValue::Single(weight)) = event {
                let weight = weight.clamp(WEIGHT_MIN, WEIGHT_MAX);
                session::update_gear_display(|settings| {
                    settings.numeric.weight = weight;
                });
                cx.notify();
            }
        })
        .detach();
        cx.subscribe(&opacity, |_this, _, event: &SliderEvent, cx| {
            if let SliderEvent::Change(SliderValue::Single(opacity)) = event {
                let opacity = opacity.clamp(0.0, 1.0);
                session::update_gear_display(|settings| {
                    settings.numeric.opacity = opacity;
                });
                cx.notify();
            }
        })
        .detach();
        cx.subscribe_in(
            &size,
            window,
            |this, _, event: &InputEvent, _window, cx| match event {
                InputEvent::Change => cx.notify(),
                InputEvent::PressEnter { .. } => this.gear_display.numeric.apply_size(cx),
                _ => {}
            },
        )
        .detach();

        Self {
            x,
            y,
            size,
            opacity,
            weight,
        }
    }

    pub fn sync(&self, cfg: &NumericConfig, window: &mut Window, cx: &mut Context<Settings>) {
        self.x.update(cx, |slider, cx| {
            slider.set_value(cfg.x_ratio * 100.0, window, cx);
        });
        self.y.update(cx, |slider, cx| {
            slider.set_value(cfg.y_ratio * 100.0, window, cx);
        });
        self.size.update(cx, |input, cx| {
            input.set_value(cfg.size_px.to_string(), window, cx);
        });
        self.opacity.update(cx, |slider, cx| {
            slider.set_value(cfg.opacity, window, cx);
        });
        self.weight.update(cx, |slider, cx| {
            slider.set_value(cfg.weight, window, cx);
        });
    }

    pub fn apply_size(&self, cx: &mut Context<Settings>) {
        let value = self.size.read(cx).value().to_string();
        if let Ok(size) = value.trim().parse::<usize>()
            && size != 0
        {
            session::update_gear_display(|settings| {
                settings.numeric.size_px = size;
            });
        }
        cx.notify();
    }
}

pub fn render_hud(props: &HudTelemetryProps, cfg: &NumericConfig) -> AnyElement {
    let text = display_text(props.gear);
    let size = cfg.size_px as f32;
    let width = text.chars().count() as f32 * size * 0.6;
    let color = with_opacity(gear_color(props.gear), cfg.opacity);
    div()
        .absolute()
        .left(relative(cfg.x_ratio))
        .top(relative(cfg.y_ratio))
        .ml(px(-width / 2.0))
        .font_family("Bahnschrift")
        .font_weight(FontWeight(cfg.weight.clamp(WEIGHT_MIN, WEIGHT_MAX)))
        .text_size(px(size))
        .line_height(relative(1.0))
        .text_color(color)
        .child(text)
        .into_any_element()
}

pub fn render_settings(view: &Settings, colors: &Colors, cx: &mut Context<Settings>) -> AnyElement {
    let cfg = session::gear_display().numeric;
    let defaults = NumericConfig::default();
    let size_control = input_apply_control(
        Input::new(&view.gear_display.numeric.size)
            .w(px(96.))
            .with_size(Size::Medium),
        Some("px"),
        "gear-num-size-apply",
        input_differs(&view.gear_display.numeric.size, cfg.size_px, cx),
        colors,
        cx,
        |this, _, cx| this.gear_display.numeric.apply_size(cx),
    );

    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(slider_row(
            "水平位置",
            "数字中心相对屏幕宽度的位置；50% 时居中。",
            "gear-num-x-reset",
            &view.gear_display.numeric.x,
            colors,
            None,
            "%",
            1,
            cfg.x_ratio != defaults.x_ratio,
            cx,
            |this, window, cx| {
                reset_field(this, window, cx, |cfg| {
                    cfg.x_ratio = NumericConfig::default().x_ratio;
                });
            },
        ))
        .child(slider_row(
            "垂直位置",
            "相对屏幕高度的位置；默认 7.5%。",
            "gear-num-y-reset",
            &view.gear_display.numeric.y,
            colors,
            None,
            "%",
            1,
            cfg.y_ratio != defaults.y_ratio,
            cx,
            |this, window, cx| {
                reset_field(this, window, cx, |cfg| {
                    cfg.y_ratio = NumericConfig::default().y_ratio;
                });
            },
        ))
        .child(setting_row_with_reset(
            colors,
            "字体大小",
            "Bahnschrift 字号，单位像素；默认 96 px。",
            "gear-num-size-reset",
            size_control,
            cfg.size_px != defaults.size_px,
            None,
            cx,
            |this, window, cx| {
                let size_px = NumericConfig::default().size_px;
                this.gear_display.numeric.size.update(cx, |input, cx| {
                    input.set_value(size_px.to_string(), window, cx);
                });
                session::update_gear_display(|settings| {
                    settings.numeric.size_px = size_px;
                });
                cx.notify();
            },
        ))
        .child(slider_row(
            "字重",
            "沿 Bahnschrift 的可变字重轴，300 最细，700 最粗，步进 10；默认 700。",
            "gear-num-weight-reset",
            &view.gear_display.numeric.weight,
            colors,
            None,
            "",
            0,
            cfg.weight != defaults.weight,
            cx,
            |this, window, cx| {
                reset_field(this, window, cx, |cfg| {
                    cfg.weight = NumericConfig::default().weight;
                });
            },
        ))
        .child(slider_row(
            "透明度",
            "整个数字的不透明度，范围 0 到 1；默认 1。",
            "gear-num-opacity-reset",
            &view.gear_display.numeric.opacity,
            colors,
            None,
            "",
            2,
            cfg.opacity != defaults.opacity,
            cx,
            |this, window, cx| {
                reset_field(this, window, cx, |cfg| {
                    cfg.opacity = NumericConfig::default().opacity;
                });
            },
        ))
        .into_any_element()
}

fn reset_field(
    this: &mut Settings,
    window: &mut Window,
    cx: &mut Context<Settings>,
    update: impl FnOnce(&mut NumericConfig),
) {
    session::update_gear_display(|settings| update(&mut settings.numeric));
    let cfg = session::gear_display().numeric;
    this.gear_display.numeric.sync(&cfg, window, cx);
    cx.notify();
}

fn slider_percent(value: SliderValue) -> f32 {
    match value {
        SliderValue::Single(value) => value,
        SliderValue::Range(start, _) => start,
    }
}

fn display_text(gear: GearValue) -> String {
    match gear {
        GearValue::Disconnected => "-".into(),
        GearValue::Reverse => "R".into(),
        GearValue::Neutral => "N".into(),
        GearValue::Forward(gear) => gear.to_string(),
    }
}

fn gear_color(gear: GearValue) -> u32 {
    match gear {
        GearValue::Reverse => REVERSE_COLOR,
        GearValue::Disconnected | GearValue::Neutral | GearValue::Forward(_) => FORWARD_COLOR,
    }
}

