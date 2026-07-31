# Editor core jobs current-source review

## Status

- Result: `static_complete / dynamic_pending`.
- Review date: 2026-07-30.
- Primary owners: Editor14 for admission, lifecycle/progress and main-thread pump; Runtime11 for the shared scheduler and task-pool resource budget; Editor02 for the downstream message-bus queue; EditorUI08 for status projection invalidation.
- Accounting: keep `zircon_editor/src/core/jobs/**` in `pending.md`. Do not add it to `review.md` before current-source managed Cargo, deterministic queue/lock/allocation counters and F0/F4 product evidence are GREEN.
- Code disposition: no Rust source was changed. Twelve existing tracked modifications and the untracked current `tests/admission_scaling_contract.rs` were reviewed and preserved.

## Exact scope

| module | files | physical lines | inline tests | ordered path-and-raw-content SHA256 |
|---|---:|---:|---:|---|
| `zircon_editor/src/core/jobs/**` | 27/27 | 4,717 | 48 | `db5f4e6820b389717e7a60364e8eb455b5f34115e084e6816b52a898159bcbf3` |

The fingerprint streams each native workspace-relative path, a zero byte, the file's raw bytes and a zero byte in sorted path order into SHA256. All 27 files were read in full. Production reachability was followed through the retained tick pump/status projection, asset import/preview/editor refresh, autosave, Welcome probing, desktop/export-wizard jobs, viewport framework resolution and export output joins.

## Per-file review

| file | current-source performance result |
|---|---|
| `cancellation_token.rs` | Shared atomic cancellation is O(1), allocation occurs once per token lineage, and polling does not lock. Cancellation remains cooperative by design. |
| `category.rs` | The enum-owned `ALL` inventories and constant admission ranks prevent duplicated bucket lists. No runtime allocation occurs. |
| `context.rs` | Cloning shares the token and event sink. Every progress call still materializes an owned message before the sink decides whether it replaces an older value. |
| `error.rs` | Typed failures share an `Arc<dyn Error>` across clones. Panic text and submit/mutex diagnostics own strings only on error paths. |
| `event_sink.rs` | One event serially takes lifecycle, progress and queue mutexes while cloning the stable label. Progress also clones its message into the active snapshot before the original message enters the event queue. |
| `event.rs` | Every lifecycle row owns the repeated label; progress/failure rows additionally own message strings. This width feeds the unbounded lifecycle lane. |
| `id.rs` | Compact scalar identity only. |
| `job.rs` | Trait boundary only; it does not create a private executor. |
| `limits.rs` | Thumbnail=2, Export=1 and Import=runtime parallelism are bounded. Compile, Index, Play and Misc default to `usize::MAX`, allowing whole ready bursts to leave Editor admission at once. |
| `mod.rs` | Module wiring and exports only. |
| `mutex_group.rs` | Validated owned identifier. Parse is construction-time; cloning the string across many specs remains measurable but is not a per-frame operation. |
| `progress.rs` | One global `BTreeMap` mutex owns all active rows. Primary snapshot correctly clones one row, while full task-panel snapshot clones all. Progress messages are cloned per update; stable retained ticks still clone the primary row. |
| `pump.rs` | Correctly limits each call to 64 events/1 ms and coalesces progress by JobId. Started/terminal rows remain an unbounded `VecDeque`; no entries/bytes/oldest-age metrics exist, and each popped row separately locks then publishes to the message bus. |
| `shutdown.rs` | Compact unfinished-job DTO; cloning occurs only for explicit shutdown reporting. |
| `spec.rs` | Owns label, mutex group and dependency `Vec`. Repeated `.after()` uses linear `contains`, making construction O(D^2) for unusually dense dependency lists; normal small DAGs are unaffected. |
| `system/mod.rs` | Reuses Runtime's scheduler and preserves cancellation/panic/ticket contracts. Submit nests progress registration under the state lock. `promote` holds that lock across every `schedule_after` call and can dispatch every ready unlimited-category job. Pending cancellation/shutdown emit and finalize rows serially on the caller. Blocking `JobTicket::wait` is exposed, but current live UI flows poll; the wizard `finish_job` wrapper has no production caller. |
| `system/pending.rs` | Current ready selection is indexed and probes at most 3 priorities x 7 categories per pass; the old O(N) scan/front removal is gone. Maps/sets and dependency ownership remain unbounded because admission has no entry/byte/age policy. Drain allocates an id vector before removing all jobs. |
| `system/state.rs` | Category/mutex/dependency records are coherent and terminal history normally caps at 256. Pinned dependencies may exceed the cap; every prune then linearly searches the deque and removes an interior row under the state mutex, which can repeat during completion/cancellation storms. |
| `test_support.rs` | Tests share one Runtime scheduler to avoid multiplying worker pools. It is test-only and does not mask the production injected-scheduler contract. |
| `tests/admission_scaling_contract.rs` | Proves 1K/10K indexed bucket probes grow linearly. It intentionally accepts all jobs and records neither retained bytes nor age, so it does not prove bounded admission. |
| `tests/background_storm_contract.rs` | Exercises 1K Thumbnail jobs, quota=2, event order and the 64/1 ms pump. Wall times are observational with no numeric SLA; queue bytes/age and unlimited-category transfer are not covered. |
| `tests/mod.rs` | Test module wiring and a small recording job only. |
| `tests/progress_contract.rs` | Covers shared/sorted snapshots, latest progress, terminal removal, cancel and shutdown visibility. It does not count stable-tick clone/invalidation work or global-lock contention. |
| `tests/pump_contract.rs` | Covers main-thread ownership, count/time deferral, latest progress, escaped context and concurrent pump order. It drains with an unbounded test budget in several cases and has no queue memory/age assertion. |
| `tests/scheduling_contract.rs` | Broadly covers ticket, dependency, quota, mutex group, priority, cancellation, panic and shutdown semantics. Dense dependencies, unlimited-category bursts and pinned-history algorithmic counters are missing. |
| `tests/thread_ownership_contract.rs` | Test-only lexer/guard scans production sources and rejects direct/aliased bare thread owners. Its 973-line parsing cost is confined to validation and protects the shared-scheduler architecture. |
| `ticket.rs` | `try_take` is a short result-slot lock and current UI owners poll it. `wait` blocks the caller and must remain worker/tool/shutdown-only; there is no current production retained-tick caller. |

## Corrected and remaining tasks

### PERF-MVP-018: pump budget exists; lifecycle retention is still unbounded

The previous MPSC/drain-to-empty description is stale. Workers now push into a mutex-owned queue, progress keeps one latest row per JobId, and the retained tick pumps at most 64 rows or 1 ms. Accepted Started/terminal edges are still lossless without a capacity reservation, so a completion storm can outpace the tick indefinitely. Bind lifecycle capacity to submission admission: reserve edge/byte budget before accepting a job, reject/backpressure before the hard bound, never drop terminal edges for accepted work, and expose remaining plus oldest age through the pump and Editor02 bus.

### PERF-MVP-020: indexed selection is not bounded scheduling

Ready selection is now at most 21 bucket probes, but four categories are unlimited. `promote` holds the Editor state mutex while repeatedly calling Runtime `schedule_after`, so a burst can move all jobs into the shared scheduler and hide its memory/age from Editor pending metrics. Reserve a bounded dispatch batch under the lock, call the scheduler outside it, then generation-safely install or roll back handles. Apply entry, estimated-byte and oldest-age limits to every category/priority, not only Thumbnail/Export/Import concurrency.

The same task owns event width and terminal history. Freeze label/spec/message owners so one progress update does not retain two strings and one lifecycle event does not clone its stable label. Replace pinned terminal deque scanning/interior removal with dependency reference counts plus an evictable ordered index. Preserve the existing late-dependency and mutex-tail semantics.

### PERF-MVP-017: stable status projection must become zero work

`primary_snapshot` removed the all-active clone, but the retained tick still clones one row, formats task/detail strings and invalidates presentation every frame. Publish a primary snapshot generation and only rebuild/refresh when identity or progress changes. The explicit task-panel API can retain a bounded/paged full view.

### Blocking API boundary

`JobTicket::wait` and `EditorJobSystem::join` are intentionally blocking. Current viewport, Welcome, desktop export and wizard tick paths use `try_take`; export output joins execute inside export work. The public wizard `finish_job` can block but has no production caller. Add a source/affinity guard before wiring it so retained/main-thread code cannot synchronously wait for arbitrary jobs.

## Acceptance plan

- Admission/dispatch: jobs `1/1K/100K`, categories/priorities all combinations, dependencies `0/1/10K`, workers `1/64`. Record Editor and Runtime queued entries/estimated bytes/oldest age, dispatch batch, scheduler calls under state lock, submit/promotion p50/p95 and RSS. Every category must have a hard bound and scheduler calls under the Editor mutex must be zero.
- Events/progress: events/job `0/1/1K`, label/message `0/4KiB/1MiB`, producer threads `1/16`, consumer stall `0/60s`. Record lifecycle/progress/bus entries+bytes+age, coalesced/rejected rows, label/message clone bytes, three-lock wait/hold and pump p50/p95. Accepted terminal loss must be zero and memory must remain bounded.
- History/dependencies: retained records `0/256/100K`, pinned ratio `0/50/100%`, cancel/shutdown bursts `1/100K`. Record dependency refcount changes, history probes/moves, state-lock hold and terminal rows. Prune must be O(1)/O(logN) per evictable row with no interior deque removal.
- Status/tickets: active jobs `0/1/1K`, stable ticks `1/100K`, progress `0/1K Hz`. Record snapshot/string clones, formatting and presentation invalidation/rebuild. Stable generation must do zero projection work; main/retained thread blocking waits must be zero.
- Run current-source managed jobs tests and F0/F4 import, autosave, Welcome, viewport and export storms. RenderDoc is not applicable to this CPU scheduling slice; viewport GPU evidence remains with Render16/17.

## Reference check

- Bevy `dev/bevy/crates/bevy_tasks/src/usages.rs` separates frame-critical compute, multi-frame async compute and I/O owners, and bounds main-thread local ticking to 100 attempts per pool. Zircon should retain the shared Runtime scheduler but express equivalent resource class and bounded main-thread consumption instead of an unlimited category.
- Unreal `TaskGraphInterfaces.h` makes target thread and task/thread priority explicit and offers completion triggering back to named threads. Zircon's single retained pump is the correct owner; completion must enter it through bounded admission rather than implicit queue growth.
- Godot `editor/file_system/editor_file_system.cpp` groups reimport work and waits at an explicit worker-group boundary. Zircon should use typed batch tickets and generation completion, not per-item UI waits.

## Static gates executed

- Read all current 27/27 Rust files and the listed production caller chains.
- `rustfmt --edition 2021 --check` passed for all 27 files.
- `git diff --check -- zircon_editor/src/core/jobs` passed. Existing tracked/untracked changes were not rewritten.
- Source inventory was 27 files, 4,717 physical lines and 48 inline tests at fingerprint `db5f4e6820b389717e7a60364e8eb455b5f34115e084e6816b52a898159bcbf3`.
- No managed Cargo, allocator/RSS/lock scale run, WPR F0/F4 product trace or independent dynamic review ran. The module remains pending and `review.md` is unchanged.
