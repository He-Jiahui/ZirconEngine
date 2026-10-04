use super::*;
use crate::ui::retained_host::primitives::ModelRc;

fn hierarchy_rows(count: usize) -> ModelRc<SceneNodeData> {
    ModelRc::with_metadata(
        (0..count)
            .map(|index| SceneNodeData {
                id: index.to_string().into(),
                name: format!("Entity {index}").into(),
                ..SceneNodeData::default()
            })
            .collect(),
        "hierarchy",
    )
}

#[test]
fn sparse_native_row_patch_reuses_unchanged_model_storage() {
    let rows = hierarchy_rows(10_000);
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.left_dock.pane.kind = "Hierarchy".into();
    presentation
        .host_scene_data
        .left_dock
        .pane
        .hierarchy
        .hierarchy_nodes = rows.clone();
    presentation.host_scene_data.right_dock.pane.kind = "Hierarchy".into();
    presentation
        .host_scene_data
        .right_dock
        .pane
        .hierarchy
        .hierarchy_nodes = rows.clone();

    assert!(patch_presented_hierarchy_rows(
        &mut presentation,
        &BTreeMap::from([(
            9_999,
            PresentedHierarchyRowPatch::new(
                Some(SceneNodeData {
                    id: "9999".into(),
                    name: "Renamed".into(),
                    selected: true,
                    ..SceneNodeData::default()
                }),
                true,
            ),
        )]),
    ));

    let patched = &presentation
        .host_scene_data
        .left_dock
        .pane
        .hierarchy
        .hierarchy_nodes;
    let mirrored = &presentation
        .host_scene_data
        .right_dock
        .pane
        .hierarchy
        .hierarchy_nodes;
    assert!(rows.shares_row_with(patched, 0));
    assert!(!rows.shares_row_with(patched, 9_999));
    assert_eq!(patched.get(9_999).unwrap().name.as_str(), "Renamed");
    assert!(patched.shares_values_with(mirrored));
}

#[test]
fn full_reflow_shares_one_native_generation_across_presented_hierarchy_panes() {
    let rows = hierarchy_rows(128);
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.left_dock.pane.kind = "Hierarchy".into();
    presentation.host_scene_data.right_dock.pane.kind = "Hierarchy".into();

    replace_presented_hierarchy_rows(&mut presentation, &rows);

    assert!(rows.shares_values_with(
        &presentation
            .host_scene_data
            .left_dock
            .pane
            .hierarchy
            .hierarchy_nodes
    ));
    assert!(rows.shares_values_with(
        &presentation
            .host_scene_data
            .right_dock
            .pane
            .hierarchy
            .hierarchy_nodes
    ));
}

#[test]
fn selection_only_patch_reuses_existing_row_content() {
    let rows = hierarchy_rows(128);
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.left_dock.pane.kind = "Hierarchy".into();
    presentation
        .host_scene_data
        .left_dock
        .pane
        .hierarchy
        .hierarchy_nodes = rows;

    assert!(patch_presented_hierarchy_rows(
        &mut presentation,
        &BTreeMap::from([(127, PresentedHierarchyRowPatch::new(None, true))]),
    ));

    let patched = presentation
        .host_scene_data
        .left_dock
        .pane
        .hierarchy
        .hierarchy_nodes
        .get(127)
        .cloned()
        .unwrap();
    assert_eq!(patched.id.as_str(), "127");
    assert_eq!(patched.name.as_str(), "Entity 127");
    assert_eq!(patched.depth, 0);
    assert!(patched.selected);
}

#[test]
fn sparse_patch_rejects_divergent_presented_generations() {
    let rows = hierarchy_rows(128);
    let mut presentation = HostWindowPresentationData::default();
    presentation.host_scene_data.left_dock.pane.kind = "Hierarchy".into();
    presentation
        .host_scene_data
        .left_dock
        .pane
        .hierarchy
        .hierarchy_nodes = rows.clone();
    presentation.host_scene_data.right_dock.pane.kind = "Hierarchy".into();
    presentation
        .host_scene_data
        .right_dock
        .pane
        .hierarchy
        .hierarchy_nodes = hierarchy_rows(128);

    assert!(!patch_presented_hierarchy_rows(
        &mut presentation,
        &BTreeMap::from([(
            100,
            PresentedHierarchyRowPatch::new(Some(SceneNodeData::default()), false),
        )]),
    ));
    assert!(rows.shares_values_with(
        &presentation
            .host_scene_data
            .left_dock
            .pane
            .hierarchy
            .hierarchy_nodes
    ));
}
