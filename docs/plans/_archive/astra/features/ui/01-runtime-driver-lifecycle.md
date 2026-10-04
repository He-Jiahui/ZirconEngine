# W3 UI Runtime Driver Lifecycle and Product Admission

Status: stage 1 integrated_pending_validation; full surface
ownership finding remains open.

## Current Evidence

The current `ui/module.rs` registers a configured `UiRuntimeDriver` whose factory
consumes `UiConfig.enabled`. `dynamic_api/session/project.rs` resolves that driver
and calls `admit_project` before UI asset, font, and layout admission. Explicitly
disabled UI therefore returns an empty surface set before project UI resources
are opened; missing and closed drivers return the typed `AdmitRuntimeUi` failure.
Focused project admission tests cover the enabled, disabled, absent, empty-root,
and post-cleanup cases. No production `UiService` contract exists to migrate or
wrap.

`text/module.rs` is the local lifecycle precedent: per-core service construction,
module cleanup closing retained handles, and a replacement service on activation.
Unreal Slate's `Framework/Application/SlateApplication.h` centralizes application
shutdown and window destruction; this supports stage 2's central owner, without
requiring a broad surface mutex or changing UI thread affinity in stage 1.

## Stage 1: Config and Lifecycle Admission

- `UiRuntimeDriver` owns its immutable activation config and admission-open state.
  `UiEventManager` continues to own route/subscription data. No forwarding service
  facade and no second surface registry are introduced.
- The stable `UI_CONFIG_KEY` is consumed once by the driver factory. Default UI is
  enabled, matching current product behavior; explicit false skips project UI
  admission. Invalid stored config fails activation, without silently enabling UI.
  Config changes apply on reactivation, not halfway through an active session.
- The actual project loader resolves the driver before UI asset/font/layout work
  and retains the existing core `ServiceCallGuard` through loading. Module drain
  therefore accounts for admitted loading without introducing another lock.
  No declared roots is a no-work path requiring no UI module. Disabled admission
  returns an empty surface set; missing/closed service is a typed `CoreError`
  carried by `RuntimeProjectError::AdmitRuntimeUi`.
- Module cleanup closes future admissions through retained driver handles;
  reactivation creates a distinct, open driver. An admission that linearizes before
  close is admitted and its loading guard participates in core drain. This stage
  does not revoke ongoing surface use after loading returns and does not claim
  ownership of existing surface teardown.
- Edit owners: `ui/module.rs` and its lifecycle/driver children, curated UI exports,
  prelude default assertion, dynamic project loader/error and focused tests.
  Existing `runtime_ui.rs` and foreign input children remain outside this slice.

## Stage 2: Canonical Runtime Surface Owner

After coordination with the dynamic input owner, implement the existing detailed
Runtime UI 09 architecture and optimize 11A/200 contracts: driver-owned session and
surface identities, window mapping, activation/publication generations, ordered
frame publication and retained resource teardown. Migrate every dynamic project,
render, accessibility, input, IME and host-request consumer to that single owner.
Preserve current thread affinity and staged publication; choose synchronization
from actual call-site requirements, not an unconditional surface-set mutex.
The event manager remains the event store rather than a competing surface owner.
Device lifetime, text/IME and accessibility acceptance follow their canonical
plans. Stage 1 acceptance cannot close these remaining capabilities or report W3
or optimize runtime200 complete.

## Batched Acceptance

- Driver: default/explicit disabled/invalid config, same-core service identity,
  cross-core isolation, config snapshot behavior, retained-handle close and fresh
  reactivation. Exercise real module activation and cleanup.
- Product loader: nonempty roots with disabled UI skip even unavailable assets;
  enabled UI reaches asset admission; missing UI service returns typed failure;
  empty roots work without a UI module. Existing real project UI integration tests
  continue to prove enabled render/input behavior.
- Run scoped formatting/source checks while implementing. Parent owns one managed
  Windows package validation batch for `zircon_runtime`; no per-test Cargo runs.
  Record hashes/test IDs for the immutable batch. Windows full-domain acceptance
  remains the target; other platforms require separate evidence.
- No frame hotpath, radius policy, hit-grid implementation or performance baseline
  changes are included. Frozen UI-A3 evidence stays independent.

## Implementation Handoff

UI module/config/lifecycle, project-loader admission, typed error propagation, and
their focused tests are integrated in the shared checkout. Driver tests cover
lifecycle and config; `project/ui_admission_tests.rs` covers disabled, enabled,
missing-service, empty-root and closed-module admission. Scoped rustfmt and diff
checks passed for the integration slice; the coordinator-managed Cargo and runtime
acceptance batch remains pending. Existing project UI integration gates remain
mandatory.

The App focus handler cancels an admitted IME composition before dispatching
`Background`. Its actual dispatch loop stops on the first rejected runtime event;
terminal session failure cannot trigger a second lifecycle call. The regression
`focused_window_handler_stops_dispatch_after_runtime_rejects_ime_cancel` exercises
both accepted and rejected cancellation through that loop. The existing
`focused_window_ime_handler_orders_cancel_and_rejects_the_stale_composition_cycle`
continues to cover late text events, focus restoration, and ABI payload contents.
These source regressions await managed Windows execution.
