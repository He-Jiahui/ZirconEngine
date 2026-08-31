---
related_code:
  - zircon_editor/src/core/jobs/**
  - zircon_editor/src/core/asset/**
  - zircon_editor/src/core/notifications/**
  - zircon_editor/src/core/recovery/**
  - zircon_editor/src/ui/retained_host/**
  - zircon_runtime/src/core/runtime/tasks/**
related_plans:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/01/2026-08-15-editor-job-facade-admission-completion-protected-plan-routing.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
write_scope: []
status: pending
---

# Editor jobs closure

This is a current-source static closure for the editor job facade, admission,
completion and retained event boundary. It remains pending: the current source
does not reach a green managed `zircon_editor` Cargo gate because compilation
stops in known unrelated runtime/UI/text production errors. No Rust source was
changed, and static evidence does not enter `review.md`.

## Scope and source state

- `zircon_editor/src/core/jobs/**`: 58 Rust files, 11,423 physical lines,
  10,211 nonempty lines, 377,766 bytes, 149 test attributes, 14 ignore
  attributes and 19 include sites. Sorted path plus NUL plus raw-content
  SHA256: `b919c077e0d73fa8ca5aee1d1f0a6277958fdc2e19de2a6cae74acb246f23adb`.
- The current tree contains foreign modified and untracked scheduler,
  admission, event-journal, lifecycle, pending-task and test-support work. The
  review preserves that work and does not reconcile its formatting or behavior.
- Isolated `rustfmt --check --edition 2024 --config skip_children=true` passes
  33/58; the 25 failures are current foreign formatting/import/assert work. The
  focused managed Cargo compile command timed out before producing an accepted
  test binary. Known build blockers, stale source-shape guards and no current
  executable mean that WPR/xperf/allocator/thread/power measurements are not
  claimed.

## Positive work to preserve

- Category limits are finite; pending admission has entry, estimated-byte and
  age limits, batch reservations, and keyed latest-work merging.
- The pending queue uses maintained BTree ready/dependency indices and a fixed
  weighted fairness schedule. Promotion selects under the state lock but calls
  the shared Runtime scheduler outside it, with a 64-item dispatch batch.
- Terminal records use indexed evictable retention and dependency pinning.
  Labels use `Arc<str>`, progress is latest-coalesced, and the event journal has
  count/byte/age limits plus an explicit gap rather than an unbounded queue.
- Pending-task panic containment, observer panic recovery/resynchronization,
  lock-external observer callbacks, sorted dependency insertion and precomputed
  batch metadata remove several earlier clone and scan findings.
- `JobTicket::wait_until` does not hold its receiver mutex while waiting, and
  cancellation tokens are O(1) cooperative atomics. These remain useful facade
  semantics once affinity and deadline ownership are made structural.

## Retained findings

1. **Two scheduler authorities remain (P0).** `EditorJobSystem` still owns
   records, category quotas, priority/fairness, dependency readiness,
   mutex-group tails, promotion and terminal retention before forwarding work
   to Runtime `schedule_after`. Runtime TaskGraph then owns another dependency
   graph, pool and completion handle. Neither layer can make one admission,
   affinity, deadline, cancellation or worker-saturation decision. The editor
   facade must map domain categories and keyed work onto one Runtime11 TaskGraph
   generation and delete its peer scheduling policy.
2. **Admission is not end-to-end (P0).** The ledger reserves pending entries and
   estimated payload bytes, but not result-channel capacity, lifecycle journal
   edges, progress/error message bytes, observer receipts or downstream message
   delivery. Local queues are bounded independently, so an accepted job can
   finish while a later bridge lacks capacity or loses its lifecycle edge.
   One request/delivery lease must reserve the full retained envelope and
   terminalize it as Succeeded, Failed, Cancelled, Backpressured or Fault exactly
   once, including consumer loss and shutdown.
3. **Completion still couples work to presentation (P0/P1).** A completion
   guard mutates editor state, updates progress, delivers observer events and
   recursively calls promotion. The observer dispatch is panic-contained and
   bounded per batch, but it drains events enqueued by callbacks in the same
   call, so a reentrant storm can extend one worker-owned delivery indefinitely.
   Completion should publish one compact receipt and one coalesced wake; named
   editor/UI affinity owns bounded callback pages.
4. **Blocking is not affinity-safe (P0/P1).** Public `JobTicket::wait` can
   block an editor, main or render caller, and `join` has the same broad
   semantics. `wait_until` supplies a timeout but no caller-affinity contract.
   Keep blocking waits for worker/tool shutdown only; reject named product
   callers or return a typed pending handle with deadline and cancellation.
5. **Lock scopes cross unrelated delivery (P1).** `JobEventSink::emit` holds
   lifecycle state while applying progress and pushing the journal, and the
   event pump holds its consumer mutex while publishing to the message bus.
   Observer delivery also queries shared progress for every callback. These
   nested locks make event pressure and bus backpressure visible to job
   completion. Move immutable event construction under a short owner lock and
   transfer bounded pages before bus/observer work.
6. **Journal enforcement has hidden work (P1).** Expiry/limit pruning and gap
   coalescing run under the journal mutex; merging a gap materializes covered
   sequence IDs in a temporary `Vec`. This is bounded by configured journal
   limits, but it still needs a byte/time budget and counters for dropped ranges.
   Logical event estimates also do not prove allocator capacity for every
   nested message.
7. **Identity counters can repeat (P1).** Job IDs, admission reservation IDs
   and terminal ordering use saturating increments. At exhaustion they can
   repeat an identity or silently share order. The journal's checked sequence
   path is a positive local precedent; converge jobs, reservations and runtime
   task generations on checked, non-repeating session-qualified identities.
8. **Dependency and shutdown proposals are incomplete (P1).** A spec's
   `after` list has no count/byte/deadline cap before dependency handles are
   materialized. Shutdown drains pending IDs into an owned vector, invokes
   cancellation callbacks serially, then waits on a condition variable. A
   deadline returns unfinished work but not a terminal settlement receipt, and
   active tasks retain the system through completion guards. Both paths need
   bounded pages, explicit cancellation outcomes and a shutdown generation.
9. **Stable status and snapshots still have an escape hatch (P1).** The
   primary-generation fast path exists, but retained status consumers can still
   call the full snapshot, clone labels/progress and format DTOs each tick.
   Full progress snapshots walk active maps and have no owned row/byte/deadline
   proposal. Stable consumers should borrow a generation cursor; owned exports
   must be explicit and bounded.
10. **Keyed replacement needs a receipt (P1).** Latest pending replacement
    correctly keeps one job/result slot, but a merge reports only the existing
    job ID. Callers cannot observe which payload generation won or whether a
    replaced request was cancelled, superseded or delivered. Publish a typed
    merge/supersession receipt tied to the same terminal job generation.

## Architecture handoff

1. Compile one immutable `EditorTaskGeneration` over the shared Runtime11
   TaskGraph: category/resource lane, priority, named affinity, prerequisites,
   cancellation/deadline policy, payload/result/lifecycle byte limits and
   session/device identity. Reject cap+1 before task/channel allocation.
2. Make the editor API a typed facade over one accepted task ticket and one
   terminal receipt. Reserve pending, started-but-undelivered, result,
   event-journal and message-bus capacity atomically; replacement, panic,
   consumer loss and shutdown all emit one terminal outcome.
3. Move completion to a compact receipt plus coalesced wake. Named editor/UI
   observers consume count+bytes+deadline pages outside producer locks. A
   callback cannot recursively invoke promotion or extend a worker completion
   indefinitely.
4. Require affinity-aware wait APIs. Main/editor/render callers receive
   `Pending`, `Busy`, `Backpressured`, `Cancelled` or `Fault` instead of
   blocking; worker/tool shutdown may use a bounded wait with a settlement
   receipt.
5. Replace recursive journal gap/prune work with indexed cursors and admitted
   maintenance pages. Report exact retained logical/owned bytes, dropped ranges,
   oldest age, callback time and downstream backpressure.
6. Use checked non-repeating job, reservation, ticket and shutdown generations
   qualified by the editor session and shared runtime scheduler generation.
   Stable status queries borrow `Arc` generations; explicit full snapshots and
   JSON exports pre-admit rows, bytes and deadlines.

## Evidence and acceptance gates

The local Unreal TaskGraph exposes one `FTaskGraphInterface` with explicit
desired named threads, task priorities and prerequisite-triggered completion in
`dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/TaskGraphInterfaces.h`.
The implementation also provisions named/high-priority/background worker lanes
in `Core/Private/Async/TaskGraph.cpp`. This supports one scheduler with
affinity and trigger-on-completion receipts; it is not a reason to copy Unreal's
indefinite wait APIs.

M0 adds RED tests for duplicate scheduler admission, result/event capacity,
observer reentrancy, named-thread waits, identity exhaustion, dependency cap,
shutdown deadline settlement and status no-work. M1-M3 perform the TaskGraph
facade hard cut and end-to-end terminal lease. M4-M6 add bounded snapshots,
diagnostic deltas and managed scale/F0/F4 evidence.

Acceptance covers 0/1/64/1k/cap+1 jobs, dependencies, category and keyed
replacement permutations, worker/editor/render affinities, cancellation and
panic at every phase, event/message capacity loss, observer storms, shutdown
before/after completion, identity exhaustion, stable/changed status generations
and diagnostics Disabled/Counters/Full. Report proposal/admission/promotion/
execution/completion latency, queue and worker counts, lock hold/wait, callback
time, retained logical/owned bytes, drops/orphans/supersessions and terminal
outcomes.

Hard gates: current-source Cargo builds; one Runtime TaskGraph owns scheduling;
all accepted jobs terminalize exactly once; cap+1 produces zero task/device or
delivery work; callbacks never run under producer locks or recursively promote;
named product callers cannot block; identities never repeat; shutdown and
consumer loss leave no orphan lifecycle/result; stable/Disabled queries clone
and allocate zero; diagnostics match actual retention, waits and drops. No
benchmark artifact or micro-fix is warranted before these ownership corrections.
