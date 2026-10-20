use gpui_kit::{div, prelude::*, px, relative, Context, Entity, IntoElement, Window};
use gpui_kit::component::{
    input::{Input, InputEvent, InputState},
    Sizable, Size,
};
use serde::{Deserialize, Serialize};

use crate::app::settings_window::colors::Colors;
use crate::app::settings_window::Settings;
use crate::components::shared::color::with_opacity;
use crate::components::shared::rpm_math::{
    calc_linear_lights, calibrate_segment_fill, is_blink_lit,
};
use crate::components::shared::number_field::{
    NumberField, NumberFieldState, opacity_as_percent, opacity_from_percent,
};
use crate::components::shared::settings_ui::{
    input_apply_control, input_differs, separator, setting_row_with_reset, settings_group,
    subscribe_number_field,
};
use crate::core::props::{CalibrationState, HudTelemetryProps};

const DEFAULT_Y_RATIO: f32 = 0.0;
const DEFAULT_THICKNESS_PX: usize = 90;
const DEFAULT_WIDTH_PERCENT: usize = 100;
const DEFAULT_LIT_OPACITY: f32 = 0.7;
const DEFAULT_DIM_OPACITY: f32 = 0.5;
const DEFAULT_BLINK_PERCENT: f32 = 95.0;
const BLINK_PERCENT_MIN: f32 = 80.0;
const BLINK_PERCENT_MAX: f32 = 100.0;
const DEFAULT_BLINK_MS: usize = 80;
const BLINK_MS_MIN: usize = 20;
const BLINK_MS_MAX: usize = 200;

const LIGHT_STEPS: usize = 3;
const GRID_COLUMNS: u16 = 7;
/// 从左到右：绿、黄、红、黄、绿。跨列 1、1、3、1、1，对应宽度 1/7、1/7、3/7、1/7、1/7。
const ZONE_SPANS: [u16; 5] = [1, 1, 3, 1, 1];
const ZONE_COLORS: [u32; 5] = [0x22cc44, 0xffcc22, 0xff3333, 0xffcc22, 0x22cc44];
/// 点亮阶段：两侧绿为 0，两侧黄为 1，中间红为 2。
const ZONE_STEPS: [usize; 5] = [0, 1, 2, 1, 0];
const DIM_COLOR: u32 = 0x2a2a2a;
const UNCALIBRATED_COLOR: u32 = 0xffcc22;
const CALIBRATING_COLOR: u32 = 0x22cc44;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Rally1Config {
    pub y_ratio: f32,
    pub thickness_px: usize,
    pub width_percent: usize,
    pub lit_opacity: f32,
    pub dim_opacity: f32,
    pub blink_percent: f32,
    pub blink_ms: usize,
}

impl Default for Rally1Config {
    fn default() -> Self {
        Self {
            y_ratio: DEFAULT_Y_RATIO,
            thickness_px: DEFAULT_THICKNESS_PX,
            width_percent: DEFAULT_WIDTH_PERCENT,
            lit_opacity: DEFAULT_LIT_OPACITY,
            dim_opacity: DEFAULT_DIM_OPACITY,
            blink_percent: DEFAULT_BLINK_PERCENT,
            blink_ms: DEFAULT_BLINK_MS,
        }
    }
}

pub struct SettingsControls {
    pub y: Entity<NumberFieldState>,
    pub thickness: Entity<InputState>,
    pub width: Entity<NumberFieldState>,
    pub lit_opacity: Entity<NumberFieldState>,
    pub dim_opacity: Entity<NumberFieldState>,
    pub blink_percent: Entity<NumberFieldState>,
    pub blink_ms: Entity<NumberFieldState>,
}

impl SettingsControls {
    pub fn new(cfg: &Rally1Config, window: &mut Window, cx: &mut Context<Settings>) -> Self {
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
        subscribe_number_field(&y, cx, |value| {
            let y_ratio = (value / 100.0).clamp(0.0, 1.0);
            write_cfg(|cfg| cfg.y_ratio = y_ratio);
        });

        let thickness = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(DEFAULT_THICKNESS_PX.to_string())
                .default_value(cfg.thickness_px.to_string())
        });
        subscribe_input(&thickness, window, cx, SettingsControls::apply_thickness);

        let width = cx.new(|cx| {
            NumberFieldState::builder()
                .min(1.0)
                .max(100.0)
                .step(1.0)
                .default_value(cfg.width_percent as f32)
                .unit("%")
                .decimals(0)
                .build(window, cx)
        });
        subscribe_number_field(&width, cx, |value| {
            let width = (value as usize).clamp(1, 100);
            write_cfg(|cfg| cfg.width_percent = width);
        });

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
        subscribe_number_field(&lit_opacity, cx, |value| {
            write_cfg(|cfg| cfg.lit_opacity = opacity_from_percent(value));
        });
        subscribe_number_field(&dim_opacity, cx, |value| {
            write_cfg(|cfg| cfg.dim_opacity = opacity_from_percent(value));
        });

        let blink_percent = cx.new(|cx| {
            NumberFieldState::builder()
                .min(BLINK_PERCENT_MIN)
                .max(BLINK_PERCENT_MAX)
                .step(0.1)
                .default_value(cfg.blink_percent)
                .unit("%")
                .decimals(1)
                .build(window, cx)
        });
        subscribe_number_field(&blink_percent, cx, |value| {
            let value = value.clamp(BLINK_PERCENT_MIN, BLINK_PERCENT_MAX);
            write_cfg(|cfg| cfg.blink_percent = value);
        });

        let blink_ms = cx.new(|cx| {
            NumberFieldState::builder()
                .min(BLINK_MS_MIN as f32)
                .max(BLINK_MS_MAX as f32)
                .step(5.0)
                .default_value(cfg.blink_ms as f32)
                .unit("ms")
                .decimals(0)
                .build(window, cx)
        });
        subscribe_number_field(&blink_ms, cx, |value| {
            let blink_ms = (value as usize).clamp(BLINK_MS_MIN, BLINK_MS_MAX);
            write_cfg(|cfg| cfg.blink_ms = blink_ms);
        });

        Self {
            y,
            thickness,
            width,
            lit_opacity,
            dim_opacity,
            blink_percent,
            blink_ms,
        }
    }

    pub fn sync(&self, cfg: &Rally1Config, window: &mut Window, cx: &mut Context<Settings>) {
        self.y.update(cx, |field, cx| {
            field.set_value(cfg.y_ratio * 100.0, window, cx);
        });
        self.thickness.update(cx, |input, cx| {
            input.set_value(cfg.thickness_px.to_string(), window, cx);
        });
        self.width.update(cx, |field, cx| {
            field.set_value(cfg.width_percent as f32, window, cx);
        });
        self.lit_opacity.update(cx, |field, cx| {
            field.set_value(opacity_as_percent(cfg.lit_opacity), window, cx);
        });
        self.dim_opacity.update(cx, |field, cx| {
            field.set_value(opacity_as_percent(cfg.dim_opacity), window, cx);
        });
        self.blink_percent.update(cx, |field, cx| {
            field.set_value(cfg.blink_percent, window, cx);
        });
        self.blink_ms.update(cx, |field, cx| {
            field.set_value(cfg.blink_ms as f32, window, cx);
        });
    }

    pub fn apply_thickness(&self, cx: &mut Context<Settings>) {
        let value = self.thickness.read(cx).value().to_string();
        if let Ok(thickness) = value.trim().parse::<usize>() {
            if thickness > 0 {
                write_cfg(|cfg| cfg.thickness_px = thickness);
            }
        }
        cx.notify();
    }
}

pub fn render_hud(props: &HudTelemetryProps, cfg: &Rally1Config) -> gpui_kit::AnyElement {
    let thickness = cfg.thickness_px.max(1) as f32;
    let lit_opacity = cfg.lit_opacity;
    let dim_opacity = cfg.dim_opacity;
    let zone_colors: [gpui_kit::Rgba; 5] = match props.calibration {
        CalibrationState::Ready { rpm_ratio, .. } => {
            let (on_count, flash) =
                calc_linear_lights(rpm_ratio, cfg.blink_percent / 100.0, LIGHT_STEPS);
            let blinking_on = !flash || is_blink_lit(props.now_ms, cfg.blink_ms);
            std::array::from_fn(|index| {
                if blinking_on && ZONE_STEPS[index] < on_count {
                    with_opacity(ZONE_COLORS[index], lit_opacity)
                } else {
                    with_opacity(DIM_COLOR, dim_opacity)
                }
            })
        }
        CalibrationState::Uncalibrated { progress, .. } => std::array::from_fn(|index| {
            let fill = calibrate_segment_fill(progress, ZONE_STEPS[index], LIGHT_STEPS);
            let rgb = if fill >= 1.0 {
                CALIBRATING_COLOR
            } else {
                UNCALIBRATED_COLOR
            };
            with_opacity(rgb, lit_opacity)
        }),
    };
    let strip = div()
        .grid()
        .grid_cols(GRID_COLUMNS)
        .grid_rows(1)
        .w_full()
        .h(px(thickness))
        .children(
            ZONE_SPANS
                .into_iter()
                .zip(zone_colors)
                .map(|(span, color)| div().col_span(span).size_full().bg(color)),
        );

    div()
        .absolute()
        .top(relative(cfg.y_ratio.clamp(0.0, 1.0)))
        .left_0()
        .right_0()
        .flex()
        .justify_center()
        .child(
            div()
                .w(relative(cfg.width_percent.clamp(1, 100) as f32 / 100.0))
                .child(strip),
        )
        .into_any_element()
}

pub fn render_settings(
    view: &Settings,
    colors: &Colors,
    cx: &mut Context<Settings>,
) -> gpui_kit::AnyElement {
    let cfg = crate::session::shift_lights().rally_1;
    let defaults = Rally1Config::default();
    let controls = &view.shift_lights.rally_1;
    let dim_warning = (!crate::core::calibration::shift_lights_calibrated())
        .then_some("当前处于未校准状态，熄灭透明度不可见");
    let thickness_control = input_apply_control(
        Input::new(&controls.thickness)
            .w(px(96.))
            .with_size(Size::Medium),
        Some("px"),
        "rally-thickness-apply",
        input_differs(&controls.thickness, cfg.thickness_px, cx),
        colors,
        cx,
        |this, _, cx| this.shift_lights.rally_1.apply_thickness(cx),
    );
    div()
        .flex()
        .flex_col()
        .gap_5()
        .child(
            settings_group(
                colors,
                "样式与位置",
                "垂直位置。",
            )
            .child(setting_row_with_reset(
                colors,
                "垂直位置",
                "百分比，范围 0–100，步进 0.5，默认 0。",
                "rally-y-reset",
                NumberField::new(&controls.y).unit_color(colors.disabled),
                cfg.y_ratio != defaults.y_ratio,
                None,
                cx,
                reset_y,
            )),
        )
        .child(separator(colors))
        .child(
            settings_group(
                colors,
                "尺寸与布局",
                "宽度与厚度。",
            )
            .child(setting_row_with_reset(
                colors,
                "宽度比例",
                "百分比，范围 1–100，步进 1，默认 100。",
                "rally-width-reset",
                NumberField::new(&controls.width).unit_color(colors.disabled),
                cfg.width_percent != defaults.width_percent,
                None,
                cx,
                reset_width,
            ))
            .child(setting_row_with_reset(
                colors,
                "厚度",
                "正整数，单位 px，默认 90。",
                "rally-thickness-reset",
                thickness_control,
                cfg.thickness_px != defaults.thickness_px,
                None,
                cx,
                reset_thickness,
            )),
        )
        .child(separator(colors))
        .child(
            settings_group(
                colors,
                "显示效果",
                "闪烁与透明度。",
            )
            .child(setting_row_with_reset(
                colors,
                "闪烁阈值",
                "百分比，范围 80–100，步进 0.1，默认 95。",
                "rally-blink-reset",
                NumberField::new(&controls.blink_percent).unit_color(colors.disabled),
                cfg.blink_percent != defaults.blink_percent,
                None,
                cx,
                reset_blink_percent,
            ))
            .child(setting_row_with_reset(
                colors,
                "闪烁速度",
                "整数毫秒，范围 20–200，步进 5，默认 80。越小越快。",
                "rally-blink-ms-reset",
                NumberField::new(&controls.blink_ms).unit_color(colors.disabled),
                cfg.blink_ms != defaults.blink_ms,
                None,
                cx,
                reset_blink_ms,
            ))
            .child(setting_row_with_reset(
                colors,
                "亮起透明度",
                "百分比，范围 0–100，步进 1，默认 70。",
                "rally-lit-opacity-reset",
                NumberField::new(&controls.lit_opacity).unit_color(colors.disabled),
                cfg.lit_opacity != defaults.lit_opacity,
                None,
                cx,
                reset_lit_opacity,
            ))
            .child(setting_row_with_reset(
                colors,
                "熄灭透明度",
                "百分比，范围 0–100，步进 1，默认 50。",
                "rally-dim-opacity-reset",
                NumberField::new(&controls.dim_opacity).unit_color(colors.disabled),
                cfg.dim_opacity != defaults.dim_opacity,
                dim_warning,
                cx,
                reset_dim_opacity,
            )),
        )
        .into_any_element()
}

fn write_cfg(f: impl FnOnce(&mut Rally1Config)) {
    crate::session::update_shift_lights(|settings| f(&mut settings.rally_1));
}

fn subscribe_input(
    input: &Entity<InputState>,
    window: &mut Window,
    cx: &mut Context<Settings>,
    apply: fn(&SettingsControls, &mut Context<Settings>),
) {
    cx.subscribe_in(
        input,
        window,
        move |this, _, event: &InputEvent, _, cx| match event {
            InputEvent::Change => cx.notify(),
            InputEvent::PressEnter { .. } => apply(&this.shift_lights.rally_1, cx),
            _ => {}
        },
    )
    .detach();
}

fn reset_y(this: &mut Settings, window: &mut Window, cx: &mut Context<Settings>) {
    let value = Rally1Config::default().y_ratio;
    write_cfg(|cfg| cfg.y_ratio = value);
    this.shift_lights.rally_1.y.update(cx, |field, cx| {
        field.set_value(value * 100.0, window, cx);
    });
    cx.notify();
}

fn reset_width(this: &mut Settings, window: &mut Window, cx: &mut Context<Settings>) {
    let value = Rally1Config::default().width_percent;
    write_cfg(|cfg| cfg.width_percent = value);
    this.shift_lights.rally_1.width.update(cx, |field, cx| {
        field.set_value(value as f32, window, cx);
    });
    cx.notify();
}

fn reset_thickness(this: &mut Settings, window: &mut Window, cx: &mut Context<Settings>) {
    let value = Rally1Config::default().thickness_px;
    write_cfg(|cfg| cfg.thickness_px = value);
    this.shift_lights
        .rally_1
        .thickness
        .update(cx, |input, cx| {
            input.set_value(value.to_string(), window, cx);
        });
    cx.notify();
}

fn reset_blink_percent(this: &mut Settings, window: &mut Window, cx: &mut Context<Settings>) {
    let value = Rally1Config::default().blink_percent;
    write_cfg(|cfg| cfg.blink_percent = value);
    this.shift_lights.rally_1.blink_percent.update(cx, |field, cx| {
        field.set_value(value, window, cx);
    });
    cx.notify();
}

fn reset_blink_ms(this: &mut Settings, window: &mut Window, cx: &mut Context<Settings>) {
    let value = Rally1Config::default().blink_ms;
    write_cfg(|cfg| cfg.blink_ms = value);
    this.shift_lights.rally_1.blink_ms.update(cx, |field, cx| {
        field.set_value(value as f32, window, cx);
    });
    cx.notify();
}

fn reset_lit_opacity(this: &mut Settings, window: &mut Window, cx: &mut Context<Settings>) {
    let value = Rally1Config::default().lit_opacity;
    write_cfg(|cfg| cfg.lit_opacity = value);
    this.shift_lights.rally_1.lit_opacity.update(cx, |field, cx| {
        field.set_value(opacity_as_percent(value), window, cx);
    });
    cx.notify();
}

fn reset_dim_opacity(this: &mut Settings, window: &mut Window, cx: &mut Context<Settings>) {
    let value = Rally1Config::default().dim_opacity;
    write_cfg(|cfg| cfg.dim_opacity = value);
    this.shift_lights.rally_1.dim_opacity.update(cx, |field, cx| {
        field.set_value(opacity_as_percent(value), window, cx);
    });
    cx.notify();
}
