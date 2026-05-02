---
related_code:
  - zircon_plugins/plugin_sdk_examples/plugin.toml
  - zircon_plugins/plugin_sdk_examples/editor/Cargo.toml
  - zircon_plugins/plugin_sdk_examples/editor/src/lib.rs
  - zircon_plugins/Cargo.toml
  - zircon_editor/src/core/editor_plugin.rs
  - zircon_editor/src/core/editor_plugin_sdk/examples.rs
implementation_files:
  - zircon_plugins/plugin_sdk_examples/plugin.toml
  - zircon_plugins/plugin_sdk_examples/editor/Cargo.toml
  - zircon_plugins/plugin_sdk_examples/editor/src/lib.rs
  - zircon_plugins/Cargo.toml
  - zircon_editor/src/core/editor_plugin.rs
plan_sources:
  - user: 2026-05-02 continue independent plugin gap implementation
  - .codex/plans/ZirconEngine 独立插件补齐计划.md
tests:
  - zircon_plugins/plugin_sdk_examples/editor/src/lib.rs
  - zircon_editor/src/tests/editor_plugin_sdk.rs
  - zircon_editor/src/tests/editor_plugin_catalog_consistency.rs
  - 2026-05-03: cargo fmt --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor -p zircon_plugin_sdk_examples_editor --check (passed)
  - 2026-05-03: cargo metadata --manifest-path zircon_plugins/Cargo.toml --no-deps --format-version 1 --locked --offline (passed)
  - 2026-05-03: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor -p zircon_plugin_sdk_examples_editor --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-independent-plugin-physics --message-format short --color never (timed out after 10 minutes without Rust diagnostics while compiling the shared editor host)
doc_type: module-detail
---

# Plugin SDK Examples

`plugin_sdk_examples` is the independent SDK fixture package for editor plugin
authors. It makes the existing SDK sample concepts available as a real
`zircon_plugins` package instead of only as `zircon_editor` unit-test helpers.

## Contributions

The package contains one editor crate and no runtime crate. Its manifest declares
SDK metadata, editor-host target support, SourceTemplate/LibraryEmbed packaging,
and a diagnostic model importer descriptor for `.glb` and `.gltf` sources.

The editor crate exposes `PluginSdkExamplesEditorPlugin`, a package-level plugin
registering all sample contributions, plus `ExampleWindowEditorPlugin` and
`ExampleAssetInspectorPlugin` as focused fixtures for the two SDK scenarios.

The package-level registration contributes `sdk.example.weather_window`,
`sdk.example.asset_inspector`, `sdk.example.asset.model_importer`, a model asset
editor, a model import settings component drawer, a model import settings asset
creation template, and menu operations for toggling the example window and
importing or creating sample assets.

## Boundary

The package is deliberately editor-only. It is a contract fixture for extension
registration and catalog/export visibility, not a production glTF importer or
runtime model asset pipeline. Runtime importer splitting remains a separate
milestone under `zircon_plugins/asset_importers`.

## Validation

The plugin crate test asserts the window, importer, inspector, drawer, template,
capabilities, and manifest metadata. Existing editor SDK tests still cover the
core descriptor types and lifecycle behavior used by both the in-tree examples
and this package. The current implementation slice has format and locked
metadata evidence; the scoped editor-host type-check is still blocked by the
known long editor compile path and timed out without a Rust diagnostic.
