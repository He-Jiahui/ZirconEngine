use crate::ui::workbench::fixture::default_preview_fixture;
use crate::ui::workbench::layout::{ActivityDrawerSlot, MainPageId};

#[test]
fn default_preview_fixture_loads_shared_workbench_state() {
    let fixture = default_preview_fixture();

    assert_eq!(fixture.layout.active_main_page, MainPageId::workbench());
    assert_eq!(fixture.layout.drawers.len(), 6);
    assert!(fixture
        .layout
        .drawers
        .contains_key(&ActivityDrawerSlot::LeftTop));
    assert!(fixture
        .descriptors
        .iter()
        .any(|descriptor| descriptor.descriptor_id.0 == "editor.scene"));
    assert!(fixture
        .descriptors
        .iter()
        .any(|descriptor| descriptor.descriptor_id.0 == "editor.game"));
    assert!(fixture
        .descriptors
        .iter()
        .any(|descriptor| descriptor.descriptor_id.0 == "editor.assets"));
    assert!(fixture
        .instances
        .iter()
        .any(|instance| instance.descriptor_id.0 == "editor.scene"));
    assert!(fixture
        .instances
        .iter()
        .any(|instance| instance.descriptor_id.0 == "editor.game"));
    assert!(fixture
        .instances
        .iter()
        .any(|instance| instance.descriptor_id.0 == "editor.assets"));
}
