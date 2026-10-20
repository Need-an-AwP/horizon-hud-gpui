use gpui_kit::{Context, Entity, IntoElement, Window, div, prelude::*, px, relative};
use gpui_kit::component::{
    Icon, IconName, Sizable, Size,
    button::{Button, ButtonVariants},
    input::InputState,
};

use crate::platform;

use super::section::SettingsSection;

use super::{Settings, colors::Colors};


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

    pub(super) fn placeholder_row(
        colors: &Colors,
        title: &'static str,
        description: &'static str,
    ) -> gpui_kit::Div {
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

    pub(super) fn reset_icon_button(
        &self,
        id: &'static str,
        cx: &mut Context<Self>,
        on_reset: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
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
        &self,
        title: &'static str,
        reset_id: &'static str,
        show_reset: bool,
        cx: &mut Context<Self>,
        on_reset: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
    ) -> gpui_kit::Div {
        div()
            .flex()
            .items_center()
            .gap_1()
            .flex_none()
            .h(px(20.))
            .child(div().text_sm().line_height(relative(1.)).child(title))
            .when(show_reset, |el| {
                el.child(self.reset_icon_button(reset_id, cx, on_reset))
            })
    }

    pub(super) fn warning_hint(colors: &Colors, text: &'static str) -> gpui_kit::Div {
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
                    .child(self.title_with_reset(title, reset_id, show_reset, cx, on_reset))
                    .child(
                        div()
                            .text_xs()
                            .text_color(colors.disabled)
                            .child(description),
                    )
                    .when_some(warning, |el, text| {
                        el.child(Self::warning_hint(colors, text))
                    }),
            )
            .child(div().flex_none().child(control))
    }

    pub(super) fn switch_control<T: Fn(&mut Self, bool, &mut Context<Self>) + 'static>(
        &self,
        id: &'static str,
        checked: bool,
        colors: &Colors,
        cx: &mut Context<Self>,
        on_toggle: T,
    ) -> impl IntoElement + use<T> {
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

    pub(super) fn input_differs(
        &self,
        input: &Entity<InputState>,
        committed: impl ToString,
        cx: &Context<Self>,
    ) -> bool {
        input.read(cx).value().trim() != committed.to_string()
    }

    pub(super) fn input_apply_control(
        &self,
        input: impl IntoElement,
        unit: Option<&'static str>,
        apply_id: &'static str,
        dirty: bool,
        colors: &Colors,
        cx: &mut Context<Self>,
        on_apply: impl Fn(&mut Self, &mut Window, &mut Context<Self>) + 'static,
    ) -> gpui_kit::Div {
        div()
            .flex()
            .items_center()
            .gap_2()
            .when(dirty, |el| {
                el.child(self.action_button(apply_id, "确认", true, cx, on_apply))
            })
            .child(input)
            .when_some(unit, |el, unit| {
                el.child(div().text_xs().text_color(colors.disabled).child(unit))
            })
    }
}
