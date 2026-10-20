#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum SettingsSection {
    Overview,
    Hud,
    ShiftLights,
    GearDisplay,
    Telemetry,
    Calibration,
    About,
}

impl SettingsSection {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Overview => "概览",
            Self::Hud => "HUD 显示",
            Self::ShiftLights => "转速指示",
            Self::GearDisplay => "挡位显示",
            Self::Telemetry => "遥测数据",
            Self::Calibration => "校准",
            Self::About => "关于",
        }
    }

    pub(super) fn description(self) -> &'static str {
        match self {
            Self::Overview => "常用选项与运行状态。",
            Self::Hud => "显示条件与图表。",
            Self::ShiftLights => "位置、尺寸与显示效果。",
            Self::GearDisplay => "位置、大小与透明度。",
            Self::Telemetry => "监听地址与数据包转发。",
            Self::Calibration => "校准时长、条件与提示。",
            Self::About => "应用信息与支持资源。",
        }
    }

    pub(super) fn icon_path(self) -> &'static str {
        match self {
            Self::Overview => "icons/layout-dashboard.svg",
            Self::Hud => "icons/monitor.svg",
            Self::ShiftLights => "icons/sliders-horizontal.svg",
            Self::GearDisplay => "icons/cog.svg",
            Self::Telemetry => "icons/gauge.svg",
            Self::Calibration => "icons/timer.svg",
            Self::About => "icons/info.svg",
        }
    }

    pub(super) fn index(self) -> u64 {
        match self {
            Self::Overview => 0,
            Self::Hud => 1,
            Self::ShiftLights => 2,
            Self::GearDisplay => 3,
            Self::Telemetry => 4,
            Self::Calibration => 5,
            Self::About => 6,
        }
    }

    pub(super) fn placeholder_rows(self) -> [(&'static str, &'static str); 3] {
        match self {
            Self::Overview => [
                ("HUD 状态", "覆盖层与游戏连接状态。"),
                ("快速操作", "常用显示和校准操作。"),
                ("通知", "状态变化与提示。"),
            ],
            Self::Hud => [
                ("游戏内可见性", "仅在游戏窗口前台时显示。"),
                ("图表显示", "扭矩、功率和转速曲线。"),
                ("托盘菜单", "与托盘菜单中的开关同步。"),
            ],
            Self::ShiftLights => [
                ("样式与位置", "贴边位置与点亮方向。"),
                ("尺寸与布局", "宽度、厚度、间隔与偏移。"),
                ("显示效果", "闪烁与透明度。"),
            ],
            Self::GearDisplay => [
                ("显示位置", "按屏幕宽高的百分比定位。"),
                ("大小", "高度，单位 px。"),
                ("透明度", "不透明度百分比。"),
            ],
            Self::Telemetry => [
                ("监听地址", "本机 UDP 地址与端口。"),
                ("数据包转发", "原样转发收到的遥测包。"),
                ("转发目标", "目标地址与端口。"),
            ],
            Self::Calibration => [
                ("校准时长", "断油检测需保持的最短时间。"),
                ("记住已校准车辆", "按车辆保存断油转速，关闭后记录仍保留。"),
                ("校准提示", "画面中央的校准进度文字。"),
            ],
            Self::About => [
                ("Horizon HUD", "版本与许可证。"),
                ("帮助与反馈", "使用说明与问题反馈。"),
                ("诊断信息", "运行日志与排查信息。"),
            ],
        }
    }
}
