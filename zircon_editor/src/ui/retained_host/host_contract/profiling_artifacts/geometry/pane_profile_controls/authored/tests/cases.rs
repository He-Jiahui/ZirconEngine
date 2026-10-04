use super::*;
use crate::ui::retained_host::host_contract::data::{TemplateNodeFrameData, TemplatePaneNodeData};
use crate::ui::retained_host::primitives::{ModelRc, VecModel};
use std::rc::Rc;

fn node(
    id: &str,
    parent: &str,
    control: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: id.into(),
        parent_node_id: parent.into(),
        control_id: control.into(),
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..Default::default()
    }
}

#[test]
fn profile_uses_visible_authored_inspector_coordinates_and_clips_to_its_painted_root() {
    let mut input = node(
        "details/name",
        "details",
        "WorkbenchInspectorNameField",
        1062.0,
        432.0,
        176.0,
        28.0,
    );
    input.component_role = "input-field".into();
    let mut clipped = node(
        "details/x",
        "details",
        "WorkbenchInspectorPositionXField",
        1260.0,
        574.0,
        80.0,
        28.0,
    );
    clipped.component_role = "number-field".into();
    let mut old = node("old/name", "", "NameField", 1062.0, 175.0, 176.0, 28.0);
    old.component_role = "input-field".into();
    let mut presentation = HostWindowPresentationData {
        workbench_window_nodes: ModelRc::from(Rc::new(VecModel::from(vec![
            node(
                "workspace",
                "",
                "WorkbenchSceneWorkspace",
                0.0,
                135.0,
                1280.0,
                637.0,
            ),
            node(
                "details",
                "workspace",
                "WorkbenchMainBandInspectorPanel",
                1054.0,
                389.0,
                226.0,
                383.0,
            ),
            input,
            clipped,
            old,
        ]))),
        ..Default::default()
    };
    presentation.host_scene_data.right_dock.region_frame = FrameRect {
        x: 1054.0,
        y: 135.0,
        width: 226.0,
        height: 637.0,
    };
    presentation.host_scene_data.right_dock.content_frame = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 226.0,
        height: 637.0,
    };
    presentation.host_scene_data.right_dock.pane.kind = "Inspector".into();
    presentation.host_scene_data.right_dock.pane.inspector.nodes =
        ModelRc::from(Rc::new(VecModel::from(vec![TemplatePaneNodeData {
            component_role: "input-field".into(),
            ..node("legacy/name", "", "NameField", 8.0, 40.0, 176.0, 28.0)
        }])));
    let controls = super::super::collect_pane_profile_controls(&presentation).template_controls;
    assert_eq!(controls.len(), 2);
    let name = controls
        .iter()
        .find(|c| c.id == "template.right.WorkbenchInspectorNameField")
        .unwrap();
    assert_eq!(name.frame.y, 432.0);
    let x = controls
        .iter()
        .find(|c| c.id == "template.right.WorkbenchInspectorPositionXField")
        .unwrap();
    assert_eq!(x.frame.y, 574.0);
    assert_eq!(x.frame.width, 20.0);
    assert!(!controls.iter().any(|c| c.id.ends_with(".NameField")));
}

#[test]
fn hidden_authored_owner_never_falls_back_to_the_unpainted_old_side_controls() {
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: ModelRc::from(Rc::new(VecModel::from(vec![
            node(
                "workspace",
                "",
                "WorkbenchSceneWorkspace",
                0.0,
                0.0,
                0.0,
                0.0,
            ),
            node(
                "details",
                "workspace",
                "WorkbenchMainBandInspectorPanel",
                0.0,
                0.0,
                0.0,
                0.0,
            ),
        ]))),
        ..Default::default()
    };
    assert!(super::super::collect_pane_profile_controls(&presentation)
        .template_controls
        .is_empty());
}
