from pathlib import Path
import unittest

from tools.runtime_ui_render_prewarm_scan_pressure import run


ROOT = Path(__file__).resolve().parents[2]
EXTRACT = ROOT / "zircon_runtime/src/ui/surface/render/extract.rs"
OWNER_TEXT_PREWARM = (
    ROOT / "zircon_runtime/src/ui/surface/render/extract/owner_text_prewarm.rs"
)


class RuntimeUiRenderPrewarmScanContractTests(unittest.TestCase):
    def test_collection_owns_overlap_admission_state(self):
        owner = EXTRACT.read_text(encoding="utf-8")
        collection = OWNER_TEXT_PREWARM.read_text(encoding="utf-8")
        self.assertIn("mod owner_text_prewarm;", owner)
        self.assertIn("struct OwnerTextPrewarmCollection", collection)
        self.assertIn("can_overlap_render_commands", collection)
        self.assertIn("collect_owner_text_prewarm_requests", collection)
        self.assertIn("owner_prewarm_collection.requests.len()", owner)
        self.assertNotIn("render_command_build_can_overlap_owner_prewarm", owner)
        self.assertNotIn("render_command_build_can_overlap_owner_prewarm", collection)

        collection_body = collection.split(
            "fn collect_owner_text_prewarm_requests", 1
        )[1].split("fn component_text_requires_shared_cache", 1)[0]
        self.assertEqual(
            collection_body.count("arranged_tree.draw_order.iter().copied()"), 1
        )
        self.assertIn(
            "component_text_requires_shared_cache(node.template_metadata.as_ref())",
            collection_body,
        )

    def test_pressure_model_counts_the_removed_admission_scan(self):
        result = run(
            node_count=100_000,
            full_extract_count=1_000,
            eligible_request_count=32,
            overlap_threshold=8,
        )
        self.assertTrue(result["admission_scan_enabled"])
        self.assertEqual(result["old_overlap_admission_visits"], 100_000_000)
        self.assertEqual(result["eliminated_overlap_admission_visits"], 100_000_000)
        self.assertEqual(result["old_total_draw_order_visits"], 300_000_000)
        self.assertEqual(result["new_total_draw_order_visits"], 200_000_000)
        self.assertAlmostEqual(result["draw_order_visit_reduction_ratio"], 1.5)

    def test_model_keeps_subthreshold_work_unchanged(self):
        result = run(
            node_count=10,
            full_extract_count=4,
            eligible_request_count=7,
            overlap_threshold=8,
        )
        self.assertFalse(result["admission_scan_enabled"])
        self.assertEqual(result["eliminated_draw_order_visits"], 0)
        self.assertEqual(result["old_total_draw_order_visits"], 80)
        self.assertEqual(result["new_total_draw_order_visits"], 80)

    def test_model_rejects_invalid_inputs(self):
        with self.assertRaises(ValueError):
            run(0, 1, 1)
        with self.assertRaises(ValueError):
            run(1, 0, 1)
        with self.assertRaises(ValueError):
            run(1, 1, 0)
        with self.assertRaises(ValueError):
            run(1, 1, 1, 0)


if __name__ == "__main__":
    unittest.main()
