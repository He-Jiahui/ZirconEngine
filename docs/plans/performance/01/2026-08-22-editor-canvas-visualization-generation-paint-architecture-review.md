---
title: Editor canvas visualization generation and paint performance review
date: 2026-08-22
module: zircon_editor sample_grid timeline_strip weight_heatmap retained projection and paint
priority: MVP-P0 editor animation and visualization surfaces
status: source_reviewed_m1_applied_static_pass_dynamic_pending
reference_engine: Unreal Engine Sequencer and CurveEditor visible-range draw cache
---

# Goal

Replace attribute-driven whole-generation rebuilds and per-primitive paint-command expansion with one
generation-qualified retained visualization artifact. Static axes, ticks, labels and fields must be
reused until their actual source/view generation changes; high-frequency playhead, selection and
marker updates must patch bounded dynamic state. Work must stay proportional to visible pixels/items,
not total source size or canvas extent.

## Reviewed source

- Rust files: 56/56
- lines: 5,172
- bytes: 154,891
- joined raw source-bytes SHA256:
  `81945b259ddf1ad9af89364b657eb439bc90d0f5d2d9cf27ba308f6778c1ad4d`
- owning commit at review: `a922089697e41e07fa29e3e42a5e4c9afc1ae31b`

| Module/folder | Files | Lines | Bytes |
| --- | ---: | ---: | ---: |
| `ui/{sample_grid,timeline_strip,weight_heatmap}/**` | 9 | 1,561 | 42,553 |
| `pane_component_projection/{roots,template_node_data,tests}` | 9 | 611 | 20,739 |
| `host_contract/data/template_nodes/*` | 3 | 18 | 528 |
| `paint_template_nodes/template_sample_grid{,_tests}/**` | 14 | 1,410 | 44,200 |
| `paint_template_nodes/template_timeline_strip{,_tests}/**` | 11 | 887 | 26,330 |
| `paint_template_nodes/template_weight_heatmap{,_tests}/**` | 10 | 685 | 20,541 |

Supporting consumers traced: generic template-node assembly, typed host data, paint command creation,
runtime text measurement, clip/geometry helpers and CPU raster paint tests.

## Correct foundations to retain

1. All three component projections gate on the canvas variant before parsing their large arrays.
2. Generation objects are shared by `Arc`; static and dynamic hashes distinguish selection/playhead
   state from stable field/tick content.
3. Timeline tick generation and heatmap cell generation have explicit 4,096-element ceilings.
4. Timeline/heatmap cache misses compute candidates outside the global mutex and recheck on insert,
   so expensive generation is not performed while holding the cache lock.
5. Collapsed sample-grid and timeline frames return before painter work; heatmap field/marker helpers
   also reject non-drawable geometry once reached.

These foundations are useful, but they cache intermediate generation data rather than the retained
draw artifact. They do not make the end-to-end projection/paint path incremental.

## Structural findings

### P0: interaction changes rebuild and rehash whole visualization generations

Each specialized projection reparses owned strings/arrays and creates a new generation. A timeline
scrub changes `current_time` but still reconstructs and hashes all keys, including selection flags.
Sample and heatmap selection changes similarly traverse the entire point/source input. Static/dynamic
hash separation is therefore downstream of O(N) source reconstruction rather than an invalidation
boundary.

The target is a compiled immutable source generation plus addressed dynamic patches. A playhead move
updates only playhead/progress/footer state; point/key/source selection updates addressed identities;
axis/tick/field artifacts survive until their source or view generation changes.

### P0: cached generation still expands into a fresh draw list

Sample-grid emits 18 one-pixel-high quad commands for every point (11 outer diamond scans plus seven
inner scans), plus two commands for a selected label. Its dashed grid work is
`O(x_ticks * plot_height / 7 + y_ticks * plot_width / 7)` before text.

Timeline emits `2r + 1` outer quad commands plus three center commands per key; selection increases
the outer radius. Every paint still measures and clones every tick label even when cached static tick
content is reused. Heatmap emits one quad per field cell plus legend/source commands. At the current
cell ceiling, field paint alone can create 4,096 commands.

The target paint contract needs batched/instanced point, key, line and heat-cell primitives plus a
retained static draw list. Adding only another generation cache would preserve the dominant command
creation, clip cloning and submission work.

### P0: sample-grid source and paint work have no visible/item budget

Sample points are unbounded. Generation and hashing are O(P), while point paint is at least 18P
commands. Tick arrays are also unbounded and dashed-line work grows with both tick count and physical
canvas extent. The current painter clamps coordinates into the plot, so out-of-range inputs can also
pile up on an edge instead of being rejected or queried by a visible range.

M2 must establish a typed visible-range query and pixel/item budget before choosing decimation. It
must preserve selected/interactive identities and visual extrema; an arbitrary stride or local
parallel loop is not an acceptable substitute.

### P0: heatmap miss is O(cells * sources) on the UI caller

`WeightHeatmapStaticField::new` evaluates every source at every cell and calls `exp` for every pair,
then hashes every resulting intensity. Cells are bounded to 4,096, but source count is not. The cache
has capacity 16 and a miss can be duplicated by concurrent callers because candidate construction is
deliberately outside the lock.

The current field is a maximum of equally shaped weighted Gaussians, not a sum. M3 must first profile
the real source/cell matrix, then compare exact direct evaluation against an exact weighted-distance
query/index and GPU/worker preparation. Approximate cutoffs or raster convolution change semantics and
require an explicit visual/error contract; worker results must be generation-qualified and only
applied on the UI thread.

### P1: global cache bookkeeping is bounded but not an ownership solution

Timeline and heatmap use process-global mutex/BTreeMap/VecDeque caches of capacity 16. A hit takes the
mutex and performs an O(16) recency retain. A miss locks twice and permits duplicate candidate work.
This is bounded bookkeeping, but it makes unrelated editor views contend and does not express view
lifetime, invalidation or memory ownership.

M2/M3 should move retained artifacts to the owning visualization/view generation. Cross-view sharing
is justified only for immutable source artifacts with measured reuse; it must not become a second
invalidation authority.

### P1: collapsed heatmap enters text work before its drawable gate

Unlike sample-grid and timeline, the heatmap root measures high/low labels to construct geometry
before checking the frame. It then allocates two owned label strings and attempts two more text
measurements in the legend painter. For a collapsed heatmap with two non-empty labels, the static call
path contains four text measurements, two string allocations and three downstream painter calls.

M1 will add the same finite-positive extent gate used by the other canvas roots before label
measurement, cache access or painter dispatch. This is behavior-preserving and independently testable.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Editor/Sequencer/Private/SequencerTimeSliderController.cpp`
- `dev/UnrealEngine/Engine/Source/Editor/SequencerWidgets/Private/SSequencerTimeSlider.cpp`
- `dev/UnrealEngine/Engine/Source/Editor/CurveEditor/Private/SCurveEditorView.cpp`
- `dev/UnrealEngine/Engine/Source/Editor/CurveEditor/Private/CurveEditorCurveDrawParamsCache.cpp`

Sequencer derives its major/minor grid metrics from the current geometry and view range, starts at the
first major line in that range and uses a minimum pixel-spacing policy. CurveEditor requests only keys
inside `InputMin..InputMax`. Its draw-param cache invalidates from active-curve and selection serials,
geometry/view-bound changes, or individual curve changed flags; the retainer is refreshed only after
affected draw data changes. Newer curve models can prepare draw data through a cache pool.

The transferable invariants are visible-range queries, pixel-derived budgets, explicit source/view/
selection generations, per-owner retained draw data and affected-item invalidation. Zircon should not
copy Unreal pointer/template types or assume Unreal timings; current-source measurements remain the
acceptance authority.

## Target architecture

1. EditorUI06 compiles typed immutable sample/timeline/heatmap source generations once, with stable
   point/key/source identity and schema diagnostics.
2. EditorUI01 publishes ordered playhead/selection/drag patches; latest-wins pointer motion is
   coalesced without dropping click/drag edges.
3. EditorUI08 owns one visualization view generation keyed by source, view bounds, geometry, DPI,
   theme/font and visible budget. Static and dynamic retained artifacts have separate invalidation.
4. Runtime UI09 exposes batched line/diamond/heat-cell primitives or an equivalent display-list path
   so command count is O(batches + visible labels), not O(pixels + points + cells).
5. Heavy heatmap preparation runs through the runtime task budget or GPU only after profiling chooses
   the algorithm; results carry generation/cancellation identity and stale results are discarded.
6. Raw attribute reconstruction and process-global presentation caches are hard-cut after all three
   visualizations consume the canonical typed generations.

## Instrumentation and acceptance

Matrix: canvas `0x0/64x64/640x360/4K`, sample points `0/1/1k/100k`, ticks `0/8/256/4096`, timeline
keys `0/1/1k/100k`, scrub `60/240/1000 Hz`, heat sources `0/1/64/4k/65k`, cells
`16/256/4096`, stable/1%/100% source change, one/eight simultaneous views.

| Evidence | Acceptance |
| --- | --- |
| parse/hash/source-generation counters | once per changed source generation |
| static/dynamic artifact rebuilds | playhead/selection does not rebuild static artifacts |
| visible queried items and command/batch counts | bounded by visible/pixel budget, not total N |
| heat pair evaluations/exp calls/cache hits/duplicate misses | attributable and bounded by selected algorithm |
| UI-thread CPU, allocation, lock wait, input latency | same current-source workload before/after |
| render pass/draw count, GPU time, overdraw | RenderDoc/current backend after launchable build |
| RSS, context switches, package power/energy | WPR on the same executable/hardware/power profile |
| pixel/text/hit/selection parity | golden plus interaction/accessibility behavior passes |

No comparison to Unreal elapsed time or power is valid until Zircon and the reference workload have
equivalent visible content, resolution, backend, build mode, hardware and capture windows.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add projection/hash/artifact/command/cache/lock/field-evaluation counters and capture matrix. | attributable baseline |
| M1 | Gate collapsed heatmaps before label measurement and painter dispatch. | focused RED-to-GREEN contract |
| M2 | Compile typed source/view generations, visible queries and addressed dynamic patches. | no whole-source rebuild on interaction |
| M3 | Move heatmap preparation behind generation-qualified task/GPU ownership after algorithm benchmark. | bounded UI-thread work and stale-result rejection |
| M4 | Add retained batched line/key/point/cell draw artifacts and granular invalidation. | command count proportional to batches/visible labels |
| M5 | Hard-cut raw rebuild and process-global presentation-cache authorities. | one source/view invalidation owner |
| M6 | Run managed scale/input/WPR/power and RenderDoc/pixel parity matrix. | quantified accepted milestone |

## M1 implementation result

The weight-heatmap root now rejects non-finite, zero and negative frame extents immediately after
component identity is established. Collapsed heatmaps never borrow the generation, measure legend
labels, allocate owned label strings, enter the static field cache or dispatch field/marker/text
painters. Drawable frames retain the existing generation, geometry and paint behavior.

Per collapsed heatmap with two non-empty legend labels:

| Static call-path work | Before | After | Change |
| --- | ---: | ---: | ---: |
| runtime text measurements | 4 | 0 | -100% |
| owned legend label strings | 2 | 0 | -100% |
| downstream painter dispatches | 3 | 0 | -100% |
| field cache accesses | 0 | 0 | unchanged; existing helper already gated |

Post-M1 owner scope:

- Rust files: 56/56
- lines: 5,252
- bytes: 156,981
- joined raw source-bytes SHA256:
  `080788dab8151b36c530bc0a5aefd45c675781b672569bb25143b70e7c88d2bd`
- unchanged owner files: 52 retain their pre-M1 content within the joined fingerprint above

| Changed file | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `template_weight_heatmap.rs` | 48 | 1,374 | `47716a89bcc2b0c285a25040b2e5e05724981c15fa96bcd2d28f42d728ed7dbb` |
| `template_weight_heatmap/geometry.rs` | 209 | 6,436 | `053fc0841522515b4c9c042c5cf5abae470d1675a10e296ed8f821eb1af5ba4e` |
| `template_weight_heatmap_tests/paint.rs` | 60 | 1,722 | `976773005ca737b2698502b2c50625030ae79e402a6cfa07d83c59aae56dd47f` |
| `template_weight_heatmap_tests/support.rs` | 53 | 1,817 | `57d4cf5241cfdf52e4a4379f6dc7ecf3e5a35991043b557171d5d4ed554df3a9` |

Focused contract:
`tools/tests/test_editor_canvas_visualization_projection_performance_contract.py`, 50 lines, 1,831
bytes, SHA256
`71fa923ae75f7dc50c29c357ae157f829234f7ddd4ca2668df07eda9127e69c2`.

## Validation state

- Full owner source review: passed, 56/56 Rust files.
- Paint/data/projection consumers and the Unreal sources above: read.
- M1 focused contract: RED 2/2 before the change, GREEN 2/2 after the change.
- Current owned performance-contract set: GREEN 52/52.
- `rustfmt --check` for the four changed Rust files and scoped `git diff --check`: passed.
- Rust behavior regressions cover extent classification and zero-command collapsed paint, but are not
  claimed passing until managed Cargo is executable.
- M0 and M2-M6 remain pending; all cost counts above are exact static call/loop/command counts, not
  elapsed-time or power measurements.
- Managed Cargo remains unavailable because Session
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is terminal `archived` and rejects Cargo
  launch with `cargo_session_not_executable`.
- WPR requires a current-source launchable editor. RenderDoc is reserved for M4/M6 GPU/draw/pixel
  validation and cannot validate the CPU projection/hash or text-allocation findings.

The module remains in `pending.md` until M0-M6 pass on one source/executable/workload fingerprint.
