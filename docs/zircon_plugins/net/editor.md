---
related_code:
  - zircon_plugins/net/plugin.toml
  - zircon_plugins/net/editor/Cargo.toml
  - zircon_plugins/net/editor/src/authoring.rs
  - zircon_plugins/net/editor/src/lib.rs
  - zircon_plugins/net/editor/src/tests/mod.rs
  - zircon_plugins/net/editor/src/tests/authoring_extensions.rs
  - zircon_plugins/net/runtime/src/lib.rs
  - zircon_plugins/editor_support/src/lib.rs
implementation_files:
  - zircon_plugins/net/editor/Cargo.toml
  - zircon_plugins/net/editor/src/authoring.rs
  - zircon_plugins/net/editor/src/lib.rs
  - zircon_plugins/net/editor/src/tests/mod.rs
  - zircon_plugins/net/editor/src/tests/authoring_extensions.rs
plan_sources:
  - user: 2026-06-04 plugin ecosystem infrastructure expansion
tests:
  - zircon_plugins/net/editor/src/tests/mod.rs
  - net_editor_plugin_contributes_authoring_extensions
  - rustfmt --edition 2021 --check over zircon_plugins/net/editor/src/lib.rs and every child file under zircon_plugins/net/editor/src/tests/ (passed 2026-06-04)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_editor --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never (passed once during M7 2026-06-14 with locks restored; final rerun blocked by unrelated untracked UI tree_view compile drift)
doc_type: module-detail
---

# Net Editor Plugin

## Purpose

`zircon_plugins/net/editor` owns the editor-facing authoring surface for the first-party Net plugin. It does not implement sockets, replication, RPC, or download behavior; those remain in `zircon_plugins/net/runtime` and optional Net feature crates. The editor plugin contributes authoring affordances that let the editor host expose Net tools through the shared editor extension registry.

## Boundary

- `Cargo.toml` depends on `zircon_editor`, shared editor plugin support, the Net runtime plugin package, and the neutral runtime package manifest type.
- `src/lib.rs` is the structural editor plugin entry. It defines the `NetEditorPlugin` descriptor wrapper, registration entry points, package manifest projection, editor capability reporting, and a host-contract marker.
- `src/authoring.rs` owns the Net editor contribution contract. It registers the authoring and diagnostics views, listener/route configuration operations, replication-schema asset creation, component drawers, graph editor, and graph node palette.
- `src/tests/mod.rs` is the structural test entry. Test assertions live in child files instead of an inline root test block.
- `src/tests/authoring_extensions.rs` validates that editor registration contributes the Net authoring capability, authoring/diagnostics views, drawer, template, menu items, operation descriptors, payload schemas, component drawers, replication-schema template, graph editor, and palette.

The editor plugin intentionally consumes `zircon_plugin_net_runtime::package_manifest()` when projecting a package manifest. This keeps editor metadata aligned with the runtime plugin package rather than inventing a second Net package identity.

## Design Notes

The editor/runtime split mirrors mature engine plugin shapes: runtime transports and gameplay networking stay in runtime packages, while editor packages contribute authoring panels, menus, templates, and diagnostics. The Net editor module is therefore a narrow authoring adapter over the shared `zircon_plugin_editor_support::register_authoring_extensions` and `register_authoring_contribution_batch` helpers.

M7 keeps the crate root short by placing concrete contribution assembly in `authoring.rs`. `NET_AUTHORING_SURFACES` registers `net.authoring` for network authoring and `net.diagnostics` for the connection diagnostics live-view entry, aligned with `docs/ui-and-layout/ai-workbench-style/ai-console-diagnostics-layout.png`. The module also registers `net.listener.configure`, `net.route.configure`, and `net.replication_schema.{open,validate,compile,create}` so later UI/template code can bind concrete `.zui` documents to stable operation paths rather than inventing new ids.

The current slice implements the editor extension data plane. It intentionally does not add rendered `.zui` template files; those should be attached by the editor/UI lane to the existing `plugins://net/editor/listener_config.zui`, `route_config.zui`, `replication_schema.zui`, and diagnostics view ids.

## Test Coverage

The current focused test covers editor extension registration shape: capability row, authoring and diagnostics views, drawer, UI template, menu items, operation descriptors, payload schemas, component drawers, replication-schema asset template, graph editor, and graph node palette.

M7 validation on 2026-06-14 passed `rustfmt --edition 2021 --check` and path-scoped `git diff --check` over the touched Net editor files. A lockfile-protected `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_net_editor --tests --offline --jobs 1 --target-dir D:\cargo-targets\zircon-net-ws-m2-0614 --message-format short --color never` passed once after the M7 implementation with existing warning noise and restored root/plugin lockfiles. A later final rerun was blocked before reaching Net editor tests by an unrelated untracked UI file at `zircon_runtime/src/ui/surface/surface/default_interactions/tree_view.rs` with duplicate definitions and missing helper symbols, so no final rerun pass is claimed until that external UI drift is resolved.
