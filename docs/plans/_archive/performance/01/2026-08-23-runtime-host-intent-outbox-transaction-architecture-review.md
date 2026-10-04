---
title: Runtime host intent outbox transaction performance review
date: 2026-08-23
module: runtime input, runtime UI, dynamic host-request ABI, and app host apply
priority: MVP-P0 runtime input and basic editor/game host behavior
status: source_reviewed_m0_m1_m1a_applied_static_validated_dynamic_pending
reference_engine: Unreal Engine Slate user and text input method contexts
---

# Goal

Converge IME, cursor, and rumble intent onto one loss-aware runtime-to-host transaction. First stop
the two current silent-loss paths without changing frame-local input visibility. Then replace the
three-lock/four-vector JSON path with a versioned bounded batch whose edge/state/command classes,
continuation wake, and main-thread apply budget are explicit. No timing claim is accepted until the
current-source executable can run under the managed Windows validator and WPR.

## Current-source review fingerprint

The host-request owner was re-read from producer through platform application at repository HEAD
`41957dde31b8a5e7b440ddb54bd698abc7755f65`:

| owner | reviewed | lines | bytes | joined SHA256 manifest |
| --- | ---: | ---: | ---: | --- |
| core input outbox (`InputManager`, snapshot, default manager, state) | 4/4 | 780 | 30,837 | `c8ad9128678effb50e28cf3c2640337baed7e6e917384836bba28a8c042d630d` |
| runtime publication (`session`, runtime UI and UI input owner) | 6/6 | 2,416 | 88,427 | `d9d5ea5ab6ce56e3a4cef5acb3f32a26b5d3682201daaa32da44925ab265d36a` |
| App host apply folder | 10/10 | 414 | 14,667 | `32a538ae02a7e0a90e404a59816ed30604202a0e73a4aabc68dd61a2cb1d01a8` |
| interface payload and limit contract | 2/2 | 700 | 21,224 | `f3feabdf6943155b0bd26126ec40c5c6a58ec232802d77df8c19a1e9fde6324c` |

Post-M1 fingerprints for the changed production owner groups are:

| owner | files | lines | bytes | joined SHA256 manifest |
| --- | ---: | ---: | ---: | --- |
| core input outbox | 4 | 796 | 31,738 | `aa99f31b973a909b80ffe38822ba6e81d1e16cef0596bfa74c3c26c5c4d82dfa` |
| runtime publication | 6 | 2,492 | 91,500 | `8c9833eaf09cdd2d87e1aee594d6ffb420b3665623afb831181bd63a5337a5a2` |
| App host apply plus rumble lifecycle (post-M1a) | 11 | 715 | 25,225 | `cced88a3b3c237f49bf680dd3f25e78eafad3e3ef2a45d301d1ab3210e6152e4` |

All files in those owner sets were read in full. The review also traced `frame_loop.rs`, the
relevant `RuntimeSession` FFI decode/release path, dynamic host-request tests, input-manager tests,
runtime event routing, frame cadence/redraw policy, transactional output prepare/commit/rollback,
and the existing Runtime10 failure handoff. `session/ffi.rs` and
`default_input_manager.rs` already contain unrelated formatting changes; this review does not
overwrite or revert them.

## Structural findings

### P0: the product frame order silently clears the core outbox

The real App order is `handle event -> request frame -> tick_frame -> drain_host_requests`. Runtime
`tick_frame` calls `level.tick`, operation apply, then `InputManager::begin_frame`. The default
manager clears `ime_host_requests`, `cursor_host_requests`, and `gamepad_rumble_requests` in that
final call. Therefore requests submitted before the tick or generated during it are destroyed
before App drain.

The dynamic test only covers `handle_event -> drain`, so it passes while the product path fails.
Moving the entire `begin_frame` call to the start is not valid: it would change when button
transitions, wheel deltas, IME commits, window status, and drag/drop events are visible. The M0
boundary is an outbox watermark: `begin_frame` advances only the frame-snapshot start, while the
pending request storage remains owned by exactly-once drain.

### P0: production runtime UI has a disconnected second IME outbox

Every `RuntimeUiSurface` owns a `UiInputManager`. Dispatch replies append enable/disable, cursor
area, and surrounding-text requests to that manager. Production `RuntimeUiSurfaceSet` never drains
them; all existing calls to `UiInputManager::drain_ime_host_requests` outside its definition are
tests. The dynamic session drains only the separate core `InputManager`, so actual retained runtime
UI IME intent cannot reach the host ABI.

M0 must publish every surface outbox into the same transactional pending batch before encoding.
It must preserve surface order and must not add an App-side private cache.

### P0: bounded pages have no continuation contract

Current source has improved since the July report: one output is limited to 256 requests, 256 KiB,
and 10 ms, with prepare/commit/rollback and a two-page unit test. The product App drains exactly one
page per frame pump. `frame_demand()` ignores `pending_host_request_output`, and `RedrawRequested`
explicitly does not schedule a runtime frame. An idle runtime can therefore strand the remaining
page indefinitely. The batch also has no `remaining` or `has_more` field.

Runtime10 must add an ABI-visible continuation receipt or equivalent direct wake. Continuation may
schedule host-request work but must not force a render/present when no surface is damaged.

The full current-source continuation chain was re-read after M1a. V1 contains only `abi_version`
and `requests`; runtime commit retains rows after the encoded prefix, but App decode discards the
batch wrapper and returns only `Vec<ZrRuntimeHostRequestV1>`. Runtime `frame_demand` observes scene
asset reload and animation only. App drains only inside the successful tick branch, and every such
pump then calls `window.request_redraw`. Therefore neither of the tempting leaf fixes is accepted:
mapping pending rows to `RuntimeFrameDemand::Immediate` adds runtime ticks and redraw/present work,
while draining every page in one call removes the existing count/encode-time bound from the event
loop thread.

The M3/M4 boundary needs a receipt such as `remaining_rows`/`has_more`, preserved by
`RuntimeSession::drain_host_requests`, plus a separate coalesced App host-work request. An
`about_to_wait` slice may then drain/apply bounded pages without ticking simulation or requesting a
redraw; only actual runtime frame demand or render damage may enter the frame cadence. Acceptance
must prove a 257-row idle queue reaches zero while tick and redraw counts do not increase with page
count.

### P1: normal publication performs three manager transactions and four collections

Core collection locks the same input state three times, performs three `mem::take`s, maps the three
vectors, and collects a fourth ABI vector. A runtime UI bridge would otherwise add another
intermediate vector. The hard-cutover target is one typed `HostIntentOutbox::drain_batch` manager
transaction and one final owned page.

The current oversized-page search also clones `pending.requests[..count]` before every JSON encode.
The normal case is one O(N) clone plus one O(encoded bytes) encode; an encoded-byte overflow can
repeat prefix clones and JSON work O(log N) times. M1 can remove prefix clones with a borrowed
serialization envelope, but JSON encode + owned ABI + JSON preflight/decode remains until a
versioned typed/binary ABI replaces V1.

### P1: state updates are not coalesced before event-loop OS calls

Cursor position/visibility/grab/hit-test, IME cursor area, and surrounding text are latest-value
state. Enable/disable and rumble stop contain ordering/lifecycle edges. Current code serializes and
applies every row on the event-loop thread. Runtime12 must classify intent before coalescing; a
blanket `dedup` would incorrectly discard lossless transitions.

### P1: App repeats a full rumble-expiry scan for every host-request row

With `gamepad-gilrs`, `apply_runtime_host_request` calls
`clear_finished_rumble_effects` before matching every IME, cursor, or rumble row. A rumble row then
calls it a second time inside `apply_runtime_gamepad_rumble_request`. The helper takes one `Instant`
and retains every active effect in every gamepad bucket, so a batch containing `N` total rows and
`R` rumble rows performs `(N + R)` full expiry scans. This is a batch invariant, not row-specific
work; unrelated IME/cursor volume can therefore amplify gamepad cleanup.

The M1a hard cut is one expiry scan after a successful non-empty batch drain and before the first
row is routed, then zero scans in row routing and the rumble leaf. Empty batches remain zero, and
the independent gamepad-poll cleanup remains unchanged. This changes expiry collection from
`O((N + R) * E)` to `O(E)` for `E` retained effects and gives every row in one host transaction the
same cleanup-time snapshot. Dynamic time and energy deltas remain unclaimed until the managed
current-source executable is available.

## Unreal source basis

Direct source under `dev/UnrealEngine` establishes the ownership pattern, not Zircon-specific
budgets:

- `SlateUser.cpp:402-455` retains cursor visibility and position in one `FSlateUser` owner and
  publishes position directly to the platform cursor.
- `SlateUser.cpp:808-1018` queries only when capture is active or `bQueryCursorRequested`, then
  applies one final `FCursorReply`; `FinishFrame` clears a temporary focus path, not platform cursor
  intent.
- `SlateApplication.cpp:3640-3720` processes capture, high-precision mode, requested position, and
  lock through that same user owner rather than independent frame-local vectors.
- `SlateEditableTextLayout.cpp:175-176,248-263,617-646` retains one text-input context, explicitly
  registers/activates/deactivates it with the platform system, and guards teardown. It does not
  leave a per-widget request vector disconnected from the application owner.
- `WindowsTextInputMethodSystem.cpp:1032-1157` routes composition through one active context and
  preserves selection/composition lifecycle ordering.
- `PlayerController.cpp:4716-4788` performs one `ProcessForceFeedbackAndHaptics` pass per frame:
  it traverses `ActiveForceFeedbackEffects` once, removes completed entries in that traversal, then
  updates dynamic feedback and the manager. It does not rescan expiry for each unrelated input or
  platform command.

The transferable rule is one retained typed owner with explicit publication and lifecycle. Zircon
still needs its dynamic-library ABI and winit adapter, but frame reset must not be a second outbox
consumer and every runtime UI surface must publish through the same transaction.

## Target architecture and milestones

| milestone | work | gate |
| --- | --- | --- |
| M0 | Add product-order behavior tests; keep pending core requests across `begin_frame` using frame watermarks; bridge runtime UI IME requests into transactional collection. | no request loss for `produce -> tick -> drain`; frame snapshots remain frame-local; second drain empty |
| M1 | Serialize a borrowed page envelope; add source counters for producer rows, manager transactions, prefix encode attempts, encoded bytes, pages, pending/oldest age. | prefix request clones `N -> 0`; normal encode attempts `1`; no behavior/ABI drift |
| M1a | Hoist rumble-expiry collection from per-row routing and the rumble leaf to one successful non-empty App batch pre-pass. | host-batch expiry scans `(N + R) -> 1`, empty `0 -> 0`; gamepad-poll lifecycle unchanged |
| M2 | Introduce one typed outbox drain transaction; classify lossless edges, latest-value state, and bounded commands; remove three independent manager drains after hard cutover. | manager lock transactions `3 -> 1`; final-state parity and edge-order matrix |
| M3 | Add versioned typed/binary batch plus continuation receipt/wake; retain V1 only at the external compatibility boundary until migration completes. | new path JSON encode/decode `1+1 -> 0`; pending pages cannot strand at idle |
| M4 | Bound App apply by count/time without forcing redraw and validate current-source executable. | 1/1K/10K queue/age/OS-call data, WPR CPU/context-switch/energy evidence, no idle present regression |

Do not add an unbounded cache, silently drop IME or rumble edges, wake by unconditional redraw, or
claim RenderDoc as a CPU profiler. RenderDoc is needed only if continuation or batching changes the
render/present path and pixel/draw parity must be proven.

## Acceptance matrix

Exercise `1/256/257/1K/10K` requests, `1/4` runtime UI surfaces, idle and continuous runtimes,
focused/unfocused windows, and repeated state mixed with enable/disable and rumble stop. Record
produced/published/applied/coalesced/dropped rows, page count, remaining/oldest age, encoded/copied
bytes, allocation count/bytes, manager lock transactions/wait, encode/decode p50/p95/p99, main
thread apply and OS-call counts, continuation wakes, render/present counts, CPU/context switches,
and process energy estimate.

All Cargo targets, ETW/WPR traces, allocator logs, power reports, executables, and optional RenderDoc
captures must be on D:, E:, or F:. Current managed Cargo cannot execute because this Session owner
is archived; no Rust behavior or timing result is claimed from static inspection.

## Validation state

- Current source and Unreal ownership evidence: reviewed and recorded before implementation.
- M0 is applied. Three request vectors now retain one pending storage plus a frame-snapshot
  watermark. `begin_frame` clears zero host outboxes (`3 -> 0`) and advances three watermarks;
  exactly-once drains reset them. Focus-loss replacement resets IME/cursor watermarks explicitly.
  Runtime UI surface IME publication sites now have one production drain bridge (`0 -> 1`) into the
  session transaction, in stable surface order.
- M1 borrowed-page serialization and counters are applied. Prefix request clones are `N per encode
  attempt -> 0`; a normal page remains one JSON encode and App-side decode. Nine spatial-scale
  counters cover UI/core producer rows, rumble/cursor rows, total/pending/page rows, encode attempts,
  and encoded bytes. The typed ABI, single manager transaction, coalescing, and continuation remain
  pending M2-M4.
- M1a is applied. A successful non-empty App drain now collects expired rumble effects once before
  routing. Per-row routing and the rumble leaf perform zero expiry scans, while empty batches and
  drain failures remain zero and the independent gamepad-poll cleanup is unchanged. The static
  host-batch operation count is `(N + R) -> 1` for non-empty batches (`0 -> 0` when empty), changing
  expiry traversal from `O((N + R) * E)` to `O(E)`.
- Focused static contract
  `tools/tests/test_runtime_host_intent_outbox_transaction_performance_contract.py`: 101 lines,
  4,590 bytes, SHA256
  `8df58bc8b0c9e5aa81e473de59d1128d743354e46870d3657f352150cc8c0749`. M0 ran RED `0/4` to
  GREEN `4/4`; M1 ran RED `4/6` to GREEN `6/6`; M1a ran RED `6/7` to GREEN `7/7` and remains GREEN
  after rustfmt.
- Rust behavior gates were added for `handle_event -> tick_frame -> drain`, IME/cursor/rumble
  watermark behavior, exactly-once second drain, and borrowed/owned V1 JSON equality. They have not
  executed. The App source guard now also requires batch cleanup before row routing. Rustfmt
  `+1.94.1 --edition 2021 --check` passed the original nine files and all four M1a Rust files;
  scoped `git diff --check` passed.
- Broad current-worktree performance discovery ran 289 contracts: 288 passed and one unrelated
  Runtime07 hotpath-boundary audit failed because its expected historical source/span/query/extract/
  asset-worker/animation owner anchors and large-file owner classes are absent in the concurrently
  modified workspace. Runtime07 plan files are already foreign dirty; this slice did not fabricate
  those missing owners.
- The managed Windows focused lib-test attempt set `TEMP/TMP` to `E:\\CodexTemp` and target to
  `F:\\cargo-targets\\zircon-engine`, but did not enter Cargo. The coordinator returned
  `cargo_session_not_executable` because
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is archived. Raw Cargo and a fabricated
  Session identity are not accepted bypasses.
- WPR, power, real-window IME/cursor/rumble, and optional render parity remain pending because no
  launchable current-source executable exists. RenderDoc was not run against a stale binary.
- Keep all owners in `pending.md`; do not add them to `review.md` until M0-M4 dynamic gates pass.
