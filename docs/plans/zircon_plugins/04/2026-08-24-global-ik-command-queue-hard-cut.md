# Plugins04 M5 Global IK Command Queue Hard Cut

> Parent plan: [`../04-animation.md`](../04-animation.md)
> Session: `plugins04-global-ik-queue-hard-cut-r1-20260824`

## Status

`source_implemented / validation_pending`

This record supersedes the current-state interpretation of the queue-backed M5-T1 rows in
[`2026-07-11-animation-m5-output-records.md`](2026-07-11-animation-m5-output-records.md) and
[`2026-08-01-current-state-and-performance-handoffs.md`](2026-08-01-current-state-and-performance-handoffs.md).
Those files remain historical evidence; their queue-driven tests do not prove a production IK
producer or an engine-scale ownership model.

No accepted milestone, service commit, performance win, power result, or WeCom closeout is claimed
until the managed validation and independent review gates complete.

## Architecture Review

The pre-cut current-source inventory found zero production callers of
`AnimationManager::queue_ik_command`. Every producer lived in unit or integration tests. Despite
that, both the Runtime fallback manager and the Animation plugin manager owned an independent
`Mutex<HashMap<WorldHandle, WorldIkCommandQueue>>`, a 4,096-command bound, replacement-epoch state,
and stable partition/drain behavior. `animation.evaluate` acquired and drained that process-wide
inbox on every tick before publishing the final pose.

This is the wrong owner boundary. A skeletal-control result depends on the input pose, compiled
skeleton targets, graph parameters, and evaluation-instance state. A global manager inbox cannot
express that lifetime without duplicating world retirement, replacement epochs, ordering,
diagnostics, and deferred-admission policy outside the graph instance.

The Unreal primary references confirm graph/evaluation-local ownership:

- `dev/UnrealEngine/Engine/Source/Runtime/AnimGraphRuntime/Public/BoneControllers/AnimNode_SkeletalControlBase.h`
  and its private implementation evaluate an input pose and reuse node-owned bone-transform output
  before blending it into the component-space pose;
- `AnimNode_TwoBoneIK` and `AnimNode_LookAt` derive from that node contract, retain their inputs and
  cached bone references with the node, and solve during graph evaluation;
- `dev/UnrealEngine/Engine/Source/Runtime/AnimationCore/Private/TwoBoneIK.cpp` remains a reusable
  mathematical primitive beneath the node owner.

Zircon therefore keeps the pure `TwoBoneIkJob` and `LookAtJob` solvers, but does not preserve a
compatibility queue while the graph-local product node is absent.

## Implemented Hard Cut

- Removed `AnimationIkCommand`, its queue error type, and all queue/drain methods from the neutral
  framework manager contract.
- Removed both duplicated per-World queue maps and replacement-epoch state owners from Runtime and
  plugin default managers.
- Removed the queue-backed postprocess adapter, execution diagnostics, IK diagnostic event/schema,
  and test-only command producers.
- Removed the unconditional tick drain/partition branch. Physics `SimulatedPoseFeed` blending and
  final pose publication remain in their existing order.
- Retained the pure TwoBone and LookAt solver APIs and their three mathematical contracts.
- Added `animation_runtime_has_no_process_wide_ik_inbox` to lock the neutral manager, plugin
  manager, and production tick against restoring the queue/drain surface.
- Updated module documentation so the queue is no longer presented as a current capability.

M5-T2 clip-event registration remains implemented. M5-T1 product integration is reopened and is
not complete until a compiled animation graph/evaluation node owns IK inputs, dense target slots,
scratch storage, diagnostics, and per-instance state.

## Performance Scope

This hard cut removes an unconditional lock/map lookup and queue partition path from the animation
tick, but no latency, throughput, RSS, or power improvement is claimed from source inspection. The
removed benchmark compared two membership algorithms inside a test-only producer model and cannot
justify retaining that model.

Before graph-local IK implementation is optimized, profiling must capture at least:

- Animation Insights/ETW or equivalent sampled CPU time for graph evaluation, target resolution,
  model-pose construction, and pose publication;
- p50/p95/p99 frame cost for 1, 100, and 1,000 active rigs, with 0/1/multiple IK nodes per rig;
- allocation counts, lock waits, cache misses, and model-pose rebuild counts per evaluated rig;
- changed-versus-stable graph instances and active-bone counts;
- CPU package energy or platform power telemetry under the same fixed workload, compared only with
  a documented reference-engine scenario.

The structural target is one prepared skeleton target table and at most one model-pose workspace per
evaluated rig, with work proportional to active nodes and affected bones. Measurements, not this
hard cut, must decide whether further caching or batching is warranted.

## Next Slice Architecture Gate

The current graph evaluator cannot own skeletal-control nodes yet. Its compiled representation ends
at `CompiledAnimationGraphEvaluation { clips: Vec<_> }`: evaluation recursively flattens
Clip/Blend/Additive/Mask nodes into clip instances, and the pipeline then samples one complete
`AnimationPoseOutput` per instance before blending those wide outputs. The pose pool is used only
inside an individual clip sample and is released before graph blending, so there is no graph-instance
pose arena or component/model-space workspace for a skeletal-control node to mutate.

This is also an algorithmic blocker, not only a missing node type:

- compilation detects shared DAG nodes with a visited set, but runtime `collect_clips` does not
  memoize them; a graph made from repeated diamond joins expands by path count rather than reachable
  node count. Depth 20 deterministically produces 1,048,576 clip instances from 21 reachable source
  nodes;
- every expanded clip can materialize a full pose containing per-bone `String` names before the
  graph-wide blend;
- an active graph player clones its `BTreeMap<String, AnimationParameterValue>` into the scene-scan
  request, then the 256-entry frame cache clones it again and performs linear equality searches.

Adding TwoBone/LookAt variants to this recursive flattener, or restoring a post-blend queue, would
preserve the wrong execution owner. The next implementation slice must first introduce a compiled
pose program and per-entity graph instance:

1. Characterize shared-subgraph execution, parameter copies, pose allocations, and output parity
   with failing contract tests and managed profiling fixtures.
2. Compile the reachable DAG into topological pose operations so every reachable node executes at
   most once; compute pose-value liveness and the required scratch-slot count during compilation.
3. Key runtime instances by entity, graph revision, skeleton revision, and replacement epoch. Each
   instance owns dense parameter storage, node state, reusable pose slots, and one reusable
   component/model-space workspace; no manager-global command inbox is permitted.
4. Compile skeletal-control target identities and parent-chain metadata to dense skeleton slots.
   Runtime evaluation must consume the upstream pose and emit the downstream pose inside the same
   program, with an explicit target-space contract and bounded diagnostics.
5. Integrate TwoBone and LookAt only after the executor contract is green, then publish one final
   local pose through the existing presentation path.

Unreal remains the primary architecture reference: `FAnimNode_SkeletalControlBase` owns a
`ComponentPose` input, cached bone references, node alpha/state, component-space evaluation, and a
bounded `OutBoneTransforms` result that is blended back into the same pose context. TwoBoneIK and
LookAt specialize that node contract while the mathematical solver remains in AnimationCore. Godot
provides a useful cross-check: its IK modifiers are attached to a concrete `Skeleton3D`, cache joint
solver state, and update that skeleton directly rather than submitting process-global commands.
The local Bevy tree has no equivalent built-in pose-graph IK owner and does not override the Unreal
decision.

### Required Profiling Before Optimization

No executor optimization begins until a managed Windows baseline records the following on an
approved D/E/F target:

- ETW/WPR sampled CPU stacks and allocator activity for graph traversal, clip sampling, pose
  materialization, blend, component/model-space construction, and publication;
- exact counters for reachable-node visits, expanded clip instances, full-pose allocations,
  parameter bytes cloned/compared, pose-pool misses, model-pose rebuilds, and skeletal-control
  affected-bone visits;
- p50/p95/p99 wall time and steady-state RSS for diamond depths 1/5/10/15/20 and for 1/100/1,000
  active rigs with zero, one, and multiple skeletal-control nodes;
- fixed-workload CPU package energy or platform power telemetry only after the test scenario and
  reference-engine comparison are documented.

The first acceptance bound is structural: work must be `O(reachable nodes + sampled tracks +
affected bones)`, each reachable node executes at most once, each distinct reachable clip samples at
most once per instance/time, and warm evaluation performs zero graph-shape or bone-name allocation.
Latency, power, and reference-engine parity remain unclaimed until the measured post-change run is
compared with the frozen baseline.

## Validation

Current exact-source evidence:

1. `rustfmt +1.94.1 --edition 2021 --check` passed for all 11 retained owned Rust files.
2. `git diff --check` passed for the complete tracked ownership set.
3. The retired production-symbol inventory returned zero matches for the command DTOs, queue/drain
   methods, postprocess adapter, execution diagnostics, and IK event constants.
4. The current module-document inventory returned zero references to the five deleted module paths.

Managed Windows Cargo has not reached the owned code:

- job `bdeb93ea01094ea1aeb6412cdc2aa2f9` targeted
  `zircon_plugin_animation_runtime::animation_ik_contract` on
  `F:\cargo-targets\zircon-engine\ephemeral\test\bdeb93ea01094ea1aeb6412cdc2aa2f9`, but Cargo stopped
  while parsing the plugin workspace because `zircon_plugins/gltf_importer/runtime/Cargo.toml`
  inherits `gltf` and the plugin workspace does not currently define that dependency;
- job `5fa40b7b99324fb5aa68e47ac76279e2` targeted the Runtime fallback-manager poison-recovery test on
  `F:\cargo-targets\zircon-engine\ephemeral\test\5fa40b7b99324fb5aa68e47ac76279e2`, but compilation
  stopped in foreign `zr_rhi_wgpu` production diagnostics with six errors: one missing `tracker`,
  two missing `PipelineStatisticsScope::query_index` methods, one missing
  `pipeline_statistics_result_value_count` method, and two unsatisfied
  `DiagnosticBatchCompletion: Default` bounds. No owned Animation source appeared in the errors and
  the target test did not run. This job predates the final source guard and is diagnostic evidence,
  not exact-fingerprint acceptance.

Remaining gates:

1. rerun managed Windows Cargo after the plugin-manifest and RHI blockers clear, using the frozen
   ownership fingerprint and an approved D/E/F target;
2. run independent review of the hard-cut contract and that exact fingerprint;
3. create the coordinator service commit and send the quantified WeCom report only after both gates
   pass.
