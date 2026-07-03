import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PLUGIN_VALIDATE_OWNER_BOUNDARIES_TEST = (
    REPO_ROOT / "tools/tests/test_plugin_validate_owner_boundaries.py"
)
PLUGIN_VALIDATE_SINGLE_TARGET = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_single_target.py"
)
PLUGIN_VALIDATE_COMMON = REPO_ROOT / "tools/zircon_export/plugin_validate_common.py"
PLUGIN_VALIDATE_LAYOUT = REPO_ROOT / "tools/zircon_export/plugin_validate_layout.py"
PLUGIN_VALIDATE_MANIFEST_SHAPE = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_manifest_shape.py"
)
PLUGIN_VALIDATE_MANIFEST_CLASSIFICATION = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_manifest_classification.py"
)
PLUGIN_VALIDATE_PACKAGE_KIND = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_package_kind.py"
)
PLUGIN_VALIDATE_TEST = REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate.py"
PLUGIN_VALIDATE_LAYOUT_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate_layout.py"
)
PLUGIN_VALIDATE_MANIFEST_SHAPE_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate_manifest_shape.py"
)
PLUGIN_VALIDATE_MANIFEST_CLASSIFICATION_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_plugin_validate_manifest_classification.py"
)
PLUGIN_VALIDATE_PACKAGE_KIND_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate_package_kind.py"
)

MANIFEST_BOUNDARY_METHODS = (
    "test_manifest_shape_tests_live_in_manifest_shape_test_owner",
    "test_manifest_shape_lives_in_manifest_shape_owner",
    "test_manifest_classification_tests_live_in_manifest_classification_test_owner",
    "test_manifest_classification_lives_in_manifest_classification_owner",
    "test_package_kind_tests_live_in_package_kind_test_owner",
    "test_package_kind_lives_in_package_kind_owner",
)


class PluginValidateManifestOwnerBoundaryTests(unittest.TestCase):
    def test_manifest_boundaries_leave_general_owner_file(self):
        general_owner_text = PLUGIN_VALIDATE_OWNER_BOUNDARIES_TEST.read_text(
            encoding="utf-8"
        )

        for method_name in MANIFEST_BOUNDARY_METHODS:
            self.assertNotIn(
                f"def {method_name}(",
                general_owner_text,
                f"{method_name} belongs in test_plugin_validate_manifest_owner_boundaries.py",
            )

        self.assertLessEqual(
            len(general_owner_text.splitlines()),
            2100,
            "general PluginValidate owner boundary tests should shrink after manifest split",
        )
        self.assertLessEqual(
            len(Path(__file__).read_text(encoding="utf-8").splitlines()),
            400,
            "focused PluginValidate manifest owner boundary file should stay narrow",
        )

    def test_manifest_shape_tests_live_in_manifest_shape_test_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_MANIFEST_SHAPE_TEST.exists(),
            "manifest shape behavior tests belong in test_plugin_validate_manifest_shape.py",
        )
        validate_test_text = PLUGIN_VALIDATE_TEST.read_text(encoding="utf-8")
        layout_test_text = PLUGIN_VALIDATE_LAYOUT_TEST.read_text(encoding="utf-8")
        manifest_shape_test_text = PLUGIN_VALIDATE_MANIFEST_SHAPE_TEST.read_text(
            encoding="utf-8"
        )

        for test_name in (
            "test_plugin_validate_rejects_manifest_identity_and_display_name_shape",
            "test_plugin_validate_rejects_manifest_version_shape",
            "test_plugin_validate_rejects_manifest_version_numeric_boundaries",
            "test_plugin_validate_rejects_unknown_root_manifest_fields",
        ):
            for source_name, source_text in (
                ("test_plugin_validate.py", validate_test_text),
                ("test_plugin_validate_layout.py", layout_test_text),
            ):
                self.assertNotIn(
                    f"def {test_name}(",
                    source_text,
                    f"{test_name} belongs in the manifest shape test owner, not {source_name}",
                )
            self.assertIn(f"def {test_name}(", manifest_shape_test_text)

    def test_manifest_shape_lives_in_manifest_shape_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_MANIFEST_SHAPE.exists(),
            "root manifest id/version/display_name checks belong in plugin_validate_manifest_shape.py",
        )
        single_target_text = PLUGIN_VALIDATE_SINGLE_TARGET.read_text(encoding="utf-8")
        common_text = PLUGIN_VALIDATE_COMMON.read_text(encoding="utf-8")
        layout_text = PLUGIN_VALIDATE_LAYOUT.read_text(encoding="utf-8")
        manifest_shape_text = PLUGIN_VALIDATE_MANIFEST_SHAPE.read_text(
            encoding="utf-8"
        )

        for symbol in (
            "PLUGIN_VALIDATE_MANIFEST_VERSION_FIELDS",
            "PLUGIN_VALIDATE_MANIFEST_ROOT_FIELDS",
            "PLUGIN_VALIDATE_U32_MAX",
            "validate_plugin_manifest_shape",
            "validate_plugin_manifest_known_fields",
            "plugin_validate_manifest_identity",
            "plugin_validate_manifest_semver",
            "plugin_validate_manifest_semver_component",
            "is not a known manifest root field",
            "must contain only lowercase ASCII letters, digits, underscores, and dots in non-empty segments",
            "must start with a lowercase ASCII letter",
            "segments must not end with an underscore or contain repeated underscores",
            "must use MAJOR.MINOR.PATCH form",
            "must contain ASCII digits",
            "must not use leading zeroes",
            "must fit in u32",
        ):
            self.assertIn(symbol, manifest_shape_text)
            for source_name, source_text in (
                ("plugin_validate_common.py", common_text),
                ("plugin_validate_layout.py", layout_text),
                ("plugin_validate_single_target.py", single_target_text),
            ):
                if symbol == "validate_plugin_manifest_shape" and source_name == "plugin_validate_single_target.py":
                    continue
                self.assertNotIn(
                    symbol,
                    source_text,
                    f"{symbol} belongs in the manifest shape owner, not {source_name}",
                )

        self.assertIn(
            "from .plugin_validate_manifest_shape import",
            single_target_text,
            "single-target owner should dispatch root manifest shape checks",
        )
        self.assertIn("validate_plugin_manifest_shape(", single_target_text)
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_single_target import",
            "from .plugin_validate_layout import",
            "from .plugin_validate_modules import",
            "from .plugin_validate_capability_statuses import",
        ):
            self.assertNotIn(
                forbidden_import,
                manifest_shape_text,
                "manifest shape owner must stay independent from entry and sibling owners",
            )
        self.assertLessEqual(
            len(manifest_shape_text.splitlines()),
            180,
            "manifest shape owner should stay small while owning root field closure",
        )

    def test_manifest_classification_tests_live_in_manifest_classification_test_owner(
        self,
    ):
        self.assertTrue(
            PLUGIN_VALIDATE_MANIFEST_CLASSIFICATION_TEST.exists(),
            "manifest classification behavior tests belong in test_plugin_validate_manifest_classification.py",
        )
        validate_test_text = PLUGIN_VALIDATE_TEST.read_text(encoding="utf-8")
        layout_test_text = PLUGIN_VALIDATE_LAYOUT_TEST.read_text(encoding="utf-8")
        manifest_shape_test_text = PLUGIN_VALIDATE_MANIFEST_SHAPE_TEST.read_text(
            encoding="utf-8"
        )
        classification_test_text = (
            PLUGIN_VALIDATE_MANIFEST_CLASSIFICATION_TEST.read_text(encoding="utf-8")
        )

        for test_name in (
            "test_plugin_validate_rejects_missing_manifest_maturity",
            "test_plugin_validate_rejects_unknown_manifest_maturity",
            "test_plugin_validate_rejects_unknown_manifest_category",
        ):
            for source_name, source_text in (
                ("test_plugin_validate.py", validate_test_text),
                ("test_plugin_validate_layout.py", layout_test_text),
                ("test_plugin_validate_manifest_shape.py", manifest_shape_test_text),
            ):
                self.assertNotIn(
                    f"def {test_name}(",
                    source_text,
                    f"{test_name} belongs in the manifest classification test owner, not {source_name}",
                )
            self.assertIn(f"def {test_name}(", classification_test_text)

    def test_manifest_classification_lives_in_manifest_classification_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_MANIFEST_CLASSIFICATION.exists(),
            "root manifest maturity classification checks belong in plugin_validate_manifest_classification.py",
        )
        single_target_text = PLUGIN_VALIDATE_SINGLE_TARGET.read_text(encoding="utf-8")
        common_text = PLUGIN_VALIDATE_COMMON.read_text(encoding="utf-8")
        layout_text = PLUGIN_VALIDATE_LAYOUT.read_text(encoding="utf-8")
        manifest_shape_text = PLUGIN_VALIDATE_MANIFEST_SHAPE.read_text(
            encoding="utf-8"
        )
        classification_text = PLUGIN_VALIDATE_MANIFEST_CLASSIFICATION.read_text(
            encoding="utf-8"
        )

        for symbol in (
            "PLUGIN_VALIDATE_MANIFEST_CATEGORY_VALUES",
            "PLUGIN_VALIDATE_MANIFEST_MATURITY_VALUES",
            "validate_plugin_manifest_classification",
            "validate_plugin_manifest_category",
            "validate_plugin_manifest_maturity",
            "is unsupported; expected one of asset_importer, authoring, diagnostics, platform, rendering, runtime, sdk",
            "is unsupported; expected one of stable, beta, experimental",
        ):
            self.assertIn(symbol, classification_text)
            for source_name, source_text in (
                ("plugin_validate_common.py", common_text),
                ("plugin_validate_layout.py", layout_text),
                ("plugin_validate_manifest_shape.py", manifest_shape_text),
                ("plugin_validate_single_target.py", single_target_text),
            ):
                if (
                    symbol == "validate_plugin_manifest_classification"
                    and source_name == "plugin_validate_single_target.py"
                ):
                    continue
                self.assertNotIn(
                    symbol,
                    source_text,
                    f"{symbol} belongs in the manifest classification owner, not {source_name}",
                )

        self.assertIn(
            "from .plugin_validate_manifest_classification import",
            single_target_text,
            "single-target owner should dispatch root manifest classification checks",
        )
        self.assertIn("validate_plugin_manifest_classification(", single_target_text)
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_single_target import",
            "from .plugin_validate_layout import",
            "from .plugin_validate_modules import",
            "from .plugin_validate_manifest_shape import",
        ):
            self.assertNotIn(
                forbidden_import,
                classification_text,
                "manifest classification owner must stay independent from entry and sibling owners",
            )
        self.assertLessEqual(
            len(classification_text.splitlines()),
            100,
            "manifest classification owner should stay small and focused",
        )

    def test_package_kind_tests_live_in_package_kind_test_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_PACKAGE_KIND_TEST.exists(),
            "package kind behavior tests belong in test_plugin_validate_package_kind.py",
        )
        validate_test_text = PLUGIN_VALIDATE_TEST.read_text(encoding="utf-8")
        manifest_shape_test_text = PLUGIN_VALIDATE_MANIFEST_SHAPE_TEST.read_text(
            encoding="utf-8"
        )
        package_kind_test_text = PLUGIN_VALIDATE_PACKAGE_KIND_TEST.read_text(
            encoding="utf-8"
        )

        for test_name in (
            "test_plugin_validate_rejects_unknown_package_kind",
            "test_plugin_validate_rejects_feature_extension_package_kind_without_rows",
            "test_plugin_validate_rejects_standard_package_kind_with_feature_rows",
        ):
            for source_name, source_text in (
                ("test_plugin_validate.py", validate_test_text),
                ("test_plugin_validate_manifest_shape.py", manifest_shape_test_text),
            ):
                self.assertNotIn(
                    f"def {test_name}(",
                    source_text,
                    f"{test_name} belongs in the package kind test owner, not {source_name}",
                )
            self.assertIn(f"def {test_name}(", package_kind_test_text)

    def test_package_kind_lives_in_package_kind_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_PACKAGE_KIND.exists(),
            "root package_kind validation belongs in plugin_validate_package_kind.py",
        )
        single_target_text = PLUGIN_VALIDATE_SINGLE_TARGET.read_text(encoding="utf-8")
        common_text = PLUGIN_VALIDATE_COMMON.read_text(encoding="utf-8")
        manifest_shape_text = PLUGIN_VALIDATE_MANIFEST_SHAPE.read_text(
            encoding="utf-8"
        )
        package_kind_text = PLUGIN_VALIDATE_PACKAGE_KIND.read_text(encoding="utf-8")

        for symbol in (
            "PLUGIN_VALIDATE_PACKAGE_KIND_VALUES",
            "validate_plugin_package_kind",
            "validate_plugin_package_kind_coherence",
            "plugin_validate_table_array_row_count",
            "should be standard or feature_extension",
            "should declare at least one feature_extensions row",
            "should not declare feature_extensions rows",
        ):
            self.assertIn(symbol, package_kind_text)
            for source_name, source_text in (
                ("plugin_validate_common.py", common_text),
                ("plugin_validate_manifest_shape.py", manifest_shape_text),
                ("plugin_validate_single_target.py", single_target_text),
            ):
                if (
                    symbol == "validate_plugin_package_kind"
                    and source_name == "plugin_validate_single_target.py"
                ):
                    continue
                self.assertNotIn(
                    symbol,
                    source_text,
                    f"{symbol} belongs in the package kind owner, not {source_name}",
                )

        self.assertIn(
            "from .plugin_validate_package_kind import",
            single_target_text,
            "single-target owner should dispatch package kind checks",
        )
        self.assertIn("validate_plugin_package_kind(", single_target_text)
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_single_target import",
            "from .plugin_validate_manifest_shape import",
            "from .plugin_validate_manifest_classification import",
        ):
            self.assertNotIn(
                forbidden_import,
                package_kind_text,
                "package kind owner must stay independent from entry and sibling owners",
            )
        self.assertLessEqual(
            len(package_kind_text.splitlines()),
            120,
            "package kind owner should stay small and focused",
        )


if __name__ == "__main__":
    unittest.main()
