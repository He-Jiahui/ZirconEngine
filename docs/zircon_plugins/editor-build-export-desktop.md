---
related_code:
  - zircon_plugins/editor_build_export_desktop/plugin.toml
  - zircon_plugins/editor_build_export_desktop/editor/Cargo.toml
  - zircon_plugins/editor_build_export_desktop/editor/src/lib.rs
  - zircon_plugins/Cargo.toml
  - zircon_editor/src/core/editor_plugin.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/manager.rs
implementation_files:
  - zircon_plugins/editor_build_export_desktop/plugin.toml
  - zircon_plugins/editor_build_export_desktop/editor/Cargo.toml
  - zircon_plugins/editor_build_export_desktop/editor/src/lib.rs
  - zircon_plugins/Cargo.toml
  - zircon_editor/src/core/editor_plugin.rs
plan_sources:
  - user: 2026-05-02 continue independent plugin gap implementation
  - .codex/plans/ZirconEngine 独立插件补齐计划.md
tests:
  - zircon_plugins/editor_build_export_desktop/editor/src/lib.rs
  - zircon_editor/src/tests/editor_plugin_catalog_consistency.rs
  - 2026-05-03: cargo fmt --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor -p zircon_plugin_sdk_examples_editor --check (passed)
  - 2026-05-03: cargo metadata --manifest-path zircon_plugins/Cargo.toml --no-deps --format-version 1 --locked --offline (passed)
  - 2026-05-03: cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_editor_build_export_desktop_editor -p zircon_plugin_sdk_examples_editor --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-independent-plugin-physics --message-format short --color never (timed out after 10 minutes without Rust diagnostics while compiling the shared editor host)
doc_type: module-detail
---

# Desktop Build Export Plugin

`editor_build_export_desktop` is an editor-only plugin package for the desktop
export authoring surface. It does not take ownership of export plan generation:
`EditorManager::{generate_export_plan, generate_native_aware_export_plan,
execute_export_build, execute_native_aware_export_build}` remain the host-owned
authority. The plugin contributes the editor surface that calls into that host
path.

## Contributions

The package manifest declares SDK/API version `0.1.0`, category `platform`,
`editor_host` target support, Windows/Linux/macOS platform support, capabilities
for the desktop export panel, diagnostics, and NativeDynamic report view, plus
SourceTemplate and LibraryEmbed as default packaging strategies.

The editor crate registers one `editor.build_export_desktop` view and
`Desktop Export Tools` drawer, a main panel UI template, SourceTemplate,
LibraryEmbed, and NativeDynamic report templates, menu-backed operations for
plan generation and each desktop packaging mode, an asset creation template and
asset editor for desktop export profiles, and a component drawer whose bindings
point at the export operations.

## Boundary

The plugin intentionally contributes descriptors and menu operations only. The
actual build plan, materialization, native package preparation, cargo invocation,
and diagnostics file write stay in `zircon_editor`/`zircon_runtime` host code so
runtime export remains deterministic and independent of editor plugin state.

## Validation

The plugin crate test checks that registration produces the panel view,
operations, report templates, menu entries, asset profile template, and component
drawer. The editor catalog consistency test covers the workspace member,
`plugin.toml`, and builtin catalog entry alignment. The current implementation
slice has format and locked metadata evidence; the scoped editor-host type-check
is still blocked by the known long editor compile path and timed out without a
Rust diagnostic.
