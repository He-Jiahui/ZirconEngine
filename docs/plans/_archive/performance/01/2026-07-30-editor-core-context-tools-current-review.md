# Editor core context and tools current-source review

## Status

- Result: `static_complete / dynamic_pending`.
- Review date: 2026-07-30.
- Milestone owner: Editor08 M3.2, with Editor05 scene-mode and Editor15 export-wizard consumers still pending.
- Accounting: keep both directories in `pending.md`; do not add either to `review.md` before current-source managed Cargo, scale/fanout counters and real consumer F4 evidence are GREEN.
- Code disposition: no Rust source was changed. Existing modified/untracked source was preserved.

## Exact scope

| module | files | physical lines | tests | ordered path-and-raw-content SHA256 |
|---|---:|---:|---:|---|
| `zircon_editor/src/core/context` | 4/4 | 632 | 4 | `08532c23711d642553ea24928bc889c334a581820faba48e1a3d067f26376c2f` |
| `zircon_editor/src/core/tools` | 4/4 | 1,336 | 14 | `6224ccf477a2eb12b4ec6df06aa541dad4bca065f4a838d4b245c3424e340bae` |

The fingerprint streams each workspace-relative path, a zero byte, the file's raw bytes and a zero byte in sorted path order into SHA256. Context files are `builder.rs`, `editor_context.rs`, `mod.rs` and `tool_scheduler.rs`; tools files are `mod.rs`, `scheduler.rs`, `tests.rs` and `tool_id.rs`. All eight were read in full. `core/mod.rs` mounts both modules and `EditorContextBuilder` creates one service, but tracked production source has no `EditorContext::tools()` acquire/release consumer outside definitions and tests.

## Confirmed performance boundary

1. The owner shape is sound. `ToolSchedulerService` keeps exactly one `Arc<Mutex<ToolScheduler>>`; every mutation produces a finite report under the lock, releases the lock, and only then publishes lifecycle messages. Bus callbacks therefore cannot re-enter the scheduler mutex.
2. Capacity is already explicit: exactly three `ExclusiveResource` variants, default queue cap 64, set-queue cap 64 and `ToolId` cap 128 bytes. `ToolResourceSet` contains at most three entries. The BTreeMap and linear duplicate/withdraw searches are consequently bounded by configuration rather than editor lifetime; no unbounded scheduler or event-history store was found.
3. `release_all` first clones every matching set request, then calls `withdraw_set` for each. Each withdrawal linearly searches and removes from the same VecDeque, so an adversarial 64-entry same-tool set queue can do quadratic comparisons/moves. This remains a bounded shutdown/cancel path and has no production caller today. Measure the cap before replacing the simple queue; if it exceeds the UI budget, use one stable retain/rebuild pass rather than a new index with harder ordering semantics.
4. `ToolSchedulerService::publish_events` reparses the static `TOPIC_TOOL` into a new owned `String` on every API call, clones that topic per event and clones each event into the bus while retaining the original report. `EditorMessageTransactionEventSink` already caches its built-in transaction topic at construction. Tool service should do the same or use a typed built-in topic slot; downstream fanout/lock work remains PERF-MVP-019 rather than a private bus or worker.
5. `holder` clones only an `Arc<str>` ToolId. Atomic resource sets preserve global FIFO and avoid partial holds. No evidence supports replacing the fixed small maps/queues with a heavyweight general scheduler before Editor05/15 consumers exist.

## Plan and acceptance

- Editor08 M3.2 keeps one authoritative service. Scene mode and export wizard must acquire through it and may not create host-local schedulers, queues or workers. Waiting export must not launch a process before its complete resource set is activated.
- Cache the built-in tool topic once per service or represent built-in topics with a shared typed identity. Continue publishing after unlock and preserve exact deactivated/activated order; integrate fanout/backpressure measurement with PERF-MVP-019.
- Matrix: queue `0/1/64`, tools `1/64`, set size `1/3`, operations `1/1M`, same-tool set requests `0/32/64`, subscribers `0/1/100/1k`, stall `0/60s`, threads `1/16`. Record queue comparisons and moved rows, `release_all` passes, topic parses/owned bytes, event clone bytes, scheduler and bus lock wait/hold, publish wall, inbox entries/bytes/age and p50/p95.
- Require queue/RSS to remain within configured caps, scheduler lock-held publish count zero, built-in topic parse no more than once per service lifetime, no process launch while queued, no partial resource holder and no duplicate lifecycle event. Only replace the bounded VecDeque algorithm if the 64-cap p95 exceeds the Editor08 UI budget.

## Cross-engine evidence and intentional divergence

- Unreal `dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/EditorModeManager.h` and `Private/EditorModeManager.cpp` keep a centralized active-mode owner; `ActivateMode` rejects duplicate activation and removes incompatible modes. `LandscapeCoreWorkflowTests.cpp` activates through that owner and asserts the mode is active, while `AutomationEditorCommon.cpp` explicitly deactivates modes before creating a new map.
- Fyrox `dev/Fyrox/editor/src/interaction/mod.rs` intentionally keeps interaction modes in an ordered `Vec`, documenting that the current maximum is about five and linear search is faster at that scale. This supports Zircon's fixed-small-set implementation rather than premature indexing.
- Zircon intentionally diverges by supporting one atomic multi-resource FIFO because a modal export wizard and viewport scene mode can contend across resource families. That queue must remain bounded and deterministic; Unreal/Fyrox mode activation does not supply the cross-resource waiting contract.

## Static gates executed

- Read context 4/4 and tools 4/4 at the recorded fingerprints, all 18 inline tests, Editor08 M3.2 owner records, current mount points and production call graph.
- Cross-engine evidence includes Unreal implementation plus automation/workflow tests and Fyrox's Rust interaction-mode container.
- `rustfmt --check --edition 2024` is not GREEN: import ordering differs in three context files and three tools files; `context/mod.rs` and `tools/scheduler.rs` are clean. The external source was not rewritten without a source lease.
- No managed Cargo, allocation/queue/fanout scale run, WPR F4 product trace or RenderDoc capture ran. RenderDoc is not applicable to this non-rendering slice. Both modules remain pending.
