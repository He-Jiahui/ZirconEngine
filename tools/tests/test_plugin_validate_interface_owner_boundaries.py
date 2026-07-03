import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PLUGIN_VALIDATE_OWNER_BOUNDARIES_TEST = REPO_ROOT / "tools/tests/test_plugin_validate_owner_boundaries.py"
PLUGIN_VALIDATE_DEPENDENCIES = REPO_ROOT / "tools/zircon_export/plugin_validate_dependencies.py"
PLUGIN_VALIDATE_INTERFACES = REPO_ROOT / "tools/zircon_export/plugin_validate_interfaces.py"
PLUGIN_VALIDATE_INTERFACE_METHODS = REPO_ROOT / "tools/zircon_export/plugin_validate_interface_methods.py"
PLUGIN_VALIDATE_INTERFACE_SIGNATURES = REPO_ROOT / "tools/zircon_export/plugin_validate_interface_signatures.py"
PLUGIN_VALIDATE_TEST = REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate.py"
PLUGIN_VALIDATE_INTERFACES_TEST = REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate_interfaces.py"

INTERFACE_BOUNDARY_METHODS = (
    "test_interface_contracts_live_in_interfaces_owner",
    "test_interface_method_contracts_live_in_interface_methods_owner",
    "test_interface_signature_contracts_live_in_interface_signatures_owner",
    "test_interface_tests_live_in_interface_test_owner",
)


class PluginValidateInterfaceOwnerBoundaryTests(unittest.TestCase):
    def test_interface_boundaries_leave_general_owner_file(self):
        general_owner_text = PLUGIN_VALIDATE_OWNER_BOUNDARIES_TEST.read_text(
            encoding="utf-8"
        )

        for method_name in INTERFACE_BOUNDARY_METHODS:
            self.assertNotIn(
                f"def {method_name}(",
                general_owner_text,
                f"{method_name} belongs in test_plugin_validate_interface_owner_boundaries.py",
            )

        self.assertLessEqual(
            len(general_owner_text.splitlines()),
            1900,
            "general PluginValidate owner boundary tests should shrink after interface split",
        )
        self.assertLessEqual(
            len(Path(__file__).read_text(encoding="utf-8").splitlines()),
            240,
            "focused PluginValidate interface owner boundary file should stay narrow",
        )

    def test_interface_contracts_live_in_interfaces_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_INTERFACES.exists(),
            "provided and dependency interface validation belongs in a focused owner",
        )
        dependencies_text = PLUGIN_VALIDATE_DEPENDENCIES.read_text(encoding="utf-8")
        interfaces_text = PLUGIN_VALIDATE_INTERFACES.read_text(encoding="utf-8")

        for symbol in (
            "validate_plugin_provided_interfaces",
            "plugin_validate_dependency_interfaces",
            "PLUGIN_VALIDATE_PROVIDED_INTERFACE_FIELDS",
            "validate_plugin_provided_interface_known_fields",
            "validate_plugin_interface_namespace",
            "is not a known provided interface field",
            "duplicates provided interface id",
            "duplicates dependency interface",
        ):
            self.assertIn(symbol, interfaces_text)
        for function_name in (
            "validate_plugin_provided_interfaces",
            "plugin_validate_dependency_interfaces",
            "validate_plugin_interface_namespace",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                dependencies_text,
                f"{function_name} belongs in plugin_validate_interfaces.py",
            )
        self.assertIn(
            "from .plugin_validate_interfaces import",
            dependencies_text,
            "dependencies owner should dispatch bridge interface checks to the leaf owner",
        )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_single_target import",
            "from .plugin_validate_dependencies import",
        ):
            self.assertNotIn(
                forbidden_import,
                interfaces_text,
                "interfaces owner must stay independent from entry and dependency owners",
            )
        self.assertLessEqual(
            len(interfaces_text.splitlines()),
            160,
            "interfaces owner should stay small and focused",
        )

    def test_interface_method_contracts_live_in_interface_methods_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_INTERFACE_METHODS.exists(),
            "provided interface method validation belongs in a focused owner",
        )
        interfaces_text = PLUGIN_VALIDATE_INTERFACES.read_text(encoding="utf-8")
        methods_text = PLUGIN_VALIDATE_INTERFACE_METHODS.read_text(encoding="utf-8")

        for symbol in (
            "validate_plugin_provided_interface_methods",
            "validate_plugin_interface_method_parameters",
            "validate_plugin_interface_method_required_capabilities",
            "PLUGIN_VALIDATE_INTERFACE_METHOD_FIELDS",
            "validate_plugin_interface_method_known_fields",
            "plugin_validate_optional_trimmed_string",
            "documentation",
            "is not a known provided interface method field",
            "duplicates provided interface method",
            "duplicates interface method parameter",
            "must not be empty when declared",
            "duplicates required capability",
        ):
            self.assertIn(symbol, methods_text)
        for function_name in (
            "validate_plugin_provided_interface_methods",
            "validate_plugin_interface_method_parameters",
            "validate_plugin_interface_method_required_capabilities",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                interfaces_text,
                f"{function_name} belongs in plugin_validate_interface_methods.py",
            )
        self.assertIn(
            "from .plugin_validate_interface_methods import",
            interfaces_text,
            "interface owner should dispatch method checks to the leaf owner",
        )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_single_target import",
            "from .plugin_validate_dependencies import",
            "from .plugin_validate_interfaces import",
        ):
            self.assertNotIn(
                forbidden_import,
                methods_text,
                "interface method owner must stay independent from entry and parent owners",
            )
        self.assertLessEqual(
            len(methods_text.splitlines()),
            170,
            "interface method owner should stay small and focused",
        )

    def test_interface_signature_contracts_live_in_interface_signatures_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_INTERFACE_SIGNATURES.exists(),
            "provided interface signature validation belongs in a focused owner",
        )
        methods_text = PLUGIN_VALIDATE_INTERFACE_METHODS.read_text(encoding="utf-8")
        signatures_text = PLUGIN_VALIDATE_INTERFACE_SIGNATURES.read_text(
            encoding="utf-8"
        )

        for symbol in (
            "ALLOWED_SCRIPT_HOST_VALUE_KINDS",
            "PLUGIN_VALIDATE_INTERFACE_METHOD_PARAMETER_FIELDS",
            "PLUGIN_VALIDATE_INTERFACE_METHOD_TYPE_REF_FIELDS",
            "validate_plugin_interface_method_return_kind",
            "validate_plugin_interface_method_parameter_signature",
            "validate_plugin_interface_method_type_ref",
            "validate_plugin_interface_signature_known_fields",
            "interface method parameter",
            "interface method type_ref",
            "is not a known {field_label} field",
            "unsupported; expected one of",
        ):
            self.assertIn(symbol, signatures_text)
        for function_name in (
            "validate_plugin_interface_method_return_kind",
            "validate_plugin_interface_method_parameter_signature",
            "validate_plugin_interface_method_type_ref",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                methods_text,
                f"{function_name} belongs in plugin_validate_interface_signatures.py",
            )
        self.assertIn(
            "from .plugin_validate_interface_signatures import",
            methods_text,
            "interface method owner should dispatch signature checks to the leaf owner",
        )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_single_target import",
            "from .plugin_validate_dependencies import",
            "from .plugin_validate_interfaces import",
            "from .plugin_validate_interface_methods import",
        ):
            self.assertNotIn(
                forbidden_import,
                signatures_text,
                "interface signature owner must stay independent from entry and parent owners",
            )
        self.assertLessEqual(
            len(signatures_text.splitlines()),
            140,
            "interface signature owner should stay small and focused",
        )

    def test_interface_tests_live_in_interface_test_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_INTERFACES_TEST.exists(),
            "bridge interface behavior tests belong in test_plugin_validate_interfaces.py",
        )
        validate_test_text = PLUGIN_VALIDATE_TEST.read_text(encoding="utf-8")
        interface_test_text = PLUGIN_VALIDATE_INTERFACES_TEST.read_text(
            encoding="utf-8"
        )

        for test_name in (
            "test_plugin_validate_rejects_malformed_provided_interface_id",
            "test_plugin_validate_rejects_duplicate_provided_interface_id",
            "test_plugin_validate_rejects_dependency_interface_namespace_and_duplicate",
            "test_plugin_validate_rejects_malformed_provided_interface_method",
            "test_plugin_validate_rejects_duplicate_provided_interface_methods",
            "test_plugin_validate_rejects_unknown_interface_fields",
            "test_plugin_validate_rejects_malformed_provided_interface_signature",
        ):
            self.assertNotIn(
                f"def {test_name}(",
                validate_test_text,
                f"{test_name} belongs in the bridge interface test owner",
            )
            self.assertIn(f"def {test_name}(", interface_test_text)


if __name__ == "__main__":
    unittest.main()
