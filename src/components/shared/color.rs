pub fn with_opacity(color: u32, opacity: f32) -> gpui_kit::Rgba {
    let alpha = (opacity.clamp(0.0, 1.0) * 255.0).round() as u32;
    gpui_kit::rgba((color << 8) | alpha)
}
