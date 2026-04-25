use crate::ui::host::NativeWindowHostState;
use crate::ui::slint_host::document_tab_pointer::build_host_document_tab_pointer_layout;
use crate::ui::slint_host::floating_window_projection::build_floating_window_projection_bundle;
use crate::ui::workbench::autolayout::{
    compute_workbench_shell_geometry, ShellFrame, ShellSizePx, WorkbenchChromeMetrics,
};
use crate::ui::workbench::fixture::default_preview_fixture;
use crate::ui::workbench::layout::{
    DocumentNode, FloatingWindowLayout, MainPageId, TabStackLayout,
};
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::view::{ViewDescriptorId, ViewHost, ViewInstance, ViewInstanceId};
use zircon_runtime::ui::layout::UiFrame;

#[test]
fn shared_document_tab_pointer_layout_prefers_native_window_host_bounds_for_floating_strip() {
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
        frame: ShellFrame::new(420.0, 180.0, 360.0, 240.0),
    });

    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    let geometry = compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        ShellSizePx::new(1440.0, 900.0),
        &WorkbenchChromeMetrics::default(),
        None,
    );
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        &geometry,
        &WorkbenchChromeMetrics::default(),
        &[NativeWindowHostState {
            window_id: window_id.clone(),
            handle: None,
            bounds: [640.0, 320.0, 700.0, 420.0],
        }],
    );
    let layout = build_host_document_tab_pointer_layout(
        &model,
        &geometry,
        &WorkbenchChromeMetrics::default(),
        None,
        &floating_window_projection_bundle,
    );
    let floating_surface = layout
        .surfaces
        .iter()
        .find(|surface| surface.key == window_id.0)
        .expect("floating window strip should exist");

    assert_eq!(
        floating_surface.strip_frame,
        UiFrame::new(
            640.0,
            320.0,
            700.0,
            WorkbenchChromeMetrics::default().document_header_height,
        ),
        "floating tab strip should follow native host bounds when they are available"
    );
}
