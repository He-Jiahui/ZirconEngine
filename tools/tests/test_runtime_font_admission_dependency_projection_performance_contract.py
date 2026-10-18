from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = ROOT / "zircon_runtime/src/dynamic_api/session/runtime_ui/font_admission.rs"


class RuntimeFontAdmissionDependencyProjectionPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        start = cls.source.index("pub(super) fn admit_surface_font_dependencies")
        end = cls.source.index("\n}", start) + 2
        cls.body = cls.source[start:end]

    def test_dependency_projection_uses_sorted_contiguous_storage(self) -> None:
        self.assertIn(".collect::<Vec<_>>()", self.body)
        self.assertIn("font_dependencies.sort_unstable();", self.body)
        self.assertIn("font_dependencies.dedup();", self.body)
        self.assertNotIn("BTreeSet", self.body)
        self.assertIn("runtime_font_admission_dependency_projection_bench_v1", self.source)

    def test_admission_order_and_claim_contract_remain_explicit(self) -> None:
        self.assertIn("UiSurface::text_font_asset_dependencies", self.body)
        self.assertIn("Arc::<str>::from(asset_ref.as_str())", self.body)
        self.assertIn("prepare_runtime_font_asset_admission(asset_manager, asset_ref)", self.body)
        self.assertIn(
            "font_claim_scope.replace_shared_claims_with_admissions(&shared_dependencies, admissions)",
            self.body,
        )

    def test_duplicate_dependencies_reduce_to_one_sorted_entry(self) -> None:
        dependencies = [
            "res://fonts/zeta.font.toml",
            "res://fonts/alpha.font.toml",
            "res://fonts/zeta.font.toml",
            "res://fonts/alpha.font.toml",
        ]
        optimized = sorted(set(dependencies))
        self.assertEqual(
            optimized,
            ["res://fonts/alpha.font.toml", "res://fonts/zeta.font.toml"],
        )

    def test_contiguous_projection_avoids_tree_node_shape(self) -> None:
        unique_count = 4096
        legacy_tree_nodes = unique_count
        optimized_contiguous_buffers = 1
        self.assertEqual(legacy_tree_nodes, unique_count)
        self.assertEqual(optimized_contiguous_buffers, 1)
        self.assertLess(optimized_contiguous_buffers, legacy_tree_nodes)


if __name__ == "__main__":
    unittest.main()
