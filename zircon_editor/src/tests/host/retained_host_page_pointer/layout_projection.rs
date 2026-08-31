use crate::tests::editor_event::support::EventRuntimeHarness;
use crate::ui::retained_host::host_page_pointer::build_host_page_pointer_layout;
use crate::ui::workbench::layout::MainPageId;
use crate::ui::workbench::model::WorkbenchViewModel;

#[test]
fn host_page_receipt_projection_keeps_typed_page_identity_without_geometry() {
    let harness = EventRuntimeHarness::new("zircon_retained_host_page_receipt_projection");
    let chrome = harness.runtime.chrome_snapshot();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );

    let layout = build_host_page_pointer_layout(&model);

    assert_eq!(layout.items.len(), model.host_strip.pages.len());
    assert_eq!(layout.items[0].page_id, MainPageId::workbench());
}
