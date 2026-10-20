---
related_code:
  - zircon_runtime/src/core/runtime/modules/mod.rs
  - zircon_runtime/src/core/runtime/modules/log.rs
  - zircon_runtime/src/core/runtime/modules/tasks.rs
  - zircon_runtime/src/core/runtime/modules/time.rs
  - zircon_runtime/src/core/runtime/modules/frame_count.rs
  - zircon_runtime/src/core/runtime/modules/diagnostics.rs
  - zircon_runtime/src/core/runtime/time.rs
  - zircon_runtime/src/core/runtime/diagnostics/store.rs
  - zircon_runtime/src/runtime_diagnostics/collect.rs
  - zircon_runtime/src/diagnostic_log/diagnostics.rs
  - zircon_app/src/plugins/groups.rs
implementation_files:
  - zircon_runtime/src/core/runtime/modules/mod.rs
  - zircon_runtime/src/core/runtime/modules/log.rs
  - zircon_runtime/src/core/runtime/modules/tasks.rs
  - zircon_runtime/src/core/runtime/modules/time.rs
  - zircon_runtime/src/core/runtime/modules/frame_count.rs
  - zircon_runtime/src/core/runtime/modules/diagnostics.rs
plan_sources:
  - user: 2026-05-16 continue Bevy-style app/prelude/state/time/tasks/log/diagnostic completion
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
  - .codex/plans/ZirconEngine Bevy 参照基础设施收束计划.md
  - dev/bevy/crates/bevy_internal/src/default_plugins.rs
  - dev/bevy/crates/bevy_diagnostic/src/log_diagnostics_plugin.rs
tests:
  - zircon_runtime/src/tests/prelude.rs
doc_type: module-detail
---

# Core Descriptor Modules

`zircon_runtime::core::runtime::modules` contains lightweight `EngineModule` descriptors for app-level infrastructure that is already owned by `CoreRuntime` or other core support modules. These descriptors let `zircon_app::plugins` build Bevy-style `MinimalPlugins`, `DefaultPlugins`, and `DevPlugins` groups without moving ownership of the concrete runtime services into `zircon_app`.

## Ownership Boundary

The descriptor modules are lifecycle registration markers. They do not install duplicate service instances, spawn threads, initialize global log sinks, or collect diagnostics by themselves.

- `TasksModule` points at the runtime-owned `TaskPools` and `JobScheduler` support.
- `TimeModule` points at the core time contracts, outer-frame handoff, and default World-policy surface.
- `FrameCountModule` names the frame counter diagnostic slot now populated from runtime time advancement.
- `DiagnosticsCoreModule` names the runtime-owned diagnostic store and snapshot support.
- `LogModule` names the process diagnostic log surface initialized by entry runners.
- `LogDiagnosticsModule` names the development-only log diagnostics surface layered into `DevPlugins`, including formatting runtime diagnostic-store snapshots into process-log lines.

This keeps the public plugin-group layer declarative while preserving the fixed architecture: `zircon_app` composes, `zircon_runtime::core` owns runtime state and services, and entry runners initialize process-local sinks.

The 2026-06-12 runtime 02 M2.2 migration moved the former `core/modules/` root directory under `core/runtime/modules/`. Callers that need the module namespace use `core::runtime::modules`; the root `core` facade still re-exports the individual descriptor types and constants for prelude stability.

## Built-In Group Policy

`MinimalPlugins` stays small and includes only foundation, tasks, time, frame count, and diagnostics core.

`DefaultPlugins` and `HeadlessPlugins` include `LogModule`, so default runtime and headless profiles expose a stable log descriptor. `DevPlugins` adds `LogDiagnosticsModule` after `DiagnosticsCoreModule`, matching the Bevy development pattern where verbose log diagnostics are layered over the default stack.

## Validation

`zircon_app/src/plugins/tests.rs` verifies built-in group membership for the log and log-diagnostics descriptors. `zircon_runtime/src/tests/prelude.rs` verifies that the stable runtime prelude exports both descriptor modules and their public names.

## Validation ownership

Module/lifecycle policy and its independent validation worklists remain separate. 此处仅展示当前现状的概述；historical test-split records do not become production module requirements.

An inventory child owns inventory.rs; a sources child owns source reads. Route split_layout.rs owns mounting, mirror_docs.rs owns independent document checks, and status_docs.rs owns status obligations. D1 capability source checks retain runtime_roots.rs, audit/SDK consumers and their own folder-backed guards; no compatibility test path is introduced.

The documented `cargo check -p zircon_runtime --lib --locked` is a retained command identity, not a command authorized for unspecific CI execution. Actual invocation still needs the explicit feature/target/profile declaration. mirror_docs_guard_present = true is the required audit result, not a new run.

### Dated split provenance

The 2026-07-06 records cover foundation status-mirror folder backing, typed-error source/child inventory, core root/generated audit routing and D1 capability routing. The 2026-07-08 record aligns Runtime07 owner-budget children and its independent status maps. Retain their stable route/status identities while keeping Cargo deferred. Detailed output belongs to 02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md.

### Retained source status identities

| Contract | Responsibility / evidence boundary |
|---|---|
| `frameworks_02_m3_core_spine_root_generated_audit_source_sync_static_passed_cargo_deferred` | Core root/generated inventories follow their actual children. |
| `frameworks_02_m3_plugin_importer_d1_capability_guard_folder_backed_static_passed_cargo_deferred` | Capability declarations and SDK/manifest registration share one reviewed source. |
| `frameworks_02_m3_review_guard_foundation_status_mirror_guard_folder_backed_static_passed_cargo_deferred` | Independent status mirrors retain their source/document responsibilities. |
| `frameworks_02_m3_runtime_07_owner_budget_child_source_current_route_sync_static_passed_cargo_deferred` | Budget sources and expected-slice maps follow actual child routes. |
| `frameworks_02_m3_typed_error_status_doc_paths_child_inventory_split_layout_sources_child_split_static_passed_cargo_deferred` | Typed-error routing, inventory and status mirrors stay independently mounted. |
| `frameworks_02_m3_typed_error_status_doc_paths_status_current_sources_child_split_static_passed_cargo_deferred` | Typed-error routing, inventory and status mirrors stay independently mounted. |
| `runtime_15_core_spine_root_generated_audit_source_sync_static_passed_cargo_deferred` | Core root/generated inventories follow their actual children. |
| `runtime_15_plugin_importer_d1_capability_guard_folder_backed_static_passed_cargo_deferred` | Capability declarations and SDK/manifest registration share one reviewed source. |
| `runtime_15_review_guard_foundation_status_mirror_guard_folder_backed_static_passed_cargo_deferred` | Independent status mirrors retain their source/document responsibilities. |
| `runtime_15_runtime_07_owner_budget_child_source_current_route_sync_static_passed_cargo_deferred` | Budget sources and expected-slice maps follow actual child routes. |
| `runtime_15_typed_error_status_doc_paths_child_inventory_split_layout_sources_child_split_static_passed_cargo_deferred` | Typed-error routing, inventory and status mirrors stay independently mounted. |
| `runtime_15_typed_error_status_doc_paths_status_current_sources_child_split_static_passed_cargo_deferred` | Typed-error routing, inventory and status mirrors stay independently mounted. |

### Source and guard ownership

The finite source, guard and provenance inventory for these responsibilities is:

- `code_review_findings/plugin_importer_dx/d1_capability_single_source/runtime_roots.rs`, `code_review_findings/plugin_importer_dx/d1_capability_single_source/split_layout.rs`, `code_review_findings/plugin_importer_dx/d1_capability_single_source.rs`.

### Validation interfaces

Keep these existing validation and domain interfaces in the gate:

- `zircon_plugins`.

### Clause provenance

These content identities preserve the source of this transfer. They are not fresh measurements, active-file dependencies or acceptance receipts.

- `02-module-kernel-and-lifecycle-unification.md`, SHA-256 `57cde522acaa735d10a18a5cf6784b4ebdb9a14a439bb12be13bdb680fe984cc`, selected source lines 58-269.
