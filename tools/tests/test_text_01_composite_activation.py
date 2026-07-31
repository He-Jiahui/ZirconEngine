from __future__ import annotations

import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]


def source(path: str) -> str:
    return (REPO_ROOT / path).read_text(encoding="utf-8")


def between(document: str, start: str, end: str) -> str:
    return document.split(start, 1)[1].split(end, 1)[0]


class Text01CompositeActivationTests(unittest.TestCase):
    def test_registration_and_project_composite_activation_have_separate_owners(self) -> None:
        database = source("zircon_runtime/src/text/font/database.rs")
        asset_lifecycle = source(
            "zircon_runtime/src/text/font/database/asset_lifecycle.rs"
        )
        fallback_queries = source(
            "zircon_runtime/src/text/font/database/fallback_queries.rs"
        )

        self.assertIn("mod asset_lifecycle;", database)
        self.assertIn("pub(crate) fn replace_font_asset(", asset_lifecycle)
        self.assertIn("fn replace_asset_registrations(", asset_lifecycle)
        self.assertNotIn("active_composite_font", asset_lifecycle)
        self.assertNotIn("composite.default_family", asset_lifecycle)
        self.assertNotIn("composite.sub_fonts", asset_lifecycle)
        self.assertIn("pub(crate) fn set_project_composite_font(", database)
        self.assertIn("mod fallback_queries;", database)
        self.assertIn(
            "pub(crate) fn fallback_candidates_for_codepoint(", fallback_queries
        )

    def test_composite_candidate_enumeration_is_a_folder_backed_leaf(self) -> None:
        font_root = source("zircon_runtime/src/text/font/mod.rs")
        fallback = source("zircon_runtime/src/text/font/fallback.rs")
        composite_resolve = source(
            "zircon_runtime/src/text/font/composite_resolve.rs"
        )

        self.assertIn("mod composite_resolve;", font_root)
        self.assertNotIn("fn candidate_families(", fallback)
        self.assertIn("pub(super) fn candidate_faces_for_cluster(", composite_resolve)

    def test_default_font_record_is_the_only_project_activation_call_site(self) -> None:
        text = source("zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs")
        font_assets = source(
            "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/font_assets.rs"
        )
        self.assertIn("composite_font: Option<CompositeFontDescriptor>", font_assets)
        self.assertNotIn("set_project_composite_font(", text)
        self.assertEqual(font_assets.count("set_project_composite_font("), 1)
        self.assertIn("fn apply_default_font_asset_projection(", font_assets)
        self.assertIn("asset_ref == super::DEFAULT_FONT_ASSET", font_assets)
        self.assertIn(
            "report.database_changed || report.asset_mapping_changed", font_assets
        )
        self.assertNotIn("publish_font_database", font_assets)

    def test_renderer_invalidation_consumes_semantic_font_asset_changes(self) -> None:
        text = source("zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs")
        render_state = source("zircon_runtime/src/text/render_state.rs")

        self.assertIn("if resolved_texts.font_faces_changed()", text)
        self.assertNotIn("text_state.publish_font_database()", text)
        self.assertNotIn("face_count_at_entry", text)
        self.assertNotIn("native_font_id_report.font_faces_changed", text)
        self.assertNotIn("fn register_font_source(", render_state)
        self.assertNotIn("fn publish_font_database(", render_state)
        self.assertIn("#[cfg(test)]\n    pub(crate) fn face_count(", render_state)

    def test_native_prepare_report_does_not_duplicate_font_change_state(self) -> None:
        text = source("zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs")
        fixtures = source(
            "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/tests/prepare_report.rs"
        )
        resolved_batches = source(
            "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/resolved_batches.rs"
        )

        native_report = between(
            text,
            "struct ScreenSpaceUiNativePrepareReport {",
            "}\n\nimpl ScreenSpaceUiTextSystem",
        )
        self.assertNotIn("font_faces_changed", native_report)
        self.assertNotIn("font_faces_changed:", fixtures)
        self.assertIn("font_faces_changed: bool", resolved_batches)

    def test_raster_worker_completion_is_face_owned_not_atlas_page_owned(self) -> None:
        native_atlas = source("zircon_runtime/src/text/native_bitmap_atlas.rs")
        source_cache = source(
            "zircon_runtime/src/text/native_bitmap_atlas/source_cache.rs"
        )
        raster_pool = source("zircon_runtime/src/text/parallel/raster_pool.rs")
        combined = native_atlas + source_cache + raster_pool

        self.assertIn("drain_completed_for_face_epoch(face_epoch)", native_atlas)
        self.assertIn("face_invalidated_count", source_cache)
        self.assertIn("disposition_for_face_epoch", raster_pool)
        for obsolete in (
            "TextRasterWorkTarget",
            "drain_completed_for_target",
            "stale_page_generation_ids",
            "stale_page_generation_count",
        ):
            self.assertNotIn(obsolete, combined)


if __name__ == "__main__":
    unittest.main()
