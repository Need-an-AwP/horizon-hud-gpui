use gpui::{Context, IntoElement, colors::Colors, div, prelude::*, px};
use gpui_component::{Icon, Sizable, Size, input::Input, scroll::ScrollableElement};

use crate::config::{
    DEFAULT_CALIBRATE_MS, DEFAULT_LISTEN_HOST, DEFAULT_LISTEN_PORT,
    DEFAULT_SHIFT_LIGHTS_BLINK_PERCENT, DEFAULT_SHIFT_LIGHTS_DIM_OPACITY,
    DEFAULT_SHIFT_LIGHTS_GAP_PX, DEFAULT_SHIFT_LIGHTS_LIT_OPACITY, DEFAULT_SHIFT_LIGHTS_OFFSET_PX,
    DEFAULT_SHIFT_LIGHTS_THICKNESS_PX, DEFAULT_SHIFT_LIGHTS_WIDTH_PERCENT,
};
use crate::hud;
use crate::settings_section::SettingsSection;
use crate::telemetry;

use super::Settings;

impl Settings {
    pub(super) fn shift_lights_settings(
        &self,
        colors: &Colors,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let direction_description = if hud::shift_lights_position().is_vertical() {
            "纵向灯条：下→上为默认方向。"
        } else {
            "横向灯条：左→右为默认方向。"
        };
        let offset_control = self.input_apply_control(
            Input::new(&self.shift_lights_offset_input)
                .w(px(96.))
                .with_size(Size::Small),
            Some("px"),
            "shift-lights-offset-apply",
            colors,
            cx,
            |this, _, cx| this.apply_shift_lights_offset(cx),
        );
        let thickness_control = self.input_apply_control(
            Input::new(&self.shift_lights_thickness_input)
                .w(px(96.))
                .with_size(Size::Small),
            Some("px"),
            "shift-lights-thickness-apply",
            colors,
            cx,
            |this, _, cx| this.apply_shift_lights_thickness(cx),
        );
        let gap_control = self.input_apply_control(
            Input::new(&self.shift_lights_gap_input)
                .w(px(96.))
                .with_size(Size::Small),
            Some("px"),
            "shift-lights-gap-apply",
            colors,
            cx,
            |this, _, cx| this.apply_shift_lights_gap(cx),
        );
        div()
            .flex()
            .flex_col()
            .gap_5()
            .child(
                Self::settings_group(
                    colors,
                    "位置与方向",
                    "选择灯条贴在哪条屏幕边缘，以及转速升高时的点亮方向。",
                )
                .child(Self::setting_row(
                    colors,
                    "位置",
                    "可放在屏幕顶部、底部、左侧或右侧。",
                    self.position_control(colors, cx),
                ))
                .child(Self::setting_row(
                    colors,
                    "方向",
                    direction_description,
                    self.direction_control(colors, cx),
                )),
            )
            .child(Self::separator(colors))
            .child(
                Self::settings_group(
                    colors,
                    "尺寸与布局",
                    "调整灯条占屏幕的比例、厚度、灯格间距，以及离边缘的距离。",
                )
                .child(self.opacity_slider_row(
                    "整体宽度",
                    "横向灯条的宽度、纵向灯条的高度，占屏幕对应边的百分比；默认 75%。",
                    "shift-lights-width-reset",
                    &self.shift_lights_width_slider,
                    colors,
                    None,
                    "%",
                    0,
                    hud::shift_lights_width_percent() != DEFAULT_SHIFT_LIGHTS_WIDTH_PERCENT,
                    cx,
                    |this, window, cx| this.reset_shift_lights_width(window, cx),
                ))
                .child(self.setting_row_with_reset(
                    colors,
                    "灯条厚度",
                    "横向灯条的高度、纵向灯条的宽度，单位像素；默认 40 px。",
                    "shift-lights-thickness-reset",
                    thickness_control,
                    hud::shift_lights_thickness_px() != DEFAULT_SHIFT_LIGHTS_THICKNESS_PX,
                    cx,
                    |this, window, cx| this.reset_shift_lights_thickness(window, cx),
                ))
                .child(self.setting_row_with_reset(
                    colors,
                    "灯格间隔",
                    "相邻灯格之间的像素距离；可为 0，默认 8 px。",
                    "shift-lights-gap-reset",
                    gap_control,
                    hud::shift_lights_gap_px() != DEFAULT_SHIFT_LIGHTS_GAP_PX,
                    cx,
                    |this, window, cx| this.reset_shift_lights_gap(window, cx),
                ))
                .child(self.setting_row_with_reset(
                    colors,
                    "边缘偏移",
                    "0 表示紧贴边缘；正整数表示向屏幕内侧偏移的像素数。",
                    "shift-lights-offset-reset",
                    offset_control,
                    hud::shift_lights_offset_px() != DEFAULT_SHIFT_LIGHTS_OFFSET_PX,
                    cx,
                    |this, window, cx| this.reset_shift_lights_offset(window, cx),
                )),
            )
            .child(Self::separator(colors))
            .child(
                Self::settings_group(
                    colors,
                    "显示效果",
                    "控制灯条闪烁时机，以及亮起与未亮起灯格的透明度。",
                )
                .child(self.opacity_slider_row(
                    "闪烁阈值",
                    "达到校准转速的该百分比后，整条灯条开始闪烁；范围 80% 到 100%，步进 0.1%；默认 95%。",
                    "shift-lights-blink-reset",
                    &self.shift_lights_blink_slider,
                    colors,
                    None,
                    "%",
                    1,
                    hud::shift_lights_blink_percent() != DEFAULT_SHIFT_LIGHTS_BLINK_PERCENT,
                    cx,
                    |this, window, cx| this.reset_shift_lights_blink(window, cx),
                ))
                .child(self.opacity_slider_row(
                    "亮起透明度",
                    "亮起的转速灯透明度，范围 0 到 1；默认 0.7。",
                    "shift-lights-lit-opacity-reset",
                    &self.shift_lights_lit_opacity_slider,
                    colors,
                    None,
                    "",
                    2,
                    hud::shift_lights_lit_opacity() != DEFAULT_SHIFT_LIGHTS_LIT_OPACITY,
                    cx,
                    |this, window, cx| this.reset_shift_lights_lit_opacity(window, cx),
                ))
                .child(self.opacity_slider_row(
                    "熄灭透明度",
                    "未亮起的转速灯透明度，范围 0 到 1。",
                    "shift-lights-dim-opacity-reset",
                    &self.shift_lights_dim_opacity_slider,
                    colors,
                    (!hud::shift_lights_calibrated()).then_some(
                        "当前处于未校准状态，熄灭透明度可能不可见",
                    ),
                    "",
                    2,
                    hud::shift_lights_dim_opacity() != DEFAULT_SHIFT_LIGHTS_DIM_OPACITY,
                    cx,
                    |this, window, cx| this.reset_shift_lights_dim_opacity(window, cx),
                )),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(colors.disabled)
                    .child("此窗口拥有焦点时会暂时覆盖「仅游戏时显示」，强制显示 HUD，便于预览。失焦、离开此页或关闭设置后恢复。"),
            )
    }

    pub(super) fn telemetry_settings(
        &self,
        colors: &Colors,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let (listen_host, listen_port) = telemetry::listen_host_port();
        let listen_uses_default =
            listen_host == DEFAULT_LISTEN_HOST && listen_port == DEFAULT_LISTEN_PORT;

        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                self.setting_row_with_reset(
                    colors,
                    "监听地址",
                    "UDP 绑定的本机地址，默认为 127.0.0.1（仅本机）。",
                    "listen-reset",
                    Input::new(&self.host_input)
                        .w(px(220.))
                        .with_size(Size::Small),
                    !listen_uses_default,
                    cx,
                    |this, window, cx| this.reset_listen_addr(window, cx),
                ),
            )
            .child(Self::setting_row(
                colors,
                "端口",
                "UDP 监听端口，默认为 9999。确认后会重新绑定。",
                self.input_apply_control(
                    Input::new(&self.port_input)
                        .w(px(96.))
                        .with_size(Size::Small),
                    None,
                    "listen-apply",
                    colors,
                    cx,
                    |this, _, cx| this.apply_listen_addr(cx),
                ),
            ))
    }

    pub(super) fn calibration_settings(
        &self,
        colors: &Colors,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let calibrate_hint_visible = hud::calibrate_hint_visible();
        let calibrate_ms_control = self.input_apply_control(
            Input::new(&self.calibrate_ms_input)
                .w(px(96.))
                .with_size(Size::Small),
            Some("ms"),
            "calibrate-ms-apply",
            colors,
            cx,
            |this, _, cx| this.apply_calibrate_ms(cx),
        );

        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(Self::setting_row(
                colors,
                "校准提示",
                "在 HUD 中央显示等待数据、校准说明与进度文字。",
                self.switch_control(
                    "calibrate-hint-visible",
                    calibrate_hint_visible,
                    colors,
                    cx,
                    |this, visible, cx| this.set_calibrate_hint_visible(visible, cx),
                ),
            ))
            .child(self.setting_row_with_reset(
                colors,
                "校准时长",
                "燃油切断检测需保持的最短时间，单位毫秒。确认后立即生效。",
                "calibrate-ms-reset",
                calibrate_ms_control,
                hud::calibrate_ms() != DEFAULT_CALIBRATE_MS,
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
            .when(self.selected_section != SettingsSection::Overview, |el| {
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
                        .child(div().flex().flex_col().gap_3().map(
                            |el| match self.selected_section {
                                SettingsSection::Hud => {
                                    el.child(self.hud_display_settings(colors, cx))
                                }
                                SettingsSection::ShiftLights => {
                                    el.child(self.shift_lights_settings(colors, cx))
                                }
                                SettingsSection::Telemetry => {
                                    el.child(self.telemetry_settings(colors, cx))
                                }
                                SettingsSection::Calibration => {
                                    el.child(self.calibration_settings(colors, cx))
                                }
                                _ => el.children(rows.into_iter().map(|(title, description)| {
                                    Self::placeholder_row(colors, title, description)
                                })),
                            },
                        ))
                        .when(
                            !matches!(
                                self.selected_section,
                                SettingsSection::Hud
                                    | SettingsSection::ShiftLights
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
            })
    }
}
