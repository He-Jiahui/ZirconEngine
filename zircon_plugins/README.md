# Zircon Plugins Workspace

`zircon_plugins` is intentionally separate from the root workspace. Runtime/editor export and plugin CI can compile this workspace when plugin packages are needed, while the root workspace keeps the minimal runtime/editor core fast by default.

Runtime-backed plugin packages use this shape:

- `zircon_plugins/<plugin_id>/plugin.toml`
- `zircon_plugins/<plugin_id>/runtime/Cargo.toml`
- `zircon_plugins/<plugin_id>/editor/Cargo.toml`

Editor-only plugin packages omit the `runtime` crate:

- `zircon_plugins/<plugin_id>/plugin.toml`
- `zircon_plugins/<plugin_id>/editor/Cargo.toml`

Runtime crates depend only on runtime contracts. Editor crates may depend on
`zircon_editor` plus their matching runtime crate when one exists.

The runtime-backed package set is `physics`, `sound`, `texture`, `net`,
`navigation`, `particles`, `animation`, `virtual_geometry`, and `hybrid_gi`.
The editor-only package set is `runtime_diagnostics`, `ui_asset_authoring`, and
`native_window_hosting`.

These packages are deliberately kept outside the root workspace so normal
runtime/editor checks only build the minimal core unless an export profile or
plugin CI job selects this workspace.
