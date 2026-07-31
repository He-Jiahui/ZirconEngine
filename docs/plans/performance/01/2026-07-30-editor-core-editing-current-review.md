# Editor core editing current-source review

## Status

- Result: `static_complete / dynamic_pending`.
- Review date: 2026-07-30.
- Primary owners: Editor01/03/05 and Runtime07 for the authoring-world access contract, Editor03 for transaction/history/journal contracts, and Editor05 for scene-selection/gizmo deltas. Editor04 owns the Play queue that consumes the retention declaration.
- Accounting: keep `zircon_editor/src/core/editing/**` in `pending.md`. Do not add it to `review.md` before current-source managed Cargo, deterministic scale counters and F4 edit/undo/gizmo evidence are GREEN.
- Code disposition: no Rust source was changed. Existing tracked modifications, the deleted legacy `editing/history.rs`, and current untracked owners were reviewed and preserved.

## Exact scope

| module | files | physical lines | inline tests | ordered path-and-raw-content SHA256 |
|---|---:|---:|---:|---|
| `zircon_editor/src/core/editing/**` | 24/24 | 4,611 | 8 | `ac861a19076c8433e58e025e3c963a8d9dd6a11d2e65b7eed3322e3aaa59d2bb` |

The fingerprint streams each workspace-relative path, a zero byte, the file's raw bytes and a zero byte in sorted path order into SHA256. All 24 files were read in full. Production reachability was followed through `EditorContextBuilder`, `EditorAuthoringWorld`, workbench snapshot/render/selection callers, scene command capture/dispatch, the workbench operation dispatcher, Play entry, project save, asset dirty delta projection and transaction lifecycle publication.

## Per-file review

| file | current-source performance result |
|---|---|
| `authoring_world.rs` | Correctly hard-cuts direct UI ownership of runtime scene types behind a stable `EditorRuntimeGatewayHandle`. Every read or mutation still performs a generation snapshot, dynamic gateway call and TLS reentrancy guard before `LevelSystem` locks the single `World` mutex for the full callback. `try_snapshot` also clones the full scene/world and is reached by Play entry and project-scene projection. The gateway boundary is positive; repeated long-held world access is PERF-MVP-600, while the Play clone remains PERF-MVP-550. |
| `command.rs` | Scene commands mutate only changed fields, and operation-group continuation no longer clones its stable key. `UpdateNodeCommand` still captures `NodeEditState { name, parent, transform }` for rename/reparent/transform and merge clones the complete `after`; transform drags therefore copy a stable name. Reflected field commands also retain and clone wide `ReflectedValue` before/after values. |
| `context.rs` | Selection capture and restore clone an `Arc` handle, not JSON. Scene callbacks enter the `LevelSystem` world owner only after the transaction engine has released its state mutex. |
| `engine/command.rs` | Compact command trait and merge/journal contracts. Default journal rejection is explicit; no eager serialization occurs during apply, revert or commit. |
| `engine/events.rs` | Lifecycle events contain ids, history, label, frame and kind only. Delivery reports backpressure/rejection instead of retaining a private unbounded event vector. |
| `engine/history.rs` | `VecDeque` history is capacity-bounded (default 128); redo truncation and eviction finalize outside the engine lock. Status is scalar O(1); details are explicit pages and clone only requested labels/participants plus shared selection handles. |
| `engine/journal.rs` | Explicit projection owns participants, selection vectors and per-command JSON payloads. It is not part of commit/undo/redo and currently has no production caller, but serialization is still performed while the engine state mutex is held. |
| `engine/mod.rs` | Re-exports and the 128-entry default only; no repeated work. |
| `engine/routing.rs` | Small typed history-routing enum; constant-time matching. |
| `engine/transaction.rs` | Command callbacks, selection restoration, finalization and event publication run outside the engine state mutex. One exclusive operation owner prevents concurrent mutations. `set_frame` takes that owner but has no production caller. `journal_transaction` is the remaining wide work under the mutex. |
| `engine/transaction/dirty_batch.rs` | A 4,096-entry generation journal returns changed histories through reset/delta cursors. Delta queries allocate a `BTreeSet`, but work is bounded and consumed by the asset dirty registry rather than polled per frame. |
| `engine/transaction/exclusive_transition.rs` | Project/Play transitions hold the transaction operation gate while clearing context/history/world, but do not hold the engine state mutex across gateway/world callbacks or record finalization. This is a correct lock-order boundary; transition wall time, including full Play snapshot, remains measurable rather than a new nested-lock defect. |
| `engine/transaction/operation_group.rs` | One open transaction merges a gesture; continuation compares the borrowed typed group key and does not clone it. Flush/commit remains serialized by the transaction engine as required for mutation correctness. |
| `engine/transaction/save_token.rs` | Save tokens retain an `Arc` lineage plus compact ids/generation. Project save uses compare-and-mark without cloning history records. |
| `intent.rs` | Small typed edit intent declaration; no queue or repeated projection is owned here. |
| `mod.rs` | Module mounting and exports only. |
| `operation/command.rs` | Bridges one operation invocation into the edit-command contract; retained invocation width is governed by registration/Play retention rather than copied during history status queries. |
| `operation/error.rs` | Typed error conversion only. |
| `operation/factory.rs` | Factory trait declaration only; no registry scan or callback execution is implemented here. |
| `operation/mod.rs` | Operation module exports only. |
| `operation/pending_edit_retention.rs` | Registration-owned `Lossless`, `Latest` and `(entries, bytes, age)` bounded policies are typed and validated. This file declares policy; enforcement and apply budgets remain Editor04 Play-queue responsibilities. |
| `operation/registration.rs` | Freezes operation metadata, factory and pending-edit retention once at registration. Construction owns strings but is not an edit hot path. |
| `paths.rs` | Streaming operation-path validation; no temporary segment collection. |
| `selection.rs` | `SceneSelection` stores `Arc<[NodeId]>`; `SelectionSnapshot` stores `Arc<SelectionState>`. Clone/capture/restore are O(1); owned `Vec` conversion is restricted to explicit journal projection. The 1/100/10K sharing test was read but not run. |

## Corrected and remaining tasks

### PERF-MVP-549: source implementation present, dynamic gate missing

The 2026-07-22 conclusion that transaction selection is a `serde_json::Value` authority and that history status deep-clones all 128 records is no longer current. Selection snapshots are typed immutable `Arc` handles, `CoreEditContext` capture/restore clones only those handles, `HistoryStatus` is scalar, and `history_details` requires a page size from 1 through 128. Undo/redo events already return compact identity and label data. Keep PERF-MVP-549 open for scale counters, managed behavior and lifecycle-backpressure evidence; do not design another selection cache or reintroduce JSON in the hot path.

Explicit journal export remains a separate slow path. Before it is wired to production persistence, Editor03 should obtain or freeze a record handle under the engine mutex, release the mutex, then project selection vectors and command JSON under an explicit bytes/deadline budget. Current source has no production `journal_transaction` caller, so this is a pre-wiring requirement rather than an observed F4 stall.

### PERF-MVP-063: field-specific scene deltas remain required

`UpdateNodeCommand` still captures the full node edit state for every rename, reparent and transform update. A transform-only gesture copies the stable name during capture and clones the complete next `after` state during merge. Editor03/05 should hard-cut this to typed rename/parent/transform deltas; the open operation group should retain only the field-specific before/current values and commit one history command. Do not add a second gizmo-side cache.

### PERF-MVP-600: authoring-world lock and repeated gateway access

The stable gateway handle has already removed the old replacement `RwLock` and capability deep clone, so PERF-MVP-068 must not be reopened. The current provider boundary is different: each `EditorAuthoringWorld::try_with_world(_mut)` loads/clones the ArcSwap generation, dispatches through a dynamic gateway and TLS guard, then holds `LevelSystem`'s single `Arc<Mutex<World>>` for the complete caller callback. Workbench hierarchy/inspector/scene snapshot construction and render extraction can format or project wide DTOs while holding that mutex; one UI operation can invoke the authoring facade several times. Transaction apply/revert/rollback release the engine state mutex correctly, but a multi-command top-level operation reacquires the world mutex for each command while the transaction operation gate remains held.

Editor01/03/05 with Runtime07 should publish one immutable authoring generation shared by stable hierarchy/inspection/render/selection reads. A changed frame may perform at most one bounded authoring read/seal; presentation and DTO formatting must run without the world mutex. Mutation batches should acquire one top-level typed scene lease and pass a borrowed context through apply/revert/rollback instead of locking per command. Preserve the gateway boundary, generation lifetime and reentrancy failure contract; do not expose runtime scene owners directly to the workbench or add a second UI scene authority.

### Reflected value width and Play retention boundary

`SetReflectedSceneFieldCommand` must own reversible before/after values, but consuming writes currently require another clone of the selected value. Editor03/05 should measure payload bytes and use a shared/owned transfer contract for wide reflected values before optimizing blindly. `PendingEditRetention` now provides the required typed policy vocabulary; PERF-MVP-551 stays open until Editor04 proves that the actual Play queue enforces per-cohort entry/byte/age bounds and budgeted apply.

## Acceptance plan

- Selection/history: selection items `1/100/10K`, histories `1/100`, records `1/128`, detail pages `1/128`. Record Arc payload sharing, selection bytes copied, labels/participants cloned, state-lock wait/hold and p50/p95. Hot capture/restore must copy zero selection payload bytes; status must not visit records; detail work must be bounded by the requested page.
- Gizmo/update: drag updates `1/1K/100K`, name lengths `0/1KiB`, nested groups `1/16`. Record scene reads/setters, `NodeEditState` and String clone bytes, merge attempts, commands committed and UI p95. Require zero stable-name clone per transform update, one changed-field setter and one final record.
- Reflected/journal: values `64B/1MiB/256MiB`, commands `1/128`, selection `1/10K`. Record value/JSON clone bytes, serde traversals, engine-lock hold and peak RSS. Apply/revert must preserve exact values; production journal projection must not hold the engine state mutex across payload serialization.
- Authoring world: nodes `1/1K/100K`, stable and `1%` changed frames at 60 Hz, commands `1/128/100K`, threads `1/8`. Record ArcSwap generation loads, Arc clones, dynamic/TLS calls, world-mutex acquire/wait/hold, callbacks, scene-clone bytes and UI p95. Require stable presentation world-lock count `0`, changed projection at most one lock per generation, top-level undo/redo batch at most one world lock, and no lock across DTO formatting or paint.
- Dirty/save/events: documents `1/100/10K`, dirty changes `0/1/100%`, lifecycle events `1/100K`, stalled consumer `0/60s`. Record cursor resets/deltas, BTree entries, event queue entries/bytes/oldest age, backpressure and lock time. Preserve undo/redo/rollback/save-token/order semantics.
- Run current-source managed editor editing tests and F4 create/rename/reparent/transform/undo/redo/save flows. RenderDoc is not applicable to this CPU transaction slice; Editor05's viewport/gizmo render submission remains a separate RenderDoc gate.

## Reference check

- Fyrox `dev/Fyrox/editor/src/command/mod.rs` uses a capacity-bounded command stack, merges commands and explicitly finalizes discarded entries. Zircon's bounded history and lock-free finalization follow the useful part of that model; the remaining optimization is narrower typed command payloads.
- Godot `dev/godot/editor/editor_undo_redo_manager.cpp` routes actions to histories, carries version/saved-version state and delegates merge mode to the history. Zircon should preserve its equivalent routing/save semantics while removing stable-field copies, not replace them with UI-local undo state.
- Unreal's scoped-transaction model remains a useful RAII boundary reference, but the current checkout no longer contains the previously cited `ScopedTransaction.h`; no current-source claim depends on that missing reference.

## Static gates executed

- Read all current 24/24 Rust files and the listed production caller chains.
- `rustfmt --edition 2021 --check` passed for all 24 files.
- `git diff --check -- zircon_editor/src/core/editing` passed. Existing tracked/untracked changes were not rewritten.
- No managed Cargo, allocator/RSS scale run, WPR F4 product trace or independent dynamic review ran. The module remains pending and `review.md` is unchanged.
