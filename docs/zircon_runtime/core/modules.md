---
related_code:
  - zircon_runtime/src/core/modules/mod.rs
  - zircon_runtime/src/core/modules/log.rs
  - zircon_runtime/src/core/modules/tasks.rs
  - zircon_runtime/src/core/modules/time.rs
  - zircon_runtime/src/core/modules/frame_count.rs
  - zircon_runtime/src/core/modules/diagnostics.rs
  - zircon_runtime/src/core/time.rs
  - zircon_runtime/src/core/diagnostics/store.rs
  - zircon_runtime/src/core/diagnostics/collect.rs
  - zircon_runtime/src/diagnostic_log/diagnostics.rs
  - zircon_app/src/plugins/groups.rs
implementation_files:
  - zircon_runtime/src/core/modules/mod.rs
  - zircon_runtime/src/core/modules/log.rs
  - zircon_runtime/src/core/modules/tasks.rs
  - zircon_runtime/src/core/modules/time.rs
  - zircon_runtime/src/core/modules/frame_count.rs
  - zircon_runtime/src/core/modules/diagnostics.rs
plan_sources:
  - user: 2026-05-16 continue Bevy-style app/prelude/state/time/tasks/log/diagnostic completion
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
  - .codex/plans/ZirconEngine Bevy 参照基础设施收束计划.md
  - dev/bevy/crates/bevy_internal/src/default_plugins.rs
  - dev/bevy/crates/bevy_diagnostic/src/log_diagnostics_plugin.rs
tests:
  - zircon_app/src/plugins/tests.rs
  - zircon_runtime/src/tests/prelude.rs
doc_type: module-detail
---

# Core Descriptor Modules

`zircon_runtime::core::modules` contains lightweight `EngineModule` descriptors for app-level infrastructure that is already owned by `CoreRuntime` or other core support modules. These descriptors let `zircon_app::plugins` build Bevy-style `MinimalPlugins`, `DefaultPlugins`, and `DevPlugins` groups without moving ownership of the concrete runtime services into `zircon_app`.

## Ownership Boundary

The descriptor modules are lifecycle registration markers. They do not install duplicate service instances, spawn threads, initialize global log sinks, or collect diagnostics by themselves.

- `TasksModule` points at the runtime-owned `TaskPools` and `JobScheduler` support.
- `TimeModule` points at the core time contracts and runtime-owned `RuntimeTimeClocks` surface.
- `FrameCountModule` names the frame counter diagnostic slot now populated from runtime time advancement.
- `DiagnosticsCoreModule` names the runtime-owned diagnostic store and snapshot support.
- `LogModule` names the process diagnostic log surface initialized by entry runners.
- `LogDiagnosticsModule` names the development-only log diagnostics surface layered into `DevPlugins`, including formatting runtime diagnostic-store snapshots into process-log lines.

This keeps the public plugin-group layer declarative while preserving the fixed architecture: `zircon_app` composes, `zircon_runtime::core` owns runtime state and services, and entry runners initialize process-local sinks.

## Built-In Group Policy

`MinimalPlugins` stays small and includes only foundation, tasks, time, frame count, and diagnostics core.

`DefaultPlugins` and `HeadlessPlugins` include `LogModule`, so default runtime and headless profiles expose a stable log descriptor. `DevPlugins` adds `LogDiagnosticsModule` after `DiagnosticsCoreModule`, matching the Bevy development pattern where verbose log diagnostics are layered over the default stack.

## Validation

`zircon_app/src/plugins/tests.rs` verifies built-in group membership for the log and log-diagnostics descriptors. `zircon_runtime/src/tests/prelude.rs` verifies that the stable runtime prelude exports both descriptor modules and their public names.
