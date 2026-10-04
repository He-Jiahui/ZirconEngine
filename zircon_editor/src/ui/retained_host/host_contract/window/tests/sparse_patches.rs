use super::super::UiHostWindow;
use super::support::ui_asset_pane;
use crate::ui::retained_host::host_contract::data::{
    HostPanePresentationLocation, HostPanePresentationPatch, HostPresentationPatch,
    HostWindowPresentationData, TemplatePaneNodeData,
};
use crate::ui::retained_host::primitives::{ModelRc, VecModel};
use std::rc::Rc;

#[test]
fn sparse_presentation_patch_rebinds_workbench_and_pane_models_in_one_generation() {
    let host = UiHostWindow::new().expect("host window should construct for sparse patch test");
    let previous_workbench = ModelRc::from(Rc::new(VecModel::from(vec![TemplatePaneNodeData {
        node_id: "workbench.row".into(),
        control_id: "workbench.row".into(),
        text: "Before".into(),
        ..TemplatePaneNodeData::default()
    }])));
    let previous_pane_nodes = ModelRc::from(Rc::new(VecModel::from(vec![TemplatePaneNodeData {
        control_id: "document.row".into(),
        text: "Before".into(),
        ..TemplatePaneNodeData::default()
    }])));
    let mut presentation = HostWindowPresentationData::default();
    presentation.workbench_window_nodes = previous_workbench.clone();
    presentation.host_scene_data.document_dock.pane =
        ui_asset_pane("document-pane", previous_pane_nodes.clone());
    host.set_host_presentation(presentation);
    let before = host.get_host_presentation_generation();
    let before_structure_generation = before.structure_generation();
    let before_geometry_generation = before.geometry_generation();
    let before_hit_generation = before.hit_test_generation();
    drop(before);

    let next_workbench = previous_workbench.with_row_patches(std::collections::BTreeMap::from([(
        0,
        TemplatePaneNodeData {
            text: "After".into(),
            ..previous_workbench.get(0).expect("workbench row").clone()
        },
    )]));
    let next_pane_nodes =
        previous_pane_nodes.with_row_patches(std::collections::BTreeMap::from([(
            0,
            TemplatePaneNodeData {
                text: "After".into(),
                ..previous_pane_nodes.get(0).expect("pane row").clone()
            },
        )]));

    assert!(host
        .patch_host_presentation(|current| {
            let previous = &current.host_scene_data.document_dock.pane;
            let mut next = previous.clone();
            next.ui_asset.nodes = next_pane_nodes.clone();
            let patch = HostPresentationPatch::new(HostPanePresentationPatch::single(
                HostPanePresentationLocation::DocumentDock,
                previous,
                next,
            ))
            .with_workbench_nodes(next_workbench.clone(), vec![0]);
            Some((patch, ()))
        })
        .is_some());

    let after = host.get_host_presentation_generation();
    assert_eq!(
        after.structure_generation(),
        before_structure_generation + 1
    );
    assert_eq!(after.geometry_generation(), before_geometry_generation + 1);
    assert_eq!(after.hit_test_generation(), before_hit_generation + 1);
    assert!(after
        .structure()
        .workbench_window_nodes
        .shares_values_with(&next_workbench));
    assert!(after
        .structure()
        .host_scene_data
        .document_dock
        .pane
        .ui_asset
        .nodes
        .shares_values_with(&next_pane_nodes));
    assert!(after.workbench_hit_index().indexes_nodes(&next_workbench));
    assert!(after
        .workbench_hit_index()
        .indexes_paint_nodes(&next_pane_nodes));
    assert!(after
        .workbench_hit_index()
        .indexes_presentation(after.structure()));
}

#[test]
fn sparse_presentation_patch_is_atomic_when_workbench_rebind_is_invalid() {
    let host = UiHostWindow::new().expect("host window should construct for sparse patch test");
    let previous_workbench = ModelRc::from(Rc::new(VecModel::from(vec![TemplatePaneNodeData {
        node_id: "workbench.row".into(),
        control_id: "workbench.row".into(),
        ..TemplatePaneNodeData::default()
    }])));
    let previous_pane_nodes = ModelRc::from(Rc::new(VecModel::from(vec![TemplatePaneNodeData {
        control_id: "document.row".into(),
        ..TemplatePaneNodeData::default()
    }])));
    let mut presentation = HostWindowPresentationData::default();
    presentation.workbench_window_nodes = previous_workbench.clone();
    presentation.host_scene_data.document_dock.pane =
        ui_asset_pane("document-pane", previous_pane_nodes.clone());
    host.set_host_presentation(presentation);
    let before = host.get_host_presentation_generation();
    let before_structure_generation = before.structure_generation();
    let before_hit_generation = before.hit_test_generation();
    drop(before);

    let invalid_workbench = ModelRc::from(Rc::new(VecModel::from(vec![TemplatePaneNodeData {
        node_id: "different.row".into(),
        control_id: "different.row".into(),
        ..TemplatePaneNodeData::default()
    }])));
    let next_pane_nodes = ModelRc::from(Rc::new(VecModel::from(vec![TemplatePaneNodeData {
        control_id: "document.next".into(),
        ..TemplatePaneNodeData::default()
    }])));

    assert!(host
        .patch_host_presentation(|current| {
            let previous = &current.host_scene_data.document_dock.pane;
            let mut next = previous.clone();
            next.ui_asset.nodes = next_pane_nodes;
            let patch = HostPresentationPatch::new(HostPanePresentationPatch::single(
                HostPanePresentationLocation::DocumentDock,
                previous,
                next,
            ))
            .with_workbench_nodes(invalid_workbench, vec![0]);
            Some((patch, ()))
        })
        .is_none());

    let after = host.get_host_presentation_generation();
    assert_eq!(after.structure_generation(), before_structure_generation);
    assert_eq!(after.hit_test_generation(), before_hit_generation);
    assert!(after
        .structure()
        .workbench_window_nodes
        .shares_values_with(&previous_workbench));
    assert!(after
        .structure()
        .host_scene_data
        .document_dock
        .pane
        .ui_asset
        .nodes
        .shares_values_with(&previous_pane_nodes));
}
