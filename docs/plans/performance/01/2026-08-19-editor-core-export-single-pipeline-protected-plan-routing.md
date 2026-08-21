---
related_code:
  - zircon_editor/src/core/export
  - zircon_editor/src/ui/host/editor_manager_plugins_export/export_build/wizard
  - zircon_runtime/src/asset
  - zircon_runtime/src/plugin/export_build_plan
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/review.md
  - docs/plans/performance/pending.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
owner_plans:
  - docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
  - docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_plugins/09-export-publishing.md
source_evidence:
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/Scripts/BuildCookRun.Automation.cs
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/Scripts/BuildProjectCommand.Automation.cs
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/Scripts/CookCommand.Automation.cs
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/Scripts/CopyBuildToStagingDirectory.Automation.cs
  - dev/UnrealEngine/Engine/Source/Programs/UnrealBuildTool/Modes/BuildMode.cs
---

# Protected plan routing: one export graph and manifest-driven stages

## Reason for routing

The main performance plan, `review.md`, `pending.md`, optimize plan and numbered owner plans are
protected/foreign dirty. This record routes the current-source findings without overwriting their
owners. Evidence source:
`2026-08-19-editor-core-export-single-pipeline-architecture-revalidation.md`.

## PERF-MVP-071 correction

Retain PERF-MVP-071 at P0, but promote its root cause from inventory implementation to product
ownership:

- current scope remains 9/9 files, 3,061 lines, 24 tests, fingerprint `361a4a15...`;
- the outer wizard already owns the selected eight-stage graph and cancellation;
- CompileHost creates a nested one-stage core graph and private `.core.json` report;
- PlatformBundle later creates a fresh executor and runs the two-stage core graph, replaying the full
  CompileHost source inventory and three/four tool probes before it can validate the bundle;
- the editor duplicates build-system dependency truth with a hand-maintained recursive source list;
- `Drop` persistence, fingerprint fallback and the system runner remain outside shared scheduling and
  cancellation budgets.

Required target: one `ExportRunGeneration`, one graph, one receipt journal, and explicit
BuildProduct/CookedArtifact/PluginPackage/Pack/StagedBundle manifests. Delete nested core graphs and
`.core.json`; let the build owner determine outdated actions; let Stage copy one explicit destination
mapping incrementally. Keep strong file identity and streaming BLAKE3 only as changed/untrusted-entry
fallback.

PERF-MVP-558 remains the output paging/durability task. It must converge on the shared Runtime11
process/output authority; do not optimize the private system runner as a second permanent path.

## Requested owner-plan updates

### Editor15

Make one headless export graph the sole implementation used by UI, commandlet and CI. Remove
`ExportWizardCoreStageProjection`, nested plan construction and private core report. Split engine
build-product validation from project bundle staging, then consume typed upstream receipts. Required
artifact identity and diagnostic/log retention must be separate.

### Runtime04

Return content-addressed cooked and packed manifests keyed by source asset/cook generations. Pack and
Stage consume existing chunks/receipts and do not reread, decode or hash unchanged source assets.

### Plugins09

Publish one native package/file manifest and carrier receipt for the export generation. Validate,
NativeDynamic, Stage and Report consume it; none rebuilds plugin inventory independently.

### Editor14 and Runtime11

Own the single process tree, bounded output pages, fingerprint fallback, incremental copy and explicit
cache/report persistence jobs. Enforce entry/byte/age/deadline/cancellation budgets. Private reader
threads and `Drop` I/O are forbidden.

### Optimize zircon_editor/01

Replace any export scenario that measures only UI responsiveness with stage-resolved F4 evidence:
graph/execution counts, build action receipt, cook/pack/stage manifests, file/process I/O, waits,
wakeups, RSS and package power across cold/warm/1% changed runs. Stable editor frames still perform
zero export work; an active export performs no UI-thread build/cook/copy/durability work.

## Requested protected index state

- `pending.md`: retain `zircon_editor/src/core/export/**` as 9/9, 3,061 lines, 24 tests, fingerprint
  `361a4a15...`, `static_complete / structural_cutover_required / dynamic_pending`; link the new
  architecture revalidation and corrected PERF-MVP-071.
- `review.md`: do not add the module. Require one graph/report/inventory generation, manifest-driven
  Build/Cook/Pack/Stage, no nested core pipeline, bounded cancellation/persistence and 31-run F4
  WPR/ETW evidence plus exported-product launch.

## Milestone and notification state

This is static architecture evidence and routing, not an accepted milestone. No git commit or WeCom
notification is due. Both become mandatory only after owner implementation, dynamic acceptance and
protected index reconciliation are complete.
