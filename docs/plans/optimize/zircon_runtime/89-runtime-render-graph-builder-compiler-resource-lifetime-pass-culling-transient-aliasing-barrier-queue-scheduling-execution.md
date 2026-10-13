---
title: Runtime Render Graph Builder、Compiler、Resource Lifetime、Pass Culling、Transient Aliasing、Barrier、Queue Scheduling、Execution 与 Product Integration 当前源码工程化差距
category: zircon_runtime
report_id: Runtime89
review_date: 2026-08-21
baseline_head: be5a281c96b6dc9d33b5c9d0f2699a8bf75afcf1
baseline_epoch: 336
related_code:
  - zircon_runtime/src/render_graph
  - zircon_runtime/src/graphics/pipeline/compiled_graph_cache.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset
  - zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene
  - zircon_runtime/crates/zr_rhi
  - zircon_runtime/crates/zr_rhi_wgpu
tests:
  - zircon_runtime/src/render_graph/tests
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile_tests
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/materialization/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_graph_execution_record/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_executor_registry/tests
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_core_render_compiled_scene/render/tests.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/performance/02/2026-08-15-render-graph-current-architecture-review.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
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
---

# Runtime Render Graph Builder、Compiler、Resource Lifetime、Pass Culling、Transient Aliasing、Barrier、Queue Scheduling、Execution 与 Product Integration 当前源码工程化差距

## 1. 结论

当前Render Graph已经不是“按pass插入顺序执行”的空壳。核心builder具备generation-scoped pass/resource handle、逻辑resource version、RAW/WAW/WAR依赖推导、Load/Store/Discard校验、version-aware culling、external alias group、descriptor-keyed transient slot、dump/stats/store lint与compute workload metadata；产品路径也能materialize真实WGPU texture/buffer、按拓扑层进行CPU并行command recording，并通过单次`queue.submit`提交有序command buffer集合。这些底座应保留。

旧`09A`中的三项事实已经过时：foreign builder handle当前会被generation拒绝；同名逻辑资源的写入已经产生version ordinal；产品authoring不再无条件给所有pass串接`previous -> pass`依赖。Runtime89按当前working tree纠正这些结论，不把它们继续登记为缺陷。

但从“图能compile”到“图一定可以在选定设备上按编译结果正确执行”仍有三条新增P0。第一，pass name只在初始feature descriptors上校验唯一；late-forward、transmission与IBL等生成pass在校验后加入，核心builder又允许重名。执行器按name线性反查并取第一个pass，名称碰撞会让第一个pass重复执行、真正的生成pass永远不执行。第二，SparseReserved texture可compile、被allocation/materialization跳过，并被validation计为完整；WGPU backend却明确不支持并拒绝创建 sparse texture，执行到资源解析才失败。第三，插件storage texture没有typed descriptor/format合同；未知名称被默认推断成`Rgba8UnormSrgb`，materializer静默剥离`STORAGE_BINDING`，generic compute直到执行才拒绝该格式。三者都是“编译成功但执行合同不成立”，不能归为长期优化。

本报告新增登记 **3项P0、48项P1、12项P2和48个资格门**。Runtime09A继续唯一拥有通用RHI barrier、native queue、GPU completion与backend lifetime；Runtime77/79及各具体render feature报告继续拥有shader/pipeline与算法质量；本篇只拥有Render Graph定义经过final normalization、编译成device-qualified immutable execution packet，并由产品执行器完整消费的闭环，不重复累计父finding。

## 2. 审查边界、currentness与证据

### 2.1 冻结语料

| 范围 | 文件 / 行 / 非空行 / bytes / test markers | 证据等级 | fingerprint |
|---|---:|---|---|
| Render Graph core production | **8 / 3,659 / 3,372 / 130,450 / 4** | E3逐文件读取builder、compiler、graph、types、error、dump、store lint | `87a894c9aa9966d3351677d07d23d312ff523d2b87886cf5444178719aefd6df` |
| Render Graph core focused tests | **10 / 2,626 / 2,423 / 90,709 / 58** | E3核对handle、cycle、ordering、version、culling、alias、compute与scale意图 | `4e1a47650d9d6d375cf47c3a6e8638a0660419d023058ce631f7ad1a0db50ab4` |
| Product compile、materialization与execution | **108 / 30,891 / 28,889 / 1,151,567 / 274** | E3逐文件结构扫描并读取compile、cache、resource lookup、pool、encoder、stage、submit与关键tests | `23e4cb23632d63f05a52e9c4b7a4d423cd18cc2d4def7265cec822ae195eda63` |
| 五引擎参考切片 | **36 / 34,455 / 29,123 / 1,504,933 / 128** | E2/E3读取RDG/RenderGraph compiler、subresource、barrier、fence、pool、tests与较低层旁证 | `209cbf8ee478b207b4ac28af7e01d0dad807445945b5c2e3e9bfcba613191282` |

fingerprint按normalized lowercase relative path排序，串联`path + NUL + lowercase per-file SHA-256 + LF`后再取SHA-256；test markers按`#[test]`与独立`#[ignore]`属性合计。冻结集合在审查时没有非本轮dirty路径；它代表2026-08-21共享working tree，不是只读HEAD、ABI或验收receipt。

Git基线为`be5a281c96b6dc9d33b5c9d0f2699a8bf75afcf1`，coordinator baseline epoch为336。Godot、Fyrox、Bevy与Unity Graphics参考revision分别为`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`与`a7e4c051d256a781ab362c64316b125a1e104694`；Unreal镜像无独立`.git`，由参考aggregate fingerprint冻结。

### 2.2 证据限制

- E3能证明当前静态控制流、数据合同与测试意图；本轮没有执行Cargo、真实GPU、RenderDoc、device loss、multi-queue、fault、soak或benchmark。
- `dev/Graphics`是Unity Graphics package源码，足以审查其RenderGraph Runtime/compiler/tests，但不能代替Unity引擎全部native backend。
- Bevy当前render graph更接近ECS schedule + pending command buffers，Fyrox没有同级RDG；两者只作最低产品提交/抽象旁证，目标架构以Unreal RDG与Unity RenderGraph为主。
- Godot `RenderingDeviceGraph`主要是resource usage/barrier command graph，不负责高层pass culling/transient alias；本报告只使用其subresource/barrier证据。
- Editor全树搜索只发现RenderGraph导航占位字符串，没有真实viewer/diagnostic consumer。用户要求暂不审查未来迁移为Rust的tooling，本篇不扩展到tooling实现。

### 2.3 当前产品链

```text
RenderFeatureDescriptor / ComputePassDescriptor
  -> filter / replacement / generated pass insertion
  -> infer resources by string name
  -> RenderGraphBuilder
     -> whole-resource read/write + queue metadata + flags
  -> compile
     -> logical versions + RAW/WAW/WAR dependencies
     -> culling + topological pass vector
     -> whole-resource lifetimes + exact-descriptor transient slots
  -> CompiledRenderPipeline
     -> CompiledRenderGraph
     -> separate pass_stages / stages vectors
  -> frame binding + transient materialization
     -> logical name -> physical texture/view/buffer
  -> hard-coded stage dispatcher
     -> pass_stages filtered by stage
     -> graph pass linearly found by name
     -> serial or CPU-parallel WGPU command recording
  -> one WGPU queue.submit(command_buffers)
  -> begin readback maps
  -> return transient backings to per-renderer pool
```

编译器产出的dependency和version尚未成为唯一执行合同：resource resolver按name和read/write类别取整个物理resource；stage dispatcher仍决定产品顺序；`QueueLane`只保留声明/回退诊断，所有command buffer最终进入同一个WGPU queue。

## 3. 既有owner、过时结论与继承边界

| owner / 历史结论 | 当前状态 | Runtime89处理 |
|---|---|---|
| Runtime09A：foreign handle可跨builder误用 | **已修复**：pass/texture/buffer/external handle都携带builder generation | 只记录currentness，不重开finding |
| Runtime09A：资源无逻辑version | **已修复**：每次write提升ordinal，read绑定latest ordinal并区分execution/culling adjacency | 保留底座；指出per-access/subresource执行合同仍缺 |
| Runtime09A：产品pass被无条件串成单链 | **已修复**：authoring不再注入全局previous dependency，compiler推导hazard | 保留底座；真实queue schedule仍继承阻塞 |
| Runtime09A | RHI barrier/state transition、multi-queue、GPU completion、device loss与backend lifetime | 继续唯一拥有backend能力；Runtime89只定义compiler输出和product consumption要求 |
| Runtime79 / 77及具体render feature报告 | PSO/shader cache、shader reflection、具体SSR/HZB/TAA/GI算法与质量 | Runtime89只要求typed resource/pass contract，不复制算法finding |
| Editor Render Graph viewer | 当前没有产品consumer，仅有导航占位 | Runtime89登记Runtime diagnostic product handoff；具体Editor UX另立owner |

## 4. 当前实现中应保留的底座

### 4.1 Builder作用域与基础校验

`RenderPassId`、`RgTextureHandle`、`RgBufferHandle`和`ExternalResource`均携带builder generation；foreign/unknown handle会返回typed error。资源名会在compile时校验唯一，compute binding会校验binding number、resource kind/access、mip range、indirect offset与per-pixel local size。这比依赖vector index或字符串静默取值可靠。

### 4.2 逻辑version、hazard与culling已开始分层

compiler按访问顺序维护latest writer、readers-since-last-write和version ordinal，推导RAW、WAW与WAR。execution adjacency保留物理hazard，culling provenance只在Load依赖旧值时保留前写者，因此clear覆盖旧版本可删除无用producer。present/readback/persistent/side-effect形成cull roots；这部分不应退回手工dependency链。

### 4.3 Transient allocation有确定性可复用轮廓

texture和buffer按完整descriptor key分bucket，再按不重叠lifetime复用slot；allocation plan、slot reservations、dense/sparse virtual bytes与alias report可观测。产品pool按descriptor key复用真实WGPU对象，并设texture/buffer预算与CPU frame retention。它尚未达到heap/placed resource allocator，但比每帧无界创建可靠。

### 4.4 产品command recording并非完全串行

`ParallelEncoderSet`用compiled dependencies计算topology layers，只对registry声明parallel-safe且无mutable owner的pass启用Rayon recording；返回结果保持bucket topology order。`FrameCommandEncoderSet`在parallel bucket前flush serial prefix，最后形成有序command buffer列表。这是真实CPU recording并行，应与未来GPU queue scheduling分开保留。

### 4.5 诊断与测试已有可扩展入口

graph dump保留pass、resource、version、lifetime、alias、queue与compile-work；execution record保留实际/声明queue、stage、dependency、resource、compute dispatch、profile与materialization report。测试覆盖handle、cycle、ordering、culling、external alias、transient slot、compute metadata与materialization。目标应把这些升级为compiler artifact/receipt，而不是删除后重新做日志。

## 5. 参考实现差异与适用边界

| 引擎 | 本地源码证据 | 对Zircon的最低要求 | 不应误读 |
|---|---|---|---|
| Unreal RDG | texture subresource layout/range覆盖mip、plane、array slice；compile先cull/merge，再编译prologue/epilogue barrier；async compute建立fork/join overlap、跨pipeline transition/fence并延长lifetime；transient allocator按allocation/deallocation fence处理alias acquire/discard | subresource是hazard/lifetime/barrier的一等公民；queue scheduling、barrier和allocation必须来自同一个compiled plan；transient reuse受GPU完成证据约束 | 不照搬Unreal宏、RHI对象层次或历史API；借鉴编译阶段与不变量 |
| Unity Graphics RenderGraph | `ResourceHandle`有index/type/version/execution validity；compiler执行Validate/Build/Cull/native pass merge/lifetime/sync/memoryless/compact；version data限制每version一个writer；PassData保存wait/insert fence及create/destroy range；tests覆盖async lifetime、fence、transient locality、handle validity、merge和no-allocation | 每次执行generation必须使旧handle失效；logical version、first/last use、queue fence和physical create/destroy形成单一packet；测试必须覆盖异步与allocation correctness | Unity package的native compiler策略不必逐字段移植，特别是平台特定memoryless规则 |
| Godot RenderingDeviceGraph | tracker以mip/layer subresource range记录usage，生成texture/buffer/memory/AS barrier，处理command buffer pool与semaphore | 即使backend抽象较高，也必须有显式subresource usage与可检查的barrier/sync结果 | 它不是高层RDG，不可用来替代culling、version或transient planner |
| Bevy | pending command buffers保持render schedule拓扑顺序，可并行finish encoder后统一submit；另有截图/readback附加submit | 中央command-buffer owner、确定顺序与CPU并行是最低产品线 | Bevy当前render graph不是versioned RDG，也不能证明单物理submit或multi-queue能力 |
| Fyrox | object-safe GraphicsServer和异步read buffer有schedule/running/try_read生命周期 | backend/readback必须有显式异步状态，不能用同步假接口 | 没有同级RDG，不作为目标compiler架构主证据 |

主导参考是Unreal RDG的subresource/barrier/async/transient闭环与Unity RenderGraph的versioned compiler/execution/tests。Godot用于证明显式barrier graph的最低线；Bevy只用于CPU recording与提交owner；Fyrox只作异步资源生命周期下界。

## 6. P0正确性阻断

### `RG89-P0-001`：最终pass集合未校验唯一，name-based执行会把重名生成pass替换成第一个同名pass

**确定性证据链：**

1. `validate_feature_descriptors`在compile入口对初始`asset_descriptors`校验pass name唯一；随后才执行particle、late-forward、transmission、replacement与half-resolution插入。
2. `author_environment_ibl_bake_passes`又在graph pass authoring之后追加`env.ibl_prefilter.mipN`、`env.ibl_irradiance_sh`和`env.ibl_irradiance_cube`，没有对最终pass集合重新校验。
3. `RenderGraphBuilder::add_pass*`与core compiler不拒绝duplicate pass name；`CompiledRenderGraph`只按`RenderPassId`建立唯一索引。
4. `CompiledRenderPipeline`另外保存`pass_stages: Vec<{pass_name, stage}>`，`from_parts`不校验它与graph pass的一一对应。
5. `execute_graph_stage`逐条遍历`pass_stages`，再对`graph.passes()`执行`.find(|pass| pass.name == stage_entry.pass_name)`，重名时永远返回第一个pass。
6. 因此插件只要使用某个后生成的保留名称，pipeline仍可compile；执行时第一个插件pass会被重复准备/执行，真正IBL或生成pass不会被执行。parallel路径还会让相同graph index覆盖prepared index，不能自动纠正。

**产品后果：** 环境烘焙artifact可在图和diagnostic中存在，却没有真实dispatch；错误pass可能重复写资源或执行side effect。执行record按name看起来仍“有该pass”，形成false green。

**必须修复：** final normalization完成所有replacement/insertion/IBL expansion后，生成唯一`CompiledPassKey`并一次性校验name/key/ID/stage/executor的exact bijection。execution packet直接保存pass index/key和executor payload，不得按字符串反查。保留名称必须进入统一namespace；冲突compile fail closed。

**2026-08-21 M0切片：** `RenderGraphBuilder::compile`现对最终pass集合返回typed `DuplicatePassName`；产品authoring把builder生成的`RenderPassId`写入每条stage metadata，IBL expansion消费graph plan返回的pass ID；stage execution与sprite stage选择按ID索引并防御性核对名称。2,048 pass release gate使用21组交替采样，要求direct-ID P95不高于legacy name scan的25%。实现与文档已完成，Cargo/性能数值和独立review仍等待分组协调器验证，因此本项尚未标记accepted，M0另外两项P0也未完成。

### `RG89-P0-002`：SparseReserved可被判为materialization完整，但当前产品backend没有任何可执行backing

**确定性证据链：**

1. core test明确构造8192x8192 sparse texture并断言`builder.compile()`成功、`sparse_texture_slot_count == 1`且dense slot为0。
2. allocation plan主动排除SparseReserved；`should_materialize_texture_lifetime`也对sparse返回false，因此不创建或绑定WGPU texture/view。
3. `validate_materialized_graph_resources`遇到sparse只增加`sparse_texture_reservation_count`，不增加required/missing texture；`materialized_resources_complete()`因此可返回true。
4. `RgResourceResolver`和GPU lookup仍按logical name要求texture view，真实pass执行时只能返回“resource is not bound”。
5. `zr_rhi::Capabilities::default`把`supports_sparse_texture`设为false，WGPU capability test断言false；`zr_rhi_wgpu::validate_texture_desc`也明确拒绝unsupported sparse residency。
6. Graph compile没有接收device capability，也没有Unsupported/Fallback/ExternalSparseProvider disposition；cache key虽记录`sparse` capability，compiler不消费它。

**产品后果：** compile/materialization telemetry可以全绿，直到具体pass访问资源才失败；若pass被cull或测试只检查stats，缺失backend永远不暴露。未来virtual texture/terrain可建立在不可执行合同上。

**必须修复：** SparseReserved必须由device-qualified compile admission决定：有真实SparseResidencyService时输出page table/heap/bind plan与provider lease；无能力时选择明确dense fallback或返回typed Unsupported错误。materialization complete必须要求每个live access都有resolveable physical binding，不得把“虚拟统计项”算作完成。

### `RG89-P0-003`：插件storage texture缺少typed descriptor，默认sRGB推断导致compile成功、执行期必然拒绝

**确定性证据链：**

1. `pipeline_graph_resources`只聚合resource name、Texture/Buffer/External、read/write与minimum buffer bytes，不保存texture format、dimension、extent class、mips、samples或usage capability。
2. 有write的plugin texture会变成transient；`author_graph_resources`仅按name调用`texture_desc_for`。
3. 未命中内建名称规则时，`texture_desc_for`默认`Rgba8UnormSrgb`，同时把所有非depth texture声明为`STORAGE | COPY_DST`。
4. `write_storage_texture`与compute metadata validation只核对logical kind/access，不核对format是否storage-capable或选定device format features。
5. materializer的`wgpu_texture_usages`对不支持的format静默不加入`STORAGE_BINDING`，没有返回错误。
6. generic compute的`compute_storage_texture_format`只接受R32Float/Rgba8Unorm/Rgba16Float/Rgba32Float，执行到pipeline layout创建才返回“不支持format”；现有WGPU product test使用显式外部`Rgba8Unorm`，没有覆盖产品compiler推断出的transient plugin output。

**产品后果：** 合法的ComputePassDescriptor/插件feature可以通过asset compile、graph compile、cache与materialization，进入frame后才失败；自定义executor若绕过generic guard，还可能把缺少STORAGE usage的view交给WGPU validation。

**必须修复：** `RenderFeatureResourceDescriptor`必须引用typed `RenderResourceSchema`，显式给出format class、extent policy、dimension、mip/layer/sample、usage与fallback。final compiler结合device format capabilities验证每个access，materializer不得静默剥离声明usage。未知资源schema必须compile fail，不再按字符串猜格式。

## 7. P1工程化差距（48项）

### 7.1 Builder、handle与访问模型（P1-001至P1-008）

| ID | 当前差距 | 目标合同 |
|---|---|---|
| RG89-P1-001 | source-only foundation: every builder access now freezes a texture subresource range, buffer byte range, or explicitly unresolved report-only external scope; exact texture/buffer/external access APIs and descriptor-bound mip/layer/aspect/buffer-window validation exist, but product lowering still emits legacy whole-resource scopes and no physical view/slice is created | `TextureSubresourceRange { mip, layer, aspect/plane }`与`BufferRange`进入每次access |
| RG89-P1-002 | source-only validation foundation: non-legacy intent now validates read/write direction, non-empty shader stages, texture/buffer class, and declared RHI usage including copy; feature/shader-binding lowering, per-subresource barrier/state output, and runtime diagnostics remain absent | typed `ResourceAccessIntent`同时驱动validation、barrier与diagnostic |
| RG89-P1-003 | source-only repair: builder compile rejects duplicate `(pass, logical resource, read/write)` rows and compiled accesses now carry stable `{ pass, access ordinal }` IDs; legacy lookup indices remain resource+kind and no canonical overlapping-subresource conflict model exists | per-access stable ID；禁止歧义duplicate或逐access保存version/range |
| RG89-P1-004 | source-only canonical scope/version slice: stable `{ pass, access ordinal }` owns metadata, produced/input versions, dump rows, and live compiler-order allocation rows; texture cells, split buffer intervals, alias-parent range projection, overlap-only RAW/WAR/WAW/cull roots, and complete token coverage are compiler facts. Same-kind duplicates remain rejected until multi-range authoring is coherent; no device binding table exists | pass close时canonicalize/validate access set并给出typed conflict |
| RG89-P1-005 | source-only foundation: versioned writes yield a builder-scoped `RenderGraphResourceVersionToken`; explicit reads and attachment Loads consume that token, compiler resolves it to an immutable compiled input-version index and rejects foreign/stale/unavailable values. Access IDs and range metadata now exist, but version tokens and legacy lookup still do not select a versioned physical view/slice | SSA-style read/write handle或显式version token，禁止“latest by visit order”成为隐含合同 |
| RG89-P1-006 | external alias group是字符串+resource type，不能描述subresource overlap、ownership或simultaneous access | typed external identity、view range、initial/final state与owner lease |
| RG89-P1-007 | `PassFlags`只有allow_culling/has_side_effects，side effect无类型、scope或completion | typed side-effect/output class、conditional execution与receipt |
| RG89-P1-008 | attachment ops附着在generic write上，缺color/depth/stencil/resolve/render-area等pass contract | explicit render attachment set、resolve target、load/store per aspect与render pass descriptor |

### 7.2 Compiler pipeline、culling与schedule（P1-009至P1-016）

| ID | 当前差距 | 目标合同 |
|---|---|---|
| RG89-P1-009 | compile是单文件过程，没有可检查的Normalize/Validate/Version/Hazard/Cull/Schedule/Allocate/Barrier阶段artifact | immutable compiler IR与逐阶段diagnostic/metrics |
| RG89-P1-010 | graph无canonical structural hash、schema/version或backend compatibility signature | stable graph key包含definition、schema、capability与compiler version |
| RG89-P1-011 | compiler不输出resource state transition或barrier batch | per-subresource prologue/epilogue transition plan，交由Runtime09A backend lowering |
| RG89-P1-012 | `QueueLane`只保存resolved/declared metadata，不输出queue batch、wait/signal或ownership transfer | device-qualified graphics/compute/copy schedule与timeline dependencies |
| RG89-P1-013 | async pass不会建立fork/join overlap，也不因跨queue同步延长resource lifetime | overlap region分析、cross-queue first/last use与fence-qualified lifetime |
| RG89-P1-014 | 没有compatible render pass/native pass merge、subpass或memoryless candidate | attachment-compatible merge plan与backend capability decision |
| RG89-P1-015 | culling只有静态root，没有运行时conditional pass、fallback producer或predicate propagation | compile/runtime predicate contract，所有分支保持producer完整 |
| RG89-P1-016 | compile validation不接设备limit/format feature；workgroup、dispatch和buffer alignment多到执行期才检查 | `DeviceCompileProfile`进入final compile，错误定位到pass/access/schema |

### 7.3 Lifetime、alias、persistent与materialization（P1-017至P1-024）

| ID | 当前差距 | 目标合同 |
|---|---|---|
| RG89-P1-017 | exact descriptor bucket只复用完整相同desc，不能规划heap/placed allocation或兼容class | physical allocation class、offset/alignment、alias acquire/discard与heap budget |
| RG89-P1-018 | product slot key只保存64-bit `bucket_key_hash + slot`，不保存完整兼容key；hash碰撞可合并独立slot | collision-free allocation ID或连同完整key复核，禁止hash作为correctness identity |
| RG89-P1-019 | texture collision fallback按descriptor拆开，buffer却合并size/usage；两者都不能证明原allocation interval互斥 | materializer严格消费compiler physical allocation ID和interval proof |
| RG89-P1-020 | `mark_persistent`设flag/cull root并从transient alias plan排除；frame materialization已有graph-owned persistent texture backing与exact access lease，但尚无跨帧generation registry | persistent resource key、generation、resize/recreate、import/export与failure policy |
| RG89-P1-021 | graph-owned persistent texture现按exact access ID建立frame lease并由logical resource去重WGPU handle；sparse/provider-owned texture及persistent buffer仍缺等价完整产品合同 | typed persistent lease与frame-to-frame version transition，不按name碰运气 |
| RG89-P1-022 | SSR coarse mip alias由固定资源名硬编码，graph本身不知道它是parent mip 1 | graph-declared view/subresource alias，删除产品特例 |
| RG89-P1-023 | pool按CPU frame age回收，缺submission/completion ticket、device generation与全局memory pressure | fence-qualified retire queue、device-loss invalidation与共享GPU budget owner |
| RG89-P1-024 | pool使用固定256 MiB texture/64 MiB buffer和8帧retention，未按adapter budget、viewport或quality调节 | budget policy、pressure telemetry、admission/eviction与OOM fallback |

### 7.4 Product authoring、compiled packet与execution（P1-025至P1-032）

| ID | 当前差距 | 目标合同 |
|---|---|---|
| RG89-P1-025 | resource descriptor由name contains/match推断，未知buffer还按pixel count猜size | typed resource schema/catalog；名称只作debug label |
| RG89-P1-026 | source-only repair: final feature normalization now resolves every attachment write to `Clear` or `Load`; internal `Load` carries and validates the preceding producer token, external initial contents remain an explicit `Load`, and authoring no longer owns a global `produced_texture_resources` set; managed validation is still absent | 每version显式initialization/Load provenance，由compiler验证 |
| RG89-P1-027 | source-only compiler + device-table foundation: every live access has a dense compiler-order `{ versioned key, optional transient physical allocation }` row keyed by exact access ID; transient materialization creates canonical views/slices, graph-owned persistent textures use a separate resource-deduplicated access-ID lease, and typed external rows use their own lease packet. Aliases project to parent allocation; sparse/provider-owned persistent rows and a small whole-resource compatibility surface remain | consumer直接绑定producer version/subresource handle |
| RG89-P1-028 | `CompiledRenderPipeline`同时保存`graph`、`stages`和`pass_stages`三份执行事实，`from_parts`不验bijective/order | 单一immutable `RenderGraphExecutionPacket`，stage是pass metadata而非第二authority |
| RG89-P1-029 | stage执行每pass扫描`graph.passes()`并比较String | compiled pass index/key直接寻址，steady frame不做名称搜索 |
| RG89-P1-030 | stage入口现按immutable compiled batches遍历并以`RenderPassStage`作服务路由过滤，但early/lighting/scene/post/history/late仍由产品函数分段调度，queue/barrier与epilogue尚未下沉 | 产品只遍历compiled batches；history/readback也建成graph epilogue pass |
| RG89-P1-031 | source-only partial hard cut: transient generic-compute bind-group及动态 `FromBuffer`/`PerPixel` dispatch已消费compiler exact-ID packet；resolver-backed non-compute texture/buffer lookup消费typed external/transient lease；GI/SSR/HZB/volumetric history epilogue冻结最终live writer access并通过transient或graph-owned persistent exact texture lease执行。unknown report-only external、persistent buffer及少量直接helper仍保留兼容路径 | per-access binding table把logical version/range映射到view/buffer slice |
| RG89-P1-032 | dependency只进入context/record与CPU partition，不形成backend transition/fence执行 | execution严格消费compiler barrier/queue packet并产生completion receipt |

### 7.5 Lifecycle、cache、diagnostic与热路径（P1-033至P1-040）

| ID | 当前差距 | 目标合同 |
|---|---|---|
| RG89-P1-033 | begin_frame后bind/materialization/execution/readback encode多条错误return不release backing或end_frame | RAII frame transaction，Success/Abort都回收、推进health并终结readback/timer |
| RG89-P1-034 | submit后立即把backing返回pool，只因当前单queue顺序通常安全；未来multi-queue不成立 | submission completion token控制reuse，跨queue取max completion |
| RG89-P1-035 | IBL writeback在graph command buffers之后临时append，未进入graph dependency/lifetime | readback/writeback epilogue进入compiled packet并持有resource lease |
| RG89-P1-036 | cache固定16项、同步miss closure、无single-flight/prewarm/compile budget | generation-aware concurrent cache、background/prewarm、deadline与admission |
| RG89-P1-037 | cache key包含精确view/render尺寸，resize/DRS可产生同步compile churn | descriptor class/extent specialization策略与bounded variant policy |
| RG89-P1-038 | source-only repair: cache-key/fingerprint construction rejects a missing selected camera with a typed error and the frame submit boundary preserves the pipeline identity in `GraphCompileFailure`; managed validation is still absent | typed precondition/error，调用方持有validated frame compile input |
| RG89-P1-039 | execution record在steady frame clone大量String、resource vectors与debug markers | interned IDs、optional capture budget、ring/streaming telemetry |
| RG89-P1-040 | dump/store lint/bandwidth ledger基于compiled metadata与base surface估算，不核对实际barrier/store/submit | compiler artifact + backend receipt联合诊断，exact mip/layer/sample/format bytes |

### 7.6 Tests、产品证据与Editor handoff（P1-041至P1-048）

| ID | 当前差距 | 目标合同 |
|---|---|---|
| RG89-P1-041 | core scale tests主要断言counter；p50/p95 timing test被ignore且只打印 | CI benchmark artifact、阈值、回归比较与固定硬件profile |
| RG89-P1-042 | 多项测试用`include_str!`检查源码字符串，不能证明runtime行为 | behavioral/fault/product tests取代source-shape assertion |
| RG89-P1-043 | 没有最终生成pass名称碰撞、pass table bijection与每live pass恰执行一次测试 | adversarial namespace + execution coverage oracle |
| RG89-P1-044 | sparse测试只断言slot统计；没有unsupported backend admission或真实provider执行测试 | Unsupported/Fallback/SparseProvider三路product test |
| RG89-P1-045 | 没有subresource disjoint/overlap、mip/layer hazard、alias acquire/discard测试 | property/model tests对照reference hazard oracle |
| RG89-P1-046 | 没有multi-queue fork/join、fence、async lifetime extension与completion reuse测试 | deterministic queue simulator + real backend conformance |
| RG89-P1-047 | 没有bind/materialize/record/readback/submit各失败点的frame transaction fault matrix | exhaustive failure injection，断言无pool/readback/timer/lease泄漏 |
| RG89-P1-048 | Editor没有graph viewer、pass/resource/barrier/alias/queue/culled原因产品consumer | Runtime发布versioned diagnostic artifact；Editor另建viewer owner |

## 8. P2长期能力（12项）

| ID | 长期能力 | 目标 |
|---|---|---|
| RG89-P2-001 | automatic async compute placement | 基于依赖、occupancy、historical timings和queue capability选择重叠区域 |
| RG89-P2-002 | multi-GPU / device group graph | 显式node mask、peer transfer与device-local allocation，不污染单GPU核心合同 |
| RG89-P2-003 | graph specialization compiler | 按stable structural key生成PSO/resource/batch specialization并持久缓存 |
| RG89-P2-004 | learned transient budget tuning | 根据场景、分辨率和历史峰值调节heap/page policy，仍服从硬预算 |
| RG89-P2-005 | render graph capture/replay | 脱离游戏逻辑重放definition、packet、resource snapshot和submission receipt |
| RG89-P2-006 | deterministic graph simulator | 无GPU验证version、culling、hazard、barrier、queue、lifetime与failure disposition |
| RG89-P2-007 | backend differential validation | WGPU/Vulkan/DX12/Metal对同packet输出可比较barrier与结果证据 |
| RG89-P2-008 | temporal resource residency forecasting | 多帧图预测persistent/transient峰值和streaming压力 |
| RG89-P2-009 | automatic pass fusion/splitting | 在shader/attachment/queue成本模型证明收益时做fusion或拆分 |
| RG89-P2-010 | shader-access reflection reconciliation | shader reflection与declared resource access自动差分，防止漏报/过报 |
| RG89-P2-011 | fleet graph telemetry | 聚合graph key、compile cost、alias efficiency、queue overlap与failure，无高基数失控 |
| RG89-P2-012 | competitive RDG benchmark suite | 同内容同硬件比较CPU compile/record、GPU frame、VRAM峰值与正确性 |

## 9. 目标架构与hard cut

### 9.1 目标组件

```text
RenderFeature / Plugin / Builtin declarations
  -> RenderResourceSchemaCatalog
  -> RenderGraphDefinitionBuilder
     -> builder-generation handles
     -> explicit version + subresource/range access
  -> RenderGraphNormalizer
     -> expand generated passes
     -> final namespace + bijection validation
  -> RenderGraphCompiler
     -> Version SSA / Hazard Graph
     -> Cull + Conditional/Fallback resolution
     -> Native pass merge candidates
     -> QueueSchedule { batches, waits, signals, fork/join }
     -> LifetimePlan { logical versions, subresources }
     -> PhysicalAllocationPlan { heap/slot/offset/alias transitions }
     -> BarrierPlan { prologue/epilogue, ownership transfer }
  -> DeviceQualifiedRenderGraphPacket
     -> pass table + executor payload
     -> per-access physical bindings
     -> queue/barrier/allocation plan
     -> diagnostic artifact + structural key
  -> RenderGraphFrameTransaction
     -> persistent/transient/sparse providers
     -> record/submit/readback/writeback
     -> completion-qualified release + terminal receipt
```

Runtime09A/RHI负责把BarrierPlan、QueueSchedule和PhysicalAllocationPlan lowering到具体backend；Runtime89负责保证它们来自同一个compiler truth，并且产品执行器不再另造stage/name/resource顺序。

### 9.2 必须维持的不变量

1. compile成功意味着选定device/backend上每个live access都有可执行binding、usage和queue disposition；Unsupported必须在compile前终结。
2. 每个pass有稳定唯一key；final packet中definition、stage、executor、dependency与执行覆盖严格一一对应。
3. resource version和subresource range是hazard、culling、lifetime、alias、barrier与execution lookup的共同identity。
4. queue overlap不得缩短lifetime；跨queue资源只有在全部signal完成后才能alias/reuse。
5. persistent、transient、external与sparse各有明确provider/lease，不用同名预导入或“统计存在”冒充资源存在。
6. compiler packet是唯一执行authority；产品stage、history、readback与writeback不能绕过它重排工作。
7. graph frame无论Success/Abort都产生terminal receipt并释放/retire所有资源、readback、timer与external lease。
8. diagnostic来自compiler artifact和backend receipt，可证明“声明、编译、实际提交”三者一致。

### 9.3 Hard cut清单

- 删除steady-frame按`pass_name`扫描compiled graph；迁移到stable pass key/index后直接删除旧路径。
- 删除`graph + pass_stages + stages`三authority；stage降为compiled pass metadata，不保留compat双执行面。
- 删除name-based resource descriptor推断作为plugin contract；所有自定义资源必须引用typed schema。
- 删除materializer静默剥离usage；unsupported format/usage在device-qualified compile失败或选择显式fallback。
- 删除“SparseReserved计数即materialized complete”；没有provider/fallback时不得产生可执行packet。
- 删除hash-only materialization slot identity与SSR名称特例，改用compiler allocation/view ID。
- 删除CPU frame age作为GPU reuse资格；完成票据接管retire/reuse。
- 不用“所有queue最终单queue提交”继续模拟async完成；无native queue能力时明确编译为Graphics fallback并报告。
- 不在Runtime89复制Runtime09A的backend实现，也不为旧API留下`pub use`、compat module或旁路facade。

## 10. 依赖有序重构里程碑

| Milestone | 内容 | 退出条件 |
|---|---|---|
| M0 三项P0封口 | final pass namespace/bijection、sparse admission、typed storage format validation | 三条adversarial product路径compile fail或正确执行，无execution-time surprise |
| M1 Typed definition hard cut | resource schema、stable pass/access key、explicit version/subresource/range | plugin/builtin不再按名称猜desc，不再按read/write折叠access |
| M2 Compiler IR与hazard | staged IR、SSA version、subresource RAW/WAW/WAR、culling predicate | model/property tests证明hazard/culling与reference oracle一致 |
| M3 Pass merge与queue schedule | render pass merge candidate、fork/join、wait/signal、async lifetime | simulator与真实backend都能证明order/overlap/fallback |
| M4 Physical allocation | persistent registry、transient heap/slot/offset、sparse provider、alias transition | memory plan可复验，collision/resize/device-loss/failure闭合 |
| M5 Barrier与backend lowering | prologue/epilogue、ownership transfer、alias acquire/discard | Runtime09A各backend conformance通过，无隐式未声明hazard |
| M6 Execution packet hard cut | 删除stage/name双authority，history/readback/writeback进入packet | 每个live pass恰执行一次，实际submit覆盖与packet一致 |
| M7 Frame transaction、cache与diagnostic | completion release、abort RAII、single-flight/prewarm、artifact/receipt | fault matrix无泄漏，cache/diagnostic有预算和generation |
| M8 产品与性能资格 | Editor diagnostic handoff、跨backend、规模、fault、soak、竞争benchmark | correctness gates全绿后再证明CPU/GPU/VRAM优势 |

M0可在不实现native multi-queue前完成；M3-M5必须与Runtime09A协作。M1是后续所有render feature plugin工程化的前置合同，不能用更多名称特例延后。

## 11. 资格门（48项）

### 11.1 P0与final normalization（G01至G08）

| Gate | 必须证明 |
|---|---|
| G01 | 初始、replacement、particle、late-forward、transmission、half-res与IBL生成pass进入同一次final唯一性校验 |
| G02 | 任意pass name/key碰撞都在compile返回typed error，frame执行不开始 |
| G03 | final pass table、executor table、stage metadata与compiled graph是exact bijection |
| G04 | 每个live pass在serial/parallel路径都恰执行一次，culled pass零次 |
| G05 | unsupported sparse device在compile得到Unsupported或明确dense fallback |
| G06 | sparse provider路径为每个live access提供binding/page plan并进入completion lifecycle |
| G07 | unknown plugin storage texture schema不得默认sRGB后继续compile |
| G08 | format/usage不兼容在device-qualified compile失败，materializer不静默删usage |

### 11.2 Version、subresource与culling（G09至G16）

| Gate | 必须证明 |
|---|---|
| G09 | read绑定明确producer version，不依赖未声明的visit-order latest |
| G10 | mip/layer/aspect不重叠访问不产生false hazard，重叠访问产生正确RAW/WAW/WAR |
| G11 | buffer offset/size range参与hazard、binding与bounds validation |
| G12 | duplicate/conflicting access有typed disposition，不被HashMap首项折叠 |
| G13 | clear覆盖旧version可cull旧producer，Load/partial range保留必要producer |
| G14 | external alias view按identity+range验证，错误type/overlap/ownership fail closed |
| G15 | conditional pass/fallback分支下每个consumer都存在唯一有效producer |
| G16 | graph dump逐access显示version、range、intent和cull reason，与compiler IR一致 |

### 11.3 Queue、barrier与lifetime（G17至G24）

| Gate | 必须证明 |
|---|---|
| G17 | graphics/compute/copy schedule包含batch、wait、signal、fork/join及fallback reason |
| G18 | cross-queue producer/consumer生成ownership transfer和正确fence |
| G19 | async region内资源lifetime延长到join，不能提前alias/reuse |
| G20 | per-subresource state transition覆盖first use、pass边界与final external state |
| G21 | alias前有discard/release，后有acquire/initialization，不读取前resource残值 |
| G22 | compatible render pass merge不改变load/store/resolve、timestamp和debug语义 |
| G23 | 无native async能力时所有pass显式fallback且执行/声明queue telemetry一致 |
| G24 | simulator和真实backend对同packet的dependency/barrier/fence顺序可比较 |

### 11.4 Allocation、persistent、sparse与budget（G25至G32）

| Gate | 必须证明 |
|---|---|
| G25 | physical allocation ID不依赖可碰撞hash，完整compatibility/alignment被复核 |
| G26 | overlapping lifetime永不共享同一physical interval，disjoint lifetime可证明复用 |
| G27 | persistent resource跨frame保持identity/version，resize/recreate有generation与fallback |
| G28 | external/persistent/transient/sparse provider lease均有明确owner和terminal release |
| G29 | SSR等mip alias由subresource view声明，不存在名称硬编码materialization分支 |
| G30 | pool reuse只在全部相关queue completion后发生，device loss使旧backing失效 |
| G31 | adapter budget/pressure下admission、eviction、degrade或OOM均有typed receipt |
| G32 | allocation report精确统计mip/layer/sample/format、heap/offset、peak与alias savings |

### 11.5 Execution、failure、cache与diagnostic（G33至G40）

| Gate | 必须证明 |
|---|---|
| G33 | 产品只遍历execution packet，不按stage/name重建顺序或查找pass |
| G34 | per-access binding table解析到正确version/view/buffer slice和physical allocation |
| G35 | history copy、readback、IBL writeback与present进入packet dependency/lease |
| G36 | bind/materialize/record/readback/submit任一失败都产生Abort receipt并清理全部owner |
| G37 | Success/Abort后pool、readback queue、timer、external lease与frame index状态一致 |
| G38 | cache single-flight避免同key重复compile，miss受budget/deadline且可prewarm |
| G39 | camera/graph/schema/capability precondition错误返回typed result，不panic |
| G40 | compiler diagnostic artifact与backend receipt可按graph key/generation关联并有界保留 |

### 11.6 产品、压力与性能（G41至G48）

| Gate | 必须证明 |
|---|---|
| G41 | generated namespace、sparse、format、duplicate access的adversarial tests全部进入CI |
| G42 | source-string assertions不作为行为验收唯一证据 |
| G43 | 10k pass/100k access compile时间、allocations与峰值RSS有固定阈值和artifact |
| G44 | 4K/8K、多viewport、DRS/resize下compile/cache/materialization无无界抖动 |
| G45 | fault injection覆盖OOM、device loss、map failure、executor failure与queue submission边界 |
| G46 | 长时soak无stale handle、pool泄漏、错误reuse、cache失控或diagnostic高基数 |
| G47 | Editor能消费versioned graph artifact查看pass/resource/barrier/queue/alias/cull原因，不读取Runtime内部对象 |
| G48 | 与Unreal/Unity同内容同硬件比较compile CPU、record CPU、GPU frame与VRAM时，先满足G01-G47再宣称更优 |

## 12. 禁止的临时实现

- 禁止只给IBL pass名称加一个新prefix；必须对最终pass namespace统一校验并删除name-based执行。
- 禁止在WGPU路径为SparseReserved偷偷创建dense texture却仍报告sparse；fallback必须进入compiled disposition、budget和diagnostic。
- 禁止继续向`texture_desc_for`添加插件名称match分支；资源格式必须由typed schema声明。
- 禁止materializer遇到unsupported usage时静默删除bit；compile必须决定Reject/Fallback。
- 禁止用128-bit hash替代完整allocation identity后宣称碰撞已解决；correctness identity必须可复核。
- 禁止把`pass_stages`排序成graph顺序后继续保留双authority；迁移完成后删除旧vector。
- 禁止把pool retention从8帧调大来模拟GPU completion。
- 禁止用更多`include_str!`测试代替collision、fault、multi-queue和真实WGPU product behavior。
- 禁止在Runtime89复制Runtime09A backend barrier/queue owner，或以compat facade长期并存新旧graph执行面。

## 13. 完成边界

本报告完成的是当前源码静态审查与重构需求登记，不是代码修复。只有M0-M8按48项资格门取得可复验回执，且Runtime09A的backend barrier/queue/GPU completion依赖完成hard cut后，Runtime89才能把`implementation_status`改为`complete`。

### 13.1 2026-08-25 source-only progress, not accepted

- `RG89-P0-002` now rejects every `SparseReserved` transient during `RenderGraphBuilder::compile` with typed `SparseTextureUnsupported`; sparse lifetimes no longer reach allocation, materialization, or green completeness telemetry without a provider. There is still no `SparseResidencyService`, device-qualified dense fallback, provider lease, or managed runtime receipt, so this is an explicit fail-closed source slice rather than sparse support.
- `RG89-P0-003` now has one `render_graph::RenderResourceSchema` contract covering texture format, extent policy, dimension, mip count, sample count, usage and fallback, plus buffer size and usage. `ComputePassDescriptor` preserves named texture and buffer schemas through feature lowering and pipeline resource aggregation; explicit schema conflicts, missing storage usage, and unsupported storage formats fail during product compilation. An untyped unknown plugin storage texture still resolves to the old default only long enough to be rejected before graph materialization. The graph compiler independently rejects unsupported transient `STORAGE` formats, and the WGPU materializer returns an error instead of silently removing `STORAGE_BINDING`.
- `RG89-P1-025` has a source-only typed-buffer foundation, not its final catalog: schema-backed transient buffers lower to exact `BufferDesc` requirements, and schema-backed external buffers carry a producer-supplied physical descriptor from runtime-prepare collection through binding and materialization. The physical descriptor is keyed by physical backing, typed backing reuse is rejected until an explicit external-alias registry exists, and WGPU-observable buffer capacity must not be smaller than the supplied descriptor. Physical buffers may be larger or carry extra legal usage; WGPU cannot introspect usage, so that portion remains an owner-supplied lease contract. Generic descriptor-less report-only imports intentionally remain compatible and fail closed when a typed contract is required. The external access-ID lease packet now feeds generic compute and resolver-backed non-compute bindings; dynamic catalog resolution, device-qualified admission, and removal of `texture_desc_for`/`buffer_desc_for` remain open.
- `RG89-P1-001` now has a source-only access-scope foundation. `render_graph::access` owns stable compiled access IDs, texture mip/layer/aspect ranges, buffer byte windows, shader-stage masks, intent DTOs, and immutable access metadata. `builder/access_authoring.rs` owns the corresponding texture/buffer/external access registration and version-token authoring APIs, leaving the builder root to state, declarations, and handle validation. Every builder access carries metadata: legacy texture/buffer calls explicitly mean full logical scope, while report-only external imports remain unresolved and may not claim typed intent. Exact texture/buffer/external APIs carry declared scope through topological reordering into `CompiledRenderGraph`; compile validates descriptor-backed mip, layer, aspect, empty-range, overflow, and bounds failures before dependency inference. P1-004 now consumes that scope canonically; transient device WGPU views/slices are built only after materialization, while feature/compute access-ID lowering remains P1-027/P1-031 work.
- `RG89-P1-002` now has a source-only intent-validation foundation. Non-legacy `RenderGraphResourceAccessIntent` values validate declared read/write direction, non-empty shader-stage visibility, texture versus buffer applicability, and exact descriptor usage (`SAMPLED`, `STORAGE`, attachment, copy, uniform, indirect, and readback) before dependency inference. Legacy paths remain explicitly untyped for product compatibility, while descriptor-less report-only external resources fail closed if assigned typed intent. This is not feature/shader-binding lowering, attachment-aspect/resolve validation, a per-subresource barrier/state plan, or an execution diagnostic packet; those P1-008/P1-011/P1-027/P1-031 contracts remain open.
- `RG89-P1-003` / `RG89-P1-004` now have a source-only fail-closed slice. Before dependency and version inference, builder compilation rejects a repeated `(logical resource, read/write)` tuple in one pass with `DuplicatePassResourceAccess`; same-pass read+write remains legal. `graph/access_index.rs` owns positions, canonical metadata, produced versions, selected input versions, and live compiler-order allocation rows under the stable `{ pass, access ordinal }` identity. `builder/access_scope_tracker.rs` resolves texture cells and split buffer intervals, projects alias-local scope to the parent descriptor, and emits overlap-only hazard/culling/token-coverage facts; `compile.rs` preserves that canonical metadata through topological reorder. Legacy `(pass, resource, read/write)` methods now delegate to the index and return no result if a future range-aware graph makes the tuple ambiguous. Compiler table cardinality or resource-kind drift returns typed errors instead of silently skipping rows; culled accesses remain diagnostic-only and cannot enter the live allocation table. Duplicate same-kind ranges still cannot be represented; the transient materialization table exists, but resolver/executor hard cut remains P1-027/P1-031 work.
- `RG89-P1-005` now has a source-only explicit-provenance foundation. Versioned texture, buffer, and external writes return a builder-scoped `RenderGraphResourceVersionToken`; explicit reads and attachment Loads accept only that same-builder producer token. Compilation inserts producer edges before topological ordering, rejects a foreign, unavailable, self-referential, or superseded producer with typed errors, and resolves the selected producer ordinal into `CompiledRenderGraph::input_version_for_access(...)`. Product authoring maps normalized `RenderFeatureResourceVersion` values through the same token table, so its explicit reads and attachment Loads no longer fall back to a name-based latest lookup. A live compiler-order row now pairs each exact key with a collision-free transient allocation identity when one exists; external/persistent lease identity and product view/slice resolver binding remain P1-027/P1-031 work.
- `RG89-P1-026` now has a source-only definition-normalization slice. After final feature filtering/insertion/replacement, `attachment_initialization` walks the same ordered pass descriptors that graph authoring will consume. An internal attachment write without explicit ops becomes `Clear` when no prior producer exists or `Load + RenderFeatureResourceVersion` when one does; an explicit internal `Load` with no prior producer fails closed. Imported external attachment initial contents remain `Load`, while subsequent external writes also receive the actual prior producer token. `pass_authoring` now requires this normalized contract and no longer owns `produced_texture_resources`. The pass ordering code treats a versioned attachment `Load` as a consumer and rejects a mismatched/latest producer. This preserves current logical ordering but does not create a versioned physical view: P1-027/P1-031 remain open.
- `RG89-P1-038` now has a source-only panic hard cut: `CompiledGraphCacheKey::from_inputs(...)` and `extract_compile_fingerprint(...)` return `RenderGraphCompileInputError::MissingSelectedCamera` instead of calling `expect`. The frame-submit boundary maps that error to `RenderFrameworkError::GraphCompileFailure` with the actual pipeline handle, so invalid frame input fails deterministically before cache mutation. A focused regression clears the selected camera and asserts the typed error. This does not establish a validated frame-compile input owner, concurrent cache safety, or managed Cargo evidence.
- `RG89-P1-033` has a narrow source-only error-path repair: after `TransientResourcePool::begin_frame`, binding, materialization, graph-stage recording, HZB readback encoding, and shared readback-copy encoding failures all release acquired graph backings and call `end_frame` before returning. Every compiled-scene pre-submit failure after a realtime-IBL batch records also terminalizes that batch as unsuccessful, including the earlier overlay, volumetric-policy, runtime-prepare, and readback-admission failures. Runtime-prepare admission occurs before the pool transaction and is intentionally excluded from pool cleanup. This is not yet the required RAII frame transaction or completion-token retirement model; it only prevents the known pre-submit resource-stranding exits.
- Schema-backed external textures retain their resolved descriptor in the compiled graph. Materialization rejects a bound view with no physical descriptor and rejects format, extent, dimension, mip-count, sample-count, or required-usage mismatch; physical resources may retain extra legal usage. Generic report-only external imports intentionally retain no strong descriptor contract, and per-adapter format-feature admission remains M1/P1-016 work. The planar reflection asset/filter format mismatch is still an asset-format hard-cut follow-up. These boundaries prevent marking either P0 slice accepted.
- Added source tests cover sparse rejection, unsupported transient storage formats, untyped plugin storage output rejection, explicit plugin `Rgba8Unorm` storage schema propagation, external texture and buffer physical-descriptor mismatch rejection, extra-usage acceptance, duplicate typed external-buffer backing rejection, physical descriptor capacity verification, non-silent materializer storage failure, duplicate same-kind pass access rejection, exact texture/buffer access metadata across topological reordering, exact-ID version/input-version and dump retention, legacy ambiguous lookup rejection, compiler-table cardinality rejection, mip and empty-buffer-range rejection, write-only intent rejection on a read, empty shader-stage and texture-versus-buffer intent rejection, sampled/uniform/copy usage rejection, report-only external typed-intent rejection, and core/product explicit producer-version provenance. Only Rust 2021 formatting/parsing, whitespace, and failure-handoff static checks ran in this session; no Cargo, GPU, screenshot, RenderDoc, performance, or CI acceptance claim is made.
- `RG89-P1-028` / `RG89-P1-029` now have a source-only hard cut: authoring emits compiler-only `{ RenderPassId, stage }` metadata, while `CompiledRenderPipeline::from_parts(...)` creates one immutable `RenderGraphExecutionPacket` holding the compiled graph, a dense graph-indexed pass table, and fixed stage-index ranges. It rejects missing or duplicate metadata; steady-frame stage execution and sprite-stage admission read `graph_pass_index` directly. The former public `stages` and `pass_stages` copies are removed. P1-030 now also has a source-only batch foundation: the packet lowers live compiled passes into immutable graph-order ranges split by effective `QueueLane`, culling gaps, and queue transitions, validates exact live-pass coverage, caches one `RenderGraphExecutionBatchReport`, and precomputes a stage-to-batch index. The report publishes planned batch/live-pass counts, queue distribution, maximum passes per batch, and queue transitions through the existing execution record, `RenderStats`, and runtime diagnostics without a steady-frame batch scan. `execute_graph_stage` and sprite-stage discovery now consume only stage-relevant immutable batches and apply `RenderPassStage` as service routing, so graph order, culling gaps, queue boundaries, and exact access identities come from one packet authority without rescanning unrelated batches per stage. History/readback/writeback epilogues, backend barrier/queue lowering, and managed execution evidence remain open.
- `RG89-P1-030` source-only ordering correction, 2026-08-30: deferred orchestration now consumes `Deferred -> AlphaMask3d -> Opaque2d -> AmbientOcclusion -> Lighting`, matching the GBuffer producer/consumer dependency while retaining the declared alpha/sprite stages; forward orchestration keeps `AmbientOcclusion -> Lighting -> scene` because its lighting preparation does not consume deferred GBuffer outputs. The common early list no longer executes AO before a deferred GBuffer exists. Focused source checks lock both boundaries; this is a correctness ordering repair, not a performance claim, and requires managed WGPU output validation before acceptance.
- `RG89-P1-030` compiled execution coverage guard, 2026-08-30: stage preparation now admits live passes by compiled graph index into a frame-local bitset and rejects culled, out-of-range, or duplicate admission; the compiled-scene tail requires every live pass exactly once. Deferred orchestration also consumes the previously omitted `AlphaMask3d` stage, preserving `Deferred -> AlphaMask3d -> Opaque2d -> AmbientOcclusion -> Lighting`. This is a source-only correctness gate; managed Cargo/WGPU, PNG/RDC, and performance evidence remain open.

#### P1-004 canonical access and overlap implementation, source-only validation pending

1. `render_graph::access` and `graph/access_index.rs` make `{ pass, access ordinal }` the compiler identity for metadata, produced/input versions, dump rows, and versioned access keys. The legacy `(pass, resource, read/write)` lookup remains compatibility-only and returns no result when ambiguous; repeated same-kind access rows remain rejected until range-authoring can represent them coherently.
2. `builder/access_scope_tracker.rs` resolves texture access to finite mip/layer/aspect cells and buffer access to finite `[offset, end)` segments before inference. Descriptor-backed external textures and buffers use their supplied descriptors; only descriptor-less report-only externals remain opaque and conservatively overlapping. No WGPU view descriptor enters the compiled graph.
3. The scope tracker records texture-cell and split-buffer-interval history. RAW, WAR, WAW, discard validation, culling provenance, and final roots query only overlapping units. Alias roots derive the parent descriptor scope and project it through the alias, and root-writer lookup reads latest writers directly without cloning reader histories.
4. Explicit version tokens now require complete producer coverage of the selected reader/load scope; stale, partial, composite, discarded, and unavailable producers fail typed. Focused source cases cover disjoint and overlapping texture/buffer scopes, depth/stencil planes, typed external final roots, aliases, and stale/partial coverage.
5. This is not product completion: transient materialization and the external access packet now own exact-ID device tables, and generic compute plus resolver-backed non-compute consumers consume typed views/windows from them. A small set of direct non-compute helper paths still uses whole-resource/name compatibility. P1-027/P1-031 therefore still own the remaining hard cut to eliminate those bypasses. The required coordinator Cargo, fixed-profile compiler metrics, PNG, RenderDoc, and independent acceptance evidence remain pending; no runtime performance or power claim is made.

#### P1-027/P1-031 binding-table status, transient device-table source-only

Architecture audit, 2026-08-25: `RgResourceResolver` still selects a whole `TextureView` or `Buffer` from `(logical resource, read/write)`, while `RenderGraphExecutionResources` and transient materialization carry logical/backing names in string-keyed maps. Static source inventory found 19 name-based GPU-lookup parameters, 12 string-keyed execution-resource maps, and 18 resolver resource+kind lookup mentions. Those counts describe migration scope only; they are not runtime timing, memory, or power measurements. `dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/RenderGraphTextureSubresource.h` establishes a finite subresource layout/range as the source of truth; `dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraphResources.cs` plus `Tests/Editor/RenderGraphTests.cs` carries resource type, version, and execution validity with version-management regression tests; `dev/godot/servers/rendering/rendering_device_graph.cpp` resolves actual texture-slice intersections and rejects overlapping slices in one command. Zircon must retain the stronger explicit physical-binding contract below rather than copying Unity's current whole-resource getter.

1. `RenderGraphVersionedAccessKey` now carries exact `{ access_id, logical resource, selected version, canonical range, intent }` for every live access after scope/version inference. `CompiledRenderGraph` remains WGPU-free and exposes compiler-order allocation rows; resource+kind queries are compatibility-only and never a binding authority.
2. `RenderGraphPhysicalAllocationId` now represents compiler-proven transient `{ allocation_id, kind, bucket_hash, slot }`; the allocation ID is collision-free and the hash/slot are diagnostic fields only. Persistent rows remain `None` because they are owned outside transient allocation; graph-owned persistent textures now use a separate frame-scoped exact access lease, while typed external rows use their own access-ID lease packet. Descriptor-less imports never enter either typed table by name.
3. The source now builds frame-scoped `graph_execution` device tables after transient/persistent materialization and declared view-alias materialization. Every live compiler row with `Some(RenderGraphPhysicalAllocationId)` gets one exact access-ID entry containing a cloned `wgpu::TextureView` for the canonical mip/layer/aspect range or a cloned `wgpu::Buffer` plus canonical finite `[offset,end)` range. Graph-owned persistent texture rows map exact access IDs to one WGPU texture handle per logical resource. Typed external rows are materialized by `CompiledRenderGraphExternalAccessPacket` after frame/plugin binding; unknown report-only imports remain the name-compatibility boundary. All typed tables retain immutable compiler identity, clear before re-materialization and pool retirement, and keep WGPU objects out of `CompiledRenderGraph`.
4. Texture-view materialization caches by `(RenderGraphPhysicalAllocationId, canonical subresource range)`. Thus repeated accesses to the same transient backing/scope clone an existing WGPU view rather than creating another one; persistent texture access rows similarly share one cloned WGPU texture handle per logical resource. `RenderGraphExecutionResourceReport.access_binding_report` currently carries the transient access/view counters only, so persistent lease counters remain test-local until the diagnostics schema is extended. These are deterministic structural counts, not CPU/GPU timing, VRAM, or power data. Generic compute, resolver-backed non-compute consumers, and the compiled history epilogue consume their applicable exact access-ID tables; sparse/provider-owned persistent resources, persistent buffers, unknown report-only imports, and a small direct-helper set remain explicit compatibility work. Names remain compile-time labels and descriptor references only, and no WGPU object enters `CompiledRenderGraph`.
5. P1-004 now supplies canonical scope/version inference before allocation rows are built: selected producers cover every texture cell or buffer segment, aliases use parent ranges, and composite/stale/discarded producers fail typed. Conditional producers remain P1-015 work. This ordering prevents the product table from freezing the former resource-global latest-writer behavior.
6. Required source/model coverage: topological reorder with nonzero access ordinal; disjoint versus overlapping mip/layer/plane and buffer ranges; partial/composite/stale input version rejection; dense table cardinality and key-to-allocation determinism; transient alias slot reuse only after non-overlap; typed external/persistent lease mismatch; resolver rejection of unknown access ID, wrong kind, and out-of-range buffer slice. Product evidence remains coordinator-managed WGPU validation with PNG plus RDC capture.
7. Static algorithm review before implementation: the old compiler allocation-row constructor linearly searched `L` resource lifetimes for each of `A` live access keys, giving `O(A*L)` lookup work. The replacement precomputes allocation facts from `T` transient allocations and `L` lifetimes once, then performs expected-constant-time map lookup per access: expected `O(T + L + A)` work and `O(L + A)` retained index/table entries. Product table construction additionally creates at most `U` WGPU views for `U` unique `(physical allocation, canonical range)` pairs rather than one view per texture access. These are source-derived algorithmic bounds only, not timing, VRAM, GPU, or power measurements.
8. Before any optimization claim, capture managed fixed-profile measurements: compiler access-binding build CPU and table bytes, live access/unique-view/slice counts, binding-cache hit rate, record CPU p50/p95, GPU frame and timestamp scopes, VRAM peak, transient alias savings, queue-submit count, and power where the platform exposes a comparable metric. Compare the same scene and device profile against a baseline; no such measurement exists in this source-only session.
9. 2026-08-30 non-compute buffer-slice audit: the executor families have been migrated in bounded slices (clustered lighting, froxel, forward/OIT, SSS, exposure, and Hybrid GI) to explicit `wgpu::BufferBinding` windows; resolver-backed typed external buffers now use the same access-ID lease table. Remaining direct `&wgpu::Buffer`/`as_entire_binding()` call sites mix full-buffer persistent resources and compatibility paths, so the next migration must keep family-specific contracts and validate each family with managed WGPU before claiming CPU/VRAM/power improvement.

#### P1-023 transient pool device-epoch correction plan, 2026-08-25

1. Review finding: `TransientResourcePool` is owned for the renderer lifetime, but its free and pending backing maps were keyed only by texture/buffer descriptors. `RenderBackend` already owns the canonical `RenderDeviceProfile { DeviceId, DeviceGeneration }` and gives that same pair to `DeviceFaultGate` and `WgpuSubmissionCoordinator`; the pool did not consume it. A recreated device could therefore receive an old-device `wgpu::Texture` or `wgpu::Buffer` that happened to have an equal descriptor.
2. Required ownership rule: `begin_frame` receives the backend profile and activates the exact `{ DeviceId, DeviceGeneration }` epoch. On a changed epoch it drops every free and pending transient backing before any submission-status polling or materialization. It must not infer device identity from WGPU pointers, labels, descriptor hashes, adapter facts, or a submission sequence number. Materialization receives the same profile and fails closed if the pool was not activated for it.
3. Test-first acceptance: retain texture and buffer backings under one offscreen backend, activate a second backend profile, assert both free and pending old-epoch entries are discarded, and prove the next texture acquisition is a creation rather than an identity reuse. Keep the existing same-epoch reuse and completion-ticket tests. Record deterministic discarded-entry counters only; they are lifecycle evidence, not a performance result.
4. This repairs the device-generation part of P1-023 only. Shared GPU memory-budget ownership, adaptive pressure policy, device-loss orchestration for all retained renderer resources, and any actual CPU/GPU/VRAM/power measurement remain open. Coordinator-managed Cargo, offscreen PNG/RDC evidence, and a measured fixed-profile baseline are still required before acceptance.
5. Source-only implementation status: `TransientResourcePool` now owns an active epoch, clears free and pending backing containers on a profile change, and materialization receives and checks the same profile. `RenderGraphTransientPoolReport` plus the render diagnostics store expose texture/buffer epoch-discard counts. The regression source covers a second offscreen backend and the exact-ID device table excludes a culled writer's diagnostic access. This session has not run Cargo, offscreen rendering, RenderDoc, or any timing/VRAM/power capture; these changes are not milestone acceptance.

#### P1-031 resolver/compute hard-cut preflight, 2026-08-25

1. Code inventory: `RenderPassExecutionContext` already receives an ordinal-checked `compiled_access_ids` slice from `execute_graph_stage`, so graphics execution has a valid compiler-to-executor identity channel. `RgResourceResolver` and `RenderPassGpuExecutionContext` still expose name-plus-read/write lookup methods. Generic compute is the principal remaining double-authority path: `BindingSchemaEntry` carries a resource string and an independent mip/full-chain or buffer window, while `RenderGraphComputePassMetadata` has no resolved access ID and generic compute resolves WGPU bindings by name.
2. Required packet design before call-site migration: compilation must produce one immutable compute-binding packet per binding with `{ binding slot, exact RenderGraphResourceAccessId, versioned key, binding kind }`. It must prove that the schema's texture mip/full-chain or buffer range equals the canonical compiled access scope, and reject a mismatch rather than intersecting or silently widening it at execution. `StorageBufferReadWrite` needs its own read/write provenance audit before it can be assigned a single writer identity; this cannot be hidden behind the current write-only lookup.
3. Execution migration order: inject that compiled packet through `RenderPassExecutionContext` into `RenderPassGpuExecutionContext`; generic compute resolves a transient texture view or buffer slice only by its exact access ID; then migrate non-compute executors in bounded families. External/persistent access rows remain unavailable to this table until their typed device lease is complete, so they require a separate declared-lease packet rather than a name fallback. The current resolver APIs can be deleted only after every product executor has an exact packet.
4. Required evidence after implementation: compiler tests for name/ordinal/range/kind mismatch and read-write provenance; offscreen WGPU tests for alias mip/layer and nonzero buffer window; then coordinator-managed Cargo, fixed-profile access-table counters, GPU timestamp/VRAM/power baseline comparison, PNG in `docs/tests/runtime/render`, and RenderDoc capture. This preflight is source analysis and an implementation plan, not a claim that the hard cut or its measurements exist.
5. 2026-08-25 implementation review result: `ComputePassDescriptor::lower_into` lowers `BindingSchemaEntry` into `RenderFeatureResourceDescriptor` rows containing only name/kind/read-or-write, while `author_pass_resource_access` calls whole-resource graph APIs. A schema `texture_mip_level`, `texture_full_mip_chain`, or `buffer_range` therefore does not yet become a canonical graph access scope. `StorageBufferReadWrite` emits two graph resource rows from one WGPU binding, so it cannot honestly carry one exact writer access ID without a separately modeled read/write pair. The next implementation must first add scoped compute-access authoring and prove it against the compiler's canonical scope; the packet/executor migration remains deliberately unstarted rather than hiding this mismatch behind a name fallback.
6. Scoped-authoring implementation plan, 2026-08-26: extend the product `RenderFeatureResourceDescriptor` with optional `RenderGraphResourceAccessMetadata`. Compute lowering derives this field from each binding: sampled/storage textures use the requested canonical mip range and compute visibility; uniform/storage buffers use the exact static byte range; `StorageBufferReadWrite` produces an explicit read row with `StorageBufferRead` intent and a separate write row with `StorageBufferReadWrite` intent. Lowering enriches a caller-declared legacy resource when the name/access pair is unambiguous and retains distinct rows for distinct scopes. Typed transient authoring must call the existing `*_with_access` builder APIs and preserve exact producer-version handling. Typed external access now has its own immutable access-ID lease packet and must not enter the transient exact-ID table through a name fallback; unknown report-only imports remain an explicit compatibility boundary. This stage is a source design/implementation step only: managed executor validation, GPU profiling, PNG, and RDC evidence remain pending.
7. Scoped-authoring implementation status, 2026-08-26: the descriptor field, lowering rules, and `pass_authoring` scoped texture/buffer calls are now present. Focused source coverage asserts a single-mip sampled binding, a nonzero read-write buffer window, their compiled read/write access IDs and versioned keys, a full-buffer canonicalization case, and the absence of transient scope on caller-declared external resources. This removes the previous whole-resource authoring mismatch for typed transient generic-compute resources. It is source-only work: Cargo, offscreen rendering, RenderDoc, performance, VRAM, power, PNG, and coordinator acceptance all remain pending.
8. Compiler packet and transient bind-group implementation status, 2026-08-26: `CompiledRenderGraph` now builds one compiler-order binding packet for every live compute pass. Each row contains the binding slot/kind and independent optional read/write `RenderGraphVersionedAccessKey`s. Packet construction creates a scoped and external lookup index once per pass, so its lookup work is expected `O(R + B)` for `R` pass access rows and `B` schema bindings, rather than scanning `R` rows for every binding. Expected binding scope is canonicalized with the declared buffer/texture descriptor, including texture-view alias projection, before matching the compiler access scope. A transient range/intent mismatch, missing row, or ambiguous row is a typed `RenderGraphError`. The execution context now receives the packet from the immutable compiled graph; generic compute requires it and resolves transient textures and buffers only through the exact access-ID WGPU view/window table, rechecking the WGPU buffer binding range against the compiler range. External rows retain only a logical exact key and use the explicitly temporary label lookup path until a typed lease packet exists. The `O(R + B)` bound is source analysis, not CPU/GPU timing, memory, or power data.
9. Dispatch-target continuation preflight, 2026-08-26: `FromBuffer` cannot be hard-cut by attaching a second ad-hoc read row. WGPU indirect dispatch needs `INDIRECT` usage while the same buffer may also be a storage binding, and the current graph correctly rejects duplicate same-kind pass accesses. The selected design is the equivalent compiler-authorized sharing rule: preserve exactly one declared read access, retain its `RenderGraphVersionedAccessKey` in a separate immutable dispatch packet, and validate the declared `BufferUsage::INDIRECT` plus the exact 12-byte command window. `PerPixel` similarly selects exactly one declared write texture access when available, otherwise one read access, and stores a resolved logical target extent plus local size. It never chooses a resource at execution by the workload label.
10. Dispatch-target source-only implementation status, 2026-08-26: `graph/compute_dispatch_access_packet.rs` compiles packets only for live passes carrying generic-compute metadata, keeping custom executor workload metadata out of this WGPU binding contract without introducing a dependency from `render_graph` to the graphics executor registry. Compilation builds declaration maps once and scans each applicable pass access list, giving expected `O(D + sum(R))` construction work for `D` declarations and `R` access rows, with one `O(1)` packet lookup per execution pass. The executor now pattern-matches the packet: indirect dispatch resolves a transient buffer by exact access ID, while an external access remains on the separately declared temporary resource path; per-pixel dispatch consumes the compiler-resolved extent. `FromBuffer` requires the declared read and, where present, its binding schema to equal exactly `[offset, offset + 12)`, after proving `BufferUsage::INDIRECT`; a containing range is rejected. The old execution-time `per_pixel_extent` resource-name path is removed. Per-pixel extent derives from the selected versioned texture access range plus the target logical view descriptor: a canonical parent mip is translated once into the alias-local mip, and binding-name mip metadata is never scanned. Source tests cover typed indirect use and exact-window rejection, exact access identity, normal, alias-local, selected-access-mip, and read-fallback per-pixel extents, execution-context injection, and custom no-metadata workload exclusion. This changes compile/recording algorithmic behavior only; no Cargo, WGPU framebuffer, RenderDoc, CPU/GPU timing, VRAM, or power measurements have run.

#### P1-030 structural execution plan, partial batch consumer foundation

2026-08-30 review closure: `CompiledRenderPipeline` now exposes the packet's precomputed stage-to-batch index through a read-only forwarding API. The packet also caches first-seen compiled stage order for late-stage discovery, removing a repeated batch scan without changing service boundaries. The history epilogue now freezes canonical source writer access IDs and resolves graph-owned transient/persistent source leases exactly, but remains intentionally outside the packet until destination history leases, queue barriers, and completion ownership are modeled; the current `copy_history_textures` encoder writes are therefore tracked as an explicit structural dependency rather than being reclassified as graph work. Stage-filtered consumers also remain dependent on the product's fixed stage order: an interleaved compiled graph is rejected by the frame-local admission cursor instead of being silently reordered, so unified global batch routing is still pending.

2026-08-30 history producer-receipt follow-up: TAA scene color and exposure current are graph-authored external destinations rather than epilogue copy sources. Their executors publish a frame-local history write receipt only after successful render/compute encoding. SSR resolve, HZB build, volumetric light-scatter, generic-compute SSAO, and Hybrid GI plugin resolve publish the same receipt after producing their source; generic compute reuses its existing storage-write metadata scan, while the plugin calls the public `FrameHistorySlot` receipt API after the dual-target resolve succeeds. The receipt travels with `RecordedGraphPass`, merges at the serial stage owner, and gates the copy report, exact-access epilogue copy, and history transaction. A compiled writer declaration alone therefore cannot validate cross-frame history or authorize copying a stale backing. Destination leases, barriers, completion ownership, managed Cargo/WGPU, PNG/RDC, timing, VRAM, and power evidence remain pending.

2026-08-30 workload-audit descriptor follow-up: froxel dispatch audit dimensions now come from the volumetric source descriptor already frozen beside the final writer access ID in `CompiledHistoryEpiloguePlan`; the per-pass production path no longer looks up `VOLUMETRIC_SCATTERING` by name in the frame resource table. This is an identity/diagnostic correctness closure, not a measured hot-loop optimization. The stale `RecordedGraphPass` unit fixture was also brought up to the current upload/UI/writeback/history-receipt shape so managed compilation can exercise the owner once the shared validation lane is available.

2026-08-30 error-terminal follow-up: deferred lighting's volumetric-parameter and subsurface-MRT invariants now return an execution error instead of panicking through production `expect` calls. The graph executor propagates the failure at its existing `Result<String>` boundary; valid WGPU binding and attachment layouts are unchanged.

2026-08-30 non-compute buffer-window slice: the deferred clustered-lighting path now consumes `LIGHT_GRID_PARAMS`, `LIGHT_ZBINS`, and `LIGHT_TILE_MASKS` through an exact `wgpu::BufferBinding` helper. Transient resources preserve the compiler-proven byte window; external and persistent declarations use an explicit full-buffer compatibility path until typed leases land. The remaining non-compute call sites stay grouped by resource family for later migration.

2026-08-30 post-process clustered-lighting continuation: the uber post-process and four SSR consumers now consume `LIGHT_LIST` through the same exact-window helper, carrying `wgpu::BufferBinding` through shared executors and bind-group entries; the clustered-lighting producer and light-grid upload use the same window for clear, storage binding, and copy-queue upload offsets. This keeps the migration bounded to one clustered-lighting family; exposure has since migrated, while probe and other persistent buffers remain on their explicit full-buffer contracts until their ownership model requires narrower scopes.

2026-08-30 clustered-lighting continuation: the froxel volumetric light-scatter executor and its two existing WGPU product request fixtures now carry explicit `wgpu::BufferBinding` values for the same three light-grid buffers. The exact compiler window therefore propagates through deferred, uber/SSR, producer/upload, and volumetric scatter consumers; test fixtures retain an explicit full-range binding for compatibility.

2026-08-30 forward light-grid consumer slice: mesh recording, OIT fragment store, and overlay `BaseScenePass` now use `optional_buffer_binding_by_name` and `Option<wgpu::BufferBinding>` for the three light-grid parameters. Transient exact byte windows reach the fragment bind group, while missing/external resources retain explicit full-range fallback bindings; shadow, probe, and other full-buffer families remain outside this bounded migration, with exposure handled by the later dedicated slice.

2026-08-30 OIT transient window slice: OIT fragment-store `layers/counts` writes, atomic count clear, and resolve reads now share `require_buffer_binding` exact `BufferBinding` values. Capacity validation uses the actual binding window and clear preserves compiler offset/size, so unused transient backing tails are not silently included in execution.

2026-08-30 OIT/forward error-terminal follow-up: missing OIT pipeline initialization and missing BaseScene forward-receiver bindings now fail closed at their existing execution boundaries instead of using production `pipeline.as_ref().unwrap()` or generic-receiver `expect`; valid bind/draw ABI remains unchanged.

2026-08-30 SSS buffer-window slice: subsurface setup/scatter tile-list, indirect-args, profiles, and params now consume exact `wgpu::BufferBinding` values. Setup clear preserves the compiler `offset/size`, scatter indirect dispatch uses `buffer + offset`, and bind groups no longer widen these resources through `as_entire_binding()`. External/persistent compatibility remains the resolver's explicit full-range contract; other buffer families are unchanged.

2026-08-30 exposure buffer-window slice: exposure histogram/resolve and color-LUT bake graph-buffer inputs now consume `wgpu::BufferBinding`. Transient histogram/exposure accesses retain compiler-proven offset/size, histogram clear is bounded to the same window, and persistent fallback is an explicit full-range binding. Shader/layout and resource-family lifetime semantics are unchanged; managed Cargo/WGPU, PNG/RDC, performance/power, and coordinator acceptance remain pending.

2026-08-30 Hybrid-GI transient handoff slice: the three plugin handoff executors for `hybrid-gi-scene`/`hybrid-gi-trace` now call `require_buffer_binding`. Scene-depth packet writes validate relative offsets against the compiler window, while trace-schedule and resolve bind groups consume exact `BufferBinding` values without widening transient backing through `as_entire_binding()`. Plugin ABI and shader binding indices are unchanged; managed plugin Cargo/WGPU, PNG/RDC, performance/power, and coordinator acceptance remain pending.

2026-08-30 advanced-lighting error-terminal follow-up: Froxel media-inject/light-scatter/integrate, planar-filter, and subsurface pipeline caches no longer enter production encoding through `as_ref().unwrap()` after initialization. Missing caches return typed errors with executor/pass identity, and the fixed three-dimensional light-scatter dispatch no longer uses a slice-conversion `unwrap`. This is source-only failure-boundary work; managed Cargo/WGPU, PNG/RDC, and performance/power evidence remain pending.

2026-08-30 compiled execution cursor wiring: the immutable packet `begin/admit/finish` cursor is now owned by `RenderGraphStageExecution`. Every live compiled pass must pass packet admission in graph order during stage preparation, and the same cursor is finalized at the frame tail. Existing stage service routing remains, but interleaved graph routing now fails closed at execution admission instead of relying only on duplicate-coverage detection.

2026-08-30 compiled batch ownership index: `RenderGraphExecutionPacket` now caches an O(1) reverse index from each live compiled pass to its queue batch (`culled -> None`). Stage consumers receive the global batch index from the precomputed stage-to-batch list, and admission rejects a pass routed through a different batch. This preserves graph order and queue boundaries while reducing future driver lookup work; history/readback/barrier lowering, typed external leases, dynamic WGPU, screenshots, and performance/power evidence remain pending.

2026-08-30 external access lease packet foundation: `CompiledRenderGraph` now lowers every live external access into an immutable `{access_id, versioned_key, external_binding, typed_desc}` packet. After frame/plugin physical binding, `RenderGraphExecutionResources` materializes an access-ID lease table; generic-compute and resolver-backed non-compute external textures, buffers, and indirect dispatch resolve through that table, and schema buffer windows must match the lease exactly. Descriptor-less report-only view-only imports remain compatible, while pipeline-layout consumers fail closed when a physical descriptor is required. Direct non-compute helpers continue to be migrated by resource family, and all dynamic WGPU/PNG/RDC/performance evidence remain pending.
2026-08-30 IBL graph output identity/window slice: graph-backed PMREM/IEM texture readback resolves every live writer through its compiled access ID and requires all PMREM mip writers to share one compiler physical allocation. The transient access table retains one full WGPU texture handle per physical allocation beside its subresource views. Irradiance SH9 storage-buffer output and readback preserve the compiler-proven offset/size through `RenderPassGpuExecutionContext::require_buffer_binding(...)`/`StorageBufferRange`, staging, and product-diagnostic admission. Backend admission rejects descriptor/window size mismatch, checked-add overflow, and physical-buffer overrun; a `64..64+SH9_SIZE` typed-external fixture covers the non-zero window. Direct environment-capture targets retain an explicit full-buffer compatibility variant. This closes the IBL non-compute texture/buffer readback bypass without changing shader/layout or direct-capture ownership.

2026-08-30 P1-031 persistent exact-view continuation: the compiler now exposes one persistent texture backing owner for both a direct texture and its logical view alias. Product materialization keeps one WGPU texture handle per backing, reuses one created view per `(backing, compiler-projected range)`, and publishes an exact view lease for every live access ID. Resolver-backed non-compute texture view/descriptor/optional/owned/physical/full-mip/selected-mip/mip-count helpers validate that graph-owned lease before use; the three buffer helper families remain on the exact transient byte-window path. Source regressions cover a direct persistent mip and a persistent-parent alias. Exact rustfmt, locked Cargo metadata, scoped diff and source-contract checks passed; managed Cargo/WGPU, PNG/RDC, timing, VRAM, power and coordinator acceptance remain pending. This is an identity/ownership correction, not measured performance evidence. Remaining P1-031 work is persistent buffers, sparse/provider-owned typed texture leases, and eliminating the compatibility view creation path for full-chain/selected-mip helpers where a fully access-scoped packet is required.

2026-08-30 P1-031 persistent exposure external-buffer continuation: source review confirmed that exposure history is renderer-owned imported memory, matching UE's multi-frame eye-adaptation buffer registration model, so no graph-owned persistent-buffer allocation table was added. The existing typed external packet now receives versioned scoped read/write/load authoring; feature descriptors carry the 16-byte schema and exact shader intent; five live exposure accesses compile to `Buffer(0..16)` access-ID leases. This also removes the external/buffer kind split previously caused by later consumers declaring `read_buffer(EXPOSURE_CURRENT)`. Source tests cover producer provenance, schema/access retention, compiled range/intent, and external packet descriptor retention. Exact rustfmt, locked Cargo metadata, scoped diff and source-contract checks passed; managed Cargo/WGPU, screenshots, RDC, timing, VRAM, power and coordinator acceptance remain pending. This is a structural correctness closure, not a measured optimization. Remaining work is other provider-owned persistent buffer families, sparse/provider-owned textures, and full-chain/selected-mip compatibility hard cuts.

2026-08-30 P1-031 provider-owned external-texture exact-view continuation: review found that the compiled external packet retained `Texture(range)` while the WGPU materializer cloned one producer default view for every access. The materializer now creates an access-scoped WGPU view from a published physical texture backing and reuses it per `(graph resource, canonical range)`; a view-only lease is accepted only when the compiler range provably covers the complete physical mip/layer/aspect view. Partial view-only requests fail before encoding instead of masquerading as exact leases, while legacy `UnresolvedExternal` resources retain their explicit default-view path. Source regressions cover mip2 backing materialization, partial view-only rejection, and canonical full-scope compatibility. This removes a correctness gap and avoids duplicate view creation for equal scopes, but no runtime performance claim is made until managed WGPU/profile evidence exists. Remaining P1-031 work is other provider-owned persistent buffers, sparse residency, view-only producers that need an explicit backing/access-view protocol, and final full-chain/selected-mip compatibility hard cuts.

2026-08-30 P1-031 TAA external-texture exact-lease continuation: the fixed View-sized TAA history pair now publishes `Rgba16Float` physical schema and exact full-texture intent at feature authoring. The previous slot is a fragment sampled read and the current slot is a color-attachment write. `TemporalHistoryStore`/`SceneFrameHistoryTextures` expose the retained backing texture beside its view and descriptor, and the compiled-scene binder imports all four identity-bearing fields so the external materializer creates the access-scoped WGPU view from the compiler packet. Source regressions cover authoring, compiled packet retention, backing publication, and live binding. This closes one real provider-owned consumer without inventing a graph-owned allocation owner. AO's 1x1 fallback, dynamically mipped HZB, and dynamic-depth volumetric history remain separate catalog/variant problems. Managed Cargo/WGPU, PNG/RDC, timing, VRAM, power, and coordinator acceptance remain pending; no performance conclusion is claimed.

2026-08-30 P1-031 Hybrid GI/SSR exact-history continuation: Hybrid GI now declares both fixed View-sized `Rgba16Float` previous textures with fragment sampled full-scope metadata and publishes their physical renderer-owned backings. SSR review found a stronger bypass: its shader already performed temporal reprojection, but the resolve pass did not declare previous history and the GPU executor read `SceneFrameHistoryTextures` directly. The previous SSR slot is now a typed external access, its binder publishes texture/view/descriptor, and resolve consumes only the optional pass-scoped access-ID view; cold-start fallback semantics remain unchanged. Uber also declares/resolves its previous-GI fallback while retaining current-GI priority, and SSR auxiliary entries no longer inject unused owner views into the shared bind group; graph executor direct GI/SSR history maps are zero. UE comparison confirms SSR history is supplied through the graph/temporal pass rather than an undeclared executor owner. This is an execution-ownership correctness repair, not a filtering or performance optimization. Managed Cargo/WGPU, consecutive-frame images, RDC, timing, VRAM, power, and coordinator acceptance remain pending.

2026-08-30 P1-031 HZB dynamic external-history continuation: review rejected copying the fixed View/single-mip history schema because HZB geometry is dynamic. The previous-HZB feature now declares exact compute-sampled full-range intent without a schema; external authoring resolves its descriptor through the built-in catalog, and current/previous HZB share one SceneLinear-allocation `HzbBuilder` path for power-of-two extent and full mip count. The history owner publishes the physical `Rgba16Float` texture/view/full-mip descriptor, so the compiled packet canonicalizes full access to the actual mip chain and the materializer creates an exact lease. Source regressions cover 1923x1081 -> 1024x1024/11 mip catalog parity, compiled packet intent/range, and 16x16 -> 8x8/4 mip physical binding. Exact rustfmt, locked metadata, source-contract and scoped diff checks passed. This fixes ownership and descriptor correctness only; cull shader/thresholds were not optimized, and managed WGPU, PNG/RDC, 300-frame timing/VRAM/power and coordinator acceptance remain pending.

2026-08-30 P1-031 volumetric dynamic 3D external-history continuation: the plugin previously declared previous scattering as a descriptor-less report-only view even though the executor already consumed it through the pass-scoped resolver. The feature now declares exact compute-sampled full-texture metadata without duplicating a schema; the built-in catalog uses one shader-quality-driven froxel descriptor helper for current and previous `160x90x48/64/96` `Rgba16Float` D3 textures. The provider publishes the borrowed texture, default view, and `SAMPLED | COPY_DST` physical descriptor, allowing the external packet to canonicalize one mip and one addressable D3 layer and materialize an access-ID view. UE 5.5.4's `VolumetricFog.cpp` provides the reference ownership model: valid `LightScatteringHistory` is registered as an external RDG texture and current scattering is queued for extraction. Source regressions cover authoring metadata, catalog parity, compiled packet range/intent, and live physical binding. No temporal, jitter, rejection, lighting, or layout algorithm changed; managed Cargo/WGPU, PNG/RDC, 300-frame timing/VRAM/power and coordinator acceptance remain pending.

2026-08-30 P1-031 AO variant preflight and SSAO descriptor single-owner continuation: review found that runtime and `rendering.ssao` independently owned the same compute pass, built-in WGSL, binding schema, dispatch, and resource declarations, while the plugin copy omitted the runtime AO output schema. The runtime built-in descriptor is now the only owner and is exposed through a narrow graphics facade; the plugin retains identity/capability/registration and delegates descriptor construction. This removes one inconsistent compile input but does not claim the previous-AO exact lease. The valid provider is a View-sized `Rgba8Unorm` history texture, whereas the cold-start path binds a 1x1 white neutral view; one fixed compiled descriptor cannot truthfully describe both. Global extent-validation relaxation and a forged View-sized physical descriptor are rejected. The next implementation must use explicit history-ready/historyless compiled variants or a same-extent neutral provider before exact access lowering. GTAO, denoise, temporal qualification, composition, WGPU/PNG/RDC, profile, power, and coordinator acceptance remain open.

2026-08-30 P1-031 fixed external-buffer physical-lease continuation: source review found that exposure graph compilation already emitted exact 16-byte storage/copy descriptors, but `bind_history_graph_resources` used `insert_buffer`, which explicitly removes the imported physical descriptor. The history owner now publishes borrowed previous/current buffers with the actual 16-byte `BufferDesc`, preserving the two-slot transaction/flip model. The new frame-scoped exact external-buffer authoring API is used by SSAO params: the runtime descriptor owns the shared 32-byte Rust/WGSL ABI, `UNIFORM | COPY_DST` schema, full-buffer compute-uniform intent, and compute binding; the producer publishes actual buffer size/usage and the compiled-scene binder materializes the access-ID lease. This is resource identity/ABI repair, not an exposure or AO algorithm optimization. Source regressions cover authoring metadata, canonical SSAO `0..32` packet, producer binding, and exposure physical descriptors. Managed Cargo/WGPU, PNG/RDC, timing, VRAM, power, and coordinator acceptance remain pending.

1. Lower the final topological pass sequence into immutable contiguous batches keyed by execution domain, resource-transition boundary, and queue lane. `RenderPassStage` remains telemetry/product metadata and must not become the batch authority.
2. Replace the renderer's long optional-argument stage calls with one frame-scoped services object. Executor capability selects borrowed mesh, sprite, post-process, overlay, and UI services without reordering the compiled pass sequence.
3. Lower scene clear to a graph prologue and history copy/readback/writeback to explicit graph epilogue passes before switching the product loop. This is jointly constrained by RG89-P1-035 and backend barrier/queue ownership in Runtime09A.
4. Only after steps 1-3, profile the compiled-batch loop against the current stage dispatcher on fixed hardware: collect per-frame batch count, live-pass count, CPU record p50/p95, transient allocation/reuse, queue-submit count, GPU timestamp scope totals, and RenderDoc markers. Acceptance requires the new loop to preserve every live pass exactly once, execute no culled pass, and remove steady-frame pass-name comparisons and graph-length prepared-index allocations.

当前已实现`RG89-P0-001`的final pass名称唯一性和compiled pass ID直接执行切片，并写入独立优化记录；`RG89-P0-002`、`RG89-P0-003`、`RG89-P1-001`的access scope/ID基础、`RG89-P1-002`的intent validation基础、`RG89-P1-004`的canonical scope/version/dependency基础、`RG89-P1-023`的device epoch pool invalidation基础、`RG89-P1-027`的live compiler allocation table基础、`RG89-P1-031`的transient/persistent-texture/external exact-ID device table、persistent exact-view/alias backing归一化、provider-owned external texture backing 的 access-scoped view、exposure persistent external buffer 16-byte physical lease、SSAO 32-byte frame external uniform lease、TAA/Hybrid GI/SSR/HZB/volumetric history exact external-texture lease、SSAO descriptor单一owner、scoped compute authoring、generic-compute bind-group及动态 dispatch partial hard cut、resolver-backed non-compute校验和history source exact-access切片、`RG89-P1-025`的typed-buffer基础、`RG89-P1-026`的附件初始化归一化、`RG89-P1-005`的显式版本 token 基础、`RG89-P1-028/P1-029`以及`RG89-P1-038`正在以上source-only状态收敛。P1-002的产品语义下沉与barrier输出、P1-025的catalog/dynamic policy、P1-031剩余其它provider-owned persistent buffer、AO history-ready/historyless variant与exact lease、sparse residency、view-only partial producer协议与full-chain/selected-mip compatibility helper hard cut、P1-030的history destination/readback/queue-barrier下沉及其余M1-M8尚未实现。协调器分组Cargo、release性能门禁、PNG/RDC证据和独立review仍待运行。未取得对应回执前，不能宣称Runtime89完成、动态测试通过，或性能/表现已经优于Unreal/Unity。
