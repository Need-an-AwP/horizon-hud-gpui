use gpui::{Context, IntoElement, colors::Colors, div, prelude::*};

use crate::hud;

use super::Settings;

impl Settings {
    pub(super) fn hud_display_settings(
        &self,
        colors: &Colors,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let only_show_in_game = hud::only_show_in_game();
        let charts_visible = hud::charts_visible();

        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(Self::setting_row(
                colors,
                "仅游戏时显示",
                "仅在游戏窗口位于前台时显示 HUD；与托盘右键菜单中的开关同步。",
                self.switch_control(
                    "hud-only-show-in-game",
                    only_show_in_game,
                    colors,
                    cx,
                    |this, visible, cx| this.set_only_show_in_game(visible, cx),
                ),
            ))
            .child(Self::setting_row(
                colors,
                "图表显示",
                "与托盘右键菜单中的图表开关同步。",
                self.switch_control(
                    "hud-charts-visible",
                    charts_visible,
                    colors,
                    cx,
                    |this, visible, cx| this.set_charts_visible(visible, cx),
                ),
            ))
            .child(
                div()
                    .text_xs()
                    .text_color(colors.disabled)
                    .child("此窗口拥有焦点时会暂时覆盖「仅游戏时显示」，强制显示 HUD，便于预览。失焦、离开此页或关闭设置后恢复。"),
            )
    }
}
