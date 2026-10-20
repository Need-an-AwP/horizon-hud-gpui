use gpui_kit::{div, prelude::*, px, relative, Context, Entity, IntoElement, Window};
use gpui_kit::component::{
    input::{Input, InputEvent, InputState},
    slider::{SliderEvent, SliderState, SliderValue},
    Sizable, Size,
};
use serde::{Deserialize, Serialize};

use crate::app::settings_window::colors::Colors;
use crate::app::settings_window::Settings;
use crate::components::shared::color::with_opacity;
use crate::components::shared::rpm_math::{
    calc_linear_lights, calibrate_segment_fill, is_blink_lit,
};
use crate::components::shared::settings_ui::{
    input_apply_control, input_differs, separator, setting_row_with_reset, settings_group,
    slider_row,
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
    pub y: Entity<SliderState>,
    pub thickness: Entity<InputState>,
    pub width: Entity<SliderState>,
    pub lit_opacity: Entity<SliderState>,
    pub dim_opacity: Entity<SliderState>,
    pub blink_percent: Entity<SliderState>,
    pub blink_ms: Entity<SliderState>,
}

impl SettingsControls {
    pub fn new(cfg: &Rally1Config, window: &mut Window, cx: &mut Context<Settings>) -> Self {
        let y = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(100.0)
                .step(0.5)
                .default_value(cfg.y_ratio * 100.0)
        });
        subscribe_slider(&y, cx, |value| {
            if value.is_finite() {
                let y_ratio = (value / 100.0).clamp(0.0, 1.0);
                write_cfg(|cfg| cfg.y_ratio = y_ratio);
            }
        });

        let thickness = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(DEFAULT_THICKNESS_PX.to_string())
                .default_value(cfg.thickness_px.to_string())
        });
        subscribe_input(&thickness, window, cx, SettingsControls::apply_thickness);

        let width = cx.new(|_| {
            SliderState::new()
                .min(1.0)
                .max(100.0)
                .step(1.0)
                .default_value(cfg.width_percent as f32)
        });
        subscribe_slider(&width, cx, |value| {
            if value.is_finite() {
                let width = (value as usize).clamp(1, 100);
                write_cfg(|cfg| cfg.width_percent = width);
            }
        });

        let lit_opacity = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(1.0)
                .step(0.05)
                .default_value(cfg.lit_opacity)
        });
        let dim_opacity = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(1.0)
                .step(0.05)
                .default_value(cfg.dim_opacity)
        });
        subscribe_slider(&lit_opacity, cx, |value| {
            if value.is_finite() {
                let value = value.clamp(0.0, 1.0);
                write_cfg(|cfg| cfg.lit_opacity = value);
            }
        });
        subscribe_slider(&dim_opacity, cx, |value| {
            if value.is_finite() {
                let value = value.clamp(0.0, 1.0);
                write_cfg(|cfg| cfg.dim_opacity = value);
            }
        });

        let blink_percent = cx.new(|_| {
            SliderState::new()
                .min(BLINK_PERCENT_MIN)
                .max(BLINK_PERCENT_MAX)
                .step(0.1)
                .default_value(cfg.blink_percent)
        });
        subscribe_slider(&blink_percent, cx, |value| {
            if value.is_finite() {
                let value = value.clamp(BLINK_PERCENT_MIN, BLINK_PERCENT_MAX);
                write_cfg(|cfg| cfg.blink_percent = value);
            }
        });

        let blink_ms = cx.new(|_| {
            SliderState::new()
                .min(BLINK_MS_MIN as f32)
                .max(BLINK_MS_MAX as f32)
                .step(5.0)
                .default_value(cfg.blink_ms as f32)
        });
        subscribe_slider(&blink_ms, cx, |value| {
            if value.is_finite() {
                let blink_ms = (value as usize).clamp(BLINK_MS_MIN, BLINK_MS_MAX);
                write_cfg(|cfg| cfg.blink_ms = blink_ms);
            }
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
        self.y.update(cx, |slider, cx| {
            slider.set_value(cfg.y_ratio * 100.0, window, cx);
        });
        self.thickness.update(cx, |input, cx| {
            input.set_value(cfg.thickness_px.to_string(), window, cx);
        });
        self.width.update(cx, |slider, cx| {
            slider.set_value(cfg.width_percent as f32, window, cx);
        });
        self.lit_opacity.update(cx, |slider, cx| {
            slider.set_value(cfg.lit_opacity, window, cx);
        });
        self.dim_opacity.update(cx, |slider, cx| {
            slider.set_value(cfg.dim_opacity, window, cx);
        });
        self.blink_percent.update(cx, |slider, cx| {
            slider.set_value(cfg.blink_percent, window, cx);
        });
        self.blink_ms.update(cx, |slider, cx| {
            slider.set_value(cfg.blink_ms as f32, window, cx);
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
        .then_some("当前处于未校准状态，熄灭透明度可能不可见");
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
                "灯条横向贴在屏幕上，按屏幕高度的比例决定垂直位置。转速升高时从两侧向中心依次点亮绿、黄、红。",
            )
            .child(slider_row(
                "垂直位置",
                "相对屏幕高度的位置，灯条上沿对齐该位置；0% 贴在屏幕顶部。默认 0%。",
                "rally-y-reset",
                &controls.y,
                colors,
                None,
                "%",
                1,
                cfg.y_ratio != defaults.y_ratio,
                cx,
                reset_y,
            )),
        )
        .child(separator(colors))
        .child(
            settings_group(
                colors,
                "尺寸与布局",
                "五段之间没有间隙，宽度固定为 1/7、1/7、3/7、1/7、1/7。这里调整整条灯相对屏幕的宽度和厚度。",
            )
            .child(slider_row(
                "宽度比例",
                "相对屏幕宽度的比例，灯条水平居中；默认 100%。",
                "rally-width-reset",
                &controls.width,
                colors,
                None,
                "%",
                0,
                cfg.width_percent != defaults.width_percent,
                cx,
                reset_width,
            ))
            .child(setting_row_with_reset(
                colors,
                "厚度",
                "灯条高度，单位像素；默认 90 px。",
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
                "控制灯条闪烁时机与速度，以及亮起与未亮起区域的透明度。未校准时五段为黄色，校准进度从两侧向中心逐段变为绿色。",
            )
            .child(slider_row(
                "闪烁阈值",
                "达到校准转速的该百分比后，五个区域全部按自身颜色闪烁；范围 80% 到 100%，步进 0.1%；默认 95%。",
                "rally-blink-reset",
                &controls.blink_percent,
                colors,
                None,
                "%",
                1,
                cfg.blink_percent != defaults.blink_percent,
                cx,
                reset_blink_percent,
            ))
            .child(slider_row(
                "闪烁速度",
                "亮灭切换的半周期时长，数值越小闪烁越快；范围 20 到 200 ms，步进 5 ms；默认 80 ms。",
                "rally-blink-ms-reset",
                &controls.blink_ms,
                colors,
                None,
                " ms",
                0,
                cfg.blink_ms != defaults.blink_ms,
                cx,
                reset_blink_ms,
            ))
            .child(slider_row(
                "亮起透明度",
                "亮起区域的透明度，范围 0 到 1；默认 0.7。",
                "rally-lit-opacity-reset",
                &controls.lit_opacity,
                colors,
                None,
                "",
                2,
                cfg.lit_opacity != defaults.lit_opacity,
                cx,
                reset_lit_opacity,
            ))
            .child(slider_row(
                "熄灭透明度",
                "未亮起区域的透明度，范围 0 到 1；默认 0.5。",
                "rally-dim-opacity-reset",
                &controls.dim_opacity,
                colors,
                dim_warning,
                "",
                2,
                cfg.dim_opacity != defaults.dim_opacity,
                cx,
                reset_dim_opacity,
            )),
        )
        .into_any_element()
}

fn write_cfg(f: impl FnOnce(&mut Rally1Config)) {
    crate::session::update_shift_lights(|settings| f(&mut settings.rally_1));
}

fn subscribe_slider(
    slider: &Entity<SliderState>,
    cx: &mut Context<Settings>,
    on_value: impl Fn(f32) + 'static,
) {
    cx.subscribe(slider, move |_this, _, event: &SliderEvent, cx| {
        if let SliderEvent::Change(SliderValue::Single(value)) = event {
            on_value(*value);
            cx.notify();
        }
    })
    .detach();
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
    this.shift_lights.rally_1.y.update(cx, |slider, cx| {
        slider.set_value(value * 100.0, window, cx);
    });
    cx.notify();
}

fn reset_width(this: &mut Settings, window: &mut Window, cx: &mut Context<Settings>) {
    let value = Rally1Config::default().width_percent;
    write_cfg(|cfg| cfg.width_percent = value);
    this.shift_lights
        .rally_1
        .width
        .update(cx, |slider, cx| {
            slider.set_value(value as f32, window, cx);
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
    this.shift_lights
        .rally_1
        .blink_percent
        .update(cx, |slider, cx| {
            slider.set_value(value, window, cx);
        });
    cx.notify();
}

fn reset_blink_ms(this: &mut Settings, window: &mut Window, cx: &mut Context<Settings>) {
    let value = Rally1Config::default().blink_ms;
    write_cfg(|cfg| cfg.blink_ms = value);
    this.shift_lights
        .rally_1
        .blink_ms
        .update(cx, |slider, cx| {
            slider.set_value(value as f32, window, cx);
        });
    cx.notify();
}

fn reset_lit_opacity(this: &mut Settings, window: &mut Window, cx: &mut Context<Settings>) {
    let value = Rally1Config::default().lit_opacity;
    write_cfg(|cfg| cfg.lit_opacity = value);
    this.shift_lights
        .rally_1
        .lit_opacity
        .update(cx, |slider, cx| {
            slider.set_value(value, window, cx);
        });
    cx.notify();
}

fn reset_dim_opacity(this: &mut Settings, window: &mut Window, cx: &mut Context<Settings>) {
    let value = Rally1Config::default().dim_opacity;
    write_cfg(|cfg| cfg.dim_opacity = value);
    this.shift_lights
        .rally_1
        .dim_opacity
        .update(cx, |slider, cx| {
            slider.set_value(value, window, cx);
        });
    cx.notify();
}
