use gpui::{Context, IntoElement, Render, Window, colors::DefaultAppearance, div, prelude::*, px};
use gpui_component::{Sizable, Size, button::Button};

use crate::settings_section::SettingsSection;

use super::Settings;

impl Render for Settings {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = self.colors(window);
        let theme_icon = self.theme_toggle_icon(window).text_color(colors.text);
        let theme_tooltip = match self.effective_appearance(window) {
            DefaultAppearance::Light => "切换到深色模式",
            DefaultAppearance::Dark => "切换到浅色模式",
        };

        div()
            .flex()
            .flex_row()
            .justify_between()
            .size_full()
            .bg(colors.background)
            .text_color(colors.text)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .justify_between()
                    .w(px(196.))
                    .flex_none()
                    .p_3()
                    .gap_5()
                    .bg(colors.container)
                    .border_r_1()
                    .border_color(colors.border)
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .pt_2()
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
                            .flex()
                            .flex_col()
                            .gap_4()
                            .pt_3()
                            .border_t_1()
                            .border_color(colors.border)
                            .child(self.exit_button())
                            .child(
                                div()
                                    .flex()
                                    .justify_between()
                                    .items_center()
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(colors.disabled)
                                            .child("Horizon HUD")
                                            .child("Made by Need_an_AwP"),
                                    )
                                    .child(
                                        Button::new("settings-theme-toggle")
                                            .outline()
                                            .with_size(Size::Small)
                                            .cursor_pointer()
                                            .tooltip(theme_tooltip)
                                            .icon(theme_icon)
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.toggle_appearance(window, cx);
                                            })),
                                    ),
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
