---
related_code:
  - tools/cargo-zircon/src/main.rs
  - tools/cargo-zircon/src/plugin/scaffold/mod.rs
  - tools/cargo-zircon/src/plugin/manifest_sync.rs
  - tools/cargo-zircon/src/plugin/check.rs
  - tools/cargo-zircon/src/plugin/validate.rs
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
  - zircon_app/Cargo.toml
implementation_files:
  - tools/cargo-zircon/src/plugin
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/04-plugin-dx-and-sdk-toolchain.md
tests:
  - tools/cargo-zircon/tests/manifest_sync.rs
  - tools/cargo-zircon/tests/plugin_commands.rs
  - .github/workflows/ci.yml
doc_type: workflow-detail
---

# Cargo Zircon Plugin Workflow

## Contract

`cargo-zircon` is the developer-facing owner for plugin scaffolding, Rust-to-TOML
manifest synchronization, repository wiring checks, and standalone static
validation. Rust `declare_plugin!` metadata is authoritative; checked-in
`plugin.toml` files are generated publication snapshots and never generate Rust
identity, capability, or ABI constants.

Install the subcommand once from the repository root:

```powershell
cargo install --path tools/cargo-zircon --locked
```

The five-minute system-plugin path is three steps:

```powershell
cargo zircon plugin new demo_probe --kind system --native
# Implement registration behavior in zircon_plugins/demo_probe/runtime/src/plugin.rs.
cargo build --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_demo_probe_dist --locked
```

`--kind` accepts `importer`, `system`, or `editor`. `--native` adds the v3 dist
crate and declaration-generated ABI projection. The command creates a complete
package skeleton, adds the runtime or editor catalog feature and registration,
adds the corresponding `zircon_app` feature, and updates plugin workspace
membership. It does not overwrite an existing package. A source-linked build can
enable `first-party-<id>-runtime-plugin` or `first-party-<id>-editor-plugin` on
`zircon_app`; a native build produces the dist artifact consumed by discovery.

## Synchronization And Checks

After changing declaration metadata, regenerate one snapshot or all snapshots:

```powershell
cargo zircon plugin sync-manifest demo_probe
cargo zircon plugin sync-manifest
```

Before review, run the read-only repository gate and package validator:

```powershell
cargo zircon plugin check
cargo zircon plugin validate zircon_plugins/demo_probe
cargo zircon plugin validate zircon_plugins/demo_probe --artifact path/to/zircon_plugin_demo_probe_dist.dll
```

`--artifact` and `plugin check --artifact-root` load the selected dynamic
libraries so the validator can inspect their exported v3 descriptor and entry
symbols. Use these options only for artifacts built from trusted source; use the
manifest-only commands when inspecting an untrusted package.

`check` covers declaration drift, workspace membership, catalog snapshot wiring,
and manifest contracts. After building the plugin workspace, pass its profile
directory with `plugin check --artifact-root zircon_plugins/target/debug` to load
each declared dist library and verify ABI v3, plugin identity, embedded manifest,
requested capabilities, entry names, and exported entry symbols. `validate`
accepts a package directory or direct `plugin.toml` path; `--artifact <file>`
performs the same probe for a third-party package. Failures use stable diagnostic
codes and repair hints. Neither command restores legacy manifest headers or
hand-written native ABI constants.
