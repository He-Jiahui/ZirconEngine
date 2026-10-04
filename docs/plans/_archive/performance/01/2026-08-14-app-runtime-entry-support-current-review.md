---
related_code:
  - zircon_app/src/entry/runtime_entry_app
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_runtime/src/dynamic_api/session/state.rs
  - zircon_runtime/src/input/runtime
  - zircon_runtime_interface/src/runtime_api/host/host_requests.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_runtime/runtime/10/failure-2026-07-19-app-entry-host-request-and-wake-boundary.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateUser.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/ApplicationCore/Private/Windows/WindowsApplication.cpp
tests:
  - runtime_entry_app current-source hash stability 79/79 passed
  - direct rustfmt 79/79 passed
  - 97 inline tests inspected by the 2026-08-14 review slices
  - managed Windows zircon_app build failed after 324.2 s with 6 current-source zircon_runtime errors; tests not run
  - WPR/Tracy host-request storm matrix pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# App runtime entry support current-source performance review (2026-08-14)

## Scope and current snapshot

This pass fully read the 35 files not already covered by the 2026-08-14 cadence, input, and
surface-present reports: config/construct/converters, failure, drag/drop, host requests,
diagnostics, window attributes/creation, and native surface target. The slice contains **35/35
files, 2,850 lines, 2,568 nonblank lines, and 54 inline tests**. Seven files contain changes from
other Sessions, so this pass did not modify production source.

The union of the four current reports now covers `runtime_entry_app/**` **79/79 files, 6,243
lines, 5,681 nonblank lines, and 97 inline tests**. Direct
`rustfmt +1.94.1 --edition 2021 --check` passed 79/79; the before/after file-set SHA-256 stayed
`0E6E03CD08538F8832FCF3FA744B7B7C4982DCE0BF4E012D807D95704B2EFBDA`. Eighteen files are
foreign dirty. This is static coverage, not dynamic acceptance.

## P0 host-request owner and ordering defect

The current nonempty request path is wider than an App loop:

1. `DefaultInputManager` appends IME, cursor, and rumble commands to three frame-local `Vec`s.
2. `RuntimeDynamicSession::drain_host_requests` performs three separate `mem::take` operations,
   maps each vector, and collects a fourth request vector.
3. Runtime serializes the complete batch to JSON and publishes an owned ABI buffer. App validates,
   JSON-decodes a new vector, frees the ABI owner, then applies every request as a window/IME/cursor
   system call on the event-loop thread.
4. No production contract limits entries, encoded bytes, age, apply time, or carry-over. Repeated
   latest-state requests such as cursor position/visibility, IME cursor area, and surrounding text
   are not generation-coalesced.

This remains existing **PERF-MVP-425** and its Runtime10 failure handoff; no duplicate failure is
created. The current source also exposes a correctness prerequisite that the old plan did not make
explicit: `tick_frame` calls `level.tick`, `operations.tick`, and then `InputManager::begin_frame`,
which clears all three host-request vectors. App calls `drain_host_requests` only after
`tick_frame` returns. Therefore a request present before that `begin_frame` is cleared before the
product drain.

Existing tests do not close this gap. Input-manager tests intentionally prove that `begin_frame`
clears pending requests; dynamic API tests prove only `handle_event -> drain` without an intervening
tick. There is no product-order test for `produce/submit -> tick -> App drain`. Optimizing or
coalescing this path before fixing the owner boundary could make an already-lost edge harder to
detect.

## Other reviewed costs

- Default window creation always asks for the primary monitor and collects all monitor handles,
  even for the default windowed/automatic-position descriptor. This is a one-time F0 startup cost,
  not a stable-frame P0. Record monitor enumeration wall/count before adding lazy policy logic.
- First-frame product diagnostics repeatedly query named series and materialize a large log string,
  but run only on the explicit first-frame evidence path. Keep observer overhead separate from the
  normal present baseline; do not remove evidence to improve startup numbers.
- Drag/drop converts and dispatches each path separately. Hover storms belong to PERF-MVP-426 input
  ingress measurements, while drop/cancel edges remain ordered and lossless.
- Unknown keyboard codes format directly into an FNV sink without heap allocation. Replacing that
  stable mapping with an enum discriminant would change the ABI contract and is not justified by
  current data.

## Unreal source basis

Unreal `SlateUser.cpp:808-920` resolves a cursor reply under one Slate owner. Its update path at
`951-986` only queries while capture is active or `bQueryCursorRequested` is set, and
`ProcessCursorReply` at `989-1018` performs the final cursor state publication. `SlateApplication.cpp:
3816-3823` delegates to that owner. On Windows, `WindowsApplication.cpp:1302-1309` retains the
high-precision state and forwards the final toggle to the input device owner.

This supports a request-driven, typed owner boundary between UI/runtime intent and platform calls.
It does not provide Zircon's queue sizes or time budgets; those must come from current-product
traces, and copying Unreal's API surface is not an acceptance criterion.

## Unified optimization and acceptance plan

| owner | required change | acceptance evidence |
| --- | --- | --- |
| Runtime10 | First freeze the produce/tick/drain ordering with a product behavior test. Then publish a versioned typed batch split into lossless edges, latest-value state, and bounded commands; keep V1/V6 compatibility and exactly-once output release. | `produce -> tick -> drain` preserves required requests; 1/1K/10K mixed requests report entries/bytes/age/coalesce/backpressure, JSON encode/decode = 0 on the new path, and no request is silently cleared. |
| Runtime12 + Runtime UI | Give each request class one generation/sequence owner. Coalesce cursor/IME latest state before ABI publication while preserving enable/disable, rumble stop, and other ordered edges. | Repeated equal state causes zero platform calls; final state is equivalent; edge order/loss tests and producer counters pass under 1K/10K bursts. |
| App entry | Apply a bounded page/time slice on the event-loop thread, publish remaining/oldest age, and request one continuation wake. Do not add an App-private unversioned cache. | Main-thread apply p50/p95/p99 and OS-call counts stay within a measured budget; backlog entries/bytes/age are hard-bounded; window teardown drains or rejects requests explicitly. |
| Runtime03 | Keep continuation wake and presentation demand distinct so request backlog does not force unconditional redraw. | Host-request continuation wakes do not increase present count when no surface damage exists; idle/active WPR counters remain correct. |

Run mixed cursor/IME/rumble at 1/1K/10K requests, 1/4 windows, and 30/60/120 Hz consumers.
Record producer requests, cleared requests, batch/pages, encoded/copied bytes, manager/session lock
wait, OS calls, continuation wakes, queue peak/age/coalesce/drop, main-thread p50/p95/p99, CPU,
context switches, and power. WPR/Tracy is the CPU authority; RenderDoc is only needed to prove that
request continuation does not create extra render/present work.

## Validation status

The managed Windows dry run rendered D-drive build/test commands without executing Cargo. An empty
E-drive temp directory created for `TEMP/TMP` was rejected as unmanaged and then removed through the
coordinator; a subsequent artifact audit returned no unmanaged paths. With `TEMP/TMP` redirected to
the existing E-drive workspace temp, a later build-only `zircon_app` matrix completed against the
D-drive managed target after **324.2 seconds** and failed with exit 101. Cargo emitted 212 warnings
and 6 compile errors in concurrently modified `zircon_runtime` source: duplicate query-state cache
state, one resource-management type-inference error, one neutral-graph buffer-view API mismatch, and
two moved-route uses in UI event routing. These files are outside this App review's write scope and
are foreign dirty, so this Session did not overwrite them. Tests and the WPR/Tracy matrix did not run.

This is a current-source **build failure**, not an App-entry performance acceptance result. The
managed job was released, its target remained on D:, and this Session placed no artifact on C:.
`runtime_entry_app/**` therefore remains in `pending.md`; it must not enter `review.md`, produce a
performance milestone commit, or trigger a WeCom completion message yet.
