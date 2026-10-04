---
related_code:
  - zircon_runtime/src/input
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/dynamic_api/session/events/keyboard_ime.rs
  - zircon_runtime/src/dynamic_api/session/events/gamepad.rs
related_plans:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
  - docs/plans/optimize/zircon_runtime/99r-runtime-input-device-event-frame-state-action-map-focus-gamepad-recording-replay-host-product-integration-current-source-review.md
write_scope:
  - zircon_runtime/src/input/runtime/default_input_manager.rs
  - zircon_runtime/src/input/runtime/input_state.rs
  - zircon_runtime/src/input/tests/input_manager/frame_state.rs
  - docs/plans/performance/pending.md
  - docs/plans/performance/01/2026-09-01-runtime-input-current-source-closure.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
  - docs/plans/optimize/zircon_runtime/99r-runtime-input-device-event-frame-state-action-map-focus-gamepad-recording-replay-host-product-integration-current-source-review.md
status: static_complete_dynamic_pending
---

# Runtime input current-source closure

This record closes the current static review of `zircon_runtime/src/input/**`.
The module remains in `pending.md`: the repository coordinator did not grant a
Cargo CPU lane, and no current product input trace or power sample exists.

## Scope and gates

- 40 Rust files, 6,714 physical lines by the repository raw-split convention,
  5,964 nonempty lines, 233,228 bytes, 81 tests, nine ignored release-only tests
  and 32 `include_str!`/`include_bytes!` sites.
- Workspace-relative `path + NUL + raw bytes + NUL`, sorted by path, SHA256:
  `d05aebaf94115bd811c0ba82ab490b7724776fed483fadf40d68aaeed65fc46d`.
- Isolated `rustfmt +1.94.1 --check --edition 2024 --config
  skip_children=true` passes 29/40. The three touched files pass; the other 11
  failures are current formatting drift and were not reformatted.
- `git diff --check -- zircon_runtime/src/input` passes with line-ending warnings
  only.
- Runtime12 input-stack audit: 1/1 pass. Dynamic event owner structure: 2/2
  pass. Empty-UI input M0 contracts: 3/3 pass.
- The first managed focused Cargo requests were rejected by
  `cargo_cpu_lane_reserved`, `cargo_reuse_target_mismatch` and
  `cargo_reuse_pool_busy`. The approved managed run then entered Cargo and failed
  in foreign-modified `zr_rhi_wgpu/src/render_pass_validation.rs:455` because
  `texture_view_layer_count` is missing. `zircon_runtime` input did not compile and
  no Rust test executed.

## Current behavior to preserve

1. Button level and edge state is explicit; focus loss and gamepad disconnect
   terminate held contributions.
2. Adjacent cursor positions use latest-value coalescing and adjacent raw mouse
   motion accumulates deltas while button, touch and focus edges remain barriers.
3. Gamepad axes and buttons implement deadzone, livezone, change threshold and
   hysteresis before frame publication.
4. Action-map changes compile action/binding/context indices once. Evaluation
   reuses frame-axis, consumed-input, context and action workspaces after warmup.
5. Host-effect queues survive frame boundaries until drained; frame snapshots
   project only the newly appended request range.
6. Recording is opt-in and its raw-event deque is count-bounded with discard
   accounting. App gamepad polling is separately bounded by 256 events or 2 ms.
7. Current keyboard, pointer, touch, wheel and gamepad handlers submit physical
   state before UI dispatch. The older blanket claim that UI consumption prevents
   those physical updates is closed. Text/IME remains an intentional UI-first
   routing policy and needs an explicit ownership receipt rather than reuse of the
   old defect statement.

## Accepted narrow optimization

`DefaultInputManager::submit_event(GamepadAxis)` previously searched the complete
frame transition vector for every accepted sample. For `E` changed samples and
`A` distinct active axes, this performed up to `O(E * A)` equality probes.

`InputState` now retains one frame-local
`HashMap<(GamepadId, GamepadAxis), usize>` alongside the ordered public transition
vector. The first sample appends and indexes a row; later samples update the row
by expected `O(1)` lookup. `begin_frame` clears both while retaining capacity.
Focus-loss and disconnect append paths index their new rows without changing the
existing public order or duplicate semantics. Resulting transition ownership is
`O(E)` expected lookup work plus the existing device-state map work and `O(A)`
extra retained indices.

The focused unit and manager tests fence first-observation order, first previous
value, latest value, interleaved axes, frame reset and next-frame previous state.
This local correction is not the target architecture and has no runtime timing
claim until the managed Rust gate executes.

## Retained structural findings

1. `InputDriver` remains an empty immediate service. `InputConfig` defaults to
   disabled while driver, physical manager and action manager still register
   immediately, so type availability is not product readiness.
2. The dynamic session resolves the manager and submits one event at a time.
   `DefaultInputManager` holds one broad mutex across physical state, transient
   vectors, host requests, event buffering and optional recording. A burst is not
   admitted or published as one qualified batch.
3. Internal physical facts still lack one window/device/user/seat/source/time/
   sequence generation. UI, actions, journal and host effects cannot prove they
   consumed the same event generation.
4. `frame_snapshot()` clones held/edge sets, touch/gamepad metadata and String/Vec
   payloads under the writer mutex. Reader frequency therefore scales copied bytes
   and lock hold time.
5. The frame event buffer has no entry/byte/age admission. It coalesces two adjacent
   motion classes, retains other events without a cap, and `begin_frame` discards
   undrained rows without a typed gap receipt.
6. No non-test product caller schedules `InputActionManager::evaluate_actions*`.
   Action results still rebuild owned ordered sets/maps and action ID Strings per
   call, while current sample/script gameplay remains a raw-control consumer.
7. `InputRecording.frames` is unbounded. Replay clones every event, ignores the
   recorded timestamps/sequence for scheduling, and can reproduce host-effect
   requests through the live manager.
8. Context priority is stored but does not compile one conflict/consume/block
   policy. Empty active contexts still mean all contexts, so it is not a complete
   per-player action program.

## Unreal evidence and boundary

- `ApplicationCore/Private/GenericPlatform/GenericPlatformInputPump.cpp:90-159`
  keeps device thread affinity explicit and fans polled input into consumer
  channels.
- `ApplicationCore/Private/GenericPlatform/AsyncInputConsumer.cpp:50-121` reuses
  consumer scratch, retains edges in order and coalesces absolute analog samples
  by `(axis, device)`.
- `ApplicationCore/Private/HAL/InputThread.cpp:81-104` separates input-thread
  waiting from actual poll work and trace ownership.
- Enhanced Input retains applied mapping contexts with explicit priority and
  rebuild policy, then evaluates the compiled player-owned program in
  `EnhancedPlayerInput`.

These sources support a qualified batch, semantic coalescing, reusable consumer
scratch and per-player compiled action generation. Unreal globals, UObject
lifetime and its exact input thread are not Zircon ABI requirements.

## Architecture hard cut

### M0: truthful readiness and qualified ingress

Replace the empty driver signal with an `InputIngressGeneration` carrying health,
device/window/user/seat, monotonic sequence/time, source epoch and teardown. Batch
admit count, bytes and age before manager mutation.

### M1: immutable physical publication

Apply one ingress batch to candidate physical state, preserve edge barriers and
publish one immutable `InputFrameGeneration`. Readers lease typed domains; they do
not deep-clone the complete manager state under the writer lock.

### M2: ownership arbitration

Publish UI/text/gameplay/host decisions against the same physical sequence.
Press, release, capture, focus and modal transitions terminalize exactly once.
Host effects use request/ack generations outside physical ingress.

### M3: compiled per-player action program

Compile stable control IDs, contexts, priority/conflict policy, triggers,
modifiers and typed values into one player generation. Evaluate once in the local
player/controller schedule and publish dense action slots rather than owned
Strings and ordered maps.

### M4: bounded journal and replay

Stream checked chunks with schema/build/map/device/clock identity, checksum,
completeness and backpressure. Replay preflights into an isolated target, preserves
the selected timing policy and does not issue real host effects by default.

### M5: observability and product gates

Diagnostics Disabled adds no recording or owned labels. Counters expose accepted,
coalesced, rejected and gapped rows plus lock wait/hold and publication age. WPR
then measures input storms, readers and replay on a current launchable product.

## Acceptance gates

- `1/1k/10k/100k` events/s and `1/4/16` producers are pre-admitted by entries,
  bytes and age; critical release/cancel edges never disappear.
- One qualified batch produces one immutable frame generation; `1/4/16/64`
  readers create zero full-state clones.
- Physical, UI, action and journal rows share exact device/window/player/sequence
  generations and cannot relabel stale work.
- Stable action generations rebuild zero and stable evaluation allocates zero;
  work scales with active compiled candidates.
- Recording/replay is bounded, complete, deterministic under its declared timing
  policy and host-effect safe.
- Every identity is checked and non-repeating; shutdown terminalizes ingress,
  effects, journal and readers.
- Current-source Cargo passes before WPR/ETW. RenderDoc is only a visible-response
  correlation aid for this CPU/input owner, not the input performance profiler.

## Disposition

The algorithmic transition-index fix is implemented but not dynamically accepted.
The module remains in `pending.md`; `review.md` is unchanged. Runtime12 owns the
input/action hard cut, Runtime10 owns dynamic ABI batching, Runtime07/11 own frame
and task scheduling, and Runtime09 owns UI routing without duplicating physical
state.
