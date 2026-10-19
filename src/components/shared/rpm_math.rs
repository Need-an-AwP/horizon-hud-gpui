/// 线性分段点亮：返回（点亮格数，是否进入超转闪烁）。
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

/// 当前帧是否处于闪烁的亮起半周期。
pub fn is_blink_lit(now_ms: u128, blink_ms: usize) -> bool {
    (now_ms / blink_ms.max(1) as u128) % 2 == 0
}

/// 校准进度在第 `index` 段上的填充比例，0 到 1。
pub fn calibrate_segment_fill(progress: f32, index: usize, count: usize) -> f32 {
    let n = count as f32;
    let start = index as f32 / n;
    let end = (index + 1) as f32 / n;
    ((progress - start) / (end - start).max(f32::EPSILON)).clamp(0.0, 1.0)
}
