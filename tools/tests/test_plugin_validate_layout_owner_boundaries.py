import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PLUGIN_VALIDATE_OWNER_BOUNDARIES_TEST = (
    REPO_ROOT / "tools/tests/test_plugin_validate_owner_boundaries.py"
)
PLUGIN_VALIDATE_SINGLE_TARGET = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_single_target.py"
)
PLUGIN_VALIDATE_LAYOUT = REPO_ROOT / "tools/zircon_export/plugin_validate_layout.py"
PLUGIN_VALIDATE_LAYOUT_COORDINATES = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_layout_coordinates.py"
)
PLUGIN_VALIDATE_LAYOUT_PUBLIC_METADATA = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_layout_public_metadata.py"
)
PLUGIN_VALIDATE_LAYOUT_TARGETS = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_layout_targets.py"
)
PLUGIN_VALIDATE_LAYOUT_ROOTS = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_layout_roots.py"
)
PLUGIN_VALIDATE_TEST = REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate.py"
PLUGIN_VALIDATE_LAYOUT_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate_layout.py"
)

LAYOUT_BOUNDARY_METHODS = (
    "test_layout_tests_live_in_layout_test_owner",
    "test_layout_coordinates_live_in_layout_coordinates_owner",
    "test_layout_public_metadata_lives_in_layout_public_metadata_owner",
    "test_layout_targets_live_in_layout_targets_owner",
    "test_layout_roots_live_in_layout_roots_owner",
)


class PluginValidateLayoutOwnerBoundaryTests(unittest.TestCase):
    def test_layout_boundaries_leave_general_owner_file(self):
        general_owner_text = PLUGIN_VALIDATE_OWNER_BOUNDARIES_TEST.read_text(
            encoding="utf-8"
        )

        for method_name in LAYOUT_BOUNDARY_METHODS:
            self.assertNotIn(
                f"def {method_name}(",
                general_owner_text,
                f"{method_name} belongs in test_plugin_validate_layout_owner_boundaries.py",
            )

        self.assertLessEqual(
            len(general_owner_text.splitlines()),
            2360,
            "general PluginValidate owner boundary tests should shrink after layout split",
        )
        self.assertLessEqual(
            len(Path(__file__).read_text(encoding="utf-8").splitlines()),
            360,
            "focused PluginValidate layout owner boundary file should stay narrow",
        )

    def test_layout_tests_live_in_layout_test_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_LAYOUT_TEST.exists(),
            "layout behavior tests belong in test_plugin_validate_layout.py",
        )
        validate_test_text = PLUGIN_VALIDATE_TEST.read_text(encoding="utf-8")
        layout_test_text = PLUGIN_VALIDATE_LAYOUT_TEST.read_text(encoding="utf-8")

        for test_name in (
            "test_plugin_validate_rejects_package_coordinate_drift",
            "test_plugin_validate_rejects_layout_public_metadata_drift",
            "test_plugin_validate_rejects_layout_target_and_platform_drift",
            "test_plugin_validate_rejects_layout_root_path_drift",
            "test_plugin_validate_rejects_layout_root_drive_separator_drift",
        ):
            self.assertNotIn(
                f"def {test_name}(",
                validate_test_text,
                f"{test_name} belongs in the layout test owner",
            )
            self.assertIn(f"def {test_name}(", layout_test_text)

    def test_layout_coordinates_live_in_layout_coordinates_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_LAYOUT.exists(),
            "package layout orchestration belongs in plugin_validate_layout.py",
        )
        self.assertTrue(
            PLUGIN_VALIDATE_LAYOUT_COORDINATES.exists(),
            "package coordinate checks belong in plugin_validate_layout_coordinates.py",
        )
        single_target_text = PLUGIN_VALIDATE_SINGLE_TARGET.read_text(encoding="utf-8")
        layout_text = PLUGIN_VALIDATE_LAYOUT.read_text(encoding="utf-8")
        coordinates_text = PLUGIN_VALIDATE_LAYOUT_COORDINATES.read_text(
            encoding="utf-8"
        )

        for symbol in (
            "validate_plugin_layout_coordinates",
            "validate_plugin_layout_coordinate_prefix",
            "validate_plugin_layout_coordinate_segment",
            "package coordinates must declare package_prefix, package_company, and package_name together or leave all empty",
            "must contain only non-empty lowercase coordinate segments",
            "must be a non-empty lowercase coordinate segment",
        ):
            self.assertIn(symbol, coordinates_text)
        for function_name in (
            "validate_plugin_layout_coordinates",
            "validate_plugin_layout_coordinate_prefix",
            "validate_plugin_layout_coordinate_segment",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                layout_text,
                f"{function_name} belongs in plugin_validate_layout_coordinates.py",
            )
            self.assertNotIn(
                f"def {function_name}(",
                single_target_text,
                f"{function_name} belongs in plugin_validate_layout_coordinates.py",
            )
        self.assertIn(
            "from .plugin_validate_layout_coordinates import",
            layout_text,
            "layout owner should dispatch coordinate checks to the coordinate owner",
        )
        self.assertIn(
            "from .plugin_validate_layout import",
            single_target_text,
            "single-target owner should dispatch root package layout checks",
        )
        self.assertIn("validate_plugin_layout(", single_target_text)
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_single_target import",
            "from .plugin_validate_modules import",
            "from .plugin_validate_capability_statuses import",
        ):
            self.assertNotIn(
                forbidden_import,
                coordinates_text,
                "layout coordinate owner must stay independent from entry and sibling owners",
            )
        self.assertLessEqual(
            len(layout_text.splitlines()),
            80,
            "layout owner should stay a small root manifest dispatcher",
        )
        self.assertLessEqual(
            len(coordinates_text.splitlines()),
            120,
            "layout coordinate owner should stay small and focused",
        )

    def test_layout_public_metadata_lives_in_layout_public_metadata_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_LAYOUT_PUBLIC_METADATA.exists(),
            "package public metadata checks belong in plugin_validate_layout_public_metadata.py",
        )
        layout_text = PLUGIN_VALIDATE_LAYOUT.read_text(encoding="utf-8")
        public_metadata_text = PLUGIN_VALIDATE_LAYOUT_PUBLIC_METADATA.read_text(
            encoding="utf-8"
        )

        for symbol in (
            "validate_plugin_layout_public_metadata",
            "validate_plugin_layout_description",
            "must be trimmed when present",
            "category",
            "description",
        ):
            self.assertIn(symbol, public_metadata_text)
        for function_name in (
            "validate_plugin_layout_public_metadata",
            "validate_plugin_layout_description",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                layout_text,
                f"{function_name} belongs in plugin_validate_layout_public_metadata.py",
            )
        self.assertIn(
            "from .plugin_validate_layout_public_metadata import",
            layout_text,
            "layout owner should dispatch public metadata checks to the public metadata owner",
        )
        self.assertIn("validate_plugin_layout_public_metadata(", layout_text)
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
                public_metadata_text,
                "layout public metadata owner must stay independent from entry and sibling owners",
            )
        self.assertLessEqual(
            len(public_metadata_text.splitlines()),
            90,
            "layout public metadata owner should stay small and focused",
        )

    def test_layout_targets_live_in_layout_targets_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_LAYOUT_TARGETS.exists(),
            "package target/platform checks belong in plugin_validate_layout_targets.py",
        )
        layout_text = PLUGIN_VALIDATE_LAYOUT.read_text(encoding="utf-8")
        targets_text = PLUGIN_VALIDATE_LAYOUT_TARGETS.read_text(encoding="utf-8")

        for symbol in (
            "PLUGIN_VALIDATE_LAYOUT_SUPPORTED_TARGETS",
            "PLUGIN_VALIDATE_LAYOUT_SUPPORTED_PLATFORMS",
            "validate_plugin_layout_targets",
            "validate_plugin_layout_string_set",
            "supported_targets",
            "supported_platforms",
            "unsupported; expected one of",
            "duplicates supported_targets",
            "duplicates supported_platforms",
        ):
            self.assertIn(symbol, targets_text)
        for function_name in (
            "validate_plugin_layout_targets",
            "validate_plugin_layout_string_set",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                layout_text,
                f"{function_name} belongs in plugin_validate_layout_targets.py",
            )
        self.assertIn(
            "from .plugin_validate_layout_targets import",
            layout_text,
            "layout owner should dispatch target/platform checks to the target owner",
        )
        self.assertIn("validate_plugin_layout_targets(", layout_text)
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
                targets_text,
                "layout targets owner must stay independent from entry and sibling owners",
            )
        self.assertLessEqual(
            len(targets_text.splitlines()),
            120,
            "layout targets owner should stay small and focused",
        )

    def test_layout_roots_live_in_layout_roots_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_LAYOUT_ROOTS.exists(),
            "package root path checks belong in plugin_validate_layout_roots.py",
        )
        layout_text = PLUGIN_VALIDATE_LAYOUT.read_text(encoding="utf-8")
        roots_text = PLUGIN_VALIDATE_LAYOUT_ROOTS.read_text(encoding="utf-8")

        for symbol in (
            "PLUGIN_VALIDATE_LAYOUT_ROOT_FIELDS",
            "validate_plugin_layout_roots",
            "validate_plugin_layout_root_array",
            "validate_plugin_layout_root_path",
            "asset_roots",
            "content_roots",
            "must be relative",
            "must use forward slashes",
            "must not contain a drive separator",
            "must not contain empty, current, or parent path segments",
            "duplicates asset_roots",
        ):
            self.assertIn(symbol, roots_text)
        for function_name in (
            "validate_plugin_layout_roots",
            "validate_plugin_layout_root_array",
            "validate_plugin_layout_root_path",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                layout_text,
                f"{function_name} belongs in plugin_validate_layout_roots.py",
            )
        self.assertIn(
            "from .plugin_validate_layout_roots import",
            layout_text,
            "layout owner should dispatch root path checks to the roots owner",
        )
        self.assertIn("validate_plugin_layout_roots(", layout_text)
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
                roots_text,
                "layout roots owner must stay independent from entry and sibling owners",
            )
        self.assertLessEqual(
            len(roots_text.splitlines()),
            120,
            "layout roots owner should stay small and focused",
        )


if __name__ == "__main__":
    unittest.main()
