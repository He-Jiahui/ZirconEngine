from pathlib import Path
import unittest


ROOT = Path(__file__).resolve().parents[2]
SOURCE = (
    ROOT
    / "zircon_runtime/src/plugin/runtime_plugin/package_validation/projection/build.rs"
)
REGRESSIONS = (
    ROOT
    / "zircon_runtime/src/plugin/runtime_plugin/package_validation/projection/tests.rs"
)


def function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    opening = source.index("{", start)
    depth = 0
    for index in range(opening, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening + 1 : index]
    raise AssertionError(f"unterminated function: {signature}")


class PreallocatedPackageIdentityIndexPerformanceContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.source = SOURCE.read_text(encoding="utf-8")
        cls.build = function_body(cls.source, "pub(in crate::plugin::runtime_plugin) fn build(")
        cls.regressions = REGRESSIONS.read_text(encoding="utf-8")

    def test_seen_identity_index_uses_exact_row_capacity(self) -> None:
        self.assertIn(
            "let identity_row_capacity = package_identity_row_capacity(package_manifest);",
            self.build,
        )
        self.assertIn("HashSet::with_capacity(identity_row_capacity)", self.build)
        self.assertIn("let mut duplicates = HashSet::new()", self.build)

    def test_capacity_covers_every_indexed_manifest_domain(self) -> None:
        capacity = function_body(self.source, "fn package_identity_row_capacity(")
        for field in (
            "capabilities",
            "asset_roots",
            "content_roots",
            "asset_importers",
            "dependencies",
            "capability_statuses",
            "options",
            "event_catalogs",
            "components",
            "ui_components",
            "optional_features",
            "feature_extensions",
            "provides_interfaces",
            "modules",
        ):
            self.assertIn(field, capacity)

    def test_capacity_matches_rows_and_order_regressions_remain(self) -> None:
        self.assertIn(
            "debug_assert_eq!(identity_rows_indexed, identity_row_capacity)",
            self.build,
        )
        self.assertIn("duplicate_ordinals_are_scoped_to_their_manifest_domain", self.regressions)
        self.assertIn("registration_projection_retains_manifest_order_and_membership", self.regressions)


if __name__ == "__main__":
    unittest.main()
