use gpui::{
    Context, Entity, IntoElement, Window, colors::Colors, div, prelude::*, px, relative, rgb,
};
use gpui_component::{
    Icon, IconName, Sizable, Size, StyledExt,
    button::{Button, ButtonVariants},
    slider::{Slider, SliderState, SliderValue},
};

use crate::hud::{self, ShiftLightsDirection, ShiftLightsPosition};
use crate::platform;
use crate::settings_section::SettingsSection;

use super::Settings;

impl Settings {
    pub(super) fn section_navigation(
        &self,
        section: SettingsSection,
        colors: &Colors,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let selected = self.selected_section == section;
        let icon_color = if selected {
            colors.selected_text
        } else {
            colors.text
        };

        div()
            .id(("settings-section", section.index()))
            .flex()
            .items_center()
            .gap_2()
            .w_full()
            .p_2()
            .rounded_md()
            .cursor_pointer()
            .text_sm()
            .text_color(icon_color)
            .when(selected, |el| el.bg(colors.selected))
            .hover(|style| {
                if selected {
                    style.opacity(0.88)
                } else {
                    style.bg(colors.container)
                }
            })
            .active(|style| style.opacity(0.8))
            .child(
                Icon::empty()
                    .path(section.icon_path())
                    .with_size(px(17.))
                    .text_color(icon_color),
            )
            .child(section.label())
            .on_click(cx.listener(move |this, _, window, cx| {
                this.select_section(section, window, cx);
            }))
    }

    pub(super) fn exit_button(&self) -> impl IntoElement {
        Button::new("settings-exit")
            .danger()
            .with_size(Size::Medium)
            .w_full()
            .cursor_pointer()
            .icon(Icon::empty().path("icons/log-out.svg"))
            .child(div().text_sm().child("退出应用"))
            .on_click(|_, _, _| {
                platform::quit_app();
            })
    }

    pub(super) fn separator(colors: &Colors) -> gpui::Div {
        div().w_full().h(px(1.)).bg(colors.border)
    }

    pub(super) fn settings_group(
        colors: &Colors,
        title: &'static str,
        description: &'static str,
    ) -> gpui::Div {
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

    pub(super) fn placeholder_row(
        colors: &Colors,
        title: &'static str,
        description: &'static str,
    ) -> gpui::Div {
        Self::setting_row(
            colors,
            title,
            description,
            div()
                .flex_none()
                .px_2()
                .py_1()
                .rounded_full()
                .bg(colors.container)
                .text_xs()
                .text_color(colors.disabled)
                .child("占位")
                .into_any_element(),
        )
    }

    pub(super) fn setting_row(
        colors: &Colors,
        title: &'static str,
        description: &'static str,
        control: impl IntoElement,
    ) -> gpui::Div {
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
                    .child(div().text_sm().child(title))
                    .child(
                        div()
                            .text_xs()
                            .text_color(colors.disabled)
                            .child(description),
                    ),
            )
            .child(control)
    }

    pub(super) fn reset_icon_button(
        &self,
        id: &'static str,
        cx: &mut Context<Self>,
        on_reset: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
    ) -> Button {
        Button::new(id)
            .ghost()
            .compact()
            .with_size(Size::Small)
            .cursor_pointer()
            .tooltip("重置")
            .icon(Icon::new(IconName::Undo2))
            .on_click(cx.listener(move |this, _, window, cx| {
                on_reset(this, window, cx);
            }))
    }

    pub(super) fn warning_hint(text: &'static str) -> gpui::Div {
        let warning_color = rgb(0xffaa33);
        div()
            .flex()
            .flex_row()
            .items_center()
            .gap_1()
            .text_xs()
            .line_height(relative(1.))
            .text_color(warning_color)
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
                            .text_color(warning_color),
                    ),
            )
            .child(text)
    }

    pub(super) fn setting_row_with_reset(
        &self,
        colors: &Colors,
        title: &'static str,
        description: &'static str,
        reset_id: &'static str,
        control: impl IntoElement,
        show_reset: bool,
        warning: Option<&'static str>,
        cx: &mut Context<Self>,
        on_reset: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
    ) -> gpui::Div {
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
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(div().text_sm().child(title))
                            .when(show_reset, |el| {
                                el.child(self.reset_icon_button(reset_id, cx, on_reset))
                            }),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(colors.disabled)
                            .child(description),
                    )
                    .when_some(warning, |el, text| el.child(Self::warning_hint(text))),
            )
            .child(control)
    }

    pub(super) fn opacity_slider_row(
        &self,
        title: &'static str,
        description: &'static str,
        reset_id: &'static str,
        slider: &Entity<SliderState>,
        colors: &Colors,
        warning: Option<&'static str>,
        suffix: &'static str,
        decimal_places: usize,
        show_reset: bool,
        cx: &mut Context<Self>,
        on_reset: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
    ) -> gpui::Div {
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
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(div().text_sm().child(title))
                            .when(show_reset, |el| {
                                el.child(self.reset_icon_button(reset_id, cx, on_reset))
                            }),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(colors.disabled)
                            .child(description),
                    )
                    .when_some(warning, |el, text| el.child(Self::warning_hint(text))),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .w(px(220.))
                    .child(Slider::new(slider).horizontal())
                    .child(
                        div()
                            .flex_none()
                            .w(if decimal_places == 0 {
                                px(36.)
                            } else {
                                px(48.)
                            })
                            .text_sm()
                            .text_color(colors.disabled)
                            .child(value_label),
                    ),
            )
    }

    pub(super) fn switch_control(
        &self,
        id: &'static str,
        checked: bool,
        colors: &Colors,
        cx: &mut Context<Self>,
        on_toggle: impl Fn(&mut Self, bool, &mut Context<Self>) + 'static,
    ) -> impl IntoElement {
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

    pub(super) fn position_control(
        &self,
        colors: &Colors,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let selected = hud::shift_lights_position();
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
            .child(self.position_option(
                "shift-lights-bottom",
                "底部",
                selected == ShiftLightsPosition::Bottom,
                ShiftLightsPosition::Bottom,
                colors,
                cx,
            ))
            .child(self.position_option(
                "shift-lights-right",
                "右侧",
                selected == ShiftLightsPosition::Right,
                ShiftLightsPosition::Right,
                colors,
                cx,
            ))
            .child(self.position_option(
                "shift-lights-top",
                "顶部",
                selected == ShiftLightsPosition::Top,
                ShiftLightsPosition::Top,
                colors,
                cx,
            ))
            .child(self.position_option(
                "shift-lights-left",
                "左侧",
                selected == ShiftLightsPosition::Left,
                ShiftLightsPosition::Left,
                colors,
                cx,
            ))
    }

    pub(super) fn position_option(
        &self,
        id: &'static str,
        label: &'static str,
        selected: bool,
        position: ShiftLightsPosition,
        colors: &Colors,
        cx: &mut Context<Self>,
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
            .on_click(cx.listener(move |this, _, _, cx| {
                this.set_shift_lights_position(position, cx);
            }))
            .child(label)
    }

    pub(super) fn direction_control(
        &self,
        colors: &Colors,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let position = hud::shift_lights_position();
        let selected = hud::shift_lights_direction();
        let (first_id, first_label, first_direction, second_id, second_label, second_direction) =
            if position.is_vertical() {
                (
                    "shift-lights-bottom-to-top",
                    "下 → 上",
                    ShiftLightsDirection::BottomToTop,
                    "shift-lights-top-to-bottom",
                    "上 → 下",
                    ShiftLightsDirection::TopToBottom,
                )
            } else {
                (
                    "shift-lights-left-to-right",
                    "左 → 右",
                    ShiftLightsDirection::LeftToRight,
                    "shift-lights-right-to-left",
                    "右 → 左",
                    ShiftLightsDirection::RightToLeft,
                )
            };

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
            .child(self.direction_option(
                first_id,
                first_label,
                selected == first_direction,
                first_direction,
                colors,
                cx,
            ))
            .child(self.direction_option(
                second_id,
                second_label,
                selected == second_direction,
                second_direction,
                colors,
                cx,
            ))
    }

    pub(super) fn direction_option(
        &self,
        id: &'static str,
        label: &'static str,
        selected: bool,
        direction: ShiftLightsDirection,
        colors: &Colors,
        cx: &mut Context<Self>,
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
            .on_click(cx.listener(move |this, _, _, cx| {
                this.set_shift_lights_direction(direction, cx);
            }))
            .child(label)
    }

    pub(super) fn action_button(
        &self,
        id: &'static str,
        label: &'static str,
        primary: bool,
        cx: &mut Context<Self>,
        on_click: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
    ) -> impl IntoElement {
        let button = Button::new(id)
            .with_size(Size::Small)
            .cursor_pointer()
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

    pub(super) fn input_apply_control(
        &self,
        input: impl IntoElement,
        unit: Option<&'static str>,
        apply_id: &'static str,
        colors: &Colors,
        cx: &mut Context<Self>,
        on_apply: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
    ) -> gpui::Div {
        div()
            .flex()
            .items_center()
            .gap_2()
            .child(input)
            .when_some(unit, |el, unit| {
                el.child(div().text_xs().text_color(colors.disabled).child(unit))
            })
            .child(self.action_button(apply_id, "确认", true, cx, on_apply))
    }
}
