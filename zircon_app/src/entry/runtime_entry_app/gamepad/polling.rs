//! 获准执行 Runtime 帧时，排空一批有界手柄事件并维护连接与效果状态。
//! 预算耗尽保留后续帧需求，分发失败通过产品诊断退出。

use gilrs::EventType;
use winit::event_loop::ActiveEventLoop;

mod drain_budget;

use self::drain_budget::GamepadDrainBudget;
use super::super::RuntimeEntryApp;
use super::events::{gamepad_id, send_axis, send_button, send_connection, send_raw_button};

impl RuntimeEntryApp {
    // TODO: [CR-APP-ENTRY-0013] 核对 DesktopApp 空闲时手柄队列的独立唤醒来源：Reactive 可进入无限 Wait，而本方法仅在获准 pump 时运行；需验证仅手柄输入能否恢复处理。
    /// 由获准的 frame pump 调用；首轮公布连接清单，后续批次遵守事件与时间预算。
    pub(in crate::entry::runtime_entry_app) fn poll_gamepads(
        &mut self,
        event_loop: &dyn ActiveEventLoop,
    ) {
        if self.gamepads.is_none() {
            return;
        }
        if !self.gamepad_connections_announced {
            self.gamepad_connections_announced = true;
            if !self.announce_connected_gamepads(event_loop) {
                return;
            }
        }

        let mut disconnected_gamepads = Vec::new();
        let mut dispatch_error = None;
        let mut drain_budget_exhausted = false;
        let mut drain_budget = GamepadDrainBudget::begin(std::time::Instant::now());
        let session = &self.session;
        let viewport = self.viewport;
        {
            let Some(gamepads) = self.gamepads.as_mut() else {
                return;
            };
            loop {
                if drain_budget.needs_continuation(std::time::Instant::now()) {
                    drain_budget_exhausted = true;
                    break;
                }
                let Some(event) = gamepads.next_event() else {
                    break;
                };
                drain_budget.record_event();
                gamepads.update(&event);
                let runtime_gamepad_id = gamepad_id(event.id);
                let result = match event.event {
                    EventType::Connected => {
                        let pad = gamepads.gamepad(event.id);
                        send_connection(
                            session,
                            viewport,
                            runtime_gamepad_id,
                            true,
                            pad.name(),
                            pad.vendor_id(),
                            pad.product_id(),
                        )
                    }
                    EventType::Disconnected => {
                        disconnected_gamepads.push(runtime_gamepad_id);
                        send_connection(
                            session,
                            viewport,
                            runtime_gamepad_id,
                            false,
                            "",
                            None,
                            None,
                        )
                    }
                    EventType::ButtonPressed(button, _) | EventType::ButtonRepeated(button, _) => {
                        send_button(session, viewport, runtime_gamepad_id, button, 1.0, true)
                    }
                    EventType::ButtonReleased(button, _) => {
                        send_button(session, viewport, runtime_gamepad_id, button, 0.0, false)
                    }
                    EventType::ButtonChanged(button, value, _) => {
                        send_raw_button(session, viewport, runtime_gamepad_id, button, value)
                    }
                    EventType::AxisChanged(axis, value, _) => {
                        send_axis(session, viewport, runtime_gamepad_id, axis, value)
                    }
                    EventType::Dropped | EventType::ForceFeedbackEffectCompleted => Ok(()),
                    _ => Ok(()),
                };
                if let Err(error) = result {
                    dispatch_error = Some(error);
                    break;
                }
            }
            if dispatch_error.is_none() {
                gamepads.inc();
            }
        }
        for gamepad_id in disconnected_gamepads {
            super::rumble::clear_gamepad_rumble_effects_for_gamepad(
                &mut self.gamepad_rumble_effects,
                gamepad_id,
            );
        }
        super::rumble::clear_finished_rumble_effects(self.gamepad_rumble_effects.as_mut());
        if let Some(error) = dispatch_error {
            self.report_fatal_failure(
                "runtime_event_dispatch",
                "gamepad_event_stream",
                format!("runtime gamepad event dispatch failed: {error}"),
                "verify the runtime library ABI and gamepad event handler, then restart zircon_runtime",
            );
            event_loop.exit();
        } else if drain_budget_exhausted {
            self.request_runtime_frame();
        }
    }

    /// 第一次轮询前补发已有连接；失败沿用普通事件流的产品终止策略。
    fn announce_connected_gamepads(&mut self, event_loop: &dyn ActiveEventLoop) -> bool {
        let session = &self.session;
        let viewport = self.viewport;
        let Some(gamepads) = self.gamepads.as_mut() else {
            return true;
        };
        let mut dispatch_error = None;
        for (id, pad) in gamepads.gamepads() {
            if let Err(error) = send_connection(
                session,
                viewport,
                gamepad_id(id),
                true,
                pad.name(),
                pad.vendor_id(),
                pad.product_id(),
            ) {
                dispatch_error = Some(error);
                break;
            }
        }
        if let Some(error) = dispatch_error {
            self.report_fatal_failure(
                "runtime_event_dispatch",
                "connected_gamepad_inventory",
                format!("runtime gamepad connection dispatch failed: {error}"),
                "verify the runtime library ABI and gamepad event handler, then restart zircon_runtime",
            );
            event_loop.exit();
            false
        } else {
            true
        }
    }
}
