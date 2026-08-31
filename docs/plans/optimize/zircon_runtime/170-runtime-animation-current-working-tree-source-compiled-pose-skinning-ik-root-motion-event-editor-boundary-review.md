---
title: Runtime Animation 当前工作树 Source、Compiled、Pose、Skinning、IK、Root Motion、Event 与 Editor Boundary 复审及重构计划
category: zircon_runtime
report_id: Runtime170
review_date: 2026-08-30
baseline_head: working-tree
baseline_epoch: 2026-08-30
verification_head: working-tree
verification_epoch: 2026-08-30
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/99zl-runtime-animation-skeleton-clip-pose-graph-state-machine-layer-mask-blend-ik-root-motion-event-extract-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
related_code:
  - zircon_runtime/src/core/framework/animation
  - zircon_runtime/src/animation
  - zircon_runtime/src/asset/importer/ingest/gltf_animation_subassets.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/skinning.rs
  - zircon_runtime/src/graphics/scene/gpu_scene/skinned_palette_arena.rs
  - zircon_plugins/animation/runtime
  - zircon_plugins/animation_graph
  - zircon_app/src/entry
  - zircon_plugins/first_party_runtime_catalog
plan_sources:
  - docs/plans/optimize/zircon_runtime/99zl-runtime-animation-skeleton-clip-pose-graph-state-machine-layer-mask-blend-ik-root-motion-event-extract-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation/AnimInstance.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation/AnimNodeBase.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation/AnimSequence.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/Animation/AnimInstanceProxy.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/BoneContainer.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/GPUSkinVertexFactory.h
  - dev/bevy/crates/bevy_animation/src/lib.rs
  - dev/bevy/crates/bevy_animation/src/graph.rs
  - dev/bevy/crates/bevy_animation/src/transition.rs
  - dev/bevy/crates/bevy_gltf/src/loader/mod.rs
  - dev/Fyrox/fyrox-animation/src/pose.rs
  - dev/Fyrox/fyrox-animation/src/track.rs
  - dev/Fyrox/fyrox-animation/src/machine
  - dev/godot/scene/animation/animation_mixer.cpp
  - dev/godot/scene/animation/animation_tree.cpp
  - dev/godot/scene/3d/skeleton_3d.h
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/GPUDriven
---

# 结论

当前工作树已经有真实的动画骨架：typed clip/skeleton/graph/state-machine asset、版本化 binary codec、target table、compiled clip cache、graph/state-machine evaluation pipeline、mask/blend-space、bounded clip-event heap、Scene 到 `SkeletalPoseTargets` 的 handoff，以及 editor document/revision/undo/last-good 的局部底座。这些不能被误判为“没有动画”。

但它还不是工程级动画系统。最大的风险不是缺少一个节点，而是同一个角色在不同阶段拥有不同的身份和数学：builtin `DefaultAnimationManager` 与 `zircon_plugins/animation/runtime` 都能解释 clip/graph/state machine；`AnimationCompileProduct` 是 source-only 诊断，插件又维护另一套 compiled evaluator；glTF 导出的 inverse-bind matrix 进入独立 Data asset，却没有进入 skeleton 或 renderer 的 canonical skin contract；runtime 与 renderer 各自用 `posed_world * bind_world.inverse()` 重新推导 palette。编辑器的 last-good 是内存对象，不是可寻址、可校验、可回滚的 cooked artifact。IK 现在只是两个独立数学 job，原先的诊断/执行/后处理文件在当前工作树被删除，生产 evaluation 没有接入。

因此本报告只做 review，不修改生产代码。实现前必须先完成唯一 source/compiled/presentation/render identity、真实 skin contract、可取消的 evaluation scheduler 和跨域 writer arbitration。

## 当前工作树证据

| 选择集 | files / lines / bytes / test attrs / ignored | fingerprint |
|---|---:|---|
| Runtime framework + builtin animation (`zircon_runtime/src/animation`、`zircon_runtime/src/core/framework/animation`) | **80 / 10,058 / 372,010 / 77 / 12** | `947af6f2f3279109ba438ed81e08911e9c3f695761099fe2ed4579d860272ae7` |
| First-party animation runtime/editor + animation graph | **166 / 18,901 / 727,120 / 165 / 1** | `81ed726fe992f714786f9fb6d40d701385ae505b50c723e1d937f66e61ebf043` |
| Editor animation document/session/timeline/curve | **47 / 4,779 / 181,269 / 23 / 0** | `2982876f5b0358bd150a25df17534d08582267effb51f85f3ccec7177eec7f9a` |
| Import/render/diagnostic boundary evidence | **198 / 45,727 / 1,808,844 / 501 / 9** | `3661f09b93f1ff25a841da3ad0190810b10abc26e48a32f6bb082b440bd06a1f` |
| Reference selection reused from Runtime137 | **32 files / 26,143 lines** | `25711706f628d09c8955b681c4435048f6d6bb78de2e9d73493309892ad2a874` (reference freeze from parent report) |

The animation directories are materially dirty: plugin evaluator, graph cache, state-machine cache, editor sessions, runtime framework assets, skinning code and IK tests are modified; `ik/diagnostic.rs`, `ik/execution_error.rs`, `ik/postprocess.rs`, framework IK command files and a blend-space point file are deleted. The report treats this physical working tree as the review baseline and does not revert it.

## Findings

### RT-AN-01 — Two runtime authorities and two compiler products (P0, Open)

`zircon_runtime/src/animation/module.rs` registers `DefaultAnimationManager` and a second `AnimationManager` service, while the first-party plugin has its own manager/module and `AnimationEvaluationPipeline`. The builtin manager directly samples `AnimationClipAsset` in `manager/pose.rs`, and the plugin evaluator compiles a target table and `CompiledAnimationClip`. `core/framework/animation/compiler/product.rs` deliberately emits source-only `AnimationCompileProduct`; plugin compile/evaluate types are separate and are not a versioned artifact ABI. Which evaluator is selected depends on composition rather than asset identity. A graph can therefore validate successfully yet have different runtime semantics, cache keys and failure policy in another target mode.

Refactor to one `AnimationSourceAsset` schema, one `AnimationCompiler` producing a versioned `AnimationArtifact` (skeleton binding, channel storage, graph/state-machine bytecode, diagnostics and compiler fingerprint), and one `AnimationEvaluator` SPI selected by the runtime catalog. Keep builtin code only as the neutral contract or delete it in a hard cutover. Every consumer must receive `(asset_id, source_revision, artifact_generation, skeleton_binding_generation)`.

### RT-AN-02 — Inverse-bind and joint mapping are not canonical (P0, Open)

`gltf_animation_subassets.rs:100-158,564-582` reads inverse-bind matrices and exports them as `Data` subassets, but `AnimationSkeletonAsset` contains only names, parent indices and local bind transforms. `zircon_plugins/animation/runtime/src/gpu_skinning/palette.rs:27-64` and `zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/skinning.rs:15-31` both recompute `posed_world * bind_world.inverse()`. This discards authored inverse-bind matrices, assumes skeleton order equals mesh joint order, and duplicates palette math. The renderer still has its own `SkinnedMeshJointPalette`, so plugin readiness does not prove the draw path uses the same result.

Introduce a canonical `SkeletonBindingArtifact` containing joint stable id, skeleton index, mesh joint index, authored inverse-bind matrix, bind-space convention and validated parent order. Import must fail or produce an explicit repair diagnostic when IBM count/order or mesh remap is incomplete. Runtime computes one palette packet per pose generation; renderer consumes the packet without recomputing bind transforms. Add identity-pose, non-uniform-scale, reordered-joint and IBM mismatch golden tests.

### RT-AN-03 — Channel semantics are only partially production grade (P1, Open)

The current clip schema supports Step/Linear/Hermite, but plugin `evaluation/clip_evaluator/hermite.rs` uses slerp for quaternion keys and ignores quaternion tangents. The builtin and plugin samplers are separate. glTF morph-target channels are explicitly rejected at `gltf_animation_subassets.rs:471-475`; cubic-spline data is accepted but only as generic tangents. No canonical compression format, quantization error budget, additive reference pose or per-channel constant/default policy is part of the artifact. Reverse playback, direction flips, seek discontinuities and exact end-point policy are not represented by the event or pose contract.

Move interpolation and compression into the compiler artifact: typed scalar/vector/quaternion codecs, quaternion continuity/sign fixing, cubic-spline validation, additive reference, channel defaults, quantization error and deterministic time domain. Add forward/reverse/loop/seek tests and a glTF morph-curve lane rather than rejecting it as a bone track.

### RT-AN-04 — Evaluation pipeline is staged but not a scheduler (P0, Open)

`zircon_plugins/animation/runtime/src/evaluation/pipeline/tick.rs:29-505` performs scene scan, resource loading, clip/graph/state-machine evaluation, event admission, sequence application, pose writeback and diagnostics in one frame function. It calls `load_animation_sequence_asset` and binds resource subscriptions inside the frame path. `AnimationEvaluationPipeline` has a frame graph cache limit of 256 and a default direct clip pool size of 4, but no per-world budget, deadline, cancellation token, priority, back-pressure, shutdown or worker completion receipt. Deferred entities are restored from previous maps, which is a useful rollback primitive but not an execution contract.

Split into immutable `AnimationEvaluationRequest`, dependency-resolved work graph, bounded scheduler, worker completion packet and commit transaction. Scheduling must be per World/level and carry tick, world generation, source revision and cancellation. A job that misses its budget must return `Deferred` or `Failed` with a typed receipt; it must never silently sample an old pose or leave a partially updated cache. Add scale tests for 1/64/1,024/10,000 actors, cancellation and replacement races.

### RT-AN-05 — Pose output writes scene transforms without writer ownership (P0, Open)

`tick.rs:431-475` publishes every bone as `SkeletalPoseTarget` with `normalized_weight: 1.0`; `pose_apply.rs` writes animation results directly to scene nodes. There is no claim/priority/range arbitration between animation, physics/ragdoll, procedural IK, gameplay, network correction and cinematic evaluation. Root motion is not a typed output, so character movement cannot distinguish extracted root delta from bone pose. The `SimulatedPoseFeed` blend is a special-case side channel rather than a general writer graph.

Add `PoseEvaluationPacket` with local pose, component masks, additive/base layer, root-motion delta, curve/morph values, event journal and writer claims. Commit through a per-entity `PoseWriterArbiter` with deterministic priority and transaction scope. Physics and network should consume/approve root motion through explicit tickets; no subsystem may write a Transform as an implicit fallback.

### RT-AN-06 — IK is math-only and currently losing its production integration (P0, Open)

The plugin exports `LookAtJob` and `TwoBoneIkJob` from `ik/mod.rs`, but there is no evaluator stage that schedules them against a compiled pose, validates target generations or publishes diagnostics. `ik/diagnostic.rs`, `execution_error.rs` and `postprocess.rs` are deleted in the current tree, as are framework IK command files. Search shows the remaining jobs are referenced only by exports/tests, not by the animation tick. This is a direct regression from a partially integrated design, not a missing optional feature.

Restore IK as a typed post-process graph: authoring node -> compiled solver descriptor -> bounded job -> pose delta -> arbiter commit -> diagnostic/event receipt. Include chain topology, limits, pole/target handles, weight, failure policy, physics handoff and deterministic iteration budget. Keep math jobs pure, but make the runtime owner explicit and add cancellation/stale-generation tests.

### RT-AN-07 — Events are bounded but not a complete notify contract (P1, Open)

`zircon_runtime/src/animation/clip_event.rs` has a useful heap, byte/event/span limits and resumable cursor, but it rejects `to_time <= from_time`, so reverse playback and direction changes cannot be expressed. Cursor ordering uses event text and track index; `AnimationEventTrackAsset` has no stable event id. Events are published through the animation event queue without a typed gameplay/physics/audio/cinematic acknowledgement or exactly-once policy across loop, seek and deferred frames.

Compile event tracks with stable ids, source order, notify class, payload schema and phase. Define a `PlaybackCursor` containing direction, loop occurrence, discontinuity and generation. Emit an event journal with dedupe key and terminal delivery receipt; let consumers ack or explicitly drop by policy. Test reverse, same-time ordering, seek, loop boundary, cancellation and queue overflow.

### RT-AN-08 — State machine cache does not provide full animation-instance semantics (P1, Open)

The pipeline now retains state-machine instances, interrupted transition sources and nested machine state, which is useful. However, cache keys and updates remain runtime maps around an asset id/entity rather than a stable per-player instance with sync group, slot, montage, blend profile, transition interruption and terminal state. `apply_active_state_update` mutates component parameters after evaluation, while transition/event diagnostics are separate vectors. There is no inertialization, sync-marker phase matching, montage branch/section or network-replicable playback state.

Define `AnimationInstanceId` and `AnimationInstanceSnapshot`, compile transition/marker/slot metadata into the artifact, and make all state writes part of the same commit. Add sync groups, marker matching, interruption policy, inertialization and deterministic replication serialization before exposing montage or motion-matching authoring.

### RT-AN-09 — GPU skinning is an admission test plus CPU path, not a renderer contract (P0, Open)

`gpu_skinning/decision.rs` returns `Gpu` only when readiness is true and joint count is at most a fixed 256; otherwise it emits a string CPU fallback. The renderer path still calls `to_morphed_model_primitive` and can CPU-skin vertices before preparing the shader source. Palette storage has a fixed ABI array and no device/profile-specific limit, residency ticket, current/previous pose generation or upload completion. Animation GPU readiness is therefore decoupled from actual draw submission, motion vectors and device loss.

Replace boolean readiness with a device-qualified `SkinningPipelineProfile` and a `SkinningPaletteHandle` carrying generation, joint remap, current/previous pose and upload ticket. Select CPU/GPU/compute deformation through the render scheduler, not a string diagnostic. Make morph, skin and velocity one deformation packet and add palette budget/eviction/device-loss gates.

### RT-AN-10 — Importer and asset lifecycle do not produce a complete animation build graph (P1, Open)

glTF importer creates labeled clip/skeleton/data entries and dependency edges, but skeleton, IBM data, mesh skin, morph curves and clip artifact are separate products with no compiler fingerprint or atomic build receipt. `.zranim` loading is a suffix dispatch to typed bytes, not a source/import/cook/artifact pipeline. External buffer/image dependencies are not represented in the animation clip itself. A reload can therefore make a clip visible before its skeleton binding or GPU-ready artifact exists.

Make animation import a content-addressed build graph: source document -> normalized skeleton/skin binding -> clip curves/events/morph -> graph/state-machine source -> compiled artifacts -> render/physics adapters. Publish only an atomic generation snapshot with dependency closure, diagnostics and last-good policy. Use the same identity in Editor, runtime cache, package and network replay.

### RT-AN-11 — Root motion, retargeting, morph curves and cinematic/ability consumers are absent (P1, Open)

The current source schema has no root-motion extraction settings, retarget profile, curve/morph channel owner, montage/cinematic section, ability notify or motion-matching database. Editor workbench assets for montage, control rig, retarget, motion matching and compression are presentation fixtures and do not correspond to runtime types. Animation events cannot carry a cross-domain request/ack contract.

Add these only after the canonical artifact and writer arbiter exist: retarget profile with skeleton mapping and scale policy; root-motion extraction and warping packet; morph/curve stream; montage/slot/section/notify artifact; motion database/index; and explicit Gameplay/Physics/Audio/Cinematic adapters. Do not add UI fields that have no runtime and cook owner.

## Refactor order

1. **M0 identity hard cut:** choose the single runtime provider, remove duplicate manager/evaluator registration, define source revision/artifact generation/skeleton binding generation and make catalog/App selection fail closed.
2. **M1 skin contract:** import authored IBM and mesh-joint remap into a canonical binding artifact; delete renderer-side palette recomputation; pass identity-pose and non-uniform-scale golden tests.
3. **M2 compiler/evaluator:** unify channel interpolation, quaternion continuity, compression, graph/state-machine bytecode, event ids and diagnostics; install artifacts atomically with last-good generation.
4. **M3 scheduler/commit:** introduce bounded per-World evaluation jobs, cancellation/deadline/receipt, pose writer arbitration, root-motion packet and deterministic state-instance commit.
5. **M4 post-process/render:** integrate IK, morph/curve, CPU/GPU/compute deformation, current/previous palette upload and device-qualified failure paths.
6. **M5 product features:** only then implement montage, retarget, sync markers, inertialization, motion matching, cinematic and gameplay/physics/network adapters plus editor previews.

## Qualification gates

| Gate | Required evidence | Current |
|---|---|---|
| RT-AN-1 one provider | Client/Server/EditorHost resolve the same animation provider and artifact ABI | Fail |
| RT-AN-2 skin identity | Authored IBM + mesh remap survives import/cook/load and identity pose is exact | Fail |
| RT-AN-3 deterministic sampling | Forward/reverse/loop/seek, quaternion and compressed curves agree across targets | Fail |
| RT-AN-4 bounded evaluation | Per-World budget, cancellation, stale generation and terminal receipt under actor scale | Fail |
| RT-AN-5 writer ownership | Animation/physics/IK/network/cinematic cannot silently overwrite each other | Fail |
| RT-AN-6 event delivery | Stable ids, reverse/loop/seek semantics, dedupe and consumer acknowledgement | Fail |
| RT-AN-7 IK production path | Compiled node, scheduled solver, pose delta, diagnostics and failure policy | Fail |
| RT-AN-8 render deformation | One palette/deformation packet drives GPU, CPU fallback, morph and velocity | Fail |
| RT-AN-9 artifact closure | Import/cook/package/reload publishes atomic dependency-complete generation | Partial |
| RT-AN-10 scale/soak | 10k actors, long loop, device loss, replacement and memory budget evidence | Fail |

Current total: **9 Fail / 1 Partial / 0 Pass**. Cargo, editor, device, scale and soak validation were intentionally not run in this review-only pass.

## Reference comparison

- Unreal's `AnimInstance`/`AnimInstanceProxy`/`AnimNodeBase` split makes per-instance state, worker evaluation and game-thread commit explicit; Zircon's pipeline has stages but no public instance/proxy/receipt boundary.
- Unreal `BoneContainer` and GPU skin vertex factory preserve compact bone-container/remap and render-facing palette contracts; Zircon exports IBM as detached JSON and recomputes bind math twice.
- Bevy's animation graph/transition and glTF loader show asset-driven graph topology plus transition state; Fyrox's pose/track/machine files show a typed track and signal model; Godot's mixer/tree separates playback ownership from skeleton application. Zircon currently mixes source validation, plugin evaluation and direct scene writes.
- Unity Graphics references provide the correct direction for source-versus-baked curves and GPU deformation capability profiles, but the local checkout is not a complete Mecanim implementation and must not be treated as one.

The previous Runtime137 report remains useful for historical findings, but this report supersedes its currentness because the working tree has deleted IK integration files and substantially rewritten graph/state-machine/cache/editor paths.
