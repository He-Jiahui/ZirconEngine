use gilrs::EventType;
use winit::event_loop::ActiveEventLoop;

use super::super::RuntimeEntryApp;
use super::events::{gamepad_id, send_axis, send_button, send_connection};

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

        let session = &self.session;
        let viewport = self.viewport;
        let Some(gamepads) = self.gamepads.as_mut() else {
            return;
        };
        while let Some(event) = gamepads.next_event() {
            gamepads.update(&event);
            let result = match event.event {
                EventType::Connected => {
                    let pad = gamepads.gamepad(event.id);
                    send_connection(
                        session,
                        viewport,
                        gamepad_id(event.id),
                        true,
                        pad.name(),
                        pad.vendor_id(),
                        pad.product_id(),
                    )
                }
                EventType::Disconnected => send_connection(
                    session,
                    viewport,
                    gamepad_id(event.id),
                    false,
                    "",
                    None,
                    None,
                ),
                EventType::ButtonPressed(button, _) | EventType::ButtonRepeated(button, _) => {
                    send_button(session, viewport, gamepad_id(event.id), button, 1.0, true)
                }
                EventType::ButtonReleased(button, _) => {
                    send_button(session, viewport, gamepad_id(event.id), button, 0.0, false)
                }
                EventType::ButtonChanged(button, value, _) => send_button(
                    session,
                    viewport,
                    gamepad_id(event.id),
                    button,
                    value,
                    value >= 0.5,
                ),
                EventType::AxisChanged(axis, value, _) => {
                    send_axis(session, viewport, gamepad_id(event.id), axis, value)
                }
                EventType::Dropped | EventType::ForceFeedbackEffectCompleted => Ok(()),
                _ => Ok(()),
            };
            if result.is_err() {
                event_loop.exit();
                return;
            }
        }
        gamepads.inc();
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
