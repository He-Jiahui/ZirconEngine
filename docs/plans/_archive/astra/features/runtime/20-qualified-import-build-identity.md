---
status: in_progress
plan_sources:
  - docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/207-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-cook-package-incremental-build-worker-determinism-current-working-tree-review.md
---

# Qualified import build identity

## Current source and repair

The production full-generation and targeted-import paths previously persisted
`config_hash` from a `DefaultHasher` digest of serialized settings. That digest
did not cover the importer function, all captured source inputs, target,
profile, runtime ABI, or toolchain. `AssetImportContext` also exposed only the
untyped settings table, so an importer could not inspect the compatibility
domain used by restore.

The repair adds a versioned `AssetImportRecipe`, a read-only
`import_settings()` view of the private TOML table, a qualified
`AssetImportBuildContext`, and a BLAKE3 `AssetImportBuildIdentity`. Both
production import paths now build the identity before restore or importer
dispatch, persist its action key through the existing `config_hash` field, and
attach the same recipe, build context, and action key to the importer context.
The key therefore participates in the existing restore predicate and is not an
unused cache DTO. Importers can read the legacy TOML view but cannot mutate it
after the recipe and action key have been captured.
The TOML field is private; `import_settings()` exposes only a borrowed table.
`with_build_identity` also rejects a recipe that differs from these settings.

The structure follows Unreal Engine's derived-data build definition boundary:
the function, constants/recipe, immutable input hashes, and compatibility
context are captured before executing the build. This slice does not introduce
the later remote scheduler or build-input resolver.

## Contract

| Domain | Canonical action-key input |
|---|---|
| Schema | `zircon.asset.import.action.v1` and recipe schema version 1 |
| Function | stable importer id |
| Recipe | recursively typed, length-delimited canonical TOML values with sorted table keys |
| Inputs | BLAKE3 over source URI, primary bytes, and every captured auxiliary relative path and byte payload |
| Target | Cargo's exact `TARGET` triple for the runtime executing project import |
| Profile | explicit `project_import` profile |
| ABI | current `ZIRCON_RUNTIME_API_VERSION_V8` |
| Toolchain | runtime/importer versions plus BLAKE3 of the actual compiler verbose identity, host/target, package features, target cfg, effective optimization/debug settings and ordered compiler flags |

`AssetImportRecipe::default` is schema 1 with an empty settings table. Existing
v7 `.zmeta` `import_settings` migrate in memory through
`from_legacy_settings` after project defaults are resolved. The `.zmeta`
format remains v7: no field is added or removed, and no commandlet migration is
required. Existing settings hashes fail the new action-key comparison once and
are replaced by the normal import transaction. The legacy `config_hash` field
name remains only as the persisted compatibility slot.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| W2-ASSET85-P1-003 | Versioned typed value tree, legacy-TOML migration, canonical encoding, immutable context view | partial_pending_validation | Importer-specific schema/default validation and versioned importer migration callbacks remain open |
| W2-ASSET85-P1-004 | BLAKE3 action key in full/targeted restore and import paths, with compiler/configuration identity | partial_pending_validation | Declared snapshots and compile-time compatibility are covered; runtime environment, provider identity, and importer implementation fingerprints remain open |
| W2-ASSET85-P1-005 | Exact build target, `project_import` profile, runtime API and runtime/importer/compiler identity in context | partial_pending_validation | Export target/profile selection and external tool identities remain open |

`build/asset_import_context.rs` captures Cargo's selected compiler using `RUSTC -vV`
when the managed build runs. The build script hashes only named compatibility
variables and `CARGO_CFG_*`/`CARGO_FEATURE_*`; it does not serialize arbitrary
environment variables. Cfg value sets are sorted, while compiler argument order
is preserved. Neither checkout root nor `OUT_DIR` enters the identity. Explicit
path-bearing compiler flags still qualify the build conservatively. Required
Cargo identity values or compiler-query failure stop the build rather than emit
an empty identity. This follows [Cargo's build-script environment contract](https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts).
The captured `TARGET` describes the importer runtime binary; a future export
variant must supply its own product target/profile instead of reusing it.

`tests/asset_import_build_compatibility.rs` reuses the production fingerprint
encoder and checks compiler/target/features/codegen changes, enumeration and cfg
order stability, workspace relocation, and argument ordering. The production
emission plan now rejects missing or empty required Cargo inputs and verifies
the two generated environment directive names and their qualified values.
`project_import_context_consumes_the_generated_build_identity` covers the
generated values entering the actual importer context. This build-script
extension has passed formatting only; neither the generated identity nor the
Rust regressions have executed yet.

Focused regression filters:

- `legacy_settings_migrate_to_the_current_recipe_schema_with_empty_defaults`
- `canonical_recipe_encoding_is_independent_of_table_insertion_order`
- `legacy_recipe_migration_preserves_typed_value_domains`
- `canonical_input_digest_covers_primary_auxiliary_path_and_bytes`
- `action_key_is_cryptographic_and_covers_every_qualified_input_domain`
- `context_exposes_the_recipe_and_qualified_build_identity`
- `context_rejects_a_build_identity_for_different_settings`
- `project_import_context_consumes_the_generated_build_identity`
- `project_import_identity_qualifies_the_production_recipe_and_toolchain`
- `project_import_action_key_rebuilds_once_and_matches_full_and_targeted_paths`
- integration target `asset_import_build_compatibility`

Pre-compiler-extension source snapshot (SHA-256; the next managed input must also
include `build.rs`, `build/asset_import_context.rs`, its fingerprint/emission
children, the manifest and integration test, and
refresh `build_identity.rs`):

```text
zircon_runtime/src/asset/importer/build_identity.rs 6457E9ECC282C99A9805590D672CF12068B900CF0485A27B16E212F6BFC0AAB9
zircon_runtime/src/asset/importer/mod.rs 9CC0EA4250F7F6A331FBA957A72D29D998B4B0A515B7C8B10D5476E7C11AB078
zircon_runtime/src/asset/importer/contract.rs 60343D94C76468C380ED4A1BA8814DEA027DB19255744F2152958470F0BEC3A2
zircon_runtime/src/asset/importer/native.rs 9C846FE0D905CC74A238AA8E5BF03232B7DE236DD9E4EB880F59B60D801A8F8E
zircon_runtime/src/asset/project/manager/scan_and_import/metadata.rs 961819B2118867D8C8A18A3642990BC883C5B840F7C33FFF56B16CC562CC57D5
zircon_runtime/src/asset/project/manager/scan_and_import/full_generation.rs D2FF906E8444087218782365D2AE8DF5413D82CDEDBC60CBD0DD0DFDDE8F6914
zircon_runtime/src/asset/project/manager/scan_and_import/targeted.rs B3061B44000ED7A91CC9A0AB07C465E882C97A7E500537DBA72D94B3F9B21451
```

The already-reviewed source snapshot dependency remained byte-identical at
`8740C89A58E4C69814554299997D9957EECCFFABE1FB1FD703FD8546643B77AC`.

The next dependency is ASSET85-P1-001: typed raw-source, logical-asset, tool,
and environment dependency edges. This slice hashes every source snapshot the
current importer declares, but it does not claim that undeclared physical reads
have already been eliminated.
