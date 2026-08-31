import unittest
from pathlib import Path


class RuntimeRenderStatsPostProcessOwnerStructureTests(unittest.TestCase):
    def setUp(self) -> None:
        self.repo_root = Path(__file__).resolve().parents[2]
        self.owner_dir = (
            self.repo_root
            / "zircon_runtime/src/core/runtime/diagnostics/render_stats_store/post_process"
        )
        self.owner = self.owner_dir / "mod.rs"
        self.legacy_owner = self.owner_dir.with_suffix(".rs")

    def test_post_process_diagnostics_use_product_readback_owners(self) -> None:
        self.assertFalse(self.legacy_owner.exists(), self.legacy_owner)
        owner_source = self.owner.read_text(encoding="utf-8")
        production_lines = [
            line
            for line in owner_source.splitlines()
            if line.strip() and not line.lstrip().startswith("//")
        ]

        self.assertLessEqual(len(production_lines), 20)
        for declaration in (
            "mod color_lut;",
            "mod exposure;",
            '#[cfg(test)]\nmod tests;',
        ):
            self.assertIn(declaration, owner_source)
        self.assertIn(
            "exposure::record(store, frame_index, stats.last_exposure_readback_report);",
            owner_source,
        )
        self.assertIn(
            "color_lut::record(store, frame_index, stats.last_color_lut_readback_report);",
            owner_source,
        )
        for forbidden in (
            "record_count(",
            "record_bool(",
            "record_bytes(",
            '"render.post_process.',
        ):
            self.assertNotIn(forbidden, owner_source)

        expected_children = {
            "exposure.rs": (
                "RenderExposureReadbackReport",
                "render.post_process.exposure.readback.available",
                "render.post_process.exposure.readback.history_valid",
                "render.post_process.exposure.readback.multiplier_micro",
            ),
            "color_lut.rs": (
                "RenderColorLutReadbackReport",
                "render.post_process.color_lut.readback.available",
                "render.post_process.color_lut.readback.reference_kind",
                "render.post_process.color_lut.readback.identity_max_abs_error_micro",
            ),
        }
        production_sources = []
        for child_name, anchors in expected_children.items():
            child = self.owner_dir / child_name
            self.assertTrue(child.is_file(), child)
            child_source = child.read_text(encoding="utf-8")
            self.assertLess(child_source.count("\n") + 1, 220, child)
            self.assertIn("pub(super) fn record", child_source)
            for anchor in anchors:
                self.assertIn(anchor, child_source)
            production_sources.append(child_source)

        combined = "\n".join(production_sources)
        self.assertEqual(combined.count("record_bool("), 7)
        self.assertEqual(combined.count("record_bytes("), 4)
        self.assertEqual(combined.count("record_count("), 14)
        self.assertEqual(combined.count('"render.post_process.'), 25)

        tests_source = (self.owner_dir / "tests.rs").read_text(encoding="utf-8")
        for test_name in (
            "post_process_diagnostics_record_color_lut_readback_identity_report",
            "post_process_diagnostics_record_color_lut_readback_user_lut_reference_report",
            "post_process_diagnostics_record_exposure_readback_report",
        ):
            self.assertIn(test_name, tests_source)


if __name__ == "__main__":
    unittest.main()
