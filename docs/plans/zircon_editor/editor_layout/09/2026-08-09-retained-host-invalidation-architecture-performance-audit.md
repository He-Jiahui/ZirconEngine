---
record_kind: architecture_performance_audit
status: M0_partial / M1_foundation_complete / M2_limited_implementation / validation_pending
created_at: 2026-08-09
owner_plan: docs/plans/zircon_editor/editor_layout/09-incremental-message-bus-and-refresh.md
related_plans:
  - docs/plans/zircon_editor/editor_layout/17-text-rendering-and-typography.md
  - docs/plans/zircon_editor/editor_layout/21-gpu-submission-and-draw-pipeline.md
  - docs/plans/zircon_runtime/runtime/09/2026-08-09-ui-architecture-performance-reassessment.md
related_code:
  - zircon_editor/src/ui/retained_host/app/invalidation/root.rs
  - zircon_editor/src/ui/retained_host/app/invalidation/root/reasons.rs
  - zircon_editor/src/ui/retained_host/app/invalidation/root/requests.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute.rs
  - zircon_editor/src/ui/retained_host/ui/apply_presentation.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/projection_cache.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface/batching.rs
---

# Retained Host Invalidation Architecture And Performance Audit

## Scope And Decision

This is an investigation and implementation plan, not an accepted milestone or a
claim that a performance defect is fixed. It covers the editor retained-host path
from an editor action to a presented UI frame. The goal is to preserve the existing
Runtime text, layout, hit-test, batch-plan, damage, and resource caches by giving
them stable upstream generations. It is not to add another cache around a full
Workbench rebuild.

The next structural slice is a scoped frame transaction at the retained-host
boundary. It must carry exact invalidation reasons plus changed surface, pane,
window, node, and resource identities. A transaction commits the smallest set of
stage outputs that must change. Full host recompute remains legal only for an
explicit all-surface reason such as startup, root size, topology, global font
metrics, or a validated index mismatch.

## Evidence

### Current Source

The current source remains the primary evidence.

1. `HostInvalidationRoot` now stores an internal transaction keyed by
   `HostInvalidationScope::{All, View(ViewInstanceId)}` and aggregate counters.
   Repeated requests in one view merge their masks deterministically. Pane,
   window, node, and resource scopes are still unimplemented.
2. `RetainedEditorHost::recompute_if_dirty` now admits an exact
   `View(ViewInstanceId) + PRESENTATION_DATA` transaction for the currently
   presented UI Asset Editor pane. That path rebuilds only the pane surface frame,
   patches its host presentation data, and requests its content-frame redraw. All
   other layout/presentation/window-metrics recomputes still build a shell
   snapshot, synchronize viewport and pointer layouts, apply presentation,
   synchronize viewport and native window presenters, and synchronize pointer
   surfaces.
3. `apply_presentation_with_template_v2_data` constructs `ShellPresentation`,
   `HostWindowSceneData`, floating-surface data, host-contract projections, and a
   complete `HostWindowPresentationData` before replacing the host presentation.
4. `HostChromeProjectionCache` reuses individual tab, preset-name, and menu-chrome
   projections, but `build_host_scene_data_with_cache` still builds page chrome,
   status chrome, drawer chrome, and floating-window data for each admitted host
   recompute. The cache reduces copies but cannot constrain the recompute workset.
5. Pure `PAINT_ONLY`, `POINTER_HOVER`, and `VIEWPORT_IMAGE` requests already use a
   retained-host fast path. Render submission is separately consumed by
   `submit_render_frame_if_dirty`. The planned work must retain both paths.
6. `CompiledUiBatchPlanCache` already reuses the compiled plan when the draw-list
   generation and projection size are stable. It supports damage scissoring and
   cached full-projection statistics. The missing link is upstream generation
   stability, not a missing GPU batch cache.

### Current-Source Measurement Decision Matrix (2026-08-10)

`RecomputeInvalidationDecision` takes the exact `View + PRESENTATION_DATA`
transaction into `apply_scoped_ui_asset_presentation` only when no legacy dirty
flag is set. The successful path builds one pane projection per changed view and
records its root/native scan work. Every other admitted presentation recompute
still builds chrome and `WorkbenchViewModel`; shell-content reuse avoids only the
template-layout frame rebuild, not the model or `ShellPresentation` construction.
This makes the full path a justified candidate, but not proof of the current p95
bottleneck.

The next source-bound 24-iteration capture decides work in this order:

| Observed counters | Interpretation | Allowed next implementation |
| --- | --- | --- |
| `slow_path_rebuild_count` and `workbench_model_build_count` are nonzero for valid local UI Asset edits | The transaction was not admitted to the pane fast path. | Trace the invalidation producer and repair scope/reason routing before adding a cache. |
| Scoped patch count rises with nonzero projection/coverage fallbacks | The fast path cannot trust its presented locations. | Implement the planned generation-checked `PresentedPaneIndex`, with an explicit fallback reason. |
| Scoped patch succeeds but floating/native visit count grows with unrelated panes or windows | The current local patch still scans global presentation rows. | Replace scan discovery with the same `PresentedPaneIndex`; retain exact presenter coverage checks. |
| Scoped patch work is bounded but `gpu_batch_plan_builds`, text-shape work, or upload bytes rise | Upstream generations are unstable after patching. | Repair the changed command/text/resource range; do not add another batch-plan cache. |
| Counters remain bounded and the target p95 is within budget | The retained-host algorithm is not the active bottleneck. | Stop this optimization branch and inspect the measured downstream span instead. |

The profile manifest must be present before any row in this matrix is used. It
links the counters to the exact source/binary pair, while exported visual evidence
continues to live only under `docs/tests/editor`.

### Workbench Sparse Projection Metadata Delta (2026-08-10)

This is a source-level optimization result, not a runtime performance acceptance
claim. `EditorWorkbenchTemplateSurface` already owns a stable source projection,
node-to-host index, topology guard, and explicit full-projection fallback. Its
single-interaction regression proves `K = 1` for a hovered-control semantic
patch; its resize regression observes more than 1,000 geometry-only patches.
Before this delta, every semantic patch still rebuilt the projection binding map,
constructed a `requested_controls` map, and depth-first traversed all authored
projection nodes to find those `K` controls. Therefore a one-node patch retained
an avoidable `O(P)` authored-tree visit, where `P` is the template projection
node count, even though geometry-only updates already use indexed direct patches.

The workbench now builds one immutable `control_id -> {attributes, style_tokens}`
metadata index after route registration and before the source projection becomes
surface-owned. A semantic patch performs one metadata lookup per patched node,
then reapplies runtime focus state after authored metadata so the established
runtime-focus precedence remains unchanged. The full projection path, topology
and index mismatch fallback, surface binding resolution, and geometry-only patch
path are deliberately unchanged. The new profile counter
`ui.workbench_template.host_projection_metadata_lookup_count` records the actual
lookup count. The residual per-patch binding-map construction remains explicitly
out of scope for this delta; it requires separate measurements before any further
cache is introduced.

Correctness gates are: the pre-order/last-write-wins metadata-index unit contract,
the existing one-node incremental-versus-full-projection equality regression, the
existing geometry-only no-semantic-rebuild regression, and the existing invalid
index full-fallback regression. Static formatting and diff checks are required
before managed validation. A source/executable-bound Windows trace must still
measure one-node hover/focus patches and record both `K` and fallback counts; no
runtime duration, allocation, or screenshot claim is valid until the external
managed-validation preflight is recoverable. Rendered screenshots, when that gate
is available, remain under `docs/tests/editor` only.

The direct invalidation entry-point audit still finds unscoped
`mark_layout_dirty`, `mark_presentation_dirty`, `mark_render_and_presentation_dirty`,
and `invalidate_host(mask)` calls across hierarchy actions, asset refresh,
build/export, docking, welcome, inspector, and profiling actions. UI Asset Editor
actions that only synchronize the current editor session now use the view scope.
Actions that save/refresh the asset workspace, create external assets, or open a
new view remain explicitly global. The UI Asset pane path is the first real M2
consumer; it bypasses the Workbench model, shell geometry, viewport/pointer, and
native-window synchronization work. It still rebuilds the aggregate hit index
after the target pane model changes, and every other pane kind remains on the
full path.

### Existing Measurements

The latest documented product result in
`runtime/09/2026-08-09-ui-architecture-performance-reassessment.md` reports:

| Scenario | Reported result |
| --- | ---: |
| Button tick p95 | 10.19 ms to 6.44 ms (-36.8%) |
| 24 resize calls average | 21.84 ms to 13.76 ms (-37.0%) |
| 24 resize calls maximum | 28.43 ms to 18.48 ms |
| SVG/visual raster calls | 344 to 46 (-86.6%) |
| Retained texture copy during resize | 124,517,408 B to 6,293,408 B (-94.9%) |

The retained profile directories on `E:` are useful diagnostic evidence but are
not a fresh current-source acceptance result. In particular,
`runtime09-resize-direct-swapchain-uia-20260809-1920/ui_hotspots.json` records
25 full command rebuilds, 25 full paints, 31,129,352 painted pixels, and the
alerts `region_request_repainted_full_frame` and
`viewport_image_dirtied_layout_or_presentation`. Those counters conflict with the
later summary claim of one reuse build for the resize sequence. The conflict means
the source SHA, executable SHA, scenario input, and generated profile directory
must be captured together before either number is used for acceptance.

Windows Performance Recorder (`wpr.exe`), Windows Performance Analyzer, and
`xperf.exe` are available on this workstation. Existing
`tools/ui-profile-capture.ps1` now defaults to `E:\zircon-profiles` and resolves
only paths beneath that root. Each future run must use an
`E:\zircon-profiles\<source-bound-session>` output root. The script
now requires an absolute coordinator-managed Windows `CARGO_TARGET_DIR` and a
prebuilt profiling executable (`-SkipBuild`); it never invokes Cargo itself.
Trace-local captures remain under `E:` for comparison metadata, while every
verification PNG is exported to
`docs/tests/editor/profile-captures/<source-bound-session>`. No repository
`target` path is used for a profiling build, temporary capture project, or
verification image.

### Profile Capture Output Discipline (2026-08-10)

The profile tool now has a focused PowerShell contract test covering the managed
target requirement, the direct-Cargo rejection, the `E:` profile-project root,
and the `docs/tests/editor/profile-captures` verification export. The output-root
contract executes accepted-root, `target`, sibling-prefix, and `..` escape cases
against a side-effect-free path helper. The contract test and PowerShell AST parse
pass. A direct script launch for a harmless
`-SkipBuild` target-resolution probe was blocked by the workstation antivirus
before the script executed; it created no target, profile, or screenshot output.
This is environment evidence only, not a source-bound performance capture. Each
capture session now writes `source_manifest.json` before the editor starts. It
binds the scenario and capture limits to the repository revision, dirty-tree
digest, SHA-256 fingerprints for retained-host/text/batching source files, and
SHA-256 fingerprints for the exact editor and Runtime binaries. The manifest is
trace-local below `E:\zircon-profiles`; it records only the dirty-tree digest and
entry count, not unrelated local-change names. The manifest is fail-closed: a
missing Git revision/status, critical source file, or editor/Runtime fingerprint
prevents session-directory creation and editor launch. A future coordinator-owned
Windows run must still record that manifest, work counters, and exported
verification PNGs before performance or visual acceptance can be claimed.

## Reference Architecture

Unreal's `FSlateInvalidationRoot` is the primary design reference. Its public
contract owns a persistent fast widget list, cached draw elements, and a hit-test
grid. Child-order, prepass/layout, and post-update work use separate ordered heaps;
the final update list is painted without rescanning the tree. Root child order and
layout can request a slow path, while screen-position changes have a narrower path.

Zircon must copy these invariants, not Unreal type names:

- retained identities survive frames;
- invalidation carries both a typed reason and exact scope;
- layout work is separate from paint and resource work;
- a committed immutable generation is shared by renderer, pointer routing, and
  automation instead of rebuilding independent mutable snapshots;
- all full fallbacks have an explicit reason, visited count, and duration.

This aligns with the existing runtime `UiSurface` incremental layout, hit-grid,
damage, text measurement, and batch-plan cache contracts.

### Reference And Algorithm Gap Review (2026-08-10)

This review used the current retained-host implementation, the retained profile
artifact on `E:`, Unreal Slate FastUpdate, and Fyrox UI as separate evidence.
It does not treat a reference implementation or an old trace as a current-source
performance result.

| Evidence | Observed invariant or cost | Required Zircon response |
| --- | --- | --- |
| `FSlateInvalidationRoot` (`dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/FastUpdate/SlateInvalidationRoot.h`) | A persistent fast-widget list, widget index, hit-test grid, ordered pre-update/prepass/post-update heaps, and one final update list separate slow path from indexed fast updates. | Keep stable presented-pane identities and generation-checked locations; do not rediscover matching panes by scanning every surface for each local edit. |
| `SlateInvalidationRoot.cpp` | Root child order/layout force a slow path; ordinary widget invalidation is inserted once into ordered work queues and the final list is painted without a tree rescan. | Make all-surface fallback reasons explicit and measured. Pane-local presentation must not rebuild shell, pointer surfaces, or native presenters when its location index is valid. |
| `dev/Fyrox/fyrox-ui/src/widget.rs` | Layout invalidation is distinct from visual invalidation; measure and arrange validity are committed independently. | Keep `PRESENTATION_DATA` separate from layout/hit-test invalidation. Do not move text/layout work into a paint-only patch. |
| Current `HostInvalidationTransaction` and recompute decision | The transaction currently has only `All` and `View` scopes; only exact `View + PRESENTATION_DATA` is eligible for M2. All other scopes enter the full shell pipeline. | Add surface/pane/window/node/resource location scopes in a later M2 slice, with deterministic ancestor collapse and generation mismatch fallback. |
| Current scoped UI Asset patch | Each edit probes four docks, iterates root floating rows and root native mirrors, then `NativeWindowPresenterStore` iterates every child presenter. A changed floating list is cloned as a whole `ModelRc`. | Instrument visits/clones/fallbacks first; then replace scans with a maintained `ViewInstanceId -> PresentedPaneLocations` index and update only the identified presentation row. |
| `E:\zircon-profiles\runtime09-svg-m7\runtime09-resize-direct-swapchain-uia-20260809-1920` | This two-frame artifact records one `recompute_if_dirty` sample at 1623.03 ms, 25 resize full paints / command rebuilds, 31,129,352 painted pixels, and viewport-image full-frame/layout alerts. Its source/executable hashes are not bound to this M2 source. | Retain it only as a hotspot baseline. Do not quote it as an M2 improvement or compare it to a new build without matching source/executable identity. |

The immediate architectural target is therefore a single immutable
`PresentedPaneIndex` committed with each full presentation generation. A location
contains the owning host or native window, pane slot or floating row identity,
content frame, and the presentation generation from which it was derived. A local
transaction resolves its locations once, validates that all expected presenters
have the same generation, applies the one pane projection, queues only its damage
regions, and falls back only with a recorded reason when an index entry is absent
or stale. The index must be rebuilt only by the existing full presentation path;
it must not become a parallel scene truth.

Before that index is implemented, M2 instrumentation now records, for every
scoped patch: floating-window rows visited, presentation rows cloned, native
windows visited, damage-region count, projection-missing fallback count, and
presenter-coverage fallback count. A floating-row visit includes each actual
predicate probe, native-presenter identity collection, and patch traversal; a
projection-missing return retains the probe work already spent. These are emitted
through the existing `UiPerfCounter` stream, not a parallel metrics cache. The
source-bound profile manifest must include repository commit, dirty-tree digest,
executable hash, scenario input, and all six work counts. The acceptance run is
24 warm UI Asset edits at 640x420, 900x620, and 1280x720, with screenshots
copied only to `docs/tests/editor`; it must also prove zero full shell,
pointer-surface, native-presenter synchronization, text-layout, and GPU-plan
rebuilds for a valid indexed presentation-only edit.

## Target Transaction

```text
editor / OS / resource event
  -> EditorUiFrameTransaction
       reasons: exact dirty mask
       scopes: surface, pane, window, node, resource
       ordered barriers and latest-wins values
  -> retained UiSurface authority
       style/text/layout patch -> hit patch -> command/damage patch
  -> immutable frame generation
       renderer / pointer / automation read the same committed view
```

The initial transaction representation is internal to `zircon_editor` and must
not leak retained-host implementation types through `zircon_runtime_interface`.
It is not a compatibility facade: once migrated, ordinary editor actions must not
call the old unscoped dirty-marking methods.

## Delivery Sequence

### Current Implementation Status (2026-08-10)

| Item | State | Evidence |
| --- | --- | --- |
| M0 output-root discipline | Complete | `tools/ui-profile-capture.ps1` defaults to `E:\zircon-profiles`; its AST/path resolver contract passes. |
| M0 source-bound performance capture | Capture infrastructure implemented / runtime evidence pending | `source_manifest.json` now binds repository revision, dirty-tree digest, critical-source hashes, binary hashes, scenario, and options before launch. Missing Git revision/status, source hashes, or binary hashes fail closed before the session directory is created; no fresh matched E: trace exists yet. |
| M1 typed transaction | Complete foundation | `HostInvalidationTransaction` merges `All` and `View(ViewInstanceId)` masks; unit contracts cover merge and legacy-global behavior. |
| M1 UI Asset action routing | Complete foundation | Current-session actions are scoped; save, asset creation, and cross-view routes remain global by explicit contract. |
| M2 scoped stage consumption | Limited implementation / static telemetry review complete / managed validation pending | UI Asset Editor `View + PRESENTATION_DATA` builds one immutable pane projection per view transaction and reuses it for the matching root dock/floating pane and native presenters. The fast path requires the exact root-declared native `window_id` set to be patched by child presenters with the same native target, or falls back to full synchronization; full and scoped floating panes share exact per-window header/border content geometry. `UiPerfCounter` now exports scoped floating-row visit/clone, native-presenter visit, damage-region, projection-missing, and presenter-coverage fallback counts. Floating-row visits cover predicate probes, native identity collection, and patch traversals, including a root projection miss. Regressions cover root, detached native redraw, presenter identity completeness, floating geometry, two-native-window isolation, and all three telemetry fallback accounting paths. |
| M3 resize/resource admission | Pending | No new source-bound capture has validated resource or resize behavior. |

The implemented M0/M1/M2 items passed Rust formatting, whitespace,
path-resolution, scope-merge, pane-isolation, native-presenter patch, and
source-contract checks. The first 2026-08-10 M2 source review found no P0/P1/P2.
A subsequent independent delta review found and the current source forward-fixes
two issues: P1 native-presenter incompleteness could be hidden by a successful
root mirror patch, and P2 scoped floating geometry diverged from full conversion.
The first repair required complete native presenter coverage before the fast-path
return and shared the exact per-window header/border geometry helper with full
conversion. A later independent repair review found that count equality was still
not identity equality: a child presenting a different native target could satisfy
the count. The current forward repair compares `BTreeSet<MainPageId>` values and
accepts a child only when its configured native target equals that map key; the
new regression detaches two UI Asset panes to two native windows and proves that
only the matching child changes without a slow-path rebuild. The final independent
repair review found P0=0/P1=0/P2=0 for the exact identity contract and the shared
floating geometry; it was static/diff review only. A 2026-08-10 managed
`zircon_editor` scoped-test request did not return a terminal result before the
command host timeout; it is not evidence of success or failure. Managed Windows
build, runtime, performance, and screenshot validation therefore remain pending,
and this is not an accepted milestone.

The subsequent source-bound observability slice adds six work/fallback counters
to the existing profile stream. Its initial independent static review found two
P1 metric-accounting defects: uncommitted floating-row clones were not counted,
and damage already queued before a presenter-coverage fallback was omitted. The
forward repair counts every cloned floating row, aggregates root/native damage
before the coverage check, and adds regressions for both contracts. A second
delta review found two further P1 defects: predicate/native-identity scans were
not included in floating-row visits, and a root projection-missing return lost
the probe work already recorded below the patch boundary. `PresentationProbe`,
`NativePresenterLookup`, and the root early-return ordering now preserve all
three scan phases and their fallback work. The final independent static repair
review found P0=0/P1=0/P2=0. This remains non-visual and has no fresh managed
trace, so the review is not acceptance evidence.

### M0: Source-Bound Baseline And Observability

1. Extend the existing profile manifest with git/source hashes for retained-host,
   runtime text, and wgpu UI files; include executable SHA and scenario input.
2. Capture startup, UI Asset Editor property edit, hierarchy rename, viewport image,
   button click, and a 24-step resize storm using WPR plus
   `tools/ui-profile-capture.ps1` with an `E:` output root.
3. Record per-stage count, visited nodes, generated command ranges, damage bounds,
   GPU upload bytes, retained-copy bytes, and fallback reason. A zero counter is
   valid only when the relevant scenario actually exercised the stage.
4. Persist only profile data below `E:\zircon-profiles`; write rendered validation
   PNGs below `docs/tests/editor`, never under `target`.

Exit condition: each scenario can identify source/executable identity and explain
why layout, presentation, hit, command, image, and GPU stages did or did not run.

### M1: Scoped Retained-Host Transaction

1. Replace the root's mask-only pending state with one internal transaction that
   combines masks by scope and preserves ordered discrete barriers.
2. Define explicit all-surface, surface, pane, window, node, and resource scopes.
   Scope collapse is deterministic: node -> pane -> surface -> all-surface only
   when the reason requires it.
3. Migrate every retained-host invalidation entry point. Startup, root resize,
   topology, global theme/font metric changes, and detected index mismatch are the
   only all-surface producers.
4. Publish one generation only at transaction commit. Do not retain an old global
   mask fallback on normal action paths.

Exit condition: UI Asset Editor property edits and hierarchy renames carry a
specific pane/surface scope; unrelated drawers, native windows, and GPU draw-list
generations remain unchanged.

### M2: Scoped Stage Consumption

1. Split `recompute_if_dirty` into explicit transaction stages. Rebuild the host
   shell only for all-surface/root-layout/topology scopes.
2. Consume pane-local changes through the existing template runtime and projection
   caches; compute changed layout roots once, collapse ancestors, and run at most
   one layout pass per affected surface per frame.
3. Patch hit cells and command ranges using old/new bounds. Preserve text layout
   invalidation for text or font changes; never translate text blindly after a
   geometry change.
4. Feed unchanged `UiSurfaceDrawList` generations to the existing batch-plan and
   GPU resource caches.

Exit condition: a pane-local change has no full shell/model build, no unrelated
pointer-surface synchronization, and no full GPU plan rebuild.

### M3: Resource And Resize Admission

1. Coalesce watcher changes by canonical resource ID with ordered rename/delete
   barriers, then submit one transaction instead of one project refresh per partial
   batch.
2. Treat resize as one root layout transaction at settled size. Intermediate native
   events may update a preview surface but must not repeatedly rebuild model,
   text, command, or resource stages.
3. Use explicit fallback telemetry for backbuffer invalidity and cache-index
   mismatch; a region redraw that repaints the full frame must name its reason.

Exit condition: no stale `viewport_image_dirtied_layout_or_presentation` or
`region_request_repainted_full_frame` alert survives a fresh source-bound capture
unless its fallback reason is recorded and approved.

## Required Tests And Acceptance Data

- Unit: scope merge/collapse, latest-wins values, ordered barriers, generation
  commit, and explicit all-surface escalation.
- Integration: UI Asset Editor edit and hierarchy rename leave unrelated surface
  layout/model/presentation counters at zero; root resize performs one settled
  affected-surface layout pass.
- Runtime: versioned damage draw lists hit `CompiledUiBatchPlanCache`; unchanged
  image resources perform no upload; text changes invalidate only required text and
  layout ranges.
- Product: source-bound E: traces plus `docs/tests/editor` captures at 640x420,
  900x620, and 1280x720. Validate nonblank pixels, command presence, visual
  containment, and no output below `target`.
- Performance: report p50/p95/max, work counts, upload/copy bytes, draw calls,
  and fallback counts for at least 24 warm iterations. Do not compare results from
  different executable or source hashes.

This record stays unaccepted until M0 has fresh source-bound evidence, M1/M2
have independent review, and M2 validates the UI Asset path under managed Windows
execution. Expanding to hierarchy, inspector, asset, animation, and scoped hit
cells remains pending work rather than an implied result of this limited slice.
