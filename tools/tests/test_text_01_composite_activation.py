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
        database = source("zircon_runtime/src/graphics/text/font/database.rs")
        registration = between(
            database,
            "    pub(crate) fn register_font_asset(",
            "    pub(crate) fn apply_system_font_policy(",
        )

        self.assertNotIn("active_composite_font", registration)
        self.assertNotIn("composite.default_family", registration)
        self.assertNotIn("composite.sub_fonts", registration)
        self.assertIn("pub(crate) fn set_project_composite_font(", database)
        self.assertIn("pub(crate) fn fallback_candidates_for_codepoint(", database)

    def test_composite_candidate_enumeration_is_a_folder_backed_leaf(self) -> None:
        font_root = source("zircon_runtime/src/graphics/text/font/mod.rs")
        fallback = source("zircon_runtime/src/graphics/text/font/fallback.rs")
        composite_resolve = source(
            "zircon_runtime/src/graphics/text/font/composite_resolve.rs"
        )

        self.assertIn("mod composite_resolve;", font_root)
        self.assertNotIn("fn candidate_families(", fallback)
        self.assertIn("pub(super) fn candidate_faces_for_cluster(", composite_resolve)

    def test_default_font_record_is_the_only_project_activation_call_site(self) -> None:
        text = source("zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs")
        font_assets = source(
            "zircon_runtime/src/graphics/scene/scene_renderer/ui/text/font_assets.rs"
        )
        constructor = between(
            text,
            "    pub(super) fn new(",
            "    pub(super) fn prepare(",
        )

        self.assertIn("composite_font: Option<CompositeFontDescriptor>", font_assets)
        self.assertEqual(constructor.count("set_project_composite_font("), 1)
        self.assertNotIn("set_project_composite_font(", font_assets)


if __name__ == "__main__":
    unittest.main()
