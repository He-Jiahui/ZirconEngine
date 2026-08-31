from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
TEMPLATE_SURFACE = ROOT / (
    "zircon_editor/src/ui/workbench/reference/template_surface.rs"
)
COMPONENTIZED_WINDOW = ROOT / (
    "zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/"
    "workbench/componentized_window.rs"
)
WORKBENCH_PROJECTION = ROOT / (
    "zircon_editor/src/ui/retained_host/ui/workbench_window_projection.rs"
)
GEOMETRY_APPLY = ROOT / (
    "zircon_editor/src/ui/retained_host/ui/apply_presentation/geometry.rs"
)
STATE = ROOT / "zircon_editor/src/ui/retained_host/host_contract/globals/state.rs"
INDEX = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/"
    "template_node/index.rs"
)
PERSISTENT_BUCKETS = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/"
    "template_node/index/persistent_buckets.rs"
)
PATCH = ROOT / (
    "zircon_editor/src/ui/retained_host/host_contract/surface_hit_test/"
    "template_node/index/geometry_patch.rs"
)


class EditorRetainedShellGeometryFastPathPerformanceContract(unittest.TestCase):
    def read(self, path: Path) -> str:
        return path.read_text(encoding="utf-8")

    def test_geometry_rows_are_published_only_for_nonsemantic_pending_work(self) -> None:
        source = self.read(TEMPLATE_SURFACE)

        self.assertIn("pending_host_projection_has_semantic_changes", source)
        self.assertIn("pending_host_projection_geometry_patch_indices", source)
        indices = source.split("fn pending_host_projection_geometry_patch_indices", 1)[1]
        self.assertIn("self.pending_host_projection_has_semantic_changes", indices)
        self.assertIn("return None", indices)
        self.assertIn("pending_host_projection_patch_indices", indices)

    def test_mount_or_scale_drift_cannot_use_the_geometry_patch(self) -> None:
        source = self.read(COMPONENTIZED_WINDOW)

        self.assertIn("committed_mount_origin", source)
        self.assertIn("committed_presentation_scale_factor", source)
        self.assertIn("pending_host_projection_geometry_patch_indices", source)
        self.assertIn("return None", source)
        self.assertIn("mark_host_projection_committed", source)

    def test_workbench_patch_preserves_semantic_payloads(self) -> None:
        source = self.read(WORKBENCH_PROJECTION)
        start = source.index(
            "pub(crate) fn build_host_contract_workbench_window_geometry_patch"
        )
        body = source[start:]

        self.assertIn("let mut projected = previous.clone()", body)
        self.assertIn("projected.frame =", body)
        self.assertIn("projected.clip_frame =", body)
        self.assertIn("projected.z_index =", body)
        self.assertIn("nodes.with_row_patches", body)

    def test_geometry_apply_uses_exact_rows_and_avoids_full_workbench_conversion(self) -> None:
        source = self.read(GEOMETRY_APPLY)

        self.assertIn("workbench_geometry_patch_indices", source)
        self.assertIn("build_host_contract_workbench_window_geometry_patch_at_mount_and_scale", source)
        self.assertIn("set_host_geometry_presentation(geometry_presentation, &workbench_patch.changed_rows)", source)
        self.assertNotIn("to_host_contract_workbench_window(", source)

    def test_publication_builds_hit_index_before_swapping_presentation(self) -> None:
        source = self.read(STATE)
        start = source.index("pub(crate) fn replace_host_geometry_presentation")
        body = source[start:]

        self.assertIn("patch_geometry_presentation", body)
        self.assertIn("self.workbench_hit_index = Arc::new(next_hit_index)", body)
        self.assertIn("self.host_presentation = Arc::new(presentation)", body)
        self.assertLess(
            body.index("self.workbench_hit_index = Arc::new(next_hit_index)"),
            body.index("self.host_presentation = Arc::new(presentation)"),
        )

    def test_hit_index_patch_is_path_copy_and_does_not_rebuild_the_full_index(self) -> None:
        index = self.read(INDEX)
        patch = self.read(PATCH)
        buckets = self.read(PERSISTENT_BUCKETS)

        self.assertIn("PersistentCellBuckets", index)
        self.assertIn("patch_geometry_presentation", patch)
        self.assertIn("CellMembershipDelta", patch)
        self.assertIn("with_updates", patch)
        self.assertIn("Arc", buckets)
        self.assertNotIn("HostWorkbenchHitIndex::from_presentation", patch)
        self.assertNotIn("base_index.grid.entries.iter()", patch)


if __name__ == "__main__":
    unittest.main()
