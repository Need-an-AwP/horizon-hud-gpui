use std::thread;
use std::time::Duration;

#[cfg(target_os = "windows")]
pub(crate) fn spawn() {
    thread::spawn(run);
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn spawn() {}

#[cfg(target_os = "windows")]
const CHARTS_ITEM_ID: &str = "charts";
#[cfg(target_os = "windows")]
const GAME_ONLY_ITEM_ID: &str = "game_only";
#[cfg(target_os = "windows")]
const SETTINGS_ITEM_ID: &str = "settings";
#[cfg(target_os = "windows")]
const EXIT_ITEM_ID: &str = "exit";

#[cfg(target_os = "windows")]
fn run() {
    use tray_icon::menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem};
    use tray_icon::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

    use crate::{hud, platform, settings};

    let icon = load_icon(include_bytes!("../assets/icon.png"));
    let charts_item = CheckMenuItem::with_id(
        CHARTS_ITEM_ID,
        "图表显示",
        true,
        hud::charts_visible(),
        None,
    );
    let game_only_item = CheckMenuItem::with_id(
        GAME_ONLY_ITEM_ID,
        "仅游戏时显示",
        true,
        hud::only_show_in_game(),
        None,
    );
    let settings_item = MenuItem::with_id(SETTINGS_ITEM_ID, "设置", true, None);
    let exit_item = MenuItem::with_id(EXIT_ITEM_ID, "退出", true, None);
    let menu = Menu::with_items(&[
        &charts_item,
        &game_only_item,
        &PredefinedMenuItem::separator(),
        &settings_item,
        &exit_item,
    ])
    .expect("tray menu");

    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(false)
        .with_menu_on_right_click(true)
        .with_tooltip("Horizon HUD")
        .with_icon(icon)
        .build()
        .expect("tray icon");

    loop {
        pump_messages();

        while let Ok(event) = TrayIconEvent::receiver().try_recv() {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                settings::request_open_shift_lights();
            }
        }

        while let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id() == CHARTS_ITEM_ID {
                hud::set_charts_visible(charts_item.is_checked());
            } else if event.id() == GAME_ONLY_ITEM_ID {
                hud::set_only_show_in_game(game_only_item.is_checked());
            } else if event.id() == SETTINGS_ITEM_ID {
                settings::request_open();
            } else if event.id() == EXIT_ITEM_ID {
                platform::quit_app();
                return;
            }
        }

        let charts_visible = hud::charts_visible();
        if charts_item.is_checked() != charts_visible {
            charts_item.set_checked(charts_visible);
        }
        let game_only = hud::only_show_in_game();
        if game_only_item.is_checked() != game_only {
            game_only_item.set_checked(game_only);
        }

        let visible = hud::force_hud_visible() || !game_only || platform::foreground_is_game();
        platform::set_hud_visible(visible);

        thread::sleep(Duration::from_millis(100));
    }
}

#[cfg(target_os = "windows")]
fn load_icon(bytes: &[u8]) -> tray_icon::Icon {
    let image = image::load_from_memory(bytes)
        .expect("decode tray icon")
        .into_rgba8();
    let (width, height) = image.dimensions();
    tray_icon::Icon::from_rgba(image.into_raw(), width, height).expect("tray icon rgba")
}

#[cfg(target_os = "windows")]
fn pump_messages() {
    use windows::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage,
    };

    unsafe {
        let mut msg = MSG::default();
        while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).into() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
