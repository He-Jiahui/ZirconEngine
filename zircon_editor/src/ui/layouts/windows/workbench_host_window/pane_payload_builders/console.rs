use super::super::pane_payload::{ConsolePanePayload, PanePayload};
use super::super::pane_presentation::PanePayloadBuildContext;

pub(super) fn build(context: &PanePayloadBuildContext<'_>) -> PanePayload {
    PanePayload::ConsoleV1(ConsolePanePayload {
        status_text: context.chrome.status_line.clone(),
    })
}
