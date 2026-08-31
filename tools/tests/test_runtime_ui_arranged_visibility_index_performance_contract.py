import re
import unittest
from pathlib import Path

from tools.runtime_ui_arranged_visibility_pressure import run


REPO_ROOT = Path(__file__).resolve().parents[2]
VISIBILITY_INDEX = (
    REPO_ROOT / "zircon_runtime/src/ui/surface/arranged_visibility.rs"
)
SURFACE = REPO_ROOT / "zircon_runtime/src/ui/surface/surface.rs"
REBUILD = REPO_ROOT / "zircon_runtime/src/ui/surface/surface/rebuild.rs"
EXTRACT = REPO_ROOT / "zircon_runtime/src/ui/surface/render/extract.rs"
OWNER_TEXT_PREWARM = (
    REPO_ROOT
    / "zircon_runtime/src/ui/surface/render/extract/owner_text_prewarm.rs"
)
POPUP_ANCHOR = (
    REPO_ROOT / "zircon_runtime/src/ui/surface/render/extract/popup_anchor.rs"
)
SURFACE_MODULE = REPO_ROOT / "zircon_runtime/src/ui/surface/mod.rs"
PROFILE_MANIFEST = REPO_ROOT / "tools/profile-capture-manifest.ps1"


class RuntimeUiArrangedVisibilityIndexPerformanceContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.visibility_index = (
            VISIBILITY_INDEX.read_text(encoding="utf-8")
            if VISIBILITY_INDEX.exists()
            else ""
        )
        cls.surface = SURFACE.read_text(encoding="utf-8")
        cls.rebuild = REBUILD.read_text(encoding="utf-8")
        cls.extract = EXTRACT.read_text(encoding="utf-8")
        cls.owner_text_prewarm = OWNER_TEXT_PREWARM.read_text(encoding="utf-8")
        cls.popup_anchor = POPUP_ANCHOR.read_text(encoding="utf-8")
        cls.surface_module = SURFACE_MODULE.read_text(encoding="utf-8")

    def test_visibility_index_is_a_cohesive_surface_module(self):
        self.assertIn("mod arranged_visibility;", self.surface_module)
        self.assertIn(
            "pub(crate) struct UiArrangedVisibilityIndex",
            self.visibility_index,
        )

    def test_surface_retains_and_refreshes_the_visibility_authority(self):
        self.assertIn("arranged_visibility: UiArrangedVisibilityIndex", self.surface)
        refresh = self.rebuild.split(
            "    fn refresh_arranged_node_indices(&mut self)", 1
        )[1].split("    fn refresh_layout_engine_selection_indices", 1)[0]
        self.assertRegex(
            refresh,
            re.compile(r"self\.arranged_visibility\s*\.rebuild\("),
        )
        self.assertIn("&self.arranged_tree", refresh)
        self.assertIn("&self.arranged_node_indices", refresh)

    def test_full_and_local_extract_share_the_retained_authority(self):
        full = self.rebuild.split(
            "    fn rebuild_render_extract_with_text_frame(", 1
        )[1].split("    fn patch_render_nodes(", 1)[0]
        local = self.rebuild.split("    fn patch_render_nodes(", 1)[1].split(
            "    fn text_cache_frame_stats", 1
        )[0]
        self.assertIn("Some(&self.arranged_visibility)", full)
        self.assertIn("&self.arranged_visibility", local)

    def test_render_extract_has_no_per_node_ancestor_visibility_walk(self):
        extract_sources = (
            self.extract,
            self.owner_text_prewarm,
            self.popup_anchor,
        )
        self.assertIn("mod owner_text_prewarm;", self.extract)
        self.assertIn("mod popup_anchor;", self.extract)
        for source in extract_sources:
            self.assertNotIn("is_arranged_render_visible_indexed(", source)
        self.assertGreaterEqual(
            sum(
                source.count("arranged_visibility.is_render_visible(")
                for source in extract_sources
            ),
            4,
        )

    def test_visibility_index_is_compact_and_iterative(self):
        self.assertIn("node_ids: Vec<UiNodeId>", self.visibility_index)
        self.assertIn("render_visible_words: Vec<u64>", self.visibility_index)
        self.assertIn("while let Some", self.visibility_index)
        self.assertNotIn("fn resolve_recursive", self.visibility_index)
        self.assertNotIn("BTreeMap<UiNodeId, bool>", self.visibility_index)
        self.assertNotIn("BTreeSet<UiNodeId>", self.visibility_index)

    def test_product_profile_distinguishes_retained_rebuild_and_fallback(self):
        self.assertIn(
            '"ui.arranged_visibility.rebuild_count"',
            self.rebuild,
        )
        self.assertIn(
            '"ui.arranged_visibility.node_resolve_count"',
            self.rebuild,
        )
        self.assertIn(
            '"ui.render_extract.visibility_index_fallback_build_count"',
            self.extract,
        )

    def test_lower_regressions_cover_inheritance_and_fail_closed_inputs(self):
        for test_name in (
            "hidden_ancestor_hides_visible_descendants",
            "self_hit_test_invisible_ancestor_remains_render_visible",
            "missing_parent_fails_closed",
            "parent_cycle_fails_closed",
        ):
            self.assertIn(test_name, self.visibility_index)

    def test_profile_manifest_binds_visibility_authority_and_extract_consumer(self):
        manifest = PROFILE_MANIFEST.read_text(encoding="utf-8")
        self.assertIn(
            '"zircon_runtime/src/ui/surface/arranged_visibility.rs"',
            manifest,
        )
        self.assertIn(
            '"zircon_runtime/src/ui/surface/render/extract.rs"',
            manifest,
        )

    def test_pressure_model_separates_publication_work_queries_and_memory(self):
        result = run(
            render_extract_count=4096,
            arranged_rebuild_count=64,
            node_count=16384,
            chain_depth=256,
            visibility_query_passes=3,
        )

        self.assertEqual(
            result["retired_parent_walks"]["ancestor_node_visits_per_pass"],
            2105344,
        )
        self.assertEqual(
            result["retained_visibility_index"]["arranged_rebuild_node_visits"],
            1048576,
        )
        self.assertEqual(
            result["retained_visibility_index"]["indexed_visibility_queries"],
            201326592,
        )
        self.assertEqual(
            result["retained_visibility_index"]["retained_payload_bytes"],
            133120,
        )
        self.assertEqual(
            result["retained_visibility_index"][
                "retained_payload_bytes_per_node"
            ],
            8.125,
        )
        self.assertGreater(
            result["delta"]["visibility_work_reduction_ratio"],
            100.0,
        )


if __name__ == "__main__":
    unittest.main()
