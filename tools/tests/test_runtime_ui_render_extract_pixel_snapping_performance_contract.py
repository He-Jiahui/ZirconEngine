from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
EXTRACT = ROOT / "zircon_runtime/src/ui/surface/render/extract.rs"
PIXEL_SNAPPING = (
    ROOT / "zircon_runtime/src/ui/surface/render/extract/pixel_snapping.rs"
)
PROFILE_MANIFEST = ROOT / "tools/profile-capture-manifest.ps1"
PRESSURE_MODEL = ROOT / "tools/runtime_ui_render_extract_pixel_snapping_pressure.py"


class RuntimeUiRenderExtractPixelSnappingPerformanceContractTests(unittest.TestCase):
    def test_pixel_snapping_is_a_cohesive_extract_submodule(self) -> None:
        extract = EXTRACT.read_text(encoding="utf-8")

        self.assertIn("mod pixel_snapping;", extract)
        self.assertIn(
            "pixel_snapping::apply_resolved_pixel_snapping_policies",
            extract,
        )
        self.assertNotIn("fn apply_resolved_pixel_snapping_policies", extract)

    def test_local_render_patch_resolves_only_command_ancestor_closure(self) -> None:
        self.assertTrue(PIXEL_SNAPPING.exists())
        source = (
            PIXEL_SNAPPING.read_text(encoding="utf-8")
            if PIXEL_SNAPPING.exists()
            else ""
        )

        self.assertIn("for command in commands.iter_mut()", source)
        self.assertIn("unresolved_path", source)
        self.assertIn("resolved.get(&current_id)", source)
        self.assertNotIn("tree.roots", source)
        self.assertNotIn("for node in tree.nodes", source)
        self.assertNotIn("tree.nodes.iter()", source)

    def test_lower_contract_rejects_unrelated_sibling_visits(self) -> None:
        source = (
            PIXEL_SNAPPING.read_text(encoding="utf-8")
            if PIXEL_SNAPPING.exists()
            else ""
        )

        self.assertIn(
            "single_command_does_not_visit_unrelated_siblings",
            source,
        )
        self.assertIn("assert_eq!(visited_node_count, 2)", source)
        self.assertIn("ui.render_extract.pixel_snapping_node_visit_count", source)

    def test_product_profile_is_source_bound_to_the_resolver(self) -> None:
        manifest = PROFILE_MANIFEST.read_text(encoding="utf-8")

        self.assertIn(
            '"zircon_runtime/src/ui/surface/render/extract/pixel_snapping.rs"',
            manifest,
        )

    def test_pressure_model_separates_local_patch_from_full_extract(self) -> None:
        self.assertTrue(PRESSURE_MODEL.exists())
        from tools.runtime_ui_render_extract_pixel_snapping_pressure import run

        result = run(
            tree_node_count=16_384,
            changed_command_node_count=1,
            ancestor_depth=8,
            update_count=4_096,
        )

        self.assertEqual(result["retired_full_tree_scan"]["node_visits"], 67_108_864)
        self.assertEqual(result["command_ancestor_closure"]["node_visits"], 32_768)
        self.assertEqual(result["delta"]["node_visit_reduction_ratio"], 2_048.0)
        self.assertFalse(result["interpretation"]["cpu_or_latency_measured"])


if __name__ == "__main__":
    unittest.main()
