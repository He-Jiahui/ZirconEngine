---
title: Runtime RenderGraph / RenderScene / GPUScene / GraphExecution / FrameSubmission 当前工作树复核
category: zircon_runtime
report_id: Runtime223
review_date: 2026-09-02
baseline_head: 9963f8eb72e2d725d2536eb50b393b30387a1ffa
baseline_epoch: 2026-09-02
verification_head: working-tree
verification_epoch: 2026-09-02
supersedes_currentness_of:
  - zircon_runtime/213-runtime-visibility-gpu-scene-culling-batching-instancing-hzb-virtual-geometry-current-working-tree-review.md
  - zircon_runtime/214-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-current-working-tree-review.md
related_owner_reports:
  - zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md
  - zircon_runtime/213-runtime-visibility-gpu-scene-culling-batching-instancing-hzb-virtual-geometry-current-working-tree-review.md
  - zircon_editor/270-editor-scene-viewport-runtime-render-scene-visibility-hzb-picking-surface-product-integration-current-working-tree-review.md
related_code:
  - zircon_runtime/src/render_graph
  - zircon_runtime/src/graphics/scene/render_scene
  - zircon_runtime/src/graphics/scene/gpu_scene
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract
  - zircon_runtime/crates/zr_rhi/src/submission.rs
  - zircon_runtime/crates/zr_rhi/src/submission_packet.rs
  - zircon_runtime/crates/zr_rhi/src/device/render_device.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/submission.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/command_submission.rs
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderGraphBuilder.h
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/RenderGraphBuilder.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/Compiler/NativePassCompiler.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphResourceRegistry.cs
  - dev/godot/servers/rendering/rendering_device_graph.h
  - dev/godot/servers/rendering/rendering_device_graph.cpp
  - dev/bevy/crates/bevy_render/src/renderer/mod.rs
  - dev/bevy/crates/bevy_render/src/renderer/render_context.rs
  - dev/Fyrox/fyrox-graphics/src/server.rs
doc_type: current_working_tree_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
tooling_scope: excluded_by_user_request
---

# Runtime223: RenderGraph / RenderScene / GPUScene / GraphExecution / FrameSubmission 当前工作树复核

- 复核日期：2026-09-02。
- 复核 HEAD：`9963f8eb72e2d725d2536eb50b393b30387a1ffa`；选择集为共享 working tree，不能当作 clean-HEAD 验收。
- 复核方式：逐文件读取 owner、调用点、负消费者、测试边界和参考引擎对应实现；未修改 Rust、Cargo、shader、ABI 或产品 UI。
- 未运行 Cargo、真实 GPU、RenderDoc/Nsight、multi-queue、device-loss、OOM、soak 或 benchmark；这些是后续动态资格证据。
- Tooling 按用户要求排除；本轮没有查询、轮询、等待或实时跟踪协调器。

## 1. 选择集与证据

本轮选择集覆盖 RenderGraph builder/compiler/access/lifetime/transient plan、RenderScene registry/journal/projector、GPUScene storage/upload/journal consumer、GraphExecution packet/recording/materialization、compiled-scene submission，以及 `zr_rhi`/WGPU submission owner。当前统计如下：

| 选择集 | files | lines | non-empty | bytes | test attrs | ignored | unsafe | dirty |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 完整选择集（上述路径去重） | **294** | **84,787** | **78,392** | **3,178,722** | **785** | **24** | **1** | **64** |

按 owner 划分的文件集合存在嵌套和重复引用，本轮只将可复算的去重 union 作为正式数字；RenderGraph、RenderScene/GPUScene、GraphExecution/submission 和 RHI 是审查边界，不把未经独立复算的分组数字写入账本。

64 个 dirty 文件来自共享工作树中的其他在途改动；本报告保留这些改动，只把源码现状记录为 `working-tree`。统计是词法/结构证据，不代表功能覆盖率或 GPU 性能。

关键证据索引：

| 文件与行 | 观察 |
|---|---|
| `graphics/scene/scene_renderer/core/scene_renderer_render_scene.rs:9-59` | RenderScene admission 由 renderer 调用 streamer/projector，并只记录 journal/residency counters。 |
| `graphics/scene/resources/resource_streamer/resource_streamer_residency.rs:71-127` | frame admission 消费 projection/resource-reference deltas；没有 GPUScene journal consumer。 |
| `graphics/scene/gpu_scene/journal_consumer.rs:322-354` | GPUScene journal staging/commit API 已存在，带 cursor/slot generation 事务。 |
| `graphics/scene/scene_renderer/mesh/build_mesh_draws/build/gpu_scene_sync.rs:34-130` | 生产同步仍遍历 `pending_draws`，逐 draw `register(..., 1)`，最后 `retain_registered_keys`。 |
| `render_graph/graph.rs:48-127,175-204` | compiled graph 仅保存 pass/access/lifetime/transient allocation/stats；没有 state/barrier/queue sync 字段。 |
| `graphics/pipeline/declarations/compiled_render_pipeline/execution_packet.rs:167-230,285-312` | execution packet 生成 graph-order queue batches；API 注释把它们定位为 future lowering 的边界。 |
| `graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage.rs:360-445` | stage executor 按 packet batch 过滤和执行 pass，但仍由 stage service 路由。 |
| `graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/submit_compiled_scene_frame.rs:45-80` | submission context 收集 command buffers、history、IBL、readback 等 side effects，未由 graph artifact 统一声明。 |
| `graphics/scene/scene_renderer/core/scene_renderer_completion.rs:13-75` | 单一 backend poll 后依次路由 scene completion、residency、IBL 和 diagnostic query。 |
| `crates/zr_rhi/src/submission_packet.rs:12-18,115-131` | RHI packet 只约束同一 queue class 的 command lists 和 diagnostic scopes。 |
| `render_graph/graph/transient_allocation.rs:11-45,145-193` | allocation identity 与 interval proof 明确声明为 compiler-local/backend-neutral。 |

## 2. 结论

当前 RenderGraph、RenderScene、GPUScene 和提交服务各自都有真实底座，但还没有形成 Unreal RDG、Unity RenderGraph 或 Godot RenderingDeviceGraph 那样的单一、设备限定、可执行 artifact。

已确认可保留的底座：

- RenderGraph 有 generation-scoped pass/resource identity、版本化 access、RAW/WAW/WAR 依赖、范围依赖、culling、拓扑顺序、transient allocation interval proof、exact access id 和 compiled cursor。
- RenderScene 有 stable primitive key、slot generation、dense storage、free-slot reuse、change journal、cursor、projection transaction 和 resource-reference delta。
- GPUScene 有 stable-key entry、dirty range/update queue、staging ring、current/previous transform、skin palette arena、morph/VG upload report 和 device-generation submission ticket。
- Frame submission 有单一 poll owner、`SubmissionTicket`、`SubmissionPollReceipt`、terminal status、device id/generation 校验、surface lease 和失败后 history 清理。

这些底座仍然没有完成跨层闭环：

1. `GpuSceneJournalConsumer` 只在其自身模块和测试中出现；产品 GPUScene 同步仍由 `build_mesh_draws/.../gpu_scene_sync.rs` 遍历 `pending_draws`，每个 draw `register(..., 1)`，再通过 `retain_registered_keys` 做完整 live-set 收缩。RenderScene journal 没有成为 GPUScene 的唯一增量输入。
2. `CompiledRenderGraph` 只有 pass、access、lifetime、allocation 和 statistics。没有 resource before/after state、barrier batch、queue wait/signal、ownership transfer、alias acquire/release、initial/final state 或 completion edge。
3. `RenderGraphExecutionBatch.queue()` 只形成逻辑连续批次。GraphExecution 会按 batch 记录 command buffers，但提交 owner 仍没有消费 queue lane 生成物理队列计划；当前 WGPU service 的 packet 仍是一个 queue class 对应的一条逻辑时间线。
4. RenderScene admission 已在 `render_frame_with_pipeline_to_target` 前执行，且 residency 使用 journal deltas；但 clear、history initialization/copy、viewport product copy、readback、present 和若干 writeback 仍是图外固定阶段，compiled artifact 不能复现整帧 GPU work。
5. RHI ticket/poll/status 解决了“哪个提交完成”的身份问题，却没有解决 graph access 如何降低为设备状态、跨队列如何同步、资源何时可安全 alias 和设备丢失后如何恢复。

因此当前不能声称 RenderGraph 已工程化完成，也没有任何证据支持性能或表现优于 Unreal。

## 3. P0 复核

本报告不重复占有 Runtime213/214 的 canonical P0；下列是对它们在当前工作树的重判。

| owner | 当前状态 | 当前证据 | 结论 |
|---|---|---|---|
| VIS213-P0-1 / RenderScene persistent truth | Partial | `RenderSceneRegistry`、projector、journal 和 streamer admission 已存在 | GPUScene/mesh/visibility 仍未消费 journal；旧 per-frame pending-draw sync 仍是产品 authority |
| VIS213-P0-4 / GPU-driven instancing | Partial | GPU indirect compaction/count/replay 和 GPUScene upload 存在 | 每个 pending draw 仍注册一个 instance；instancing/upload plan 没有共同 product consumer |
| VIS213-P0-5 / HZB truth publication | Partial | previous-HZB、readback、history invalidation 和 report 存在 | HZB 结果仍未作为统一 receipt 发布给 FrameVisibility、streaming、picking、shadow、VG |
| VIS213-P0-7 / visibility-first prepare | Open | `gpu_scene_sync_pending_draws` 在 visibility 后的完整 draw prepare 中执行 | material/deformation/GPUScene/command 准备仍可能为最终不可见对象付费 |
| RG166-P0-001 / device state-barrier-queue completion | Open | compiled graph 和 execution packet 没有 state/barrier/wait/signal/ownership 字段；关键扫描命中为 0 | queue statistics 不等于 queue synchronization；必须新增 device-qualified lowering artifact |
| RG166-P0-002 / exact access authority | Partial | exact access id/table 已存在 | validator 对 `Legacy` 直接放行，executor/resolver 仍可 name/declaration fallback；name 仍是产品路径 |
| RG166-P0-003 / execution authority | Partial | cursor 能防重复、遗漏和乱序 | fixed stage orchestrator 仍决定执行服务；clear/history/copy/readback/init 不在同一 execution plan |

## 4. P1 差异与重构要求

### 4.1 Identity、generation 与 owner

- RenderScene handle 的 slot generation 与 stable key 能防止局部 stale reference，但 GPUScene entry 仍以裸 `u64` stable key 为主，RenderGraph resource/access generation 也未进入统一 frame artifact。需要 `WorldGeneration + SceneGeneration + GpuResidentGeneration + GraphArtifactGeneration` 的组合 owner，并在每个跨层 receipt 中拒绝旧代。
- `GpuSceneJournalApplyPlan` 的 `stable_key_lookup_count()` 固定返回 0，说明当前 consumer 只做 slot 验证；这对本地 journal 正确性足够，但不能替代 stable-key 到 GPU handle 的持久索引和设备退役。
- RenderScene dense relocation/free slot 只保证 CPU 容器语义，不保证 GPU descriptor、material payload、visibility bitset、hit-proxy 和 indirect args 的稳定句柄。重构必须定义可回收 GPU handle、retirement fence 和重新映射的原子协议。

### 4.2 RenderGraph schema 与访问

- `CompiledRenderPass` 字段只有 `declared_queue`、`queue`、flags、dependencies、culled、executor 和 workload；缺少 pass condition、fallback producer、side-effect root、attachment compatibility、native pass merge、async eligibility 和 completion/output policy。
- `RenderGraphResourceAccessIntent::Legacy` 在 `builder/access_validation.rs` 直接 `return Ok(())`。这使 whole-resource/name API 绕过 type/usage/stage validation；必须禁止新增 Legacy，并迁移现有 `*_by_name` resolver。
- transient allocation bucket 当前包含 width/height/depth/array_layers/mips/sample/format/dimension/residency/usage，但不含 `view_formats`。materializer 若比较 view-format bits，会把不兼容推迟到运行时。该问题应在 compile admission 阶段拒绝，而不是让 pool 试探。
- `CompiledRenderGraphTransientAllocationId` 与 `RenderGraphPhysicalAllocationId` 的注释明确它们是 compiler-local/backend-neutral identity，不是 RHI physical allocation/lease。interval proof 只覆盖 compiled pass index 的不重叠，未覆盖 subresource、queue-time、fence 或 heap alignment。
- external alias group 仍是 `Option<String>`，persistent buffer 没有与 persistent texture 相同的统一 lease；initial/final state、ownership 和 export/release handoff 缺失。

### 4.3 Queue、barrier 与执行

- 当前源码对 `queue_wait`、`queue_signal`、`ownership_transfer`、`BarrierBatch`、`ResourceState`、`pipeline_barrier` 的生产路径命中为 0。不能用 `QueueLane` 统计、command list 顺序或 WGPU 默认隐式状态替代显式合同。
- `RenderGraphExecutionPacket` 已把 authored stage 解析为 immutable graph index，并生成按 graph order 的 batch，这是良好基础；但 batch 是 metadata，`execute_graph_stage` 仍按 stage 过滤，最终没有把 batch 降低为 native queue submission、wait/signal 或 ownership transfer。
- `RenderGraphStageExecution` 会在 pass 记录时收集 uploads、HZB commits、history writes、plugin outputs 和 profile 数据，但这些 side effects 不是 compiled graph node/access，因此 culling、lifetime、barrier、capture、failure transaction 看不到完整工作集。
- `FrameCommandEncoderSet` 可以合并 serial/parallel command buffer；这改善 record 并行性，但没有证明 pass dependency、resource state 或 cross-queue hazard 被编码进 command stream。

### 4.4 RenderScene、GPUScene 与 visibility

- `SceneRenderer::admit_render_scene_frame` 只把 projector 结果交给 `ResourceStreamer`；streamer 应用的是 resource reference deltas，未调用 `GpuSceneJournalConsumer`。需要让 GPUScene、visibility、shadow、picking、streaming 都从同一 sealed scene generation 派生。
- `gpu_scene_sync_pending_draws` 先为 pending draw 做 material/geometry/deformation/skinning/morph/GPUScene 写入，再使用 `retain_registered_keys`，保持 full live-set 语义。必须先完成空间相关性、per-view relevance、residency demand 和共享 draw packet，再为可见对象 materialize。
- CPU bounds 仍可从 transform translation/scale 近似 sphere，GPU primitive 又使用 resolved local bounds；HZB shader 对 local bounds 做 world transform。缺少 extract/CPU spatial/GPU/HZB/shadow/streaming 同一 bounds artifact 和非原点、非均匀缩放、负 determinant 的真实 GPU parity。
- GPUScene 的 `light_buffer_desc()` 是 descriptor metadata，不是 RenderGraph external binding 或 graph-owned upload node；light shadow 与 render graph access 仍可能分离。
- HZB 的 report/readback 是诊断和局部反馈，不代表 GPU visibility 结果已经更新 FrameVisibility、LOD、shadow caster、picking 或 VG page demand。需要 per-view final visibility receipt，并只在成功 completion 后发布。

### 4.5 Submission、completion 与 device loss

- `SubmissionTicket` 生命周期包含 Accepted/Submitted/Completed/Failed/Cancelled/DeviceLost，`SubmissionPollReceipt` 也验证 device/generation/monotonic sequence，这是应保留的 RHI 底座。
- 但 `RhiSubmissionPacket` 只携带同一 queue class 的 command lists；没有 graph barrier plan、wait/signal dependency、resource lease set、present/readback/history publication contract。`wait_for_submission` 是 convenience blocking loop，不是 graph scheduler。
- `SceneSubmissionCompletionJournal` 能消费同一次 poll 并对 terminal status 做记录；它目前只服务 completion/status，不会将完成事实回写到 RenderScene generation、GPUScene retirement、HZB history、material residency 或 graph pool 的统一 transaction。
- WGPU `WgpuSubmissionService` 有 bounded unresolved/terminal history、backpressure、command context pool 和 device generation；仍需 native device-lost recovery、stale packet cancellation、queue-specific status、resource retirement lease 和 retry policy 的动态证明。
- Surface present/discard 有同设备 ticket 前提，但 offscreen target、history initialization、viewport product copy 等 frame products 仍在 graph 外完成，无法保证失败/取消/设备丢失时所有产品一起 commit 或 rollback。

### 4.6 P1 交叉切片账本

下表只用于 Runtime223 的实施分解；与 Runtime213/214 重叠的 canonical finding 仍由原报告唯一计数。

| ID | 状态 | 差异与重构要求 |
|---|---|---|
| RENDER223-P1-001 | Partial | RenderScene slot generation、resource readiness、RHI device generation 已分别存在；需组合为跨 scene/GPU/graph/frame 的 owner generation。 |
| RENDER223-P1-002 | Open | `GpuSceneJournalConsumer` 无产品 caller；必须成为 GPUScene 增量同步入口。 |
| RENDER223-P1-003 | Open | GPU entry 仍由裸 stable key 和 allocator index 拼接；需 generational GPU handle、descriptor/payload identity 与 fence retirement。 |
| RENDER223-P1-004 | Partial | GPUScene dirty range、staging、previous transform/palette 已可用；需由 sealed scene delta 驱动而非 pending-draw live set。 |
| RENDER223-P1-005 | Open | 每个 pending draw 固定一个 instance；需 shared draw packet、instance span 和真实产品 instancing consumer。 |
| RENDER223-P1-006 | Open | CPU/GPU/HZB 使用的 bounds authority 分裂；需 canonical local/deformed/world/motion bounds artifact。 |
| RENDER223-P1-007 | Open | HZB 最终结果不发布给 FrameVisibility/LOD/shadow/picking/VG/streaming；需 completion-qualified per-view receipt。 |
| RENDER223-P1-008 | Open | Legacy intent 直接跳过 access validation，name resolver 仍可执行；需 typed access/version/range/intent hard cut。 |
| RENDER223-P1-009 | Open | transient bucket 缺 `view_formats`，physical ID 不是 RHI lease；需 compile-time descriptor compatibility 与统一 physical lease。 |
| RENDER223-P1-010 | Open | compiled graph 无 before/after resource state 和 barrier batch；需 device-qualified state lowering。 |
| RENDER223-P1-011 | Open | logical QueueLane 无 wait/signal/ownership/native queue consumer；需 queue mapping、fork/join 与 single-queue fallback evidence。 |
| RENDER223-P1-012 | Partial | immutable execution packet、graph-order batch 和 cursor 可保留；需通用 interpreter 替代 fixed stage authority。 |
| RENDER223-P1-013 | Open | clear/history/init/copy/readback/present/writeback 在图外；需 graph-owned prologue/epilogue/terminal unit。 |
| RENDER223-P1-014 | Partial | SubmissionTicket/PollReceipt/status/backpressure 已有；需 packet lease set、graph completion edge 与产品 commit/rollback。 |
| RENDER223-P1-015 | Open | device-loss 只在局部状态机中终结 ticket；需 stale graph/cache/lease 取消、resource rebuild 和 surface/history 恢复。 |
| RENDER223-P1-016 | Partial | profile counters、GPU query delivery 和 ignored benchmark 提供局部观测；需 required backend matrix 与 always-on 性能/内存门禁。 |

状态统计：**11 Open / 5 Partial / 0 Closed**。

## 5. 参考引擎差异

| 参考 | 工程化机制 | Zircon 差异 |
|---|---|---|
| Unreal RDG | Builder 在 Execute 前统一做 culling、resource lifetime、per-subresource transitions、prologue/epilogue barrier、async compute fork/join、extraction 和 pass execution | Zircon 有 dependency/culling/interval/cursor，但无 state/barrier/fence artifact，图外 frame work 不能被 RDG dump/capture/replay 描述 |
| Unity RenderGraph | `NativePassCompiler` 合并 attachment pass，计算 first/last use、graphics fence、async resource lifetime extension，并在执行期实际 wait/create/release | Zircon attachment 只有 load/store 级别，queue batch 无 fence consumer，lifetime 不延伸到跨队列等待 |
| Godot RenderingDeviceGraph | `ResourceTracker` 维护 usage/access/subresource dirty 状态，执行前按 barrier index 分组并调用实际 pipeline barrier | Zircon 有范围 dependency 但没有运行时 state tracker 或 barrier group |
| Bevy | Render/Submit/Finish 是显式调度阶段，submit、readback、screenshot、present 共享清晰生命周期 | Zircon 有集中 submission，但 graph-external terminal work 和 graph execution 没有一份统一 plan |
| Fyrox | GraphicsServer 明确 flush、blocking finish 和 swap/present contract | Zircon ticket 更细，但没有将 finish/present/readback/retire policy 写入 compiled graph artifact |

## 6. 重构路线

### M0：冻结当前语义与禁止扩散

- 保存现有 visibility/HZB/GPUScene/render output 的 golden receipt、generation、fallback 和 upload/CPU/GPU timing。
- CI 禁止新增 Legacy whole-resource/name resolver；所有新 pass 必须提交 typed access、version、range、intent、side-effect/output declaration。
- 修复 transient bucket 的 `view_formats` compatibility key；device fixture unavailable 必须输出明确 `Unavailable/Skipped` receipt，required suite 不得静默通过。

### M1：统一 scene identity 与 GPU residency owner

- 将 RenderScene journal 接入 GPUScene、visibility、shadow、picking、streaming 的唯一增量输入。
- 定义 stable key 到 generational GPU handle、descriptor slot、material payload、visibility index 的原子映射和 fence-qualified retirement。
- 删除产品路径中基于 `pending_draws` 的完整 live-set sync；保留它作为 characterization/fallback，直至 journal consumer 完成切换。

### M2：Bounds / ViewFamily / Visibility first

- 建立 canonical bounds artifact，统一 local/deformed/world/motion bounds 的变换、精度、conservative policy 和 generation。
- spatial index、multi-view broadphase、shadow view 与 HZB result 使用同一 scene snapshot；不可见对象不能先进入昂贵 material/deformation/draw preparation。
- HZB 输出 per-view visibility receipt，成功提交后再发布到 FrameVisibility、LOD、shadow、picking、VG 和 residency demand。

### M3：Graph artifact schema 与 physical lease

- `CompiledRenderGraph` 增加 schema version、source graph hash、device/profile compatibility、exact access state、subresource/range lifetime、initial/final state、external/persistent lease。
- transient/external/persistent texture/buffer/view alias 统一为 `PhysicalResourceLease`；编译期生成 heap class/alignment/offset/alias acquire/release plan。
- name 仅保留诊断标签；resolver 只能消费 compiled access token，unknown external 必须 fail-close。

### M4：Barrier、queue 与 native pass lowering

- 从 typed intent 生成 backend-neutral before/after state、pipeline/stage/access、UAV/alias barrier、prologue/epilogue 和 ownership transfer。
- 对 logical queue 做 device-qualified mapping；支持真实 wait/signal、async overlap、single-queue fallback evidence 和 queue-aware lifetime extension。
- attachment schema 至少包含 resolve、sample count、subpass/input feedback、memoryless/discard、native merge/break reason。

### M5：统一 execution/submission transaction

- 用 generic plan interpreter 消费 execution unit、barrier batch、lease、executor id 和 terminal unit；stage 只能是 service registration projection。
- clear、history init/copy、product copy、readback、IBL writeback、present 全部成为 graph-owned node 或 typed prologue/epilogue。
- record/enqueue/submit/present/completion/readback/history publish/retire 共用一个 typed transaction；失败、取消、device loss 共享 abort/retry/stale policy。

### M6：资格与性能

- 建立 WGPU command validation、barrier oracle、external state、alias fault、queue fallback、device loss/OOM、multi-view、100K instance 和 long soak 矩阵。
- 将 ignored performance tests 升级为 always-on bounded benchmark，记录 compile/record/submit/GPU/VRAM/queue wait/upload 和 allocation ceiling。
- 只有同画质、同可见物、同硬件、同 build set 的可复现实验，才能与 Unreal/Unity 做性能结论。

## 7. 资格门

| Gate | 当前 | 通过条件 |
|---|---|---|
| RG223-G01 scene identity | Fail | RenderScene journal 成为 GPUScene/visibility/streaming/shadow 的唯一增量输入 |
| RG223-G02 bounds parity | Fail | extract/CPU/GPU/HZB/shadow/streaming 共用 canonical bounds artifact |
| RG223-G03 visibility-first | Fail | 不可见对象不进行完整 material/deformation/GPUScene/draw prepare |
| RG223-G04 true instancing | Fail | 共享 packet 能产生 `instance_count > 1` 且由产品绘制 consumer 消费 |
| RG223-G05 HZB publication | Partial | GPU HZB 有真实执行和 readback；缺统一 per-view final receipt |
| RG223-G06 exact authoring | Fail | Legacy/name 不再是产品访问 authority |
| RG223-G07 artifact schema | Fail | graph hash/schema/device profile/lease/state 进入 immutable artifact |
| RG223-G08 barrier/state | Fail | 每个 live access 有实际消费的 before/after state 和 barrier batch |
| RG223-G09 queue sync | Fail | physical queue mapping、wait/signal、ownership、single-queue fallback 可验证 |
| RG223-G10 lifetime/alias | Partial | pass interval proof 存在；缺 subresource/queue/fence/heap/retirement proof |
| RG223-G11 execution authority | Partial | cursor 已有；graph 外 terminal work 仍由固定 stage 插入 |
| RG223-G12 submission transaction | Partial | ticket/poll/status 已有；graph lease、present/readback/history 不共享 transaction |
| RG223-G13 device loss | Fail | stale artifact/lease 取消、native recovery、retry receipt 通过 required backend suite |
| RG223-G14 backend qualification | Fail | 不允许 device unavailable 静默 early return，barrier/queue/external/alias fault 有动态证据 |
| RG223-G15 performance | Fail | 有固定场景、设备、阈值、趋势和公平 Unreal 对照 |
| RG223-G16 editor handoff | Fail | Editor 只读 Runtime immutable artifact，不复制 compiler/lease/execution authority |

汇总：**12 Fail / 4 Partial / 0 Pass**。`Partial` 仅表示局部源码底座可保留，不表示跨模块或真实设备验收通过。

P0 记账说明：本报告新增 **0 项 canonical P0**；表中的 7 行只是 VIS213/RG166 既有 owner 的当前交叉切片重判，当前证据合并为 **2 Open / 5 Partial**，仍由原 owner 负责唯一计数。

## 8. 本轮交付边界

本轮只写 review 和重构计划，没有实现代码修正。Runtime213/214 的旧结论已按当前工作树纠偏并保留 canonical ownership；Runtime223 记录的是 2026-09-02 选择集，实施前必须重新计算 fingerprint、dirty boundary 和调用点。下一阶段应按 M0-M2 先收口 scene identity、bounds 和 visibility-first，再进入 graph state/barrier/queue；不应通过增加更多 DTO、固定 workload、source-string test 或 ignored benchmark 来宣称工程化完成。
