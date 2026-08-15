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
implementation_status: pending
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

### P0-2：磁盘cache key不能证明source/compiler/backend一致，会接受陈旧WGSL

`ShaderVariantPrewarmSource`的ID包含label、WGSL、include hashes、template revision、Naga和WGPU版本；`ShaderVariantCacheDiskKey::from_variant_key`却只hash `ShaderVariantKey::canonical_string`和include hashes。写入meta包含template/Naga/WGPU版本，但lookup只验证schema、hash与canonical string，不接收expected source ID/version。compiler升级、template生成器改动或同revision源码变化都可能被旧entry命中。

目标定义一个不可拆分的`ShaderArtifactId`：source/module DAG content IDs、typed definitions、material layout/options、geometry/shading/pass/quality、template compiler revision、Naga/backend compiler version、target/platform capability profile共同进入hash。disk metadata只用于诊断。读取API必须接收完整expected ID并返回typed miss reason；禁止以读后再比较WGSL字符串补救错误key。corrupt/stale清理作为有预算I/O job执行。

### P0-3：编译调度是三套互不兼容的局部机制，默认仍同步阻塞

Mesh cache构造`mesh-shader-validate`和`mesh-pipeline-compile`两个私有OS线程，bounded channel只限制pending count；`finish_pending*`阻塞recv，`Drop`无期限join。异步pipeline默认false，即使开启也只覆盖Base pass。prewarm公开预算明确拒绝`max_in_flight_variants != 1`并串行做WGPU module/pipeline validation与disk write。Vulkan driver cache同步读最多64MiB并在`Drop`同步`get_data`和atomic write。

目标由Runtime11共享executor承载preprocess、Naga、backend affinity create、compression/I/O和publish。按content key single-flight，预算至少覆盖job count、source bytes、resident bytes、driver bytes、priority、age与deadline；支持取消、supersede、shutdown drain/abort。render/editor线程不得调用finish/wait、filesystem或driver persist。compiler worker若需要进程隔离可在P2升级，但P0先删除每cache私有线程和析构阻塞。

### P0-4：没有跨asset/source/variant/PSO/prepared-material的原子generation与last-good发布

Asset facade能发Modified并保留resource generation，但Shader import反向依赖、template source、variant disk、Mesh module/PSO HashMap、material bind group和compiled pipeline各自失效。repo搜索没有发现product shader-change到Mesh pipeline invalidation的闭环；`ShaderCache`式reverse dependent graph只在prewarm CLI inventory局部存在。一次reload可能让新material layout、旧WGSL、旧bind group和旧PSO同时可见。

目标链固定为：

`AssetCatalogGeneration -> ShaderModuleGraphGeneration -> ShaderPermutationGeneration -> PipelineLayoutGeneration -> RhiPipelineGeneration -> PreparedMaterialGeneration`。

每层发布immutable artifact和reverse edges；变更计算affected closure，后台编译全部required产物后一次原子切代。失败保留last-good generation并发布诊断，删除/ABI不兼容使用共享error material。old generation按09A submission completion retirement，不按CPU frame或HashMap替换立即释放。device loss产生新device generation并使所有GPU handle fail-closed。

### P0-5：Shader/Material readiness和ABI校验会产生false-ready资产

WGSL importer用Naga验证语法后无条件写`ShaderAssetKind::Surface`，dependencies/source_files/imports/property/resources/layout均为空。`ShaderReadinessReport::is_ready`只要求有runtime WGSL、已有entry/defs无诊断和validation diagnostics为空；它不要求至少一个entry point、surface所需vertex/fragment contract或pipeline layout。因此compute-only/不匹配entry的合法WGSL可被标记ready，直到具体pipeline创建失败。

目标按kind/target声明或reflection artifact建立严格readiness：surface必须满足约定entry、stage IO、bind groups、material function和pass ABI；compute/fullscreen分别校验entry/workgroup/resource contract。Raw WGSL importer不得猜surface；缺少`.zshader` descriptor时只能导入generic module artifact，不能直接成为material shader。reflection/layout hash必须来自编译artifact并与authoring schema对拍，禁止两套手写layout独立通过。

### P0-6：Material/Shader Graph产品表面未连接，且存在两套不兼容资产模型

canonical `MaterialGraphAsset`只有6种节点和String pin；Material Editor validation不验证from/to pin合法性、pin类型、重复incoming edge或完整topology。compiler递归常量求值，texture-backed Add/Multiply失败，输出只是传统MaterialAsset字段。六个命令没有operation factory，两个引用资源不存在。optional Shader Graph另有7种节点/第二套asset，按Vec顺序输出未消毒identifier和引用字符串；缺ID、type、cycle、missing-ref、binding和target/pass验证；无output时返回magenta WGSL，post executor永远`Ok(())`不编码任何命令。

目标硬切到一个typed `MaterialGraphDocument -> MaterialIR -> SurfaceArtifact`链。graph node/pin使用stable schema ID和value type，编译先做topological/type/stage/domain验证，再生成可复用IR、uniform/texture ABI、source map与per-pass surface函数。Material Editor命令必须绑定transactional operation handler，真实ZUI/template随plugin打包；preview消费同一artifact service和PSO handle。删除plugin-local ShaderGraphAsset、noop executor和dead `graphics/shader/shader_assets.rs` DTO，或迁移后明确只留一个canonical model。

### P0-7：PSO miss/failure语义以`SkipDraw`隐藏正确性问题

Base异步compile首次miss、queue full、worker unavailable或terminal failure都可能返回None；默认placeholder policy为`SkipDraw`。这避免frame thread等待，但把错误表现为几何体消失。异步又默认关闭，普通产品更可能在首次见到variant时同步source/cache/pipeline创建。DepthOnly枚举存在但Base固定SkipDraw；其他pass没有统一last-good/error policy。

目标按pipeline domain定义显式policy：required startup PSO在受控bootstrap deadline前ready；hot reload使用last-good；新可选variant使用共享error material/diagnostic draw或明确defer；depth/shadow/velocity不能随意用color fallback。每次miss/late/fail都带variant ID、material/entity/view、state age和fallback reason，进入frame/profile/editor。产品测试必须证明失败不会静默消失、不会阻塞submission，也不会把不兼容last-good与新binding混用。

## 5. P1 差距清单

### P1-1：Shader模板每variant重复分配、全文替换、拼接和解析

`ShaderTemplateInclude::new`反复拥有token/source/owner、扫描include、strip directive并hash派生module；builtin registry每次assembly重建。material surface用11次顺序`String::replace`专化大段WGSL，随后forward/deferred/TAA各自拼接、重命名entry和Naga parse。目标缓存parsed module DAG和typed specialization slots；同source generation parse一次，variant只patch definitions/IR并共享line map。

### P1-2：plugin shading include解析按descriptor/token/ready-record三重扫描

每个plugin shading model的forward/GBuffer/deferred token都过滤全部ready shader records，反复trim、slash replace、lowercase、suffix比较，命中后同步load并clone WGSL。目标由AssetCatalogGeneration发布`NormalizedImportToken -> ShaderModuleArtifactId`索引及冲突诊断；stable generation扫描/normalize/load/source clone均为0。

### P1-3：variant identity重复且部分维度只存在于局部key

canonical `ShaderVariantKey`、entry-point `RenderShaderVariantKey`、Mesh `PipelineKey/MeshPipelineVariantKey`和dead public `graphics::shader::ShaderVariantKey`并存。Mesh key含alpha cutoff、texture presence和render-state booleans；shader key只投影部分feature。target format/sample/layout、device/backend capability不在统一key。目标区分`SourceVariantId`、`PipelineLayoutId`和`PipelineDescriptorId`，由typed lowering唯一生成，禁止consumer手拼或clone整套key。

### P1-4：GeometrySource公开ID范围与`packed_dims`布局冲突

`GeometrySourceId::is_plugin_range`只判断`>=4`，允许到255；`ShaderVariantKey::packed_dims`却把geometry放bits 0..并从bit4开始放8-bit shading model。geometry >15会重叠。当前`packed_dims`除测试外没有产品consumer，因此不是现役碰撞，但它是待启用的错误合同。目标要么显式限制/分配4-bit ID并在注册/反序列化拒绝越界，要么扩大/版本化packed schema；canonical hash不得依赖无验证bit pack。

### P1-5：texture-presence变体只计数“可归一化”，没有实际归一化

`MeshPipelineVariantRegistry::has_texture_presence_equivalent_variant`枚举16种texture-presence组合，仅增加`texture_presence_normalized_variant_count`；它不复用existing variant。`MeshPipelineVariantKey`仍hash完整PipelineKey，可能为不改变source/descriptor的texture presence生成重复PSO。目标由Material ABI声明哪些选项是static permutation、dynamic uniform、binding presence或fallback texture；只把真实改变shader/PSO的维度放入key。

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

本篇review完成、implementation pending。没有修改production source，没有运行Cargo或真实GPU。现有working tree在Material/Shader/Pipeline/Mesh cache/WGSL范围存在大量其他Session改动，所有行号、callsite数和文件统计都是2026-08-15读取时快照；M0必须重查后才能实施。

下一审查单元是09D texture/mesh/material residency、streaming、upload和eviction；09C只把依赖generation与PreparedMaterial边界交给它，不提前把streaming实现混入Shader compiler owner。
