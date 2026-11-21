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

    def test_runtime_cooked_features_select_zstd_without_native_host(self):
        manifest = REPO_ROOT / "zircon_runtime" / "Cargo.toml"
        # These are local feature requests, not a target-resolved dependency graph.
        # target-web may request dep:zstd without selecting the crate on wasm32.
        for feature, expects_cooked_assets in (
            ("ui", True),
            ("graphics", True),
            ("cooked-assets", True),
            ("target-web", True),
            ("asset-types", False),
            ("core-min", False),
        ):
            with self.subTest(feature=feature):
                closure = resolve_feature_closure(manifest, (feature,))
                self.assertIn("core-min", closure.local_features)
                self.assertNotIn("native-host", closure.local_features)
                self.assertNotIn("dep:zip", closure.dependency_edges)
                self.assertEqual(
                    "cooked-assets" in closure.local_features, expects_cooked_assets
                )
                if expects_cooked_assets:
                    self.assertIn("asset-types", closure.local_features)
                    self.assertIn("dep:zstd", closure.dependency_edges)
                else:
                    self.assertNotIn("dep:zstd", closure.dependency_edges)


if __name__ == "__main__":
    unittest.main()
