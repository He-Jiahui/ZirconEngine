//! Host negotiation handoff for the private V2 IME route.

use zircon_runtime_interface::{ZrRuntimeImeCandidateRectV2, ZrRuntimeImeCompositionNegotiation};

use super::RuntimeDynamicSession;

impl RuntimeDynamicSession {
    /// Installs the negotiated wire version, window lifetime token, and current caret topology
    /// before a native composition callback is admitted. A fresh route fences callbacks from the
    /// retired window/focus generation and keeps an unconfigured session fail-closed on V2 state 10.
    pub(super) fn configure_app_session_v2(
        &mut self,
        negotiation: zircon_runtime_interface::ZrRuntimeImeCompositionNegotiation,
        window_generation: u64,
        candidate_rect: ZrRuntimeImeCandidateRectV2,
    ) {
        self.ime_composition_route.reconfigure_with_candidate_rect(
            negotiation,
            window_generation,
            candidate_rect,
        );
    }
}
