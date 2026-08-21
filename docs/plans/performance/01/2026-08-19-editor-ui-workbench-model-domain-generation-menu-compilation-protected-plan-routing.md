---
related_code:
  - zircon_editor/src/ui/workbench/model
  - zircon_editor/src/core/commands/menu.rs
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/shell/builder.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
owner_plans:
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
  - docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
  - docs/plans/zircon_editor/editor_layout/07-windowing-chrome-tabs-and-dockable-drawers.md
  - docs/plans/zircon_editor/editor_layout/08-plugin-page-interface-and-messaging.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-editor-event-full-reflection-rebuild.md
source_evidence:
  - dev/UnrealEngine/Engine/Source/Developer/ToolMenus/Private/ToolMenu.cpp
  - dev/UnrealEngine/Engine/Source/Developer/ToolMenus/Private/ToolMenus.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp
---

# Protected plan routing: Workbench model domain generation and menu compilation

## Reason for routing

Optimize01/08/50 are foreign dirty, and the shared Performance01 indexes and numbered owner plans
must remain single-owner authorities. This record routes the 44/44-file current evidence without
overwriting those plans. Detailed evidence source:
`2026-08-19-editor-ui-workbench-model-domain-generation-menu-compilation-architecture-review.md`.

## Requested Performance01 updates

### PERF-MVP-076

Retain P0 and make the menu cost explicit. The base model scans the complete command registry once
for each of seven top-level labels while the product holds the command mutex. The contributing path
then sorts all extension menu rows, allocates path vectors, uses linear recursive branch insertion
and recursively scans the menu tree for every contributed view. Acceptance must require stable
command/contribution generations to perform zero scans/sorts/row builds, one indexed structural
compile per changed generation, and context work near affected rows with no lock across projection
or publication.

### PERF-MVP-099

Expand the full-owned-model finding with the exact duplicate authorities. The active recursive page
is deep-cloned, flattened into owned document rows and also retained recursively; drawer snapshots
are cloned wholesale and separately projected into tool-window rows; floating windows repeat owned
tab rows. Require compact generation handles, one shared pane/tab artifact and shell serialized
payload clone bytes `=0`. Do not accept `Arc<WorkbenchViewModel>` as the ownership fix.

### PERF-MVP-105 and PERF-MVP-106

Pointer/focus/status/viewport changes must consume stable generation-owned row indexes and context
patches. They cannot construct the full model to discover one ID or rebuild menu/layout/document
domains. Main, native, floating and reflection consumers share the same immutable pane rows and
advance per-consumer cursors once per generation.

### PERF-MVP-107 and PERF-MVP-603

Plugin pane preparation and drag/drop must not trigger a new monolithic model with default or stale
contribution state. They read the mounted extension generation and canonical layout/session index.
Detach early-outs before model/menu work; drop commits one typed layout delta and invalidates only
affected layout/pane rows.

## Requested Optimize and owner updates

### Optimize08 + Editor08

Publish a command/menu structural generation with stable top-level buckets, immutable row metadata,
hierarchy index and operation-to-row index. Context dependencies produce compact enablement/check/
visibility patches. Menu widget generation occurs at explicit menu-open/refresh boundaries, not on
every shell event. Snapshot immutable handles under the command lock and release it before build.

### Optimize50 + EditorLayout08

Every contributed menu/view row carries owner and mount generation. Enable, disable, revoke and
reload apply one indexed diff and atomically remove stale rows/callbacks. Product consumers cannot
recollect the complete contribution snapshot and reconstruct the tree on each model build.

### Optimize01 + EditorUI08

Replace the monolithic Workbench model with a typed domain DAG: layout, session/view, command/menu,
context and status generations. Dirty IDs coalesce once per frame; unchanged domains perform zero
builds. Classic reflection and retained host snapshot generation handles under short locks, build
outside locks and generation-check commit. Serialized view payload stays with the active content
owner rather than shell projections.

### EditorLayout07

Page, drawer, document tab and floating window topology must publish stable row IDs and affected
subtree deltas. Drawer ring, tool-window, native/floating and reflection presenters share one
generation artifact. Do not add another owned tab graph to compensate for the current duplicate
Workbench model.

## Requested protected index state

- `pending.md`: add or retain one concise module row for
  `zircon_editor/src/ui/workbench/model/**`, 44/44 files, 1,179 lines, fingerprint
  `005b3d2a9f8c...`, `source_recheck_required=true`, and
  `static_complete / domain_generation_hard_cut_required / dynamic_pending`.
- `review.md`: do not add the module. Require typed domain generations, indexed menu owner
  lifecycle, zero stable-domain work, short-lock generation snapshots, current-source managed Cargo,
  F4 and WPR/ETW CPU/allocation/lock/power evidence.
- Keep protected indexes module-level and concise; detailed evidence remains in the companion review.

## Acceptance handoff

| owner | required proof |
|---|---|
| Optimize08 + Editor08 | stable generation command/menu scans, sorts and row builds `=0`; one structural compile per change; context visits near affected rows; command lock does not span build/publish |
| Optimize50 + EditorLayout08 | owner-generation menu/view add, revoke and reload are one indexed diff; stale rows/callbacks and full recollection `=0` |
| Optimize01 + EditorUI08 | unchanged domain build/visit/clone bytes `=0`; each dirty domain builds at most once per frame; shell/command lock hold and wait stay within budget |
| EditorLayout07 | active page is a stable handle; pane/tab data materializes once per generation; shell serialized payload clone bytes `=0` |
| Performance01 | 31-run F4 menu/input/layout/plugin WPR/ETW CPU, allocation, lock, input latency, RSS and package-power matrix; artifacts on D/E/F |

RenderDoc remains conditional on rendering-visible changes. It cannot replace CPU complexity,
allocation, lock or package-power evidence for this module.
