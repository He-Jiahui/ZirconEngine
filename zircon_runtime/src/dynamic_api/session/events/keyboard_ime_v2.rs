//! The V2 branch inserted into `keyboard_ime.rs` by the private integration patch.

use zircon_runtime_interface::ui::dispatch::UiInputEvent;
use zircon_runtime_interface::{ZrRuntimeEventV1, ZrStatus};

use super::super::ime_composition_route::RuntimeImeCompositionRouteError;
use super::super::status::{capability_denied, invalid_argument};
use super::super::RuntimeDynamicSession;

impl RuntimeDynamicSession {
    /// Decode, generation-check, and route a negotiated composition event. The route returns
    /// both UI and core events only after all validation succeeds, so malformed/stale callbacks
    /// cannot partially mutate either queue.
    pub(super) fn handle_ime_composition_v2(&mut self, event: ZrRuntimeEventV1) -> ZrStatus {
        // Route decoding and generation fencing run before allocating a UI metadata sequence.
        // Malformed or stale callbacks therefore leave retained UI sequencing untouched.
        let mut dispatch = match self.ime_composition_route.consume_event(
            event,
            zircon_runtime_interface::ui::dispatch::UiInputEventMetadata::default(),
        ) {
            Ok(dispatch) => dispatch,
            Err(RuntimeImeCompositionRouteError::CapabilityUnavailable) => {
                return capability_denied(
                    b"runtime IME composition V2 capability was not negotiated",
                );
            }
            Err(RuntimeImeCompositionRouteError::Malformed(_)) => {
                return invalid_argument(b"invalid runtime IME composition V2 payload");
            }
            Err(RuntimeImeCompositionRouteError::WrongEventState) => {
                return invalid_argument(
                    b"runtime IME composition event used an unsupported state",
                );
            }
            Err(RuntimeImeCompositionRouteError::StaleGeneration) => {
                // A stale callback is a harmless no-op. It must not fall back to V1 because that
                // would reintroduce the very late-callback mutation the generation gate fences.
                return ZrStatus::ok();
            }
        };
        if let UiInputEvent::Ime(ime) = &mut dispatch.ui_event {
            ime.metadata = self.runtime_ui.next_input_metadata();
        }
        match self
            .runtime_ui
            .dispatch_input(self.camera_controller.viewport_size(), dispatch.ui_event)
        {
            Ok(true) => ZrStatus::ok(),
            Ok(false) => {
                self.submit_input_event(dispatch.core_event);
                ZrStatus::ok()
            }
            Err(error) => super::super::status::error_status(format!(
                "dispatch declared runtime UI IME composition input: {error}"
            )),
        }
    }
}
