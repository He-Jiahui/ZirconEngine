"""Static contract for qualified asset-import build identity and restore keys."""

from __future__ import annotations

import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
IMPORTER = ROOT / "zircon_runtime/src/asset/importer"
BUILD_IDENTITY = IMPORTER / "build_identity.rs"
CONTEXT = IMPORTER / "contract.rs"
FULL_GENERATION = (
    ROOT / "zircon_runtime/src/asset/project/manager/scan_and_import/full_generation.rs"
)
TARGETED = ROOT / "zircon_runtime/src/asset/project/manager/scan_and_import/targeted.rs"
METADATA = ROOT / "zircon_runtime/src/asset/project/manager/scan_and_import/metadata.rs"
BUILD_CONTEXT = ROOT / "zircon_runtime/build/asset_import_context.rs"
FINGERPRINT = ROOT / "zircon_runtime/build/asset_import_context/fingerprint.rs"
EMISSION = ROOT / "zircon_runtime/build/asset_import_context/emission.rs"
BUILD_TESTS = ROOT / "zircon_runtime/tests/asset_import_build_compatibility.rs"


def production(source: str) -> str:
    """Return the production portion when a Rust owner keeps tests inline."""

    marker = "\n#[cfg(test)]\nmod tests {"
    return source.rsplit(marker, 1)[0] if marker in source else source


class RuntimeQualifiedImportIdentityContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.identity_raw = BUILD_IDENTITY.read_text(encoding="utf-8")
        cls.context_raw = CONTEXT.read_text(encoding="utf-8")
        cls.metadata_raw = METADATA.read_text(encoding="utf-8")
        cls.identity = production(cls.identity_raw)
        cls.context = production(cls.context_raw)
        cls.full = FULL_GENERATION.read_text(encoding="utf-8")
        cls.targeted = TARGETED.read_text(encoding="utf-8")
        cls.metadata = METADATA.read_text(encoding="utf-8")
        cls.build_context = BUILD_CONTEXT.read_text(encoding="utf-8")
        cls.fingerprint = FINGERPRINT.read_text(encoding="utf-8")
        cls.emission = EMISSION.read_text(encoding="utf-8")
        cls.build_tests = BUILD_TESTS.read_text(encoding="utf-8")

    def test_recipe_is_versioned_typed_and_borrowed(self) -> None:
        for anchor in (
            "pub const ASSET_IMPORT_RECIPE_SCHEMA_VERSION: u32 = 1;",
            "pub struct AssetImportRecipe {",
            "schema_version: u32,",
            "settings: BTreeMap<String, AssetImportRecipeValue>,",
            "pub enum AssetImportRecipeValue {",
            "pub fn from_legacy_settings(settings: toml::Table) -> Self",
            "AssetImportRecipeValue::from_toml",
            "pub fn settings(&self) -> &BTreeMap<String, AssetImportRecipeValue>",
            "pub fn import_settings(&self) -> &toml::Table",
            "pub fn import_recipe(&self) -> &AssetImportRecipe",
        ):
            self.assertIn(anchor, self.identity + self.context, anchor)

        context_struct = self.context.split("pub struct AssetImportContext {", 1)[1].split(
            "}\n\n#[derive(Clone, Debug)]\nstruct ProjectImportResolver", 1
        )[0]
        self.assertIn("import_settings: toml::Table", context_struct)
        self.assertNotIn("pub import_settings:", context_struct)

    def test_action_key_is_versioned_and_covers_every_qualified_domain(self) -> None:
        for anchor in (
            'hasher.update(b"zircon.asset.import.action.v1")',
            "ASSET_IMPORT_ACTION_KEY_SCHEMA_VERSION",
            'b"function"',
            'b"recipe"',
            'b"inputs"',
            'b"target"',
            'b"profile"',
            'b"engine_abi"',
            'b"toolchain"',
            'format!("blake3:{}", hasher.finalize().to_hex())',
        ):
            self.assertIn(anchor, self.identity, anchor)

    def test_input_digest_covers_primary_uri_and_all_relative_snapshots(self) -> None:
        for anchor in (
            'hasher.update(b"zircon.asset.import.inputs.v1")',
            'b"source_identity"',
            'b"primary"',
            "hasher.update(&(snapshots.len() as u64).to_le_bytes());",
            "path.strip_prefix(root).ok()",
            "relative.components()",
            'b"snapshot"',
            'format!("blake3:{}", hasher.finalize().to_hex())',
            "BTreeMap<PathBuf, Vec<u8>>",
        ):
            self.assertIn(anchor, self.identity, anchor)

    def test_build_context_and_build_script_bind_exact_target_profile_abi_and_toolchain(self) -> None:
        for anchor in (
            'const PROJECT_IMPORT_BUILD_PROFILE: &str = "project_import";',
            'env!("ZR_ASSET_IMPORT_BUILD_TARGET")',
            'zircon_runtime_interface::ZIRCON_RUNTIME_API_VERSION_V8',
            'env!("ZR_ASSET_IMPORT_BUILD_ID")',
            "format!(",
            "{};build={}",
        ):
            self.assertIn(anchor, self.identity, anchor)

        for anchor in (
            'let rustc = std::env::var_os("RUSTC")',
            '.arg("-vV")',
            "compiler.status.success()",
            "fingerprint::canonical_configuration",
            "emission::directives(&compiler.stdout, &configuration)",
        ):
            self.assertIn(anchor, self.build_context, anchor)

        for anchor in (
            '"TARGET"',
            '"HOST"',
            '"PROFILE"',
            '"OPT_LEVEL"',
            '"DEBUG"',
            '"CARGO_PKG_VERSION"',
            'cargo:rustc-env=ZR_ASSET_IMPORT_BUILD_TARGET=',
            'cargo:rustc-env=ZR_ASSET_IMPORT_BUILD_ID=',
        ):
            self.assertIn(anchor, self.emission, anchor)

    def test_recipe_and_context_are_immutable_at_import_dispatch(self) -> None:
        for anchor in (
            "pub(crate) fn with_build_identity(mut self, identity: AssetImportBuildIdentity)",
            "identity.recipe().canonical_bytes() == settings_recipe.canonical_bytes()",
            "self.import_recipe = recipe;",
            "self.build_context = Some(build_context);",
            "self.build_action_key = Some(action_key);",
            "pub fn with_source_file_snapshots(",
            "self.source_file_snapshots_authoritative = true;",
            "pub(crate) fn source_file_snapshot(",
            "source_file_snapshots.keys().map(PathBuf::as_path)",
        ):
            self.assertIn(anchor, self.context, anchor)

        self.assertIn("assert!(", self.context)
        self.assertIn("asset import build identity recipe must match the context settings", self.context)

    def test_full_and_targeted_paths_qualify_before_restore_or_dispatch(self) -> None:
        for source, label in ((self.full, "full generation"), (self.targeted, "targeted import")):
            for anchor in (
                "canonical_import_input_digest(",
                "build_identity_for_import(",
                "let config_hash = build_identity.action_key().to_string();",
                ".with_build_identity(build_identity)",
                ".with_source_file_snapshots(source_file_snapshots)",
            ):
                self.assertIn(anchor, source, f"{label}: {anchor}")

        self.assertLess(
            self.full.index("canonical_import_input_digest("),
            self.full.index("build_identity_for_import("),
        )
        self.assertLess(
            self.full.index(".with_build_identity(build_identity)"),
            self.full.index("self.restore_imported_artifact("),
        )
        self.assertLess(
            self.targeted.index(".with_build_identity(build_identity)"),
            self.targeted.index("self.importer.import_context(&context)"),
        )
        for source in (self.full, self.targeted):
            self.assertIn("meta.config_hash", source)
            self.assertIn("with_config_hash(config_hash", source)

    def test_build_fingerprint_is_canonical_and_fail_closed(self) -> None:
        for anchor in (
            'name.starts_with("CARGO_CFG_")',
            'name.starts_with("CARGO_FEATURE_")',
            "values.sort_unstable();",
            "values.dedup();",
            "BTreeMap<String, String>",
            'hasher.update(b"zircon.runtime.asset.import.build.v1")',
            "hash_field(&mut hasher, compiler_verbose_version)",
            "hash_field(&mut hasher, name.as_bytes())",
            "hash_field(&mut hasher, value.as_bytes())",
        ):
            self.assertIn(anchor, self.fingerprint, anchor)
        for anchor in (
            "compiler identity was empty",
            "Cargo TARGET must be a single printable value",
            "is_empty()",
            "configuration[\"TARGET\"]",
        ):
            self.assertIn(anchor, self.emission, anchor)

    def test_regression_names_cover_recipe_context_paths_and_compiler_identity(self) -> None:
        for name in (
            "legacy_settings_migrate_to_the_current_recipe_schema_with_empty_defaults",
            "canonical_recipe_encoding_is_independent_of_table_insertion_order",
            "legacy_recipe_migration_preserves_typed_value_domains",
            "action_key_is_cryptographic_and_covers_every_qualified_input_domain",
            "canonical_input_digest_covers_primary_auxiliary_path_and_bytes",
            "project_import_context_consumes_the_generated_build_identity",
            "context_exposes_the_recipe_and_qualified_build_identity",
            "context_rejects_a_build_identity_for_different_settings",
            "project_import_identity_qualifies_the_production_recipe_and_toolchain",
            "project_import_action_key_rebuilds_once_and_matches_full_and_targeted_paths",
            "import_build_emission_binds_generated_names_to_their_qualified_values",
            "import_build_emission_rejects_missing_or_invalid_identity_inputs",
            "import_build_identity_distinguishes_compiler_target_features_and_codegen",
            "import_build_identity_ignores_environment_order_and_workspace_paths",
            "import_build_identity_preserves_flag_order_and_field_boundaries",
        ):
            self.assertIn(
                f"fn {name}(",
                self.identity_raw + self.context_raw + self.metadata_raw + self.build_tests,
                name,
            )


if __name__ == "__main__":
    unittest.main()
