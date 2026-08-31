---
related_code:
  - zircon_app/src/entry/export_bootstrap.rs
  - zircon_app/src/entry/entry_runner/bootstrap.rs
  - zircon_app/src/entry/product_composition/request.rs
  - zircon_app/src/entry/product_composition/composition.rs
  - zircon_app/src/entry/mod.rs
  - zircon_app/src/lib.rs
  - zircon_app/src/entry/tests/export_bootstrap.rs
  - zircon_app/src/entry/tests/mod.rs
  - zircon_runtime/src/plugin/export_build_plan/main_template.rs
  - zircon_runtime/src/plugin/export_build_plan/platform_host_files.rs
  - zircon_runtime/src/plugin/export_build_plan/plugin_selection_template.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan_feature_provider.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan_platform.rs
implementation_files:
  - docs/zircon_app/export-bootstrap.md
  - zircon_app/src/entry/export_bootstrap.rs
  - zircon_app/src/entry/entry_runner/bootstrap.rs
  - zircon_app/src/entry/product_composition/request.rs
  - zircon_app/src/entry/product_composition/composition.rs
  - zircon_runtime/src/plugin/export_build_plan/main_template.rs
  - zircon_runtime/src/plugin/export_build_plan/platform_host_files.rs
  - zircon_runtime/src/plugin/export_build_plan/plugin_selection_template.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - docs/engine-architecture/generated-code-boundary.md
tests:
  - rustfmt --edition 2021 --check zircon_app/src/entry/export_bootstrap.rs zircon_app/src/entry/entry_runner/bootstrap.rs zircon_app/src/entry/tests/export_bootstrap.rs
  - rustc --edition 2021 --test zircon_runtime/src/tests/runtime_absorption/generated_code_guard.rs
  - cargo check -p zircon_app --lib --no-default-features --features core-min --locked --target-dir D:/cargo-targets/zircon-export-bootstrap-0612-app-core-min --message-format short --color never
  - cargo test -p zircon_app --lib export_bootstrap --no-default-features --features core-min --locked --target-dir D:/cargo-targets/zircon-export-bootstrap-0612-app-core-min --message-format short --color never -- --nocapture --test-threads=1
  - cargo test -p zircon_runtime --lib export_build_plan --locked --target-dir D:/cargo-targets/zircon-export-bootstrap-0612-runtime --message-format short --color never -- --nocapture --test-threads=1
doc_type: module-detail
---

# Export Bootstrap

## Purpose

`zircon_app::entry::export_bootstrap` is the handwritten owner for exported runtime startup. It exists so generated export scaffolds do not assemble `EntryConfig`, call `EntryRunner`, or drive native plugin loading directly.

Generated export code now passes an `ExportRuntimeBootstrapConfig` into one stable facade:

- `bootstrap_export_runtime` for linked/static source-template exports;
- `bootstrap_export_runtime_with_native_plugins_from_export_root` for exports that also carry native dynamic plugin packages;
- `discover_export_root` for locating the export root that contains `plugins/native_plugins.toml`.

## Ownership

The generated `zircon_plugins.rs` file may still build data tables and linked registration provider rows. It does not own runtime startup order or execute registration calls directly. The app layer converts that data into `EntryConfig`, executes provider rows into registration reports, and submits one `ProductCompositionRequest`.

`ExportRuntimeBootstrapConfig` has one configuration authority: the `ExportProfile` plus the project plugin manifest. It does not carry a second `EntryProfile` or target-mode field. The app derives the product role from the export target mode and platform before resolution: desktop targets map to `DesktopClient`, headless server exports map to `Server`, Android maps to `AndroidClient`, browser/Wasm maps to `WebClient`, and the iOS embedded library maps to `Embedded`. Roles without a dedicated host owner fail with `UnsupportedProductRole` before module composition or native plugin filesystem access; generated scaffolds must not silently relabel them as desktop clients.

Resolution starts from the selected runtime profile's plugin manifest and overlays generated export selections by plugin ID. An empty generated project manifest therefore preserves the profile defaults instead of erasing them. Provenance records both `RuntimeProfile` and `ExportProfile` in a compact `ProductConfigSourceSet` when both contribute.

`ProductCompositionRequest` is the only lower-level merge point. It resolves the product request before touching the native export root, preserves linked registration reports supplied by the export scaffold, loads native runtime plugin reports through `NativePluginHostHandle`, and moves both report sets into one module compilation. `ProductComposition` then retains the resolved config, immutable module selection receipt and identity, `CoreHandle`, compiled plugin plan, bridge lifecycle state, diagnostics, and optional native host owner. There is intentionally no public `core` borrow, `into_core` shortcut, or public `EngineEntry::bootstrap` path because detaching the cloneable Core handle would make plugin lifetime correctness depend on caller discipline.

Both export bootstrap facades return `ProductComposition`. Executable templates retain it in their main stack frame. Mobile and browser library templates retain it in one process-level `Mutex<ZirconProductCompositionState>` with `Vacant`, `Starting`, `Running`, and `Stopping` states. Bootstrap and composition destruction run outside the mutex: reentrant or concurrent start/shutdown cannot deadlock on plugin initialization or overlap two product generations. Successful destruction transitions `Stopping -> Vacant`; a destruction panic leaves the owner in `Stopping`, so a new generation cannot start after incomplete cleanup. ABI event/resource callbacks return `false` unless the state is `Running`.

Every generated C/JNI export delegates through one `catch_unwind` guard and converts a panic to `false`, so Rust unwinding cannot cross the foreign ABI boundary. Android JNI and the iOS C header expose the same explicit shutdown symbol; the Android activity calls it from `onDestroy`, and the iOS app delegate calls it from `applicationWillTerminate`. Browser `pagehide` preserves the owner for a back-forward-cache transition and invokes shutdown only for a non-persisted terminal page hide.

## Current Boundaries

Allowed in generated export scaffolds:

- `zircon_plugins::export_runtime_bootstrap_config()`;
- `zircon_app::bootstrap_export_runtime(...)`;
- `zircon_app::bootstrap_export_runtime_with_native_plugins_from_export_root(...)`;
- `zircon_app::discover_export_root()?`.

Forbidden in generated export scaffolds:

- `EntryRunner::...`;
- `EntryConfig::new(...)`;
- generated `EntryProfile` selection or a duplicate target-mode constructor argument;
- `NativePluginLoader`;
- `load_runtime_from_load_manifest`;
- direct `runtime_plugin_registrations()` calls from `main.rs` or platform `lib.rs`.
- direct `plugin_registration()` or `plugin_feature_registration()` immediate calls in plugin-selection templates.
- extracting and retaining only `CoreHandle` while discarding the product composition receipt and plugin owners.

## Verification Notes

The app library check passed under `core-min` for the initial export-bootstrap facade. After the provider-table follow-up, the next app library check did not reach `zircon_app`: it failed first in the runtime dependency on unrelated render drift, with `zircon_runtime/src/scene/world/render.rs` importing missing `render_mesh_stable_instance_key` and `render_mesh_transform_revision` from `core::framework::render`. The later adapter-only audit update did not start a new Cargo run because other Cargo/rustc lanes were already active.

The two focused Cargo test commands for `zircon_app --lib export_bootstrap` and `zircon_runtime --lib export_build_plan` were attempted before the provider-table follow-up but timed out during Windows test-target compilation after 300 seconds, while other active sessions were also compiling runtime test targets. The orphaned validation cargo processes from this slice were stopped; unrelated active session builds were left running.

The runtime absorption guard now has dedicated entry-template and provider-table regressions. It rejects any generated entry template that reintroduces `EntryRunner`, `EntryConfig::new`, `NativePluginLoader`, direct manifest loading, or direct linked-registration calls. It also rejects `plugin_selection_template.rs` if it reintroduces `plugin_registration()` or `plugin_feature_registration()` immediate call forms instead of provider-table handoff.

The generated-code structural audit accepts provider-table rows as generated data adapters. After that decision, the generated-code boundary reports `template_file_count=9`, `behavior_location_count=6`, `allowed_adapter_location_count=6`, `migration_debt_location_count=0`, `behavior_decision_count=3`, `unclassified_behavior_label_count=0`, `generated_boundary_migration_debt_count=0`, and `m1_gate_status=classified-and-clear`.

On 2026-08-27 the bootstrap surface was hard-cut to `ProductCompositionRequest -> ProductComposition`; the old report/no-report and plugin-source `bootstrap_with_*` permutations and the detachable `EntryRuntimeBootstrap` / `NativePluginRuntimeBootstrap` results were removed. Rust 1.94 formatting and scoped source/path checks passed for this source slice. Managed Cargo, export behavior, and feature-matrix validation remain pending and this source record does not supersede them.
