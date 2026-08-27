use gpui::{Context, IntoElement, Render, Window, div, prelude::*, px};
use gpui_component::{
    Sizable, Size,
    button::{Button, ButtonVariants},
};

use crate::settings_section::SettingsSection;

use super::Settings;

impl Render for Settings {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.colors(window);
        let theme_icon = self.theme_toggle_icon(window).text_color(colors.text);

        div()
            .flex()
            .flex_row()
            .size_full()
            .bg(colors.background)
            .text_color(colors.text)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .w(px(196.))
                    .flex_none()
                    .p_3()
                    .gap_5()
                    .bg(colors.container)
                    .border_r_1()
                    .border_color(colors.border)
                    .child(
                        Button::new("settings-theme-toggle")
                            .ghost()
                            .with_size(Size::Small)
                            .w_full()
                            .cursor_pointer()
                            .icon(theme_icon)
                            .child(div().text_sm().child("切换主题"))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.toggle_appearance(window, cx);
                            })),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(div().text_xs().text_color(colors.disabled).child("设置"))
                            .child(self.section_navigation(SettingsSection::Overview, &colors, cx))
                            .child(self.section_navigation(SettingsSection::Hud, &colors, cx))
                            .child(self.section_navigation(
                                SettingsSection::ShiftLights,
                                &colors,
                                cx,
                            ))
                            .child(self.section_navigation(
                                SettingsSection::GearDisplay,
                                &colors,
                                cx,
                            ))
                            .child(self.section_navigation(SettingsSection::Telemetry, &colors, cx))
                            .child(self.section_navigation(
                                SettingsSection::Calibration,
                                &colors,
                                cx,
                            ))
                            .child(self.section_navigation(SettingsSection::About, &colors, cx)),
                    )
                    .child(
                        div()
                            .mt_auto()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .pt_3()
                            .border_t_1()
                            .border_color(colors.border)
                            .child(self.exit_button())
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(colors.disabled)
                                    .child("Horizon HUD"),
                            ),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .child(self.section_content(&colors, cx)),
            )
    }
}
