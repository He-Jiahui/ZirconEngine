import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
MANIFEST_SCHEMA = REPO_ROOT / "tools/plugin_structure_audits/manifest_schema.py"
OPTIONAL_FEATURE_SCHEMA = (
    REPO_ROOT / "tools/plugin_structure_audits/manifest_schema_optional_features.py"
)

MOVED_SYMBOLS = (
    "OPTIONAL_FEATURE_FIELDS",
    "collect_optional_features_schema_violations",
    "collect_optional_feature_schema_violations",
    "collect_optional_feature_identity_violations",
    "collect_optional_feature_dot_namespace_violations",
    "collect_optional_feature_capability_violations",
    "collect_optional_feature_capability_namespace_violations",
    "collect_optional_feature_dependency_schema_violations",
    "collect_optional_feature_dependency_primary_count_violation",
    "collect_optional_feature_dependency_duplicate_identity_violations",
    "collect_optional_feature_dependency_primary_target_violations",
    "collect_optional_feature_distribution_schema_violations",
)


def _line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8")
    return len(text.splitlines())


class PluginStructureAuditManifestSchemaOptionalFeatureOwnerBoundaryTests(
    unittest.TestCase
):
    def test_optional_feature_schema_owner_exists(self):
        self.assertTrue(
            OPTIONAL_FEATURE_SCHEMA.exists(),
            "optional feature manifest schema traversal belongs in its own owner file",
        )

    def test_optional_feature_schema_symbols_live_in_optional_owner(self):
        manifest_text = MANIFEST_SCHEMA.read_text(encoding="utf-8")
        optional_text = (
            OPTIONAL_FEATURE_SCHEMA.read_text(encoding="utf-8")
            if OPTIONAL_FEATURE_SCHEMA.exists()
            else ""
        )

        failures: list[str] = []
        for symbol in MOVED_SYMBOLS:
            if f"def {symbol}(" in manifest_text or f"{symbol} =" in manifest_text:
                failures.append(f"manifest_schema.py still owns {symbol}")
            if f"def {symbol}(" not in optional_text and f"{symbol} =" not in optional_text:
                failures.append(f"optional feature owner missing {symbol}")

        if failures:
            self.fail("\n".join(failures))

    def test_manifest_schema_dispatches_to_optional_feature_owner(self):
        manifest_text = MANIFEST_SCHEMA.read_text(encoding="utf-8")
        optional_text = (
            OPTIONAL_FEATURE_SCHEMA.read_text(encoding="utf-8")
            if OPTIONAL_FEATURE_SCHEMA.exists()
            else ""
        )

        self.assertIn(
            "from .manifest_schema_optional_features import (",
            manifest_text,
            "root manifest schema should dispatch optional_features to the focused owner",
        )
        self.assertNotIn(
            ".manifest_schema_optional_features",
            optional_text,
            "optional feature schema owner must not import itself or its parent dispatch edge",
        )

    def test_manifest_schema_and_optional_feature_owner_stay_under_line_budgets(self):
        self.assertLessEqual(
            _line_count(MANIFEST_SCHEMA),
            760,
            "manifest_schema.py should shrink after optional feature owner split",
        )
        self.assertTrue(
            OPTIONAL_FEATURE_SCHEMA.exists(),
            "optional feature manifest schema owner file is missing",
        )
        self.assertLessEqual(
            _line_count(OPTIONAL_FEATURE_SCHEMA),
            460,
            "optional feature manifest schema owner should stay focused",
        )


if __name__ == "__main__":
    unittest.main()
