---
title: Animation State Machine Pose Capability Current-Source Algorithm Performance Review
date: 2026-08-24
scope:
  - zircon_plugins/animation/runtime/src/state_machine
  - zircon_plugins/animation/runtime/src/evaluation/pipeline
  - zircon_plugins/animation/runtime/src/gpu_skinning
  - zircon_plugins/animation/runtime/src/ik
  - zircon_plugins/animation/runtime/src/mask
status: m1_parameter_proxy_implemented_component_parameter_owner_hard_cut_red_dynamic_build_blocked
canonical_owners:
  - docs/plans/optimize/zircon_plugins/13-first-party-animation-source-runtime-editor-dist-catalog-skeleton-clip-pose-graph-state-machine-ik-skinning-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_editor/82-editor-animation-blend-space-axis-sample-triangulation-interpolation-filter-per-bone-additive-sync-runtime-evaluation-preview-product-integration-current-source-review.md
references:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimNode_StateMachine.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/BlendSpace.cpp
  - dev/UnrealEngine/Engine/Plugins/Runtime/AnimationBudgetAllocator/Source/AnimationBudgetAllocator/Private/AnimationBudgetAllocator.cpp
---

# Animation State Machine Pose Capability Current-Source Algorithm Performance Review

## 1. Coverage and product reachability

The initial state-machine/GPU/IK/mask audit scope is **50/50 Rust files**, **2,362 physical / 2,111 non-empty lines**, **75,920 bytes** and **1 inline test**. At repository revision `1538a67d526d4c8dff93aa96e189751c06f80ad6`, the captured worktree fingerprint is `f1f45e34bdf267a817b006a0fe991794e09b27dd7748384f9fbfd035a2ccc392`. Section 12 extends the implementation review into `evaluation/pipeline`; the historical inventory below is intentionally not recast as a current whole-pipeline count.

| Folder | Files | Static review result |
|---|---:|---|
| `state_machine` root and compiled/condition/transition | 25 | Dense slots and bounded nesting are positive; runtime DTOs/checkpoints still clone maps and strings. |
| `state_machine/blend_space` | 7 | 1D lookup is sorted; 2D construction and outside-hull sampling are structurally unfit for runtime scale. |
| `state_machine/layer` | 5 | Source machine is duplicated into a base artifact; runtime layers allocate/copy full poses. |
| `gpu_skinning` | 5 | Palette math exists, but no renderer/product caller was found. |
| `ik` | 4 | Small solver jobs remain; concurrent work removed the process-wide IK inbox/postprocess path. No scene pipeline caller remains. |
| `mask` | 4 | Compiler exists, but state-machine layers consume dense weights through a separate path. |

A repository-wide Rust caller search outside this plugin found no consumer for `AnimationGpuSkinningDecision`, `SkinningPalette`, `TwoBoneIkJob`, `LookAtJob`, `AvatarMaskCompiler` or `CompiledAvatarMask`. These are library exports, not qualified engine features.

## 2. Structural performance findings

### P0: BlendSpace2D compilation is at least O(N^4)

`blend_space_2d.rs` enumerates every point triple, checks every other point against the circumcircle, and then compares candidate triangles against the selected list for overlap. Duplicate-point validation is O(N^2). This is not a mature Delaunay implementation and its output depends on greedy candidate order.

Runtime sampling linearly scans all triangles. An outside-hull query reconstructs an ordered edge-count map from every triangle on every sample, then projects against all hull edges. Neither the hull nor triangle adjacency is stored; no previous-triangle hint survives in an animation instance.

### P0: exported GPU skinning, IK and mask capabilities are not product paths

GPU palette creation builds name maps and several full joint arrays, computes bind-world and pose-world matrices and inverts each bind matrix. The double buffer clones a whole palette on upload. None of this reaches the renderer, so its cost/quality cannot be compared with a rendered frame and its public availability is misleading.

The shared worktree correctly deletes the global IK command inbox and postprocess application, removing an unsafe second ownership path. The remaining solvers are now standalone math utilities. They need an explicit animation-node/compiler/scheduler integration before runtime capability can be claimed. Avatar-mask compilation similarly lacks a single compiler/runtime/editor artifact route.

### P1: state-machine evaluation rebuilds transactional state instead of owning one instance

Compiled state-machine evaluation allocates parameter reference collections and returns owned DTOs with cloned parameters, state names and references. Nested instance keys allocate lineage and owner strings. Runtime admission checkpoints clone active entity sets and nested transition/source maps so deferred events can roll back; transition pose/events/normalized-time paths repeatedly resolve graphs and resources.

Depth is bounded at eight and cycles are rejected, which is a good safety baseline. The remaining issue is ownership: one per-entity instance should update tentative dense state and publish a small commit receipt only after pose/event admission succeeds.

### P1: layer compilation and evaluation duplicate whole products

Layer compilation clones the complete source state machine into `base` and clears its layers. Runtime evaluation then allocates two `PoseBuffer` values per layer, copies every base and layer transform into them, blends, and rebuilds the named bone vector. Interrupted transitions retain full poses in `Arc`; checkpoint maps clone those handles and keys.

Layers must be part of one compiled program with shared state/graph artifacts, dense mask ranges and arena-owned intermediate poses. Full-pose snapshots should be admitted only when an interruption policy actually requires them and must have explicit byte/count limits.

## 3. Unreal source constraints

- `BlendSpace.cpp:2755-2908` resamples authoring data once, uses `FDelaunayTriangleGenerator`, and stores triangles or grid elements. `2994-3135` starts sampling from a cached triangle index and walks precomputed neighbor edges/perimeter instead of rebuilding a hull map per frame.
- `AnimNode_StateMachine.cpp:380-490` runs state-machine Update on any thread, uses baked state indices and explicitly bounds transitions per frame. `954-1031` evaluates indexed state pose links and caches poses within the pass.
- `AnimationBudgetAllocator.cpp:425-790` distributes measured work by significance, tick rate, interpolation and reduced-work policy. It smooths work-unit cost and budget pressure instead of evaluating every component identically.

Zircon should use a proven Delaunay implementation or independently validated equivalent, persist triangle adjacency/hull and cache the current triangle per instance. It should adopt baked state indices, reusable pose-pass storage and measured workload reduction, not copy Unreal's UObject types.

## 4. Dependency-ordered optimization plan

### M0: make capability truth explicit

Mark GPU skinning, IK postprocess and avatar-mask product integration unavailable until compiler, runtime scheduler, renderer/physics and editor consumers exist. Keep standalone math APIs internal or experimental. Preserve the removal of the process-wide IK inbox.

### M1: replace BlendSpace2D construction and query

Use a vetted Delaunay triangulation implementation with deterministic input ordering and explicit duplicate/collinear/point-count limits. Compile triangles, neighbor indices, convex-hull edges and optional grid acceleration into one immutable artifact. Store the previous triangle in the animation instance and walk adjacency; outside-hull queries use precompiled edges.

### M2: compile one indexed state/layer program

Lower states, transitions, conditions, nested machines, layers, masks, clip/event dependencies and interruption policy into dense immutable arrays. Share the base machine instead of cloning source. Use parameter slots and change masks; keep instance state in dense mutable storage with bounded transition count/depth.

### M3: replace checkpoint cloning with tentative commit

Evaluate into an instance-local tentative state and pose arena. Admit events and dependencies first; commit only the changed state slots, consumed triggers and generation receipt. Defer by retaining a compact pending receipt, not cloned global maps or named poses.

### M4: integrate pose modifiers and skinning through one schedule

Compile IK/look-at/mask nodes into the animation program, evaluate them in worker-owned dense pose buffers, and publish one versioned pose handle. Rendering consumes a prepared palette/buffer generation without name lookup or whole-palette clone; physics consumes explicit pre/post-physics poses under one ownership rule.

### M5: qualify algorithms and product reachability

Benchmark blend points `4/32/128/512`, query trajectories inside/outside hull, states/transitions `10/100/1k`, layers `0/4/16`, bones `32/128/256`, interruption/event pressure and visible/offscreen populations. Record compile/query/update/evaluate p50/p95/p99, scale slopes, allocations/bytes, cached-triangle steps, pose-arena misses, reduced work and output parity.

## 5. Acceptance gates

1. BlendSpace2D compile follows a validated Delaunay algorithm and its measured growth is consistent with the selected implementation, not O(N^4).
2. Steady sampling allocates zero hull maps and starts from a persisted triangle/grid hint.
3. Each state/layer instance owns dense mutable state; stable frames clone no parameter map, state name, lineage string or named pose.
4. IK, masks and GPU skinning are either reachable through one compiled product schedule or truthfully unavailable.
5. Renderer palette upload copies only dirty generations/ranges and has RenderDoc-visible draw/GPU evidence.
6. Budget/relevance tests preserve root motion, event, physics and transition correctness while bounding frame work.

## 6. Validation status

- Per-production-Rust-file static review: **50/50 complete** for the captured fingerprint.
- Outside-plugin reachability scan: complete; no production consumer was found for the exported GPU skinning, IK solver or compiled mask APIs.
- Shared IK deletion: reviewed and preserved; it removes a duplicate global inbox but does not create a replacement product path.
- Cargo/tests and dynamic benchmarks: pending because the managed Windows validation session is unavailable.
- RenderDoc: pending until a current-source executable and renderer-connected skinning path exist; `renderdoccmd.exe`/`qrenderdoc.exe` are not currently available.
- Protected ledgers, milestone commit and WeCom completion remain pending.

## 7. 2026-08-26 pre-implementation profile and compiler-owner recheck

The current-source review was repeated before changing the algorithm. The production path still has
two compiler authorities:

- `zircon_runtime/src/core/framework/animation/compiler` is a new 15-file / 3,249-line pure
  source compiler. It produces deterministic graph/state-machine IR, but no production consumer
  reads that IR and its 2D blend-space artifact contains samples only, not triangulation, adjacency,
  or hull data.
- `zircon_plugins/animation/runtime/src/state_machine` remains the executable compiler. Its cache
  recompiles `AnimationStateMachineAsset` directly, and `BlendSpace2D::compile` still performs the
  triple enumeration, all-point circumcircle scan, and greedy overlap pass described above.

Therefore changing only the core compiler's cubic non-collinearity check would optimize an
unintegrated path and leave the actual runtime cook unchanged. M1 must hard-cut production
consumers to one immutable compiler artifact before any local micro-optimization can count.

### RED profile

The production triangulation and overlap predicates were copied semantically into a standalone
release-mode Rust harness. `rustc -C opt-level=3` read the source from stdin; `TEMP`, `TMP`, the
executables, and all generated files were placed under
`F:/zircon-profiles/animation-blend-space-20260826`. This is a pre-implementation algorithm probe,
not a substitute for the blocked managed package benchmark.

| Input | Samples | Candidate triangles | Selected triangles | Time |
|---|---:|---:|---:|---:|
| deterministic pseudo-random | 16 | 25 | 25 | 47 us |
| deterministic pseudo-random | 32 | 53 | 53 | 234 us |
| deterministic pseudo-random | 64 | 118 | 116 | 1,675 us |
| deterministic pseudo-random | 128 | 248 | 243 | 11,434 us |
| co-circular direction samples | 16 | 31 | 9 | 35 us |
| co-circular direction samples | 32 | 21 | 5 | 240 us |
| co-circular direction samples | 64 | 2 | 2 | 2,485 us |
| co-circular direction samples | 96 | 0 | 0 | 9,900 us |
| co-circular direction samples | 128 | 0 | 0 | 27,199 us |

Harness SHA-256 values are
`a26ab5f16ddc9616bf1a9842654cea8a456b8fe39a38814159d305f4d41ad6c6`
for the pseudo-random case and
`c4b7d5395f250f0ce7ffd96368cee7d8f339a80f4f085bb2fb50ec3c8dce9056`
for the co-circular case. The 16-to-128 random series grows by 243.3x for an 8x input increase.
More importantly, the float-predicate implementation rejects valid non-collinear co-circular
direction sets at 96 and 128 samples as if no triangle existed. This is a correctness failure
before it is a latency optimization.

The real managed `zircon_editor` production build reached neither animation compiler because
foreign `zircon_runtime_interface/src/hub_protocol/recent_projects/store.rs` failed first: a
Rust-2024 let-chain is compiled under edition 2021 and `File::by_ref` is ambiguous between
`Read` and `Write`. No current-source package timing or passing animation test is claimed.

### Reference decision and implementation order

Unreal remains the primary architecture reference: `BlendSpace.cpp:2755-2908` prepares
triangulation/grid data outside steady evaluation, and `2994-3135` walks precomputed neighbors
from a cached triangle index. Fyrox independently uses `spade::DelaunayTriangulation` when points
change and stores the resulting triangles. Spade 2.15.1 documents exact geometric predicates,
stable bulk loading, convex-hull iteration, and average `O(n log n)` bulk loading; it is the vetted
geometry implementation to evaluate instead of extending Zircon's handwritten predicates.

Implementation remains dependency ordered:

1. Add RED regressions for valid co-circular point sets, deterministic triangles/hull, duplicate and
   truly collinear rejection, and shared compiler/runtime artifact parity.
2. Keep source semantics in the neutral Runtime compiler artifact and make the plugin lower that IR
   into runtime-specific triangles, neighbors, and ordered hull edges with stable sample indices.
3. Delete the plugin source compiler, public point-construction API, and handwritten
   Delaunay/overlap path; production caches call the shared-compiler runtime services.
4. Sample outside the hull from precompiled edges with no per-query map allocation, then persist a
   per-instance triangle hint in the state-machine instance owner.
5. Re-run managed scales `4/32/128/512`, inside/outside trajectories, allocations, p50/p95/p99, and
   correctness parity before reporting an optimization result or updating protected ledgers.

## 8. 2026-08-26 implementation and post-algorithm profile

M1 source authority and topology construction are implemented in the shared checkout:

- graph and state-machine production caches now call `compile_animation_graph_runtime` and
  `compile_animation_state_machine_runtime_bundle`; both invoke the framework semantic compiler
  first and lower only an accepted indexed artifact;
- layer lowering consumes the same state-machine artifact and no longer clones a complete base
  source machine or redefines layer-weight diagnostics;
- the public plugin `BlendSpacePoint*` and associated source `compile` APIs are removed;
- `BlendSpace2D` uses Spade 2.15.1 stable Delaunay bulk loading and stores canonical triangles,
  opposite-edge neighbors, and convex-hull edges; steady hull sampling reads the retained edge
  array instead of allocating a `BTreeMap`;
- sampling walks prepared neighbors from a deterministic initial triangle with a bounded linear
  correctness fallback. Persisting the previous triangle per animation instance remains an M1
  follow-up because the current shared evaluator API has no instance-owned blend query state.

The post-implementation release harness mirrors production topology lowering, including triangle
canonicalization and neighbor/hull construction. It was compiled with the managed build's exact
Spade 2.15.1 rlib; source, executable, PDB, `TEMP`, and `TMP` remain under
`F:/zircon-profiles/animation-blend-space-20260826`. Harness SHA-256 is
`2bcc29c4ea3161e77306627353ec04bfff00bdec0ab8cba795a34f80c8b07975`.

| Shape | N | Samples | Median | P95 | Triangles | Hull edges |
|---|---:|---:|---:|---:|---:|---:|
| deterministic pseudo-random | 16 | 101 | 22.8 us | 38.4 us | 23 | 7 |
| deterministic pseudo-random | 32 | 101 | 50.8 us | 63.7 us | 51 | 11 |
| deterministic pseudo-random | 64 | 101 | 126.1 us | 180.8 us | 115 | 11 |
| deterministic pseudo-random | 96 | 101 | 175.3 us | 285.0 us | 179 | 11 |
| deterministic pseudo-random | 128 | 101 | 245.0 us | 302.6 us | 241 | 13 |
| deterministic pseudo-random | 512 | 31 | 1,421.4 us | 2,149.6 us | 1,004 | 18 |
| deterministic pseudo-random | 1,024 | 21 | 2,804.3 us | 3,108.2 us | 2,027 | 19 |
| co-circular direction samples | 96 | 101 | 142.8 us | 183.2 us | 94 | 96 |
| co-circular direction samples | 128 | 101 | 194.2 us | 229.9 us | 126 | 128 |
| co-circular direction samples | 1,024 | 21 | 2,295.0 us | 2,636.9 us | 1,022 | 1,024 |

For the same deterministic input generators, the old 128-point random probe took 11.434 ms while
the new median is 0.245 ms; the old 96/128-point co-circular probes took 9.900/27.199 ms and
produced zero triangles, while the new implementation produces the required `N - 2` triangles in
0.143/0.194 ms median. These comparisons are directional because the RED values were single probe
runs while the new values are sampled medians. The new random series grows by 11.4x from 128 to
1,024 points for an 8x input increase, versus the old 243.3x growth from 16 to 128, which is
consistent with replacing the quartic construction path by the documented average `O(N log N)`
bulk load plus `O(N log N)` deterministic post-processing.

The managed Windows package build remains blocked before this plugin at the same foreign
`zircon_runtime_interface/src/hub_protocol/recent_projects/store.rs` edition-2021 let-chain and
ambiguous `File::by_ref` errors. Spade itself compiled successfully in that managed D-drive lane;
the plugin, integration tests, allocation counters, per-instance cached-triangle trajectory, and
power measurements have not passed a production executable gate. No end-to-end engine latency,
power parity, acceptance, commit, or WeCom completion is claimed.

## 9. Shared compiler validation RED recheck

The topology-only post profile above did not include the framework compiler's source validation.
A second end-to-end code review found that `compile_state_kind` still used linear `Vec::contains`
for every BlendSpace sample and that `contains_non_collinear_points` enumerated every point triple
for a fully collinear input. The resulting pre-topology costs are `O(N^2)` for valid unique data
and `O(N^3)` for the invalid collinear rejection path, so the earlier topology measurements cannot
yet qualify the complete compiler as average `O(N log N)`.

The current validation algorithms were reproduced exactly in a release-mode standalone harness.
Its source SHA-256 is
`d7127b47b7993f414d464daa3cb8fcc3c0d30eb48c979085d35ccf679ddaadaa`
(case-insensitive hex), and the source, executable, PDB, `TEMP`, and `TMP` are under
`F:/zircon-profiles/animation-blend-space-20260826`.

| Shape | N | Samples | Median | P95 | Accepted |
|---|---:|---:|---:|---:|---|
| deterministic pseudo-random | 128 | 31 | 6.6 us | 7.2 us | yes |
| deterministic pseudo-random | 512 | 31 | 95.1 us | 205.5 us | yes |
| deterministic pseudo-random | 1,024 | 31 | 325.8 us | 447.0 us | yes |
| deterministic pseudo-random | 2,048 | 11 | 1,534.8 us | 2,658.3 us | yes |
| deterministic pseudo-random | 4,096 | 11 | 5,848.7 us | 8,549.8 us | yes |
| exactly collinear | 64 | 11 | 64.1 us | 66.0 us | no |
| exactly collinear | 128 | 11 | 584.0 us | 754.8 us | no |
| exactly collinear | 256 | 11 | 5,223.3 us | 7,437.9 us | no |
| exactly collinear | 512 | 5 | 38,257.1 us | 46,965.3 us | no |

The random series grows by about 886x for a 32x input increase, close to the expected quadratic
trend. The collinear series grows by about 597x for an 8x input increase, close to cubic growth and
large enough to make invalid authoring input an avoidable compiler-latency hazard. The repair is
structural: ordered canonical float-bit keys provide `O(log N)` duplicate admission, and one
baseline pair followed by one linear scan replaces triple enumeration. Collinearity uses
`robust::orient2d`, the adaptive exact-predicate implementation already selected transitively by
Spade, rather than an epsilon whose answer changes with authored coordinate scale.

### Shared validation post profile

The optimized harness mirrors the production canonical keys, `BTreeSet` admission, baseline
selection, and robust orientation scan. It was compiled against the exact managed-lane
`robust 1.2.0` rlib. Its source SHA-256 is
`79aa3cee682948002010c1cfe1bf239479ab381b24aa334591d0353ae11b44a5`; all artifacts remain in
the same F-drive profile directory.

| Shape | N | Samples | Median | P95 | Accepted | Pre/post median |
|---|---:|---:|---:|---:|---|---:|
| deterministic pseudo-random | 128 | 31 | 7.2 us | 9.4 us | yes | 0.92x |
| deterministic pseudo-random | 512 | 31 | 33.5 us | 43.0 us | yes | 2.84x |
| deterministic pseudo-random | 1,024 | 31 | 74.4 us | 128.6 us | yes | 4.38x |
| deterministic pseudo-random | 2,048 | 11 | 173.0 us | 201.5 us | yes | 8.87x |
| deterministic pseudo-random | 4,096 | 11 | 437.1 us | 456.0 us | yes | 13.38x |
| exactly collinear | 64 | 11 | 11.4 us | 12.7 us | no | 5.62x |
| exactly collinear | 128 | 11 | 22.9 us | 23.6 us | no | 25.50x |
| exactly collinear | 256 | 11 | 46.6 us | 48.8 us | no | 112.09x |
| exactly collinear | 512 | 5 | 112.2 us | 113.5 us | no | 340.97x |

For valid data, a 32x increase from 128 to 4,096 points now grows by 60.7x instead of 886x,
consistent with ordered `O(N log N)` admission. The fully collinear 64-to-512 series grows by 9.8x
for an 8x input increase instead of 597x. The new correctness regressions also accept a genuinely
non-collinear triangle whose area is below `f32::EPSILON` and continue to reject signed-zero
duplicate positions. Combining the independent 1,024-point random medians gives about 2.879 ms
for shared validation plus topology lowering (0.074 ms + 2.804 ms), but this is still a standalone
algorithm estimate, not a managed package or product-frame measurement.

## 10. Graph parameter rebinding RED and hard-cut decision

The compiler/evaluator pairing review found a second authority leak after node edges had already
become dense indexes: `AnimationCompiledGraphNode::{Blend, Additive}` still retained
`weight_parameter: Option<String>`. Plugin lowering rebuilt a `BTreeMap<&str, ParameterSlot>` from
the artifact parameter table and looked up every node name again. This duplicated semantic binding,
retained strings in the shared IR, and made runtime lowering `O(P log P + N log P)` instead of a
bounded slot conversion.

A release-mode harness isolates the current string rebinding and the proposed dense-slot lowering
while retaining the same output allocation and checksum. Its source SHA-256 is
`5ebcb9bea5be178b9d8315d78046ea088cec819185b0cf5086fc2957fdce350c`; source, executable, PDB,
`TEMP`, and `TMP` remain under the F-drive profile directory.

| Parameters / nodes | Samples | String rebind median / P95 | Dense slot median / P95 |
|---:|---:|---:|---:|
| 128 | 101 | 42.1 / 55.0 us | 0.2 / 0.2 us |
| 512 | 101 | 243.7 / 485.7 us | 0.4 / 0.5 us |
| 1,024 | 101 | 538.4 / 782.3 us | 0.6 / 0.7 us |
| 4,096 | 31 | 3,771.7 / 6,066.5 us | 1.9 / 2.2 us |
| 16,384 | 31 | 25,461.3 / 52,301.1 us | 53.0 / 83.5 us |

These are compiler-lowering probes, not typical asset-size or end-to-end timings. They establish
the algorithmic direction: the framework artifact must own `Option<usize>` parameter slots, the
plugin may only capacity-check and narrow those indexes, and the lowering-time name map and lookup
must be deleted. Tests must assert the shared slot value and the absence of plugin rebinding before
the hard cut is considered implemented.

The hard cut is now implemented: both compiled blend/additive nodes retain `Option<usize>`, the
framework test asserts the authored `weight` parameter resolves to slot zero, and plugin lowering
only performs the checked `usize -> u32` narrowing used by its evaluator. Repository search finds
no remaining compiled `Option<String>` weight parameter, plugin `BTreeMap`, or
`resolve_parameter` path. The 11/11 scene-animation boundary batch includes this source/consumer
contract, but managed Rust compilation and contract tests remain blocked by the foreign interface
file documented above.

## 11. Blend query scale-consistency RED

Reviewing the query path before profiling persisted hints exposed a correctness mismatch with the
new exact source admission. Framework compilation now accepts every finite, unique, genuinely
non-collinear `f32` triangle, but plugin `geometry::barycentric` still rejects any denominator whose
absolute value is at most `f32::EPSILON`. A triangle at coordinates
`(0,0), (1e-20,0), (0,1e-20)` has a valid signed double area near `1e-40`; runtime sampling rejects
its interior and falls back to a hull projection whose `f32` squared length also falls below the
same unrelated epsilon, collapsing the result to one vertex. Conversely, coordinates near `1e30`
overflow the `f32` cross products to infinity and produce `inf / inf` NaN weights.

Unreal normally queries normalized blend-space coordinates, but Zircon's current asset contract
does not own axis normalization bounds, so silently relying on author scale would make compiler and
runtime acceptance disagree. The repair must calculate barycentric and segment projection
intermediates in `f64`, test the already validated triangle denominator against exact zero, and
return dimensionless weights in `Real`. Contract tests must require the same interior 0.5/0.25/0.25
blend at both tiny and large scales before hint-state optimization proceeds.

The repair is implemented in the single geometry owner. A release harness using the exact old and
new barycentric formulas reports `None -> Some([0.5, 0.25, 0.25])` at `1e-20` and
`Some([NaN, NaN, NaN]) -> Some([0.5, 0.25, 0.25])` at `1e30`. On normalized coordinates the old
f32 path measured 17.102 ns/query median and 21.744 ns P95; the f64 path measured 18.924 ns median
and 21.790 ns P95, a 1.822 ns median cost for scale-consistent results. Harness SHA-256 is
`281ebee29d3acad0ebf19865e18f09007e7305426a07ecb7ba82f1c6007a16c8`; all artifacts remain in
the F-drive profile directory. The external blend-space contract now runs the same interior query
at `1e-20` and `1e30`; managed execution remains pending behind the foreign interface failure.

## 12. Persisted triangle hint RED and instance-owner decision

Current `BlendSpace2D::sample` always seeds the adjacency walk with triangle zero. Unreal
`FBlendSpaceData::GetSamples2D` instead receives an in/out cached triangle index owned by the
animation instance, validates it against the prepared topology, and advances that index while
walking. Zircon's equivalent owner is the plugin `AnimationEvaluationPipeline` keyed by
`MachineInstanceKey`, including nested machines. A retained hint is performance-only, but its map
must still be bounded/retired with instance lifetime and replacement epoch; placing it inside the
shared compiled asset would incorrectly mix entities.

A release harness mirrors production Spade topology canonicalization, opposite-edge neighbors,
f64 barycentric queries, bounded neighbor walk, and full-scan fallback. It samples 8,192-point
smooth circular trajectories through co-circular topologies. Source SHA-256 is
`e74718d9a0efd6bb92012f9acf6ca1dc3cc3a9d4606ee52a815b8eeac565067c`; all artifacts remain under
the F-drive profile directory.

| Samples | Start mode | Median / P95 | Mean walk steps | Fallback checks |
|---:|---|---:|---:|---:|
| 96 | triangle zero | 224.475 / 437.329 ns | 9.111 | 0 |
| 96 | retained hint | 15.955 / 36.169 ns | 1.002 | 0 |
| 128 | triangle zero | 196.106 / 326.184 ns | 8.543 | 0 |
| 128 | retained hint | 13.867 / 15.295 ns | 1.002 | 0 |
| 512 | triangle zero | 224.011 / 287.708 ns | 9.608 | 0 |
| 512 | retained hint | 14.014 / 17.688 ns | 1.002 | 0 |
| 1,024 | triangle zero | 231.042 / 281.592 ns | 9.049 | 0 |
| 1,024 | retained hint | 13.965 / 41.663 ns | 1.002 | 0 |

The smooth-trajectory median improves by roughly 14x to 16.5x. The production hard cut now keeps a
dense `Option<usize>` hint per compiled state slot in `StateMachineBlendSamplingState`, while the
bounded 4,096-entry instance cache is owned by `AnimationEvaluationPipeline` and keyed by the full
`MachineInstanceKey`, including nested machines and layers. Shared `CompiledAnimationStateMachine`
artifacts remain immutable. Entry evaluation, transition-candidate evaluation, normalized time,
clip-event graph sampling, and pose graph sampling all pass the same instance key; the previous
stateless `graph_samples_for_state` hot path has been deleted rather than retained as compatibility.

The hint is deliberately excluded from event-admission semantic checkpoints. A follow-up release
probe modeled the exact `BTreeMap` filter and a 16-slot boxed hint row: at 128/512/4,096 active
instances, scanning costs 2.1/9.7/164.2 us median, while cloning the hint map costs
24.5/235.4/2,291.9 us median. The incremental deep-copy costs are 22.4/225.7/2,127.7 us, enough to
erase the warm-start gain. The harness SHA-256 is
`22a8e5cbb18e139f8936eaa70e2fc50bd833597726018b3e3035b4e3b5e1c891`; source, executable, PDB,
`TEMP`, and `TMP` are confined to the F-drive profile directory. Because the hint cannot change
sampling output, speculative evaluation may safely warm it without transaction rollback.
Replacement reset still clears instance hints, state-count drift resets the dense hint row, and
topology revision drift is safe because `sample_with_hint` rejects out-of-range indexes before
walking. Unit regressions cover per-state hint retention and deterministic second-chance eviction. The
scene-animation source boundary remains 11/11 GREEN in 34.349 seconds, rejects production graph
sampling that omits the instance cache, and rejects adding the performance cache to the semantic
runtime checkpoint. The latest managed Windows package
validation request `9fd854d4743f49959bea70760f862d27` failed before Cargo acquisition with
`unmanaged_artifacts_detected` for `E:/ZirconBuilds/mvp-resource-management-projects`; no Cargo job or
Rust test binary was created. Therefore the implementation and standalone algorithm profile are
recorded, but package tests, allocation/power attribution, and product-frame acceptance remain
pending.

## 13. Per-instance dense parameter projection RED and implementation plan

The next whole-module review found a repeated binding stage after source parameters had already
been compiled to dense slots. `CompiledAnimationStateMachine::parameter_values` walks every
compiled parameter name, performs a `BTreeMap<String, AnimationParameterValue>` lookup, and
allocates a new `Vec<Option<&AnimationParameterValue>>` for every `evaluate` and
`graph_samples_for_state_with_blend_sampling` call. A normal root-machine sample performs entry,
normalized-time, event, and pose projections; transitions, layers, and nested machines add more.
The ECS projection already records `AnimationStateMachinePlayerComponent::last_changed()` and the
state-machine asset revision, but neither generation reaches the compiled evaluator. The existing
instance cache therefore retains topology warm state while discarding the denser and more broadly
consumed parameter projection after every call.

A release harness reproduces the current ordered name lookup and allocation with four consumers,
then compares projecting once and reusing the same dense row. Its source SHA-256 is
`4f74132f9711508214e1f8bc042c29c7cd87d6bf8bed95a218911f3c1a30649c`; source, executable,
PDB, `TEMP`, and `TMP` remain under `F:/zircon-profiles/animation-blend-space-20260826`.

| Parameters | Consumers | Repeated projection median | Project once median | Median ratio |
|---:|---:|---:|---:|---:|
| 8 | 4 | 2.0 us | 0.6 us | 3.33x |
| 32 | 4 | 11.9 us | 3.2 us | 3.72x |
| 128 | 4 | 76.0 us | 19.5 us | 3.90x |
| 512 | 4 | 363.2 us | 92.1 us | 3.94x |

This probe isolates parameter projection and is not a frame-time claim. It establishes that local
lookup micro-optimizations would preserve the wrong lifecycle. Unreal's `FAnimInstanceProxy`
copies required instance data in `PreUpdate`, updates the graph once through a frame-owned
`FAnimationUpdateContext`, then evaluates through the same proxy; its update/evaluation traversal
counters and frame counters belong to the instance rather than the shared animation asset. Fyrox
likewise owns one `ParameterContainer` on `Machine` and passes the same container through layer
update, transition conditions, and pose evaluation. Zircon should preserve its stronger compiled
dense-slot contract while adopting the same instance/update ownership principle.

Implementation order is fixed before production edits:

1. Add RED source/unit coverage requiring one per-instance dense parameter row, invalidation by
   player change tick and compiled parameter-layout identity, reset/eviction with the existing
   bounded instance-cache eviction, and absence of the production `parameter_values` allocation path.
2. Make the compiled parameter layout immutable shared identity, and add an owned dense-value row
   consumed directly by transition conditions and BlendSpace slots. The public pure evaluator may
   create one temporary row; runtime consumers may not rebuild it per operation.
3. Carry a typed parameter projection revision from the ECS scan through root, nested, layer,
   transition, normalized-time, event, and pose calls. The `MachineInstanceKey`-owned cache rebuilds
   only when the player revision or compiled layout identity changes.
4. Keep the row outside semantic event checkpoints: it is a deterministic projection of the
   authoritative parameter map, not mutable animation state. Pipeline replacement/reset and cache
   eviction discard it together with the triangle hint.
5. Re-run the focused source guard, Rust unit/contract tests when the managed lane is available,
   and a post harness measuring unchanged and per-frame-changing parameters separately. No power,
   frame-time, or optimal-scale conclusion is permitted from the standalone projection probe.

### Dense projection first implementation and post profile (rejected as product closure)

The hard cut is now implemented in the plugin product path:

- `PendingStateMachinePoseSample` carries a typed revision derived from the ECS player change tick;
  root, nested, layer, interruption, normalized-time, event, and pose calls pass one borrowed
  `StateMachineParameterProjection` rather than recreating map-only argument surfaces;
- compiled parameter names are immutable `Arc<[String]>` layout identity. The bounded
  `MachineInstanceKey` cache owns one `Box<[Option<AnimationParameterValue>]>` per instance and
  rebuilds it only when the ECS revision or layout identity changes;
- the same instance entry owns BlendSpace hint state, is removed with inactive entities, resets on
  replacement, and uses one existing-entry tree lookup rather than `contains_key` plus `get_mut`;
- the product pipeline no longer constructs shared `AnimationStateMachineEvaluation`: that DTO
  cloned the entire parameter map and retained `graph`/`transitioned` fields that no pipeline caller
  consumed. A plugin-private result owns only active state and requested transition. The public pure
  evaluator remains available and projects one temporary dense row;
- condition bytecode reads both borrowed and owned dense rows through one internal value-table
  contract, so the standalone public condition evaluator does not regress to cloning every value.

The post harness models the retired product path as one full parameter-map clone plus four name
lookup/allocation projections. It compares an unchanged cached revision and a changed revision that
rebuilds the owned dense row once, with all four consumers reading that row. Per measurement it
batches 64 evaluations to avoid timer quantization. Source SHA-256 is
`a1509abd27755a60e2304b4d5d9d4ce2f2bd6901d71312725947fc82e04568ea`; source, release
executable, PDB, `TEMP`, and `TMP` are under the F-drive profile directory.

| Parameters | Old clone + four projections median / P95 | Unchanged revision median / P95 | Changed revision median / P95 | Old/unchanged | Old/changed |
|---:|---:|---:|---:|---:|---:|
| 8 | 2.742 / 5.100 us | 0.028 / 0.031 us | 0.590 / 1.256 us | 97.50x | 4.64x |
| 32 | 16.957 / 40.512 us | 0.089 / 0.101 us | 3.168 / 6.296 us | 190.40x | 5.35x |
| 128 | 144.017 / 386.246 us | 0.414 / 1.054 us | 20.082 / 33.898 us | 347.82x | 7.17x |
| 512 | 491.103 / 828.115 us | 1.815 / 11.875 us | 104.743 / 143.218 us | 270.49x | 4.69x |

These ratios isolate parameter-map clone, name lookup, allocation, dense-row rebuild, and direct row
consumption. They intentionally exclude the global `BTreeMap<MachineInstanceKey, ...>` lookup,
graph evaluation, pose work, allocator attribution, and power; they are not product-frame ratios.
The source boundary is 11/11 GREEN in 35.192 seconds and now rejects reintroducing the shared DTO
clone, map-only product projection, stale instance retention, or cache checkpointing.

Managed Windows job `75f8a49cf9c34f3099a12150a5c34a4a` entered Cargo but failed first at
foreign `zircon_runtime_interface/src/hub_protocol/recent_projects/store.rs:170` because an
edition-2024 let-chain is compiled as edition 2021; the job released with an empty process tree.
Coordinator validation-copy `d48dbc5045304b069ca3a3c9b5673473` first required an explicit ZrVM
descriptor. A second copy `fc2edfe9091341289f62e2fef02b32a3`, pinned to external ZrVM commit
`61b79becf64efdae8406385ba2c880620831b4b3`, then failed closure planning on the known missing
compile-time resource `core/framework/render/environment/skybox.rs`. The plugin and its new Rust
unit tests therefore have not compiled or executed in a valid production package lane; no
acceptance, power result, commit, or milestone completion is claimed.

### Post-implementation lifecycle re-review: ECS change tick is not a parameter revision

A second product-path review found that the implementation above is not yet an acceptable closure.
`PendingStateMachinePoseSample` still clones the complete `AnimationParameterMap` while scanning
every playing instance. More importantly, `AnimationStateMachinePlayerComponent::last_changed()`
cannot identify parameter changes: the same component owns `active_state`, and the normal state
machine commit rewrites that field. A running machine can therefore receive a new component change
tick every frame even when its parameter values are unchanged. Keying the dense row by that tick
rebuilds the row on the exact steady-state path that the cache is meant to preserve.

This does not invalidate the measured cost of the retired repeated projections, but it narrows the
post-profile interpretation. The `unchanged revision` column is the lower-level instance-cache path,
not yet the end-to-end ECS product path. The current source must not be accepted or described as a
complete product-path parameter projection optimization until the scan-time clone and false
invalidation are removed.

The corrective architecture is fixed before the next production edit:

1. Introduce one runtime-instance `AnimationParameterSet` owner in the shared animation framework.
   It stores an immutable shared `AnimationParameterMap` snapshot plus an opaque content revision.
   Clones used by ECS transactions and frame requests share the snapshot; an actual insert/remove
   performs copy-on-write and advances the independent revision.
2. Hard-cut graph and state-machine player components to this set. Do not retain a `BTreeMap`
   compatibility field, conversion facade, or component-change fallback. Serialization remains the
   existing map representation; the runtime revision is reconstructed and never becomes asset data.
3. Carry the set revision directly into `StateMachineParameterProjection`. Active-state, playing,
   skeleton, and asset changes continue to control sampling admission, but may not invalidate the
   dense parameter row.
4. Re-profile the true steady scan/request/cache path with map clone versus shared-set clone and
   direct dense consumption. Report changed-parameter copy-on-write separately. Keep frame time,
   graph/pose work, allocator attribution, and power outside the claim until a production build can
   execute.

A pre-edit release harness models those three lifecycle paths with 64 frames per sample and four
dense consumers. It keeps the old path's full map clone and dense rebuild, includes an `Arc` request
clone plus direct dense consumption for the proposed stable path, and forces an actually shared map
before each changed-parameter copy-on-write. Source SHA-256 is
`6b292578df5e288488ab19ef1a9df04bf476a0a15c8ece9f2a48ec7efe9fc657`; source, release
executable, PDB, `TEMP`, and `TMP` are confined to
`F:/zircon-profiles/animation-blend-space-20260826`.

| Parameters | Old map clone + rebuild median / P95 | Shared stable median / P95 | Changed COW + rebuild median / P95 | Old/stable median |
|---:|---:|---:|---:|---:|
| 8 | 1.615 / 2.284 us | 0.051 / 0.057 us | 2.073 / 2.287 us | 31.67x |
| 32 | 7.903 / 12.996 us | 0.153 / 0.446 us | 7.285 / 8.281 us | 51.65x |
| 128 | 37.851 / 130.412 us | 0.596 / 1.657 us | 37.706 / 73.123 us | 63.51x |
| 512 | 197.060 / 1,430.942 us | 2.276 / 5.500 us | 181.076 / 1,256.429 us | 86.58x |

The changed path intentionally remains comparable to the old allocation path: the optimization
moves the map copy and dense rebuild to real parameter mutation instead of pretending that mutation
is free. The stable ratios remain a data-structure probe, not a product frame or power claim.

### Shared runtime parameter owner and proxy implementation status

The non-schema foundation is now implemented without retaining the rejected change-tick key:

- shared framework `AnimationParameterSet` owns `Arc<AnimationParameterMap>` plus an opaque,
  process-unique content revision. Clone is constant-size; insert/remove/clear use copy-on-write and
  advance the revision only when content changes. Serialization remains the parameter map and does
  not persist the runtime revision;
- graph and state-machine ECS projections retain one set per entity, synchronize it from the current
  authoritative player map, and pass cheap set clones to frame requests. Removed entities,
  replacement reset, and disabled graph/state-machine features retire those snapshots;
- the state-machine instance cache keys its dense row by `AnimationParameterRevision` plus compiled
  layout identity. Active-state updates can no longer force dense-row rebuilds;
- the graph frame cache also owns the set instead of another deep map clone. Equality first benefits
  from shared `Arc` identity and still falls back to content equality, preserving cross-entity cache
  hits for independently projected but equal parameter values.

A second release harness measures the implemented proxy rather than the final component-owner
shape. Its stable path performs the current map equality synchronization, clones the shared set, and
reuses the dense row; its changed path copies the map into the proxy and rebuilds the row. Source
SHA-256 is `858c47aa3cc3b1f532ccabcb5f3db2ac2b6747e1535d0fadbecc268e3c546bcb`, and all
source/binary/PDB/temp artifacts remain under the same F-drive profile directory.

| Parameters | Old map clone + rebuild median / P95 | Implemented proxy median / P95 | Changed proxy + rebuild median / P95 | Old/proxy median |
|---:|---:|---:|---:|---:|
| 8 | 1.632 / 2.040 us | 0.196 / 0.215 us | 1.790 / 2.332 us | 8.33x |
| 32 | 7.806 / 13.679 us | 0.742 / 1.190 us | 7.845 / 13.717 us | 10.52x |
| 128 | 39.771 / 56.420 us | 3.029 / 6.378 us | 40.918 / 71.142 us | 13.13x |
| 512 | 182.368 / 248.093 us | 13.123 / 22.068 us | 178.129 / 211.425 us | 13.90x |

This is an executable intermediate layer, not the final hard cut. Until graph and state-machine
player components own `AnimationParameterSet` directly, projection synchronization must compare the
authoritative `BTreeMap` with the retained snapshot on every sampled frame, so the stable scan still
has `O(P)` comparison work. Coordinator transfer preview request
`486ecb93e9cd405281b0daefcc1886ed` at baseline epoch 443 rejected the required
`scene/world/project_io/scene_asset.rs` consumer migration with `source_owner_executable`; current
blob SHA-256 is `e3305645731840b122ee2b4f41636a74796ea2e1f27716482f89fd327654aa92`, and its current owner is
active MVP00 session `mvp00-current-source-convergence-r2-01a00797-20260818`. The remaining scene
fixture consumer `scene/tests/support.rs` was transferred without editing under fingerprint
`0f7a18551f898541e09aace5cc52d646065f43f0c94c98bfb2b5a77cdc064c7c`; apply request
`4a22affb28004d6c9393bcfdce3310b7` succeeded. Frameworks01 therefore does not edit the blocked
scene-asset blob, does not leave a compatibility field, and keeps the component hard-cut source
guard RED until the owner releases or transfers the exact blob.

The full scene-animation boundary batch currently passes 13/14 tests in 44.235 seconds. Its only
failure is the separate component-owner guard requiring both player components to own
`AnimationParameterSet`; all shared-owner, compiler, graph/state proxy, robust-geometry, and
dependency-audit guards are GREEN. Exact touched-file rustfmt and diff checks are also GREEN. Rust
product compilation remains blocked by the unchanged foreign interface file SHA-256
`753b88faad1ef07ef9d17765dacb1cb4807146f93e8362e50cf4f6651c17d8bf` and the still-missing
`core/framework/render/environment/skybox.rs` validation-copy closure. No accepted milestone,
commit, frame-time, allocator, power, or optimal-scale claim is made.

## 14. Blend-space outside-hull location RED and optimization plan

The post-Spade whole-module review found one remaining structural query defect in
`BlendSpace2D::sample_with_hint`. A production sampling state starts with no triangle hint, so its
first query skips adjacency walking and linearly tests every prepared triangle. More importantly,
when a hinted walk reaches a convex-hull edge it returns the same value as an invalid/looping walk.
The caller then linearly retests every triangle, scans every hull edge, and stores no next hint.
Continuous joystick/controller values outside the authored hull therefore repeat `O(T + H)` work
per sample for `T` triangles and `H` hull edges even though the topology walk has already proved the
query is outside.

Unreal's `FBlendSpaceData::GetSamples2D` is the primary behavior reference: it initializes an
invalid cached triangle to the middle prepared triangle, walks adjacency, handles a convex-perimeter
exit directly, and retains the boundary triangle for the next sample. Fyrox's current
`BlendSpace::fetch_weights` linearly scans all triangles and then all triangle edges; that is a
useful semantic comparison but not the target complexity. Zircon must preserve its existing exact
nearest-hull projection and deterministic prepared topology while adopting the retained-location
lifecycle:

1. distinguish `Inside`, proven `OutsideHull`, and abnormal `Failed` walk outcomes;
2. start a missing/invalid hint at a deterministic middle triangle;
3. on a proven hull exit, skip the redundant all-triangle fallback, evaluate the prepared unique
   hull edges, and retain the boundary triangle;
4. reserve the linear triangle scan for an abnormal topology-walk failure only; and
5. add behavioral and source guards for outside-hull hint retention and the absence of the normal
   hull-exit-to-triangle-scan path.

The semantic upper bound after this slice is `O(W + H)` for a normal outside query, where `W <= T`
is the adjacency walk length; temporally coherent repeated queries begin at the retained boundary
triangle.

The pre-implementation release harness is
`F:/zircon-profiles/animation-blend-space-20260826/blend-space-outside-hull-location-profile.rs`,
SHA-256 `71f1e9d4083f673152d64182cdce8472bb03d899efab7ff92376769f057608b3`. Source, executable, and
temporary compiler output all remain on F. It constructs dense regular-grid topology and isolates
the already-proven-outside work: the retired path retests all triangles and then scans prepared
unique hull edges, while the proposed path scans only those hull edges.

| Grid side | Triangles | Hull edges | Iterations | Retired median / P95 | Proposed median / P95 | Median ratio |
|---:|---:|---:|---:|---:|---:|---:|
| 16 | 450 | 60 | 50,000 | 4.6144 / 7.8505 us | 0.6109 / 0.9298 us | 7.55x |
| 32 | 1,922 | 124 | 20,000 | 24.1781 / 54.7058 us | 1.6255 / 3.8075 us | 14.87x |
| 64 | 7,938 | 252 | 5,000 | 137.5800 / 155.1148 us | 2.1737 / 7.5648 us | 63.29x |
| 128 | 32,258 | 508 | 1,500 | 298.3611 / 378.4278 us | 4.8079 / 21.1065 us | 62.06x |

This establishes that the normal hull-exit fallback is a real topology-scale bottleneck; it is a
data-structure probe, not current product frame time, power, or full-walk evidence.

The retained-location implementation is now present:

- `TriangleWalk` distinguishes an inside hit, a proven hull exit, and an abnormal failed walk;
- missing or stale hints start at the deterministic middle prepared triangle, matching the Unreal
  cached-index lifecycle rather than entering a full scan;
- a normal hull exit computes the same nearest projection over prepared unique hull edges and
  returns the boundary triangle as the next per-instance hint; and
- only `Failed` enters a complete triangle scan. That rare correctness fallback deliberately skips
  nothing, because failure may occur after visiting more than the original hinted triangle.

The product unit coverage retains a boundary hint across repeated outside samples and compares a
rolling hinted walk with exhaustive sampling over a 33-by-33 query grid spanning both sides of a
nine-point authored hull. The source guard requiring the three-state walk, deterministic initial
location, and retained hull hint is GREEN.

The post-implementation release model is
`F:/zircon-profiles/animation-blend-space-20260826/blend-space-outside-hull-retained-post-profile.rs`,
SHA-256 `638fea67062f5234fc73d43c51491a427bcb3304a53a0fb97175d0855497396c`. It includes the prepared
triangle adjacency, actual walk state machine, continuous hint update, abnormal fallback, and
prepared hull scan. Source, executable, and temporary output remain on F.

| Grid side | Triangles | Hull edges | Iterations | Retired median / P95 | Retained median / P95 | Median ratio |
|---:|---:|---:|---:|---:|---:|---:|
| 16 | 450 | 60 | 40,000 | 3.9369 / 6.9689 us | 0.6364 / 0.8220 us | 6.19x |
| 32 | 1,922 | 124 | 15,000 | 24.1178 / 37.4619 us | 1.7345 / 7.6680 us | 13.90x |
| 64 | 7,938 | 252 | 4,000 | 67.0902 / 85.0357 us | 2.1968 / 5.2609 us | 30.54x |
| 128 | 32,258 | 508 | 1,200 | 258.4417 / 330.3635 us | 5.2138 / 16.2565 us | 49.57x |

The normal repeated outside query now scales with the prepared hull rather than all triangles plus
the hull in this topology model, so the identified bottleneck is absent from the implemented
control flow. Managed product unit execution remains pending behind the recorded foreign compile
blockers; these results still are not frame-time, allocator, power, or engine-parity acceptance.

## 15. State-machine instance-cache eviction RED, rejected index, and clock implementation

The next current-module review found that the retained per-instance sampling/parameter cache has
LRU semantics but not LRU-scale eviction. `StateMachineInstanceCache::state_for` stores at most
4,096 entries in a `BTreeMap`; every miss at capacity calls `min_by_key(last_used)` across the whole
map before removing one entry. A stream of `C` new machine instances at capacity therefore performs
`O(C * N)` comparisons for `N = 4,096`, even though key lookup and insertion are already ordered.
This cost is on the production path that was added to retain BlendSpace topology hints and dense
parameter rows, so it cannot be dismissed as an editor-only or validation-only concern.

Unreal is the architectural reference rather than a direct container recipe. Its
`FAnimNode_StateMachine` owns `CurrentState`, elapsed time, active transitions, pose links, and state
pose caches in the animation-node instance itself (`AnimNode_StateMachine.h:179-223`); it does not
find mutable instance state by scanning a global eviction table. Fyrox likewise updates state and
pose-node instances in place. Zircon's plugin-owned side table remains an MVP bridge until opaque
plugin instance state has a component/runtime-handle lifecycle. During that bridge it must keep
bounded retirement without making entity churn quadratic.

A release-mode Rust harness mirrors the exact current `BTreeMap` scan and compares it with a
`BTreeMap` plus `BTreeSet<(access_sequence, key)>` eviction index. Both variants prefill 4,096
entries and then admit unique keys; each row alternates 15 paired samples. The source-string SHA-256
is `cf4007e41c3821ee73de5393460306c157f3a00667d7a999450058bd2cf60dd7`, and source input,
executable, PDB/TEMP/TMP output remain under
`F:/zircon-profiles/animation-blend-space-20260826/state-machine-instance-lru`.

| New instances at capacity | Linear scan median / P95 | Ordered index median / P95 | Median ratio |
|---:|---:|---:|---:|
| 128 | 1,285.2 / 2,100.0 us | 50.9 / 100.6 us | 25.25x |
| 512 | 5,826.4 / 10,227.8 us | 218.6 / 635.5 us | 26.65x |
| 4,096 | 48,029.6 / 54,230.3 us | 1,574.6 / 2,769.3 us | 30.50x |
| 16,384 | 190,763.3 / 258,417.4 us | 6,920.1 / 12,724.0 us | 27.57x |

The exact ordered-index implementation was then rejected during post-implementation review because
it optimized misses by taxing every hit. A realistic full `MachineInstanceKey` harness compared the
retired scan, the ordered index, and a map-to-slot intrusive list for 262,144 stable hits followed by
4,096 new instances. Source SHA-256 is
`2fc193e5244e6492e6b20b039dd085bb72afc05393e04be49e73e385960ada25`.

| Candidate | Stable hits median / P95 | 4,096 churn median / P95 |
|---|---:|---:|
| retired scan/LRU timestamp | 36,200.9 / 95,974.5 us | 192,811.2 / 210,040.2 us |
| ordered `BTreeSet` index | 169,620.2 / 211,996.5 us | 5,858.6 / 12,124.9 us |
| intrusive linked slots | 52,980.1 / 92,180.3 us | 4,302.5 / 6,177.9 us |

The ordered index's 4.69x stable-hit regression is unacceptable for a frame hot path. The final
implementation instead uses a bounded second-chance clock: the `BTreeMap` remains the single lookup
owner, a fixed-length `VecDeque<MachineInstanceKey>` records probation order, and a later hit sets
one reference bit. At capacity, a referenced candidate is cleared and rotated once; a cold candidate
is removed. New one-shot nested instances remain probationary. Continuous churn is therefore
amortized `O(N + C)` rather than repeating an `O(N)` minimum scan for every one of `C` misses, while
steady hits do not rebalance another tree. `clear` and active-entity retirement update both
structures; Rust and Python source guards require the clock/pop-front path and reject `min_by_key`
inside the instance-cache implementation.

The final mixed-workload post harness uses the same full key and parameter/layout refresh shape.
Source SHA-256 is `41acc84b1379272e4d330c5dff30ce82fc2a0dafdfda0f5609b4f4ac5a4e04c2`;
21 alternating samples cover 1,048,576 stable hits plus 4,096 unique misses at capacity.

| Path | Stable hits median / P95 | 4,096 churn median / P95 |
|---|---:|---:|
| retired full-map minimum scan | 179,936.7 / 268,815.2 us | 180,332.9 / 249,040.3 us |
| implemented second-chance clock | 228,627.1 / 304,725.5 us | 6,555.8 / 12,114.1 us |

The churn bottleneck improves 27.51x. This mixed batch observed a 1.271x lookup-model median, equal
to about 0.19 ms per 4,096 hits; a separate 2,097,152-hit-only run measured 231,659.1 us retired
versus 235,349.7 us clock (1.016x), so scheduler/code-layout noise is material and no zero-regression
claim is made. The latter harness SHA-256 is
`88da57eae44bddac76e22759042e8dd4b2befb856736c5c786672b18ef73cd6c`.

All benchmark source input, executable, PDB/TEMP/TMP output remain under
`F:/zircon-profiles/animation-blend-space-20260826/state-machine-instance-lru`. These are cache data
structure probes, not product-frame, allocation, power, or engine-parity evidence. The longer-term
owner remains instance-local runtime state, matching the reference engines; this slice removes the
demonstrated MVP-side-table churn bottleneck without pretending the bridge is the final architecture.
The fresh full Frameworks01 Scene/Animation source batch remains 13/14 in 44.235 seconds: the clock,
compiler, dense projection, robust geometry, and retained-hull guards are GREEN; the sole RED remains
the separately owned player-component parameter hard cut.

## 16. State-machine admission rollback RED and first-write journal plan

The next whole-pipeline review found a second state-machine side-table cost, independent of cache
eviction. Before evaluating any state-machine sample, `tick.rs` calls
`state_machine_runtime_checkpoint` with every sampled entity. The checkpoint clones the active entity
set and filters/clones all matching entries from interrupted-transition sources, nested-machine
states, and nested-machine transitions. Only after pose/event evaluation does event-queue admission
identify the usually empty set of deferred entities. The current design therefore pays
`O(A log E + S)` selection/copy work for `A` active entities and `S` stored state entries even when
no entity is deferred; when deferral occurs, restoration additionally retains across all three live
maps before extending their filtered snapshots.

The state lifetime itself remains an MVP side-table bridge. Unreal's `FAnimNode_StateMachine` owns
`CurrentState`, elapsed time, active transitions, pose links, and state caches in the node instance
(`AnimNode_StateMachine.h:179-223`). Fyrox's `MachineLayer` likewise mutates `active_state` and
`active_transition` in the layer instance. Zircon cannot remove event-admission rollback until its
plugin instance state has the same owner lifecycle, because a deferred entity must observe exactly
its pre-evaluation semantic state on the next tick. The bounded structural fix is therefore a
transactional first-write journal, not removal of rollback and not another complete snapshot:

1. retire `state_machine_runtime_checkpoint(active_entities)` and begin an empty journal only after
   inactive side-table entries have been retained away;
2. route interrupted-source, nested-state, and nested-transition mutations through pipeline-owned
   methods;
3. on the first semantic write to one `MachineInstanceKey`, store one combined entry containing the
   previous optional values from all three maps; later writes to that key add no copies;
4. finish evaluation by moving out the journal, then restore only journaled keys whose entity is in
   the admission result's deferred set; and
5. keep warm-start hints outside the journal because they affect cost, not event/output semantics.

A first candidate with three independent journal maps was rejected before production editing. Its
release harness source-string SHA-256 is
`039903cd5f1d84c878b69194cb9ed04fa959ba8c9795fb57800019c63bfe0a2e`. At 4,096 instances and 10%
writes it improved the median by 7.42x without deferral, but at 100% writes it regressed from
38.4240 ms to 43.3034 ms (0.887x) and widened P95 from 47.6371 ms to 74.5035 ms. Repeating a full
key clone and tree insertion for each of the three maps is therefore not accepted.

The selected combined-entry harness uses the production-shaped `MachineInstanceKey`, three
`BTreeMap` state tables, string-owning state/transition values, and an `Arc` pose payload. Each row
alternates 11 measured release samples after two warmups; values are milliseconds per modeled frame.
Its source-string SHA-256 is
`23e1f690dd8c5fb41fda7aa6b6b29ca595eff90e8fa792305d3334fac22b0934`.

| Instances | Writes | Deferred among writes | Snapshot median / P95 | Combined journal median / P95 | Median ratio |
|---:|---:|---:|---:|---:|---:|
| 128 | 0% | 0% | 0.3337 / 0.6232 ms | 0.000019 / 0.000026 ms | 17,799.17x |
| 128 | 10% | 0% | 0.3191 / 1.9625 ms | 0.0518 / 0.1057 ms | 6.16x |
| 128 | 100% | 0% | 0.6744 / 2.3726 ms | 0.6276 / 0.8928 ms | 1.07x |
| 512 | 10% | 0% | 1.8753 / 2.8236 ms | 0.2327 / 0.4328 ms | 8.06x |
| 512 | 100% | 0% | 4.3248 / 11.0909 ms | 3.4209 / 5.6279 ms | 1.26x |
| 4,096 | 10% | 0% | 18.5637 / 22.5309 ms | 2.6972 / 3.9291 ms | 6.88x |
| 4,096 | 100% | 0% | 33.7578 / 96.3119 ms | 32.8532 / 70.9728 ms | 1.03x |
| 4,096 | 10% | 50% | 20.8453 / 28.7609 ms | 2.3308 / 3.4466 ms | 8.94x |
| 4,096 | 100% | 50% | 69.1878 / 96.8353 ms | 48.5893 / 95.5067 ms | 1.42x |

Both result files, executables, and compiler output remain under
`F:/zircon-profiles/animation-blend-space-20260826/state-machine-runtime-checkpoint`; no profile
artifact is on C. These are isolated data-structure measurements, not product frame-time,
allocation, power, or engine-parity evidence.

The hard cut is now implemented. `tick.rs` retains inactive state before beginning an empty runtime
transaction, evaluates ordinary and layered state machines, then moves the journal out before event
admission. The retired active-entity checkpoint type and API no longer exist. Pipeline-owned methods
record one combined previous-value entry on the first write and own every production insert/remove
in `state_machine_step.rs` and `state_machine_layers.rs`; deferred restoration iterates only journaled
keys. The product unit test covers an admitted update, a deferred replacement, an unchanged active
instance, and a deferred insertion whose three previously absent values must all disappear.

A final release harness repeats the combined-journal lookup before each of the three modeled owner
writes, matching the production method shape rather than assuming one lookup per key. Its
source-string SHA-256 is
`b5bb159468977557aaad3b366b6b7b8d66e8f78ea888d03a02d08ea95f4322e3`.

| Instances | Writes | Deferred among writes | Snapshot median / P95 | Exact owner journal median / P95 | Median ratio |
|---:|---:|---:|---:|---:|---:|
| 512 | 10% | 0% | 1.8270 / 2.2833 ms | 0.1921 / 0.3475 ms | 9.51x |
| 512 | 10% | 50% | 2.5254 / 3.9861 ms | 0.2264 / 0.5217 ms | 11.16x |
| 512 | 100% | 0% | 3.6117 / 5.3665 ms | 2.9631 / 3.7764 ms | 1.22x |
| 512 | 100% | 50% | 4.7964 / 8.0806 ms | 5.4100 / 7.7858 ms | 0.887x |
| 4,096 | 10% | 0% | 15.6739 / 22.5749 ms | 2.0363 / 4.1392 ms | 7.70x |
| 4,096 | 10% | 50% | 21.7381 / 34.9712 ms | 2.8355 / 3.6513 ms | 7.67x |
| 4,096 | 100% | 0% | 34.3202 / 64.0555 ms | 26.3290 / 70.7939 ms | 1.30x |
| 4,096 | 100% | 50% | 37.8918 / 59.2856 ms | 29.8528 / 34.3581 ms | 1.27x |

The 512-instance, 100%-write, 50%-deferred median is an observed 1.127x regression, while its P95
improves and the preceding combined-entry run measured the same shape at 1.23x faster; no
zero-regression claim is made. At the 4,096-entry production cap the exact method shape removes the
identified full-snapshot scaling defect in every measured write/rollback ratio. The fresh complete
Frameworks01 Scene/Animation source batch is 13/14 in 37.219 seconds. All journal-specific guards are
GREEN; the sole RED remains the separately owned player-component `AnimationParameterSet` hard cut.
Rust product execution remains blocked by unchanged foreign current-source failures, so this slice
is implemented/static-verified but not an accepted milestone.

## 17. Per-frame graph-evaluation content index - implemented / static verified

The next pipeline review found a per-frame algorithmic mismatch in `evaluate_graph`. The cache is
cleared by `begin_evaluation_frame`, accepts at most 256 graph-evaluation results, and currently
searches its `VecDeque` from the front for every request. Entries compare graph id, skeleton id, and
then `AnimationParameterSet` content. For `E` distinct same-graph instances with `P` parameters, a
frame can therefore perform `O(E^2 * P)` value-comparison work before the actual compiled graph
evaluation. FIFO insertion was previously corrected from `Vec::remove(0)` to `VecDeque::pop_front`,
but that only removed shift cost; it did not address the dominant lookup design.

Unreal's animation graph traversal uses instance-local update/evaluation counters and explicit
cached-pose nodes; Fyrox evaluates pose-node instances owned by each machine layer. Neither uses a
global sequential content scan across unrelated animation instances. Zircon can retain same-content
deduplication because its current compiled graph evaluation is a pure function of graph, skeleton,
and parameters, but the MVP cache must use a content identity index:

1. `AnimationParameterSet` owns an opaque runtime content fingerprint alongside its `Arc` values and
   content revision; construction and successful mutation recompute it, while clone remains O(1);
2. equal parameter values must always produce the same fingerprint, including normalization of
   `-0.0` and `0.0`; the fingerprint is not serialized or persisted;
3. `PartialEq` may reject unequal fingerprints early but must still compare complete values when
   fingerprints match, so collisions cannot change behavior;
4. the per-frame cache becomes a `BTreeMap<(graph, skeleton, fingerprint), entry>` and verifies full
   parameter equality before returning a hit; and
5. after 256 distinct keys the cache stops admitting new entries for that frame instead of evicting
   useful early entries. Frame reset still clears the complete map.

The release harness constructs production-shaped ordered parameter maps whose differing value is at
the end, then cycles hits across 32/128/256 entries. It compares the current sequential content scan
with a cached-fingerprint `BTreeMap`; query counts are inversely scaled by `entries * parameters` so
each row performs a similar comparison budget. There are 11 measured samples after two warmups.
Source-string SHA-256 is
`ee014344f568fdab2d948c21bb187f9d8009415a66b5ca25f543941a08bd57dd`.

| Entries | Parameters | Queries | Linear median / P95 | Indexed median / P95 | Median ratio |
|---:|---:|---:|---:|---:|---:|
| 32 | 8 | 8,192 | 2.576 / 6.074 us | 0.024 / 0.170 us | 106.64x |
| 32 | 32 | 2,048 | 10.627 / 15.053 us | 0.024 / 0.026 us | 444.19x |
| 32 | 128 | 2,048 | 61.425 / 72.481 us | 0.023 / 0.025 us | 2,637.28x |
| 128 | 8 | 2,048 | 11.700 / 15.029 us | 0.038 / 0.046 us | 304.85x |
| 128 | 32 | 2,048 | 51.368 / 62.473 us | 0.038 / 0.054 us | 1,340.15x |
| 128 | 128 | 2,048 | 342.718 / 420.191 us | 0.039 / 0.118 us | 8,884.63x |
| 256 | 8 | 2,048 | 26.033 / 42.792 us | 0.051 / 0.126 us | 505.84x |
| 256 | 32 | 2,048 | 95.080 / 175.476 us | 0.058 / 0.088 us | 1,648.80x |
| 256 | 128 | 2,048 | 1,072.504 / 1,369.803 us | 0.054 / 0.077 us | 19,895.72x |

The complete result, executable, and compiler output remain under
`F:/zircon-profiles/animation-blend-space-20260826/graph-evaluation-content-index`. The earlier fixed
131,072-query run timed out after two rows and is not acceptance evidence. This is an adversarial
lookup probe, not full graph evaluation, product frame time, allocation, power, or engine-parity
evidence.

The production hard cut is now implemented. `AnimationParameterSet` owns the runtime-only
fingerprint, refreshes it only after construction or a successful content mutation, normalizes
signed zero, and retains full map equality after a fingerprint match. Serialization still contains
only the parameter map. `AnimationEvaluationPipeline` owns a per-frame
`BTreeMap<(graph, skeleton, fingerprint), CachedGraphEvaluation>`; lookup is indexed and
collision-verified, frame reset clears the map, and the 256-entry limit rejects later admissions
instead of evicting earlier reusable entries. Admission uses one `entry` traversal rather than a
separate membership check followed by insertion. The old `VecDeque`, sequential `iter().find`, and
`pop_front` path are absent.

The post-cut release harness uses the same `DefaultHasher`, retained `Arc<BTreeMap<...>>`,
fingerprint-gated equality, three-part ordered key, 256-entry `BTreeMap`, and full equality check on
hit. It runs 512 cyclic hit queries per sample, nine measured samples after two warmups, and also
measures the full ordered-map fingerprint refresh paid on changed content. Its source-string SHA-256
is `bb2bc4a315c2c6bc66f25cd313d82c44918a20be5c8d5f835cc293dbc428c3cb`; result SHA-256 is
`e47f46d5403b7aa76be93121c22de570da398962d92a04fd6855a54fd4ba00c1`; executable SHA-256 is
`1b88863bdacd50d4255a4fe72000cd8d9640276b295ff1694d37f4b73add4ffb`.

| Entries | Parameters | Linear median / P95 | Indexed median / P95 | Median ratio | Fingerprint refresh median / P95 |
|---:|---:|---:|---:|---:|---:|
| 256 | 8 | 17.989 / 21.422 us | 0.0566 / 0.0674 us | 317.60x | 0.253 / 0.264 us |
| 256 | 32 | 88.563 / 123.206 us | 0.0553 / 0.0916 us | 1,602.27x | 1.025 / 1.299 us |
| 256 | 128 | 801.954 / 849.080 us | 0.0619 / 0.1441 us | 12,952.70x | 9.954 / 55.092 us |

The changed-content refresh remains `O(P)` and its 128-parameter P95 is noisy, so the result does
not justify an `O(1)` mutation claim. It removes the demonstrated `O(E^2 * P)` cross-instance scan:
steady indexed lookup is `O(log E)` plus one full equality check only for the matching fingerprint,
while fingerprint refresh is paid at actual content change. Product unit coverage locks signed-zero
and mutation identity, forced-fingerprint collision equality, bounded admission, same-content reuse,
and distinct-content separation; the Frameworks01 source guard locks absence of the sequential scan.
The fresh complete Frameworks01 Scene/Animation boundary batch is 13/14 in 39.653 seconds; the sole
failure remains the separately owned graph/state player-component `AnimationParameterSet` hard cut.
Rust product execution remains blocked by unchanged foreign current source, so this slice is not a
dynamic product-frame, power, engine-parity, or accepted-milestone result.

## 18. Compiled graph DAG evaluation - implemented / static and isolated-profile verified

The next current-source review confirms that graph compilation and graph execution still disagree
about the executable representation. Framework compilation already emits one dependency-first
`evaluation_order` for output-reachable nodes, but `compile_animation_graph_runtime` discards it.
`CompiledAnimationGraph::evaluate` instead calls recursive `collect_clips` from the output and
re-enters a shared dependency once per incoming path. A diamond graph can therefore produce
exponential node visits and duplicate instances of the same clip node even though the authored and
compiled graph contain only linear nodes. A sufficiently deep valid graph also returns to call-stack
risk after the non-recursive compiler has accepted it.

Reviewed current-source SHA-256 values are:

- `evaluation/compiled_graph/evaluate.rs`:
  `718e63d75eb8ec376f3520b517569c775aa4bac4a3aa651d67d62b06cd759e02`;
- `evaluation/compiled_graph/compile.rs`:
  `d9e687cb50f7559637fbcec709803d5d39f384fbf89bcd3f08292edf3f41c292`;
- shared `animation/compiler/graph.rs`:
  `0bd4bfc9f466f009bde98ae1af04d9e28827d0056cff424cb647306f2a2e23f0`.

Unreal remains the primary phase/instance reference. Its `FAnimNode_SaveCachedPose` uses an
evaluation counter and a thread-local scoped cache so a shared pose is evaluated once for the
current evaluation generation; `PostGraphUpdate` collects active consumer contexts and updates the
source from the highest-weight one. Fyrox pose nodes retain their own `output_pose`, while blend
nodes read child node handles and write that node-local output. Zircon does not need to copy either
object model, but a compiled node must remain a node instance rather than silently becoming one
runtime invocation per graph path.

The release baseline builds a repeated diamond from one clip plus three nodes per layer. It compares
the exact current recursive weight/path expansion with a scalar reverse-topology accumulator. There
are 11 measured samples after two warmups; source-string SHA-256 is
`88ed7655fdbca0d7e3aadc8e721423c9f730b7a0ca989352dfc0153c0fc24228`, result SHA-256 is
`8a21035da57fd2b99e726891dc4c60547369737371d55143381b684548f1d2d5`, and executable SHA-256 is
`2150e30a09a444dc2f719af7e70a891f704f8514a5a2af54fcc0964d412dd487`.

| Diamond layers | Compiled nodes | Recursive clip outputs | Topology clip outputs | Recursive median / P95 | Topology median / P95 | Median ratio |
|---:|---:|---:|---:|---:|---:|---:|
| 8 | 25 | 256 | 1 | 5.000 / 46.200 us | 0.400 / 0.700 us | 12.50x |
| 12 | 37 | 4,096 | 1 | 60.900 / 99.800 us | 0.500 / 1.500 us | 121.80x |
| 16 | 49 | 65,536 | 1 | 1,051.000 / 1,204.300 us | 1.200 / 2.600 us | 875.83x |
| 20 | 61 | 1,048,576 | 1 | 20,125.800 / 28,837.000 us | 3.200 / 4.100 us | 6,289.31x |

The accepted implementation direction is a reverse traversal of the already compiled
dependency-first topology:

1. retain `evaluation_order` in `CompiledAnimationGraph` and delete recursive `collect_clips`;
2. seed the output node with one `(mask context, additive mode) -> weight` contribution, then visit
   every reachable node exactly once in reverse topology after all of its consumers have
   contributed;
3. merge repeated contributions at the child node instead of duplicating paths. Use a hybrid
   `Empty / One / Many(BTreeMap)` accumulator so the common single-context tree path allocates no
   map, while graphs with multiple mask/additive contexts retain logarithmic merge behavior;
4. preserve the nearest-to-clip mask rule and additive dominance explicitly in the context, and
   emit at most one result per `(clip node, mask context, additive mode)` in stable slot order; and
5. keep parameter lookup behavior unchanged in this slice. Raw-map-to-dense parameter admission
   remains the separately blocked Scene owner hard cut.

The resulting execution is non-recursive. For graphs without mask/additive context splits it is
`O(V + E + C)` and one clip-node output per reachable clip. With `K` distinct semantic contexts it
is bounded by `O((V + E) * K log K + C)` rather than path count; `K` is bounded by additive mode and
compiled mask-node identities.

The production hard cut is now implemented. Runtime lowering retains the shared compiler's
dependency-first `evaluation_order`. Evaluation seeds the output, visits that order in reverse,
moves each node's accumulated input once, and propagates weights through compiled slots. The old
recursive `collect_clips` function and call are absent. `GraphContextWeights` uses `Empty` and `One`
without a tree allocation on the ordinary path and promotes to a `BTreeMap` only when distinct
mask/additive contexts meet at a node. Repeated paths with the same context merge their weight;
distinct contexts remain separate. Traversal toward a clip overwrites an outer mask with the
nearest inner mask, while additive mode is monotonic. Contributions are materialized once per
`(clip slot, mask slot, additive mode)` and sorted by Base/Additive mode, clip source slot, then mask
source slot. This ordering is an explicit deterministic runtime contract because downstream
additive rotations are order-sensitive; authoring edge traversal order is not retained as a hidden
ordering API.

Authored product contract test source covers a 12-layer diamond, nested masks plus additive mode,
stable source-slot ordering, and a 4,096-node chain. The Frameworks01 source guard requires retained
evaluation order, reverse traversal, the hybrid accumulator, and absence of `collect_clips`. An
independent Rust 2021 typecheck fixture included the exact current `evaluate.rs` and executed all
three semantic shapes without Cargo; it passed. The fresh complete Frameworks01 Scene/Animation
source batch is 13/14 in 41.615 seconds; its sole failure is the separately owned player-component
`AnimationParameterSet` hard cut. Full product tests remain unavailable because the unchanged shared
current source still fails before this plugin test target is generated.

The post-cut release harness models the production `Empty / One / Many(BTreeMap)` accumulator,
per-node vector, reverse topology, per-edge propagation, contribution allocation, and stable sort.
Each row uses two warmups and 12 measured samples with equal alternating order. To avoid treating
sub-microsecond timer noise as signal, topology samples batch 4,096 evaluations; recursive samples
batch 1,024/64/4/1 evaluations for 8/12/16/20 layers, and all values below are normalized per
evaluation. Source-string SHA-256 is
`acf9979e61a681e7fd41bd6c1f34cb6572740b4f75335eeb87a1c72d57fbf05f`, result SHA-256 is
`fb5fe74ae77d1ece1e1eb989ffa9cc56ea3e7dc8ce4d6d2a38deb3d9e38da583`, and executable SHA-256 is
`3bb1bcf62883c51773eeaf2fa7071f0d630a2ed90355eb92ef38a46f4b522da0`.

| Diamond layers | Compiled nodes | Recursive outputs | Topology outputs | Recursive median / P95 | Topology median / P95 | Median ratio |
|---:|---:|---:|---:|---:|---:|---:|
| 8 | 25 | 256 | 1 | 6.721 / 78.853 us | 0.865 / 7.518 us | 7.77x |
| 12 | 37 | 4,096 | 1 | 83.082 / 230.704 us | 1.104 / 1.629 us | 75.26x |
| 16 | 49 | 65,536 | 1 | 2,788.200 / 5,943.750 us | 1.367 / 1.853 us | 2,039.65x |
| 20 | 61 | 1,048,576 | 1 | 57,462.000 / 78,045.900 us | 1.779 / 3.463 us | 32,300.17x |

The 8-layer P95 rows show scheduler noise and are reported without filtering. The scale result still
demonstrates that runtime work is now bounded by compiled graph/context size instead of DAG path
count. Evidence remains under
`F:/zircon-profiles/animation-blend-space-20260826/compiled-graph-dag-evaluation`; no artifact is on
C. This is an isolated algorithm/data-structure result, not a product frame-time, allocation,
energy, complete pose-program, or reference-engine parity result. This section therefore records
implementation and static/isolated verification, not an accepted milestone.

## 19. Animation pose sealed-publication ownership - implemented / isolated post-profile green

The next whole-module review followed final poses beyond evaluation rather than optimizing one
clone call in isolation. Before the hard cut, `AnimationEvaluationPipeline::presentation_poses`
owned an `Arc<BTreeMap<EntityId, AnimationPoseOutput>>`. When event admission deferred even one
entity, `update_presentation_poses` deep-cloned every retained `AnimationPoseOutput`, including every
bone name and transform, before replacing the admitted rows. It then compared the complete new and
old maps. Any change caused `publish_skeletal_pose_targets` to clear and rebuild every entity's
physics target array, cloning every bone name again. Render extraction subsequently cloned each
complete pose into `RenderSkeletalPoseExtract`, and `FrameHistoryValidationKey` cloned the same pose
a third time. A 1% admitted/deferred change therefore scaled with all retained pose bytes rather
than the changed rows.

Reviewed current-source SHA-256 values before implementation are:

- animation pipeline owner: `6371ba7cd947fbf22ebbf24569199f46ad7ee6b9dbc2db8084625e7f2f00bb43`;
- animation tick/project owner: `fcb77fb7027c8e3ab7479f884a8cac9af77dc43852171e08a0e10455b65c654a`;
- neutral pose DTO: `fca6dde9056d80e8544cc46aab5461aa8be7b5af178946ad93d7aaf6cb550a18`;
- Level animation publisher: `a6bd676a461dcbd01f4b234cb6a2e83a1be3f59334d6ad4db28484dc232d25d0`;
- Level frame snapshot: `de95ed8a9a02159bad00da6a1533019861b20f8f5a9eb7bddc50a82a1a9d5db0`;
- Level render extract: `0a868b059db582d783f77d862113a028b2e2083a29eee8c6d3e02a94863e8da8`;
- render pose row: `8b64fe8f7dcbbd0029df71048274dcb80e100c770caaeff2aa24fc93b9742ee1`;
- frame-history validation key: `3ccf9b1584d8efe7c961134ba920ca20dfea2fe345688c0b5b65c3835723f789`;
- neutral skeletal-target store: `1435fd10ce79b044803f3de79fc7608ec2ddeb0002106f71aae987200c5b3409`.

The architecture references reject a global deep-owned pose-map handoff. Unreal's
`FAnimationPoseData` is a non-owning evaluation view over caller-owned pose/curve/attribute storage;
`FAnimInstanceProxy::EvaluateAnimation` writes into the output context and retains per-instance,
double-buffered runtime state. Fyrox `PlayAnimation` and machine layers own reusable output poses
and copy into retained storage rather than materializing one new global owned map for every reader.
Zircon does not need either engine's object model, but its sealed frame boundary must likewise
publish stable pose handles rather than repeatedly taking ownership of the same bone arrays.

The accepted MVP hard-cut direction is:

1. keep `AnimationPoseOutput` owned and mutable through sample/blend/IK/scene projection, then seal
   each final entity result once as `Arc<AnimationPoseOutput>`;
2. make the presentation and Level frame snapshot map own those row handles; render extract and
   frame-history keys clone the same handle, never the bone/name payload;
3. preserve the full-update fast path: compare the complete incoming map once, move it directly
   into the outer snapshot, and rebuild all physics targets only when content changed;
4. on a partial update, compare only supplied rows plus removed entity identities, return immediately
   on an unchanged batch, shallow-clone only the ordered map nodes when a change exists, and publish
   an explicit changed/removed entity list; and
5. mutate `SkeletalPoseTargets` only for that partial change list. A full update retains the existing
   clear-and-rebuild behavior, while a removed entity deletes only its target row.

This is a hard contract cut, not a compatibility facade. The old
`Arc<BTreeMap<EntityId, AnimationPoseOutput>>` publication shape and owned
`RenderSkeletalPoseExtract::pose` row must not remain as parallel APIs. Public inspection may still
materialize one requested owned pose; frame/render/physics production consumers use sealed handles.

The pre-implementation release harness models the exact current partial/full algorithms, 64-bone
poses with owned bone names, complete equality, skeletal-target materialization, the proposed
`Arc<AnimationPoseOutput>` rows, unchanged-batch early return, shallow partial-map copy, and
changed-row target replacement. Each row has two warmups and 11 measured samples in alternating
order. Allocation counts/bytes cover only publication and target projection; input pose evaluation
is prepared outside the measured interval. Final-pose sealing is measured separately so its cost is
not hidden. Source SHA-256 is
`c46b1853304131a443865a4b6580a3a27d0c9228876549b8c8f30f8a70c6fdbf`, result SHA-256 is
`9b4559bd66167ae6ebb40128d443ae28767a8a694afdd1c29e8b58f66a27176a`, and executable SHA-256 is
`e2e3aec0458e7ac2368e0db03c7b3baebdfbb1a1f4ddf619302956d822d42ae1`.

| Entities | Updated | Current median / P95 | Sealed-row median / P95 | Median ratio | Current allocs / bytes | Sealed-row allocs / bytes |
|---:|---:|---:|---:|---:|---:|---:|
| 128 | 0% | 1.824 / 13.713 ms | 0.0072 / 0.0097 ms | 253.38x | 8,461 / 785,576 | 0 / 0 |
| 128 | 1% | 3.197 / 14.696 ms | 0.0313 / 0.1154 ms | 102.14x | 16,930 / 1,571,976 | 147 / 14,916 |
| 128 | 10% | 2.973 / 8.259 ms | 0.1218 / 0.2134 ms | 24.41x | 16,930 / 1,571,976 | 877 / 82,990 |
| 128 | 100% | 2.081 / 6.215 ms | 1.699 / 3.316 ms | 1.22x | 8,470 / 786,440 | 8,470 / 786,440 |
| 512 | 0% | 11.130 / 24.645 ms | 0.0309 / 0.1196 ms | 360.20x | 33,841 / 3,142,280 | 0 / 0 |
| 512 | 1% | 13.237 / 21.374 ms | 0.1464 / 0.3521 ms | 90.42x | 67,717 / 6,287,784 | 448 / 46,780 |
| 512 | 10% | 10.955 / 19.111 ms | 0.6819 / 1.1430 ms | 16.07x | 67,717 / 6,287,784 | 3,492 / 330,688 |
| 512 | 100% | 7.425 / 69.484 ms | 7.305 / 20.184 ms | 1.02x | 33,877 / 3,145,544 | 33,877 / 3,145,544 |
| 4,096 | 0% | 62.336 / 94.069 ms | 0.2492 / 0.6322 ms | 250.14x | 270,712 / 25,131,600 | 0 / 0 |
| 4,096 | 1% | 125.866 / 163.961 ms | 1.285 / 5.049 ms | 97.94x | 541,729 / 50,298,128 | 3,092 / 328,758 |
| 4,096 | 10% | 180.681 / 375.300 ms | 5.229 / 11.928 ms | 34.56x | 541,729 / 50,298,128 | 27,484 / 2,611,060 |
| 4,096 | 100% | 59.635 / 94.233 ms | 55.616 / 75.183 ms | 1.07x | 271,018 / 25,166,568 | 271,018 / 25,166,568 |

Sealing 128/512/4,096 already-evaluated poses costs respectively 0.011/0.040/0.262 ms median
versus 0.002/0.006/0.111 ms for moving the owned rows without handles. At 4,096 entities the
increment is 0.151 ms, 4,095 additional small allocations, and 65,536 additional allocated bytes.
That overhead is explicit and must later converge into the planned dense pose-page/arena owner; it
is accepted for this MVP slice because it removes 25-50 MiB publication churn and hundreds of
thousands of allocations on stable/partial frames without regressing the modeled full-update target
rebuild. The 128/512 current P95 values are noisy and are not filtered.

The first implementation review found one remaining structural RED after the pipeline/physics post
probe: `LevelSystem::record_animation_pose_snapshot` computes
`published.animation_poses().as_ref() == animation_poses.as_ref()` before either pointer-identity
fast path. Because `Arc<AnimationPoseOutput>::PartialEq` compares the pointed-to pose, an unchanged
frame still walks every entity and bone even when the outer snapshot is identical. The existing
post table therefore covers pipeline publication plus physics projection, not the complete
LevelSystem publication closure. Before this slice can be considered implemented, the Level owner
must treat the sealed outer `Arc` identity as the publication identity, remove semantic deep-map
comparison, lock that hard cut in the boundary guard, and rerun an isolated profile that includes
the Level admission decision. Restoring a content-equality compatibility path is explicitly
rejected because the pipeline already reuses the prior snapshot when semantic content is unchanged.
That RED is now resolved: `record_animation_pose_snapshot` compares the sealed outer snapshot with
`Arc::ptr_eq` before admission and no longer contains a semantic map-equality path.

The subsequent real-consumer audit found a third RED in temporal-history admission.
`ViewportFrameHistory::incompatibility_reason` compares the complete `FrameHistoryValidationKey`
every frame, while `FrameHistoryAnimationPoseValidationKey` derives `PartialEq`; the shared pose row
therefore still reaches `AnimationPoseOutput` equality. The accepted correction keeps value equality
for camera, mesh, lighting, and post-process fields, but gives each animation row explicit
`entity + skeleton + Arc::ptr_eq(pose)` equality. Comparing the visible ordered pose set remains the
necessary `O(visible poses)` operation; bone/name payload comparison is forbidden. A release probe
must measure freshly materialized history-row vectors that share pose handles, because reusing the
same outer animation snapshot would not model the actual render-extract/history path.
That RED is now resolved by an explicit row `PartialEq` implementation; the outer validation key
continues to compare all non-animation fields by value.

Production now defines `AnimationPoseHandle`, `AnimationPoseMap`, and `AnimationPoseSnapshot` in the
neutral animation contract. The evaluation pipeline seals changed owned rows exactly once, retains
unchanged handles on partial publication, returns an explicit full/partial change receipt, and
returns without allocation for an unchanged batch. `LevelSystem`, render extract, and frame-history
validation store the same handles. `SkeletalPoseTargets` applies the partial receipt entity by entity
and removes only disappeared rows; the old owned whole-map recording entry point is deleted rather
than retained as a compatibility path. Unit coverage locks unchanged-handle identity, changed-row
replacement, removal, and full/partial target behavior. The fresh Frameworks01 Scene/Animation
static batch is `14/15` in 40.692 seconds: the new sealed-publication guard is GREEN, while the sole
RED is the already-owned Scene component `AnimationParameterSet` hard cut and is not repaired here.

The first post-cut release harness measures the pipeline and physics publication operation in one
interval: compare, seal changed rows, rebuild or shallow-copy the ordered map, and project physics
targets. Unlike the RED harness's separately reported finalization row, this combined measurement is
the authority for that subpath only. It uses the same 64-bone population, two warmups, 11
alternating-order samples, and allocation counter. Source SHA-256 is
`c9bef7292e0a02ac56a3b5401dde92b73c36eda2df794728b0058f5ee8aed44a`, result SHA-256 is
`09e94013a4ee542bb99c462203a7f50bfd0a49a8899a555b4fc8b0b6a0661a84`, and executable SHA-256 is
`4649847ed7848a10d03e82d492ab07e4216c7b4834352bdc7c47a9d0cb5c5450`.

| Entities | Updated | Retired median / P95 | Sealed-row median / P95 | Median ratio | Retired allocs / bytes | Sealed-row allocs / bytes |
|---:|---:|---:|---:|---:|---:|---:|
| 128 | 0% | 1,496.9 / 1,684.7 us | 6.3 / 8.5 us | 237.60x | 8,461 / 785,576 | 0 / 0 |
| 128 | 1% | 2,339.8 / 4,685.5 us | 29.5 / 36.1 us | 79.32x | 16,930 / 1,571,976 | 149 / 15,644 |
| 128 | 10% | 2,541.7 / 4,610.9 us | 124.5 / 195.7 us | 20.42x | 16,930 / 1,571,976 | 877 / 82,990 |
| 128 | 100% | 1,373.1 / 1,686.5 us | 1,301.1 / 1,472.9 us | 1.06x | 8,470 / 786,440 | 8,611 / 799,080 |
| 512 | 0% | 8,612.9 / 13,194.8 us | 28.6 / 36.4 us | 301.15x | 33,841 / 3,142,280 | 0 / 0 |
| 512 | 1% | 10,905.8 / 14,582.2 us | 115.0 / 147.0 us | 94.83x | 67,717 / 6,287,784 | 448 / 46,780 |
| 512 | 10% | 10,173.2 / 133,655.0 us | 735.3 / 909.1 us | 13.84x | 67,717 / 6,287,784 | 3,492 / 330,688 |
| 512 | 100% | 5,412.0 / 8,571.8 us | 5,909.7 / 11,924.9 us | 0.92x | 33,877 / 3,145,544 | 34,439 / 3,204,392 |
| 4,096 | 0% | 61,729.0 / 139,811.3 us | 236.3 / 282.3 us | 261.23x | 270,712 / 25,131,600 | 0 / 0 |
| 4,096 | 1% | 83,535.9 / 112,380.9 us | 829.9 / 1,481.5 us | 100.66x | 541,729 / 50,298,128 | 3,133 / 338,542 |
| 4,096 | 10% | 82,914.0 / 168,650.1 us | 6,248.2 / 10,601.0 us | 13.27x | 541,729 / 50,298,128 | 27,484 / 2,611,060 |
| 4,096 | 100% | 63,724.4 / 97,722.0 us | 58,809.8 / 82,009.8 us | 1.08x | 271,018 / 25,166,568 | 275,491 / 25,634,952 |

Stable and 1% partial frames remove the structural whole-population payload clone: at 4,096
entities, unchanged publication is 261.23x faster with zero new allocations, while 1% publication
is 100.66x faster and reduces allocation bytes from 50,298,128 to 338,542. The hard cut does not
claim a universal win: 512-entity full replacement is 9.2% slower by median, and the 4,096 full
replacement adds 4,473 allocations / 468,384 bytes even though its median is 1.08x faster. Full-row
sealing remains the next dense pose-page/arena optimization target; it must be solved with the
planned instance-local pose owner rather than by restoring deep-owned frame APIs.

A second release harness isolates the Level snapshot-admission decision that the first post probe
missed. It profiles the production stable case (`same_outer`) and a deliberately rejected
compatibility case (`equal_rewrapped`) with 64 bones/entity, two warmups, 21 alternating-order
samples, separately calibrated old/new iteration counts, and no allocation inside either admission
operation. The stable case demonstrates that Rust's prior map/row equality reaches the pose payload
even when the outer `Arc` is shared; the new contract checks publication identity only. Source
SHA-256 is `3bbf66d0a9937f8462dddf6c8c2617970364ae50f6a5ac5686b8288f6b119007`, result SHA-256 is
`31d0cdeff9ec56e2b905835121481a8080b59d3fbb71a13c0dc2bc57d7254abe`, and executable SHA-256 is
`b2b949e6b40883afe8aa6a3f318e1fdf66349290d8ae78391f2cf4a5844efed4`.

| Entities | Scenario | Retired median / P95 | Pointer median / P95 | Median ratio |
|---:|---|---:|---:|---:|
| 128 | same outer | 180.219 / 213.664 us | 6.949 / 9.807 ns | 25,934.48x |
| 128 | equal rewrapped | 183.140 / 544.390 us | 6.274 / 7.364 ns | 29,190.31x |
| 512 | same outer | 1,859.444 / 2,345.156 us | 6.530 / 14.704 ns | 284,754.09x |
| 512 | equal rewrapped | 2,514.715 / 4,383.465 us | 6.816 / 12.259 ns | 368,915.87x |
| 4,096 | same outer | 17,114.562 / 20,924.702 us | 6.294 / 7.928 ns | 2,719,186.84x |
| 4,096 | equal rewrapped | 20,904.300 / 26,463.250 us | 6.343 / 7.946 ns | 3,295,648.75x |

These two harnesses intentionally remain separate; medians from independently sampled subpaths are
not added into a synthetic whole-frame ratio. Together they show that the pipeline/physics deep-copy
churn and the Level deep-equality scan are both removed from the stable publication path.

The history release harness constructs two fresh ordered row vectors per frame shape while sharing
the same sealed pose handles, matching render-extract/history materialization. It uses 64 bones per
pose, two warmups, 21 alternating-order samples, and separately calibrated iteration counts. Source
SHA-256 is `565d98cdee7e7a78dc7a5d0d960a8c9f6fa4c3764a04e62f0e2044fd5aafa9e5`, result SHA-256 is
`527014e8b415d0f6e8e09fb8a98144c02145152d2e904f608c423eedde6a5e6b`, and executable SHA-256 is
`cc68d18ff7cdbaefe02e0762c318cb7085087d1294437d303a5eb81a21ce235c`.

| Visible pose rows | Retired median / P95 | Identity-row median / P95 | Median ratio |
|---:|---:|---:|---:|
| 128 | 148.989 / 297.968 us | 0.200 / 0.381 us | 745.46x |
| 512 | 1,588.954 / 2,456.516 us | 0.756 / 1.112 us | 2,101.79x |
| 4,096 | 13,492.200 / 15,848.380 us | 5.810 / 7.713 us | 2,322.24x |

The remaining `O(visible poses)` comparison is intentional: history must reject an entity,
skeleton, row-order, or sealed-pose identity change. A future generation/fingerprint may reduce
that scan only if it is produced by the same visible-pose projection authority; hashing bone payloads
or comparing pose values would recreate the retired complexity.

`animation_evaluation_pipeline.rs` is 929 lines after this slice, inside the repository's 900-1100
split-planning band. The current coordinator scope is immutable and does not admit a new sibling;
the smallest follow-up boundary is `evaluation/pipeline/pose_publication.rs`, owning the change
receipt, compare/seal algorithm, and its unit tests. The current file remains a single pipeline-state
orchestrator and is not allowed to gain another publication responsibility before that scope
rotation.

Evidence remains under
`F:/zircon-profiles/animation-blend-space-20260826/pose-publication`; no artifact is on C. Managed
Cargo request `8484973835bc46a2a5066129b6eac35b` failed before starting a job because the coordinator
detected the foreign unmanaged directory `E:/ZirconBuilds/mvp-resource-management-projects`; this
session does not remove or take ownership of it. These results prove the isolated ownership and
publication algorithm plus static contract only. They do not prove product frame time, power,
dense-pose closure, GPU deformation, or Unreal/Fyrox parity, and they do not promote M1.
