import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PLUGIN_VALIDATE_OWNER_BOUNDARIES_TEST = (
    REPO_ROOT / "tools/tests/test_plugin_validate_owner_boundaries.py"
)
PLUGIN_VALIDATE = REPO_ROOT / "tools/zircon_export/plugin_validate.py"
PLUGIN_VALIDATE_SINGLE_TARGET = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_single_target.py"
)
PLUGIN_VALIDATE_DEPENDENCIES = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_dependencies.py"
)
PLUGIN_VALIDATE_OPTIONS = REPO_ROOT / "tools/zircon_export/plugin_validate_options.py"
PLUGIN_VALIDATE_EVENT_CATALOGS = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_event_catalogs.py"
)
PLUGIN_VALIDATE_COMPONENTS = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_components.py"
)
PLUGIN_VALIDATE_MODULES = REPO_ROOT / "tools/zircon_export/plugin_validate_modules.py"
PLUGIN_VALIDATE_MODULE_ROWS = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_module_rows.py"
)
PLUGIN_VALIDATE_MODULE_CRATES = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_module_crates.py"
)
PLUGIN_VALIDATE_MODULE_SYSTEMS = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_module_systems.py"
)
PLUGIN_VALIDATE_DISTRIBUTION_MODULES = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_distribution_modules.py"
)
PLUGIN_VALIDATE_TEST = REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate.py"
PLUGIN_VALIDATE_DISTRIBUTION_MODULES_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate_distribution_modules.py"
)
PLUGIN_VALIDATE_EVENT_CATALOGS_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate_event_catalogs.py"
)
PLUGIN_VALIDATE_COMPONENTS_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate_components.py"
)
PLUGIN_VALIDATE_MODULES_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate_modules.py"
)

EVENT_COMPONENT_MODULE_BOUNDARY_METHODS = (
    "test_event_catalogs_lives_in_event_catalog_owner",
    "test_event_catalog_tests_live_in_event_catalog_test_owner",
    "test_components_lives_in_components_owner",
    "test_component_tests_live_in_component_test_owner",
    "test_modules_lives_in_modules_owner",
    "test_module_workspace_crate_checks_live_in_module_crates_owner",
    "test_module_system_contracts_live_in_module_systems_owner",
    "test_module_tests_live_in_module_test_owner",
)


class PluginValidateEventComponentModuleOwnerBoundaryTests(unittest.TestCase):
    def test_event_component_module_boundaries_leave_general_owner_file(self):
        general_owner_text = PLUGIN_VALIDATE_OWNER_BOUNDARIES_TEST.read_text(
            encoding="utf-8"
        )

        for method_name in EVENT_COMPONENT_MODULE_BOUNDARY_METHODS:
            self.assertNotIn(
                f"def {method_name}(",
                general_owner_text,
                f"{method_name} belongs in test_plugin_validate_event_component_module_owner_boundaries.py",
            )

        self.assertLessEqual(
            len(general_owner_text.splitlines()),
            1100,
            "general PluginValidate owner boundary tests should shrink after event/component/module split",
        )
        self.assertLessEqual(
            len(Path(__file__).read_text(encoding="utf-8").splitlines()),
            510,
            "focused PluginValidate event/component/module owner boundary file should stay narrow",
        )

    def test_event_catalogs_lives_in_event_catalog_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_EVENT_CATALOGS.exists(),
            "event catalog validation belongs in a focused root owner",
        )
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        single_target_text = PLUGIN_VALIDATE_SINGLE_TARGET.read_text(encoding="utf-8")
        event_text = PLUGIN_VALIDATE_EVENT_CATALOGS.read_text(encoding="utf-8")

        for symbol in (
            "validate_plugin_event_catalogs",
            "validate_plugin_event_catalog_row",
            "validate_plugin_event_rows",
            "validate_plugin_event_known_fields",
            "PLUGIN_VALIDATE_EVENT_CATALOG_FIELDS",
            "PLUGIN_VALIDATE_EVENT_FIELDS",
            "plugin_validate_event_catalog_namespace_index",
            "is not a known {field_label} field",
            "event catalog",
            "should stay under package namespace",
            "duplicates event catalog namespace",
            "version segment should be a positive integer",
            "without leading zeroes",
        ):
            self.assertIn(symbol, event_text)
        self.assertIn(
            "from .plugin_validate_event_catalogs import validate_plugin_event_catalogs",
            single_target_text,
            "single-target owner should dispatch event catalog checks",
        )
        self.assertIn("validate_plugin_event_catalogs(", single_target_text)
        for parent_text, parent_name in (
            (validate_text, "plugin_validate.py"),
            (single_target_text, "plugin_validate_single_target.py"),
            (
                PLUGIN_VALIDATE_DEPENDENCIES.read_text(encoding="utf-8"),
                "plugin_validate_dependencies.py",
            ),
            (
                PLUGIN_VALIDATE_OPTIONS.read_text(encoding="utf-8"),
                "plugin_validate_options.py",
            ),
        ):
            self.assertNotIn(
                "def validate_plugin_event_catalogs(",
                parent_text,
                f"{parent_name} must not own root event catalog validation",
            )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_single_target import",
            "from .plugin_validate_dependencies import",
            "from .plugin_validate_options import",
            "from .plugin_validate_asset_importers import",
        ):
            self.assertNotIn(
                forbidden_import,
                event_text,
                "event catalog owner must stay independent from entry and sibling owners",
            )
        self.assertLessEqual(
            len(event_text.splitlines()),
            280,
            "event catalog owner should stay focused on manifest event rows",
        )

    def test_event_catalog_tests_live_in_event_catalog_test_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_EVENT_CATALOGS_TEST.exists(),
            "event catalog behavior tests belong in test_plugin_validate_event_catalogs.py",
        )
        validate_test_text = PLUGIN_VALIDATE_TEST.read_text(encoding="utf-8")
        event_test_text = PLUGIN_VALIDATE_EVENT_CATALOGS_TEST.read_text(
            encoding="utf-8"
        )

        for test_name in (
            "test_plugin_validate_rejects_malformed_event_catalog",
            "test_plugin_validate_rejects_malformed_event_rows",
            "test_plugin_validate_rejects_duplicate_event_catalog_namespace",
            "test_plugin_validate_rejects_unknown_event_catalog_fields",
        ):
            self.assertNotIn(
                f"def {test_name}(",
                validate_test_text,
                f"{test_name} belongs in the event catalog test owner",
            )
            self.assertIn(f"def {test_name}(", event_test_text)

    def test_components_lives_in_components_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_COMPONENTS.exists(),
            "component and UI component validation belongs in a focused root owner",
        )
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        single_target_text = PLUGIN_VALIDATE_SINGLE_TARGET.read_text(encoding="utf-8")
        components_text = PLUGIN_VALIDATE_COMPONENTS.read_text(encoding="utf-8")

        for symbol in (
            "validate_plugin_components",
            "validate_plugin_component_rows",
            "validate_plugin_ui_component_rows",
            "validate_plugin_component_properties",
            "validate_plugin_component_known_fields",
            "PLUGIN_VALIDATE_COMPONENT_FIELDS",
            "PLUGIN_VALIDATE_COMPONENT_PROPERTY_FIELDS",
            "PLUGIN_VALIDATE_UI_COMPONENT_FIELDS",
            "plugin_validate_component_identity_index",
            "is not a known {field_label} field",
            "component property",
            "ui_component",
            "should reference a .zui component asset",
            "component type_id",
            "duplicates {identity_name}",
        ):
            self.assertIn(symbol, components_text)
        self.assertIn(
            "from .plugin_validate_components import validate_plugin_components",
            single_target_text,
            "single-target owner should dispatch component checks",
        )
        self.assertIn("validate_plugin_components(", single_target_text)
        for parent_text, parent_name in (
            (validate_text, "plugin_validate.py"),
            (single_target_text, "plugin_validate_single_target.py"),
            (
                PLUGIN_VALIDATE_DEPENDENCIES.read_text(encoding="utf-8"),
                "plugin_validate_dependencies.py",
            ),
            (
                PLUGIN_VALIDATE_OPTIONS.read_text(encoding="utf-8"),
                "plugin_validate_options.py",
            ),
            (
                PLUGIN_VALIDATE_EVENT_CATALOGS.read_text(encoding="utf-8"),
                "plugin_validate_event_catalogs.py",
            ),
        ):
            self.assertNotIn(
                "def validate_plugin_components(",
                parent_text,
                f"{parent_name} must not own root component validation",
            )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_single_target import",
            "from .plugin_validate_dependencies import",
            "from .plugin_validate_options import",
            "from .plugin_validate_event_catalogs import",
            "from .plugin_validate_asset_importers import",
        ):
            self.assertNotIn(
                forbidden_import,
                components_text,
                "components owner must stay independent from entry and sibling owners",
            )
        self.assertLessEqual(
            len(components_text.splitlines()),
            340,
            "components owner should stay focused on manifest component rows",
        )

    def test_component_tests_live_in_component_test_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_COMPONENTS_TEST.exists(),
            "component behavior tests belong in test_plugin_validate_components.py",
        )
        validate_test_text = PLUGIN_VALIDATE_TEST.read_text(encoding="utf-8")
        component_test_text = PLUGIN_VALIDATE_COMPONENTS_TEST.read_text(
            encoding="utf-8"
        )

        for test_name in (
            "test_plugin_validate_rejects_malformed_component_row",
            "test_plugin_validate_rejects_duplicate_component_type_id",
            "test_plugin_validate_rejects_ui_component_retired_document_path",
            "test_plugin_validate_rejects_unknown_component_fields",
        ):
            self.assertNotIn(
                f"def {test_name}(",
                validate_test_text,
                f"{test_name} belongs in the component test owner",
            )
            self.assertIn(f"def {test_name}(", component_test_text)

    def test_modules_lives_in_modules_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_MODULES.exists(),
            "module row validation belongs in a focused root owner",
        )
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        single_target_text = PLUGIN_VALIDATE_SINGLE_TARGET.read_text(encoding="utf-8")
        distribution_text = PLUGIN_VALIDATE_DISTRIBUTION_MODULES.read_text(
            encoding="utf-8"
        )
        modules_text = PLUGIN_VALIDATE_MODULES.read_text(encoding="utf-8")
        module_rows_text = PLUGIN_VALIDATE_MODULE_ROWS.read_text(encoding="utf-8")

        for symbol in (
            "validate_plugin_modules",
            "validate_plugin_feature_extension_modules",
            "plugin_validate_root_supported_targets",
        ):
            self.assertIn(symbol, modules_text)
        for symbol in (
            "validate_plugin_module_rows",
            "validate_plugin_module_known_fields",
            "validate_plugin_module_row",
            "validate_plugin_module_target_modes",
            "PLUGIN_VALIDATE_MODULE_FIELDS",
            "PLUGIN_VALIDATE_MODULE_KINDS",
            "is not a known module field",
            "duplicates module name",
            "should be covered by package supported_targets",
        ):
            self.assertIn(symbol, module_rows_text)
        self.assertIn(
            "from .plugin_validate_module_rows import",
            modules_text,
            "module orchestration should delegate row semantics to the leaf owner",
        )
        feature_provider_schema = (
            REPO_ROOT / "tools/zircon_export/plugin_validate_feature_provider_module_schema.py"
        ).read_text(encoding="utf-8")
        self.assertIn("from .plugin_validate_module_rows import", feature_provider_schema)
        for symbol in ("validate_plugin_module_rows", "validate_plugin_module_row"):
            self.assertNotIn(
                f"def {symbol}(", modules_text, f"{symbol} belongs in the row leaf"
            )
        self.assertIn(
            "from .plugin_validate_modules import validate_plugin_modules",
            single_target_text,
            "single-target owner should dispatch module row checks",
        )
        self.assertIn("validate_plugin_modules(", single_target_text)
        for parent_text, parent_name in (
            (validate_text, "plugin_validate.py"),
            (single_target_text, "plugin_validate_single_target.py"),
            (distribution_text, "plugin_validate_distribution_modules.py"),
            (
                PLUGIN_VALIDATE_COMPONENTS.read_text(encoding="utf-8"),
                "plugin_validate_components.py",
            ),
            (
                PLUGIN_VALIDATE_EVENT_CATALOGS.read_text(encoding="utf-8"),
                "plugin_validate_event_catalogs.py",
            ),
        ):
            self.assertNotIn(
                "def validate_plugin_modules(",
                parent_text,
                f"{parent_name} must not own root module row validation",
            )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_single_target import",
            "from .plugin_validate_distribution_modules import",
            "from .plugin_validate_components import",
            "from .plugin_validate_event_catalogs import",
        ):
            self.assertNotIn(
                forbidden_import,
                modules_text,
                "modules owner must stay independent from entry and sibling owners",
            )
            self.assertNotIn(
                forbidden_import,
                module_rows_text,
                "module row leaf must stay independent from entry and sibling owners",
            )
        self.assertLessEqual(
            len(modules_text.splitlines()),
            160,
            "modules owner should stay focused on manifest orchestration",
        )
        self.assertLessEqual(
            len(module_rows_text.splitlines()),
            300,
            "module row leaf should stay focused on module row semantics",
        )

    def test_module_workspace_crate_checks_live_in_module_crates_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_MODULE_CRATES.exists(),
            "module workspace crate ownership checks belong in plugin_validate_module_crates.py",
        )
        module_rows_text = PLUGIN_VALIDATE_MODULE_ROWS.read_text(encoding="utf-8")
        module_crates_text = PLUGIN_VALIDATE_MODULE_CRATES.read_text(encoding="utf-8")
        single_target_text = PLUGIN_VALIDATE_SINGLE_TARGET.read_text(encoding="utf-8")

        for function_name in (
            "validate_plugin_module_workspace_crate",
            "plugin_validate_optional_feature_root",
            "plugin_validate_path_is_relative_to",
            "plugin_validate_workspace_relative_path",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                module_rows_text,
                f"{function_name} belongs in plugin_validate_module_crates.py",
            )
            self.assertIn(f"def {function_name}(", module_crates_text)

        self.assertIn(
            "from .plugin_validate_module_crates import",
            module_rows_text,
            "module row owner should dispatch workspace crate checks to the leaf owner",
        )
        self.assertIn(
            "workspace_crate_index=workspace_crate_index",
            single_target_text,
            "single-target owner should pass workspace crate metadata into module validation",
        )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_single_target import",
            "from .plugin_validate_modules import",
        ):
            self.assertNotIn(
                forbidden_import,
                module_crates_text,
                "module crate owner must stay independent from entry and module row owners",
            )
        self.assertLessEqual(
            len(module_crates_text.splitlines()),
            110,
            "module crate owner should stay a small leaf",
        )

    def test_module_system_contracts_live_in_module_systems_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_MODULE_SYSTEMS.exists(),
            "module system field checks belong in plugin_validate_module_systems.py",
        )
        module_rows_text = PLUGIN_VALIDATE_MODULE_ROWS.read_text(encoding="utf-8")
        module_systems_text = PLUGIN_VALIDATE_MODULE_SYSTEMS.read_text(encoding="utf-8")

        for function_name in (
            "validate_plugin_module_system_contracts",
            "validate_plugin_module_system_names",
            "validate_plugin_module_system_namespace",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                module_rows_text,
                f"{function_name} belongs in plugin_validate_module_systems.py",
            )
            self.assertIn(f"def {function_name}(", module_systems_text)

        self.assertIn(
            "from .plugin_validate_module_systems import",
            module_rows_text,
            "module row owner should dispatch system checks to the leaf owner",
        )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_single_target import",
            "from .plugin_validate_modules import",
            "from .plugin_validate_module_crates import",
        ):
            self.assertNotIn(
                forbidden_import,
                module_systems_text,
                "module systems owner must stay independent from entry and sibling owners",
            )
        self.assertLessEqual(
            len(module_systems_text.splitlines()),
            120,
            "module systems owner should stay a small leaf",
        )

    def test_module_tests_live_in_module_test_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_MODULES_TEST.exists(),
            "module behavior tests belong in test_plugin_validate_modules.py",
        )
        validate_test_text = PLUGIN_VALIDATE_TEST.read_text(encoding="utf-8")
        distribution_test_text = PLUGIN_VALIDATE_DISTRIBUTION_MODULES_TEST.read_text(
            encoding="utf-8"
        )
        module_test_text = PLUGIN_VALIDATE_MODULES_TEST.read_text(encoding="utf-8")

        for test_name in (
            "test_plugin_validate_rejects_malformed_module_row",
            "test_plugin_validate_rejects_duplicate_module_name",
            "test_plugin_validate_rejects_optional_feature_module_namespace",
            "test_plugin_validate_rejects_feature_extension_module_namespace",
            "test_plugin_validate_rejects_module_crate_missing_workspace_member",
            "test_plugin_validate_rejects_module_crate_outside_package_root",
            "test_plugin_validate_rejects_malformed_module_system_names",
            "test_plugin_validate_rejects_duplicate_module_system_names",
            "test_plugin_validate_rejects_non_runtime_module_system_names",
            "test_plugin_validate_rejects_unknown_module_fields",
        ):
            for source_text, source_name in (
                (validate_test_text, "test_plugin_validate.py"),
                (
                    distribution_test_text,
                    "test_plugin_validate_distribution_modules.py",
                ),
            ):
                self.assertNotIn(
                    f"def {test_name}(",
                    source_text,
                    f"{test_name} belongs in the module test owner, not {source_name}",
                )
            self.assertIn(f"def {test_name}(", module_test_text)


if __name__ == "__main__":
    unittest.main()
