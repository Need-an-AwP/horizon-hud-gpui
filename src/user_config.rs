use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};

use crate::config::{
    DEFAULT_CALIBRATE_MS, DEFAULT_GEAR_DISPLAY_DIM_OPACITY, DEFAULT_GEAR_DISPLAY_LIT_OPACITY,
    DEFAULT_GEAR_DISPLAY_SIZE_PX, DEFAULT_GEAR_DISPLAY_X_RATIO, DEFAULT_GEAR_DISPLAY_Y_RATIO,
    DEFAULT_LISTEN_HOST, DEFAULT_LISTEN_PORT, DEFAULT_SHIFT_LIGHTS_BLINK_PERCENT,
    DEFAULT_SHIFT_LIGHTS_DIM_OPACITY, DEFAULT_SHIFT_LIGHTS_GAP_PX,
    DEFAULT_SHIFT_LIGHTS_LIT_OPACITY, DEFAULT_SHIFT_LIGHTS_OFFSET_PX,
    DEFAULT_SHIFT_LIGHTS_THICKNESS_PX, DEFAULT_SHIFT_LIGHTS_WIDTH_PERCENT,
};
use crate::hud::{self, ShiftLightsDirection, ShiftLightsPosition};
use crate::telemetry;

const CONFIG_FILE_NAME: &str = "horizon-hud.toml";
const APP_DIR_NAME: &str = "horizon-hud-gpui";

static APPLYING: AtomicBool = AtomicBool::new(false);
static ACTIVE_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct UserConfig {
    pub only_show_in_game: bool,
    pub charts_visible: bool,
    pub calibrate_hint_visible: bool,
    pub shift_lights_position: ShiftLightsPosition,
    pub shift_lights_direction: ShiftLightsDirection,
    pub shift_lights_lit_opacity: f32,
    pub shift_lights_dim_opacity: f32,
    pub shift_lights_offset_px: usize,
    pub shift_lights_thickness_px: usize,
    pub shift_lights_gap_px: usize,
    pub shift_lights_width_percent: usize,
    pub shift_lights_blink_percent: f32,
    pub gear_display_visible: bool,
    pub gear_display_x_ratio: f32,
    pub gear_display_y_ratio: f32,
    pub gear_display_size_px: usize,
    pub gear_display_lit_opacity: f32,
    pub gear_display_dim_opacity: f32,
    pub calibrate_ms: usize,
    pub listen_host: String,
    pub listen_port: u16,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            only_show_in_game: true,
            charts_visible: false,
            calibrate_hint_visible: false,
            shift_lights_position: ShiftLightsPosition::Top,
            shift_lights_direction: ShiftLightsDirection::LeftToRight,
            shift_lights_lit_opacity: DEFAULT_SHIFT_LIGHTS_LIT_OPACITY,
            shift_lights_dim_opacity: DEFAULT_SHIFT_LIGHTS_DIM_OPACITY,
            shift_lights_offset_px: DEFAULT_SHIFT_LIGHTS_OFFSET_PX,
            shift_lights_thickness_px: DEFAULT_SHIFT_LIGHTS_THICKNESS_PX,
            shift_lights_gap_px: DEFAULT_SHIFT_LIGHTS_GAP_PX,
            shift_lights_width_percent: DEFAULT_SHIFT_LIGHTS_WIDTH_PERCENT,
            shift_lights_blink_percent: DEFAULT_SHIFT_LIGHTS_BLINK_PERCENT,
            gear_display_visible: true,
            gear_display_x_ratio: DEFAULT_GEAR_DISPLAY_X_RATIO,
            gear_display_y_ratio: DEFAULT_GEAR_DISPLAY_Y_RATIO,
            gear_display_size_px: DEFAULT_GEAR_DISPLAY_SIZE_PX,
            gear_display_lit_opacity: DEFAULT_GEAR_DISPLAY_LIT_OPACITY,
            gear_display_dim_opacity: DEFAULT_GEAR_DISPLAY_DIM_OPACITY,
            calibrate_ms: DEFAULT_CALIBRATE_MS,
            listen_host: DEFAULT_LISTEN_HOST.to_string(),
            listen_port: DEFAULT_LISTEN_PORT,
        }
    }
}

impl UserConfig {
    fn from_runtime() -> Self {
        let (listen_host, listen_port) = telemetry::listen_host_port();
        let (gear_display_x_ratio, gear_display_y_ratio) = hud::gear_display_position_ratio();
        Self {
            only_show_in_game: hud::only_show_in_game(),
            charts_visible: hud::charts_visible(),
            calibrate_hint_visible: hud::calibrate_hint_visible(),
            shift_lights_position: hud::shift_lights_position(),
            shift_lights_direction: hud::shift_lights_direction(),
            shift_lights_lit_opacity: hud::shift_lights_lit_opacity(),
            shift_lights_dim_opacity: hud::shift_lights_dim_opacity(),
            shift_lights_offset_px: hud::shift_lights_offset_px(),
            shift_lights_thickness_px: hud::shift_lights_thickness_px(),
            shift_lights_gap_px: hud::shift_lights_gap_px(),
            shift_lights_width_percent: hud::shift_lights_width_percent(),
            shift_lights_blink_percent: hud::shift_lights_blink_percent(),
            gear_display_visible: hud::gear_display_visible(),
            gear_display_x_ratio,
            gear_display_y_ratio,
            gear_display_size_px: hud::gear_display_size_px(),
            gear_display_lit_opacity: hud::gear_display_lit_opacity(),
            gear_display_dim_opacity: hud::gear_display_dim_opacity(),
            calibrate_ms: hud::calibrate_ms(),
            listen_host,
            listen_port,
        }
    }

    fn apply(&self) {
        hud::set_only_show_in_game(self.only_show_in_game);
        hud::set_charts_visible(self.charts_visible);
        hud::set_calibrate_hint_visible(self.calibrate_hint_visible);
        hud::set_gear_display_visible(self.gear_display_visible);
        hud::set_shift_lights_position(self.shift_lights_position);
        hud::set_shift_lights_direction(self.shift_lights_direction);
        let _ = hud::set_shift_lights_lit_opacity(self.shift_lights_lit_opacity);
        let _ = hud::set_shift_lights_dim_opacity(self.shift_lights_dim_opacity);
        hud::set_shift_lights_offset_px(self.shift_lights_offset_px);
        let _ = hud::set_shift_lights_thickness_px(self.shift_lights_thickness_px);
        hud::set_shift_lights_gap_px(self.shift_lights_gap_px);
        let _ = hud::set_shift_lights_width_percent(self.shift_lights_width_percent);
        let _ = hud::set_shift_lights_blink_percent(self.shift_lights_blink_percent);
        let _ = hud::set_gear_display_position_ratio(
            self.gear_display_x_ratio,
            self.gear_display_y_ratio,
        );
        let _ = hud::set_gear_display_size_px(self.gear_display_size_px);
        let _ = hud::set_gear_display_lit_opacity(self.gear_display_lit_opacity);
        let _ = hud::set_gear_display_dim_opacity(self.gear_display_dim_opacity);
        let _ = hud::set_calibrate_ms(self.calibrate_ms);
        telemetry::configure_listen_addr(&self.listen_host, self.listen_port);
    }
}

pub(crate) fn load_and_apply() {
    let cfg = load();
    APPLYING.store(true, Ordering::Relaxed);
    cfg.apply();
    APPLYING.store(false, Ordering::Relaxed);
}

pub(crate) fn persist() {
    if APPLYING.load(Ordering::Relaxed) {
        return;
    }
    save(&UserConfig::from_runtime());
}

pub(crate) fn reset_to_defaults() {
    let cfg = UserConfig::default();
    APPLYING.store(true, Ordering::Relaxed);
    cfg.apply();
    APPLYING.store(false, Ordering::Relaxed);
    save(&cfg);
}

fn load() -> UserConfig {
    for path in candidate_paths() {
        if !path.is_file() {
            continue;
        }
        match confy::load_path::<UserConfig>(&path) {
            Ok(cfg) => {
                set_active_path(Some(path));
                return cfg;
            }
            Err(_) => {
                set_active_path(Some(path));
                return UserConfig::default();
            }
        }
    }

    let cfg = UserConfig::default();
    save(&cfg);
    cfg
}

fn save(cfg: &UserConfig) {
    if let Some(path) = active_path()
        && store_to(&path, cfg)
    {
        return;
    }

    for path in candidate_paths() {
        if store_to(&path, cfg) {
            set_active_path(Some(path));
            return;
        }
    }
}

fn store_to(path: &Path, cfg: &UserConfig) -> bool {
    if let Some(parent) = path.parent()
        && std::fs::create_dir_all(parent).is_err()
    {
        return false;
    }
    confy::store_path(path, cfg).is_ok()
}

fn candidate_paths() -> Vec<PathBuf> {
    [exe_dir_config_path(), appdata_config_path()]
        .into_iter()
        .flatten()
        .collect()
}

fn exe_dir_config_path() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join(CONFIG_FILE_NAME)))
}

fn appdata_config_path() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var_os("APPDATA").map(|appdata| {
            PathBuf::from(appdata)
                .join(APP_DIR_NAME)
                .join(CONFIG_FILE_NAME)
        })
    }
    #[cfg(not(windows))]
    {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
            .map(|dir| dir.join(APP_DIR_NAME).join(CONFIG_FILE_NAME))
    }
}

pub(crate) fn active_path() -> Option<PathBuf> {
    ACTIVE_PATH
        .lock()
        .unwrap_or_else(|err| err.into_inner())
        .clone()
}

pub(crate) fn open_active() {
    let Some(path) = active_path() else {
        return;
    };

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        let _ = std::process::Command::new("cmd")
            .args(["/C", "start", "", &path.to_string_lossy()])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn();
    }

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(&path).spawn();
    }

    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let _ = std::process::Command::new("xdg-open").arg(&path).spawn();
    }
}

fn set_active_path(path: Option<PathBuf>) {
    *ACTIVE_PATH.lock().unwrap_or_else(|err| err.into_inner()) = path;
}
