use gpui::{Context, IntoElement, div, prelude::*, px};
use gpui_component::{Icon, Sizable, StyledExt, scroll::ScrollableElement};

use super::{Settings, colors::Colors};

const APP_NAME: &str = "Horizon HUD";
const APP_SUMMARY: &str = "Horizon HUD 是一个基于 GPUI 框架的游戏内覆盖软件，用于在 Forza Horizon 中显示简洁可靠的 HUD，并提供丰富的自定义选项。";

impl Settings {
    pub(super) fn about(&self, colors: &Colors, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .overflow_y_scrollbar()
            .p_6()
            .gap_6()
            .child(
                div()
                    .flex_shrink_0()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .my_2()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .text_2xl()
                            .child(
                                Icon::empty()
                                    .path("icons/info.svg")
                                    .with_size(px(24.))
                                    .text_color(colors.selected),
                            )
                            .child("关于"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(colors.disabled)
                            .child("查看应用信息、版本与支持资源。"),
                    ),
            )
            .child(
                div()
                    .w_full()
                    .flex_shrink_0()
                    .overflow_x_hidden()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p_5()
                    .rounded_lg()
                    .bg(colors.container)
                    .border_1()
                    .border_color(colors.border)
                    .child(
                        div()
                            .flex()
                            .gap_4()
                            .items_center()
                            .child(div().text_lg().font_semibold().child(APP_NAME))
                            .child(
                                div()
                                    .px_2()
                                    .py_1()
                                    .rounded_md()
                                    .bg(colors.background)
                                    .text_xs()
                                    .text_color(colors.disabled)
                                    .child(format!("v{}", env!("CARGO_PKG_VERSION"))),
                            ),
                    )
                    .child(
                        div()
                            .w_full()
                            .overflow_x_hidden()
                            .text_sm()
                            .text_color(colors.text)
                            .child(APP_SUMMARY),
                    ),
            )
            .child(
                div()
                    .mt_2()
                    .w_full()
                    .flex()
                    .border_1()
                    .rounded_lg()
                    .bg(colors.container)
                    .border_color(colors.border)
                    .p_5()
                    .gap_3()
                    .child(div().text_sm().child("Made by"))
                    .child(
                        div()
                            .id("about-made-by-link")
                            .cursor_pointer()
                            .text_sm()
                            .text_color(colors.selected)
                            .child("Need-an-AwP")
                            .on_click(cx.listener(move |_, _, _, _| {
                                let _ = webbrowser::open("https://github.com/Need-an-AwP");
                            })),
                    ),
            )
            .child(
                div()
                    .flex_shrink_0()
                    .flex()
                    .flex_col()
                    .pt_2()
                    .gap_2()
                    .child(Self::about_link(
                        "about-github",
                        "GitHub",
                        "查看源码与更新记录。",
                        "https://github.com/Need-an-AwP/horizon-hud-gpui",
                        colors,
                        cx,
                    ))
                    .child(Self::about_link(
                        "about-docs",
                        "帮助文档",
                        "使用说明与常见问题。",
                        "https://github.com/Need-an-AwP/horizon-hud-gpui",
                        colors,
                        cx,
                    )),
            )
    }

    fn about_link(
        id: &'static str,
        label: &'static str,
        detail: &'static str,
        url: &'static str,
        colors: &Colors,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .id(id)
            .w_full()
            .overflow_x_hidden()
            .flex()
            .flex_col()
            .gap_1()
            .p_4()
            .rounded_lg()
            .bg(colors.background)
            .border_1()
            .border_color(colors.border)
            .cursor_pointer()
            .hover(|style| style.bg(colors.container))
            .active(|style| style.opacity(0.85))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(div().text_sm().font_medium().child(label))
                    .child(div().text_sm().text_color(colors.selected).child(url)),
            )
            .child(div().text_xs().text_color(colors.disabled).child(detail))
            .on_click(cx.listener(move |_, _, _, _| {
                let _ = webbrowser::open(url);
            }))
    }
}
