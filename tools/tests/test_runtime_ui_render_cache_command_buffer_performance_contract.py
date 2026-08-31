import unittest
from pathlib import Path

from tools.runtime_ui_render_cache_command_buffer_pressure import run


REPO_ROOT = Path(__file__).resolve().parents[2]
RENDER_CACHE = REPO_ROOT / "zircon_runtime/src/ui/surface/render/cache.rs"
SURFACE = REPO_ROOT / "zircon_runtime/src/ui/surface/surface.rs"
REBUILD = REPO_ROOT / "zircon_runtime/src/ui/surface/surface/rebuild.rs"
UPDATE_TESTS = (
    REPO_ROOT / "zircon_runtime/src/ui/surface/render/cache/tests/update.rs"
)
PROFILE_MANIFEST = REPO_ROOT / "tools/profile-capture-manifest.ps1"


class RuntimeUiRenderCacheCommandBufferPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.source = RENDER_CACHE.read_text(encoding="utf-8")
        cls.surface = SURFACE.read_text(encoding="utf-8")
        cls.rebuild = REBUILD.read_text(encoding="utf-8")
        cls.update = cls.source.split("    pub fn update(", 1)[1].split(
            "    pub fn update_for_arranged(", 1
        )[0]
        cls.tests = UPDATE_TESTS.read_text(encoding="utf-8")

    def test_update_borrows_the_owned_input_command_buffer(self):
        self.assertIn(
            "extract.list.commands.iter().enumerate()",
            self.update,
        )
        self.assertNotIn("for command in extract.list.commands", self.update)

    def test_update_does_not_materialize_a_second_command_vector(self):
        self.assertNotIn("retained_commands", self.update)
        self.assertNotIn("Vec::with_capacity(extract.list.commands.len())", self.update)

    def test_update_returns_the_original_extract(self):
        self.assertIn("UiRenderCacheUpdate { extract, stats }", self.update)

    def test_previous_extract_is_the_only_full_command_authority(self):
        self.assertIn("previous_extract: &UiRenderExtract", self.update)
        self.assertRegex(
            self.update,
            r"previous_extract\s*\.list\s*\.commands\s*\.get",
        )
        self.assertIn("previous.frame == entry.frame", self.update)
        self.assertIn("struct UiCachedRenderCommandMetadata", self.source)
        self.assertIn("command_index: usize", self.source)
        self.assertIn("frame: UiFrame", self.source)
        self.assertNotIn(
            "struct UiCachedRenderCommand {\n    command: UiRenderCommand",
            self.source,
        )
        self.assertNotIn("command: command.clone()", self.source)
        self.assertNotIn("command: next.clone()", self.source)

    def test_derived_render_cache_is_not_serialized_with_the_surface(self):
        self.assertIn(
            "#[serde(default, skip)]\n    pub render_cache: UiSurfaceRenderCache",
            self.surface,
        )
        self.assertIn(
            "surface_serialization_omits_derived_render_cache",
            self.tests,
        )
        self.assertNotIn("Keep persisted one-command node caches compatible", self.source)

    def test_empty_metadata_damages_the_authoritative_previous_extract(self):
        self.assertIn("let cache_metadata_was_empty = self.entries.is_empty()", self.update)
        self.assertRegex(
            self.update,
            r"if cache_metadata_was_empty\s*\{\s*for previous in &previous_extract\.list\.commands",
        )
        self.assertIn(
            "surface_restore_damages_old_and_new_frames_when_commands_move_or_disappear",
            self.tests,
        )

    def test_force_rebuild_keeps_metadata_for_old_frame_damage(self):
        self.assertNotRegex(
            self.rebuild,
            r"if force_rebuild\s*\{\s*self\.render_cache = Default::default\(\);\s*\}",
        )

    def test_update_reconciles_stale_buckets_in_one_retain_pass(self):
        self.assertIn("self.entries.retain(|node_id, entries|", self.update)
        self.assertIn("entries.remove_from(retained_count, &mut damage)", self.update)
        self.assertNotIn("let stale_nodes", self.update)
        self.assertNotIn(".collect::<Vec<_>>()", self.update)

    def test_update_builds_command_ranges_in_the_primary_command_pass(self):
        self.assertIn(
            "for (command_offset, command) in extract.list.commands.iter().enumerate()",
            self.update,
        )
        self.assertIn("UiRenderCommandBuildState::new(command_offset)", self.update)
        self.assertIn("self.command_ranges = command_build_states", self.update)
        self.assertNotIn("reindex_command_ranges", self.source)

    def test_geometry_patchable_refresh_has_no_intermediate_node_id_vector(self):
        refresh = self.source.split("    fn refresh_geometry_patchable_nodes(", 1)[1]
        refresh = refresh.split("impl PartialEq for UiSurfaceRenderCache", 1)[0]

        self.assertIn("let mut patchable_node_ids = BTreeSet::new()", refresh)
        self.assertIn("self.node_is_geometry_patchable(", refresh)
        self.assertIn("self.geometry_patchable_node_ids = patchable_node_ids", refresh)
        self.assertNotIn(".keys().copied().collect::<Vec<_>>()", refresh)

    def test_rust_regression_checks_pointer_and_capacity_identity(self):
        self.assertIn(
            "render_cache_update_retains_the_input_command_buffer_allocation",
            self.tests,
        )
        self.assertIn("update.extract.list.commands.as_ptr()", self.tests)
        self.assertIn("update.extract.list.commands.capacity()", self.tests)
        self.assertIn(
            "render_cache_range_lookup_fails_closed_for_non_contiguous_node_commands",
            self.tests,
        )

    def test_pressure_model_preserves_m236_and_counts_compact_authority(self):
        result = run(
            update_count=4096,
            commands_per_update=32768,
            cache_entry_count=16384,
            changed_commands_per_update=8,
            modeled_command_payload_bytes=512,
        )

        self.assertEqual(
            result["delta"]["avoided_command_vector_allocations"],
            4096,
        )
        self.assertEqual(
            result["delta"]["avoided_inter_vector_command_header_moves"],
            134217728,
        )
        self.assertEqual(
            result["borrowed_input_update"]["inter_vector_command_header_moves"],
            0,
        )
        self.assertEqual(
            result["retired_stale_reconciliation"]["cache_entry_visits"],
            134217728,
        )
        self.assertEqual(
            result["single_pass_stale_reconciliation"]["cache_entry_visits"],
            67108864,
        )
        self.assertEqual(result["delta"]["avoided_cache_entry_visits"], 67108864)
        self.assertEqual(result["delta"]["cache_entry_visit_reduction_ratio"], 2.0)
        self.assertEqual(
            result["delta"]["avoided_geometry_node_id_vector_allocations"],
            4096,
        )
        self.assertEqual(
            result["delta"]["avoided_inter_vector_node_id_moves"],
            67108864,
        )
        self.assertEqual(
            result["retired_command_range_reindex"]["command_visits"],
            134217728,
        )
        self.assertEqual(
            result["inline_command_range_publication"][
                "node_state_publication_visits"
            ],
            67108864,
        )
        self.assertEqual(
            result["delta"]["avoided_command_range_work_units"],
            67108864,
        )
        self.assertEqual(
            result["delta"]["command_range_work_reduction_ratio"],
            2.0,
        )
        self.assertEqual(
            result["retired_full_command_mirror"]["retained_command_count"],
            32768,
        )
        self.assertEqual(
            result["compact_derived_metadata"]["retained_full_command_count"],
            0,
        )
        self.assertEqual(
            result["delta"]["avoided_retained_payload_bytes"],
            16777216,
        )
        self.assertEqual(
            result["delta"]["avoided_cold_build_command_clones"],
            32768,
        )
        self.assertEqual(
            result["delta"]["avoided_local_patch_command_clones"],
            32768,
        )
        self.assertEqual(
            result["retired_surface_serialization"]["command_record_count"],
            65536,
        )
        self.assertEqual(
            result["compact_surface_serialization"]["command_record_count"],
            32768,
        )
        self.assertEqual(result["delta"]["serialization_command_record_ratio"], 2.0)

    def test_profile_manifest_binds_runtime_render_cache(self):
        manifest = PROFILE_MANIFEST.read_text(encoding="utf-8")
        self.assertIn(
            '"zircon_runtime/src/ui/surface/render/cache.rs"',
            manifest,
        )


if __name__ == "__main__":
    unittest.main()
