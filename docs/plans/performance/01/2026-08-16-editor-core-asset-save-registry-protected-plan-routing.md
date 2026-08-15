---
related_code:
  - zircon_editor/src/core/asset
  - zircon_editor/src/ui/host/editor_document_autosave.rs
  - zircon_editor/src/ui/host/editor_manager_layout.rs
  - zircon_editor/src/ui/host/editor_ui_host.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
owner_plans:
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
source_evidence:
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/UObject/Package.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/PackageAutoSaver.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/FileHelpers.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/UObject/SavePackage/SavePackageUtilities.cpp
---

# Protected plan routing: editor core asset/save/registry

## Reason for routing

Performance01, `pending.md`, `review.md` and the owner plans are protected/foreign dirty in this
session. This record requests their current-source correction without overwriting them. Canonical
evidence is `2026-08-16-editor-core-asset-save-registry-current-architecture-review.md`.

## Requested Performance01 corrections

Replace the stale module accounting with **35/35 Rust files, 8,068 physical lines, 59 tests** and
normalized ordered raw fingerprint
`c08f63d8a4c30b6d1a3a59793a47e4ba8600a3140d0a5a6b53fb39c7d3d72cfe`.

Update current facts before preserving tasks:

- `DirtyRegistry` is production-reachable through EditorContext/EditorUiHost, native close,
  explicit save and autosave preparation. Close/autosave still call `changes_since(None)`.
- enabled-capability asset materialization now uses the batch API and publishes an immutable asset
  creation-menu generation; extension candidate validation still rebuilds builtins and applies all
  contributions one at a time.
- `EditorAssetIndex` and `EditorAssetImportFlow` remain without product callers.
- explicit save submits `InteractiveSave` with category limit one and a shared save mutex, but the UI
  immediately waits; the worker calls back through a weak EditorManager into mutable host/toolkit
  authority, and admission charges only the small job struct.
- the new SaveDirtyViews batch/job adapter is private and test-only. It reserves/materializes the
  complete batch, pending admission does not bound running payload memory, and completion waits for
  every ticket.

Retain and correct PERF-MVP-554/555/556/562 as specified in the canonical review. Link dirty autosave
projection/storage to PERF-MVP-592, job completion/event ownership to PERF-MVP-627, asset generation
to PERF-MVP-637 and immutable authoring state to proposed PERF-MVP-641.

## Proposed PERF-MVP-642

| id | priority | current diagnosis | required cutover | acceptance |
|---|---|---|---|---|
| PERF-MVP-642 | P0 | Explicit save is a worker indirection followed by UI `ticket.wait()`; worker re-enters mutable editor/toolkit authority, job bytes ignore document payload, and dirty effects commit one by one. The disconnected Save All adapter materializes the full batch and cannot bound running serialization/output memory. | Editor03/09/14 + Runtime04/11 create one project/document-generation save coordinator. Capture immutable intent/artifact generation, return a non-blocking ticket, stream through bounded per-document lanes, use one durable Runtime11 transaction and apply one generation-checked dirty/registry/catalog/UI receipt. Explicit, Save All and autosave share the lane; old waiting callback and whole-batch adapter are deleted. | docs `1/16/1K/16K`, bytes `1KiB/64MiB/1GiB`, stalls `0/10ms/10s`, mutation at every phase: UI wait/serialization/I/O 0; materialized full batch 0; queued+running+result bytes hard-bounded; same-document overlap 0; stale clears 0; one success publishes one generation chain; cancel/partial/retry deterministic; managed Cargo and F1/F4 WPR CPU/RSS/file-I/O/power pass |

## Requested owner-plan updates

### Editor03

Make the bounded dirty cursor the only save/autosave/close demand source. Publish immutable
per-document dirty/effect generations and one compare-and-clear commit receipt. Remove consumer
`changes_since(None)` polling and per-effect generation commits. Reuse PERF-MVP-641 authoring
generations for save artifact identity.

### Editor09

Own `PreparedSaveArtifact` and the post-durable RuntimeAssetRegistry/EditorAssetCatalog delta. Finish
PERF-MVP-555/556/562 without a second registry or index. Save completion must trigger one import/
refresh generation, not direct save plus watcher duplicate work.

### Editor14

Replace immediate UI wait with a ticket/completion cursor. Admission must bound queued, running,
serialized buffer and retained result bytes, not only pending request structs. Save All uses a
bounded window and stable partial results. Reuse the unique job system and current document mutex.

### Runtime04 and Runtime11

Runtime04 owns the authoritative durable asset/registry generation. Runtime11 owns serialize/write/
flush/atomic-replace/import-refresh phases, cancellation, deadlines, shutdown/project fences and
phase/byte counters. Do not add an editor-private I/O pool or retain the worker-to-mutable-host
callback after cutover.

### EditorUI08 and Render17

UI actions return after admission and consume bounded progress/completion deltas. Render17 records
main-thread wait, queue/running/result bytes, phase wall/CPU, locks, file I/O, allocations/RSS and
power. RenderDoc remains with thumbnail/Browser GPU owners, not this save/control task.

## Requested protected index state

- `pending.md`: replace the stale `zircon_editor/src/core/asset/**` row with current counts,
  fingerprint, `static_complete / dynamic_blocked` and the canonical review link.
- `review.md`: do not add the module until PERF-MVP-554/555/556/562/642 acceptance, current managed
  Cargo, product F1/F4 reachability, WPR/xperf and quantified RSS/power evidence are green.

## Milestone and notification state

This is a static architecture milestone only. The product build blocker prevents dynamic
acceptance, so no performance milestone commit or WeCom notification is due yet. Commit and the
quantified WeCom report become mandatory after the owner plans accept the routing and the dynamic
matrix passes.
