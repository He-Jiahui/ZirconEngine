use zircon_runtime_interface::{ZrRuntimeHighlightRenderAttributesV1, ZrRuntimeHighlightSetV1};

use super::super::{EditorRuntimeHighlightSet, GatewayError};
use super::gateway::SessionGateway;
use super::protocol::ensure_status;

impl SessionGateway {
    pub(super) fn submit_highlight_set(
        &self,
        set: EditorRuntimeHighlightSet,
    ) -> Result<(), GatewayError> {
        if !set.is_valid() {
            return Err(GatewayError::Protocol {
                message: "invalid runtime highlight set".to_owned(),
            });
        }
        if !self
            .capabilities
            .core_capabilities()
            .iter()
            .any(|capability| capability == "runtime.editor_overlay.highlight_set")
        {
            return Err(GatewayError::CapabilityMissing {
                capability: "runtime.editor_overlay.highlight_set",
            });
        }
        let submit = Self::required(
            self.api.submit_highlight_set,
            "runtime.editor_overlay.highlight_set",
        )?;
        let request = ZrRuntimeHighlightSetV1::new(
            set.viewport(),
            set.generation(),
            set.entities(),
            ZrRuntimeHighlightRenderAttributesV1 {
                outline_enabled: if set.outline_enabled() { 1 } else { 0 },
                tint_rgba: set.tint_rgba(),
            },
        );
        ensure_status(
            unsafe { submit(self.session, request) },
            "submit runtime highlight set",
        )
    }
}
