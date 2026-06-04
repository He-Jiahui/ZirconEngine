---
related_code:
  - zircon_plugins/net/plugin.toml
  - zircon_plugins/net/editor/Cargo.toml
  - zircon_plugins/net/editor/src/lib.rs
  - zircon_plugins/net/editor/src/tests/mod.rs
  - zircon_plugins/net/editor/src/tests/authoring_extensions.rs
  - zircon_plugins/net/runtime/src/lib.rs
  - zircon_plugins/editor_support/src/lib.rs
implementation_files:
  - zircon_plugins/net/editor/Cargo.toml
  - zircon_plugins/net/editor/src/lib.rs
  - zircon_plugins/net/editor/src/tests/mod.rs
  - zircon_plugins/net/editor/src/tests/authoring_extensions.rs
plan_sources:
  - user: 2026-06-04 plugin ecosystem infrastructure expansion
tests:
  - zircon_plugins/net/editor/src/tests/mod.rs
  - net_editor_plugin_contributes_authoring_extensions
  - rustfmt --edition 2021 --check over zircon_plugins/net/editor/src/lib.rs and every child file under zircon_plugins/net/editor/src/tests/ (passed 2026-06-04)
doc_type: module-detail
---

# Net Editor Plugin

## Purpose

`zircon_plugins/net/editor` owns the editor-facing authoring surface for the first-party Net plugin. It does not implement sockets, replication, RPC, or download behavior; those remain in `zircon_plugins/net/runtime` and optional Net feature crates. The editor plugin contributes authoring affordances that let the editor host expose Net tools through the shared editor extension registry.

## Boundary

- `Cargo.toml` depends on `zircon_editor`, shared editor plugin support, the Net runtime plugin package, and the neutral runtime package manifest type.
- `src/lib.rs` is the structural editor plugin entry. It defines the Net authoring view, drawer, template identifiers, the `NetEditorPlugin` descriptor wrapper, registration entry points, package manifest projection, editor capability reporting, and a host-contract marker.
- `src/tests/mod.rs` is the structural test entry. Test assertions live in child files instead of an inline root test block.
- `src/tests/authoring_extensions.rs` validates that editor registration contributes the Net authoring capability, view, drawer, template, menu item, and operation descriptor.

The editor plugin intentionally consumes `zircon_plugin_net_runtime::package_manifest()` when projecting a package manifest. This keeps editor metadata aligned with the runtime plugin package rather than inventing a second Net package identity.

## Design Notes

The editor/runtime split mirrors mature engine plugin shapes: runtime transports and gameplay networking stay in runtime packages, while editor packages contribute authoring panels, menus, templates, and diagnostics. The Net editor module is therefore a narrow authoring adapter over the shared `zircon_plugin_editor_support::register_authoring_extensions` helper.

The crate root stays short and structural. Future Net authoring work should add child modules for specific editor responsibilities such as connection diagnostics, replication graph inspection, RPC schema browsers, or content-download manifests. It should not add those workflows directly to `src/lib.rs`.

## Test Coverage

The current focused test covers editor extension registration shape: capability row, authoring view, drawer, UI template, menu item, and operation descriptor. Focused Cargo validation remains pending while other workspace Cargo lanes are active; low-interference validation for this slice is rustfmt, diff-check, trailing-whitespace, and conflict-marker scanning.
