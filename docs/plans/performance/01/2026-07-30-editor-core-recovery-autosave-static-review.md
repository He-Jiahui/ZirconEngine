# Editor core recovery autosave current-source static review

## Status

- Result: `static_complete / dynamic_pending`.
- Review date: 2026-07-30.
- Accounting: keep `zircon_editor/src/core/recovery` in `pending.md`; do not add it to `review.md` before current-source managed Cargo, scale/filesystem counters and F4 recovery evidence are GREEN.
- Code disposition: no Rust source was changed. The complete recovery directory is external untracked work and was preserved.

## Exact scope

| module | files | physical lines | tests | ordered path-and-raw-content SHA256 |
|---|---:|---:|---:|---|
| `zircon_editor/src/core/recovery` | 3/3 | 824 | 10 | `61918343ac96d01542781e37c2eb6a4dc886fc482d3c6eb43f42add6d701a1a2` |

The exact files are `autosave.rs`, `mod.rs` and `tests.rs`; all three were reread at their current contents while standardizing this record to the repository-wide path-plus-raw-content fingerprint. Consumer tracing covered Editor17's M2.1 output, the Editor14 admission/save-mutex failure, Editor16 session-lock handoff, the shared runtime atomic writer and all source references to the recovery API. The behavior findings below did not drift.

## Confirmed performance boundary

1. There is no production `AutosaveScheduler`, `AutosaveStore` or `write_snapshot` consumer outside tests. The existing Editor14 failure explicitly records that job admission/terminal wiring is absent. Costs below are therefore pre-wiring capacity risks, not measured current editor stalls.
2. At a due interval, `AutosaveScheduler::plan` clones every dirty document ID into a `BTreeSet` and then a sorted `Vec`. The planned Editor14 adapter is per document. Without an admission window, a project with many dirty documents can enqueue all jobs and, if payload production occurs before admission, retain many complete serialized documents simultaneously.
3. `AutosaveStore::write_snapshot` receives a complete byte slice. Before writing it enumerates every file in the document directory to reject a reused numeric sequence; after the durable write it enumerates the same directory again, allocates owned filename strings and paths, builds a `BTreeMap<u64, Vec<PathBuf>>`, then removes old sequences. Recognized sequences are normally small, but unrelated/orphan entries make both scans and transient allocations scale with directory contents.
4. Recovery owns another temporary-file/write/`sync_all`/rename/parent-sync implementation even though the runtime Foundation exports `atomic_write` and Editor workspace persistence already consumes it. The local writer also prevents shared streaming, fault-injection and durability counters from covering autosave.
5. The in-process sequence reservation set is bounded by concurrent writes and released by RAII; callback execution is intended for `Background/Misc` under the save mutex. These are sound boundaries to preserve.

## Plan and acceptance

- `PERF-MVP-592` / Editor17 + Editor14 + Runtime11: admit a bounded number of document tickets, then lazily capture one immutable document generation; stream serialization into a bounded staged writer; publish only after the shared atomic commit. Never build every dirty payload before queue admission.
- Maintain a tiny durable per-document sequence/retention manifest or fixed ring slots so steady writes do not enumerate the directory. Startup/recovery performs one bounded reconciliation of snapshots and orphan staging files; sequence conflict, session-lock and crash semantics remain explicit.
- Matrix: dirty documents `1/100/10k`, document bytes `1KiB/64MiB/1GiB`, directory entries/orphans `3/1k/100k`, interval `1s/300s`, filesystem latency `0/10ms/2s`, writers `1/16`. Record pre-admission serialized docs, full payload owners, buffer/RSS, directory visits, filename/path allocations, write/fsync bytes, queue entries/bytes/oldest age and p50/p95.
- Require queued payload count zero, peak extra payload memory bounded independently of document size, steady-state directory scans zero, fixed retention owner count, UI caller filesystem wall zero and source digest unchanged. Verify sequence races, rotate failure, crash old/new, cancel/shutdown and kill/restart restore.

## Reference check

- Unreal `dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/PackageAutoSaver.cpp` tracks dirty package sets and advances `AutoSaveIndex` modulo the configured maximum backup count. That demonstrates bounded slot rotation without rediscovering the entire backup directory on every save. Zircon should keep its background job and durability contracts rather than copying Unreal's synchronous editor presentation path.

## Static gates executed

- Read 3/3 exact recovery files plus the listed owner/consumer and shared-writer evidence.
- `rustfmt --check --edition 2021` passed all three current files.
- No managed Cargo, filesystem fault/latency benchmark, WPR F4 product trace or kill/restart recovery run executed. RenderDoc is not applicable to this non-rendering slice. The module remains pending.
