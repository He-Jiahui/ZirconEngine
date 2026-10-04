use super::super::UiHostWindow;
use crate::scene::viewport::{CapturedFrame, RenderViewportHandle};
use crate::ui::retained_host::host_contract::data::{
    FrameRect, HostWindowGeometryPresentationData, HostWindowPresentationData, PaneData,
    TemplateNodeFrameData, TemplatePaneNodeData,
};
use crate::ui::retained_host::host_contract::PaneSurfaceHostContext;
use crate::ui::retained_host::primitives::{ModelRc, VecModel};
use std::rc::Rc;
use std::sync::Arc;

#[test]
fn presentation_generation_reuses_structure_until_a_structural_publish() {
    let host = UiHostWindow::new().expect("host window should construct for generation test");

    let initial = host.get_host_presentation_generation();
    let stable = host.get_host_presentation_generation();

    assert!(initial.shares_structure_with(&stable));
    assert!(initial.shares_theme_with(&stable));
    assert_eq!(
        initial.structure_generation(),
        stable.structure_generation()
    );

    host.set_host_presentation(initial.structure().clone());
    let published = host.get_host_presentation_generation();

    assert!(!initial.shares_structure_with(&published));
    assert!(published.structure_generation() > initial.structure_generation());
    assert_eq!(
        initial.hit_test_generation(),
        published.hit_test_generation()
    );
}

#[test]
fn semantic_update_does_not_cow_for_internal_generation_bookkeeping() {
    let host = UiHostWindow::new().expect("host window should construct for generation test");
    host.set_host_presentation(HostWindowPresentationData::default());
    let before = host.get_host_presentation_generation();
    let presentation_address = before.structure() as *const HostWindowPresentationData;
    let structure_generation = before.structure_generation();
    drop(before);

    host.update_host_presentation(|presentation| {
        presentation.host_scene_data.left_dock.pane.title = "changed".into();
    });

    let after = host.get_host_presentation_generation();
    assert_eq!(
        after.structure() as *const HostWindowPresentationData,
        presentation_address,
        "internal semantic identity must not force presentation copy-on-write"
    );
    assert!(after.structure_generation() > structure_generation);
    assert_eq!(
        after
            .structure()
            .host_scene_data
            .left_dock
            .pane
            .title
            .as_str(),
        "changed"
    );
}

#[test]
fn geometry_publish_preserves_semantic_generation_and_pane_handles() {
    let host = UiHostWindow::new().expect("host window should construct for geometry test");
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.left_dock.pane = PaneData {
        id: "left.semantic".into(),
        body_surface_frame: Some(Arc::new(
            zircon_runtime_interface::ui::surface::UiSurfaceFrame::default(),
        )),
        ..PaneData::default()
    };
    presentation.host_scene_data.document_dock.pane = PaneData {
        id: "document.semantic".into(),
        body_surface_frame: Some(Arc::new(
            zircon_runtime_interface::ui::surface::UiSurfaceFrame::default(),
        )),
        ..PaneData::default()
    };
    presentation.host_scene_data.right_dock.pane = PaneData {
        id: "right.semantic".into(),
        body_surface_frame: Some(Arc::new(
            zircon_runtime_interface::ui::surface::UiSurfaceFrame::default(),
        )),
        ..PaneData::default()
    };
    presentation.host_scene_data.bottom_dock.pane = PaneData {
        id: "bottom.semantic".into(),
        body_surface_frame: Some(Arc::new(
            zircon_runtime_interface::ui::surface::UiSurfaceFrame::default(),
        )),
        ..PaneData::default()
    };
    presentation.workbench_window_nodes =
        ModelRc::from(Rc::new(VecModel::from(vec![TemplatePaneNodeData {
            control_id: "geometry.control".into(),
            frame: TemplateNodeFrameData {
                x: 0.0,
                y: 0.0,
                width: 24.0,
                height: 24.0,
            },
            ..TemplatePaneNodeData::default()
        }])));
    host.set_host_presentation(presentation);

    let baseline = host.get_host_presentation_generation();
    let baseline_structure_generation = baseline.structure_generation();
    let baseline_geometry_generation = baseline.geometry_generation();
    let baseline_hit_generation = baseline.hit_test_generation();
    let left_pane = Arc::clone(
        baseline
            .structure()
            .host_scene_data
            .left_dock
            .pane
            .body_surface_frame
            .as_ref()
            .expect("left surface frame"),
    );
    let document_pane = Arc::clone(
        baseline
            .structure()
            .host_scene_data
            .document_dock
            .pane
            .body_surface_frame
            .as_ref()
            .expect("document surface frame"),
    );
    let right_pane = Arc::clone(
        baseline
            .structure()
            .host_scene_data
            .right_dock
            .pane
            .body_surface_frame
            .as_ref()
            .expect("right surface frame"),
    );
    let bottom_pane = Arc::clone(
        baseline
            .structure()
            .host_scene_data
            .bottom_dock
            .pane
            .body_surface_frame
            .as_ref()
            .expect("bottom surface frame"),
    );
    let mut geometry = HostWindowGeometryPresentationData::from_presentation(baseline.structure());
    geometry.host_layout.center_band_frame.width = 1600.0;
    geometry.host_scene_data.left_dock.region_frame.width = 320.0;
    geometry.workbench_window_nodes =
        ModelRc::from(Rc::new(VecModel::from(vec![TemplatePaneNodeData {
            control_id: "geometry.control".into(),
            frame: TemplateNodeFrameData {
                x: 40.0,
                y: 0.0,
                width: 24.0,
                height: 24.0,
            },
            ..TemplatePaneNodeData::default()
        }])));

    assert!(host.set_host_geometry_presentation(geometry, &[0]));
    let resized = host.get_host_presentation_generation();

    assert!(baseline.shares_structure_with(&resized));
    assert_eq!(
        resized.structure_generation(),
        baseline_structure_generation
    );
    assert!(resized.geometry_generation() > baseline_geometry_generation);
    assert!(resized.hit_test_generation() > baseline_hit_generation);
    assert_eq!(
        baseline.structure().host_layout.center_band_frame.width,
        0.0
    );
    assert_eq!(
        resized.structure().host_layout.center_band_frame.width,
        1600.0
    );
    assert_eq!(
        resized
            .structure()
            .host_scene_data
            .left_dock
            .region_frame
            .width,
        320.0
    );
    assert!(Arc::ptr_eq(
        &left_pane,
        resized
            .structure()
            .host_scene_data
            .left_dock
            .pane
            .body_surface_frame
            .as_ref()
            .expect("resized left surface frame")
    ));
    assert!(Arc::ptr_eq(
        &document_pane,
        resized
            .structure()
            .host_scene_data
            .document_dock
            .pane
            .body_surface_frame
            .as_ref()
            .expect("resized document surface frame")
    ));
    assert!(Arc::ptr_eq(
        &right_pane,
        resized
            .structure()
            .host_scene_data
            .right_dock
            .pane
            .body_surface_frame
            .as_ref()
            .expect("resized right surface frame")
    ));
    assert!(Arc::ptr_eq(
        &bottom_pane,
        resized
            .structure()
            .host_scene_data
            .bottom_dock
            .pane
            .body_surface_frame
            .as_ref()
            .expect("resized bottom surface frame")
    ));
}

#[test]
fn hover_updates_only_the_interaction_generation_and_skip_equal_values() {
    let host = UiHostWindow::new().expect("host window should construct for generation test");
    let baseline = host.get_host_presentation_generation();
    let frame = FrameRect {
        x: 12.0,
        y: 24.0,
        width: 96.0,
        height: 20.0,
    };

    host.set_hovered_template_node_for_pointer_move("toolbar.play", &frame);
    let hovered = host.get_host_presentation_generation();

    assert!(baseline.shares_structure_with(&hovered));
    assert_eq!(
        baseline.structure_generation(),
        hovered.structure_generation()
    );
    assert_eq!(
        baseline.hit_test_generation(),
        hovered.hit_test_generation()
    );
    assert!(hovered.interaction_generation() > baseline.interaction_generation());
    assert_eq!(
        hovered
            .materialize()
            .pane_interaction_state
            .hovered_template_control_id
            .as_str(),
        "toolbar.play"
    );

    host.set_hovered_template_node_for_pointer_move("toolbar.play", &frame);
    let repeated = host.get_host_presentation_generation();

    assert_eq!(
        repeated.interaction_generation(),
        hovered.interaction_generation()
    );
    assert!(hovered.shares_structure_with(&repeated));
}

#[test]
fn viewport_capture_advances_only_the_viewport_generation() {
    let host = UiHostWindow::new().expect("host window should construct for generation test");
    let baseline = host.get_host_presentation_generation();

    assert!(host
        .global::<PaneSurfaceHostContext>()
        .set_scene_viewport_capture(
            RenderViewportHandle::new(7),
            CapturedFrame::new(1, 1, vec![255, 0, 0, 255], 11),
        ));
    let captured = host.get_host_presentation_generation();

    assert!(baseline.shares_structure_with(&captured));
    assert!(captured.structure().viewport_images.scene().is_none());
    assert_eq!(
        baseline.structure_generation(),
        captured.structure_generation()
    );
    assert_eq!(
        baseline.interaction_generation(),
        captured.interaction_generation()
    );
    assert!(captured.viewport_generation() > baseline.viewport_generation());
    assert_eq!(
        captured
            .materialize()
            .viewport_images
            .scene()
            .expect("capture should materialize")
            .resource_key,
        "viewport:7:11"
    );
}
