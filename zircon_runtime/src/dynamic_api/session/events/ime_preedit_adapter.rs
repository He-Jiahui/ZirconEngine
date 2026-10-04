use zircon_runtime_interface::ui::dispatch::{
    UiImeInputEvent, UiImeInputEventKind, UiInputEvent, UiInputEventMetadata,
};
use zircon_runtime_interface::{ZrRuntimeImeCompositionOperationV2, ZrRuntimeImeCompositionV2};

/// Converts only the validated V2 Preedit operation. Clause rows are moved through unchanged;
/// this adapter does not infer or synthesize platform attributes.
pub(in crate::dynamic_api::session) fn decoded_preedit_to_ui_event(
    decoded: ZrRuntimeImeCompositionV2,
    metadata: UiInputEventMetadata,
) -> Option<UiInputEvent> {
    if decoded.operation != ZrRuntimeImeCompositionOperationV2::Preedit {
        return None;
    }
    Some(UiInputEvent::Ime(UiImeInputEvent {
        metadata,
        kind: UiImeInputEventKind::Preedit,
        text: decoded.text,
        cursor_range: decoded.cursor_range,
        preedit_clauses: decoded.clauses,
        delete_surrounding: None,
    }))
}

#[cfg(test)]
#[path = "tests/ime_preedit_adapter.rs"]
mod tests;
