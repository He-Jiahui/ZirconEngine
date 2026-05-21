---
related_code:
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime_interface/src/resource/locator.rs
  - zircon_runtime_interface/src/resource/asset_reference.rs
  - zircon_runtime_interface/src/resource/resource_id.rs
  - zircon_runtime/src/asset/project/meta.rs
  - zircon_runtime/src/asset/project/package_asset_registry.rs
  - zircon_runtime/src/asset/project/manager/package_assets.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/project/manager/source_path_for_uri.rs
  - zircon_runtime/src/asset/project/manager/source_uri_for_path.rs
  - zircon_runtime/src/asset/assets/shader/shader_asset.rs
  - zircon_runtime/src/asset/assets/shader/zshader.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime/src/asset/assets/material/texture_slot.rs
  - zircon_runtime/src/asset/assets/material/validation.rs
  - zircon_runtime/src/asset/assets/material/zmaterial.rs
  - zircon_runtime/src/asset/assets/material/dependency_set.rs
  - zircon_runtime/src/core/framework/render/material/diagnostic_source.rs
  - zircon_runtime/src/core/framework/render/material/readiness_report.rs
  - zircon_runtime/src/core/framework/render/material/validation_error.rs
  - zircon_runtime/src/asset/importer/ingest/asset_importer.rs
  - zircon_runtime/src/asset/importer/ingest/import_material.rs
  - zircon_runtime/src/asset/importer/ingest/import_shader_package.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - zircon_editor/src/ui/host/editor_asset_manager/records.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/asset_details.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/folder_projection.rs
  - zircon_editor/src/ui/workbench/project/asset_workspace_state.rs
  - zircon_editor/src/ui/workbench/project/constants.rs
  - zircon_editor/src/ui/workbench/project/editor_project_document_ensure_runtime_assets.rs
  - zircon_editor/src/ui/workbench/project/runtime_asset_helpers.rs
  - zircon_editor/src/ui/workbench/project/assets/default_pbr.zshader
  - zircon_editor/src/ui/workbench/project/assets/default_pbr.wgsl
  - zircon_editor/src/tests/workbench/project/renderable_template.rs
  - zircon_editor/src/tests/workbench/project/document_roundtrip.rs
  - zircon_editor/src/ui/layouts/views/asset_browser.rs
  - zircon_runtime/src/asset/tests/project/zmeta.rs
  - zircon_runtime/src/asset/tests/assets/material.rs
  - zircon_runtime/src/asset/tests/pipeline/manager.rs
  - zircon_runtime/src/asset/tests/assets/importer.rs
  - docs/assets-and-rendering/fixtures/zmeta-shader-material
implementation_files:
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime_interface/src/resource/locator.rs
  - zircon_runtime_interface/src/resource/asset_reference.rs
  - zircon_runtime/src/asset/project/meta.rs
  - zircon_runtime/src/asset/project/package_asset_registry.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/project/manager/source_path_for_uri.rs
  - zircon_runtime/src/asset/assets/shader/shader_asset.rs
  - zircon_runtime/src/asset/assets/shader/zshader.rs
  - zircon_runtime/src/asset/importer/ingest/import_shader_package.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime/src/asset/assets/material/texture_slot.rs
  - zircon_runtime/src/asset/assets/material/validation.rs
  - zircon_runtime/src/asset/assets/material/zmaterial.rs
  - zircon_runtime/src/asset/assets/material/dependency_set.rs
  - zircon_runtime/src/core/framework/render/material/diagnostic_source.rs
  - zircon_runtime/src/core/framework/render/material/readiness_report.rs
  - zircon_runtime/src/core/framework/render/material/validation_error.rs
  - zircon_runtime/src/asset/importer/ingest/asset_importer.rs
  - zircon_runtime/src/asset/importer/ingest/import_material.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - zircon_editor/src/ui/host/editor_asset_manager/records.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/asset_details.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/folder_projection.rs
  - zircon_editor/src/ui/workbench/project/asset_workspace_state.rs
  - zircon_editor/src/ui/workbench/project/constants.rs
  - zircon_editor/src/ui/workbench/project/editor_project_document_ensure_runtime_assets.rs
  - zircon_editor/src/ui/workbench/project/runtime_asset_helpers.rs
  - zircon_editor/src/ui/workbench/project/assets/default_pbr.zshader
  - zircon_editor/src/ui/workbench/project/assets/default_pbr.wgsl
  - zircon_editor/src/ui/layouts/views/asset_browser.rs
  - zircon_runtime/src/asset/tests/project/zmeta.rs
  - zircon_editor/src/tests/workbench/project/renderable_template.rs
  - zircon_editor/src/tests/workbench/project/document_roundtrip.rs
  - docs/assets-and-rendering/fixtures/zmeta-shader-material
plan_sources:
  - .codex/plans/资产 .zmeta 与 Shader Material 资产化计划.md
  - docs/superpowers/specs/2026-05-17-zmaterial-material-editor-design.md
  - docs/superpowers/plans/2026-05-17-zmaterial-material-editor.md
  - user: 2026-05-19 finish runtime UI graph and direct-surface damage, then close the `.zmaterial` workspace blocker
tests:
  - zircon_runtime_interface/src/tests/resource_contracts.rs
  - zircon_runtime/src/asset/tests/project/package_assets.rs
  - zircon_runtime/src/asset/tests/project/zmeta.rs
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-asset-package-m2 cargo test -p zircon_runtime_interface --locked resource --jobs 1 --message-format short --color never (2026-05-20 package roots M2: passed, 12 passed)
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-asset-package-m2 cargo test -p zircon_runtime --lib --locked asset::tests::project::package_assets --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 package roots M2: passed, 3 passed)
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-asset-package-m2 cargo test -p zircon_runtime --lib --locked asset::tests::project::zmeta --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 package roots M2: passed, 8 passed)
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-asset-package-m2 cargo test -p zircon_runtime --lib --locked plugin_package_manifest --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 package roots M2: passed, 6 passed)
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-asset-package-m2 cargo test -p zircon_runtime --locked package --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 package roots M2: passed after warm cache, 43 package-filtered runtime lib tests plus package-filtered integration binaries)
  - CARGO_TARGET_DIR=F:\cargo-targets\zircon-asset-package-m2 cargo test --manifest-path zircon_plugins/Cargo.toml --locked --jobs 1 --message-format short --color never package -- --test-threads=1 (2026-05-20 package roots M2: passed after moving off full D: target dir)
  - zircon_runtime/src/asset/tests/assets/material.rs
  - CARGO_TARGET_DIR=F:\cargo-targets\zircon-zmeta-shader-material-m3 cargo test -p zircon_runtime --lib --locked material_asset_reports_shader_contract_diagnostics_without_blocking_import --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 M3 required shader property diagnostics: passed, 1 passed)
  - CARGO_TARGET_DIR=F:\cargo-targets\zircon-zmeta-shader-material-m3 cargo test -p zircon_runtime --lib --locked shader --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 M3 broader shader validation: passed, 15 passed)
  - CARGO_TARGET_DIR=F:\cargo-targets\zircon-zmeta-shader-material-m3 cargo test -p zircon_runtime --lib --locked material --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 M3 broader material validation: passed, 71 passed)
  - CARGO_TARGET_DIR=F:\cargo-targets\zircon-zmeta-shader-material-m3 cargo test -p zircon_runtime --lib --locked asset::tests::project --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 M3 broader project validation: passed, 26 passed)
  - cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --target-dir E:\Git\ZirconEngine\zircon_plugins\target --message-format short --color never (2026-05-20 WGSL capture facade re-export: initially failed with E0425 for `crate::asset::validate_wgsl_captures`; passed after top-level re-export, existing warnings only)
  - cargo test -p zircon_runtime --lib --locked documented_zmeta_shader_material_fixture_parses --jobs 1 --target-dir F:\cargo-targets\zircon-zmeta-shader-material-m3 --message-format short --color never -- --test-threads=1 (2026-05-20 M4 fixture capture closeout: passed, 1 passed)
  - cargo test -p zircon_runtime --lib --locked asset::tests::project::zmeta --jobs 1 --target-dir F:\cargo-targets\zircon-zmeta-shader-material-m3 --message-format short --color never -- --test-threads=1 (2026-05-20 M4 fixture capture closeout: passed, 8 passed)
  - zircon_runtime/src/asset/tests/pipeline/manager.rs
  - zircon_runtime/src/asset/tests/assets/importer.rs
  - cargo check -p zircon_runtime_interface --locked
  - cargo check -p zircon_runtime --locked --lib --message-format=short
  - cargo test -p zircon_runtime_interface --locked resource --jobs 1 -- --nocapture
  - cargo test -p zircon_runtime --lib --locked asset::tests::project::zmeta --jobs 1 -- --nocapture
  - cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-zmeta-validation --lib asset::tests::project::zmeta --jobs 1 -- --nocapture
  - cargo test -p zircon_runtime --lib --locked asset::tests::watcher --jobs 1 -- --nocapture
  - cargo test -p zircon_runtime --lib --locked asset::tests::assets::material --jobs 1 -- --nocapture
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-m2-wsl cargo test -p zircon_runtime --lib --locked shader --jobs 1
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-m2-wsl cargo test -p zircon_runtime --lib --locked material_asset_reports_shader_contract_diagnostics_without_blocking_import --jobs 1
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-m2-wsl cargo test -p zircon_runtime --lib --locked project_manager_imports_zshader_with_wgsl_capture_diagnostics --jobs 1
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-final-wsl cargo test -p zircon_runtime --lib material --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-final-wsl cargo test -p zircon_runtime --lib shader --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-final-wsl cargo test -p zircon_runtime --lib asset::tests::project::zmeta --locked --offline --jobs 1 --message-format short --color never
  - cargo test -p zircon_runtime --lib --locked render_product_assets_shader_selects_runtime_wgsl_and_entry_contracts --jobs 1 -- --nocapture
  - cargo test -p zircon_runtime --lib --locked package_manifest --jobs 1 -- --nocapture
  - cargo check -p zircon_editor --locked --lib --message-format=short
  - cargo check -p zircon_editor --locked --tests --message-format=short
  - cargo test -p zircon_editor --lib --locked sync_from_project_exposes_zmeta_package_and_compound_shader_details --jobs 1 -- --nocapture
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-final-wsl cargo test -p zircon_editor --lib create_renderable_template_scaffolds_directory_project_defaults --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-presenter-check-wsl cargo test -p zircon_editor --lib create_renderable_template_scaffolds_directory_project_defaults --locked --offline --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-final-wsl cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_virtual_geometry_runtime --lib --locked --jobs 1 --message-format short --color never
  - CARGO_TARGET_DIR=/mnt/f/cargo-targets/zircon-zmaterial-final-wsl cargo test -p zircon_runtime --test virtual_geometry_visibility_debug_contract --locked --offline --jobs 1 --message-format short --color never
  - .\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -TargetDir F:\cargo-targets\zircon-zmeta-validation
  - cargo test --manifest-path zircon_plugins\Cargo.toml --locked --target-dir F:\cargo-targets\zircon-zmeta-validation
  - cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-zmeta-validation --lib runtime_backed_workspace_plugin_manifests_are_present_in_builtin_catalog -- --nocapture
  - cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-zmeta-validation --lib documented_zmeta_shader_material_fixture_parses -- --nocapture
  - cargo test -p zircon_editor --lib --locked --target-dir F:\cargo-targets\zircon-zmeta-validation -- --nocapture
  - cargo build -p zircon_hub --locked --target-dir F:\cargo-targets\zircon-zmeta-validation
  - .\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -TargetDir F:\cargo-targets\zircon-zmeta-validation
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-codex-render-damage cargo test -p zircon_runtime --lib material_asset_serialization_rewrites_stale_canonical_overrides --locked --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-codex-render-damage cargo test -p zircon_runtime --lib material_asset --locked --jobs 1 --message-format short --color never -- --test-threads=1
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-codex-render-damage cargo test -p zircon_runtime --lib asset::tests::pipeline::manager --locked --jobs 1 --message-format short --color never -- --test-threads=1
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-codex-render-damage cargo test -p zircon_runtime --lib --locked --jobs 1 --message-format short --color never -- --test-threads=1
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-codex-render-damage cargo test --workspace --locked --jobs 1 --message-format short --color never -- --test-threads=1
doc_type: module-detail
---

# ZMeta Shader Material Assets

## Purpose

The asset identity path is now owned by `zircon_runtime::asset` and `zircon_runtime::core::resource`: `.zmeta` stores UUID identity, human-readable URL, source unit, included files, subasset entries, importer state, artifact locators, and dependency locators. There is no second asset database.

## Locator Rules

- `res://path/to/asset` maps to `{project_root}/assets/path/to/asset`.
- `package://com.zircon.navigation/path/to/asset` maps through `PackageAssetRegistry` to a registered package `assets/` root.
- `PluginPackageManifest::package_id()` composes `package_prefix.package_company.package_name` when all three fields are present, otherwise it falls back to the manifest id. `asset_roots_or_default()` returns explicit `asset_roots` or `assets` when the manifest omits roots.
- `ProjectManager::register_package_asset_root(...)` registers an explicit package root. `register_package_manifest_asset_roots(...)` reads a manifest root and currently requires exactly one relative, contained root so every `package://{package_id}/...` path has one unambiguous filesystem base.
- Public references serialize as `{ uuid, url }`. Runtime lookup tries UUID first; stale URLs are retained as diagnostics/repair hints through the manager lookup surface.
- `url#label` remains the subasset address form, but every root and subasset entry persists its own UUID and derives `AssetId` from that UUID.

## Source Units

Single assets use sidecars such as `assets/textures/hero.png.zmeta`.

Compound assets use a `.zmeta` root and same-name directory, for example:

```text
assets/shaders/unlit_shader.zmeta
assets/shaders/unlit_shader/unlit.zshader
assets/shaders/unlit_shader/unlit.wgsl
```

The scanner treats the `.zmeta` root as one `AssetSourceUnit::Compound`, records the directory files in `included_files`, and prevents those included files from being imported again as standalone assets. The same scanner handles registered package roots: project sources keep `res://` URLs, while package sources and compound included files use `package://{package_id}/...` URLs and the same `.zmeta` schema, importer registry, artifact writer, dependency resolver, and UUID index.

## Shader And Material

`.zshader` is TOML. It describes WGSL source files, entry points, import redirects, material property schema, texture slots, and editor hints. The compound shader importer reads the `.zmeta` root, loads the same-name directory, emits a root `ShaderAsset`, and emits `.zshader`/`.wgsl` files as labeled data subassets.

`ShaderAsset` carries `source_files`, `imports`, `property_schema`, `texture_slots`, `editor`, and `validation_diagnostics`. Texture slot schema is shader-owned: each `ShaderTextureSlotAsset` records the slot `name`, `kind`, optional fallback class, sampler hint, grouping label, and editor metadata. During compound shader import, `validate_wgsl_captures(...)` scans the combined WGSL source for declared property and texture-slot names; missing captures are recorded as `wgsl_capture` diagnostics on the shader asset but do not stop import. The helper is re-exported through the top-level `zircon_runtime::asset` facade so fixture tests and public callers do not need to depend on the internal `assets::material` module path.

`.zmaterial` is the only built-in material source suffix. It references one shader with `{ uuid, url }`, stores scalar/vector instance state in `[overrides]`, and stores texture bindings under `[textures.<slot>]`. The built-in importer id is `zircon.builtin.zmaterial`; `.material.toml` is intentionally not registered and now reports as an unknown typed TOML suffix. `ZMaterialDocument` denies unknown top-level fields, so the old top-level PBR `.material.toml` shape is rejected instead of being silently translated. `MaterialAsset` keeps legacy PBR runtime fields as transitional in-memory data, but source parse/serialization flows through `ZMaterialDocument` and shader-driven overrides/texture slots.

When a hydrated `MaterialAsset` is serialized back to `.zmaterial`, the canonical runtime fields are authoritative over stale matching entries inside `property_values` and `texture_slots`. `base_color`, `metallic`, `roughness`, `emissive`, non-opaque `alpha_mode`, and `double_sided` rewrite or remove their corresponding `[overrides]` entries according to the current field value; canonical texture references rewrite their `[textures.<slot>]` reference while preserving fallback metadata. Unknown shader-specific overrides and fallback-only texture slots are preserved. This keeps editor/runtime mutations such as changing `MaterialAsset.base_color` from re-emitting the old `overrides.base_color` bytes and blocking asset watcher reimport/revision updates.

New editor renderable projects scaffold the same contract: `default.zmaterial` points at the compound shader root `res://shaders/pbr_shader`, `pbr_shader.zmeta` marks that root as `AssetSourceUnit::Compound`, and the included `pbr.zshader`/`pbr.wgsl` files live under `assets/shaders/pbr_shader/`. The raw WGSL remains an included shader source, not the material's referenced shader identity.

Material direct dependencies include the shader reference and every texture slot that carries a concrete `AssetReference`. Texture slots may also contain only a fallback class, such as `white`, `black`, `normal`, or `missing`; fallback-only slots do not become `.zmeta` dependencies.

Material/schema mismatches are represented as typed readiness diagnostics rather than importer failures. `MaterialAsset::shader_contract_diagnostics(...)` compares `[overrides]` with `ShaderAsset.property_schema` and `[textures.<slot>]` with `ShaderAsset.texture_slots`; it records unknown overrides, override type mismatches, missing required shader properties, and unknown texture slots with stable document paths. `MaterialAsset::readiness_report_with_shader_contract(...)` merges those diagnostics with dependency-resolution readiness and also exposes shader-side WGSL capture diagnostics as `RenderMaterialValidationError::MissingWgslCapture`.

The persistent fixture under `docs/assets-and-rendering/fixtures/zmeta-shader-material/` mirrors a project `assets/` tree and includes a compound `unlit_shader.zmeta`, `unlit.zshader`, `unlit.wgsl`, and `hero_unlit.zmaterial` with `{ uuid, url }` shader and texture references. The fixture WGSL references every property and texture-slot name declared by the `.zshader`, and `documented_zmeta_shader_material_fixture_parses` checks the same WGSL capture rule as the importer so the example stays diagnostic-clean as the schema evolves.

## Editor Surfacing

The editor asset manager keeps using the runtime project registry as the authority. Asset details now include package id, source unit, included files, and labeled subassets from the loaded `.zmeta` document. The Asset Browser metadata tab displays the adapter/package/unit summary and lists included files plus subassets beside runtime diagnostics. Package assets are projected into their own `package://{package_id}` folder roots instead of being folded under `res://`.

## Validation

Runtime/interface scoped checks and the focused zmeta, watcher, package manifest, shader-selection, and material tests pass. Editor library and editor test-target checks pass, and the direct editor sync test passes after the editor test harness finishes linking.

The runtime package validator passes with `-TargetDir F:\cargo-targets\zircon-zmeta-validation`, and the plugin workspace test command passes against the same external target directory. The `zr_vm_language` catalog consistency gap is closed by registering that package id, crate, target modes, and both capabilities in `RuntimePluginDescriptor::builtin_catalog()`, so runtime-backed package manifest projection can see it through the same path as the other built-in plugin packages.

The final acceptance matrix passes with `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -TargetDir F:\cargo-targets\zircon-zmeta-validation`: the validator performed the target-dir cleanup gate, then completed workspace `cargo build --workspace --locked` and `cargo test --workspace --locked`. The only observed follow-up diagnostics are non-blocking Cargo warnings about the `zircon_runtime.pdb` output-name collision and an unused `RuntimeSession::create` helper outside the `.zmeta` asset path.

The 2026-05-19 `.zmaterial` hard-cutover closeout passed the focused WSL runtime checks on `/mnt/f/cargo-targets/zircon-zmaterial-final-wsl`: runtime `material` tests (`68` passed), runtime `shader` tests (`13` passed), runtime `asset::tests::project::zmeta` tests (`8` passed), the Virtual Geometry runtime plugin library check, and `virtual_geometry_visibility_debug_contract` (`3` passed). The editor renderable scaffold command initially exposed unrelated retained-host GPU presenter test-scope drift; after that presenter state was corrected by the active UI changes, the same scaffold test passed on `/mnt/f/cargo-targets/zircon-presenter-check-wsl` with `1` passed and `1400` filtered out.

The later 2026-05-19 runtime UI graph closeout exposed and fixed a Windows workspace blocker in this material serialization layer: asset watcher/reimport tests mutated `MaterialAsset.base_color`, but `to_toml_string()` preserved stale `overrides.base_color`, so source bytes and resource revisions did not change. The fix is covered by `material_asset_serialization_rewrites_stale_canonical_overrides`, the focused `material_asset` filter (`8` passed), the asset manager pipeline filter (`9` passed), full `zircon_runtime --lib` (`1634` passed), and full workspace `cargo test --workspace --locked --jobs 1 --message-format short --color never -- --test-threads=1`, all using `CARGO_TARGET_DIR=D:\cargo-targets\zircon-codex-render-damage`.

The 2026-05-20 M3 required-property closeout records missing required shader schema properties as typed material diagnostics instead of importer failures. The focused regression passed on `F:\cargo-targets\zircon-zmeta-shader-material-m3` with `1` test, followed by broader scoped runtime filters: `shader` (`15` passed), `material` (`71` passed), and `asset::tests::project` (`26` passed), all with `--locked` and `--test-threads=1`.

The 2026-05-20 M4 fixture capture closeout updated the documented `unlit.wgsl` fixture so it references the declared `base_color` property and texture slot, then extended `documented_zmeta_shader_material_fixture_parses` to call the same WGSL capture validator as the importer. The focused fixture test passed with `1` test, and the broader `asset::tests::project::zmeta` filter passed with `8` tests on `F:\cargo-targets\zircon-zmeta-shader-material-m3`.
