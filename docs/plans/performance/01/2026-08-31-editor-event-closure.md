---
related_code:
  - zircon_editor/src/core/editor_event/**
  - zircon_editor/src/tests/editor_event/**
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/core/editor_message/**
  - zircon_editor/src/core/runtime_event_consumer/**
related_plans:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
write_scope: []
status: pending
---

# Editor event closure

This is a current-source static closure for the editor event stamp, journal,
listener and replay boundary. It remains pending because the current workspace
does not produce a green editor Cargo test; compilation stops in unrelated
`zircon_runtime` production errors. No Rust source was changed and this scope
does not enter `review.md` from static evidence alone.

## Scope and source state

- `zircon_editor/src/core/editor_event/**`: 40 Rust files, 3,470 physical
  lines, 3,120 nonempty lines, 112,531 bytes, 23 tests, five ignored tests and
  seven include sites. Sorted path/raw-content SHA256:
  `3fff583ec87012f824bcd83954b612b847204e95f000841e9def62026c257672`.
- `zircon_editor/src/tests/editor_event/**`: 32 Rust files, 9,600 physical
  lines, 8,813 nonempty lines, 345,818 bytes, 156 tests and 26 include sites.
  Sorted path/raw-content SHA256:
  `1bfb15ac9a7cf0e8dc46e08869045fdeabeefaa683847fe3715225fd29f1c35`.
- The current tree contains foreign listener/journal/retention splits and
  source-shape/performance fixtures. Those changes are preserved. Ignored
  performance fixtures were not executed and no benchmark result is claimed.
- Isolated `rustfmt --check --edition 2024 --config skip_children=true`
  passes 46/72; the 26 failures are current foreign import/assert formatting.
- `cargo test -p zircon_editor --lib editor_event --no-run
  --message-format=short` reaches `zircon_runtime` and fails with 29 existing
  workspace compile errors before editor tests; no dynamic pass is claimed.

## Positive work to preserve

- `EditorEventRetentionPolicy` separates durable replay, frame-local and latest
  state budgets by record count, encoded bytes and age. Retention queues index
  event sequence, delivery cursor and expiry, coalesce latest-state keys, and
  expose bounded delivery pages with acknowledgement cursors.
- `SharedEditorEventRecord` lets the journal and listener inboxes share one
  immutable payload. Listener filters normalize prefixes/groups/sources once;
  delivery routes are rebuilt as an immutable `Arc` snapshot, and listener JSON
  projection occurs after the listener handle is released.
- Journal snapshots cache the last shared-record generation, replay validates
  expected failures, and current tests cover ordering, retention drops,
  latest-state composition, page limits, acknowledgement and listener
  reconfiguration.

## Retained findings

1. `SharedEditorEventRecord::new` calls `serde_json::to_vec(&record)` for every
   event solely to obtain `encoded_bytes`, allocating and traversing the full
   JSON payload before dropping the buffer. Pointer/resize/hover events can
   therefore pay a complete serialization allocation even when latest-state
   coalescing immediately removes the prior row. Use a counting serializer or
   an admitted immutable encoded owner, but count the work once per generation.
2. The normal successful host dispatch builds an `EditorEventRecord`, then
   clones the complete record to make `journal_record` before passing it to the
   service. The record contains event payloads, effects, optional JSON
   arguments, operation strings and result values. This is a second full
   payload authority on the high-frequency editor path; journal retention
   policy should project a borrowed/Arc record or move a single candidate with
   an explicit result-redaction view.
3. `EditorEventJournalStore::snapshot` obtains the journal mutex and, whenever
   the shared generation changes, clones every retained record into an
   `Arc<[EditorEventRecord]>`. `EditorEventRetentionStore::records` first merges
   the three queues, and public journal callers receive the complete retained
   set rather than a bounded page. This is valid explicit replay/export work,
   but its lock hold, clone bytes and deadline are not proposed and can stall
   event recording at the 16,384-record/64 MiB journal limit.
4. `EditorEventService::record` pushes the shared record into the journal, then
   clones the route snapshot and sequentially locks each listener inbox. Each
   route has its own bounded queue, but there is no one proposal reserving
   producer serialization bytes, all listener enqueue work, callback/output
   capacity and terminal delivery state. A slow listener or many filters can
   add producer latency even though local retention is bounded.
5. Listener status and page controls are bounded by the retention policy and a
   256-row page, but page projection materializes JSON values and clones every
   dynamic record field. Full listener lists and journal snapshots have no
   count/byte/deadline admission at their public boundary. Diagnostics modes do
   not gate these allocations; a default or polling caller can accidentally
   request full retained projections.
6. Event, delivery-cursor and revision counters use `saturating_add` in the
   service and retention store. At `u64::MAX`, subsequent events can reuse the
   same event/sequence/delivery identities, defeating replay, coalescing and
   cursor ordering. This is a latent exhaustion defect, not a measured current
   cost; it requires checked, session-qualified non-repeating generations.
7. The editor event, editor message, runtime-event consumer and gateway paths
   maintain separate queue/callback lifetimes. A bridge can retain one Arc
   record while another consumer drops or replaces its delivery metadata. The
   current local budgets do not prove exactly-once terminal outcomes across
   those boundaries, especially on listener replacement, callback panic,
   session shutdown or cross-queue backpressure.

## Architecture handoff

1. Compile one `EditorEventGeneration` with session identity, checked event and
   delivery generations, event schema/retention class, encoded/object byte
   quote, listener fan-out count, callback capacity and deadline. Cap+1 rejects
   before serialization or queue mutation.
2. Store one immutable record owner. Compute encoded size with a counting pass or
   retain one shared encoded slab, then let journal/listener/message consumers
   borrow or clone only a handle. Result redaction is a view, not a deep record
   clone.
3. Keep journal and listener delivery on cursor-indexed pages. Journal/export
   snapshots must be explicit bounded operations with lock-free or short-lock
   generation capture; full replay/export owns its own byte/deadline lease.
4. Make listener fan-out a scheduled batch: preflight every route, apply an
   admitted enqueue budget, and publish Delivered/Dropped/Backpressured/
   Cancelled/Fault terminal facts exactly once. Slow callbacks execute on an
   affinity/deadline-aware scheduler outside producer and inbox locks.
5. Converge event, message, runtime-event and gateway bridges on one
   session-qualified delivery lease. Replacement, shutdown, panic and stale
   generation each settle the lease and cannot leave an orphan payload.
6. Diagnostics Disabled performs no serialization, full snapshot, label or
   callback projection. Counters/Sampled/Full borrow dense event/listener IDs
   and admit only requested page/row bytes.

## Evidence and gates

The local Unreal messaging reference constructs a dedicated router thread for
each message bus (`dev/UnrealEngine/Engine/Source/Runtime/Messaging/Private/Bus/MessageBus.cpp`
20-27); `FMessageRouter::Run` processes commands and delayed messages before
waiting on a pooled synchronization event (`MessageRouter.cpp` 53-61). Slate's
fast invalidation root explicitly includes TaskGraph support. These sources
support independent scheduling and event-driven waiting, not a direct Zircon
ABI copy.

M0 adds RED tests for counting-vs-encoded bytes, successful host record clone
bytes, journal snapshot lock/deadline, listener fan-out cap+1, cross-queue
replacement and identity exhaustion. M1-M3 implement one record/generation
owner, bounded cursor pages and terminal delivery leases. M4-M5 add managed
scale and F4 evidence before acceptance.

Hard gates: current-source Cargo builds; cap+1 performs zero serialization or
enqueue work; record payloads have one owner; journal/listener projections are
bounded and generation-qualified; stale/replaced/panicking/shutdown consumers
terminalize exactly once; IDs never repeat; Disabled diagnostics allocate no
event reports; and measured diagnostics match serialization, clone, lock, row,
byte, drop and callback work. No benchmark artifact or micro-fix is warranted
before these ownership corrections.
