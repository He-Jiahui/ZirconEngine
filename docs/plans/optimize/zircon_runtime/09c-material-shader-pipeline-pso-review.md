---
related_code:
  - zircon_runtime/src/asset/assets/shader
  - zircon_runtime/src/asset/assets/material
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/core/framework/render/shader
  - zircon_runtime/src/core/framework/render/material
  - zircon_runtime/src/graphics/material
  - zircon_runtime/src/graphics/shader
  - zircon_runtime/src/graphics/pipeline
  - zircon_runtime/src/graphics/scene/resources/pipeline
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache
  - zircon_runtime/src/bin/zircon_shader_prewarm
  - zircon_runtime/src/dynamic_api/shader_prewarm.rs
  - zircon_plugins/material_editor
  - zircon_plugins/rendering/features/shader_graph
  - zircon_plugins/shader_wgsl_importer
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/performance/02-unreal-aligned-engine-system-hard-cutover.md
  - docs/plans/performance/01/2026-08-15-renderer-material-shader-streaming-current-architecture-review.md
  - docs/plans/performance/01/2026-08-15-renderer-material-shader-streaming-protected-plan-routing.md
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - docs/plans/zircon_runtime/render/01-render-graph-rdg-alignment.md
  - docs/plans/zircon_runtime/render/02-mesh-draw-command-pipeline.md
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/ShaderCompiler/ShaderCompiler.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/ShaderCompiler/ShaderCompilerJobCache.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/ShaderPipelineCache.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/VertexFactory.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Public/MaterialShader.h
  - dev/bevy/crates/bevy_render/src/render_resource/pipeline_cache.rs
  - dev/bevy/crates/bevy_shader/src/shader_cache.rs
  - dev/bevy/crates/bevy_material/src/specialize.rs
  - dev/godot/servers/rendering/renderer_rd/shader_rd.cpp
  - dev/godot/servers/rendering/renderer_rd/shader_rd.h
  - dev/godot/servers/rendering/shader_compiler.cpp
  - dev/godot/servers/rendering/rendering_shader_container.cpp
  - dev/Fyrox/fyrox-material/src/shader/mod.rs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Data/Graphs/GraphData.cs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Data/Graphs/MaterialSlot.cs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Generation/Processors/Generator.cs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Importers/ShaderGraphImporter.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/ShaderStripping/ShaderPreprocessor.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: in_progress
source_recheck_required: true
---

# 09C · Material / Shader / Pipeline / PSO 工程化差距

## 1. 结论

Zircon 当前不是没有材质和 Shader 系统。现有实现已经具备 `.zshader v2` 的 surface/include/compute/fullscreen 文档，材质 property/option/texture-slot ABI，GeometrySource、ShadingModel、pass、quality 与 feature 维度，forward/GBuffer/depth/shadow/velocity/TAA 模板，Naga 与 WGPU 校验，内存 variant registry、压缩磁盘缓存、预热 manifest、miss/diagnostic report，以及 Vulkan driver pipeline cache。Mesh 路径也已经能把 static/skinned/morph/skinned-morph/VG 与材质正交组装；这些能力必须保留，不能退回每个功能内嵌一段 WGSL 或一个 `create_render_pipeline` 的临时实现。

但这些能力没有组成一个工程级编译与发布系统。资产导入、include 解析、模板拼接、Naga 校验、WGPU module、Mesh PSO、后处理/UI/粒子 PSO、预热磁盘缓存和 driver cache分别拥有键、线程、错误、I/O 和生命周期。聚焦范围内有三套公开 Shader variant/graph DTO；横向产品源码仍有47处 `create_render_pipeline`，分散在44个文件，生产 Rust 中出现57处 `cache: None`而`cache: Some`只有1处。所谓 pipeline cache主要覆盖Mesh局部，不能代表整个renderer已有PSO authority。

运行时编译的默认语义也不工程化。异步编译默认关闭；开启后只覆盖Base Mesh变体，并在首帧或失败时返回 `None`，以 `SkipDraw`让几何体静默消失。其他Mesh pass和绝大多数builtin pipeline仍同步创建。两个Mesh私有compiler线程、串行prewarm worker和Vulkan driver cache互不协调；同步finish、无期限join、启动读取和`Drop`持久化都绕过Runtime11的统一预算、优先级、取消和deadline。

缓存正确性存在硬缺口。预热source ID包含完整WGSL、include hash、template revision、Naga/WGPU版本，但磁盘key只包含canonical variant string和include hash；lookup不接收期望source ID或compiler/backend版本，版本只作为metadata写入。因此一次命中不能证明产物属于当前源码、编译器和后端。Mesh产品路径还用常量 `wgpu-runtime`作platform token，没有设备generation、adapter/driver capability、target format、sample count或pipeline-layout ABI的统一身份。

作者工具表面同样超前。Material Editor确有递归求值、cycle检测和11个直接单测，不是全空壳；但六个命令只注册descriptor，没有operation factory，引用的`graph.zui`和default material graph模板不存在。其compiler只把少量节点常量折叠成传统`MaterialAsset`，纹理只能直连base color，texture-backed math直接报错，不生成typed material IR或WGSL。另一个optional `shader_graph`插件又维护第二套graph model，按输入顺序字符串拼WGSL，不做ID/pin/type/topology验证，其post-process executor是noop；Editor feature只有descriptor。这两条路径都没有进入canonical artifact/PSO generation。

本轮登记7项P0、16项P1、6项P2。P0先建立唯一artifact/generation identity、共享compile service、全renderer PSO authority、原子hot reload/last-good、严格readiness/ABI、真实Material Graph产品链和可见失败策略；P1再收敛source DAG、模板IR、variant normalization、material binding、prewarm/cook、driver cache、Editor diagnostics和测试；P2才进入分布式编译、remote DDC、vendor/native binary library、全项目PSO usage capture与高级pipeline domain。完成真实cold/warm/reload/device-loss/100k variant/多平台捕获前，不能声称Shader/PSO工程化完成或性能优于Unreal。

## 2. 审查边界与覆盖

### 2.1 已读范围

| 子域 | Rust文件 / 物理行 | `#[test]` | 本轮判定 |
|---|---:|---:|---|
| asset shader | 8 / 1,969 | 5 | E3：资产、依赖、readiness、property layout与zshader v2 |
| asset material | 14 / 2,325 | 1 | E3：材质实例、parent/dependency、validation与management DTO |
| `graphics/material` | 5 / 891 | 16 | E3：builtin/plugin shading model与include source解析 |
| `graphics/shader` | 32 / 10,824 | 129 | E3：global ABI、template、IDE、variant disk/prewarm与测试 |
| `graphics/pipeline` | 49 / 8,778 | 66 | E3：async compiler、driver cache；Render Graph authoring引用09A |
| Mesh pipeline | 10 / 1,869 | 31 | E3：各pass WGPU pipeline descriptor与创建入口 |
| Mesh pipeline cache | 18 / 7,701 | 60 | E3：variant registry、source、disk、prewarm、async与fallback |
| core render shader | 21 / 4,747 | 37 | E3：asset ABI、geometry、variant、prewarm、IDE与diagnostics contract |
| core render material | 48 / 6,616 | 44 | E3：material ABI、readiness与management/query surface |
| shader prewarm CLI | 21 / 7,893 | 78 | E3：inventory、module dependency、manifest、validation与输出 |
| Material Editor | 6 / 959 | 11 | E3 compiler/validation；E1 product operation/template |
| Shader Graph feature | 6 / 278 | 2 | E3 compiler/feature registration；E0 render execution |
| WGSL importer | 4 / 503 | 7 | E3 parse/validate/import；E1 surface ABI semantics |

以上唯一聚焦集合合计242个Rust文件、55,353物理行、487个test属性；另有42个WGSL文件、3,828行。还定向追踪了`dynamic_api/shader_prewarm`、SceneRenderer启动配置、ResourceStreamer shader/material consumer和全`zircon_runtime/src`的pipeline/module创建点。横向统计排除了常规tests目录，但仍可能包含名称不符合tests约定的fixture；它用于证明authority分散，不用于声称精确启动创建数量。

2026-08-15 performance current review已经冻结过material/pipeline/shader/resource-streamer/backend 136/136 Rust文件，本轮复用了其逐文件结论并重读所有current modified/untracked重点、作者插件、asset/core contract、Mesh consumer和横向PSO callsite。`graphics/material`、`graphics/pipeline`、Mesh cache、Shader template/cache/WGSL及core contract当前有其他Session大量tracked或untracked修改，故实现前必须重新fingerprint并标记`source_recheck_required`。

### 2.2 与09A、09B、09D的归属

- 09A拥有Render Graph resource version、pass dependency、compiled graph cache、RHI product implementation、submission ticket和GPU lifetime；本篇只要求PSO artifact通过该RHI owner发布，不重复设计RDG拓扑。
- 09B拥有persistent render scene、visibility、GPUScene、mesh command/indirect和material command cache消费顺序；本篇拥有material/shader/PSO identity与prepared binding artifact。
- 09D将拥有texture/mesh/material residency、streaming budget、upload和eviction。本篇只规定shader/material依赖如何产生dirty generation，不展开资源流送算法。

### 2.3 参考引擎边界

- Unreal是上限参考：ShaderCompiler集中提交/处理作业，JobCache按完整compiler input hash做single-flight和内存淘汰，ShaderPipelineCache以显式batch/time/memory预算预编译PSO；MaterialShader/VertexFactory把材质、pass、几何来源和platform编译环境组合成确定身份。可迁移原则是唯一输入身份、全局作业authority、异步发布、可预算PSO和cook/runtime分层，不是复制UE宏或worker协议。
- Bevy是WGPU可行性参考：PipelineCache公开稳定ID及`Queued/Creating/Ok/Err`，异步任务完成后发布；ShaderCache维护imports、waiting和reverse dependents，源变化只重新排队受影响pipeline。Zircon不能用WGPU解释私有线程、同步finish或全局反向依赖缺失。
- Godot以`ShaderRD::Version`持有variant数组、group compile task、dirty/valid状态和cache load；它仍有字符串模板成本，但shader version与编译group是显式生命周期对象。Zircon当前asset revision、source cache、variant registry和PSO HashMap没有等价原子version owner。
- Unity Shader Graph用typed `MaterialSlot`、GraphData validation、upstream node收集、active field/requirement传播、Target/Pass descriptor和Importer dependency生成多目标Shader；core pipeline还在构建期执行variant stripping/report。Zircon的两个graph compiler目前只覆盖字符串/常量求值，不能按“节点存在”宣称Shader Graph完成。
- Fyrox的`ShaderDefinition`将resources、passes、draw parameters、source和disabled passes放在单一资产定义，并保留source line位置。它不是Unreal级编译上限，但证明较小引擎也应有可序列化pass/resource authority；Zircon的Raw WGSL importer和parallel DTO尚未稳定跨过此基线。

### 2.4 明确未做

- 没有修改production code，没有运行Cargo、Editor/App、真实GPU、PIX/RenderDoc、WPR、driver cache、device loss、shader compile storm、cook/export或跨平台测试。本篇是current-source静态审查和重构计划，不是实现验收。
- 没有逐审lighting/shadow、post/temporal算法质量；这里只统计它们的PSO创建authority。它们进入后续09E/09F单元。
- 没有要求P0立即复制Unreal分布式worker、所有native shader format或PSO database。P0要求先让正确身份、生命周期、失败和发布闭环成立。

## 3. 当前必须保留的基础

### 3.1 `.zshader v2`和material ABI已有可迁移骨架

文档区分surface/include/compute/fullscreen，拒绝未知字段和错误stage；property/option/texture slot能生成layout hash、option bits和WGSL material block。`ShaderAsset`保留entry points、dependency、render state、queue、disabled pass、resource与pipeline layout。这些字段应降为immutable source/artifact输入，而不是在硬切换时删除后重新发明。

### 3.2 GeometrySource和ShadingModel注册避免了材质与形变的硬编码笛卡尔积

static/skinned/morphed/skinned-morphed/VG描述符、required bindings/defines、builtin/plugin shading model ID、GBuffer channel mask和重复ID/token校验已经存在。目标是把注册表纳入plugin/catalog generation、给ID分配和卸载代际，而不是恢复fallback shader特判。

### 3.3 variant source table、miss report和磁盘压缩提供了可升级的观测面

prewarm manifest已经把共享source与variant request分开，source ID包含WGSL和compiler版本，source validation在batch内按ID去重；miss report记录memory/disk/compile和pipeline diagnostics。它们应迁入canonical artifact service，而不是因当前cache key错误整体丢弃。

### 3.4 pipeline descriptor和error scope测试具有局部正确性价值

各Mesh pass明确声明vertex layout、blend/depth/cull与entry point，WGPU error scope能捕获异步creation错误；大量Naga/WGPU产品测试验证了形变、环境和pass source。这些局部测试要继续存在，但验收必须增加跨代际、cache、device和真实frame行为。

## 4. P0 差距清单

### P0-1：没有唯一Shader artifact和PSO authority，整个renderer仍直接创建对象

产品源码横向可见47处`create_render_pipeline`、67处`create_shader_module`，分别分散到44和60个文件。Mesh拥有多张pass HashMap、两个compiler和driver cache；post、UI、sprite、particle、overlay、deferred、froxel、OIT、SSR、TAA、DOF、bloom、IBL和output converter分别在构造或首次使用时创建自己的module/layout/pipeline。生产Rust中57处descriptor写`cache: None`，只有1处`cache: Some`。

目标建立唯一`ShaderArtifactService`和RHI-owned `PipelineService`。所有render/compute pipeline必须通过typed descriptor/content ID申请稳定`PipelineHandle`，状态统一为`Missing/Queued/Creating/Ready/Failed/Retired`；consumer只能poll handle或使用明确fallback。直接WGPU创建仅允许RHI backend内部和隔离测试。`graphics/pipeline`不能只缓存render graph schema，Mesh cache也不能继续充当全引擎PSO owner。

### P0-2：历史磁盘cache key不能证明source/compiler/backend一致（source-cache 身份子边界已修复，完整 artifact/PSO identity 仍开放）

以下为初始审查时的缺口，当前状态由第14、17和34节取代：旧 `ShaderVariantPrewarmSource` 的 ID 包含 label、WGSL、include hashes、template revision、Naga 和 WGPU 版本；旧 `ShaderVariantCacheDiskKey::from_variant_key` 却只 hash `ShaderVariantKey::canonical_string` 和 include hashes。写入 meta 包含 template/Naga/WGPU 版本，但 lookup 只验证 schema、hash 与 canonical string，不接收 expected source ID/version。compiler 升级、template 生成器改动或同 revision 源码变化都可能被旧 entry 命中。当前 source-cache 已以完整 source contract 和 payload rehash fail-closed，但 target/device/pipeline identity、compiled artifact 与共享 service 仍未闭环。

目标定义一个不可拆分的`ShaderArtifactId`：source/module DAG content IDs、typed definitions、material layout/options、geometry/shading/pass/quality、template compiler revision、Naga/backend compiler version、target/platform capability profile共同进入hash。disk metadata只用于诊断。读取API必须接收完整expected ID并返回typed miss reason；禁止以读后再比较WGSL字符串补救错误key。corrupt/stale清理作为有预算I/O job执行。

### P0-3：编译调度是三套互不兼容的局部机制，默认仍同步阻塞

Mesh cache构造`mesh-shader-validate`和`mesh-pipeline-compile`两个私有OS线程，bounded channel只限制pending count；`finish_pending*`阻塞recv，`Drop`无期限join。异步pipeline默认false，即使开启也只覆盖Base pass。prewarm公开预算明确拒绝`max_in_flight_variants != 1`并串行做WGPU module/pipeline validation与disk write。Vulkan driver cache同步读最多64MiB并在`Drop`同步`get_data`和atomic write。

2026-08-26 对全部 7 个 Mesh PSO 创建调用点的后续复核又确认一个独立正确性缺陷：`track_pipeline_creation_error_scope` 在入队前已经用 `pollster::block_on(error_scope.pop())` 同步解析结果，但旧实现仍把成功结果保留到下一帧入口或 prewarm `finish`；帧入口只在本帧创建前 drain 一次，因此同一帧前 64 个成功 scope 会占满所谓 pending 队列，第 65 个本来合法的 PSO 被当成“诊断队列饱和”而失效并进入终止 failure。这不是 workload budget，而是把已完成 receipt 错当成未完成 job。源码修复保持 64 项内存上限，在 rollover 时先消费整个已解析批次，再保留当前 receipt；不再因成功 receipt 数量拒绝 PSO，prewarm 对当前成功 scope 的消费语义保持不变。同步 `block_on` 仍是 P0-3 的结构性 stall 候选，不在没有 profile 时伪装成已优化；源码已用 `render/shader_pipeline/wgpu_pipeline_error_scope_pop` scope 以及 `mesh_pipeline_diagnostic_queue_depth` / `mesh_pipeline_diagnostic_rollover` counters 接入既有 profiling/Tracy 通道。托管测量必须覆盖 cold 1/64/65/1,000 variants，分别记录 scope CPU p50/p95/p99、frame submission wait、terminal failure 数和峰值队列长度，并证明 65 边界不再产生假失败。

目标由Runtime11共享executor承载preprocess、Naga、backend affinity create、compression/I/O和publish。按content key single-flight，预算至少覆盖job count、source bytes、resident bytes、driver bytes、priority、age与deadline；支持取消、supersede、shutdown drain/abort。render/editor线程不得调用finish/wait、filesystem或driver persist。compiler worker若需要进程隔离可在P2升级，但P0先删除每cache私有线程和析构阻塞。

### P0-4：没有跨asset/source/variant/PSO/prepared-material的原子generation与last-good发布

Asset facade能发Modified并保留resource generation，但Shader import反向依赖、template source、variant disk、Mesh module/PSO HashMap、material bind group和compiled pipeline各自失效。repo搜索没有发现product shader-change到Mesh pipeline invalidation的闭环；`ShaderCache`式reverse dependent graph只在prewarm CLI inventory局部存在。一次reload可能让新material layout、旧WGSL、旧bind group和旧PSO同时可见。

目标链固定为：

`AssetCatalogGeneration -> ShaderModuleGraphGeneration -> ShaderPermutationGeneration -> PipelineLayoutGeneration -> RhiPipelineGeneration -> PreparedMaterialGeneration`。

每层发布immutable artifact和reverse edges；变更计算affected closure，后台编译全部required产物后一次原子切代。失败保留last-good generation并发布诊断，删除/ABI不兼容使用共享error material。old generation按09A submission completion retirement，不按CPU frame或HashMap替换立即释放。device loss产生新device generation并使所有GPU handle fail-closed。

2026-08-26 对 material 到 draw payload 的调用图复核确认，`ensure_material` 原先在三个 blocking readiness 条件（非法 mask cutoff、缺 runtime shader source、不支持的 UV channel）下先把失败 candidate 覆盖进 `ResourceStreamer::materials`，随后返回错误；`ensure_scene_resources` 再以 `?` 终止整帧。与此同时 `MeshDrawCommandPayload` 已把 pipeline key/variant 与 custom/standard 两套 material bind handle 保存在同一 immutable payload，Base replay 也会在 builtin fallback shader 下选择对应 standard binding；因此正确的最小切入点不是再造一个孤立 PSO，而是禁止不可绘制 candidate 破坏已发布 bundle。当前源码已扩展这个叶子边界：blocking readiness、shader dependency preparation error 和 texture residency error 遇到已有可绘制 `PreparedMaterial` 时都保留其 revision/runtime/pipeline key/uniforms，在同一 entry 上保存 rejected candidate revision/readiness report，render 继续 last-good；dependency error 以 `DependencyResolution` validation 投影给管理面。root `PreparedShader` 只在 import/registry dependency closure 成功后发布，失败 reload 不再提前覆盖其旧 source。成功材质候选先确保全部纹理，再创建 custom/standard uniform buffers，失败候选不再在纹理 residency 前产生两次无用 GPU buffer 创建。cold failure 仍返回原始错误并保持 fail-closed。

同轮 cache identity 复核发现，旧 `PreparedMaterial` 只比较 material revision、texture upload support 和 texture revisions，直接 shader-only 修改会被入口 cache hit 完全吞掉。该阶段先保存了 direct shader `(locator, ResourceId, revision)` snapshot；第 15 节随后以 readiness reverse closure 和 process-local dependency revision 完成了传递 shader generation，取代本段“尚无 transitive generation”的阶段性结论。逐项 texture ensure、PSO readiness、显式 error material、跨 pass 原子发布和 submission-fence retirement仍未解决。rejected candidate 也仍不能仅按 shader generation 抑制重试，因为精确恢复身份还必须覆盖 root load-state、texture residency 和 GPU upload failure。P0-4/M6 不前移。

源码已加入 `render/material/prepare` scope，以及 `material_prepare_cache_hit`、`material_prepare_rebuild`、`material_last_good_rejection`、`material_uniform_buffer_creations`、`shader_artifact_publish` counters。受管 profile 必须覆盖 1/100/10,000 materials 的 warm cache、direct shader-only valid reload、invalid dependency reload 和 recovery：warm steady state 的 rebuild 与 uniform creation 必须为 0；invalid reload 必须保留 draw、root shader publish 为 0、uniform creation为0；recovery 只能发布一次新 shader/material generation。另记录 prepare CPU p50/p95/p99、registry lookup、allocation、RSS、frame submission wait 与 300-frame invalid candidate retry次数。后者在完整 closure identity 前预计仍非零，只作为下一结构性修复的瓶颈证据，不伪装成已消失。

### P0-5：Shader/Material readiness和ABI校验会产生false-ready资产

WGSL importer用Naga验证语法后无条件写`ShaderAssetKind::Surface`，dependencies/source_files/imports/property/resources/layout均为空。`ShaderReadinessReport::is_ready`只要求有runtime WGSL、已有entry/defs无诊断和validation diagnostics为空；它不要求至少一个entry point、surface所需vertex/fragment contract或pipeline layout。因此compute-only/不匹配entry的合法WGSL可被标记ready，直到具体pipeline创建失败。

目标按kind/target声明或reflection artifact建立严格readiness：surface必须满足约定entry、stage IO、bind groups、material function和pass ABI；compute/fullscreen分别校验entry/workgroup/resource contract。Raw WGSL importer不得猜surface；缺少`.zshader` descriptor时只能导入generic module artifact，不能直接成为material shader。reflection/layout hash必须来自编译artifact并与authoring schema对拍，禁止两套手写layout独立通过。

2026-08-26 当前 source 继续完成该 P0 的第二个基础子边界：`ShaderAsset::surface_source_contract()` 以注释感知、零分配的单趟扫描把显式 Surface 分类为 `MaterialFunction`；material function 同时必须具备唯一 `ZrVertexOutput` 参数（也接受 `zr_surface_types.wgsl` 的公开 `ZrSurfaceInput` alias）与 `ZrSurfaceOutput` 返回。缺失/重复入口、签名不匹配、material function 与 executable entries 混用以及 full-pass Surface 均返回 typed error。readiness 消费同一分类；streaming 发布一次分类结果，mesh pipeline source resolution 只做 O(1) 枚举比较，不再逐请求扫描整份 WGSL。仓库 6 个 tracked Surface package 均使用 canonical material function；唯一旧 full-pass Surface 是 builtin PBR，且 fallback key 运行时本就组装标准模板而不消费 raw source，因此兼容分支已硬删除。builtin PBR 改为无 executable entry rows 的 225-byte material function，较旧 24,220-byte 拼接载荷减少 23,995 bytes / 99.07%，两份 resident source string 直接减少约 47,990 bytes；该数据仅是源码规模，不代表 CPU/GPU/功耗收益。三个曾写成不存在 `ZrMaterialSurface` 返回类型的 streaming 夹具同步修正，不再依赖旧 substring gate 掩盖坏 ABI。compiler reflection 已从同一遍 Naga `Module + ModuleInfo` 发布 entry stage IO、entry-reachable resource、merged visibility 与 interface/resource layout hash；本切片静态硬切换契约 `15/15` 与限定格式通过。但 authored schema 对拍、specialization identity 和 WGPU fail-closed admission 尚未完成，因此 P0-5 保持未完成；Cargo/WGPU/RenderDoc、1/100/10k profile、RSS、GPU timing 与功耗数据均待托管验收。

specialization identity 已继续收敛基础边界：Naga 29 的 module override count、逐 entry workgroup override 维度，以及 stage-IO/resource type 的 specialization dependency 现在显式进入 reflection；dependency flag 参与 layout hash，raw Naga handle index 不参与稳定身份。当前 tracked WGSL `override` 为 0，139 个 WGPU `compilation_options` 行全部为 default，因此该切片不新增当前 permutation/PSO 维度。未来 exact `PipelineDescriptorId` 必须携带 selected constant values，不能把依赖 specialization 的 reflection hash 当成 exact ABI。specialization 静态契约 `10/10` 通过，workgroup 与 override-sized resource 两项 Naga regression 已暂存但未本地执行；P0-5 与托管门禁不变。

同日对 `ShaderAsset.pipeline_layout` 的 owner 复审否决了“直接让 reflection 对拍 authored DTO”的下一步：`.zshader v2` 已禁止该字段，当前 Surface package 全部导入空 descriptor，material ABI diagnostic 对空值直接 opt-out；生产路径仅做 cache clone/readiness report，`pipeline_layout_descriptor()` 没有生产消费者，也没有 WGPU layout 由该 DTO 构造，字符串 push-constant ranges 同样无运行时消费者。P0-5 的正确 admission authority 是 specialized reflection 对拍实际 pass pipeline-layout owner，再通过 WGPU validation scope 发布 module/PSO；共享 layout 的未引用额外 binding 可保留。DTO 删除是独立 serialized schema/cache hard cut，须先完成 migration inventory，本切片不实施。性能与里程碑状态不变。

### P0-6：Material/Shader Graph产品表面未连接，且存在两套不兼容资产模型

canonical `MaterialGraphAsset`只有6种节点和String pin；Material Editor validation不验证from/to pin合法性、pin类型、重复incoming edge或完整topology。compiler递归常量求值，texture-backed Add/Multiply失败，输出只是传统MaterialAsset字段。六个命令没有operation factory，两个引用资源不存在。optional Shader Graph另有7种节点/第二套asset，按Vec顺序输出未消毒identifier和引用字符串；缺ID、type、cycle、missing-ref、binding和target/pass验证；无output时返回magenta WGSL，post executor永远`Ok(())`不编码任何命令。

目标硬切到一个typed `MaterialGraphDocument -> MaterialIR -> SurfaceArtifact`链。graph node/pin使用stable schema ID和value type，编译先做topological/type/stage/domain验证，再生成可复用IR、uniform/texture ABI、source map与per-pass surface函数。Material Editor命令必须绑定transactional operation handler，真实ZUI/template随plugin打包；preview消费同一artifact service和PSO handle。删除plugin-local ShaderGraphAsset、noop executor和dead `graphics/shader/shader_assets.rs` DTO，或迁移后明确只留一个canonical model。

### P0-7：PSO miss/failure语义以`SkipDraw`隐藏正确性问题

Base异步compile首次miss、queue full、worker unavailable或terminal failure都可能返回None；默认placeholder policy为`SkipDraw`。这避免frame thread等待，但把错误表现为几何体消失。异步又默认关闭，普通产品更可能在首次见到variant时同步source/cache/pipeline创建。DepthOnly枚举存在但Base固定SkipDraw；其他pass没有统一last-good/error policy。

目标按pipeline domain定义显式policy：required startup PSO在受控bootstrap deadline前ready；hot reload使用last-good；新可选variant使用共享error material/diagnostic draw或明确defer；depth/shadow/velocity不能随意用color fallback。每次miss/late/fail都带variant ID、material/entity/view、state age和fallback reason，进入frame/profile/editor。产品测试必须证明失败不会静默消失、不会阻塞submission，也不会把不兼容last-good与新binding混用。

2026-08-26 当前 source 已暂存 P0-7 的 Base-pass 前置闭环。旧 `PipelinePlaceholderPolicy::{SkipDraw, DepthOnly}` 和 `Option<RenderPipeline>` admission 被删除，Base 现在返回 `PipelineAdmission::{Ready, Deferred, Failed}`；`compile_queued`、`compile_pending`、`queue_saturated` 是可恢复 defer，worker unavailable、job panic、unknown variant、wrong pass、geometry/source 缺失和 WGPU pipeline validation failure 是终止 failure。每个原因变化会重置 state age，避免把 queue age 错算为 failure age。Opaque/transparent Base consumer 将 variant ID、canonical shader/material identity、entity、consumer、state/action/reason、age 和 occurrence 写入每帧有界 `ShaderVariantMissReport`；重复上下文原地合并，最多保留 8 条诊断，正常 ready draw 不分配诊断字符串；缓冲区满载后的新上下文只累计总数，并在 canonical key/consumer/reason 分配前返回，已有上下文仍更新次数和最大 age。共享 `mesh_pipeline_shader_source_with_cache` 的所有分支本就选择 disk hit 或 assembled WGSL，返回类型现由 `Option<String>` 硬切为 `String`，8 个 mesh pass 不再以 `?` 伪造 cache-source failure；真实 source assembly failure 和 OIT entry 缺失保持显式。

同日的其余 pass 调用图复核确认旧 GBuffer、DepthPrepass、ShadowDepth、Velocity、TAA reactive mask 与 OIT 都以同步 `Option<&RenderPipeline>` 折叠 unknown/wrong-pass、geometry/source 与 WGPU validation failure，部分消费端随后 `expect`，形成 frame-path panic。共享失败/age 表不能只按 variant id 泛化：OIT 故意复用透明 Base variant id，却拥有独立 PSO/layout。基础设施因此以 `(PipelineCreationTarget, MeshPipelineVariantId)` 为键，Base async 开关只清理 Base 域，并以同 id 的 Base/OIT 双键测试锁定。

随后完成全部 mesh consumer 的源码级 typed admission 收敛。GBuffer 的 command projection 与同步 PSO 创建、DepthPrepass、Velocity、ShadowDepth/ShadowDepthAlphaMask、TAA reactive/material mask 以及 OIT 都返回 `PipelineAdmission`；ready path 在 cache hit 后直接取已发布 PSO，不投影或克隆完整 variant key。terminal failure 分别记录 `deferred_gbuffer`、`depth_prepass`、`shadow_atlas`、`velocity_object`、`taa_reactive_mask` 和 `oit_fragment_store` / `RejectDraw`，并在拒绝 draw 后失效 replay state。Shadow 与 TAA 由 consumer 传入 exact kind，使共用缓存容器内的子域失败互不污染；OIT 保持独立 `PipelineCreationTarget::Oit`，并把缺少 `fs_oit` 单独分类为 `oit_fragment_store_unavailable`。OIT 图执行仍在 mesh admission 失败后返回错误，没有把终止失败静默降级为成功。TAA 两类 PSO 的重复源码组装、module 创建和 error-scope 流程已合并为一个按 exact kind 分派的创建状态机。旧五类同步 `Option` API 与生产帧路径 `expect` 已删除，并由集中源码契约测试锁定。

这仍不是完整 fallback。Unreal `MaterialRenderProxy::GetMaterialWithFallback` 与 Base/Depth/Shadow mesh processor 都在完整 shader map 与对应 material proxy/binding 之间一起切换；因此 Zircon 不能只换一个 ABI 可能不兼容的 error PSO。当前 deferred draw 明确报告 `DeferDraw`，terminal failure 明确报告 `RejectDraw`；last-good generation、共享 error material+binding 原子切换、view identity、required bootstrap deadline 和统一 cross-pass artifact publication 仍未实施。Bevy 的 `Queued/Creating/Ok/Err` 证明 WGPU 下显式状态可行，但其 consumer 把非-Ok 再折叠为 skip 的做法不作为 Zircon 完成标准。Cargo/WGPU/PNG/RenderDoc/profile/功耗均未在本切片运行，历史 WGPU 结果不能覆盖本次 hard cut，P0-7 保持 `in_progress`。

## 5. P1 差距清单

### P1-1：Shader模板每variant重复分配、全文替换、拼接和解析

`ShaderTemplateInclude::new`反复拥有token/source/owner、扫描include、strip directive并hash派生module；builtin registry每次assembly重建。material surface用11次顺序`String::replace`专化大段WGSL，随后forward/deferred/TAA各自拼接、重命名entry和Naga parse。目标缓存parsed module DAG和typed specialization slots；同source generation parse一次，variant只patch definitions/IR并共享line map。

### P1-2：plugin shading include解析按descriptor/token/ready-record三重扫描

每个plugin shading model的forward/GBuffer/deferred token都过滤全部ready shader records，反复trim、slash replace、lowercase、suffix比较，命中后同步load并clone WGSL。目标由AssetCatalogGeneration发布`NormalizedImportToken -> ShaderModuleArtifactId`索引及冲突诊断；stable generation扫描/normalize/load/source clone均为0。

### P1-3：variant identity重复且部分维度只存在于局部key

canonical `ShaderVariantKey`、entry-point `RenderShaderVariantKey`、Mesh `PipelineKey/MeshPipelineVariantKey`和dead public `graphics::shader::ShaderVariantKey`并存。Mesh key含alpha cutoff、texture presence和render-state booleans；shader key只投影部分feature。target format/sample/layout、device/backend capability不在统一key。目标区分`SourceVariantId`、`PipelineLayoutId`和`PipelineDescriptorId`，由typed lowering唯一生成，禁止consumer手拼或clone整套key。

### P1-4：GeometrySource公开ID范围与`packed_dims`布局冲突

旧 `ShaderVariantKey::packed_dims` 只给 geometry 分配 bits `0..3`，却允许 `GeometrySourceId` 使用完整 `u8` 插件段；geometry >15 会与从 bit 4 开始的 shading model 重叠。2026-08-26 current source 已硬切为 54-bit 无重叠内存布局：geometry `0..7`、shading `8..15`、pass `16..19`、features `20..51`、quality `52..53`。没有产品 consumer 或持久化 entry 依赖旧 packed value，`canonical_string_v1` 及磁盘 identity 不变；回归锁定 geometry=16 与 shading=1 不再碰撞。限定格式、diff integrity 和静态合同通过，Cargo/序列化产品验证仍待里程碑测试阶段。

### P1-5：texture-presence变体只计数“可归一化”，没有实际归一化

旧实现的`MeshPipelineVariantRegistry::has_texture_presence_equivalent_variant`枚举16种texture-presence组合，却只增加计数而不复用existing variant。2026-08-26 current source 已硬切绑定与pipeline identity：base-color、metallic-roughness、occlusion、emissive使用固定ABI与中性fallback，不再进入`PipelineKey`或`ShaderPipelinePrewarmState`；normal texture因改变切线框架与生成代码，仍通过`HAS_NORMAL_TEXTURE`进入shader key。16路等价扫描已删除，当前注册路径只做一次canonical key lookup。source implementation staged，Cargo/WGPU/profile待受管验收。

### P1-6：ShadingModel/GeometrySource注册没有catalog generation和卸载生命周期

注册表能拒绝重复ID/token和unsupported GBuffer channel，这是正向基础；但ID由plugin descriptor静态提供，include source又从当前ready catalog即时解析。没有plugin generation、owner、quiesce、unload/reload remap和artifact revocation。目标由plugin catalog原子分配/验证稳定IDs，descriptor携owner generation，卸载等待PSO/material consumers退休后撤销。

### P1-7：Mesh variant/module/pipeline HashMap没有容量、年龄、字节和device-generation淘汰

variant IDs和多张pass pipeline map只增长，shader module也按String key常驻；没有场景/project切换、material删除、quality/platform切换或device reset的统一retire。目标由PipelineService按resident bytes/count/last-use/submission fence管理；stable handles可复用，evicted GPU object回到Queued而非ID失效。

### P1-8：prewarm验证使用串行临时device/layout，不能证明运行时精确命中

prewarm manifest按variant串行创建module/pipeline，validation helper重建临时layout、GPUScene与joint palette资源。它验证“某个兼容descriptor可创建”，不一定是产品exact pipeline layout、target format、feature profile和driver cache entry。目标让cook/prewarm调用同一typed lowering与RHI pipeline descriptor，产出exact artifact IDs；启动报告证明required set全部disk/source/driver命中且runtime compile miss为0。

### P1-9：driver cache仅Vulkan，启动和析构执行同步I/O

Windows MVP DX12明确走UnsupportedBackend；Vulkan seed最大64MiB同步读取/hash，`Drop`同步读取driver blob并写盘。没有age/total bytes/driver version policy、write coalescing或shutdown deadline。目标把backend pipeline library作为RHI capability；DX12/Vulkan/Metal分别报告支持和原因，I/O由artifact job执行，显式flush ticket可超时/取消，Drop只释放内存句柄。

### P1-10：material bind group/prepared state没有generation级共享

09B已确认draw构造晚于材质解析；既有performance review还发现Mesh draw可为同一material generation重复创建custom/standard bind groups。当前material asset、uniform bytes、texture slots、layout和pipeline选择在ResourceStreamer/Mesh builder间多次投影。目标发布`PreparedMaterialHandle`，按(material generation, layout, texture generations, device generation)唯一拥有uniform allocation、bind group和fallback policy；stable frame create/upload=0。

### P1-11：材质parent/dependency、Shader import和PSO invalidation没有同一reverse DAG

MaterialAsset有parent、texture、shader依赖和readiness；ShaderAsset有dependencies/import redirects；prewarm CLI又建立文件module dependency。三者没有共享graph owner。目标Runtime04只发布一张typed dependency DAG，cycle、missing、redirect和affected closure由同一算法生成；render、cook、IDE和management view消费同一snapshot。

### P1-12：core material management surface体量大于产品消费证据

`core/framework/render/material`有48文件、6,616行，其中management拥有overview、status/issue index/view、query/filter/facet/page/selection/action和大量测试。产品搜索主要落在AssetManagement record构建和ResourceStreamer accessor，没有确认Editor稳定消费闭环。现有API会clone/sort/detail material rows。目标先确认真实consumer；只发布generation-owned compact row/index，details按selected ID惰性取。未消费的query层删除或移到Editor，不应常驻runtime core contract。

### P1-13：Shader IDE/preview重新建立环境，而非消费compiler artifact

IDE env generation、preview、validation拥有独立source/include/diagnostic拼装；Editor typing/reload可能再次扫描和复制，而不是借用runtime/cook的module graph、source map与compile ticket。目标Editor只提交overlay source generation并消费相同diagnostic artifact；debounce/coalesce/supersede旧请求，UI线程零parse/Naga/WGPU/I/O。

### P1-14：编译与cache diagnostics缺少统一可操作身份

局部miss report很详细，但后处理/UI等direct pipeline无同等状态；错误字符串跨Naga/WGPU/material/graph/cache边界丢失source generation、include stack、node/pin、pass、device和fallback。目标统一`ShaderDiagnosticId`与source map，支持node -> IR -> generated source -> backend message映射，并按artifact/consumer聚合，不把每帧重复错误刷成新记录。

### P1-15：cook/export没有把required Shader/PSO集合变成发布门槛

prewarm CLI能扫描并写cache，但runtime仍允许任意首次编译；manifest与package/export成功没有“required artifact完整、target profile精确、runtime miss=0”硬门。目标cook按scene/material/plugin/quality/platform收集required和optional集合，执行variant stripping、编译、签名和pack；shipping默认禁runtime source compilation，缺required artifact直接阻止发布或启动。

### P1-16：测试数量高，但过多锁source shape，缺跨生命周期产品矩阵

487个test属性中有大量`include_str!`、`.contains(...)`和结构预算断言；有价值的Naga/WGPU测试也多为单pipeline/单device。缺cold/warm restart、compiler version bump、corrupt cache、plugin reload、material parent affected closure、compile storm、queue saturation、device loss、last-good、shipping no-compile和全pipeline inventory。目标用behavior/artifact identity/product pixel tests替换源形状锁。

## 6. P2 差距清单

### P2-1：没有隔离Shader compiler worker process和crash quarantine

P0共享executor先解决权威和预算；之后把不可信/高内存compiler job移入可重启worker process，支持heartbeat、OOM/crash诊断和job retry/quarantine，避免拖垮Editor/runtime。

### P2-2：没有本地/共享DDC和跨机器content-addressed artifact复用

当前cache是project本地WGSL压缩文件。高级目标是分层local/shared/CI artifact store、签名与provenance、LRU/bytes/age、并发single-flight upload/download和离线可复现；不能在identity未修复前先上远端cache。

### P2-3：没有完整native binary/PSO library跨backend策略

未来按RHI capability支持DXIL/SPIR-V/Metal library、driver PSO library和pipeline binary，key包含adapter/driver/compiler/feature profile。WGSL仍可作为portable source artifact，但shipping不应在所有平台强制现场翻译。

### P2-4：没有真实PSO usage capture、合并、排序和持续profile治理

建立session usage capture、first-use hitch、frequency、scene/quality/platform provenance；离线合并生成startup/level/optional batch并按时间/内存预算排序。cache autosave必须避免持大锁和析构I/O。

### P2-5：Material Graph缺subgraph/function/custom code沙箱与增量编译

在typed IR稳定后增加subgraph、function library、stage/domain capability、custom code permission、determinism和局部dirty propagation；大型图只重新编译affected closure，preview按node/slot输出共享中间artifact。

### P2-6：高级pipeline domain尚未进入统一identity

ray tracing pipeline/SBT、mesh/task shader、work graph、multi-view/device-group和bindless specialization需要未来扩展点。先用versioned descriptor/domain enum和opaque backend extension保留能力，不能提前在现有Mesh key继续加bool。

## 7. 目标架构与所有权

### 7.1 唯一artifact链

```text
AssetCatalogGeneration
  -> ShaderModuleGraphGeneration
  -> MaterialIRGeneration
  -> ShaderPermutationGeneration
  -> PipelineLayoutGeneration
  -> RhiPipelineGeneration
  -> PreparedMaterialGeneration
  -> RenderScene / Pass consumers
```

每个箭头是content-addressed immutable input/output，所有generation有owner、reverse edges、last-good、diagnostics和retirement fence。Editor、cook、prewarm、runtime不再各建一套source graph。

### 7.2 核心类型边界

| 类型 | 唯一职责 | 禁止内容 |
|---|---|---|
| `ShaderModuleArtifact` | parsed module、imports、source map、reflection、content ID | WGPU object、Editor widget、filesystem path lookup |
| `MaterialIrArtifact` | typed graph、properties/textures/options、surface outputs | raw node String求值、pipeline object |
| `ShaderPermutationArtifact` | canonical definitions/pass/geometry/shading lowering与backend-neutral IR/source | device handle、mutable cache state |
| `PipelineLayoutArtifact` | reflected bind groups/push constants/vertex-fragment IO ABI | consumer手写重复layout |
| `PipelineDescriptorArtifact` | target/render state/layout/shaders/specialization的完整PSO identity | material instance动态值 |
| `PipelineHandle` | stable ID与状态poll | direct wait、filesystem或compiler调用 |
| `PreparedMaterialHandle` | device-generation binding/uniform/texture/fallback集合 | asset load、graph compile、source parse |

### 7.3 编译服务与线程规则

- preprocess/parse/IR可在共享CPU task pool；backend object creation进入RHI affinity lane；I/O/compress进入I/O lane。
- 所有job按ArtifactId single-flight，waiter只订阅ticket；新generation supersede旧job。
- 主/render/UI线程只submit/poll，不finish/wait；shipping runtime默认不submit source compile。
- shutdown先拒绝新job，再deadline drain required publish，之后cancel optional；Drop无I/O和join。

### 7.4 fallback与错误策略

- cold required：bootstrap显式等待有deadline，超时为启动失败或共享error pipeline，不能无限等。
- hot reload：旧ABI兼容时last-good持续；ABI不兼容时整套material+PSO+binding原子切换。
- new optional：可defer或error material，必须有可见diagnostic和counter。
- depth/shadow/velocity：按pass contract定义安全fallback，不得复用不兼容color PSO。
- device loss：所有old device handles退役，新generation从portable/native disk artifact重建。

## 8. 依赖顺序与重构里程碑

| 顺序 | 里程碑 | 交付与删除门槛 | 依赖 |
|---:|---|---|---|
| M0 | current-source freeze | 重取242文件/横向callsite fingerprint；锁定artifact术语、现有行为和dirty overlap | 本报告 |
| M1 | canonical identity | versioned module/material/permutation/layout/pipeline IDs；完整source/compiler/backend hash；越界ID校验 | Runtime04、Render08 |
| M2 | module DAG与strict import | 单一import/token/reverse DAG、reflection、source map、严格kind/readiness；Raw WGSL不再猜surface | M1 |
| M3 | typed Material IR | 合并两套graph model，类型/拓扑/stage验证，真实operation/ZUI/template/preview；删除noop/parallel DTO | M1-M2、Editor09 |
| M4 | shared compile service | single-flight、budget/priority/cancel/deadline、typed ticket；删除私有PipelineAsyncCompiler和串行worker contract | Runtime11、M1-M3 |
| M5 | all-renderer PipelineService | 全47个直接render PSO及compute同类入口迁入RHI handle/state；删除Mesh局部PSO authority和cache None旁路 | 09A、M4 |
| M6 | atomic publish/last-good | dependency closure、ABI atomic swap、fallback、device generation与fence retirement | M2-M5、09A/09B |
| M7 | cook/prewarm/cache | exact artifact prewarm、variant stripping、shipping no-compile、显式driver cache flush；删除弱disk key | M1-M6 |
| M8 | prepared materials | generation-shared uniform/bind group/texture fallback；stable frame零create/upload | M3、M5-M7、09D |
| M9 | Editor/diagnostics | shared source map、node/backend diagnostics、coalesced preview、management compact index | M2-M8 |
| M10 | dynamic acceptance | cold/warm/reload/storm/device-loss/cross-platform/product capture与性能门槛 | 全部 |

M5不允许只把47个callsite包一层函数而保留各自HashMap和同步创建。完成判据是descriptor identity、queue、state、error、budget、retirement和driver cache都由唯一service拥有。M3不允许继续保留“简化v1 compiler”作为同名Material Editor产品路径；需要迁移资产后硬删除旧模型。

## 9. 量化验收矩阵

| 维度 | 场景 | 硬验收 |
|---|---|---|
| source DAG | 1/100/10k modules；depth 1/100/1k；stable/1% change/cycle | stable parse/hash/scan=0；changed work近reverse closure；cycle/source map确定 |
| variants | 1/1k/100k；shared source 0/50/99%；workers 1/2/8/64 | duplicate compile=0；queue/source/RSS有界；render/UI wait=0 |
| PSO inventory | Mesh/post/UI/particle/lighting/all builtin；cold/warm | direct product WGPU creation outsideRHI=0；warm create=0；状态全可查询 |
| cache | cold/warm/corrupt/schema/compiler/backend/driver/device change | stale hit=0；version change精确miss；caller/Drop I/O=0；总bytes有界 |
| reload | source/include/material parent/plugin/ABI变化；1/100/10k dependents | unaffected compile=0；last-good无闪烁；atomic swap不混代；旧GPU对象按fence退役 |
| fallback | queue full/panic/OOM/validation/worker loss/missing artifact | 不静默消失；不阻塞submission；typed reason、age、consumer可观测 |
| Material Graph | type mismatch/cycle/missing pin/subgraph/texture math/preview | diagnostics映射node/pin；同artifact供preview/cook/runtime；noop executor=0 |
| shipping | Windows DX12、Linux Vulkan、macOS Metal目标包 | required artifact完整；runtime source compile=0；backend capability有证据 |
| device | loss/recreate/adapter change/driver cache invalid | old handle不可复活；generation重建；无use-after-free；恢复有deadline |
| frame | 1/1k/100k draws，1/100/10k materials，300稳定帧 | stable material bind create/upload=0；runtime compile=0；p95/p99无compile hitch |

动态验收必须记录同一build fingerprint、artifact schema、compiler/backend版本和run ID：

- WPR/xperf：compiler/RHI/I/O worker、ReadyThread/wait、queue depth/age、filesystem、alloc/RSS；
- GPU timestamp与PIX/RenderDoc：PSO首次创建、warm reuse、pass/draw缺失、resource binding和device rebuild；
- CPU counters：source parse/hash bytes、single-flight waiter、cache hit/miss reason、module/PSO create、fallback和retire backlog；
- Editor trace：typing/reload storm、preview coalesce/cancel、UI thread parse/compile/I/O必须为0；
- 至少三次cold/warm/reload runs，报告p50/p95/p99、peak RSS、artifact bytes、compile count和能耗。

在这些证据前，只能说局部Shader模板和Mesh variant可运行，不能说已经具备Unreal级工程完成度，更不能声称Shader/PSO吞吐、hitch或能耗优于Unreal。

## 10. 删除清单

替代里程碑不完成下列同切片删除，就不算硬切换：

- `PipelineAsyncCompiler`私有线程、blocking finish API和无期限join；
- serial-only prewarm execution contract以及render/editor同步disk cache API；
- 弱`ShaderVariantCacheDiskKey`和只在metadata保存compiler版本的lookup；
- `RuntimePipelineCache::drop`持久化和backend外driver cache生命周期；
- Mesh多张局部PSO/module HashMap作为产品authority，以及非RHI direct create旁路；
- template每assembly builtin parse/hash/clone和11次全文replace专化；
- shading-model include对ready catalog的D*T*R扫描；
- plugin-local `ShaderGraphAsset`、noop render executor、dead `graphics/shader/shader_assets.rs` DTO；
- 无handler命令、缺失Material Editor ZUI/template和只做常量折叠的同名产品compiler；
- `SkipDraw`无consumer-visible diagnostic的静默fallback；
- source-shape测试中要求保留上述旧实现的断言。

## 11. 本轮状态

本篇 review 完成、implementation `in_progress`。2026-08-26 已实施部分 P0/P1 基础边界，包括 artifact/source identity、Surface contract/reflection、texture-presence key hard cut 和 Base typed admission；每项当前状态以对应 P0/P1 段落为准。没有在本切片运行 Cargo 或真实 GPU。工作树在 Material/Shader/Pipeline/Mesh cache/WGSL 范围仍有并发改动，2026-08-15 的 callsite/文件统计只作为原始审查快照；后续里程碑必须重新盘点当前 source。

下一审查单元是09D texture/mesh/material residency、streaming、upload和eviction；09C只把依赖generation与PreparedMaterial边界交给它，不提前把streaming实现混入Shader compiler owner。

## 12. 2026-08-24 P0复审：dielectric F0 / IOR 与能量守恒

### 12.1 状态与边界

状态：**source implementation staged; managed validation pending**。本节是对当前工作树 source 的重新审计，不是 GPU profile 结论；没有运行 Cargo、产品场景或真实 GPU capture，不能从静态代码推导帧时、功耗或“最优规模”。

审计范围是 Standard PBR 的 opaque direct light、environment IBL、glTF `KHR_materials_ior` 投影和其 render-path 选择。`KHR_materials_specular` 仍保持不支持：该扩展语义同时包含 `specularFactor`、线性 `specularColorFactor`、`specularTexture` 的 alpha factor 与 sRGB `specularColorTexture` 的 RGB color，且两张 texture 均有独立的 transform/sampler 投影。当前标准材质只有五个基础贴图槽，尚没有这两个 source/ownership/ABI；不能只接受 scalar 而声称支持 required extension。规范依据：[KHR_materials_ior](https://github.com/KhronosGroup/glTF/blob/main/extensions/2.0/Khronos/KHR_materials_ior/README.md)、[KHR_materials_specular](https://github.com/KhronosGroup/glTF/blob/main/extensions/2.0/Khronos/KHR_materials_specular/README.md)。

### 12.2 已确认的错误数据流

| 环节 | 当前事实 | 后果 |
|---|---|---|
| CPU feature | `StandardPbrMaterialFeatures` 保存并归一化 `ior`，默认值为 1.5。 | glTF 投影未丢失原始属性。 |
| material uniform | 2026-08-24 审计快照中 `gpu_material_uniform_resource.rs` 把 IOR 写入 `data10.w`（f32 #43），当时 `data12.yzw`（#49-51）未使用，整个 uniform 固定 256 B；该空闲布局已由 §39.7 取代。 | 当时存在无 ABI 增长的 CPU 预计算 F0 通道；当前 F0/clearcoat 布局见 §39.7。 |
| surface assembly | `material_surface.rs` 以 `ZR_FEATURE_PBR_TRANSMISSION` 保护 `surface.ior`。 | 非 transmission 的 `KHR_materials_ior` 没有进入 opaque BRDF。 |
| direct light | `zr_shading_standard_pbr.wgsl` 与 basic 版本都用 `mix(vec3(0.04), base_color, metallic)`，diffuse 仅乘 `(1 - metallic)`。 | 非默认 dielectric F0 不生效；Fresnel 反射与 diffuse 同时计能。 |
| environment IBL | `zr_environment_core.wgsl` 同样固定 0.04、diffuse 只按 metallic 衰减。 | direct/indirect 结果不一致且均不满足同一剩余能量模型。 |
| render route | `PipelineKey::requires_forward_path` 和 `AdvancedPbrMaterialFrameUsage` 只覆盖 clearcoat/anisotropy/transmission。 | 非默认 IOR 会错误进入无法表达其 F0 的 deferred GBuffer。 |

### 12.3 目标设计及不变量

1. 在 CPU 唯一计算 `dielectric_f0 = ((max(ior, 1)-1)/(max(ior, 1)+1))^2`。2026-08-24 首版写入 `data12.yzw`；§39.7 已将三个恒等通道收敛为 `data12.y` 单标量并由 WGSL splat，仍不扩展 256 B ABI、不新增 bind group、也不在 fragment 每像素做除法和平方。
2. `ZrSurfaceOutput` 持有该 `vec3`。Standard PBR 的最终 F0 统一为 `mix(dielectric_f0, base_color, metallic)`；direct 与 IBL 调用同一公共 helper。这里原定的“diffuse 使用对应视角 Fresnel 剩余能量”已被 2026-08-26 的完整参考链复审否决，并由 12.9 的统一材质分解取代。Blinn-Phong、unlit 与已独立的 clearcoat/transmission lobe 不借此改写。
3. 只有非默认 dielectric F0 的 opaque Standard PBR 进入既有 late forward opaque pass。`PipelineKey` 增加 routing-only 标志，并由 `AdvancedPbrMaterialFrameUsage` 记录；该标志**不**进入 `ShaderFeatureBits`，标准 forward shader 对默认/非默认 IOR 共用同一 permutation。默认 IOR 继续使用 deferred，避免 GBuffer 扩展和全场景带宽回归。
4. environment 的 generic API 继续表达默认 metallic-roughness 语义；新增显式 F0 入口只给 Standard PBR forward 使用。不是 compatibility shim，而是区分 generic material caller 与具有 dielectric override 的 surface caller，避免修改所有 environment consumers 的 ABI。
5. `KHR_materials_specular` 保持 preflight 拒绝；不得因新增 IOR 路径把它加入 supported-required 集合。

### 12.4 实施顺序

| 次序 | 交付 | 性能约束 |
|---:|---|---|
| S1 | `StandardPbrMaterialFeatures` 的有限、可测试 F0 helper；uniform #49-51 写入及 layout 测试。 | 常数大小/绑定数不变。 |
| S2 | surface F0 字段和 template 赋值；默认 surface/fallback 赋默认 0.04。 | 无 shader feature bit、无 PSO permutation 维度。 |
| S3 | direct 与 environment 共享 F0 owner；原“按各路径 Fresnel 计算剩余 diffuse energy”已由 12.9 覆盖，改为 source-independent metallic diffuse decomposition。 | F0 只服务 specular；MVP diffuse 不增加方向反照率 LUT、采样或新的逐光 Fresnel 工作。 |
| S4 | IOR routing 到 advanced opaque、default IOR 留 deferred；覆盖 feature/frame/pass/pipeline-key 单元测试。 | 无 scene-color copy；默认材质 draw/pass 不增加。 |
| S5 | 外部 glTF non-default-IOR product fixture、PNG/RDC/timestamp sidecar；由协调器管理当前 source 运行。 | 没有当前 source capture 前不宣称正确或快。 |

### 12.5 profile 设计与待采集数据

在 S1-S4 静态验证通过后，必须使用同一 build fingerprint 的 coordinator-managed Windows run 比较默认 IOR 与 1.0/1.33/1.5/2.5 IOR，分别在 1080p 与 4K、100 与 1,000 不透明材质、300 warm frames 采集：

- RenderDoc（`D:\Tools\renderdoc`）：advanced opaque pass 是否只在非默认 IOR 出现、默认路径仍走 GBuffer、scene-color copy=0、draw/pass/resource binding 与像素结果；
- GPU timestamps：GBuffer、advanced opaque、environment IBL 分项 p50/p95/p99；
- WPR/xperf：render/worker CPU、等待、PSO create、allocation、RSS 与 GPU submission；
- 若平台计量权限允许，记录同一 run 的 GPU 能耗/平均功耗；否则报告 `not collected`，不得以耗时替代功耗；
- 每项至少 cold/warm/reload 三次，写入 raw run ID、adapter/driver、shader artifact schema 和 capture 路径。

通过条件不是与 Unreal 的绝对毫秒相等，而是默认 IOR 的 pass/draw/ABI/PSO create 无回归，非默认 IOR 的增量只限既有 forward opaque 工作量，且 capture 中不再存在 deferred 对非默认 F0 的错误表达。实际数据及是否消除瓶颈必须等当前 source capture 后补入本节。

### 12.6 2026-08-24 暂存实施记录

- S1/S2 已暂存：`StandardPbrMaterialFeatures` 以默认 IOR 的精确常量和有限 IOR 公式生成 F0，`data12.yzw` 写入三通道 F0；256 B material uniform、binding 数和 texture slot 数不变。默认 surface/fallback 保持 `0.04`。
- S3 历史实现已暂存：新 `zr_pbr_common.wgsl` 是 Schlick、metallic F0 与当时 remaining-diffuse-energy 的唯一 WGSL owner；环境、forward、deferred 和 fallback 依赖同一公式。后续完整 Unreal/Bevy 复审发现该公式按 direct/environment/baked source 分别选择 `VoH`/`NoV`/常量，已由 12.9 的 source-independent metallic decomposition 覆盖。GGX owner 已恢复为只返回其真实消费者 specular，错误设计新增的 result struct、基础/各向异性 terms wrapper 和无消费者 Fresnel 返回值均删除；`KHR_materials_specular` 仍未列为 supported。
- S4 已暂存：非默认 IOR 记录到 `AdvancedPbrMaterialFrameUsage` 和 late forward opaque route，默认 IOR 保留 deferred、scene-color copy 不增加。`PipelineKey::pipeline_variant_identity` 删除 route-only bit，mesh variant registry 复用既有 PSO identity，避免 route 本身生成 duplicate PSO。
- S5 未开始：当前 coordinator-managed Windows source 尚未完成 compile/WGPU/PNG/RDC/timestamp/WPR 运行。下次受管 run 必须先执行 source/contract tests，再产出本节 12.5 的 raw evidence；在此之前没有量化帧时、内存、功耗或截图验收结论。

### 12.7 2026-08-24 静态复审补记

- 复审了 material asset/glTF projection、frame extract、draw phase、mesh variant registry 和全部 `data12` 消费点：非默认 IOR 保留在 draw routing 以进入 advanced opaque，但仅在 variant registry 中移除该 routing-only bit，PSO identity 与 shader permutation 不会因 IOR 数值分裂；environment-only Base profile 对该材质强制保留 generic forward receiver。新增 source contract 覆盖该排除条件。
- `zr_pbr_common.wgsl` 被置于 fallback 前，fallback 的 Fresnel/F0 适配函数直接委派给公共 owner；diffuse energy 则由所有路径复用既有 `zr_surface_metallic_diffuse_energy_scale`，不在 common include 创建第二 owner。相关 include 集合的静态函数名冲突扫描结果为 0。`data12.yzw` 在改动前无消费，`data12.x` 的 normal scale 语义未改。
- 这不是 WGPU/WGSL 编译或图像验证。现有 `docs/tests/runtime/shader` 截图/RDC 均属于此前 source 或其它 IBL 里程碑，不能作为本次 non-default-IOR patch 的验收证据；S5 继续等待 coordinator-managed 当前 source run。
- 现有 `zircon_shader_pbr_viewer` 刻意固定为 environment-only Base profile，项目材质是无 direct-light/no-receiver 的镜面球；其自身 review 已限制该工具不能证明完整 material/deferred matrix。非默认 IOR 要求 generic forward receiver，不能通过给该 viewer 临时追加 `--ior` 并复用其 profile 来验收。S5 应由独立的 generic-forward scene corpus/evidence schema 覆盖，不改变既有 environment-only baseline 的 cold/warm 指标定义。
- 2026-08-24 的独立只读复审再次检查 material route、PSO identity、common F0 helper、deferred/fallback/manual WGSL assembly 与本节的 `KHR_materials_specular` 边界，结论为 `Critical 0 / Important 0 / Minor 0`。审查未运行 Cargo、WGPU、产品导出或 RenderDoc；这只提高当前 source 结构可信度，S5 的 current-source Windows/PNG/timestamp/WPR/RDC 门禁不变。

### 12.8 2026-08-24 `KHR_materials_specular` 架构复审与性能门

状态：**owner map retained；ABI design reopened；保持 unsupported；未开始实现**。这不是 IOR 修复的遗漏，也不能用“先导入 factor、以后补 texture”的半实现关闭。glTF 的 `KHR_materials_specular` 同时规定 `specularFactor`、线性 `specularColorFactor`、`specularTexture` 的 alpha strength、sRGB `specularColorTexture` 的 RGB color，以及每张 texture 各自的 `texCoord` 和 `KHR_texture_transform`；任何一项丢失都会令同一资产在两条导入链和两个 shader 路径产生不同 F0。

2026-08-28 current-source amendment：§39.7 的 clearcoat packing 已占用本节原设计依赖的 `data12.zw` 与 `data15.z`，并把 F0 收敛为 `data12.y` scalar；因此下面的 2026-08-24 inline-capacity 数字和“constant-only 直接复用 `data12.yzw`”候选已失效。实现状态仍为 unsupported，但“设计完成”降级为 **owner map retained；ABI design reopened**。任何实现必须先从当前 256 B/288 B layout 重新完成 R1/R2，不得照搬旧槽位。

#### 当前结构复核

| 层 | 当前 source 事实 | 对完整 specular 的约束 |
|---|---|---|
| asset/import | `project_gltf_material_extensions` 是内建 ingest 与 stable plugin 的唯一 extension 投影 owner；目前对该扩展给显式 diagnostic。 | factor、color、两张 texture URI、每张 UV channel/transform 和 required-extension policy 必须只在此处解释。 |
| material contract | `StandardPbrMaterialFeatures` 已拥有 IOR/clearcoat/anisotropy/transmission；`MaterialAsset`/`StandardMaterialDescriptor` 已拥有五个基础 texture transform 和独立 clearcoat-normal transform/UV/scale。 | 新字段必须进入 feature owner 和 descriptor，不得塞入 `property_values` 后让 renderer 自行猜 slot。 |
| uniform ABI | `StandardMaterialPropertyUniform` 固定 `data0..data15`（256 B）；当前 IOR-derived dielectric F0 是 `data12.y` scalar，clearcoat normal 占用 `data1.w`、`data7.yzw`、`data12.zw`、`data15.z`，五个基础贴图与 clearcoat normal 的 UV selector 压入 `data7.x` 精确 6-bit mask，仅 `data15.w` 保留。 | constant factor/color 也不能假定复用旧 `data12.yzw`；两张 texture 的 transform + rotation + UV channel 共需 14 个标量，当前 256 B 只剩 1 个标量，ABI 必须重审。 |
| binding / bindless | per-material Standard PBR group 目前到 binding 12；bindless payload 为 16 个 property rows + 8 texture indices（6 个已用、2 个 reserve），共 288 B。 | 两个 reserve index 恰可保存 factor/color texture；它们的 transform metadata 仍需有同源 storage，不能只让 non-bindless path 正确。 |
| shading / route | `zr_pbr_common.wgsl` 统一消费 surface dielectric F0；deferred GBuffer 不保存非默认 F0；non-default IOR 已路由 late forward opaque。 | non-default specular F0 同样必须进入 late forward opaque，不得把错误 F0 写进 deferred；两张 texture 的 presence 分别决定 shader binding/source。 |

#### 设计结论

1. `F0_dielectric = dielectric_f0(ior) * clamp(specularFactor, 0, 1) * clamp(specularColorFactor, 0, 1)`；有 `specularTexture` 时在 fragment 中再乘其 alpha，有 `specularColorTexture` 时再乘其 sRGB-decoded RGB。metallic 部分仍由 `base_color` 控制，最终 F0 继续由公共 `mix(dielectric_f0, base_color, metallic)` owner 生成。该公式和无额外像素 sample 的 constant-only 目标保留，但 current-source storage 未定：IOR 现为单标量，不能再复用已失效的 `data12.yzw` 假设。
2. default factor/color、无 texture 时保持默认 deferred 和现有 PSO identity。任何实际改变 dielectric F0 的 opaque 材质只增加 routing-only late-forward 标志，和 IOR 一样从 pipeline variant identity 剥离；它不得生成按 factor/color 值分裂的 PSO。
3. `specularTexture` 与 `specularColorTexture` 需要独立的 `PBR_SPECULAR_FACTOR_TEXTURE` / `PBR_SPECULAR_COLOR_TEXTURE` feature，因为它们可单独存在，且各自增加 sampler binding 和一次 fragment sample。四种 presence 组合必须保持 factor/color/UV transform 为动态 material data，并在 generic forward 与 bindless 路径有相同含义；不得为了减少 variant 数而让单贴图材质无条件采样另一个 fallback texture。
4. 不接受未经 profile 就把两组 transform 直接扩进标准 256 B uniform。current-source 已使用 63/64 个标量；把 scalar F0 替换为 RGB 净增 2 个标量，再加入两张 texture 的 transform、rotation 和 UV channel 共 14 个标量，合计需要 79 个标量并按 vec4 对齐到 80 个：Standard PBR uniform 将从 256 B 到 320 B（+25%），bindless payload 从 288 B 到 352 B（+22.22%），即使绝大多数材质从不使用该 extension。当前没有 profile 证明这部分常驻 GPU 内存、upload 和 cache-locality 代价可以接受。
5. feature-only auxiliary material block 只保留为待测方向，不是已批准设计。R2 必须重新验证最小 metadata 大小、各 adapter 的 material bind-group limit、layout identity、bindless sidecar ownership、fallback texture 和 upload locality；constant-only storage 也必须与当前 scalar F0/clearcoat packing 一起重做。auxiliary block 不能成为 viewer-only 旁路、另一个 material ABI，或默认 288 B bindless row 的隐式扩张。

#### 实施和测量顺序

| 次序 | 前置/交付 | 默认路径性能不变量 |
|---:|---|---|
| R1 | 以 fixture 记录 factor、color、两张 texture、各自 `texCoord`/`KHR_texture_transform`、alpha linear/sRGB RGB、以及 transmission 组合的语义；补齐 source owner map。 | 不改变 production shader/ABI。 |
| R2 | 设计并 review feature-only auxiliary block、per-material/bindless layout 与 fallback policy；枚举 four presence layouts 与 PSO identity。 | default material uniform=256 B、bindless payload=288 B、binding count 和 shader sample count 不变。 |
| R3 | 仅在 R2 通过后实现 import -> descriptor -> runtime -> forward surface 的单向投影，并将 non-default F0 路由到 existing late forward opaque。 | constant-only non-default values不增加 PSO；每张 texture presence各自增加一个静态 feature 维度。 |
| R4 | 由协调器执行 current-source Windows Cargo/WGPU、generic-forward external glTF corpus、PNG/GPU timestamp/RenderDoc 和 WPR。 | 验证默认材质仍为 deferred、无 auxiliary bind group、无新 PSO/create/upload；texture 变体的额外 bind/sample 只出现在实际使用资产。 |

R4 必须在同一 adapter/driver/build fingerprint 下采集 100、1,000、10,000 个默认材质和 0%、1%、100% factor-texture、color-texture、two-texture 材质密度，分别记录 material uniform/bindless bytes、auxiliary metadata bytes、bind group/pipeline creation、draw/pass 数、GPU timestamp p50/p95/p99、CPU allocation/RSS，以及能耗计量（不可用则记为 `not collected`）。RenderDoc 使用 `D:\\Tools\\renderdoc` 验证每个 texture variant 的实际 binding/sample 和默认 deferred 路径零回归。没有这些原始数据，不宣称该扩展的带宽、功耗或整体吞吐表现；在 R1-R4 完成前，stable plugin 的 `extensionsRequired` 必须继续拒绝 `KHR_materials_specular`。

### 12.9 2026-08-26 diffuse energy decomposition 结构复审

状态：**设计更正与 source implementation staged；managed GPU/profile pending**。本节在修改公式前完整复核 Standard-PBR 的 forward/basic、fallback、deferred、environment-only、lightmap 与 environment-core 调用链，并对照 Unreal Default Lit/可选 energy-conservation 路径和 Bevy PBR。它只建立结构与公式边界；没有运行 Cargo、WGPU、产品场景、RenderDoc、GPU timing 或功耗采集。

| 路径 | diffuse decomposition | 结构结论 |
|---|---|---|
| 修复前 Zircon direct | `(1 - F(VoH)) * (1 - metallic)` | 同一材质随每个光源的 half vector 改变漫反射反照率；`VoH -> 0` 时 diffuse 趋近零。 |
| 修复前 Zircon environment | `(1 - F(NoV)) * (1 - metallic)` | 同一材质又随观察角改变 diffuse IBL，且与 direct 的角度变量不同。 |
| 修复前 Zircon baked lightmap | `(1 - F0) * (1 - metallic)` | baked、direct、environment 三类 source 对相同材质使用三个不同 albedo。 |
| Unreal Default Lit 默认路径 | `DiffuseColor = BaseColor - BaseColor * Metallic`，Lambert 消费该 source-independent 值 | 默认路径不额外把 per-light Schlick Fresnel 乘入 diffuse。 |
| Unreal 可选 energy conservation | 由 `Roughness/NoV/F0` directional-albedo LUT/analytic fit 得到能量项，再做 energy preservation | 这是有独立资源/拟合、模式和测量成本的高级 multiple-scattering 模型，不等价于 `1-F(VoH)`。 |
| Bevy/Filament 系 | `base_color * (1 - metallic)`，再叠加 transmission 因子 | direct、irradiance、lightmap 复用同一个 material diffuse color。 |

MVP 决策如下：

1. 将 `zr_surface_metallic_diffuse_energy_scale` 的唯一 owner 从通用 `zr_surface_types.wgsl` 硬迁移到 `zr_pbr_common.wgsl`，表达 `1 - clamp(metallic, 0, 1)`；不保留旧 owner 或兼容别名，并删除这轮新增的重复 `zr_pbr_diffuse_energy_scale`。该 helper 不得再接收 Fresnel、`VoH`、`NoV` 或 light-source 类型，独立 fallback/deferred 产品也必须使用同一 PBR owner。
2. Standard-PBR base color 仍在物理消费边界限制到 `[0,1]`。direct、ambient、environment 与 baked lightmap 必须复用同一 source-independent decomposition；Unlit、Blinn-Phong 和 custom model 仍保留原始 base-color 语义。
3. dielectric/metallic F0 只服务 specular BRDF、split-sum GF 与其遮蔽；non-default IOR 的 uniform、forward routing、PSO identity 和 `KHR_materials_specular` unsupported 边界不改变。
4. Unreal directional-albedo LUT/analytic fit、multiple scattering 和能量补偿只作为高级候选。在引入纹理 binding、LUT sample、analytic `pow` 或额外 permutation 前，必须先用当前 source 收集 RenderDoc、GPU timestamp、WPR 与功耗数据，并单独验证图像收益和默认材质成本。
5. 本次更正不新增 binding、material/GBuffer ABI、feature bit、shader permutation、PSO identity 或 bake recipe；也不据静态源码声称 GPU 性能收益。

公式锚点使用 `F0=0.04, metallic=0`。旧 baked scale 固定为 `0.96`；旧 environment scale 在 `NoV=0.5` 为 `0.93`、在 `NoV=0.1` 约为 `0.39313`，后者仅为 baked scale 的 `40.95%`；旧 direct scale 在 `VoH=0` 为 `0`。若只在均匀 cosine 参数域积分，旧 Schlick remaining 的均值为 `0.8`，比 baked 的 `0.96` 低 `16.67%`。这些是解析公式差异，不是场景统计、帧缓冲证据或性能数据。新 MVP scale 对三类 source 均为 `1.0`。

#### Source implementation record

- 唯一 scalar owner 已从通用 surface-types 硬迁移到 `zr_pbr_common.wgsl`，没有旧别名；主 forward/basic、environment-only、environment-core、两个 lightmap template，以及独立 fallback、deferred lighting 和 deferred environment-only 产品的 15 个 production 消费点全部复用该 owner。错误设计新增的 duplicate helper、`ZrPbrGgxTerms` 及两套 terms wrapper 已删除；Fresnel 只留在 isotropic/anisotropic GGX specular、transmission 和 clearcoat 消费点。
- direct light 的 `direct_diffuse_brdf` 恢复为 light-grid 入口的一次性每像素准备值，并穿过内部函数参数；Blinn-Phong 仍只接收原始 `diffuse_color`。对 N 个实际遍历光源，源码表达的 metallic clamp/subtract、RGB scale 与 `/PI` 从 N 次降为 1 次，即减少 N-1 次该计算。编译器可能内联或自行提升，因此这不是最终指令数、GPU 时间或功耗结论。
- 静态契约先后记录主 include/source closure `6/6 red -> 6/6 green`、独立 fallback/deferred 产品 closure `5/5 red -> 5/5 green`、无消费者 GGX result 清理 `5/5 red -> 5/5 green`、逐光不变量提升 `4/4 red -> 4/4 green`，并以 `8/8` 签名/legacy-model 检查防止准备值误入 Blinn owner；最终 WGSL 函数 arity 检查覆盖 fallback/deferred 的 11 组关键 owner/调用链。公共 scalar owner 另有 `3/3` 装配契约，锁定唯一声明、精确 `1-clamp(metallic)` 公式和无 Fresnel 输入。限定 Rust 格式与 diff integrity 通过；Cargo、Naga/WGPU、PNG、RenderDoc、GPU timing、WPR 和功耗仍等待受管运行。

## 13. 2026-08-26 P0-5 原始着色器资产分类与 fail-closed readiness 复审

状态：**结构设计与 source implementation staged；managed validation pending**。本节先于实现记录所有权和性能边界，并在实施后补记静态闭包；没有运行 Cargo、WGPU、RenderDoc 或产品截图，也不把静态审查当作 P0-5 的最终验收。

### 13.1 已确认的结构错误

内建 WGSL/GLSL/SPIR-V importer、stable shader importer plugin 和 WGSL-only plugin 都先用 Naga parse/validate，再把任何合法 module 无条件写成 `ShaderAssetKind::Surface`。这些资产没有 `.zshader v2` 的 `shading_model`、material schema、options、texture slots 或 material ABI，却会通过当前 `ShaderReadinessReport::is_ready`：该函数只检查 runtime WGSL、entry/definition diagnostics 和已有 validation diagnostics，不检查资产 kind 的契约。因此 compute-only、普通 vertex/fragment program 和零 entry helper module 都可能以“ready material surface”进入 catalog；旧 artifact cache 又会持久化这个错误种类。

这不是入口点字符串或单个 importer 的细节问题，而是 source module、material domain 与 executable pipeline 三种身份被压成一个枚举值。只改新 importer 不能修复旧 cache；只要求 entry point 非空又会错误拒绝合法 helper module 和以 engine template 组装的 Surface。

### 13.2 参考实现结论

- Unreal 的 `FShader` 由 `FShaderType` 表达显式 meta type/frequency，而材质路径使用派生的 `FMaterialShaderType`，`FMaterialShader::ShaderMetaType` 明确绑定该类型；材质身份不由源码是否能编译推断。
- Bevy 的 `Shader` 资产保存 raw source/imports/defs，render/compute pipeline descriptor 分别拥有 shader reference 与 entry point；通用 source asset 本身不冒充 material surface。
- Zircon 现有 `Include` 也不能承担通用 module：`.zshader include` 会拒绝 entry point 和 `@group` binding，并保留 import ABI/命名空间约束。把任意原始 WGSL 改标成 Include 会把一个错误分类替换为另一个。

### 13.3 MVP 硬边界

1. `ShaderAssetKind` 新增序列化 token 为 `module` 的 `Module`。它表示已解析/验证的通用源码 artifact，可包含零个或多个合法 entry point，但不参与 material variants，也不是 `.zshader v2` 可声明的产品 domain。
2. 三条 raw importer 路径统一产出 `Module`；显式 `.zshader v2` 继续只接受 `Surface`、`Include`、`Compute`、`Fullscreen`。不得按 entry point stage 猜测 Compute/Fullscreen，因为同一 module 可有多个 stage，pipeline/domain 选择属于显式 descriptor。
3. readiness 增加结构化 `kind` 与 `kind_diagnostic`，不能把 kind 错误伪装为 WGSL capture diagnostic。`is_ready` 必须同时要求 kind contract 通过；material readiness 和 pipeline compile diagnostics 直接消费该字段。
4. kind contract：`Module` 不额外要求 entry；`Surface` 要求非空 `shading_model`；`Include` 不得拥有 entry；`Compute` 至少一个 entry 且全部为 compute；`Fullscreen` 至少一个 entry 且全部为 fragment。stage 判断复用同一份 canonical entry readiness，不重复解析字符串。
5. 内建 `builtin://shader/pbr.wgsl` 是默认材质的显式 Surface，不迁为 Module；补齐 `standard_pbr` shading-model 契约。旧 raw cache 因 `Surface + shading_model=None` fail-closed，重新导入后成为 Module；cache 的缺省 kind 仍保留 Surface，以免静默把老的声明式材质改成通用 module。
6. 本切片不实现 reflection-derived bind layout、module DAG、pipeline identity 或 native shader binary；它只关闭 P0-5 的错误身份和假就绪入口，为 M2 的严格 module graph 建立不可绕过的基础类型。

### 13.4 性能约束与验证门

- importer 只改变枚举赋值，不增加 Naga parse/validate、source clone、binding、permutation 或 GPU 工作。
- readiness 构造 entry rows 后，以同一 canonical rows 做一次 O(E) kind 检查；Surface/Module 为 O(1)，成功路径不新增 String allocation，只有失败时构造一条 diagnostic。
- 静态门必须覆盖：三个 raw importer 均为 Module；Module 不进入 material variants；旧 Surface cache shape 不再 ready；合法 Surface/Include/Compute/Fullscreen 通过；错误 stage/空 executable entry fail-closed；`.zshader kind="module"` 仍被拒绝。
- 受管动态门在 current-source Windows run 中记录 1/100/10k raw modules 的 import/readiness cold/warm p50/p95/p99、Naga parse count、diagnostic allocation、catalog ready/not-ready 数和 peak RSS。通过条件是每个 module 仍仅一次 Naga validation、warm cache 不重复 import、错误 Surface ready count 归零；GPU timing/功耗不适用于纯 catalog 切片，若没有 GPU submission 必须明确记录为 `not applicable`，不得伪造收益。

### 13.5 Source implementation record

- `ShaderAssetKind::Module` 已成为 raw WGSL/GLSL/SPIR-V 的统一 importer 输出；stable shader importer plugin、WGSL-only plugin 与 runtime 内建 importer 不再把已通过 Naga 的通用 module 冒充 Surface。`.zshader v2` 仍拒绝 `kind = "module"`，内建 PBR 则保持显式 Surface 并补齐 `standard_pbr`。
- readiness report/summary/record-set 已发布 `kind`、`kind_diagnostic` 与对应计数；Surface、Include、Compute、Fullscreen 的 kind contract fail-closed，material readiness、material validation 和 pipeline compile diagnostics 都消费结构化错误。material consumer 额外要求 Surface，因此一个自身 ready 的通用 Module 也不能进入材质域。
- material artifact regeneration 现在以 `participates_in_material_variants()` 为边界：非 Surface cache hydration 会清空陈旧 property layout、option table 与 generated material WGSL，避免空 Module 被隐式生成 16 B material uniform。cache 继续序列化新 Module 身份，而缺失 kind 的旧 wire 仍默认 Surface；旧 raw `Surface + shading_model=None` 会在 readiness 处失败并等待重新导入。
- 静态源码闭包 `16/16` 通过：枚举/token、四个 raw 赋值、`.zshader` 拒绝、kind readiness、canonical entry 复用、Surface shading model、Compute/Fullscreen stage、builtin PBR、material Surface 门、两条结构化 diagnostic 消费链、kind/domain regressions 和非 Surface cache material-artifact 门均存在。限定 production Rust `rustfmt --check --config skip_children=true` 与 scoped `git diff --check` 通过；artifact-cache Module 往返回归已实施但未运行 Cargo test。动态 1/100/10k 数据、peak RSS 与 current-source Windows 编译/测试仍由协调器托管，因此不能关闭 P0-5 或作性能收益声明。

### 13.6 离线预热域边界补漏

继续复审 `zircon_shader_prewarm` 后确认，离线工具仍绕过上述硬切：`shader_source_from_wgsl(...)` 把 standalone 或 single-unit `.zmeta` WGSL 重新标成 Surface，并直接附加完整材质 pass；material join 也只按 label/resource id 查找，不核对 source kind。于是运行时明确拒绝的 generic Module 会在 stage 阶段生成看似合法的材质变体与缓存条目，P0-5 在离线链路上并未闭环。

当前 source 已将裸 WGSL 固定为 `Module + empty pass set`，并在 source 预热入口和 material join 分别以 `participates_in_material_variants()` 建立双重门禁。独立 Module 仍被 inventory 读取、空源校验并可导出 resource revision，但不会写入材质 prewarm source table 或 variant；`.zmaterial` 若解析到 Module，则返回包含 material path、shader label 与 actual kind 的 `MaterialShaderKindMismatch`，不会静默跳过、回退 builtin 或开始 template expansion。几何维度和 registry-revision 测试夹具已改用显式 `kind = "surface"` 描述符，raw regression 则锁定编辑前后均为零材质请求。

旧算法对每个误分类 raw module 形成 `6 x Q x G` 个材质请求，并在后续路径触发同规模 template assembly、hash/validation/cache 工作；新边界保留 O(source bytes) 的一次 inventory/read/hash 成本，但材质请求规模严格为 0，无需等 profile 才能判断这批工作在语义上不应存在。这个量化是算法规模结论，不是实测耗时或功耗收益。离线 hard-cut 静态契约 `8/8` 已通过，限定 `rustfmt` 与 scoped diff integrity 通过；Cargo、Windows CLI、WGPU、1/100/10k timing/RSS、PNG、RenderDoc 与功耗仍由协调器托管，P0-5 和现有里程碑状态均不关闭。

## 14. 2026-08-26 P0-2 生成 WGSL 源缓存身份复审

状态：**generated-WGSL source-cache sub-boundary implemented; managed validation and PSO identity work pending**。本节只修复磁盘源码 artifact 的身份与完整性，不把它描述为编译后二进制缓存、driver cache 或运行时 PSO cache；没有运行 Cargo、Naga/WGPU、产品场景、RenderDoc、WPR 或功耗采集。

### 14.1 结构结论

当前 Mesh runtime 在磁盘 lookup 前已经完成完整 WGSL assembly、内容哈希并排队 Naga source validation；磁盘 hit 只返回压缩保存的同一 WGSL，随后仍需在 renderer device 上创建 WGPU shader module 和 render pipeline。因此该层不能跳过 assembly、Naga、WGPU module 或 PSO 工作，warm hit 还可能增加 metadata read、压缩 payload read、解压与校验成本。历史 M5 结论“shader prewarm 不是 runtime pipeline cache”保持有效；本切片是 correctness hard cut，不是启动耗时优化。

旧实现的磁盘 key 只组合 `ShaderVariantKey` 与 include hashes，template revision、Naga/WGPU version 只写 metadata 而不参与 lookup identity；runtime 必须在 hit 后全文比较 WGSL 才能发现错误复用。`ShaderVariantPrewarmSourceId` 又把 `source_label` 混入 artifact ID，使内容相同但路径/标签不同的 source 无法共享，并让 provenance 变化错误失效内容缓存。

Unreal 的可迁移原则仍是以完整 compiler input hash 建立唯一作业/cache identity并对同 key single-flight；Bevy 的可迁移原则仍是 source/import dependency 与受影响 pipeline 的显式反向失效。Zircon 本轮只采用“完整内容契约决定身份、provenance 不决定 artifact”的基础部分，不提前复制 UE job protocol，也不把 Bevy pipeline state 旁路成源码文件缓存。

### 14.2 已实施硬边界

1. `ShaderVariantPrewarmSourceId` 现在只由 WGSL BLAKE3、按序 include content hashes、template revision、Naga version 和 WGPU version 计算；`source_label` 仅作 provenance。manifest schema 从 v2 推进到 v3，v2 在 schema gate 显式拒绝；动态 builtin manifest 对内容相同的 source ID 只注册一次。
2. source record 持久化 `source_hash`，完整性校验同时重算 WGSL payload hash 和 canonical source ID。缺少该字段的旧 JSON 可被 serde 读取以到达版本门，但不能通过 canonical integrity。
3. 磁盘缓存 schema 从 v1 推进到 v2，目录与 key 自然隔离旧 entry。key 由完整 variant canonical string 与上述 source ID 组成；metadata 同时绑定 source ID、source hash、template/Naga/WGPU version，读取逐字段校验。
4. 写入前和解压读取后都重算 WGSL hash；不一致返回结构化 `SourceHashMismatch`。在完整 key 与 payload hash 成立后，runtime 不再做一次全文 WGSL equality compare。
5. 不新增 per-draw、per-frame、binding、permutation 或 GPU 工作。新增常驻成本是每个唯一 prewarm source 的一个 64 字符哈希字符串，已进入 `resident_bytes`；key clone、metadata I/O 与 payload rehash 只位于 source resolution/prewarm 边界。

### 14.3 性能分析门

在决定保留、禁用或替换这层压缩 WGSL 缓存前，协调器必须在同一 current-source build fingerprint、adapter/driver 和 E 盘工作/缓存根下采集 1、100、10,000 variants 的 cold/warm/reload 三轮数据：

- CPU p50/p95/p99：template assembly、source hash、disk metadata read、compressed bytes read、decompress、payload rehash、Naga parse/validate、WGPU module create、PSO create；
- 计数/规模：source/variant 去重率、hit/miss/error reason、实际读写字节、module/PSO create 数、single-flight waiter、peak RSS 与 cache resident bytes；
- 产品门：默认/IOR fixture 的 shader/PSO identity、Ready 时间、GPU timestamp、PNG 与 `D:\Tools\renderdoc` capture/replay；有权限时记录同机功耗，否则明确 `not collected`。

只有 warm source-cache I/O 明确小于它避免的工作，且总体 Ready/CPU/RSS 有可重复收益时才能保留“性能优化”结论。若 assembly/Naga/module/PSO 仍占主导或 warm hit 净增加耗时，应在 M2/M4 的 compiled artifact/shared compile service 设计中删除或降级此 WGSL disk lookup，而不是继续微调压缩级别。算法规模目标是每个唯一 source contract 一次 validation、每个完整 pipeline identity 一次创建、variant 请求 O(1) lookup；当前 source 只证明身份基础，尚未证明这些规模目标或功耗接近参考引擎。

### 14.4 Source record

TDD 静态门先记录 `5/5 red`：无 cache-contract constructor、label 仍参与身份、disk key 不消费 source ID、metadata 不绑定 source ID、runtime 仍全文比较 WGSL。实施后 `17/17` 契约检查通过，覆盖 schema、source/payload hash、版本字段、worker/runtime 完整输入、动态 source 去重及其回归、legacy manifest fail-closed；限定 Rust `rustfmt --check` 与 scoped `git diff --check` 通过。Cargo/WGPU/PNG/RDC/profile 均未运行，所以 P0-2 的 target/device/pipeline identity、native compiled artifact 与共享 compile service 仍是后续工作，本节不能关闭完整 P0-2。

## 15. 2026-08-26 P0-4 传递 shader 代际与运行时 PSO 身份复审

状态：**runtime-generation source implemented; managed validation and fenced retirement pending**。本节先冻结算法和持久化边界，再记录 source implementation；不以静态审查代替 Cargo/WGPU、热重载产品场景、RenderDoc、GPU timing 或功耗验收。

### 15.1 当前结构性缺口

`PreparedShader` 只保存根资源 revision。根 revision 命中时，`ensure_shader_source` 仍递归访问完整 import/registry dependency closure，因此稳定帧反复承担 `O(V + E)` 图遍历；但任意叶子 shader 变化又不会改变材质保存的直接 shader revision 或 `PipelineKey::shader_revision`。结果是材质可能从热路径提前返回，已有 Mesh PSO key 也继续命中旧代源码。当前 last-good leaf 能保留旧 artifact，却还没有可识别“新候选闭包”的统一代际。

磁盘/预热域不能直接复用进程内代际。`ResourceReadinessGeneration::dependency_revision` 是不可变分片投影上的单调 epoch，fingerprint 包含 load state，并使用实现内部哈希；它适合当前进程的 O(1) dirty check，不是稳定跨进程内容摘要。相反，variant disk key 已绑定最终组装 WGSL 的 BLAKE3、include content hashes、template revision 和 Naga/WGPU version，读写还重算 payload hash。把 readiness epoch 写入 `ShaderVariantKey` 或 prewarm manifest 会混淆两种身份且破坏跨运行复用。

### 15.2 参考实现结论

- Unreal `FMaterialShaderMapId` 把 base material、referenced functions/collections、shader/pipeline/VF dependencies、texture references、expression includes 和 external code references纳入完整 shader-map identity；`BuildShaderMapIdOverride` 统一填充这些依赖，而不是仅比较根材质 revision。
- Bevy `ShaderCache` 显式保存 resolved imports、dependents 和使用该 shader 的 pipelines；shader set/remove 会沿 dependents 清理处理后源码并将所有受影响 pipeline 重新排队。可迁移原则是“依赖闭包变化必须进入 pipeline identity/失效”，不是照搬其容器实现。
- Zircon 离线 prewarm 已以 SCC 压缩的 include DAG 在 `O(V + E)` 工作内生成稳定 topology hash，并有直接、传递、registry module 变化测试；runtime 已有资源级 reverse closure 和 `dependency_revision`。当前缺口是消费既有运行时投影，而不是建立第三套 per-frame shader 图。

### 15.3 MVP 实施边界

1. `PreparedShader` 同时保存根 `revision` 与 readiness `dependency_revision`。一次 `ensure_shader_source` 固定读取同一不可变 readiness generation；两者均命中时直接 `O(1)` 返回，不再递归闭包。
2. 代际变化时仍按依赖优先顺序准备完整 shader closure，并只在所有依赖成功后发布根 `PreparedShader`。失败继续保留旧根与旧材质 bundle；恢复后发布新代。
3. `PreparedMaterialShaderDependency` 保存直接 shader id/revision 和传递 dependency revision。材质 cache hit 必须同时匹配三者，叶子 include 变化因此进入 candidate preparation，即使材质和根 shader revision 都没变。
4. 仅运行时的 `PipelineKey` 增加 `shader_dependency_revision`，并由所有 Mesh pass/OIT 共用的 variant registry 哈希。`ShaderVariantKey`、prewarm manifest 和 disk canonical string 保持持久内容语义不变；最终 WGSL/source ID 继续负责跨进程正确性。
5. 本切片不在无 frame/fence owner 的情况下直接删除旧 variant id 或 WGPU pipeline。旧代 PSO/variant 的有界退休、异步任务取消和跨 pass 原子 swap 仍属于后续 generation/fence 工作；必须记录热重载 resident growth，不能把正确换键误报为完整生命周期收敛。

### 15.4 性能与验收门

算法规模目标是稳定 material/shader hit 为一个 generation snapshot 加常数次哈希表/标量比较，叶子变化才承担一次受影响闭包重建；不得在 per-draw 或每个 pass 重走依赖图。新增计数应区分 shader artifact hit/rebuild、dependency-generation invalidation、publish 与 last-good rejection。

受管 Windows profile 必须覆盖 1/100/10,000 个共享 include 的材质、深度 16 的依赖链和 diamond fan-out：采集 warm hit 的 dependency edge visits、material rebuild、uniform allocation、module/PSO create、CPU p50/p95/p99 与 RSS；再执行有效叶子 reload、无效叶子 reload 300 帧、恢复三阶段。通过条件为稳定帧 dependency edge visits 为 0，无效候选不发布根 shader/材质/PSO，恢复只形成一个新代；旧代 variant/PSO resident 数必须量化并进入后续退休设计。PNG、`D:\Tools\renderdoc` capture/replay、GPU timestamp 与同机功耗仍是里程碑验收项，本 source 切片不声明实测收益。

### 15.5 Source implementation record

- `PreparedShader` 现在发布根 revision 与 `ResourceReadinessGeneration::dependency_revision` 的成对身份。一次顶层 ensure 固定持有同一不可变 generation；稳定命中不再调用 `ensure_shader_dependency_sources`，从每次完整闭包遍历收敛为 O(1) 身份比较。只有根或闭包代际变化才按 dependency-first 顺序重建，失败仍不覆盖旧根。
- `PreparedMaterialShaderDependency` 同时快照直接 shader id/revision 与传递代际；正常材质 cache hit 通过一次 registry lookup 和 readiness row lookup比较该三元组。叶子 shader 变化无需合成 material revision 或根 shader revision即可进入 candidate prepare。
- `PipelineKey::shader_dependency_revision` 进入共享 Mesh variant registry 的 `Hash/Eq`，因此 Base、GBuffer、DepthPrepass、Shadow、Velocity、TAA reactive 与 OIT 派生同一新代。该字段没有投影到 `ShaderVariantKey`；disk/prewarm 继续由最终 WGSL/source contract 区分，且相同组装源码仍可复用 shader-module source-hash entry。
- 新 profile counters 为 `shader_artifact_cache_hit`、`shader_artifact_rebuild`、`shader_dependency_generation_invalidation`，并与已有 `shader_artifact_publish`、material prepare/last-good/uniform counters 联合使用。纯契约测试固定“运行时 dependency epoch 改变 PipelineKey、但不改变持久 ShaderVariantKey”；产品回归改为根 shader/material revision 不变的叶子 include `valid -> invalid -> recovered` 序列，要求失败期保持旧 shader/material/pipeline/uniform，恢复期只推进 dependency revision 并形成新共享 pipeline key。
- 静态 mesh command cache 的旧失效身份只看资产 material revision，但其不可变 payload 实际持有 `PipelineKey`、材质纹理/采样器 bind group 和两套 uniform bind group。传递 shader leaf reload、frame 级 volumetric-fog specialization，以及 mip streaming 替换实际 `GpuTextureResource` 时，都可能在资产 revision 不变的情况下误复用旧 payload。当前 draw-facing bundle 每次成功发布分配非持久 `draw_generation`；mesh draw 在 anisotropy 与最终 pipeline policy 落定后，再把该代际、完整 `PipelineKey`、六项纹理绑定身份和两套 uniform 资源身份合成为非零 `material_submission_revision`，作为 command cache 的 material invalidation authority。冷路径无权威 revision 时仍保持 0，不能被指针 hash 意外升级为可缓存。该身份只用于进程内 resident command payload，不进入资产、磁盘或 prewarm key。
- `PendingPreparedMeshBatchKey` 同时补入此前遗漏的 clearcoat normal binding identity，使 prepared-queue unique-batch 统计与实际六纹理材质布局一致。这里没有宣称降低 CPU/GPU 时间；managed profile 仍需分别量化正常稳定帧 cache hit、mip residency replacement、shader leaf reload 和 volumetric policy change 的 rebuild 数量及主线程耗时，确认稳态无额外 rebuild、变化帧只失效受影响 draw。
- 限定 Rust `rustfmt` 与 scoped `git diff --check` 已通过；未运行 Cargo、WGPU 或产品捕获。由于 variant registry 目前采用稳定递增 id 且各 pass 缓存按 id 持有 WGPU pipeline，旧代不会在本切片中强删。跨 pass swap 与同身份 terminal retry suppression 已在 15.6 实施；300-frame 动态证明、旧代 resident 上界和 fence retirement仍为 P0-4 后续工作，本节不关闭里程碑。

### 15.6 Cross-pass publication owner review

复审前的帧序是 `ResourceStreamer::ensure_scene_resources` 先发布新 `PreparedMaterial`，随后 `SceneRendererCore::render_scene` 才 build mesh draws，并在各 Base/GBuffer/DepthPrepass/Shadow/Velocity/TAA/OIT record owner 中分别解析 variant 和创建 PSO。这解释了为何共享 `PipelineKey` 只能防止旧 PSO 误命中，却不能阻止某些 pass 已切新代、另一些 pass 仍 Deferred/Failed 的混代帧。资产 owner 也不能反向调用 `MeshPipelineCache`，否则会形成 resource preparation -> renderer PSO 的错误依赖方向。

material revision、shader/texture dependency identity、`MaterialRuntime`、custom uniform 和 standard uniform 已收拢为单一 `PreparedMaterialBundle`。已有 last-good 的成功热更新不再立即覆盖 published bundle，而是进入 staged candidate；cold success 同样先进入 staged candidate，`PreparedMaterial.published` 在无 last-good 时允许为空。材质依赖或 readiness 失败会清除不再匹配的 staged candidate并保留旧 published bundle。

renderer-side publication coordinator 已接入 direct 与 compiled 两条 scene path。当前帧先完全用 published bundle build draw/command payload；同时仅在 active staged candidate 存在时，从当前 scene 实际引用它的 material/geometry/quality、最终 volumetric-fog policy 和 compiled graph executor set 生成 requirement census。culling 与“本帧恰好已有 previous velocity history”不参与候选身份，避免隐藏对象重新可见或下一帧刚产生 history 后才补 PSO；material `disabled_passes`、alpha/forward/transmission phase、shadow policy、geometry eligibility 和当前 graph feature仍精确裁剪集合。这覆盖 8 个 `MeshPassPipelineKind` 与独立 OIT target，但不会预编译当前 graph、scene 和 geometry 组合之外的 pass。没有任何 draw 引用的 candidate 不再以“零 requirement”名义直接发布，而是在 terminal 被 park 为非 active staged；以后 `ensure_material` 真正触及同一 staged identity 时再 O(1) re-activate。

每个 requirement 在同一 admission owner 中解析 exact variant，并推进 Base/GBuffer/DepthPrepass/ShadowDepth/ShadowDepthAlphaMask/Velocity/TAA reactive/TAA material mask/OIT。整个集合都被检查，terminal `Failed` 优先于 `Deferred`。相机提交链复审确认 `SceneRenderer::generation` 每个 camera submission 都递增，不能充当逻辑帧边界；框架实际按 camera sequence 顺序提交，并由 `ViewportCameraStackOutputPolicy` 显式标记 viewport submission start 与 terminal owner。candidate 在 start 先清除上一次未完成/失败相机循环留下的聚合位，再把本 viewport 已观察/deferred 状态随 staged generation 聚合：非终端相机只推进 PSO，任一相机 Deferred 会阻止本 viewport 发布；仅在 viewport-terminal 且本轮至少一个 draw 引用 candidate、所有已观察 requirement 均 Ready 时才一次移动 staged bundle 到 published。切换发生在当前相机 draws 已固定之后，因此当前 viewport 提交仍是完整旧代，下一次提交才观察新代。`Deferred` 保留候选和旧代；terminal failure保留候选的完整 shader/texture/support identity作为拒绝缓存，从 active census索引移除，同一身份随后 O(1) cache hit，不再每帧重新创建两项 uniform buffer。任何根/传递 shader、材质、纹理 revision或 texture-support identity变化都会使该 bundle identity失效并重建，同时重置 admission cycle；资产回退到 published identity时会清除历史拒绝态。

稳态性能边界是显式的：`ResourceStreamer` 维护 active staged material ID `HashSet`，没有 active candidate 时 compiled graph executor scan与 draw census均跳过，新增 admission 工作为 O(1) `is_empty` 和常数 counters；未引用 candidate 在一个 viewport cycle 后退出 active set，避免离场资产永久维持 O(D) scan，真实 material preparation cache hit 才将其重新激活。候选存在时单 camera submission 工作为 O(P + D + R)，viewport camera stack 总量为各 submission 的该项求和，其中 P 是当前 graph pass数、D 是候选发布期间的 scene draw数、R 是按 target/key/geometry/quality去重后的 requirement数。cycle state 每 candidate 仅两个 bool，不保存或全排列展开 requirement set；候选发布顺序按 `ResourceId` 排序以保持确定性。profile 新增 staged/cache/terminal-cache/reactivated、requirement/ready、candidate published/deferred/failed/unobserved counters；以上是源码算法规模，不是 CPU/GPU 时间或功耗实测。

source-ready cold start 没有 last-good 时不再直接发布真实 candidate。draw-facing accessor 在 `published=None` 时成对返回 default `PipelineKey`、fallback custom/standard uniform、fallback textures，并以洋红 base tint 形成 engine-owned error proxy；candidate 本身只供 exact requirement census/admission，全部 Ready 后才进入 published，terminal failure则保留 error proxy与结构化诊断。该代理复用已经预置的标准材质 ABI 和各 mesh pass 的 default pipeline，不做“只换 shader”的错误替代；独立可编辑 error-material asset不是 MVP 必需项。shader/material readiness 在 candidate 构造前失败的 fail-closed 返回仍属独立边界，不由本段伪装为 PSO fallback。旧 bundle 的 uniform/texture/geometry拥有型句柄由本帧 draw/command payload持有到提交，WGPU命令资源生命期不会因 published swap提前结束；但 variant/module/PSO 的显式有界 retirement仍未接线，不能把引用安全等同于 P0-4 生命周期完成。

这仍不是跨未知 viewport/context 的最终原子策略。框架持有一个共享 renderer，而 viewport 分别提交；第一个 viewport 完成发布后，后续首次出现的不同 graph、shader quality、volumetric policy 或 geometry source 仍可能需要新 PSO。为所有 4 个 quality、fog 双态、全部 pass 和 geometry 做笛卡尔积预编译会把 MVP 变成 compile storm，因此本轮明确不采用。最终方向应与 Unreal 的 material render proxy/fallback proxy 一致：保留 last-good generation，并让 draw admission 按实际 context 选择新代或旧代，直到该 context 的完整 PSO set Ready；同时由 cook/usage capture 提供常用 domain prewarm。该项与统一 `PipelineService` 一并保持 P0-4 开放，当前实现只声称“单 viewport camera sequence 不混代”。

2026-08-26 的受管 Windows 验证尝试均在 Cargo 启动前停止：首次 `cargo.acquire` 已接收但 reconciliation 超时；第二次被 `unmanaged_artifacts_detected` 拒绝，涉及 D/E/F 上其它会话的既有未登记产物。当前会话没有删除或接管这些产物，也没有绕过 validator 直接运行 Cargo。因此本节只有限定 `rustfmt`、源码契约和 scoped diff integrity 证据；Cargo type check、WGPU、RenderDoc、PNG、timing、RSS 与功耗仍全部待验收。

回收不需要新建 frame-delay fence。`RenderBackend` 已通过 `WgpuSubmissionCoordinator` 发行包含 device id、device generation、queue class 和 monotonic sequence 的 `SubmissionTicket`，`SubmissionHistory` 在可见 terminal status 被有界淘汰后仍保留压缩 terminal ranges；WGPU resource registry 也已有“所有 last-use ticket 终态后才 drop”的 retirement 实现。compiled render path 已把同一 ticket 传给 transient resource pool，direct path 则仍丢弃 `submit_graphics_command_buffers` 的返回值。PSO/module retirement 的正确后续是让实际 mesh variant usage在 submit 后关联这个 ticket，并复用 `poll_submission_completions` / `submission_status`；不得创建第二套 `N-frame` 猜测或单独调用 native queue completion。

2026-08-26 驻留复审确认 `MeshPipelineVariantRegistry` 的 `variant_keys: Vec` 与 `variant_ids: HashMap` 只增不减，Base/OIT/GBuffer/Depth/Velocity/Shadow/TAA PSO maps 和 shader-module map 也没有正常代际回收。variant ID 同时被 command cache、异步 job、failure/admission key 和 diagnostics 使用，因此不能在没有 usage ticket 的情况下删除或复用。现有 `ShaderVariantMissReport` 已提供 `registered_pipeline_variant_count`、`registered_shader_variant_count`、`cached_render_pipeline_count`、`cached_shader_module_count`、创建次数和创建 CPU 微秒，不再重复添加同义 counter。300-frame valid/invalid/recovery profile 必须逐帧导出这些 gauge，并满足：同一 terminal-invalid identity 不增长；每次 recovered content generation 的暂态增长与实际 pass/geometry/quality requirement 数一致；接入 retirement 后，submission terminal 之前不下降，terminal 之后回到“当前 generation + 明确 last-good/context fallback”的有界驻留。实现顺序固定为“从最终 MeshPass command buffers 提取实际 variant usage -> submit 返回 ticket -> 更新每 variant last-use ticket set -> terminal 后统一淘汰 registry tombstone/module/各 pass PSO/failure state”；direct 与 compiled 必须共用该路径。在 usage set 接线前，本轮不做误删风险高的回收，也不做给全表每帧续命的 O(N) 假实现。

## 16. 2026-08-27 P0-5 Mesh WGSL 验证准入与反射生命周期复审

状态：**source admission implemented; managed compile/WGPU/profile pending**。本节修复的是“验证不参与发布”的结构错误，不是已有 profile 证明的性能优化。

### 16.1 已确认的结构错误

Mesh 的 bounded Naga worker 原本只回传诊断，成功反射被丢弃；runtime 可以先写 disk cache、创建 shader module 和 PSO，稍后才观察验证结果。该顺序把 authoring diagnostics 与 driver publication 分成两套真相。Unreal 本地源码 `MaterialShared.cpp` 的 `FMaterial::HasValidGameThreadShaderMap` 明确要求 ShaderMap 已 `IsCompilationFinalized()`，而 render-thread 另持有已发布 ShaderMap；cmftStudio 只提供 filter 后台化参考，其共享 `ThreadStatus` 不具备 Zircon 所需的 immutable publication 语义。

### 16.2 MVP 硬边界

1. 以完整 `ShaderVariantKey + validation_source_identity` 建立 `Missing/Pending/Ready/Failed` 单一状态机；Pending 不得写 cache 或进入 WGPU，Naga rejection、worker unavailable、job panic 必须保留 exact typed terminal identity。
2. `Ready` 携带 `Arc<ShaderTemplateReflection>`；WGPU module 与 reflection 进入同一 `CachedMeshShaderModule` owner。只有 module 实际安装后才转移 Ready artifact，Base async PSO queue full 不得触发重复 Naga。
3. 7 个物理 pipeline owner、9 个 target kind 全部先走 source gate 和 cached-module gate。纯程序表覆盖 10 个 executable mode：Depth opaque 与 alpha 分离，Shadow opaque vertex-only，两个 TAA/OIT fragment 名不允许 alias。
4. 正常 frame 只 poll/drain，不等待 validation；显式 finish 只属于 startup prewarm 与 tests。worker bound 为 64，state/counter lookup 为 O(1)。
5. 当前只校验 exact executable entry。完整 bind-group ABI 必须由 scene/material/GPU-scene layout owner 发布 descriptor 后再比较；不得从 opaque WGPU handle 或重复常量猜 DTO。

### 16.3 性能模型与待测风险

稳定 module hit 不访问 validation state；新 exact source 通常执行一次 Naga。PSO queue saturation 时 Ready reflection 常驻但 disk lookup 可能重试；invalid hot-reload identities 在 retirement 接线前也会累积。两项都必须进入 1/100/10,000 variant cold/warm/reload profile：记录 Naga 次数与 p50/p95/p99、disk lookup 次数/字节、Ready/Failed identity count、module/PSO create、queue saturation、CPU/RSS，并与 current/last-good generations 联合观察。只有数据证明 lookup 或 resident failure rows 成为主要成本后，才允许设计 validated-source resident cache 或 identity retirement；不能先加 per-frame O(N) sweep、时间退避或盲目容量淘汰。

### 16.4 Source record

TDD 先得到缺少纯 program mapping 的编译红灯，实施后独立 harness `11/11` 通过；限定 `rustfmt --check`、7-owner gate/consume inventory、production file budget 和 scoped `git diff --check` 通过。磁盘 lookup 已复核 metadata 全 key 与 decoded WGSL source hash，因此 reflection/source identity一致。没有运行 Cargo、WGPU、PNG、`D:\Tools\renderdoc`、shader-stage profile、RSS、GPU timing、能耗或功耗；P0-5 只记录为 source implemented，不能关闭完整 PSO/ShaderMap 生命周期或宣称性能收益。

## 17. 2026-08-27 Shader cache 工具链身份复审

状态：**source identity hardened；managed compile/profile pending**。这是一项 P0-2/P0-5 基础正确性修复，不是新的性能里程碑。

### 17.1 根因

工作区以兼容范围声明 `naga = 29.0.1`、`wgpu = 29.0.1`，但当前 `Cargo.lock` 已把两者解析为 `29.0.3`。Mesh runtime、dynamic builtin prewarm 与独立 asset-scan prewarmer 仍各自把 `naga-29.0.1`、`wgpu-29.0.1` 写入 source/disk cache identity。实际执行 29.0.3 parser/backend，却以 29.0.1 命名 cache artifact；若补丁版本改变 WGSL 接受范围或翻译行为，旧新 artifact 可能错误同键。

### 17.2 已实施硬边界

1. 工作区直接依赖精确钉住当前锁文件的 `=29.0.3`，以后补丁升级不能在未审查 cache identity 时静默发生；本次锁文件已有该版本，不引入 dependency resolution 变化。
2. `core::framework::render::shader::variant_prewarm` 成为唯一工具链身份 owner，公开 Naga/WGPU cache token；runtime、dynamic API 与 prewarm binary 只消费公共常量，不再拥有版本字面量。
3. 单元契约使用已有 `toml` parser 结构化读取 workspace manifest 与 lock packages，同时要求“精确声明版本 = resolved version = cache token”。未来升级必须在同一 source snapshot 显式推进三者。
4. source ID、disk key 与 metadata 的现有哈希结构不变；版本 token 从错误的 29.0.1 修正为真实的 29.0.3，会按设计形成一次新 namespace，不添加兼容别名或读取旧版本回退。

### 17.3 性能边界与证据

热路径新增成本为 0：常量替换不增加 per-frame/per-draw hash、allocation、Naga、WGPU、文件 I/O 或分支。正确的 namespace cutover 会产生一次预期 cold miss；必须在受管 1/100/10,000 variant cold/warm/restart profile 中记录 miss、Naga/module/PSO create、disk bytes 与各阶段 p50/p95/p99，不能把“避免潜在错误 hit”直接写成耗时收益。

TDD 只读契约先得到 `5 defects` 红灯：2 个 declaration/lock 不一致与 3 个独立生产字面量；实施后报告 exact workspace/lock identity 与 3 个 shared consumers 通过，限定 `rustfmt --check` 和 scoped diff integrity 通过。没有运行 Cargo、WGPU、PNG、`D:\Tools\renderdoc`、GPU timing、RSS、WPR、能耗或功耗，因此 P0-2、P0-5 与 M6-M8 状态不关闭。

## 18. 2026-08-27 Mesh resource-class ABI admission 复审

状态：**resource-class source admission implemented；managed compile/product/profile pending**。本节继续关闭 P0-5 的发布正确性缺口，不将其描述为已完成的性能优化或完整 WGPU ABI 验证。

### 18.1 根因与 owner 边界

entry name/stage 正确仍不足以证明 PSO 可发布：shader 可在同名 entry 中声明错误的 group/binding、buffer access、texture dimension/sample class、multisample 或 comparison sampler。此前 scene/material/GPU-scene layout 以 opaque WGPU handle 进入 Mesh cache，资产 readiness DTO 又缺少上述语义；从资产 magic constant 猜 layout 会产生第二套 ABI 真相。

当前由五个真实 renderer owner 暴露其实际用于 `create_bind_group_layout` 的 entry 集合，Mesh cache 构造期只做一次语义投影，并固定三份不可变合同：full `0/1/2/3`、environment-only `0/2/3`、OIT `0/1/2/3/4`。Naga reflection 对 required entry 的 reachable globals 分类，Ready source 在 disk lookup/write 与 WGPU module 前校验；cached module 在 reuse 前按 exact target 再校验。额外 layout binding 与无关 entry 合法，不扩大 variant domain。

### 18.2 MVP fail-closed 范围

合同精确比较 uniform/storage buffer 及其只读性、sampled texture 的 view dimension/sample class/multisample、sampler comparison 和 stage visibility。storage texture、binding array、acceleration structure、external texture 及 Naga 无法投影的 handle 类型统一 `Unsupported` 并拒绝发布。

这不是完整 buffer/operation ABI。后续 §20 已将具有可信 shader 语义来源的 `min_binding_size` 前移；buffer member byte layout、dynamic offset、float texture filterability 与 sampler filtering operation 仍由真实 WGPU descriptor 和 error scope 最终校验。其余字段在两侧没有 exact semantic source 前不得伪造等价判断；后续扩展应先建立 canonical shader ABI DTO 与 layout owner 的同源生成，而不是继续添加位置常量。

### 18.3 算法与 profile 门

三份 layout 合同各在 cache construction 做一次 `O(L)` HashMap 建立；每个新 source/cached module admission 只遍历 required entries 的 reachable resources，以 `(group,binding)` 做期望 `O(R)` lookup。稳定已安装 pipeline 不访问 reflection/合同，不存在 per-frame layout/variant 全表扫描。

受管 Windows profile 必须覆盖 1/100/10,000 exact variants，并分别记录 cold/warm/restart：合同构造耗时与 map count/capacity、每 entry reachable resource count、resource admission p50/p95/p99、Naga/disk/module/PSO 次数、interface rejection、queue saturation 和 peak RSS。通过条件为稳定 cached frame 新增 admission 次数为 0；错误 ABI 的 source 在 disk write 与 WGPU module create 前终止；总 admission 工作量随 reachable resource 数线性增长，而不随 registry resident variant 总数增长。GPU timestamp/功耗只有在产品提交存在时才采集；纯拒绝样本必须记为 not applicable，不能伪造收益。

### 18.4 Source implementation record

后端无关的 `ShaderBindingResourceType/Stage/Visibility` 成为 reflection 与 WGPU layout 的共享语义层；full/environment-only/OIT 合同由实际 entry owner 构造。entry contract 提供 required-entry visitor，source Ready gate 与 cached-module gate 复用同一 `validate_reflection_shader_contract`。unsupported 类型 fail-closed，HashMap key 保持 `(group,binding)`，没有引入 per-frame allocation 或 scan。

TDD 初始 RED 为缺少 resource-contract source；当前纯契约 `9/9`、现有 WGPU rlib 类型转换 `5/5`、Naga reflection `9/9`、owner/gate 静态合同 `16/16` 通过，限定 `rustfmt` 与 scoped diff integrity 通过。`reflection.rs` 的新增分类实现已拆到独立子模块，主文件从 995 行收敛到 961 行。临时测试产物仅在 E 盘生成并已清理。没有运行 Cargo、产品 WGPU/DX12、PNG、`D:\Tools\renderdoc`、GPU timing、RSS、WPR、能耗或功耗；P0-5、M6-M8 和 bottleneck-removal 结论保持开放。

## 19. 2026-08-27 Authoring layout visibility admission 复审

状态：**source admission corrected；managed compile/product pending**。本节修复序列化 shader layout 的 stage 集合判定，不把 authoring DTO 与 runtime/WGPU reflection DTO 错误合并。

### 19.1 分层结论

`RenderShaderBindingResourceType` 是资产/authoring 的稳定粗粒度枚举，`ShaderBindingResourceType` 是 runtime 准入所需的精确语义。Unreal 同样以 `FShaderParametersMetadata::FMember` 保存 base type/offset/dimensions/layout signature，再由 `ValidateShaderParameterTypes` 对 compiler reflection；RHI layout 是后置层。Zircon 因此保留两层，并在发布门显式比较，不把资产格式绑定到 WGPU texture dimension、sample class 或 sampler operation。

进一步复核也否决了“资产 expected layout 直接由完整 GPUScene WGPU entries 生成”：真实 group 3 有 12 个 vertex/fragment/compute binding，而 Surface authoring contract 只公开 0-4 的 draw-facing 子集。两者差异是有意的域裁剪，不能用全量 backend layout 覆盖资产 ABI。

### 19.2 确认并修复的错误

旧 `binding_has_required_visibility` 使用集合交集非空作为成功条件。对只允许 fragment 的材质 texture，声明 `[fragment, compute]` 仍会被放行；错误 compute 可见性直到更晚的 runtime/WGPU 门才暴露。现在非空声明必须是 allowed stage set 的子集；空列表沿用既有 opt-out。字段和诊断同步从 `required/include` 改为 `allowed/subset`，避免继续误导调用者。

### 19.3 复杂度与证据

检查保持 `O(B * S)`，其中可序列化 stage 只有 vertex/fragment/compute 三项；不增加 frame、Naga、WGPU、allocation、I/O、GPU 或 cache 工作。RED 契约锁定 overlap `.any(...)` 与缺失 mixed-stage regression；GREEN 源码契约 `4/4`、独立 truth table `1/1`、限定 `rustfmt` 和 scoped diff integrity 通过。临时测试只在 E 盘生成并已清理。没有 Cargo、产品 WGPU/DX12、PNG、RenderDoc、timing、RSS 或功耗证据，P0-5 与 M6-M8 不关闭。

### 19.4 Material group-2 唯一布局 owner

历史 group-2 ABI 漂移记录说明问题不只在可见性判定：WGPU material layout creator 与 authoring validator 曾各自维护一份 0-12 的 13 行表，即使当前数值相同，也没有工程边界阻止后续 texture slot/clearcoat 只改一侧。当前新增 scene 内部静态 material contract，统一拥有 group、binding、粗粒度 resource class、allowed visibility 与 diagnostic label；真实 WGPU entry factory 和 authoring validator 均从该表投影。binding 连续且唯一由单元不变量固定。

该 owner 不跨越错误的抽象边界：authoring DTO 不吸收 WGPU 的 texture dimension/filterability/multisample、sampler class、buffer min size/dynamic offset，也不替代 Naga runtime reflection。GPUScene authoring 仍只校验 draw-facing 0-4 子集，但数值改为消费真实 GPUScene owner 已有的五个命名常量；完整 12-binding backend layout 不会反向扩张 Surface asset ABI。

复杂度没有被包装成未经测量的收益。renderer construction 只生成一次固定 13 项栈数组，无 heap allocation；authoring admission 读取静态 slice，仍为 `O(B * S)`，错误路径沿用既有诊断字符串分配。GPUScene 剩余的 5 行 draw-facing expected table 也进入同一静态 owner：它消费真实 binding 常量但仍只公开有意的 `0..4` 子集；validator 不再逐次构造局部表，contract constructor 收窄为 owner 私有。TDD RED 为 canonical owner 缺失、GPUScene 局部表及开放 constructor；GREEN 纯 contract `3/3`、现有 `wgpu 29.0.3` entry projection `3/3`、source ownership `3/3`、旧局部 symbol inventory、限定 `rustfmt` 与 scoped diff integrity 通过。未运行 Cargo、产品 WGPU/DX12、PNG、`D:\Tools\renderdoc`、CPU/GPU timing、RSS、能耗或功耗，所以这里只关闭双重定义风险，不关闭 P0-5/M6-M8，也不声明瓶颈消失。

### 19.5 Authored `pipeline_layout` 硬切清单

tracked `.zshader/.zmeta/TOML/JSON` 的实际 authoring 命中为 0，schema-v2 也已显式拒绝 `pipeline_layout`，因此长期方向应删除 legacy DTO，而不是继续把它包装成 renderer ABI authority。但当前 Rust 数据链仍把该字段序列化进 artifact-cache shader payload，投影进 readiness/management report，并由 importer、builtin 和大量 programmatic fixture 复制；删除还会影响 material diagnostic 的 empty-layout opt-out。artifact store 有版本化 manifest，不能在未决定 cache invalidation/旧 payload 行为时依赖 serde 偶然兼容。

该硬切必须作为独立跨模块迁移执行：先冻结 readiness/management 外部字段去留与 cache schema 策略，再在同一 snapshot 删除 `ShaderAsset` 字段、framework DTO export、artifact payload、validator、constructor default 与 fixture，最后覆盖 cache round-trip、v2 forbidden-field、readiness、material publication 和受管 WGPU。当前 asset/importer/framework owner 正被其它 Session 修改，Shader06 不覆盖或吸收这些变更。本条只是结构调研和迁移门，不是运行时优化，也不阻塞继续完成其它不相交源码任务。

## 20. 2026-08-27 Buffer minimum-size ABI admission 复审

状态：**source admission implemented；managed compile/product pending**。锁定的 `wgpu-core 29.0.3` 以 Naga `TypeInner::size(module.to_ctx())` 计算 shader buffer minimum；Naga 对 trailing runtime-sized array 计入一个 element。pipeline layout 显式 `Some(min_binding_size)` 小于 shader minimum 时应在 pipeline creation 失败；layout `None` 则故意保留为 draw/dispatch 的 effective buffer-range late check，不能在前置 gate 误拒绝。

现有 validated-module reflection 现在只对 uniform/storage 发布该 minimum，Mesh WGPU owner projection 保留 descriptor 的 `Option<u64>`，source Ready 与 cached-module gate 复用同一比较：仅拒绝 `Some(layout) < Some(shader)`。实现不建立第二个 Layouter，不新增 parse、validation、I/O、WGPU、wait、frame-thread 或 per-draw 工作；type-layout hash 已覆盖 size 的决定输入，因此不重复推进 reflection hash namespace。

边界仍然明确：host owner 尚无可与 shader `type_layout_hash` 比较的 member offset/type-shape hash，dynamic offset 不是 shader 声明，layout `None` 的实际 buffer range 仍由 WGPU command-time 检查；texture filterability 与 sampler operation 已由后续 §21 的 exact sampling-pair gate 前移。RED source gate 为 5 个缺口；GREEN 纯 semantic contract `5/5`、现有 WGPU conversion `6/6`、Naga reflection `9/9`、source wiring `5/5`，覆盖 16-byte uniform、4-byte runtime storage、exact/larger/smaller/None layout。限定 `rustfmt` 与 scoped diff integrity 通过。未运行 Cargo、产品 WGPU/DX12、PNG、`D:\Tools\renderdoc`、timing、RSS、能耗或功耗，因此只声称更早的正确拒绝，不关闭 P0-5/M6-M8，也不声明性能瓶颈消失。

## 21. 2026-08-27 Texture sampling-pair ABI 与 PSO retirement 前置条件复审

状态：**sampling operation source admission implemented；managed compile/product/profile pending**。本轮先完成发布正确性的基础设施，并修正 P0-4 的结构性优化前提；没有在数据不足时实现容量淘汰，也不把聚焦测试写成产品性能收益。

### 21.1 P0-4 retirement 结构复审

最终 Mesh variant usage 已能绑定 direct/compiled 的真实 `SubmissionTicket`，共享 submission coordinator 也能给出 terminal status；但 registry 尚未固定“当前已发布 material generation + 明确 last-good/context fallback”所需的 variant 集合，容量上限也没有 1/100/10,000 variant profile 数据。此时直接添加 LRU、age 或固定条数淘汰，会把稀疏使用但仍属当前代的 PSO 删除并反复重建，无法证明 RSS、功耗或总耗时改善。

正确顺序冻结为：先发布 active generation/required variants；再 pin 当前代、last-good/context fallback、queued work 和所有 non-terminal submission users；根据 profile 建立 resident budget/pressure threshold；最后在全部 ticket terminal 后，以单个事务淘汰 registry tombstone、shader module、各 pass PSO 以及 failure/admission rows。该边界与 Unreal 的 cache generation/ref usage 和 Bevy 的 stable pipeline identity 一致；cmftStudio `ThreadStatus` 只适合作业生命周期，不承担 GPU resource lifetime。当前不添加 eviction source，P0-4 继续开放。

### 21.2 Sampling-pair 根因与实现

资源声明集合相同不代表 pipeline-layout 操作 ABI 相同。WGSL `texture_2d<f32>` 不声明 concrete WGPU view 是否 filterable；是否可用 `Filtering` sampler 取决于 exact entry 中发生的 texture/sampler 配对。Naga 29.0.3 已在 validated `FunctionInfo::sampling_set` 计算该关系并透过 helper 参数传播，wgpu-core 29.0.3 也按 entry sampling pair 联合校验 provided layout，因而不应另写 IR walker 或从 binding 位置猜关系。

反射现在按 entry 发布 canonical texture/sampler binding pairs，并把它们纳入 v2 entry/module resource-layout hash；同资源集合但 `textureSample` 与非采样 use 不再错误同键。WGPU owner projection 保留 float texture 的 `filterable` 和 sampler 的 `Filtering/NonFiltering/Comparison`，Ready 与 cached-module gate 都验证 Naga-proven pair：Filtering + non-filterable float、Filtering + Sint/Uint fail closed，NonFiltering 合法，comparison 仍由 exact resource class 保证。未参与采样的 sampler 不会误伤同 layout 的其它 texture。

复杂度为每个新 entry 对 `P` 个已有 Naga pair 做一次 canonical `O(P log P)` 排序，准入用 `(group,binding)` map 做期望 `O(P)` 查询；稳定已安装 pipeline 为 0 次，不新增 frame/per-draw 全表扫描、parse、worker、I/O、wait 或 WGPU object。v2 namespace 的一次 cold invalidation 是语义变更的必要结果，不能计为性能收益。

TDD RED 为 5 个 production 缺口，GREEN source gate `5/5`；复用 E 盘既有 `naga/wgpu/blake3` 产物完成独立 rustc 编译，反射 `11/11`、semantic contract/WGPU projection `8/8`，合计 `19/19`。覆盖 helper argument 传播、同资源集合不同操作 hash 分离、filtering/non-filtering 配对；限定 `rustfmt --check` 通过，临时 E 盘目录已删除。未运行 Cargo、managed product compile、WGPU/DX12、PNG、`D:\Tools\renderdoc`、timing、RSS、能耗或功耗，所以这里只记录 source infrastructure，不关闭 P0-4/P0-5/M6-M8，不声明瓶颈或功耗问题已经消失。

## 22. 2026-08-27 跨阶段 ABI 准入与 error-proxy 边界复审

状态：**vertex/fragment source admission implemented；P0-7 bootstrap/context identity 与受管验证待完成**。本节补的是 pipeline 创建前的接口正确性，并修正旧计划对 error material 现状的误判，不把源码门禁写成性能收益。

### 22.1 跨阶段接口的真实规则

Naga 只证明单个 entry 合法；此前 exact entry 与 resource ABI 均可通过，但选中的 fragment `@location` 仍可能缺少对应 vertex output，或在类型、interpolation、sampling、`per_primitive` 上不兼容，最后才由 WGPU PSO 创建拒绝。直接比较现有 type-layout hash 会错误拒绝 WebGPU 合法的 producer-wide/consumer-narrow 关系。

实现按锁定的 `wgpu-core 29.0.3` 规则保存最小 numeric DTO：scalar kind/width 与 scalar/vector/matrix dimension。fragment scalar 可消费同 kind 的 vertex scalar/vector；fragment vector size 不得超过 vertex vector；scalar width 不得更宽；matrix 行列必须相等；interpolation、sampling 和 `per_primitive` 必须精确相等，额外 vertex output 合法。Ready-source 与 cached-module 共用同一 exact program link gate，发生在 resource admission、disk publication 和 WGPU module 创建之前。

反射的 stage IO 已按 binding canonical 排序，因此校验使用双游标归并，复杂度 `O(V + F)`、零分配；不增加 IR 遍历、worker、文件 I/O、WGPU object、frame/per-draw 分支或 registry 全表扫描。稳定已安装 pipeline 的新增成本为 0。独立 E 盘 rustc harness 由缺少实现的 RED 转为 `16/16` GREEN，覆盖 `vec4<f32> -> vec3<f32>`、unused output、missing location、scalar-kind、interpolation 与 sampling mismatch。限定 formatting/source-order gate 通过；Cargo、产品 WGPU/DX12、RenderDoc、PNG、CPU/GPU timing、RSS 和功耗仍未执行，所以 P0-5/M6-M8 不关闭。

### 22.2 P0-7 当前源码事实

现有 `PreparedMaterial.published=None` 已在 draw payload 建立前原子选择完整 engine-owned error proxy：洋红 custom/standard uniform、六张 fallback texture、空 material runtime 与 default standard-PBR pipeline key；previous-or-error 的 material context admission 也已有覆盖。因此“尚未实现 error material+binding atomic switch”已过时，不应再复制第二套 fallback owner。

剩余问题是 fallback PSO 没有显式 startup-ready deadline，以及共享 renderer 跨不同 viewport/graph/quality/fog/geometry context 时缺少 exact context identity 来选择 current 或 last-good generation。fallback PSO 自身 Deferred 会明确 `DeferDraw`，不会静默发布半套材质，但不能据此宣称首帧 error proxy 必然可绘制。`context_admission_material_ids` 的驻留/清理也没有 profile 证据。本轮不做盲目 scan 删除、LRU 或固定容量；P0-7 保持开放，后续顺序固定为 bootstrap required-PSO set/deadline -> context identity + generation pin -> 1/100/10,000 variant profile -> 再决定有界 retirement。

## 23. 2026-08-27 Error proxy actual-context PSO 闭环

状态：**source closure implemented；managed compile/product/profile pending**。本节收敛 §22.2 已定位的 fallback 自身 Deferred 缺口，不声称跨 context generation/retirement 已完成。

### 23.1 根因与结构修复

旧 current/previous context admission 在两代都不可用时选择完整 error proxy，但没有为这个第三层结果建立 requirement census；后续 mesh pass 仍可能因 default PSO 未 Ready 返回 `DeferDraw`。cold candidate 还有隐式形式：`published=None` 已投影 error proxy，而稀疏选择枚举仍是默认 `Published`，只查显式 override 会漏掉首代等待期。

现在 context admission 增加唯一终端第三层：按最终 proxy `.runtime().is_none()` 判断显式/隐式 error proxy，使用与真实材质同一个 requirement builder，把当前 graph feature、actual geometry、quality、fog、depth/shadow/velocity eligibility 投影成跨材质去重集合。默认 proxy 是 opaque Standard-PBR、cast/receive shadow、无 disabled pass、reactive=0，因此不会产生 OIT、TAA reactive 或 alpha-mask shadow 变体。稳定帧先以 O(1) error counter 和既有 `has_active_staged_material_candidates()` index 退出：前者覆盖 context admission 失败后显式写入的 `ErrorProxy`，后者覆盖 cold `published=None` 的隐式 fallback。复核先删除不存在的推测 API `material_pipeline_publication_required()`，又拒绝了虽可编译但过宽的 `has_material_pipeline_admission_work()`；后者包含当前尚未退休的 context row，会把额外 census 永久留在 O(D) 路径。

该集合走 synchronous fallback admission：Base 明确绕过 background defer并可收敛已有 async target；Naga source state 最多三次推进，覆盖 queue-full -> queued -> Ready；每个 Ready variant 完成 retained WGPU error-scope diagnostic。terminal 或三次后仍 Deferred 直接转换为 `build_mesh_draws` 的 `GraphicsError`，不允许继续寻找第四套 fallback 或静默丢 draw。

### 23.2 Unreal 对照与算法边界

Unreal `Material.cpp` 在 default materials 全部初始化后才调用 `PrecacheFallbackMaterialPSOs`，并以高优先级覆盖 vertex factory/mobility/reverse-culling domain。Zircon 已有 environment-only Base 的同步/后台 warmup，但标准 renderer 拥有四个 builtin geometry family 与 plugin domain，启动时复制全 pass x geometry x quality 会产生编译风暴。本轮采用 actual-context lazy deadline：renderer startup 保留最小 profile warmup，只有真实落到 error proxy 的 draw 才同步保证其 exact set。

稳定新增成本是 O(1)；异常/publication context 为 `O(D + R)`，D 是 pending draws，R 是去重后的 fallback requirement，缺失项才产生 Naga/module/PSO/WGPU 工作。TDD RED 为独立 harness 缺少 `has_error_proxies`、source gate 4 defects、推测 API 未满足，以及 broad context gate 会保留 O(D) 路径；GREEN 稀疏选择 `1/1`、source closure `7/7`、requirements/publication 最小类型编译 `2/2`，限定 formatting 与 file budget 通过。没有 Cargo、managed product compile、WGPU/DX12 frame、PNG、`D:\Tools\renderdoc`、timing、RSS、能耗或功耗证据。P0-7 仅关闭“error proxy requirement 未准入”源码缺口，跨 viewport/context generation identity、context admission row retirement、P0-4 与 M6-M8 继续开放。

## 24. 2026-08-27 Material generation exact-requirement ledger

状态：**identity infrastructure implemented；persistent census retirement / managed validation pending**。本节先建立跨 context 优化所需的正确 identity 和 owner，不把缓存接线写成已经消除 frame bottleneck。

### 24.1 结构复核与 owner 决策

把 readiness 绑定 viewport ID 的方案被否决。`ViewportRenderFrame` 当前没有 viewport handle；即使扩散 runtime/framework API，同一 viewport 后续新增 geometry、pass、quality 或 fog 组合仍会让“viewport 已 Ready”成为 false-ready。Unreal `MaterialRenderProxy.cpp:865` 的 `GetMaterialWithFallback` 只接受 rendering-thread shader map complete 的 material；`MaterialShader.cpp:3410` 的 `FMaterialShaderMap::IsComplete` 对 shader/pipeline 与 vertex factory layout 做完整成员校验，`Material.cpp:2832` 的 PSO precache 另行处理。Zircon 使用 lazy exact PSO，因此等价 owner 应是 `MeshPipelineCache` 的 material-generation artifact ledger，而不是 ResourceStreamer 或 viewport cache。

当前实现按 `ResourceId × draw_generation × MaterialPipelineRequirement` 保存 Ready。requirement 保留 exact target、完整 PipelineKey、geometry source、quality；fog 已进入 PipelineKey，不用 64-bit fingerprint 代替相等性。ResourceStreamer 仅投影 `[published, previous_published, staged_candidate]` 三个 live generation，每次 ledger 访问先按这个三元组裁剪，不引入 LRU、时间阈值或 generation 数值排序。staged coordinator、current context、previous context 都传实际 generation；只有完整 requirement set 返回 `Ready` 才取并集写入，Deferred/Failed 的 partial-ready 不会成为 generation complete。无 generation 的 error proxy 继续走独立同步终止路径。

### 24.2 算法规模、观测面与剩余边界

单 material 裁剪最多检查 3 个 live generation；exact set lookup/record 为 `O(R)`，总驻留上界为 `O(M * 3 * R)`。新增 counters 为 `material_generation_admission_cache_hit/miss`、retained material/generation/requirement count。重复 camera-stack/context 对同一 exact set 可跳过 variant resolve 与 PSO ensure。

旧 `context_admission_material_ids` 仍未退休，因此当前帧路径依然可能做 persistent `O(D)` census；本节不声称 CPU、功耗或 RSS 已改善。下一顺序固定为：把 exact requirement discovery 融入已有 pending-draw traversal -> 用 ledger miss 生成稀疏 census -> 删除 material-level 永驻 row -> 为未来 PSO eviction 建立 requirement invalidation -> 再测 1/100/10,000 materials。TDD RED 为 ledger type 缺失；GREEN E 盘 ledger harness `1/1`、publication 类型 harness 通过，source-order gate 锁定 prune-before-lookup 与 Ready-before-record。Cargo/product/WGPU/DX12/PNG/RenderDoc/timing/RSS/能耗/功耗均未运行，P0-4、P0-7、M6-M8 保持开放。

## 25. 2026-08-27 Pending-draw census 融合与永驻 owner 退休

状态：**source structural optimization implemented；managed compile/product/profile pending**。本节完成 §24 已冻结的下一步，但不把源码复杂度改进写成实测帧耗或功耗收益。

### 25.1 根因与硬切

`context_admission_material_ids` 在 staged candidate 发布时插入，却没有“该 generation 在所有未来 context 已完整”的可定义退休事件。只要任一材质发布过，后续稳定帧就永久对完整 `pending_draws` 再做一次 requirement census；直接在没有 exact generation ledger 时删除它又会漏掉后来出现的 geometry/pass/quality/fog requirement。

现在 current published census 融入 pending-draw collector：每个 mesh instance 扩展后只观察本次 append range，在数据仍热时累计 exact requirement。`PendingMaterialDraw` 显式保留未哈希的 `draw_generation`；cold error proxy 为 `None`，不会混入 current-generation census。requirement inputs 直接来自已投影 pending material，不再对每个 draw 回查 ResourceStreamer material HashMap/proxy；shadow eligibility 同时改为消费 renderer/material 合并后的 effective `CastShadowsMode`，不再为 renderer 已关闭的 shadow 过度准入 PSO。

context admission 接收显式 census，先按 `[published, previous, staged]` 裁剪 ledger，再删除 exact Ready 行，仅对 generation miss 做 variant resolve/PSO ensure。ledger hit 保持默认 Published；miss Deferred/Failed 仍按完整 PreviousPublished -> synchronous ErrorProxy 顺序回退。ResourceStreamer 的字段、构造、publication insert、accessor、replacement cleanup 和 submission-failure cleanup 同一 snapshot 硬删除，production 永驻 owner 引用为 0。

### 25.2 规模与性能门

正确复杂度不是“稳定帧 O(1)”。pending collection 本身为 `O(D)`；融合 observer 仍需为每个 draw 投影并去重最多 `P` 个 requirement，增量为 `O(D * P)`。对 `M` 个 observed material 的 generation prune/exact lookup 为 `O(M * (3 + R))`。本轮删除的是后置 whole-vector traversal、per-draw ResourceStreamer lookup、无界 material-ID owner，以及 exact Ready 后的 variant resolve/PSO admission，而不是删除所有 draw-context 判断。

后续 profile 必须分别记录 collector base、fused observer、ledger probe、miss admission 的 p50/p95/p99，以及 1/100/10,000 materials 下 draw count、unique requirement、hit/miss、PSO create、RSS 和能耗。若 observer/dedup 常数仍主导，再建立 generation-qualified compact draw-context index；该 index 必须保留 exact requirement 可验证映射，不能用有碰撞 fingerprint 替代 identity。P0-4 eviction 仍必须与 ledger invalidation 同事务完成。

TDD 初次结构门 `3/3` RED；去除 per-draw proxy 的第二轮为 `2/4` RED；最终 E 盘 source harness `4/4` GREEN。限定 formatting、owner inventory 与 diff integrity 通过。Windows managed validation 只提交一次，`cargo.acquire` 已 accepted，但 request `407a3c26779543bda7bad9163125c9e1` 在 post-response reconciliation 返回 `command_post_timeout`；它不是 compile ticket，未重试或轮询。产品 WGPU/DX12、PNG、`D:\Tools\renderdoc`、CPU/GPU timing、RSS、能耗与功耗均未采集。P0-7 的 generation identity/永驻 census 源码缺口已关闭但尚未验收，P0-4 与 M6-M8 继续开放。

生命周期复核没有支持现在就添加 ledger 反向索引。七条同步 Mesh PSO 创建路径均在返回 Ready 前 `block_on(error_scope.pop)`，失败时立即 drain 并删除 target；async Base worker 同样在返回可安装 product 前解析 error scope。当前也没有容量 eviction API，因此 diagnostic 不会在 generation Ready 记录之后异步删除 PSO。反向 material-generation edge 仍是未来 P0-4 retirement transaction 的硬前置，但应与真实 removal consumer 同一里程碑实现，不能先留下无消费者的维护状态。

同一 `census.retain` 现同步累计 material hit/miss 与 requirement observed/hit/miss，并只在帧末发布 counters；没有第二次 census，也没有 per-draw profile scope。后续 profiling build 应以同场景、同 draw/material/graph/quality/fog 输入比较 combined collector 的前后版本，并用 requirement hit/miss 解释 PSO admission 工作量；不要在每个 mesh callback 内加计时而污染热点。

## 26. 2026-08-27 Requirement census 算法收敛

状态：**source structural implementation complete；managed compile/product profile pending**。§25 消除了第二次 draw-vector traversal，但 fused observer 仍在每个 draw 上克隆完整 `PipelineKey`、枚举 pass 并哈希相同 requirement，最后才由 requirement set 去重。generation ledger 只能跳过后续 variant resolve/PSO admission，不能消除 collector 内部的重复构造。对同材质、同 geometry ABI 的大量实例，旧实现仍为 `O(D * P)` 次重对象构造，其中 `D` 为 observed draw 数、`P` 为该 graph 下最多七类 requirement。

实际 draw-to-PSO 投影已逐字段复核。requirement identity 只消费 exact material/generation owner、最终 shader geometry source、dynamic mobility 对 velocity eligibility 的布尔投影、effective cast-shadows、graph feature、quality、fog 和 material pipeline inputs。`uses_indirect_draw`、mesh LOD、instance/entity/mesh identity、visibility 与 renderer `receive_shadows` 不改变这里创建的 PSO key；后者是 GPUScene material uniform 数据，而实际 `PipelineKey` 保持 material authored feature。不得用 hash fingerprint 代替这些 typed fields 的相等性。

实现使用 exact `(ResourceId, draw_generation)` census owner，并把 candidate、context set 与 requirement set 收敛到同一 row；current 每 draw 只做一次 owner lookup，admission 直接消费 row generation，不再回查 proxy 猜代次。staged/previous 每 material 只解析一次 candidate；previous selection 没有 `PreviousPublished` row 时以 O(1) gate 返回；error proxy 跨 material 共享 context set。2026-08-27 shadow raster identity 修复后，实际 context 域为四个现有 shader geometry family × velocity eligibility ×三态 shadow policy 共 24 项，使用 typed enum 到 `u32` bitset 的双射，零 heap、无碰撞，不以 fingerprint 代替 equality。复杂度保持 `O(D + U * P + M)`，临时状态为 `O(M + R)`，且 current `U <= 24M`、error proxy `U <= 24`。

每个 census scope 末尾从 slice/cache/set 的 `len()` 发布 observed draw、unique context 与 candidate resolution 数，热循环没有为 profile 增加计数递增。shadow 扩维前的 E 盘 release rustc 等价工作量基线用 9 个中位样本并按场景重复执行，逐场景先断言新旧 exact requirement 集合相等：10,000 相同实例将 full-key clone `90,000 -> 9`、requirement hash `70,000 -> 7`，模型中位数 `16.563 ms -> 0.436 ms`（38.0x）；100 material/10,000 draws 为 `17.841 -> 0.757 ms`（23.6x）；10,000 material 各 1 draw 为 `50.272 -> 47.770 ms`（1.052x），未出现全唯一退化或超线性增长。三态扩维只改变定长 bit 身份与必要的 shadow requirement 数，但当前源码尚未重跑该模型；这些数只保留为 census 算法基线，不能替代产品帧 timing/RSS/功耗。

TDD source gate 初始新增 2 项均 RED，最终跨文件结构 `7/7`、production requirements 类型/行为 `10/10`、限定 rustfmt 通过。生产文件测试模块拆分后为 762 行。新的 Windows managed compile 执行在 124 秒无输出超时，没有 request/compile ticket；未重试或轮询。产品 WGPU/DX12、RenderDoc、PNG、RSS、WPR、能耗和功耗仍未完成，因此 P0-4、M6-M8 与 bottleneck-removal 验收结论保持开放。

## 27. 2026-08-27 Shadow raster identity 与 pass-local variant 复审

状态：**architecture reviewed；correctness repair authorized；variant convergence/profile pending**。本节先记录完整源码与参考实现结论，再实施最小 shadow cull 正确性闭环；没有 current-source 产品 profile 时，不提前做跨 pass 的大规模 variant identity 收缩。

### 27.1 当前结构性矛盾

`CastShadowsMode::TwoSided` 在 `PendingMeshDraw -> MeshDraw` 之间仍保留 typed mode，但进入 `MeshBatchRef`、pending command-cache plan/extract 与 requirement census 时被压成 `casts_shadow: bool`。因此 renderer 级“仅阴影双面”语义在 variant resolution 前永久丢失。与此同时 `create_shadow_mesh_pipeline` 使用 `wgpu::PrimitiveState::default()`；锁定的 wgpu-types 29.0.3 对该结构派生 `Default`，其中 `cull_mode: Option<Face>` 的默认值为 `None`。结果不是保守的一面阴影，而是所有 opaque/alpha-mask shadow PSO 都禁用背面剔除。

这同时造成正确性与性能边界倒置：`On` 的单面材质错误地以双面投影，增加 shadow atlas raster/early-depth 工作；`TwoSided` 虽然碰巧得到无剔除，却没有可追踪的 PSO identity。现有 `PipelineKey.double_sided` 已进入 registry/WGSL feature identity，但 shadow raster state 完全不读取该 key；material 双面与 renderer shadow-only 双面也没有合并成最终 shadow policy。

Unreal 的边界可直接迁移而不是凭经验推断：`SetupShadowCullMode` 先计算 `Material.IsTwoSided() || PrimitiveSceneProxy->CastsShadowAsTwoSided()`，双面时选择 `CM_None`，否则保留或按 shadow view 类型反转 mesh cull；同一最终 cull mode 进入实时 draw 与 PSO precache。Zircon 当前没有 one-pass point/VSM reverse-cull 分支，因此 MVP 的 exact rule 冻结为 `effective_shadow_two_sided = material.double_sided || renderer.cast_shadows == TwoSided`；为真使用 `None`，否则使用 `Back`。`Off`/material cast gate 继续决定是否存在 shadow command，`ShadowsOnly` 只影响 main-view routing，不应隐式变成双面。

### 27.2 Variant 身份过宽的独立问题

`MeshPipelineVariantKey` 当前对所有 pass 保存几乎完整的 `PipelineKey`，而 pass template 的真实依赖明显更窄。opaque Shadow/Depth/Velocity template 不要求 material surface；alpha-mask 版本只要求 alpha surface；shadow raster 只额外消费最终双面状态。当前 `receive_shadows`、normal/PBR lobe、volumetric fog 等差异仍可为 Shadow 注册独立 variant，即使对应 pass 不消费这些功能。该设计可能同时放大 registry rows、source assembly/hash、Naga/module/PSO create 和 resident cache。

本轮不直接清空这些字段。custom Surface 的 material options/includes 可能改变 alpha，未来 material vertex deformation 也会改变 opaque shadow vertex code；持久 prewarm `ShaderVariantKey`、runtime generation、shader source hash 与 WGPU PSO identity 目前共存在一个复合 key 中。没有产品 variant distribution 和 source-hash equivalence 数据时，贸然投影会把“可共享”与“必须失效”混为一体。

### 27.3 实施与 profile 门

最小正确性修复只沿已有 draw/command/requirement 链保留 shadow mode 的三态投影：no shadow、one-sided shadow、forced-two-sided shadow。candidate 的 material `double_sided` 与 context 的 forced flag 在构造 exact shadow requirement 时合并；同一最终 key 必须同时供 live command、generation ledger、prewarm/ensure 与 `create_shadow_mesh_pipeline` 使用。typed census 域因此从 `4 geometry x 2 velocity x 2 shadow eligibility = 16` 调整为 `4 x 2 x 3 shadow policy = 24`，仍使用无碰撞定长 bitset，复杂度保持 `O(D + U * P + M)`，当前 `U <= 24M`、error proxy `U <= 24`。

在做 pass-local key 收缩前，受管 current-source 产品 profile 必须按 pass 发布 registered variant、unique source hash、shader-module create、PSO create、creation CPU、resident bytes 与 draw/triangle/raster invocation；场景至少覆盖 1/100/10,000 materials，并正交变化 `double_sided`、renderer `TwoSided`、alpha mask、receive shadow、normal/PBR/fog、shader option 与 geometry family。RenderDoc 应验证单面 `On` 的 shadow pipeline 为 back-face cull、material/renderer 双面为 no-cull，并比较同画质 shadow-pass GPU duration/fragment or raster work。只有确认不同 full key 产生相同 source/PSO state且无 vertex/alpha dependency 后，才实现 pass-local identity projection；功耗未采集时必须记录 `not collected`，不能用 CPU 微基准代替。

### 27.4 Source implementation record

2026-08-27 最小正确性闭环已写入当前源码。`PendingMaterialDraw` 在 material merge 之外保留 renderer 原始 `CastShadowsMode`；requirement census 由两个 shadow boolean 含义收敛为 `Disabled / OneSided / ForcedTwoSided`，使用 24-bit-in-`u32` 的无碰撞定长身份。只在构造 Shadow requirement 时把 candidate material `double_sided` 与 renderer forced flag 合并，Base/GBuffer/Depth/Velocity key 不被 renderer shadow override 污染。静态命令缓存的 material submission revision 额外哈希最终合并后的 `CastShadowsMode`，因此 `On <-> TwoSided` 不会复用旧 shadow payload。

live `MeshDraw` 与 pending cache rebuild 都把 forced flag 投影到 `MeshBatchRef::shadow_two_sided`；`effective_shadow_pipeline_key()` 成为 shadow processor 的单一最终 key，并同时用于 variant resolution 与 command payload。ensure、runtime prewarm、prewarm validation 和 `create_shadow_mesh_pipeline` 均继续传递该 key；WGPU primitive state 对单面使用 `Back` cull，对 material/renderer 双面使用 `None`。结构 TDD 从 16 个生产缺口收敛到 `0`，shadow raster contract 通过，限定 `rustfmt --check` 与 `git diff --check` 通过。

该修复没有新增每 draw heap 或全 key 扫描，census 复杂度仍为 `O(D + U * P + M)`；它会为同一 material 的 renderer `On/TwoSided` 建立两个必要的正确 raster identity，而不是继续错误合并。旧的 shadow-irrelevant full-key variant 膨胀仍未处理。当前没有 managed Rust/WGPU compile ticket、DX12 GPU timing、RenderDoc pipeline-state replay、PNG、RSS、WPR 或功耗数据，因此本节只能记为 source implementation complete，correctness/performance acceptance 与 M6-M8 仍为 `in_progress`。

## 28. 2026-08-27 Pass-local PSO profile attribution 设计

状态：**source measurement infrastructure implemented；managed compile/product profile pending**。本节是 §27.3 的测量前置，不授权 pass-local key 投影，也不把累计计数写成性能收益。

当前 `ShaderVariantMissReport` 只发布全局 registered variant、cached object、shader-module create、render-pipeline create 与 creation CPU 总量。它能证明总体 compile/create 是否增长，却不能回答增长属于 Base、ShadowDepth、ShadowDepthAlphaMask、GBuffer、Depth、Velocity、TAA、HitProxy 还是 OIT；因此无法区分 Shadow `On/TwoSided` 的必要两态与 receive-shadow/normal/PBR/fog 等无关维度造成的膨胀。每帧扫描 `variant_keys`、pipeline HashMap 或 shader module table 会把观测成本变成 `O(V + P + S)`，并污染正要测量的稳定帧。

实现边界冻结如下：

1. 公共诊断使用固定十类 target：Base、GBuffer、DepthPrepass、HitProxy、ShadowDepth、ShadowDepthAlphaMask、Velocity、TaaReactiveMask、TaaReactiveMaterialMask、OIT。分类不以自由字符串或 consumer 名称作为身份。
2. registry 只在新 variant 插入时递增 target registered count；source hash 只在 exact target 的 source 已组装时做一次集合插入；shader-module/render-pipeline create 在实际 WGPU 调用完成后记录 target、次数和 CPU microseconds。cache hit/draw/replay 热路径不加锁、不分配、不计时。
3. 报告读取复制十个定长 POD row，复杂度 `O(10)`，不扫描 registry、pipeline cache 或 module table。跨 camera/report 聚合对累计快照取逐字段最大值，避免把同一共享 renderer 的累计值重复相加。
4. WGPU 不公开可靠的 PSO/module resident bytes；本层只报告对象数、唯一 source hash 与累计 creation cost。resident bytes、RSS、driver allocation、GPU raster invocation 和功耗必须由 WPR/ETW、RenderDoc/PIX 与进程级采样提供，禁止用 Rust struct 大小或 WGSL 字节数伪装。

受管产品 profile 必须对 1/100/10,000 materials 正交变化 `double_sided`、renderer `TwoSided`、alpha mask、receive shadow、normal/PBR/fog、shader option 与 geometry family，并至少记录三次 cold/warm run。Shadow 优化只有在以下条件同时成立后才允许实施：同 target 多个 full key 的 WGSL source hash 与最终 raster/depth/layout state 等价；registered variant/PSO create 与 creation CPU 显著受该重复身份驱动；RenderDoc 证明必要的 one-sided/two-sided cull state 仍精确；key 投影后 draw、triangle、raster work 和图像不退化。否则保留完整 identity，继续查找真实瓶颈。

### 28.1 Source implementation record

固定 target ABI、累计 row 与诊断导出已进入当前源码。`ShaderPipelineTarget` 明确定义十类稳定 identity；每行同时发布 registered variant、unique source hash、shader-module/render-pipeline 实际创建次数及其 CPU microseconds。旧 JSON 缺少该字段时默认全零，跨共享 renderer snapshot 的合并以 `(count, time)` coherent snapshot 最大值为准，避免同一 lifetime 累计值被 camera/report 重复相加。

`MeshPipelineVariantRegistry` 在新 exact variant 插入时以定长十项数组递增 target count，report/reset 只复制该数组，不重新扫描 variant key。source hash owner 使用 target 分区的集合，仅首次观察分配字符串；重复 source 先 borrowed lookup。Base 同步/异步、GBuffer、DepthPrepass、HitProxy、Shadow 两类、Velocity、TAA 两类与 OIT 的实际 module/PSO 创建点都携带 exact target。预热共享 primary/companion module 只向实际创建 module 的 primary 记账，companion 记录 source 与自身 PSO；OIT 使用 fragment-store 变换后的 source hash，因而 global object totals 与 target object totals 不会重复计费。

诊断层使用编译期静态路径发布 `10 targets x 6 metrics = 60` 条 series；每次 report 为固定 `O(10)` POD copy/iteration，不做路径格式化，也不扫描 registry、pipeline cache 或 shader-module table。已安装 PSO 的 draw/cache-hit/replay 路径不进入 source 集合或 creation timer。结构 TDD 从 15 个缺口收敛为 PASS，新增测试覆盖 legacy serde、固定 target 顺序、coherent snapshot 合并、source 去重、registry reset 后 target count 以及静态诊断路径；限定 `rustfmt` 与 scoped diff integrity 通过。

这只是让下一轮产品 profile 可归因。当前没有 managed Rust/WGPU compile ticket、1/100/10,000-material 产品数据、DX12 GPU timing、resident bytes、RSS/WPR、`D:\Tools\renderdoc` capture、PNG、能耗或功耗证据；因此不声明 shadow identity 膨胀已经是产品瓶颈，也不实施 pass-local key 投影，P0-4/P0-5 与 M6-M8 保持开放。

## 29. 2026-08-27 Mesh Vertex Factory 输入 ABI 准入复审

状态：**source admission implemented；managed compile/product validation pending**。本节继续完成 P0-5 的基础正确性，不属于 pass-local key 优化，也不授权任意 geometry vertex layout。源码清单显示 8 个 Mesh render-pipeline creator 全部直接使用生产 `GpuMeshVertex::layout()`；只有 Velocity 再增加 `GpuMeshVertex::previous_position_layout()` 的 location 8。`GeometrySourceDescriptor.vertex_attributes` 与 `required_bindings` 目前只参与描述/注册校验，没有生产 PSO vertex declaration 消费者；custom geometry include 也只替换 fetch helper，不能替换模板 `ZrVertexInput`。因此当前 MVP 的真实架构是固定 Mesh Vertex Factory ABI，而不是插件自定义 vertex layout。

Unreal 的 `FLocalVertexFactory` 由同一 owner 生成运行时 `GetVertexElements`/`InitDeclaration` 和 `GetPSOPrecacheVertexFetchElements`，避免预热 identity 与真实 vertex declaration 分裂。锁定的 `wgpu-core 29.0.3` 在 render-pipeline validation 中先把每个 `VertexAttribute` 按 location 投影为 numeric type，再要求 shader vertex input location 存在；其 vertex-stage兼容规则只比较 scalar kind，向量维数和 scalar width 不参与该判断，额外 vertex attribute 合法。Zircon 的 admission 必须精确复制这一语义，不能用 type-layout hash 或 exact vector-size 比较制造 backend 不存在的拒绝。

实现新增独立的 neutral/WGPU 两层 vertex contract。renderer construction 从实际 `GpuMeshVertex` layouts 一次性生成 standard 8-row 与 Velocity 9-row 两个排序契约，并拒绝重复 location；target selector 只为 Velocity 选择扩展契约。异步 reflection Ready 与 cached ShaderModule admission 都在 disk publication/WGPU PSO 创建前验证 exact vertex entry：缺失 location、unsupported scalar kind、scalar-kind mismatch 或 vertex `per_primitive` fail closed；builtins 与额外 layout attributes 不参与拒绝。错误仍进入既有 `ShaderInterfaceMismatch` owner，没有第二套 failure lifecycle。未来若支持真正的插件 vertex declaration，必须让 geometry/vertex-factory owner同时生成实际 WGPU layout、shader include、prewarm identity 与 admission contract，不能继续把未消费的 metadata 当作 authority。

构造期只分配两个小 `Vec`，合计 17 个 attribute row；每个新 source/module target admission 对最多 `I<=9` 个 shader input 做排序数组二分，复杂度 `O(I log A)`、`A=8/9`。已安装 pipeline 的 draw/cache-hit/replay 新增成本为 0，不增加 Naga parse、worker、I/O、WGPU object 或 frame 全表扫描。E 盘独立 release `rustc` harness 已验证 production WGPU projection 的类型、生命周期、排序 lookup 和 Float/Uint 映射；结构契约 `7/7`、限定 `rustfmt` 通过。受管 focused 验证在 124 秒内没有产生任何输出并由调用侧超时终止，因此没有 compile/test ticket，不能计为 Cargo/Rust/WGPU 通过。当前也未运行 DX12 frame、PNG、`D:\Tools\renderdoc`、CPU/GPU timing、RSS/WPR、能耗或功耗，因此这里只关闭 vertex-input 晚失败的源码入口，P0-5 与 M6-M8 继续开放。

## 30. 2026-08-27 异步 Naga 源验证重复工作测量面

状态：**measurement source implemented；validation identity unchanged；managed/product profile pending**。本轮遵守“先 profile、后优化”的门禁，只补齐 source assembly 后异步 Naga parse/validate/reflection 的累计归因，不收窄 `ShaderSourceValidationKey`，也不把隔离集合基准当成产品瓶颈已经成立。

### 30.1 当前 owner 与结构性假设

`ShaderSourceValidationKey` 当前由完整 `ShaderVariantKey + validation_source_identity` 组成；后者已经包含 exact WGSL source hash 与诊断 segment provenance。worker 的实际结果只由 WGSL 与 segments 决定，target-specific entry/resource/vertex/stage-link contract 在 reflection Ready 后另行验证。Ready reflection 又会在 ShaderModule 安装后由 `take_ready_shader_source_validation` 移除，因此不同 variant key 若生成同一 source/provenance，不能复用已完成的 Naga 结果，理论上可能重复 queue、parse、validate 与 reflection。

这只是需要验证的结构性假设。variant-qualified key 仍原样保留，因为产品场景尚未证明重复比例、Naga CPU 占比、失败诊断重映射与热重载行为；在这些数据之前把 key 改成 source-only 会把正确性风险伪装成优化。Unreal 的 shader map/DDC 思路同样要求编译产物身份与材质/PSO 身份分层，而不是让完整 PSO permutation 永久绑住可共享的 source compiler work。

### 30.2 新测量合同与开销边界

`MeshPipelineCreationMetrics` 现在累计四类 queue outcome、实际 worker job、unique source contract、duplicate job、success/failure、queue wait microseconds 与 Naga validation CPU microseconds。worker 另有 `render/shader_pipeline/source_validation_worker` profile scope，可由产品 profiler读取 sample p50/p95/p99；报告和 `DiagnosticStore` 发布 `render.shader_variant.source_validation.*` 11 条稳定 series。报告合并选择同一 lifetime 内较新的 coherent cumulative snapshot，避免共享 renderer 被多次读取时重复相加。

重复判定只在 worker 真正启动时以 borrowed source-identity lookup 查询集合；仅首次观察新 source contract 才复制字符串。测量状态为 `O(S)`，S 是 renderer lifetime 内 unique source/provenance 数；每个实际 validation job 增加一次 mutex、hash lookup 和两次计时，已安装 pipeline 的 draw/cache-hit/replay 路径新增成本为 0。queue result 计数发生在 admission owner，因而 `queued/already_pending/full/worker_unavailable` 与实际 job 数可以解释队列背压；worker panic 会表现为 started job 未进入 success/failure，不能被误记成 Naga failure。

E 盘 release-rustc 身份集合下界基准使用 21 个 sample 的 median，只比较 key cardinality/hash/allocation，不执行 Naga。100 请求共享 1 个源合同时，当前 variant-qualified 集合为 100 row、source-contract 集合为 1 row，`16.491 us` 对 `13.956 us`。10,000 请求共享 1 个源合同时为 10,000 对 1，`3.948 ms` 对 `1.720 ms`；10,000 请求/100 个源为 10,000 对 100，`4.258 ms` 对 `1.805 ms`；10,000 个全唯一源时两者均为 10,000，`3.793 ms` 对 `3.380 ms`。这些数字只说明重复 identity 具备可测的状态下界，不包含 Naga、source assembly、线程调度、WGPU、RSS 或功耗，也不授权 source-only key。

后续产品 profile 必须在同一 DX12 场景下分别跑 1/100/10,000 materials 的 cold/warm 至少三次，记录 `job_count`、`unique_source_count`、`duplicate_job_count`、queue outcomes、queue wait、validation CPU scope p50/p95/p99、source assembly、module/PSO creation、frame CPU/GPU、RSS/WPR 与功耗。只有当 duplicate job 与 Naga CPU 构成显著 cold-start/热重载瓶颈，并且 exact diagnostics、hot reload、target contract 与 cache invalidation 测试均通过，才允许把 compiler-result identity 收敛到 source/provenance owner。结构契约 `7/7` 与限定 `rustfmt --check` 已通过。唯一一次受管 focused 请求 `34fd4b6732114735a6cfcc7045b0a231` 已 accepted，但 `cargo.acquire` 在 post-response reconciliation 返回 `command_post_timeout`，没有 compile/test ticket，未重试或轮询。产品 WGPU/DX12、RenderDoc、PNG、RSS/WPR、能耗和功耗仍未执行，P0-5、P0-4 与 M6-M8 保持开放。

## 31. 2026-08-27 Mesh fragment-output / pass attachment ABI 准入

状态：**source admission implemented；managed compile/product validation pending**。本节继续关闭 P0-5 的 WGPU 晚失败入口，不改变材质、PBR/IBL 或 attachment 算法，也不把前置拒绝写成性能收益。

### 31.1 Backend 与参考引擎规则

锁定的 `wgpu-core 29.0.3` 在 fragment stage link 完成后遍历 shader 实际输出；只有同 location 存在 `Some(ColorTargetState)` 时才调用 `check_texture_format`。兼容方向是 target numeric type 必须是 shader output 的 subtype：scalar kind 相同、target scalar width 不大于 shader、target component count 不大于 shader vector。shader 的额外输出被忽略，attachment 没有对应 shader 输出也合法。Velocity 的 `vec4<f32>` 写 `Rg16Float` 因而正确；不能误改成 exact vector-size 或“必须写满 MRT”的更严格规则。

Unreal 的 `RenderCore/Private/ShaderMaterialDerivedHelpers.cpp` 先从 pass/material 条件推导 `PIXELSHADEROUTPUT_MRT0..6`，`RenderCore/Private/GBufferInfo.cpp` 再由 GBuffer owner定义语义到目标/通道的 packing。对应到 Zircon，输出反射属于 compiler artifact，目标 numeric shape 属于真实 pass attachment owner；二者只在 PSO publication admission 相交，不能把完整 attachment 表重新塞进 authored shader DTO。

### 31.2 实施边界与算法规模

反射层新增后端无关 fragment numeric contract，并精确复制 WGPU subtype 方向。Mesh cache construction 从真实 Base target format、GBuffer 4 个已有常量、HitProxy 3 个已有常量、Velocity 与 TAA 的唯一格式常量生成 6 份常驻合同；DepthPrepass、两类 Shadow 与 OIT 共享 empty color contract。Velocity/TAA 创建 API 同时删除可漂移的 format 参数，runtime、prewarm 与 WGPU validation 都只能使用各自 owner 常量。

Ready source 与 cached ShaderModule 在同一个 `validate_reflection_shader_contract` 中依次执行 entry、vertex input、vertex/fragment link、fragment output、resource/sampling ABI；fragment mismatch 在 disk publication 与 WGPU module/PSO 创建前进入既有 `ShaderInterfaceMismatch` failure owner。构造期共保存 10 个 `(location,numeric type)` row，排序成本 `O(T log T)`；每个新 fragment entry 最多按输出数 `O(O log T)` 查找，当前 `O<=4`、`T<=4`。已安装 pipeline 的 draw/cache-hit/replay 路径不访问该合同，不新增 Naga、I/O、worker、WGPU object 或 frame scan。

TDD 先得到 reflection API 缺失的结构 RED；GREEN 新增用例锁定宽输出覆盖窄目标、uint/float mismatch 和未配对 location 合法。E 盘 release-rustc harness 直接编译两份生产 contract 文件并通过 HitProxy、Velocity 与 empty OIT 场景；锁定 wgpu-core 规则与生产 owner 的结构门 `9/9`、限定 `rustfmt --check` 通过。复核还修正了 `MeshPipelineCache` worker 字段缺少 `ShaderTemplateReflection` 导入的潜在编译错误。唯一一次受管 focused 请求 `8f3403a7fa914865bdd54af59707754b` 已 accepted，但 `cargo.acquire` 在 post-response reconciliation 返回 `command_post_timeout`，没有 compile/test ticket，未重试或轮询。Cargo/Naga 单元、产品 WGPU/DX12、PNG、`D:\Tools\renderdoc`、CPU/GPU timing、RSS/WPR、能耗与功耗仍未执行，因此 P0-5 与 M6-M8 保持开放。

## 32. 2026-08-27 GPU Scene dynamic palette minimum 的 ABI owner 修复

状态：**false-negative source admission repaired；managed compile/product validation pending**。fragment-output 切片后的 P0-5 全字段复核发现，Mesh resource contract 为了重建不可从 `wgpu::BindGroupLayout` 反射的 group 3 描述，调用 `gpu_scene_bind_group_layout_entries` 时向两个 skinned palette binding 传入了构造占位 `Some(1)`；projection 随后却把该值解释为真实显式 minimum。生产 WGSL 的 bindings 3/4 均为 `array<mat4x4<f32>>`，反射最小尺寸至少是一个 64-byte stride，因此 skinning 与 previous-skinning entry 会被错误报告为“layout 只有 1 byte”，合法 shader 在 WGPU 前 false-negative。

锁定 WGPU 的正确所有权是：`min_binding_size: Some(n)` 才允许在 pipeline creation 时用 n 对拍 shader minimum；`None` 把实际 buffer range 校验保留到 bind group / draw 使用阶段。GPU Scene live layout 的 palette minimum 来自真正的 palette capacity owner，Mesh cache 只持有已创建的 opaque layout，不能伪造该容量。修复只对 projection 副本中的 `GPU_SCENE_SKINNED_JOINT_PALETTE_BINDING` 与 `GPU_SCENE_PREVIOUS_SKINNED_JOINT_PALETTE_BINDING` 清除占位 minimum；实际 layout 不变，其它 scene/material/forward/OIT 显式 minimum 仍严格校验。Unreal 同样把 `FShaderParametersMetadata` / `FRHIUniformBufferLayout` 的结构描述与 `RHICreateUniformBuffer` 的实际资源实例、RHI validation 分开；这里遵循的是 owner 分层，不复制其 API。

构造额外工作为两次在固定 12-row 数组中的查找，严格上界 24 次 binding 比较，常驻 row 数、worker/Naga/WGPU object 数不变；新 source admission、已安装 pipeline、draw/cache-hit/replay 的新增成本均为 0。TDD source gate 先以缺少 late-bound projection owner 为 RED，修复后 `4/4`；源码单元回归锁定两个 binding 为 `None`。E 盘 release-rustc harness 直接复用生产 `MeshShaderPipelineLayoutContract`，证明 `Some(1)` 会拒绝 64-byte requirement，而 `None` 正确通过前置 ABI 并把容量交回 WGPU；限定 `rustfmt --check` 通过。该修复发生在受管请求 `8f3403a7fa914865bdd54af59707754b` 之后，且该请求本身没有 compile/test ticket，因此不能覆盖本快照；未追加请求、未轮询。Cargo/WGPU/DX12、PNG、RenderDoc、timing、RSS/WPR、能耗与功耗均仍待托管验收，P0-5 与 M6-M8 保持开放。

## 33. 2026-08-27 P0-7 context 结论纠正与 fallback queue-debt 观测

状态：**architecture re-audited；measurement source implemented；scheduler optimization pending product profile**。本节取代 §22.2、§23 中“仍缺 persistent viewport/context identity”的旧开放项；它不取代 P0-4 的 PSO resident retirement，也不把新增 counter 写成性能优化。

### 33.1 不再新增 context ID

当前 draw selection 不是按 material-level Ready 位发布。pending-draw collector 从实际 geometry、mobility/velocity、shadow policy 和 graph feature 构造 typed context；requirement 保留 exact target、完整 `PipelineKey`、geometry source 与 quality，fog 在构造时写入 `PipelineKey`。`MeshPipelineCache` 再以 `ResourceId x draw_generation x exact requirement` 保存 complete-Ready ledger，并只保留 ResourceStreamer 的 published、previous-published、staged 三个 live generation。后续 viewport 出现不同 pass、geometry、quality 或 fog 时会产生新的 exact requirement miss，重新走 current -> complete previous bundle -> complete error proxy，而不会复用另一个 viewport 的粗粒度 Ready 结论。`PublishedMaterialDrawProxy` 又从同一个 immutable bundle 一次投影 runtime、两套 uniform、纹理与 generation，不存在把新 PSO 与旧 binding 拼接的入口。

因此独立 persistent viewport/context ID 既不能增加正确性，也会复制 exact requirement owner。未来若产品 profile 证明 typed census/dedup 仍占主导，可以建立 generation-qualified compact context index；它只能是 exact requirement 的可验证加速索引，不能成为第二套 publication identity。P0-7 的 cross-context generation-selection source 缺口据此纠正为已关闭，剩余项是 current-source 受管产品验证和 fallback 启动调度，而非再造 context registry。

### 33.2 真正的启动调度欠债

错误代理的 exact requirement admission 最多尝试三次不是任意 magic retry：bounded source-validation FIFO 饱和时，最坏阶段为 `Full -> drain`、`Queued -> finish own job`、`Ready`。该次数保证 correctness 收敛，但 `finish_pending_shader_source_validations()` 会等待并回收 worker 的全部 pending job；一次 fallback draw 因而可能替其它无关变体支付最多 64 个 Naga 作业的队列债务。通用 worker 已有“同步到目标”的 FIFO primitive，但目标在 queue-full 阶段尚未入队，单纯替换调用仍不能建立 required-job priority 或 reservation。

Unreal 的 owner 分离支持这个判断。`MaterialRenderProxy.cpp:865` 的 `GetMaterialWithFallback` 只在 shader map 完整时选择对应 proxy，缺失 material 提交自己的 compile jobs；`PSOPrecacheMaterial.cpp` 则以 request ID 保存预缓存生命周期，通过 complete prerequisites 观察完成，并在 draw 需要时仅把相关请求提升到 Highest，而不是清空全局编译队列。Zircon 的最终方向应是统一 compile service 上的 required request priority/reservation、deadline、target wait 与 cancellation，不是 shader 模块内再启动一条专用线程，也不是在 frame path 同步执行 Naga。

### 33.3 先测量再改 scheduler

本轮只补因果观测。source-validation finish API 返回 worker 实际完成数；真实 non-empty error-proxy admission 累计并发布：

- `error_proxy_pipeline_admission_attempt_count`；
- `error_proxy_source_validation_sync_count`；
- `error_proxy_source_validation_sync_completed_count`；
- `error_proxy_source_validation_sync_wait_micros`；
- queued、pending、queue-saturated 三类同步原因数。

空 requirement 在分配统计状态和发布 counter 前返回；非 fallback 的已安装 draw/cache-hit/replay 路径新增成本为 0。fallback 事件只增加固定 7 个标量与每次已有 admission/sync 的常数次饱和加法；不扫描 queue、state、variant 或 PSO 表，不增加 worker、Naga、WGPU object、锁或 heap collection。失败返回也发布本次 debt，避免只观测成功样本。

产品 profile 冻结为同一 DX12 fixture 的 0/32/64 个既有 validation job 三档压力，分别覆盖 static/skinned geometry 与 Base/GBuffer/Depth/Shadow/Velocity 实际 graph。每档至少 30 次 cold process，关联上述 7 项、全局 source-validation queue outcomes/job/duplicate/queue-wait/Naga-CPU、error-proxy scope、first-present CPU/GPU、RSS/WPR 与能耗/功耗。只有 `sync_completed_count > 1` 与 `sync_wait_micros` 在 fallback 首帧 p95 中构成主要占比，才授权统一 service 的 priority/reservation 改造；若主要成本是目标自身 Naga/WGPU PSO，则优先补 required-domain prewarm/cook，而不是改 queue。改造后必须证明只等待目标及其 FIFO 前置/依赖、后续 unrelated job 保持 pending、queue-full 不再要求全量 drain，并比较相同画质与功耗。

TDD source gate 由缺少 5 个核心 debt series 的 RED 收敛为 `7/7`；限定 `rustfmt --check` 与 `git diff --check` 通过。没有新增 managed request：当前受管请求仍无 compile/test ticket，且本快照更新更晚。Cargo/WGPU/DX12 产品帧、PNG、`D:\Tools\renderdoc`、CPU/GPU timing、RSS/WPR、能耗与功耗均未运行，P0-7 的产品验收、P0-4 和 M6-M8 保持开放。

## 34. 2026-08-27 P0-2 runtime WGSL disk-cache 分解测量面

对 current source 的再次端到端复核确认，第14节的 source-cache 身份修复真实到达 runtime consumer，而不是只存在于离线 prewarm。Mesh 在完成 template assembly、WGSL content hash 和异步 Naga reflection 后，以完整 `ShaderVariantKey + source ID` 读取磁盘；source ID 覆盖最终 WGSL hash、按序 include content hashes、template revision 以及与 workspace lock 精确一致的 Naga/WGPU 版本。metadata 逐字段匹配，解压 payload 再次 hash 后才能命中。该边界可以排除陈旧 WGSL 的 false hit，但不能省掉 assembly、Naga、renderer-device module 或 PSO 创建。

因此当前优化假设不是“怎样让 zstd 更快”，而是“这层 runtime WGSL I/O 是否应存在”。旧 profile 只有整个 lookup/write scope 和字节 counters，无法判断 warm hit 是 metadata、payload read、decompress、rehash，还是已有 assembly/Naga/WGPU 工作占主导。生产路径现在增加 11 个固定静态 scope：disk-key 构造、写前 source hash、compress、metadata encode、payload/meta commit、metadata read/decode、payload read、decompress 与 payload rehash。它们只包住原有操作，不增加 I/O、hash、allocation、lock、worker、Naga、WGPU 或 draw-path 工作；粗粒度 lookup/write scope 和现有字节 counters 保持兼容。

下一次同 fingerprint、同 adapter/driver、E 盘 cache/work root 的受管 profile 必须将这些 scope 与 `mesh_source_build`、`source_hash`、`source_validation_worker`/`naga_validation`、分 target module/PSO creation CPU、first-present frame CPU/GPU、RSS/WPR 和功耗关联。矩阵至少为 1/100/10,000 variants 的 cold、warm、reload，各场景至少 30 个冷进程；同时记录 hit/miss/error、读取/压缩/解压字节以及 source/variant 去重率。只有 warm lookup p95 明显小于被避免的工作且 Ready/first-present/RSS 有可重复净收益，才允许优化或保留该层。若 hit 仍不能减少 Naga/module/PSO 且 disk stages 主导 p95，应在 P0-1/P0-3 共享 artifact/compile service 中删除 runtime lookup 或仅保留离线 provenance，而不是修改压缩级别。

TDD 源码门从 `0/11` 变为 `11/11`；限定 `rustfmt --check` 与 scoped `git diff --check` 通过（仅有 Git 的 LF/CRLF 提示，无 whitespace error）。没有新增 coordinator validation 请求，也没有运行 Cargo/WGPU、DX12 fixture、PNG、RenderDoc、WPR、能耗或功耗采集。P0-2 的 generated-WGSL source-cache 身份正确性可标记为 `source_closed_pending_managed_validation`；完整 artifact/target/device/PSO identity 与 runtime cache 的性能去留仍为 `in_progress`，M6-M8 状态不变。

## 35. 2026-08-27 P0-1 RHI programmable-stage 合同复审与 PMREM 公式处置

状态：**architecture contract corrected；cross-owner implementation deferred；PMREM mapping retained**。本节先修正 09A/09C 初始审查中过时的“公开 RHI 没有生产实现”结论。当前 `zr_rhi_wgpu::production::WgpuRenderDevice` 已持有真实 native shader module/pipeline registry、device generation，以及由 submission last-use 驱动的资源退役；但产品 `SceneRenderer` 仍大范围直接消费 WGPU，聚焦生产源码的 current-source inventory 仍见 48 个 `create_render_pipeline` 调用分散在 45 个文件、15 个 `create_compute_pipeline` 调用分散在 15 个文件、67 个 `create_shader_module` 调用分散在 61 个文件。P0-1 的真实缺口已从“实现不存在”收敛为“产品没有硬切到唯一 RHI pipeline authority，且现有中立合同不能表达 Mesh 的真实 programmable-stage identity”。

具体合同错误是 `ShaderModuleDesc` 同时拥有 source、`ShaderStage` 与 `entry_point`，而 `PipelineDesc` 只保存 vertex/fragment/compute shader handles。production WGPU pipeline creator只能从 module descriptor 回读 entry point。Mesh 则把同一份 assembled WGSL module交给 Base/GBuffer/Depth/HitProxy/Shadow/Velocity/TAA/OIT 等 PSO，并由每个 PSO选择不同 entry；按当前 RHI 迁移会为同一 source/compiled module按 stage/entry 建立重复 module handle/native module，也没有 pipeline compilation constants 的位置。这会把 source artifact identity、stage selection 和 PSO identity错误折叠，不能作为全 renderer hard cut 的基础。

Unreal 的 `FGraphicsPipelineStateInitializer.BoundShaderState` 与 `PipelineStateCache::GetAndOrCreateGraphicsPipelineState`/precache 保持相反的正确分层：shader artifact/RHI shader object先存在，完整 PSO initializer再选择各 stage shader和固定功能状态。Zircon 的目标合同据此冻结为：`ShaderModuleDesc` 只拥有 source/content artifact identity和reflection provenance；`ProgrammableStageDesc` 在 `PipelineDesc` 内拥有 module handle、entry point和specialization constants；最终 `PipelineId` 另外覆盖 pipeline layout、vertex declaration、attachment/depth/sample、raster/blend、device capability profile与generation。source/module去重与 exact PSO去重必须是两级身份，不能再用 entry-qualified module伪装成 pipeline cache。

当前 `zircon_runtime/crates/zr_rhi` 与 `zr_rhi_wgpu` 有大范围其它 Session tracked/untracked 修改，因此本 Session 不跨所有权改这些 descriptor。后续实现必须先由 09A owner 完成 descriptor hard cut和 production RHI tests，再让 Shader09C 的 shared artifact/compile service迁移 Mesh，最后按调用点域迁移 post/UI/particle/environment；禁止在 SceneRenderer 内增加第二个 `PipelineService` 包装 raw WGPU。

同轮 PBR/IBL 复核没有授权修改 roughness-to-mip。Unreal CPU 端把 `FloorLog2(capture size)` 作为最大 mip 索引上传，shader 使用 `max_mip - 1 - (1 - 1.2 * log2(roughness))`，所以 roughness=1有意选择倒数第三层；其 capture filter在 roughness>0.99 时也因 GGX 分布为常数而切到 cosine importance sampling。Zircon 的 canonical CPU recipe、CPU PMREM、GPU PMREM和 runtime WGSL使用同一 1.0/1.2 可逆映射及 cosine fast path。cmft 的 glossiness/specular-power按完整 mip chain映射属于另一套资产约定，不能混入当前 Unreal-aligned recipe。此处结论是保留现有算法并补计划证据，不是视觉、性能或功耗通过；managed WGPU、PNG、RenderDoc、timing、RSS/WPR与功耗仍待协调器验收，P0-1、P0-3、P0-4、P0-5和 M6-M8保持开放。

## 36. 2026-08-27 P0-5 authored pipeline-layout DTO 硬切迁移合同

状态：**inventory complete；implementation deferred for overlapping owners**。对 tracked authoring data 的复核未发现 `.zshader`、`.zmeta`、TOML 或 JSON 资产仍声明 `pipeline_layout`；`.zshader v2` 已在 importer 边界拒绝该字段。Rust 侧唯一生产读取链是 `resource_streamer_ensure_material -> renderer_material_layout_diagnostics`，空 descriptor 明确 opt-out；`ShaderAsset::pipeline_layout_descriptor()` 没有生产调用者，也没有任何 WGPU bind-group/pipeline-layout 创建从该 DTO 取值。`push_constant_ranges: Vec<String>` 同样只有 cache/readiness/测试投影，没有执行消费者。因此该 DTO 不能作为 specialized reflection 的 authored ABI authority，保留它只会维持一条默认空值和程序化测试可注入的并行合同。

删除仍不能只改 `ShaderAsset`。`ArtifactCacheShaderAsset` 以 sequential bincode 保存该字段，而 cache owner 已明确说明 skipped/default 字段不能提供 wire compatibility；当前 manifest magic/schema 为 `ZRARTM06`/v6，并在 payload 反序列化前校验版本。硬切必须在同一快照中推进 manifest magic/schema，先拒绝旧 payload，再删除 `ShaderAsset` 字段/accessor、cache payload member、readiness/management 的 layout report/count、framework DTO/export、importer/builtin 默认构造、material validator 与程序化 fixture。验收必须覆盖旧 v6 manifest fail-closed、新 cache round-trip、`.zshader v2` authored-field rejection、Surface readiness/material publication，以及实际 pass-layout 对 specialized reflection 的 WGPU admission；不能用 serde default 或读后补救维持双轨。

本轮没有实施该硬切：`shader_asset.rs`、`readiness.rs` 与 material layout validator 已有其它 Session 的未归属修改，直接编辑会覆盖正在进行的 kind/readiness 和 canonical binding-contract 工作。P0-5 的下一正确实现边界因此被冻结为上述跨模块 schema migration，而不是在 scene renderer 内再造一个 authored-layout adapter。这里没有新增运行时工作，也没有 Cargo/WGPU、PNG、RenderDoc、性能或功耗证据；P0-5 与 M6-M8 状态不变。

## 37. 2026-08-27 P0-4 live-generation resolved-PSO pin 前置

状态：**retirement safety prerequisite implemented；capacity/eviction still profile-gated**。submission ticket 与 compiled-command pin 已能证明 GPU/CPU 使用，但原 generation ledger 只保存 authored `MaterialPipelineRequirement`，没有保存它最终归一化到的 `(PipelineCreationTarget, MeshPipelineVariantId)`。未来若直接按 registry candidate 做 retirement，只能重新 resolve 或全表扫描 material generation；更严重的是无法 O(1) 证明当前、last-good previous 或 staged generation 仍引用该 exact PSO。Unreal 同样把 material shader-map completeness 与 PSO precache/residency 分层；这里没有把 generation ledger 变成第二个 pipeline registry，只补 publication owner 到 exact PSO identity 的不可丢失引用边。

`MaterialPipelineGenerationAdmission` 现在同时保存 authored requirement set 与 resolved pipeline set；完整 Ready 的 cache-miss 在既有单次 admission 遍历中顺带收集 resolved rows，partial Ready/Deferred/Failed 不发布任何 pin。同 generation 重复观察由 set 去重；跨 generation 或跨 material 共享同一 exact PSO 时由 reverse count 累加，live-generation prune 退出最后一个 owner 后才删除 pin。`PipelineCreationTarget` 保留在 key 中，因此 Base 与 OIT 即使 numeric variant ID 相同也不互相代替。新增 `material_generation_admission_pinned_pipeline_count` 直接读取 reverse-map 长度，为 1/100/10,000 material profile 提供 resident-owner 分母。

稳定 generation hit 仍只执行现有 requirement membership 检查，不创建 resolved `Vec`、不重新 resolve variant，也不增加 Naga、WGPU 或 registry scan。miss 新增一个精确按 `R` 预分配的 resolved `Vec` 和至多 `R` 次 set/map 更新；generation prune 只遍历该 material 的至多 3 个 live row 及真正退出 row 的 resolved set。空间规模为 `O(M * 3 * (R + V) + U)`，其中 `M` 是 ledger material 数、`R/V` 是每代 authored/resolved 项数、`U` 是被至少一代引用的 unique exact PSO。被 ResourceStreamer 删除且之后永不再观察的 material 当前会留下保守 stale pin；这会阻止回收而不会误驱逐，故真正 eviction 前仍必须把 material removal/streamer cleanup 事件接入同一 unpin owner。

TDD source contract 从 production owner `0/4` 转为闭包 `8/8`；E 盘 release `rustc` harness 直接 include 生产 ledger，验证 same-generation dedup、两代共享计数 `2 -> 1 -> 0`、Base/OIT target 隔离与最后一代 material-row 清理，运行通过。限定 `rustfmt --check` 与 whitespace/conflict-marker 检查通过。没有运行 Cargo、WGPU/DX12 产品、PNG、RenderDoc、RSS/WPR、能耗或功耗；没有容量阈值、LRU、tombstone 或删除操作，因此 P0-4、M6-M8 与 realtime-SH9 Failure 状态不变。

## 38. 2026-08-27 F2 environment-only dielectric F0 消费闭环

状态：**source contract implemented；managed shader assembly/product validation pending**。完整 Standard-PBR 与 basic Forward 已把 `surface.dielectric_f0` 传给 split-sum 环境 BRDF，但 `ENVIRONMENT_ONLY_PBR` 专用 include 仍在组件层固定注入 `vec3(0.04)`。这使同一材质切换到 environment-only profile 时，直接光路径被有意裁剪的同时，材质本身已经发布的 dielectric F0 也被静默丢失。Unreal 的环境 BRDF 以 GBuffer/材质 `SpecularColor` 为输入；固定 0.04 只适合作为没有该通道的兼容默认值，不能覆盖 Forward 已有的材质 ABI。cmft 负责环境卷积资产，不定义消费端材质 F0，因此这里不从 cmft 的 PMREM 约定推导材质常量。

专用 `zr_environment_pbr_components`/`zr_environment_pbr_indirect` 现在显式接收 `dielectric_f0`，Forward 传入 `surface.dielectric_f0`，共享 `zr_pbr_material_f0` 与 BRDF LUT 继续作为唯一求值 owner。environment-only deferred 的当前 GBuffer 只保存 metallic/roughness/occlusion 与 shading-model 数据，没有独立 dielectric F0 通道，所以该调用点显式传 `vec3(0.04)`；这将受限降级留在 GBuffer 边界，而不再污染可保真的 Forward 路径。若未来扩展 GBuffer，必须同时升级 encode/decode/layout 与 deferred 调用，禁止在 BRDF owner 内恢复全局硬编码。

运行时算法规模仍为每像素 `O(1)`。相对修复前新增 1 个 `vec3<f32>` 参数传递；纹理采样 `+0`、分支 `+0`、循环 `+0`、归一化 `+0`、反射探针遍历 `+0`，也不新增 shader permutation、Naga job、WGPU module/PSO 或磁盘缓存 identity。源码 RED 合同为 `0/4`，修复后为 `4/4`；两组 Rust source-contract 测试同时锁定 Forward 材质值和 Deferred 显式默认值，限定 `git diff --check` 通过。当前没有 managed shader assembly/Naga/WGPU ticket、DX12 产品帧、PNG、`D:\Tools\renderdoc` capture、寄存器/occupancy、CPU/GPU timing、RSS/WPR、能耗或功耗数据，因此 F2 仅标记为该消费缺口 `source_closed_pending_managed_validation`，M6-M8 与 realtime-SH9 Failure 保持开放。

后续路由复审没有顺手删除 non-default-IOR 的专用变体排除。当前 `MeshPipelineVariantKey::new` 仍在投影 PSO identity 前读取 `pbr_ior_override`，并用它阻止 `ENVIRONMENT_ONLY_PBR`；viewer warmup 也只在默认 IOR 时启用该 profile。因此 non-default IOR 画面继续正确走 generic Forward，但“专用 provider 只能固定 0.04”的历史理由已经消失，剩余问题变成 generic source/receiver PSO 的潜在启动冗余。相关 registry/warmup 文件已有其它未归属修改，且没有当前 generic 对 specialized 的 assembled bytes、Naga/module/PSO CPU、first-present CPU/GPU、RSS/WPR 与功耗对比，本轮不以静态推测改 route。下一切片必须在同一 IOR image oracle 下先量化这些指标，再原子更新 feature eligibility、warmup exact key 与 viewer Ready schema；这项保持 `profile_gated`，不影响本节 F0 消费缺口的源码关闭。

## 39. 2026-08-28 F2 glTF flat-normal / MikkTSpace TBN 基础设施重审

### 39.1 源码实现与测量状态

builtin base-mesh、morph target frame 与默认产品几何 owner 源码已按冻结顺序收敛；完整 decode/animation/material/texture 双 importer authority 仍未关闭。共享 tangent owner 已删除旧逐三角累加算法，改用固定 `bevy_mikktspace = 1.0.0`，保留 UV0 兼容入口并增加显式 UV0/UV1 入口，输出 `w` 统一转换到 Zircon 右手 TBN。glTF 使用 flat policy 展开 position/UV/color/skin stream，Mikk 先保留 face-corner tangent 并按最小 tangent group 拆分最终 render vertex；morph target 再从 absolute target position 重建 flat normal/Mikk tangent，并保存相对 completed base frame 的 delta。VG/SDF 只在最终 vertex/index 确定后 cook。normal texture 的有效 UV 使用共享 `KHR_texture_transform` 投影，缺失输入或 UV2+ 直接返回 parse error；OBJ 继续使用 smooth policy，authored glTF normals/tangents 保持不变。

静态证据为 scoped `rustfmt`、`git diff --check` 与 `cargo metadata --locked --no-deps` 通过，workspace/target 均解析到 E 盘。聚焦管理验证 job `149f8578c42a4166b6392aba1a4cf3b0`、replacement `bc7e5000f4ed43b5a5575ecb1d9c6c82` 和 ticket `6032100d054f4e05bc97b2ad75ec231e` 都在后续源码变化后失效；最后一张票早于最终 VG/SDF cook-order 修改。三者均不轮询、不视为当前 compile/test 通过。`Cargo.lock` 与 `zircon_runtime/Cargo.toml` 仍属于另一个 executable Session，故其 additive dependency rows 尚不能进入 scoped candidate；失败的 delayed patch `150` 未进入待处理状态，也不是验收证据。

下一性能阶段维持原 gate：在相同 importer source、同一机器与 E/D 盘受管工作根下，对 1/100/large mesh 记录旧算法与 Mikk 的 cold/warm import CPU、角点数/顶点数、peak RSS 和输出 hash；缺失 normal 的 case 另记录 `I/V` expansion ratio。只有量化确认 import-time 增量可接受且产品 normal-map image oracle 一致后，才开始 RenderDoc/WPR/功耗比较。本节仍未关闭 P0、M6-M8 或 realtime-SH9 Failure。

### 39.2 normal-convention 元数据消费审计

`TextureNormalConvention` 的 current-source runtime consumer inventory 仍为 `0`：Standard Material template、clearcoat normal 和 fallback mesh shader 共 3 个真实采样调用全部固定传 `ZR_NORMAL_CONVENTION_DX`。但它已不是“只有 parser/validation”的死元数据；runtime texture asset 层现在唯一拥有 import-time canonicalization，builtin importer 与 stable texture plugin 都在 mip generation/BC5 transcode 或资产发布前调用它，把显式 GL payload 的 G 通道翻转并把 descriptor 收敛为 DX。`zr_normal.wgsl` 的精确行为是 DX=`encoded * 2 - 1`，GL=在此基础上翻转 Y；因此当前架构方向是 import-time canonicalization + shader 零分支，而不是把 metadata 接成每像素 branch。

glTF labeled texture subasset 目前不经过 stable texture importer 的 `apply_texture_import_settings`，而是直接以 `decoded_rgba8_for_import_usage(Normal)` 发布 raw pixels 和 DX descriptor。Khronos glTF 2.0 对 normal texel 的规范映射本身是 `sample * 2 - 1`、`+Y` 向上；这与 Zircon 当前名为 DX 的 raw-decode branch 数学一致，但命名、图片上传方向和产品画面尚未由同一个 image oracle 冻结。这里不把 glTF metadata 改成 GL，否则 stable texture importer 一旦接管 labeled subasset 会额外翻 G 并改变现有数学合同。

standalone/builtin 的重复 owner 债已经关闭：stable plugin 只保留错误类型适配，实际绿色通道循环位于 runtime asset 层；非 normal fast path 在 descriptor 物化前返回，DX/None normal 不复制像素，GL normal 只执行一次 `O(P)` 原地 G 通道转换。glTF labeled subasset 的下一步仍须先用产品 image oracle 冻结 Zircon canonical TBN、Mikk `w`、texture upload 方向与 descriptor 命名，再决定是否经未来 `TextureBuildService` 进入同一 canonicalization；禁止只翻 glTF G、只改 metadata 字段或新增未入 key 的 runtime uniform。当前 glTF raw pixels、authored/generated `w` 与固定 no-flip shader 路径保持不动。

### 39.3 texture-transform、clearcoat 与 morph 边界

`KHR_texture_transform.texCoord` 当前解决 Mikk 输入的 UV channel 选择，affine offset/scale/rotation 在 shader sampling 时应用。Khronos ratified extension 定义的精确矩阵顺序为 `translation * rotation * scale`，并建议 exporter 可同时发布 pre/post-transform UV set；官方 Sample Renderer 在有 authored tangent 时也继续使用 vertex TBN，只对采样 UV 应用 transform，而 clearcoat normal 复用 base TBN。由此不能在 importer 中擅自把 affine UV 再喂给 authored/generated Mikk，否则会偏离 authored tangent 与官方 reference 的共同合同。rotation、negative scale/reflection 的产品 oracle 仍需用 Khronos `TextureTransformMultiTest` normal row 冻结；当前 source 保持 shader-only transform，不宣称该边界已验收。

`clearcoatNormalTexture` 的独立 texCoord/transform/scale 已贯穿 shared glTF extension projection、builtin/stable importer、texture-slot descriptor、`MaterialAsset`、`StandardPbrMaterialFeatures`、`MaterialRuntime`、256 B GPU uniform 与 Standard-PBR shader。base 与 clearcoat normal map 即使使用不同 UV set 也复用 glTF 规定的同一 vertex TBN；各自的 affine 变换只作用于采样坐标，不二次生成 tangent basis。required `KHR_materials_clearcoat` 仍因 factor/roughness texture owner 缺失而 fail closed；optional extension 只投影 factor、roughness factor 与完整 clearcoat normal，并为未支持的两张 factor texture 输出诊断。

glTF morph frame 源码 owner 已按规范形成每个 target 的 absolute target positions/normals，生成 flat normal/Mikk tangent，再保存相对 completed base frame 的 delta；因为 morph tangent 不表达 `w`，target handedness 改变或 target corner group 无法由 completed base split 表达时 fail closed。该 slice 仍缺 fresh managed compile/test，不进入当前 MVP geometry convergence 的验收声明；affine/clearcoat 产品 oracle 与 managed shader validation 保持 open。

### 39.4 默认产品 glTF 几何 owner 收敛

注册复核确认 `gltf_importer.gltf` priority 120 覆盖 builtin priority 10；原先仅修 builtin 不会改变默认产品。stable plugin 曾独立维护 smooth missing-normal、丢 authored tangent/color、默认 tangent、无条件 VG cook，并让 root Model 与 Mesh subasset 同时持有完整 geometry。当前源码将 index admission、flat/smooth policy、face-corner expansion、attribute preservation 与 VG/SDF request cook 收到 runtime `asset::importer` 的单一 projection；builtin OBJ/glTF 和 stable plugin 共用该 owner。stable path 同时接入 normalTexture effective UV + MeshAsset Mikk，root/mesh Model 只保留 Mesh reference，geometry 只在 Mesh subasset 发布，default settings 下 optional VG cook 为 0。

复杂度仍是 import/cook `O(V + I)`，flat case 精确输出 `I` 个 vertex/index row；默认产品删除一份 root geometry 后，常驻 payload 静态减少一个完整 `O(V + I)` 副本。runtime sample/branch/permutation/Naga/WGPU/frame-scan 增量均为 0。上述是结构与规模界，不是 CPU、RSS、功耗或产品像素测量；新 product-route tests 仍需 fresh exact managed validation，Runtime93 `MESH93-P1-11` 的完整 decode/animation/material/texture 双 authority 也没有在本节关闭。

### 39.5 架构依据

状态：**builtin base-mesh/default product/morph frame 与 clearcoat normal texture-slot ABI source implemented；affine/clearcoat product oracle open；managed/product/perf pending**。端到端 TBN 复核推翻了“看到 `TangentSpaceDx` 元数据就只翻 glTF normal texture G 通道”的局部方案。glTF 明确规定 `bitangent = cross(normal, tangent.xyz) * tangent.w`，Zircon template/fallback shader 也使用同一公式；若只把 `+Y` 像素改成 `-Y` 而不同时迁移 tangent basis handedness，会直接改变物理法线。当前 raw glTF pixels、authored `tangent.w` 与 shader cross 公式在数学上相互一致，`TextureNormalConvention` 由 texture importer 在 cook-time 消费但尚未进入 runtime shader identity；这要求统一 importer owner，而不是用破坏画面的单通道改写掩盖。

真正的基础缺口有两层。第一，glTF 缺少 `NORMAL` 时规范要求 flat normals；当前 importer 通过通用 `generate_normals` 在共享索引上累加并归一化，实际得到 smooth normals。第二，glTF 缺少 `TANGENT` 且材质有 normal texture 时规范建议用关联 UV 的默认 MikkTSpace；当前 importer 固定填 `[1,0,0,1]`，而共享 `MeshAsset::try_generate_missing_tangents` 又是普通逐三角 tangent/bitangent 求和，不满足 normal-map bake 的 Mikk 分组、镜像与 seam 合同。Unreal 的 mesh build 通过 `ComputeTangentsAndNormals` 与 `UseMikkTSpace` 统一 NTB owner；Bevy glTF loader 同样先 duplicate indexed vertices/compute flat normals，再调用共享 Mikk owner。Zircon 因此必须修共享 mesh authority 后再接 importer，禁止新增第二套 glTF-only 手写算法。

实现顺序冻结为：共享 `MeshAsset` tangent owner 迁移到 safe Rust、无运行依赖、与原始 C 输出等价的 `bevy_mikktspace` v1，并允许调用者选择 UV0/UV1；glTF primitive 解析 normalTexture 的 `texCoord` 及 `KHR_texture_transform.texCoord` override，只接受当前 GPU vertex ABI 可表达的 0/1；缺失 authored normals 时按 index 一次性展开所有 vertex/morph streams、忽略 authored tangent，生成 flat normals；缺失 authored tangents且 normal texture存在时在对应 UV channel生成 Mikk tangent。authored normals/tangents保持不改写，超过 UV1 的材质 fail closed，不能静默回落 UV0。

这一工作全部发生在 import/cook 阶段，运行时纹理采样、分支、permutation、Naga、WGPU module/PSO 与每帧扫描增量均为 `0`。flat 展开时间/空间为 `O(I)`，只在缺失 normals 的 glTF primitive 发生；Mikk 生成按 triangle/corner/vertex 规模执行，必须先保留 `I` 个 corner result，再投影成 `V + S` 个 render vertex，其中 `S` 是 tangent group 需要的最小附加 split 数。其内部工作集、`S/V` 与相对旧手写算法的 import CPU 必须用 1/100/large-mesh cold/warm profile 量化。正确性 gates 包括 indexed hard-edge flat normals、morph/skin/UV/color stream remap、UV1 normal map、mirrored UV handedness、authored tangent preservation与 unsupported UV fail-closed；性能、RSS、产品 PNG、RenderDoc与功耗在受管 profile前均不宣称通过。

### 39.6 Mikk face-corner 到 render vertex 的结构性重审

源码对照确认旧 adapter 存在 last-corner-wins 缺陷。`bevy_mikktspace::Geometry::set_tangent` 的输出单位是 `(face, corner)`；Unreal `MeshUtilities.cpp::MikkSetTSpaceBasic` 同样写入 `FaceIdx * 3 + VertIdx` 的 wedge row，随后 `StaticMeshBuilder::BuildVertexBuffer` 才用包含 tangent basis、normal、UV、color 的完整 pending vertex 做复用。旧 Zircon callback 直接用 source index 写 `tangents[index]`，共享顶点跨 Mikk orientation/tangent group 时会由后访问角点静默覆盖先前结果，平面 quad 测试无法覆盖该问题。

共享 `MeshAsset` owner 已实现两阶段投影：先生成精确 corner tangents，再以 `(source vertex, tangent bits)` 建立期望 `O(1)` 分组，首组复用 source vertex，仅为额外组追加全部 base/morph attribute row 并重写 index；U16 只有越界时才提升 U32。实现不再因为 `S>0` 复制全部 `V` 行，额外属性搬运从 `O((A+M)·(V+S))` 收敛为 `O((A+M)·S)`，其中 `A` 是 base attribute stream 数、`M` 是全部 morph attribute stream 数；失败回滚只需删除 tangent、截断追加行并恢复 index。普通单组网格保持 `S=0`、不复制 vertex payload，最坏输出为 `I` rows，核心分组时间 `O(V + I)`、工作集为 `O(V + I + (A+M)·S)`，且 runtime sample/branch/permutation/frame scan 增量仍为 0。

morph target 已先重建 absolute frame，再写相对 base 的 xyz delta；target `w` 改变或在既有 base split 上出现不可表达 corner 分歧时 fail closed。Virtual Geometry page/ordinal 与 Mesh SDF source hash 移到最终 split 之后 cook，避免发布 stale pre-split derived data。ticket `6032100d054f4e05bc97b2ad75ec231e` 早于该最终顺序，已失效；当前源码尚未形成 fresh compile/test、CPU/RSS、截图、RenderDoc 或功耗验收证据。

### 39.7 clearcoat normal 独立采样合同与 ABI 收敛

源码复核确认 group 2 binding 11/12 与 clearcoat normal texture sample 早已存在；缺陷不是缺贴图槽，而是 glTF 的独立 `texCoord`、`KHR_texture_transform` 和 `scale` 在材质投影时丢失，shader 复用 base-normal UV 且固定 scale=1。当前实现以共享 glTF extension projection 作为 builtin/stable 双 importer 的唯一语义 owner，把 clearcoat factor、roughness factor 和 normal texture metadata 投影到既有材质系统；factor/roughness texture 仍无 owner，因此 optional extension 给出诊断，required extension 继续拒绝，避免把部分实现伪装成完整 conformance。

GPU 侧没有扩大 256 B `data0..data15` uniform、288 B bindless material row、binding 数或 shader permutation。五个旧 UV selector 从五个 `f32` 收敛为 `data7.x` 的精确 6-bit mask，dielectric F0 从重复 RGB 收敛为 `data12.y` 单标量；释放的 8 个 scalar 正好容纳 clearcoat normal 的 scale/offset、预计算 `(cos,sin)`、UV bit 与 normal scale。若直接增加两个 vec4，uniform 将从 256 B 到 288 B（+12.5%），bindless row 将从 288 B 到 320 B（+11.11%）；当前增量均为 0 B。CPU 每次 material uniform rebuild 仅新增一次 `sin_cos`，复杂度 `O(1)`；shader 的 clearcoat variant 保持同一张 texture sample，只新增一次 bit test、一次 affine UV 变换和 XY scale/normalize，不新增循环、探针遍历或每帧材质扫描。identity/no-clearcoat 变体不会携带 clearcoat binding/helper。

这只是结构和算法上界，不是性能验收。必须由 fresh managed Rust/Naga/WGPU 验证证明 256 B host/WGSL layout 与两条 importer fixture 可编译，再用 Khronos clearcoat/texture-transform 产品图、`docs/tests/runtime/shader` PNG、`D:\Tools\renderdoc` capture 比对实际采样坐标、sample count、寄存器与 GPU timing；WPR/能耗数据尚未采集。M5-M8 与 realtime-SH9 Failure 状态不变。

后续 tangent owner 复审补上了材质投影之外的必要 admission。glTF 只有一套 vertex tangent basis；Khronos 要求 clearcoat-only normal 在没有 base normal texture 时必须有已定义 tangent space，并建议 base/coat 两张 normal map 使用同一 texture coordinates。builtin/stable 现在共用一条解析规则：验证两张 normal map 的有效 UV0/UV1 均真实存在；缺 tangent 时只按 base normal texture 的有效 UV 生成 Mikk；clearcoat-only normal 缺 authored `NORMAL+TANGENT` 直接拒绝。base/coat 异 UV 并非规范 MUST error，故有 authored/base-derived tangent 时保留但进入产品图 gate。该检查是 import-time `O(1)`，runtime sample/branch/permutation/frame-scan 增量为 0。

### 39.8 glTF texture-subasset 单一 owner 收敛

完整 importer 调研确认材质语义共享之后仍存在更低层的结构分叉：builtin decode、builtin labeled-subasset 与默认 priority-120 stable plugin 分别/重复拥有 BasisU gate、core/WebP source、decoded image 校验与 RGBA8 展开、usage/color-space variant 以及 sampler 映射。当前实现将完整算法下沉到 runtime `asset::importer`；builtin 的 decode 和 publication 直接调用它，stable plugin 只 re-export，保留自身 material/mesh/scene 装配。生产 texture-subasset authority 从三处 partial/full owner 收敛到一处，后续 extension、format 和 sampler 修复不会再形成 builtin/default-product 行为漂移。

本切片刻意不把架构清理伪装成产品优化。对 `T` 个 texture 与全部发布 variant 的 `Q` 个像素，source 计数、RGBA 展开和输出仍为 `O(T + Q)`；多个 texture 共用 image、同一 texture 需要多个语义 variant 时，当前独立 `TextureAsset` payload 所需的 clone 行为保持不变。runtime sample、binding、branch、permutation、Naga/WGPU 与 frame scan 均 `+0`。共享 decoded payload、Arc/COW 或延迟 transcode 只有在 managed cold/warm import profile 同时证明 CPU copy、peak RSS 与产品吞吐瓶颈后才可实施，并需记录 output hash 与 asset mutation/lifetime 语义。限定 rustfmt、`git diff --check`、`cargo metadata --no-deps` 已通过；fresh managed compile/test、CPU/RSS、PNG、RenderDoc、WPR 与功耗仍待完成，M5-M8 和 realtime-SH9 Failure 状态不变。

### 39.9 decoded-RGBA8 build kernel 与 glTF 完整 mip 闭环

- 结构性瓶颈不是 shader 采样细节，而是 glTF 直接发布单 mip，绕过 standalone texture cook；相同材质语义因 importer owner 不同而得到不同 minification 与 streaming 资格。
- 参照 Unreal `FTextureBuildFunction` 的 stateless + versioned build 约束，拒绝向 `AssetImportContext` 塞入 service locator。runtime 现在提供无状态 decoded-RGBA8 v1 kernel；source bytes/descriptor 是输入，packed mip chain/descriptor 是输出。
- 算法在一个最终 buffer 内逐级写入，sRGB LUT 每纹理一次，Kaiser 权重按轴/级缓存，normal 每级重归一化。完整链 `O(P)`，输出内存 `O(P)`，filter scratch `O(W+H)`；Box/normal 每输出 texel 最多 4 个样本，Kaiser 最多 25 个样本。
- glTF color/data/normal variants 均调用该 owner；显式非 mip `minFilter=NEAREST/LINEAR` 保持 base-only，mipmap filter/未指定 default 才声明 `GenerateOffline`，避免无用 cook 和采样回归。builtin importer version=3、stable importer version=2，二者由 build version 派生，旧单 mip artifact 必须失效。
- Plugins07 的旧 mip 实现含未提交 foreign 性能改动，本切片不覆盖或移动；后续 ownership-transfer 应以删除 duplicate、standalone importer 转调 runtime owner 收口，不能长期保留两套算法。
- 当前只完成 source/static closure。尚无受管 compile、cold/warm CPU、peak RSS、WPR energy/power 或 RenderDoc 数据，因此不声明性能提升；BC5/KTX2/BasisU、alpha coverage 与 streaming mip/mip-tail 分离继续作为高级 texture build 能力。
