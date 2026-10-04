use super::super::UiHostWindow;
use super::support::ui_asset_pane;
use crate::ui::retained_host::host_contract::data::{
    HostPanePresentationLocation, HostPanePresentationPatch, HostWindowPresentationData,
    TemplatePaneNodeData,
};
use crate::ui::retained_host::primitives::{ModelRc, VecModel};
use std::rc::Rc;

#[test]
fn pane_patch_rebinds_only_the_changed_paint_model_in_one_atomic_publish() {
    let host = UiHostWindow::new().expect("host window should construct for pane patch test");
    let previous_nodes = ModelRc::from(Rc::new(VecModel::from(vec![TemplatePaneNodeData {
        control_id: "document.previous".into(),
        ..TemplatePaneNodeData::default()
    }])));
    let next_nodes = ModelRc::from(Rc::new(VecModel::from(vec![TemplatePaneNodeData {
        control_id: "document.next".into(),
        ..TemplatePaneNodeData::default()
    }])));
    let untouched_nodes = ModelRc::from(Rc::new(VecModel::from(vec![TemplatePaneNodeData {
        control_id: "left.untouched".into(),
        ..TemplatePaneNodeData::default()
    }])));
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.document_dock.pane =
        ui_asset_pane("document-pane", previous_nodes);
    presentation.host_scene_data.left_dock.pane =
        ui_asset_pane("left-pane", untouched_nodes.clone());
    host.set_host_presentation(presentation);
    let before = host.get_host_presentation_generation();
    let before_hit_generation = before.hit_test_generation();
    drop(before);

    assert!(host
        .patch_host_presentation_panes(|current| {
            let previous = &current.host_scene_data.document_dock.pane;
            let mut next = previous.clone();
            next.ui_asset.nodes = next_nodes.clone();
            Some((
                HostPanePresentationPatch::single(
                    HostPanePresentationLocation::DocumentDock,
                    previous,
                    next,
                ),
                (),
            ))
        })
        .is_some());

    let after = host.get_host_presentation_generation();
    assert!(after.hit_test_generation() > before_hit_generation);
    assert!(after
        .structure()
        .host_scene_data
        .document_dock
        .pane
        .ui_asset
        .nodes
        .shares_values_with(&next_nodes));
    assert!(after
        .structure()
        .host_scene_data
        .left_dock
        .pane
        .ui_asset
        .nodes
        .shares_values_with(&untouched_nodes));
    assert!(after.workbench_hit_index().indexes_paint_nodes(&next_nodes));
    assert!(after
        .workbench_hit_index()
        .indexes_paint_nodes(&untouched_nodes));
    assert!(after
        .workbench_hit_index()
        .indexes_presentation(after.structure()));
}

#[test]
fn pane_patch_rejects_a_changed_stable_identity_without_mutating_the_presentation() {
    let host = UiHostWindow::new().expect("host window should construct for pane patch test");
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.document_dock.pane =
        ui_asset_pane("current-pane", ModelRc::default());
    host.set_host_presentation(presentation);
    let before = host.get_host_presentation_generation();
    let before_generation = before.structure_generation();
    drop(before);

    assert!(host
        .patch_host_presentation_panes(|current| {
            let previous = &current.host_scene_data.document_dock.pane;
            let mut next = previous.clone();
            next.id = "different-pane".into();
            Some((
                HostPanePresentationPatch::single(
                    HostPanePresentationLocation::DocumentDock,
                    previous,
                    next,
                ),
                (),
            ))
        })
        .is_none());

    let after = host.get_host_presentation_generation();
    assert_eq!(after.structure_generation(), before_generation);
    assert_eq!(
        after
            .structure()
            .host_scene_data
            .document_dock
            .pane
            .id
            .as_str(),
        "current-pane"
    );
}

#[test]
fn pane_patch_updates_only_the_selected_persistent_floating_row() {
    let host = UiHostWindow::new().expect("host window should construct for pane patch test");
    let first_nodes = ModelRc::from(Rc::new(VecModel::from(vec![TemplatePaneNodeData {
        control_id: "floating.first".into(),
        ..TemplatePaneNodeData::default()
    }])));
    let second_nodes = ModelRc::from(Rc::new(VecModel::from(vec![TemplatePaneNodeData {
        control_id: "floating.second".into(),
        ..TemplatePaneNodeData::default()
    }])));
    let next_nodes = ModelRc::from(Rc::new(VecModel::from(vec![TemplatePaneNodeData {
        control_id: "floating.first.next".into(),
        ..TemplatePaneNodeData::default()
    }])));
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.floating_layer.floating_windows =
        ModelRc::from(Rc::new(VecModel::from(vec![
            crate::ui::retained_host::host_contract::FloatingWindowData {
                window_id: "floating-a".into(),
                active_pane: ui_asset_pane("pane-a", first_nodes),
                ..Default::default()
            },
            crate::ui::retained_host::host_contract::FloatingWindowData {
                window_id: "floating-b".into(),
                active_pane: ui_asset_pane("pane-b", second_nodes.clone()),
                ..Default::default()
            },
        ])));
    host.set_host_presentation(presentation);

    assert!(host
        .patch_host_presentation_panes(|current| {
            let window = current
                .host_scene_data
                .floating_layer
                .floating_windows
                .get(0)
                .expect("first floating row");
            let mut next = window.active_pane.clone();
            next.ui_asset.nodes = next_nodes.clone();
            Some((
                HostPanePresentationPatch::single(
                    HostPanePresentationLocation::Floating {
                        row: 0,
                        window_id: window.window_id.clone(),
                    },
                    &window.active_pane,
                    next,
                ),
                (),
            ))
        })
        .is_some());

    let after = host.get_host_presentation_generation();
    let windows = &after
        .structure()
        .host_scene_data
        .floating_layer
        .floating_windows;
    assert!(windows
        .get(0)
        .expect("first floating row")
        .active_pane
        .ui_asset
        .nodes
        .shares_values_with(&next_nodes));
    assert!(windows
        .get(1)
        .expect("second floating row")
        .active_pane
        .ui_asset
        .nodes
        .shares_values_with(&second_nodes));
}

#[test]
fn pane_patch_rejects_splitting_a_model_shared_by_an_unpatched_pane() {
    let host = UiHostWindow::new().expect("host window should construct for pane patch test");
    let shared_nodes = ModelRc::from(Rc::new(VecModel::from(vec![TemplatePaneNodeData {
        control_id: "shared.model".into(),
        ..TemplatePaneNodeData::default()
    }])));
    let next_nodes = ModelRc::from(Rc::new(VecModel::from(vec![TemplatePaneNodeData {
        control_id: "document.next".into(),
        ..TemplatePaneNodeData::default()
    }])));
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.document_dock.pane =
        ui_asset_pane("document-pane", shared_nodes.clone());
    presentation.host_scene_data.left_dock.pane = ui_asset_pane("left-pane", shared_nodes.clone());
    host.set_host_presentation(presentation);
    let before = host.get_host_presentation_generation();
    let before_generation = before.structure_generation();
    drop(before);

    assert!(host
        .patch_host_presentation_panes(|current| {
            let previous = &current.host_scene_data.document_dock.pane;
            let mut next = previous.clone();
            next.ui_asset.nodes = next_nodes;
            Some((
                HostPanePresentationPatch::single(
                    HostPanePresentationLocation::DocumentDock,
                    previous,
                    next,
                ),
                (),
            ))
        })
        .is_none());

    let after = host.get_host_presentation_generation();
    assert_eq!(after.structure_generation(), before_generation);
    assert!(after
        .structure()
        .host_scene_data
        .document_dock
        .pane
        .ui_asset
        .nodes
        .shares_values_with(&shared_nodes));
}

#[test]
fn pane_patch_rejects_merging_into_a_model_owned_by_an_unpatched_pane() {
    let host = UiHostWindow::new().expect("host window should construct for pane patch test");
    let previous_nodes = ModelRc::from(Rc::new(VecModel::from(vec![TemplatePaneNodeData {
        control_id: "document.previous".into(),
        ..TemplatePaneNodeData::default()
    }])));
    let occupied_nodes = ModelRc::from(Rc::new(VecModel::from(vec![TemplatePaneNodeData {
        control_id: "left.occupied".into(),
        ..TemplatePaneNodeData::default()
    }])));
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.document_dock.pane =
        ui_asset_pane("document-pane", previous_nodes.clone());
    presentation.host_scene_data.left_dock.pane =
        ui_asset_pane("left-pane", occupied_nodes.clone());
    host.set_host_presentation(presentation);
    let before = host.get_host_presentation_generation();
    let before_generation = before.structure_generation();
    drop(before);

    assert!(host
        .patch_host_presentation_panes(|current| {
            let previous = &current.host_scene_data.document_dock.pane;
            let mut next = previous.clone();
            next.ui_asset.nodes = occupied_nodes.clone();
            Some((
                HostPanePresentationPatch::single(
                    HostPanePresentationLocation::DocumentDock,
                    previous,
                    next,
                ),
                (),
            ))
        })
        .is_none());

    let after = host.get_host_presentation_generation();
    assert_eq!(after.structure_generation(), before_generation);
    assert!(after
        .structure()
        .host_scene_data
        .document_dock
        .pane
        .ui_asset
        .nodes
        .shares_values_with(&previous_nodes));
}
