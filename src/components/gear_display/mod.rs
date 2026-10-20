use gpui_kit::component::{
    Icon, IndexPath, Sizable, Size,
    select::{Select, SelectEvent, SelectItem, SelectState},
};
use gpui_kit::{
    AnyElement, Context, Entity, IntoElement, SharedString, Window, div, prelude::*, px,
};
use serde::{Deserialize, Serialize};

use crate::app::settings_window::Settings;
use crate::app::settings_window::colors::Colors;
use crate::components::shared::settings_ui::{setting_row_with_reset, switch_control};
use crate::core::props::HudTelemetryProps;
use crate::session;

pub mod numeric;
pub mod seven_segment;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GearDisplayStyle {
    #[default]
    SevenSegment,
    Numeric,
}

impl GearDisplayStyle {
    pub fn label(self) -> &'static str {
        match self {
            Self::SevenSegment => "七段数码管",
            Self::Numeric => "大号数字",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::SevenSegment => "七段数码管。",
            Self::Numeric => "大号数字。",
        }
    }

    fn index(self) -> usize {
        match self {
            Self::SevenSegment => 0,
            Self::Numeric => 1,
        }
    }
}

impl SelectItem for GearDisplayStyle {
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
pub struct GearDisplaySettings {
    pub visible: bool,
    pub style: GearDisplayStyle,
    pub seven_segment: seven_segment::SevenSegmentConfig,
    pub numeric: numeric::NumericConfig,
}

impl Default for GearDisplaySettings {
    fn default() -> Self {
        Self {
            visible: true,
            style: GearDisplayStyle::default(),
            seven_segment: seven_segment::SevenSegmentConfig::default(),
            numeric: numeric::NumericConfig::default(),
        }
    }
}

pub struct GearDisplayControls {
    pub style_select: Entity<SelectState<Vec<GearDisplayStyle>>>,
    pub seven_segment: seven_segment::SettingsControls,
    pub numeric: numeric::SettingsControls,
}

impl GearDisplayControls {
    pub fn new(cfg: &GearDisplaySettings, window: &mut Window, cx: &mut Context<Settings>) -> Self {
        let style_select = cx.new(|cx| {
            SelectState::new(
                vec![GearDisplayStyle::SevenSegment, GearDisplayStyle::Numeric],
                Some(IndexPath::new(cfg.style.index())),
                window,
                cx,
            )
        });
        cx.subscribe_in(
            &style_select,
            window,
            |_this, _, event: &SelectEvent<Vec<GearDisplayStyle>>, _window, cx| {
                if let SelectEvent::Confirm(Some(style)) = event {
                    session::update_gear_display(|settings| settings.style = *style);
                    cx.notify();
                }
            },
        )
        .detach();
        Self {
            style_select,
            seven_segment: seven_segment::SettingsControls::new(&cfg.seven_segment, window, cx),
            numeric: numeric::SettingsControls::new(&cfg.numeric, window, cx),
        }
    }

    pub fn sync(&self, cfg: &GearDisplaySettings, window: &mut Window, cx: &mut Context<Settings>) {
        self.style_select.update(cx, |select, cx| {
            select.set_selected_value(&cfg.style, window, cx);
        });
        self.seven_segment.sync(&cfg.seven_segment, window, cx);
        self.numeric.sync(&cfg.numeric, window, cx);
    }
}

pub fn render_hud(props: &HudTelemetryProps, settings: &GearDisplaySettings) -> AnyElement {
    if !settings.visible {
        return div().absolute().into_any_element();
    }
    match settings.style {
        GearDisplayStyle::SevenSegment => seven_segment::render_hud(props, &settings.seven_segment),
        GearDisplayStyle::Numeric => numeric::render_hud(props, &settings.numeric),
    }
}

pub fn render_settings(view: &Settings, colors: &Colors, cx: &mut Context<Settings>) -> AnyElement {
    let visible = session::gear_display().visible;
    let visible_row = setting_row_with_reset(
        colors,
        "显示挡位表",
        "显示当前挡位。",
        "gear-display-visible-reset",
        switch_control(
            "gear-display-visible",
            visible,
            colors,
            cx,
            |_this, visible, cx| {
                session::update_gear_display(|settings| settings.visible = visible);
                cx.notify();
            },
        ),
        !visible,
        None,
        cx,
        |_this, _window, cx| {
            session::update_gear_display(|settings| {
                settings.visible = GearDisplaySettings::default().visible;
            });
            cx.notify();
        },
    );
    let style = session::gear_display().style;
    let style_row = setting_row_with_reset(
        colors,
        "样式",
        style.description(),
        "gear-display-style-reset",
        div().flex_none().w(px(228.)).child(
            Select::new(&view.gear_display.style_select)
                .w_full()
                .with_size(Size::Medium)
                .icon(Icon::empty().path("icons/chevron-down.svg"))
                .bg(colors.container)
                .border_color(colors.border),
        ),
        style != GearDisplayStyle::SevenSegment,
        None,
        cx,
        |this, window, cx| {
            session::update_gear_display(|settings| {
                settings.style = GearDisplayStyle::SevenSegment
            });
            this.gear_display.sync(&session::gear_display(), window, cx);
            cx.notify();
        },
    );
    let body = match style {
        GearDisplayStyle::SevenSegment => seven_segment::render_settings(view, colors, cx),
        GearDisplayStyle::Numeric => numeric::render_settings(view, colors, cx),
    };

    div()
        .flex()
        .flex_col()
        .gap_3()
        .child(
            div()
                .text_xs()
                .text_color(colors.warning)
                .child("此窗口拥有焦点时会暂时覆盖「仅游戏时显示」，强制显示 HUD，便于预览。失焦、离开此页或关闭设置后恢复。"),
        )
        .child(visible_row)
        .child(style_row)
        .child(body)
        .into_any_element()
}
