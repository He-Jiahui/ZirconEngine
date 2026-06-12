---
related_code:
  - zircon_app/src/entry/export_bootstrap.rs
  - zircon_app/src/entry/entry_runner/bootstrap.rs
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

The generated `zircon_plugins.rs` file may still build data tables and linked registration provider rows. It does not own runtime startup order or execute registration calls directly. The app layer converts that data into `EntryConfig`, executes provider rows into registration reports, merges linked and native registration reports, and then delegates to `EntryRunner`.

`EntryRunner::bootstrap_with_runtime_plugin_and_feature_registrations_and_native_plugins_from_export_root` is the lower-level merge point. It preserves linked registration reports supplied by the export scaffold, loads native runtime plugin reports through `NativePluginLiveHost`, merges both report sets, and keeps the live native host alive in `NativePluginRuntimeBootstrap`.

## Current Boundaries

Allowed in generated export scaffolds:

- `zircon_plugins::export_runtime_bootstrap_config()`;
- `zircon_app::bootstrap_export_runtime(...)`;
- `zircon_app::bootstrap_export_runtime_with_native_plugins_from_export_root(...)`;
- `zircon_app::discover_export_root()?`.

Forbidden in generated export scaffolds:

- `EntryRunner::...`;
- `EntryConfig::new(...)`;
- `NativePluginLoader`;
- `load_runtime_from_load_manifest`;
- direct `runtime_plugin_registrations()` calls from `main.rs` or platform `lib.rs`.
- direct `plugin_registration()` or `plugin_feature_registration()` immediate calls in plugin-selection templates.

## Verification Notes

The app library check passed under `core-min` for the initial export-bootstrap facade. After the provider-table follow-up, the next app library check did not reach `zircon_app`: it failed first in the runtime dependency on unrelated render drift, with `zircon_runtime/src/scene/world/render.rs` importing missing `render_mesh_stable_instance_key` and `render_mesh_transform_revision` from `core::framework::render`. The later adapter-only audit update did not start a new Cargo run because other Cargo/rustc lanes were already active.

The two focused Cargo test commands for `zircon_app --lib export_bootstrap` and `zircon_runtime --lib export_build_plan` were attempted before the provider-table follow-up but timed out during Windows test-target compilation after 300 seconds, while other active sessions were also compiling runtime test targets. The orphaned validation cargo processes from this slice were stopped; unrelated active session builds were left running.

The runtime absorption guard now has dedicated entry-template and provider-table regressions. It rejects any generated entry template that reintroduces `EntryRunner`, `EntryConfig::new`, `NativePluginLoader`, direct manifest loading, or direct linked-registration calls. It also rejects `plugin_selection_template.rs` if it reintroduces `plugin_registration()` or `plugin_feature_registration()` immediate call forms instead of provider-table handoff.

The generated-code structural audit accepts provider-table rows as generated data adapters. After that decision, the generated-code boundary reports `template_file_count=9`, `behavior_location_count=6`, `allowed_adapter_location_count=6`, `migration_debt_location_count=0`, `behavior_decision_count=3`, `unclassified_behavior_label_count=0`, `generated_boundary_migration_debt_count=0`, and `m1_gate_status=classified-and-clear`.
