use crate::{default_preview_fixture, ActivityDrawerSlot, MainPageId, WorkbenchViewModel};

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

#[test]
fn default_preview_fixture_projects_into_workbench_view_model() {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();

    let view_model = WorkbenchViewModel::build(&chrome);

    assert_eq!(view_model.host_strip.active_page, MainPageId::workbench());
    assert!(view_model.drawer_ring.visible);
    assert_eq!(
        view_model.status_bar.primary_text,
        fixture.editor.status_line
    );
}
