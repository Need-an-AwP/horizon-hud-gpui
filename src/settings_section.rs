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
            Self::Overview => "管理 Horizon HUD 的常用选项与运行状态。",
            Self::Hud => "配置 HUD 的显示条件与图表可见性。",
            Self::ShiftLights => "按位置、尺寸和显示效果配置转速指示灯。",
            Self::GearDisplay => "调整七段数码管挡位显示的位置、大小和透明度。",
            Self::Telemetry => "配置 UDP 遥测监听地址和端口。",
            Self::Calibration => "配置转速指示灯校准时长、条件与提示样式。",
            Self::About => "查看应用信息、版本与支持资源。",
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
                ("HUD 状态", "在这里查看覆盖层与游戏连接状态。"),
                ("快速操作", "常用显示和校准操作将集中于此。"),
                ("通知", "重要状态变化和提示将在这里出现。"),
            ],
            Self::Hud => [
                ("游戏内可见性", "设置 HUD 仅在游戏窗口前台时显示。"),
                ("图表显示", "控制 HUD 中扭矩、功率和转速历史曲线。"),
                ("托盘菜单", "以上选项也可通过托盘右键菜单快速调整。"),
            ],
            Self::ShiftLights => [
                ("位置与方向", "选择转速灯在屏幕边缘的位置和点亮方向。"),
                ("尺寸与布局", "设置整体宽度、灯条厚度、灯格间隔和边缘偏移。"),
                ("显示效果", "设置闪烁阈值，以及亮起与熄灭状态的透明度。"),
            ],
            Self::GearDisplay => [
                (
                    "显示位置",
                    "使用 X, Y 坐标将挡位显示放置在屏幕上的指定位置。",
                ),
                ("大小", "调整七段数码管的高度。"),
                ("透明度", "调整挡位显示整体的不透明程度。"),
            ],
            Self::Telemetry => [
                ("数据源", "选择遥测协议和连接方式。"),
                ("更新频率", "配置数据采样与界面刷新策略。"),
                ("校准", "管理转速、扭矩及功率的校准流程。"),
            ],
            Self::Calibration => [
                ("校准时长", "燃油切断检测需要保持的最短时间。"),
                (
                    "记住已校准车辆",
                    "将断油转速按车辆写入配置，可随时关闭而不删除已有记录。",
                ),
                ("校准提示样式", "自定义校准过程中的提示外观。"),
            ],
            Self::About => [
                ("Horizon HUD", "应用版本、更新记录与许可证信息。"),
                ("帮助与反馈", "常见问题、使用说明和问题反馈入口。"),
                ("诊断信息", "导出运行日志与故障排查所需信息。"),
            ],
        }
    }
}
