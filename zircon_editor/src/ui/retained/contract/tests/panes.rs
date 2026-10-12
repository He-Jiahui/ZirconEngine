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
fn hidden_unreal_content_owner_keeps_unpainted_side_docks_suppressed() {
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: ModelRc::from(Rc::new(VecModel::from(vec![
            TemplatePaneNodeData {
                control_id: "UnrealLevelEditorContent".into(),
                ..Default::default()
            },
        ]))),
        ..Default::default()
    };
    assert!(owns_ordinary_panes(&presentation));
    assert!(authored_panes(&presentation).is_empty());
}

fn unreal_level_viewport_fixture() -> (HostWindowPresentationData, HostDocumentDockSurfaceData) {
    let node = |id: &str, parent: &str, control: &str, x, y, width, height| TemplatePaneNodeData {
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
    };
    let presentation = HostWindowPresentationData {
        workbench_window_nodes: ModelRc::from(Rc::new(VecModel::from(vec![
            node("root", "", "WorkbenchWindowRoot", 0.0, 0.0, 1280.0, 800.0),
            node(
                "content",
                "root",
                "UnrealLevelEditorContent",
                0.0,
                74.0,
                1280.0,
                626.0,
            ),
            node(
                "viewport",
                "content",
                "WorkbenchMainBandViewportPanel",
                90.0,
                110.0,
                680.0,
                400.0,
            ),
            node(
                "viewport/body",
                "viewport",
                "ViewportBody",
                103.0,
                142.0,
                640.0,
                360.0,
            ),
            node(
                "viewport/media",
                "viewport/body",
                "WorkbenchViewportSurface",
                103.0,
                142.0,
                640.0,
                360.0,
            ),
        ]))),
        ..Default::default()
    };
    let mut document = HostDocumentDockSurfaceData::default();
    document.surface_key = "document:scene:authored".into();
    document.pane.kind = "Scene".into();
    document.region_frame = FrameRect {
        x: 50.0,
        y: 80.0,
        width: 760.0,
        height: 520.0,
    };
    // Deliberately different: native dock/header geometry must not become the media owner.
    document.content_frame = FrameRect {
        x: 17.0,
        y: 51.0,
        width: 100.0,
        height: 80.0,
    };
    (presentation, document)
}

#[test]
fn authored_level_viewport_uses_descendant_media_frame_for_scene_and_game() {
    let (presentation, mut document) = unreal_level_viewport_fixture();
    for kind in ["Scene", "Game"] {
        document.pane.kind = kind.into();
        let viewport =
            authored_level_viewport(&presentation, &document).expect("authored media owner");
        assert_eq!(
            viewport.root.control_id.as_str(),
            "WorkbenchMainBandViewportPanel"
        );
        assert_eq!(viewport.media.node_id.as_str(), "viewport/media");
        assert_eq!((viewport.frame.x, viewport.frame.y), (103.0, 142.0));
        assert_eq!(
            (viewport.frame.width, viewport.frame.height),
            (640.0, 360.0)
        );
        assert_eq!((viewport.clip.x, viewport.clip.y), (103.0, 142.0));
        assert_eq!((viewport.clip.width, viewport.clip.height), (640.0, 360.0));
        assert!(owns_authored_level_viewport(&presentation, &document));
    }
}

#[test]
fn authored_level_viewport_clips_media_to_authored_inherited_clip() {
    let (mut presentation, document) = unreal_level_viewport_fixture();
    let mut nodes = presentation
        .workbench_window_nodes
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    let media = nodes
        .iter_mut()
        .find(|node| node.control_id.as_str() == "WorkbenchViewportSurface")
        .unwrap();
    media.has_clip_frame = true;
    media.clip_frame = TemplateNodeFrameData {
        x: 111.0,
        y: 157.0,
        width: 301.0,
        height: 147.0,
    };
    presentation.workbench_window_nodes = ModelRc::from(Rc::new(VecModel::from(nodes)));
    let viewport = authored_level_viewport(&presentation, &document).unwrap();
    assert_eq!(
        (viewport.frame.width, viewport.frame.height),
        (640.0, 360.0)
    );
    assert_eq!((viewport.clip.x, viewport.clip.y), (111.0, 157.0));
    assert_eq!((viewport.clip.width, viewport.clip.height), (301.0, 147.0));

    // The DTO clip is already resolved by arrangement, including inherited ancestor clips.
    // A nonempty clip disjoint from the media must remove its authored viewport ownership.
    let mut nodes = presentation
        .workbench_window_nodes
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    nodes
        .iter_mut()
        .find(|node| node.control_id.as_str() == "WorkbenchViewportSurface")
        .unwrap()
        .clip_frame = TemplateNodeFrameData {
        x: 900.0,
        y: 700.0,
        width: 64.0,
        height: 32.0,
    };
    presentation.workbench_window_nodes = ModelRc::from(Rc::new(VecModel::from(nodes)));
    assert!(!owns_authored_level_viewport(&presentation, &document));
}

#[test]
fn authored_level_viewport_does_not_claim_neighbor_or_non_viewport_document() {
    let (presentation, mut document) = unreal_level_viewport_fixture();
    let mut neighbor = document.clone();
    neighbor.surface_key = "document:scene:neighbor".into();
    neighbor.region_frame.x = 780.0;
    assert!(!owns_authored_level_viewport(&presentation, &neighbor));
    for kind in ["ScriptEditor", "Inspector", "Welcome"] {
        document.pane.kind = kind.into();
        assert!(!owns_authored_level_viewport(&presentation, &document));
    }
    document.pane.kind = "Scene".into();
    document.surface_key = "".into();
    assert!(!owns_authored_level_viewport(&presentation, &document));
}

#[test]
fn authored_level_viewport_requires_complete_media_containment() {
    let (presentation, mut document) = unreal_level_viewport_fixture();
    document.region_frame = FrameRect {
        x: 103.0,
        y: 142.0,
        width: 639.0,
        height: 360.0,
    };
    assert!(!owns_authored_level_viewport(&presentation, &document));
    document.region_frame.width = 639.75;
    assert!(owns_authored_level_viewport(&presentation, &document));
}

#[test]
fn authored_level_viewport_rejects_unrelated_media_and_cyclic_ancestry() {
    let (mut presentation, document) = unreal_level_viewport_fixture();
    let mut nodes = presentation
        .workbench_window_nodes
        .iter()
        .cloned()
        .collect::<Vec<_>>();
    nodes
        .iter_mut()
        .find(|node| node.node_id.as_str() == "viewport/body")
        .unwrap()
        .parent_node_id = "viewport/media".into();
    presentation.workbench_window_nodes = ModelRc::from(Rc::new(VecModel::from(nodes)));
    assert!(!owns_authored_level_viewport(&presentation, &document));
}
