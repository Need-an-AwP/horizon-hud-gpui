use std::sync::atomic::{AtomicBool, Ordering};

use gpui::{App, Window};

#[cfg(target_os = "windows")]
use std::sync::OnceLock;

#[cfg(target_os = "windows")]
use raw_window_handle::{HasWindowHandle, RawWindowHandle};
#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{HWND, RECT};

static QUIT_REQUESTED: AtomicBool = AtomicBool::new(false);

#[cfg(target_os = "windows")]
static HUD_HWND: OnceLock<isize> = OnceLock::new();
#[cfg(target_os = "windows")]
static INSTANCE_MUTEX: OnceLock<isize> = OnceLock::new();
#[cfg(target_os = "windows")]
static ACTIVATE_EVENT: OnceLock<isize> = OnceLock::new();

#[cfg(target_os = "windows")]
fn window_hwnd(window: &Window) -> Option<HWND> {
    let handle = HasWindowHandle::window_handle(window).ok()?;
    let RawWindowHandle::Win32(win32) = handle.as_raw() else {
        return None;
    };
    Some(HWND(win32.hwnd.get() as _))
}

#[cfg(target_os = "windows")]
pub(crate) fn register_hud_window(window: &Window) {
    if let Some(hwnd) = window_hwnd(window) {
        let _ = HUD_HWND.set(hwnd.0 as isize);
    }
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn register_hud_window(_window: &Window) {}

#[cfg(target_os = "windows")]
pub(crate) fn configure_hud_window(window: &Window) {
    use windows::Win32::Graphics::Dwm::{
        DWMNCRP_DISABLED, DWMWA_NCRENDERING_POLICY, DwmSetWindowAttribute,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GWL_EXSTYLE, GetWindowLongPtrW, HWND_TOPMOST, SWP_FRAMECHANGED, SWP_NOACTIVATE,
        SWP_SHOWWINDOW, SetWindowLongPtrW, SetWindowPos, WS_EX_LAYERED, WS_EX_NOACTIVATE,
        WS_EX_TRANSPARENT,
    };

    let Some(hwnd) = window_hwnd(window) else {
        return;
    };
    unsafe {
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        SetWindowLongPtrW(
            hwnd,
            GWL_EXSTYLE,
            ex_style
                | WS_EX_LAYERED.0 as isize
                | WS_EX_TRANSPARENT.0 as isize
                | WS_EX_NOACTIVATE.0 as isize,
        );

        let policy = DWMNCRP_DISABLED;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_NCRENDERING_POLICY,
            &policy as *const _ as *const _,
            std::mem::size_of::<i32>() as u32,
        );

        if let Some((x, y, width, height)) = monitor_cover_rect(hwnd) {
            let _ = SetWindowPos(
                hwnd,
                Some(HWND_TOPMOST),
                x,
                y,
                width,
                height,
                SWP_NOACTIVATE | SWP_FRAMECHANGED | SWP_SHOWWINDOW,
            );
        } else {
            let _ = SetWindowPos(
                hwnd,
                Some(HWND_TOPMOST),
                0,
                0,
                0,
                0,
                SWP_NOACTIVATE | SWP_FRAMECHANGED,
            );
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn configure_hud_window(_window: &Window) {}

#[cfg(target_os = "windows")]
pub(crate) fn configure_settings_window(window: &Window) {
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateIcon, ICON_BIG, ICON_SMALL, SendMessageW, WM_SETICON,
    };

    let Some(hwnd) = window_hwnd(window) else {
        return;
    };
    let Ok(image) = image::load_from_memory(include_bytes!("../assets/icon.png")) else {
        return;
    };
    let image = image.into_rgba8();
    let (width, height) = image.dimensions();
    let mut pixels = image.into_raw();
    for pixel in pixels.chunks_exact_mut(4) {
        pixel.swap(0, 2);
    }
    let mask_stride = (width as usize).div_ceil(32) * 4;
    let mask = vec![0_u8; mask_stride * height as usize];

    unsafe {
        let Ok(icon) = CreateIcon(
            None,
            width as i32,
            height as i32,
            1,
            32,
            mask.as_ptr(),
            pixels.as_ptr(),
        ) else {
            return;
        };
        SendMessageW(
            hwnd,
            WM_SETICON,
            Some(WPARAM(ICON_SMALL as usize)),
            Some(LPARAM(icon.0 as isize)),
        );
        SendMessageW(
            hwnd,
            WM_SETICON,
            Some(WPARAM(ICON_BIG as usize)),
            Some(LPARAM(icon.0 as isize)),
        );
    }
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn configure_settings_window(_window: &Window) {}

#[cfg(target_os = "windows")]
fn monitor_cover_rect(hwnd: HWND) -> Option<(i32, i32, i32, i32)> {
    use windows::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow,
    };

    unsafe {
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if !GetMonitorInfoW(monitor, &mut info).as_bool() {
            return None;
        }

        let monitor = info.rcMonitor;
        let insets = gpui_client_area_insets(hwnd);
        Some((
            monitor.left - insets.left,
            monitor.top - insets.top,
            (monitor.right - monitor.left) + insets.left + insets.right,
            (monitor.bottom - monitor.top) + insets.top + insets.bottom,
        ))
    }
}

// GPUI shrinks the client area in WM_NCCALCSIZE when titlebar is hidden but the
// window is not in its internal fullscreen mode. Expand the outer window rect so
// the client area still covers the full monitor.
#[cfg(target_os = "windows")]
fn gpui_client_area_insets(hwnd: HWND) -> RECT {
    use windows::Win32::UI::HiDpi::{GetDpiForWindow, GetSystemMetricsForDpi};
    use windows::Win32::UI::WindowsAndMessaging::{SM_CXPADDEDBORDER, SM_CXSIZEFRAME};

    unsafe {
        let dpi = GetDpiForWindow(hwnd);
        let frame = GetSystemMetricsForDpi(SM_CXSIZEFRAME, dpi)
            + GetSystemMetricsForDpi(SM_CXPADDEDBORDER, dpi);
        let top = (dpi as f32 / 96.0).round() as i32;
        RECT {
            left: frame,
            top,
            right: frame,
            bottom: frame,
        }
    }
}

#[cfg(target_os = "windows")]
const GAME_EXE: &str = "forzahorizon6.exe";

#[cfg(target_os = "windows")]
pub(crate) fn foreground_is_game() -> bool {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
        QueryFullProcessImageNameW,
    };
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};
    use windows::core::PWSTR;

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return false;
        }
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return false;
        }
        let Ok(process) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) else {
            return false;
        };
        let mut buf = [0u16; 1024];
        let mut size = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            PWSTR(buf.as_mut_ptr()),
            &mut size,
        );
        let _ = CloseHandle(process);
        if ok.is_err() || size == 0 {
            return false;
        }
        let path = String::from_utf16_lossy(&buf[..size as usize]);
        path.rsplit(['\\', '/'])
            .next()
            .is_some_and(|name| name.eq_ignore_ascii_case(GAME_EXE))
    }
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn foreground_is_game() -> bool {
    true
}

#[cfg(target_os = "windows")]
pub(crate) fn set_hud_visible(visible: bool) {
    use windows::Win32::UI::WindowsAndMessaging::{SW_HIDE, SW_SHOWNA, ShowWindow};

    let Some(hwnd_value) = HUD_HWND.get() else {
        return;
    };
    let hwnd = HWND(*hwnd_value as _);
    unsafe {
        let _ = ShowWindow(hwnd, if visible { SW_SHOWNA } else { SW_HIDE });
    }
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn set_hud_visible(_visible: bool) {}

pub(crate) fn quit_app() {
    QUIT_REQUESTED.store(true, Ordering::Relaxed);
}

pub(crate) fn poll_quit(cx: &mut App) {
    if QUIT_REQUESTED.swap(false, Ordering::Relaxed) {
        cx.quit();
    }
}

#[cfg(target_os = "windows")]
const INSTANCE_MUTEX_NAME: windows::core::PCWSTR =
    windows::core::w!("Local\\horizon-hud-gpui.single-instance");
#[cfg(target_os = "windows")]
const ACTIVATE_EVENT_NAME: windows::core::PCWSTR =
    windows::core::w!("Local\\horizon-hud-gpui.activate");

pub(crate) fn try_become_singleton() -> bool {
    #[cfg(target_os = "windows")]
    {
        become_windows_singleton()
    }
    #[cfg(not(target_os = "windows"))]
    {
        true
    }
}

#[cfg(target_os = "windows")]
fn become_windows_singleton() -> bool {
    use windows::Win32::Foundation::{CloseHandle, WAIT_ABANDONED, WAIT_OBJECT_0, WAIT_TIMEOUT};
    use windows::Win32::System::Threading::{
        CreateEventW, CreateMutexW, EVENT_MODIFY_STATE, OpenEventW, SetEvent, WaitForSingleObject,
    };

    unsafe {
        let Ok(mutex) = CreateMutexW(None, false, INSTANCE_MUTEX_NAME) else {
            return true;
        };
        let wait = WaitForSingleObject(mutex, 0);
        if wait == WAIT_TIMEOUT {
            let _ = CloseHandle(mutex);
            if let Ok(event) = OpenEventW(EVENT_MODIFY_STATE, false, ACTIVATE_EVENT_NAME) {
                let _ = SetEvent(event);
                let _ = CloseHandle(event);
            }
            return false;
        }
        if wait != WAIT_OBJECT_0 && wait != WAIT_ABANDONED {
            let _ = CloseHandle(mutex);
            return true;
        }
        let _ = INSTANCE_MUTEX.set(mutex.0 as isize);
        if let Ok(event) = CreateEventW(None, false, false, ACTIVATE_EVENT_NAME) {
            let _ = ACTIVATE_EVENT.set(event.0 as isize);
        }
        true
    }
}

pub(crate) fn take_activate_request() -> bool {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Foundation::{HANDLE, WAIT_OBJECT_0};
        use windows::Win32::System::Threading::WaitForSingleObject;

        let Some(event) = ACTIVATE_EVENT.get() else {
            return false;
        };
        unsafe { WaitForSingleObject(HANDLE(*event as _), 0) == WAIT_OBJECT_0 }
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}
