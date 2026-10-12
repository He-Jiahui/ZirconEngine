use crate::ui::retained_host::UiHostWindow;
use crate::ui::workbench::layout::MainPageId;

pub(crate) fn resolve_callback_source_window_id(ui: &UiHostWindow) -> Option<MainPageId> {
    let generation = ui.get_host_presentation_generation();
    let host_shell = &generation.structure().host_shell;
    if !host_shell.native_floating_window_mode {
        return None;
    }

    owned_non_blank_window_id(&host_shell.native_floating_window_id).map(MainPageId::new)
}

fn owned_non_blank_window_id(window_id: &str) -> Option<String> {
    if window_id.trim().is_empty() {
        None
    } else {
        Some(window_id.to_owned())
    }
}

#[cfg(test)]
#[path = "tests/resolution.rs"]
mod tests;
