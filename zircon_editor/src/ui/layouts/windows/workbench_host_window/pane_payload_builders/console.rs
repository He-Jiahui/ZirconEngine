use super::super::pane_payload::{ConsolePanePayload, PanePayload};
use super::super::pane_presentation::PanePayloadBuildContext;

pub(super) fn build(context: &PanePayloadBuildContext<'_>) -> PanePayload {
    PanePayload::ConsoleV1(ConsolePanePayload {
        status_text: context.chrome.console_output.text_arc(),
        levels: context.chrome.console_output.levels_arc(),
        counts: context.chrome.console_output.counts(),
        filter: context.chrome.console_output.filter(),
        source_filter: context.chrome.console_output.source_filter(),
        jump_sequences: context.chrome.console_output.jump_sequences_arc(),
    })
}
