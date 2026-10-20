use gpui_kit::{
    AnyElement, Bounds, Context, Entity, PathBuilder, Pixels, Window, canvas, div, point,
    prelude::*, px, relative,
};
use gpui_kit::component::{
    input::{Input, InputEvent, InputState},
    Sizable, Size,
};
use serde::{Deserialize, Serialize};

use crate::app::settings_window::Settings;
use crate::app::settings_window::colors::Colors;
use crate::components::shared::color::with_opacity;
use crate::components::shared::number_field::{
    NumberField, NumberFieldState, opacity_as_percent, opacity_from_percent,
};
use crate::components::shared::settings_ui::{
    input_apply_control, input_differs, setting_row_with_reset, subscribe_number_field,
};
use crate::core::props::HudTelemetryProps;
use crate::session;

const SEGMENT_DIM_COLOR: u32 = 0xffffff;
const SEGMENT_LIT_COLOR: u32 = 0xff3333;
const REVERSE_SEGMENT_LIT_COLOR: u32 = 0xffcc22;
const SEGMENT_A: u8 = 1 << 0;
const SEGMENT_B: u8 = 1 << 1;
const SEGMENT_C: u8 = 1 << 2;
const SEGMENT_D: u8 = 1 << 3;
const SEGMENT_E: u8 = 1 << 4;
const SEGMENT_F: u8 = 1 << 5;
const SEGMENT_G: u8 = 1 << 6;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SevenSegmentConfig {
    pub x_ratio: f32,
    pub y_ratio: f32,
    pub size_px: usize,
    pub lit_opacity: f32,
    pub dim_opacity: f32,
}

impl Default for SevenSegmentConfig {
    fn default() -> Self {
        Self {
            x_ratio: 0.5,
            y_ratio: 0.075,
            size_px: 64,
            lit_opacity: 1.0,
            dim_opacity: 0.12,
        }
    }
}

pub struct SettingsControls {
    pub x: Entity<NumberFieldState>,
    pub y: Entity<NumberFieldState>,
    pub size: Entity<InputState>,
    pub lit_opacity: Entity<NumberFieldState>,
    pub dim_opacity: Entity<NumberFieldState>,
}

impl SettingsControls {
    pub fn new(cfg: &SevenSegmentConfig, window: &mut Window, cx: &mut Context<Settings>) -> Self {
        let x = cx.new(|cx| {
            NumberFieldState::builder()
                .min(0.0)
                .max(100.0)
                .step(0.5)
                .default_value(cfg.x_ratio * 100.0)
                .unit("%")
                .decimals(1)
                .build(window, cx)
        });
        let y = cx.new(|cx| {
            NumberFieldState::builder()
                .min(0.0)
                .max(100.0)
                .step(0.5)
                .default_value(cfg.y_ratio * 100.0)
                .unit("%")
                .decimals(1)
                .build(window, cx)
        });
        let size = cx.new(|cx| InputState::new(window, cx).default_value(cfg.size_px.to_string()));
        let lit_opacity = cx.new(|cx| {
            NumberFieldState::builder()
                .min(0.0)
                .max(100.0)
                .step(1.0)
                .default_value(opacity_as_percent(cfg.lit_opacity))
                .unit("%")
                .decimals(0)
                .build(window, cx)
        });
        let dim_opacity = cx.new(|cx| {
            NumberFieldState::builder()
                .min(0.0)
                .max(100.0)
                .step(1.0)
                .default_value(opacity_as_percent(cfg.dim_opacity))
                .unit("%")
                .decimals(0)
                .build(window, cx)
        });

        subscribe_number_field(&x, cx, |value| {
            session::update_gear_display(|settings| {
                settings.seven_segment.x_ratio = (value / 100.0).clamp(0.0, 1.0);
            });
        });
        subscribe_number_field(&y, cx, |value| {
            session::update_gear_display(|settings| {
                settings.seven_segment.y_ratio = (value / 100.0).clamp(0.0, 1.0);
            });
        });
        subscribe_number_field(&lit_opacity, cx, |value| {
            session::update_gear_display(|settings| {
                settings.seven_segment.lit_opacity = opacity_from_percent(value);
            });
        });
        subscribe_number_field(&dim_opacity, cx, |value| {
            session::update_gear_display(|settings| {
                settings.seven_segment.dim_opacity = opacity_from_percent(value);
            });
        });
        cx.subscribe_in(
            &size,
            window,
            |this, _, event: &InputEvent, _window, cx| match event {
                InputEvent::Change => cx.notify(),
                InputEvent::PressEnter { .. } => this.gear_display.seven_segment.apply_size(cx),
                _ => {}
            },
        )
        .detach();

        Self {
            x,
            y,
            size,
            lit_opacity,
            dim_opacity,
        }
    }

    pub fn sync(&self, cfg: &SevenSegmentConfig, window: &mut Window, cx: &mut Context<Settings>) {
        self.x.update(cx, |field, cx| {
            field.set_value(cfg.x_ratio * 100.0, window, cx);
        });
        self.y.update(cx, |field, cx| {
            field.set_value(cfg.y_ratio * 100.0, window, cx);
        });
        self.size.update(cx, |input, cx| {
            input.set_value(cfg.size_px.to_string(), window, cx);
        });
        self.lit_opacity.update(cx, |field, cx| {
            field.set_value(opacity_as_percent(cfg.lit_opacity), window, cx);
        });
        self.dim_opacity.update(cx, |field, cx| {
            field.set_value(opacity_as_percent(cfg.dim_opacity), window, cx);
        });
    }

    pub fn apply_size(&self, cx: &mut Context<Settings>) {
        let value = self.size.read(cx).value().to_string();
        if let Ok(size) = value.trim().parse::<usize>() {
            if size != 0 {
                session::update_gear_display(|settings| {
                    settings.seven_segment.size_px = size;
                });
            }
        }
        cx.notify();
    }
}

pub fn render_hud(props: &HudTelemetryProps, cfg: &SevenSegmentConfig) -> AnyElement {
    let value = props.gear.segment_text();
    let size = cfg.size_px as f32;
    let lit_opacity = cfg.lit_opacity;
    let dim_opacity = cfg.dim_opacity;
    let width = value.chars().count() as f32 * size * 0.62
        + value.chars().count().saturating_sub(1) as f32 * size * 0.12;
    div()
        .absolute()
        .left(relative(cfg.x_ratio))
        .top(relative(cfg.y_ratio))
        .ml(px(-width / 2.0))
        .flex()
        .gap(px(size * 0.12))
        .children(
            value.chars().map(move |character| {
                seven_segment_digit(character, size, lit_opacity, dim_opacity)
            }),
        )
        .into_any_element()
}

pub fn render_settings(view: &Settings, colors: &Colors, cx: &mut Context<Settings>) -> AnyElement {
    let cfg = session::gear_display().seven_segment;
    let defaults = SevenSegmentConfig::default();
    let opacity_warning = (cfg.lit_opacity < cfg.dim_opacity)
        .then_some("亮起透明度低于熄灭透明度，可能影响挡位辨认。");
    let size_control = input_apply_control(
        Input::new(&view.gear_display.seven_segment.size)
            .w(px(96.))
            .with_size(Size::Medium),
        Some("px"),
        "gear-seg-size-apply",
        input_differs(&view.gear_display.seven_segment.size, cfg.size_px, cx),
        colors,
        cx,
        |this, _, cx| this.gear_display.seven_segment.apply_size(cx),
    );
    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(setting_row_with_reset(
            colors,
            "水平位置",
            "百分比，范围 0–100，步进 0.5，默认 50。",
            "gear-seg-x-reset",
            NumberField::new(&view.gear_display.seven_segment.x).unit_color(colors.disabled),
            cfg.x_ratio != defaults.x_ratio,
            None,
            cx,
            |this, window, cx| {
                reset_field(this, window, cx, |segment| {
                    segment.x_ratio = SevenSegmentConfig::default().x_ratio;
                });
            },
        ))
        .child(setting_row_with_reset(
            colors,
            "垂直位置",
            "百分比，范围 0–100，步进 0.5，默认 7.5。",
            "gear-seg-y-reset",
            NumberField::new(&view.gear_display.seven_segment.y).unit_color(colors.disabled),
            cfg.y_ratio != defaults.y_ratio,
            None,
            cx,
            |this, window, cx| {
                reset_field(this, window, cx, |segment| {
                    segment.y_ratio = SevenSegmentConfig::default().y_ratio;
                });
            },
        ))
        .child(setting_row_with_reset(
            colors,
            "大小",
            "正整数，单位 px，默认 64。",
            "gear-seg-size-reset",
            size_control,
            cfg.size_px != defaults.size_px,
            None,
            cx,
            |this, window, cx| {
                reset_field(this, window, cx, |segment| {
                    segment.size_px = SevenSegmentConfig::default().size_px;
                });
            },
        ))
        .child(setting_row_with_reset(
            colors,
            "亮起透明度",
            "百分比，范围 0–100，步进 1，默认 100。",
            "gear-seg-lit-opacity-reset",
            NumberField::new(&view.gear_display.seven_segment.lit_opacity)
                .unit_color(colors.disabled),
            cfg.lit_opacity != defaults.lit_opacity,
            opacity_warning,
            cx,
            |this, window, cx| {
                reset_field(this, window, cx, |segment| {
                    segment.lit_opacity = SevenSegmentConfig::default().lit_opacity;
                });
            },
        ))
        .child(setting_row_with_reset(
            colors,
            "熄灭透明度",
            "百分比，范围 0–100，步进 1，默认 12。",
            "gear-seg-dim-opacity-reset",
            NumberField::new(&view.gear_display.seven_segment.dim_opacity)
                .unit_color(colors.disabled),
            cfg.dim_opacity != defaults.dim_opacity,
            opacity_warning,
            cx,
            |this, window, cx| {
                reset_field(this, window, cx, |segment| {
                    segment.dim_opacity = SevenSegmentConfig::default().dim_opacity;
                });
            },
        ))
        .child(
            div()
                .text_xs()
                .text_color(colors.disabled)
                .child("无遥测数据时显示 “-”。"),
        )
        .into_any_element()
}

fn reset_field(
    this: &mut Settings,
    window: &mut Window,
    cx: &mut Context<Settings>,
    update: impl FnOnce(&mut SevenSegmentConfig),
) {
    session::update_gear_display(|settings| update(&mut settings.seven_segment));
    let cfg = session::gear_display().seven_segment;
    this.gear_display.seven_segment.sync(&cfg, window, cx);
    cx.notify();
}

fn seven_segment_mask(character: char) -> u8 {
    match character {
        '0' => SEGMENT_A | SEGMENT_B | SEGMENT_C | SEGMENT_D | SEGMENT_E | SEGMENT_F,
        '1' => SEGMENT_B | SEGMENT_C,
        '2' => SEGMENT_A | SEGMENT_B | SEGMENT_D | SEGMENT_E | SEGMENT_G,
        '3' => SEGMENT_A | SEGMENT_B | SEGMENT_C | SEGMENT_D | SEGMENT_G,
        '4' => SEGMENT_B | SEGMENT_C | SEGMENT_F | SEGMENT_G,
        '5' => SEGMENT_A | SEGMENT_C | SEGMENT_D | SEGMENT_F | SEGMENT_G,
        '6' => SEGMENT_A | SEGMENT_C | SEGMENT_D | SEGMENT_E | SEGMENT_F | SEGMENT_G,
        '7' => SEGMENT_A | SEGMENT_B | SEGMENT_C,
        '8' => SEGMENT_A | SEGMENT_B | SEGMENT_C | SEGMENT_D | SEGMENT_E | SEGMENT_F | SEGMENT_G,
        '9' => SEGMENT_A | SEGMENT_B | SEGMENT_C | SEGMENT_D | SEGMENT_F | SEGMENT_G,
        'r' => SEGMENT_A | SEGMENT_E | SEGMENT_F,
        'n' => SEGMENT_C | SEGMENT_E | SEGMENT_G,
        '-' => SEGMENT_G,
        _ => 0,
    }
}

fn seven_segment_lit_color(character: char) -> u32 {
    if character == 'r' {
        REVERSE_SEGMENT_LIT_COLOR
    } else {
        SEGMENT_LIT_COLOR
    }
}

fn seven_segment_digit(
    character: char,
    size: f32,
    lit_opacity: f32,
    dim_opacity: f32,
) -> gpui_kit::Div {
    let width = size * 0.62;
    let height = size;
    let mask = seven_segment_mask(character);
    let lit_color = seven_segment_lit_color(character);

    div().relative().w(px(width)).h(px(height)).child(
        canvas(
            |_, _, _| {},
            move |bounds, _, window, _| {
                paint_seven_segment(bounds, mask, lit_color, lit_opacity, dim_opacity, window);
            },
        )
        .absolute()
        .inset_0(),
    )
}

/// 设计画布尺寸（宽 62 × 高 100），七段多边形坐标直接写死，绘制时仅整体缩放。
const DESIGN_WIDTH: f32 = 62.0;
const DESIGN_HEIGHT: f32 = 100.0;
/// 整体向右倾斜（tan 5°），模仿参考图的斜体数码管。
const SLANT: f32 = 0.0875;

/// 顺序与 `SEGMENT_A..SEGMENT_G` 位掩码一致：a b c d e f g。
const SEGMENT_POLYGONS: [[(f32, f32); 6]; 7] = [
    // a 上
    [(11.0, 7.5), (16.5, 2.0), (45.5, 2.0), (51.0, 7.5), (45.5, 13.0), (16.5, 13.0)],
    // b 右上
    [(51.5, 9.0), (57.0, 14.5), (57.0, 43.0), (51.5, 48.5), (46.0, 43.0), (46.0, 14.5)],
    // c 右下
    [(51.5, 51.5), (57.0, 57.0), (57.0, 85.5), (51.5, 91.0), (46.0, 85.5), (46.0, 57.0)],
    // d 下
    [(11.0, 92.5), (16.5, 87.0), (45.5, 87.0), (51.0, 92.5), (45.5, 98.0), (16.5, 98.0)],
    // e 左下
    [(10.5, 51.5), (16.0, 57.0), (16.0, 85.5), (10.5, 91.0), (5.0, 85.5), (5.0, 57.0)],
    // f 左上
    [(10.5, 9.0), (16.0, 14.5), (16.0, 43.0), (10.5, 48.5), (5.0, 43.0), (5.0, 14.5)],
    // g 中
    [(11.0, 50.0), (16.5, 44.5), (45.5, 44.5), (51.0, 50.0), (45.5, 55.5), (16.5, 55.5)],
];

fn paint_seven_segment(
    bounds: Bounds<Pixels>,
    mask: u8,
    lit_color: u32,
    lit_opacity: f32,
    dim_opacity: f32,
    window: &mut Window,
) {
    let scale_x = bounds.size.width / px(1.0) / DESIGN_WIDTH;
    let scale_y = bounds.size.height / px(1.0) / DESIGN_HEIGHT;
    if scale_x <= 0.0 || scale_y <= 0.0 {
        return;
    }

    for (index, polygon) in SEGMENT_POLYGONS.iter().enumerate() {
        let enabled = mask & (1u8 << index) != 0;
        let color = with_opacity(
            if enabled { lit_color } else { SEGMENT_DIM_COLOR },
            if enabled { lit_opacity } else { dim_opacity },
        );
        let points = polygon.map(|(x, y)| {
            point(
                bounds.origin.x + px((x + (DESIGN_HEIGHT / 2.0 - y) * SLANT) * scale_x),
                bounds.origin.y + px(y * scale_y),
            )
        });
        let mut builder = PathBuilder::fill();
        builder.add_polygon(&points, true);
        if let Ok(path) = builder.build() {
            window.paint_path(path, color);
        }
    }
}
