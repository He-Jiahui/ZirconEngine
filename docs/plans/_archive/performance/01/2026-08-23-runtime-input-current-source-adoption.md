---
title: Runtime Input Current Source Adoption
date: 2026-08-23
scope:
  - zircon_runtime/src/input
status: static_complete_dynamic_pending
canonical_owner:
  - docs/plans/optimize/zircon_runtime/99r-runtime-input-device-event-frame-state-action-map-focus-gamepad-recording-replay-host-product-integration-current-source-review.md
references:
  - dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/EnhancedInput/Private/EnhancedInputSubsystemInterface.cpp
  - dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/EnhancedInput/Private/EnhancedPlayerInput.cpp
  - dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/EnhancedInput/Public/InputMappingContext.h
---

# Runtime Input Current Source Adoption

## 1. Coverage and currentness

The current `zircon_runtime/src/input` tree is **32/32 Rust files**, **4,658 physical / 4,154 non-empty lines**, **161,512 bytes**, and **55 test markers**. Its workspace-relative `path + NUL + raw bytes + NUL` SHA-256 is `3207d00b60dc881bb06de52c935dd90c3bf29ae76ac503e2e3a347ec0e91ccbd` at current HEAD `0e2bdaa9d3f6949e351ce4e77ccf1aca9e7032b1` plus the preserved shared-worktree edits listed below.

All module/config/descriptor, action-evaluator generation/index/workspace, default manager, frame event buffer, recorder/replay and focused test files were read directly. The count includes the test-only module extracted during validation. Runtime117 is adopted as the broader current-source owner because it also follows the contracts through Core framework input, dynamic session, App ingress, script host and sample product. It is currently an untracked shared-worktree report and must be preserved for coordinator adoption; this report does not claim or rewrite it.

Existing foreign edits in `runtime/default_input_manager.rs`, `runtime/input_state.rs`, `tests/input_manager/frame_state.rs` and `tests/input_manager/host_requests.rs` are preserved. They make host-effect requests survive a frame boundary for later drain while limiting a frame snapshot to newly appended requests. This is a valid bounded-copy reduction for those vectors, but it does not change the physical event buffer or the global snapshot/lock architecture.

## 2. Accepted foundations

- button level and just-pressed/just-released state is explicit, and gamepad disconnect removes held contributions;
- adjacent cursor/mouse-motion events coalesce while focus transitions remain barriers;
- gamepad axes/buttons have deadzone, livezone, change threshold and hysteresis;
- action-map mutation builds a reusable generation with action/binding/context indices;
- each evaluation builds a frame-axis index and consumed-input index once and reuses workspace storage;
- the raw script hot path can query `button_pressed` without cloning a complete `InputSnapshot`;
- App gamepad polling already has count/time budgets and continuation;
- recording has a bounded retained-record queue and exposes discarded-record count.

These are local algorithmic foundations. They do not prove product input readiness because production still has no action evaluation owner or recording/replay consumer.

## 3. Revalidated structural bottlenecks

1. The built-in module publishes an immediate zero-field `InputDriver`, manager and action manager even when input config defaults to disabled. Type registration can therefore be mistaken for a ready input capability.
2. Dynamic-session UI propagation can stop before physical state submission. Press and release may not reach the same state owner, which can leave permanent held state or manufacture a release edge after capture/focus changes.
3. Persisted action bindings use transient gamepad IDs and unstable keyboard derivation. Reconnect or engine-version changes can silently redirect a binding.
4. No production caller schedules `InputActionManager::evaluate_actions*`; sample and script gameplay use raw key strings. Action evaluator performance is therefore not yet product performance.
5. One manager mutex covers physical state, transient vectors, event buffer, recording and host-effect queues. Every event takes that lock; enabled recording also clones the event and calls `SystemTime::now` per record.
6. `frame_snapshot()` clones held/edge sets, touches, gamepad/device metadata, strings and event vectors while holding the same lock. The host-request delta reduced three vectors only. Reader count and snapshot frequency scale lock hold time and copied bytes.
7. The frame event vector has no count/byte/time budget. Only adjacent cursor/motion events coalesce; other events can grow until `begin_frame`, which then clears undrained physical events without a gap receipt.
8. `InputRecording.frames` is unbounded, frame capture drains global records, and replay clones each event and submits immediately while ignoring original time/sequence. It is a test helper, not a deterministic streaming journal.
9. Action contexts are not a compiled ownership policy: an empty active-context list means all, unknown contexts can become enabled, priority does not arbitrate conflicts, and contextless actions remain globally active.

The first three remain Runtime117 P0s. Items 4-9 are architectural P1 prerequisites; micro-optimizing individual set lookups before the owner/schedule/generation contract exists would benchmark a non-product path.

## 4. Unreal constraints

Unreal Enhanced Input supplies the primary comparable product boundary. `EnhancedInputSubsystemInterface.cpp` stores applied mapping-context data on the player input with explicit priority and registration tracking. Context add/remove, user-setting changes and profile changes request control-mapping rebuild instead of rebuilding every query. `EnhancedPlayerInput.cpp` then evaluates mappings, modifiers, triggers and injected inputs in the player input phase. The test suite covers context consumption, held-key rebuild policy and player-mappable profile behavior.

Zircon should adopt the per-player owner, prioritized compiled mapping generation, explicit rebuild/held policy and typed action lifecycle. It should not copy UObject/Blueprint structure or Unreal globals. Godot/Bevy remain secondary evidence for retaining device/window/physical/logical/repeat identity; they do not replace the Unreal-primary action architecture.

## 5. Dependency-ordered optimization plan

### M0: capability and physical truth

Delete the empty-driver readiness signal or connect it to a real ingress owner, clock, device registry, health and teardown. Submit every qualified physical fact before UI/gameplay ownership decisions. Add deterministic tests for consumed release, capture transfer and focus transition.

### M1: stable device/control wire contract

Carry window, device, user/seat, source generation, monotonic time and sequence through App, dynamic ABI and Core. Replace debug-string/hash and transient gamepad IDs with versioned physical/logical control and device/profile identity.

### M2: immutable physical-state publication

Separate ingress ownership from host effects and recording. Apply an event batch once, publish an immutable frame generation, and let UI/action/script readers lease it without cloning under the writer lock. Admission is bounded by event count, bytes and time and emits typed gap/backpressure receipts.

### M3: compiled per-player action program

Compile validated contexts, priorities, conflicts, triggers/modifiers, typed values and bindings into one per-player generation. Local Player/Controller schedule evaluates it once per frame; UI ownership supplies decisions for the same input sequence. Cut sample/script shipping paths from raw key strings to typed action handles.

### M4: bounded journal and replay

Write ingress and ownership decisions to chunked bounded artifacts with schema/build/map/device/clock identity, checksums and completeness. Replay preflights the artifact, resets an isolated target, preserves order/time policy and cannot issue OS host effects by default.

### M5: product qualification

Qualify runtime, editor viewport, multiple windows/players/devices, reconnect, layout/IME, focus/capture, recording/replay and shutdown through the same generations. Only after correctness gates pass should snapshot, action and ingress costs be compared with reference engines.

## 6. Quantified acceptance

1. Ingress: keyboard/mouse/touch/gamepad at `1/1k/10k/100k` events/s, `1/4/16` producers and idle/burst/held patterns. Record submit p50/p95/p99/max, lock wait/hold, batches, coalesces, retained bytes/events, gap receipts, writer CPU and energy.
2. Publication: `1/4/16/64` readers at `30/60/120/240 Hz` over small and large device/touch sets. Require one state application/publication per generation and zero full-state clones per reader; record lease latency, copied bytes and peak RSS.
3. Actions: `1/64/1k` actions, `1/8/64` contexts, `1/4/16` local players and conflict/trigger/chord cases. Record compile and evaluate p50/p95/p99, visited bindings/axes, workspace growth and allocations. Work must scale with active compiled candidates, not every stored mapping for every query.
4. Fault/correctness: UI-consumed press/release, capture/focus/modal transfer, device disconnect/reconnect, overflow, mapping rebuild while held, corrupted/incomplete replay and shutdown. No missing terminal edge, stale generation or real host effect during default replay.
5. WPR/ETW on a launchable current-source Windows product captures CPU samples, locks/waits, context switches, allocations, RSS and energy for idle, event storm, multi-reader and replay. Input latency requires source monotonic timestamp to published/action generation; frame time alone is insufficient.
6. RenderDoc is not an input profiler. It may verify visible UI/gameplay response and frame identity only after WPR/input receipts identify the corresponding generation.

## 7. Current result

- Static review is complete for **32/32** Rust files under `zircon_runtime/src/input`, with Runtime117 adopted for the cross-product chain.
- The four existing host-request edits are correctly scoped partial progress and remain foreign changes.
- A deterministic maintenance fix moved the unchanged inline `DefaultInputManager` tests to a test-only child module, reducing the production owner below its 500-line audit limit. The input audit now classifies both `/tests/` and `tests.rs` as test-only, and stale assertions were synchronized from 18 to 20 runtime modules and 21 to 25 behavior anchors. This changes no runtime behavior.
- No additional production behavior edit is safe before the M0 ownership/order contract.
- Static verification passed: 32-file `rustfmt --check`, the Runtime12 input-stack audit and three empty-UI input M0 contract tests. The audit reports no missing, unexpected or oversized modules and `risks=[]`.
- Rust/Cargo, real-window/device, WPR/ETW, replay soak and product benchmarks were not run yet. The module remains dynamically pending and is not eligible for milestone commit or WeCom acceptance notification.
