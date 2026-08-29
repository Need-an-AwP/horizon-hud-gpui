mod assets;
mod config;
mod hud;
mod platform;
mod ring_buffer;
mod settings;
mod settings_section;
mod telemetry;
mod tray;
mod ui;
mod user_config;

use std::time::Duration;

use gpui::{
    App, AppContext, Application, Timer, WindowBackgroundAppearance, WindowBounds, WindowKind,
    WindowOptions,
};

use hud::RpmHud;
use platform::{configure_hud_window, register_hud_window};

fn main() {
    user_config::load_and_apply();
    tray::spawn();
    Application::new()
        .with_assets(assets::Assets)
        .run(|cx: &mut App| {
            gpui_component::init(cx);
            cx.init_colors();

            let display = cx
                .primary_display()
                .or_else(|| cx.displays().into_iter().next());
            let display_id = display.as_ref().map(|d| d.id());
            let bounds = display.map(|d| d.bounds()).unwrap_or_default();

            cx.spawn(async move |cx| {
                loop {
                    Timer::after(Duration::from_millis(50)).await;
                    let _ = cx.update(|cx| {
                        settings::poll_open(cx);
                        settings::poll_sync(cx);
                        platform::poll_quit(cx);
                    });
                }
            })
            .detach();

            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Fullscreen(bounds)),
                    titlebar: None,
                    kind: WindowKind::PopUp,
                    display_id,
                    focus: false,
                    is_movable: false,
                    is_resizable: false,
                    window_background: WindowBackgroundAppearance::Transparent,
                    ..Default::default()
                },
                |window, cx| {
                    register_hud_window(window);
                    configure_hud_window(window);
                    cx.new(RpmHud::new)
                },
            )
            .unwrap();
            cx.activate(true);
        });
}
