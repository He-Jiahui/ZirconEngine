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
| RG89-P1-001 | graph access只到whole texture/buffer；mip view选择只存在compute binding/executor | `TextureSubresourceRange { mip, layer, aspect/plane }`与`BufferRange`进入每次access |
| RG89-P1-002 | read/write不能表达shader stage、attachment、copy、indirect、present等精确usage/state | typed `ResourceAccessIntent`同时驱动validation、barrier与diagnostic |
| RG89-P1-003 | `(pass, resource, read/write)`索引用`or_insert`折叠同pass同类多次access | per-access stable ID；禁止歧义duplicate或逐access保存version/range |
| RG89-P1-004 | builder允许重复access；同pass重复write可制造self-dependency/cycle，重复read又被索引折叠 | pass close时canonicalize/validate access set并给出typed conflict |
| RG89-P1-005 | version ordinal只在compile后查询，authoring API不能引用明确produced version | SSA-style read/write handle或显式version token，禁止“latest by visit order”成为隐含合同 |
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
| RG89-P1-020 | `mark_persistent`只设flag/cull root并从transient plan排除，没有graph-owned persistent registry | persistent resource key、generation、resize/recreate、import/export与failure policy |
| RG89-P1-021 | persistent texture由调用方按同名预导入；buffer没有等价完整产品合同 | typed persistent lease与frame-to-frame version transition，不按name碰运气 |
| RG89-P1-022 | SSR coarse mip alias由固定资源名硬编码，graph本身不知道它是parent mip 1 | graph-declared view/subresource alias，删除产品特例 |
| RG89-P1-023 | pool按CPU frame age回收，缺submission/completion ticket、device generation与全局memory pressure | fence-qualified retire queue、device-loss invalidation与共享GPU budget owner |
| RG89-P1-024 | pool使用固定256 MiB texture/64 MiB buffer和8帧retention，未按adapter budget、viewport或quality调节 | budget policy、pressure telemetry、admission/eviction与OOM fallback |

### 7.4 Product authoring、compiled packet与execution（P1-025至P1-032）

| ID | 当前差距 | 目标合同 |
|---|---|---|
| RG89-P1-025 | resource descriptor由name contains/match推断，未知buffer还按pixel count猜size | typed resource schema/catalog；名称只作debug label |
| RG89-P1-026 | first write的Clear/后续Load由全局`produced_texture_resources`字符串集合决定 | 每version显式initialization/Load provenance，由compiler验证 |
| RG89-P1-027 | `input_version`只帮助同stage排序，实际builder仍读“当前latest”whole resource | consumer直接绑定producer version/subresource handle |
| RG89-P1-028 | `CompiledRenderPipeline`同时保存`graph`、`stages`和`pass_stages`三份执行事实，`from_parts`不验bijective/order | 单一immutable `RenderGraphExecutionPacket`，stage是pass metadata而非第二authority |
| RG89-P1-029 | stage执行每pass扫描`graph.passes()`并比较String | compiled pass index/key直接寻址，steady frame不做名称搜索 |
| RG89-P1-030 | early/lighting/scene/post/history/late由硬编码函数顺序调度，不是compiler schedule的唯一消费 | 产品只遍历compiled batches；history/readback也建成graph epilogue pass |
| RG89-P1-031 | resolver按logical name返回整resource，不消费version、subresource或physical allocation ID | per-access binding table把logical version/range映射到view/buffer slice |
| RG89-P1-032 | dependency只进入context/record与CPU partition，不形成backend transition/fence执行 | execution严格消费compiler barrier/queue packet并产生completion receipt |

### 7.5 Lifecycle、cache、diagnostic与热路径（P1-033至P1-040）

| ID | 当前差距 | 目标合同 |
|---|---|---|
| RG89-P1-033 | begin_frame后bind/materialization/execution/readback encode多条错误return不release backing或end_frame | RAII frame transaction，Success/Abort都回收、推进health并终结readback/timer |
| RG89-P1-034 | submit后立即把backing返回pool，只因当前单queue顺序通常安全；未来multi-queue不成立 | submission completion token控制reuse，跨queue取max completion |
| RG89-P1-035 | IBL writeback在graph command buffers之后临时append，未进入graph dependency/lifetime | readback/writeback epilogue进入compiled packet并持有resource lease |
| RG89-P1-036 | cache固定16项、同步miss closure、无single-flight/prewarm/compile budget | generation-aware concurrent cache、background/prewarm、deadline与admission |
| RG89-P1-037 | cache key包含精确view/render尺寸，resize/DRS可产生同步compile churn | descriptor class/extent specialization策略与bounded variant policy |
| RG89-P1-038 | `extract_compile_fingerprint`对selected camera使用`expect`，cache key构造可panic | typed precondition/error，调用方持有validated frame compile input |
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

当前已实现`RG89-P0-001`的final pass名称唯一性和compiled pass ID直接执行切片，并写入独立优化记录；协调器分组Cargo与release性能门禁仍待运行，`RG89-P0-002`、`RG89-P0-003`及M1-M8尚未实现。未取得对应回执前，不能宣称Runtime89完成、动态测试通过，或性能/表现已经优于Unreal/Unity。
