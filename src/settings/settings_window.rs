use std::sync::Mutex;

use gpui::{
    App, AppContext, Bounds, Entity, Global, TitlebarOptions, Window, WindowBounds, WindowHandle,
    WindowKind, WindowOptions, px, size,
};
use gpui_component::Root;

use crate::settings_section::SettingsSection;

use super::Settings;

static WINDOW: Mutex<Option<WindowHandle<Root>>> = Mutex::new(None);

struct OpenSettings(Entity<Settings>);

impl Global for OpenSettings {}

pub(super) fn open(cx: &mut App, section: Option<SettingsSection>) {
    let existing = WINDOW.lock().unwrap().as_ref().copied();
    if let Some(handle) = existing {
        if handle
            .update(cx, |_, window, cx| {
                window.activate_window();
                if let Some(section) = section {
                    select_open_section(section, window, cx);
                }
            })
            .is_ok()
        {
            cx.activate(true);
            return;
        }
    }

    let initial_section = section.unwrap_or(SettingsSection::Overview);
    let bounds = Bounds::centered(None, size(px(960.), px(640.)), cx);
    let Ok(handle) = cx.open_window(
        WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            window_min_size: Some(size(px(760.), px(540.))),
            titlebar: Some(TitlebarOptions {
                title: Some("Horizon HUD".into()),
                ..Default::default()
            }),
            kind: WindowKind::Normal,
            focus: true,
            is_movable: true,
            is_resizable: true,
            window_background: gpui::WindowBackgroundAppearance::Opaque,
            ..Default::default()
        },
        |window, cx| {
            crate::platform::configure_settings_window(window);
            let settings = cx.new(|cx| Settings::new(window, cx, initial_section));
            cx.set_global(OpenSettings(settings.clone()));
            cx.new(|cx| Root::new(settings, window, cx))
        },
    ) else {
        return;
    };
    *WINDOW.lock().unwrap() = Some(handle);
    cx.activate(true);
}

fn select_open_section(section: SettingsSection, window: &Window, cx: &mut App) {
    let Some(settings) = cx.try_global::<OpenSettings>().map(|open| open.0.clone()) else {
        return;
    };
    settings.update(cx, |this, cx| {
        this.select_section(section, window, cx);
    });
}

pub(super) fn sync(cx: &mut App) {
    let existing = WINDOW.lock().unwrap().as_ref().copied();
    let Some(handle) = existing else {
        return;
    };
    if handle.update(cx, |_, _, _| ()).is_err() {
        crate::hud::set_force_hud_visible(false);
        return;
    }
    let Some(settings) = cx.try_global::<OpenSettings>().map(|open| open.0.clone()) else {
        return;
    };
    settings.update(cx, |this, cx| {
        this.sync_hud_settings(cx);
    });
}
