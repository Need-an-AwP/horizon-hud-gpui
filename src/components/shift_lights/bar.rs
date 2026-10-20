use gpui_kit::{div, prelude::*, px, relative, Context, Entity, IntoElement, Window};
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

const DEFAULT_THICKNESS_PX: usize = 40;
const DEFAULT_GAP_PX: usize = 8;
const DEFAULT_WIDTH_PERCENT: usize = 75;
const DEFAULT_OFFSET_PX: usize = 0;
const DEFAULT_LIT_OPACITY: f32 = 0.7;
const DEFAULT_DIM_OPACITY: f32 = 0.5;
const DEFAULT_BLINK_PERCENT: f32 = 95.0;
const BLINK_PERCENT_MIN: f32 = 80.0;
const BLINK_PERCENT_MAX: f32 = 100.0;
const DEFAULT_BLINK_MS: usize = 80;
const BLINK_MS_MIN: usize = 20;
const BLINK_MS_MAX: usize = 200;

const RPM_LIGHT_COUNT: usize = 5;
const RPM_LIGHT_COLORS: [u32; RPM_LIGHT_COUNT] = [0x22cc44, 0x22cc44, 0xffcc22, 0xffcc22, 0xff3333];
const RPM_LIGHT_UNCALIBRATED: u32 = 0xffcc22;
const RPM_LIGHT_CALIBRATING: u32 = 0x22cc44;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BarDirection {
    LeftToRight,
    RightToLeft,
    BottomToTop,
    TopToBottom,
}

impl BarDirection {
    pub fn default_for(position: EdgePosition) -> Self {
        if position.is_vertical() {
            Self::BottomToTop
        } else {
            Self::LeftToRight
        }
    }

    pub fn is_compatible_with(self, position: EdgePosition) -> bool {
        matches!(
            (position.is_vertical(), self),
            (false, Self::LeftToRight | Self::RightToLeft)
                | (true, Self::BottomToTop | Self::TopToBottom)
        )
    }

    pub fn reverses_visual_order(self) -> bool {
        matches!(self, Self::RightToLeft | Self::BottomToTop)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct BarConfig {
    pub position: EdgePosition,
    pub direction: BarDirection,
    pub width_percent: usize,
    pub thickness_px: usize,
    pub gap_px: usize,
    pub offset_px: usize,
    pub lit_opacity: f32,
    pub dim_opacity: f32,
    pub blink_percent: f32,
    pub blink_ms: usize,
}

impl Default for BarConfig {
    fn default() -> Self {
        let position = EdgePosition::Top;
        Self {
            position,
            direction: BarDirection::default_for(position),
            width_percent: DEFAULT_WIDTH_PERCENT,
            thickness_px: DEFAULT_THICKNESS_PX,
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
    pub thickness: Entity<InputState>,
    pub gap: Entity<InputState>,
    pub width: Entity<NumberFieldState>,
    pub blink_percent: Entity<NumberFieldState>,
    pub blink_ms: Entity<NumberFieldState>,
}

impl SettingsControls {
    pub fn new(cfg: &BarConfig, window: &mut Window, cx: &mut Context<Settings>) -> Self {
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
            write_bar(|cfg| cfg.lit_opacity = opacity_from_percent(value));
        });
        subscribe_number_field(&dim_opacity, cx, |value| {
            write_bar(|cfg| cfg.dim_opacity = opacity_from_percent(value));
        });

        let offset = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(DEFAULT_OFFSET_PX.to_string())
                .default_value(cfg.offset_px.to_string())
        });
        let thickness = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(DEFAULT_THICKNESS_PX.to_string())
                .default_value(cfg.thickness_px.to_string())
        });
        let gap = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(DEFAULT_GAP_PX.to_string())
                .default_value(cfg.gap_px.to_string())
        });
        subscribe_input(&offset, window, cx, SettingsControls::apply_offset);
        subscribe_input(&thickness, window, cx, SettingsControls::apply_thickness);
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
            write_bar(|cfg| cfg.width_percent = width);
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
            write_bar(|cfg| cfg.blink_percent = value);
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
            write_bar(|cfg| cfg.blink_ms = blink_ms);
        });

        Self {
            lit_opacity,
            dim_opacity,
            offset,
            thickness,
            gap,
            width,
            blink_percent,
            blink_ms,
        }
    }

    pub fn sync(&self, cfg: &BarConfig, window: &mut Window, cx: &mut Context<Settings>) {
        self.lit_opacity.update(cx, |field, cx| {
            field.set_value(opacity_as_percent(cfg.lit_opacity), window, cx);
        });
        self.dim_opacity.update(cx, |field, cx| {
            field.set_value(opacity_as_percent(cfg.dim_opacity), window, cx);
        });
        self.offset.update(cx, |input, cx| {
            input.set_value(cfg.offset_px.to_string(), window, cx);
        });
        self.thickness.update(cx, |input, cx| {
            input.set_value(cfg.thickness_px.to_string(), window, cx);
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
            write_bar(|cfg| cfg.offset_px = offset);
        }
        cx.notify();
    }

    pub fn apply_thickness(&self, cx: &mut Context<Settings>) {
        let value = self.thickness.read(cx).value().to_string();
        if let Ok(thickness) = value.trim().parse::<usize>() {
            if thickness > 0 {
                write_bar(|cfg| cfg.thickness_px = thickness);
            }
        }
        cx.notify();
    }

    pub fn apply_gap(&self, cx: &mut Context<Settings>) {
        let value = self.gap.read(cx).value().to_string();
        if let Ok(gap) = value.trim().parse::<usize>() {
            write_bar(|cfg| cfg.gap_px = gap);
        }
        cx.notify();
    }
}

pub fn render_hud(props: &HudTelemetryProps, cfg: &BarConfig) -> gpui_kit::AnyElement {
    let position = cfg.position;
    let thickness = cfg.thickness_px as f32;
    let gap = cfg.gap_px as f32;
    let lit_opacity = cfg.lit_opacity;
    let dim_opacity = cfg.dim_opacity;
    let reverse = cfg.direction.reverses_visual_order();
    let strip = match props.calibration {
        CalibrationState::Ready { rpm_ratio, .. } => {
            let blink_frac = cfg.blink_percent / 100.0;
            let (on_count, flash) = calc_linear_lights(rpm_ratio, blink_frac, RPM_LIGHT_COUNT);
            let lit = !flash || is_blink_lit(props.now_ms, cfg.blink_ms);
            shift_light_strip(
                position,
                thickness,
                gap,
                (0..RPM_LIGHT_COUNT).map(move |i| {
                    let rpm_index = if reverse { RPM_LIGHT_COUNT - 1 - i } else { i };
                    let color = if lit && rpm_index < on_count {
                        with_opacity(RPM_LIGHT_COLORS[rpm_index], lit_opacity)
                    } else {
                        with_opacity(0x2a2a2a, dim_opacity)
                    };
                    shift_light_cell(position, thickness, color)
                }),
            )
        }
        CalibrationState::Uncalibrated { progress, .. } => {
            let vertical = position.is_vertical();
            shift_light_strip(
                position,
                thickness,
                gap,
                (0..RPM_LIGHT_COUNT).map(move |i| {
                    let rpm_index = if reverse { RPM_LIGHT_COUNT - 1 - i } else { i };
                    let fill = calibrate_segment_fill(progress, rpm_index, RPM_LIGHT_COUNT);
                    calibrate_progress_cell(fill, reverse, vertical, lit_opacity, thickness)
                }),
            )
        }
    };
    edge_strip_container(
        position,
        cfg.width_percent,
        cfg.thickness_px,
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
    let cfg = crate::session::shift_lights().bar;
    let defaults = BarConfig::default();
    let controls = &view.shift_lights.bar;
    let dim_warning = (!crate::core::calibration::shift_lights_calibrated())
        .then_some("当前处于未校准状态，熄灭透明度不可见");
    let direction_description = if cfg.position.is_vertical() {
        "默认自下而上。"
    } else {
        "默认自左向右。"
    };

    let position_control = position_control(cfg.position, colors, cx);
    let direction_control = direction_control(cfg.position, cfg.direction, colors, cx);
    let thickness_control = input_apply_control(
        Input::new(&controls.thickness)
            .w(px(96.))
            .with_size(Size::Medium),
        Some("px"),
        "bar-thickness-apply",
        input_differs(&controls.thickness, cfg.thickness_px, cx),
        colors,
        cx,
        |this, _, cx| this.shift_lights.bar.apply_thickness(cx),
    );
    let gap_control = input_apply_control(
        Input::new(&controls.gap).w(px(96.)).with_size(Size::Medium),
        Some("px"),
        "bar-gap-apply",
        input_differs(&controls.gap, cfg.gap_px, cx),
        colors,
        cx,
        |this, _, cx| this.shift_lights.bar.apply_gap(cx),
    );
    let offset_control = input_apply_control(
        Input::new(&controls.offset).w(px(96.)).with_size(Size::Medium),
        Some("px"),
        "bar-offset-apply",
        input_differs(&controls.offset, cfg.offset_px, cx),
        colors,
        cx,
        |this, _, cx| this.shift_lights.bar.apply_offset(cx),
    );
    div()
        .flex()
        .flex_col()
        .gap_5()
        .child(
            settings_group(
                colors,
                "样式与位置",
                "贴边位置与点亮方向。",
            )
            .child(setting_row_with_reset(
                colors,
                "位置",
                "上、下、左、右。",
                "bar-position-reset",
                position_control,
                cfg.position != defaults.position,
                None,
                cx,
                reset_position,
            ))
            .child(setting_row_with_reset(
                colors,
                "方向",
                direction_description,
                "bar-direction-reset",
                direction_control,
                cfg.direction != BarDirection::default_for(cfg.position),
                None,
                cx,
                reset_direction,
            )),
        )
        .child(separator(colors))
        .child(
            settings_group(
                colors,
                "尺寸与布局",
                "宽度、厚度、间隔与偏移。",
            )
            .child(setting_row_with_reset(
                colors,
                "整体宽度",
                "百分比，范围 1–100，步进 1，默认 75。",
                "bar-width-reset",
                NumberField::new(&controls.width).unit_color(colors.disabled),
                cfg.width_percent != defaults.width_percent,
                None,
                cx,
                reset_width,
            ))
            .child(setting_row_with_reset(
                colors,
                "灯条厚度",
                "正整数，单位 px，默认 40。",
                "bar-thickness-reset",
                thickness_control,
                cfg.thickness_px != defaults.thickness_px,
                None,
                cx,
                reset_thickness,
            ))
            .child(setting_row_with_reset(
                colors,
                "灯格间隔",
                "非负整数，单位 px，默认 8。",
                "bar-gap-reset",
                gap_control,
                cfg.gap_px != defaults.gap_px,
                None,
                cx,
                reset_gap,
            ))
            .child(setting_row_with_reset(
                colors,
                "边缘偏移",
                "非负整数，单位 px，默认 0。",
                "bar-offset-reset",
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
                "bar-blink-reset",
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
                "bar-blink-ms-reset",
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
                "bar-lit-opacity-reset",
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
                "bar-dim-opacity-reset",
                NumberField::new(&controls.dim_opacity).unit_color(colors.disabled),
                cfg.dim_opacity != defaults.dim_opacity,
                dim_warning,
                cx,
                reset_dim_opacity,
            )),
        )
        .into_any_element()
}

fn write_bar(f: impl FnOnce(&mut BarConfig)) {
    crate::session::update_shift_lights(|settings| f(&mut settings.bar));
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
            InputEvent::PressEnter { .. } => apply(&this.shift_lights.bar, cx),
            _ => {}
        },
    )
    .detach();
}

fn shift_light_strip(
    position: EdgePosition,
    thickness: f32,
    gap: f32,
    cells: impl IntoIterator<Item = impl IntoElement>,
) -> gpui_kit::Div {
    match position {
        EdgePosition::Left | EdgePosition::Right => div()
            .flex()
            .flex_col()
            .h_full()
            .w(px(thickness))
            .gap(px(gap))
            .children(cells),
        EdgePosition::Bottom | EdgePosition::Top => div()
            .flex()
            .flex_row()
            .w_full()
            .gap(px(gap))
            .children(cells),
    }
}

fn shift_light_cell(position: EdgePosition, thickness: f32, color: gpui_kit::Rgba) -> gpui_kit::Div {
    let cell = div().flex_1().bg(color);
    if position.is_vertical() {
        cell.w(px(thickness))
    } else {
        cell.h(px(thickness))
    }
}

fn calibrate_progress_cell(
    fill: f32,
    reverse: bool,
    vertical: bool,
    lit_opacity: f32,
    thickness: f32,
) -> gpui_kit::Div {
    let base = div()
        .flex_1()
        .overflow_hidden()
        .bg(with_opacity(RPM_LIGHT_UNCALIBRATED, lit_opacity));
    let base = if vertical {
        base.w(px(thickness))
    } else {
        base.h(px(thickness))
    };
    if vertical && reverse {
        base.child(
            div().flex().flex_col().size_full().justify_end().child(
                div()
                    .w_full()
                    .h(relative(fill))
                    .bg(with_opacity(RPM_LIGHT_CALIBRATING, lit_opacity)),
            ),
        )
    } else if vertical {
        base.child(
            div()
                .w_full()
                .h(relative(fill))
                .bg(with_opacity(RPM_LIGHT_CALIBRATING, lit_opacity)),
        )
    } else if reverse {
        base.child(
            div().flex().h_full().w_full().justify_end().child(
                div()
                    .h_full()
                    .w(relative(fill))
                    .bg(with_opacity(RPM_LIGHT_CALIBRATING, lit_opacity)),
            ),
        )
    } else {
        base.child(
            div()
                .h_full()
                .w(relative(fill))
                .bg(with_opacity(RPM_LIGHT_CALIBRATING, lit_opacity)),
        )
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
        write_bar(|cfg| {
            cfg.position = position;
            cfg.direction = BarDirection::default_for(position);
        });
        cx.notify();
    }
}

fn bind_direction(
    direction: BarDirection,
) -> impl Fn(&mut Settings, &mut Window, &mut Context<Settings>) + 'static {
    move |_, _, cx| {
        crate::session::update_shift_lights(|settings| {
            if direction.is_compatible_with(settings.bar.position) {
                settings.bar.direction = direction;
            }
        });
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
            "bar-bottom",
            "底部",
            selected == EdgePosition::Bottom,
            colors,
            cx,
            bind_position(EdgePosition::Bottom),
        ))
        .child(choice_button(
            "bar-right",
            "右侧",
            selected == EdgePosition::Right,
            colors,
            cx,
            bind_position(EdgePosition::Right),
        ))
        .child(choice_button(
            "bar-top",
            "顶部",
            selected == EdgePosition::Top,
            colors,
            cx,
            bind_position(EdgePosition::Top),
        ))
        .child(choice_button(
            "bar-left",
            "左侧",
            selected == EdgePosition::Left,
            colors,
            cx,
            bind_position(EdgePosition::Left),
        ))
}

fn direction_control(
    position: EdgePosition,
    selected: BarDirection,
    colors: &Colors,
    cx: &mut Context<Settings>,
) -> gpui_kit::Div {
    let (first_id, first_label, first_direction, second_id, second_label, second_direction) =
        if position.is_vertical() {
            (
                "bar-bottom-to-top",
                "下 → 上",
                BarDirection::BottomToTop,
                "bar-top-to-bottom",
                "上 → 下",
                BarDirection::TopToBottom,
            )
        } else {
            (
                "bar-left-to-right",
                "左 → 右",
                BarDirection::LeftToRight,
                "bar-right-to-left",
                "右 → 左",
                BarDirection::RightToLeft,
            )
        };
    segmented_row(colors)
        .child(choice_button(
            first_id,
            first_label,
            selected == first_direction,
            colors,
            cx,
            bind_direction(first_direction),
        ))
        .child(choice_button(
            second_id,
            second_label,
            selected == second_direction,
            colors,
            cx,
            bind_direction(second_direction),
        ))
}

fn reset_position(_this: &mut Settings, _window: &mut Window, cx: &mut Context<Settings>) {
    let position = BarConfig::default().position;
    write_bar(|cfg| {
        cfg.position = position;
        cfg.direction = BarDirection::default_for(position);
    });
    cx.notify();
}

fn reset_direction(_this: &mut Settings, _window: &mut Window, cx: &mut Context<Settings>) {
    crate::session::update_shift_lights(|settings| {
        settings.bar.direction = BarDirection::default_for(settings.bar.position);
    });
    cx.notify();
}

fn reset_width(this: &mut Settings, window: &mut Window, cx: &mut Context<Settings>) {
    let value = BarConfig::default().width_percent;
    write_bar(|cfg| cfg.width_percent = value);
    this.shift_lights.bar.width.update(cx, |field, cx| {
        field.set_value(value as f32, window, cx);
    });
    cx.notify();
}

fn reset_thickness(this: &mut Settings, window: &mut Window, cx: &mut Context<Settings>) {
    let value = BarConfig::default().thickness_px;
    write_bar(|cfg| cfg.thickness_px = value);
    this.shift_lights.bar.thickness.update(cx, |input, cx| {
        input.set_value(value.to_string(), window, cx);
    });
    cx.notify();
}

fn reset_gap(this: &mut Settings, window: &mut Window, cx: &mut Context<Settings>) {
    let value = BarConfig::default().gap_px;
    write_bar(|cfg| cfg.gap_px = value);
    this.shift_lights.bar.gap.update(cx, |input, cx| {
        input.set_value(value.to_string(), window, cx);
    });
    cx.notify();
}

fn reset_offset(this: &mut Settings, window: &mut Window, cx: &mut Context<Settings>) {
    let value = BarConfig::default().offset_px;
    write_bar(|cfg| cfg.offset_px = value);
    this.shift_lights.bar.offset.update(cx, |input, cx| {
        input.set_value(value.to_string(), window, cx);
    });
    cx.notify();
}

fn reset_blink_percent(this: &mut Settings, window: &mut Window, cx: &mut Context<Settings>) {
    let value = BarConfig::default().blink_percent;
    write_bar(|cfg| cfg.blink_percent = value);
    this.shift_lights.bar.blink_percent.update(cx, |field, cx| {
        field.set_value(value, window, cx);
    });
    cx.notify();
}

fn reset_blink_ms(this: &mut Settings, window: &mut Window, cx: &mut Context<Settings>) {
    let value = BarConfig::default().blink_ms;
    write_bar(|cfg| cfg.blink_ms = value);
    this.shift_lights.bar.blink_ms.update(cx, |field, cx| {
        field.set_value(value as f32, window, cx);
    });
    cx.notify();
}

fn reset_lit_opacity(this: &mut Settings, window: &mut Window, cx: &mut Context<Settings>) {
    let value = BarConfig::default().lit_opacity;
    write_bar(|cfg| cfg.lit_opacity = value);
    this.shift_lights.bar.lit_opacity.update(cx, |field, cx| {
        field.set_value(opacity_as_percent(value), window, cx);
    });
    cx.notify();
}

fn reset_dim_opacity(this: &mut Settings, window: &mut Window, cx: &mut Context<Settings>) {
    let value = BarConfig::default().dim_opacity;
    write_bar(|cfg| cfg.dim_opacity = value);
    this.shift_lights.bar.dim_opacity.update(cx, |field, cx| {
        field.set_value(opacity_as_percent(value), window, cx);
    });
    cx.notify();
}
