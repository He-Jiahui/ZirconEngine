import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PLUGIN_VALIDATE_OWNER_BOUNDARIES_TEST = (
    REPO_ROOT / "tools/tests/test_plugin_validate_owner_boundaries.py"
)
PLUGIN_VALIDATE = REPO_ROOT / "tools/zircon_export/plugin_validate.py"
PLUGIN_VALIDATE_COMMON = REPO_ROOT / "tools/zircon_export/plugin_validate_common.py"
PLUGIN_VALIDATE_DIST_CRATE = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_dist_crate.py"
)
PLUGIN_VALIDATE_DIST_CRATE_DEPENDENCY = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_dist_crate_dependency.py"
)

DIST_CRATE_BOUNDARY_METHODS = (
    "test_dist_crate_cargo_preflight_lives_in_dist_crate_owner",
    "test_dist_crate_sdk_dependency_lives_in_dependency_owner",
    "test_dist_crate_workspace_member_resolution_lives_in_dist_crate_owner",
    "test_dist_crate_feature_constant_does_not_borrow_build_owner",
)


class PluginValidateDistCrateOwnerBoundaryTests(unittest.TestCase):
    def test_dist_crate_boundaries_leave_general_owner_file(self):
        general_owner_text = PLUGIN_VALIDATE_OWNER_BOUNDARIES_TEST.read_text(
            encoding="utf-8"
        )

        for method_name in DIST_CRATE_BOUNDARY_METHODS:
            self.assertNotIn(
                f"def {method_name}(",
                general_owner_text,
                f"{method_name} belongs in test_plugin_validate_dist_crate_owner_boundaries.py",
            )

        self.assertLessEqual(
            len(general_owner_text.splitlines()),
            3820,
            "general PluginValidate owner boundary tests should shrink after dist-crate split",
        )
        self.assertLessEqual(
            len(Path(__file__).read_text(encoding="utf-8").splitlines()),
            260,
            "focused PluginValidate dist-crate owner boundary file should stay narrow",
        )

    def test_dist_crate_cargo_preflight_lives_in_dist_crate_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_DIST_CRATE.exists(),
            "dist crate Cargo preflight belongs in plugin_validate_dist_crate.py",
        )
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        dist_crate_text = PLUGIN_VALIDATE_DIST_CRATE.read_text(encoding="utf-8")

        self.assertNotIn(
            "def validate_plugin_dist_crate_feature(",
            validate_text,
            "validate_plugin_dist_crate_feature belongs in plugin_validate_dist_crate.py",
        )
        self.assertIn(
            "def validate_plugin_dist_crate_feature(",
            dist_crate_text,
        )
        self.assertIn(
            "from .plugin_validate_dist_crate_dependency import",
            dist_crate_text,
            "dist crate preflight owner should dispatch dependency checks to its leaf owner",
        )

    def test_dist_crate_sdk_dependency_lives_in_dependency_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_DIST_CRATE_DEPENDENCY.exists(),
            "dist crate SDK dependency/ABI helper checks belong in plugin_validate_dist_crate_dependency.py",
        )
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        dist_crate_text = PLUGIN_VALIDATE_DIST_CRATE.read_text(encoding="utf-8")
        dependency_text = PLUGIN_VALIDATE_DIST_CRATE_DEPENDENCY.read_text(
            encoding="utf-8"
        )

        for function_name in (
            "validate_plugin_dist_crate_sdk_dependency",
            "plugin_validate_find_dependency",
            "plugin_validate_dependency_tables",
            "plugin_validate_sdk_dependency_enables_abi_helpers",
            "plugin_validate_feature_enables_dependency",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                validate_text,
                f"{function_name} must not be owned by plugin_validate.py",
            )
            self.assertNotIn(
                f"def {function_name}(",
                dist_crate_text,
                f"{function_name} belongs in plugin_validate_dist_crate_dependency.py",
            )
            self.assertIn(
                f"def {function_name}(",
                dependency_text,
            )

        for constant_name in (
            "PLUGIN_VALIDATE_SDK_DEPENDENCY",
            "PLUGIN_VALIDATE_SDK_ABI_FEATURES",
            "PLUGIN_VALIDATE_FORBIDDEN_DIST_FEATURE_DEPENDENCIES",
        ):
            self.assertNotIn(
                f"{constant_name} =",
                validate_text,
                f"{constant_name} must not be owned by plugin_validate.py",
            )
            self.assertNotIn(
                f"{constant_name} =",
                dist_crate_text,
                f"{constant_name} belongs in plugin_validate_dist_crate_dependency.py",
            )
            self.assertIn(
                f"{constant_name} =",
                dependency_text,
            )

        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
        ):
            self.assertNotIn(
                forbidden_import,
                dependency_text,
                "dist crate dependency owner must stay independent from build and validate entry owners",
            )
        self.assertLessEqual(
            len(dist_crate_text.splitlines()),
            150,
            "dist crate Cargo preflight owner should stay below the next split budget",
        )
        self.assertLessEqual(
            len(dependency_text.splitlines()),
            130,
            "dist crate dependency owner should stay a focused leaf module",
        )

    def test_dist_crate_workspace_member_resolution_lives_in_dist_crate_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_DIST_CRATE.exists(),
            "dist crate workspace-member resolution belongs in plugin_validate_dist_crate.py",
        )
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        dist_crate_text = PLUGIN_VALIDATE_DIST_CRATE.read_text(encoding="utf-8")

        for function_name in (
            "plugin_validate_dist_crate_manifest",
            "validate_plugin_dist_crate_workspace_member",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                validate_text,
                f"{function_name} belongs in plugin_validate_dist_crate.py",
            )
            self.assertIn(
                f"def {function_name}(",
                dist_crate_text,
            )

        self.assertNotIn(
            "is not a cdylib workspace member",
            validate_text,
            "dist crate workspace-member diagnostics belong in plugin_validate_dist_crate.py",
        )
        self.assertIn(
            "is not a cdylib workspace member",
            dist_crate_text,
        )

    def test_dist_crate_feature_constant_does_not_borrow_build_owner(self):
        common_text = PLUGIN_VALIDATE_COMMON.read_text(encoding="utf-8")
        dist_crate_text = PLUGIN_VALIDATE_DIST_CRATE.read_text(encoding="utf-8")

        self.assertIn(
            "PLUGIN_VALIDATE_DIST_FEATURE =",
            common_text,
            "validate-specific dist Cargo feature constant belongs in plugin_validate_common.py",
        )
        self.assertNotIn(
            "PLUGIN_BUILD_DIST_FEATURE",
            dist_crate_text,
            "dist crate validation must not borrow dist Cargo feature from plugin_build.py",
        )
        self.assertNotIn(
            "from .plugin_build import",
            dist_crate_text,
            "dist crate validation must stay independent from build owner imports",
        )


if __name__ == "__main__":
    unittest.main()
