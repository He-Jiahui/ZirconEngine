---
related_code:
  - zircon_editor/src/core/recovery
  - zircon_editor/src/core/asset/dirty
  - zircon_editor/src/core/extension/toolkit
  - zircon_editor/src/ui/host/editor_document_autosave.rs
  - zircon_editor/src/ui/host/editor_manager_layout.rs
  - zircon_editor/src/ui/host/editor_manager_project_session.rs
  - zircon_editor/src/ui/retained_host/app/autosave.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/review.md
  - docs/plans/performance/pending.md
owner_plans:
  - docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md
  - docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
source_evidence:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/PackageAutoSaver.cpp
---

# Protected plan routing: recovery autosave generation

## Reason for routing

Performance01, `review.md`, `pending.md` and the owner plans are protected/foreign dirty in this
session. This record requests current-source corrections without overwriting their owners. Evidence
is `2026-08-15-editor-core-recovery-autosave-generation-current-architecture-review.md`.

## Requested Performance01 corrections

Replace the stale recovery accounting with 20/20 current Rust files, 5,009 physical lines, 54 tests,
zero ignored and ordinal fingerprint
`57baefc04ab5e4d77cf9a4a1c91b3dee7a1d77f9b41cf5da72f8b8606078e781`. The old 3-file/824-line/10-
test row predates the production adapter/service, recovery catalog, platform lease, liveness and
split tests.

Correct `PERF-MVP-592` rather than preserving the old "no production adapter/two scans" diagnosis:

- a unique EditorJobSystem adapter now has bounded entry/estimated-byte/age admission, reservation
  before request materialization, lazy capture, foreground-save mutex, fairness and a completion
  ticket budget;
- the retained orchestration method exists but is never called by current product code, so autosave
  remains unreachable and cannot have product performance evidence;
- if wired today, every due interval reconstructs the complete dirty/toolkit/path projection,
  scheduler selection scans all dirty input and request lookup is `O(W*D)`;
- pending bytes charge only `size_of::<AutosaveDocumentRequest>()`, while each worker captures a
  complete unaccounted `Vec<u8>` under its document session lock;
- one snapshot performs three full document-directory scans plus a recovery metadata read/decode,
  then uses a private atomic writer outside Runtime11 counters;
- project generations do not fence autosave jobs/receipts, restore flow is not production-wired and
  session heartbeat has no production scheduler.

Do not resolve reachability by adding only `poll_editor_autosave()` to tick. Production integration
is the last step after incremental demand, payload bounds, O(1) storage state and project fencing.

## Required target architecture

1. Editor03 dirty-journal deltas maintain one recovery-owned `AutosaveDemandIndex`; no second dirty
   registry. Stable ticks do zero work and changed-document work scales with the delta.
2. Due selection takes a bounded page directly from the index and produces an immutable
   `(project generation, document, dirty generation)` token. Remove all-dirty vectors, toolkit map,
   dual selection sets and linear intent lookup.
3. Editor14 admission charges a trustworthy actual upper bound or a fixed streaming buffer. A queued
   job owns no serialized payload; capture remains off UI under the foreground-save exclusion; newer
   dirty generations supersede old queued work before capture.
4. Runtime11 owns one per-project/document durable lane, exact generation receipts, cancellation and
   project/shutdown fences. Recovery removes its private atomic writers after cutover.
5. Editor17 owns a versioned per-document manifest/fixed three-slot ring. Startup performs one
   bounded reconcile; steady next sequence, source identity and rotation are O(1), with zero
   directory scans and metadata rereads.
6. Recovery discovery uses bounded/resumable pages and isolates corrupt entries. Editor16 schedules
   coalesced session heartbeat I/O on Runtime11 and orders final fence before guard release.
7. EditorUI08 connects autosave only after these gates; retained frame work is bounded completion and
   changed-demand pumping, never full document projection or synchronous filesystem work.

## Requested owner-plan updates

### Editor17

Update `17-editor-services-and-recovery.md` and the existing M2.1 record. Preserve the current
correctness foundations, but replace the stale two-scan claim with three scans plus metadata reread,
and own `AutosaveGeneration`, demand index, manifest/ring, corruption isolation and recovery paging.
No scan-derived steady sequence/retention or private durable writer remains after migration.
Split the current recovery test owners by feature responsibility until every owner is at or below the
existing 800-line structure threshold; do not delete coverage or relax the contract. Run rustfmt on
the complete owned recovery slice after the split; current `mod.rs` and both oversized test owners
are the only 3 of 20 files failing the format gate.

### Editor14

Keep the existing open autosave adapter handoff open. Its bounded admission and completion-pump work
is current and should not be reimplemented. Extend the byte contract from request size to actual
payload/buffer ownership, add per-document-generation supersession, and expose project-generation
fence/terminal accounting. The editor must still use the unique job system and foreground save mutex.

### Editor03

Expose the existing bounded dirty-change cursor as the sole generation source for autosave demand.
Document close/save/project switch must emit enough delta/generation information to remove or
supersede demand without a full snapshot. Do not add autosave booleans or a parallel dirty owner.

### Editor16

Update the existing project-session-lock recovery handoff: current project activation now claims and
retains one OS-backed `SessionGuard`, so the old "guard owner missing" statement is obsolete. The
remaining work is restore admission/decision integration, a Runtime11-scheduled heartbeat, exact
project generation and close/shutdown fence-before-release. Preserve active/residual lock semantics.

### Runtime11

Provide a shared bounded streaming/atomic durable lane suitable for autosave snapshots, manifests and
session heartbeat. It must report generation, buffered bytes, write/flush/rename work, cancellation,
fault injection and fences. Do not create a recovery-private worker or weaken durability.

### EditorUI08

Wire retained autosave only after the structural hard cut. Stable frames must do no dirty/toolkit/path
projection; changed frames consume bounded deltas, and due frames select only the admitted page.
Completion status applies only to the current project generation.

### Render17

Own demand delta/resync, selection/examination, estimated/actual/peak payload bytes, mutex wait/hold,
manifest/reconcile/directory scans, write/flush/rename, project fences/stale completion, heartbeat and
recovery-page counters. Compare same-machine F0/F4 WPR CPU/thread/wake/lock/file-I/O p50/p95,
allocation/RSS/package power against local Unreal under matched project, frame cap and power plan.

## Primary-source gate

`PackageAutoSaver.cpp:1175-1218` maintains dirty map/content/user-restore sets from dirty callbacks and
removes clean packages. `DoPackagesNeedAutoSave` checks counts (`1288-1300`), and autosave consumes
those maintained sets (`330-383`). The next backup slot is modulo the configured maximum
(`350-357`). These are the required local primary-source grounds for incremental demand and O(1)
bounded slot selection. Unreal's synchronous slow-task save is not a threading model for Zircon.

## Acceptance additions

- Documents 1/100/10K, changed 0/1/100 and window 1/16: stable work zero, change work delta-scaled,
  due selection O(W), full dirty/toolkit/path projection and linear intent lookup zero.
- Payload 1KiB/64MiB/1GiB, writers 1/4/16 and stall 0/10ms/2s: queued payload zero, peak bytes under
  explicit cap, UI serialization/I/O wall zero, foreground save overlap zero.
- Entries/orphans 3/1K/100K: steady directory scans and metadata rereads zero, next slot O(1), latest
  bytes/retention/restart collision correct.
- Switch/close/crash at capture/write/flush/rename/manifest: every owned job terminal or fenced,
  stale-generation apply zero, guard release ordered, residual recovery preserved.
- Recovery documents 0/100/10K and corruption 0/1/10%: bounded page/bytes/time, valid candidates stay
  visible, repair cursor resumes.
- Current managed Cargo and F0/F4 WPR CPU/RSS/file-I/O/power evidence are mandatory. RenderDoc is not
  applicable to this CPU/filesystem module.

## Requested protected index state

- `pending.md`: replace the stale module row with one concise
  `zircon_editor/src/core/recovery/**` row, `static_complete / dynamic_pending`, current counts and
  review link.
- `review.md`: do not add the module until current managed Cargo, crash/restart and scale matrices,
  the recovery test-owner contract, product reachability, F0/F4 WPR and quantified CPU/RSS/power
  evidence pass.

## Milestone and notification state

This is a static architecture review, not an accepted performance milestone. No commit or WeCom
notification is due. Both become required after the dynamic matrix and protected indexes are
accepted by their owners.
