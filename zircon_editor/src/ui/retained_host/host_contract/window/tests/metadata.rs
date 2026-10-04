use super::super::UiHostWindow;
use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostAssetDeletionBlockerData, HostClosePromptData, HostWindowPresentationData,
    TemplatePaneNodeData,
};
use crate::ui::retained_host::primitives::{ModelRc, VecModel};
use std::rc::Rc;

#[test]
fn overlay_payload_updates_preserve_the_workbench_hit_index_authority() {
    let host = UiHostWindow::new().expect("host window should construct for overlay test");
    let paint_nodes = ModelRc::from(Rc::new(VecModel::from(vec![TemplatePaneNodeData {
        control_id: "overlay.indexed.paint".into(),
        ..TemplatePaneNodeData::default()
    }])));
    let mut presentation = HostWindowPresentationData::default();
    presentation.root_template_nodes = paint_nodes.clone();
    host.set_host_presentation(presentation);
    let before = host.get_host_presentation_generation();

    host.set_close_prompt(HostClosePromptData {
        visible: true,
        title: "Unsaved changes".into(),
        overlay_frame: FrameRect {
            width: 1_280.0,
            height: 720.0,
            ..FrameRect::default()
        },
        ..HostClosePromptData::default()
    });
    let close_prompt = host.get_host_presentation_generation();
    assert!(close_prompt.structure_generation() > before.structure_generation());
    assert!(close_prompt.geometry_generation() > before.geometry_generation());
    assert_eq!(
        close_prompt.hit_test_generation(),
        before.hit_test_generation()
    );
    assert!(close_prompt
        .workbench_hit_index()
        .indexes_paint_nodes(&paint_nodes));

    host.set_asset_deletion_blocker(HostAssetDeletionBlockerData {
        visible: true,
        target: "asset://blocked".into(),
        overlay_frame: FrameRect {
            width: 1_280.0,
            height: 720.0,
            ..FrameRect::default()
        },
        ..HostAssetDeletionBlockerData::default()
    });
    let blocker = host.get_host_presentation_generation();
    assert!(blocker.structure_generation() > close_prompt.structure_generation());
    assert!(blocker.geometry_generation() > close_prompt.geometry_generation());
    assert_eq!(
        blocker.hit_test_generation(),
        close_prompt.hit_test_generation()
    );
    assert!(blocker
        .workbench_hit_index()
        .indexes_paint_nodes(&paint_nodes));
}

#[test]
fn native_floating_metadata_updates_preserve_the_workbench_hit_index_authority() {
    let host = UiHostWindow::new().expect("host window should construct for native metadata test");
    let paint_nodes = ModelRc::from(Rc::new(VecModel::from(vec![TemplatePaneNodeData {
        control_id: "native.metadata.indexed.paint".into(),
        ..TemplatePaneNodeData::default()
    }])));
    let mut presentation = HostWindowPresentationData::default();
    presentation.root_template_nodes = paint_nodes.clone();
    host.set_host_presentation(presentation);
    let before = host.get_host_presentation_generation();
    let hit_index_address = before.workbench_hit_index() as *const _;
    let bounds = FrameRect {
        x: 12.0,
        y: 24.0,
        width: 640.0,
        height: 480.0,
    };

    assert!(host.set_native_floating_window_presentation("window-1", "tree-1", "Tools", &bounds,));
    let changed = host.get_host_presentation_generation();
    assert!(changed.structure_generation() > before.structure_generation());
    assert!(changed.geometry_generation() > before.geometry_generation());
    assert_eq!(changed.hit_test_generation(), before.hit_test_generation());
    assert_eq!(
        changed.workbench_hit_index() as *const _,
        hit_index_address,
        "native window metadata must not replace the pane hit-index authority"
    );
    assert!(changed
        .workbench_hit_index()
        .indexes_paint_nodes(&paint_nodes));
    assert_eq!(changed.structure().host_shell.native_window_bounds, bounds);

    let resized_bounds = FrameRect {
        width: 800.0,
        height: 600.0,
        ..bounds.clone()
    };
    assert!(host.set_native_floating_window_presentation(
        "window-1",
        "tree-1",
        "Tools",
        &resized_bounds,
    ));
    let resized = host.get_host_presentation_generation();
    assert_eq!(
        resized.structure_generation(),
        changed.structure_generation(),
        "bounds-only resize must not invalidate semantic structure"
    );
    assert!(resized.geometry_generation() > changed.geometry_generation());
    assert_eq!(resized.hit_test_generation(), changed.hit_test_generation());

    assert!(!host.set_native_floating_window_presentation(
        "window-1",
        "tree-1",
        "Tools",
        &resized_bounds,
    ));
    let stable = host.get_host_presentation_generation();
    assert!(resized.shares_structure_with(&stable));
    assert_eq!(
        stable.structure_generation(),
        resized.structure_generation()
    );
    assert_eq!(stable.geometry_generation(), resized.geometry_generation());
    assert_eq!(stable.hit_test_generation(), resized.hit_test_generation());
}
