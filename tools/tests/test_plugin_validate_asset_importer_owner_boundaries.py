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
PLUGIN_VALIDATE_ASSET_IMPORTERS = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_asset_importers.py"
)
PLUGIN_VALIDATE_ASSET_IMPORTER_IDS = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_asset_importer_ids.py"
)
PLUGIN_VALIDATE_ASSET_IMPORTER_GLOBAL_IDS = (
    REPO_ROOT
    / "tools/zircon_export/plugin_validate_asset_importer_global_ids.py"
)
PLUGIN_VALIDATE_ASSET_IMPORTER_NUMBERS = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_asset_importer_numbers.py"
)
PLUGIN_VALIDATE_ASSET_IMPORTER_METADATA_ARRAYS = (
    REPO_ROOT
    / "tools/zircon_export/plugin_validate_asset_importer_metadata_arrays.py"
)
PLUGIN_VALIDATE_ASSET_IMPORTER_SCHEMA = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_asset_importer_schema.py"
)
PLUGIN_VALIDATE_ASSET_IMPORTER_RESOURCE_KINDS = (
    REPO_ROOT / "tools/zircon_export/plugin_validate_asset_importer_resource_kinds.py"
)
PLUGIN_VALIDATE_ASSET_IMPORTER_REQUIRED_CAPABILITIES = (
    REPO_ROOT
    / "tools/zircon_export/plugin_validate_asset_importer_required_capabilities.py"
)
PLUGIN_VALIDATE_ASSET_IMPORTER_REQUIRED_CAPABILITY_GATES = (
    REPO_ROOT
    / "tools/zircon_export/plugin_validate_asset_importer_required_capability_gates.py"
)

ASSET_IMPORTER_BOUNDARY_METHODS = (
    "test_asset_importers_lives_in_asset_importers_owner",
    "test_asset_importer_ids_lives_in_ids_owner",
    "test_asset_importer_global_ids_lives_in_global_id_owner",
    "test_asset_importer_numbers_lives_in_numbers_owner",
    "test_asset_importer_metadata_arrays_lives_in_metadata_array_owner",
    "test_asset_importer_required_capabilities_lives_in_required_capability_owner",
    "test_asset_importer_required_capability_gates_lives_in_gate_owner",
    "test_asset_importer_schema_lives_in_schema_owner",
    "test_asset_importer_resource_kinds_lives_in_resource_kind_owner",
)


class PluginValidateAssetImporterOwnerBoundaryTests(unittest.TestCase):
    def test_asset_importer_boundaries_leave_general_owner_file(self):
        general_owner_text = PLUGIN_VALIDATE_OWNER_BOUNDARIES_TEST.read_text(
            encoding="utf-8"
        )

        for method_name in ASSET_IMPORTER_BOUNDARY_METHODS:
            self.assertNotIn(
                f"def {method_name}(",
                general_owner_text,
                f"{method_name} belongs in test_plugin_validate_asset_importer_owner_boundaries.py",
            )

        self.assertLessEqual(
            len(general_owner_text.splitlines()),
            2600,
            "general PluginValidate owner boundary tests should shrink after asset-importer split",
        )
        self.assertLessEqual(
            len(Path(__file__).read_text(encoding="utf-8").splitlines()),
            680,
            "focused PluginValidate asset-importer owner boundary file should stay narrow",
        )

    def test_asset_importers_lives_in_asset_importers_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_ASSET_IMPORTERS.exists(),
            "asset_importers full_suffixes validation belongs in plugin_validate_asset_importers.py",
        )
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        single_target_text = PLUGIN_VALIDATE_SINGLE_TARGET.read_text(encoding="utf-8")
        asset_importers_text = PLUGIN_VALIDATE_ASSET_IMPORTERS.read_text(
            encoding="utf-8"
        )

        for parent_text, parent_name in (
            (validate_text, "plugin_validate.py"),
            (single_target_text, "plugin_validate_single_target.py"),
        ):
            self.assertNotIn(
                "def validate_plugin_asset_importers(",
                parent_text,
                f"{parent_name} must not own asset_importers full_suffixes validation",
            )
            self.assertNotIn(
                "full_suffixes",
                parent_text,
                f"{parent_name} must not inline asset importer suffix policy",
            )

        self.assertIn(
            "from .plugin_validate_asset_importers import validate_plugin_asset_importers",
            single_target_text,
            "single-target orchestration should dispatch asset_importers validation through the leaf owner",
        )
        self.assertIn(
            "def validate_plugin_asset_importers(",
            asset_importers_text,
        )
        self.assertIn(
            "def validate_plugin_asset_importer_source_selector(",
            asset_importers_text,
        )
        self.assertIn("asset_importers", asset_importers_text)
        self.assertIn("full_suffixes", asset_importers_text)
        self.assertIn("source_extensions", asset_importers_text)
        self.assertIn("must not be empty when declared", asset_importers_text)
        self.assertIn("must declare source_extensions or full_suffixes", asset_importers_text)
        self.assertIn("duplicates entry", asset_importers_text)
        self.assertIn("must be a dotted suffix", asset_importers_text)
        self.assertIn("must be lowercase", asset_importers_text)
        self.assertIn("read_toml", asset_importers_text)
        self.assertIn(
            "plugin_validate_retired_ui_asset_pattern_suffix",
            asset_importers_text,
            "asset_importers owner should reuse the retired UI suffix predicate",
        )
        self.assertIn(
            "from .plugin_validate_asset_importer_schema import",
            asset_importers_text,
            "asset_importers owner should dispatch field schema checks to the schema owner",
        )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_single_target import",
        ):
            self.assertNotIn(
                forbidden_import,
                asset_importers_text,
                "asset_importers owner must not borrow entry or parent orchestration owners",
            )
        self.assertLessEqual(
            len(asset_importers_text.splitlines()),
            120,
            "asset_importers owner should stay small and focused",
        )

    def test_asset_importer_ids_lives_in_ids_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_ASSET_IMPORTER_IDS.exists(),
            "asset_importer dot-namespaced id checks belong in a focused leaf owner",
        )
        asset_importers_text = PLUGIN_VALIDATE_ASSET_IMPORTERS.read_text(
            encoding="utf-8"
        )
        schema_text = PLUGIN_VALIDATE_ASSET_IMPORTER_SCHEMA.read_text(encoding="utf-8")
        ids_text = PLUGIN_VALIDATE_ASSET_IMPORTER_IDS.read_text(encoding="utf-8")

        for symbol in (
            "validate_plugin_asset_importer_id",
            "plugin_validate_asset_importer_id_segment",
            "at least two dot-separated namespace segments",
            "empty namespace segments",
            "lowercase ASCII letters, digits, underscores, and dots",
        ):
            self.assertIn(symbol, ids_text)
        self.assertIn(
            "from .plugin_validate_asset_importer_ids import",
            asset_importers_text,
            "asset_importers owner should dispatch id namespace checks to the id owner",
        )
        self.assertIn("validate_plugin_asset_importer_id(", asset_importers_text)
        self.assertNotIn(
            "def validate_plugin_asset_importer_id(",
            asset_importers_text,
            "id namespace helper belongs in plugin_validate_asset_importer_ids.py",
        )
        for phrase in (
            "at least two dot-separated namespace segments",
            "empty namespace segments",
            "lowercase ASCII letters, digits, underscores, and dots",
        ):
            self.assertNotIn(
                phrase,
                schema_text,
                "schema owner should not inline dot-namespaced id policy",
            )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_single_target import",
            "from .plugin_validate_asset_importers import",
            "from .plugin_validate_asset_importer_schema import",
        ):
            self.assertNotIn(
                forbidden_import,
                ids_text,
                "asset importer id owner must stay independent",
            )
        self.assertLessEqual(
            len(ids_text.splitlines()),
            90,
            "asset importer id owner should stay a small leaf module",
        )

    def test_asset_importer_global_ids_lives_in_global_id_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_ASSET_IMPORTER_GLOBAL_IDS.exists(),
            "asset_importer global id uniqueness checks belong in a focused --all owner",
        )
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        single_target_text = PLUGIN_VALIDATE_SINGLE_TARGET.read_text(encoding="utf-8")
        asset_importers_text = PLUGIN_VALIDATE_ASSET_IMPORTERS.read_text(
            encoding="utf-8"
        )
        global_ids_text = PLUGIN_VALIDATE_ASSET_IMPORTER_GLOBAL_IDS.read_text(
            encoding="utf-8"
        )

        for symbol in (
            "validate_plugin_asset_importer_global_ids",
            "validate_plugin_asset_importer_manifest_global_ids",
            "plugin_validate_asset_importer_global_package_label",
            "asset_importers",
            "is duplicated by",
            "plugin validate asset_importers id",
        ):
            self.assertIn(symbol, global_ids_text)
        self.assertIn(
            "from .plugin_validate_asset_importer_global_ids import",
            validate_text,
            "plugin validate --all should dispatch global importer id checks",
        )
        self.assertIn("validate_plugin_asset_importer_global_ids(", validate_text)
        for parent_text, parent_name in (
            (single_target_text, "plugin_validate_single_target.py"),
            (asset_importers_text, "plugin_validate_asset_importers.py"),
        ):
            self.assertNotIn(
                "validate_plugin_asset_importer_global_ids(",
                parent_text,
                f"{parent_name} must not own cross-manifest importer id checks",
            )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate_single_target import",
            "from .plugin_validate_asset_importers import",
            "from .plugin_validate_asset_importer_schema import",
        ):
            self.assertNotIn(
                forbidden_import,
                global_ids_text,
                "global importer id owner must stay independent",
            )
        self.assertLessEqual(
            len(global_ids_text.splitlines()),
            110,
            "global importer id owner should stay a small leaf module",
        )

    def test_asset_importer_numbers_lives_in_numbers_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_ASSET_IMPORTER_NUMBERS.exists(),
            "asset_importer numeric range checks belong in a focused leaf owner",
        )
        asset_importers_text = PLUGIN_VALIDATE_ASSET_IMPORTERS.read_text(
            encoding="utf-8"
        )
        schema_text = PLUGIN_VALIDATE_ASSET_IMPORTER_SCHEMA.read_text(encoding="utf-8")
        numbers_text = PLUGIN_VALIDATE_ASSET_IMPORTER_NUMBERS.read_text(
            encoding="utf-8"
        )

        for symbol in (
            "validate_plugin_asset_importer_numbers",
            "validate_plugin_asset_importer_priority_range",
            "validate_plugin_asset_importer_version_range",
            "I32_MIN",
            "I32_MAX",
            "U32_MAX",
            "must fit i32",
            "must be a positive u32",
        ):
            self.assertIn(symbol, numbers_text)
        self.assertIn(
            "from .plugin_validate_asset_importer_numbers import",
            asset_importers_text,
            "asset_importers owner should dispatch numeric range checks",
        )
        self.assertIn("validate_plugin_asset_importer_numbers(", asset_importers_text)
        for phrase in ("must fit i32", "must be a positive u32"):
            self.assertNotIn(
                phrase,
                schema_text,
                "schema owner should not inline numeric range policy",
            )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_single_target import",
            "from .plugin_validate_asset_importers import",
            "from .plugin_validate_asset_importer_schema import",
        ):
            self.assertNotIn(
                forbidden_import,
                numbers_text,
                "asset importer numeric owner must stay independent",
            )
        self.assertLessEqual(
            len(numbers_text.splitlines()),
            90,
            "asset importer numeric owner should stay a small leaf module",
        )

    def test_asset_importer_metadata_arrays_lives_in_metadata_array_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_ASSET_IMPORTER_METADATA_ARRAYS.exists(),
            "asset_importer metadata array duplicate checks belong in a focused leaf owner",
        )
        asset_importers_text = PLUGIN_VALIDATE_ASSET_IMPORTERS.read_text(
            encoding="utf-8"
        )
        schema_text = PLUGIN_VALIDATE_ASSET_IMPORTER_SCHEMA.read_text(encoding="utf-8")
        metadata_arrays_text = PLUGIN_VALIDATE_ASSET_IMPORTER_METADATA_ARRAYS.read_text(
            encoding="utf-8"
        )

        for symbol in (
            "ASSET_IMPORTER_METADATA_ARRAYS",
            "validate_plugin_asset_importer_metadata_arrays",
            "validate_plugin_asset_importer_metadata_array_unique_entries",
            "additional_output_kinds",
            "required_capabilities",
            "must not be empty when declared",
            "duplicates entry",
        ):
            self.assertIn(symbol, metadata_arrays_text)
        self.assertIn(
            "from .plugin_validate_asset_importer_metadata_arrays import",
            asset_importers_text,
            "asset_importers owner should dispatch metadata array duplicate checks",
        )
        self.assertIn(
            "validate_plugin_asset_importer_metadata_arrays(",
            asset_importers_text,
        )
        self.assertNotIn(
            "def validate_plugin_asset_importer_metadata_arrays(",
            asset_importers_text,
            "metadata array duplicate helper belongs in its focused leaf owner",
        )
        self.assertNotIn(
            "def validate_plugin_asset_importer_metadata_arrays(",
            schema_text,
            "schema owner should not inline metadata array duplicate policy",
        )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_single_target import",
            "from .plugin_validate_asset_importers import",
            "from .plugin_validate_asset_importer_schema import",
        ):
            self.assertNotIn(
                forbidden_import,
                metadata_arrays_text,
                "asset importer metadata array owner must stay independent",
            )
        self.assertLessEqual(
            len(metadata_arrays_text.splitlines()),
            90,
            "asset importer metadata array owner should stay a small leaf module",
        )

    def test_asset_importer_required_capabilities_lives_in_required_capability_owner(
        self,
    ):
        self.assertTrue(
            PLUGIN_VALIDATE_ASSET_IMPORTER_REQUIRED_CAPABILITIES.exists(),
            "asset_importer required capability namespace checks belong in a focused leaf owner",
        )
        asset_importers_text = PLUGIN_VALIDATE_ASSET_IMPORTERS.read_text(
            encoding="utf-8"
        )
        schema_text = PLUGIN_VALIDATE_ASSET_IMPORTER_SCHEMA.read_text(encoding="utf-8")
        required_capabilities_text = (
            PLUGIN_VALIDATE_ASSET_IMPORTER_REQUIRED_CAPABILITIES.read_text(
                encoding="utf-8"
            )
        )

        for symbol in (
            "validate_plugin_asset_importer_required_capabilities",
            "validate_plugin_asset_importer_required_capability_namespace",
            "plugin_validate_asset_importer_required_capability_segment",
            "required_capabilities",
            "at least two dot-separated namespace segments",
            "lowercase ASCII letters, digits, underscores, and dots",
        ):
            self.assertIn(symbol, required_capabilities_text)
        self.assertIn(
            "from .plugin_validate_asset_importer_required_capabilities import",
            asset_importers_text,
            "asset_importers owner should dispatch required capability namespace checks",
        )
        self.assertIn(
            "validate_plugin_asset_importer_required_capabilities(",
            asset_importers_text,
        )
        for phrase in (
            "at least two dot-separated namespace segments",
            "lowercase ASCII letters, digits, underscores, and dots",
        ):
            self.assertNotIn(
                phrase,
                schema_text,
                "schema owner should not inline required capability namespace policy",
            )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_single_target import",
            "from .plugin_validate_asset_importers import",
            "from .plugin_validate_asset_importer_schema import",
        ):
            self.assertNotIn(
                forbidden_import,
                required_capabilities_text,
                "asset importer required capability owner must stay independent",
            )
        self.assertLessEqual(
            len(required_capabilities_text.splitlines()),
            110,
            "asset importer required capability owner should stay a small leaf module",
        )

    def test_asset_importer_required_capability_gates_lives_in_gate_owner(
        self,
    ):
        self.assertTrue(
            PLUGIN_VALIDATE_ASSET_IMPORTER_REQUIRED_CAPABILITY_GATES.exists(),
            "asset_importer required capability declared/host gates belong in a focused leaf owner",
        )
        validate_text = PLUGIN_VALIDATE.read_text(encoding="utf-8")
        asset_importers_text = PLUGIN_VALIDATE_ASSET_IMPORTERS.read_text(
            encoding="utf-8"
        )
        namespace_text = PLUGIN_VALIDATE_ASSET_IMPORTER_REQUIRED_CAPABILITIES.read_text(
            encoding="utf-8"
        )
        gate_text = PLUGIN_VALIDATE_ASSET_IMPORTER_REQUIRED_CAPABILITY_GATES.read_text(
            encoding="utf-8"
        )

        for symbol in (
            "plugin_validate_static_declared_capabilities",
            "validate_plugin_asset_importer_required_capability_gates",
            "plugin_validate_required_capability_is_host_owned",
            "runtime.asset.importer.native",
            "runtime.capability.",
            "should reference a declared static package/feature capability",
        ):
            self.assertIn(symbol, gate_text)
            self.assertNotIn(
                symbol,
                namespace_text,
                "declared/host capability gates must not live in namespace format checks",
            )
            self.assertNotIn(
                symbol,
                validate_text,
                "declared/host capability gates must not live in plugin_validate.py",
            )
        self.assertIn(
            "from .plugin_validate_asset_importer_required_capability_gates import",
            asset_importers_text,
            "asset_importers owner should dispatch declared/host capability gates",
        )
        self.assertIn(
            "validate_plugin_asset_importer_required_capability_gates(",
            asset_importers_text,
        )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_asset_importers import",
        ):
            self.assertNotIn(
                forbidden_import,
                gate_text,
                "required capability gate owner must stay independent from entry, build, and parent owners",
            )
        self.assertLessEqual(
            len(gate_text.splitlines()),
            150,
            "asset_importer required capability gate owner should stay focused",
        )

    def test_asset_importer_schema_lives_in_schema_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_ASSET_IMPORTER_SCHEMA.exists(),
            "asset_importers field schema checks belong in plugin_validate_asset_importer_schema.py",
        )
        asset_importers_text = PLUGIN_VALIDATE_ASSET_IMPORTERS.read_text(
            encoding="utf-8"
        )
        schema_text = PLUGIN_VALIDATE_ASSET_IMPORTER_SCHEMA.read_text(encoding="utf-8")

        for function_name in (
            "validate_plugin_asset_importer_schema",
            "validate_plugin_asset_importer_required_string",
            "validate_plugin_asset_importer_required_integer",
            "validate_plugin_asset_importer_positive_integer",
            "validate_plugin_asset_importer_string_array",
            "validate_plugin_asset_importer_source_extensions",
            "validate_plugin_asset_importer_plugin_id_match",
        ):
            self.assertNotIn(
                f"def {function_name}(",
                asset_importers_text,
                f"{function_name} belongs in plugin_validate_asset_importer_schema.py",
            )
            self.assertIn(f"def {function_name}(", schema_text)

        for schema_phrase in (
            "PLUGIN_VALIDATE_ASSET_IMPORTER_FIELDS",
            "validate_plugin_asset_importer_known_fields",
            "id",
            "plugin_id",
            "package_id",
            "must match package id",
            "priority",
            "output_kind",
            "importer_version",
            "source_extensions",
            "full_suffixes for dotted suffixes",
            "must be lowercase",
            "duplicates entry",
            "additional_output_kinds",
            "required_capabilities",
            "must be a non-empty string",
            "must be a positive integer",
            "is not a known asset_importer field",
        ):
            self.assertIn(schema_phrase, schema_text)
        self.assertNotIn(
            "def validate_plugin_asset_importer_source_selector(",
            schema_text,
            "cross-field selector presence belongs in plugin_validate_asset_importers.py",
        )
        self.assertNotIn(
            "must be a dotted suffix",
            schema_text,
            "full_suffixes format policy belongs in plugin_validate_asset_importers.py",
        )

        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_single_target import",
        ):
            self.assertNotIn(
                forbidden_import,
                schema_text,
                "asset importer schema owner must not borrow entry or parent orchestration owners",
            )
        self.assertLessEqual(
            len(schema_text.splitlines()),
            140,
            "asset importer schema owner should stay small while owning row field closure",
        )

    def test_asset_importer_resource_kinds_lives_in_resource_kind_owner(self):
        self.assertTrue(
            PLUGIN_VALIDATE_ASSET_IMPORTER_RESOURCE_KINDS.exists(),
            "asset_importer output ResourceKind checks belong in a focused leaf owner",
        )
        asset_importers_text = PLUGIN_VALIDATE_ASSET_IMPORTERS.read_text(
            encoding="utf-8"
        )
        schema_text = PLUGIN_VALIDATE_ASSET_IMPORTER_SCHEMA.read_text(encoding="utf-8")
        resource_kinds_text = PLUGIN_VALIDATE_ASSET_IMPORTER_RESOURCE_KINDS.read_text(
            encoding="utf-8"
        )

        for symbol in (
            "ASSET_IMPORTER_OUTPUT_KINDS",
            "validate_plugin_asset_importer_output_kinds",
            "validate_plugin_asset_importer_known_output_kind",
            "must be a known ResourceKind",
            "output_kind",
            "additional_output_kinds",
            "UiWidget",
        ):
            self.assertIn(symbol, resource_kinds_text)
            self.assertNotIn(
                f"def {symbol}(",
                asset_importers_text,
                f"{symbol} belongs in the asset importer ResourceKind owner",
            )
        self.assertIn(
            "from .plugin_validate_asset_importer_resource_kinds import",
            schema_text,
            "schema owner should dispatch known ResourceKind checks to the leaf owner",
        )
        self.assertIn(
            "validate_plugin_asset_importer_output_kinds(",
            schema_text,
            "schema owner should invoke ResourceKind validation after string shape checks",
        )
        for forbidden_import in (
            "from .plugin_build import",
            "from .plugin_validate import",
            "from .plugin_validate_single_target import",
            "from .plugin_validate_asset_importer_schema import",
        ):
            self.assertNotIn(
                forbidden_import,
                resource_kinds_text,
                "asset importer ResourceKind owner must stay independent",
            )
        self.assertLessEqual(
            len(resource_kinds_text.splitlines()),
            120,
            "asset importer ResourceKind owner should stay a small leaf module",
        )


if __name__ == "__main__":
    unittest.main()
