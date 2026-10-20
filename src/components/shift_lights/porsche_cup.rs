use gpui_kit::{div, prelude::*, px, Context, Entity, IntoElement, Window};
use gpui_kit::component::{
    input::{Input, InputEvent, InputState},
    Sizable, Size,
};
use serde::{Deserialize, Serialize};

use crate::app::settings_window::colors::Colors;
use crate::app::settings_window::Settings;
use crate::components::shared::color::with_opacity;
use crate::components::shared::layouts::{edge_strip_container, EdgePosition};
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

const DEFAULT_DIAMETER_PX: usize = 40;
const DEFAULT_GAP_PX: usize = 8;
const DEFAULT_WIDTH_PERCENT: usize = 75;
const DEFAULT_OFFSET_PX: usize = 5;
const DEFAULT_LIT_OPACITY: f32 = 0.85;
const DEFAULT_DIM_OPACITY: f32 = 0.5;
const DEFAULT_BLINK_PERCENT: f32 = 95.0;
const BLINK_PERCENT_MIN: f32 = 80.0;
const BLINK_PERCENT_MAX: f32 = 100.0;
const DEFAULT_BLINK_MS: usize = 80;
const BLINK_MS_MIN: usize = 20;
const BLINK_MS_MAX: usize = 200;

const RPM_CIRCLE_COUNT: usize = 16;
const RPM_CIRCLE_HALF: usize = 8;
const RPM_CIRCLE_COLORS: [u32; RPM_CIRCLE_COUNT] = [
    0x22cc44, 0x22cc44, 0x22cc44, 0xffcc22, 0xffcc22, 0xffcc22, 0xff3333, 0xff3333, 0xff3333,
    0xff3333, 0xffcc22, 0xffcc22, 0xffcc22, 0x22cc44, 0x22cc44, 0x22cc44,
];
const RPM_LIGHT_BLINK_BLUE: u32 = 0x3399ff;
const RPM_LIGHT_UNCALIBRATED: u32 = 0xffcc22;
const RPM_LIGHT_CALIBRATING: u32 = 0x22cc44;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PorscheCupConfig {
    pub position: EdgePosition,
    pub width_percent: usize,
    pub diameter_px: usize,
    pub gap_px: usize,
    pub offset_px: usize,
    pub lit_opacity: f32,
    pub dim_opacity: f32,
    pub blink_percent: f32,
    pub blink_ms: usize,
}

impl Default for PorscheCupConfig {
    fn default() -> Self {
        Self {
            position: EdgePosition::Top,
            width_percent: DEFAULT_WIDTH_PERCENT,
            diameter_px: DEFAULT_DIAMETER_PX,
            gap_px: DEFAULT_GAP_PX,
            offset_px: DEFAULT_OFFSET_PX,
            lit_opacity: DEFAULT_LIT_OPACITY,
            dim_opacity: DEFAULT_DIM_OPACITY,
            blink_percent: DEFAULT_BLINK_PERCENT,
            blink_ms: DEFAULT_BLINK_MS,
        }
    }
}

pub struct SettingsControls {
    pub lit_opacity: Entity<NumberFieldState>,
    pub dim_opacity: Entity<NumberFieldState>,
    pub offset: Entity<InputState>,
    pub diameter: Entity<InputState>,
    pub gap: Entity<InputState>,
    pub width: Entity<NumberFieldState>,
    pub blink_percent: Entity<NumberFieldState>,
    pub blink_ms: Entity<NumberFieldState>,
}

impl SettingsControls {
    pub fn new(cfg: &PorscheCupConfig, window: &mut Window, cx: &mut Context<Settings>) -> Self {
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

        let offset = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(DEFAULT_OFFSET_PX.to_string())
                .default_value(cfg.offset_px.to_string())
        });
        let diameter = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(DEFAULT_DIAMETER_PX.to_string())
                .default_value(cfg.diameter_px.to_string())
        });
        let gap = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(DEFAULT_GAP_PX.to_string())
                .default_value(cfg.gap_px.to_string())
        });
        subscribe_input(&offset, window, cx, SettingsControls::apply_offset);
        subscribe_input(&diameter, window, cx, SettingsControls::apply_diameter);
        subscribe_input(&gap, window, cx, SettingsControls::apply_gap);

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
            lit_opacity,
            dim_opacity,
            offset,
            diameter,
            gap,
            width,
            blink_percent,
            blink_ms,
        }
    }

    pub fn sync(&self, cfg: &PorscheCupConfig, window: &mut Window, cx: &mut Context<Settings>) {
        self.lit_opacity.update(cx, |field, cx| {
            field.set_value(opacity_as_percent(cfg.lit_opacity), window, cx);
        });
        self.dim_opacity.update(cx, |field, cx| {
            field.set_value(opacity_as_percent(cfg.dim_opacity), window, cx);
        });
        self.offset.update(cx, |input, cx| {
            input.set_value(cfg.offset_px.to_string(), window, cx);
        });
        self.diameter.update(cx, |input, cx| {
            input.set_value(cfg.diameter_px.to_string(), window, cx);
        });
        self.gap.update(cx, |input, cx| {
            input.set_value(cfg.gap_px.to_string(), window, cx);
        });
        self.width.update(cx, |field, cx| {
            field.set_value(cfg.width_percent as f32, window, cx);
        });
        self.blink_percent.update(cx, |field, cx| {
            field.set_value(cfg.blink_percent, window, cx);
        });
        self.blink_ms.update(cx, |field, cx| {
            field.set_value(cfg.blink_ms as f32, window, cx);
        });
    }

    pub fn apply_offset(&self, cx: &mut Context<Settings>) {
        let value = self.offset.read(cx).value().to_string();
        if let Ok(offset) = value.trim().parse::<usize>() {
            write_cfg(|cfg| cfg.offset_px = offset);
        }
        cx.notify();
    }

    pub fn apply_diameter(&self, cx: &mut Context<Settings>) {
        let value = self.diameter.read(cx).value().to_string();
        if let Ok(diameter) = value.trim().parse::<usize>() {
            if diameter > 0 {
                write_cfg(|cfg| cfg.diameter_px = diameter);
            }
        }
        cx.notify();
    }

    pub fn apply_gap(&self, cx: &mut Context<Settings>) {
        let value = self.gap.read(cx).value().to_string();
        if let Ok(gap) = value.trim().parse::<usize>() {
            write_cfg(|cfg| cfg.gap_px = gap);
        }
        cx.notify();
    }
}

pub fn render_hud(props: &HudTelemetryProps, cfg: &PorscheCupConfig) -> gpui_kit::AnyElement {
    let position = cfg.position;
    let diameter = cfg.diameter_px as f32;
    let gap = cfg.gap_px as f32;
    let lit_opacity = cfg.lit_opacity;
    let dim_opacity = cfg.dim_opacity;
    let strip = match props.calibration {
        CalibrationState::Ready { rpm_ratio, .. } => {
            let blink_frac = cfg.blink_percent / 100.0;
            let (on_count, flash) = calc_linear_lights(rpm_ratio, blink_frac, RPM_CIRCLE_HALF);
            let lit = !flash || is_blink_lit(props.now_ms, cfg.blink_ms);
            shift_circle_strip(
                position,
                diameter,
                gap,
                (0..RPM_CIRCLE_COUNT).map(move |i| {
                    let rpm_index = i.min(RPM_CIRCLE_COUNT - 1 - i);
                    let color = if lit && rpm_index < on_count {
                        let rgb = if flash {
                            RPM_LIGHT_BLINK_BLUE
                        } else {
                            RPM_CIRCLE_COLORS[i]
                        };
                        with_opacity(rgb, lit_opacity)
                    } else {
                        with_opacity(0x2a2a2a, dim_opacity)
                    };
                    shift_light_circle(diameter, color)
                }),
            )
        }
        CalibrationState::Uncalibrated { progress, .. } => shift_circle_strip(
            position,
            diameter,
            gap,
            (0..RPM_CIRCLE_COUNT).map(move |i| {
                let pair = i.min(RPM_CIRCLE_COUNT - 1 - i);
                let fill = calibrate_segment_fill(progress, pair, RPM_CIRCLE_HALF);
                let rgb = if fill >= 1.0 {
                    RPM_LIGHT_CALIBRATING
                } else {
                    RPM_LIGHT_UNCALIBRATED
                };
                shift_light_circle(diameter, with_opacity(rgb, lit_opacity))
            }),
        ),
    };
    edge_strip_container(
        position,
        cfg.width_percent,
        cfg.diameter_px,
        cfg.offset_px,
        strip,
    )
    .into_any_element()
}

pub fn render_settings(
    view: &Settings,
    colors: &Colors,
    cx: &mut Context<Settings>,
) -> gpui_kit::AnyElement {
    let cfg = crate::session::shift_lights().porsche_cup;
    let defaults = PorscheCupConfig::default();
    let controls = &view.shift_lights.porsche_cup;
    let dim_warning = (!crate::core::calibration::shift_lights_calibrated())
        .then_some("当前处于未校准状态，熄灭透明度不可见");

    let position_control = position_control(cfg.position, colors, cx);
    let diameter_control = input_apply_control(
        Input::new(&controls.diameter)
            .w(px(96.))
            .with_size(Size::Medium),
        Some("px"),
        "porsche-diameter-apply",
        input_differs(&controls.diameter, cfg.diameter_px, cx),
        colors,
        cx,
        |this, _, cx| this.shift_lights.porsche_cup.apply_diameter(cx),
    );
    let gap_control = input_apply_control(
        Input::new(&controls.gap).w(px(96.)).with_size(Size::Medium),
        Some("px"),
        "porsche-gap-apply",
        input_differs(&controls.gap, cfg.gap_px, cx),
        colors,
        cx,
        |this, _, cx| this.shift_lights.porsche_cup.apply_gap(cx),
    );
    let offset_control = input_apply_control(
        Input::new(&controls.offset)
            .w(px(96.))
            .with_size(Size::Medium),
        Some("px"),
        "porsche-offset-apply",
        input_differs(&controls.offset, cfg.offset_px, cx),
        colors,
        cx,
        |this, _, cx| this.shift_lights.porsche_cup.apply_offset(cx),
    );
    div()
        .flex()
        .flex_col()
        .gap_5()
        .child(
            settings_group(
                colors,
                "样式与位置",
                "贴边位置。",
            )
            .child(setting_row_with_reset(
                colors,
                "位置",
                "上、下、左、右。",
                "porsche-position-reset",
                position_control,
                cfg.position != defaults.position,
                None,
                cx,
                reset_position,
            )),
        )
        .child(separator(colors))
        .child(
            settings_group(
                colors,
                "尺寸与布局",
                "宽度、直径、间隔与偏移。",
            )
            .child(setting_row_with_reset(
                colors,
                "整体宽度",
                "百分比，范围 1–100，步进 1，默认 75。",
                "porsche-width-reset",
                NumberField::new(&controls.width).unit_color(colors.disabled),
                cfg.width_percent != defaults.width_percent,
                None,
                cx,
                reset_width,
            ))
            .child(setting_row_with_reset(
                colors,
                "灯珠直径",
                "正整数，单位 px，默认 40。",
                "porsche-diameter-reset",
                diameter_control,
                cfg.diameter_px != defaults.diameter_px,
                None,
                cx,
                reset_diameter,
            ))
            .child(setting_row_with_reset(
                colors,
                "灯珠间隔",
                "非负整数，单位 px，默认 8。",
                "porsche-gap-reset",
                gap_control,
                cfg.gap_px != defaults.gap_px,
                None,
                cx,
                reset_gap,
            ))
            .child(setting_row_with_reset(
                colors,
                "边缘偏移",
                "非负整数，单位 px，默认 5。",
                "porsche-offset-reset",
                offset_control,
                cfg.offset_px != defaults.offset_px,
                None,
                cx,
                reset_offset,
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
                "porsche-blink-reset",
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
                "porsche-blink-ms-reset",
                NumberField::new(&controls.blink_ms).unit_color(colors.disabled),
                cfg.blink_ms != defaults.blink_ms,
                None,
                cx,
                reset_blink_ms,
            ))
            .child(setting_row_with_reset(
                colors,
                "亮起透明度",
                "百分比，范围 0–100，步进 1，默认 85。",
                "porsche-lit-opacity-reset",
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
                "porsche-dim-opacity-reset",
                NumberField::new(&controls.dim_opacity).unit_color(colors.disabled),
                cfg.dim_opacity != defaults.dim_opacity,
                dim_warning,
                cx,
                reset_dim_opacity,
            )),
        )
        .into_any_element()
}

fn write_cfg(f: impl FnOnce(&mut PorscheCupConfig)) {
    crate::session::update_shift_lights(|settings| f(&mut settings.porsche_cup));
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
            InputEvent::PressEnter { .. } => apply(&this.shift_lights.porsche_cup, cx),
            _ => {}
        },
    )
    .detach();
}

fn shift_light_circle(diameter: f32, color: gpui_kit::Rgba) -> gpui_kit::Div {
    div()
        .flex_none()
        .w(px(diameter))
        .h(px(diameter))
        .rounded_full()
        .bg(color)
}

fn shift_circle_strip(
    position: EdgePosition,
    diameter: f32,
    gap: f32,
    cells: impl IntoIterator<Item = impl IntoElement>,
) -> gpui_kit::Div {
    match position {
        EdgePosition::Left | EdgePosition::Right => div()
            .flex()
            .flex_col()
            .h_full()
            .w(px(diameter))
            .gap(px(gap))
            .justify_center()
            .items_center()
            .children(cells),
        EdgePosition::Bottom | EdgePosition::Top => div()
            .flex()
            .flex_row()
            .w_full()
            .gap(px(gap))
            .justify_center()
            .items_center()
            .children(cells),
    }
}

fn segmented_row(colors: &Colors) -> gpui_kit::Div {
    div()
        .flex()
        .flex_row()
        .flex_none()
        .p_1()
        .gap_1()
        .rounded_md()
        .bg(colors.container)
        .border_1()
        .border_color(colors.border)
}

fn choice_button(
    id: &'static str,
    label: &'static str,
    selected: bool,
    colors: &Colors,
    cx: &mut Context<Settings>,
    on_click: impl Fn(&mut Settings, &mut Window, &mut Context<Settings>) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .px_3()
        .py_1()
        .rounded_sm()
        .text_xs()
        .cursor_pointer()
        .when(selected, |el| {
            el.bg(colors.selected).text_color(colors.selected_text)
        })
        .when(!selected, |el| el.text_color(colors.text))
        .hover(|style| {
            if selected {
                style.opacity(0.9)
            } else {
                style.bg(colors.background)
            }
        })
        .on_click(cx.listener(move |this, _, window, cx| {
            on_click(this, window, cx);
        }))
        .child(label)
}

fn bind_position(
    position: EdgePosition,
) -> impl Fn(&mut Settings, &mut Window, &mut Context<Settings>) + 'static {
    move |_, _, cx| {
        write_cfg(|cfg| cfg.position = position);
        cx.notify();
    }
}

fn position_control(
    selected: EdgePosition,
    colors: &Colors,
    cx: &mut Context<Settings>,
) -> gpui_kit::Div {
    segmented_row(colors)
        .child(choice_button(
            "porsche-bottom",
            "底部",
            selected == EdgePosition::Bottom,
            colors,
            cx,
            bind_position(EdgePosition::Bottom),
        ))
        .child(choice_button(
            "porsche-right",
            "右侧",
            selected == EdgePosition::Right,
            colors,
            cx,
            bind_position(EdgePosition::Right),
        ))
        .child(choice_button(
            "porsche-top",
            "顶部",
            selected == EdgePosition::Top,
            colors,
            cx,
            bind_position(EdgePosition::Top),
        ))
        .child(choice_button(
            "porsche-left",
            "左侧",
            selected == EdgePosition::Left,
            colors,
            cx,
            bind_position(EdgePosition::Left),
        ))
}

fn reset_position(_this: &mut Settings, _window: &mut Window, cx: &mut Context<Settings>) {
    write_cfg(|cfg| cfg.position = PorscheCupConfig::default().position);
    cx.notify();
}

fn reset_width(this: &mut Settings, window: &mut Window, cx: &mut Context<Settings>) {
    let value = PorscheCupConfig::default().width_percent;
    write_cfg(|cfg| cfg.width_percent = value);
    this.shift_lights.porsche_cup.width.update(cx, |field, cx| {
        field.set_value(value as f32, window, cx);
    });
    cx.notify();
}

fn reset_diameter(this: &mut Settings, window: &mut Window, cx: &mut Context<Settings>) {
    let value = PorscheCupConfig::default().diameter_px;
    write_cfg(|cfg| cfg.diameter_px = value);
    this.shift_lights
        .porsche_cup
        .diameter
        .update(cx, |input, cx| {
            input.set_value(value.to_string(), window, cx);
        });
    cx.notify();
}

fn reset_gap(this: &mut Settings, window: &mut Window, cx: &mut Context<Settings>) {
    let value = PorscheCupConfig::default().gap_px;
    write_cfg(|cfg| cfg.gap_px = value);
    this.shift_lights
        .porsche_cup
        .gap
        .update(cx, |input, cx| {
            input.set_value(value.to_string(), window, cx);
        });
    cx.notify();
}

fn reset_offset(this: &mut Settings, window: &mut Window, cx: &mut Context<Settings>) {
    let value = PorscheCupConfig::default().offset_px;
    write_cfg(|cfg| cfg.offset_px = value);
    this.shift_lights
        .porsche_cup
        .offset
        .update(cx, |input, cx| {
            input.set_value(value.to_string(), window, cx);
        });
    cx.notify();
}

fn reset_blink_percent(this: &mut Settings, window: &mut Window, cx: &mut Context<Settings>) {
    let value = PorscheCupConfig::default().blink_percent;
    write_cfg(|cfg| cfg.blink_percent = value);
    this.shift_lights.porsche_cup.blink_percent.update(cx, |field, cx| {
        field.set_value(value, window, cx);
    });
    cx.notify();
}

fn reset_blink_ms(this: &mut Settings, window: &mut Window, cx: &mut Context<Settings>) {
    let value = PorscheCupConfig::default().blink_ms;
    write_cfg(|cfg| cfg.blink_ms = value);
    this.shift_lights.porsche_cup.blink_ms.update(cx, |field, cx| {
        field.set_value(value as f32, window, cx);
    });
    cx.notify();
}

fn reset_lit_opacity(this: &mut Settings, window: &mut Window, cx: &mut Context<Settings>) {
    let value = PorscheCupConfig::default().lit_opacity;
    write_cfg(|cfg| cfg.lit_opacity = value);
    this.shift_lights.porsche_cup.lit_opacity.update(cx, |field, cx| {
        field.set_value(opacity_as_percent(value), window, cx);
    });
    cx.notify();
}

fn reset_dim_opacity(this: &mut Settings, window: &mut Window, cx: &mut Context<Settings>) {
    let value = PorscheCupConfig::default().dim_opacity;
    write_cfg(|cfg| cfg.dim_opacity = value);
    this.shift_lights.porsche_cup.dim_opacity.update(cx, |field, cx| {
        field.set_value(opacity_as_percent(value), window, cx);
    });
    cx.notify();
}
