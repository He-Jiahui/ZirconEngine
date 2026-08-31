---
title: Runtime Render Graph Builder、Compiler、Resource Lifetime、Pass Culling、Transient Aliasing、Barrier、Queue Scheduling、Execution 当前源码工程化差距复审
category: zircon_runtime
report_id: Runtime166
review_date: 2026-08-30
baseline_head: working-tree
baseline_epoch: 2026-08-30
verification_head: working-tree
verification_epoch: 2026-08-30
supersedes_currentness_of:
  - zircon_runtime/89-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-product-integration-current-source-review.md
related_owner_reports:
  - zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md
  - zircon_runtime/91-runtime-material-shader-module-graph-permutation-compiler-reflection-layout-pipeline-pso-cache-prewarm-hot-reload-product-integration-current-source-review.md
  - zircon_runtime/94-runtime-visibility-spatial-index-bounds-frustum-occlusion-hzb-culling-batching-instancing-gpu-scene-indirect-submission-instance-lifecycle-product-integration-current-source-review.md
  - zircon_runtime/96-runtime-environment-sky-atmosphere-cloud-ibl-reflection-probe-capture-convolution-sh-pmrem-cache-residency-submission-product-integration-current-source-review.md
related_editor_reports:
  - zircon_editor/226-editor-asset-workspace-content-browser-current-source-review.md
related_code:
  - zircon_runtime/src/render_graph
  - zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline.rs
  - zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_render_with_pipeline
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer
  - zircon_runtime/src/graphics/backend/render_backend/render_backend_submission.rs
  - zircon_runtime/crates/zr_rhi/src/submission.rs
  - zircon_runtime/crates/zr_rhi/src/submission_packet.rs
  - zircon_runtime/crates/zr_rhi/src/device/render_device.rs
  - zircon_runtime/crates/zr_rhi/src/device_profile.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/device/native_submission.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/submission.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/command_submission.rs
tests:
  - zircon_runtime/src/render_graph/tests
  - zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_graph_stage_tests.rs
  - zircon_runtime/crates/zr_rhi/src/tests/submission.rs
  - zircon_runtime/crates/zr_rhi_wgpu/src/production/tests/submission_packet.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/89-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderGraphBuilder.h
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/RenderGraphBuilder.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderGraphResources.h
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/RenderGraphPass.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraph.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraph.Compiler.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphResourceRegistry.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Editor/RenderGraphTests.cs
  - dev/godot/servers/rendering/rendering_device_graph.h
  - dev/godot/servers/rendering/rendering_device_graph.cpp
  - dev/bevy/crates/bevy_render/src/renderer/mod.rs
  - dev/bevy/crates/bevy_render/src/renderer/render_context.rs
  - dev/Fyrox/fyrox-graphics/src/server.rs
doc_type: current_source_review
review_status: complete
implementation_status: in_progress
source_recheck_required: true
tooling_scope: excluded_by_user_request
---

# Runtime Render Graph Builder、Compiler、Resource Lifetime、Pass Culling、Transient Aliasing、Barrier、Queue Scheduling、Execution 当前源码工程化差距复审

## 1. 结论

当前 Render Graph 已有可保留的编译基础：builder 使用 generation-scoped pass/resource identity，支持 resource version、RAW/WAW/WAR 依赖、范围校验、版本感知 culling、transient allocation interval、typed compute access packet、resource materialization report，以及 CPU command recording 的 serial/parallel 两条路径。Runtime89 中“末尾 pass 名称重复”“SparseReserved 可无条件进入 materialization”“storage texture schema 被默认推断”三项，当前源码确实加入了唯一名校验、SparseReserved admission error、typed schema/format fail-closed 等修复候选。

但这些修复仍是 source-only 状态，不能把 Render Graph 视为工程级完成。当前图的 compiled artifact 仍只描述逻辑 pass、访问和 transient allocation；它没有产出 device-qualified resource state、barrier batch、queue wait/signal、ownership transfer 或 completion dependency。`QueueLane` 只是 pass metadata，WGPU 产品最终仍经一个 graphics submission service 提交 command buffers。更严重的是，exact access identity 只覆盖 transient allocation，external/persistent 资源和非 compute buffer binding 仍可退回 declaration name 与 whole-resource lookup；`RenderGraphExecutionPacket` 的 batch/cursor 也没有取代产品侧按 `RenderPassStage` 调度的硬编码流程。

本轮新增登记 3 项当前 P0、48 项 P1 重判和 12 项 P2 重判。Runtime09A/Runtime90 继续拥有 RHI barrier、native queue、GPU completion、device loss 与 backend lifetime；本报告只拥有“图定义经过最终归一化，编译成设备可执行 immutable packet，并由产品执行器完整、唯一、按同步契约消费”的边界。历史报告中的 source-only 修复不计入已关闭，亦不重复计数为额外 finding。当前不能宣称性能超过 Unreal、Unity Graphics 或 Godot；没有 managed Cargo、真实设备、RenderDoc、multi-queue 或 benchmark 证据。

## 2. 审查范围与冻结证据

### 2.1 当前源码选择集

| 范围 | files / lines / bytes / test attrs / ignored | fingerprint |
|---|---:|---|
| `zircon_runtime/src/render_graph` | 43 / 15,109 / 544,994 / 141 / 2 | `421fa3b672381d5bf8845edd7416be8650f90eb3503ee3ca364a27a29eca4952` |
| compiled pipeline declaration + execution packet | 7 / 1,742 / 66,457 / 19 / 2 | `3128f51028495940d09a18c67bf8f4804cdea69d06f1a8f0d74bdcb8853ae2ef` |
| graph execution resources, resolver, materialization, pool, packet record | 74 / 22,529 / 835,680 / 212 / 4 | `c4977936d614666f9b3a77b4c1f5801a2378a105ee32d433c0f911bf21553ccc` |
| compiled scene graph, stage executor, history/terminal/submission owner | 49 / 11,406 / 456,297 / 106 / 0 | `70f970e99cc84f11de066e7ad6fe2063a9dff4829ecb43177de4b90c4153019d` |
| RHI submission/device queue slice | 7 / 2,589 / 93,048 / 1 / 0 | `27993c20ebd1010f8b80f4422a0ad069a99ecde87955a17c4d45a109e915f84d` |
| production union, de-duplicated | **180 / 53,375 / 1,996,476 / 479 / 8** | `032497a51e18580186509650cad9339cae85d93db99bc0b0b4c0e23c7e8fc8c0` |

Fingerprint 是本轮工作树中按规范化相对路径和文件 byte length 生成的冻结标识。它不是 Cargo build、ABI、GPU 或性能资格证明；工作树继续变化时必须重新冻结。

### 2.2 当前 owner 边界

1. `RenderGraphBuilder::compile` 负责 uniqueness/admission、依赖推导、culling、拓扑排序、resource lifetime 和 transient allocation plan（`zircon_runtime/src/render_graph/builder/compile.rs:18-121`）。
2. `CompiledRenderGraph` 持有 pass/access/version/lifetime/transient allocation table 和 stats，但字段中没有 state transition、barrier batch、queue submission plan 或 completion edge（`zircon_runtime/src/render_graph/graph.rs:101-120`）。
3. `RenderGraphExecutionPacket` 生成 stage index、graph-order batch 和 cursor；它仍保留 `passes_for_stage`，`stage_for_pass_name` 仍按 name 线性反查（`zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline/execution_packet.rs:66-74,226-331`）。
4. 物理解析器优先查 exact transient access，失败后调用 `require_texture_view_for_declaration` / `require_buffer_for_declaration`；external/persistent rows 没有同级的 typed physical lease（`zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/resource_resolver.rs:130-173`）。
5. 产品执行器由 `execute_compiled_scene_graph_stages` 依次调用 early、forward、scene、post-process、late、present 等 stage，并在中间插入 history copy；每个 stage 再调用 `execute_graph_stage`（`zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/execute_compiled_scene_graph_stages.rs:27-37,101-236,241-400`）。
6. `FrameCommandEncoderSet` 只保证 serial prefix、parallel buffers、serial suffix 的 command-buffer 顺序；它不是 GPU queue schedule（`zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/frame_command_encoder_set.rs:3-53`）。
7. 终端 packet 会追加 viewport product copy 和 diagnostic readback，然后 `finish()`；提交 owner 最后一次调用 graphics submission service，并以 `SubmissionTicket` 退休 transient backing（`.../terminal_frame_packet.rs:28-69`、`.../submit_compiled_scene_frame.rs:92-155`）。

## 3. 当前 P0

### RG166-P0-001：编译产物没有设备级 barrier、队列同步与完成契约

**证据。** `CompiledRenderGraph` 的 immutable fields 只有 pass/access/lifetime/transient/allocation/stats；没有 per-subresource state、transition list、barrier batch、queue wait/signal 或 completion dependency（`graph.rs:101-120`）。`QueueLane` 只有 `Graphics/AsyncCompute/AsyncCopy` 三个枚举值，`PassFlags` 只有 `allow_culling` 与 `has_side_effects`（`types.rs:717-737`）。execution batch 只按 queue label 和 culling gaps 分组（`execution_packet.rs:164-189`）。真实执行使用 WGPU encoder，并把 command buffers 交给 graphics submission owner；没有把 batch queue、资源状态或跨 queue fence 下沉到 RHI（`execute_graph_stage.rs:537-575`、`submit_compiled_scene_frame.rs:92-99`）。

**为什么是 P0。** 图可以在 CPU 上 compile、materialize、record 并获得 `SubmissionTicket`，但这不能证明 GPU 资源在 producer/consumer 之间处于正确 layout/access，也不能证明 AsyncCompute/AsyncCopy 在设备有多个物理队列时被等待或 signal。当前的 `SubmissionTicket` 是提交时间线事实，不是由 graph access 降低的同步计划。任何需要 UAV write -> sampled read、copy -> compute、graphics -> async compute、alias acquire/release 或 external access mode 的 feature 都只能依赖 backend 隐式行为或单队列串行化。

**必须重构。** 在 compile 后增加 device admission 和 `CompiledRenderGraphSyncPlan`：每个 access 绑定 subresource/range、before/after state、pipeline stage、queue class、ownership transfer、wait/signal value、barrier batch 和 error policy；再由 Runtime09A 的 RHI lowering 把它变成 backend packet。单物理队列必须明确记录 serialization fallback，而不是把 fallback 隐藏在 `QueueLane` metadata。未生成并消费该 plan 时，图不得以“compiled”状态进入 product submit。

### RG166-P0-002：external/persistent/non-compute binding 仍可降级到 name 与 whole-resource

**证据。** 逻辑 resource 只区分 `TransientTexture`、`TransientBuffer`、`External`（`types.rs:96-109`）；external binding 只有 resource type 与 `ReportOnly/Required`，没有 physical handle、generation、subresource lease 或 final state（`types.rs:186-205`）。lifetime 虽保留 schema-backed external descriptor 和 optional view alias，但没有统一 physical lease（`types.rs:303-325`）。resolver 在 exact transient access 命中时返回 access-specific view，否则返回 declaration-level view/buffer；buffer path 直接返回 `&wgpu::Buffer`，不带 byte-range lease（`resource_resolver.rs:130-173`）。

**为什么是 P0。** Runtime89 已加入 transient access allocation table，但“非 transient”正是 history、external surface、persistent IBL、plugin imported buffer、readback 和跨 frame resource 的主要来源。当前 pass context 对 executor 暴露 `require_*_by_name`，这使编译时 access/version identity 与录制时物理绑定不一致成为可能。对 mip、array layer、depth/stencil aspect、buffer window 或 alias view 的错误绑定，可能不会在 source tests 中失败，却会在真实 backend 产生 data hazard 或 silent corruption。

**必须重构。** 统一 `RenderGraphPhysicalLease`：`ResourceAccessId + ResourceVersion + DeviceGeneration + PhysicalAllocationId + SubresourceRange/BufferRange + UsageState` 必须对 transient、external、persistent、view alias 都可解析；executor API 只接收 lease/view token，不再以 name 作为产品 binding key。name 只能保留给 debug marker、dump 和 editor viewer。所有 external import 要么携带 resolved schema 和 initial/final state，要么 compile fail-closed。

### RG166-P0-003：compiled batch/cursor 不是唯一执行权威，图外工作仍改变图语义

**证据。** packet 同时生成 graph-order batch 与 per-stage index（`execution_packet.rs:143-199`），但产品入口直接遍历固定 stage 数组，并在 post-process 后执行 history copy，再根据 `active_late_graph_stages` 重复选择 late stage（`execute_compiled_scene_graph_stages.rs:27-37,101-236,241-400`）。`execute_graph_stage` 以 `execution_passes_for_stage(stage)` 准备 pass，只有 coverage guard 防止漏执行或重复执行（`execute_graph_stage.rs:356-399`）；它没有消费 `execution_batches()` 来决定 command encoder、queue submit 或 wait/signal。terminal packet 还负责 product copy、diagnostic readback，submission owner 负责 IBL/history/retirement 侧 effect（`terminal_frame_packet.rs:31-68`、`submit_compiled_scene_frame.rs:100-155`）。

**为什么是 P0。** coverage guard 只能发现 stage routing 已经做错，不能使 stage routing 正确。一个新 stage、条件 pass、runtime fallback、history write、readback 或 plugin pass 只要没有被硬编码 orchestration 接入，图就会 compile 但无法完整执行。图外 copy/writeback 也无法参与 culling、lifetime、barrier、capture/replay 和 failure transaction，导致 graph dump 与实际 GPU work 不一致。

**必须重构。** 把 `CompiledRenderGraphExecutionPlan` 作为唯一驱动：plan 产生 ordered execution units，每个 unit 绑定 pass IDs、queue batch、barrier prologue/epilogue、resource leases、optional post/terminal callback 和 completion outputs。scene renderer 只提供 executor registry 与 frame services，不再知道固定 stage 顺序。history copy、surface copy、diagnostic readback、IBL writeback 必须作为 graph node 或显式 graph-owned post-execute node；所有 node 必须参与 coverage、resource lifetime、capture/replay 和 transaction abort。

## 4. P1 重判矩阵

状态定义：`Open` 表示没有可接受的 owner 实现；`Partial` 表示有 source foundation 但缺少跨层 contract、真实 backend 或失败闭环；`Candidate` 只表示已有源码迹象，不能作为完成。

| group | 当前重判 | 仍需重构的工程合同 |
|---|---|---|
| RG166-P1-001..005 builder identity/access | Partial | generation、version token、scope/intent、access id 基础存在，但仍需 collision-free public identity、全资源 range contract 与 compile-to-RHI schema。 |
| RG166-P1-006..008 pass/resource contract | Open | external alias group 仍是字符串；PassFlags 没有 typed completion/scope；attachment contract 没有完整 format/sample/load/store/resolve/subpass约束。 |
| RG166-P1-009..012 compiler artifact | Open | 没有 structural hash/schema/backend compatibility signature、state plan、queue batch/wait/signal、device-limit finalization。 |
| RG166-P1-013..016 culling/schedule | Partial | 拓扑与 culling 可用；runtime predicate/fallback producer、cross-pipeline dependency、async overlap 与 schedule invalidation 仍缺失。 |
| RG166-P1-017..020 lifetime/alias | Partial | interval 和 transient slot 已有；placed heap、aliasing proof、collision-free allocation id、subresource lifetime 仍不完整。 |
| RG166-P1-021..024 persistent/external/pool | Open | persistent registry、buffer equivalent、external final state、memory pressure/eviction/retire policy 未成为 graph-owned contract。 |
| RG166-P1-025..027 product authoring/materialization | Partial | typed buffer foundation、attachment normalization、exact transient device table已出现；资源描述仍由 name inference 驱动，external/persistent/non-compute 未闭合。 |
| RG166-P1-028..032 execution packet/queue | Partial | direct graph index、execution batch report、generic compute 与 coverage guard已出现；packet未携带barrier/fence，batch未被submit owner消费，非compute lease缺失。 |
| RG166-P1-033..034 failure/reuse | Partial | known pre-submit cleanup、device epoch pool invalidation、completion ticket已有；缺 RAII graph transaction、submission/retire/reuse 的统一状态机和 device-loss replay。 |
| RG166-P1-035..037 history/cache/variant | Open | IBL/history/readback writeback仍有图外路径；cache固定容量/同步 miss、variant churn、structural invalidation和artifact retention缺失。 |
| RG166-P1-038..040 diagnostics/recording | Partial | missing-camera typed error、parallel recording与debug marker已有；timing/error receipts、clone-free immutable packet、backend barrier dump/replay不完整。 |
| RG166-P1-041..044 validation/tests | Open | 现有测试以 source shape 和 neutral counts 为主，缺真实 device barrier、external range、multi-queue fence、alias/backend fault matrix。 |
| RG166-P1-045..048 editor/operations | Open | 没有 graph viewer 的 live compiled artifact、resource lease inspection、capture/replay、editor-to-runtime graph edit admission；Tooling 不在本轮实现范围。 |

### 4.1 关键 P1 细项

- **访问模型。** `RenderGraphPassResourceAccess` 只有 name/kind/access/attachment ops（`types.rs:405-411`），不能表达 pipeline stage、read/write mask、subresource aspect、compression、atomic/UAV barrier、clear/discard 语义。需把 access intent 降低为 RHI-neutral state，而不是在 executor 中重新推断。
- **culling。** 当前 root 依赖 present/readback/persistent flags；缺 graph output handle、runtime condition、fallback producer、external access mode 和 post-execute output 的统一 root。Unreal 的 prologue/epilogue sentinel 还承担 barrier 与 extraction root，当前图没有对应结构。
- **transient aliasing。** 当前 lifetime 是拓扑序上的 first/last pass span，alias parent 会扩展父 span；这不等于真实 subresource overlap proof，也不包括 cross-queue fence、heap alignment、memory class、compression/fast-memory constraints。
- **attachment/native pass。** 只有 `load/store`（`types.rs:358-385`），没有 attachment format/sample count/resolve target/feedback/input attachment/merge compatibility；不能实现 Unity native pass merge 或 Godot discardable attachment dependency。
- **queue。** RHI 已能报告 `RenderDeviceQueueTopology::single_serialized_queue()` 并有 logical queue classes，但 graph 不将 logical-to-physical mapping、queue capability、ownership transfer和timeline semaphore写入 compiled artifact。单队列 fallback 可成立，隐式 fallback 不成立。
- **resource pool。** transient pool 已有 device epoch、submission ticket 和 pending retire，但容量/预算仍是固定策略，缺跨 graph residency、memory pressure callback、budget admission、eviction priority、alias heap和跨 frame persistent lease。
- **执行与失败。** `validate_graph_execution` 只能在 stage execution 之后检查 live pass 是否恰好一次；当 command recording、upload enqueue、diagnostic readback 或 IBL writeback失败时，必须让 graph transaction 统一 abort，并生成可重试 receipt，而不是分别清理多个 subsystem。

## 5. P2 重判

| ID | 状态 | 差异 |
|---|---|---|
| RG166-P2-001 | Open | graph structural cache 只有局部 packet/cache 迹象，没有 backend capability/schema key 的持久化 artifact。 |
| RG166-P2-002 | Partial | dump/stats/store lint可用，但缺 barrier/queue/lease/capture schema。 |
| RG166-P2-003 | Open | render-pass merge/subpass/memoryless/discard optimization 缺少设备验证。 |
| RG166-P2-004 | Open | async setup/record task 没有 graph-owned fork/join、deadline、worker fault receipt。 |
| RG166-P2-005 | Open | graph capture/replay 无 immutable resource snapshot、external substitution 和 deterministic packet. |
| RG166-P2-006 | Open | multi-GPU/adapter migration 没有 graph artifact portability policy。 |
| RG166-P2-007 | Partial | GPU timestamps 和 profile records已有，但无法覆盖 barrier wait、queue idle、allocation/alias、map/readback stage。 |
| RG166-P2-008 | Open | learned/telemetry-driven budget、pass workload feedback没有 admission authority。 |
| RG166-P2-009 | Open | debug immediate mode、single-pass isolation、fault injection没有与 compiled graph parity。 |
| RG166-P2-010 | Open | shader reflection/resource schema 与 graph access contract仍分裂，layout change不能原子使 graph artifact失效。 |
| RG166-P2-011 | Open | editor graph viewer、live resource lifetime/alias overlay、execution cursor tracing尚未接入。 |
| RG166-P2-012 | Open | 100K pass/access、multi-camera、multi-surface、long-running pool soak、device-loss recovery没有资格证据。 |

## 6. 与参考引擎的差异

| 参考 | 已具备的工程机制 | Zircon 当前差异 |
|---|---|---|
| Unreal RDG | `FRDGBuilder::Execute` 统一完成 compile/cull/execute；pass parameter struct 驱动 resource lifetime；prologue/epilogue sentinel承载 barrier/extraction；`FRDGSubresourceState`跟踪每个subresource；async compute有依赖与 fence；external access mode有显式切换。见 `RenderGraphBuilder.h:45-48,203-218,248-249,306-317,411-412,456-462`、`RenderGraphResources.h:71-128,617-652`。 | compile 与 product stage execution 分裂；没有 sentinel/barrier/state/fence artifact；external/persistent lease不完整；Async queue只保留metadata。 |
| Unity Graphics | NativePassCompiler 在 compile cache miss 时生成 compiler context，再由 `ExecuteGraph` 执行；resource registry 维护 imported/shared/transient handle、write version、create/release/purge；native render pass自动决定 load/store、merge和break。见 `RenderGraph.Compiler.cs:11-40`、`RenderGraphResourceRegistry.cs:353-463,466-684`、`RenderGraph.cs:338-348`。 | packet有版本和transient table，但没有统一 native pass/compiler context；attachment merge、resource create/release和产品 graph execution仍由多个owner拼接。 |
| Godot RenderingDeviceGraph | 以 `ResourceTracker`、texture subresource range、usage/access bits、normalization/transition/buffer/AS barrier group、command list和secondary command buffer跟踪真实资源；可处理slice与parent dirty list。见 `rendering_device_graph.h:155-207,778-810,817-847`、`rendering_device_graph.cpp:403-601`。 | Zircon有range-aware dependency，但没有等价的运行时 state tracker/barrier group；buffer access非compute仍whole buffer；没有 secondary queue execution contract。 |
| Bevy | root `RenderGraph` schedule明确 `Begin -> Render -> Submit -> Finish`，submit系统集中提交 pending command buffers，render system在同一生命周期处理 screenshot/readback/present。见 `renderer/mod.rs:50-101`。 | Zircon也有 terminal/submission owner，但图外工作没有成为 graph node；stage orchestrator仍隐藏在 scene renderer，packet batch没有唯一Submit系统消费。 |
| Fyrox | GraphicsServer显式区分 `flush` 与 blocking `finish`，并将 command submission/present作为 server contract。见 `server.rs:183-203`。 | Zircon有 nonblocking `SubmissionTicket`，但 Render Graph 没有把 wait/finish policy、readback completion、retire dependency声明为 graph-level operation。 |

参考差异说明：这些实现不是要求逐字复制。共同工程事实是，resource identity、subresource state、physical allocation、pass schedule、barrier/fence、execution callback 和 completion/retirement 必须形成一个可检查的 artifact；名称、stage 数组和统计报告只能是 debug/presentation projection。

## 7. 重构路线

### M0：冻结现状并阻止继续扩散

- 保留当前 builder/version/culling/transient 基础，但禁止新 executor 使用 `*_by_name` 作为 physical binding key。
- 在 compile 输出中加入 `artifact_schema_version`、source graph fingerprint、device/profile compatibility key；旧 packet 不能静默复用。
- 将 Runtime89 三个 source-only 修复标为 candidate，补齐 device/backend acceptance owner，不再以 source tests 关闭 P0。

### M1：统一 logical access 与 physical lease

- 定义 `ResourceAccessId`、`ResourceVersion`、`SubresourceRange/BufferRange`、`DeviceGeneration`、`PhysicalAllocationId` 的不可变 lease。
- transient/external/persistent/view alias 全部通过同一 resolver；declaration name 仅用于 diagnostics。
- external import 必须携带 schema、initial state、final state、ownership、required/report-only policy；缺任一项 compile fail-closed。

### M2：增加 device-qualified sync/lifetime compiler

- 生成 per-access state transition、barrier group、queue batch、wait/signal、ownership transfer、alias acquire/release、attachment merge decision。
- 使用 device profile 的 queue topology、format/usage、limits、transient support、sparse support 和 memory class 做 admission。
- 将 whole-resource lifetime 改为 subresource/range lifetime，并以 alignment、heap class、compression、cross-queue fence做 alias proof。

### M3：让 execution plan 成为唯一产品驱动

- `execute_compiled_scene_graph_stages` 改为通用 plan interpreter；scene renderer只注册 executor和 frame services。
- `FrameCommandEncoderSet`升级为 queue-aware command recording plan，按 compiled batch 产生 native packet，而不是按固定 stage拼接。
- history copy、terminal copy、diagnostic readback、IBL writeback、surface present都建模为 graph-owned node/post-execute output，并参与 coverage/culling/lifetime/failure transaction。

### M4：统一 submission/completion/retirement

- graph transaction在 record、upload enqueue、submit、completion、readback、retire各阶段有明确状态和可重试 receipt。
- 将 transient pool retire、persistent lease、readback staging、query frame、surface present、device-loss cancellation绑定到同一 submission graph ticket。
- 多物理队列使用 timeline semaphore/fence；单队列设备必须显式生成 serialization fallback 证据。

### M5：可验证性能与工具接入

- 增加 null backend、WGPU真实设备、RenderDoc capture、barrier validation、external range fault、alias overlap、multi-queue fence、device loss、replay parity 测试。
- 记录 compile time、packet cache hit、cull count、barrier count/bytes、queue wait/signal、transient peak/alias saving、record CPU、submit CPU/GPU、readback latency。
- Runtime166 不实现 Tooling；未来 Editor graph viewer 只能消费 compiled artifact 和 execution receipt，不能另造图 authority。

## 8. 资格门

| gate | 当前 | 通过条件 |
|---|---|---|
| G1 logical identity | Partial | 所有 executor 只用 stable access/version token，name lookup仅debug。 |
| G2 resource schema | Partial | texture/buffer/external/view alias的shape/usage/format/range完整且版本化。 |
| G3 culling/schedule | Partial | runtime predicate、fallback、external access、post node都进入同一根图。 |
| G4 barrier/state | Fail | 每个live access有backend-neutral before/after state和barrier batch。 |
| G5 queue sync | Fail | queue mapping、wait/signal、ownership transfer和single-queue fallback可验证。 |
| G6 lifetime/alias | Partial | subresource/range、heap alignment、cross-queue fence和memory pressure证明完整。 |
| G7 materialization | Partial | transient/external/persistent统一 physical lease，设备能力在compile阶段拒绝。 |
| G8 execution authority | Fail | plan interpreter完全取代固定stage orchestration，live pass恰好一次。 |
| G9 completion/retirement | Partial | submit/readback/present/retire/device-loss共享ticket状态机。 |
| G10 backend validation | Fail | WGPU真实设备、command validation、fault、capture/replay通过。 |
| G11 performance | Fail | 在相同场景与设备上提供 compile/record/GPU/memory 基线及回归阈值。 |
| G12 editor handoff | Fail | Editor viewer读取Runtime artifact，不复制编译或物理资源authority。 |

## 9. 本轮交付与限制

- 本轮只做当前源码 review、参考引擎对照、索引与覆盖记录，不修改 production Rust、tests、Cargo、ABI 或 Tooling。
- 进行了文件级静态扫描、source shape/line/byte/test marker 统计和参考源码定位；没有运行 Cargo、真实 WGPU device、RenderDoc、跨队列 GPU、fault、replay、scale、soak 或 benchmark。
- Runtime89 的 direct pass identity、SparseReserved admission、typed storage schema 等改动被记录为 source-only candidate；在 M2-M4 完成前，不得将 Runtime89 或 Runtime166 的 P0 标记为 Closed。
- 当前工作树存在其他会话的 dirty/untracked 变化，本报告 fingerprint 只代表本轮冻结瞬间；实施前必须重取选择集、hash、Cargo 状态和 owner manifest。

