import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
PATCH = ROOT / "zircon_runtime/src/ui/tree/hit_test/cell_membership_patch.rs"
GEOMETRY = ROOT / "zircon_runtime/src/ui/tree/hit_test/geometry_patch.rs"
HIT_TEST = ROOT / "zircon_runtime/src/ui/tree/hit_test.rs"


class RuntimeUiA3HitGridCellBatchingContractTests(unittest.TestCase):
    def test_membership_stage_groups_cells_and_publishes_preflighted_replacements(self) -> None:
        source = PATCH.read_text(encoding="utf-8")
        stage = source[source.index("pub(super) fn stage("):source.index("    /// Builds every replacement")]
        apply = source[source.index("pub(super) fn apply"):source.index("fn retain_without_sorted_memberships")]

        self.assertIn("by_cell: BTreeMap<usize, UiCellMembershipDelta>", source)
        self.assertIn("self.by_cell.entry(left).or_default()", stage)
        self.assertNotIn("cells.get_mut", stage)
        self.assertIn("Builds every replacement before publishing any cell", source)
        self.assertIn("let mut replacements = Vec::with_capacity(self.by_cell.len());", apply)
        self.assertIn("cells.get(cell_index).ok_or(())?", apply)
        self.assertIn("for (cell_index, entries) in replacements", apply)
        self.assertIn("cell.entries = entries.into();", apply)
        self.assertLess(apply.index("cells.get(cell_index).ok_or(())?"), apply.index("for (cell_index, entries) in replacements"))

    def test_geometry_patch_stages_all_updates_before_cell_and_entry_publication(self) -> None:
        source = GEOMETRY.read_text(encoding="utf-8")
        body = source[source.index("fn patch_geometry_with_routes_and_stats_inner("):source.index("fn record_hit_grid_persistent_cow")]

        for anchor in (
            "let mut updates = Vec::with_capacity(changed_node_ids.len());",
            "let mut membership_patches = UiCellMembershipPatches::default();",
            "membership_patches.stage(",
            "let membership_stats = membership_patches.apply(&mut self.grid.cells",
            "let (current_entry, stats) = self",
            "self.entry_cells.insert(entry_node_id, next_cells);",
            "record_hit_grid_persistent_cow(entry_cow_stats, membership_stats);",
        ):
            self.assertIn(anchor, body)

        self.assertLess(body.index("let mut updates"), body.index("let membership_stats"))
        self.assertLess(body.index("let membership_stats"), body.index("let (current_entry, stats)"))
        self.assertLess(body.index("let (current_entry, stats)"), body.index("record_hit_grid_persistent_cow"))

    def test_base_hit_grid_route_uses_the_same_batch_owner(self) -> None:
        source = HIT_TEST.read_text(encoding="utf-8")
        body = source[source.index("pub(crate) fn patch_cell_memberships"):source.index("pub fn from_grid")]

        self.assertIn("let mut batch = cell_membership_patch::UiCellMembershipPatches::default();", body)
        self.assertIn("batch.stage(entry_index, &previous_cells, &next_cells, false);", body)
        self.assertIn("let stats = batch.apply(cells, sort_key)?;", body)
        self.assertNotIn("cells.get_mut", body)

    def test_rust_regressions_cover_dense_atomic_and_painter_order_cases(self) -> None:
        patch_tests = PATCH.read_text(encoding="utf-8")
        geometry_tests = GEOMETRY.read_text(encoding="utf-8")

        for anchor in (
            "dense_batches_match_expected_membership_for_1k_and_10k_entries",
            "rejects_missing_cell_before_publishing_valid_replacements",
            "unchanged_footprints_stage_no_cell_work",
            "changed_order_reinserts_membership_in_unchanged_cell",
            "one_cell_batch_deletes_and_inserts_multiple_memberships_once",
        ):
            self.assertIn(anchor, patch_tests)
        for anchor in (
            "dense_geometry_batches_match_full_rebuild_and_visit_each_cell_once",
            "geometry_batch_preflight_failure_preserves_index",
            "cold_lookup_preflight_failure_preserves_reverse_maps_and_grid",
            "painter_reorder_then_cell_move_removes_nonmonotonic_entry_index",
        ):
            self.assertIn(anchor, geometry_tests)
        self.assertIn("for entry_count in [1_000, 10_000]", geometry_tests)


if __name__ == "__main__":
    unittest.main()
