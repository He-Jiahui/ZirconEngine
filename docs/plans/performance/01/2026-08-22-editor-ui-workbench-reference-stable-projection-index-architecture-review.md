---
title: Editor workbench reference stable projection index performance review
date: 2026-08-22
module: zircon_editor/src/ui/workbench/reference
priority: MVP-P1
status: source_reviewed_m1_static_complete_dynamic_pending
reference_engine: Unreal Engine Slate invalidation root and widget list
---

# Goal

Keep one stable workbench surface/projection identity across ordinary state and geometry changes.
Incremental refresh must validate topology without cloning it, publish compact changed-row deltas and
reserve full projection rebuilds for explicit topology/index fallback. Remove the handwritten visual
reference tree from production compilation after its tests have a declarative fixture owner.

## Reviewed source

- folder: `zircon_editor/src/ui/workbench/reference`
- Rust files: 9/9
- lines: 2,066
- bytes: 69,860
- joined current-tree UTF-8 SHA256:
  `44a1608a5b56be05c91a98d46360dbcec6fc7c1ad0236e773c39727ecbde84e0`
- owning commit before review: `08094b9b9e17f6c80372e15c17b01204038b305b`
- related integration tests identified: 8 in `src/tests/workbench/reference_surface.rs`

| File | Lines | SHA256 |
| --- | ---: | --- |
| `builder/mod.rs` | 255 | `901269bbfbffb6966807232b28af5fc85ec9bc2bc98010833a8aaf60386a2384` |
| `builder/nodes.rs` | 420 | `b18c6cd1aa154934219b4016c5e0786b99ce5bb915ac91668b42a659fc443643` |
| `builder/panels.rs` | 499 | `43c7eaed918a6e927182f1d69a5dd5d61223109a1f8713748f92fb66ce195a87` |
| `ids.rs` | 47 | `19744b5bdf6b1b95774fd8de24fe35597edcc20b68014f95662c9c5b54133531` |
| `metrics.rs` | 60 | `dd157d81ee8562e4d1b40044f6a14e7d47d765879d7821f68503057fb3bc6b6d` |
| `mod.rs` | 18 | `d159f4dd025f947af22d60e3f554da556a4c4d30ec243282454b8989638eb166` |
| `surface.rs` | 31 | `8e3b71f8625836fc35985c6659bf6508741b741b4f446a75d89d5b7bfe86a901` |
| `template_surface.rs` | 694 | `0e28c9f0a41ca4f3890d853082ce744edb2d70516ea36b746b8be29c89457b1a` |
| `tokens.rs` | 42 | `f48ab930434a8054378eca78c0ab37aeed77dd1b50ab597bb40572c030491d9a` |

All nine files were read in full. The current tree already had an unrelated one-line
`component_event: None` update in `builder/nodes.rs`; it was preserved. The production call chain was
traced through the componentized workbench bridge, retained-host recompute/presentation patch,
host-contract row projection and incremental hit-index rebind. The existing retained-host
invalidation architecture audit and the first four template-surface regression tests were read.

## Result

### Correct foundations to retain

`EditorWorkbenchTemplateSurface` owns stable control-to-node and node-to-host indices, a topology
snapshot, source metadata index and explicit full-refresh fallback. State refresh separates semantic
and geometry node sets. An empty workset has a projection noop fast path; geometry-only resize avoids
semantic projection rebuild. Existing tests prove a one-node semantic patch, a focus/visibility
patch, more than 1,000 resize geometry patches and invalid-index fallback, each against a full
projection baseline.

This is materially closer to the required retained architecture than rebuilding the workbench tree.
No second projection cache is justified.

### P1: fast-path topology validation deep-clones the value it only compares

`can_patch_host_projection` iterates every changed node and constructs
`HostProjectionNodeIdentity::from_surface_node`. That construction clones node path, children vector,
component and optional control ID before comparing them to the committed topology snapshot. The
resize regression already proves the geometry workset exceeds 1,000 nodes, so one resize event can
perform more than 1,000 unnecessary topology object/string/vector materializations even though
geometry invalidation cannot itself change topology.

M1 now uses a borrowed field comparison against the committed identity. The incremental guard has
one borrowed matcher call and zero `from_surface_node` calls; full snapshot construction retains the
single owned call site. No invalidation semantics or fallback condition changed.

### P1: incremental payload crosses the boundary as full owned nodes

After the workset is patched in place, `pending_host_projection_patch_nodes` clones every changed
`RetainedUiHostNodeModel` into a new vector. The presentation layer builds another projection index,
converts each full model into an owned `TemplatePaneNodeData`, inserts it into a `BTreeMap` row patch,
collects changed rows into another vector and then incrementally rebinds the hit index. Resize makes
this pipeline proportional to more than 1,000 full node DTO clones/conversions.

The target is one generation-checked patch descriptor carrying stable row/node identity and the
minimal semantic and/or geometry payload. Presentation and hit-index consumers must read the same
delta; they must not receive a full cloned host node merely to update frame/clip/z.

### P2: a test-only reference tree remains a production API

`builder/**`, `ids.rs`, `surface.rs` and `tokens.rs` total 1,294 lines and are called only by
`src/tests/workbench/reference_surface.rs`. They hand-build a second workbench tree with hard-coded
paths, strings, TOML values, bindings, colors and geometry. It does not execute per frame, so it is
not a runtime hotspot; however, release compilation and the public module surface still carry a
parallel visual truth that can drift from the declarative workbench asset.

Move the baseline to a test fixture or generated declarative artifact, then delete the production
builder/export. `EditorWorkbenchReferenceMetrics` remains until the real host size contract has a
better owner.

## Unreal source basis

Direct source read:

- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/FastUpdate/SlateInvalidationRoot.h`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationRoot.cpp`
- `dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationWidgetList.cpp`

Unreal builds a persistent `FSlateInvalidationWidgetList` and assigns each widget a proxy index while
the list is constructed (`SlateInvalidationWidgetList.cpp:432-535`). Ordinary invalidation updates
the existing proxy reason and inserts that proxy into unique pre-update, prepass or post-update work
heaps (`SlateInvalidationRoot.cpp:299-337`). `ProcessInvalidation` consumes those indexed heaps and a
final update list; root layout or child-order invalidation explicitly requests the slow path
(`288-291`, `1281-1392`).

The transferable invariant is that topology identity is committed on list build and ordinary work
queues carry stable indices. The fast path does not deep-copy the widget's topology merely to prove
that the stable proxy still refers to it.

## Target architecture

1. Keep the current committed control/node/host indices and topology snapshot as the only projection
   identity authority.
2. Validate a changed node by borrowed comparison. Geometry-only work uses the stable node/index
   receipt directly; topology is rebuilt only for child-order/topology generation changes.
3. Replace `Vec<RetainedUiHostNodeModel>` patch export with a compact `WorkbenchProjectionDelta`:
   generation, stable host row, changed mask, semantic payload when needed, and frame/clip/z for
   geometry. Coalesce repeated changes by row before commit.
4. Apply the same delta to presentation rows, damage calculation and hit-index rebind. Record an
   explicit generation/index/topology fallback reason before any full rebuild.
5. Move the handwritten reference surface to test ownership and then remove it after declarative
   fixture parity. Do not retain a release-visible compatibility export.

Complexity target:

- one-node semantic update: O(1) indexed lookup plus payload work;
- K-node geometry update: O(K), zero topology/string/vector clones;
- patch memory: O(K) compact deltas, no full host-node DTO clone for geometry-only rows;
- unchanged refresh: zero projection or patch allocation;
- topology change: one explicit O(N) rebuild and snapshot publication.

## Instrumentation and acceptance

Add or extend counters before M2:

| Evidence | Target |
| --- | --- |
| changed semantic/geometry nodes | exact exercised K |
| topology identity materializations on incremental path | 0 |
| cloned host-node count/bytes | 0 for geometry-only patch |
| compact delta count/bytes | proportional to K |
| projection/hit-index full fallback by reason | 0 in valid warm scenarios |
| layout passes per event/frame | at most one per affected surface |

Matrix: single hover/focus/property edit; `24/240` resize events; sizes
`640x420/900x620/1280x720/1920x1080`; scale `1/1.25/1.5/2`; no-op, geometry-only,
semantic-only, mixed, topology change and deliberate stale-index fallback. Report median/p95/max,
K, allocations/bytes, patch bytes, full fallback reasons, input-to-damage, damage-to-submit, RSS and
package energy on one source/executable fingerprint.

Run the current UI profiler and WPR/ETW with targets/artifacts on D/E/F. RenderDoc is only a final
draw/pixel parity check for a launchable current-source GPU UI path; it cannot validate retained-host
CPU allocation removal.

## Milestones

| Milestone | Work | Gate |
| --- | --- | --- |
| M0 | Capture source-bound state/resize baseline and clone/allocation counts. | UI trace plus WPR artifacts |
| M1 | Borrow topology fields during incremental validation. | Existing equality/fallback tests |
| M2 | Publish and consume compact generation-checked projection deltas. | zero geometry full-node clones |
| M3 | Coalesce per-frame deltas and prove explicit fallback reasons. | scale and invalidation matrix |
| M4 | Move/delete the handwritten production reference builder. | declarative fixture parity and deletion contract |
| M5 | Run product interaction, WPR, power and pixel acceptance. | quantified before/after evidence |

## Validation state

- Full folder review: passed, 9/9 current files.
- Current behavior regressions identified: 8; focused first four cover incremental projection.
- Managed Cargo and current-source profile: pending while shared Cargo lanes are active.
- M1 source implementation: complete; targeted `rustfmt`, `git diff --check` and static call-site
  accounting pass. Managed behavior and performance acceptance remain pending.

The folder remains in `pending.md` until M0-M5 pass on one fingerprint. A source-only M1 must not be
recorded in `review.md` or used for a performance improvement claim.
