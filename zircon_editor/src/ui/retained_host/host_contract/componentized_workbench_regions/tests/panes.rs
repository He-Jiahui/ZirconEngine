use super::super::super::data::{SceneNodeData, TemplateNodeFrameData};
use super::*;
use crate::ui::retained_host::primitives::{ModelRc, VecModel};
use std::rc::Rc;

#[test]
fn native_hierarchy_uses_relocated_authored_viewport_and_row_pitch() {
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.left_dock.pane.kind = "Hierarchy".into();
    presentation
        .host_scene_data
        .left_dock
        .pane
        .hierarchy
        .hierarchy_nodes = ModelRc::from(Rc::new(VecModel::from(
        (0..40)
            .map(|index| SceneNodeData {
                id: index.to_string().into(),
                name: format!("Entity {index}").into(),
                selected: index == 25,
                ..Default::default()
            })
            .collect::<Vec<_>>(),
    )));
    let make = |control: &str, x, y, width, height| TemplatePaneNodeData {
        control_id: control.into(),
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..Default::default()
    };
    presentation.workbench_window_nodes = ModelRc::from(Rc::new(VecModel::from(vec![
        make("WorkbenchSceneTree", 960.0, 150.0, 320.0, 250.0),
        make("WorkbenchSceneRootItem", 960.0, 150.0, 320.0, 22.0),
        make("WorkbenchSceneEnvironmentItem", 960.0, 176.0, 320.0, 22.0),
    ])));
    let authored = authored_hierarchy(&presentation).expect("authored native hierarchy");
    assert_eq!(authored.viewport.x, 960.0);
    assert_eq!(authored.metrics.row_height, 22.0);
    assert_eq!(authored.metrics.row_gap, 4.0);
    assert_eq!(authored.pane.hierarchy.hierarchy_nodes.row_count(), 40);
    assert!(
        authored
            .pane
            .hierarchy
            .hierarchy_nodes
            .get(25)
            .unwrap()
            .selected
    );
}

#[test]
fn hidden_authored_scene_workspace_keeps_the_old_dock_owner_suppressed() {
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: ModelRc::from(Rc::new(VecModel::from(vec![
            TemplatePaneNodeData {
                control_id: "WorkbenchSceneWorkspace".into(),
                ..Default::default()
            },
        ]))),
        ..Default::default()
    };
    assert!(owns_ordinary_panes(&presentation));
    assert!(authored_panes(&presentation).is_empty());
}
