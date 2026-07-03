import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PLUGIN_VALIDATE = REPO_ROOT / "tools/zircon_export/plugin_validate.py"
PLUGIN_VALIDATE_COMMON = REPO_ROOT / "tools/zircon_export/plugin_validate_common.py"


class PluginValidateOwnerBoundaryTests(unittest.TestCase):
    def test_distribution_enum_helper_lives_in_common_owner(self):
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        common_text = PLUGIN_VALIDATE_COMMON.read_text(encoding="utf-8")

        self.assertNotIn(
            "def plugin_validate_allowed_string_values(",
            validate_text,
            "closed-set distribution helper belongs in plugin_validate_common.py",
        )
        self.assertIn(
            "def plugin_validate_allowed_string_values(",
            common_text,
        )

    def test_distribution_allowed_value_constants_live_in_common_owner(self):
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        common_text = PLUGIN_VALIDATE_COMMON.read_text(encoding="utf-8")

        for constant_name in (
            "PLUGIN_VALIDATE_DIST_PACKAGING",
            "PLUGIN_VALIDATE_DISTRIBUTION_FORMS",
            "PLUGIN_VALIDATE_DEFAULT_PACKAGING",
        ):
            self.assertNotIn(
                f"{constant_name} =",
                validate_text,
                f"{constant_name} belongs in plugin_validate_common.py",
            )
            self.assertIn(
                f"{constant_name} =",
                common_text,
            )

    def test_validate_cli_form_constant_does_not_borrow_build_owner(self):
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        common_text = PLUGIN_VALIDATE_COMMON.read_text(encoding="utf-8")

        self.assertIn(
            "PLUGIN_VALIDATE_DIST_FORM =",
            common_text,
            "validate CLI form constant belongs in plugin_validate_common.py",
        )
        self.assertNotIn(
            "PLUGIN_BUILD_DIST_FORM",
            validate_text,
            "plugin validate CLI must not borrow the form constant from plugin_build.py",
        )

    def test_general_owner_boundary_file_is_closed_out(self):
        for moved_method in (
            "test_options_required_capability_gates_lives_in_options_owner",
            "test_dependencies_lives_in_dependencies_owner",
            "test_option_schema_lives_in_schema_owner",
            "test_option_global_keys_lives_in_global_key_owner",
            "test_distribution_contract_tests_live_in_distribution_contract_test_owner",
            "test_distribution_module_tests_live_in_distribution_modules_test_owner",
        ):
            self.assertFalse(
                hasattr(self, moved_method),
                f"{moved_method} belongs in a focused PluginValidate owner boundary file",
            )
        self.assertLessEqual(
            len(Path(__file__).read_text(encoding="utf-8").splitlines()),
            90,
            "general PluginValidate owner boundary file should stay as a thin common-owner guard",
        )


if __name__ == "__main__":
    unittest.main()
