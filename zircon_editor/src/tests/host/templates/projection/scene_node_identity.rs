use zircon_runtime::scene::WorldInspectionHierarchyRow;
use zircon_runtime_interface::ui::component::UiValue;
use zircon_runtime_interface::ui::layout::UiSize;

use crate::core::editor_message::{
    SceneInspectionFieldsDelta, SceneInspectionHierarchyAnchor, SceneInspectionMessage,
    SceneInspectionSelectionDelta,
};
use crate::ui::retained_host::callback_dispatch::BuiltinWorkbenchWindowTemplateSurfaceBridge;
use crate::ui::workbench::snapshot::{SceneEntries, SceneInspectionHierarchyFragment};

use super::{control_string, env_lock};

#[test]
fn hierarchy_control_identity_preserves_the_full_u64_range_through_sparse_patches() {
    let _guard = env_lock().lock().unwrap();
    let ids = [i64::MAX as u64, i64::MAX as u64 + 1, u64::MAX];
    let controls = [
        "WorkbenchSceneRootItem",
        "WorkbenchSceneEnvironmentItem",
        "WorkbenchSceneLevelItem",
    ];
    let mut bridge =
        BuiltinWorkbenchWindowTemplateSurfaceBridge::new(UiSize::new(1672.0, 941.0)).unwrap();
    bridge
        .sync_scene_and_inspector(
            &SceneEntries::from_hierarchy_rows_at_generation(
                ids.iter()
                    .enumerate()
                    .map(|(index, &id)| row(id, &format!("Before {index}"), index as u64 + 1))
                    .collect::<Vec<_>>(),
                [],
                7,
            ),
            None,
        )
        .unwrap();

    for (&id, &control) in ids.iter().zip(&controls) {
        assert_eq!(
            control_string(&bridge, control, "scene_node_id"),
            Some(id.to_string())
        );
        assert_eq!(bridge.scene_entity_for_control(control), Some(id));
    }

    let changed_rows = ids
        .iter()
        .enumerate()
        .map(|(index, &id)| row(id, &format!("After {index}"), index as u64 + 11))
        .collect::<Vec<_>>();
    let fragment = patch(
        7,
        8,
        changed_rows
            .iter()
            .map(|row| SceneInspectionHierarchyAnchor::new(row.entity, None, 0, row.subtree_hash))
            .collect(),
        changed_rows,
    );
    let applied = bridge.apply_scene_hierarchy_fragment(&fragment).unwrap();
    assert!(applied.applied());
    assert_eq!(applied.updated_rows(), ids.len());
    assert!(!applied.reflowed());

    for (index, (&id, &control)) in ids.iter().zip(&controls).enumerate() {
        assert_eq!(
            control_string(&bridge, control, "scene_node_id"),
            Some(id.to_string())
        );
        assert_eq!(bridge.scene_entity_for_control(control), Some(id));
        assert_eq!(
            control_string(&bridge, control, "text"),
            Some(format!("After {index}"))
        );
    }

    bridge
        .mutate_control_property_for_test(
            controls[1],
            "scene_node_id",
            UiValue::String(ids[0].to_string()),
        )
        .unwrap();
    let rejected = bridge
        .apply_scene_hierarchy_fragment(&patch(
            8,
            9,
            vec![SceneInspectionHierarchyAnchor::new(ids[1], None, 0, 99)],
            vec![row(ids[1], "Must not apply", 99)],
        ))
        .unwrap();
    assert!(!rejected.applied());
    assert_eq!(
        control_string(&bridge, controls[1], "text"),
        Some("After 1".to_string())
    );
}

fn row(entity: u64, display_name: &str, subtree_hash: u64) -> WorldInspectionHierarchyRow {
    WorldInspectionHierarchyRow {
        entity,
        parent: None,
        depth: 0,
        display_name: display_name.to_string(),
        kind: "Entity".to_string(),
        subtree_hash,
        active_in_hierarchy: true,
        has_children: false,
    }
}

fn patch(
    previous_generation: u64,
    generation: u64,
    anchors: Vec<SceneInspectionHierarchyAnchor>,
    changed_rows: Vec<WorldInspectionHierarchyRow>,
) -> SceneInspectionHierarchyFragment {
    SceneInspectionHierarchyFragment::patch(
        SceneInspectionMessage::delta(
            previous_generation,
            generation,
            None,
            Vec::new(),
            anchors,
            Vec::new(),
            false,
            SceneInspectionFieldsDelta::unchanged(None),
            SceneInspectionSelectionDelta::unchanged(),
        ),
        changed_rows,
    )
    .unwrap()
}
