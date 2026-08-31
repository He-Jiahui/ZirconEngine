from pathlib import Path
import unittest

from tools.runtime_ui_text_decoration_source_map_pressure import (
    pressure_report,
    pressure_suite,
)


ROOT = Path(__file__).resolve().parents[2]
TEXT_GEOMETRY = ROOT / (
    "zircon_runtime_interface/src/ui/surface/render/text_geometry/mod.rs"
)


class RuntimeUiTextDecorationSourceMapPressureTests(unittest.TestCase):
    def test_localized_selection_materializes_only_touched_line_maps(self):
        report = pressure_report(65_536, 1, 3, 32)

        self.assertEqual(
            report["eager_all_line_maps"]["source_map_constructions"], 65_536
        )
        self.assertEqual(
            report["lazy_touched_line_maps"]["source_map_constructions"], 1
        )
        self.assertEqual(
            report["avoided"]["cluster_projection_visits"], 2_097_120
        )
        self.assertEqual(report["construction_reduction_ratio"], 65_536)
        self.assertEqual(
            report["eager_all_line_maps"]["line_range_probes"],
            report["lazy_touched_line_maps"]["line_range_probes"],
        )

    def test_default_scale_suite_is_not_product_timing(self):
        suite = pressure_suite([128, 4096, 65_536], 1, 3, 32)

        self.assertEqual(
            [scenario["construction_reduction_ratio"] for scenario in suite["scenarios"]],
            [128, 4096, 65_536],
        )
        self.assertFalse(suite["is_product_timing"])

    def test_rejects_invalid_pressure_inputs(self):
        with self.assertRaises(ValueError):
            pressure_report(0, 0, 0, 0)
        with self.assertRaises(ValueError):
            pressure_report(10, 11, 1, 1)
        with self.assertRaises(ValueError):
            pressure_report(10, 1, 0, 1)
        with self.assertRaises(ValueError):
            pressure_suite([], 0, 0, 0)

    def test_production_uses_touched_line_hash_cache_before_source_map_build(self):
        source = TEXT_GEOMETRY.read_text(encoding="utf-8")
        compact = "".join(source.split())

        self.assertIn("maps:HashMap<usize,UiTextLineSourceMap<'a>>", compact)
        self.assertIn(
            "range.start>=line.source_range.end||line.source_range.start>=range.end",
            compact,
        )
        self.assertIn("self.maps.entry(line_index)", source)
        self.assertNotIn("self.maps.get(&line_index)", source)
        self.assertNotIn(".map(UiTextLineSourceMap::new)", source)
        self.assertIn(
            "localized_selection_and_preedit_share_one_intersecting_line_source_map",
            source,
        )


if __name__ == "__main__":
    unittest.main()
