---
title: Runtime Render Graph Builder、Compiler、Resource Lifetime、Pass Culling、Transient Aliasing、Barrier、Queue Scheduling、Execution 当前工作树复核
category: zircon_runtime
report_id: Runtime214
review_date: 2026-09-01
baseline_head: 5798051603e7f7f565538125c9aba96d5beabae2
baseline_epoch: 2026-09-01
verification_head: working-tree
verification_epoch: 2026-09-01
supersedes_currentness_of:
  - zircon_runtime/166-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-current-source-review.md
related_owner_reports:
  - zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md
  - zircon_runtime/91-runtime-material-shader-module-graph-permutation-compiler-reflection-layout-pipeline-pso-cache-prewarm-hot-reload-product-integration-current-source-review.md
  - zircon_runtime/213-runtime-visibility-gpu-scene-culling-batching-instancing-hzb-virtual-geometry-current-working-tree-review.md
  - zircon_editor/270-editor-scene-viewport-runtime-render-scene-visibility-hzb-picking-surface-product-integration-current-working-tree-review.md
related_code:
  - zircon_runtime/src/render_graph
  - zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline.rs
  - zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs
  - zircon_runtime/src/graphics/pipeline/async_compile.rs
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
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderGraphBuilder.h
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/RenderGraphBuilder.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderGraphResources.h
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/RenderGraphPass.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraph.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraph.Compiler.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphResourceRegistry.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Editor/RenderGraphTests.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/Compiler/CompilerContextData.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/Compiler/NativePassCompiler.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/Compiler/PassesData.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/Compiler/ResourcesData.cs
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

# Runtime214: Render Graph Builder / Compiler / Lifetime / Culling / Aliasing / Barrier / Queue / Execution 当前工作树复核

- 复核日期：2026-09-01
- 复核 HEAD：`5798051603e7f7f565538125c9aba96d5beabae2`
- 复核类型：review-only；未修改 Rust、Cargo、ABI、tests、shader 或产品 UI，也未运行 Cargo、真实 GPU、RenderDoc、device-loss、multi-queue、soak 或 benchmark。
- 当前约束：MVP 仍未通过；本文只允许先收口 identity、schema、state/barrier、lifetime、execution authority 等基础合同，不授权 native-pass optimizer、多适配器或高级调度抢跑。
- Tooling：按用户要求排除；本轮没有查询、轮询、等待或实时跟踪协调器。

## 1. 冻结范围与证据

本轮逐文件复核 Render Graph builder/compiler/access/lifetime/transient plan、compiled pipeline packet/cache、graph execution resource materialization/resolver/pool、compiled scene stage/history/terminal/submission owner及 RHI submission slice；同时执行 authoring call-site、consumer、test-marker、early-return、barrier/queue/device-loss 与 ignored performance 负扫描。

| 范围 | files | lines | bytes | tests | ignored | `include_str!` | `.contains(` | device-fixture early return | fingerprint |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---|
| `zircon_runtime/src/render_graph` | 45 | 15,734 | 568,064 | 147 | 3 | 19 | 92 | 0 | `fb658f97ecb9b7b7c375c18b0803b623088c852d2e046f627dee94b02b9a11c3` |
| compiled pipeline declaration/cache/packet | 8 | 2,208 | 85,241 | 25 | 3 | 4 | 35 | 0 | `91ac03e33bf67f08fc2ec9eed9652a79ac2f77f96fe3f26143b2de3caf858b62` |
| graph execution/materialization/resolver/pool | 82 | 26,735 | 1,003,412 | 254 | 5 | 89 | 352 | 22 | `b7dd547abd0862e7fdbc9cdd6e01a9d282e387867302901938a8c60dcf0a2043` |
| compiled scene/stage/history/terminal/submission | 56 | 12,440 | 500,875 | 120 | 1 | 80 | 242 | 17 | `a0cf25c1eb611224d395f66584e922adb854de8dd67913282132f7cf58822322` |
| RHI submission/device queue slice | 8 | 2,980 | 109,553 | 6 | 0 | 6 | 41 | 0 | `bc1177627a5643f62f148beb6845064f6b8931755bb1bb9f653fcbd43b64de94` |
| de-duplicated union | **199** | **60,097** | **2,267,145** | **552** | **12** | **198** | **762** | **39** | `cada4815353c03d484e48aaa2a9ea41df2fc577c83aa8e9ce98b5e77a14ccb42` |
| 五引擎参考切片 | **17** | **24,197** | **1,063,317** | n/a | n/a | n/a | n/a | n/a | `5cae5413afb7ecae456b393eb13ca93ff445a34ee1fdeda0e4d60876823094f1` |

参考 revision：Bevy `fb89a864...`、Fyrox `8d815db3...`、Godot `8c7e6c58...`、Unity Graphics `a7e4c051...`；Unreal 镜像由 `Build.version` 冻结为 6.0.0 / UE5 / changelist 0，并计入参考 aggregate fingerprint。

这是共享 working tree 的冻结快照，不是 clean-HEAD 或 GPU 验收收据。实现开始前必须重算选择集 fingerprint，并重查 graph/access schema、compiled packet、resolver、pool、history/terminal work 与 submission consumer。

## 2. 总结论

Zircon 当前 Render Graph 已经不是空壳。generation-scoped pass/resource identity、resource version、exact access ID、texture subresource/buffer range dependency、RAW/WAW/WAR、version-aware culling、collision-free transient allocation ID、exact transient/external/persistent-texture access binding、completion-ticket pool retirement、compiled pass cursor、bounded async compile worker和并行 command recording都是真实可保留底座。Runtime166 对 external/persistent exact binding、allocation ID、pool budget/completion 和 packet authority 的部分描述已经过时，必须降为 `Partial` 而不是继续写成完全缺失。

但它仍不是工程级、设备可执行的 Render Graph compiler。`CompiledRenderGraph` 只有 pass/access/lifetime/allocation/statistics，没有 per-access before/after state、barrier batch、queue wait/signal、ownership transfer、initial/final state、alias acquire/release或completion edge（`render_graph/graph.rs:48-60,105-127`）。`QueueLane` 和 execution batch只形成连续逻辑分组，源码注释仍把 transition/encoder grouping称为“future work”，产品最终忽略 `batch.queue()`，把全部 scene command buffer交给一次 graphics submission（`execution_packet.rs:34-53`、`execute_graph_stage.rs:398-446`、`submit_compiled_scene_frame.rs:102-109`）。

访问合同也没有真正硬切。`RenderGraphResourceAccessIntent::Legacy` 明示为兼容 whole-resource API，validator直接返回成功（`resource_access_intent.rs:3-12`、`builder/access_validation.rs:185-194`）。对 `zircon_runtime/src` 实际工作树全部 **9,020** 个 Rust 文件重算后，六类 broad authoring API 共 **680** 次，而六类 typed/range-aware API只有 **41** 次；因此 exact access table不能弥补 authoring truth 的缺失。executor 仍大量通过 `texture_view_by_name` / `buffer_by_name` 解析，Unknown report-only external还被设计为回退 legacy name路径（`resource_resolver.rs:130-181,381-400`）。

产品执行链比 Runtime166 有进展：cursor现在能阻止 live pass重复、遗漏和乱序，stage executor也按 compiled batch枚举。但固定 early/forward/scene/post/late stage编排仍决定服务路由，scene clear、history copy、viewport product copy、diagnostic readback与history initialization仍在图外编码（`execute_compiled_scene_graph_stages.rs:99-314`、`terminal_frame_packet.rs:28-69`）。所以 graph dump/culling/lifetime/barrier/capture/failure transaction依然不能描述实际全部 GPU work。

本轮沿用并刷新 Runtime166 canonical finding，不新增重复 ID：P0 为 **1 Open / 2 Partial / 0 Closed**；48 项 P1 细化后为 **24 Open / 24 Partial / 0 Closed**；12 项 P2 为 **8 Open / 4 Partial / 0 Closed**；16 道资格门为 **9 Fail / 7 Partial / 0 Pass**。当前没有任何证据支持“性能或表现优于 Unreal”。

## 3. 当前 Authority 链

```text
RenderGraphBuilder
  -> validate/admit/dependency/cull/topological order/lifetime/transient slot
  -> CompiledRenderGraph
       pass + access/version + lifetime + allocation + stats
       [缺 state/barrier/fence/physical queue/completion]
  -> RenderGraphExecutionPacket
       stage projection + contiguous QueueLane batches + cursor
  -> fixed Scene stage orchestrator
       graph pass recording
       + graph-external clear/history/copy/readback/init
  -> FrameCommandEncoderSet
  -> one graphics submission service
  -> SubmissionTicket
  -> transient pool completion collection/history publication
```

目标 authority 必须是：

```text
sealed logical graph + device profile + external initial/final leases
  -> immutable device-qualified execution artifact
       exact access state transitions
       native pass/queue units
       barrier batches + wait/signal + ownership transfer
       subresource lifetime + physical lease + alias acquire/release
       graph-owned prologue/epilogue/terminal outputs
  -> generic plan interpreter
  -> queue-aware submission transaction
  -> completion/readback/present/retirement receipt
```

## 4. Runtime166 P0 重判

### RG166-P0-001：Compiled artifact 没有设备级 barrier、队列同步与完成契约

**状态：Open。** `CompiledRenderPass`只有 declared/effective queue、dependency、flags和resource rows；`CompiledRenderGraph`没有任何 state transition或sync字段。全选择集对 `barrier` 的有效命中只有一处 future-work注释，`queue_wait`、`queue_signal`、`multi_queue`和`device_loss`没有对应 graph compiler/qualification测试。Unreal RDG在 compile期建立 prologue/epilogue barrier pass、transition batch和async graphics fork/join fence；Unity NativePassCompiler把 `waitOnGraphicsFencePassId`、`insertGraphicsFence` 与跨队列 lifetime extension写入 compiler context并在执行时实际创建/等待 fence。Zircon的 queue count/report不等于上述合同。

**必须重构。** 在 `CompiledRenderGraph` 与 RHI之间增加 device-qualified lowering：每个 exact access产生 backend-neutral state、pipeline/stage/access、queue family、before/after、discard/load/store语义；编译器输出 barrier batch、queue wait/signal、ownership transfer、single-queue fallback evidence和completion dependencies。Runtime09A/90拥有 RHI/native queue实现，本报告拥有 graph-to-RHI lowering artifact与唯一消费。

### RG166-P0-002：Exact binding 有进展，但 Legacy/name/whole-resource 仍是产品主路径

**状态：Partial。** 新增 `RenderGraphResourceAccessId`、versioned access key、exact transient access table、typed external access bindings和persistent texture access bindings，generic compute也能消费 exact compute binding packet；这些应保留。问题是公共/product resolver仍以名字为入口，并在 exact transient/external未命中时调用 declaration级 physical lookup；Unknown report-only external明确绕回 legacy。persistent只有texture等价物，没有persistent buffer统一合同。大量 deferred、mesh、OIT、postprocess、surface、particle、velocity和history executor仍调用 `require_*_by_name`。

**必须重构。** authoring阶段只允许 `ResourceAccessId + Version + Intent + Range`；transient/external/persistent/view alias全部物化成统一 `PhysicalResourceLease`。executor只能由compiled access token取 texture view/buffer slice；name仅保留诊断。Unknown import不能进入可执行图，report-only资源必须是独立diagnostic sideband，不得享有隐式物理解析。

### RG166-P0-003：Cursor 已约束 pass 顺序，但 execution plan 仍不是唯一产品权威

**状态：Partial。** `RenderGraphExecutionCursor`与stage batch枚举现在能证明 live compiled pass恰好一次、保持graph order，这是实质进展。仍未闭合的是：stage数组决定具体服务；`scene_clear.record_frame_clear`在pass前执行；history copy在PostProcess后由固定函数插入；terminal阶段追加viewport product copy、diagnostic readback，并把history initialization command buffer插到索引0。`CompiledHistoryEpiloguePlan`虽能选exact writer，输出映射仍依赖固定resource name，copy本身仍不在图内。

**必须重构。** 用通用 plan interpreter替换固定stage orchestration。clear、history、surface/product copy、readback、IBL writeback、present、initialization必须成为普通graph node或typed graph-owned prologue/epilogue unit，统一参与culling、lifetime、barrier、capture、failure和completion。scene renderer只注册executor与frame service，不再手工知道最终pass顺序。

## 5. P1 逐项重判

| ID | 状态 | 当前工作树差异与重构要求 |
|---|---|---|
| RG166-P1-001 | Partial | pass/resource generation identity存在，但仍是builder局部身份；编译artifact需携带source graph generation与schema version。 |
| RG166-P1-002 | Partial | resource version与explicit input version可查询；所有legacy latest-resource访问必须硬切为显式producer token。 |
| RG166-P1-003 | Partial | exact `RenderGraphResourceAccessId`已进入compiled/access binding；公开executor仍未把它作为唯一key。 |
| RG166-P1-004 | Partial | texture subresource与buffer range能参与dependency；Legacy仍退化whole-resource，所有资源类型没有统一range schema。 |
| RG166-P1-005 | Partial | texture view alias能投影到parent scope；alias group仍不是稳定typed identity，alias链/ownership/lease未闭合。 |
| RG166-P1-006 | Open | external alias group仍为`Option<String>`；改为generation-qualified alias domain与compile-time collision/admission。 |
| RG166-P1-007 | Open | `PassFlags`只有allow_culling/has_side_effects；缺condition、fallback、async eligibility、completion/output/extraction语义。 |
| RG166-P1-008 | Open | attachment只有load/store；缺format/sample/resolve/input-feedback/memoryless/subpass/native-pass compatibility。 |
| RG166-P1-009 | Open | compiled graph无artifact schema version、canonical structural hash和serialized compatibility contract。 |
| RG166-P1-010 | Open | cache capability fingerprint只是局部摘要；artifact无device generation、完整limits/formats/queue topology/backend key。 |
| RG166-P1-011 | Open | access intent没有lower为resource state plan；Legacy还绕过usage/type-intent validation。 |
| RG166-P1-012 | Open | format/view/usage/limits在materialization期仍可能失败；必须在device-qualified compile完成最终admission。 |
| RG166-P1-013 | Partial | RAW/WAW/WAR、manual dependency、topological order和版本culling较完整；需形成统一schedule artifact。 |
| RG166-P1-014 | Partial | present/readback/persistent与side-effect可做root；缺typed extraction/output/terminal root。 |
| RG166-P1-015 | Open | 没有runtime predicate、conditional pass、fallback producer与skip时的版本映射。 |
| RG166-P1-016 | Open | queue lane不产生async overlap window、fork/join、physical mapping或schedule invalidation。 |
| RG166-P1-017 | Partial | transient first/last interval与alias parent extension存在；仍是whole logical resource拓扑span，不是subresource/queue-time lifetime。 |
| RG166-P1-018 | Partial | allocation ID已经collision-free，旧报告必须纠偏；但ID还不表示heap offset/memory class/device generation。 |
| RG166-P1-019 | Open | texture allocation bucket漏掉`view_formats`，materializer却要求`view_format_bits`相等，可能compile复用后在materialize期拒绝（`transient_allocation.rs:434-516`、`transient_materialization.rs:225-237`）。 |
| RG166-P1-020 | Open | 无placed heap、alignment、memory type、compression/fast memory、alias acquire/release和cross-queue alias proof。 |
| RG166-P1-021 | Partial | persistent texture有exact per-access backing/view；它仍借用pool acquire class，不是跨帧graph registry。 |
| RG166-P1-022 | Open | 没有persistent buffer等价物与统一persistent lease/version/retirement。 |
| RG166-P1-023 | Partial | typed external texture/buffer校验exact range且required缺失会fail；initial/final state、ownership和release handoff仍缺。 |
| RG166-P1-024 | Partial | pool有device epoch、256 MiB texture/64 MiB buffer预算、8-frame stale策略、ticket completion和LRU式eviction；无动态GPU budget/pressure/admission/priority。 |
| RG166-P1-025 | Open | broad authoring API词法调用680次、typed调用41次；必须禁止新增Legacy并按模块迁移现存作者。 |
| RG166-P1-026 | Partial | exact transient/external/persistent texture表真实存在；non-compute executor与Unknown external仍走name/declaration fallback。 |
| RG166-P1-027 | Partial | materializer能复核physical descriptor并报告；compile/materialize分裂和`view_formats`键缺失证明admission还不封闭。 |
| RG166-P1-028 | Partial | contiguous live queue batch与batch report存在；batch仍只是metadata/statistics。 |
| RG166-P1-029 | Open | 没有logical-to-physical queue mapping、queue capability或明确single serialized queue plan。 |
| RG166-P1-030 | Open | 没有timeline/binary wait-signal、ownership transfer、queue idle/overlap receipt。 |
| RG166-P1-031 | Partial | cursor验证恰好一次并由stage executor消费；固定stage service routing仍不是通用plan interpreter。 |
| RG166-P1-032 | Open | graph-external clear/history/product copy/readback/init改变真实work，compiled artifact无法完整复现。 |
| RG166-P1-033 | Partial | known pre-submit清理、pool terminal/error fail-close已有；缺统一RAII graph transaction和可重试abort receipt。 |
| RG166-P1-034 | Partial | submission ticket驱动pool reuse与device epoch discard；readback/present/history/device-loss没有共用状态机。 |
| RG166-P1-035 | Partial | history epilogue能按exact writer/access选择source；固定name映射和图外copy仍需收口。 |
| RG166-P1-036 | Partial | graph cache key覆盖pipeline revision、quality/frame/capability fingerprint；固定capacity 16、同步miss、O(n) eviction且无artifact persistence。 |
| RG166-P1-037 | Partial | bounded single compile worker有dedup、panic containment、targeted finish；无cancel/deadline/device-generation stale-result拒绝，Drop会join。 |
| RG166-P1-038 | Partial | stats、batch report、pool report、markers存在；缺state/barrier/queue/lease dump与stable artifact diff schema。 |
| RG166-P1-039 | Partial | serial/parallel recording与顺序合并存在；per-pass dependency/resource/record数据仍有clone，未证明steady-frame allocation ceiling。 |
| RG166-P1-040 | Open | 没有统一compile/record/submit/completion error receipt与barrier wait/queue idle/alias memory timing。 |
| RG166-P1-041 | Partial | dependency/culling/range/external/pool测试覆盖不少算法分支；大量仍是源码形状和neutral CPU测试。 |
| RG166-P1-042 | Open | 39处device fixture在backend不可用时early return，不能形成CI强制真实设备矩阵。 |
| RG166-P1-043 | Open | 没有真实barrier、multi-queue fence、ownership、device-loss和external final-state资格测试。 |
| RG166-P1-044 | Open | 12项performance/scale/evidence测试全部ignored；没有checked-in回归阈值或同场景Unreal对比。 |
| RG166-P1-045 | Open | Editor没有读取live immutable compiled artifact的graph viewer。 |
| RG166-P1-046 | Open | 没有包含external substitution、resource snapshot、queue/barrier和deterministic packet的capture/replay。 |
| RG166-P1-047 | Open | Editor/plugin graph edit没有schema/capability/owner generation admission与安全发布事务。 |
| RG166-P1-048 | Open | 无artifact retention/version migration/lease inspection/cursor trace/production incident导出闭环；Tooling不在本轮实施范围。 |

状态统计：**24 Open / 24 Partial / 0 Closed**。`Partial`只表示可保留源码底座，不代表跨层合同或真实设备验收通过。

## 6. P2 重判

| ID | 状态 | 当前差异 |
|---|---|---|
| RG166-P2-001 | Partial | cache已有较完整frame/capability key，但compiled artifact自身无schema/backend signature、磁盘持久化或跨进程复用。 |
| RG166-P2-002 | Partial | dump/stats/store lint可用，仍缺barrier/queue/lease/capture schema。 |
| RG166-P2-003 | Open | native render-pass merge/subpass/memoryless/discard/resolve optimizer未实现。 |
| RG166-P2-004 | Partial | async compile与parallel record存在，缺graph-owned fork/join、cancel/deadline和worker fault transaction。 |
| RG166-P2-005 | Open | capture/replay没有immutable resource snapshot、external substitution和deterministic execution packet。 |
| RG166-P2-006 | Open | multi-GPU/adapter migration没有artifact portability与lease relocation policy。 |
| RG166-P2-007 | Partial | GPU timestamps/profile records存在，不能覆盖barrier wait、queue idle、allocation/alias、map/readback阶段。 |
| RG166-P2-008 | Open | telemetry-driven memory/workload budget没有compile admission authority。 |
| RG166-P2-009 | Open | immediate/single-pass isolation/fault injection没有compiled graph parity。 |
| RG166-P2-010 | Open | shader reflection/layout schema与graph access contract仍分裂，layout change不能原子使artifact失效。 |
| RG166-P2-011 | Open | editor graph viewer、lifetime/alias overlay与execution cursor trace未接入。 |
| RG166-P2-012 | Open | 100K pass/access、multi-camera/surface、pool soak和device-loss recovery无资格证据。 |

状态统计：**8 Open / 4 Partial / 0 Closed**。

## 7. 测试、故障与性能证据

| 证据面 | 当前事实 | 结论 |
|---|---|---|
| 源码形状测试 | union中198个`include_str!`、762个`.contains(`；热点文件单个可达59个include与173个contains | 可防止局部接线回退，但不能验证Rust语义、GPU command或设备行为 |
| 真实设备fixture | 39处“设备不可用即return”，集中在materialization、external binding、compute pipeline等 | 环境缺设备时会把关键验证降为静默通过，不能作为CI gate |
| 同步/队列/fault | 有效barrier只有future注释；queue wait/signal、multi-queue、device-loss graph test为0 | P0-001没有动态证据，queue report不能代替fence验证 |
| 性能/规模 | 12个ignored：direct write、short-circuit、identity lookup、indirect args、borrow、MRU、compute export、workload p95、executor hash、dependency move、graph scale、compile projection | 没有always-on ceiling、趋势或同机对照，不能宣称性能领先 |
| 正向基础 | 依赖/culling/range、exact external/persistent binding、pool ticket/eviction、cursor coverage有单元测试 | 保留为M0-M3回归底座，并升级到真实backend matrix |

## 8. 参考引擎差异

| 参考 | 可迁移的工程机制 | Zircon 当前差异 |
|---|---|---|
| Unreal RDG | prologue/epilogue sentinel同时承担graph边界barrier、extraction与cull root；per-subresource state、barrier begin/end dependency、transition create queue和async compute graphics fork/join fence均在compile/execution authority内（`RenderGraphBuilder.h:458-493,713-729,787-844`、`RenderGraphBuilder.cpp:1319-1678,1806-2206`、`RenderGraphPass.cpp:217-437`）。 | 无sentinel/state/barrier/fence artifact；clear/history/terminal work在图外；logical QueueLane没有物理消费者。 |
| Unity Graphics | `NativePassCompiler`先merge native pass，再计算first/last use与cross-queue fence；async资源释放延长到graphics waiter，执行期实际Create/Wait fence、Begin/EndRenderPass、Create/ReleasePooledResource（`NativePassCompiler.cs:153-157,616-732,856-1010,2086-2212`、`PassesData.cs:135-156,293-425,493-509,600-688`）。 | attachment合同不足以merge；lifetime不含queue fence；资源物化/释放与compiled schedule不是一个context。 |
| Godot RenderingDeviceGraph | `ResourceTracker`保存usage/access、texture subresource/parent dirty状态；recorded command携带normalization/transition/buffer/AS barrier index，执行前分组并实际调用pipeline barrier（`rendering_device_graph.h:155-213,778-843,854-885`、`.cpp:366-603,1371-1462`）。 | Zircon虽有range-aware dependency，却没有运行时state tracker或barrier group；说明即使采用轻量路线也未达到显式同步下限。 |
| Bevy | Render schedule显式分为Render/Submit/Finish；pending encoder可并行finish但保持topological order，submit owner集中提交，screenshot/readback/present有明确生命周期位置（`renderer/mod.rs:50-101`、`render_context.rs:25-139,167-298`）。 | Zircon已有集中submission，但graph pass与图外terminal work不共享同一plan；queue batch也不控制submit。 |
| Fyrox | `GraphicsServer`把flush、blocking finish与swap/present作为明确server contract（`server.rs:103-203`）。 | Zircon的ticket比该接口更先进，但graph artifact没有声明wait/finish/readback/retire policy，能力未贯通。 |

参考代码不是要求逐字移植。共同最低事实是：logical identity、access/state、physical lease、schedule、barrier/fence、execution callback与completion必须形成一个可检查、可重放、设备限定的不可变artifact；名称、stage数组和统计只能是projection。

## 9. 重构路线

### M0：停止 Legacy 与源码形状测试继续扩散

- 新authoring API必须提交typed intent/range/version，CI禁止新增六类broad访问调用和executor `*_by_name`。
- 给compiled artifact增加schema version、source graph fingerprint、device/profile compatibility key；cache miss/hit都输出typed receipt。
- 修复transient texture bucket漏掉`view_formats`的问题，并加入compile-time不兼容拒绝测试。
- 把39处device fixture early return改为明确Skipped/Unavailable收据；required backend suite中Unavailable必须失败。

### M1：统一 Logical Access 与 Physical Lease

- 定义唯一 `ResourceAccessId + ResourceVersion + AccessIntent + Range + OwnerGeneration`。
- transient/external/persistent texture/persistent buffer/view alias统一物化到`PhysicalResourceLease`；删除执行期name fallback。
- external import必须声明schema、initial/final state、ownership、required policy；release/export进入graph epilogue。

### M2：Device-qualified State / Barrier Compiler

- 根据device profile把access intent降低为pipeline/stage/access/layout/state。
- 编译per-subresource/range transition、barrier begin/end batch、UAV/alias barrier与prologue/epilogue sentinel。
- 对unsupported format/usage/view/limit在compile阶段fail-close，不把兼容性错误推迟到materialization。

### M3：Lifetime、Heap 与 Memory Budget

- lifetime从whole logical resource span升级为subresource/range与queue-time interval。
- 引入heap class/alignment/offset、alias acquire/release、compression/fast-memory约束和cross-queue proof。
- pool消费动态device memory budget/pressure，统一persistent/transient/readback/query residency与completion retirement。

### M4：Native Pass 与 Queue Schedule

- 完整attachment schema驱动native render-pass merge、subpass/resolve/memoryless/load-store decision与break audit。
- 编译logical-to-physical queue mapping、async overlap window、wait/signal、ownership transfer和显式single-queue fallback。
- 每个schedule decision必须有可dump原因，不允许driver/executor临时重新推断。

### M5：Execution Plan 成为唯一产品驱动

- 通用interpreter消费ordered execution unit、barrier prologue/epilogue、lease与executor ID。
- 删除固定stage执行authority；stage只作为diagnostic tag或service registration projection。
- clear/history/product copy/readback/IBL/present/init全部纳入graph-owned node或terminal unit。

### M6：Submission / Completion / Failure Transaction

- record、enqueue、submit、present、completion、readback、history publish、retire共享一个transaction与typed receipt。
- 所有失败路径统一abort/rollback/retry policy；device loss取消stale lease、cache artifact和pending compile。
- 多队列用timeline/fence receipt；单队列也输出serialization evidence，不能靠隐式实现细节。

### M7：资格、诊断与 Editor Handoff

- 建立真实WGPU command validation、barrier oracle、multi-queue/fallback、external state、alias fault、device-loss矩阵。
- 将ignored scale/perf证据升级为managed benchmark：compile/record/submit/GPU/memory/queue wait均有固定场景、设备、阈值和趋势。
- Runtime只发布immutable artifact/dump/capture；Editor viewer消费它，不复制compiler、physical resource或execution authority。

M0-M3是当前MVP允许的先行基础。M4-M7只能在identity/schema/state/lifetime闭合后按依赖顺序推进。

## 10. 资格门

| Gate | 当前 | 通过条件 |
|---|---|---|
| G01 identity/version | Partial | 所有pass/resource/access/version跨builder、cache、executor、receipt稳定且代际可拒绝。 |
| G02 exact authoring | Fail | 产品无Legacy broad access，name只用于diagnostic。 |
| G03 resource schema | Partial | texture/buffer/external/persistent/view alias的shape/usage/range/initial-final state完整。 |
| G04 dependency/culling | Partial | runtime condition、fallback、extraction、terminal output都进入同一图。 |
| G05 state/barrier | Fail | 每个live access都有device-qualified before/after state与实际消费的barrier batch。 |
| G06 queue sync | Fail | physical mapping、wait/signal、ownership、overlap与single-queue fallback可验证。 |
| G07 attachment/native pass | Fail | resolve/subpass/memoryless/merge/break decision由compiler生成并有device test。 |
| G08 lifetime/alias | Partial | subresource/range、heap alignment、cross-queue fence、memory pressure proof完整。 |
| G09 physical lease/pool | Partial | transient/external/persistent/buffer/view统一lease并completion-qualified回收。 |
| G10 execution authority | Partial | cursor基础已在；还需plan interpreter覆盖全部graph与terminal work。 |
| G11 submission/completion | Partial | ticket和pool retirement已有；readback/present/history/failure需同一transaction。 |
| G12 failure/device loss | Fail | compile/record/submit/device-loss取消、恢复、stale拒绝与重试receipt通过。 |
| G13 backend validation | Fail | required WGPU matrix不允许early-return伪通过，barrier/queue/external/alias测试齐全。 |
| G14 performance | Fail | always-on ceiling与同场景同设备对照证明目标达成。 |
| G15 diagnostics/capture | Fail | artifact dump/diff、barrier/queue/lease、capture/replay与incident export闭合。 |
| G16 editor handoff | Fail | Editor只读Runtime immutable artifact，viewer/edit admission不复制authority。 |

汇总：**9 Fail / 7 Partial / 0 Pass**。

## 11. 本轮交付与限制

本文只刷新 Runtime166 的当前性并给出依赖有序重构路线；canonical ID仍由 `RG166-*` 唯一记账。没有修改实现，也没有把任何源码迹象误记为关闭。未运行Cargo是有意的：本轮是docs-only review，且真实设备、multi-queue、RenderDoc、fault、soak与benchmark才是后续动态资格证据，普通`cargo test`不能替代它们。

当前最先需要实施的不是增加新render feature，而是M0：禁止Legacy/name authority继续扩散、修正transient descriptor compatibility key、建立artifact schema/device key和强制backend test receipt。只要P0-001仍Open，任何“Render Graph已工程化完成”或“性能优于Unreal”的结论都不成立。
