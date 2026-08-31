from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
WORKBENCH_BRIDGE = ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench"
)
HIERARCHY_REFRESH = ROOT / (
    "zircon_editor/src/ui/retained_host/app/host_lifecycle/scene_hierarchy_refresh.rs"
)
HIERARCHY_ROW_PATCH = ROOT / (
    "zircon_editor/src/ui/retained_host/app/host_lifecycle/"
    "scene_hierarchy_refresh/hierarchy_row_patch.rs"
)
HIERARCHY_RENAME = ROOT / (
    "zircon_editor/src/ui/retained_host/app/hierarchy_rename.rs"
)
HIERARCHY_DRAG_SOURCE = ROOT / (
    "zircon_editor/src/ui/retained_host/app/hierarchy_pointer/drag_source.rs"
)


class EditorHierarchyNativeAuthorityPerformanceContractTests(unittest.TestCase):
    def test_hierarchy_retained_tree_is_bounded_to_the_authored_skeleton(self) -> None:
        source = (WORKBENCH_BRIDGE / "scene_tree_rows.rs").read_text(encoding="utf-8")
        reconcile = source.split("fn reconcile_scene_tree_row_capacity", 1)[1].split(
            "fn scene_tree_control_ids", 1
        )[0]

        self.assertIn("SCENE_TREE_AUTHORED_ROW_COUNT", source)
        self.assertIn("entry_count.min(SCENE_TREE_AUTHORED_ROW_COUNT)", reconcile)
        self.assertNotIn(".reconcile(\n            &mut self.template_surface.surface,\n            entry_count,", reconcile)

    def test_projection_keeps_all_logical_entities_not_only_materialized_controls(self) -> None:
        source = (WORKBENCH_BRIDGE / "scene_hierarchy_projection.rs").read_text(
            encoding="utf-8"
        )

        self.assertIn("rows_by_entity", source)
        self.assertIn("pub(super) fn contains_entity", source)
        self.assertIn("self.rows_by_entity = rows", source)
        self.assertIn("controls_by_entity", source)

    def test_sparse_patch_treats_unmaterialized_rows_as_logical_rows(self) -> None:
        source = (WORKBENCH_BRIDGE / "scene_hierarchy_fragment.rs").read_text(
            encoding="utf-8"
        )
        apply_patch = source.split("fn apply_scene_hierarchy_fragment", 1)[1].split(
            "fn apply_selection_delta", 1
        )[0]
        entity_guard = source.split("fn selection_entities_exist", 1)[1].split(
            "fn sync_scene_row", 1
        )[0]

        self.assertIn("if let Some(control_id)", apply_patch)
        self.assertIn("patch_row(row)", apply_patch)
        self.assertIn("contains_entity", entity_guard)

    def test_sparse_receipt_patches_the_native_row_model_without_flattening(self) -> None:
        refresh = HIERARCHY_REFRESH.read_text(encoding="utf-8")
        row_patch = HIERARCHY_ROW_PATCH.read_text(encoding="utf-8")

        self.assertIn("logical_row_patches()", refresh)
        self.assertIn("patch_presented_hierarchy_rows", refresh)
        self.assertIn("with_row_patches", row_patch)
        self.assertIn("row_patches.is_empty()", row_patch)
        self.assertIn("replace_presented_hierarchy_rows", refresh)
        self.assertIn("ModelRc::with_metadata", refresh)
        self.assertNotIn("hierarchy_nodes.iter().cloned().collect", row_patch)

    def test_sparse_patch_builds_one_shared_generation_for_all_hierarchy_panes(self) -> None:
        row_patch = HIERARCHY_ROW_PATCH.read_text(encoding="utf-8")
        patch = row_patch.split("fn patch_presented_hierarchy_rows", 1)[1].split(
            "fn replace_hierarchy_pane", 1
        )[0]

        self.assertIn("let patched_rows =", patch)
        self.assertIn("shares_values_with", patch)
        self.assertIn("replace_presented_hierarchy_rows(presentation, &patched_rows)", patch)
        self.assertEqual(patch.count("with_row_patches(materialized_patches)"), 1)

    def test_double_click_rename_resolves_the_current_name_by_entity(self) -> None:
        source = HIERARCHY_RENAME.read_text(encoding="utf-8")
        click = source.split("fn track_hierarchy_click_for_rename", 1)[1].split(
            "fn dispatch_hierarchy_rename_edit", 1
        )[0]

        self.assertIn("self.runtime.editor_snapshot().scene_entries", click)
        self.assertIn("hierarchy_name_for_entity", click)
        self.assertNotIn("begin_hierarchy_rename(entry.entity, &entry.display_name)", click)

    def test_drag_payload_resolves_the_current_row_by_entity(self) -> None:
        source = HIERARCHY_DRAG_SOURCE.read_text(encoding="utf-8")
        drag = source.split("fn hierarchy_drag_source_from_route", 1)[1].split(
            "fn hierarchy_reparent_target_from_route", 1
        )[0]

        self.assertIn("authoritative_scene_entries", drag)
        self.assertIn("find(|candidate| candidate.entity == entry.entity)", drag)
        self.assertIn("scene_drag_payload_from_entry(authoritative_entry)", drag)
        self.assertNotIn("scene_drag_payload_from_entry(entry)", drag)

    def test_failed_sparse_publication_invalidates_bridge_authority_before_reflow(self) -> None:
        refresh = HIERARCHY_REFRESH.read_text(encoding="utf-8")

        self.assertEqual(refresh.count("invalidate_scene_hierarchy_projection();"), 2)
        for failure in refresh.split("if !self.publish_sparse_hierarchy_host_nodes(")[1:]:
            block = failure.split("return;", 1)[0]
            invalidate = block.index("invalidate_scene_hierarchy_projection();")
            reflow = block.index("resync_scene_hierarchy_from_message")
            self.assertLess(invalidate, reflow)
        unavailable = refresh.split("fn resync_scene_hierarchy_from_message", 1)[1].split(
            "fn commit_scene_hierarchy_reflow", 1
        )[0]
        self.assertGreaterEqual(unavailable.count("self.mark_layout_dirty();"), 2)


if __name__ == "__main__":
    unittest.main()
