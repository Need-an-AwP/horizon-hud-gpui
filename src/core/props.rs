#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GearValue {
    Disconnected,
    Reverse,
    Neutral,
    Forward(u8),
}

impl GearValue {
    /// 与现有 RpmHud::gear_display 一致：None->Disconnected, 0->Reverse, >=11->Neutral, 其余 Forward
    pub(crate) fn from_raw(gear: Option<u8>) -> Self {
        match gear {
            None => Self::Disconnected,
            Some(0) => Self::Reverse,
            Some(gear) if gear >= 11 => Self::Neutral,
            Some(gear) => Self::Forward(gear),
        }
    }

    /// 七段数码管用：Disconnected "-"（仅 G 段）, Reverse "r", Neutral "n", Forward 十进制数字字符串（1..=10，不要大于 10 的档进到这里）
    pub(crate) fn segment_text(self) -> String {
        match self {
            Self::Disconnected => "-".into(),
            Self::Reverse => "r".into(),
            Self::Neutral => "n".into(),
            Self::Forward(gear) => gear.to_string(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub(crate) enum CalibrationState {
    Uncalibrated {
        progress: f32,
        elapsed_s: f32,
        duration_s: f32,
    },
    Ready { limit_rpm: f32, rpm_ratio: f32 },
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub(crate) struct HudTelemetryProps {
    pub(crate) has_data: bool,
    pub(crate) rpm: f32,
    pub(crate) max_rpm: f32,
    pub(crate) calibration: CalibrationState,
    pub(crate) gear: GearValue,
    pub(crate) power_w: f32,
    pub(crate) torque_nm: f32,
    pub(crate) now_ms: u128,
    pub(crate) strict_calibrate: bool,
    pub(crate) calibrate_hint_visible: bool,
    pub(crate) listen_addr: String,
}

#[derive(Clone, Debug)]
pub(crate) struct ChartFrame {
    pub(crate) rpm: Vec<f32>,
    pub(crate) torque: Vec<f32>,
    pub(crate) power: Vec<f32>,
    pub(crate) limit_rpm: Option<f32>,
    pub(crate) rpm_now: f32,
    pub(crate) torque_now: f32,
    pub(crate) power_now: f32,
    pub(crate) max_rpm: f32,
    pub(crate) has_data: bool,
}
