use gpui_kit::colors::Colors;
use gpui_kit::component::{
    Sizable, Size,
    input::{Input, InputEvent, InputState},
    slider::{Slider, SliderEvent, SliderScale, SliderState, SliderValue},
};
use gpui_kit::{
    App, Context, Entity, EventEmitter, FocusHandle, IntoElement, MouseButton, RenderOnce, Rgba,
    TextAlign, Window, div, prelude::*, px,
};

const FIELD_WIDTH: f32 = 232.;
const INPUT_WIDTH: f32 = 64.;

/// 数值变化。拖动滑条时连续发出；输入框在回车或失焦并成功采纳后发出。
pub enum NumberFieldEvent {
    Change(f32),
}

/// 滑条与输入框共用的数值状态。
pub struct NumberFieldState {
    slider: Entity<SliderState>,
    input: Entity<InputState>,
    /// 滑条等非输入区域共用；`track_focus` 后点击会把焦点从 Input 移开并触发 blur。
    chrome_focus: FocusHandle,
    unit: &'static str,
    decimals: usize,
    syncing: bool,
}

impl EventEmitter<NumberFieldEvent> for NumberFieldState {}

impl NumberFieldState {
    pub fn builder() -> NumberFieldBuilder {
        NumberFieldBuilder::default()
    }

    /// 外部同步或重置。同时写入滑条和输入框，不发出 [`NumberFieldEvent`]。
    pub fn set_value(&mut self, value: f32, window: &mut Window, cx: &mut Context<Self>) {
        self.syncing = true;
        self.slider.update(cx, |slider, cx| {
            slider.set_value(value, window, cx);
        });
        self.write_input(value, window, cx);
        self.syncing = false;
    }

    pub fn value(&self, cx: &App) -> f32 {
        match self.slider.read(cx).value() {
            SliderValue::Single(value) => value,
            SliderValue::Range(start, _) => start,
        }
    }

    fn mirror_slider(&mut self, value: f32, window: &mut Window, cx: &mut Context<Self>) {
        if self.syncing {
            return;
        }
        self.syncing = true;
        self.write_input(value, window, cx);
        self.syncing = false;
        cx.emit(NumberFieldEvent::Change(value));
    }

    fn commit_input(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.syncing {
            return;
        }
        let current = self.value(cx);
        let text = self.input.read(cx).value();
        let next = match text.trim().parse::<f32>() {
            Ok(value) if value.is_finite() => self.normalize(value, cx),
            _ => current,
        };
        self.syncing = true;
        self.slider.update(cx, |slider, cx| {
            slider.set_value(next, window, cx);
        });
        self.write_input(next, window, cx);
        self.syncing = false;
        if !nearly_eq(current, next) {
            cx.emit(NumberFieldEvent::Change(next));
        }
    }

    fn write_input(&mut self, value: f32, window: &mut Window, cx: &mut Context<Self>) {
        let text = format_value(value, self.decimals);
        if self.input.read(cx).value().as_ref() == text {
            return;
        }
        self.input.update(cx, |input, cx| {
            input.set_value(text, window, cx);
        });
    }

    fn normalize(&self, value: f32, cx: &App) -> f32 {
        let slider = self.slider.read(cx);
        let mut min = slider.min_value();
        let mut max = slider.max_value();
        if min > max {
            std::mem::swap(&mut min, &mut max);
        }
        let step = slider.step_value();
        let clamped = value.clamp(min, max);
        let snapped = if step.is_finite() && step > 0.0 {
            ((clamped / step).round() * step).clamp(min, max)
        } else {
            clamped
        };
        if snapped == 0.0 { 0.0 } else { snapped }
    }
}

/// [`NumberFieldState`] 的构造参数。滑条的上下界和步进只能在创建时写入。
pub struct NumberFieldBuilder {
    min: f32,
    max: f32,
    step: f32,
    default_value: f32,
    unit: &'static str,
    decimals: usize,
}

impl Default for NumberFieldBuilder {
    fn default() -> Self {
        Self {
            min: 0.0,
            max: 100.0,
            step: 1.0,
            default_value: 0.0,
            unit: "",
            decimals: 0,
        }
    }
}

impl NumberFieldBuilder {
    /// 最小值，默认 0。对数刻度时必须大于 0。
    pub fn min(mut self, min: f32) -> Self {
        self.min = min;
        self
    }

    /// 最大值，默认 100。
    pub fn max(mut self, max: f32) -> Self {
        self.max = max;
        self
    }

    /// 步进，默认 1。滑条拖动和输入提交都按这个间隔对齐。
    pub fn step(mut self, step: f32) -> Self {
        self.step = step;
        self
    }

    /// 初始值，默认 0。超出范围时夹到上下界，不按步进对齐。
    pub fn default_value(mut self, value: f32) -> Self {
        self.default_value = value;
        self
    }

    /// 显示在输入框右侧的单位，不参与解析。默认空，不占位。
    pub fn unit(mut self, unit: &'static str) -> Self {
        self.unit = unit;
        self
    }

    /// 输入框显示的小数位数，默认 0。
    pub fn decimals(mut self, decimals: usize) -> Self {
        self.decimals = decimals;
        self
    }

    pub fn build(self, window: &mut Window, cx: &mut Context<NumberFieldState>) -> NumberFieldState {
        let value = if self.default_value.is_finite() {
            self.default_value.clamp(self.min, self.max)
        } else {
            self.min
        };
        let text = format_value(value, self.decimals);
        let slider = cx.new(|_| {
            // 必须先 max 再 min：`SliderState::new()` 默认 max=100，若先 min(>100) 会在
            // `update_thumb_pos` 里对 value 做 clamp(min, max) 并 panic。
            SliderState::new()
                .max(self.max)
                .min(self.min)
                .step(self.step)
                .scale(SliderScale::Linear)
                .default_value(value)
        });
        let input = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value(text)
                .min(self.min as f64)
                .max(self.max as f64)
                .step(self.step as f64)
        });

        cx.subscribe_in(
            &slider,
            window,
            |this, _, event: &SliderEvent, window, cx| {
                if let SliderEvent::Change(SliderValue::Single(value)) = event {
                    this.mirror_slider(*value, window, cx);
                }
            },
        )
        .detach();
        cx.subscribe_in(
            &input,
            window,
            |this, _, event: &InputEvent, window, cx| match event {
                InputEvent::PressEnter { .. } | InputEvent::Blur => this.commit_input(window, cx),
                InputEvent::Change | InputEvent::Focus => {}
            },
        )
        .detach();

        let chrome_focus = cx.focus_handle().tab_stop(false);

        NumberFieldState {
            slider,
            input,
            chrome_focus,
            unit: self.unit,
            decimals: self.decimals,
            syncing: false,
        }
    }
}

/// 固定样式的滑条加数值输入框。
#[derive(IntoElement)]
pub struct NumberField {
    state: Entity<NumberFieldState>,
    unit_color: Option<Rgba>,
}

impl NumberField {
    pub fn new(state: &Entity<NumberFieldState>) -> Self {
        Self {
            state: state.clone(),
            unit_color: None,
        }
    }

    /// 单位文字颜色。未设置时跟随窗口外观的禁用色。
    pub fn unit_color(mut self, color: Rgba) -> Self {
        self.unit_color = Some(color);
        self
    }
}

impl RenderOnce for NumberField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (slider, input, unit, chrome_focus) = {
            let state = self.state.read(cx);
            (
                state.slider.clone(),
                state.input.clone(),
                state.unit,
                state.chrome_focus.clone(),
            )
        };
        let unit_color = self
            .unit_color
            .unwrap_or_else(|| Colors::for_appearance(window).disabled);

        div()
            .flex()
            .items_center()
            .gap_3()
            .w(px(FIELD_WIDTH))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .track_focus(&chrome_focus)
                    .child(Slider::new(&slider).horizontal()),
            )
            .child(
                div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .gap_2()
                    .child(
                        Input::new(&input)
                            .with_size(Size::Small)
                            .w(px(INPUT_WIDTH))
                            .flex_none()
                            .text_align(TextAlign::Center),
                    )
                    .when(!unit.is_empty(), |row| {
                        row.child(
                            div()
                                .flex_none()
                                .text_xs()
                                .text_color(unit_color)
                                .on_mouse_down(MouseButton::Left, {
                                    let chrome_focus = chrome_focus.clone();
                                    move |_, window, cx| chrome_focus.focus(window, cx)
                                })
                                .child(unit),
                        )
                    }),
            )
    }
}

fn format_value(value: f32, decimals: usize) -> String {
    format!("{value:.decimals$}")
}

fn nearly_eq(a: f32, b: f32) -> bool {
    (a - b).abs() <= 1e-4
}

/// 将 0–1 不透明度转为设置 UI 中的 0–100 百分比。
pub fn opacity_as_percent(opacity: f32) -> f32 {
    (opacity.clamp(0.0, 1.0) * 100.0).round()
}

/// 将设置 UI 中的百分比转为 0–1 不透明度。
pub fn opacity_from_percent(percent: f32) -> f32 {
    (percent.clamp(0.0, 100.0) / 100.0).clamp(0.0, 1.0)
}
