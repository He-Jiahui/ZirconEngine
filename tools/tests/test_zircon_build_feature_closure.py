import unittest
from pathlib import Path

from tools.build.zircon_build_feature_closure import (
    audit_product_feature_closure,
    resolve_feature_closure,
)


REPO_ROOT = Path(__file__).resolve().parents[2]


class FeatureClosureTests(unittest.TestCase):
    def test_runtime_shipping_closure_reports_current_dynamic_api_boundary(self):
        result = audit_product_feature_closure(
            REPO_ROOT,
            app_features=("runtime-product",),
            runtime_features=("shipping-runtime",),
        )
        self.assertEqual(result["status"], "needs-split")
        self.assertIn("text", result["heavy_features"])
        self.assertIn("ui", result["heavy_features"])

    def test_local_feature_aliases_and_dependency_edges_are_sorted(self):
        closure = resolve_feature_closure(
            REPO_ROOT / "zircon_runtime" / "Cargo.toml",
            ("shipping-runtime",),
            package_name="zircon_runtime",
        )
        self.assertEqual(tuple(sorted(closure.local_features)), closure.local_features)
        self.assertIn("platform-winit", closure.local_features)
        self.assertIn("dep:winit", closure.dependency_edges)


if __name__ == "__main__":
    unittest.main()
