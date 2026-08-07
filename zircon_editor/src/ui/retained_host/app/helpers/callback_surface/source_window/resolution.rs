use crate::ui::retained_host::UiHostWindow;
use crate::ui::workbench::layout::MainPageId;

pub(crate) fn resolve_callback_source_window_id(ui: &UiHostWindow) -> Option<MainPageId> {
    let generation = ui.get_host_presentation_generation();
    let host_shell = &generation.structure().host_shell;
    if !host_shell.native_floating_window_mode {
        return None;
    }

    let window_id = host_shell.native_floating_window_id.clone();
    if window_id.trim().is_empty() {
        None
    } else {
        Some(MainPageId::new(window_id))
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_callback_source_window_id;
    use crate::ui::retained_host::UiHostWindow;
    use crate::ui::workbench::layout::MainPageId;

    #[test]
    fn resolve_callback_source_window_id_returns_none_for_root_shell() {
        let ui = UiHostWindow::new().expect("workbench shell should instantiate");
        assert_eq!(resolve_callback_source_window_id(&ui), None);
    }

    #[test]
    fn resolve_callback_source_window_id_reads_native_child_window_identity() {
        let ui = UiHostWindow::new().expect("workbench shell should instantiate");
        let mut host_presentation = ui.get_host_presentation();
        host_presentation.host_shell.native_floating_window_mode = true;
        host_presentation.host_shell.native_floating_window_id = "window:native-preview".into();
        ui.set_host_presentation(host_presentation);

        assert_eq!(
            resolve_callback_source_window_id(&ui),
            Some(MainPageId::new("window:native-preview"))
        );
    }
}
