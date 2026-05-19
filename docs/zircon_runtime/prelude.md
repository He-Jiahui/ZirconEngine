---
related_code:
  - zircon_runtime/src/prelude.rs
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/core/modules/log.rs
  - zircon_runtime/src/core/tasks/mod.rs
  - zircon_runtime/src/core/tasks/report.rs
  - zircon_runtime/src/core/time.rs
  - zircon_runtime/src/core/runtime/handle/time.rs
  - zircon_runtime/src/input/mod.rs
  - zircon_runtime/src/platform/mod.rs
  - zircon_runtime/src/core/framework/window/mod.rs
  - zircon_runtime/src/core/framework/window/constants.rs
  - zircon_runtime/src/core/framework/window/descriptor.rs
  - zircon_runtime/src/core/framework/time/mod.rs
  - zircon_runtime/src/core/state/mod.rs
  - zircon_runtime/src/core/diagnostics/mod.rs
  - zircon_runtime/src/diagnostic_log/mod.rs
  - zircon_runtime/src/diagnostic_log/diagnostics.rs
  - zircon_runtime/src/diagnostic_log/level.rs
  - zircon_runtime/src/diagnostic_log/settings.rs
  - zircon_runtime/src/engine_module/mod.rs
implementation_files:
  - zircon_runtime/src/prelude.rs
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/core/tasks/report.rs
  - zircon_runtime/src/core/framework/window/mod.rs
  - zircon_runtime/src/core/framework/window/constants.rs
  - zircon_runtime/src/core/framework/window/descriptor.rs
  - zircon_runtime/src/diagnostic_log/level.rs
  - zircon_runtime/src/diagnostic_log/settings.rs
plan_sources:
  - user: 2026-05-08 continue ZirconEngine Bevy completion roadmap M2 stable prelude
  - user: 2026-05-08 continue ZirconEngine Bevy completion roadmap M3 State
  - user: 2026-05-16 continue Bevy-style task pools and default log diagnostics
  - user: 2026-05-16 continue Bevy-style platform/window/input stable prelude completion
  - user: 2026-05-16 continue Bevy-style profile diagnostics stable prelude completion
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
tests:
  - zircon_runtime/src/tests/prelude.rs
  - zircon_runtime/src/tests/time.rs
  - zircon_runtime/src/platform/tests.rs
  - zircon_runtime/src/input/tests/boundary.rs
  - zircon_runtime/src/input/tests/input_manager.rs
  - zircon_runtime/src/tests/state.rs
  - zircon_runtime/src/tests/mod.rs
  - .github/workflows/ci.yml
doc_type: module-detail
---

# Zircon Runtime Prelude

`zircon_runtime::prelude` is the stable convenience import surface for runtime-facing application code, modules, examples, and tests. It is intentionally behavior-free: the module only re-exports types that are already owned by lower runtime layers.

## Ownership Boundary

- `zircon_runtime::core` owns lifecycle, descriptors, runtime handles, event/config primitives, `FrameClock`, runtime-owned `RuntimeTimeClocks`, the compatibility `JobScheduler`, and runtime-owned `TaskPools`.
- `zircon_runtime::core::state` owns the M3 runtime-wide state contracts and transition hooks.
- `zircon_runtime::core::framework::time` owns the Bevy-inspired `Time<Real>`, `Time<Virtual>`, and `Time<Fixed>` contracts.
- `zircon_runtime::core::diagnostics` and `zircon_runtime::diagnostic_log` own diagnostic snapshots, stores, paths, and log filters.
- `zircon_runtime::platform` owns target capability declarations for windows, event-loop policy, desktop/mobile/browser/headless target modes, and backend status reporting.
- `zircon_runtime::core::framework::window` owns neutral window descriptors, resolution, resize constraints, position, mode, present mode, and primary-window handle DTOs.
- `zircon_runtime::input` owns neutral input events, button state, frame snapshots, gamepad/touch vocabulary, and the default runtime input reducer.
- `zircon_runtime::engine_module` owns module/service descriptor helpers and the `EngineModule` trait.
- `zircon_runtime::prelude` only curates those public surfaces for ergonomic imports.

This keeps the prelude behavior-free: milestones can add stable contracts after their owning modules exist, but the prelude itself does not introduce behavior or hidden ownership.

## Export Groups

The runtime prelude exports:

- core runtime types such as `CoreRuntime`, `CoreHandle`, `ModuleDescriptor`, `RegistryName`, `DependencySpec`, `StartupMode`, and `LifecycleState`,
- module/service helper contracts such as `EngineModule`, `EngineService`, `EngineDriver`, `EngineManager`, and descriptor-construction helpers,
- current foundation utilities such as `JobScheduler`, `TaskPools`, `TaskPoolKind`, `FrameClock`, `FoundationModule`, and core descriptor modules for log, tasks, time, frame count, diagnostics, and development log diagnostics,
- platform/window/input contracts such as `PlatformCapabilityMatrix`, `PlatformFeatureSelection`, `PlatformTarget`, `PlatformConfig`, `PLATFORM_CONFIG_KEY`, `WindowBackend`, `WindowDescriptor`, `WindowResolution`, `WindowResizeConstraints`, `WindowPresentMode`, `PrimaryWindowHandle`, `PRIMARY_WINDOW_DESCRIPTOR_CONFIG_KEY`, `InputBackend`, `GamepadBackend`, `PlatformModule`, `InputModule`, `InputEvent`, `InputButton`, `InputFrameSnapshot`, and `DefaultInputManager`,
- state contracts `StateSpec`, `State`, `NextState`, `StateTransitionEvent`, `OnEnter`, `OnExit`, and `OnTransition`,
- time contracts and runtime clock snapshots such as `Time`, `Real`, `Virtual`, `Fixed`, `FixedStepPlan`, `RuntimeTimeClocks`, and `RuntimeTimeAdvance`,
- Bevy-style runtime Time diagnostic path constants such as `TIME_FRAME_COUNT_DIAGNOSTIC`, `TIME_FIXED_STEPS_DIAGNOSTIC`, `TIME_FRAME_TIME_DIAGNOSTIC`, and `TIME_FPS_DIAGNOSTIC`,
- diagnostics and log filter types such as `DiagnosticStore`, `DiagnosticPath`, `RuntimeDiagnosticsSnapshot`, `DiagnosticLogFilter`, `DiagnosticLogLevel`, `DiagnosticLogSettings` / `LogSettings`, diagnostic-store log formatting helpers, `DiagnosticStoreLogSchedule`, and the `ZIRCON_LOG_FILTER` / `ZIRCON_LOG` / `RUST_LOG` environment constant names,
- runtime profile and target selection types such as `RuntimeProfileDescriptor`, `RuntimeProfileId`, `RuntimeCoreProfile`, `EditorCoreProfile`, `PluginMaturity`, `RuntimePluginId`, and `RuntimeTargetMode`.

## Non-Goals

The prelude does not re-export every runtime subsystem. Graphics, scene, UI, asset, native plugin ABI, and renderer-specific data remain in their owning modules unless a later milestone promotes a stable subset. This prevents the prelude from becoming a broad compatibility shim or hiding ownership boundaries.

The prelude also does not introduce new API names. Later Bevy-aligned milestones such as concrete winit host event conversion, gilrs polling, or renderer/editor exports should be implemented in their owning modules first and then added here only after their contracts are stable.

## Validation

`zircon_runtime/src/tests/prelude.rs` exercises the public surface by importing only `crate::prelude::*` and constructing runtime descriptors, lifecycle values, state contracts, time clocks, runtime-owned clock snapshots, task pools, task-pool diagnostic reports, diagnostic stores, log filters, runtime profiles, the current job scheduler, core descriptor modules, and the promoted platform/input contracts.

Current coverage includes:

- task-pool exports for `TaskPools`, `TaskPoolKind`, `TaskPoolOptions`, `TaskPoolReport`, `TaskPoolReportEntry`, thread counts, and the compatibility `JobScheduler`;
- log and diagnostics descriptor exports for `LogModule` and `LogDiagnosticsModule`;
- diagnostic-log formatting helpers for runtime `DiagnosticStoreSnapshot` values;
- diagnostic-log settings exports for `DiagnosticLogSettings`, its Bevy-aligned `LogSettings` alias, sink toggles, and stable settings diagnostics;
- diagnostic-log cadence helpers for Bevy-style dev-profile diagnostic-store output;
- platform/window/input exports for `PlatformCapabilityMatrix`, `PlatformFeatureSelection::bevy_default_platform()`, `PLATFORM_CONFIG_KEY`, desktop `Winit`/winit-input/`Gilrs` capability status, neutral `WindowDescriptor` / `PrimaryWindowHandle` defaults, `PRIMARY_WINDOW_DESCRIPTOR_CONFIG_KEY`, `InputEvent`, `InputButton`, `PlatformModule`, and `InputModule`;
- runtime profile diagnostics for `RuntimeProfileDescriptor`, `RuntimeCoreProfile`, and `PluginMaturity`;
- state transition exports for `State`, `NextState`, `OnEnter`, and `StateTransitionEvent`;
- Time diagnostic path constants for frame count, fixed steps, frame time, and FPS, matching the runtime-owned Time diagnostics recorded by `CoreHandle::advance_time_by(...)`.

Milestone acceptance still needs the repository validation gate from `.github/workflows/ci.yml` once concurrent workspace builds are clear. During implementation, formatting and diff hygiene checks are the lightweight guard for this prelude slice.
