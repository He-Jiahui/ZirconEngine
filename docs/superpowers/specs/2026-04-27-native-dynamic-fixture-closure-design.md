# Native Dynamic Fixture Closure Design

## Goal

Close the real dynamic-library path for native plugins by adding one buildable fixture plugin that exports the ABI v1 descriptor symbol plus runtime and editor entry symbols. The fixture must prove that `NativePluginLoader` can load an actual `.dll`/`.so`, read descriptor metadata, call both entries, and merge diagnostics into registration reports.

## Scope

- Add a narrow `zircon_plugins/native_dynamic_fixture` plugin package with a `cdylib` crate.
- Keep the fixture independent of editor UI, render feature migration, and app entry reshaping.
- Keep `zircon_plugins` independent from the root workspace default members.
- Use the existing ABI v1 structs and loader behavior unless a failing test proves a missing seam.

## Architecture

- The fixture package lives under `zircon_plugins/native_dynamic_fixture` with `plugin.toml` plus `native/Cargo.toml` and `native/src/lib.rs`.
- The native crate builds as `cdylib` and exports:
  - `zircon_native_plugin_descriptor_v1`
  - `zircon_native_dynamic_fixture_runtime_entry_v1`
  - `zircon_native_dynamic_fixture_editor_entry_v1`
- Exported symbols return static ABI structs and static C strings so pointers remain valid after the call.
- The package manifest embedded in the descriptor and entry reports uses the existing `PluginPackageManifest` TOML contract.

## Data Flow

1. A focused runtime test builds the fixture `cdylib` in the plugin workspace.
2. The test creates a temporary native package, copies `plugin.toml`, and copies the platform-named built library into that package's `native/` directory.
3. `NativePluginLoader.load_discovered_all(...)` discovers the package, loads the library, probes `zircon_native_plugin_descriptor_v1`, and calls both entry symbols from the descriptor.
4. Assertions verify descriptor metadata, runtime/editor entry reports, package manifest merging, and diagnostic propagation.

## Error Handling

- Existing loader behavior remains non-fatal: missing libraries, missing descriptor symbols, unsupported ABI versions, invalid manifest TOML, and entry failures produce diagnostics instead of panics.
- The fixture entries intentionally return deterministic diagnostics so the test proves entry diagnostics are preserved and attributed to the plugin id.

## Testing

- First add a failing focused runtime test for loading the real fixture dynamic library.
- Then add the fixture crate and any minimal workspace wiring needed for `cargo build` under `zircon_plugins`.
- Validate with a narrow command first, expected shape:
  - `cargo test -p zircon_runtime --lib native_dynamic_fixture --locked --jobs 1`
  - `cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_native_dynamic_fixture_native --locked --jobs 1`
- Do not claim final migration acceptance until later final-regression steps run separately.

## Non-Goals

- No editor UI window verification in this slice.
- No broad export runner matrix in this slice beyond what is needed to make the fixture loadable.
- No cleanup of physics/sound/animation/navigation/particles/net/texture/render migration residue in this slice.
- No changes to current active editor viewport, operation, or render-feature plugin cutover areas.
