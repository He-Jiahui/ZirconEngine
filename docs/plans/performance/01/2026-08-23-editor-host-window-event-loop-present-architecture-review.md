---
title: Editor host window event-loop and present architecture performance review
date: 2026-08-23
module: zircon_editor retained-host window, event loop, resize, input and present dispatch
priority: MVP-P0 editor window responsiveness, idle power and resize presentation
status: source_reviewed_architecture_pending_dynamic
reference_engine: Unreal Engine WindowsApplication, SlateApplication, SWindow and Slate invalidation
---

# Goal

Keep the MVP editor event-driven at idle, bound high-frequency native input and resize work, and make
every presenter consume the same retained resize transaction. Window scheduling must not poll
background jobs indefinitely, rebuild the full software scene for every resize message, or contaminate
performance captures with synchronous snapshot work on the present thread.

## Reviewed source

- owner Rust files: 43/43
- lines: 5,241
- bytes: 186,688
- source-only SHA256 over lexicographically sorted owner files:
  `8587488b09531c503648f0cd48c35267bcff0c72704ae6cb5ba8c8b3cc729a2e`
- owning commit at review: `5f9704056761542857d74e733ce516f434de03dd`

| Owner group | Files | Lines | Bytes |
| --- | ---: | ---: | ---: |
| `host_contract/window.rs` | 1/1 | 84 | 2,325 |
| `host_contract/window/*.rs` | 17/17 | 2,297 | 78,914 |
| `host_contract/window/event_loop/**` | 14/14 | 2,473 | 91,442 |
| `host_contract/window/handle/**` | 1/1 | 29 | 589 |
| `host_contract/window/presentation/**` | 3/3 | 104 | 3,787 |
| `host_contract/window/text_input/**` | 7/7 | 254 | 9,631 |

All production files and colocated tests in these groups were read in full. Direct call chains were
also read through GPU/softbuffer presenter resize, profile-artifact submission, viewport render-
framework readiness, editor Job tickets, retained tick, text-focus state and the cited Unreal source.
Those supporting files are not counted as owner coverage here.

## Correct foundations to retain

1. Native idle is event-driven. `about_to_wait_impl` selects `ControlFlow::Wait` when no real deadline
   exists and `WaitUntil` only for runtime/maintenance frames, resize reflow, surface retry or presenter
   upgrade. It no longer polls native surface size, position or maximized state per event batch.
2. Background-event and window-attention wakes are atomic edge-coalesced flags. The event-loop proxy
   is signalled only when a pending flag changes from false to true.
3. Redraw state is bounded at both host and event-loop boundaries. `queue_redraw` asks winit for one
   native redraw only on the empty-to-pending transition; later events merge into one retained request.
4. Retryable surface presents keep exact current region/full semantics in one bounded slot and use
   exponential backoff from 8ms to 250ms. Success resets the backoff and consumes the measured input
   batch once.
5. Native resize retains only the latest physical size, configures that size before present and defers
   retained layout. The GPU presenter builds one frozen command snapshot per resize transaction and
   reuses it for later intermediate sizes.
6. Stable present borrows the immutable presentation generation. Full materialization is absent from
   the normal present path and occurs only when profiling/capture explicitly asks for a snapshot.
7. Input outcome tracking retains only the active input and one pending present batch. It does not
   accumulate an unbounded sequence collection.

## Structural findings

### P0: software fallback rebuilds and repaints the full editor for every resize event

Every distinct native size queues a full presentation without retained reflow
(`events/resize.rs:77-88`). The GPU override freezes one command draw-list and retargets it, but
`SoftbufferHostPresenter` inherits the trait default `present_during_native_resize`, which calls
ordinary full `present`. Its resize path clears the backbuffer, then the next intermediate event
replans diagnostics, rebuilds the complete chrome command stream, software-paints every pixel, copies
the full buffer and submits it.

For K resize events and scene size N, fallback work is O(K*N + sum(width*height)) on the UI thread.
This is not a small backend detail: softbuffer is the recovery path used when GPU presenter creation
fails, exactly when low-end or unstable graphics hardware makes CPU headroom more valuable. M2 creates
one backend-neutral `NativeResizeTransaction` with a retained source generation and one prepared
snapshot. Both GPU and fallback consume it; no backend may silently fall back to ordinary full scene
construction per native size event.

### P0: runtime presenter readiness is an indefinite 20Hz main-thread poll

The window schedules `RUNTIME_PRESENTER_UPGRADE_POLL_INTERVAL = 50ms` and repeatedly calls
`factory.poll_ready()` (`event_loop/lifecycle.rs:23,187-232`). The concrete factory takes the viewport
state mutex and polls a background `JobTicket`. Job completion has no direct window wake; the injected
background wake is used for asset/resource channels, not this ticket. `Ok(false)` has no attempt,
elapsed-time or terminal bound, so a queued, hung or never-completing resolver keeps the native loop
awake 20 times per second indefinitely.

The resolver correctly runs off-thread; the defect is completion delivery. M1 adds a one-shot typed
ready/failed generation published by the Job and an edge-coalesced event-loop wake. The UI thread
consumes that state once. A watchdog may report a stalled startup, but it must not be the normal
scheduler. Acceptance requires zero readiness polls and zero timer wakes after submission, one native
wake at terminal completion, and idle package energy equal to the no-factory baseline.

### P0: capture support perturbs the present thread and can invalidate measurements

Profile artifact I/O now runs in a bounded injected Job and is requested only once, which fixes the
old per-present export behavior. However, admission is followed by `materialize_presentation()` and
optional full software snapshot paint before the Job is committed
(`profiling_artifacts/export.rs:83-123`). First-presented-frame capture separately materializes and
software-paints the whole presentation, PNG-encodes, flushes and `sync_all`s synchronously from the
successful present callback (`redraw/present.rs:54,78`; `window/capture.rs`).

These are explicit diagnostic paths, not steady product cost, but they can add full-tree projection,
W*H CPU raster, PNG compression and durable disk I/O to startup/input-to-present traces. M3 separates
product present latency from evidence collection: capture owns an explicit generation receipt,
presenter readback or already-prepared immutable artifact, a bounded worker handoff and separate
capture-overhead counters. Measurement readiness begins only after capture preparation completes or
the capture workload is excluded by an explicit phase marker.

### P1: resize scheduling uses a fixed end debounce instead of a frame-budgeted transaction

An 80ms timer is reset by every size event. Intermediate frames present a frozen snapshot while
retained layout is delayed until the event stream becomes quiet. This bounds reflow, but it makes
layout responsiveness depend on an arbitrary silence interval and gives no maximum visual age during a
long drag. The target is one resize transaction: retain one snapshot, coalesce to the latest metrics,
permit at most one budgeted layout/update per display frame when necessary, and always run one final
exact reflow on resize completion or quiet. Record snapshot age and skipped/coalesced metrics rather
than treating fewer reflows alone as success.

### P1: focused text editing copies the complete value for every character

Insert and backspace clone the focus DTO, convert immutable `SharedString` to a new `String`, mutate it,
publish a new `Arc` focus value and dispatch the complete new value. Typing N bytes into an unbounded
field therefore copies O(N^2) bytes in aggregate, before the receiving control recomputes its model.
The MVP fields are currently simple append/backspace controls, so a rope is not justified by source
evidence. M4 first measures actual field lengths and callback rebuilds, then moves edit ownership to a
persistent mutable text buffer with revision, caret/selection/composition state and narrow changed
ranges. Commit/public binding snapshots remain immutable.

### P1: present damage remains a single bounding rectangle

The window path accepts `Option<FrameRect>` and reconstructs the same lossy shape after retry. The
adjacent redraw review already owns the bounded `DamageRegionSet` migration; this module must carry
that exact set through pending redraw, retry and both presenters. The window review does not duplicate
or locally wrap that work.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Private/Windows/WindowsApplication.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/SWindow.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Input/SEditableText.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/Text/SlateEditableTextLayout.cpp`

`FWindowsApplication::PumpMessages` drains platform messages and `ProcessDeferredEvents` first moves
the pending list before dispatch to avoid reentrant duplicate processing (lines 3059-3100). Slate
computes active-timer/input idleness and skips notification update plus `DrawWindows` when asleep
(`SlateApplication.cpp:1859-1882`). Each `SWindow` persistently owns its hit grid and invalidation root,
then calls `PaintInvalidationRoot` (`SWindow.cpp:2070-2149`). The invalidation root rebuilds the slow
path only when required, updates the fast invalid list when nonempty, and performs no widget paint
update when both are clean (`SlateInvalidationRoot.cpp:356-424`).

For text, `FSlateEditableTextLayout` retains the text layout, cursor and selection; character input
calls `TextLayout->InsertAt` and backspace calls `RemoveAt` on the addressed grapheme before targeted
layout update (`SlateEditableTextLayout.cpp:1698-1724,1575-1623`). Unreal still has transaction and
undo snapshot costs, so the transferable rule is persistent edit authority and changed-range
invalidation, not a claim that all text operations are constant time.

The transferable window invariant is: platform events may be batched, but expensive widget/paint work
is retained and invalidation-owned; async work participates through explicit timer/completion
ownership; all presenter backends consume the same invalidation transaction.

## Target architecture

1. Replace readiness polling with `RuntimePresenterReadiness::{Pending,Ready,Failed}` plus one terminal
   wake carrying a generation. Keep a watchdog only for diagnostics/recovery.
2. Introduce one `NativeResizeTransaction` owned above presenters: source generation, original
   projection size, latest physical metrics, transaction age, prepared draw/bitmap snapshot and final
   reflow receipt.
3. Make GPU and softbuffer implement the same resize contract. Intermediate size events may resize and
   retarget/scale a prepared artifact; they may not rebuild the retained scene. End/quiet commits one
   exact layout and resets ordinary damage baselines.
4. Keep native event scheduling O(1) and bounded. Carry a bounded damage-region set plus orthogonal
   frame-update/scenario attribution through external, pending, retry and presenter boundaries.
5. Move profiling and first-frame captures behind an explicit request state. Prepare immutable capture
   input outside the measured present section and perform encoding/durable I/O on a bounded Job.
6. Give text input a persistent edit-state authority and changed-range callback. Full immutable value
   publication happens only when a consumer contract requires it.
7. Delete fixed polling and backend-default full-resize compatibility behavior once all callers use
   the event/transaction contracts.

## Instrumentation and acceptance

| Evidence | Acceptance |
| --- | --- |
| idle native wakes, readiness polls and mutex acquisitions | zero after submission; one terminal completion wake |
| resize events / native redraws / surface resizes | bounded to empty-to-pending and latest size |
| resize snapshot builds and reuses by backend | one prepared scene artifact per transaction |
| retained frame updates/reflows during K size events | budgeted, no K-way full scene rebuild; one final exact reflow |
| command visits, CPU painted/copied pixels and allocations | reported separately for GPU and softbuffer |
| snapshot age and final geometry/pixel parity | bounded age; exact final layout and pixels |
| capture materialize/raster/encode/fsync on present thread | zero inside measured product present |
| text bytes copied, callbacks and presentation rebuilds | measured at 1/1K/1M characters; changed range only target |
| CPU/RSS/p95 input latency/context switches/package energy | same source, executable, workload and power state |
| RenderDoc draws/uploads/scissors/GPU time and pixel parity | current-source GPU resize and steady-present captures |

Matrix: readiness `immediate/50ms/5s/fail/stall`; resize events `1/100/1,000`, sizes
`720p/1080p/4K`, backend `GPU/softbuffer`, damage `none/one/8/full`, retry `0/1/5`, capture
`off/geometry/screenshot/first-frame`, text bytes `1/1K/1M`, and runtime demand
`on-demand/sleep/continuous`.

WPR/ETW records UI-thread CPU, wakeups, timer resolution, context switches, disk I/O and package energy.
RenderDoc is used only for GPU draw/upload/scissor/pixel parity; it cannot validate Job polling,
software fallback CPU or package power. All artifacts and target directories stay on D/E/F.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add readiness, wake, resize-transaction, backend scene-build, capture-overhead and text-copy counters; capture same-build baseline. | costs attributable without collector self-noise |
| M1 | Publish Job terminal readiness and edge-wake the event loop; remove the 50ms normal poll. | zero polling/timer wakes, correct ready/fail handoff |
| M2 | Add shared resize transaction and softbuffer prepared snapshot; hard-cut default full-resize fallback. | one scene artifact per transaction on both backends |
| M3 | Separate capture preparation/export from measured present and add explicit phase receipts. | no materialize/raster/encode/fsync in measured present |
| M4 | Introduce measured persistent text edit state and changed-range dispatch. | no aggregate O(N^2) immutable-value copying for long edits |
| M5 | Complete multi-region propagation, WPR/power and RenderDoc/pixel matrix. | quantified current-source acceptance |

## Validation state

- Owner source review: passed, 43/43 Rust files.
- Presenter, viewport readiness Job, capture artifact, retained tick, text state and Unreal reference
  call chains: read and mapped.
- No Rust optimization was applied in this pass. Each high-value change crosses an ownership boundary;
  a local timer tweak, softbuffer special case or string micro-optimization would preserve the faulty
  architecture and cannot be accepted without the matrix above.
- Focused Python performance contracts passed 11/11: native-window resize, surface-present retry,
  drawer resize and collection-source window. They validate the current static scheduling contracts,
  not elapsed-time, power, GPU behavior or Rust compilation.
- Managed Cargo, WPR and RenderDoc remain pending while the managed Cargo Session is terminal
  `archived` with `cargo_session_not_executable`; no current-source elapsed-time, GPU or power claim is
  made.

This module remains in `pending.md` until M0-M5 pass on one source/executable/workload fingerprint.
