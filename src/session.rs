use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{LazyLock, Mutex, RwLock};

use serde::{Deserialize, Serialize};

use crate::components::gear_display::{GearDisplaySettings, GearDisplayStyle};
use crate::components::gear_display::numeric::NumericConfig;
use crate::components::gear_display::seven_segment::SevenSegmentConfig;
use crate::components::shared::layouts::EdgePosition;
use crate::components::shift_lights::bar::{BarConfig, BarDirection};
use crate::components::shift_lights::porsche_cup::PorscheCupConfig;
use crate::components::shift_lights::{ShiftLightsSettings, ShiftLightsStyle};
use crate::core::general_config::GeneralConfig;
use crate::core::telemetry::{self, CarId};
use crate::core::calibration::CarMemory;

const CONFIG_FILE_NAME: &str = "horizon-hud.toml";
const APP_DIR_NAME: &str = "horizon-hud-gpui";

static APPLYING: AtomicBool = AtomicBool::new(false);
static GENERATION: AtomicU64 = AtomicU64::new(0);
static FORCE_HUD_VISIBLE: AtomicBool = AtomicBool::new(false);
static CONFIG: LazyLock<RwLock<AppConfig>> = LazyLock::new(|| RwLock::new(AppConfig::default()));
static ACTIVE_PATH: Mutex<Option<PathBuf>> = Mutex::new(None);

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct AppConfig {
    pub(crate) general: GeneralConfig,
    pub(crate) shift_lights: ShiftLightsSettings,
    pub(crate) gear_display: GearDisplaySettings,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) calibrated_cars: Vec<CalibratedCar>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            shift_lights: ShiftLightsSettings::default(),
            gear_display: GearDisplaySettings::default(),
            calibrated_cars: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct CalibratedCar {
    pub ordinal: i32,
    pub class: i32,
    pub pi: i32,
    pub fuel_cut_rpm: f32,
}

impl CalibratedCar {
    fn from_car(id: CarId, fuel_cut_rpm: f32) -> Self {
        Self {
            ordinal: id.ordinal,
            class: id.class,
            pi: id.pi,
            fuel_cut_rpm,
        }
    }

    fn matches(&self, id: CarId) -> bool {
        self.ordinal == id.ordinal && self.class == id.class && self.pi == id.pi
    }
}

pub(crate) struct SessionMemory;

impl CarMemory for SessionMemory {
    fn remember(&self) -> bool {
        general().remember_calibrated_cars
    }

    fn fuel_cut_rpm(&self, id: CarId) -> Option<f32> {
        fuel_cut_rpm_for(id)
    }

    fn save(&self, id: CarId, rpm: f32) {
        upsert_calibrated_car(id, rpm);
    }
}

pub(crate) fn load_and_apply() {
    let cfg = load();
    APPLYING.store(true, Ordering::Relaxed);
    telemetry::configure_listen_addr(&cfg.general.listen_host, cfg.general.listen_port);
    telemetry::configure_forward(
        cfg.general.forward_enabled,
        &cfg.general.forward_host,
        cfg.general.forward_port,
    );
    *write_config() = cfg;
    APPLYING.store(false, Ordering::Relaxed);
}

pub(crate) fn persist() {
    if APPLYING.load(Ordering::Relaxed) {
        return;
    }
    let cfg = read_config();
    save(&cfg);
}

pub(crate) fn reset_to_defaults() {
    let cars = read_config().calibrated_cars;
    let mut cfg = AppConfig::default();
    cfg.calibrated_cars = cars;
    APPLYING.store(true, Ordering::Relaxed);
    telemetry::configure_listen_addr(&cfg.general.listen_host, cfg.general.listen_port);
    telemetry::configure_forward(
        cfg.general.forward_enabled,
        &cfg.general.forward_host,
        cfg.general.forward_port,
    );
    *write_config() = cfg.clone();
    APPLYING.store(false, Ordering::Relaxed);
    save(&cfg);
    GENERATION.fetch_add(1, Ordering::Relaxed);
}

pub(crate) fn general() -> GeneralConfig {
    read_config().general
}

pub(crate) fn update_general(f: impl FnOnce(&mut GeneralConfig)) {
    let applied = {
        let mut cfg = write_config();
        let before = cfg.general.clone();
        f(&mut cfg.general);
        if cfg.general == before {
            return;
        }
        let listen_changed = cfg.general.listen_host != before.listen_host
            || cfg.general.listen_port != before.listen_port;
        let forward_changed = cfg.general.forward_enabled != before.forward_enabled
            || cfg.general.forward_host != before.forward_host
            || cfg.general.forward_port != before.forward_port;
        let general = cfg.general.clone();
        (listen_changed, forward_changed, general)
    };
    let (listen_changed, forward_changed, general) = applied;
    if listen_changed {
        telemetry::configure_listen_addr(&general.listen_host, general.listen_port);
    }
    if forward_changed {
        telemetry::configure_forward(
            general.forward_enabled,
            &general.forward_host,
            general.forward_port,
        );
    }
    commit_change();
}

pub(crate) fn shift_lights() -> ShiftLightsSettings {
    read_config().shift_lights
}

pub(crate) fn update_shift_lights(f: impl FnOnce(&mut ShiftLightsSettings)) {
    let changed = {
        let mut cfg = write_config();
        let before = cfg.shift_lights.clone();
        f(&mut cfg.shift_lights);
        cfg.shift_lights != before
    };
    if changed {
        commit_change();
    }
}

pub(crate) fn gear_display() -> GearDisplaySettings {
    read_config().gear_display
}

pub(crate) fn update_gear_display(f: impl FnOnce(&mut GearDisplaySettings)) {
    let changed = {
        let mut cfg = write_config();
        let before = cfg.gear_display.clone();
        f(&mut cfg.gear_display);
        cfg.gear_display != before
    };
    if changed {
        commit_change();
    }
}

pub(crate) fn generation() -> u64 {
    GENERATION.load(Ordering::Relaxed)
}

pub(crate) fn force_hud_visible() -> bool {
    FORCE_HUD_VISIBLE.load(Ordering::Relaxed)
}

pub(crate) fn set_force_hud_visible(force: bool) {
    FORCE_HUD_VISIBLE.store(force, Ordering::Relaxed);
}

pub(crate) fn fuel_cut_rpm_for(id: CarId) -> Option<f32> {
    let cfg = read_config();
    if !cfg.general.remember_calibrated_cars {
        return None;
    }
    cfg.calibrated_cars
        .iter()
        .find(|car| car.matches(id))
        .map(|car| car.fuel_cut_rpm)
}

pub(crate) fn upsert_calibrated_car(id: CarId, fuel_cut_rpm: f32) {
    let changed = {
        let mut cfg = write_config();
        if !cfg.general.remember_calibrated_cars {
            return;
        }
        if let Some(existing) = cfg.calibrated_cars.iter_mut().find(|car| car.matches(id)) {
            if existing.fuel_cut_rpm == fuel_cut_rpm {
                false
            } else {
                existing.fuel_cut_rpm = fuel_cut_rpm;
                true
            }
        } else {
            cfg.calibrated_cars
                .push(CalibratedCar::from_car(id, fuel_cut_rpm));
            true
        }
    };
    if changed {
        persist();
    }
}

pub(crate) fn remove_calibrated_car(id: CarId) {
    let removed = {
        let mut cfg = write_config();
        if !cfg.general.remember_calibrated_cars {
            return;
        }
        let before = cfg.calibrated_cars.len();
        cfg.calibrated_cars.retain(|car| !car.matches(id));
        cfg.calibrated_cars.len() != before
    };
    if removed {
        persist();
    }
}

pub(crate) fn current_car_has_saved_calibration() -> bool {
    crate::core::calibration::current_car_id()
        .and_then(fuel_cut_rpm_for)
        .is_some()
}

pub(crate) fn reset_current_car_calibration() {
    if let Some(id) = crate::core::calibration::current_car_id() {
        remove_calibrated_car(id);
    }
    crate::core::calibration::request_reset();
}

fn commit_change() {
    if !APPLYING.load(Ordering::Relaxed) {
        let cfg = read_config();
        save(&cfg);
    }
    GENERATION.fetch_add(1, Ordering::Relaxed);
}

fn read_config() -> AppConfig {
    CONFIG
        .read()
        .unwrap_or_else(|err| err.into_inner())
        .clone()
}

fn write_config() -> std::sync::RwLockWriteGuard<'static, AppConfig> {
    CONFIG.write().unwrap_or_else(|err| err.into_inner())
}

fn load() -> AppConfig {
    for path in candidate_paths() {
        if !path.is_file() {
            continue;
        }
        let Ok(text) = std::fs::read_to_string(&path) else {
            set_active_path(Some(path));
            return AppConfig::default();
        };
        match toml::from_str::<toml::Value>(&text) {
            Ok(value) => {
                set_active_path(Some(path));
                return migrate_value(value);
            }
            Err(_) => {
                set_active_path(Some(path));
                return AppConfig::default();
            }
        }
    }

    let cfg = AppConfig::default();
    save(&cfg);
    cfg
}

fn migrate_value(value: toml::Value) -> AppConfig {
    let Some(table) = value.as_table() else {
        return AppConfig::default();
    };
    let nested = table.get("general").is_some_and(|item| item.is_table())
        || table.get("shift_lights").is_some_and(|item| item.is_table());
    if nested {
        return AppConfig::deserialize(value).unwrap_or_default();
    }
    migrate_legacy(table)
}

fn migrate_legacy(table: &toml::Table) -> AppConfig {
    let defaults = GeneralConfig::default();
    let general = GeneralConfig {
        only_show_in_game: bool_or(table, "only_show_in_game", defaults.only_show_in_game),
        charts_visible: bool_or(table, "charts_visible", defaults.charts_visible),
        calibrate_hint_visible: bool_or(
            table,
            "calibrate_hint_visible",
            defaults.calibrate_hint_visible,
        ),
        strict_calibrate_conditions: bool_or(
            table,
            "strict_calibrate_conditions",
            defaults.strict_calibrate_conditions,
        ),
        remember_calibrated_cars: bool_or(
            table,
            "remember_calibrated_cars",
            defaults.remember_calibrated_cars,
        ),
        calibrate_ms: usize_or(table, "calibrate_ms", defaults.calibrate_ms),
        listen_host: string_or(table, "listen_host", &defaults.listen_host),
        listen_port: u16_or(table, "listen_port", defaults.listen_port),
        forward_enabled: bool_or(table, "forward_enabled", defaults.forward_enabled),
        forward_host: string_or(table, "forward_host", &defaults.forward_host),
        forward_port: u16_or(table, "forward_port", defaults.forward_port),
    };

    let position = edge_position(table.get("shift_lights_position"));
    let direction = bar_direction(table.get("shift_lights_direction"));
    let dim = f32_or(table, "shift_lights_dim_opacity", BarConfig::default().dim_opacity);
    let blink_percent = f32_or(
        table,
        "shift_lights_blink_percent",
        BarConfig::default().blink_percent,
    );
    let blink_ms = usize_or(table, "shift_lights_blink_ms", BarConfig::default().blink_ms);
    let flat_thickness = usize_or(
        table,
        "shift_lights_thickness_px",
        BarConfig::default().thickness_px,
    );
    let flat_gap = usize_or(table, "shift_lights_gap_px", BarConfig::default().gap_px);
    let flat_width = usize_or(
        table,
        "shift_lights_width_percent",
        BarConfig::default().width_percent,
    );
    let flat_offset = usize_or(
        table,
        "shift_lights_offset_px",
        BarConfig::default().offset_px,
    );
    let flat_lit = f32_or(
        table,
        "shift_lights_lit_opacity",
        BarConfig::default().lit_opacity,
    );
    let circles_thickness = opt_usize(table, "shift_lights_circles_thickness_px");
    let circles_gap = opt_usize(table, "shift_lights_circles_gap_px");
    let circles_width = opt_usize(table, "shift_lights_circles_width_percent");
    let circles_offset = opt_usize(table, "shift_lights_circles_offset_px");
    let circles_lit = opt_f32(table, "shift_lights_circles_lit_opacity");
    let has_circles = circles_thickness.is_some()
        || circles_gap.is_some()
        || circles_width.is_some()
        || circles_offset.is_some()
        || circles_lit.is_some();
    let porsche_style = matches!(
        table.get("shift_lights_style").and_then(|value| value.as_str()),
        Some("symmetric_circles") | Some("porsche_cup")
    );
    let porsche_defaults = PorscheCupConfig::default();

    let (bar, porsche) = if has_circles {
        let mut bar = BarConfig {
            thickness_px: flat_thickness,
            gap_px: flat_gap,
            width_percent: flat_width,
            offset_px: flat_offset,
            lit_opacity: flat_lit,
            ..BarConfig::default()
        };
        let mut porsche = PorscheCupConfig {
            diameter_px: circles_thickness.unwrap_or(porsche_defaults.diameter_px),
            gap_px: circles_gap.unwrap_or(porsche_defaults.gap_px),
            width_percent: circles_width.unwrap_or(porsche_defaults.width_percent),
            offset_px: circles_offset.unwrap_or(porsche_defaults.offset_px),
            lit_opacity: circles_lit.unwrap_or(porsche_defaults.lit_opacity),
            ..PorscheCupConfig::default()
        };
        apply_shared_bar(&mut bar, position, direction, dim, blink_percent, blink_ms);
        apply_shared_porsche(&mut porsche, position, dim, blink_percent, blink_ms);
        (bar, porsche)
    } else if porsche_style {
        let mut bar = BarConfig::default();
        let mut porsche = PorscheCupConfig {
            diameter_px: flat_thickness,
            gap_px: flat_gap,
            width_percent: flat_width,
            offset_px: flat_offset,
            lit_opacity: flat_lit,
            ..PorscheCupConfig::default()
        };
        apply_shared_bar(&mut bar, position, direction, dim, blink_percent, blink_ms);
        apply_shared_porsche(&mut porsche, position, dim, blink_percent, blink_ms);
        (bar, porsche)
    } else {
        let mut bar = BarConfig {
            thickness_px: flat_thickness,
            gap_px: flat_gap,
            width_percent: flat_width,
            offset_px: flat_offset,
            lit_opacity: flat_lit,
            ..BarConfig::default()
        };
        let mut porsche = PorscheCupConfig::default();
        apply_shared_bar(&mut bar, position, direction, dim, blink_percent, blink_ms);
        apply_shared_porsche(&mut porsche, position, dim, blink_percent, blink_ms);
        (bar, porsche)
    };

    let segment_defaults = SevenSegmentConfig::default();
    let gear_display = GearDisplaySettings {
        visible: bool_or(table, "gear_display_visible", true),
        style: GearDisplayStyle::SevenSegment,
        seven_segment: SevenSegmentConfig {
            x_ratio: f32_or(table, "gear_display_x_ratio", segment_defaults.x_ratio),
            y_ratio: f32_or(table, "gear_display_y_ratio", segment_defaults.y_ratio),
            size_px: usize_or(table, "gear_display_size_px", segment_defaults.size_px),
            lit_opacity: f32_or(
                table,
                "gear_display_lit_opacity",
                segment_defaults.lit_opacity,
            ),
            dim_opacity: f32_or(
                table,
                "gear_display_dim_opacity",
                segment_defaults.dim_opacity,
            ),
        },
        numeric: NumericConfig::default(),
    };

    AppConfig {
        general,
        shift_lights: ShiftLightsSettings {
            style: if porsche_style {
                ShiftLightsStyle::PorscheCup
            } else {
                ShiftLightsStyle::Bar
            },
            bar,
            porsche_cup: porsche,
        },
        gear_display,
        calibrated_cars: table
            .get("calibrated_cars")
            .cloned()
            .and_then(|value| Vec::<CalibratedCar>::deserialize(value).ok())
            .unwrap_or_default(),
    }
}

fn apply_shared_bar(
    bar: &mut BarConfig,
    position: EdgePosition,
    direction: BarDirection,
    dim: f32,
    blink_percent: f32,
    blink_ms: usize,
) {
    bar.position = position;
    bar.direction = if direction.is_compatible_with(position) {
        direction
    } else {
        BarDirection::default_for(position)
    };
    bar.dim_opacity = dim;
    bar.blink_percent = blink_percent;
    bar.blink_ms = blink_ms;
}

fn apply_shared_porsche(
    cfg: &mut PorscheCupConfig,
    position: EdgePosition,
    dim: f32,
    blink_percent: f32,
    blink_ms: usize,
) {
    cfg.position = position;
    cfg.dim_opacity = dim;
    cfg.blink_percent = blink_percent;
    cfg.blink_ms = blink_ms;
}

fn edge_position(value: Option<&toml::Value>) -> EdgePosition {
    match value.and_then(|item| item.as_str()) {
        Some("bottom") => EdgePosition::Bottom,
        Some("right") => EdgePosition::Right,
        Some("left") => EdgePosition::Left,
        _ => EdgePosition::Top,
    }
}

fn bar_direction(value: Option<&toml::Value>) -> BarDirection {
    match value.and_then(|item| item.as_str()) {
        Some("right_to_left") => BarDirection::RightToLeft,
        Some("bottom_to_top") => BarDirection::BottomToTop,
        Some("top_to_bottom") => BarDirection::TopToBottom,
        _ => BarDirection::LeftToRight,
    }
}

fn bool_or(table: &toml::Table, key: &str, default: bool) -> bool {
    table
        .get(key)
        .and_then(|value| value.as_bool())
        .unwrap_or(default)
}

fn f32_or(table: &toml::Table, key: &str, default: f32) -> f32 {
    match table.get(key) {
        Some(toml::Value::Float(value)) => *value as f32,
        Some(toml::Value::Integer(value)) => *value as f32,
        _ => default,
    }
}

fn opt_f32(table: &toml::Table, key: &str) -> Option<f32> {
    match table.get(key) {
        Some(toml::Value::Float(value)) => Some(*value as f32),
        Some(toml::Value::Integer(value)) => Some(*value as f32),
        Some(_) => Some(PorscheCupConfig::default().lit_opacity),
        None => None,
    }
}

fn usize_or(table: &toml::Table, key: &str, default: usize) -> usize {
    table
        .get(key)
        .and_then(|value| value.as_integer())
        .filter(|value| *value >= 0)
        .map(|value| value as usize)
        .unwrap_or(default)
}

fn opt_usize(table: &toml::Table, key: &str) -> Option<usize> {
    match table.get(key) {
        Some(toml::Value::Integer(value)) if *value >= 0 => Some(*value as usize),
        Some(_) => Some(0),
        None => None,
    }
}

fn u16_or(table: &toml::Table, key: &str, default: u16) -> u16 {
    table
        .get(key)
        .and_then(|value| value.as_integer())
        .filter(|value| (0..=u16::MAX as i64).contains(value))
        .map(|value| value as u16)
        .unwrap_or(default)
}

fn string_or(table: &toml::Table, key: &str, default: &str) -> String {
    table
        .get(key)
        .and_then(|value| value.as_str())
        .unwrap_or(default)
        .to_string()
}

fn save(cfg: &AppConfig) {
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

fn store_to(path: &Path, cfg: &AppConfig) -> bool {
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
