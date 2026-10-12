use super::super::pane_payload::{PanePayload, UiComponentShowcasePanePayload};
use super::super::pane_presentation::PanePayloadBuildContext;

pub(super) fn build(_context: &PanePayloadBuildContext<'_>) -> PanePayload {
    PanePayload::UiComponentShowcaseV1(UiComponentShowcasePanePayload {
        state_summary: "Runtime UI component showcase".to_string(),
    })
}
