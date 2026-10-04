---
related_code:
  - zircon_editor/src/ui/workbench/snapshot
  - zircon_editor/src/ui/workbench/project/asset_workspace_state.rs
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/shell/builder.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
owner_plans:
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
  - docs/plans/zircon_editor/editor/09-editor-asset-management.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
source_evidence:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Widgets/InvalidateWidgetReason.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp
  - dev/UnrealEngine/Engine/Source/Editor/ContentBrowser/Private/SAssetView.cpp
  - dev/UnrealEngine/Engine/Source/Editor/PropertyEditor/Private/SDetailsViewBase.cpp
  - dev/UnrealEngine/Engine/Source/Developer/OutputLog/Private/SOutputLog.cpp
---

# Protected plan routing: Workbench snapshot domain generation and projection

## Reason for routing

The main performance plan, `pending.md`, `review.md`, Optimize01/04/05/11 and numbered owner plans
are protected or foreign dirty. The active MVP00 session owns the Editor source tree; one scoped
source and one focused test also contain foreign changes. This record routes the 39/39-file current
evidence without editing those authorities. Detailed evidence source:
`2026-08-19-editor-ui-workbench-snapshot-domain-generation-projection-architecture-review.md`.

## Requested Performance01 updates

### PERF-MVP-095

Correct the accepted scope. Current source performs one catalog scan/sort, but
`build_surface_snapshots()` then deep-clones the complete Activity content into Explorer. The later
asset-type registry projection walks and rewrites both visible-asset vectors. Add acceptance for
second-surface content clone bytes `=0`, duplicate type projection `=0`, and one shared immutable
asset content generation with surface-local mode/view/utility state.

### PERF-MVP-099

Add the exact full-refresh chain under `WorkbenchShellStateData`: capability/customization and field
editor reconstruction, World/Inspector, both asset surfaces, two console authorities, descriptor/
instance maps, recursive Workbench snapshot, Workbench model and reflection publication. Required
target: short-lock generation capture, work outside the lock, exact per-domain invalidation and each
changed domain at most one build per frame/generation.

### PERF-MVP-102 and PERF-MVP-104

Retain PERF-MVP-102's accepted pointer-path fix, but do not infer that the asset snapshot producer is
accepted. Stable pointer reads reuse the committed Arc; any slow refresh still rebuilds folders,
visible rows/resource lookups and then deep-copies the second surface. PERF-MVP-104 must count those
rows, clone bytes, registry rewrites and resource shard probes independently when event batches force
an Editor/chrome snapshot.

### PERF-MVP-106

Add the upstream representation cascade to the Full fallback: owned manager descriptor/instance
vectors -> hash maps -> recursive `WorkbenchSnapshot` -> cloned active page/drawers -> another tab
model set. Final projection-cache hits do not erase this cost. Stable layout generation must make all
upstream map/box/page/drawer/tab/JSON clone work zero on data-only invalidations.

### PERF-MVP-567 and Optimize11 continuation

Add snapshot-side amplification to the existing Inspector and logging owners. Dynamic Inspector
projection clones components, schema and fields, then linearly searches fields per visible schema
field (`O(F^2)`). Product console builds the private output and then replaces it with a complete
Activity Log format/join before tail bounding. Route implementation to Optimize05 typed schema/value
slots and Optimize11's single journal cursor/window; do not create local snapshot caches.

## Requested Optimize and owner updates

### Optimize01 + EditorUI08 + Runtime09

Own immutable project/status, hierarchy, Inspector, asset, log, layout and render generations. One
frame receipt coalesces dirty domains. Shell access only captures handles/commits receipts; World
reflection, scans, model construction and reflection publication run outside it. Unrelated domain
visits and stable builds are exact zero, not equality-discarded temporary work.

### Optimize04 + Editor09 + Runtime04

Publish one asset content generation with normalized search keys, folder topology, resource/type
presentation and paged rows. Activity and Explorer share content and only own surface preferences.
Selection or one asset delta replaces exact slots. Resource lookup/index changes remain owned by
Runtime04 and follow the existing measured index gate; no Editor duplicate truth.

### Optimize05

Retain schema slot/order, customization identity and field-value generation. Inspector projection is
near dirty visible slots, not full dynamic-component/schema/value cloning. Remove quadratic field
name matching and support virtualized property rows with mixed/stale generation semantics.

### Optimize11

Delete the private console after migration. Journal query applies retention/page bounds before
formatting, append consumes a cursor delta, filter work is indexed/cancellable and stable chrome
does no scan or format.

## Requested protected index state

- `pending.md`: add or retain one concise row for `zircon_editor/src/ui/workbench/snapshot/**` with
  39/39 files, 2,110 lines, fingerprint `2eb3f12ccb72...`,
  `source_recheck_required=true`, and
  `static_complete / structural_cutover_required / dynamic_pending`.
- `review.md`: do not add the module. Require domain-generation cutover, shared asset/layout content,
  retained Inspector/log projections, current-source Cargo, F4, WPR/ETW, power and any applicable
  RenderDoc/pixel parity.
- Keep protected indexes module-level and concise; detailed evidence remains in the companion review.

## Acceptance handoff

| owner | required proof |
|---|---|
| Optimize01 + EditorUI08 + Runtime09 | unrelated domain visits/builds/clones `=0`; one changed-domain build per frame/generation; projection outside shell lock; reflection/surface parity |
| Optimize04 + Editor09 + Runtime04 | one shared asset content projection; second-surface clone/type rewrite `=0`; stable scan/resource probes `=0`; delta/page work bounded |
| Optimize05 | stable schema/component clone and field compare `=0`; no `O(F^2)` match; value work near dirty visible slots; mixed/stale parity |
| Optimize11 | one journal owner; retention bound before format; stable scan/format `=0`; append near delta; filter/page bounded |
| Performance01 | 1/1k/100k scale counters plus 31-run WPR/ETW CPU, allocation, lock, RSS, latency and package-power matrix on identical hardware/assets/settings; D/E/F artifacts |

RenderDoc is conditional on rendering-visible changes and proves draw/resource/pixel parity only. It
does not replace WPR/ETW evidence for snapshot CPU, allocation, shell locks or power.
