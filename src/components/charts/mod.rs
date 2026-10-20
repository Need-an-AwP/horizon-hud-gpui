use gpui_kit::{
    AnyElement, Bounds, PathBuilder, Pixels, Rgba, Window, canvas, div, point, prelude::*, px, rgb,
};

use crate::core::general_config::HISTORY_LEN;
use crate::core::props::ChartFrame;

pub fn render(frame: &ChartFrame) -> AnyElement {
    let rpm_color = rgb(0xffffff);
    let torque_color = rgb(0x33ccff);
    let zero_line_color = rgb(0xffaa33);
    let power_color = rgb(0x66ff99);
    let rpm_label = format!("{:.0} / {:.0} RPM", frame.rpm_now, frame.max_rpm);
    let torque_label = if frame.has_data {
        format!("{:.0} Nm", frame.torque_now)
    } else {
        String::new()
    };
    let power_label = if frame.has_data {
        format!("{:.1} kW", frame.power_now / 1000.0)
    } else {
        String::new()
    };

    div()
        .flex()
        .flex_col()
        .gap_3()
        .w_full()
        .flex_1()
        .min_h_0()
        .child(
            div()
                .flex()
                .flex_row()
                .gap_3()
                .w_1_2()
                .mx_auto()
                .flex_1()
                .min_h_0()
                .child(chart_with_center_label(
                    signed_history_chart(
                        frame.torque.clone(),
                        frame.torque_now,
                        torque_color,
                        zero_line_color,
                    ),
                    torque_label,
                    torque_color,
                ))
                .child(chart_with_center_label(
                    signed_history_chart(
                        frame.power.clone(),
                        frame.power_now,
                        power_color,
                        zero_line_color,
                    ),
                    power_label,
                    power_color,
                )),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .w_1_2()
                .mx_auto()
                .flex_1()
                .min_h_0()
                .child(chart_with_center_label(
                    history_chart(
                        frame.rpm.clone(),
                        frame.rpm_now,
                        rpm_color,
                        frame.limit_rpm,
                        zero_line_color,
                    ),
                    rpm_label,
                    rpm_color,
                )),
        )
        .into_any_element()
}

fn signed_history_chart(
    samples: Vec<f32>,
    current: f32,
    line_color: Rgba,
    zero_color: Rgba,
) -> impl IntoElement {
    canvas(
        move |_, _, _| {},
        move |bounds, _, window, _| {
            let (min, max) = signed_range(&samples, current);
            paint_level_line(bounds, 0.0, min, max, zero_color, window);
            paint_ranged_history_line(&samples, bounds, min, max, line_color, window);
        },
    )
    .size_full()
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
    color: Rgba,
    marker: Option<f32>,
    marker_color: Rgba,
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
    .size_full()
}

fn chart_with_center_label(
    chart: impl IntoElement,
    label: impl IntoElement,
    color: Rgba,
) -> gpui_kit::Div {
    div()
        .relative()
        .flex()
        .flex_col()
        .flex_1()
        .w_full()
        .min_h_0()
        .child(div().flex_1().w_full().min_h_0().child(chart))
        .child(
            div()
                .absolute()
                .inset_0()
                .flex()
                .items_center()
                .justify_center()
                .text_color(color)
                .child(label),
        )
}

fn value_y(value: f32, min: f32, max: f32, bounds: Bounds<Pixels>) -> Pixels {
    let range = (max - min).max(f32::EPSILON);
    bounds.origin.y + bounds.size.height * (1.0 - (value - min) / range)
}

fn paint_level_line(
    bounds: Bounds<Pixels>,
    value: f32,
    min: f32,
    max: f32,
    color: Rgba,
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

fn paint_ranged_history_line(
    samples: &[f32],
    bounds: Bounds<Pixels>,
    min: f32,
    max: f32,
    color: Rgba,
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
