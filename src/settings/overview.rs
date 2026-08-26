use gpui::{Context, IntoElement, colors::Colors, div, prelude::*, px};
use gpui_component::{Icon, Sizable, StyledExt};

use crate::hud;
use crate::telemetry;

use super::Settings;

impl Settings {
    pub(super) fn overview(&self, colors: &Colors, _cx: &mut Context<Self>) -> impl IntoElement {
        let (listener_state, listener_detail) = if let Some(error) = telemetry::listen_error() {
            ("监听异常", error)
        } else {
            (
                "正在监听",
                format!("UDP · {}", telemetry::listen_addr_display()),
            )
        };
        let calibrated = hud::shift_lights_calibrated();

        div()
            .flex()
            .flex_col()
            .size_full()
            .items_center()
            .justify_center()
            .p_6()
            .child(
                div()
                    .w_full()
                    .max_w(px(660.))
                    .flex()
                    .flex_col()
                    .gap_5()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_center()
                            .gap_2()
                            .child(div().text_2xl().font_semibold().child("Horizon HUD"))
                            .child(
                                div()
                                    .text_sm()
                                    .text_color(colors.disabled)
                                    .child("准备好后，启动游戏并发送遥测数据即可。"),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_3()
                            .child(self.status_card(
                                "overview-telemetry-status",
                                "遥测监听",
                                listener_state,
                                &listener_detail,
                                "icons/gauge.svg",
                                colors,
                            ))
                            .child(self.status_card(
                                "overview-calibration-status",
                                "换挡灯校准",
                                if calibrated {
                                    "已完成"
                                } else {
                                    "等待校准"
                                },
                                if calibrated {
                                    "已记录当前车辆的转速范围。"
                                } else {
                                    "同时按下手刹和油门，等待转速上升完成校准"
                                },
                                "icons/sliders-horizontal.svg",
                                colors,
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            .p_5()
                            .rounded_lg()
                            .bg(colors.container)
                            .border_1()
                            .border_color(colors.border)
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .size(px(38.))
                                    .rounded_md()
                                    .bg(colors.background)
                                    .child(
                                        Icon::empty()
                                            .path("icons/info.svg")
                                            .with_size(px(20.))
                                            .text_color(colors.selected),
                                    ),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(div().text_sm().font_medium().child("首次使用？"))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(colors.disabled)
                                            .child("请先阅读配置指南，完成游戏端的遥测设置。"),
                                    ),
                            )
                            .child(
                                div()
                                    .flex_none()
                                    .px_3()
                                    .py_2()
                                    .rounded_md()
                                    .bg(colors.selected)
                                    .text_sm()
                                    .font_medium()
                                    .text_color(colors.selected_text)
                                    .child("配置指南"),
                            ),
                    ),
            )
    }

    fn status_card(
        &self,
        id: &'static str,
        label: &'static str,
        value: &'static str,
        detail: &str,
        icon_path: &'static str,
        colors: &Colors,
    ) -> impl IntoElement {
        div()
            .id(id)
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .gap_3()
            .p_4()
            .rounded_lg()
            .bg(colors.container)
            .border_1()
            .border_color(colors.border)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(div().text_sm().text_color(colors.disabled).child(label))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(px(28.))
                            .rounded_md()
                            .bg(colors.background)
                            .child(
                                Icon::empty()
                                    .path(icon_path)
                                    .with_size(px(15.))
                                    .text_color(colors.selected),
                            ),
                    ),
            )
            .child(div().text_lg().font_medium().child(value))
            .child(
                div()
                    .min_h(px(32.))
                    .text_xs()
                    .text_color(colors.disabled)
                    .child(detail.to_string()),
            )
    }
}
