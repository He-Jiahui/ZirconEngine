//! V2 IME payload route between the dynamic event API and the retained UI dispatcher.
//!
//! All payload decoding and generation checks happen before the caller submits a core input
//! event or calls `RuntimeUiSurfaceSet::dispatch_input`.  The route returns both views of one
//! validated event so the actual `keyboard_ime` handler can preserve the existing UI-first
//! dispatch order without losing clause metadata.

use zircon_runtime_interface::ui::dispatch::{
    UiImeInputEvent, UiImeInputEventKind, UiInputEvent, UiInputEventMetadata,
};
use zircon_runtime_interface::{
    ZrRuntimeEventV1, ZrRuntimeImeCandidateRectV2, ZrRuntimeImeCompositionNegotiation,
    ZrRuntimeImeCompositionOperationV2, ZrRuntimeImeCompositionV2, ZrRuntimeImeCompositionV2Error,
    ZR_RUNTIME_EVENT_KIND_IME_V1, ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1,
    ZR_RUNTIME_IME_COMPOSITION_V2_EVENT_STATE,
};

use crate::core::framework::input::{ImeEvent, ImePreedit, InputEvent};

use super::events::ime_preedit_adapter::decoded_preedit_to_ui_event;
use super::ime_composition_generation::RuntimeImeCompositionGenerationGate;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum RuntimeImeCompositionRouteError {
    CapabilityUnavailable,
    Malformed(ZrRuntimeImeCompositionV2Error),
    WrongEventState,
    StaleGeneration,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct RuntimeImeCompositionDispatch {
    pub(super) context: zircon_runtime_interface::ZrRuntimeImeCompositionContextV2,
    pub(super) ui_event: UiInputEvent,
    pub(super) core_event: InputEvent,
}

pub(super) struct RuntimeImeCompositionRoute {
    negotiation: ZrRuntimeImeCompositionNegotiation,
    generation: RuntimeImeCompositionGenerationGate,
    candidate_rect: Option<ZrRuntimeImeCandidateRectV2>,
}

impl RuntimeImeCompositionRoute {
    pub(super) fn legacy(window_generation: u64) -> Self {
        Self::new(
            ZrRuntimeImeCompositionNegotiation::legacy(),
            window_generation,
        )
    }

    pub(super) fn new(
        negotiation: ZrRuntimeImeCompositionNegotiation,
        window_generation: u64,
    ) -> Self {
        Self {
            negotiation,
            generation: RuntimeImeCompositionGenerationGate::new(window_generation),
            candidate_rect: None,
        }
    }

    /// Rebinds the route when the host has completed capability negotiation for a new window.
    /// Rebinding also resets the generation high-water mark, so callbacks from the retired
    /// window/focus lifetime cannot mutate the new session.
    pub(super) fn reconfigure(
        &mut self,
        negotiation: ZrRuntimeImeCompositionNegotiation,
        window_generation: u64,
    ) {
        if self.generation.window_generation() == window_generation
            && self.negotiation == negotiation
        {
            // Cursor-area updates and repeated host configuration for the same live binding must
            // not erase the high-water tuple.  A delayed preedit/commit/cancel remains stale
            // after a same-window retry; only a new window or changed negotiated contract gets a
            // fresh gate.
            self.candidate_rect = None;
            return;
        }
        *self = Self::new(negotiation, window_generation);
    }

    pub(super) fn reconfigure_with_candidate_rect(
        &mut self,
        negotiation: ZrRuntimeImeCompositionNegotiation,
        window_generation: u64,
        candidate_rect: ZrRuntimeImeCandidateRectV2,
    ) {
        if self.generation.window_generation() == window_generation
            && self.negotiation == negotiation
        {
            // Rebinding the caret topology does not re-open a completed composition.  Keep the
            // existing generation gate while replacing the checked rectangle consumed by the
            // current host binding.
            self.candidate_rect = Some(candidate_rect);
            return;
        }
        *self = Self {
            negotiation,
            generation: RuntimeImeCompositionGenerationGate::new(window_generation),
            candidate_rect: Some(candidate_rect),
        };
    }

    #[cfg(test)]
    pub(super) fn candidate_rect(&self) -> Option<ZrRuntimeImeCandidateRectV2> {
        self.candidate_rect
    }

    pub(super) fn candidate_rect_for_window_generation(
        &self,
        window_generation: u64,
    ) -> Option<ZrRuntimeImeCandidateRectV2> {
        (self.generation.window_generation() == window_generation)
            .then_some(self.candidate_rect)
            .flatten()
    }

    pub(super) fn consume(
        &mut self,
        payload: &[u8],
        metadata: UiInputEventMetadata,
    ) -> Result<RuntimeImeCompositionDispatch, RuntimeImeCompositionRouteError> {
        if !self.negotiation.supports_v2() {
            return Err(RuntimeImeCompositionRouteError::CapabilityUnavailable);
        }
        // Decode into owned values before touching the gate or either downstream queue.
        let decoded = ZrRuntimeImeCompositionV2::decode(payload)
            .map_err(RuntimeImeCompositionRouteError::Malformed)?;
        let context = decoded.context;
        match decoded.operation {
            ZrRuntimeImeCompositionOperationV2::Preedit => {
                if !self.generation.begin_or_update(context) {
                    return Err(RuntimeImeCompositionRouteError::StaleGeneration);
                }
                let ui_event = decoded_preedit_to_ui_event(decoded.clone(), metadata)
                    .expect("validated preedit must map to a UI event");
                let ZrRuntimeImeCompositionV2 {
                    text, cursor_range, ..
                } = decoded;
                let cursor = cursor_range.map(|range| {
                    crate::core::framework::input::ImeCursorRange::new(
                        range.start_byte as usize,
                        range.end_byte as usize,
                    )
                });
                Ok(RuntimeImeCompositionDispatch {
                    context,
                    ui_event,
                    core_event: InputEvent::Ime(ImeEvent::Preedit(ImePreedit::new(text, cursor))),
                })
            }
            ZrRuntimeImeCompositionOperationV2::Commit => {
                if !self.generation.commit(context) {
                    return Err(RuntimeImeCompositionRouteError::StaleGeneration);
                }
                let text = decoded.text;
                Ok(RuntimeImeCompositionDispatch {
                    context,
                    ui_event: UiInputEvent::Ime(UiImeInputEvent {
                        metadata,
                        kind: UiImeInputEventKind::Commit,
                        text: text.clone(),
                        cursor_range: None,
                        preedit_clauses: Vec::new(),
                        delete_surrounding: None,
                    }),
                    core_event: InputEvent::Ime(ImeEvent::Commit(text)),
                })
            }
            ZrRuntimeImeCompositionOperationV2::Cancel => {
                if !self.generation.cancel(context) {
                    return Err(RuntimeImeCompositionRouteError::StaleGeneration);
                }
                Ok(RuntimeImeCompositionDispatch {
                    context,
                    ui_event: UiInputEvent::Ime(UiImeInputEvent {
                        metadata,
                        kind: UiImeInputEventKind::Cancel,
                        text: String::new(),
                        cursor_range: None,
                        preedit_clauses: Vec::new(),
                        delete_surrounding: None,
                    }),
                    core_event: InputEvent::Ime(ImeEvent::Disabled),
                })
            }
        }
    }

    /// API-facing seam used by `events.rs`: the borrowed C event is copied/validated before the
    /// route touches its generation gate.  `handle_event` calls this synchronously while the
    /// host-owned payload remains alive.
    pub(super) fn consume_event(
        &mut self,
        event: ZrRuntimeEventV1,
        metadata: UiInputEventMetadata,
    ) -> Result<RuntimeImeCompositionDispatch, RuntimeImeCompositionRouteError> {
        if event.kind != ZR_RUNTIME_EVENT_KIND_IME_V1
            || event.state != ZR_RUNTIME_IME_COMPOSITION_V2_EVENT_STATE
        {
            return Err(RuntimeImeCompositionRouteError::WrongEventState);
        }
        let payload = unsafe {
            event
                .payload
                .checked_slice(ZR_RUNTIME_EVENT_PAYLOAD_MAX_ENCODED_BYTES_V1)
                .map_err(|_| {
                    RuntimeImeCompositionRouteError::Malformed(
                        ZrRuntimeImeCompositionV2Error::LengthMismatch,
                    )
                })?
        };
        self.consume(payload, metadata)
    }
}

#[cfg(test)]
#[path = "tests/ime_composition_route.rs"]
mod tests;
