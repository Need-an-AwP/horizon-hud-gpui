use gpui_kit::{div, prelude::*, px, relative, IntoElement};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgePosition {
    Bottom,
    Right,
    Top,
    Left,
}

impl EdgePosition {
    pub fn is_vertical(self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }
}

/// 复刻转速灯外层容器：absolute；横向铺满并居中，子元素宽度为 `span_percent`；
/// 纵向铺满并居中，子元素高度为 `span_percent`、宽度为 `thickness_px`；按边偏移 `offset_px`。
pub fn edge_strip_container(
    position: EdgePosition,
    span_percent: usize,
    thickness_px: usize,
    offset_px: usize,
    child: impl IntoElement,
) -> gpui_kit::Div {
    let vertical = position.is_vertical();
    let offset = px(offset_px as f32);
    let span = relative(span_percent as f32 / 100.0);
    let thickness = px(thickness_px as f32);
    div()
        .absolute()
        .flex()
        .when(!vertical, |el| el.left_0().right_0().justify_center())
        .when(vertical, |el| {
            el.top_0()
                .bottom_0()
                .flex_col()
                .items_center()
                .justify_center()
        })
        .when(position == EdgePosition::Top, |el| el.top(offset))
        .when(position == EdgePosition::Bottom, |el| el.bottom(offset))
        .when(position == EdgePosition::Right, |el| el.right(offset))
        .when(position == EdgePosition::Left, |el| el.left(offset))
        .child(
            div()
                .when(!vertical, |el| el.w(span))
                .when(vertical, |el| el.h(span).w(thickness))
                .child(child),
        )
}

/// 子元素左上角放在屏幕比例处。这里不做水平居中。
#[allow(dead_code)]
pub fn floating_xy_container(
    x_ratio: f32,
    y_ratio: f32,
    child: impl IntoElement,
) -> gpui_kit::Div {
    div()
        .absolute()
        .left(relative(x_ratio))
        .top(relative(y_ratio))
        .child(child)
}
