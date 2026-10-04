# Editor core event input, transaction, replay, and audit architecture review

## Status

- Result: `static_complete / dynamic_pending`.
- Review date: 2026-08-16.
- MVP priority: P0. Viewport input and basic editor command/undo/replay behavior are MVP paths.
- Owners: EditorUI01 owns raw input routing; Editor03 owns committed transaction history and replay;
  Editor02 owns audit/listener delivery; Editor08 owns command identity; EditorUI08 consumes typed
  invalidation/effect publications. Runtime11 may schedule only explicitly non-main audit work.
- Accounting: keep `zircon_editor/src/core/editor_event/**` in `pending.md`; do not add it to
  `review.md` before the dynamic matrix below passes.
- Code disposition: no Rust source changed. Twelve modified and two untracked production files were
  read at the recorded fingerprint and preserved. This report complements the 2026-08-15 retention
  review; it does not overwrite that foreign/untracked record.

## Exact scope

| scope | files | physical lines | tests | ordered path-and-raw-content SHA256 |
|---|---:|---:|---:|---|
| `zircon_editor/src/core/editor_event/**` | 36/36 | 2,667 | 8 | `56de05ee5ca871b79aaa37370bb0ecbd20b863e855ef91ede324d83848e1e4da` |
| `zircon_editor/src/tests/editor_event/**` | 30/30 | 8,128 | 138 | `c611be77d08414f426cc72863b0863ad445d9e4ab308bacc62b60164e2c27684` |

The fingerprint is SHA256 over normalized sorted path, NUL, raw file bytes, NUL. Every production
file was read in full in this review. The unchanged external-test fingerprint and inventory match the
full 2026-08-15 read. Production callers were traced through retained viewport dispatch,
`editor_event_dispatch`, execution/effect projection, command reverse routing, journal/listener
control, and replay. Raw marker counting reports 12 `#[test]`/`#[cfg(test)]` markers; the exact test
count is eight `#[test]` functions.

## Per-file acceptance record

| file | current-source verdict |
|---|---|
| `dispatcher.rs` | The public contract returns an owned full record, forcing execution and audit ownership to remain coupled. |
| `hierarchy_host_event.rs` | Typed payload only; vector/string cost belongs to the semantic hierarchy operation. |
| `inspector_field_change.rs` | Typed payload only; no local hot loop. |
| `journal.rs` | Product snapshot deep-clones every retained record; no cursor export. |
| `listener/filter.rs` | Prefixes normalize once; per-record vector scans remain measurable, not yet a proven root cause. |
| `listener/mod.rs` | Re-export boundary only. |
| `listener/projection.rs` | Reprojects owned delivery DTOs into JSON, retaining duplicate boundary materialization. |
| `listener/registry.rs` | Immutable route snapshot correctly shortens the registry lock; configuration rebuild is admin-time work. |
| `listener/route.rs` | Each matching arrival still locks and mutates a separate inbox; page count is capped at 256. |
| `listener/types.rs` | Page creation clones wide strings/JSON/result before final JSON projection. |
| `mod.rs` | Export boundary exposes journal, replay, listener, input and workbench concepts as one event subsystem. |
| `replay.rs` | Re-executes every supplied journal record without consulting retention class or undo/replay policy. |
| `retention.rs` | Indexed/cursor fixes are valid; every arrival still creates a discarded full JSON buffer for byte accounting. |
| `selection_host_event.rs` | Typed selection payload only. |
| `service/editor_event_service.rs` | Sequence, journal and listener locks are split, but all stages still run synchronously on the caller. |
| `service/listener_control.rs` | Inbox work is outside the registry lock; count-only pages still clone DTO then JSON. |
| `service/mod.rs` | Re-export boundary only. |
| `service/stamp.rs` | Stamp has one revision pair but cannot represent no-op/failure commit semantics independently. |
| `service/state.rs` | One mutex serializes event id, delivery order and authoring revision allocation. |
| `types.rs` | Raw input, commands, external requests, transient UI state, audit fields and replay data share one wide enum/record. |
| `workbench/activity_drawer_mode.rs` | Small value type only. |
| `workbench/activity_drawer_slot.rs` | Small value type; canonicalization is constant time. |
| `workbench/console_message_filter.rs` | Small value type only. |
| `workbench/console_source_filter.rs` | Small value type only. |
| `workbench/layout_command.rs` | Semantic layout payload; path vectors allocate only when such a command is constructed. |
| `workbench/main_page_id.rs` | Owned string id; not an independent loop. |
| `workbench/menu_action.rs` | Semantic actions and external side effects currently share one replayable event family. |
| `workbench/mod.rs` | Re-export boundary only. |
| `workbench/split_axis.rs` | Small value type only. |
| `workbench/split_placement.rs` | Small value type only. |
| `workbench/tab_insertion_anchor.rs` | Small semantic payload. |
| `workbench/tab_insertion_side.rs` | Small value type only. |
| `workbench/view_descriptor_id.rs` | Owned string id; not an independent loop. |
| `workbench/view_host.rs` | Semantic host/path payload; no local hot loop. |
| `workbench/view_instance_id.rs` | Owned string id; not an independent loop. |
| `workbench/workspace_target.rs` | Small semantic payload. |

## Structural verdict

The current subsystem is not merely an expensive queue. It conflates four different contracts:

1. realtime input delivery and frame-local interaction state;
2. editor command execution and committed transaction history;
3. UI invalidation/effect publication;
4. audit, remote observation, plugin delivery and executable replay.

That conflation puts optional diagnostics on the mandatory input path and makes audit data executable.
Container-level fixes cannot close this architecture.

### P0: the journal is not a valid replay authority

`EditorEventReplay::replay` clones and dispatches every record it receives (`replay.rs:6-28`). The
journal returns all three retention classes, not only durable semantic commands (`journal.rs:49-60`;
`retention.rs:509-541`). Therefore replay can execute pointer moves, press/release, scroll, viewport
resize, transient UI changes, project open/save/close, import requests, and recorded failures.

`EditorEventUndoPolicy` does not protect this path: replay never reads it. The policy also marks every
viewport event as delegated to the transaction engine, including pointer motion and resize
(`ui/host/editor_event_execution/undo_policy.rs:3-34`). `OpenCommandPalette` and most other unmatched
events fall into durable retention even when they are non-authoring UI actions
(`retention.rs:639-665`). Audit evidence, input history, side-effect requests and deterministic
transaction replay are consequently not separable by the current data model.

This is a correctness and performance problem. It retains and serializes events that must never be in
the authoring replay stream, and it prevents the replay owner from storing a compact committed intent
or transaction delta.

### P0: raw pointer motion pays the complete command/audit pipeline

Each handled viewport move becomes `EditorViewportEvent::PointerMoved` and is dispatched immediately
(`ui/retained_host/callback_dispatch/viewport/pointer_dispatch.rs:75-102`). Normal dispatch then:

- locks the sequence/revision state and advances the authoring revision;
- locks the command registry and scans for reverse event metadata when no operation identity exists;
- locks the complete workbench shell for execution;
- allocates a `Vec<EditorEventEffect>` and a JSON result object;
- deep-clones the successful record so it can both be returned and retained;
- allocates and fills `serde_json::to_vec(&record)` only to read its length, then discards the bytes;
- locks and indexes the journal, then probes every route and locks every matching inbox.

The main locations are `ui/host/editor_event_dispatch.rs:52-152`,
`ui/host/editor_event_execution/dispatch.rs:14-52`, `service/editor_event_service.rs:41-59`, and
`retention.rs:263-275`. At zero listeners, command reverse discovery, record cloning, JSON allocation
and journal indexing still occur. The 2026-08-16 commands review separately routes the reverse scan to
`PERF-MVP-645`; removing that scan alone does not remove the remaining synchronous stages.

Replacing `serde_json::to_vec` with a counting writer is not sufficient. It would remove one temporary
allocation but preserve a complete JSON traversal on every pointer event and keep encoded wire size as
the wrong proxy for typed heap retention.

### P0: revision allocation describes attempts, not successful authoring commits

`begin_event` advances revision before command lookup and execution (`editor_event_service.rs:41-79`;
`editor_event_dispatch.rs:59-60`). Failed and unchanged events retain an incremented `after_revision`.
At pointer frequency this turns an authoring generation into an input-attempt sequence and creates
false invalidation/cache dependencies. Event order and committed document revision need separate
monotonic authorities.

### P1: bounded final state does not bound fanout work

The August indexed retention and immutable-route fixes are correct: latest lookup no longer scans a
`VecDeque`, page cursors are applied before materialization, and the registry mutex excludes filtering
and inbox enqueue. However, one shared record still creates `L` filter probes and up to `L` separately
locked/indexed inbox mutations. A 1,000-listener by 1,000-event test proves eventual bounded state, not
bounded input-thread wall time.

The next design should measure a central immutable audit log plus generation-stamped subscriber
membership/cursors against the current per-listener inbox model. A shared log can store payload and
latest-state replacement once and give each listener an ack/lag cursor; subscription indexes or a
subscriber bitmap can avoid duplicating ordered trees. This is a measured candidate, not a mandated
container rewrite: sparse filters and different retention requirements must be included in the model.

### P1: two latest-state keys lose per-node state

`HoverNode` and `PressNode` use global latest-state keys while their payloads contain `node_path`
(`retention.rs:244-253,679-689`). If a listener previously observed node A as hovered/pressed, a
pending A-clear can be replaced by a B-set and leave A stale. `FocusNode` is legitimately single-owner;
hover and press require a node-qualified key or one complete interaction-state publication. Any
coalescing optimization must first preserve this state contract.

## Required unified architecture

1. EditorUI01 routes `RealtimeInput` directly to the owning interaction state. Move/resize/latest
   values may coalesce at the existing frame boundary; press/release/cancel edges remain ordered.
   Realtime input performs no command-registry reverse discovery and creates no replay record.
2. Editor08 produces a typed `CommandRoute` before execution. Editor03 begins a transaction only for
   a semantic authoring operation, assigns document revision only after a successful changed commit,
   and stores a compact operation/transaction delta with schema and target identity.
3. Execution returns one shared typed `EditorExecutionReceipt`: event/order id, commit revision,
   changed flag, fixed-size invalidation mask, optional external request and optional transaction id.
   UI effect application does not require JSON or a second full record clone.
4. Replay consumes only versioned committed operation/transaction entries with an explicit replay
   disposition. Raw input, presentation state, failures and external side effects are diagnostic data,
   never executable records. Legacy journals pass through an explicit compatibility decoder and reject
   ambiguous records instead of replaying everything.
5. Editor02 derives optional audit envelopes from receipts and explicit input-observation policies.
   In-memory charge uses a calibrated owned-heap contract; wire encoding happens once only at an ABI,
   persistence or remote boundary that actually needs bytes. Pages have count, byte and deadline caps.
6. Plugin listeners declare topic, delivery class and affinity. Main-affinity callbacks stay on the
   editor owner under a budget; non-main callbacks use the existing bounded Runtime11 scheduler with
   cancellation and generation checks. No private event thread or unbounded handoff is added.
7. EditorUI08 consumes typed invalidation generations. Stable/no-op input advances neither authoring
   revision nor presentation generation; exact hover/press deltas patch only the addressed state.

## Reference-engine evidence

- Unreal handles mouse motion in `FSlateApplication::ProcessMouseMoveEvent`, locates the widget path,
  and routes the pointer event with a dedicated cycle counter
  (`dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp:6478-6527`).
  `RoutePointerMoveEvent` operates on the current widget path, capture and drag state
  (`SlateApplication.cpp:5761-5829`). It does not first convert each move into an editor transaction or
  serialized audit record. Zircon should adopt that contract separation, not copy Slate internals.
- Unreal begins and ends undo history only through an explicit `FScopedTransaction`; construction can
  decline to transact, and destruction ends only an outstanding transaction
  (`dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/ScopedTransaction.cpp:9-38`). Its
  transaction object stores either an explicit custom change or serialized changed-object state
  (`dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorTransaction.cpp:91-144`). This supports
  committed semantic history rather than replaying raw UI input and external requests.
- Slate invalidation explicitly chooses slow versus cached fast paint and only processes the dirty
  update lists on the fast path
  (`dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp:356-424,1281-1379`).
  Zircon's receipt should likewise publish fixed typed invalidation domains, not require a full audit
  record to decide presentation work.
- Unreal's message router first resolves recipients, invokes `AnyThread` receivers directly and sends
  named-thread receivers through TaskGraph
  (`dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageRouter.cpp:118-182`). This is
  evidence for declared observer affinity. Its private router loop is not a reason to add a second
  Zircon scheduler or an unbounded queue.

These references establish ownership and stage separation. They do not provide a portable numeric
latency or power target; Zircon acceptance remains same-machine and source-bound.

## Dependency-ordered plan

### M0: measure the current pipeline

- Add per-stage counters/spans for input route, command lookup, shell mutation, effect allocation,
  record clone, result JSON, size JSON, journal indexes, route probes and each inbox wait/hold.
- Record event execution class, replay disposition, changed/failed outcome, authoring revision advance,
  bytes allocated/cloned/encoded and caller affinity.
- Run pointer 125/500/1,000 Hz and semantic command/replay matrices before changing ownership.

### M1: separate receipt, revision and invalidation

- Introduce the typed execution receipt and fixed invalidation mask.
- Split attempt/event order from successful authoring revision. Failed and unchanged input advances
  no document revision.
- Make retained-host effect application share/move the receipt; remove the success record deep clone,
  per-event JSON result object and vector effect allocation from realtime input.

### M2: hard-cut executable replay

- Define the versioned committed-operation/transaction replay schema and explicit disposition.
- Migrate valid semantic journal producers and replay tests, add a fail-closed legacy decoder, then
  delete replay of raw `EditorEventRecord` slices.
- Test project save/import/close, failures, transient UI, viewport input and no-op events as
  non-executable audit rows.

### M3: detach and bound audit/listener delivery

- Encode only for a real wire/persistence consumer and apply count+owned-byte+deadline admission.
- Benchmark central shared-log/subscriber-cursor delivery against per-inbox trees; choose from measured
  route density, ack, lag and retention behavior.
- Publish node-qualified hover/press state or a complete interaction-state generation before enabling
  latest coalescing for those topics.

### M4: product acceptance

- Use managed current-source tests, the repository product profiler and WPR/xperf on the same machine.
- Use RenderDoc only to confirm downstream dirty-region/draw parity after event invalidation changes;
  it cannot validate replay, locks, allocation, CPU latency or package power.
- Compare current and candidate source with identical warmup, measured and quiescence windows.

## Quantified acceptance

| matrix | required measurements | acceptance |
|---|---|---|
| pointer move/resize/scroll 125/500/1,000 Hz; idle/drag; 1/1M events | registry visits, shell/journal/listener locks, effect/result/record/JSON alloc+clone bytes, input-to-damage p50/p95/p99 | realtime route registry work=0; audit JSON=0 unless explicitly observed; stable move revision/present=0; ordered edges preserved |
| semantic commands 1/1k/100k; changed/no-op/failure | attempt/order ids, transaction begins/commits, revision advances, receipt bytes, invalidation domains | revision advances exactly once per successful changed commit; failure/no-op=0; one shared receipt and no redundant full clone |
| replay classes: transaction, raw input, transient UI, failure, save/import/close | decoded/executed/rejected rows, external side effects, final state/hash, allocations and wall p95 | only explicit committed semantic entries execute; ambiguous legacy fails closed; raw input/external side effects execute=0; deterministic final authoring state |
| listeners 0/1/1k/10k; match 0/50/100%; stalled 0/60 s | route probes, per-owner index writes, lock wait/hold, queue bytes/age, lag/drop/coalesce, caller wall | zero listeners observer work=0; accepted work is bounded by admission; one stalled listener cannot serialize unrelated delivery; ack/order preserved |
| payload 64 B/2 MiB/64 MiB; page 1/64/256 | typed heap charge, encoded/temporary/DTO/JSON bytes, rows visited/returned, deadline/remaining, RSS | no discarded full encoding; one final ABI encoding; count+byte+deadline caps hold; product polling never full-snapshots the journal |
| F4 product before/after | WPR CPU stacks, ready/running time, contention, context switches, allocations/RSS, package power; input-to-present p50/p95/p99 | stages separately attributable; same-machine deltas reported; basic input, command, undo/replay and plugin semantics pass; no invented Unreal budget |

## Static gates and blockers

- Source and external-test recount reproduced both fingerprints and exact inventories above.
- `rustfmt --edition 2021 --check` is green for all 36 production entry files; no source was
  formatted or changed.
- `git diff --check` is green for the owned documents, and all 21 explicit routing/reference paths
  resolve (`21/21`).
- The two owned documents have zero documentation-convention violations. The repository baseline is
  671 violations across 241 of 2,510 checked documents; this review does not claim ownership of that
  pre-existing debt.
- Plan audit and the `codex-performance-audit-20260814` session heartbeat are green after the
  documents were created.
- The 2026-08-15 retention report remains valid for indexed latest replacement, cursor-first pages and
  immutable route snapshots. This report supersedes any inference that those fixes make the complete
  event architecture acceptable.
- Managed Cargo remains blocked because `tools/build-editor.ps1:130` rejects approved D:/E:/F: target
  roots through its literal separator bug. See
  `failure-2026-08-15-build-editor-approved-root-separator.md`.
- No WPR/xperf or RenderDoc product capture can be source-bound until a current editor binary launches.
  No latency, power, replay-correctness or performance-improvement claim is made.
- No simple Rust edit was applied: the relevant source is foreign dirty, and replacing only the JSON
  length implementation would preserve the invalid input/transaction/audit ownership model.
