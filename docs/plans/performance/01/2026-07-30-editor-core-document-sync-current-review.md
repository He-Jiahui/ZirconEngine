# Editor core document and sync current-source review

## Status

- Result: `static_complete / dynamic_pending`.
- Review date: 2026-07-30.
- Accounting: keep both directories in `pending.md`; do not add either to `review.md` before current-source managed Cargo, scale counters and F0/F4 product evidence are GREEN.
- Code disposition: no Rust source was changed. Both directories are external untracked work and were preserved.

## Exact scope

| module | files | physical lines | tests | ordered path-and-raw-content SHA256 |
|---|---:|---:|---:|---|
| `zircon_editor/src/core/document` | 2/2 | 387 | 8 | `065a1b1be06d7abdd5875fd1aea045d992c8f36813fe37620cb1ac73cfdc907e` |
| `zircon_editor/src/core/sync` | 3/3 | 450 | 8 | `f14f186ab83503926e4e923b1dabc68044d74ca46b34f76b1c921f743efb45a3` |

The fingerprint streams each workspace-relative path, a zero byte, the file's raw bytes and a zero byte in sorted path order into SHA256. Document files are `lifecycle.rs` and `mod.rs`; sync files are `mod.rs`, `watch_map.rs` and `watch_map/tests.rs`. All five were read in full at their current contents. The document row supersedes the earlier 202-line/4-test fingerprint after `lifecycle.rs` gained bounded-retention behavior and four scale/identity tests. Consumer tracing covered EditorManager's direct/startup/prepared project open, successful save and committed close paths; no production `WorldWatchMap` consumer was found outside the sync definitions and tests.

## Confirmed performance boundary

1. The earlier query-path allocation and triple-root ownership are no longer current. `active_document` stores only `DocumentId`, `ids_by_root` is the single path owner, and `activate`/`close`/`save` borrow the caller's `Path` before a miss. Known-root no-op/save/close therefore create no additional root body.
2. Closed-root retention is now hard bounded by `MAX_TRACKED_DOCUMENT_ROOTS = 1_024`. Reopening an evicted root rederives the stable FNV-based id; current tests cover 100K-root churn, owner-count bounds, close/open ordering and collision stepping. This removes the old unbounded RSS claim, but those tests were not dynamically executed in this pass.
3. Residual churn remains under the state mutex. Every new root scans all retained ids for an id collision; after the cap, `trim_closed_roots` linearly finds a non-active BTreeMap row, clones its `PathBuf` and removes it. The work is hard bounded to 1,024 rows, but 100K distinct-root automation can still perform roughly cap-sized scans per activation. `to_string_lossy` hashing can also allocate for non-UTF-8 paths. Event vectors remain at most `Closed` plus `Opened`, and callers publish after the authority returns; those boundaries should be preserved.
4. `WorldWatchMap::project` correctly visits only dirty tokens, directly probes the token map and borrows the bound view when coalescing. It still constructs `seen`, `duplicates` and `unknown` BTreeSets for every batch and finally allocates two diagnostic vectors. This is the already recorded PERF-MVP-468 normal-versus-malformed batch boundary, not a new root cause.
5. Current sync source has no production consumer, so its cost is pre-wiring capacity risk. Editor02 must require canonical bounded runtime batches before wiring it into the editor thread; a test-only microbenchmark must not be reported as current F4 product impact.

## Plan and acceptance

- `PERF-MVP-593` / Editor01: preserve the current single path owner, active id and 1,024-row hard cap. Add collision-visit, trim-visit/path-clone and mutex counters before introducing another index. If the 100K-root gate exceeds its visit/lock budget, keep an insertion-order closed-root queue plus direct id occupancy index so eviction and collision lookup do not rescan the cap; do not restore a reverse map with a second path body.
- Roots `1/1k/100k`, path bytes `16/4KiB`, operations `1/1M` and threads `1/16`: record path allocations/clone bytes, body owner count, map nodes/RSS, collision/trim visits, mutex wait/hold and p50/p95. Require zero path allocation for no-op/save/close, at most one path body per known document, retained roots at most 1,024, stable id/order after eviction and bounded visits per activation. Accept the current simpler map if measured p95 and visits pass; only then close PERF-MVP-593 after Cargo/F0/F4 evidence.
- `PERF-MVP-468` / Editor02 remains the sync owner: canonical batches carry count/bytes/age budgets and sorted-unique evidence; the normal path performs no three-tree diagnostic rebuild, while malformed input uses a separately budgeted diagnostic slow path. Preserve borrowed view coalescing and deterministic unknown/duplicate reporting.

## Reference check

- Bevy `dev/bevy/crates/bevy_asset/src/id.rs` separates a small runtime `AssetId::Index` from stable UUID identity. `dev/bevy/crates/bevy_asset/src/handle.rs` stores the optional asset path once on an `Arc<StrongHandle>`, while cloned handles share that owner and expose the path by reference. Zircon should adapt the handle-first ownership principle without copying Bevy's asset lifetime semantics into document lifecycle.

## Static gates executed

- Read all current document 2/2 and sync 3/3 Rust files plus the listed consumer paths and prior PERF-MVP-468 evidence. The expanded 387-line document scope was reread after its fingerprint drifted.
- `rustfmt --check --edition 2024 --config skip_children=true` passed document 1/2; current `lifecycle.rs` differs in one test import order and two multiline match layouts. Sync 3/3 remains not clean because external untracked `watch_map.rs` has one multiline `debug_assert!` layout difference. No source was rewritten.
- No managed Cargo, allocation/RSS scale run, WPR F0/F4 product trace or RenderDoc capture ran. RenderDoc is not applicable to this non-rendering slice. Both modules remain pending.
