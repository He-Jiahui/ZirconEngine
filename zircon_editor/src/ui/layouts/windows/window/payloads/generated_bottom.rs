use super::super::pane_payload::{GeneratedBottomPanePayload, PanePayload};
use super::super::pane_presentation::PanePayloadBuildContext;

pub(super) fn build(_context: &PanePayloadBuildContext<'_>) -> PanePayload {
    PanePayload::GeneratedBottomV1(GeneratedBottomPanePayload {
        status: "Generated editor feedback panels".to_string(),
    })
}
