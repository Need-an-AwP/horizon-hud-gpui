要实现你想要的 **React 风格组件化架构**，核心思想只有一句话：

> **每个样式都是一个高内聚的独立模块（Co-located Component），内部同时包含自己的 `Config`（配置）、`View`（纯数据驱动的 HUD 渲染）和 `SettingsView`（设置表单）；底层引擎只负责把 UDP 原始数据加工成统一的 `HudTelemetryProps` 喂给当前激活的组件。**

下面为你设计一套直接对标 React 开发心智的 Rust + GPUI 代码结构。

---

### 一、React 概念与当前项目的映射关系

| React 概念 | 在本项目的对应设计 | 作用 |
| :--- | :--- | :--- |
| **`Props`（组件入参）** | `HudTelemetryProps` | 底层处理好后传给组件的只读数据包（已算好校准状态、归一化转速比、挡位枚举等），**组件完全不关心 UDP 怎么收、断油怎么检测** |
| **自定义 Hooks / Utils** | `components/shared/` | 抽离出的通用纯函数：如分段点亮计算 `use_stepped_lights()`、闪烁时钟 `use_blink_clock()`、屏幕贴边容器 `<EdgeAnchor>`、自由坐标容器 `<FloatingAnchor>` |
| **组件自身 State / Config** | 各样式文件内的 `pub struct Config` | 每个样式独有的配置项（带 `#[derive(Serialize, Deserialize)]`），互不干扰 |
| **组件本体 + 配置面板** | 同一个文件导出 `render_hud(&Props, &Config)` 与 `render_settings(...)` | 新增一个样式时，**80% 的代码都只写在这一个新建的文件里** |
| **Core / General 层** | `core/`（网络、校准状态机、系统窗口） | 最底层的通用能力，与任何具体 UI 样式彻底解耦 |

---

### 二、推荐的目录结构（按领域与组件就近组织）

```text
src/
├── main.rs
├── core/                         # 【底层 General 层】与具体样式完全无关
│   ├── mod.rs
│   ├── telemetry.rs              # UDP 监听、端口转发、原始数据包解析
│   ├── calibration.rs            # 断油转速检测状态机、已校准车辆存档管理
│   ├── general_config.rs         # General 设置 (监听/转发端口、校准参数、仅游戏内显示等)
│   └── props.rs                  # 生成给 UI 组件消费的标准化 HudTelemetryProps
│
├── components/                   # 【React 风格组件层】只受 Props + 自身 Config 驱动
│   ├── mod.rs
│   ├── shared/                   # 通用计算逻辑 (Hooks) 与布局容器 (Layout Wrappers)
│   │   ├── rpm_math.rs           # 转速区间映射、阈值判断、闪烁相位计算等纯函数
│   │   ├── layouts.rs            # 贴四边容器 (EdgeContainer)、自由 X/Y 容器 (FloatingContainer)
│   │   └── color.rs              # 透明度合成、颜色插值等
│   │
│   ├── shift_lights/             # 【转速灯组件库】
│   │   ├── mod.rs                # ShiftLightsStyle 枚举路由 + 公共开关/闪烁配置
│   │   ├── bar.rs                # 样式 1：基础拉力灯条 (含自己的 Config + HUD DOM + Settings DOM)
│   │   ├── porsche_cup.rs        # 样式 2：保时捷 992 对称圆灯 (含自己的 Config + HUD DOM + Settings DOM)
│   │   └── f1_led.rs             # (未来新增) 样式 3：只需新建此文件并注册到 mod.rs
│   │
│   ├── gear_display/             # 【挡位组件库】
│   │   ├── mod.rs                # GearDisplayStyle 枚举路由 + 公共开关/位置配置
│   │   ├── seven_segment.rs      # 样式 1：七段数码管 (含自己的 Config + HUD DOM + Settings DOM)
│   │   └── modern_card.rs        # (未来新增) 样式 2：现代字卡/序列档显示
│   │
│   └── charts/                   # 遥测折线图组件
│       └── mod.rs
│
└── app/                          # 【窗口宿主与设置框架】
    ├── hud_window.rs             # HUD 顶层画布：拿 Props -> 渲染 <ShiftLights /> 和 <GearDisplay />
    └── settings_window/          # 设置窗口外壳（侧边栏导航、通用表单控件库 Slider/Input/Switch）
```

---

### 三、核心接口设计（代码长什么样？）

#### 1. 底层输出的标准化 `Props`（`src/core/props.rs`）

底层把 UDP 包、车辆校准记录、校准进度全部消化完，每帧组装成一个极简、干净的 `HudTelemetryProps` 传给所有 UI 组件：

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GearValue {
    Disconnected, // "--"
    Reverse,      // "R"
    Neutral,      // "N"
    Forward(u8),  // 1..=10
}

#[derive(Clone, Copy, Debug)]
pub enum CalibrationState {
    /// 未校准（非电动车且无记录）：附带当前拉转校准进度 0.0..=1.0
    Uncalibrated { progress: f32 },
    /// 已就绪（已校准或电动车）：提供断油/红线转速，以及直接算好的归一化比例 (rpm / limit_rpm)
    Ready { limit_rpm: f32, rpm_ratio: f32 },
}

/// 传给所有 HUD 视觉组件的只读 Props（类似 React 组件的 props）
#[derive(Clone, Copy, Debug)]
pub struct HudTelemetryProps {
    pub has_data: bool,
    pub rpm: f32,
    pub max_rpm: f32,
    pub calibration: CalibrationState,
    pub gear: GearValue,
    pub power_kw: f32,
    pub torque_nm: f32,
    pub now_ms: u128, // 统一帧时间戳，供任何需要闪烁/动画的组件使用
}
```
> **好处**：写任何新的转速灯或挡位样式时，你拿到的是直接能用的 `props.calibration` 和 `props.gear`，不需要再去碰 `num_cylinders`、`fuel_cut` 检测算法或 UDP 偏移量。

---

#### 2. 抽离通用计算逻辑与布局组件（`src/components/shared/`）

像 React 的自定义 Hooks 和 Layout 组件一样，把複数样式会共用的逻辑抽成纯函数，**样式自己决定要不要用**：

```rust
// src/components/shared/rpm_math.rs

/// 通用计算 1：线性分段点亮逻辑（返回：点亮几格, 是否触发超转闪烁）
/// Bar（5格）和 PorscheCup（单侧5颗）都可以直接调它；
/// 如果新样式是非线性点亮（比如 70% 才亮第一颗），可以调 use_ranged_lights(ratio, start_frac, blink_frac, count)
pub fn calc_linear_lights(rpm_ratio: f32, blink_frac: f32, total_steps: usize) -> (usize, bool) {
    if rpm_ratio > blink_frac {
        (total_steps, true)
    } else {
        let on = ((rpm_ratio / blink_frac) * total_steps as f32)
            .floor()
            .clamp(0.0, total_steps as f32) as usize;
        (on, false)
    }
}

/// 通用计算 2：根据当前帧时间戳和周期，计算当前是否处于闪烁的“亮起”半周期
pub fn is_blink_lit(now_ms: u128, blink_ms: usize) -> bool {
    (now_ms / (blink_ms.max(1) as u128)) % 2 == 0
}
```

```rust
// src/components/shared/layouts.rs

/// 通用布局 1：贴屏幕四边的容器（供条形灯、圆点灯使用）
pub fn edge_strip_container(
    position: EdgePosition,
    width_percent: usize,
    thickness_px: usize,
    offset_px: usize,
    child: impl IntoElement,
) -> Div { ... }

/// 通用布局 2：按屏幕 X/Y 百分比自由定位的容器（供挡位表、或未来的居中弧形转速表使用）
pub fn floating_xy_container(
    x_ratio: f32,
    y_ratio: f32,
    child: impl IntoElement,
) -> Div { ... }
```

---

#### 3. 单个样式的完整写法（以 `porsche_cup.rs` 为例）

在新的结构下，**一个样式就是一个独立文件**，里面内聚了它的 **Config（含默认值）**、**HUD 渲染函数**、**Settings 渲染函数**：

```rust
// src/components/shift_lights/porsche_cup.rs

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct PorscheCupConfig {
    pub position: EdgePosition,
    pub width_percent: usize,
    pub diameter_px: usize,      // 名字就是直观的 diameter_px，不用再借用 thickness_px！
    pub gap_px: usize,
    pub offset_px: usize,
    pub lit_opacity: f32,
    pub dim_opacity: f32,
    pub blink_percent: f32,
    pub blink_ms: usize,
}

impl Default for PorscheCupConfig { ... }

/// 1. 纯数据驱动的 HUD 视图（等价于 React 的 <PorscheCupLights {...props} config={config} />）
pub fn render_hud(props: &HudTelemetryProps, cfg: &PorscheCupConfig) -> impl IntoElement {
    let blink_frac = cfg.blink_percent / 100.0;
    let strip = match props.calibration {
        CalibrationState::Ready { rpm_ratio, .. } => {
            let (on_count, flash) = calc_linear_lights(rpm_ratio, blink_frac, 5);
            let lit = !flash || is_blink_lit(props.now_ms, cfg.blink_ms);
            // 渲染 10 颗对称圆灯 DOM ...
        }
        CalibrationState::Uncalibrated { progress } => {
            // 渲染未校准进度 DOM ...
        }
    };

    edge_strip_container(cfg.position, cfg.width_percent, cfg.diameter_px, cfg.offset_px, strip)
}

/// 2. 该样式专属的设置面板（等价于 React 的 <PorscheCupSettings config={config} onChange={...} />）
pub fn render_settings(view: &Settings, colors: &Colors, cx: &mut Context<Settings>) -> impl IntoElement {
    // 直接渲染属于保时捷圆灯的设置项：位置、灯珠直径、灯珠间距、闪烁阈值...
    // 不再需要任何 `if circles { ... } else { ... }`！
}
```

---

#### 4. 顶层路由分发（`shift_lights/mod.rs` & `gear_display/mod.rs`）

父组件只做一件事：根据当前选中的枚举，把 `props` 和对应的子 `config` 转发给子组件：

```rust
// src/components/shift_lights/mod.rs

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ShiftLightsSettings {
    pub enabled: bool,
    pub style: ShiftLightsStyle,
    pub bar: bar::BarConfig,
    pub porsche_cup: porsche_cup::PorscheCupConfig,
}

pub fn render_hud(props: &HudTelemetryProps, settings: &ShiftLightsSettings) -> impl IntoElement {
    match settings.style {
        ShiftLightsStyle::Bar => bar::render_hud(props, &settings.bar).into_any_element(),
        ShiftLightsStyle::PorscheCup => porsche_cup::render_hud(props, &settings.porsche_cup).into_any_element(),
    }
}

pub fn render_settings_page(view: &Settings, colors: &Colors, cx: &mut Context<Settings>) -> impl IntoElement {
    let style = current_style();
    div()
        .child(view.style_selector_row(colors, cx)) // 顶部：切换样式下拉框
        .child(match style {                        // 下方：直接挂载当前样式自己的设置组件
            ShiftLightsStyle::Bar => bar::render_settings(view, colors, cx).into_any_element(),
            ShiftLightsStyle::PorscheCup => porsche_cup::render_settings(view, colors, cx).into_any_element(),
        })
}
```

---

### 四、这套结构下，未来「新增一个样式」只需要哪几步？

假设你要新增一个 **F1 15颗 LED + 双侧爆闪** 的转速灯样式，或者一个 **带转速变色背景框的现代数字挡位表**：

1. **不动 `core/`**：UDP、校准逻辑、配置文件读写、General 设置完全不用碰。
2. **不动其它样式文件**：`bar.rs`、`porsche_cup.rs`、`seven_segment.rs` 完全不用碰。
3. **只需新建 `src/components/shift_lights/f1_led.rs`**：
   - 定义 `pub struct F1LedConfig`（比如有 `start_rpm_percent: f32`、`shift_rpm_percent: f32`、`led_size: usize`）；
   - 写 `pub fn render_hud(props: &HudTelemetryProps, cfg: &F1LedConfig)`，从 `shared` 里按需挑通用计算函数；
   - 写 `pub fn render_settings(...)` 声明它自己的设置项。
4. **在 `shift_lights/mod.rs` 的枚举里加一个分支**，完成接入。

---

### 五、设置页控件（Slider / Input）如何配合动态样式？

在 GPUI 中，`SliderState` 和 `InputState` 需要 `Entity` 句柄。为了让每个样式能自由定义自己的设置项，而不必在顶层 `Settings` 结构体里手写几十个字段，有两种乾淨的做法：

- **方案 A（按样式拆分子 State Struct，推荐、最稳妥）**：
  每个样式模块除了导出 `Config`，还导出一个 `pub struct SettingsControls`（内部持有该样式用到的 `Entity<SliderState>` / `Entity<InputState>`），在 `Settings` 窗口初始化时统一创建。这样每个样式的控件也是完全隔离在各自文件里的。
- **方案 B（通用受控组件池 / Keyed Store）**：
  在 `Settings` 里维护一个 `HashMap<&'static str, Entity<SliderState>>` 和 `HashMap<&'static str, Entity<InputState>>`，提供类似 React Hook 的辅助方法：
  `view.use_slider("f1.start_rpm", cfg.start_rpm, 50.0..=95.0, cx, |val| ...)`
  这样在任何样式的 `render_settings` 里都可以直接声明式地写滑块和输入框，**连控件字段都不用提前定义**。

---

如果你觉得这套 **`core`（产 Props） + `components/shared`（通用 Hooks/容器） + `components/{shift_lights, gear_display}/<style>.rs`（单文件高内聚组件）** 的结构符合预期，我们可以直接按这个蓝图开始重构。你更倾向于设置页控件采用 **方案 A（每个样式自带 Controls 结构体）** 还是 **方案 B（按 Key 自动按需创建的声明式控件池）**？