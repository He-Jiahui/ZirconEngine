---
title: Editor template-node borrowed paint, damage and hover performance review
date: 2026-08-22
module: zircon_editor retained-host template_node_pipeline and template_nodes
priority: MVP-P0 shared editor retained-node paint path
status: source_reviewed_m1_applied_static_pass_dynamic_pending
reference_engine: Unreal Engine Slate invalidation widget proxy, cached element list and virtualized list view
---

# Goal

Make source/view generations and granular invalidation authoritative for retained template-node paint.
Stable visible nodes must be borrowed, not cloned into a second model on every frame. Hover, transform,
clip and damage changes must rebuild only the affected node or retained draw range. The final target is
a generation-qualified retained display list, not a faster full-frame reconstruction loop.

## Reviewed source

- Rust files: 22/22
- lines: 1,113
- bytes: 38,983
- joined path-and-raw-source-bytes SHA256:
  `926cc2415b2fca3963f5c04d60ac81127aceec89fa5bb7e17575a4ddc958f56e`
- owning commit at review: `a922089697e41e07fa29e3e42a5e4c9afc1ae31b`

| Module/folder | Files | Lines |
| --- | ---: | ---: |
| `paint_template_nodes/template_node_pipeline.rs` + `template_nodes.rs` | 2 | 96 |
| `paint_template_nodes/template_node_pipeline/**` | 5 | 249 |
| `paint_template_nodes/template_node_pipeline_tests/**` | 7 | 456 |
| `paint_template_nodes/template_nodes/**` | 8 | 312 |

## Correct foundations to retain

1. Borrowed geometry is rejected before the current ownership conversion, so fully clipped rows do
   not pay the node clone.
2. Optional transforms are explicit and can reject a node; transformed geometry is culled again
   after the transform.
3. Damage-row indices restrict traversal before node lookup, and duplicate/out-of-range rows are
   ignored without panicking.
4. Hover application mutates an owned temporary and leaves source projection data immutable.
5. `TemplateNodeCloneCount` and filtered/visited row counters already expose the first M1 effect.

## Structural findings

### P0: stable visible nodes are cloned and culled twice

The normal no-transform path first intersects borrowed `source_node` with `effective_clip`, then
unconditionally clones the full `TemplatePaneNodeData` and clip, builds an owned tuple, and repeats
the same intersection test. The clone can include labels, option/menu rows and nested node payloads.
For `V` visible stable nodes this is `V` model clones, `V` clip clones and `2V` equivalent geometry
tests before command generation.

Only one control can match the current hover id. M1 now keeps the stable non-matching path borrowed,
clones only a hover-targeted node, and preserves ownership for the transform path. Static stable-frame
counts are node clones `V -> 0`, clip clones `V -> 0`, and equivalent culls `2V -> V`; with one
matching hover, node clones are bounded by one.

### P0: damage still rebuilds commands instead of patching a retained draw artifact

Damage indices reduce node traversal, but every call starts from a mutable command Vec and runs the
complete specialized/fallback dispatcher for selected rows. There is no node-owned retained command
range keyed by source, geometry, clip, style and interaction generations. A small damage set therefore
recreates commands and payload strings rather than patching stable ranges.

M2 must retain per-node command ranges and update only dirty ranges. Acceptance is based on rebuilt
nodes/commands/bytes proportional to damage, not merely on a smaller input row list.

### P0: hover overlays rebuild whole option/menu row collections

Once hover targets a node, option and menu helpers map the full row model to replace one row state.
This is O(row count) cloning for a one-row interaction change. The separately reviewed typed
selection-option generation plan owns the source/window contract; M3 integrates that contract here as
a compact interaction overlay or addressed row patch instead of another complete node projection.

### P0: transforms consume and return the full node model

`TemplateNodeTransform` consumes `TemplatePaneNodeData` and returns another full node plus clip. That
forces ownership even where a transform changes only geometry. M4 must separate compact transform/
clip instance data from immutable node content and retain transformed geometry by transform generation.

### P1: specialized dispatch classifies every node on every rebuild

The dispatcher walks primary and secondary role gates before fallback for every visited node. This is
correct but repeats type/identity routing that can be compiled when a node generation is created. M5
adds a painter kind or prepared callback/table index to the retained node artifact after command-range
ownership exists; it must not introduce dynamic allocation or duplicate presentation authority.

### P1: whole-list ordering remains downstream work

Node command creation appends to a shared Vec and the downstream command path validates/sorts order.
The previously recorded render-command plan owns canonical retained batch order. This module must
publish stable node ranges and ordering keys to that owner rather than add a local second sorter.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/WidgetProxy.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Rendering/DrawElements.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Views/SListView.h`

`FWidgetProxy::Update` selects repaint only from explicit update flags such as `NeedsRepaint` and
`NeedsVolatilePaint`; other update domains do not automatically reconstruct the widget's paint data.
`FSlateWindowElementList` manages widget cache handles, cached elements, typed draw storage and a
separate render-batch merge stage. `SListView` owns an item source and generates visible rows instead
of recreating every logical item during paint.

The transferable constraints are explicit dirty domains, retained widget/node draw ownership,
visible-row materialization and compact per-frame instance state. Zircon should not copy Unreal
pointer lifetimes or assume Unreal timing numbers; current-source Zircon captures remain mandatory.

## Target architecture

1. Projection publishes immutable node generations plus stable ids and a compiled painter kind.
2. Host retains per-node prepared command ranges keyed by source/style/geometry/clip/resource
   generations and patches only invalidated ranges.
3. Hover/selection/focus are compact overlays addressed by control and row id; one interaction change
   cannot clone an unrelated node or full row collection.
4. Transform and clip are compact instance data with their own generations; unchanged node content is
   borrowed/shared through transform and command stages.
5. The downstream prepared render list consumes canonical node ranges and performs one batch merge;
   no duplicate full-list ordering or presentation model survives.
6. Profiling distinguishes visited, culled, cloned, rebuilt and reused nodes/commands/bytes so a cache
   status cannot conceal actual reconstruction.

## Instrumentation and acceptance

Matrix: nodes `0/1/1k/10k/100k`, damage `0/1/1%/100%`, hover `none/stable/move`, transform
`none/stable/change`, clip `full/10%/empty`, payload `text/options/menu/mixed`, one/eight panes.

| Evidence | Acceptance |
| --- | --- |
| node/clip clones and cloned bytes | zero for stable no-transform nodes; at most one hover node |
| borrowed/pre-cull/post-transform-cull counts | every rejection attributable; no duplicate stable cull |
| rebuilt/reused node command ranges and bytes | rebuild proportional to dirty nodes/ranges |
| hover row/node copies | O(changed rows), zero for non-target nodes |
| transform/content rebuilds | unchanged content not cloned; geometry updates by generation |
| dispatch/classification count | once per changed node generation after M5 |
| CPU/allocation/RSS/latency/context switches/power | same executable/workload before and after |
| RenderDoc draw/batch/GPU and pixel parity | accepted current backend build |

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Add cloned-byte, retained-range rebuild/reuse, hover-row and dispatch counters; capture matrix. | attributable baseline |
| M1 | Borrow stable untransformed nodes and clone only matching hover/transform nodes. | stable clones `V -> 0`, culls `2V -> V` |
| M2 | Retain generation-qualified per-node command ranges and patch damage. | rebuild proportional to dirty ranges |
| M3 | Replace whole option/menu hover projection with addressed interaction patches. | O(changed rows) hover work |
| M4 | Split immutable node content from transform/clip instance state. | no content clone for geometry-only update |
| M5 | Compile painter classification and converge canonical downstream order. | no repeated role-chain dispatch for reused nodes |
| M6 | Run managed scale/input/WPR/power and RenderDoc/pixel parity matrix. | quantified accepted milestone |

## M1 implementation result

The no-transform collector now rejects borrowed geometry once and sends a stable non-hovered source
node directly to command generation. A shared hover-target predicate gates the only ownership
conversion on that path. A matching hover clones and mutates one node while leaving source projection
data immutable. The transform route retains its owned node and clip plus post-transform cull.

For `V` visible nodes on a stable no-transform frame:

| Static collection work | Before | After | Change |
| --- | ---: | ---: | ---: |
| `TemplatePaneNodeData` clones | `V` | `0` | -100% |
| `FrameRect` clip clones | `V` | `0` | -100% |
| equivalent source-geometry culls | `2V` | `V` | -50% |
| non-target hover node clones | `V` | `0` | -100% |
| one matching-hover node clones | `V` | `1` | bounded by target count |

These are source-path operation counts, not elapsed-time claims. M0 must still measure cloned bytes,
allocator traffic and latency on the workload matrix.

Post-M1 direct owner scope:

- Rust files: 22/22
- lines: 1,156
- bytes: 40,552
- joined path-and-raw-source-bytes SHA256:
  `409febacdc35958901e82af2f706d07cbf8852570af1e7b3733ea07768f40bf0`
- unchanged direct owner files: 19 retain their pre-M1 fingerprints inside the joined hash above

| Changed direct owner file | Lines | Bytes | SHA256 |
| --- | ---: | ---: | --- |
| `template_node_pipeline/draw.rs` | 150 | 5,986 | `3c351ce7b3bf07cb961d282af4e352da1a90fa2fc3a61b25293e81ef5783dfc5` |
| `template_node_pipeline/hover.rs` | 83 | 2,788 | `0809f22f07d929b7bc1fefebb3ae51ff87da6406fa82664f1fcce2473706d235` |
| `template_node_pipeline_tests/transform.rs` | 155 | 4,905 | `0625b9eb85cd345961570e2915a6648ad0d29833916fb00c540e9e640dd69209` |

Focused static contract:
`tools/tests/test_editor_template_node_borrowed_paint_performance_contract.py`, 55 lines, 2,255
bytes, SHA256
`79cb2f64f47ba8c8a4d59685d2164495ba1b9ce864059350c14ad9e3117c6562`.

## Validation state

- Full owner review: passed, 22/22 Rust files.
- Relevant Unreal sources above: read and mapped to explicit dirty/retained/visible-row constraints.
- M1 focused contract: RED 3/3 before the change, GREEN 3/3 after the change.
- Existing hover performance contract updated to preserve its semantic early-return check: GREEN 2/2.
- Current owned editor performance-contract set: GREEN 59/59.
- Broad editor performance-contract discovery: 86/91 passed; the five failures are the pre-existing
  two missing test-source files, missing UI asset palette slot helper and two `.roots.clone()` asset
  dirty-mark paths. None is owned by this module.
- `rustfmt --check` for the three changed Rust files and scoped `git diff --check`: passed.
- Existing Rust transform/parity/source-immutability tests were updated or retained, but are not
  claimed passing until managed Cargo is executable.
- M0 and M2-M6 remain pending; no elapsed-time, GPU or power claim is made from static counts.
- Managed Cargo remains unavailable because Session
  `validate-matrix:019ffe1c-46d5-7933-97cb-65996b76f552` is terminal `archived` and rejects Cargo
  launch with `cargo_session_not_executable`.
- WPR and RenderDoc remain pending a launchable current-source editor. RenderDoc cannot validate
  model cloning, CPU allocation or interaction projection costs.

The module remains in `pending.md` until M0-M6 pass on one source/executable/workload fingerprint.
