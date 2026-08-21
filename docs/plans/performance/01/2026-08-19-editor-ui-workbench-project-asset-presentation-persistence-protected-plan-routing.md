---
related_code:
  - zircon_editor/src/ui/workbench/project
  - zircon_editor/src/ui/host/layout_persistence.rs
  - zircon_editor/src/ui/host/project_access.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/pane_payloads.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
owner_plans:
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/13-layout-profile-workspace-state-docking-tab-window-restore-migration-review.md
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
  - docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
  - docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
source_evidence:
  - dev/UnrealEngine/Engine/Source/Editor/ContentBrowser/Private/SAssetView.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/TabManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/LayoutService.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/PackageAutoSaver.cpp
---

# Protected plan routing: Workbench project asset presentation and persistence

## Reason for routing

The main performance plan, `pending.md`, `review.md`, Optimize02/04/13 and numbered owner plans are
protected or foreign dirty. The active session owns the Editor source tree and six focused
source/test files contain foreign changes. This record routes the 16/16-file current evidence
without editing those authorities. Detailed evidence source:
`2026-08-19-editor-ui-workbench-project-asset-presentation-persistence-architecture-review.md`.

## Requested Performance01 updates

### PERF-MVP-095 and PERF-MVP-102

The accepted single asset scan does not make the producer stable. Every aggregate snapshot still
rebuilds/sorts folder topology, scans all assets, allocates parent path strings, lowercases searchable
fields, clones visible rows and resource state, then deep-clones content into the second surface.
Add acceptance for stable visits/sorts/path/lowercase/row/resource work `=0` and second-surface
content clone bytes `=0`. Pointer-cache hits cannot satisfy producer acceptance.

### PERF-MVP-099 and PERF-MVP-106

Add layout preset discovery to the shell-held aggregate/pane payload chain. Full and targeted pane
recompute call `preset_names()`, which clones every Runtime asset URI, filters/sorts/deduplicates and
decodes the config preset map. Required target: a shared immutable preset catalog generation;
unrelated presentation changes perform zero registry/config work and projection happens after short
generation capture.

### PERF-MVP-100 and PERF-MVP-107

Extend the existing unchanged preset/filesystem gate to the actual product path. Stable full/native/
targeted pane collection must have registry visits, URI clones, config loads/decodes, metadata probes
and preset sorts `=0`. A downstream Slint model equality hit is insufficient. One project/config
delta builds and publishes at most one preset generation shared by main and native presenters.

### PERF-MVP-104

When asset event batches request presentation refresh, count the producer work separately: folder
visits/sorts, full asset visits, parent-path and lowercase allocations, visible row/diagnostic clone
bytes, resource probes, selection detail work and second-surface clone bytes. Same-generation
refresh must consume one published asset-presentation generation, not rebuild it under snapshot.

## Requested Optimize and owner updates

### Optimize04 + Editor09 + Runtime04

Publish one immutable asset-presentation generation from Runtime registry deltas. Retain folder
topology/order, direct membership, normalized search keys, type/resource presentation and visible
pages. Activity and Explorer share content and keep only local mode/view/utility/selection state.
Runtime04 must provide delta/prefix/type queries without cloning every project asset URI.

### Optimize13 + EditorUI08

Add `LayoutPresetCatalogGeneration` as the only presentation owner for project and config preset
names. Asset/config change events invalidate it; full and targeted recompute read an Arc slice and
generation receipt. Remove registry enumeration, config decode, sort and dedup from pane payload
collection. Keep named layout save/load as explicit commands, not presentation work.

Optimize13 must also route workspace and preset documents through bounded version dispatch,
migration, validation, staged restore, atomic write, last-known-good and quarantine. Named preset
load currently ignores its own version/name, and direct `fs::write` is not a durable baseline.

### Optimize02 + Editor17

Make project scene and workspace typed participants in the shared asynchronous save coordinator.
Preserve the tested previous-workspace compensation; do not remove the capture read as a local I/O
optimization. Encode/write/import/catalog repair moves off the input thread, completion is
generation-bound, and post-commit projection failure remains a durable retryable state.

### Editor10

Keep path/URI resolution at explicit command admission. Add a source guard preventing
`project_root_path()` and source-path resolution from frame, pointer, presentation or snapshot
helpers. Cache any admitted path by active project generation and invalidate it on project switch.

## Requested protected index state

- `pending.md`: add or retain one concise module row for
  `zircon_editor/src/ui/workbench/project/**`, 16/16 files, 1,283 lines, fingerprint
  `fed12d140e74...`, `source_recheck_required=true`, and
  `static_complete / structural_cutover_required / dynamic_pending`.
- `review.md`: do not add the module. Require shared asset/preset generations, asynchronous save,
  bounded durable restore, current-source Cargo/F4, WPR/ETW allocation/lock/CPU/power and applicable
  RenderDoc/pixel parity.
- Keep protected indexes module-level and concise; detailed evidence remains in the companion review.

## Acceptance handoff

| owner | required proof |
|---|---|
| Optimize04 + Editor09 + Runtime04 | stable asset visits/sorts/path/lowercase/row clone/resource probes `=0`; second-surface clone `=0`; delta/page work bounded at 1/1k/100k assets |
| Optimize13 + EditorUI08 | unrelated full/targeted/native pane recompute registry/config/preset work `=0`; one preset generation per change; versioned bounded atomic restore/persistence parity |
| Optimize02 + Editor17 | bounded UI capture, background encode/write, generation receipt, compensation parity, retryable post-commit projection failure and disk/crash fault injection |
| Editor10 | project path/source resolution absent from frame/pointer/projection paths; project-generation invalidation and path error parity |
| Performance01 | 31-run WPR/ETW CPU, allocation, shell lock, input latency, RSS and package-power matrix on identical hardware/assets/settings; artifacts on D/E/F |

RenderDoc remains conditional on rendering-visible changes and proves GPU event/resource/pixel parity
only. It cannot replace WPR/ETW evidence for catalog CPU, filesystem stalls, shell locks or power.
