use gilrs::EventType;
use winit::event_loop::ActiveEventLoop;

use super::super::RuntimeEntryApp;
use super::events::{gamepad_id, send_axis, send_button, send_connection, send_raw_button};

impl RuntimeEntryApp {
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
        let mut should_exit = false;
        let session = &self.session;
        let viewport = self.viewport;
        {
            let Some(gamepads) = self.gamepads.as_mut() else {
                return;
            };
            while let Some(event) = gamepads.next_event() {
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
                if result.is_err() {
                    should_exit = true;
                    break;
                }
            }
            if !should_exit {
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
        if should_exit {
            event_loop.exit();
        }
    }

    fn announce_connected_gamepads(&mut self, event_loop: &dyn ActiveEventLoop) -> bool {
        let session = &self.session;
        let viewport = self.viewport;
        let Some(gamepads) = self.gamepads.as_mut() else {
            return true;
        };
        for (id, pad) in gamepads.gamepads() {
            if send_connection(
                session,
                viewport,
                gamepad_id(id),
                true,
                pad.name(),
                pad.vendor_id(),
                pad.product_id(),
            )
            .is_err()
            {
                event_loop.exit();
                return false;
            }
        }
        true
    }
}
