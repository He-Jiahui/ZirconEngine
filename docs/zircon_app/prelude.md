---
related_code:
  - zircon_app/src/prelude.rs
  - zircon_app/src/lib.rs
  - zircon_app/src/entry/mod.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/entry_runner/bootstrap.rs
  - zircon_app/src/plugins/mod.rs
  - zircon_runtime/src/prelude.rs
  - zircon_runtime/src/core/runtime/state_machine/mod.rs
implementation_files:
  - zircon_app/src/prelude.rs
  - zircon_app/src/lib.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/entry_runner/bootstrap.rs
plan_sources:
  - user: 2026-05-08 continue ZirconEngine Bevy completion roadmap M2 stable prelude
  - user: 2026-05-08 continue ZirconEngine Bevy completion roadmap M3 State
  - user: 2026-05-16 continue Bevy-style profile/module group diagnostics completion
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
tests:
  - zircon_app/src/tests/prelude.rs
  - zircon_app/src/entry/tests/builtin_engine_entry.rs
  - zircon_runtime/src/tests/state.rs
  - zircon_app/src/tests/mod.rs
  - .github/workflows/ci.yml
doc_type: module-detail
---

# Zircon App Prelude

`zircon_app::prelude` is the app-host convenience import surface for entry runners, entry profiles, and Bevy-style plugin group composition. It layers the app-owned host API on top of `zircon_runtime::prelude` so examples and package-level tests can import one stable surface without bypassing the runtime ownership model.

## Ownership Boundary

- `zircon_app` owns process entry selection, `EntryConfig`, `EntryProfile`, `ProductCompositionRequest`, `ProductComposition`, `EntryRunner`, and plugin group composition. The lower `BuiltinEngineEntry` compiler is crate-private so callers cannot detach a cloneable Core handle from its plugin owners.
- `zircon_runtime` owns lifecycle, module descriptors, runtime profiles, state contracts, diagnostics, time contracts, and concrete runtime modules.
- `zircon_app::prelude` re-exports `zircon_runtime::prelude::*` but does not move runtime ownership into the app crate.

This mirrors the Bevy convention of an application-level prelude while preserving Zircon's fixed package roles: app hosts and composes, runtime owns lifecycle and services, editor remains a separate authoring host.

## Export Groups

The app prelude exports:

- entry types: `EntryConfig`, `ResolvedProductHostConfig`, `EntryProfile`, `EntryRunMode`, `EntryRunner`, `EntryModuleSelection`, `EntryModuleSelectionReport`, `ProductCompositionRequest`, `ProductComposition`, `ProductExitClass`, `ProductProcessExitCode`, and product config/provenance types,
- plugin group types: `PluginGroup`, `PluginGroupBuilder`, `PluginGroupError`, `ResolvedPluginGroup`, `MinimalPlugins`, `DefaultPlugins`, `DevPlugins`, and `HeadlessPlugins`,
- all stable runtime prelude exports through `zircon_runtime::prelude::*`, including M3 state contracts owned by `zircon_runtime::core::framework::state`.

## Non-Goals

The app prelude does not re-export editor internals, runtime presenter details, window handler internals, or native plugin loader implementation details. Those stay in their owner modules until a specific public app-host API is designed.

## Validation

`zircon_app/src/tests/prelude.rs` imports only `crate::prelude::*` and verifies that entry construction, the unified product composition request, module selection diagnostics, product config provenance, platform diagnostics, built-in plugin groups, custom plugin group construction, runtime state contracts, and runtime prelude foundations are available through the app surface.

Fresh M2 validation evidence from 2026-05-08:

- `cargo test -p zircon_app --lib prelude --locked --message-format short` passed: 2 prelude tests, 0 failed.
- `.\.opencode\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_app` passed: Cargo build OK and Cargo test OK for the package.

Fresh M3 state update evidence from 2026-05-08:

- App prelude source and test imports were updated to include runtime-owned state contracts through `zircon_runtime::prelude::*`.
- `cargo check -p zircon_app --lib --locked --message-format short` is currently blocked by active `zircon_runtime::asset::importer` migration errors before app prelude validation can execute.

The 2026-08-27 composition hard cut replaced the detachable native bootstrap result and provider-specific runner permutations with `ProductCompositionRequest` and `ProductComposition`. `EngineEntry`, `BuiltinEngineEntry`, and the composition's Core borrow are crate-private; the prelude cannot be used to clone Core away from the owners retained by the product composition. The same slice exposes a typed process-exit policy: semantic host failures converge to portable code `1`, success is `0`, and an explicit command result preserves any nonzero `u8` code. Rust 1.94 formatting and scoped static checks passed; managed Cargo validation for the new public surface remains pending.
