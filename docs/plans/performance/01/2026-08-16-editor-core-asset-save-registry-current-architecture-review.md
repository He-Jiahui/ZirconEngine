---
related_code:
  - zircon_editor/src/core/asset
  - zircon_editor/src/core/jobs
  - zircon_editor/src/core/extension/toolkit
  - zircon_editor/src/ui/host/editor_document_autosave.rs
  - zircon_editor/src/ui/host/editor_manager_layout.rs
  - zircon_editor/src/ui/host/editor_ui_host.rs
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/retained_host/app/native_window_close.rs
reference_code:
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/UObject/Package.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/PackageAutoSaver.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/FileHelpers.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/UObject/SavePackage/SavePackageUtilities.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/AssetRegistry/Private/AssetRegistryState.cpp
  - dev/godot/editor/file_system/editor_file_system.cpp
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
  - docs/plans/optimize/coverage.md
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
doc_type: implementation-evidence
status: static_complete_dynamic_blocked
created_at: 2026-08-16
---

# Editor core asset/save/registry current-architecture review (2026-08-16)

## Status and scope freeze

- Result: `static_complete / dynamic_blocked`.
- Scope: `zircon_editor/src/core/asset/**`, **35/35 Rust files, 8,068 physical lines and
  59 inline tests**.
- Normalized ordered path-and-raw-content SHA256:
  `c08f63d8a4c30b6d1a3a59793a47e4ba8600a3140d0a5a6b53fb39c7d3d72cfe`.
- Every current file was reconciled. The 31-file July review was reread, the three commits after
  that freeze were diffed, and every added or changed implementation/test file was read in full.
  Production reachability was followed through editor context/host dirty state, native close,
  explicit UI/animation save, autosave, job admission, extension materialization, workbench asset
  creation menu and runtime asset registry callers.
- This report supersedes the 31-file/5,889-line/43-test scope in
  `2026-07-30-editor-core-asset-current-review.md`. In particular, `DirtyRegistry` is now a
  production owner; the old statement that it had no core-external caller is no longer true.
- Code disposition: no Rust source changed. Nine tracked files and the new save-job adapter tree are
  foreign dirty work. The active defects cross Editor03/09/14, Runtime04/11 and UI ownership; a local
  micro-edit would preserve the wrong save and publication boundary.
- Accounting: keep the module in `pending.md`. It cannot enter `review.md` until the managed Windows
  current-source tests, scale counters, F1/F4 product traces and CPU/RSS/power evidence pass.

The approved D/E/F editor build still fails in `tools/build-editor.ps1:130` before Cargo because an
approved-root separator is parsed literally. WPR/xperf therefore has no current product executable,
and no timing, power or algorithm-optimality claim is made here. RenderDoc is not applicable to this
CPU/control slice; thumbnail upload and Browser rendering stay with their render-owner plans.

## Per-file review

| file | current-source performance result |
|---|---|
| `dirty/error.rs` | Typed failure payloads only. Eight-attempt instability remains observable rather than returning a false-clean result. |
| `dirty/external_effect_id.rs` | Canonical validation is construction-time and lookup can borrow the identifier. Ordered ownership supports snapshot search without query-time allocation. |
| `dirty/mod.rs` | Exports dirty authority, save-batch contracts and the new job adapter. The latter two are still private to `core::asset` and test-only. |
| `dirty/registry.rs` | A 4,096-change journal and transaction cursor make stable delta replay useful. Current close/autosave callers restart with `changes_since(None)`, and reset/changed paths still clone document/effect maps under the mutex. `clear_saved_external_effects` reacquires the lock and advances the journal once per effect. |
| `dirty/registry` tests in `dirty/tests.rs` | Cover 10K stable/change behavior, cursor/reset/removal and races. They do not measure cloned bytes, mutex hold/wait, per-effect clear cost or product close/save latency. |
| `dirty/save_batch.rs` | Whole-batch preflight accumulates errors and completion apply is generation-safe. It sorts/copies all candidates and toolkits, retains a second owned intent set, then applies terminal results serially; this is a control-plane prototype, not a production Save All owner. |
| `dirty/save_batch/tests.rs` | Four tests cover aggregate preflight, partial/stale/cancel results and malformed completions. No large-batch allocation, main-thread budget or durable I/O behavior is exercised. |
| `dirty/save_job_adapter.rs` | Uses the shared job system, one per-resource mutex lane, atomic batch reservation and a 64-ticket poll budget. It reserves/materializes the complete batch, clones every intent into a job, and returns a result only when all tickets terminate. Pending admission bytes are released when work starts and do not bound serialized buffers or running-result bytes. No production caller exists. |
| `dirty/save_job_adapter/tests.rs` | Eight tests prove reservation order, rollback, mutex reuse, partial failure, cancellation and ticket-count pumping. They use synthetic tiny payloads and a busy-yield waiter; they cannot validate product affinity, payload RSS or UI responsiveness. |
| `import_flow/error.rs` | Typed rejected-path diagnostics only. |
| `import_flow/flight.rs` | Exact observers share one flight, but admission/result Condvars can block callers and each observer clones the owned result/status strings. |
| `import_flow/job.rs` | Runs backend work through Editor jobs with panic/cancel cleanup. Start/completion still format the URI into progress messages. |
| `import_flow/mod.rs` | Declares generation tickets and finite entry/estimated-byte/age limits. Public blocking wait remains an integration hazard. |
| `import_flow/state.rs` | Exact generation single-flight and UUID lifecycle serialization are useful. Mutex identity allocation formats strings and scans all active UUIDs for collisions; same-timestamp vector removal is linear, so a distinct-UUID storm can approach quadratic work. |
| `import_flow/submit.rs` | Avoids overlapping index/state/backend locks. Existing observers and UUID transitions can still wait synchronously on another submitter; each new flight formats a label and clones the request. |
| `import_flow/tests.rs` | Covers success/failure/retry, duplicate coalescing, migration, progress, panic, shutdown, cancellation and admission limits. No bounded submit-latency or distinct-UUID allocation gate exists. |
| `import_flow/tests/concurrency.rs` | Covers admission propagation and lifecycle races. The tests intentionally use blocking handoffs and do not prove UI-safe submission. |
| `index.rs` | Rows borrow registry/metadata authority, but `rows()` inherits a full collect and path sort. Registry replacement scans every projection, and unknown added watch paths can remain unbounded. It remains test-only and must not become a second Browser truth. |
| `index/tests.rs` | Twelve tests cover ordering, watch deltas and atomic projections; no stable-query allocation or unknown-path storm budget is measured. |
| `mod.rs` | Public façade exports dirty/index/import/type contracts but deliberately does not expose the save prototype. No independent work. |
| `source_authority.rs` | Locator classification is linear in the input path and outside frame work; only explicit root handling owns a temporary sentinel string. |
| `toolkit_route.rs` | Stable locator/operation DTO with borrowed accessors; no repeated projection. |
| `type_registry/asset_type_id.rs` | Canonical ID validation is linear at construction and lookups can borrow `str`; no hot owned-ID conversion. |
| `type_registry/builtin.rs` | Static single-kind lookup uses `OnceLock`, but every `with_builtins()` reconstructs 26 definitions through 26 single-entry commits. Validation and cache misses repeatedly pay this cost. |
| `type_registry/context_command.rs` | Capability sort/dedup is construction-time. Owned strings increase generation size but accessors are borrowed. |
| `type_registry/contribution.rs` | Owned serializable delta. Host validation still clones existing and candidate contributions before applying them. |
| `type_registry/creation_template.rs` | Sort/dedup is construction-time; default documents must be counted in retained contribution bytes. |
| `type_registry/definition.rs` | Materialized owned definition with borrowed queries; no local stable-frame rebuild. |
| `type_registry/error.rs` | Diagnostics only; missing-field joining allocates only on rejection. |
| `type_registry/mod.rs` | Module wiring and exports only. |
| `type_registry/presentation.rs` | Compact validated presentation DTO; constant-time borrowed access. |
| `type_registry/registry.rs` | Batch publication and immutable creation-menu generations are useful. A batch that adds templates compiles the complete menu once; repeated single-entry validation can therefore rebuild the full menu repeatedly. `with_builtins()` still reconstructs the base registry. |
| `type_registry/registry/batch.rs` | Groups by asset type, validates/stages, sorts each touched collection once and publishes one registry generation. Owner strings are copied into claims and final maps. Product materialization now uses the batch, but extension candidate validation still applies existing/candidate rows one at a time. |
| `type_registry/thumbnail_provider.rs` | Descriptor/palette data only; no decode, upload or draw work. |
| `type_registry/toolkit.rs` | Construction-time capability normalization and borrowed accessors; no independent hot path. |

## Current production chain and architecture verdict

The current explicit save chain is:

`retained action -> EditorManager -> EditorUiHost::save_document_toolkit ->
InteractiveSave job -> Weak<EditorManager> upgrade -> save_document_toolkit_canonical ->
toolkit callback -> dirty saved-top/effect clear -> caller ticket.wait()`.

`InteractiveSave` has a finite default concurrency of one and uses the same source-derived mutex as
autosave. Those are correct exclusion primitives. They do not make the workflow asynchronous: the
retained/UI caller submits and immediately waits for the ticket, while the worker calls back into the
mutable EditorManager/host. The job estimate is `size_of::<ForegroundDocumentSaveJob>()`, not the
serialized document or write buffer. Moving work to a worker while the UI blocks and the worker
re-enters editor authority is a scheduling indirection, not a save transaction.

The new Save All prototype improves batch admission locally but is disconnected. It atomically
reserves every document, resolves every mutex, clones every intent and materializes every job before
any work can proceed. A large batch can therefore be rejected by the pending-entry/byte cap instead
of streaming through the one-save lane. Once admitted, pending bytes are released as jobs enter the
runtime scheduler, so they do not cap running serialization/output/result memory. Completion is held
until the slowest document terminates and only then applied serially.

Dirty publication is also split. Marking one effect immediately snapshots the document again to
update the view projection. Native close and the dormant autosave path both rebuild a toolkit map and
call `changes_since(None)`, converting the bounded journal back into a full reset. Save completion
then clears external effects one by one, taking the registry lock and publishing a generation for
each effect. This is not one document commit generation.

The required hard-cut chain is:

`ProjectAssetGeneration -> DocumentDirtyGeneration -> PreparedSaveArtifact ->
DurableAssetCommitGeneration -> RuntimeAssetRegistryDelta -> EditorAssetCatalogDelta ->
RetainedAssetSurfaceDelta`.

There must be one project/document generation token, one bounded save coordinator, one durable I/O
owner and one completion receipt. UI code captures an intent and returns; it does not wait, serialize,
perform file I/O or let a worker call arbitrary mutable host callbacks. A save artifact is produced
from an immutable authoring/document generation, admitted by a real payload/buffer bound, written by
Runtime11, then committed against the still-current dirty generation. Import/refresh consumes the
single durable delta. Save All feeds a bounded window through the same lanes and reports stable
partial results; it does not materialize all jobs or create a second dirty/index authority.

## Required task corrections

### PERF-MVP-554: preserve the journal, remove reset and payload reconstruction

The journal is now production-reachable and useful, but close/autosave consumers throw away the
cursor by passing `None`. Give each consumer an owned cursor/demand index, publish immutable
per-document effect slices and page reset. Clear saved effects as one generation-checked document
commit, not one registry transaction per effect. Link the autosave-specific due/storage work to
PERF-MVP-592 rather than duplicating it here.

### PERF-MVP-555: retain single-flight, cut blocking and repeated model import

Keep exact generation coalescing and finite retention. Replace Condvar admission/result waits with a
pending ticket plus deadline/cancel; use a typed monotonic mutex identity and indexed order removal;
share terminal results. Product manual/watch/model-derived imports must enter this one transaction,
with one model parse/scan and one atomic derived-resource publication.

### PERF-MVP-556: delete or absorb the dormant second index

Do not connect `EditorAssetIndex::rows()` to the Browser. Runtime04/Editor09 must expose stable
ordered slots, affected UUIDs and visible paging through the existing catalog generation. Unknown
watch input needs entry/byte/age bounds and resync, not an unbounded side set.

### PERF-MVP-562: finish batch materialization and base-generation sharing

Product enabled-capability materialization now uses `apply_contributions`, which corrects the old
"always single-entry" statement. Extension candidate validation still rebuilds builtins and applies
all existing/candidate contributions individually. Publish a shared immutable builtin base and
validate/materialize one capability generation through one batch while preserving input-indexed
diagnostics and valid-entry isolation.

### Proposed PERF-MVP-642: one non-blocking durable document-save transaction

Replace the waiting foreground wrapper and disconnected whole-batch prototype with one coordinator:

1. capture `(project, toolkit, document dirty, authoring)` generations plus resource identity and a
   bounded artifact estimate under short owner leases;
2. return an explicit save ticket immediately; no UI `wait`, Condvar or arbitrary worker callback
   into mutable EditorManager/host;
3. admit a bounded streaming artifact or trustworthy maximum, with the same document lane for
   explicit save, Save All and autosave;
4. let Runtime11 own serialize/write/flush/atomic replace/import-refresh phases and counters;
5. apply one generation-checked terminal receipt, clearing saved-top/effects and publishing the
   asset/catalog/UI delta once; stale completions leave the newer dirty generation intact;
6. stream Save All through a bounded window with stable partial/cancel/retry results and without a
   complete request/job/ticket materialization.

This task depends on PERF-MVP-641's immutable authoring commit generation, PERF-MVP-627's bounded job
completion/event ownership, PERF-MVP-592's autosave demand/payload work and PERF-MVP-637's unified
runtime/editor asset generation. It must reuse them, not introduce another scheduler or registry.

## Reference-engine evidence and adaptation boundary

### Unreal Engine primary reference

- `Package.h:266-298` exposes package dirty-state delegates and distinguishes state-change events
  from every mark attempt. `PackageAutoSaver.cpp:1175-1218` maintains dirty package sets from those
  events; due checks inspect set counts and save consumes the maintained sets (`:1288-1300`,
  `:330-383`). This grounds a maintained dirty demand index rather than repeated full projection.
- `FileHelpers.cpp:4336-4368` gives Save Dirty a named CPU scope, gathers the batch once and delegates
  the workflow. `InternalPromptForCheckoutAndSave` separates full-load/clean/preparation from the
  per-package save loop and preserves partial failures (`:4514-4627`).
- `FileHelpers.cpp:5944-5990` batches checkout before save and mark-for-add after successful saves.
  This supports explicit prepare/commit/post-commit phases and stable partial results.
- `SavePackageUtilities.cpp:59-72` records phase timings and bytes. Async file writes have an
  explicit outstanding counter (`:175`, `:582-609`) and a named wait boundary (`:1562-1575`). Zircon
  needs equivalent phase/byte/owner counters, but the Unreal interactive loop and global wait are
  not evidence that Zircon should block its retained UI thread.
- `AssetRegistryState.cpp` queries compiled filters over indexed asset data. This supports merging
  the dormant editor index into the authoritative generation rather than activating a second full
  collect/sort projection.

### Corroborating reference

Godot `editor_file_system.cpp` performs filesystem scanning in a low-priority background path,
coalesces pending scans and applies targeted file updates. Zircon should use its shared Editor14/
Runtime11 scheduler and generation receipts, not a private thread, while retaining the affected-file
shape.

## Measurement and acceptance gates

| gate | matrix | required result |
|---|---|---|
| dirty delta | documents/effects `1/100/10K`, changed `0/1/1%/reset`, writers `1/16` | stable visits/cloned bytes 0; change work near delta; reset paged; effect clear one document commit; lock p95 independent of total clean documents |
| save admission | documents `1/16/1K/16K`, payload `1KiB/64MiB/1GiB`, stalls `0/10ms/10s` | UI wait/serialize/I/O 0; whole-batch materialization 0; queued+running+result bytes hard-bounded; one document lane; cancel/deadline terminal |
| save generations | edits during capture/serialize/write/flush/rename/import/commit | stale receipt never clears newer dirty data; one successful save publishes one dirty, durable, registry, catalog and UI generation; partial outcomes deterministic |
| import | duplicate/distinct UUID `1/10K/1M`, model animations `0/1/100/1K` | submit blocked time 0; same generation job/parse/scan <=1; identity/order near linear or indexed; terminal bytes/age bounded |
| registry/index | types/assets/templates `1/100/10K/100K`, stable/change/capability reload | stable rebuild/sort 0; materialize publish 1/generation; each touched collection sort <=1; Browser work affected+visible page; unknown path bytes/age bounded |
| product | at least 31 comparable cold/warm F1/F4 runs after build repair | WPR/xperf CPU sampling, CSwitch, waits/locks, file I/O, thread time, allocations/RSS and package power report median/p95/CI/effect size; compare matched local Unreal project, frame cap and power plan |

Static unit behavior is not milestone acceptance. Current managed Cargo, allocation/lock/RSS scale
counters, product reachability, WPR/power capture and an independent current-source review remain
mandatory.

## Static gates executed

- Recomputed current inventory: 35 files, 8,068 physical lines, 59 inline tests and normalized
  ordered raw fingerprint
  `c08f63d8a4c30b6d1a3a59793a47e4ba8600a3140d0a5a6b53fb39c7d3d72cfe`.
- `rustfmt --edition 2021 --check`: **29/35 GREEN, 6/35 RED**. The six foreign current files are
  `dirty/error.rs`, `dirty/mod.rs`, `dirty/save_job_adapter.rs`,
  `dirty/save_job_adapter/tests.rs`, `dirty/tests.rs` and root `mod.rs`. They were not rewritten.
- `git diff --check -- zircon_editor/src/core/asset`: GREEN; only existing checkout line-ending
  warnings were emitted.
- New-document metadata: 24/24 referenced source/plan paths exist. Both new documents have zero
  trailing-whitespace findings.
- Repository documentation gate: RED with 671 existing violations across 241 of 2,500 documents;
  violations owned by the two new documents: **0**.
- Session coordinator plan audit and the `codex-performance-audit-20260814` heartbeat: GREEN.
- Managed Cargo, allocation/lock/RSS scale tests, product build, WPR/xperf, power and independent
  review: not executed because the recorded approved-root build defect remains unresolved.
