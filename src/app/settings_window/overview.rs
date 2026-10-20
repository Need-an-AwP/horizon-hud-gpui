use gpui_kit::{ClipboardItem, Context, IntoElement, div, prelude::*, px, rgb};
use gpui_kit::component::{
    Icon, Sizable, Size, StyledExt,
    button::{Button, ButtonVariants},
    input::Input,
    tooltip::Tooltip,
};

use crate::core::calibration;
use crate::core::telemetry;
use crate::session;
use crate::app::settings_window::section::SettingsSection;

use super::{Settings, colors::Colors};

impl Settings {
    pub(super) fn overview(&self, colors: &Colors, cx: &mut Context<Self>) -> impl IntoElement {
        let (listener_state, listener_detail, listener_color) =
            if let Some(error) = telemetry::listen_error() {
                ("监听异常", error, rgb(0xff3333))
            } else {
                (
                    "正在监听",
                    format!("UDP · {}", telemetry::listen_addr_display()),
                    colors.success,
                )
            };
        let calibrated = calibration::shift_lights_calibrated();
        let electric = calibration::electric_car();
        let (calibration_state, calibration_detail, calibration_color) = if electric {
            (
                "无需校准",
                "当前车辆为电动车，无需执行校准。".to_string(),
                colors.success,
            )
        } else if calibrated {
            (
                "已完成",
                "已记录当前车辆的转速范围。".to_string(),
                colors.success,
            )
        } else {
            (
                "等待校准",
                if session::general().strict_calibrate_conditions {
                    "同时按下手刹和油门，等待转速上升完成校准".to_string()
                } else {
                    "按住油门拉到断油转速完成校准".to_string()
                },
                colors.warning,
            )
        };

        let config_path = session::active_path();

        div().flex().flex_col().size_full().p_6().child(
            div()
                .flex_1()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
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
                                .child(div().text_sm().text_color(colors.disabled).child("💨")),
                        )
                        .child(
                            div()
                                .flex()
                                .gap_5()
                                .child(self.status_card(
                                    "overview-telemetry-status",
                                    "遥测监听",
                                    listener_state,
                                    &listener_detail,
                                    listener_color,
                                    "icons/braces.svg",
                                    SettingsSection::Telemetry,
                                    false,
                                    colors,
                                    cx,
                                ))
                                .child(self.status_card(
                                    "overview-calibration-status",
                                    "换挡指示校准",
                                    calibration_state,
                                    &calibration_detail,
                                    calibration_color,
                                    "icons/gauge.svg",
                                    SettingsSection::Calibration,
                                    session::current_car_has_saved_calibration(),
                                    colors,
                                    cx,
                                )),
                        )
                        .child(
                            div()
                                .w_full()
                                .grid()
                                .grid_cols(5)
                                .gap_5()
                                .child(self.overview_guide_card(colors, cx))
                                .child(self.overview_config_card(
                                    colors,
                                    config_path.is_some(),
                                    cx,
                                )),
                        )
                        .child(self.overview_launch_card(colors, cx)),
                ),
        )
    }

    fn overview_guide_card(&self, colors: &Colors, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .col_span(3)
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
                    .min_w_0()
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
                    .id("overview-config-guide")
                    .flex_none()
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .bg(colors.selected)
                    .text_sm()
                    .font_medium()
                    .cursor_pointer()
                    .text_color(colors.selected_text)
                    .child("配置指南")
                    .on_click(cx.listener(move |_, _, _, _| {
                        let _ = webbrowser::open(
                            "https://github.com/Need-an-AwP/horizon-hud-gpui?tab=readme-ov-file#%E4%BD%BF%E7%94%A8%E6%96%B9%E6%B3%95",
                        );
                    })),
            )
    }

    fn overview_config_card(
        &self,
        colors: &Colors,
        config_available: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let confirming = self.reset_config_confirming;
        let subtitle = if confirming {
            ("确认覆盖为默认值？", colors.warning)
        } else if self.config_open_hovered {
            ("打开配置", colors.selected)
        } else {
            (
                "重置配置",
                if self.config_reset_hovered {
                    colors.warning
                } else {
                    colors.disabled
                },
            )
        };

        div()
            .col_span(2)
            .flex()
            .items_center()
            .gap_4()
            .p_5()
            .rounded_lg()
            .bg(colors.container)
            .border_1()
            .border_color(colors.border)
            .when(confirming, |el| {
                el.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            Button::new("overview-reset-cancel")
                                .outline()
                                .with_size(Size::Small)
                                .w(px(28.))
                                .h(px(28.))
                                .cursor_pointer()
                                .child(div().text_sm().child("✕"))
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.cancel_reset_config(cx);
                                })),
                        )
                        .child(
                            Button::new("overview-reset-confirm")
                                .danger()
                                .with_size(Size::Small)
                                .w(px(28.))
                                .h(px(28.))
                                .cursor_pointer()
                                .child(div().text_sm().child("√"))
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.confirm_reset_config(window, cx);
                                })),
                        ),
                )
            })
            .when(!confirming, |el| {
                el.child(
                    div()
                        .id("overview-reset")
                        .flex()
                        .items_center()
                        .justify_center()
                        .size(px(28.))
                        .rounded_md()
                        .bg(colors.background)
                        .cursor_pointer()
                        .hover(|style| style.bg(colors.warning))
                        .on_hover(cx.listener(|this, hovered, _, cx| {
                            if this.config_reset_hovered != *hovered {
                                this.config_reset_hovered = *hovered;
                                cx.notify();
                            }
                        }))
                        .child(
                            Icon::empty()
                                .path("icons/undo-2.svg")
                                .with_size(px(15.))
                                .text_color(colors.selected),
                        )
                        .on_click(cx.listener(|this, _, _, cx| {
                            this.arm_reset_config(cx);
                        })),
                )
            })
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(div().text_sm().font_medium().child("配置文件"))
                    .child(div().text_xs().text_color(subtitle.1).child(subtitle.0)),
            )
            .child(
                div()
                    .id("overview-config-open")
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(28.))
                    .rounded_md()
                    .bg(colors.background)
                    .when(config_available, |el| {
                        el.cursor_pointer()
                            .hover(|style| style.bg(colors.container))
                            .on_hover(cx.listener(|this, hovered, _, cx| {
                                if this.config_open_hovered != *hovered {
                                    this.config_open_hovered = *hovered;
                                    cx.notify();
                                }
                            }))
                            .tooltip(|window, cx| {
                                Tooltip::new("点击打开配置文件").build(window, cx)
                            })
                            .on_click(cx.listener(|_, _, _, _| {
                                session::open_active();
                            }))
                    })
                    .child(
                        Icon::empty()
                            .path("icons/square-arrow-out-up-right.svg")
                            .with_size(px(15.))
                            .text_color(if config_available {
                                colors.selected
                            } else {
                                colors.disabled
                            }),
                    ),
            )
    }

    fn overview_launch_card(&self, colors: &Colors, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_3()
            .p_5()
            .rounded_lg()
            .bg(colors.container)
            .border_1()
            .border_color(colors.border)
            .child(div().text_sm().font_medium().child("随游戏启动"))
            .child(
                div()
                    .text_xs()
                    .text_color(colors.disabled)
                    .child(
                        "粘贴到 Steam 游戏属性的启动选项。自动生成路径，请勿修改。",
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Input::new(&self.launch_command_input)
                            .flex_1()
                            .min_w(px(0.))
                            .readonly(true)
                            .with_size(Size::Medium),
                    )
                    .child(
                        Button::new("overview-copy-launch")
                            .outline()
                            .with_size(Size::Medium)
                            .cursor_pointer()
                            .tooltip("复制启动命令")
                            .icon(Icon::empty().path("icons/copy.svg"))
                            .on_click(cx.listener(|this, _, _, cx| {
                                let command =
                                    this.launch_command_input.read(cx).value().to_string();
                                cx.write_to_clipboard(ClipboardItem::new_string(command));
                            })),
                    ),
            )
    }

    fn status_card(
        &self,
        id: &'static str,
        label: &'static str,
        value: &'static str,
        detail: &str,
        value_color: gpui_kit::Rgba,
        icon_path: &'static str,
        section: SettingsSection,
        show_reset: bool,
        colors: &Colors,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let tooltip = format!("前往{}", section.label());
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
            .cursor_pointer()
            .hover(|style| style.bg(colors.background))
            .active(|style| style.opacity(0.85))
            .tooltip({
                let tooltip = tooltip.clone();
                move |window, cx| Tooltip::new(tooltip.clone()).build(window, cx)
            })
            .on_click(cx.listener(move |this, _, window, cx| {
                this.select_section(section, window, cx);
            }))
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
                            .gap_2()
                            .when(show_reset, |el| {
                                el.child(
                                    div()
                                        .id((id, 99u64))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .size(px(28.))
                                        .rounded_md()
                                        .bg(colors.background)
                                        .cursor_pointer()
                                        .tooltip(|window, cx| {
                                            Tooltip::new("重置当前车辆的断油转速").build(window, cx)
                                        })
                                        .hover(|style| style.bg(colors.warning))
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.reset_current_car_calibration(cx);
                                        }))
                                        .child(
                                            Icon::empty()
                                                .path("icons/refresh-ccw.svg")
                                                .with_size(px(15.))
                                                .text_color(colors.selected),
                                        ),
                                )
                            })
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
                    ),
            )
            .child(
                div()
                    .text_lg()
                    .font_medium()
                    .text_color(value_color)
                    .child(value),
            )
            .child(
                div()
                    .min_h(px(32.))
                    .text_xs()
                    .text_color(colors.disabled)
                    .child(detail.to_string()),
            )
    }
}
