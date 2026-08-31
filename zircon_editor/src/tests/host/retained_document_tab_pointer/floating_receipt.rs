use crate::ui::retained_host::document_tab_pointer::build_host_document_tab_pointer_layout;
use crate::ui::workbench::fixture::default_preview_fixture;
use crate::ui::workbench::layout::{
    DocumentNode, FloatingWindowLayout, MainPageId, TabStackLayout,
};
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::view::{ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId};

#[test]
fn document_tab_receipt_projection_keeps_floating_window_identity_without_geometry() {
    let mut fixture = default_preview_fixture();
    let window_id = MainPageId::new("window:preview");
    let scene_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.scene#float"),
        descriptor_id: ViewDescriptorId::new("editor.scene"),
        title: "Scene".to_string(),
        serializable_payload: serde_json::json!({ "path": "crate://scene/floating.scene" }),
        dirty: false,
        host: ViewHost::FloatingWindow(window_id.clone(), vec![]),
    };
    fixture.instances.push(scene_instance.clone());
    fixture.layout.floating_windows.push(FloatingWindowLayout {
        window_id: window_id.clone(),
        title: "Preview Popout".to_string(),
        workspace: DocumentNode::Tabs(TabStackLayout {
            tabs: vec![scene_instance.instance_id.clone()],
            active_tab: Some(scene_instance.instance_id.clone()),
        }),
        focused_view: Some(scene_instance.instance_id.clone()),
        frame: Default::default(),
    });

    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let layout = build_host_document_tab_pointer_layout(&model);
    let floating_surface = layout
        .surfaces
        .iter()
        .find(|surface| surface.key == window_id.0)
        .expect("floating window receipt surface should exist");

    assert_eq!(floating_surface.items.len(), 1);
    assert_eq!(
        floating_surface.items[0].instance_id,
        scene_instance.instance_id
    );
}
