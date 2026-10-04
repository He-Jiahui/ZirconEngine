use super::*;
use crate::ui::retained_host::host_contract::data::SceneNodeData;
use crate::ui::retained_host::primitives::{ModelRc, VecModel};
use std::rc::Rc;

#[test]
fn hidden_authored_panes_do_not_report_retired_native_dock_frames() {
    let mut presentation = HostWindowPresentationData::default();
    let scene = &mut presentation.host_scene_data;
    for dock in [&mut scene.left_dock, &mut scene.right_dock] {
        dock.pane.kind = "Hierarchy".into();
        dock.region_frame = FrameRect {
            x: 12.0,
            y: 24.0,
            width: 240.0,
            height: 320.0,
        };
        dock.content_frame = FrameRect {
            x: 0.0,
            y: 0.0,
            width: 240.0,
            height: 320.0,
        };
        dock.pane.hierarchy.hierarchy_nodes =
            ModelRc::from(Rc::new(VecModel::from(vec![SceneNodeData {
                id: "entity-25".into(),
                name: "Actual selected entity".into(),
                selected: true,
                ..Default::default()
            }])));
    }
    // The authored workspace owns these panes, but responsive layout has hidden them.
    presentation.workbench_window_nodes = ModelRc::from(Rc::new(VecModel::from(vec![
        crate::ui::retained_host::host_contract::data::TemplatePaneNodeData {
            control_id: "WorkbenchSceneWorkspace".into(),
            ..Default::default()
        },
    ])));
    assert!(crate::ui::retained_host::host_contract::componentized_workbench_regions::owns_ordinary_panes(&presentation));
    assert!(crate::ui::retained_host::host_contract::componentized_workbench_regions::authored_hierarchy(&presentation).is_none());
    assert!(
        collect_native_hierarchy_profiles(&presentation).is_empty(),
        "unpainted old side docks must not publish ghost native frames"
    );
}

#[test]
fn profile_joins_authoritative_rows_to_painted_frames_and_scroll() {
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.left_dock.region_frame = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 180.0,
        height: 120.0,
    };
    presentation.host_scene_data.left_dock.content_frame = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 180.0,
        height: 120.0,
    };
    presentation.host_scene_data.left_dock.pane.kind = "Hierarchy".into();
    presentation
        .host_scene_data
        .left_dock
        .pane
        .hierarchy
        .hierarchy_nodes = ModelRc::from(Rc::new(VecModel::from(vec![
        SceneNodeData {
            id: "sun".into(),
            name: "Sun".into(),
            selected: true,
            ..Default::default()
        },
        SceneNodeData {
            id: "child".into(),
            name: "Cube".into(),
            depth: 1,
            ..Default::default()
        },
    ])));
    presentation.pane_interaction_state.hierarchy_scroll_px = 7.0;

    let profiles = collect_native_hierarchy_profiles(&presentation);
    let profile = profiles.first().expect("hierarchy profile");
    assert_eq!(profile.scroll_offset, 7.0);
    assert_eq!(profile.selected_node_ids, vec!["sun"]);
    assert_eq!(profile.selected_names, vec!["Sun"]);
    assert_eq!(profile.rows[1].name, "Cube");
    assert_eq!(profile.rows[1].depth, 1);
    assert!(
        profile.rows[0].frame.as_ref().is_some_and(
            |frame| frame.y < profile.rows[1].frame.as_ref().expect("second row frame").y
        )
    );
}

#[test]
fn profile_joins_active_inline_rename_focus_to_the_authoritative_node() {
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.left_dock.region_frame = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 180.0,
        height: 120.0,
    };
    presentation.host_scene_data.left_dock.content_frame = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 180.0,
        height: 120.0,
    };
    presentation.host_scene_data.left_dock.pane.kind = "Hierarchy".into();
    presentation
        .host_scene_data
        .left_dock
        .pane
        .hierarchy
        .hierarchy_nodes = ModelRc::from(Rc::new(VecModel::from(vec![SceneNodeData {
        id: "sun".into(),
        name: "Sun".into(),
        selected: true,
        ..Default::default()
    }])));
    presentation.text_input_focus.control_id = HIERARCHY_INLINE_RENAME_CONTROL_ID.into();
    presentation.text_input_focus.dispatch_kind = "hierarchy_inline_rename:sun".into();
    presentation.text_input_focus.value_text = "太阳".into();

    let profile = collect_native_hierarchy_profiles(&presentation).remove(0);
    let rename = profile.inline_rename.expect("inline rename receipt");
    assert_eq!(rename.node_id, "sun");
    assert_eq!(rename.value_text, "太阳");
}
