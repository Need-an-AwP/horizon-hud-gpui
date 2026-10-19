pub mod bar;
pub mod porsche_cup;
pub mod rally_1;

use gpui::{div, prelude::*, px, Context, Entity, IntoElement, SharedString, Window};
use gpui_component::{
    select::{Select, SelectEvent, SelectItem, SelectState},
    Icon, IndexPath, Sizable, Size,
};
use serde::{Deserialize, Serialize};

use crate::app::settings_window::colors::Colors;
use crate::app::settings_window::Settings;
use crate::components::shared::settings_ui::setting_row_with_reset;
use crate::core::props::HudTelemetryProps;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShiftLightsStyle {
    #[default]
    Bar,
    #[serde(alias = "symmetric_circles")]
    PorscheCup,
    #[serde(rename = "rally_1")]
    Rally1,
}

impl ShiftLightsStyle {
    pub fn label(self) -> &'static str {
        match self {
            Self::Bar => "基础灯条",
            Self::PorscheCup => "Porsche 911 GT3 Cup (992)",
            Self::Rally1 => "Rally 1",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Bar => "仿照拉力赛车样式的转速指示灯。",
            Self::PorscheCup => {
                "仿照保时捷 911 GT3 Cup (992) 对称环形转速指示灯，由两侧同时向中心点亮。"
            }
            Self::Rally1 => {
                "顶部五段转速灯，无间隙铺满宽度，由两侧向中心按绿、黄、红依次点亮。"
            }
        }
    }

    fn index(self) -> usize {
        match self {
            Self::Bar => 0,
            Self::PorscheCup => 1,
            Self::Rally1 => 2,
        }
    }
}

impl SelectItem for ShiftLightsStyle {
    type Value = Self;

    fn title(&self) -> SharedString {
        SharedString::from(self.label())
    }

    fn value(&self) -> &Self::Value {
        self
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ShiftLightsSettings {
    pub style: ShiftLightsStyle,
    pub bar: bar::BarConfig,
    pub porsche_cup: porsche_cup::PorscheCupConfig,
    pub rally_1: rally_1::Rally1Config,
}

impl Default for ShiftLightsSettings {
    fn default() -> Self {
        Self {
            style: ShiftLightsStyle::Bar,
            bar: bar::BarConfig::default(),
            porsche_cup: porsche_cup::PorscheCupConfig::default(),
            rally_1: rally_1::Rally1Config::default(),
        }
    }
}

pub struct ShiftLightsControls {
    pub style_select: Entity<SelectState<Vec<ShiftLightsStyle>>>,
    pub bar: bar::SettingsControls,
    pub porsche_cup: porsche_cup::SettingsControls,
    pub rally_1: rally_1::SettingsControls,
}

impl ShiftLightsControls {
    pub fn new(
        cfg: &ShiftLightsSettings,
        window: &mut Window,
        cx: &mut Context<Settings>,
    ) -> Self {
        let style_select = cx.new(|cx| {
            SelectState::new(
                vec![
                    ShiftLightsStyle::Bar,
                    ShiftLightsStyle::PorscheCup,
                    ShiftLightsStyle::Rally1,
                ],
                Some(IndexPath::new(cfg.style.index())),
                window,
                cx,
            )
        });
        cx.subscribe_in(
            &style_select,
            window,
            |_this, _, event: &SelectEvent<Vec<ShiftLightsStyle>>, _window, cx| {
                if let SelectEvent::Confirm(Some(style)) = event {
                    crate::session::update_shift_lights(|settings| settings.style = *style);
                    cx.notify();
                }
            },
        )
        .detach();
        Self {
            style_select,
            bar: bar::SettingsControls::new(&cfg.bar, window, cx),
            porsche_cup: porsche_cup::SettingsControls::new(&cfg.porsche_cup, window, cx),
            rally_1: rally_1::SettingsControls::new(&cfg.rally_1, window, cx),
        }
    }

    pub fn sync(&self, cfg: &ShiftLightsSettings, window: &mut Window, cx: &mut Context<Settings>) {
        self.style_select.update(cx, |select, cx| {
            select.set_selected_value(&cfg.style, window, cx);
        });
        self.bar.sync(&cfg.bar, window, cx);
        self.porsche_cup.sync(&cfg.porsche_cup, window, cx);
        self.rally_1.sync(&cfg.rally_1, window, cx);
    }
}

pub fn render_hud(props: &HudTelemetryProps, settings: &ShiftLightsSettings) -> gpui::AnyElement {
    match settings.style {
        ShiftLightsStyle::Bar => bar::render_hud(props, &settings.bar),
        ShiftLightsStyle::PorscheCup => porsche_cup::render_hud(props, &settings.porsche_cup),
        ShiftLightsStyle::Rally1 => rally_1::render_hud(props, &settings.rally_1),
    }
}

pub fn render_settings(
    view: &Settings,
    colors: &Colors,
    cx: &mut Context<Settings>,
) -> gpui::AnyElement {
    let style = crate::session::shift_lights().style;
    let style_row = setting_row_with_reset(
        colors,
        "样式",
        style.description(),
        "shift-lights-style-reset",
        div().flex_none().w(px(228.)).child(
            Select::new(&view.shift_lights.style_select)
                .w_full()
                .with_size(Size::Small)
                .icon(Icon::empty().path("icons/chevron-down.svg"))
                .bg(colors.container)
                .border_color(colors.border),
        ),
        style != ShiftLightsStyle::Bar,
        None,
        cx,
        |this, window, cx| {
            crate::session::update_shift_lights(|settings| settings.style = ShiftLightsStyle::Bar);
            this.shift_lights
                .sync(&crate::session::shift_lights(), window, cx);
            cx.notify();
        },
    );
    let body = match style {
        ShiftLightsStyle::Bar => bar::render_settings(view, colors, cx),
        ShiftLightsStyle::PorscheCup => porsche_cup::render_settings(view, colors, cx),
        ShiftLightsStyle::Rally1 => rally_1::render_settings(view, colors, cx),
    };
    div()
        .flex()
        .flex_col()
        .gap_5()
        .child(
            div().text_xs().text_color(colors.warning).child(
                "此窗口拥有焦点时会暂时覆盖「仅游戏时显示」，强制显示 HUD，便于预览。失焦、离开此页或关闭设置后恢复。",
            ),
        )
        .child(style_row)
        .child(body)
        .into_any_element()
}
