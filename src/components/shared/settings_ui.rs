use gpui_kit::{div, prelude::*, px, relative, Context, Entity, IntoElement, Window};
use gpui_kit::component::{
    button::{Button, ButtonVariants},
    input::InputState,
    Icon, IconName, Sizable, Size, StyledExt,
    slider::{Slider, SliderState, SliderValue},
};

use crate::app::settings_window::colors::Colors;
use crate::app::settings_window::Settings;

pub fn separator(colors: &Colors) -> gpui_kit::Div {
    div().w_full().h(px(1.)).bg(colors.border)
}

pub fn settings_group(
    colors: &Colors,
    title: &'static str,
    description: &'static str,
) -> gpui_kit::Div {
    div().flex().flex_col().gap_2().child(
        div()
            .flex()
            .flex_col()
            .gap_1()
            .px_1()
            .child(div().text_sm().font_medium().child(title))
            .child(
                div()
                    .text_xs()
                    .text_color(colors.disabled)
                    .child(description),
            ),
    )
}

#[allow(dead_code)]
pub fn setting_row(
    colors: &Colors,
    title: &'static str,
    description: &'static str,
    control: impl IntoElement,
) -> gpui_kit::Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .gap_4()
        .p_4()
        .border_1()
        .border_color(colors.border)
        .rounded_lg()
        .bg(colors.background)
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .gap_1()
                .min_w_0()
                .child(div().text_sm().child(title))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.disabled)
                        .child(description),
                ),
        )
        .child(div().flex_none().child(control))
}

pub fn warning_hint(colors: &Colors, text: &'static str) -> gpui_kit::Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .gap_1()
        .text_xs()
        .line_height(relative(1.))
        .text_color(colors.warning)
        .child(
            div()
                .flex()
                .flex_none()
                .items_center()
                .justify_center()
                .size(px(12.))
                .child(
                    Icon::empty()
                        .path("icons/triangle-alert.svg")
                        .with_size(px(12.))
                        .text_color(colors.warning),
                ),
        )
        .child(text)
}

fn reset_icon_button(
    id: &'static str,
    cx: &mut Context<Settings>,
    on_reset: impl Fn(&mut Settings, &mut Window, &mut Context<Settings>) + 'static,
) -> Button {
    Button::new(id)
        .ghost()
        .compact()
        .with_size(Size::XSmall)
        .flex_none()
        .cursor_pointer()
        .tooltip("重置")
        .icon(Icon::new(IconName::Undo2))
        .on_click(cx.listener(move |this, _, window, cx| {
            on_reset(this, window, cx);
        }))
}

fn title_with_reset(
    title: &'static str,
    reset_id: &'static str,
    show_reset: bool,
    cx: &mut Context<Settings>,
    on_reset: impl Fn(&mut Settings, &mut Window, &mut Context<Settings>) + 'static,
) -> gpui_kit::Div {
    div()
        .flex()
        .items_center()
        .gap_1()
        .flex_none()
        .h(px(20.))
        .child(div().text_sm().line_height(relative(1.)).child(title))
        .when(show_reset, |el| {
            el.child(reset_icon_button(reset_id, cx, on_reset))
        })
}

pub fn setting_row_with_reset(
    colors: &Colors,
    title: &'static str,
    description: &'static str,
    reset_id: &'static str,
    control: impl IntoElement,
    show_reset: bool,
    warning: Option<&'static str>,
    cx: &mut Context<Settings>,
    on_reset: impl Fn(&mut Settings, &mut Window, &mut Context<Settings>) + 'static,
) -> gpui_kit::Div {
    div()
        .flex()
        .flex_row()
        .items_center()
        .justify_between()
        .gap_4()
        .p_4()
        .border_1()
        .border_color(colors.border)
        .rounded_lg()
        .bg(colors.background)
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .gap_1()
                .min_w_0()
                .child(title_with_reset(title, reset_id, show_reset, cx, on_reset))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.disabled)
                        .child(description),
                )
                .when_some(warning, |el, text| el.child(warning_hint(colors, text))),
        )
        .child(div().flex_none().child(control))
}

fn slider_value_width(suffix: &str, decimal_places: usize) -> gpui_kit::Pixels {
    match (suffix, decimal_places) {
        ("%", 0) => px(44.),
        ("%", _) => px(58.),
        (_, 0) => px(44.),
        _ => px(48.),
    }
}

pub fn slider_row(
    title: &'static str,
    description: &'static str,
    reset_id: &'static str,
    slider: &Entity<SliderState>,
    colors: &Colors,
    warning: Option<&'static str>,
    suffix: &'static str,
    decimal_places: usize,
    show_reset: bool,
    cx: &mut Context<Settings>,
    on_reset: impl Fn(&mut Settings, &mut Window, &mut Context<Settings>) + 'static,
) -> gpui_kit::Div {
    let value = match slider.read(cx).value() {
        SliderValue::Single(value) => value,
        SliderValue::Range(start, _) => start,
    };
    let value_label = format!("{value:.prec$}{suffix}", prec = decimal_places);
    div()
        .flex()
        .items_center()
        .justify_between()
        .gap_4()
        .p_4()
        .border_1()
        .border_color(colors.border)
        .rounded_lg()
        .bg(colors.background)
        .child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .min_w_0()
                .child(title_with_reset(title, reset_id, show_reset, cx, on_reset))
                .child(
                    div()
                        .text_xs()
                        .text_color(colors.disabled)
                        .child(description),
                )
                .when_some(warning, |el, text| el.child(warning_hint(colors, text))),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap_3()
                .w(px(232.))
                .child(Slider::new(slider).horizontal())
                .child(
                    div()
                        .flex_none()
                        .w(slider_value_width(suffix, decimal_places))
                        .whitespace_nowrap()
                        .text_sm()
                        .text_right()
                        .text_color(colors.disabled)
                        .child(value_label),
                ),
        )
}

pub fn switch_control<T>(
    id: &'static str,
    checked: bool,
    colors: &Colors,
    cx: &mut Context<Settings>,
    on_toggle: T,
) -> impl IntoElement + use<T>
where
    T: Fn(&mut Settings, bool, &mut Context<Settings>) + 'static,
{
    let track_bg = if checked {
        colors.selected
    } else {
        colors.border
    };
    div()
        .id(id)
        .flex_none()
        .relative()
        .w(px(36.))
        .h(px(20.))
        .rounded_full()
        .bg(track_bg)
        .cursor_pointer()
        .child(
            div()
                .absolute()
                .top(px(2.))
                .when(checked, |el| el.right(px(2.)))
                .when(!checked, |el| el.left(px(2.)))
                .size(px(16.))
                .rounded_full()
                .bg(colors.selected_text),
        )
        .hover(|style| style.opacity(0.9))
        .active(|style| style.opacity(0.8))
        .on_click(cx.listener(move |this, _, _, cx| {
            on_toggle(this, !checked, cx);
        }))
}

pub fn action_button(
    id: &'static str,
    label: &'static str,
    primary: bool,
    cx: &mut Context<Settings>,
    on_click: impl Fn(&mut Settings, &mut Window, &mut Context<Settings>) + 'static,
) -> impl IntoElement {
    let button = Button::new(id)
        .with_size(Size::Small)
        .cursor_pointer()
        .icon(Icon::empty().path("icons/corner-down-left.svg"))
        .child(div().text_sm().child(label));

    let button = if primary {
        button.primary()
    } else {
        button.outline()
    };

    button.on_click(cx.listener(move |this, _, window, cx| {
        on_click(this, window, cx);
    }))
}

pub fn input_differs(
    input: &Entity<InputState>,
    committed: impl ToString,
    cx: &Context<Settings>,
) -> bool {
    input.read(cx).value().trim() != committed.to_string()
}

pub fn input_apply_control(
    input: impl IntoElement,
    unit: Option<&'static str>,
    apply_id: &'static str,
    dirty: bool,
    colors: &Colors,
    cx: &mut Context<Settings>,
    on_apply: impl Fn(&mut Settings, &mut Window, &mut Context<Settings>) + 'static,
) -> gpui_kit::Div {
    div()
        .flex()
        .items_center()
        .gap_2()
        .when(dirty, |el| {
            el.child(action_button(apply_id, "确认", true, cx, on_apply))
        })
        .child(input)
        .when_some(unit, |el, unit| {
            el.child(div().text_xs().text_color(colors.disabled).child(unit))
        })
}
