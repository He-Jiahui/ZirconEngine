use winit::event::Ime;
use zircon_runtime_interface::ui::dispatch::{
    UiImeInputEvent, UiImeInputEventKind, UiInputEvent, UiInputEventMetadata, UiTextByteRange,
    UiTextInputEvent,
};

pub(crate) fn native_ime_event_to_shared_input(
    metadata: UiInputEventMetadata,
    event: &Ime,
) -> Option<UiInputEvent> {
    match event {
        Ime::Preedit(text, cursor_range) => Some(UiInputEvent::Ime(UiImeInputEvent {
            metadata,
            kind: UiImeInputEventKind::Preedit,
            text: text.clone(),
            cursor_range: cursor_range.map(|(start, end)| {
                UiTextByteRange::new(clamp_byte_index(start), clamp_byte_index(end))
            }),
        })),
        Ime::Commit(text) => Some(UiInputEvent::Text(UiTextInputEvent {
            metadata,
            text: text.clone(),
        })),
        Ime::Disabled => Some(UiInputEvent::Ime(UiImeInputEvent {
            metadata,
            kind: UiImeInputEventKind::Cancel,
            text: String::new(),
            cursor_range: None,
        })),
        Ime::Enabled | Ime::DeleteSurrounding { .. } => None,
    }
}

fn clamp_byte_index(value: usize) -> u32 {
    value.min(u32::MAX as usize) as u32
}
