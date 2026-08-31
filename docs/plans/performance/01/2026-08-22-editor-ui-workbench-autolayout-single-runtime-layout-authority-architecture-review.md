---
related_code:
  - zircon_editor/src/ui/workbench/autolayout
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/shell
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/recompute/floating_projection.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench
  - zircon_editor/src/ui/workbench/reference/template_surface.rs
  - zircon_runtime/src/ui/layout
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
tests:
  - zircon_editor/tests/integration_contracts/workbench_autolayout.rs
  - zircon_editor/src/tests/workbench/layout/editor_layout_contracts/breakpoints.rs
  - zircon_editor/src/tests/workbench/layout/editor_layout_contracts/geometry.rs
  - zircon_editor/src/tests/workbench/layout/editor_layout_contracts/region_contracts.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/pending.md
  - docs/plans/performance/review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/13-layout-profile-workspace-state-docking-tab-window-restore-migration-review.md
owner_plans:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_editor/editor_layout/07-windowing-chrome-tabs-and-dockable-drawers.md
  - docs/plans/zircon_runtime/runtime/09/2026-08-09-ui-architecture-performance-reassessment.md
  - docs/plans/optimize/zircon_runtime/76-runtime-ui-layout-box-model-measure-arrange-flex-grid-overflow-scroll-virtualization-dpi-product-integration-review.md
reference_code:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/SWidget.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Widgets/SInvalidationPanel.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Widgets/SBoxPanel.cpp
doc_type: current-architecture-performance-review
status: static_complete_single_runtime_layout_authority_hard_cut_required_dynamic_pending
source_recheck_required: true
created_at: 2026-08-22
---

# Editor workbench autolayout single-authority architecture review

## Status

- Result: `static_complete / single_runtime_layout_authority_hard_cut_required /
  dynamic_pending`.
- MVP priority: P0 for bounded, internally consistent shell geometry and one layout authority; P1
  for allocation/index cleanup after the authority hard cut.
- Accounting: keep `zircon_editor/src/ui/workbench/autolayout/**` in `pending.md` as one concise
  `44/44 static reviewed, dynamic pending` module entry. It must not move to `review.md` until the
  correctness, authority, counter, current Cargo, real-window, WPR/ETW and power gates below pass.
- Code disposition: no Rust source changed. Fixing only the manual solver would preserve a second
  product geometry authority and violate EditorUI08's stated `.zui`/runtime ownership.

## Exact scope

| scope | files | physical lines | tests | raw bytes | sorted path-LF-content SHA256 |
|---|---:|---:|---:|---:|---|
| `zircon_editor/src/ui/workbench/autolayout/**` | 44/44 | 4,466 | in-module tests included | 152,690 | `466d32bce76a0b8dc290aacb689645ab7379daf21262cf0ff90e2df177b97041` |
| four focused integration/contract files listed above | 4/4 | 909 | 25 | 29,434 | `e9764783a19a6f454bb830bc890c4a82ecaeec63ff19101d0f0d0cf23df91993` |

The source fingerprint is SHA256 over each sorted repository-relative path, LF, then raw bytes.
All 44 production/module-test files and all four focused test files were read in full. The product
trace additionally covers retained shell snapshot construction, root/workbench template bridges,
floating-window source projection, runtime `UiSurface` full/incremental rebuild and UI performance
counter/capture surfaces.

## Current architecture verdict

| area | current-source verdict |
|---|---|
| business inputs | `WorkbenchLayout`, descriptors, chrome snapshots, drawer model, design tokens, transient resize extents and scale mode all feed the manual shell solver. |
| manual geometry | `compute_workbench_shell_geometry_with_region_defaults_and_scale_mode` independently computes region, splitter, viewport and floating frames into owned maps. |
| root template | the same shell recompute calls the root template bridge, which rebuilds/projects its runtime surface and derives the workbench mount frame. |
| workbench template | the mounted bridge applies model/chrome/drawer/responsive mutations, marks every surface root layout-dirty, then calls `EditorWorkbenchTemplateSurface::recompute_layout`. |
| runtime pass used | `EditorWorkbenchTemplateSurface::recompute_layout` calls `UiSurface::compute_layout`, an unconditional full tree layout, arranged-tree rebuild, hit-grid rebuild, render extract, popup reconciliation and projected hit-test rebuild. It does not use `rebuild_dirty`. |
| floating source | the slow recompute also invokes a separate floating-window source surface. Same-size calls now no-op, but its frames and native projection remain a separate layout product. |
| cache boundary | manual model/descriptor/token preparation and full manual geometry finish before `shares_mounted_layout_frames_with` decides whether workbench bridge frames can be reused. A cache hit therefore skips only a late bridge pass. |
| publication | manual `WorkbenchShellGeometry` and componentized runtime frames feed different retained presentation, hit, drag, native and viewport consumers. No parity gate proves they are the same generation or geometry. |
| declarative sources | `shell_regions.toml` and `WorkbenchShellRegionsAsset` are test-only. CSS-like declaration parsing is test-only; production constructs a CSS-like value only to resolve one token extent. Product defaults come from hardcoded `WorkbenchSkeleton::jetbrains_default()`. |
| observability | scopes expose shell/model/root/workbench/floating phases and a late layout-cache hit, but scenario counters do not attribute manual solves, full runtime passes, root dirties, temporary allocations or pre-solve reuse. |

The module is not one auto-layout implementation. It is a parallel editor geometry system next to
the runtime UI layout system that EditorUI08 already declares authoritative.

## P0 findings

### P0: the manual horizontal solver can publish frames outside the shell

`zircon_runtime::ui::layout::solve_axis_constraints` correctly preserves hard minimums when their
sum is larger than available space. `compact_side_widths` then limits only side widths and adds any
released side width back to the document. It never resolves a remaining all-minimum deficit or
verifies that the final row fits.

The current default 640 logical-pixel fixture is a deterministic counterexample:

```text
shell width                                      640
two 1px separators                               -2
available row width                              638
left Project minimum                             220
active Scene document minimum                    640
right collapsed rail minimum                      34
minimum total                                    894
unresolved deficit                               256
published right edge                             896
```

The resulting sequence is left `[0,220]`, document `[221,861]`, right `[862,896]`. The existing
`narrow_workbench_geometry_collapses_right_drawer_to_rail` test checks only the right width,
splitter width and a relative document-width inequality. It does not assert finite/nonnegative
frames, final right edge, sum conservation, containment or overlap. The componentized bridge test
at the same 640 width expects the right drawer shell to be absent, which further demonstrates that
the two paths do not even expose one identical compact-layout contract.

The red test must be added before a safety change. If the manual path remains temporarily as a
fallback/oracle, it must resolve over-constrained minima by an explicit policy and never publish an
out-of-bounds frame. That safety fix does not close the architectural item.

### P0: one slow shell recompute executes multiple layout authorities

The current slow path performs, in order:

```text
read/clone layout and descriptors
  -> build chrome and full WorkbenchViewModel
  -> project chrome state to bridge
  -> convert cached logical token extents to a new physical map
  -> manual WorkbenchShellGeometry solve
  -> compare completed manual geometry for late reuse
  -> root template runtime layout/project
  -> mounted workbench runtime layout/project
  -> floating source layout check/project
  -> retained presentation/native consumers
```

The manual `compact_side_widths` policy and bridge `reserve_document_width` policy are separate
algorithms. The bridge additionally calls `mark_roots_layout_dirty`, then takes the full
`compute_layout` path, so runtime incremental layout cannot amortize the work. Root-size or state
changes are allowed to require layout, but they must enter one runtime layout transaction, not two
pixel solvers plus multiple surfaces with unrelated publications.

### P0: the reuse decision occurs after the expensive work it should prevent

`requested_shell_layout_reuse` is checked only after layout/descriptors/chrome/model/token
conversion and manual geometry computation. `WindowMetrics` reuses committed model values but
still reruns manual geometry plus root/workbench bridge layout. Stable shell content therefore has
no pre-solve generation gate, and a reported `ui.shell_content.layout_cache_hit_count` does not mean
that shell layout preparation was avoided.

The gate must compare immutable input generations before model materialization, descriptor indexing
or geometry work. Stable definition/layout/chrome/window/scale generations require zero manual
solve, zero root/workbench/floating surface layout, zero descriptor scan and zero temporary layout
allocation.

## P1 findings

### P1: every manual solve rebuilds indexes and short-lived containers

The solver rebuilds a descriptor `HashMap<&str, &ViewDescriptor>`, region constraint vectors,
solved-axis vectors, region/splitter/floating `BTreeMap`s and a physical-to-logical transient map.
The retained builder first converts cached logical token extents into a newly allocated physical
map; geometry then converts that input back into another logical map. Active tabs, expansion and
extent are derived through separate model/slot traversals, and floating frame publication clones
window IDs.

Small fixed region counts make any one allocation modest, but their frequency is one per shell
recompute and they sit before the late reuse decision. Do not micro-optimize these containers as the
first milestone. Compile descriptor/region/token lookup into the same immutable editor-layout
definition generation, then remove the manual path from normal product execution. Any remaining
fixed four-region runtime representation should use typed fields/arrays rather than tree maps.

### P1: dead declarative surfaces create a third definition story

`shell_regions.toml` is included only by contract tests. `WorkbenchShellRegionsAsset` has no product
consumer. `CssLikeConstraint::from_declarations` and `apply_declaration` have no production caller;
the only production `into_layout_style` caller constructs a constraint merely to resolve a token
used by hardcoded skeleton defaults.

This repeats the preset/template review's dead-asset problem. Select one authored source and compile
it with `.zui` component definitions into one typed, immutable `EditorLayoutDefinitionGeneration`.
Delete unused TOML/parser/hardcoded parallel authorities in the replacing milestone. A test that
only parses a dead asset is not product wiring evidence.

### P1: current tests prove examples, not scale or authority

The 25 focused tests and module tests cover parsing, tokens, breakpoints, DPI examples and selected
frame values. Missing gates include:

- bounded/finite/nonnegative/nonoverlapping/contained final geometry invariants across width,
  height, scale and active content combinations;
- parity between any temporary manual fallback and the runtime `.zui` frame generation;
- stable-generation call/allocation counts and changed-generation proportional work;
- proof that `shell_regions.toml` is consumed by product, or proof that it was deleted;
- real-window resize/drawer-drag latency, CPU, allocation, lock, RSS and package-power evidence.

Source-text tests that assert an implementation string exists do not close these gates.

## Complexity target

| operation | current structural cost | required target |
|---|---|---|
| stable shell frame | model/descriptor preparation + manual solve; late bridge reuse only | O(1) generation comparison, zero layout/model/index/allocation work |
| shell/root-size change | manual solve + root runtime pass + workbench full runtime pass + floating check | one runtime layout transaction; O(affected widget tree), one frame publication |
| drawer extent/mode change | business mutation followed by manual and bridge width policies | one typed property delta; desired-size propagation plus affected layout owners only |
| descriptor/layout generation change | repeated descriptor indexing and region scans per shell recompute | compile/index once per definition generation; consumers borrow typed handles |
| paint-only/input-only update | may be promoted by root dirties/full bridge recompute | layout solve count 0; consume current immutable frame/index generation |

No claim that Unreal docking itself is constant-time is made. The target is proportional work and a
single cached layout authority, not a guessed universal asymptotic bound.

## Unreal primary-source evidence

- `SWidget.cpp:674-713` skips prepass on the fast update path unless `bNeedsPrepass`; the same block
  carries explicit cycle, memory and asset trace scopes. `973-981` caches computed desired size.
- `SWidget.cpp:1313-1367` converts prepass and child-order changes into typed layout reasons and
  marks the exact fast-path proxy dirty instead of unconditionally invalidating every root.
- `SlateInvalidationRoot.cpp:179-329` owns one persistent widget list and separate pre-update,
  prepass and post-update queues. Root child-order/layout invalidation explicitly selects the slow
  path; otherwise dirty proxies are inserted uniquely into typed queues.
- `SlateInvalidationRoot.cpp:356-424` performs slow paint only when fast update is disallowed or the
  root requires it; otherwise it consumes `PaintFastPath`.
- `SInvalidationPanel.cpp:190-234` recaches when layer, geometry, clip, clipping state or color
  prerequisites change. `408-445` descends the child prepass loop only when a slow path is needed.
- `SBoxPanel.cpp:116-203` arranges locally and computes panel desired size from cached child desired
  sizes. It does not introduce an editor-specific parallel pixel geometry authority.

These sources establish the direction: one widget-tree layout owner, cached desired size, typed
invalidation queues, explicit slow-path admission and local panel arrangement. Zircon runtime
already has incremental layout, dirty-domain reports and surface generations; the editor hard cut
must use them rather than add another scheduler.

## Required hard cut

1. Make the compiled `.zui` runtime `UiSurface` the sole normal-product geometry authority for shell,
   drawers, splitters, document viewport, popup anchors and floating-window source frames.
2. Keep `WorkbenchLayout` and editor model as business state. Publish typed property/layout deltas;
   editor code must not independently calculate product pixel frames.
3. Publish one immutable, generation-stamped runtime layout frame consumed by paint, hit test,
   pointer/drag routing, native-window projection and accessibility. No consumer may read a
   tree-local or manual geometry generation from the same transaction.
4. Put a generation gate before chrome/model/descriptor/token preparation. Stable generations do
   zero work; coalesce multiple dirty inputs into one layout transaction per presented frame.
5. Replace full `compute_layout` bridge refreshes with runtime `rebuild_dirty`/incremental layout
   after typed property mutation. Delete unconditional root dirtying from ordinary drawer/state
   projection.
6. Compile descriptor IDs, region bindings, token slots, control-node lookup and authored defaults
   once per `EditorLayoutDefinitionGeneration`; share borrowed/dense handles with every surface.
7. Remove the manual solver from normal execution. If temporarily retained, name it fallback/oracle,
   run it only on an explicit reason, enforce containment invariants and parity-test it against the
   same definition generation. Delete it in the replacing milestone; no compatibility facade.
8. Either compile the shell-regions/CSS declaration source into that definition generation or
   delete the dead asset/parser and hardcoded duplicate. One source, one compiler, one generation.

## Instrumentation first

M0 must add scenario-exported counters, not only free-form profile scopes:

| counter | purpose |
|---|---|
| shell pre-solve generation hit/miss | prove the guard runs before preparation |
| manual shell solve count/time | prove normal product reaches zero after hard cut |
| descriptor rows indexed/scanned | expose repeated definition work |
| token/transient entries converted | expose logical/physical map churn |
| root template full/incremental layout count | attribute first runtime surface work |
| workbench template full/incremental layout count | attribute mounted runtime work |
| floating source layout/reuse count | separate floating source behavior |
| root layout-dirty mark count | prove ordinary projection does not invalidate all roots |
| layout visited/skipped/geometry-changed nodes | verify proportional runtime work |
| shell layout temporary allocations/bytes | prove container removal, not just lower CPU noise |
| immutable layout-frame publications | require at most one publication per window/generation |

Capture `startup`, stable idle, hover, click, drawer resize, window resize, page switch and plugin
view add/remove/reload at 1/16/1k views and 1/16/1k windows where applicable. The current UI profile
scripts already cover several interaction scenarios; their exported hotspot schema must include the
new counters before baselines are accepted.

## Milestones

| milestone | deliverable | dependency |
|---|---|---|
| M0 | Attribution counters, allocator scopes and red 640px containment/parity tests. | current source recheck |
| M1 | One compiled editor layout definition generation; dead parallel declarations removed. | layout-preset M1 + Runtime UI76 |
| M2 | Runtime surface is sole shell geometry owner; immutable frame generation feeds every consumer. | EditorUI08 + Optimize01 |
| M3 | Pre-solve generation gate, coalesced typed deltas and incremental runtime layout; stable work zero. | M0-M2 |
| M4 | Manual solver/facades deleted; fallback reason count zero in product acceptance. | M2/M3 parity period |
| M5 | Managed Windows Cargo/F4 plus WPR/ETW allocation, lock, latency, RSS and package-power matrix. | M0-M4 |

## Acceptance matrix

| gate | matrix | required result |
|---|---|---|
| geometry correctness | width/height `0/1/34/420/640/900/1260/4k`, scale `1/1.25/1.5/2/3`, all drawer/content combinations | all frames finite and nonnegative; required frames contained; no invalid overlap; one canonical generation |
| stable generation | idle/hover/paint-only repeated `1/1k/1M` | manual/root/workbench/floating layout, root dirties, descriptor scans, token conversion, temporary layout bytes and publication all `=0` |
| changed generation | drawer/window/page/plugin changes repeated `1/1k` | one transaction/publication per frame; runtime visited nodes proportional to affected owners; full pass has typed reason |
| authority | paint/hit/drag/native/accessibility frame IDs | identical layout generation; manual product geometry owner/caller count `=0`; fallback count `=0` |
| definitions | startup plus plugin add/remove/reload `1/1k` | one compiled owner; stable compile/build/alloc `=0`; dead TOML/CSS/hardcoded parallel authorities `=0` |
| product | F4 launch, repeated resize/drawer drag/page switch, multiwindow, plugin lifecycle; 31 measured runs | WPR/ETW CPU, allocation, lock wait/hold, input-to-pixel p50/p95/p99, RSS and package power reported on identical hardware/config; artifacts only on D/E/F |

RenderDoc is required only if the hard cut changes visible GPU UI output or batch composition. It
can establish pixel/draw parity, but cannot prove CPU layout authority, allocation, lock behavior or
power by itself.

## Static gates executed

- Read 44/44 autolayout Rust files and the four focused files in full; reproduced current counts and
  both fingerprints above on 2026-08-22.
- `rustfmt --edition 2021 --check` passed for all 44 owned Rust files plus the four focused contract
  files (`48/48`).
- `tools/check_conventions.py --only docs --json` reports zero violations for both owned records.
  The repository-wide baseline remains `726` violations in `250` documents and is not attributed to
  this module.
- `tools/audit_plan_output_records.py --self-test` passed. The full audit still reports two
  cross-owner child-record-limit violations in Optimize01 and EditorUI08; the protected routing
  record asks those owners to absorb this finding without rewriting their plans here.
- The managed Windows Cargo dry run accepted
  `cargo test -p zircon_editor --locked --test integration_contracts workbench_autolayout` with
  `F:\cargo-targets\verify`. The real run was rejected before Cargo start by shared-pool job
  `7752addc38c549f7807ffcf4003bf5e3`; concurrent `cargo`/`rustc` processes confirm that job is still
  compiling. Therefore executed test count remains zero and the dynamic gate remains pending.
- Traced both full and window-metrics shell snapshot paths through manual geometry, late reuse,
  root/workbench template bridges and floating-window projection.
- Confirmed the workbench bridge marks all roots layout-dirty and calls the unconditional
  `UiSurface::compute_layout` path; confirmed runtime also exposes incremental `rebuild_dirty`.
- Reproduced the 640px minimum-deficit overflow from current constants and code; confirmed focused
  tests omit final containment/sum invariants.
- Confirmed the shell-regions asset is test-only and CSS declaration parsing has no production
  caller; confirmed the product uses hardcoded skeleton defaults cached by token Arc identity.
- Read the cited Unreal prepass, desired-size cache, invalidation root, invalidation panel and box
  panel implementations directly under `dev/UnrealEngine`.
- Confirmed existing UI counters cannot attribute manual/full layout authorities or pre-solve
  reuse. No fabricated latency, allocation or power values are recorded.

## Completion rule

The module remains pending until M0-M5 pass on one current-source fingerprint. A local vector/map
cleanup, a patched 640px manual solver, a late geometry equality hit, source-text tests or RenderDoc
alone do not qualify. No milestone commit or WeCom completion message is permitted before the
single-authority hard cut and quantified current-product evidence are accepted.
