---
related_code:
  - zircon_editor/src/ui/workbench/autolayout
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/shell
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench
  - zircon_editor/src/ui/workbench/reference/template_surface.rs
  - zircon_runtime/src/ui/layout
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
plan_sources:
  - docs/plans/performance/01/2026-08-22-editor-ui-workbench-autolayout-single-runtime-layout-authority-architecture-review.md
owner_plans:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/13-layout-profile-workspace-state-docking-tab-window-restore-migration-review.md
  - docs/plans/optimize/zircon_runtime/76-runtime-ui-layout-box-model-measure-arrange-flex-grid-overflow-scroll-virtualization-dpi-product-integration-review.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_editor/editor_layout/07-windowing-chrome-tabs-and-dockable-drawers.md
source_evidence:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/SWidget.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/SInvalidationPanel.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/SBoxPanel.cpp
doc_type: protected-plan-routing
status: requested
created_at: 2026-08-22
---

# Protected plan routing: Workbench autolayout single runtime authority

## Reason for routing

Performance01, `pending.md`, `review.md` and numbered optimize/editor/runtime plans are shared owner
authorities. This record routes current 44/44-file evidence without overwriting concurrent work.
Detailed source and acceptance evidence is in
`2026-08-22-editor-ui-workbench-autolayout-single-runtime-layout-authority-architecture-review.md`.

## Requested Performance01 updates

### Add one dedicated P0 shell-layout-authority item

Record the current normal slow path as manual `WorkbenchShellGeometry` plus root template runtime
layout plus mounted workbench runtime layout plus floating source layout/check. The manual and
bridge paths use different width budgeting policies and publish frames to different consumers.

Acceptance must require:

- `.zui`/runtime `UiSurface` is the sole normal-product pixel geometry authority;
- paint, hit, drag, native projection and accessibility consume one immutable layout generation;
- manual geometry normal-product caller/solve/fallback count is zero after hard cut;
- stable generations perform zero model/index/token/layout/allocation/publication work;
- one changed shell generation produces at most one runtime layout transaction and publication.

Add the P0 correctness evidence: current 640px defaults have 638px row budget but minimums
`220 + 640 + 34 = 894`, and the manual path publishes a right edge at 896. The existing narrow test
does not assert containment. M0 requires a red invariant test before any temporary fallback safety
patch.

### PERF-MVP-106

Expand the full-recompute DAG evidence. Shell snapshot reuse is a late post-solve comparison: full
layout/descriptors/chrome/model/token preparation and manual geometry happen before the cache hit can
skip the workbench bridge. WindowMetrics still runs manual geometry and both root/workbench bridge
passes. Require a pre-solve generation gate and counters separating model, manual solve, root
template, workbench template, floating source and publication work.

### PERF-MVP-129

Keep the accepted same-size floating-source no-op, but do not treat it as shell-layout convergence.
The floating source remains one of several independently generated frame products. Its final owner
must be the same runtime layout-frame generation or an explicitly mounted child surface with a
generation relation; same-size reuse count is one local gate only.

### PERF-MVP-131 and PERF-MVP-077

Drawer resize and layout/page commands must publish one typed state/layout delta into the runtime
layout transaction. They must not trigger both `compact_side_widths` and `reserve_document_width`,
nor full model/geometry work per command. No-move/no-op behavior remains zero transactions; changed
batched behavior is at most one transaction/publication per presented frame.

### Pending/review accounting

Record `zircon_editor/src/ui/workbench/autolayout/**` as one concise module entry:
`44/44 static reviewed; single runtime layout authority hard cut and dynamic evidence pending`.
Do not list 44 paths individually. Do not move it to `review.md` until M0-M5 in the detailed review
pass on the same source fingerprint.

## Requested Optimize01 updates

Optimize01 already identifies that auto-layout expands local dirty state to the highest continuous
container root and calls for bottom-up desired-size propagation. Add the editor bridge prerequisite:

- delete ordinary `mark_roots_layout_dirty` calls;
- stop calling unconditional `UiSurface::compute_layout` for stable/model-only projection;
- mutate typed properties, then use one runtime `rebuild_dirty` transaction;
- expose full/incremental layout count, visited/skipped/geometry-changed nodes and full-pass reason;
- publish one immutable frame generation shared by hit/render/native/accessibility consumers.

Do not add a second editor-specific incremental scheduler. Runtime UI owns desired-size queues,
layout owner selection and fallback reasons, following the Slate single-widget-tree direction.

## Requested Runtime UI76 updates

Runtime UI76 owns layout correctness and proportional measure/arrange. Add these requirements:

- over-constrained minima return a typed deterministic outcome and never silently publish frames
  outside the root; add property/invariant tests across widths, scales and container backends;
- layout transactions expose whether work was full, incremental or generation no-op;
- root-size changes may be full, but only once per surface generation;
- desired-size and slot/index artifacts are generation-owned and reused across editor surfaces;
- final runtime layout frame is the geometry source for arranged tree, hit grid, render extract and
  accessibility publication.

The editor's manual solver is not a runtime fallback implementation to preserve. It must be deleted
after a bounded parity period.

## Requested EditorUI08 updates

EditorUI08 already states `.zui` is the shell layout authority and "editor changes state, runtime
changes pixels." Reopen incomplete migration gates because current source still executes manual
shell geometry in the same recompute as runtime bridge layouts.

Required replacing milestone:

1. Compile shell components, region bindings, control IDs and token slots into one
   `EditorLayoutDefinitionGeneration` shared by root/workbench/floating mounted surfaces.
2. Publish business-state deltas only; consume runtime layout frames directly for regions,
   splitters, viewport, popup anchors, drag and native projection.
3. Remove normal-product calls to `compute_workbench_shell_geometry*`, then delete
   `WorkbenchShellGeometry` and obsolete bridge/facade code in the same hard cut.
4. Delete or product-compile `shell_regions.toml` and CSS declaration parsing; test-only parsing is
   not completion.
5. Put the generation/no-op gate before chrome/model/descriptor/token preparation and coalesce dirty
   domains into one layout publication per frame.

Temporary manual geometry may exist only behind a named fallback/oracle reason with containment and
parity tests. It must never co-execute on the accepted normal product path.

## Requested EditorLayout07 and Optimize13 updates

EditorLayout07 supplies stable dock/stack/region identities and typed deltas. Its layout tree must
not calculate a second set of pixel frames. Persistent/profile topology belongs to Optimize13;
runtime pixel geometry belongs to the runtime surface generation.

Add one contract between them:

```text
exact LayoutProfile / LayoutTransaction
  -> stable business topology generation + typed changed owners
  -> runtime UI property/layout transaction
  -> immutable pixel frame generation
```

Page switch, dock, detach and drawer resize must use that route once. Persistence, migration and
durability remain outside UI locks and layout scopes.

## Required M0 evidence handoff

Owner implementations must return scenario-exported counters for:

- pre-solve generation hit/miss;
- manual shell solve count/time;
- descriptor scans/index builds and token/transient conversions;
- root/workbench full and incremental runtime layout counts;
- floating source layout/reuse count;
- root layout-dirty marks;
- visited/skipped/geometry-changed nodes;
- temporary layout allocation count/bytes;
- immutable frame publications and consumer generation IDs.

The baseline matrix includes startup, idle, hover, click, drawer resize, window resize, page switch
and plugin add/remove/reload. Stable-generation work is zero. Changed work is at most one layout
transaction/publication per frame and proportional to affected layout owners.

## Acceptance handoff

Completion must return one current-source bundle containing:

1. 640px red-to-green containment plus broad finite/nonnegative/containment/overlap invariants;
2. temporary manual/runtime parity proof followed by normal-product manual caller count zero and
   hard deletion;
3. product proof of one compiled shell/region/token definition owner and no dead parallel asset;
4. managed Windows Cargo/F4 and real-window drawer/resize/page/plugin lifecycle tests;
5. WPR/ETW CPU, allocation, lock, input-to-pixel p50/p95/p99, RSS and package-power evidence with
   artifacts only on D/E/F;
6. RenderDoc pixel/draw parity only when visible GPU UI output changes.

Until this handoff, keep the module pending and do not issue a milestone commit or WeCom completion
message for this workstream.
