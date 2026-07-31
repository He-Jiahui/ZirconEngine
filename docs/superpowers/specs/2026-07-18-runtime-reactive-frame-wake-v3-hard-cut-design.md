# Runtime Reactive Frame Wake V3 Hard-Cut Design

**Date:** 2026-07-18

**Status:** Approved for implementation

**Owners:** Runtime03 schedule/frame loop, Runtime10 dynamic ABI/lifecycle, Runtime11 task model

## Context

The Runtime03 MVP cadence slice now has a tested profile state machine: Game, Continuous, and Mobile poll; DesktopApp is reactive; Headless owns a stable fixed deadline. The static implementation suppresses idle Desktop ticks, host-request drains, and redraw requests, while preserving the admitted tick -> host request -> redraw order.

That slice cannot be accepted yet. `ApplicationHandler::proxy_wake_up` is only a consumer. The runtime cannot currently express that a long animation or timer needs another frame, and a worker that completes frame-visible work cannot wake a sleeping winit event loop. Restoring `Poll` or unconditional redraw would recreate the idle CPU failure.

The current `ZrRuntimeApiV2` is frozen. Runtime10 explicitly requires a new table version for field, signature, order, type, or meaning changes and forbids silent same-version tail extension. The remaining producer therefore requires a coordinated V3-only hard cut, not a V2 compatibility addition.

## Goals

1. Let each successful runtime tick return one ABI-safe next-frame demand: idle, immediate, or after a bounded delay.
2. Let only background work that can change frame-visible state wake the host event loop after completion.
3. Make the wake registration session-scoped and make `destroy_session` a quiescence barrier.
4. Keep winit and Rust-owned callback objects out of the runtime/interface ABI.
5. Preserve idle Desktop behavior, continuous Game behavior, and stable Headless deadlines.
6. Complete focused Cargo, product wake, shutdown-race, and WPR acceptance before returning the Runtime03 failure as fixed.

## Non-goals

- No V2 fallback, forwarding export, optional old loader, or same-version tail field.
- No `EventLoopProxy`, `Arc`, trait object, closure, or unowned userdata pointer across the DLL boundary.
- No use of diagnostics, plugin events, or drain-only host requests as a wake transport.
- No wake on every task completion. Invisible compute work must not manufacture frames.
- No global polling timer that hides missing producer ownership.

## Design

### 1. V3-only runtime table

Runtime10 introduces `ZrRuntimeApiV3` and `zircon_runtime_get_api_v3`, migrates the app loader and runtime exports atomically, and hard-deletes the V2 symbol/table/loader path. Stable V1 DTO names remain only where their carrier and semantics are unchanged; that is not a compatibility path.

Session creation uses `ZrRuntimeSessionConfigV2`. It includes a session-scoped `ZrRuntimeWakeSinkV1`:

```rust
#[repr(C)]
pub struct ZrRuntimeWakeSinkV1 {
    pub abi_version: u32,
    pub token: u64,
    pub wake: Option<unsafe extern "C" fn(u64)>,
}
```

The token is an opaque host registry key. `zircon_app` owns the matching `EventLoopProxy`; the runtime never interprets the token and never receives a Rust object pointer. A sink is valid only when its ABI version is supported and token/callback are either both present (`token != 0`, `wake = Some`) or both absent (`token == 0`, `wake = None`).

The V3 tick function writes `ZrRuntimeFrameDemandV1` to a caller-owned output:

```rust
pub const ZR_RUNTIME_FRAME_DEMAND_IDLE_V1: u32 = 0;
pub const ZR_RUNTIME_FRAME_DEMAND_IMMEDIATE_V1: u32 = 1;
pub const ZR_RUNTIME_FRAME_DEMAND_AFTER_V1: u32 = 2;

#[repr(C)]
pub struct ZrRuntimeFrameDemandV1 {
    pub abi_version: u32,
    pub kind: u32,
    pub delay_nanoseconds: u64,
}
```

The ABI carrier keeps `kind` as raw `u32`; a Rust enum is used only after checked conversion inside each crate. `Idle` sets delay to zero. `Immediate` requests one coalesced reactive frame. `After` carries a finite relative delay. Unknown values are rejected before constructing the internal enum. The runtime clamps producer delays to a named maximum before conversion, and the host uses checked `Instant` arithmetic; overflow becomes the named maximum rather than wrapping or panicking.

### 2. Session wake lifetime

The runtime converts the ABI sink into an internal `RuntimeWakeRegistration`. Its state is shared only by producers belonging to that session and contains:

- an enabled/closing state;
- an in-flight callback count;
- a condition variable for shutdown quiescence;
- the copied function pointer and opaque token.

A wake attempt enters only while enabled, increments the in-flight count, invokes the callback, then decrements and notifies. The `zircon_app` callback trampoline itself wraps all Rust work in `catch_unwind` and always returns without unwinding; this host-side containment is the ABI safety proof. Runtime-side containment may remain as defense in depth but cannot make a panicking `extern "C"` callback safe after unwind has already crossed the boundary.

Every ABI session call first acquires a `SessionSlot` action guard. The slot owns one open/closing state, an active-call count, the session value, and a condition variable. Acquiring while open increments the count; dropping the guard decrements it outside the session lock and notifies. Acquiring after closing starts returns the canonical missing/closing-session status.

`destroy_session` performs this order:

1. mark the registry slot closing so new calls cannot acquire an action guard;
2. disable new wake entries;
3. wait until both active ABI calls and in-flight wake callbacks reach zero;
4. take and drop the session value, then remove the registry slot;
5. return success to the host.

Only after successful destroy may `zircon_app` remove the token-to-proxy registration. A call that raced before closing completes before destroy returns; a late call cannot clone a session out of the slot. A late worker observes the disabled wake registration and becomes a no-op; it cannot dereference host memory.

### 3. Runtime11 terminal observer

Runtime11 adds a general `JobHandle::on_terminal` observer. It is not a winit or dynamic-API feature.

- registration is race-safe before or after terminal state;
- each observer runs exactly once;
- observer panic is contained and recorded;
- observers run outside the job-state lock;
- dependency continuations retain their existing ordering and non-blocking semantics;
- no scheduler-wide automatic wake is added.

The specific subsystem that owns frame-visible asynchronous work attaches an observer that calls the session wake registration. Invisible jobs attach no observer.

### 4. Runtime03 demand aggregation

Runtime03 owns a per-frame demand accumulator inside the dynamic session. Producers merge demands with these rules:

- `Immediate` dominates every delayed or idle demand;
- two delayed demands produced during the same tick retain the shortest finite delay;
- idle contributes nothing;
- consuming the accumulator resets it to idle for the next tick.

Animation scanning already visits each player. It records `Immediate` when at least one enabled player remains active, avoiding a second full-world scan. Runtime UI timers contribute their earliest due delay. Frame-visible async owners attach the Runtime11 terminal observer and wake the host when their result becomes consumable.

### 5. Host cadence integration

`EntryRunner` creates the event loop and proxy registry token before creating the runtime session. `RuntimeEntryApp` consumes the tick demand after an admitted frame:

- Desktop `Idle`: `ControlFlow::Wait` and no pending frame;
- Desktop `Immediate`: coalesce one frame request and call `EventLoopProxy::wake_up` through the host-owned registry;
- Desktop `After`: replace the previous runtime-owned deadline with the new tick snapshot and use `WaitUntil`;
- Game/Continuous/Mobile: keep `Poll`; demand does not reduce continuous cadence;
- Headless: keep its existing fixed interval; an earlier explicit deadline may be merged only when the owning product contract requires it.

OS/device/resume events continue to request a reactive frame. `RedrawRequested` remains excluded from self-scheduling.

The host does not merge a new delayed snapshot with an older runtime deadline: timer cancellation or postponement must be able to remove that deadline. Earliest-deadline merging is limited to producers inside one runtime tick. Independent OS/resume requests remain separate immediate admission signals.

## Rejected Alternatives

- **Extend V2 in place:** violates the frozen-table guard and hard-cut policy.
- **Put the callback in global `ZrHostApiV1`:** not session-scoped and cannot prove destroy lifetime.
- **Pass proxy/userdata pointers across the DLL:** exposes Rust layout and lifetime to the ABI.
- **Use diagnostics/plugin events/host requests:** diagnostics are observational, plugin events belong to plugin subscriptions, and drain-only requests cannot wake an already sleeping loop.
- **Wake for every scheduler completion:** invisible work would create unnecessary frames and Runtime11 would own application policy.
- **Return to Poll or a permanent fixed timer:** recreates the original Desktop idle CPU defect.

## Validation

1. Interface layout/version and V2-retirement guards.
2. Unknown, closing, destroyed, double-destroy, and concurrent destroy/wake tests.
3. Terminal-observer before/after completion, exactly-once, panic, dependency, and lock-reentrancy tests.
4. Runtime demand merge tests for idle/immediate/earliest delay.
5. Active animation continuation, timer deadline, visible task completion, coalescing, and redraw no-feedback product tests.
6. Coordinator-managed Windows checks for `zircon_runtime_interface`, `zircon_runtime`, and `zircon_app`.
7. Desktop 30-second idle WPR trace plus continuous Game comparison. The budget is fixed before capture in `docs/plans/zircon_runtime/runtime/03/2026-07-18-desktop-idle-cadence-wpr-budget.md`: after a 5-second warmup, idle frame-pump/host-drain/redraw counter deltas are zero, runtime process sampled CPU is at most 1.0% of one logical core, event-loop wakeups are at most 2 per second, and the continuous Game comparison loses no more than 2% median frame throughput. Capture uses `wpr -start GeneralProfile -filemode`, a 30-second measured window, and `wpr -stop <etl>`; raw ETL and parsed counter artifacts are retained.

## Current Status

| Area | Status | Evidence |
|---|---|---|
| Runtime03 profile cadence state machine | `implemented_static_green` | schedule audit 3/3, rustfmt, scoped diff check, independent review C0/M0 |
| Headless stable deadline | `implemented_static_green` | early wake does not pump or shift deadline regression |
| Runtime-origin producer | `foundation_static_producers_pending` | session-scoped demand accumulator and wake registration exist; animation/timer/visible-task producers remain open |
| V3 ABI and destroy barrier | `implemented_static_pending_managed_validation` | V3-only interface/export/app/editor callers, session action/callback quiescence, production V2 hits 0, independent lifecycle/app reviews C0/I0/M0 |
| terminal observer | `implemented_static_pending_managed_validation_and_commit` | exactly-once/panic/reentrant/dependency tests, behavior inventory 26, independent review C0/I0/M0 |
| managed Cargo/product/WPR | `pending` | no acceptance claim |
