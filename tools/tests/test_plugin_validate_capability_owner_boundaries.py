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
PLUGIN_VALIDATE_COMMON = REPO_ROOT / "tools/zircon_export/plugin_validate_common.py"
PLUGIN_VALIDATE_DEPENDENCIES = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_dependencies.py"
)
PLUGIN_VALIDATE_DISTRIBUTION_PACKAGING = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_distribution_packaging.py"
)
PLUGIN_VALIDATE_CAPABILITY_STATUSES = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_capability_statuses.py"
)
PLUGIN_VALIDATE_CAPABILITIES = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_capabilities.py"
)
PLUGIN_VALIDATE_DEFAULT_PACKAGING_METADATA = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_default_packaging.py"
)
PLUGIN_VALIDATE_CAPABILITY_STATUS_TARGETS = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_capability_status_targets.py"
)
PLUGIN_VALIDATE_CAPABILITY_STATUS_REFERENCES = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_capability_status_references.py"
)
PLUGIN_VALIDATE_DEPENDENCY_CAPABILITIES = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_dependency_capabilities.py"
)
PLUGIN_VALIDATE_TEST = REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate.py"
PLUGIN_VALIDATE_DISTRIBUTION_CONTRACT_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate_distribution_contract.py"
)
PLUGIN_VALIDATE_CAPABILITY_STATUSES_TEST = (
    REPO_ROOT
    / "tools/zircon_export/tests/test_plugin_validate_capability_statuses.py"
)
PLUGIN_VALIDATE_CAPABILITIES_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate_capabilities.py"
)
PLUGIN_VALIDATE_DEFAULT_PACKAGING_TEST = (
    REPO_ROOT / "tools/zircon_export/tests/test_plugin_validate_default_packaging.py"
)

CAPABILITY_BOUNDARY_METHODS = (
    "test_root_capabilities_tests_live_in_root_capabilities_test_owner",
    "test_root_capabilities_lives_in_root_capabilities_owner",
    "test_default_packaging_tests_live_in_default_packaging_test_owner",
    "test_default_packaging_lives_in_default_packaging_owner",
    "test_capability_statuses_live_in_capability_statuses_owner",
    "test_capability_status_tests_live_in_capability_status_test_owner",
    "test_capability_status_targets_live_in_capability_status_targets_owner",
    "test_capability_status_references_live_in_capability_status_references_owner",
    "test_dependency_capabilities_lives_in_capabilities_owner",
)


class PluginValidateCapabilityOwnerBoundaryTests(unittest.TestCase):
    def test_capability_boundaries_leave_general_owner_file(self):
        general_owner_text = PLUGIN_VALIDATE_OWNER_BOUNDARIES_TEST.read_text(
            encoding="utf-8"
        )

        for method_name in CAPABILITY_BOUNDARY_METHODS:
            self.assertNotIn(
                f"def {method_name}(",
                general_owner_text,
                f"{method_name} belongs in test_plugin_validate_capability_owner_boundaries.py",
            )

        self.assertLessEqual(
            len(general_owner_text.splitlines()),
            1500,
            "general PluginValidate owner boundary tests should shrink after capability split",
        )
        self.assertLessEqual(
            len(Path(__file__).read_text(encoding="utf-8").splitlines()),
            560,
            "focused PluginValidate capability owner boundary file should stay narrow",
        )

    def test_root_capabilities_tests_live_in_root_capabilities_test_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_CAPABILITIES_TEST.exists(),
            "root capabilities behavior tests belong in test_plugin_validate_capabilities.py",
        )
        validate_test_text = PLUGIN_VALIDATE_TEST.read_text(encoding="utf-8")
        statuses_test_text = PLUGIN_VALIDATE_CAPABILITY_STATUSES_TEST.read_text(
            encoding="utf-8"
        )
        capabilities_test_text = PLUGIN_VALIDATE_CAPABILITIES_TEST.read_text(
            encoding="utf-8"
        )

        for test_name in (
            "test_plugin_validate_rejects_malformed_root_capabilities",
            "test_plugin_validate_rejects_empty_root_capabilities",
        ):
            for source_name, source_text in (
                ("test_plugin_validate.py", validate_test_text),
                (
                    "test_plugin_validate_capability_statuses.py",
                    statuses_test_text,
                ),
            ):
                self.assertNotIn(
                    f"def {test_name}(",
                    source_text,
                    f"{test_name} belongs in the root capabilities test owner, not {source_name}",
                )
            self.assertIn(f"def {test_name}(", capabilities_test_text)

    def test_root_capabilities_lives_in_root_capabilities_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_CAPABILITIES.exists(),
            "root package capability declarations belong in plugin_validate_capabilities.py",
        )
        single_target_text = PLUGIN_VALIDATE_SINGLE_TARGET.read_text(encoding="utf-8")
        common_text = PLUGIN_VALIDATE_COMMON.read_text(encoding="utf-8")
        statuses_text = PLUGIN_VALIDATE_CAPABILITY_STATUSES.read_text(
            encoding="utf-8"
        )
        capabilities_text = PLUGIN_VALIDATE_CAPABILITIES.read_text(encoding="utf-8")

        for symbol in (
            "PLUGIN_VALIDATE_CAPABILITIES_DUPLICATE_MESSAGE",
            "validate_plugin_capabilities",
            "validate_plugin_capability_values",
            "validate_plugin_capability_namespace",
            "duplicates capabilities",
            "should use at least two dot-separated namespace segments",
        ):
            self.assertIn(symbol, capabilities_text)
            for source_name, source_text in (
                ("plugin_validate_common.py", common_text),
                ("plugin_validate_capability_statuses.py", statuses_text),
                ("plugin_validate_single_target.py", single_target_text),
            ):
                if (
                    symbol == "validate_plugin_capabilities"
                    and source_name == "plugin_validate_single_target.py"
                ):
                    continue
                self.assertNotIn(
                    symbol,
                    source_text,
                    f"{symbol} belongs in the root capabilities owner, not {source_name}",
                )

        self.assertIn(
            "from .plugin_validate_capabilities import",
            single_target_text,
            "single-target owner should dispatch root capability checks to the leaf owner",
        )
        self.assertIn("validate_plugin_capabilities(", single_target_text)
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_single_target import",
            "from .plugin_validate_dependencies import",
            "from .plugin_validate_modules import",
            "from .plugin_validate_interfaces import",
        ):
            self.assertNotIn(
                forbidden_import,
                capabilities_text,
                "root capabilities owner must stay independent from entry and sibling owners",
            )
        self.assertLessEqual(
            len(capabilities_text.splitlines()),
            100,
            "root capabilities owner should stay small and focused",
        )

    def test_default_packaging_tests_live_in_default_packaging_test_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_DEFAULT_PACKAGING_TEST.exists(),
            "default packaging metadata tests belong in test_plugin_validate_default_packaging.py",
        )
        validate_test_text = PLUGIN_VALIDATE_TEST.read_text(encoding="utf-8")
        distribution_test_text = PLUGIN_VALIDATE_DISTRIBUTION_CONTRACT_TEST.read_text(
            encoding="utf-8"
        )
        default_packaging_test_text = PLUGIN_VALIDATE_DEFAULT_PACKAGING_TEST.read_text(
            encoding="utf-8"
        )

        for test_name in (
            "test_plugin_validate_rejects_missing_root_default_packaging",
            "test_plugin_validate_rejects_malformed_root_default_packaging",
            "test_plugin_validate_rejects_malformed_optional_feature_default_packaging",
            "test_plugin_validate_rejects_malformed_feature_extension_default_packaging",
        ):
            for source_name, source_text in (
                ("test_plugin_validate.py", validate_test_text),
                (
                    "test_plugin_validate_distribution_contract.py",
                    distribution_test_text,
                ),
            ):
                self.assertNotIn(
                    f"def {test_name}(",
                    source_text,
                    f"{test_name} belongs in the default packaging metadata test owner, not {source_name}",
                )
            self.assertIn(f"def {test_name}(", default_packaging_test_text)

    def test_default_packaging_lives_in_default_packaging_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_DEFAULT_PACKAGING_METADATA.exists(),
            "root, optional feature, and feature extension default_packaging metadata belongs in plugin_validate_default_packaging.py",
        )
        single_target_text = PLUGIN_VALIDATE_SINGLE_TARGET.read_text(encoding="utf-8")
        common_text = PLUGIN_VALIDATE_COMMON.read_text(encoding="utf-8")
        distribution_packaging_text = PLUGIN_VALIDATE_DISTRIBUTION_PACKAGING.read_text(
            encoding="utf-8"
        )
        default_packaging_text = PLUGIN_VALIDATE_DEFAULT_PACKAGING_METADATA.read_text(
            encoding="utf-8"
        )

        for symbol in (
            "PLUGIN_VALIDATE_DEFAULT_PACKAGING_METADATA_DUPLICATE_MESSAGE",
            "validate_plugin_default_packaging",
            "validate_plugin_default_packaging_values",
            "validate_plugin_optional_feature_default_packaging",
            "validate_plugin_feature_extension_default_packaging",
            "duplicates default_packaging",
        ):
            self.assertIn(symbol, default_packaging_text)
            for source_name, source_text in (
                ("plugin_validate_common.py", common_text),
                (
                    "plugin_validate_distribution_packaging.py",
                    distribution_packaging_text,
                ),
                ("plugin_validate_single_target.py", single_target_text),
            ):
                if (
                    symbol == "validate_plugin_default_packaging"
                    and source_name == "plugin_validate_single_target.py"
                ):
                    continue
                self.assertNotIn(
                    symbol,
                    source_text,
                    f"{symbol} belongs in the default packaging metadata owner, not {source_name}",
                )

        self.assertIn(
            "from .plugin_validate_default_packaging import",
            single_target_text,
            "single-target owner should dispatch default packaging metadata checks",
        )
        self.assertIn("validate_plugin_default_packaging(", single_target_text)
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_single_target import",
            "from .plugin_validate_distribution_contract import",
            "from .plugin_validate_distribution_packaging import",
        ):
            self.assertNotIn(
                forbidden_import,
                default_packaging_text,
                "default packaging metadata owner must stay independent from distribution and entry owners",
            )
        self.assertLessEqual(
            len(default_packaging_text.splitlines()),
            120,
            "default packaging metadata owner should stay small and focused",
        )

    def test_capability_statuses_live_in_capability_statuses_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_CAPABILITY_STATUSES.exists(),
            "package capability status validation belongs in a focused owner",
        )
        single_target_text = PLUGIN_VALIDATE_SINGLE_TARGET.read_text(encoding="utf-8")
        statuses_text = PLUGIN_VALIDATE_CAPABILITY_STATUSES.read_text(
            encoding="utf-8"
        )

        for symbol in (
            "PLUGIN_VALIDATE_CAPABILITY_STATUS_VALUES",
            "PLUGIN_VALIDATE_CAPABILITY_STATUS_FIELDS",
            "validate_plugin_capability_statuses",
            "validate_plugin_capability_status_rows",
            "validate_plugin_capability_status_known_fields",
            "validate_plugin_capability_status_row",
            "plugin_validate_capability_status_owned_capabilities",
            "is not a known capability_status field",
            "duplicates capability_status",
            "must reference a package or optional feature capability declared by the same package",
        ):
            self.assertIn(symbol, statuses_text)
        for function_name in (
            "validate_plugin_capability_statuses",
            "validate_plugin_capability_status_rows",
            "validate_plugin_capability_status_row",
            "plugin_validate_capability_status_owned_capabilities",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                single_target_text,
                f"{function_name} belongs in plugin_validate_capability_statuses.py",
            )
        self.assertIn(
            "from .plugin_validate_capability_statuses import",
            single_target_text,
            "single-target owner should dispatch capability status checks to the leaf owner",
        )
        self.assertIn(
            "validate_plugin_capability_statuses(",
            single_target_text,
        )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_single_target import",
            "from .plugin_validate_dependencies import",
            "from .plugin_validate_modules import",
            "from .plugin_validate_interfaces import",
        ):
            self.assertNotIn(
                forbidden_import,
                statuses_text,
                "capability status owner must stay independent from entry and sibling owners",
            )
        self.assertLessEqual(
            len(statuses_text.splitlines()),
            180,
            "capability status owner should stay small and focused",
        )

    def test_capability_status_tests_live_in_capability_status_test_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_CAPABILITY_STATUSES_TEST.exists(),
            "capability status behavior tests belong in test_plugin_validate_capability_statuses.py",
        )
        validate_test_text = PLUGIN_VALIDATE_TEST.read_text(encoding="utf-8")
        statuses_test_text = PLUGIN_VALIDATE_CAPABILITY_STATUSES_TEST.read_text(
            encoding="utf-8"
        )

        for test_name in (
            "test_plugin_validate_rejects_malformed_capability_status_row",
            "test_plugin_validate_rejects_duplicate_capability_status_row",
            "test_plugin_validate_rejects_unknown_capability_status_fields",
            "test_plugin_validate_rejects_capability_status_target_modes_drift",
            "test_plugin_validate_rejects_capability_status_bevy_reference_drift",
        ):
            self.assertNotIn(
                f"def {test_name}(",
                validate_test_text,
                f"{test_name} belongs in the capability status test owner",
            )
            self.assertIn(f"def {test_name}(", statuses_test_text)

    def test_capability_status_targets_live_in_capability_status_targets_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_CAPABILITY_STATUS_TARGETS.exists(),
            "capability status target mode checks belong in a focused owner",
        )
        statuses_text = PLUGIN_VALIDATE_CAPABILITY_STATUSES.read_text(encoding="utf-8")
        targets_text = PLUGIN_VALIDATE_CAPABILITY_STATUS_TARGETS.read_text(
            encoding="utf-8"
        )

        for symbol in (
            "PLUGIN_VALIDATE_CAPABILITY_STATUS_TARGET_MODES",
            "validate_plugin_capability_status_targets",
            "plugin_validate_capability_status_supported_targets",
            "duplicates capability_status target_modes",
            "should be covered by package supported_targets",
        ):
            self.assertIn(symbol, targets_text)
        for function_name in (
            "validate_plugin_capability_status_targets",
            "plugin_validate_capability_status_supported_targets",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                statuses_text,
                f"{function_name} belongs in plugin_validate_capability_status_targets.py",
            )
        self.assertIn(
            "from .plugin_validate_capability_status_targets import",
            statuses_text,
            "capability status owner should dispatch target checks to the leaf owner",
        )
        self.assertIn(
            "validate_plugin_capability_status_targets(",
            statuses_text,
        )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_single_target import",
            "from .plugin_validate_capability_statuses import",
            "from .plugin_validate_modules import",
            "from .plugin_validate_interfaces import",
        ):
            self.assertNotIn(
                forbidden_import,
                targets_text,
                "capability status targets owner must stay independent from entry and sibling owners",
            )
        self.assertLessEqual(
            len(targets_text.splitlines()),
            150,
            "capability status target owner should stay small and focused",
        )

    def test_capability_status_references_live_in_capability_status_references_owner(
        self,
    ):
        self.assertTrue(
            PLUGIN_VALIDATE_CAPABILITY_STATUS_REFERENCES.exists(),
            "capability status Bevy reference checks belong in a focused owner",
        )
        statuses_text = PLUGIN_VALIDATE_CAPABILITY_STATUSES.read_text(encoding="utf-8")
        references_text = PLUGIN_VALIDATE_CAPABILITY_STATUS_REFERENCES.read_text(
            encoding="utf-8"
        )

        for symbol in (
            "validate_plugin_capability_status_references",
            "validate_plugin_capability_status_bevy_reference",
            "duplicates capability_status bevy_references",
            "should start with dev/bevy/",
            "should not contain empty, current, or parent path segments",
        ):
            self.assertIn(symbol, references_text)
        for function_name in (
            "validate_plugin_capability_status_references",
            "validate_plugin_capability_status_bevy_reference",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                statuses_text,
                f"{function_name} belongs in plugin_validate_capability_status_references.py",
            )
        self.assertIn(
            "from .plugin_validate_capability_status_references import",
            statuses_text,
            "capability status owner should dispatch Bevy reference checks to the leaf owner",
        )
        self.assertIn(
            "validate_plugin_capability_status_references(",
            statuses_text,
        )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_single_target import",
            "from .plugin_validate_capability_statuses import",
            "from .plugin_validate_capability_status_targets import",
            "from .plugin_validate_modules import",
            "from .plugin_validate_interfaces import",
        ):
            self.assertNotIn(
                forbidden_import,
                references_text,
                "capability status references owner must stay independent from entry and sibling owners",
            )
        self.assertLessEqual(
            len(references_text.splitlines()),
            140,
            "capability status references owner should stay small and focused",
        )

    def test_dependency_capabilities_lives_in_capabilities_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_DEPENDENCY_CAPABILITIES.exists(),
            "top-level dependency capability resolution belongs in a focused owner",
        )
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        dependencies_text = PLUGIN_VALIDATE_DEPENDENCIES.read_text(encoding="utf-8")
        capabilities_text = PLUGIN_VALIDATE_DEPENDENCY_CAPABILITIES.read_text(
            encoding="utf-8"
        )

        for symbol in (
            "plugin_validate_dependency_capability_target_index",
            "validate_plugin_dependency_capability_gate",
            "plugin_validate_dependency_capability_is_host_owned",
            "referenced static plugin package",
            "runtime.module.* or runtime.capability.*",
        ):
            self.assertIn(symbol, capabilities_text)
        self.assertIn(
            "from .plugin_validate_dependency_capabilities import",
            dependencies_text,
            "dependencies owner should dispatch capability resolution to the leaf owner",
        )
        self.assertIn("validate_plugin_dependency_capability_gate(", dependencies_text)
        for function_name in (
            "plugin_validate_dependency_capability_target_index",
            "validate_plugin_dependency_capability_gate",
            "plugin_validate_dependency_capability_is_host_owned",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                dependencies_text,
                f"{function_name} belongs in dependency capabilities owner",
            )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_single_target import",
            "from .plugin_validate_options import",
            "from .plugin_validate_asset_importers import",
            "from .plugin_validate_feature_provider_dependencies import",
        ):
            self.assertNotIn(
                forbidden_import,
                capabilities_text,
                "dependency capabilities owner must stay independent",
            )
        self.assertNotIn(
            "validate_plugin_dependency_capability_gate(",
            validate_text,
            "validate entry owner must not dispatch dependency capability resolution",
        )
        self.assertLessEqual(
            len(capabilities_text.splitlines()),
            120,
            "dependency capabilities owner should stay small and focused",
        )


if __name__ == "__main__":
    unittest.main()
