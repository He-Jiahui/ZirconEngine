---
title: Editor Animation Sequence、Clip、Channel Binding、Interpolation、Compression、Event、Root Motion、Sync、Preview、Compiler 与 Product Integration 当前源码复核
category: zircon_editor
report_id: Editor198
review_date: 2026-08-28
baseline_head: 681588f7a1cbfaae3147e8b93e1be6705d810f21
related_code:
  - zircon_runtime/src/core/framework/animation/asset
  - zircon_runtime/src/core/framework/animation/compiler
  - zircon_runtime/src/core/framework/animation/timeline.rs
  - zircon_runtime/src/core/framework/animation/clip_event_sampling.rs
  - zircon_runtime/src/core/framework/animation/event.rs
  - zircon_runtime/src/animation/clip_event.rs
  - zircon_runtime/src/animation/sequence
  - zircon_runtime/src/scene/components/scene/animation.rs
  - zircon_plugins/animation/runtime/src/channel_sampling
  - zircon_plugins/animation/runtime/src/evaluation/clip_evaluator
  - zircon_plugins/animation/runtime/src/evaluation/compiled_animation_clip
  - zircon_plugins/animation/runtime/src/evaluation/pipeline
  - zircon_editor/src/core/editing/animation_document
  - zircon_editor/src/ui/animation_editor
  - zircon_editor/src/ui/host/animation_editor_sessions
  - zircon_runtime/src/asset/importer/ingest/gltf_animation_subassets.rs
tests:
  - zircon_runtime/src/core/framework/animation/compiler/sequence/tests.rs
  - zircon_runtime/src/animation/sequence/tests.rs
  - zircon_runtime/src/animation/clip_event.rs
  - zircon_runtime/src/asset/tests/assets/animation.rs
  - zircon_runtime/src/asset/tests/assets/gltf_importer
  - zircon_plugins/animation/runtime/src/evaluation/clip_evaluator
  - zircon_plugins/animation/runtime/tests
  - zircon_editor/src/core/editing/animation_document/tests.rs
  - zircon_editor/src/ui/animation_editor/session/tests.rs
  - zircon_editor/src/tests/editor_event/animation_runtime/sequence.rs
  - zircon_editor/src/tests/host/animation_editor.rs
  - zircon_editor/src/tests/host/binding_dispatch/animation.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_editor/14-animation-sequence-graph-state-machine-timeline-curve-preview-compiler-authoring-review.md
  - docs/plans/optimize/zircon_editor/32-model-mesh-skeleton-geometry-import-lod-collision-retarget-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/45-cinematic-sequencer-shot-track-binding-take-recorder-movie-render-queue-authoring-review.md
  - docs/plans/optimize/zircon_editor/63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/69-editor-scene-viewport-realtime-update-preview-simulation-time-domain-pause-step-animation-particle-physics-audio-visibility-throttling-invalidation-performance-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/75-editor-animation-timeline-dope-sheet-curve-editor-track-key-selection-transport-scrub-snap-clipboard-transaction-virtualization-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/76-editor-animation-graph-state-machine-node-edge-parameter-condition-compiler-runtime-transition-blend-preview-transaction-persistence-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/184-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/190-editor-scene-viewport-realtime-update-preview-simulation-time-domain-pause-step-animation-particle-physics-audio-visibility-throttling-invalidation-performance-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/196-editor-animation-timeline-dope-sheet-curve-editor-track-key-selection-transport-scrub-snap-clipboard-transaction-virtualization-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/197-editor-animation-graph-state-machine-node-edge-parameter-condition-compiler-runtime-transition-blend-preview-transaction-persistence-product-integration-current-source-review.md
  - docs/plans/mvp/00-current-source-baseline-recovery.md
  - docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation/AnimSequenceBase.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation/AnimSequence.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation/AnimMontage.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimSequence.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimSequenceBase.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimMontage.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Animation/AnimationCompressionDerivedData.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/Animation/AnimSync.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/Animation/AnimNotifyQueue.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Animation/AnimData/IAnimationDataModel.h
  - dev/godot/scene/resources/animation.h
  - dev/godot/scene/resources/animation.cpp
  - dev/godot/scene/animation/animation_mixer.cpp
  - dev/godot/scene/animation/animation_player.cpp
  - dev/godot/editor/animation/animation_track_editor.h
  - dev/godot/editor/animation/animation_track_editor.cpp
  - dev/godot/editor/animation/animation_bezier_editor.h
  - dev/godot/editor/animation/animation_bezier_editor.cpp
  - dev/Fyrox/fyrox-animation/src/track.rs
  - dev/Fyrox/fyrox-animation/src/signal.rs
  - dev/Fyrox/fyrox-animation/src/lib.rs
  - dev/Fyrox/fyrox-impl/src/resource/gltf/animation.rs
  - dev/Fyrox/editor/src/plugins/inspector/editors/animation.rs
  - dev/bevy/crates/bevy_animation/src/animation_curves.rs
  - dev/bevy/crates/bevy_animation/src/animation_event.rs
  - dev/bevy/crates/bevy_animation/src/lib.rs
  - dev/bevy/crates/bevy_gltf/src/loader/gltf_ext/scene.rs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/Expressions/VFXExpressionBakeCurve.cs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/Expressions/VFXExpressionSampleCurve.cs
  - dev/Graphics/Packages/com.unity.visualeffectgraph/Editor/Models/Slots/Implementations/VFXSlotAnimationCurve.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Editor/Tools/Converters/AnimationClipConverter/EditorCurveBindingUtils.cs
doc_type: review-and-refactor-plan
refreshes:
  - docs/plans/optimize/zircon_editor/77-editor-animation-sequence-clip-channel-binding-interpolation-compression-event-root-motion-sync-preview-compiler-product-integration-current-source-review.md
canonical_owner: docs/plans/optimize/zircon_editor/77-editor-animation-sequence-clip-channel-binding-interpolation-compression-event-root-motion-sync-preview-compiler-product-integration-current-source-review.md
implementation_owner: docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# Editor Animation Sequence、Clip、Channel Binding、Interpolation、Compression、Event、Root Motion、Sync、Preview、Compiler 与 Product Integration 当前源码复核

## 1. 结论

当前源码已经出现一个应保留的共享 Sequence source compiler：`zircon_runtime/src/core/framework/animation/compiler/sequence/compile.rs` 会检查 duration/FPS、空 binding、重复 property track、key time 顺序、值类型、有限性、四元数可归一化性和 Hermite tangent 兼容性，并生成带内嵌 key/value/tangent 的 `AnimationCompiledSequence`。`AnimationCompileProduct::Sequence` 与 Editor 文档的 current/last-known-good（LKG）状态也是真实底座。Animation Runtime 插件的 Clip evaluator 也已经有 skeleton target table、stable target hash、revision-bounded cache、pose pool、严格递增 key 时间和值类型检查。这些不能被旧报告的“完全没有 compiler/compiled artifact”描述覆盖。

但工程闭环仍没有建立。共享 Sequence IR 是 source-only artifact，实际播放仍使用 `zircon_runtime/src/animation/sequence/compiled.rs` 的另一种 `CompiledAnimationSequence`：它只保存 writer 与 `(binding_index, track_index)`，采样时重新索引外部 `AnimationSequenceAsset`。插件 Clip 又有独立 validator、sampler 和 Hermite 文件，Editor curve projection 还有自己的 component/tangent fallback。于是同一个 source 可以经过三条不同语义路径，且 Editor 文档编译结果没有被 pane、save、preview、cook 或 runtime install 消费。

采样 correctness 仍有明确错误：内置 Sequence、插件 `channel_sampling`、插件 Clip evaluator 都在内部精确 key 使用前一值的 Step 语义，测试还把这一行为固化为通过；Quaternion Hermite 在两条实现中都只做 slerp、忽略 tangent，而 shared compiler 反而允许该组合。Clip 编译检查了 T/R/S 值类型、finite 与零长 rotation，却没有一次性验证 clip duration、key range、interpolation/tangent domain 和 event schema。glTF 能读取 CubicSpline 三元组，但不支持 morph weights，且没有把 import -> prepared artifact -> sample 的 golden 契约接上。

事件系统相比旧版已有 bounded/resumable batch、最大事件数/字节/播放跨度、heap 候选、实体级 admission、deferred retry 与 capacity diagnostic；但它仍只支持 `to_time > from_time`，播放器入口把时间推进到 `max(0.0)`，因此 reverse、reverse-loop、direction flip 与 seek policy 不存在。cursor 以播放时间、event 文本和 Vec track index 续传；event 没有 stable ID，reimport/reorder 后无法恢复。发布时 `AnimationEventRecord.clip` 被写成 `None`，同一逻辑事件还同时发送 record 和 raw event 两种类型。

播放也没有 terminal contract。`AnimationPlayerComponent` / `AnimationSequencePlayerComponent` 只有 `playing`、`looping`、speed 和浮点 time；非循环到末尾后继续累计并永久 clamp 到端点，没有 Completed/Stopped/Faulted、一次性 terminal receipt、hold/reset end behavior 或 completion delivery。多个 Sequence/Clip writer 没有 claim、priority、blend 或 range arbitration，最终属性由 query/asset 遍历顺序决定。root motion、additive base、sync marker、compression/cook/DDC、streaming page 和 runtime-backed Editor preview 仍没有生产类型或调用链。

本轮将旧 Editor77 的 13 项 P1、6 项 P2 重新按当前源码判定：P1 为 **8 Open / 5 Partial / 0 Closed**，P2 为 **4 Open / 2 Partial / 0 Closed**；48 个资格门为 **40 Fail / 8 Partial / 0 Pass**。Partial 只表示共享编译、缓存、事件预算或 Editor LKG 这类底座已出现，不代表产品、语义或运行资格通过。本报告不新增父 owner 的 P0 计数；Editor14/32/69/75/76/184/190 与 Runtime08C 的通用阻断仍需回链。

本轮只做静态 current-source review 与文档建账，不修改 production Rust/ZUI，不运行 Cargo、真实 Editor、GUI/GPU、save/reopen、reimport、cook、fault/soak/profile 或同语义跨引擎 benchmark。不能据此宣称性能或表现达到、更不能宣称超过 Unreal。按用户要求不审 Tooling，也不查询、轮询、等待或实时跟踪协调器。

## 2. 审查边界、owner 与冻结语料

### 2.1 本报告唯一纵向边界

本报告拥有“Sequence/Clip 持久 source 如何经过 semantic validation/compiler/preparation，进入 sampling、playback、event、root-motion/sync 提交，并由 Editor 使用同一 artifact 预览”的纵向链。Editor75/196 拥有 Timeline、Curve、selection、scrub、snap、clipboard 和交互 controller；Editor76/197 拥有 Graph、State Machine、layer、blend-space 与 shared animation compiler 总 authority；Editor32 拥有 skeleton、retarget、glTF/import/reimport；Editor14 拥有默认 toolkit、通用 authoring transaction 与 runtime-backed preview；Runtime08C 拥有调度、pose、IK、prepared/compressed platform、root motion、sync、montage/action。

本轮引用这些 owner 的边界证据，但不重复登记其 P0。Sequence/Clip 的 stable identity、channel truth table、event traversal、property arbitration 和 source/prepared separation 属于本报告的新增/刷新范围。

### 2.2 当前源码拓扑

```text
AnimationSequenceAsset / AnimationClipAsset / AnimationTimelineDescriptor
  -> shared source compiler (Sequence only)
  -> Editor AnimationAuthoringDocument current/LKG (document only)
  -> Runtime world-bound sequence compiler (separate artifact)
  -> Plugin Clip validator + target table + pose cache (separate artifact)
  -> three channel samplers + two Hermite implementations
  -> pose/property/event effects committed at different pipeline stages

GLTF channel
  -> Step / Linear / CubicSpline -> Step / Linear / Hermite
  x MorphTargetWeights rejected
  x no cooked/prepared/compression/root-motion/sync artifact

Clip event request
  -> bounded heap batch -> Entity admission/backlog -> record + raw event publication
  x no reverse/seek policy, stable EventId, clip/player identity or single delivery
```

### 2.3 证据快照

本轮逐段阅读 shared compiler sequence model/compile/tests、runtime sequence compiled/channel/interpolation/time/conversion/target、asset channel/clip/sequence/timeline/event、plugin channel sampling 与 Clip evaluator/compile/cache/pipeline、Editor document/mutation/session/timeline/curve/pane/save、GLTF animation ingest，以及五套参考引擎的对应 source。冻结基线为 `681588f7a1cbfaae3147e8b93e1be6705d810f21`；工作树含在途和未跟踪文件，本文只把当前磁盘源码作为事实，不回退、不归属、不提交 production 变更。

| 范围 | 文件 / 行 / 非空行 / bytes / tests | 本轮证据 | working-tree fingerprint |
|---|---:|---|---|
| Schema/compiler/import | **33 / 6,849 / 6,261 / 229,022 / 27** | animation asset/binary/timeline/event、shared compiler 与 glTF animation ingest | `5141fbabd0e2c203d2b4ff72fb0ca45c2694086d08dc447910ad70e54f9bf0a3` |
| Runtime evaluation | **44 / 6,516 / 6,002 / 232,526 / 32** | Sequence world binding/sample、Clip evaluator/cache、event 与 pipeline commit | `cbe2b05573dbe2be503bb00c5e6fd72fa9bed1e3fa64dc99f7f8eec6f59473cd` |
| Editor authoring/product | **40 / 5,251 / 4,919 / 191,794 / 15** | document/revision/CAS/LKG、mutation/session、pane/slot 与 save | `72f773f3fd9cab07306bee56f294d3b5476e03529d4e7df9a9f629db5d347df3` |
| Focused tests | **42 / 9,878 / 9,124 / 363,457 / 169** | compiler、Sequence、event、glTF、runtime plugin 与 Editor contract | `8acf16408fac4d029c064fab57280e3b845bdceb50824ecc0160734eadc9ad16` |
| Zircon 去重合计 | **154 / 27,034 / 24,961 / 963,277 / 222** | 上述四组按 normalized path 去重 | `e8b5b44c88062e925e07b15caef05f90602fb96745b5170213689617ece09323` |
| Unreal selected set | **10 / 14,211 / 11,948 / 526,528 / 0** | data model、sequence/compression、notify/sync/montage/root motion | `53aedd73c674eef9c1ac2d652f68d777b0c6619584cc076bd8a63ff11b0e67ca` |
| Godot selected set | **8 / 24,932 / 21,174 / 911,878 / 0** | typed tracks、mixer/player、track/Bezier Editor | `f8e4bf1f872752919d21ad8d6c14d7a419cfb167678e42e61cbf616e479b9611` |
| Fyrox selected set | **5 / 2,380 / 2,158 / 88,554 / 0** | UUID track/signal、reverse、root motion、Editor inspector | `19e61c80aab11aefc93bd6780f2aaa8d130d1a4737224b3a072566680adc0511` |
| Bevy selected set | **4 / 2,861 / 2,595 / 106,085 / 11** | typed curve/evaluator、event traversal 与 glTF loader | `f0539d8a87f7f037c0d85d4d2c09df56d7154b351ce46b63c2db77f8a00e258a` |
| Unity Graphics selected set | **4 / 154 / 132 / 5,575 / 0** | curve bake/sample、typed slot 与 binding conversion | `521293c58ede460374bb4bfeb6009b7f712eac80f3bb3bbf6f8fc590a543e7e2` |
| 五引擎参考合计 | **31 / 44,538 / 38,007 / 1,638,620 / 11** | 五组显式路径去重 | `8caecf8bf258d7035e0aa7cdc75e063682384843d0f99b216a882834219bee69` |

fingerprint 按小写规范化相对路径排序，将每个 `path + NUL + lowercase file SHA-256 + LF` 聚合后再取 SHA-256；它是本轮静态输入 receipt，不是 compiler artifact key 或 asset revision。

共享 compiler 的三项测试覆盖 canonical key 保留、time/type/write conflict 与 invalid quaternion/tangent；runtime sequence tests 覆盖 world writer、numeric target、stale target 和 topology retry；event tests 覆盖 bounded loop、byte deferral、same-time cursor 和 heap comparison；但这些测试没有把 shared artifact 接入 runtime，也没有 reverse、completion、property conflict、root motion、compression 或 Editor preview golden。Clip evaluator 的性能 benchmark 仍 `#[ignore]`，不能作为已通过的资格证据。

## 3. 当前可保留底座与源码校正

### 3.1 可保留底座

1. `AnimationCompileSource` / `AnimationCompileProduct` 统一入口、typed diagnostics、severity 和 source element 定位。
2. Sequence source compiler 的 finite/time/type/duplicate-track 检查与内嵌 key/tangent model。
3. Editor `AnimationAuthoringDocument` 的 revision/CAS、Document history、current/LKG compilation；无效编辑仍可 undo，不污染 last-good。
4. Plugin skeleton target table、stable target path hash、compiled clip cache、pose pool、resource revision invalidation 和诊断保留。
5. Clip event 的 max events/bytes/span、heap candidate、entity admission、deferred retry 与 capacity diagnostic。
6. glTF hierarchy cycle/depth 检查、skin dependency 与 target path 生成意图。

### 3.2 对旧报告的源码校正

| 旧结论 | 当前源码 | 本轮裁决 |
|---|---|---|
| Sequence 完全没有 shared compiler | `core/framework/animation/compiler/sequence` 已生成 typed source IR | shared compiler 为 Partial；未贯通 runtime artifact |
| compiled Sequence 必然只有 source Vec index | shared IR 已内嵌 key/value/tangent；world-bound `CompiledAnimationSequence` 仍回查 source Vec | P1-03 从 Open 改为 Partial，不关闭 |
| event 每批全量线性排序且无预算 | `BinaryHeap`、max events/bytes/span、deferred admission 已存在 | budget/候选底座 Partial；direction/identity/delivery 仍 Open |
| Clip evaluator 仅是临时逐帧 source 解释 | `CompiledAnimationClip` 已解析 target slot，cache 按 skeleton/clip revision，pose pool 可复用 | target/cache 底座 Partial；channel/event/artifact 仍分裂 |
| Editor mutation 直接改 asset 且没有 LKG | document revision/CAS/history 与 current/LKG 已接入 | authoring transaction 部分关闭；product/preview/save install 仍缺 |
| GLTF 完全丢失 cubic tangent | CubicSpline 三元组已写进 key tangent | import 可保留数据，但 Quaternion Hermite 与 golden 语义仍 Fail |

## 4. P1：Sequence/Clip 生产差距

### ED198-P1-01 · Partial · 三份持久模型仍不等价，`AnimationTimelineDescriptor` 是无人消费的并行 schema

`AnimationSequenceAsset` 使用 binding/entity path/property channel；`AnimationClipAsset` 使用 skeleton/bone T/R/S/event track；`AnimationTimelineDescriptor` 另有 clips、generic tracks、events、avatar mask 和自动 sanitize。当前 grep 只找到 timeline descriptor 的定义/re-export，没有 production compiler、serializer owner 或 runtime consumer。Shared product 只覆盖 Sequence/Graph/State Machine，不把 Clip 与 Timeline descriptor 映射到同一个 source document。

要求建立 versioned `AnimationSourceDocument`，明确 property、skeletal、event、marker、action track 的 typed roles；旧三种 schema 必须由确定性 migration 生成 source/artifact receipt。Timeline descriptor 要么成为唯一 source projection，要么硬切删除，不能继续作为第四写入口。

### ED198-P1-02 · Partial · Sequence source compile 已 strict，但 world pipeline 丢弃 compile/apply outcome，LKG 未进入运行资格

`compile_animation_sequence` 在任何 Error 时返回 `artifact=None`；Editor document 会把成功产品记录为 LKG。但 `zircon_plugins/animation/runtime/src/evaluation/pipeline/sequences.rs` 仍在 asset load 后直接调用 `compile_sequence_for_world`，失败时 `sequence_cache.remove` 后 `continue`，apply 返回值使用 `let _ =`。Asset load 的 `.ok()` 也会把不可用 source 静默过滤。Runtime 不消费 Editor document 的 current/LKG generation，也没有 `AnimationApplyReceipt` 让上层知道每个 track 的 missing/rejected/stale disposition。

要求把 shared compile product、world binding、prepared artifact 和 LKG publication 接成 generation-qualified install；required binding/error 必须 fail-close，optional binding 必须有 stable diagnostic。Pipeline 不能以 `continue` 或 `let _` 替代 typed outcome。

### ED198-P1-03 · Partial · source-only IR 自包含，实际 world-bound `CompiledAnimationSequence` 仍依赖外部 asset Vec index

`core/framework/animation/compiler/sequence/model.rs` 的 compiled key 确实拥有 value/tangent；然而 `zircon_runtime/src/animation/sequence/compiled.rs` 的 `CompiledAnimationSequenceTrack` 只保存 `binding_index` 和 `track_index`，`apply_compiled_sequence_to_world` 每帧从传入 `AnimationSequenceAsset` 重新 `.get(...)` 后再 sample。缓存只按 asset ID/revision 保留 writer；artifact 没有 source hash、schema/compiler/recipe version、dependency generation 或 prepared channel page。

要求 world compiler 消费 shared IR，生成可独立 sample 的 `CompiledPropertySequence`，包括 resolved writer program、prepared channel、source/artifact identity、dependency stamp 与 error metrics。source mutation 不得改变已发布 generation；cache key 必须覆盖完整语义输入。

### ED198-P1-04 · Open · Step 精确关键帧仍返回前一值，测试固化错误 truth table

内置 Sequence `channel_sample.rs`、插件 `channel_sampling/channel_sample.rs` 和插件 `clip_evaluator/channel_sample.rs` 都在内部 key 采用 `time < sample_time` 的 partition/window，再返回 left。runtime test `step_interpolation_keeps_the_preceding_value_at_an_exact_interior_key` 与 Clip test `interior_lookup_keeps_step_key_boundaries_left_inclusive` 将 `t == key.time` 断言为旧值。

要求一个共享 sampling kernel 与明确的半开/闭区间表：Step 在精确 key 切换到该 key；首/末 key、ULP、loop wrap、reverse 和 duplicate rejection 全部由同一 kernel 覆盖。旧测试必须先改成 RED expected semantics，不能作为正确性证明。

### ED198-P1-05 · Open · Quaternion Hermite 公开允许但运行时忽略 tangent

Sequence 与 plugin Hermite 实现对 Quaternion 都只把两个值 normalize 后 slerp，`in_tangent/out_tangent` 不进入计算。Shared Sequence validator 的 `tangent_is_compatible` 却允许 Quaternion/Vec4 tangent，因此 GLTF `CubicSpline` rotation 会被标成 Hermite 后执行非 cubic 语义。

要求明确 quaternion cubic 的数学定义、tangent 空间、shortest-path、归一化和退化诊断；无法支持时 compiler 必须拒绝该组合，不能以 Hermite 名义静默执行 slerp。GLTF quaternion cubic fixtures 需要逐 key、区间和 normalization golden。

### ED198-P1-06 · Open · Sequence、Clip、Editor 三套 validator/sampler 仍有冲突 fallback

Sequence shared compiler 检查 generic scalar/vector/quaternion domain；world sampler 对 type mismatch 返回 left；plugin Clip validator 只允许 T/R/S 的 Vec3/Quaternion，并另有一套 channel sampler/Hermite；Editor curve foundation 把 vector tangent 拆分为 float，quaternion 留在 timeline lane。缺失或错误 tangent 在 sampler 侧转成 zero，通道用途也未在 schema 中声明。

要求唯一 `AnimationChannelSemanticValidator` 与唯一 sampling kernel；用途特定约束用 typed `ChannelRole` 扩展。任何 mismatch、missing required tangent 或 unsupported interpolation 都必须产生稳定 code/location/outcome，禁止 left/zero 猜测。

### ED198-P1-07 · Partial · Clip compiler 有 target/finite/type 校验，但没有完整 duration/key/interpolation/event 合同

`clip_evaluator/sample.rs` 先调用 `validate_clip_channels`，再 `CompiledAnimationClip::compile`。当前覆盖 strict key time、finite value/tangent、T/R/S value type、zero-length rotation、canonical target、duplicate target 和 skeleton parent cycle；但 `AnimationClipAsset.duration_seconds` 没有在同一 compile 阶段要求 finite/positive，key 不检查是否位于 duration，interpolation/tangent domain 不统一验证，event track 也完全不进入 Clip compile。采样端仍对非法 duration/time fallback 到 0 或 clamp。

要求一次 compiler 产出 validation report、prepared channels、event index 和 target binding；strict cook 不允许 silent repair。任何 import repair 必须是显式 migration receipt，不能藏在 runtime sample。

### ED198-P1-08 · Open · event traversal 没有 reverse、seek、loop boundary 与 direction-change policy

`sample_clip_events_budgeted` 在 event track 为空、非 finite、`to <= from` 或 max_events 为零时直接返回空；`event_sampling_range` 只构造 forward range。`scan_clip_players` 与 `scan_sequence_players` 以 `(time + delta * speed).max(0.0)` 推进，负 speed 虽可写入 Editor/组件，却不会产生 reverse event。loop event 只在 forward heap 中追加 duration occurrence，没有 reverse-loop、seek suppress/fire、boundary ownership 或 direction flip state。

要求 `AnimationTraversal` 携带 from/to、direction、loop ordinal、seek policy、boundary inclusion 和 event class；Forward/Reverse/ForwardLooping/ReverseLooping、多 loop、零点/末点和 speed sign change 需同一 truth table。

### ED198-P1-09 · Open · event cursor 身份依赖字符串与 Vec 下标，不能跨 reimport/reorder 稳定恢复

`AnimationClipEventSamplingCursor` 只有 playback time、`last_event: Box<str>` 和 `last_track_index`；`AnimationEventTrackAsset` 没有 EventId。`event_is_after_cursor` 以 event 字符串排序，再以 track index 断同时间事件。改名、插入、排序、reimport 或 artifact replacement 都会改变 cursor 所指语义，无法对事件做 ack、迁移或诊断定位。

要求 source 使用 stable `AnimationEventId`，compiled index 产生 generation-qualified ordinal；cursor 绑定 artifact generation/traversal ID，stale cursor 必须拒绝或显式 resync。

### ED198-P1-10 · Open · event delivery 丢 clip/player identity 且同一逻辑事件双发

`AnimationClipEventSamplingRange` 虽携带 `clip_id`，但 `AnimationClipEvent` 不携带 clip，`animation_event_record` 在 pipeline 转换时写 `clip: None`。随后 `publish_clip_events` 先 publish `AnimationEventRecord`，再 publish 原始 `AnimationClipEvent`。订阅两种类型的 consumer 会执行两次副作用，且没有 player instance、artifact generation、direction、loop ordinal、delivery sequence 或统一 retry receipt。

要求只发布一个 typed `AnimationEventDelivery`；兼容 raw event 必须是显式 adapter 并带去重身份。delivery、bus rejection、consumer fault 与 retry 都必须保留 clip/player/event identity 和 terminal disposition。

### ED198-P1-11 · Open · 多 writer 没有 property claim、优先级、blend 或范围仲裁

`apply_compiled_sequence_to_world` 对每条 writer 直接写 World；多个 sequence player、多个 binding 或 Clip/property track 可以写同一属性。Sequence compiler 只检查一个 binding 内的重复 property path，不知道其他 player；测试甚至允许把 `AnimationPlayer.weight` 写成 `2.0`。没有 exclusive/override/additive/blend operator、priority、range validation 或 deterministic conflict diagnostic。

要求 compiler 生成 `AnimationPropertyClaim`，playback transaction 先收集 contribution，再按 schema operator/priority/arbitration 解析，最后一次 commit。不同 query/asset 顺序必须得到同一结果，非法 domain 在 commit 前拒绝。

### ED198-P1-12 · Open · 非循环播放没有 Completed/Stopped/Faulted terminal contract

`AnimationPlayerComponent` 与 `AnimationSequencePlayerComponent` 只有 bool playing/looping、speed 和浮点 time。`parameter_apply.rs` 在 playing 时只做 `time + delta * speed`；`resolve_sample_time`/`resolve_sequence_sample_time` 在 clip 外 clamp，非循环不会写 terminal state。到末帧后会每帧重复写端点，没有一次性 completion event、hold/freeze/reset 行为，也没有负向起点 terminal。

要求 `AnimationPlaybackState` 明确 Playing/Paused/Completed/Stopped/Faulted、end behavior 和 crossing receipt。正向末尾、反向起点、零 duration、speed/duration change、loop off 和 artifact replacement 必须各自产生一次确定性 terminal disposition。

### ED198-P1-13 · Partial · Editor、Timeline、Sequence、Clip 没有共享 time domain 与 playback commit stage

Editor session 用 frame/FPS projection 和任意 finite speed；Timeline descriptor 又把 speed clamp 到 non-negative；Sequence/Clip 各自有 clamp/rem-euclid helper；event 只接受 forward range。pose、property、event 在 pipeline 的不同阶段提交，没有统一的 currentness recheck、root motion、pre-animated restore 和 rollback。

要求消费 Editor75/196 的 time-domain owner，建立 `AnimationPlaybackTransaction`：validate generation -> advance traversal -> sample prepared pose/property -> resolve claims -> collect delivery -> extract root motion/sync -> revalidate -> atomic commit -> receipt。Editor preview、PIE 和 game runtime 只能切换 policy，不能切换 kernel。

## 5. P2：格式、诊断、性能与测试债务

### ED198-P2-01 · Open · binary channel 的 arity 字段仍不校验

`AnimationChannelValueBinary` 序列化 bool/integer/scalar/Vec2/Vec3/Vec4/quaternion 与 `arity`，但 `TryFrom` 只匹配 `tag`，不检查 arity 是否为 0/1/2/3/4，也不检查 unused scalar payload。损坏或伪造 payload 仍能进入内存。Decode 必须校验 version、tag、arity、payload shape、limits，并给出字段位置。

### ED198-P2-02 · Open · source target/event identity 仍是 `Option<String>`

Clip track、event track、Sequence binding 和 Timeline descriptor 都以字符串保存 target；runtime 虽已有 `AnimationTargetId`，只在部分 skeleton table 编译时 hash。字符串 parse/format、大小写与 path 变更会导致身份漂移。持久 schema 应保存 typed stable ID，并保留可读 source locator 作为迁移/诊断信息。

### ED198-P2-03 · Open · Clip diagnostic 去重依赖格式化错误文本

`clip_evaluator/diagnostics.rs` 将 `(skeleton id/revision, clip id/revision, error.to_string())` 作为 `EvaluationDiagnosticKey`。文案、浮点格式或 localization 变化会改变 identity；同一结构故障不能稳定聚合。应使用 stable diagnostic code、source address、artifact generation 和 bounded arguments。

### ED198-P2-04 · Open · event byte budget 只统计字符串，oversized first event 仍穿透

`event_text_bytes` 只加 event、target_id、payload 字符串长度，不含 `AnimationClipEvent`、Vec capacity、queue envelope 或 allocator overhead。为保证 cursor 前进，单条超过 `max_event_bytes` 的首事件仍被发送并仅标记 `oversized_event_count`。应分离 hard admission limit 与 page budget，用保守 owned-byte 上界拒绝不可入场事件并给出 terminal diagnostic。

### ED198-P2-05 · Partial · event 分页有 heap/预算底座，但每批重建候选且 release benchmark 被 ignored

heap 选择减少了旧的全量 `min_by` 访问，测试有 comparison upper bound；但每个 batch 仍从所有 event track 重新构建 `BinaryHeap`，cursor 和 emitted event 仍 clone 字符串。唯一 release 性能用例 `event_candidate_heap_release_benchmark` 标记 `#[ignore]`，没有多 player、长 payload、同时间事件、跨 loop 的帧预算证据。需要 cooked event index、arena/无分配遍历、可执行 P95/bytes/alloc qualification。

### ED198-P2-06 · Partial · 测试覆盖扩展但仍缺语义矩阵，且部分用例固化旧行为

当前新增 shared compiler、target cache、event budget、LKG 和 topology 测试是可保留基础；仍缺 Step exact-key RED、Quaternion cubic/unsupported diagnostic、duration/range corpus、forward/reverse/seek/direction flip、stable cursor migration、single delivery、claim arbitration、completion、root motion/sync、compression determinism、Editor/runtime artifact parity 和 fault rollback。任何旧 Step 预期必须先改为正确 truth table，再把 malformed corpus 与大规模 profile 设为非 ignored qualification。

## 6. 五套参考源码的可执行差距

### 6.1 Unreal：source model、controller bracket、compressed data、notify、root motion、sync 与 montage 分层

`IAnimationDataModel` 把 float/transform/attribute curve、GUID generation、modification notification、bracket、evaluation/modification lock 与 source timing 分开；`AnimSequence` 再承载 raw/source、platform compressed data、compression recipe/derived-data key、additive、retarget 与 root-motion policy。`AnimSequenceBase`/`AnimNotifyQueue` 处理 sorted notify 的跨区间遍历，`AnimSync` 处理 marker leader/follower，`AnimMontage` 引入 section/slot/branching/blend。

Zircon 当前把 source key、world writer、event text 和 pose side effect 混在少数 runtime structs 中，没有 compression artifact、DDC identity、root-motion track、sync marker 或 action/montage program。应借鉴层次和 receipt，不复制 Unreal 的 UObject API。

### 6.2 Godot：typed track/update/loop modes 与可逆 Editor mutation

Godot Animation 区分 value、position、rotation、scale、blend-shape、method、Bezier、audio 和 animation track，显式保存 interpolation/update/loop mode；Mixer/Player 分别处理 discrete/capture、method/audio/reset 与 root motion。Editor `AnimationTrackEditor`/Bezier editor 通过 UndoRedo 修改 typed resource。

这证明 generic `AnimationChannelValueAsset` 不能替代 method/audio/discrete side-effect contract，也不能把所有轨道最终都当作 SceneProperty writer。

### 6.3 Fyrox：UUID track/signal、双向 passage、root motion 与 preview recovery

Fyrox track 与 signal 使用 UUID；signal passage 按正反方向判断，player 保存 speed/time/loop/root-motion 状态并暴露 completed 语义。Editor command execute/revert/finalize，preview 恢复节点状态。Zircon 当前 cursor 用 event text/index，negative speed 没有 event，preview 也没有 isolated world 或 pre-animated restore。

### 6.4 Bevy：typed curve/evaluator 与显式四类 event traversal

Bevy 以 `AnimationTargetId`、typed `AnimationCurve` 和 evaluator id 绑定目标，`animation_event.rs`/player 明确 Forward、Reverse、ForwardLooping、ReverseLooping 的触发语义，并为 seek、loop 与边界提供测试；glTF loader 保留 cubic triplet/tangent、rotation normalization 和 wide morph curve。

这是 Zircon 收敛 Rust typed source/kernel/traversal 的直接参考，但 Bevy 不等同于完整 Editor、DDC 或 montage 产品，不能把单个 runtime crate 当全引擎对标。

### 6.5 Unity Graphics：仅作为 curve bake/sample、typed binding 与 history 的有限证据

本地 `dev/Graphics` 不是完整 Mecanim 源码。本轮只采用 VFX `BakeCurve`/`SampleCurve` 的 source-versus-baked 分层、typed animation curve slot，以及 URP `EditorCurveBinding` 转换工具体现的稳定 property binding 原则。不能由这些文件推断 Unity 完整 clip/notify/runtime 能力。

## 7. 目标架构与唯一 authority

### 7.1 Versioned source

```text
AnimationSourceDocument
  identity: AssetId + SourceRevision + SchemaVersion
  time_domain: rational tick resolution + display rate + duration policy
  stable ids: BindingId / TrackId / ChannelId / KeyId / EventId / MarkerId
  typed tracks: Property / Bone / Curve / Method / Audio / Event / RootMotion
  policy: interpolation / tangent space / additive / sync / completion / compression
```

Source 只表达作者意图。Import repair 必须生成 before/after receipt；Editor mutation 必须通过已有 Document transaction/controller，禁止 runtime sample 时隐式修复。

### 7.2 Compiler/artifact

唯一 `AnimationSemanticCompiler` 分阶段产出：

1. `AnimationValidationReport`：structure、finite、time、binding、event、policy 与 dependency。
2. `CompiledPropertySequence`：prepared channel blocks、resolved writer program、property claims。
3. `PreparedAnimationClip`：target slots、bone/curve/morph/attribute pages 与 compression metadata。
4. `CookedEventIndex`：stable EventId、direction/loop lookup、payload schema、budget metadata。
5. `RootMotionTrack` 与 `SyncMarkerTable`：独立 pose，明确 extraction/consumption policy。
6. `AnimationActionArtifact`：section/slot/branching/montage semantics，依赖 Runtime08C。

每个 artifact 必须携带 source content hash、schema/compiler/recipe version、dependency generations、platform variant、error metrics、memory/streaming metadata；publication 只原子替换 last-good generation。

### 7.3 Playback transaction

```text
AnimationPlaybackTransaction
  1. validate player/artifact/world generation
  2. advance rational AnimationTimeDomain and emit TraversalReceipt
  3. sample prepared pose/property/curve contributions
  4. resolve property claims, additive base and blend deterministically
  5. collect typed event delivery and sync result
  6. extract root motion
  7. revalidate currentness and budgets
  8. atomically commit pose + property + event + root motion
  9. publish PlaybackReceipt and terminal state
```

Editor preview、PIE 和 game runtime 必须使用同一 artifact/kernel，只通过 policy 决定是否写 authoring world、是否 suppress gameplay event、是否恢复 pre-animated state。

## 8. 重构里程碑

### ED198-M0：owner、truth table 与 RED evidence

- 与 Editor14/32/75/76、Runtime08C 冻结唯一 schema/compiler/sampler owner。
- 写出 Step、Linear、Hermite/quaternion、duration、event forward/reverse/loop/seek、completion、claim conflict truth table。
- 将精确 Step key、quaternion Hermite、negative speed event、partial compile、oversized first event 先转为失败 RED fixtures。

### ED198-M1：stable source identity 与时间域

- 引入 versioned source document、Binding/Track/Channel/Key/Event/Marker IDs、rational tick/display rate。
- 为旧 Sequence/Clip/Timeline schema 生成确定性 migration/round-trip receipt。
- 旧 `AnimationTimelineDescriptor` 硬切为 projection 或删除并行写 authority。

### ED198-M2：唯一 semantic validator 与 prepared artifact

- 合并 Sequence/Clip channel validator、唯一 sampling kernel 和用途 role schema。
- shared IR 直接 lowering 到 self-contained `CompiledPropertySequence`/`PreparedAnimationClip`，禁止外部 source Vec index。
- required error fail-close，optional binding/diagnostic 与 LKG generation 原子安装；pipeline 不再吞 load/compile/apply outcome。

### ED198-M3：interpolation/glTF conformance

- 修正 Step exact-key/ULP/loop 边界；定义 linear、cubic scalar/vector/quaternion tangent 语义。
- GLTF Step/Linear/CubicSpline、rotation normalization、morph/wide curve import -> artifact -> sample golden。
- 不支持的 rotation cubic/morph 必须在 compiler/import boundary 显式拒绝并有 stable code。

### ED198-M4：event/notify traversal 与单次 delivery

- stable EventId、cooked index、generation-qualified cursor、Forward/Reverse/Looping/seek/direction truth table。
- `AnimationEventDelivery` 单一 typed bus，保留 clip/player/artifact/direction/loop identity。
- hard admission 与 page budget 分离，fault/retry/consumer rejection 有 receipt。

### ED198-M5：playback transaction、claim arbitration 与 completion

- 引入 Traversal/Playback/Apply receipts、Playing/Paused/Completed/Stopped/Faulted、end behavior。
- 先收集 property contribution，再按 claim/operator/priority/range 原子 commit；失败不得留下半帧 pose/property/event/root motion。

### ED198-M6：compression/cook/DDC/streaming

- 建立 recipe/version/platform/dependency derived-data key、prepared pages 与 source/shipping separation。
- 记录 compression error、memory、decode throughput、page residency、determinism 与 invalidation receipt。

### ED198-M7：root motion、additive、sync、action/montage

- Root motion extraction/lock/scale 与 additive base pose 成为独立 typed artifact/consumer。
- Sync marker group、leader/follower、section/slot/branching/action program 依 Runtime08C 分层接入。

### ED198-M8：Editor Sequence/Clip 产品与 runtime-backed preview

- Sequence slot 与 Clip toolkit 消费 immutable typed projection，显示 hierarchy/channel/key/event/diagnostic/currentness，而不是字符串或空 slot。
- Preview 建立 isolated runtime world、seek/play/pause/step/reverse/loop/event policy、pre-animated restore 与 current/LKG/stale/fault feedback。
- save/reopen、undo/redo、asset replacement、plugin revoke 与 preview exit 有恢复矩阵。

### ED198-M9：规模、故障与跨引擎资格

- 100K/1M keys、10K tracks、多 player、密集 event、large payload、multi-surface scrub 的 P95/alloc/bytes/lock profile。
- malformed corpus、stale generation、worker failure、queue rejection、device loss、reimport/reorder soak。
- 与 Unreal/Godot/Fyrox/Bevy/Unity selected semantics 做相同 fixture/time range/hardware 的可复现正确性和性能报告；未有 receipt 不得宣称超过 Unreal。

## 9. 48 个资格门

| Gate | 资格 | 当前 |
|---|---|---|
| ED198-G01 | Sequence/Clip/Timeline source model 有唯一 owner 与字段映射 | Partial |
| ED198-G02 | `AnimationTimelineDescriptor` 不再是并行写 authority | Fail |
| ED198-G03 | Binding/Track/Channel/Key/Event/Marker 具有持久 stable ID | Fail |
| ED198-G04 | time base 使用 rational tick/display-rate contract | Fail |
| ED198-G05 | binary decode 校验 version、limit、tag、arity、payload shape | Fail |
| ED198-G06 | source round-trip 不静默修正或改写语义 | Partial |
| ED198-G07 | Sequence 与 Clip 使用同一 semantic validator/kernel | Fail |
| ED198-G08 | required binding/track compile failure 不能 partial publish | Fail |
| ED198-G09 | optional binding disposition 具有 stable diagnostic | Fail |
| ED198-G10 | artifact 拥有 source/compiler/recipe/dependency identity | Fail |
| ED198-G11 | prepared artifact 离开 source 后可独立 sample | Partial |
| ED198-G12 | last-good artifact 按 generation 原子替换并可被 runtime consume | Partial |
| ED198-G13 | 内置 Sequence 与插件 Clip 只使用一个 sampling kernel | Fail |
| ED198-G14 | Step 精确 key、ULP、loop 边界符合统一 truth table | Fail |
| ED198-G15 | Linear scalar/vector/quaternion golden 通过 | Partial |
| ED198-G16 | Hermite scalar/vector tangent golden 通过 | Partial |
| ED198-G17 | Hermite quaternion 有真实数学语义或被 compiler 拒绝 | Fail |
| ED198-G18 | type/tangent mismatch 不再静默 left/zero fallback | Fail |
| ED198-G19 | glTF Step/Linear/CubicSpline import 到 sample golden 通过 | Partial |
| ED198-G20 | glTF morph/rotation normalization/wide curve 通过 | Fail |
| ED198-G21 | key duration/range/finite/order 一次 compile 完成 | Fail |
| ED198-G22 | event duration/range/payload schema compile 完成 | Fail |
| ED198-G23 | Forward event boundary matrix 通过 | Fail |
| ED198-G24 | Reverse event boundary matrix 通过 | Fail |
| ED198-G25 | ForwardLooping 与 multi-loop matrix 通过 | Fail |
| ED198-G26 | ReverseLooping 与 direction flip matrix 通过 | Fail |
| ED198-G27 | seek suppress/fire policy 可配置且确定 | Fail |
| ED198-G28 | event cursor 绑定 stable ID 与 artifact generation | Fail |
| ED198-G29 | same-time/reorder/reimport 分页不重不漏 | Fail |
| ED198-G30 | 一个逻辑事件只有一个 typed authoritative delivery | Fail |
| ED198-G31 | delivery 保留 clip/player/event/direction/loop identity | Fail |
| ED198-G32 | event hard admission 与 page budget 有可执行证据 | Partial |
| ED198-G33 | player 具有 Playing/Paused/Completed/Stopped/Faulted 合同 | Fail |
| ED198-G34 | 非循环末尾/起点只产生一次 terminal receipt | Fail |
| ED198-G35 | loop、speed/duration change 结果确定 | Fail |
| ED198-G36 | multi-writer property claim 在 compile 期可见 | Fail |
| ED198-G37 | property conflict/blend 不依赖遍历顺序 | Fail |
| ED198-G38 | property domain/range 在 commit 前校验 | Fail |
| ED198-G39 | pose/property/event/root motion 按事务原子提交 | Fail |
| ED198-G40 | currentness 失败不留下半帧副作用 | Fail |
| ED198-G41 | root motion 提取与消费 policy 可验证 | Fail |
| ED198-G42 | additive base、sync group/marker 合同可验证 | Fail |
| ED198-G43 | source 与 compressed/prepared shipping 数据物理分离 | Fail |
| ED198-G44 | compression error/memory/decode/determinism receipt 通过 | Fail |
| ED198-G45 | Editor preview 与 game runtime 使用同一 artifact/kernel | Fail |
| ED198-G46 | preview 退出、切 asset、undo、fault 可恢复 authoring state | Fail |
| ED198-G47 | 大 clip/multi-player/dense event hot path 达到既定预算 | Fail |
| ED198-G48 | 同质量同场景跨引擎 correctness/performance qualification 通过 | Fail |

## 10. 验证边界、状态与路由

### 10.1 本轮验证

- 已静态阅读报告 frontmatter 所列 current source、tests 与参考引擎文件，并沿 compile -> cache -> sample -> event -> pipeline -> Editor projection/save 调用链复核。
- 未运行 Cargo、Editor、GUI/GPU/native input、save/reopen、reimport、cook、fault/soak/profile 或跨引擎 benchmark；源码中的 unit test、`debug_assert` 和 `#[ignore]` 只证明意图，不证明通过。
- 本报告只写 docs/index/coverage 记录，不修改 production Rust/ZUI；工作树原有在途变化保持不动。

### 10.2 状态与禁止项

- review：`current_source_refresh_complete`；implementation：`pending`；canonical owner：Editor77；本轮不新增跨报告 canonical finding 总数。
- P1：13 项，8 Open / 5 Partial / 0 Closed；P2：6 项，4 Open / 2 Partial / 0 Closed；Gate：40 Fail / 8 Partial / 0 Pass。
- 禁止把 shared source compiler、target cache、event heap 或 Editor LKG 单独描述成“Sequence/Clip 已完成”。禁止继续用空 slot、字符串列表、`Option<String>`、time bits/Vec index、`let _`/`continue` 吞掉运行结果；禁止以 `#[ignore]` benchmark、单元测试或静态 capability 表宣称超越 Unreal。
- 实施前必须重算本报告 source/reference fingerprint，复核 Editor14/32/69/75/76/184/190 与 Runtime08C owner 终态，从 ED198-M0 RED guards 开始。

## 11. 最终判断

Zircon 已从“Sequence 直接解释 source、Clip 只做临时 pose”推进到“有 shared Sequence validator/IR、Editor LKG、Clip target cache、bounded event queue”的骨架阶段。这些底座应保留并统一，而不是推倒重写。

但当前仍不是工程级 Animation Sequence/Clip 系统：实际 runtime artifact 与 shared IR 分叉，三套 sampler 共享错误 Step 语义，Quaternion Hermite 与 glTF cubic 不一致，Clip duration/event validation 缺失，事件反向/身份/单次 delivery 不存在，播放完成态和多 writer arbitration 缺失，source/prepared/compression/root-motion/sync/action 分层没有形成，Editor product/preview 也没有消费同一 artifact。下一步应先封闭 stable identity、time/interpolation/event truth table 与唯一 compiler/kernel，再做 prepared/cook/playback transaction，最后才实现完整 Editor Clip/Sequence 产品与大规模资格。继续给当前空 slot、字符串 payload 或旁路 sampler 追加零散功能，只会再制造一套不可验证的动画 authority。
