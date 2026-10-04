//! Portable host producer for the negotiated IME composition envelope.
//!
//! Winit's `Ime::Preedit` remains on the V1 text/cursor path because it does not expose native
//! conversion attributes or a source epoch.  Native adapters call `native_preedit` only after
//! they have read real attributes and supplied the context that owns the callback.

use zircon_runtime_interface::ui::dispatch::{UiImePreeditClause, UiTextByteRange};
use zircon_runtime_interface::{
    ZrByteSlice, ZrRuntimeEventV1, ZrRuntimeImeCandidateRectV2, ZrRuntimeImeCompositionContextV2,
    ZrRuntimeImeCompositionNegotiation, ZrRuntimeImeCompositionOperationV2,
    ZrRuntimeImeCompositionV2, ZrRuntimeImeCompositionV2Error, ZrRuntimeImeCompositionWireVersion,
    ZrRuntimeImePreeditClauseAvailabilityV2, ZrRuntimeViewportHandle,
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_EVENT_KIND_IME_V1, ZR_RUNTIME_IME_CURSOR_HIDDEN_V1,
    ZR_RUNTIME_IME_STATE_COMMIT_V1, ZR_RUNTIME_IME_STATE_DISABLED_V1,
    ZR_RUNTIME_IME_STATE_PREEDIT_V1,
};

use zircon_runtime_interface::ZR_RUNTIME_IME_COMPOSITION_V2_EVENT_STATE;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImeCompositionProducerError {
    InvalidWindowGeneration,
    GenerationExhausted,
    InvalidCursorRange,
    Encode(ZrRuntimeImeCompositionV2Error),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImeCompositionClauseAvailability<'a> {
    Unavailable,
    Available(&'a [UiImePreeditClause]),
}

/// A lifetime token allocated by the host window/focus/composition owner.  It never uses a UI
/// node id, so the same local node id in another window cannot collide with a delayed callback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeImeCompositionContextAllocator {
    window_generation: u64,
    focus_generation: u64,
    composition_generation: u64,
}

impl RuntimeImeCompositionContextAllocator {
    pub const fn new(window_generation: u64) -> Result<Self, ImeCompositionProducerError> {
        if window_generation == 0 {
            return Err(ImeCompositionProducerError::InvalidWindowGeneration);
        }
        Ok(Self {
            window_generation,
            focus_generation: 1,
            composition_generation: 1,
        })
    }

    pub const fn current(&self) -> ZrRuntimeImeCompositionContextV2 {
        ZrRuntimeImeCompositionContextV2 {
            window_generation: self.window_generation,
            focus_generation: self.focus_generation,
            composition_generation: self.composition_generation,
        }
    }

    /// Focus retirement advances both the focus and composition lifetime before the host sends
    /// cancellation.  Checked arithmetic fails closed rather than reusing a token.
    pub fn focus_changed(
        &mut self,
    ) -> Result<ZrRuntimeImeCompositionContextV2, ImeCompositionProducerError> {
        self.focus_generation = self
            .focus_generation
            .checked_add(1)
            .ok_or(ImeCompositionProducerError::GenerationExhausted)?;
        self.composition_generation = self
            .composition_generation
            .checked_add(1)
            .ok_or(ImeCompositionProducerError::GenerationExhausted)?;
        Ok(self.current())
    }

    pub fn begin_composition(
        &mut self,
    ) -> Result<ZrRuntimeImeCompositionContextV2, ImeCompositionProducerError> {
        self.composition_generation = self
            .composition_generation
            .checked_add(1)
            .ok_or(ImeCompositionProducerError::GenerationExhausted)?;
        Ok(self.current())
    }
}

/// Owns payload bytes until the synchronous runtime event dispatch returns.  The event itself is
/// rebuilt on demand so its borrowed slice always points at this envelope's current allocation.
#[derive(Clone, Debug)]
pub struct OwnedRuntimeImeEvent {
    viewport: ZrRuntimeViewportHandle,
    state: u32,
    key_code: u32,
    scan_code: u32,
    payload: Vec<u8>,
}

impl OwnedRuntimeImeEvent {
    pub fn event(&self) -> ZrRuntimeEventV1 {
        let mut event = ZrRuntimeEventV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            ZR_RUNTIME_EVENT_KIND_IME_V1,
            self.viewport,
        );
        event.state = self.state;
        event.key_code = self.key_code;
        event.scan_code = self.scan_code;
        event.payload = ZrByteSlice {
            data: self.payload.as_ptr(),
            len: self.payload.len(),
        };
        event
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub const fn state(&self) -> u32 {
        self.state
    }
}

pub struct RuntimeImeCompositionProducer {
    negotiation: ZrRuntimeImeCompositionNegotiation,
    context: RuntimeImeCompositionContextAllocator,
    candidate_rect: Option<ZrRuntimeImeCandidateRectV2>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NativeImeCompositionInput<'a> {
    Preedit {
        text: &'a str,
        cursor: Option<UiTextByteRange>,
        clauses: ImeCompositionClauseAvailability<'a>,
        begin: bool,
    },
    Commit {
        text: &'a str,
    },
    Cancel,
}

impl RuntimeImeCompositionProducer {
    pub fn focus_changed_event(
        &mut self,
        viewport: ZrRuntimeViewportHandle,
    ) -> Result<OwnedRuntimeImeEvent, ImeCompositionProducerError> {
        self.context.focus_changed()?;
        self.native_cancel(viewport)
    }

    pub fn native_event(
        &mut self,
        viewport: ZrRuntimeViewportHandle,
        input: NativeImeCompositionInput<'_>,
    ) -> Result<OwnedRuntimeImeEvent, ImeCompositionProducerError> {
        match input {
            NativeImeCompositionInput::Preedit {
                text,
                cursor,
                clauses,
                begin,
            } => {
                if begin {
                    self.context.begin_composition()?;
                }
                self.native_preedit(viewport, text, cursor, clauses)
            }
            NativeImeCompositionInput::Commit { text } => self.native_commit(viewport, text),
            NativeImeCompositionInput::Cancel => self.native_cancel(viewport),
        }
    }
}

impl RuntimeImeCompositionProducer {
    pub fn new(
        negotiation: ZrRuntimeImeCompositionNegotiation,
        context: RuntimeImeCompositionContextAllocator,
    ) -> Self {
        Self::new_with_optional_candidate_rect(negotiation, context, None)
    }

    pub fn new_with_candidate_rect(
        negotiation: ZrRuntimeImeCompositionNegotiation,
        context: RuntimeImeCompositionContextAllocator,
        candidate_rect: ZrRuntimeImeCandidateRectV2,
    ) -> Self {
        Self::new_with_optional_candidate_rect(negotiation, context, Some(candidate_rect))
    }

    fn new_with_optional_candidate_rect(
        negotiation: ZrRuntimeImeCompositionNegotiation,
        context: RuntimeImeCompositionContextAllocator,
        candidate_rect: Option<ZrRuntimeImeCandidateRectV2>,
    ) -> Self {
        Self {
            negotiation,
            context,
            candidate_rect,
        }
    }

    pub const fn context(&self) -> ZrRuntimeImeCompositionContextV2 {
        self.context.current()
    }

    /// Returns the negotiated window-relative candidate bounds only for the owning window
    /// generation. A callback retained from a retired window cannot publish its old rectangle.
    pub const fn candidate_rect_for_window_generation(
        &self,
        window_generation: u64,
    ) -> Option<ZrRuntimeImeCandidateRectV2> {
        if self.context.current().window_generation == window_generation {
            self.candidate_rect
        } else {
            None
        }
    }

    pub fn context_allocator_mut(&mut self) -> &mut RuntimeImeCompositionContextAllocator {
        &mut self.context
    }

    pub fn update_candidate_rect(&mut self, candidate_rect: ZrRuntimeImeCandidateRectV2) {
        self.candidate_rect = Some(candidate_rect);
    }

    /// This is the Winit fallback.  It deliberately emits V1 and cannot claim attributes or a
    /// source generation that Winit did not provide.
    pub fn winit_preedit(
        &self,
        viewport: ZrRuntimeViewportHandle,
        text: &str,
        cursor: Option<UiTextByteRange>,
    ) -> Result<OwnedRuntimeImeEvent, ImeCompositionProducerError> {
        Self::legacy_preedit(viewport, text, cursor)
    }

    pub fn native_preedit(
        &self,
        viewport: ZrRuntimeViewportHandle,
        text: &str,
        cursor: Option<UiTextByteRange>,
        clauses: ImeCompositionClauseAvailability<'_>,
    ) -> Result<OwnedRuntimeImeEvent, ImeCompositionProducerError> {
        if self.negotiation.wire_version() != ZrRuntimeImeCompositionWireVersion::CompositionV2 {
            return Self::legacy_preedit(viewport, text, cursor);
        }
        let (clause_availability, clauses) = match clauses {
            ImeCompositionClauseAvailability::Unavailable => (
                ZrRuntimeImePreeditClauseAvailabilityV2::Unavailable,
                Vec::new(),
            ),
            ImeCompositionClauseAvailability::Available(clauses) => (
                ZrRuntimeImePreeditClauseAvailabilityV2::Available,
                clauses.to_vec(),
            ),
        };
        let composition = ZrRuntimeImeCompositionV2 {
            operation: ZrRuntimeImeCompositionOperationV2::Preedit,
            context: self.context.current(),
            text: text.to_owned(),
            cursor_range: cursor,
            clauses,
            clause_availability,
        };
        self.v2_event(viewport, composition)
    }

    pub fn native_commit(
        &self,
        viewport: ZrRuntimeViewportHandle,
        text: &str,
    ) -> Result<OwnedRuntimeImeEvent, ImeCompositionProducerError> {
        if self.negotiation.wire_version() != ZrRuntimeImeCompositionWireVersion::CompositionV2 {
            return Ok(Self::legacy_event(
                viewport,
                ZR_RUNTIME_IME_STATE_COMMIT_V1,
                0,
                0,
                text.as_bytes().to_vec(),
            ));
        }
        self.v2_event(
            viewport,
            ZrRuntimeImeCompositionV2 {
                operation: ZrRuntimeImeCompositionOperationV2::Commit,
                context: self.context.current(),
                text: text.to_owned(),
                cursor_range: None,
                clauses: Vec::new(),
                clause_availability: ZrRuntimeImePreeditClauseAvailabilityV2::Unavailable,
            },
        )
    }

    pub fn native_cancel(
        &self,
        viewport: ZrRuntimeViewportHandle,
    ) -> Result<OwnedRuntimeImeEvent, ImeCompositionProducerError> {
        if self.negotiation.wire_version() != ZrRuntimeImeCompositionWireVersion::CompositionV2 {
            return Ok(Self::legacy_event(
                viewport,
                ZR_RUNTIME_IME_STATE_DISABLED_V1,
                0,
                0,
                Vec::new(),
            ));
        }
        self.v2_event(
            viewport,
            ZrRuntimeImeCompositionV2 {
                operation: ZrRuntimeImeCompositionOperationV2::Cancel,
                context: self.context.current(),
                text: String::new(),
                cursor_range: None,
                clauses: Vec::new(),
                clause_availability: ZrRuntimeImePreeditClauseAvailabilityV2::Unavailable,
            },
        )
    }

    fn legacy_preedit(
        viewport: ZrRuntimeViewportHandle,
        text: &str,
        cursor: Option<UiTextByteRange>,
    ) -> Result<OwnedRuntimeImeEvent, ImeCompositionProducerError> {
        let (key_code, scan_code) = cursor
            .map(|range| (range.start_byte, range.end_byte))
            .unwrap_or((
                ZR_RUNTIME_IME_CURSOR_HIDDEN_V1,
                ZR_RUNTIME_IME_CURSOR_HIDDEN_V1,
            ));
        Ok(Self::legacy_event(
            viewport,
            ZR_RUNTIME_IME_STATE_PREEDIT_V1,
            key_code,
            scan_code,
            text.as_bytes().to_vec(),
        ))
    }

    fn v2_event(
        &self,
        viewport: ZrRuntimeViewportHandle,
        composition: ZrRuntimeImeCompositionV2,
    ) -> Result<OwnedRuntimeImeEvent, ImeCompositionProducerError> {
        let payload = composition
            .encode()
            .map_err(ImeCompositionProducerError::Encode)?;
        Ok(Self::legacy_event(
            viewport,
            ZR_RUNTIME_IME_COMPOSITION_V2_EVENT_STATE,
            0,
            0,
            payload,
        ))
    }

    fn legacy_event(
        viewport: ZrRuntimeViewportHandle,
        state: u32,
        key_code: u32,
        scan_code: u32,
        payload: Vec<u8>,
    ) -> OwnedRuntimeImeEvent {
        OwnedRuntimeImeEvent {
            viewport,
            state,
            key_code,
            scan_code,
            payload,
        }
    }
}

#[cfg(test)]
#[path = "tests/composition_v2.rs"]
mod tests;
