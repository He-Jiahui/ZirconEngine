# Editor core export current-source review

## Status

- Result: `static_complete / dynamic_pending`.
- Review date: 2026-07-30.
- Owners: Editor15 owns the export pipeline, preset and wizard integration; Runtime11 owns bounded filesystem/process work; EditorUI08 owns retained projection cadence. PERF-MVP-071 is the canonical generation-inventory item, PERF-MVP-107 owns pane metadata polling and PERF-MVP-558 owns wizard output backpressure.
- Accounting: keep `zircon_editor/src/core/export/**` in `pending.md`. The current pipeline is product-wired through `JobCategory::Export`, but current-source Cargo, cold/warm scale counters, cancellation latency and F4 export traces have not run.
- Code disposition: no Rust source was changed. Existing tracked and untracked source changes were preserved.

## Exact scope

| module | files | physical lines | inline tests | ordered path-and-raw-content SHA256 |
|---|---:|---:|---:|---|
| `zircon_editor/src/core/export/**` | 9/9 | 3,061 | 24 | `361a4a15d3a4254ddbe2c7f5518320d5242091791bbf6843641ed1cc519ac4c0` |

All nine Rust files were read in full. The fingerprint was reproduced twice and streams each sorted native workspace-relative path, a zero byte, raw bytes and a zero byte into SHA256. Product reachability was traced through the retained Build/Export projection, export wizard `EditorJobSystem` submission, cancellable process adapter and native-dynamic staging consumer. Those callers are evidence boundaries, not part of the 9-file accounting.

## Per-file review

| file | current-source performance result |
|---|---|
| `inventory.rs` | Positive baseline: overlapping root/child digests share a generation cache; cache misses hash through one 64 KiB buffer; persistent hits use strong file identity and avoid content reads. Remaining warm work still recursively enumerates and sorts every directory, canonicalizes and stats paths, and Windows file identity opens each file for multiple metadata queries. Every new generation still launches Python/Cargo/rustc and optional Node version probes even when persisted tool identity is unchanged. `Drop` clones the complete file/tool cache, pretty-encodes it, writes, flushes and atomically replaces it on the caller; invalidation and pruning scan whole maps. PERF-MVP-071 remains P0. |
| `mod.rs` | Module wiring and exports only. |
| `pipeline.rs` | Resume correctly prepares before reuse and validates prior outputs, but this means every skipped stage still pays its complete input preparation and output identity walk. The current core plan has only two stages, so duplicate/dependency scans and Vec removal are insignificant; skipped output/diagnostic clones are bounded by the current compact report contract. |
| `preset.rs` | Presets are small, but load/save are synchronous whole-file operations and save owns a private staging writer with `sync_all`. Production target projection and wizard option construction load presets; retained projection caching prevents stable full reparse, while PERF-MVP-107 still owns per-preset metadata work and PERF-MVP-570 owns duplicate text parsing. No separate task is warranted. |
| `stages/compile_host.rs` | The exported system runner creates two private reader threads, captures bounded 64 KiB tails with per-byte `VecDeque` operations, syncs both logs and a pretty JSON manifest, and has no deadline/cancellation or explicit child cleanup on early capture/wait errors. The production wizard does not use this fallback: it adapts CompileHost to its cancellable `ProcessCommandRunner`. Product output tail/backpressure/durability therefore stays under PERF-MVP-558 instead of a duplicate task. |
| `stages/executor.rs` | CompileHost prepare fingerprints many repository roots and probes three or four tools before reuse is known. Execute then fingerprints the staged tree and log artifacts; PlatformBundle validates and fingerprints overlapping outputs. The generation cache prevents duplicate content reads inside one executor, but not the complete recursive walk/canonicalize/stat work. Fingerprinting has no cancellation/deadline callback, so cancelling during a large prepare/bundle scan waits for the scan and then for inventory `Drop` persistence. PERF-MVP-071 owns this path. |
| `stages/mod.rs` | Stage module wiring only. |
| `stages/platform_bundle.rs` | A fixed set of directory/file checks and path construction; no independent hotspot. Large bundle cost comes from executor fingerprinting, not layout validation. |
| `tests.rs` | Fourteen tests cover graph validity, resume/output tamper behavior, preset round-trip, command construction and bundle layout. Inventory adds nine tests and CompileHost adds one. The ignored warm-cache test covers one 1 MiB file and content-read count, but does not measure recursive metadata/canonicalization, tool subprocesses, cache clone/encode/fsync, cancellation or 1K/100K-file behavior. |

## Canonical plan correction

PERF-MVP-071 now explicitly includes four current-source costs:

- stable resume recursively walks, canonicalizes, sorts and stats all declared source/output roots before it can skip;
- persisted tool records do not suppress Python/Cargo/rustc/Node version subprocesses in a fresh generation;
- fingerprinting has no entry/byte/deadline cancellation checkpoints;
- inventory destruction performs full-cache clone, pretty JSON encoding and durable replacement synchronously.

The target design is one generation-scoped changed-path/directory inventory shared by CompileHost, PlatformBundle and native staging. Tool probes must be keyed by resolved executable/toolchain-selection identity and invalidated only by relevant executable, environment or toolchain changes. Cache persistence must be an explicit Runtime11 job with entry/byte/deadline budgets and observable errors; `Drop` performs no I/O. Fingerprint walks must poll cancellation at bounded entry/byte intervals.

## Acceptance plan

- Inputs: 1/1K/100K files, 1 MiB/1 GiB single files, cold/warm/1% changed/deleted/tampered trees, client/server target modes and 0/1/4 changed tool identities.
- Record directory walks, `read_dir`, canonicalize/stat/file-handle queries, content bytes/reads, tool subprocess count/wall, cache clone/encode/write/fsync bytes, cancellation latency, caller wall and peak RSS. Warm unchanged content reads and tool probes are zero; directory/metadata work is near changed subtrees; hash scratch remains at most 64 KiB; `Drop` I/O is zero.
- Use Tracy spans/counters around prepare/reuse/persist, Windows WPR/ETW file/process/CPU traces and Process Monitor path verification. Run current-source managed export/lib tests, wizard cancellation/resume tests and F4 cold/warm Build/Export traces after the coordinator lane is available.
- RenderDoc is not applicable to this module because the export core does not submit render work. Bundle launch/render acceptance remains in the consuming rendering plan.

## Reference check

- Godot `dev/godot/editor/export/editor_export_platform.{h,cpp}` keeps a per-path `FileExportCache`, checks modification time before MD5, marks used records and writes only surviving entries. Zircon should keep its stronger file identity and content confirmation while adopting changed-entry reuse rather than whole-tree warm traversal.
- Fyrox `dev/Fyrox/editor/src/export/mod.rs` moves export work to `ExportWorkerThread` and exposes cancellation/child cleanup. Zircon already improves on UI isolation through the shared `JobCategory::Export`; the remaining requirement is bounded, cancellable work inside that job rather than another private export pool.

## Static gates executed

- Read all current 9/9 Rust files and the named product/reference call chains.
- Two consecutive source fingerprints matched `361a4a15d3a4254ddbe2c7f5518320d5242091791bbf6843641ed1cc519ac4c0`.
- `rustfmt --edition 2021 --check` passed all nine files.
- `git diff --check -- zircon_editor/src/core/export` passed with existing LF-to-CRLF warnings only.
- `review.md` remained unchanged. No managed Cargo, WPR/ETW, Tracy, Process Monitor, F4 or RenderDoc run was performed.
