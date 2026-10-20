use std::time::{SystemTime, UNIX_EPOCH};

use gpui_kit::{Context, Render, Window, div, hsla, prelude::*, rgb};

use crate::components::{charts, gear_display, shift_lights};
use crate::core::calibration::{self, CalibrationParams, TelemetryEngine};
use crate::core::general_config::{SAMPLE_HZ, calibrate_frames};
use crate::core::props::CalibrationState;
use crate::core::telemetry;
use crate::session::{self, SessionMemory};

pub(crate) struct HudWindow {
    engine: TelemetryEngine,
}

impl HudWindow {
    pub(crate) fn new(cx: &mut Context<Self>) -> Self {
        start_sample_pump(cx);
        Self {
            engine: TelemetryEngine::new(),
        }
    }
}

fn start_sample_pump(cx: &mut Context<HudWindow>) {
    let queue = telemetry::spawn_udp_listener();
    cx.spawn(async move |this, cx| {
        let mut last_generation = session::generation();
        let mut last_listen_generation = telemetry::listen_generation();
        let mut last_remember = session::general().remember_calibrated_cars;
        loop {
            let samples = {
                let mut queue = queue.lock().unwrap();
                std::mem::take(&mut *queue)
            };
            let reset = calibration::take_reset_request();
            let remember = session::general().remember_calibrated_cars;
            let generation = session::generation();
            let listen_generation = telemetry::listen_generation();
            let remember_just_enabled = remember && !last_remember;
            if !samples.is_empty()
                || reset
                || generation != last_generation
                || listen_generation != last_listen_generation
                || remember != last_remember
            {
                last_generation = generation;
                last_listen_generation = listen_generation;
                last_remember = remember;
                this.update(cx, |this, cx| {
                    if reset {
                        this.engine.clear_runtime_calibration();
                    }
                    if remember_just_enabled {
                        this.engine.restore_if_remembered(&SessionMemory);
                    }
                    let general = session::general();
                    let params = CalibrationParams {
                        strict: general.strict_calibrate_conditions,
                        calibrate_frames: calibrate_frames(general.calibrate_ms),
                    };
                    let memory = SessionMemory;
                    for sample in samples {
                        this.engine.apply_sample(sample, &params, &memory);
                    }
                    cx.notify();
                })
                .ok();
            }
            cx.background_executor()
                .timer(std::time::Duration::from_secs_f64(1.0 / SAMPLE_HZ as f64))
                .await;
        }
    })
    .detach();
}

impl Render for HudWindow {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|elapsed| elapsed.as_millis())
            .unwrap_or(0);
        let general = session::general();
        let props = self.engine.props(
            now_ms,
            general.strict_calibrate_conditions,
            general.calibrate_hint_visible,
            telemetry::listen_addr_display(),
            calibrate_frames(general.calibrate_ms),
        );
        let calibrate_hint = if general.calibrate_hint_visible {
            if !props.has_data {
                Some(format!("等待数据  UDP {}", props.listen_addr))
            } else {
                match props.calibration {
                    CalibrationState::Uncalibrated {
                        elapsed_s,
                        duration_s,
                        ..
                    } if elapsed_s > 0.0 => {
                        Some(format!("校准中 {elapsed_s:.1}/{duration_s:.1}s"))
                    }
                    CalibrationState::Uncalibrated { .. } => Some(
                        if general.strict_calibrate_conditions {
                            "手刹+油门原地拉转校准"
                        } else {
                            "按住油门拉到断油转速校准"
                        }
                        .into(),
                    ),
                    CalibrationState::Ready { .. } => None,
                }
            }
        } else {
            None
        };

        div()
            .flex()
            .flex_col()
            .gap_3()
            .bg(hsla(0.0, 0.0, 0.0, 0.0))
            .size_full()
            // .p_8()
            .text_xl()
            .when(general.charts_visible, |el| {
                el.child(charts::render(&self.engine.charts()))
            })
            .child(shift_lights::render_hud(&props, &session::shift_lights()))
            .child(gear_display::render_hud(&props, &session::gear_display()))
            .when_some(calibrate_hint, |el, hint| {
                el.child(
                    div()
                        .absolute()
                        .inset_0()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_color(rgb(0xffffff))
                        .child(hint),
                )
            })
    }
}
