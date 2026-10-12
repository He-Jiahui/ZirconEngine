use super::super::UiHostWindow;
use crate::ui::retained_host::host_contract::data::{
    HostDockPresentationPatch, HostWindowPresentationData, TemplatePaneNodeData,
};
use crate::ui::retained_host::primitives::{ModelRc, VecModel};
use std::rc::Rc;

#[test]
fn dock_patch_mutates_the_unshared_structure_without_materializing_the_window() {
    let host = UiHostWindow::new().expect("host window should construct for dock patch test");
    let previous_nodes = ModelRc::from(Rc::new(VecModel::from(vec![TemplatePaneNodeData {
        control_id: "left.previous".into(),
        ..TemplatePaneNodeData::default()
    }])));
    let next_nodes = ModelRc::from(Rc::new(VecModel::from(vec![TemplatePaneNodeData {
        control_id: "left.next".into(),
        ..TemplatePaneNodeData::default()
    }])));
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.left_dock.rail_nodes = previous_nodes.clone();
    host.set_host_presentation(presentation);

    let initial = host.get_host_presentation_generation();
    let structure_address = initial.structure() as *const HostWindowPresentationData;
    let expected_generation = initial.structure_generation();
    let mut next_dock = initial.structure().host_scene_data.left_dock.clone();
    next_dock.rail_nodes = next_nodes.clone();
    drop(initial);

    assert!(host.patch_host_presentation_dock(
        expected_generation,
        Default::default(),
        Default::default(),
        HostDockPresentationPatch::Left(next_dock),
        &[(previous_nodes, next_nodes.clone())],
    ));

    let patched = host.get_host_presentation_generation();
    assert_eq!(
        patched.structure() as *const HostWindowPresentationData,
        structure_address,
        "an unshared retained presentation must be patched in place"
    );
    assert!(patched
        .structure()
        .host_scene_data
        .left_dock
        .rail_nodes
        .shares_values_with(&next_nodes));
}

#[test]
fn dock_patch_preserves_an_outstanding_generation_snapshot() {
    let host = UiHostWindow::new().expect("host window should construct for dock patch test");
    let previous_nodes = ModelRc::from(Rc::new(VecModel::from(vec![TemplatePaneNodeData {
        control_id: "left.previous".into(),
        ..TemplatePaneNodeData::default()
    }])));
    let next_nodes = ModelRc::from(Rc::new(VecModel::from(vec![TemplatePaneNodeData {
        control_id: "left.next".into(),
        ..TemplatePaneNodeData::default()
    }])));
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.left_dock.rail_nodes = previous_nodes.clone();
    host.set_host_presentation(presentation);

    let outstanding = host.get_host_presentation_generation();
    let old_address = outstanding.structure() as *const HostWindowPresentationData;
    let mut next_dock = outstanding.structure().host_scene_data.left_dock.clone();
    next_dock.rail_nodes = next_nodes.clone();

    assert!(host.patch_host_presentation_dock(
        outstanding.structure_generation(),
        Default::default(),
        Default::default(),
        HostDockPresentationPatch::Left(next_dock),
        &[(previous_nodes.clone(), next_nodes.clone())],
    ));

    assert!(outstanding
        .structure()
        .host_scene_data
        .left_dock
        .rail_nodes
        .shares_values_with(&previous_nodes));
    let patched = host.get_host_presentation_generation();
    assert_ne!(
        patched.structure() as *const HostWindowPresentationData,
        old_address,
        "an outstanding immutable generation requires copy-on-write"
    );
    assert!(patched
        .structure()
        .host_scene_data
        .left_dock
        .rail_nodes
        .shares_values_with(&next_nodes));
}
