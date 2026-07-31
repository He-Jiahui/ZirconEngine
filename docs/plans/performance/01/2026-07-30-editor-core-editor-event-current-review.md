# Editor core editor_event current-source performance review

## Status

- Review state: `static_complete / dynamic_pending`.
- Production scope: `zircon_editor/src/core/editor_event/**`, **32/32** current Rust files, **2,164** lines, **4** inline tests, **0** ignored.
- Production fingerprint (ordered `path + NUL + raw content + NUL` SHA-256): `e4aaebb4071cc5f86ba2a9e9a3bc8f2a3bda002051cf7701d51c54c79f0da17d`.
- External test scope: `zircon_editor/src/tests/editor_event/**`, **22/22** current Rust files, **6,671** lines, **117** tests, **0** ignored.
- External-test fingerprint (same algorithm): `092834c672e5688189da39ca8566affc38a0dda31f99c62b795a6da0b0efffae`.
- Production callers checked: retained-host event dispatch and runtime-access journal projection.
- Code disposition: no Rust source changed. Existing dirty/untracked source was read at its current hash and left untouched.

This record supersedes the old 21-file external-test count for this subtree. It does not move either subtree into `review.md`: current-source managed Cargo, scale counters, F4 traces, and independent acceptance are still missing.

## Current architecture verified

- Journal and listener inboxes share one `Arc<SharedEditorEventRecord>` rather than cloning the event once per listener.
- Durable replay, frame-local, and latest-state queues have separate entry/byte/age budgets. The current implementation is bounded, so the old blanket "unbounded journal/listener" diagnosis is stale.
- Sequence, journal, and listener state use separate mutexes. Filter normalization occurs when the filter is installed, not for every event.
- Acknowledge accounts removed bytes in one pass, and status reads first/last retained sequence from queue diagnostics without materializing records.
- Existing tests cover 10k paused-listener events, 1k latest-state coalescing, byte/age budgets, lag/ack/order, and shared `Arc` ownership.

## Remaining P0 bottleneck: PERF-MVP-067

1. `SharedEditorEventRecord::new` serializes the complete record into a counting writer for every event solely to obtain exact retained bytes. This traverses every JSON/string field before any actual wire or persistence consumer serializes it.
2. Latest-state coalescing linearly scans its `VecDeque`; removing a middle entry shifts the tail. The queue is bounded, but this work is repeated in every matching listener inbox.
3. `EditorEventService::record` holds the global listener-registry mutex while iterating every listener, applying filters, pruning retention, coalescing, and enqueueing. Listener count and inbox work therefore extend one shared critical section.
4. Retained-host success paths call `record(record.clone())` and return the original, deep-cloning the full event record before the service wraps it in `Arc`.
5. `EditorEventRetentionStore::records` clones all retained `Arc`s from three queues and fully sorts them on every journal/listener query. `QueryDeliveriesSince` applies its cursor only after this merge and sort.
6. Listener queries then deep-clone listener id, source, operation strings, arguments, and result into owned deliveries; the control response materializes those rows into JSON again. Journal snapshots likewise deep-clone every complete event record.

The public journal/query paths are not proven per-frame product hot paths, so this review does not label them as such. They remain a scale amplifier for polling, diagnostics, plugins, and stalled consumers.

## Optimization contract

- Cache encoded size on the actual shared encoded owner when serialization is required, or derive a construction-time accounting contract without a second complete payload traversal. Keep hard byte budgets exact enough to preserve admission semantics.
- Publish an immutable listener route/filter generation under a short registry lock. Enqueue through stable per-listener inbox owners after releasing that global lock; do not call foreign code and do not invent a private thread pool.
- Maintain an indexed latest-state key or a representation that replaces a key without scanning/shifting the whole queue. Preserve sequence, coalesced count, and class budgets.
- Let successful dispatch move or share one record owner instead of deep-cloning the record merely to retain and return it.
- Query by cursor before materialization. Merge the three already sequence-ordered queues with a bounded iterator/page rather than cloning and sorting the full retained set.
- Add count, bytes, and deadline page limits with `remaining`, oldest age, and lag information. Return shared/borrowed rows inside the process and create owned JSON only at the final ABI boundary.
- Preserve durable/frame/latest classification, ack count, dropped/coalesced diagnostics, replay/undo ordering, poison recovery, and current filter semantics.

## Dynamic acceptance matrix

| Dimension | Cases | Required counters / assertions |
| --- | --- | --- |
| listeners and events | 0, 1, 1k, 10k | global registry lock wait/hold, listener visits, per-owner enqueue, queue entries/bytes/oldest age, p50/p95/RSS |
| payload | 64 B, 2 MiB, 64 MiB | serde size traversals, encoded owners, record/delivery/JSON clone bytes; size traversal `<= 1/event`, redundant full record clone `= 0` |
| routing | filter match 0%, 50%, 100%; enabled/disabled mutation | route-generation rebuild/publish, filter probes, stable-generation allocations; no stale route apply |
| retention | durable, frame-local, latest-state; stalled 0/60 s | coalesce visits/shifts, dropped/coalesced/lag sequences; latest replacement near O(1), all hard budgets respected |
| polling | cursor at 0%, 1%, 99%; page count/bytes/deadline | rows visited before cursor, merge/sort count, returned rows/bytes, delivery/JSON materialization; no full-set sort/materialization |
| contention | 1 and 16 producer/control threads | callback-in-lock, global/per-owner lock wait/hold, ordering/loss/dup; slow listener must not serialize independent owners |

Managed gates must include focused `editor_event` lib tests, the 117 current external tests, 1k/10k storm counters, and an F4 retained-host WPR trace. This non-rendering slice does not justify RenderDoc; RenderDoc remains required at the downstream viewport/render boundary.

## Current compile boundary

Static source inspection found two current `EditorEventRecord` literals that omit the newly required `binding_path`, `transaction_id`, and `save_generation` fields:

- `zircon_editor/src/core/editor_event/listener/registry.rs` inline test.
- `zircon_editor/src/tests/editor_event/retention.rs` helper.

No managed Cargo command was run, so this is a static compile-boundary finding, not a RED test result. These foreign dirty files were not edited. Source formatting is currently clean for **26/32** production files and **7/22** external test files; the remaining drift is mainly import/re-export order plus `dispatcher.rs` indentation.

## Reference-engine routing

- Bevy `Messages`/`MessageCursor` keeps a consumer cursor and derives pending/missed counts from monotonic message counts. Zircon should adopt cursor-first paging and O(1) lag accounting, while retaining its three explicit replay classes and larger audit window.
- Godot `CallQueue` exposes bounded page usage and maximum-buffer telemetry. Zircon should mirror explicit admission/usage counters, while preserving per-listener ack and replay semantics.
- Fyrox editor messaging uses an unbounded standard channel in the inspected path; it is not a suitable retention/backpressure model for this contract.

## Static gates

- Full current production and test manifests were read file by file.
- Relevant retained-host production callers and all three reference implementations above were inspected.
- Planned documentation validation: `git diff --check`, exact task/link/source-fingerprint guards, and proof that `review.md` remains unchanged.
- No Cargo reservation, code commit, or RenderDoc capture belongs to this static-only slice.
