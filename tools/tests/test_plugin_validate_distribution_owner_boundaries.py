import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
PLUGIN_VALIDATE_OWNER_BOUNDARIES_TEST = (
    REPO_ROOT / "tools/tests/test_plugin_validate_owner_boundaries.py"
)
PLUGIN_VALIDATE = REPO_ROOT / "tools/zircon_export/plugin_validate.py"
PLUGIN_VALIDATE_COMMON = REPO_ROOT / "tools/zircon_export/plugin_validate_common.py"
PLUGIN_VALIDATE_DISTRIBUTION_CONTRACT = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_distribution_contract.py"
)
PLUGIN_VALIDATE_DISTRIBUTION_DESCRIPTOR_SYMBOL = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_distribution_descriptor_symbol.py"
)
PLUGIN_VALIDATE_DISTRIBUTION_ENTRIES = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_distribution_entries.py"
)
PLUGIN_VALIDATE_DISTRIBUTION_PACKAGING = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_distribution_packaging.py"
)
PLUGIN_VALIDATE_DISTRIBUTION_SCALARS = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_distribution_scalars.py"
)
PLUGIN_VALIDATE_DISTRIBUTION_ASSETS = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_distribution_assets.py"
)
PLUGIN_VALIDATE_DISTRIBUTION_ASSET_MATCHES = (
    REPO_ROOT
    / "tools/zircon_export/plugin_validate_distribution_asset_matches.py"
)
PLUGIN_VALIDATE_DISTRIBUTION_ENGINE_COMPAT = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_distribution_engine_compat.py"
)
PLUGIN_VALIDATE_DISTRIBUTION_MODULES = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_distribution_modules.py"
)
PLUGIN_VALIDATE_DISTRIBUTION_MODULE_TARGET_MODES = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_distribution_module_target_modes.py"
)
PLUGIN_VALIDATE_ENGINE_VERSION = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_engine_version.py"
)

DISTRIBUTION_BOUNDARY_METHODS = (
    "test_distribution_contract_lives_in_distribution_contract_owner",
    "test_distribution_descriptor_symbol_lives_in_descriptor_owner",
    "test_distribution_entries_live_in_entries_owner",
    "test_distribution_packaging_lives_in_packaging_owner",
    "test_distribution_assets_lives_in_distribution_assets_owner",
    "test_distribution_engine_compat_lives_in_engine_compat_owner",
    "test_distribution_module_target_modes_lives_in_target_modes_owner",
    "test_distribution_scalar_fields_live_in_scalars_owner",
    "test_distribution_dist_form_constant_does_not_borrow_build_owner",
)


class PluginValidateDistributionOwnerBoundaryTests(unittest.TestCase):
    def test_distribution_boundaries_leave_general_owner_file(self):
        general_owner_text = PLUGIN_VALIDATE_OWNER_BOUNDARIES_TEST.read_text(
            encoding="utf-8"
        )

        for method_name in DISTRIBUTION_BOUNDARY_METHODS:
            self.assertNotIn(
                f"def {method_name}(",
                general_owner_text,
                f"{method_name} belongs in test_plugin_validate_distribution_owner_boundaries.py",
            )

        self.assertLessEqual(
            len(general_owner_text.splitlines()),
            3320,
            "general PluginValidate owner boundary tests should shrink after distribution split",
        )
        self.assertLessEqual(
            len(Path(__file__).read_text(encoding="utf-8").splitlines()),
            620,
            "focused PluginValidate distribution owner boundary file should stay narrow",
        )

    def test_distribution_contract_lives_in_distribution_contract_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_DISTRIBUTION_CONTRACT.exists(),
            "distribution manifest contract belongs in plugin_validate_distribution_contract.py",
        )
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        distribution_text = PLUGIN_VALIDATE_DISTRIBUTION_CONTRACT.read_text(
            encoding="utf-8"
        )

        for function_name in (
            "validate_plugin_distribution",
            "validate_plugin_distribution_known_fields",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                validate_text,
                f"{function_name} belongs in plugin_validate_distribution_contract.py",
            )
            self.assertIn(
                f"def {function_name}(",
                distribution_text,
            )
        self.assertIn(
            "PLUGIN_VALIDATE_DISTRIBUTION_FIELDS =",
            distribution_text,
            "distribution field closure belongs in the distribution contract owner",
        )
        self.assertIn(
            "is not a known distribution field",
            distribution_text,
            "unknown distribution diagnostics belong in the distribution contract owner",
        )

    def test_distribution_descriptor_symbol_lives_in_descriptor_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_DISTRIBUTION_DESCRIPTOR_SYMBOL.exists(),
            "distribution.descriptor_symbol validation belongs in plugin_validate_distribution_descriptor_symbol.py",
        )
        distribution_text = PLUGIN_VALIDATE_DISTRIBUTION_CONTRACT.read_text(
            encoding="utf-8"
        )
        descriptor_text = PLUGIN_VALIDATE_DISTRIBUTION_DESCRIPTOR_SYMBOL.read_text(
            encoding="utf-8"
        )

        self.assertNotIn(
            "def plugin_validate_descriptor_symbol(",
            distribution_text,
            "descriptor symbol validation must leave the distribution contract owner",
        )
        self.assertIn(
            "def plugin_validate_descriptor_symbol(",
            descriptor_text,
        )
        self.assertNotIn(
            "PLUGIN_VALIDATE_DESCRIPTOR_SYMBOL_V3 =",
            distribution_text,
            "descriptor symbol constant belongs in the descriptor owner",
        )
        self.assertIn(
            "PLUGIN_VALIDATE_DESCRIPTOR_SYMBOL_V3 =",
            descriptor_text,
        )
        self.assertIn(
            "from .plugin_validate_distribution_descriptor_symbol import",
            distribution_text,
            "distribution contract owner should dispatch descriptor_symbol validation to the leaf owner",
        )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_distribution_contract import",
        ):
            self.assertNotIn(
                forbidden_import,
                descriptor_text,
                "distribution descriptor symbol owner must stay independent from entry, build, and parent contract owners",
            )
        self.assertLessEqual(
            len(distribution_text.splitlines()),
            190,
            "distribution contract owner should shrink after descriptor symbol owner split",
        )
        self.assertLessEqual(
            len(descriptor_text.splitlines()),
            60,
            "distribution descriptor symbol owner should stay a focused leaf module",
        )

    def test_distribution_entries_live_in_entries_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_DISTRIBUTION_ENTRIES.exists(),
            "distribution runtime/editor entry validation belongs in plugin_validate_distribution_entries.py",
        )
        distribution_text = PLUGIN_VALIDATE_DISTRIBUTION_CONTRACT.read_text(
            encoding="utf-8"
        )
        entries_text = PLUGIN_VALIDATE_DISTRIBUTION_ENTRIES.read_text(
            encoding="utf-8"
        )

        self.assertNotIn(
            "plugin_validate_optional_trimmed_string",
            distribution_text,
            "runtime/editor entry parsing belongs in the entries owner",
        )
        self.assertNotIn(
            "must declare runtime_entry or editor_entry",
            distribution_text,
            "entry-required diagnostics must leave the distribution contract owner",
        )
        self.assertIn(
            "def plugin_validate_distribution_entries(",
            entries_text,
        )
        self.assertIn(
            "must declare runtime_entry or editor_entry",
            entries_text,
        )
        self.assertIn(
            "from .plugin_validate_distribution_entries import",
            distribution_text,
            "distribution contract owner should dispatch runtime/editor entries to the leaf owner",
        )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_distribution_contract import",
        ):
            self.assertNotIn(
                forbidden_import,
                entries_text,
                "distribution entries owner must stay independent from entry, build, and parent contract owners",
            )
        self.assertLessEqual(
            len(distribution_text.splitlines()),
            170,
            "distribution contract owner should shrink after entry owner split",
        )
        self.assertLessEqual(
            len(entries_text.splitlines()),
            70,
            "distribution entries owner should stay a focused leaf module",
        )

    def test_distribution_packaging_lives_in_packaging_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_DISTRIBUTION_PACKAGING.exists(),
            "distribution forms/default_packaging validation belongs in plugin_validate_distribution_packaging.py",
        )
        distribution_text = PLUGIN_VALIDATE_DISTRIBUTION_CONTRACT.read_text(
            encoding="utf-8"
        )
        packaging_text = PLUGIN_VALIDATE_DISTRIBUTION_PACKAGING.read_text(
            encoding="utf-8"
        )

        for symbol in (
            "PLUGIN_VALIDATE_DIST_FORM",
            "PLUGIN_VALIDATE_DIST_PACKAGING",
            "PLUGIN_VALIDATE_DISTRIBUTION_FORMS",
            "PLUGIN_VALIDATE_DEFAULT_PACKAGING",
            "PLUGIN_VALIDATE_DISTRIBUTION_FORMS_DUPLICATE_MESSAGE",
            "PLUGIN_VALIDATE_DEFAULT_PACKAGING_DUPLICATE_MESSAGE",
            "plugin_validate_allowed_string_values",
            "plugin_validate_append_once",
            "plugin_validate_string_array",
            "plugin_validate_distribution_unique_values",
        ):
            self.assertNotIn(
                symbol,
                distribution_text,
                f"{symbol} belongs behind the distribution packaging owner import boundary",
            )
            self.assertIn(
                symbol,
                packaging_text,
                f"{symbol} should be consumed by the distribution packaging owner",
            )
        self.assertIn(
            "def plugin_validate_distribution_packaging(",
            packaging_text,
        )
        self.assertIn(
            "from .plugin_validate_distribution_packaging import",
            distribution_text,
            "distribution contract owner should dispatch forms/default_packaging to the leaf owner",
        )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_distribution_contract import",
        ):
            self.assertNotIn(
                forbidden_import,
                packaging_text,
                "distribution packaging owner must stay independent from entry, build, and parent contract owners",
            )
        self.assertLessEqual(
            len(distribution_text.splitlines()),
            120,
            "distribution contract owner should shrink after packaging owner split",
        )
        self.assertLessEqual(
            len(packaging_text.splitlines()),
            95,
            "distribution packaging owner should stay a focused leaf module",
        )

    def test_distribution_assets_lives_in_distribution_assets_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_DISTRIBUTION_ASSETS.exists(),
            "distribution.assets glob validation belongs in plugin_validate_distribution_assets.py",
        )
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        distribution_text = PLUGIN_VALIDATE_DISTRIBUTION_CONTRACT.read_text(
            encoding="utf-8"
        )
        assets_text = PLUGIN_VALIDATE_DISTRIBUTION_ASSETS.read_text(encoding="utf-8")
        self.assertTrue(
            PLUGIN_VALIDATE_DISTRIBUTION_ASSET_MATCHES.exists(),
            "distribution asset filesystem matching belongs in a dedicated leaf owner",
        )
        asset_matches_text = PLUGIN_VALIDATE_DISTRIBUTION_ASSET_MATCHES.read_text(
            encoding="utf-8"
        )

        self.assertNotIn(
            "def plugin_validate_distribution_assets(",
            validate_text,
            "plugin_validate.py must not own distribution.assets glob validation",
        )
        self.assertNotIn(
            "def plugin_validate_distribution_assets(",
            distribution_text,
            "distribution contract owner must dispatch distribution.assets validation through its leaf owner",
        )
        self.assertIn(
            "def plugin_validate_distribution_assets(",
            assets_text,
        )
        self.assertNotIn(
            "PLUGIN_VALIDATE_RETIRED_UI_ASSET_SUFFIXES",
            distribution_text,
            "retired UI asset suffix checks belong in plugin_validate_distribution_assets.py",
        )
        self.assertIn("PLUGIN_VALIDATE_RETIRED_UI_ASSET_SUFFIXES =", assets_text)
        self.assertIn(
            "def plugin_validate_retired_ui_asset_suffix(",
            assets_text,
        )
        self.assertIn(
            "def plugin_validate_retired_ui_asset_pattern_suffix(",
            assets_text,
        )
        self.assertIn(
            "from .plugin_validate_distribution_assets import",
            distribution_text,
            "distribution contract owner should import distribution.assets validation from the leaf owner",
        )
        self.assertNotIn("plugin_root.glob(", distribution_text)
        self.assertNotIn(
            "plugin_root.glob(",
            assets_text,
            "filesystem glob matching belongs in its dedicated leaf owner",
        )
        self.assertIn("plugin_root.glob(", asset_matches_text)
        self.assertIn(
            "from .plugin_validate_distribution_asset_matches import",
            assets_text,
        )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_distribution_contract import",
        ):
            self.assertNotIn(
                forbidden_import,
                assets_text,
                "distribution.assets owner must not borrow entry or parent orchestration owners",
            )
        self.assertLessEqual(
            len(distribution_text.splitlines()),
            300,
            "distribution contract owner should stay below the split budget",
        )
        self.assertLessEqual(
            len(assets_text.splitlines()),
            120,
            "distribution.assets owner should stay small and focused",
        )
        self.assertLessEqual(
            len(asset_matches_text.splitlines()),
            70,
            "distribution asset filesystem match owner should stay focused",
        )

    def test_distribution_engine_compat_lives_in_engine_compat_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_DISTRIBUTION_ENGINE_COMPAT.exists(),
            "distribution.engine_compat parsing belongs in plugin_validate_distribution_engine_compat.py",
        )
        distribution_text = PLUGIN_VALIDATE_DISTRIBUTION_CONTRACT.read_text(
            encoding="utf-8"
        )
        engine_compat_text = PLUGIN_VALIDATE_DISTRIBUTION_ENGINE_COMPAT.read_text(
            encoding="utf-8"
        )

        for function_name in (
            "plugin_validate_engine_compat",
            "plugin_validate_engine_compat_matches",
            "plugin_validate_parse_engine_comparator",
            "plugin_validate_parse_engine_version",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                distribution_text,
                f"{function_name} belongs in plugin_validate_distribution_engine_compat.py",
            )
            self.assertIn(
                f"def {function_name}(",
                engine_compat_text,
            )

        self.assertIn(
            "from .plugin_validate_distribution_engine_compat import",
            distribution_text,
            "distribution contract owner should dispatch engine_compat checks to the leaf owner",
        )
        engine_version_text = PLUGIN_VALIDATE_ENGINE_VERSION.read_text(
            encoding="utf-8"
        )
        self.assertIn(
            "from .plugin_validate_distribution_engine_compat import",
            engine_version_text,
            "engine version owner should consume the shared engine version parser from the leaf owner",
        )
        self.assertNotIn(
            "from .plugin_validate_distribution_contract import plugin_validate_parse_engine_version",
            engine_version_text,
            "engine version owner must not borrow parser helpers from distribution contract owner",
        )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_distribution_contract import",
        ):
            self.assertNotIn(
                forbidden_import,
                engine_compat_text,
                "engine compatibility owner must stay independent from entry, build, and parent contract owners",
            )
        self.assertLessEqual(
            len(distribution_text.splitlines()),
            230,
            "distribution contract owner should stay below the next split budget",
        )
        self.assertLessEqual(
            len(engine_compat_text.splitlines()),
            120,
            "distribution engine compatibility owner should stay a focused leaf module",
        )

    def test_distribution_module_target_modes_lives_in_target_modes_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_DISTRIBUTION_MODULE_TARGET_MODES.exists(),
            "distribution module target_modes checks belong in plugin_validate_distribution_module_target_modes.py",
        )
        distribution_modules_text = (
            PLUGIN_VALIDATE_DISTRIBUTION_MODULES.read_text(encoding="utf-8")
        )
        target_modes_text = PLUGIN_VALIDATE_DISTRIBUTION_MODULE_TARGET_MODES.read_text(
            encoding="utf-8"
        )

        for symbol in (
            "PLUGIN_VALIDATE_RUNTIME_TARGET_MODES",
            "PLUGIN_VALIDATE_EDITOR_TARGET_MODE",
            "PLUGIN_VALIDATE_TARGET_MODES",
        ):
            self.assertNotIn(
                f"{symbol} =",
                distribution_modules_text,
                f"{symbol} belongs in plugin_validate_distribution_module_target_modes.py",
            )
            self.assertIn(f"{symbol} =", target_modes_text)

        self.assertNotIn(
            "def plugin_validate_distribution_module_target_modes(",
            distribution_modules_text,
            "target_modes helper belongs in plugin_validate_distribution_module_target_modes.py",
        )
        self.assertIn(
            "def plugin_validate_distribution_module_target_modes(",
            target_modes_text,
        )
        self.assertIn(
            "from .plugin_validate_distribution_module_target_modes import",
            distribution_modules_text,
            "distribution modules owner must dispatch target_modes checks to the target_modes owner",
        )
        self.assertNotIn(
            "from .plugin_validate_distribution_modules import",
            target_modes_text,
            "target_modes owner must not import its parent distribution modules owner",
        )
        self.assertLessEqual(
            len(distribution_modules_text.splitlines()),
            110,
            "distribution modules owner should stay focused on manifest source/module binding",
        )
        self.assertLessEqual(
            len(target_modes_text.splitlines()),
            90,
            "distribution module target_modes owner should stay a small leaf",
        )

    def test_distribution_scalar_fields_live_in_scalars_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_DISTRIBUTION_SCALARS.exists(),
            "distribution scalar field parsing belongs in plugin_validate_distribution_scalars.py",
        )
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        distribution_text = PLUGIN_VALIDATE_DISTRIBUTION_CONTRACT.read_text(
            encoding="utf-8"
        )
        scalars_text = PLUGIN_VALIDATE_DISTRIBUTION_SCALARS.read_text(
            encoding="utf-8"
        )

        for symbol in (
            "PLUGIN_VALIDATE_ABI_VERSION_V3",
            "plugin_validate_int",
            "plugin_validate_trimmed_string",
            "plugin_validate_distribution_dist_crate",
            "plugin_validate_distribution_abi_version",
            "PluginValidateDistributionScalars",
        ):
            self.assertNotIn(
                symbol,
                distribution_text,
                f"{symbol} belongs behind the distribution scalar owner import boundary",
            )
            self.assertIn(
                symbol,
                scalars_text,
                f"{symbol} should be consumed by the distribution scalar owner",
            )
        self.assertIn(
            "def plugin_validate_distribution_scalars(",
            scalars_text,
        )

        self.assertNotIn(
            "plugin_validate_distribution_dist_crate",
            validate_text,
            "plugin validate must not own distribution.dist_crate parsing",
        )
        self.assertNotIn(
            "plugin_validate_distribution_abi_version",
            validate_text,
            "plugin validate must not own distribution.abi_version parsing",
        )
        self.assertIn(
            "from .plugin_validate_distribution_scalars import",
            distribution_text,
            "distribution contract owner should dispatch dist_crate/abi_version to the scalar owner",
        )
        self.assertNotIn(
            "plugin_distribution_dist_crate",
            validate_text,
            "plugin validate must not borrow distribution field parsing from plugin_build.py",
        )
        self.assertNotIn(
            "plugin_distribution_abi_version",
            validate_text,
            "plugin validate must not borrow distribution field parsing from plugin_build.py",
        )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_distribution_contract import",
        ):
            self.assertNotIn(
                forbidden_import,
                scalars_text,
                "distribution scalar owner must stay independent from entry, build, and parent contract owners",
            )
        self.assertLessEqual(
            len(distribution_text.splitlines()),
            120,
            "distribution contract owner should stay small while owning field closure",
        )
        self.assertLessEqual(
            len(scalars_text.splitlines()),
            90,
            "distribution scalar owner should stay a focused leaf module",
        )

    def test_distribution_dist_form_constant_does_not_borrow_build_owner(self):
        common_text = PLUGIN_VALIDATE_COMMON.read_text(encoding="utf-8")
        distribution_text = PLUGIN_VALIDATE_DISTRIBUTION_CONTRACT.read_text(
            encoding="utf-8"
        )

        self.assertIn(
            "PLUGIN_VALIDATE_DIST_FORM =",
            common_text,
            "validate-specific dist form constant belongs in plugin_validate_common.py",
        )
        self.assertNotIn(
            "PLUGIN_BUILD_DIST_FORM",
            distribution_text,
            "distribution contract validation must not borrow dist form from plugin_build.py",
        )
        self.assertNotIn(
            "from .plugin_build import",
            distribution_text,
            "distribution contract validation must stay independent from build owner imports",
        )


if __name__ == "__main__":
    unittest.main()
