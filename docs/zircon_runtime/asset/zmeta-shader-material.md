---
related_code:
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
  - zircon_runtime/src/asset/assets/material/zmaterial.rs
  - zircon_runtime/src/asset/assets/material/dependency_set.rs
  - zircon_runtime/src/asset/importer/ingest/asset_importer.rs
  - zircon_runtime/src/asset/importer/ingest/import_material.rs
  - zircon_runtime/src/asset/importer/ingest/import_shader_package.rs
  - zircon_runtime/src/plugin/package_manifest/plugin_package_manifest.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - zircon_editor/src/ui/host/editor_asset_manager/records.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/asset_details.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/folder_projection.rs
  - zircon_editor/src/ui/workbench/project/asset_workspace_state.rs
  - zircon_editor/src/ui/layouts/views/asset_browser.rs
  - zircon_runtime/src/asset/tests/project/zmeta.rs
  - zircon_runtime/src/asset/tests/assets/material.rs
  - zircon_runtime/src/asset/tests/assets/importer.rs
  - docs/assets-and-rendering/fixtures/zmeta-shader-material
implementation_files:
  - zircon_runtime_interface/src/resource/locator.rs
  - zircon_runtime_interface/src/resource/asset_reference.rs
  - zircon_runtime/src/asset/project/meta.rs
  - zircon_runtime/src/asset/project/package_asset_registry.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/project/manager/source_path_for_uri.rs
  - zircon_runtime/src/asset/assets/shader/zshader.rs
  - zircon_runtime/src/asset/importer/ingest/import_shader_package.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime/src/asset/assets/material/texture_slot.rs
  - zircon_runtime/src/asset/assets/material/zmaterial.rs
  - zircon_runtime/src/asset/assets/material/dependency_set.rs
  - zircon_runtime/src/asset/importer/ingest/asset_importer.rs
  - zircon_runtime/src/asset/importer/ingest/import_material.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - zircon_editor/src/ui/host/editor_asset_manager/records.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/default_editor_asset_manager/asset_details.rs
  - zircon_editor/src/ui/host/editor_asset_manager/manager/folder_projection.rs
  - zircon_editor/src/ui/workbench/project/asset_workspace_state.rs
  - zircon_editor/src/ui/layouts/views/asset_browser.rs
  - zircon_runtime/src/asset/tests/project/zmeta.rs
  - docs/assets-and-rendering/fixtures/zmeta-shader-material
plan_sources:
  - .codex/plans/资产 .zmeta 与 Shader Material 资产化计划.md
  - docs/superpowers/specs/2026-05-17-zmaterial-material-editor-design.md
  - docs/superpowers/plans/2026-05-17-zmaterial-material-editor.md
tests:
  - zircon_runtime_interface/src/tests/resource_contracts.rs
  - zircon_runtime/src/asset/tests/project/zmeta.rs
  - zircon_runtime/src/asset/tests/assets/material.rs
  - zircon_runtime/src/asset/tests/assets/importer.rs
  - cargo check -p zircon_runtime_interface --locked
  - cargo check -p zircon_runtime --locked --lib --message-format=short
  - cargo test -p zircon_runtime_interface --locked resource --jobs 1 -- --nocapture
  - cargo test -p zircon_runtime --lib --locked asset::tests::project::zmeta --jobs 1 -- --nocapture
  - cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-zmeta-validation --lib asset::tests::project::zmeta --jobs 1 -- --nocapture
  - cargo test -p zircon_runtime --lib --locked asset::tests::watcher --jobs 1 -- --nocapture
  - cargo test -p zircon_runtime --lib --locked asset::tests::assets::material --jobs 1 -- --nocapture
  - cargo test -p zircon_runtime --lib --locked render_product_assets_shader_selects_runtime_wgsl_and_entry_contracts --jobs 1 -- --nocapture
  - cargo test -p zircon_runtime --lib --locked package_manifest --jobs 1 -- --nocapture
  - cargo check -p zircon_editor --locked --lib --message-format=short
  - cargo check -p zircon_editor --locked --tests --message-format=short
  - cargo test -p zircon_editor --lib --locked sync_from_project_exposes_zmeta_package_and_compound_shader_details --jobs 1 -- --nocapture
  - .\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime -TargetDir F:\cargo-targets\zircon-zmeta-validation
  - cargo test --manifest-path zircon_plugins\Cargo.toml --locked --target-dir F:\cargo-targets\zircon-zmeta-validation
  - cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-zmeta-validation --lib runtime_backed_workspace_plugin_manifests_are_present_in_builtin_catalog -- --nocapture
  - cargo test -p zircon_runtime --locked --target-dir F:\cargo-targets\zircon-zmeta-validation --lib documented_zmeta_shader_material_fixture_parses -- --nocapture
  - cargo test -p zircon_editor --lib --locked --target-dir F:\cargo-targets\zircon-zmeta-validation -- --nocapture
  - cargo build -p zircon_hub --locked --target-dir F:\cargo-targets\zircon-zmeta-validation
  - .\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -TargetDir F:\cargo-targets\zircon-zmeta-validation
doc_type: module-detail
---

# ZMeta Shader Material Assets

## Purpose

The asset identity path is now owned by `zircon_runtime::asset` and `zircon_runtime::core::resource`: `.zmeta` stores UUID identity, human-readable URL, source unit, included files, subasset entries, importer state, artifact locators, and dependency locators. There is no second asset database.

## Locator Rules

- `res://path/to/asset` maps to `{project_root}/assets/path/to/asset`.
- `package://com.zircon.navigation/path/to/asset` maps through `PackageAssetRegistry` to a registered package `assets/` root.
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

The scanner treats the `.zmeta` root as one `AssetSourceUnit::Compound`, records the directory files in `included_files`, and prevents those included files from being imported again as standalone assets.

## Shader And Material

`.zshader` is TOML. It describes WGSL source files, entry points, import redirects, material property schema, texture slots, and editor hints. The compound shader importer reads the `.zmeta` root, loads the same-name directory, emits a root `ShaderAsset`, and emits `.zshader`/`.wgsl` files as labeled data subassets.

`ShaderAsset` carries `source_files`, `imports`, `property_schema`, `texture_slots`, `editor`, and `validation_diagnostics`. Texture slot schema is shader-owned: each `ShaderTextureSlotAsset` records the slot `name`, `kind`, optional fallback class, sampler hint, grouping label, and editor metadata.

`.zmaterial` is the only built-in material source suffix. It references one shader with `{ uuid, url }`, stores scalar/vector instance state in `[overrides]`, and stores texture bindings under `[textures.<slot>]`. The built-in importer id is `zircon.builtin.zmaterial`; `.material.toml` is intentionally not registered and now reports as an unknown typed TOML suffix. `MaterialAsset` keeps legacy PBR runtime fields as transitional in-memory data, but source parse/serialization flows through `ZMaterialDocument` and shader-driven overrides/texture slots.

Material direct dependencies include the shader reference and every texture slot that carries a concrete `AssetReference`. Texture slots may also contain only a fallback class, such as `white`, `black`, `normal`, or `missing`; fallback-only slots do not become `.zmeta` dependencies.

The persistent fixture under `docs/assets-and-rendering/fixtures/zmeta-shader-material/` mirrors a project `assets/` tree and includes a compound `unlit_shader.zmeta`, `unlit.zshader`, `unlit.wgsl`, and `hero_unlit.zmaterial` with `{ uuid, url }` shader and texture references. `documented_zmeta_shader_material_fixture_parses` keeps that example parseable as the schema evolves.

## Editor Surfacing

The editor asset manager keeps using the runtime project registry as the authority. Asset details now include package id, source unit, included files, and labeled subassets from the loaded `.zmeta` document. The Asset Browser metadata tab displays the adapter/package/unit summary and lists included files plus subassets beside runtime diagnostics. Package assets are projected into their own `package://{package_id}` folder roots instead of being folded under `res://`.

## Validation

Runtime/interface scoped checks and the focused zmeta, watcher, package manifest, shader-selection, and material tests pass. Editor library and editor test-target checks pass, and the direct editor sync test passes after the editor test harness finishes linking.

The runtime package validator passes with `-TargetDir F:\cargo-targets\zircon-zmeta-validation`, and the plugin workspace test command passes against the same external target directory. The `zr_vm_language` catalog consistency gap is closed by registering that package id, crate, target modes, and both capabilities in `RuntimePluginDescriptor::builtin_catalog()`, so runtime-backed package manifest projection can see it through the same path as the other built-in plugin packages.

The final acceptance matrix passes with `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -TargetDir F:\cargo-targets\zircon-zmeta-validation`: the validator performed the target-dir cleanup gate, then completed workspace `cargo build --workspace --locked` and `cargo test --workspace --locked`. The only observed follow-up diagnostics are non-blocking Cargo warnings about the `zircon_runtime.pdb` output-name collision and an unused `RuntimeSession::create` helper outside the `.zmeta` asset path.
