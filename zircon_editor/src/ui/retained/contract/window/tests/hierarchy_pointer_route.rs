use super::*;
use crate::ui::retained_host::host_contract::data::FrameRect;

#[test]
fn native_hierarchy_source_does_not_keep_its_host_window_alive() {
    let ui = UiHostWindow::new().expect("child host window");
    ui.set_native_floating_window_presentation(
        "window:hierarchy",
        "tree:hierarchy",
        "Hierarchy",
        &FrameRect::default(),
    );
    let source = ui.hierarchy_pointer_source();
    assert_eq!(
        source.native_floating_window_id().as_deref(),
        Some("window:hierarchy")
    );
    drop(ui);
    assert!(source.native_floating_window_id().is_none());
}
