---
related_code:
  - zircon_runtime/src/prelude.rs
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/core/framework/time/mod.rs
  - zircon_runtime/src/core/state/mod.rs
  - zircon_runtime/src/core/diagnostics/mod.rs
  - zircon_runtime/src/diagnostic_log/mod.rs
  - zircon_runtime/src/engine_module/mod.rs
implementation_files:
  - zircon_runtime/src/prelude.rs
  - zircon_runtime/src/lib.rs
plan_sources:
  - user: 2026-05-08 continue ZirconEngine Bevy completion roadmap M2 stable prelude
  - user: 2026-05-08 continue ZirconEngine Bevy completion roadmap M3 State
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
tests:
  - zircon_runtime/src/tests/prelude.rs
  - zircon_runtime/src/tests/state.rs
  - zircon_runtime/src/tests/mod.rs
  - .github/workflows/ci.yml
doc_type: module-detail
---

# Zircon Runtime Prelude

`zircon_runtime::prelude` is the stable convenience import surface for runtime-facing application code, modules, examples, and tests. It is intentionally behavior-free: the module only re-exports types that are already owned by lower runtime layers.

## Ownership Boundary

- `zircon_runtime::core` owns lifecycle, descriptors, runtime handles, event/config primitives, `FrameClock`, and the current `JobScheduler`.
- `zircon_runtime::core::state` owns the M3 runtime-wide state contracts and transition hooks.
- `zircon_runtime::core::framework::time` owns the Bevy-inspired `Time<Real>`, `Time<Virtual>`, and `Time<Fixed>` contracts.
- `zircon_runtime::core::diagnostics` and `zircon_runtime::diagnostic_log` own diagnostic snapshots, stores, paths, and log filters.
- `zircon_runtime::engine_module` owns module/service descriptor helpers and the `EngineModule` trait.
- `zircon_runtime::prelude` only curates those public surfaces for ergonomic imports.

This keeps the prelude behavior-free: milestones can add stable contracts after their owning modules exist, but the prelude itself does not introduce behavior or hidden ownership.

## Export Groups

The runtime prelude exports:

- core runtime types such as `CoreRuntime`, `CoreHandle`, `ModuleDescriptor`, `RegistryName`, `DependencySpec`, `StartupMode`, and `LifecycleState`,
- module/service helper contracts such as `EngineModule`, `EngineService`, `EngineDriver`, `EngineManager`, and descriptor-construction helpers,
- current foundation utilities such as `JobScheduler`, `FrameClock`, `FoundationModule`, and core descriptor modules for tasks, time, frame count, and diagnostics,
- state contracts `StateSpec`, `State`, `NextState`, `StateTransitionEvent`, `OnEnter`, `OnExit`, and `OnTransition`,
- time contracts `Time`, `Real`, `Virtual`, `Fixed`, and `FixedStepPlan`,
- diagnostics and log filter types such as `DiagnosticStore`, `DiagnosticPath`, `RuntimeDiagnosticsSnapshot`, `DiagnosticLogFilter`, and `DiagnosticLogLevel`,
- runtime profile and target selection types such as `RuntimeProfileDescriptor`, `RuntimeProfileId`, `RuntimePluginId`, and `RuntimeTargetMode`.

## Non-Goals

The prelude does not re-export every runtime subsystem. Graphics, scene, UI, asset, native plugin ABI, and renderer-specific data remain in their owning modules unless a later milestone promotes a stable subset. This prevents the prelude from becoming a broad compatibility shim or hiding ownership boundaries.

The prelude also does not introduce new API names. Later Bevy-aligned milestones such as task pools or log settings should be implemented in their owning modules first and then added here once stable.

## Validation

`zircon_runtime/src/tests/prelude.rs` exercises the public surface by importing only `crate::prelude::*` and constructing runtime descriptors, lifecycle values, state contracts, time clocks, diagnostic stores, log filters, runtime profiles, the current job scheduler, and core descriptor modules.

Fresh M2 validation evidence from 2026-05-08:

- `cargo test -p zircon_runtime --lib prelude --locked --message-format short` passed: 3 prelude tests, 0 failed.
- `.\.opencode\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime` passed: Cargo build OK and Cargo test OK for the package.
- Runtime package validation also exercised the lower support regressions that blocked this milestone gate: direct `World` serde rehydration of runtime-only ECS identity/presence and first-party importer capability-status metadata.

Fresh M3 state update evidence from 2026-05-08:

- `rustfmt --edition 2021 --check <M3 state/prelude files>` passed.
- Runtime prelude Cargo checks are currently blocked by active asset-stack M3 `AssetImportOutcome` hard-cut migration errors before prelude tests can execute.
