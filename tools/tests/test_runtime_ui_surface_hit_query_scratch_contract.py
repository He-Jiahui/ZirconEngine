from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SURFACE = ROOT / "zircon_runtime/src/ui/surface/surface.rs"
FRAME_HIT_TEST = ROOT / "zircon_runtime/src/ui/surface/frame_hit_test.rs"
FRAME_PUBLICATION = ROOT / "zircon_runtime/src/ui/surface/surface/frame_publication.rs"
HIT_TEST = ROOT / "zircon_runtime/src/ui/tree/hit_test.rs"
RUST_TESTS = ROOT / "zircon_runtime/src/ui/tests/hit_grid.rs"


class RuntimeUiSurfaceHitQueryScratchContractTests(unittest.TestCase):
    def test_surface_instance_queries_use_the_published_frame_authority(self):
        source = SURFACE.read_text(encoding="utf-8")
        start = source.index("    pub fn hit_test_with_query(")
        end = source.index("\n    pub(super) fn rendered_popup_background", start)
        query = source[start:end]

        self.assertIn(
            "self.hit_test_published_surface_frame_with_query(query)",
            query,
        )
        self.assertNotIn("authoritative_grid", query)
        self.assertNotIn("surface_frame()", query)

    def test_published_frame_query_borrows_retained_domains_without_arc_clone(self):
        source = FRAME_PUBLICATION.read_text(encoding="utf-8")
        start = source.index("    pub(super) fn hit_test_published_surface_frame_with_query(")
        end = source.index("\n    pub fn surface_frame", start)
        query = source[start:end]

        self.assertIn("publication.frame.as_deref()", query)
        self.assertIn("hit_test_surface_frame_with_query_using_index(", query)
        self.assertIn("&self.hit_test", query)
        self.assertNotIn("Arc::clone", query)
        self.assertNotIn("self.surface_frame()", query)
        self.assertNotIn("publication.dirty", query)

    def test_frame_and_instance_entry_points_share_one_query_core(self):
        source = FRAME_HIT_TEST.read_text(encoding="utf-8")

        self.assertIn(
            "pub(super) fn hit_test_surface_frame_with_query_using_index(",
            source,
        )
        self.assertIn(
            "hit_test_index.hit_test_owned_grid_arranged_with_query(",
            source,
        )
        self.assertIn(
            "hit_test_surface_frame_with_query_using_index(surface_frame, query, &hit_test_index)",
            source,
        )

    def test_hit_index_owns_reusable_radius_query_scratch(self):
        source = HIT_TEST.read_text(encoding="utf-8")

        self.assertIn("query_scratch: UiHitQueryScratchCell", source)
        self.assertIn("pub(crate) fn hit_test_owned_grid_arranged_with_query(", source)
        self.assertIn("&self.query_scratch", source)
        self.assertIn("let query_scratch = UiHitQueryScratchCell::default();", source)

    def test_lower_regression_exercises_the_surface_product_path(self):
        source = RUST_TESTS.read_text(encoding="utf-8")
        start = source.index(
            "fn dense_radius_query_reuses_generation_scratch_with_linear_dedupe_probes()"
        )
        test = source[start:]

        self.assertGreaterEqual(test.count("surface.hit_test_with_query("), 2)
        self.assertIn("hit_test_surface_frame_with_query(&surface.surface_frame()", test)
        self.assertIn("assert_eq!(frame_hit, first_hit);", test)


if __name__ == "__main__":
    unittest.main()
