---
related_code:
  - zircon_plugins/terrain/plugin.toml
  - zircon_plugins/terrain/README.md
  - zircon_plugins/terrain/runtime/src/lib.rs
  - zircon_plugins/terrain/editor/src/lib.rs
  - zircon_plugins/tilemap_2d/plugin.toml
  - zircon_plugins/tilemap_2d/README.md
  - zircon_plugins/tilemap_2d/runtime/src/lib.rs
  - zircon_plugins/tilemap_2d/editor/src/lib.rs
  - zircon_plugins/prefab_tools/plugin.toml
  - zircon_plugins/prefab_tools/README.md
  - zircon_plugins/prefab_tools/runtime/src/lib.rs
  - zircon_plugins/prefab_tools/editor/src/lib.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
implementation_files:
  - zircon_plugins/terrain/plugin.toml
  - zircon_plugins/terrain/runtime/src/lib.rs
  - zircon_plugins/terrain/editor/src/lib.rs
  - zircon_plugins/tilemap_2d/plugin.toml
  - zircon_plugins/tilemap_2d/runtime/src/lib.rs
  - zircon_plugins/tilemap_2d/editor/src/lib.rs
  - zircon_plugins/prefab_tools/plugin.toml
  - zircon_plugins/prefab_tools/runtime/src/lib.rs
  - zircon_plugins/prefab_tools/editor/src/lib.rs
  - zircon_runtime/src/plugin/runtime_plugin/builtin_catalog.rs
plan_sources:
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
tests:
  - zircon_plugins/terrain/runtime/src/lib.rs
  - zircon_plugins/tilemap_2d/runtime/src/lib.rs
  - zircon_plugins/prefab_tools/runtime/src/lib.rs
  - zircon_runtime/src/tests/plugin_extensions/manifest_contributions.rs
validation:
  - 2026-05-31: cargo test --manifest-path .\zircon_plugins\terrain\runtime\Cargo.toml terrain_runtime_plugin_contributes_component_and_importers --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-authoring-runtime-metadata --color never --quiet (red before linked maturity metadata, then passed with existing runtime warnings)
  - 2026-05-31: cargo test --manifest-path .\zircon_plugins\tilemap_2d\runtime\Cargo.toml tilemap_runtime_plugin_contributes_component_and_importers --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-authoring-runtime-metadata --color never --quiet (passed with existing runtime warnings)
  - 2026-05-31: cargo test --manifest-path .\zircon_plugins\prefab_tools\runtime\Cargo.toml prefab_runtime_plugin_contributes_component_and_importers --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-authoring-runtime-metadata --color never --quiet (passed with existing runtime warnings)
  - 2026-05-31: cargo test --manifest-path .\Cargo.toml -p zircon_runtime --lib authoring_plugin_manifests_match_catalog_and_workspace_shape --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-authoring-runtime-metadata --color never --quiet (passed with existing runtime warnings after the first cold compile timed out and completed in the background)
doc_type: module-detail
---

# Authoring Runtime Plugins

## Purpose

Terrain, Tilemap 2D, and Prefab Tools are runtime-backed authoring plugins. Each
package has a runtime crate for scene-facing descriptors and diagnostic import
metadata, plus an editor crate for authoring surfaces, menus, operations, asset
templates, component drawers, and editor-only payload schemas.

The authoring packages are intentionally not default runtime feature promotion.
They are selectable plugin packages that must remain visible to runtime export,
Plugin Manager, and editor-host catalog projections without implying that terrain
editing, tilemap editing, or prefab workflows are complete gameplay/runtime
systems.

## Package Contracts

- `terrain` owns `runtime.plugin.terrain` and
  `editor.extension.terrain_authoring`. The runtime side contributes the terrain
  component descriptor plus diagnostic heightfield importer metadata for `raw`,
  `r16`, and `png` sources.
- `tilemap_2d` owns `runtime.plugin.tilemap_2d` and
  `editor.extension.tilemap_2d_authoring`. The runtime side contributes the
  tilemap component descriptor plus diagnostic Tiled importer metadata for
  `tmx`, `tsx`, and `json` sources.
- `prefab_tools` owns `runtime.plugin.prefab_tools` and
  `editor.extension.prefab_tools_authoring`. The runtime side contributes the
  prefab instance component descriptor plus diagnostic prefab importer metadata
  for `.prefab.toml` sources.
- Static `plugin.toml`, linked runtime package manifests, and
  `RuntimePluginDescriptor::builtin_catalog()` classify all three packages as
  category `authoring`, maturity `beta`, and primary runtime capability status
  `partial`.

## Behavior Model

The runtime crates use `RuntimePluginDescriptor` as the source for linked package
metadata. `package_manifest()` extends that linked descriptor with the package's
component and importer declarations. This means runtime export and editor-host
package projection see the same category, maturity, capability, and status rows
that static plugin TOML and the built-in catalog expose.

The editor crates layer authoring descriptors on top of the runtime manifest:
they register plugin-specific views, drawers, asset editors, operations, menu
items, templates, importers, and payload schema ids. Editor-only state stays in
the editor plugin boundary; runtime DTOs remain limited to component/importer
contracts that runtime profiles can serialize and reason about.

## Validation Notes

2026-05-31 metadata validation first used the Terrain runtime registration test
as the red linked guard. The test reached the intended failure after two narrow
shared support fixes from the active asset/render lane: re-exporting
`MeshRendererPrimitiveBinding`, restoring `RenderMeshSnapshot` derives, and
cloning the mesh snapshot value while building geometry phase inputs. The
intended red failure was linked Terrain maturity `Experimental` instead of
`Beta`.

After descriptor parity was added to Terrain, Tilemap 2D, and Prefab Tools, the
three linked runtime tests passed. The runtime manifest/catalog regression also
passed and now asserts static TOML and built-in catalog category, maturity, and
partial status for all three packages. These are scoped metadata/package tests;
they do not claim terrain, tilemap, or prefab authoring product completion.

## Open Issues

- Runtime importers remain diagnostic-only stubs. Concrete terrain heightfield,
  Tiled, and prefab import backends still need feature-specific implementation
  and acceptance tests before promotion.
- Editor operation descriptors are registered, but operation execution and full
  authoring workflows still need editor-host validation.
- These packages are authoring extensions; they do not close default
  `DefaultPlugins` runtime parity or broad scene/asset/editor promotion gates.
