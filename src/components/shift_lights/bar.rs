use gpui::{div, prelude::*, px, relative, Context, Entity, IntoElement, Window};
use gpui_component::{
    input::{Input, InputEvent, InputState},
    slider::{SliderEvent, SliderState, SliderValue},
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
use crate::components::shared::settings_ui::{
    input_apply_control, input_differs, separator, setting_row_with_reset, settings_group, slider_row,
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
    pub lit_opacity: Entity<SliderState>,
    pub dim_opacity: Entity<SliderState>,
    pub offset: Entity<InputState>,
    pub thickness: Entity<InputState>,
    pub gap: Entity<InputState>,
    pub width: Entity<SliderState>,
    pub blink_percent: Entity<SliderState>,
    pub blink_ms: Entity<SliderState>,
}

impl SettingsControls {
    pub fn new(cfg: &BarConfig, window: &mut Window, cx: &mut Context<Settings>) -> Self {
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
                write_bar(|cfg| cfg.lit_opacity = value);
            }
        });
        subscribe_slider(&dim_opacity, cx, |value| {
            if value.is_finite() {
                let value = value.clamp(0.0, 1.0);
                write_bar(|cfg| cfg.dim_opacity = value);
            }
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
                write_bar(|cfg| cfg.width_percent = width);
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
                write_bar(|cfg| cfg.blink_percent = value);
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
                write_bar(|cfg| cfg.blink_ms = blink_ms);
            }
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
        self.lit_opacity.update(cx, |slider, cx| {
            slider.set_value(cfg.lit_opacity, window, cx);
        });
        self.dim_opacity.update(cx, |slider, cx| {
            slider.set_value(cfg.dim_opacity, window, cx);
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
        self.width.update(cx, |slider, cx| {
            slider.set_value(cfg.width_percent as f32, window, cx);
        });
        self.blink_percent.update(cx, |slider, cx| {
            slider.set_value(cfg.blink_percent, window, cx);
        });
        self.blink_ms.update(cx, |slider, cx| {
            slider.set_value(cfg.blink_ms as f32, window, cx);
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

pub fn render_hud(props: &HudTelemetryProps, cfg: &BarConfig) -> gpui::AnyElement {
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
) -> gpui::AnyElement {
    let cfg = crate::session::shift_lights().bar;
    let defaults = BarConfig::default();
    let controls = &view.shift_lights.bar;
    let dim_warning = (!crate::core::calibration::shift_lights_calibrated())
        .then_some("当前处于未校准状态，熄灭透明度可能不可见");
    let direction_description = if cfg.position.is_vertical() {
        "纵向灯条：下→上为默认方向。"
    } else {
        "横向灯条：左→右为默认方向。"
    };

    let position_control = position_control(cfg.position, colors, cx);
    let direction_control = direction_control(cfg.position, cfg.direction, colors, cx);
    let thickness_control = input_apply_control(
        Input::new(&controls.thickness)
            .w(px(96.))
            .with_size(Size::Small),
        Some("px"),
        "bar-thickness-apply",
        input_differs(&controls.thickness, cfg.thickness_px, cx),
        colors,
        cx,
        |this, _, cx| this.shift_lights.bar.apply_thickness(cx),
    );
    let gap_control = input_apply_control(
        Input::new(&controls.gap).w(px(96.)).with_size(Size::Small),
        Some("px"),
        "bar-gap-apply",
        input_differs(&controls.gap, cfg.gap_px, cx),
        colors,
        cx,
        |this, _, cx| this.shift_lights.bar.apply_gap(cx),
    );
    let offset_control = input_apply_control(
        Input::new(&controls.offset).w(px(96.)).with_size(Size::Small),
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
                "选择转速灯样式、贴边位置，以及转速升高时的点亮方向。",
            )
            .child(setting_row_with_reset(
                colors,
                "位置",
                "可放在屏幕顶部、底部、左侧或右侧。",
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
                "调整灯条占屏幕的比例、厚度、灯格间距，以及离边缘的距离。",
            )
            .child(slider_row(
                "整体宽度",
                "横向灯条的宽度、纵向灯条的高度，占屏幕对应边的百分比；默认 75%。",
                "bar-width-reset",
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
                "灯条厚度",
                "横向灯条的高度、纵向灯条的宽度，单位像素；默认 40 px。",
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
                "相邻灯格之间的像素距离；可为 0，默认 8 px。",
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
                "0 表示紧贴边缘；正整数表示向屏幕内侧偏移的像素数。",
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
                "控制灯条闪烁时机与速度，以及亮起与未亮起灯格的透明度。",
            )
            .child(slider_row(
                "闪烁阈值",
                "达到校准转速的该百分比后，整条灯条开始闪烁；范围 80% 到 100%，步进 0.1%；默认 95%。",
                "bar-blink-reset",
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
                "bar-blink-ms-reset",
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
                "亮起的转速灯透明度，范围 0 到 1；默认 0.7。",
                "bar-lit-opacity-reset",
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
                "未亮起的转速灯透明度，范围 0 到 1。",
                "bar-dim-opacity-reset",
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

fn write_bar(f: impl FnOnce(&mut BarConfig)) {
    crate::session::update_shift_lights(|settings| f(&mut settings.bar));
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
) -> gpui::Div {
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

fn shift_light_cell(position: EdgePosition, thickness: f32, color: gpui::Rgba) -> gpui::Div {
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
) -> gpui::Div {
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

fn segmented_row(colors: &Colors) -> gpui::Div {
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
) -> gpui::Div {
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
) -> gpui::Div {
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
    this.shift_lights.bar.width.update(cx, |slider, cx| {
        slider.set_value(value as f32, window, cx);
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
    this.shift_lights
        .bar
        .blink_percent
        .update(cx, |slider, cx| {
            slider.set_value(value, window, cx);
        });
    cx.notify();
}

fn reset_blink_ms(this: &mut Settings, window: &mut Window, cx: &mut Context<Settings>) {
    let value = BarConfig::default().blink_ms;
    write_bar(|cfg| cfg.blink_ms = value);
    this.shift_lights.bar.blink_ms.update(cx, |slider, cx| {
        slider.set_value(value as f32, window, cx);
    });
    cx.notify();
}

fn reset_lit_opacity(this: &mut Settings, window: &mut Window, cx: &mut Context<Settings>) {
    let value = BarConfig::default().lit_opacity;
    write_bar(|cfg| cfg.lit_opacity = value);
    this.shift_lights
        .bar
        .lit_opacity
        .update(cx, |slider, cx| {
            slider.set_value(value, window, cx);
        });
    cx.notify();
}

fn reset_dim_opacity(this: &mut Settings, window: &mut Window, cx: &mut Context<Settings>) {
    let value = BarConfig::default().dim_opacity;
    write_bar(|cfg| cfg.dim_opacity = value);
    this.shift_lights
        .bar
        .dim_opacity
        .update(cx, |slider, cx| {
            slider.set_value(value, window, cx);
        });
    cx.notify();
}
