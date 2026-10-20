use gpui_kit::{Context, IntoElement, div, prelude::*, px};
use gpui_kit::component::{Icon, Sizable, Size, input::Input, scroll::ScrollableElement};

use crate::core::general_config::{
    DEFAULT_CALIBRATE_MS, DEFAULT_FORWARD_HOST, DEFAULT_FORWARD_PORT, DEFAULT_LISTEN_HOST,
    DEFAULT_LISTEN_PORT,
};
use crate::core::telemetry;
use crate::session;

use super::section::SettingsSection;
use super::{Settings, colors::Colors};

impl Settings {
    pub(super) fn shift_lights_settings(
        &self,
        colors: &Colors,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        crate::components::shift_lights::render_settings(self, colors, cx)
    }

    pub(super) fn gear_display_settings(
        &self,
        colors: &Colors,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        crate::components::gear_display::render_settings(self, colors, cx)
    }

    pub(super) fn telemetry_settings(
        &self,
        colors: &Colors,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let (listen_host, listen_port) = telemetry::listen_host_port();
        let listen_uses_default =
            listen_host == DEFAULT_LISTEN_HOST && listen_port == DEFAULT_LISTEN_PORT;
        let listen_dirty = self.input_differs(&self.host_input, &listen_host, cx)
            || self.input_differs(&self.port_input, listen_port, cx);
        let listen_control = div()
            .flex()
            .items_center()
            .gap_2()
            .when(listen_dirty, |el| {
                el.child(self.action_button(
                    "listen-apply",
                    "确认",
                    true,
                    cx,
                    |this, window, cx| this.apply_listen_addr(window, cx),
                ))
            })
            .child(
                Input::new(&self.host_input)
                    .w(px(148.))
                    .with_size(Size::Medium),
            )
            .child(div().text_sm().text_color(colors.disabled).child(":"))
            .child(
                Input::new(&self.port_input)
                    .w(px(72.))
                    .with_size(Size::Medium),
            );

        let forward_enabled = telemetry::forward_enabled();
        let (forward_host, forward_port) = telemetry::forward_host_port();
        let forward_uses_default = !forward_enabled
            && forward_host == DEFAULT_FORWARD_HOST
            && forward_port == DEFAULT_FORWARD_PORT;
        let forward_dirty = self.input_differs(&self.forward_host_input, &forward_host, cx)
            || self.input_differs(&self.forward_port_input, forward_port, cx);
        let forward_warning = self.forward_addr_warning.or_else(|| {
            (forward_enabled && telemetry::forward_addr() == telemetry::listen_addr())
                .then_some("转发地址与监听地址相同，数据包不会被转发。")
        });
        let forward_control = div()
            .flex()
            .items_center()
            .gap_2()
            .child(self.switch_control(
                "forward-enabled",
                forward_enabled,
                colors,
                cx,
                |this, enabled, cx| this.set_forward_enabled(enabled, cx),
            ))
            .when(forward_dirty, |el| {
                el.child(self.action_button(
                    "forward-apply",
                    "确认",
                    true,
                    cx,
                    |this, window, cx| this.apply_forward_addr(window, cx),
                ))
            })
            .child(
                Input::new(&self.forward_host_input)
                    .w(px(148.))
                    .with_size(Size::Medium),
            )
            .child(div().text_sm().text_color(colors.disabled).child(":"))
            .child(
                Input::new(&self.forward_port_input)
                    .w(px(72.))
                    .with_size(Size::Medium),
            );

        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(self.setting_row_with_reset(
                colors,
                "监听地址",
                "UDP 绑定的本机地址与端口，默认为 127.0.0.1:9999。",
                "listen-reset",
                listen_control,
                !listen_uses_default,
                self.listen_addr_warning,
                cx,
                |this, window, cx| this.reset_listen_addr(window, cx),
            ))
            .child(self.setting_row_with_reset(
                colors,
                "UDP 转发",
                "开启后，将收到的游戏遥测数据包原样转发到指定地址，供其它应用使用。默认关闭，目标默认为 127.0.0.1:10000。",
                "forward-reset",
                forward_control,
                !forward_uses_default,
                forward_warning,
                cx,
                |this, window, cx| this.reset_forward_addr(window, cx),
            ))
    }

    pub(super) fn calibration_settings(
        &self,
        colors: &Colors,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let general = session::general();
        let calibrate_hint_visible = general.calibrate_hint_visible;
        let calibrate_ms_control = self.input_apply_control(
            Input::new(&self.calibrate_ms_input)
                .w(px(96.))
                .with_size(Size::Medium),
            Some("ms"),
            "calibrate-ms-apply",
            self.input_differs(&self.calibrate_ms_input, general.calibrate_ms, cx),
            colors,
            cx,
            |this, _, cx| this.apply_calibrate_ms(cx),
        );

        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(self.setting_row_with_reset(
                colors,
                "校准提示",
                "在 HUD 中央显示等待数据、校准说明与进度文字。",
                "calibrate-hint-visible-reset",
                self.switch_control(
                    "calibrate-hint-visible",
                    calibrate_hint_visible,
                    colors,
                    cx,
                    |this, visible, cx| this.set_calibrate_hint_visible(visible, cx),
                ),
                calibrate_hint_visible,
                None,
                cx,
                |this, window, cx| this.reset_calibrate_hint_visible(window, cx),
            ))
            .child(self.setting_row_with_reset(
                colors,
                "更严格校准条件",
                "开启后需同时按住手刹和油门才能校准；关闭后只需按住油门拉到断油转速。",
                "strict-calibrate-conditions-reset",
                self.switch_control(
                    "strict-calibrate-conditions",
                    general.strict_calibrate_conditions,
                    colors,
                    cx,
                    |this, strict, cx| this.set_strict_calibrate_conditions(strict, cx),
                ),
                general.strict_calibrate_conditions,
                None,
                cx,
                |this, window, cx| this.reset_strict_calibrate_conditions(window, cx),
            ))
            .child(self.setting_row_with_reset(
                colors,
                "记住已校准车辆",
                "开启后将断油转速按车辆写入配置，下次遇到同一车辆时自动恢复。关闭后不再读取或写入，已有记录会保留。",
                "remember-calibrated-cars-reset",
                self.switch_control(
                    "remember-calibrated-cars",
                    general.remember_calibrated_cars,
                    colors,
                    cx,
                    |this, remember, cx| this.set_remember_calibrated_cars(remember, cx),
                ),
                !general.remember_calibrated_cars,
                None,
                cx,
                |this, window, cx| this.reset_remember_calibrated_cars(window, cx),
            ))
            .child(self.setting_row_with_reset(
                colors,
                "校准时长",
                "燃油切断检测需保持的最短时间，单位毫秒。确认后立即生效。",
                "calibrate-ms-reset",
                calibrate_ms_control,
                general.calibrate_ms != DEFAULT_CALIBRATE_MS,
                Some("校准时长过低可能导致识别失败，尤其是极限转速较低的车辆。"),
                cx,
                |this, window, cx| this.reset_calibrate_ms(window, cx),
            ))
    }

    pub(super) fn section_content(
        &self,
        colors: &Colors,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let rows = self.selected_section.placeholder_rows();

        div()
            .size_full()
            .when(self.selected_section == SettingsSection::Overview, |el| {
                el.child(self.overview(colors, cx))
            })
            .when(self.selected_section == SettingsSection::About, |el| {
                el.child(self.about(colors, cx))
            })
            .when(
                !matches!(
                    self.selected_section,
                    SettingsSection::Overview | SettingsSection::About
                ),
                |el| {
                    el.child(
                        div()
                            .flex()
                            .flex_col()
                            .size_full()
                            .overflow_y_scrollbar()
                            .p_6()
                            .gap_6()
                            .child(
                                div()
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
                                                    .path(self.selected_section.icon_path())
                                                    .with_size(px(24.))
                                                    .text_color(colors.selected),
                                            )
                                            .child(self.selected_section.label()),
                                    )
                                    .child(
                                        div()
                                            .max_w(px(620.))
                                            .text_sm()
                                            .text_color(colors.disabled)
                                            .child(self.selected_section.description()),
                                    ),
                            )
                            .child(div().flex().flex_col().gap_3().map(|el| {
                                match self.selected_section {
                                    SettingsSection::Hud => {
                                        el.child(self.hud_display_settings(colors, cx))
                                    }
                                    SettingsSection::ShiftLights => {
                                        el.child(self.shift_lights_settings(colors, cx))
                                    }
                                    SettingsSection::GearDisplay => {
                                        el.child(self.gear_display_settings(colors, cx))
                                    }
                                    SettingsSection::Telemetry => {
                                        el.child(self.telemetry_settings(colors, cx))
                                    }
                                    SettingsSection::Calibration => {
                                        el.child(self.calibration_settings(colors, cx))
                                    }
                                    _ => {
                                        el.children(rows.into_iter().map(|(title, description)| {
                                            Self::placeholder_row(colors, title, description)
                                        }))
                                    }
                                }
                            }))
                            .when(
                                !matches!(
                                    self.selected_section,
                                    SettingsSection::Hud
                                        | SettingsSection::ShiftLights
                                        | SettingsSection::GearDisplay
                                        | SettingsSection::Telemetry
                                        | SettingsSection::Calibration
                                ),
                                |el| {
                                    el.child(
                                        div()
                                            .mt_auto()
                                            .pt_2()
                                            .text_xs()
                                            .text_color(colors.disabled)
                                            .child("此区域为设置项预留空间，当前不连接真实配置。"),
                                    )
                                },
                            ),
                    )
                },
            )
    }
}
