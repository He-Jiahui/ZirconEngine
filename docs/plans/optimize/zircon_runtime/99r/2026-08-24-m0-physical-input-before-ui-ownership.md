---
title: Runtime117 M0 Physical Input Before UI Ownership
date: 2026-08-24
status: implementation_source_complete_dynamic_pending
parent_plan: docs/plans/optimize/zircon_runtime/99r-runtime-input-device-event-frame-state-action-map-focus-gamepad-recording-replay-host-product-integration-current-source-review.md
runtime_owner: docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
session: runtime117-m0-physical-first-integration-r1-20260824
---

# Runtime117 M0 Physical Input Before UI Ownership

## Scope

This slice implements the first correctness boundary for `INP-P0-002`, `INP-G03`, and Runtime12:
every physical event admitted by the current per-kind checks reaches the shared `InputManager`
before Runtime UI decides whether semantic propagation continues. It does not claim the later
complete qualification, stable device ID, shared sequence, per-player action arbitration,
immutable publication, recording, replay, or performance milestones.

The existing Runtime12 gamepad-storm failure remains open because App producer batching is outside
this source slice.

## Architecture Decision

`DefaultInputManager::submit_event` is currently the physical state and raw event-journal owner.
Runtime UI owns hit testing, focus, pointer capture, component mutation, and the decision to stop
later semantic propagation. The dynamic session therefore uses this order:

```text
decode host event and apply the current per-kind admission checks
  -> submit physical InputEvent and update ingress counters
  -> dispatch the corresponding Runtime UI event
  -> stop only later camera/gameplay semantics when UI handles or blocks
```

UI dispatch failure cannot roll back a physical fact that already arrived from the host. Text and
IME remain semantic text-target events and keep their existing UI-first ownership in this M0 slice.

Unreal's physical `FKeyState` remains separate from Enhanced Input mapping/action consumption and
has explicit held-key rebuild policy. Bevy updates `ButtonInput` from every mouse/keyboard event,
and Fyrox updates engine input state from OS events before downstream consumers. Those references
support the same separation without copying their object or ECS layouts.

## Delivered Source

- pointer position and raw mouse motion now commit before Runtime UI dispatch;
- mouse press/release and both wheel encodings now commit before Runtime UI dispatch;
- touch cursor/contact facts now commit before Runtime UI dispatch;
- keyboard press/release now commit and update ingress counters before Runtime UI dispatch;
- gamepad button and axis facts now commit before navigation/analog UI dispatch;
- camera pointer, button, and scroll behavior remains after UI ownership, so a handled UI event
  still suppresses gameplay semantics;
- empty Runtime UI keeps lazy metadata construction, including the pre-existing last-surface move
  optimization in the transferred mixed blob.

The production change covers 8 UI-routable physical ingress branches in
`zircon_runtime/src/dynamic_api/session/events.rs`. No compatibility facade, duplicate input state,
or test-only production hook was added.

## Correctness Review

- mouse button kind/state, typed wheel unit/delta, touch phase, keyboard payload/action, and
  gamepad button state are still checked before physical submission; M0 does not weaken the
  existing admission boundary;
- physical submission is ordered before UI only after those checks, while camera movement,
  camera button ownership, menu actions, and camera scroll stay after the UI decision;
- a UI dispatch error intentionally leaves the already-arrived physical fact committed because
  UI mutation is not the owner of host input truth;
- retry deduplication after that error is not claimed here. Stable source sequence and journal
  idempotency remain requirements of the later Runtime117 device/sequence/recording milestones.

## Next Milestone Dependency Freeze

The M117.1 current-source review found that `InputDriver` is an empty zero-sized service and has no
relationship to the real ingress path. The product host currently calls the frozen Runtime API V7
`handle_event(session, event)` entry once per event, and the dynamic session submits those events to
`InputManager`. Therefore the next milestone must not rename or wrap the empty driver and claim an
ingress owner. Unreal's `IInputDevice` is not a precedent for that shell: it is a real polling owner
that requires `Tick`, `SendControllerEvents`, and `SetMessageHandler`, with an explicit thread
affinity contract. Its hard cut requires one coordinated dependency slice:

1. remove the empty `InputDriver` descriptor/type and the readiness evidence that treats its
   registration as capability;
2. introduce the bounded batch/page ABI in a new runtime API table version, with Interface and App
   producers migrated together;
3. make the runtime ingress broker publish health, backpressure, clock/device-registry dependency,
   and terminal teardown evidence before the Input module can report Ready.

Those files are outside this session's immutable scope and are intentionally unchanged here.

`events.rs` is currently 852 lines: above the repository's 800-line review warning and below the
1000-line hard gate. M0 only reorders existing branches and does not add another event-domain
responsibility. Before M117.1 adds ingress behavior, the combined owner must split pointer,
keyboard/IME, and gamepad handling into folder-backed leaves while keeping the session root as the
orchestrator; this narrow session cannot legally add those new paths.

## Ownership Evidence

The pre-edit `events.rs` SHA-256 was
`a5d7bc7c3dc18927c40c67d05f53c691cd32ad1ae9a93534031e9f6dde310087`. Coordinator transfer
preview fingerprint `15811d7ebde68043997690d01091ee6202977e0dfa4b0a07e0e949d337c14950`
reported no blockers and transferred the complete mixed blob from the cancelled Interface01
session. The existing lazy UI metadata and clone-avoidance changes were preserved.

## Validation Status

| Item | Status | Evidence |
|---|---|---|
| Current source and mixed-blob review | complete | exact-hash ownership transfer, no foreign rewrite |
| Unreal/Bevy/Fyrox architecture comparison | complete | physical state is independent of semantic consumption |
| Physical-first production ordering | source complete | 8 ingress branches reordered; text/IME explicitly excluded |
| Real pointer-capture regression source | complete | Runtime UI Slider capture releases outside its bounds |
| Scoped rustfmt and diff check | pass | Rust 1.94.1 rustfmt; no whitespace errors |
| Input structure audit | pass | `test_runtime_input_stack_audit.py` 1/1 |
| Empty-UI and last-surface performance guards | pass | 6/6 static tests; lazy metadata and move-only last consumer preserved |
| Physical-before-UI source-order review | pass | 8/8 ingress anchors ordered correctly |
| Admission/error/gameplay ordering review | pass | existing checks stay before commit; later semantics stay after UI |
| Production owner size review | warning | `events.rs` 852 lines; folder-backed split required before new behavior |
| RED attempt `394f67e068844a84beaffe3a41821cea` | invalid infrastructure result | test never ran; validation-copy materialization failed on 6 unrelated unowned `AM` paths with `validation_copy_baseline_drift` |
| RED retry `f935edf5c91c4e33b19c905e3be79821` | invalid infrastructure result | same frozen manifest; materialization again failed on the same 6 unrelated `AM` paths before compile/test |
| Focused RED/GREEN Cargo | pending | coordinator-managed tickets only; no direct Cargo executed |
| Runtime regression suite and product build | pending | required before integration candidate |
| Performance and power comparison | not claimed | correctness M0 only; no WPR/ETW or competitive benchmark |

This record must not be promoted to a completed Runtime117 milestone until the real capture test,
focused runtime regressions, and an applicable product build pass from immutable coordinator source
manifests.

The failed materialization above is not counted as RED. Runtime12 test ownership and Runtime117
production ownership remain separate until a replacement immutable run reaches the test assertion.
After two identical validation-copy failures, no third blind retry is submitted while those foreign
paths remain unstable; this is environment evidence, not a Runtime117 behavior result.
