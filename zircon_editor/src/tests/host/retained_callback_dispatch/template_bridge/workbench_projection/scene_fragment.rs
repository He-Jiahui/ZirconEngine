use zircon_runtime::scene::WorldInspectionHierarchyRow;
use zircon_runtime_interface::ui::layout::UiSize;

use crate::core::editor_message::{
    SceneInspectionFieldsDelta, SceneInspectionHierarchyAnchor, SceneInspectionMessage,
    SceneInspectionSelectionDelta,
};
use crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use crate::ui::workbench::snapshot::{SceneEntries, SceneInspectionHierarchyFragment};

use super::{control_bool, control_integer, control_string, control_visibility, env_lock};

#[test]
fn hierarchy_patch_updates_only_its_runtime_rows_at_the_expected_generation() {
    let _guard = env_lock().lock().unwrap();
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    bridge
        .sync_scene_and_inspector(
            &hierarchy_entries(
                7,
                vec![
                    hierarchy_row(1, None, 0, "World", 11),
                    hierarchy_row(2, Some(1), 1, "Camera", 12),
                    hierarchy_row(3, Some(1), 1, "Light", 13),
                ],
            ),
            None,
        )
        .unwrap();

    let changed_rows = vec![
        hierarchy_row(1, None, 0, "World", 99),
        hierarchy_row(2, Some(1), 1, "Gameplay Camera", 42),
    ];
    let fragment = SceneInspectionHierarchyFragment::patch(
        delta(
            7,
            8,
            vec![anchor(1, None, 0, 99), anchor(2, Some(1), 1, 42)],
            SceneInspectionSelectionDelta::unchanged(),
        ),
        changed_rows,
    )
    .unwrap();

    assert!(fragment.changed_rows().is_some());
    assert!(fragment.reflow_entries().is_none());
    let applied = bridge.apply_scene_hierarchy_fragment(&fragment).unwrap();

    assert!(applied.applied());
    assert_eq!(applied.updated_rows(), 2);
    assert!(!applied.reflowed());
    assert_eq!(
        applied.changed_control_ids(),
        &[
            "WorkbenchSceneEnvironmentItem".to_string(),
            "WorkbenchSceneRootItem".to_string(),
        ]
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchSceneEnvironmentItem", "text").as_deref(),
        Some("Gameplay Camera")
    );
    assert_eq!(
        control_string(&bridge, "WorkbenchSceneEnvironmentItem", "scene_parent_id").as_deref(),
        Some("1")
    );
    assert_eq!(
        control_string(
            &bridge,
            "WorkbenchSceneEnvironmentItem",
            "scene_subtree_hash"
        )
        .as_deref(),
        Some("42")
    );
}

#[test]
fn hierarchy_reflow_is_explicit_for_a_structural_change() {
    let _guard = env_lock().lock().unwrap();
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    bridge
        .sync_scene_and_inspector(
            &hierarchy_entries(
                20,
                vec![
                    hierarchy_row(1, None, 0, "World", 21),
                    hierarchy_row(2, Some(1), 1, "Group", 22),
                    hierarchy_row(3, Some(2), 2, "Camera", 23),
                    hierarchy_row(4, Some(1), 1, "Light", 24),
                ],
            ),
            None,
        )
        .unwrap();

    let next_entries = hierarchy_entries(
        21,
        vec![
            hierarchy_row(1, None, 0, "World", 31),
            hierarchy_row(2, Some(1), 1, "Group", 32),
            hierarchy_row(4, Some(1), 1, "Light", 24),
        ],
    );
    let fragment = SceneInspectionHierarchyFragment::reflow(
        delta(
            20,
            21,
            vec![anchor(1, None, 0, 31), anchor(2, Some(1), 1, 32)],
            SceneInspectionSelectionDelta::unchanged(),
        ),
        next_entries.clone(),
    )
    .unwrap();

    assert!(!bridge
        .apply_scene_hierarchy_fragment(&fragment)
        .unwrap()
        .applied());
    bridge.resync_scene_hierarchy(&next_entries).unwrap();

    assert_eq!(
        control_string(&bridge, "WorkbenchSceneLevelItem", "text").as_deref(),
        Some("Light")
    );
    assert_eq!(
        control_integer(&bridge, "WorkbenchSceneLevelItem", "tree_depth"),
        Some(1)
    );
    assert_eq!(
        control_visibility(&bridge, "WorkbenchScenePropsItem"),
        Some(zircon_runtime_interface::ui::tree::UiVisibility::Collapsed)
    );
}

#[test]
fn hierarchy_patch_requires_an_immediate_retained_generation_base() {
    let _guard = env_lock().lock().unwrap();
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    bridge
        .sync_scene_and_inspector(
            &hierarchy_entries(10, vec![hierarchy_row(1, None, 0, "World", 1)]),
            None,
        )
        .unwrap();
    let fragment = SceneInspectionHierarchyFragment::patch(
        delta(
            8,
            9,
            vec![anchor(1, None, 0, 2)],
            SceneInspectionSelectionDelta::unchanged(),
        ),
        vec![hierarchy_row(1, None, 0, "Stale World", 2)],
    )
    .unwrap();

    let applied = bridge.apply_scene_hierarchy_fragment(&fragment).unwrap();

    assert!(!applied.applied());
    assert_eq!(applied.updated_rows(), 0);
    assert_eq!(
        control_string(&bridge, "WorkbenchSceneRootItem", "text").as_deref(),
        Some("World")
    );
}

#[test]
fn hierarchy_patch_escalates_parent_or_depth_changes_to_an_explicit_reflow() {
    let _guard = env_lock().lock().unwrap();
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    bridge
        .sync_scene_and_inspector(
            &hierarchy_entries(
                40,
                vec![
                    hierarchy_row(1, None, 0, "World", 41),
                    hierarchy_row(2, Some(1), 1, "Group", 42),
                    hierarchy_row(3, Some(2), 2, "Camera", 43),
                ],
            ),
            None,
        )
        .unwrap();
    let patch = SceneInspectionHierarchyFragment::patch(
        delta(
            40,
            41,
            vec![anchor(3, Some(1), 1, 45)],
            SceneInspectionSelectionDelta::unchanged(),
        ),
        vec![hierarchy_row(3, Some(1), 1, "Camera", 45)],
    )
    .unwrap();

    assert!(!bridge
        .apply_scene_hierarchy_fragment(&patch)
        .unwrap()
        .applied());
}

#[test]
fn hierarchy_patch_updates_only_the_selection_delta_controls() {
    let _guard = env_lock().lock().unwrap();
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    bridge
        .sync_scene_and_inspector(
            &hierarchy_entries_with_selection(
                50,
                vec![
                    hierarchy_row(1, None, 0, "World", 51),
                    hierarchy_row(2, Some(1), 1, "Camera", 52),
                    hierarchy_row(3, Some(1), 1, "Light", 53),
                ],
                [2],
            ),
            None,
        )
        .unwrap();
    let patch = SceneInspectionHierarchyFragment::patch(
        delta(
            50,
            50,
            Vec::new(),
            SceneInspectionSelectionDelta::delta(vec![3], vec![2]),
        ),
        Vec::new(),
    )
    .unwrap();

    let applied = bridge.apply_scene_hierarchy_fragment(&patch).unwrap();

    assert!(applied.applied());
    assert_eq!(applied.updated_rows(), 2);
    assert!(!control_bool(
        &bridge,
        "WorkbenchSceneEnvironmentItem",
        "selected"
    ));
    assert!(control_bool(&bridge, "WorkbenchSceneLevelItem", "selected"));
}

#[test]
fn coalesced_latest_selection_delta_converges_from_the_retained_revision() {
    let _guard = env_lock().lock().unwrap();
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    bridge
        .sync_scene_and_inspector(
            &hierarchy_entries(
                60,
                vec![
                    hierarchy_row(1, None, 0, "World", 61),
                    hierarchy_row(2, Some(1), 1, "Camera", 62),
                    hierarchy_row(3, Some(1), 1, "Light", 63),
                ],
            ),
            None,
        )
        .unwrap();
    let latest = SceneInspectionHierarchyFragment::patch(
        delta(
            60,
            60,
            Vec::new(),
            SceneInspectionSelectionDelta::between(0, 2, vec![2, 3], Vec::new()),
        ),
        Vec::new(),
    )
    .unwrap();

    let applied = bridge.apply_scene_hierarchy_fragment(&latest).unwrap();

    assert!(applied.applied());
    assert_eq!(applied.updated_rows(), 2);
    assert!(control_bool(
        &bridge,
        "WorkbenchSceneEnvironmentItem",
        "selected"
    ));
    assert!(control_bool(&bridge, "WorkbenchSceneLevelItem", "selected"));
}

#[test]
fn selection_revision_gap_repairs_only_the_selection_overlay() {
    let _guard = env_lock().lock().unwrap();
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    bridge
        .sync_scene_and_inspector(
            &hierarchy_entries(
                60,
                vec![
                    hierarchy_row(1, None, 0, "World", 61),
                    hierarchy_row(2, Some(1), 1, "Camera", 62),
                ],
            ),
            None,
        )
        .unwrap();
    let patch = SceneInspectionHierarchyFragment::patch(
        delta(
            60,
            61,
            vec![anchor(2, Some(1), 1, 63)],
            SceneInspectionSelectionDelta::between(3, 4, vec![2], Vec::new()),
        ),
        vec![hierarchy_row(2, Some(1), 1, "Gameplay Camera", 63)],
    )
    .unwrap();

    let gap = bridge.apply_scene_hierarchy_fragment(&patch).unwrap();
    assert!(gap.selection_resync_required());

    let repaired = bridge.resync_scene_hierarchy_selection(4, &[2]).unwrap();
    assert!(repaired.applied());
    assert_eq!(repaired.updated_rows(), 1);
    let applied = bridge.apply_scene_hierarchy_fragment(&patch).unwrap();
    assert!(applied.applied());
    assert_eq!(applied.updated_rows(), 1);
    assert!(control_bool(
        &bridge,
        "WorkbenchSceneEnvironmentItem",
        "selected"
    ));
    assert_eq!(
        control_string(&bridge, "WorkbenchSceneEnvironmentItem", "text").as_deref(),
        Some("Gameplay Camera")
    );
}

#[test]
fn hierarchy_patch_rejects_a_runtime_reflow_marker() {
    let message = SceneInspectionMessage::delta(
        60,
        61,
        None,
        Vec::new(),
        Vec::new(),
        Vec::new(),
        true,
        SceneInspectionFieldsDelta::unchanged(None),
        SceneInspectionSelectionDelta::unchanged(),
    );

    let error = SceneInspectionHierarchyFragment::patch(message, Vec::new()).unwrap_err();

    assert_eq!(
        error,
        crate::ui::workbench::snapshot::SceneInspectionHierarchyFragmentError::PatchContainsStructuralRows
    );
}

#[test]
fn hierarchy_patch_rejects_duplicate_anchor_entities() {
    let message = delta(
        61,
        62,
        vec![anchor(1, None, 0, 11), anchor(1, None, 0, 11)],
        SceneInspectionSelectionDelta::unchanged(),
    );
    let error = SceneInspectionHierarchyFragment::patch(
        message,
        vec![
            hierarchy_row(1, None, 0, "World", 11),
            hierarchy_row(2, None, 0, "Other", 12),
        ],
    )
    .unwrap_err();

    assert_eq!(
        error,
        crate::ui::workbench::snapshot::SceneInspectionHierarchyFragmentError::PatchRowMismatch {
            entity: 1,
        }
    );
}

#[test]
fn large_patch_validates_rows_through_the_entity_index() {
    let changed_rows = (0..10_000)
        .map(|entity| hierarchy_row(entity, None, 0, "Renamed Entity", entity + 1))
        .collect::<Vec<_>>();
    let changed_anchors = changed_rows
        .iter()
        .map(|row| anchor(row.entity, row.parent, row.depth, row.subtree_hash))
        .collect();

    let patch = SceneInspectionHierarchyFragment::patch(
        delta(
            69,
            70,
            changed_anchors,
            SceneInspectionSelectionDelta::unchanged(),
        ),
        changed_rows,
    )
    .unwrap();

    assert_eq!(patch.changed_rows().map(<[_]>::len), Some(10_000));
}

#[test]
fn ten_thousand_row_patch_does_not_reflow_the_projection() {
    let _guard = env_lock().lock().unwrap();
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    let rows = (0..10_000)
        .map(|entity| hierarchy_row(entity, None, 0, "Entity", entity))
        .collect();
    bridge
        .sync_scene_and_inspector(&hierarchy_entries_with_selection(70, rows, 0..10_000), None)
        .unwrap();
    let patch = SceneInspectionHierarchyFragment::patch(
        delta(
            70,
            71,
            vec![anchor(9_999, None, 0, 10_001)],
            SceneInspectionSelectionDelta::unchanged(),
        ),
        vec![hierarchy_row(9_999, None, 0, "Renamed Entity", 10_001)],
    )
    .unwrap();

    let applied = bridge.apply_scene_hierarchy_fragment(&patch).unwrap();

    assert!(applied.applied());
    assert_eq!(applied.updated_rows(), 1);
    assert!(!applied.reflowed());
}

#[test]
fn virtual_hierarchy_row_rename_applies_without_reflow() {
    let _guard = env_lock().lock().unwrap();
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    let rows = (1..=11)
        .map(|entity| hierarchy_row(entity, None, 0, "Entity", entity))
        .collect();
    bridge
        .sync_scene_and_inspector(&hierarchy_entries(90, rows), None)
        .unwrap();

    let patch = SceneInspectionHierarchyFragment::patch(
        delta(
            90,
            91,
            vec![anchor(11, None, 0, 92)],
            SceneInspectionSelectionDelta::unchanged(),
        ),
        vec![hierarchy_row(11, None, 0, "Renamed Virtual Entity", 92)],
    )
    .unwrap();

    let applied = bridge.apply_scene_hierarchy_fragment(&patch).unwrap();

    assert!(applied.applied());
    assert!(!applied.reflowed());
    assert_eq!(applied.updated_rows(), 1);
    assert_eq!(
        control_string(&bridge, "WorkbenchSceneVirtualItem11", "text").as_deref(),
        Some("Renamed Virtual Entity")
    );
    assert_eq!(
        control_integer(&bridge, "WorkbenchSceneVirtualItem11", "scene_node_id"),
        Some(11)
    );
}

fn delta(
    previous_generation: u64,
    generation: u64,
    changed_anchors: Vec<SceneInspectionHierarchyAnchor>,
    selection: SceneInspectionSelectionDelta,
) -> SceneInspectionMessage {
    SceneInspectionMessage::delta(
        previous_generation,
        generation,
        None,
        Vec::new(),
        changed_anchors,
        Vec::new(),
        false,
        SceneInspectionFieldsDelta::unchanged(None),
        selection,
    )
}

fn hierarchy_entries(generation: u64, rows: Vec<WorldInspectionHierarchyRow>) -> SceneEntries {
    hierarchy_entries_with_selection(generation, rows, [])
}

fn hierarchy_entries_with_selection(
    generation: u64,
    rows: Vec<WorldInspectionHierarchyRow>,
    selected: impl IntoIterator<Item = u64>,
) -> SceneEntries {
    SceneEntries::from_hierarchy_rows_at_generation(rows, selected, generation)
}

fn hierarchy_row(
    entity: u64,
    parent: Option<u64>,
    depth: u32,
    display_name: &str,
    subtree_hash: u64,
) -> WorldInspectionHierarchyRow {
    WorldInspectionHierarchyRow {
        entity,
        parent,
        depth,
        display_name: display_name.to_string(),
        kind: "Entity".to_string(),
        subtree_hash,
        focused: false,
        active_in_hierarchy: true,
        has_children: false,
    }
}

fn anchor(
    entity: u64,
    parent: Option<u64>,
    depth: u32,
    subtree_hash: u64,
) -> SceneInspectionHierarchyAnchor {
    SceneInspectionHierarchyAnchor::new(entity, parent, depth, subtree_hash)
}
