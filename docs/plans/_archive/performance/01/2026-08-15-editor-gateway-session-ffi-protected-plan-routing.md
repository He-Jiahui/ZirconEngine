---
related_code:
  - zircon_editor/src/core/gateway
  - zircon_editor/src/tests/gateway
  - zircon_editor/src/core/runtime_event_consumer
  - zircon_editor/src/core/sync/pump.rs
  - zircon_editor/src/ui/host/editor_host_event_controller.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/tick.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/review.md
  - docs/plans/performance/pending.md
owner_plans:
  - docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/zircon_editor/editor/04-pie-and-simulation.md
  - docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
  - docs/plans/zircon_runtime/runtime/10-dynamic-api-and-interface-convergence.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
source_evidence:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/PlayLevel.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageRouter.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageDispatchTask.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Tasks/Pipe.h
---

# Protected plan routing: editor gateway session FFI

## Reason for routing

`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`, `review.md`, `pending.md` and
the owner plans are protected/foreign dirty in this session. This record preserves current-source
corrections and owner instructions without overwriting their work. The evidence source is
`2026-08-15-editor-gateway-session-ffi-current-architecture-review.md`.

## Requested Performance01 correction

Update the gateway summary and `PERF-MVP-597` with current scope:

- production is 21/21 files, 2,711 physical lines and 10 tests; external gateway tests are 11/11
  files, 2,082 lines and 51 tests;
- stable gateway reads use ArcSwap and a generation-bound capability Arc; shared read lock and
  stable String deep-clone findings remain closed;
- frame storage stays provider-owned and zero-copy through gateway release/drop; frame-demand kind
  and delay propagation remain fixed;
- plugin pages remain hard-bound to 64 deliveries/128 KiB and WorldSync responses to 1 MiB;
- all current production and external gateway files pass rustfmt.

Retain `PERF-MVP-597` P0 and add the current structural evidence:

- the retained UI tick calls Play `tick_frame`, then per-consumer plugin-event FFI drain and JSON
  decode inline;
- the 4 ms pump budget is checked before a drain and between callbacks, so it cannot preempt one slow
  provider call or decode;
- `with_current_gateway_generation` holds the gateway replacement mutex across WorldSync
  watch/unwatch transport calls and editor watch-map mutation;
- SessionGateway WorldSync drain may execute foreign work plus a 1 MiB decode on its caller;
- `replace` materializes incoming capabilities under the writer mutex.

The target architecture is one Runtime11-owned, ordered, per-session ticket lane with explicit
operation affinity, single-flight tick/drain, hard queue entries/bytes/age, generation-tagged
immutable completions, cancellation and stale-result rejection. Do not create a gateway-private
pool or one OS thread per gateway. The retained host only polls/applies completions. Watch allocation
uses a strong generation lease, runs foreign work outside the writer mutex, commits under a short
generation comparison, and compensates stale tokens through the retained old generation.

Add acceptance for provider 0/1/16ms/10s, 30/60/120Hz, consumers 0/1/64, 0/1/64-row pages,
0/1KiB/128KiB plugin payloads, 0/1KiB/1MiB WorldSync payloads, watch/unwatch crossed with 0/1/1K
replacement and stop/unload during queued/running work. Required results: UI-thread foreign/JSON wall
zero, per-session in-flight at most one, queues hard-bound, foreign wall under replacement mutex zero,
no accepted event loss/dup/reorder, stale applies/token leaks zero, and bounded cancel/join.

## Requested owner-plan updates

### Editor01

Own the stable generation lease and explicit gateway operation-affinity classes. Split generation
lease acquisition, foreign execution and short commit. Materialize incoming capabilities before the
publication lock. Preserve ArcSwap stable reads, old-generation lifetime and panic recovery.

### Editor02

Route serialized query/watch/unwatch/invalidation drain through the session lane. Publish immutable
generation-tagged WorldSync completions and make empty serialized drains demand/wake driven. Keep the
in-process path direct and transport-neutral DTO semantics unchanged.

### Editor04

Own Play session generation, start/stop ordering, lane creation/destruction and stale completion
rejection. Stop/unload must cancel queued work and join boundedly before releasing the provider.
Keep one Play lifecycle authority, matching Unreal's queued request ownership.

### Editor05

When highlight overlay becomes product-reachable, publish one shared sorted/deduplicated selection
generation. Measure Vec sort/dedup against the current BTreeSet construction before changing the
algorithm. Do not add per-frame reconstruction.

### Runtime10

Declare which dynamic ABI calls are session-serial, render/native-thread-affine or caller-safe, and
keep provider-owned output valid until the lane decodes/releases it. ABI functions remain transport
contracts, not scheduler owners.

### Runtime11

Provide the per-session ordered lane using the existing JobSystem/task model: FIFO non-concurrent
execution, hard entry/byte/age bounds, one running ticket, cancellation, terminal observer and
diagnostics. This is the Zircon counterpart to Unreal `FPipe`; do not create a new private executor.

### Render17

Extend profiling export with session lane queue/wait/run/decode/release and caller-thread identity.
Use WPR for CPU/session evidence. Use RenderDoc only for a runnable viewport present/readback frame,
not for the CPU gateway scheduling claim.

### EditorUI08

Poll immutable current-generation tick/event/WorldSync completions from retained tick and apply each
accepted generation once. Remove foreign FFI and JSON decode from UI frame wall. Preserve demand
application, error display and callback budgets.

## Requested protected index state

- `pending.md`: replace the stale gateway row with one concise module row covering
  `zircon_editor/src/core/gateway/**` and `zircon_editor/src/tests/gateway/**`, status
  `static_complete / dynamic_pending`, current file/line/test counts and the current review link.
- `review.md`: do not add either tree. Current managed Cargo, slow-provider/scale counters, F4 WPR,
  same-machine latency/RSS/power and lifecycle stress are absent.

## Milestone and notification state

This is a static review/routing record, not an accepted performance milestone. No git commit or WeCom
notification is due. Commit and quantified WeCom notification occur only after current-source dynamic
evidence closes the acceptance matrix and the protected indexes are reconciled by their owner.
