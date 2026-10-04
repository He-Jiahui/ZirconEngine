use std::sync::Arc;

use crate::tests::support::env_lock;
use crate::ui::retained_host::callback_dispatch::BuiltinViewportToolbarTemplateBridge;
use crate::ui::retained_host::{HostWindowPresentationData, UiHostWindow};

use super::attach_viewport_toolbar_surface_frames_to_ui;

#[test]
fn stable_toolbar_publication_does_not_advance_presentation_generations() {
    let _guard = env_lock().lock().unwrap();
    let host = UiHostWindow::new().expect("host window should construct for toolbar test");
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.document_dock.pane.kind = "Scene".into();
    presentation.host_scene_data.document_dock.pane.show_toolbar = true;
    host.set_host_presentation(presentation);
    let mut bridge =
        BuiltinViewportToolbarTemplateBridge::new().expect("viewport toolbar template should load");

    attach_viewport_toolbar_surface_frames_to_ui(&host, &mut bridge, Some(1_280.0));
    let first = host.get_host_presentation_generation();
    let first_frame = first
        .structure()
        .host_scene_data
        .document_dock
        .pane
        .viewport
        .toolbar_surface_frame
        .clone()
        .expect("first publication should attach the toolbar frame");
    let first_structure_generation = first.structure_generation();
    let first_geometry_generation = first.geometry_generation();
    let first_hit_test_generation = first.hit_test_generation();
    drop(first);

    attach_viewport_toolbar_surface_frames_to_ui(&host, &mut bridge, Some(1_280.0));
    let stable = host.get_host_presentation_generation();
    let stable_frame = stable
        .structure()
        .host_scene_data
        .document_dock
        .pane
        .viewport
        .toolbar_surface_frame
        .as_ref()
        .expect("stable publication should retain the toolbar frame");

    assert!(Arc::ptr_eq(&first_frame, stable_frame));
    assert_eq!(stable.structure_generation(), first_structure_generation);
    assert_eq!(stable.geometry_generation(), first_geometry_generation);
    assert_eq!(stable.hit_test_generation(), first_hit_test_generation);
}

#[test]
fn astra_leaf_toolbars_publish_independent_widths_and_reuse_stable_generation() {
    let _guard = env_lock().lock().unwrap();
    let host = UiHostWindow::new().expect("toolbar test host");
    let mut presentation = HostWindowPresentationData::default();
    let leaf = |key: &str, width| {
        let mut dock =
            crate::ui::retained_host::host_contract::HostDocumentDockSurfaceData::default();
        dock.surface_key = key.into();
        dock.content_frame.width = width;
        dock.content_frame.height = 300.0;
        dock.pane.id = key.into();
        dock.pane.kind = "Scene".into();
        dock.pane.show_toolbar = true;
        dock
    };
    presentation.host_scene_data.document_leaves = vec![
        leaf("document:first", 450.0),
        leaf("document:second", 750.0),
    ];
    host.set_host_presentation(presentation);
    let mut bridge = BuiltinViewportToolbarTemplateBridge::new().unwrap();
    attach_viewport_toolbar_surface_frames_to_ui(&host, &mut bridge, None);
    let first = host.get_host_presentation_generation();
    let frames: Vec<_> = first
        .structure()
        .host_scene_data
        .document_leaves
        .iter()
        .map(|leaf| {
            leaf.pane
                .viewport
                .toolbar_surface_frame
                .clone()
                .expect("leaf toolbar")
        })
        .collect();
    assert_eq!(frames.len(), 2);
    assert!(!Arc::ptr_eq(&frames[0], &frames[1]));
    let generation = first.structure_generation();
    drop(first);
    attach_viewport_toolbar_surface_frames_to_ui(&host, &mut bridge, None);
    let stable = host.get_host_presentation_generation();
    assert_eq!(stable.structure_generation(), generation);
    for (leaf, expected) in stable
        .structure()
        .host_scene_data
        .document_leaves
        .iter()
        .zip(frames)
    {
        assert!(Arc::ptr_eq(
            leaf.pane.viewport.toolbar_surface_frame.as_ref().unwrap(),
            &expected
        ));
    }
}
