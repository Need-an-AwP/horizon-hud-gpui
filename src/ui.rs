use gpui::{
    Context, PathBuilder, Render, Window, canvas, div, hsla, point, prelude::*, px, relative, rgb,
    rgba,
};

use crate::config::HISTORY_LEN;
use crate::hud::{CalibrateProgressDirection, RpmHud, ShiftLightsDirection, ShiftLightsPosition};
use crate::telemetry;

fn signed_history_chart(
    samples: Vec<f32>,
    current: f32,
    line_color: gpui::Rgba,
    zero_color: gpui::Rgba,
) -> impl IntoElement {
    canvas(
        move |_, _, _| {},
        move |bounds, _, window, _| {
            let (min, max) = signed_range(&samples, current);
            paint_level_line(bounds, 0.0, min, max, zero_color, window);
            paint_ranged_history_line(&samples, bounds, min, max, line_color, window);
        },
    )
    .w_full()
    .flex_1()
}

fn signed_range(samples: &[f32], current: f32) -> (f32, f32) {
    let mut min = current;
    let mut max = current;
    for &value in samples {
        min = min.min(value);
        max = max.max(value);
    }
    min = min.min(0.0);
    max = max.max(0.0);
    if max - min < 1.0 {
        max = min + 1.0;
    }
    (min, max)
}

fn unsigned_range(samples: &[f32], current: f32) -> (f32, f32) {
    let mut min = current;
    let mut max = current;
    for &value in samples {
        min = min.min(value);
        max = max.max(value);
    }
    min = min.max(0.0);
    let range = max - min;
    if range < 1.0 {
        max = min + 1.0;
    } else {
        let pad = range * 0.05;
        min = (min - pad).max(0.0);
        max += pad;
    }
    (min, max)
}

fn history_chart(
    samples: Vec<f32>,
    current: f32,
    color: gpui::Rgba,
    marker: Option<f32>,
    marker_color: gpui::Rgba,
) -> impl IntoElement {
    canvas(
        move |_, _, _| {},
        move |bounds, _, window, _| {
            let (min, max) = unsigned_range(&samples, current);
            if let Some(value) = marker {
                paint_level_line(bounds, value, min, max, marker_color, window);
            }
            paint_ranged_history_line(&samples, bounds, min, max, color, window);
        },
    )
    .w_full()
    .flex_1()
}

fn value_y(value: f32, min: f32, max: f32, bounds: gpui::Bounds<gpui::Pixels>) -> gpui::Pixels {
    let range = (max - min).max(f32::EPSILON);
    bounds.origin.y + bounds.size.height * (1.0 - (value - min) / range)
}

fn paint_level_line(
    bounds: gpui::Bounds<gpui::Pixels>,
    value: f32,
    min: f32,
    max: f32,
    color: gpui::Rgba,
    window: &mut Window,
) {
    let y = value_y(value, min, max, bounds);
    let mut builder = PathBuilder::stroke(px(1.0));
    builder.move_to(point(bounds.origin.x, y));
    builder.line_to(point(bounds.origin.x + bounds.size.width, y));
    if let Ok(path) = builder.build() {
        window.paint_path(path, color);
    }
}

const RPM_LIGHT_COUNT: usize = 5;
const RPM_LIGHT_BLINK_MS: u128 = 80;
const RPM_LIGHT_COLORS: [u32; RPM_LIGHT_COUNT] = [0x22cc44, 0x22cc44, 0xffcc22, 0xffcc22, 0xff3333];
const RPM_LIGHT_UNCALIBRATED: u32 = 0xffcc22;
const RPM_LIGHT_CALIBRATING: u32 = 0x22cc44;

fn shift_blink_on() -> bool {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| (d.as_millis() / RPM_LIGHT_BLINK_MS) % 2 == 0)
        .unwrap_or(true)
}

fn shift_light_row(gap: f32, cells: impl IntoIterator<Item = impl IntoElement>) -> gpui::Div {
    div()
        .flex()
        .flex_row()
        .w_full()
        .gap(px(gap))
        .children(cells)
}

fn shift_light_strip(
    position: ShiftLightsPosition,
    thickness: f32,
    gap: f32,
    cells: impl IntoIterator<Item = impl IntoElement>,
) -> gpui::Div {
    match position {
        ShiftLightsPosition::Left | ShiftLightsPosition::Right => div()
            .flex()
            .flex_col()
            .h_full()
            .w(px(thickness))
            .gap(px(gap))
            .children(cells),
        ShiftLightsPosition::Bottom | ShiftLightsPosition::Top => shift_light_row(gap, cells),
    }
}

fn shift_light_cell(position: ShiftLightsPosition, thickness: f32, color: gpui::Rgba) -> gpui::Div {
    let cell = div().flex_1().bg(color);
    match position {
        ShiftLightsPosition::Left | ShiftLightsPosition::Right => cell.w(px(thickness)),
        ShiftLightsPosition::Bottom | ShiftLightsPosition::Top => cell.h(px(thickness)),
    }
}

fn with_opacity(color: u32, opacity: f32) -> gpui::Rgba {
    let alpha = (opacity.clamp(0.0, 1.0) * 255.0).round() as u32;
    rgba((color << 8) | alpha)
}

fn calibrate_segment_fill(progress: f32, index: usize) -> f32 {
    let n = RPM_LIGHT_COUNT as f32;
    let start = index as f32 / n;
    let end = (index + 1) as f32 / n;
    ((progress - start) / (end - start).max(f32::EPSILON)).clamp(0.0, 1.0)
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

fn rpm_shift_lights(
    rpm: f32,
    fuel_cut: Option<f32>,
    calibrate: Option<(f32, f32)>,
    _calibrate_direction: CalibrateProgressDirection,
    position: ShiftLightsPosition,
    direction: ShiftLightsDirection,
    lit_opacity: f32,
    dim_opacity: f32,
    blink_frac: f32,
    thickness: f32,
    gap: f32,
) -> impl IntoElement {
    match fuel_cut {
        Some(max) if max > 0.0 => {
            let (on_count, flash) = {
                let ratio = rpm / max;
                if ratio > blink_frac {
                    (RPM_LIGHT_COUNT, true)
                } else {
                    let on = ((ratio / blink_frac) * RPM_LIGHT_COUNT as f32)
                        .floor()
                        .clamp(0.0, RPM_LIGHT_COUNT as f32) as usize;
                    (on, false)
                }
            };
            let lit = !flash || shift_blink_on();
            let reverse = direction.reverses_visual_order();
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
        _ => {
            let progress = match calibrate {
                Some((elapsed, duration)) if duration > 0.0 => (elapsed / duration).clamp(0.0, 1.0),
                _ => 0.0,
            };
            let reverse = direction.reverses_visual_order();
            let vertical = matches!(
                position,
                ShiftLightsPosition::Left | ShiftLightsPosition::Right
            );
            shift_light_strip(
                position,
                thickness,
                gap,
                (0..RPM_LIGHT_COUNT).map(|i| {
                    let rpm_index = if reverse { RPM_LIGHT_COUNT - 1 - i } else { i };
                    let fill = calibrate_segment_fill(progress, rpm_index);
                    calibrate_progress_cell(fill, reverse, vertical, lit_opacity, thickness)
                }),
            )
        }
    }
}

fn paint_ranged_history_line(
    samples: &[f32],
    bounds: gpui::Bounds<gpui::Pixels>,
    min: f32,
    max: f32,
    color: gpui::Rgba,
    window: &mut Window,
) {
    if samples.len() < 2 {
        return;
    }
    let mut builder = PathBuilder::stroke(px(1.5));
    let w = bounds.size.width;
    for (i, value) in samples.iter().enumerate() {
        let x = bounds.origin.x + w * (i as f32 / (HISTORY_LEN - 1) as f32);
        let y = value_y(*value, min, max, bounds);
        let p = point(x, y);
        if i == 0 {
            builder.move_to(p);
        } else {
            builder.line_to(p);
        }
    }
    if let Ok(path) = builder.build() {
        window.paint_path(path, color);
    }
}

impl Render for RpmHud {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let rpm_color = rgb(0xffffff);
        let torque_color = rgb(0x33ccff);
        let zero_line_color = rgb(0xffaa33);
        let power_color = rgb(0x66ff99);
        let rpm_label = format!("{:.0} / {:.0} RPM", self.rpm(), self.max_rpm());
        let calibrate_hint = if self.calibrate_hint_visible() {
            if !self.has_data() {
                Some(format!(
                    "等待数据  UDP {}",
                    telemetry::listen_addr_display()
                ))
            } else if self.fuel_cut_rpm().is_none() {
                if let Some((elapsed, duration)) = self.calibrate_progress() {
                    Some(format!("校准中 {:.1}/{:.1}s", elapsed, duration))
                } else {
                    Some("手刹+油门原地拉转校准".into())
                }
            } else {
                None
            }
        } else {
            None
        };
        let torque_label = if self.has_data() {
            format!("{:.0} Nm", self.torque())
        } else {
            String::new()
        };
        let power_label = if self.has_data() {
            format!("{:.1} kW", self.power() / 1000.0)
        } else {
            String::new()
        };
        let rpm_samples = self.rpm_samples();
        let torque_samples = self.torque_samples();
        let power_samples = self.power_samples();
        let show_charts = self.charts_visible();
        let lights_position = self.shift_lights_position();
        let lights_direction = self.shift_lights_direction();
        let lights_lit_opacity = self.shift_lights_lit_opacity();
        let lights_dim_opacity = self.shift_lights_dim_opacity();
        let lights_offset = px(self.shift_lights_offset_px() as f32);
        let lights_thickness = self.shift_lights_thickness_px() as f32;
        let lights_gap = self.shift_lights_gap_px() as f32;
        let lights_width = relative(self.shift_lights_width_percent() as f32 / 100.0);
        let lights_blink_frac = self.shift_lights_blink_percent() / 100.0;
        let lights_vertical = matches!(
            lights_position,
            ShiftLightsPosition::Left | ShiftLightsPosition::Right
        );
        let lights = div()
            .absolute()
            .flex()
            .when(!lights_vertical, |el| {
                el.left_0().right_0().justify_center()
            })
            .when(lights_vertical, |el| {
                el.top_0()
                    .bottom_0()
                    .flex_col()
                    .items_center()
                    .justify_center()
            })
            .when(lights_position == ShiftLightsPosition::Top, |el| {
                el.top(lights_offset)
            })
            .when(lights_position == ShiftLightsPosition::Bottom, |el| {
                el.bottom(lights_offset)
            })
            .when(lights_position == ShiftLightsPosition::Right, |el| {
                el.right(lights_offset)
            })
            .when(lights_position == ShiftLightsPosition::Left, |el| {
                el.left(lights_offset)
            })
            .child(
                div()
                    .when(!lights_vertical, |el| el.w(lights_width))
                    .when(lights_vertical, |el| {
                        el.h(lights_width).w(px(lights_thickness))
                    })
                    .child(rpm_shift_lights(
                        self.rpm(),
                        self.fuel_cut_rpm(),
                        self.calibrate_progress(),
                        self.calibrate_progress_direction(),
                        lights_position,
                        lights_direction,
                        lights_lit_opacity,
                        lights_dim_opacity,
                        lights_blink_frac,
                        lights_thickness,
                        lights_gap,
                    )),
            );

        div()
            .flex()
            .flex_col()
            .gap_3()
            .bg(hsla(0.0, 0.0, 0.0, 0.0))
            .size_full()
            .p_8()
            .text_xl()
            .when(show_charts, |el| {
                el.child(
                    div()
                        .flex()
                        .flex_row()
                        .gap_3()
                        .w_1_2()
                        .mx_auto()
                        .flex_1()
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .flex_1()
                                .gap_1()
                                .child(div().text_color(torque_color).child(torque_label))
                                .child(signed_history_chart(
                                    torque_samples,
                                    self.torque(),
                                    torque_color,
                                    zero_line_color,
                                )),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .flex_1()
                                .gap_1()
                                .child(div().text_color(power_color).child(power_label))
                                .child(signed_history_chart(
                                    power_samples,
                                    self.power(),
                                    power_color,
                                    zero_line_color,
                                )),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .w_1_2()
                        .mx_auto()
                        .flex_1()
                        .gap_1()
                        .child(div().text_color(rpm_color).child(rpm_label))
                        .child(history_chart(
                            rpm_samples,
                            self.rpm(),
                            rpm_color,
                            self.fuel_cut_rpm(),
                            zero_line_color,
                        )),
                )
            })
            .child(lights)
            .when_some(calibrate_hint, |el, hint| {
                el.child(
                    div()
                        .absolute()
                        .inset_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(rpm_color)
                        .child(hint),
                )
            })
    }
}
