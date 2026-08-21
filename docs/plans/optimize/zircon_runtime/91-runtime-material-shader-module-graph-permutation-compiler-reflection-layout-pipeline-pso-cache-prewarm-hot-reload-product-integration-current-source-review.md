---
title: Runtime Material、Shader Module Graph、Permutation、Compiler、Reflection、Layout、Pipeline、PSO Cache、Prewarm、Hot Reload 与 Product Integration 当前源码工程化差距
category: zircon_runtime
report_id: Runtime91
review_date: 2026-08-21
baseline_head: be5a281c96b6dc9d33b5c9d0f2699a8bf75afcf1
baseline_epoch: 336
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
tests:
  - zircon_runtime/src/asset/assets/shader
  - zircon_runtime/src/asset/assets/material
  - zircon_runtime/src/core/framework/render/shader
  - zircon_runtime/src/core/framework/render/material
  - zircon_runtime/src/graphics/shader
  - zircon_runtime/src/graphics/pipeline
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache
  - zircon_runtime/src/bin/zircon_shader_prewarm
  - zircon_plugins/material_editor/editor/src/tests.rs
  - zircon_plugins/rendering/features/shader_graph/runtime/src/lib.rs
  - zircon_plugins/shader_wgsl_importer/runtime/src/lib.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/85-runtime-asset-import-source-discovery-importer-recipe-subasset-derived-data-artifact-cook-package-incremental-build-worker-determinism-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/86-runtime-asset-type-schema-imported-payload-project-document-validation-dependency-serialization-versioning-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/88-runtime-asset-watch-change-ingress-coalescing-rename-overflow-targeted-reimport-generation-reload-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/89-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/90-runtime-rhi-wgpu-adapter-device-capability-resource-command-queue-submission-completion-readback-surface-device-loss-product-integration-current-source-review.md
  - docs/plans/zircon_runtime/render/08-material-shader-permutation.md
  - docs/plans/zircon_runtime/runtime/11-job-system-task-model.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/ShaderCompiler/ShaderCompiler.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/ShaderCompiler/ShaderCompilerJobCache.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/ShaderPipelineCache.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/RHI/Private/PipelineStateCache.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Private/ShaderLibrary/ShaderCodeLibrary.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Materials/MaterialShader.cpp
  - dev/bevy/crates/bevy_render/src/render_resource/pipeline_cache.rs
  - dev/bevy/crates/bevy_shader/src/shader_cache.rs
  - dev/bevy/crates/bevy_material/src/specialize.rs
  - dev/godot/servers/rendering/shader_compiler.cpp
  - dev/godot/servers/rendering/shader_compiler.h
  - dev/godot/servers/rendering/renderer_rd/shader_rd.cpp
  - dev/godot/servers/rendering/renderer_rd/shader_rd.h
  - dev/godot/servers/rendering/rendering_shader_container.cpp
  - dev/godot/servers/rendering/rendering_shader_container.h
  - dev/Fyrox/fyrox-material/src/shader/mod.rs
  - dev/Fyrox/fyrox-material/src/shader/loader.rs
  - dev/Fyrox/fyrox-impl/src/renderer/cache/shader.rs
  - dev/Fyrox/editor/src/plugins/material/editor.rs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Data/Graphs/GraphData.cs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Data/Graphs/MaterialSlot.cs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Generation/Processors/Generator.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/ShaderStripping/ShaderPreprocessor.cs
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Runtime Material、Shader Module Graph、Permutation、Compiler、Reflection、Layout、Pipeline、PSO Cache、Prewarm、Hot Reload 与 Product Integration 当前源码工程化差距

## 1. 结论

当前Zircon的Material/Shader实现不是空壳。`.zshader v2`已经区分Surface、Include、Compute与Fullscreen，能够拒绝未知字段和错误stage；材质property、option、texture slot、GeometrySource、ShadingModel、pass、quality、feature、forward/GBuffer/depth/shadow/velocity/TAA模板、Naga/WGPU验证、variant miss report、压缩磁盘缓存、prewarm manifest以及Vulkan driver pipeline cache均是真实代码。预热链相较Runtime09C冻结时也已有实质进展：source table以源码内容、include、模板、Naga和WGPU版本生成content ID，asset inventory有边界，module依赖使用SCC DAG，单个source在batch内只做一次Naga验证。以上基础应迁入统一owner，不能被删回“每个feature内嵌一段WGSL”。

但这批功能仍没有组成工程级Shader发布系统。当前生产式扫描得到81处`create_shader_module`、49处`create_render_pipeline`、27处`create_compute_pipeline`；`cache: None`为71处，`cache: Some`只有`pipeline_cache_gate.rs`内部给Mesh创建driver cache的1处。它们分布于scene renderer、后处理、UI、粒子、插件、IBL、streaming与独立feature，说明Mesh cache不能代表全renderer的PSO authority。资产、include、模板、Naga、WGPU module、pipeline layout、PSO、prepared material、prewarm与hot reload仍各自维护身份、线程、失败、缓存与寿命。

Runtime09C登记的7项P0仍未闭合，但其中缓存描述必须纠偏：Mesh当前通过`cache_content_hashes`加入完整WGSL source hash，dynamic prewarm source也以内容寻址，不能继续说“Mesh key完全不含source”。真正仍开放的是全产物身份没有统一到source/module DAG、compiler/backend/device profile、layout ABI与PSO descriptor；prewarm worker和disk metadata也没有构成一次可证明的事务发布。类似地，prewarm inventory已经不是旧报告所述的多次无界扫描且无DAG，而是已有bounded inventory和SCC；剩余问题是串行worker、无全局scheduler、无优先级/取消/supersede、以及无法证明runtime exact PSO命中。

作者工具仍是最明显的产品断链。`material_editor`声明六个命令、view、drawer、palette和graph descriptor，但引用的`plugins://material_editor/editor/graph.zui`与`plugins://material_editor/templates/default_material_graph.toml`不存在，dist又发布空command manifest、零bridge method和`invoke_command: None`。其compiler只递归常量折叠六类节点并生成传统`MaterialAsset`，texture-backed math明确拒绝。另一个`rendering.shader_graph`插件维护第二套graph模型，按Vec顺序把未消毒ID插入WGSL，无typed pin/topology/layout/reflection，其render executor是noop。`graphics/shader/shader_assets.rs`还存在第三套极小DTO。三者均未成为canonical Shader artifact或产品PSO输入。

本报告对7项既有P0进行current-source复核，不新增重复P0；新增登记 **48项P1、12项P2与48个资格门**。目标不是增加另一层facade，而是硬切到`ShaderSourceAuthority + ShaderModuleGraphCompiler + ShaderArtifactService + ShaderReflectionArtifact + ShaderPermutationDomain + PipelineLayoutCatalog + PipelineService + PipelineCacheStore + ShaderPrewarmService + ShaderGenerationCoordinator + PreparedMaterialService + ShaderAuthoringService`。在cold/warm、compile storm、reload、device loss、cook/export和100k permutation的真实产品证据闭合前，不能声称该系统达到Unreal级，更不能声称性能优于Unreal。

## 2. 审查边界、currentness与证据

### 2.1 冻结语料

| 范围 | 文件 / 行 / 非空行 / bytes / test markers | 证据等级 | fingerprint |
|---|---:|---|---|
| Runtime production owner roots | **194 / 42,335 / 38,961 / 1,553,883 / 279** | E3逐文件读取asset、framework、material、shader、pipeline、Mesh cache、prewarm CLI与dynamic API | `572cc9bb3280c7059e5db917a05bcb0db575a7110192bbb7a2b8a18e3ff53d82` |
| Runtime focused tests | **36 / 12,831 / 11,940 / 482,583 / 204** | E3读取source、template、cache、async、prewarm和product guard测试 | `9467b7fa47ff9afe252dd386352247b6eefd996f0eed91267ba8d4ab40868862` |
| canonical WGSL | **42 / 3,828 / 3,523 / 133,668 / 0** | E2/E3读取material、geometry、environment、pass与shared include源码 | `ceb8c33da67ee71f2bb089d718d39e4318d5e10348ccdb250c363dda49ac13d2` |
| Authoring plugin production | **15 / 1,552 / 1,410 / 58,169 / 13** | E3读取Material Editor、Shader Graph与WGSL importer产品实现 | `3ac21e5e23aa9c4173143e2040bf790c2c98c7b3cd826096df8dd367daa13951` |
| Authoring plugin focused tests | **1 / 277 / 247 / 9,052 / 9** | E3读取Material Graph compiler/descriptor tests | `5f1bddddec1991432fe82e10139dd961673b31ab32f0e8e1b5c7322282eb77f4` |
| direct shader/pipeline/cache callsite corpus | **87 / 18,994 / 17,819 / 735,183 / 108** | E3冻结全产品直接WGPU创建和cache接入点 | `4a1bf070cdad79849de91a2f6982a04ed3c6787a19d765084f5cfc776186fecc` |
| 五引擎参考切片 | **23 / 37,144 / 32,026 / 1,436,530** | E2/E3读取Unreal compiler/job/PSO/library、Bevy cache/reload、Godot version/reflection、Fyrox asset/editor与Unity Graph/stripping | `7664495eb87579a86166effc315e932a72df6b9bb7276ea10fae439984c80e8b` |

冻结集合代表2026-08-21共享working tree，不是只读HEAD、ABI freeze或实现验收receipt。Git基线为`be5a281c96b6dc9d33b5c9d0f2699a8bf75afcf1`，coordinator baseline epoch为336。Bevy、Godot、Fyrox与Unity Graphics参考revision分别为`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`与`a7e4c051d256a781ab362c64316b125a1e104694`。Unreal镜像的`Build.version`为6.0.0/UE5/changelist 0且无独立`.git`，因此由reference aggregate fingerprint冻结。

写报告前已有共享会话修改`zircon_plugins/shader_wgsl_importer/runtime/src/lib.rs`以及四个prewarm CLI测试/导入格式文件；本轮冻结和结论包含这些版本，但不拥有也未修改它们。WGSL importer的借用优化减少了Naga前clone，之后仍必须为asset持有source；它没有改变固定Surface与空schema/layout的语义。

### 2.2 子域物理范围

| 子域 | 文件 / 行 / test markers | 当前判定 |
|---|---:|---|
| asset shader | **8 / 1,969 / 5** | source、entry、dependency、readiness、property layout和`.zshader v2` |
| asset material | **14 / 2,325 / 1** | material instance、parent/dependency、validation、readiness和serialization |
| core render shader | **21 / 4,747 / 37** | module/import/layout、variant/prewarm、compute/fullscreen、IDE contract |
| core render material | **48 / 6,616 / 44** | material ABI、readiness和management/query surface |
| graphics material | **5 / 891 / 16** | builtin/plugin shading model和include source |
| graphics shader | **32 / 10,848 / 129** | global ABI、template、IDE、variant disk/prewarm和canonical WGSL |
| graphics pipeline | **49 / 9,452 / 77** | async compiler、compiled graph cache、driver cache和pipeline documents |
| scene resource pipeline | **3 / 243 / 5** | pipeline key与default key |
| Mesh pipeline | **10 / 1,869 / 31** | Base/depth/GBuffer/OIT/shadow/velocity/TAA PSO创建 |
| Mesh pipeline cache | **18 / 7,701 / 60** | source、variant registry、disk/prewarm、async/fallback |
| shader prewarm CLI | **21 / 7,900 / 78** | inventory、SCC dependency、manifest、resource registry和validation |

### 2.3 横向创建点统计

统计使用production-like Rust路径，排除常规`tests`、`benches`和`examples`目录；名称未遵守约定的测试helper仍可能保留，因此它只证明authority分散，不等同于单帧创建次数。

| 模式 | occurrences / files | 结论 |
|---|---:|---|
| `create_shader_module` | **81 / 74** | 排除任意路径名含`test`后仍为80 / 73；无唯一module owner |
| `create_render_pipeline` | **49 / 46** | Mesh只占局部，post/UI/plugin/scene路径继续直建 |
| `create_compute_pipeline` | **27 / 27** | compute pipeline没有共享identity、scheduler和cache |
| `cache: None` | **71 / 67** | driver pipeline cache没有成为产品默认路径 |
| `cache: Some` | **1 / 1** | 唯一命中是`pipeline_cache_gate.rs:91`内部字段赋值 |

### 2.4 证据限制

- 本轮只做current-source review，没有修改Rust、Cargo、asset或tooling，也没有运行Cargo、Editor/App、真实GPU、RenderDoc、cook/export、device-loss、compile storm、soak或benchmark。
- `dev/Graphics`是Unity Graphics/Shader Graph package源码，不含Unity native renderer；本篇只使用typed graph生成与build-time variant stripping，不把它当完整Unity Shader compiler证据。
- Fyrox提供较小引擎的serializable shader asset、binding validation、program cache与实际Material Editor最低线，不是性能上限。
- 用户已明确tooling未来迁移到Rust，本篇不审查Python/Node工具链，也不把新增外部脚本作为方案。
- Runtime89拥有Render Graph编译/执行packet；Runtime90拥有RHI device/submission/completion/device loss；本篇拥有Shader/Material artifact到PSO请求及其generation，避免重复设计backend。

## 3. 当前产品链与所有权裂缝

```text
Source asset / raw WGSL / .zshader / plugin include
  -> importer-specific ShaderAsset
  -> template String assembly + Naga validation
  -> consumer-specific wgpu::ShaderModule
  -> consumer-specific bind group / pipeline layout
  -> consumer-specific render/compute pipeline
  -> mostly cache: None

Mesh special case
  -> ShaderVariantKey / MeshPipelineVariantKey
  -> source validation + disk WGSL cache
  -> private async Base compiler or synchronous other pass creation
  -> Mesh-local Vulkan RuntimePipelineCache

Prewarm special case
  -> bounded asset inventory + SCC module dependencies
  -> content-addressed source table
  -> serial worker: Naga -> optional WGPU module/pipeline -> disk write
  -> no shared runtime PipelineService receipt

Hot reload
  -> asset/source revision creates new local keys
  -> no atomic affected-closure publication across module/layout/PSO/material
  -> old HashMap entries remain resident
```

当前不是缺少数据结构，而是数据结构没有落在唯一authority上。`ShaderAsset`、core `ShaderVariantKey`、Mesh keys、prewarm source/key、plugin graph DTO、WGPU module key与driver cache key各自正确一部分，却没有任何一个ID能证明“同一source DAG、同一compiler、同一layout ABI、同一device profile、同一generation、同一PSO descriptor”。

## 4. 必须保留的工程底座

### 4.1 `.zshader v2`和Material ABI

Surface/Include/Compute/Fullscreen、strict field/stage validation、entry point、definition、dependency、resource、render state、queue、disabled pass、material property/option/texture layout均有真实实现。硬切时应把它们编译为immutable source/module/schema artifacts，不应改回松散TOML或raw WGSL猜测。

### 4.2 GeometrySource、ShadingModel与pass specialization

static/skinned/morphed/skinned-morphed/virtual geometry和builtin/plugin shading model已避免材质与几何形变的硬编码笛卡尔积。目标是给注册表加入catalog generation、稳定ID分配、卸载和artifact revocation，不是删除注册表。

### 4.3 Prewarm source table、SCC DAG与bounded inventory

当前prewarm source ID包含source label、WGSL、include、template revision、Naga/WGPU version；manifest把共享source和variant request分离，module_dependencies已有SCC与DAG，inventory和resource registry已有边界。Runtime09C关于“无DAG/重复无界扫描”的判断在本版本已过时，本篇明确关闭该currentness描述。

### 4.4 Mesh source hash与磁盘缓存

`dynamic_api/shader_prewarm.rs:432-433`将assembly include hashes与完整WGSL hash一起放入`cache_content_hashes`；Mesh runtime也把exact WGSL source hash放入disk key输入。因此“Mesh cache key完全不含源码”的旧判断不再成立。仍需修复的是完整compiler/backend/layout/device identity、metadata lookup资格、双文件事务和全renderer接入。

### 4.5 局部Naga/WGPU tests与diagnostics

现有大量模板、环境、形变、pass、Naga与WGPU error-scope测试能捕获局部语法/ABI错误；variant miss report也已有memory/disk/compile分类。它们应作为统一artifact service的底层测试保留，同时增加跨generation与产品帧验收。

## 5. Runtime09C继承P0的当前状态

本节复核既有7项P0，**新增P0计数为0**。编号继续由Runtime09C唯一计数，避免总账重复；Runtime91接替其currentness与验收边界。

| 继承项 | 当前状态 | 2026-08-21证据与纠偏 |
|---|---|---|
| Runtime09C-P0-1 唯一Shader artifact/PSO authority | open | 81 module、49 render PSO、27 compute PSO创建点仍分散；71个`cache: None`，Mesh-local cache不是全renderer owner |
| Runtime09C-P0-2 cache identity正确性 | open，范围收窄 | Mesh与dynamic prewarm已加入source hash，旧“完全不含source”结论关闭；但compiler/backend/layout/device profile、metadata资格、pair transaction和其他consumer仍不闭合 |
| Runtime09C-P0-3 共享编译调度 | open | `PipelineAsyncCompiler`仍每实例一个OS线程，Mesh async默认false且只覆盖Base；prewarm worker仍串行，无全局priority/cancel/supersede |
| Runtime09C-P0-4 原子generation/reverse dependency/last-good | open | key含revision/source hash只能产生新项；没有source/import/material/PSO/prepared binding affected closure、原子发布与有界retirement |
| Runtime09C-P0-5 readiness/reflection/ABI | open | `ShaderReadinessReport::is_ready`不要求非空entry、kind stage和layout；WGSL importer固定Surface且清空schema/layout |
| Runtime09C-P0-6 Material/Shader Graph产品链 | open | Material Editor缺资源与operation factory；两套plugin graph加一套graphics DTO；runtime Shader Graph executor为noop |
| Runtime09C-P0-7 可见失败策略 | open | Base async placeholder固定`SkipDraw`；`DepthOnly`只有枚举/测试证据，terminal failure仍可让可见几何永久消失 |

### 5.1 P0-2纠偏后的准确边界

Disk schema v1对canonical variant和传入content hashes做hash，Mesh调用者目前包含WGSL source hash。lookup却没有将metadata中的template/Naga/WGPU version作为expected contract逐项比较；prewarm source ID与disk key也仍是两个对象。WGSL与metadata分两次写入，进程崩溃可留下pair不一致；通用`atomic_write`在Windows目标已存在时允许保留旧target并返回成功的分支，使“写成功”不必然等于“目标内容已替换”。Mesh runtime命中后再比较WGSL并回退当前source，降低了运行时错误执行风险，但不能证明预热、provenance和持久化receipt真实。

### 5.2 P0-5的直接false-ready证据

`asset/assets/shader/readiness.rs:120-130`只要求runtime WGSL、已有entry/definition项无diagnostic以及validation diagnostics为空；空entry集合通过`all`，`has_pipeline_layout`只进入summary。`shader_wgsl_importer/runtime/src/lib.rs:49-74`解析Naga entry后无条件写`ShaderAssetKind::Surface`，dependencies/imports/property/options/resources/layout均为空。合法compute-only WGSL因此可被误归类为Surface，空或不匹配产品ABI的module也能先ready再在PSO创建阶段失败。

### 5.3 P0-7的产品风险

Mesh构造在`construct.rs:213`把async compile默认设为false；开启后`ensure_pipeline.rs:129-134`只允许Base进入异步路径，placeholder在21-22行固定`SkipDraw`。queue unavailable、budget拒绝或terminal failure都会保留无pipeline状态；`DepthOnly`虽定义于`async_compile.rs`，没有产品选择者。正确策略应按pass/domain区分startup-required、last-good、error material、defer和fatal，不能用一个“不卡帧”的消失策略替代正确性。

## 6. P1差距清单

### 6.1 Source、asset与module graph（P1-01..08）

| ID | 差距 | 必须重构为 |
|---|---|---|
| Runtime91-P1-01 | Raw WGSL importer把所有stage集合固定为Surface，无法表达generic library/compute/fullscreen owner | 由显式descriptor决定kind；无descriptor时只生成`GenericShaderModuleSource`，禁止成为material-ready asset |
| Runtime91-P1-02 | importer丢弃imports、dependencies、source files、property/resources/layout，Naga只证明语法合法 | `ShaderSourceArtifact`携source map、declared imports、dependency edges和reflection request |
| Runtime91-P1-03 | `.zshader`、raw WGSL、plugin includes、builtin templates各有解析/normalization路径 | `ShaderSourceAuthority`统一canonical URI、content ID、language、owner generation和diagnostic span |
| Runtime91-P1-04 | plugin shading include按descriptor/token/ready records反复normalize、扫描、load和clone源码 | generation内发布`NormalizedImportToken -> ModuleArtifactId`索引，冲突fail-closed |
| Runtime91-P1-05 | template assembly继续使用String replace/concat/entry rename，variant间重复解析 | parse-once immutable module DAG与typed specialization slot，保持source map |
| Runtime91-P1-06 | module import循环和SCC只在prewarm CLI局部表达，runtime artifact链没有同一DAG | `ShaderModuleGraphCompiler`成为runtime/cook/prewarm/IDE共同owner |
| Runtime91-P1-07 | source dependency只覆盖声明路径，generated include/template/version没有统一edge type | typed edge区分Import、GeneratedInclude、Schema、CompilerRevision、PluginOwner与TargetProfile |
| Runtime91-P1-08 | source错误的身份分散为asset diagnostic、Naga string、WGPU scope与miss report | stable diagnostic ID、source span、module path、generation、consumer closure与repair action |

### 6.2 Compiler、reflection与layout（P1-09..16）

| ID | 差距 | 必须重构为 |
|---|---|---|
| Runtime91-P1-09 | Naga validation、template compile、IDE preview、prewarm和product PSO重复解析同一WGSL | content-keyed single-flight compile stage，一次parse/validate供多consumer复用 |
| Runtime91-P1-10 | compiler revision由局部字符串常量表达，没有toolchain manifest和兼容域 | `ShaderCompilerProfileId`包含frontend、IR schema、Naga、backend、target和optimization profile |
| Runtime91-P1-11 | runtime没有持久化typed reflection artifact，layout多由手写schema并行构造 | `ShaderReflectionArtifact`持bind groups、bindings、visibility、entry IO、workgroup、push/specialization常量 |
| Runtime91-P1-12 | Material validator使用字符串`contains`验证WGSL symbol，不能证明声明类型/地址空间/ABI | 以reflection和material schema做双向typed compatibility check |
| Runtime91-P1-13 | 未知material property若TOML值是任意string可绕过unknown-property诊断 | property name/type必须来自schema；asset reference/string literal使用不同typed value |
| Runtime91-P1-14 | pipeline layout由多个consumer手建，global/material/pass bind group ABI没有唯一版本 | `PipelineLayoutCatalog`以typed binding schema生成稳定LayoutId与兼容性诊断 |
| Runtime91-P1-15 | readiness只看diagnostic为空，不证明kind-specific entry/stage IO/layout完整 | `ShaderArtifactReadiness`按Surface/Compute/Fullscreen/Library分别定义required contract |
| Runtime91-P1-16 | compile失败只留String或局部enum，缺source/variant/layout/backend因果链 | structured `ShaderCompileFailure`与可重放input manifest，区分source/schema/backend/device原因 |

### 6.3 Permutation、specialization与material binding（P1-17..24）

| ID | 差距 | 必须重构为 |
|---|---|---|
| Runtime91-P1-17 | `ShaderVariantKey`、render entry key、Mesh key、prewarm key和dead graphics key重复 | 明确`SourceVariantId`、`LayoutId`、`PipelineDescriptorId`三层，typed lowering唯一生成 |
| Runtime91-P1-18 | `packed_dims`从bit 0放geometry、bit 4放shading，geometry ID大于15即重叠 | versioned bit layout并在注册/deserialize验证范围，或删除packed identity只用canonical hash |
| Runtime91-P1-19 | material option、texture presence、render state中哪些是static/dynamic没有schema owner | `ShaderPermutationDomain`声明每维StaticDefine、Specialization、Uniform、BindingPresence或RuntimeBranch |
| Runtime91-P1-20 | texture-presence normalization只计数equivalent，仍可能生成重复source/PSO | canonicalize后实际复用同一SourceVariant/PSO，fallback texture不进入不必要permutation |
| Runtime91-P1-21 | geometry/shading plugin ID没有catalog generation、卸载和remap协议 | plugin-owned stable ID lease，artifact带owner generation，卸载按submission retirement撤销 |
| Runtime91-P1-22 | platform token是自由字符串，不能证明backend/device capability/format/sample | typed `ShaderTargetProfileId`引用Runtime90 DeviceProfile与render target compatibility class |
| Runtime91-P1-23 | prepared material/bind group没有跨draw共享的generation-qualified artifact | `PreparedMaterialService`按MaterialRevision+LayoutId+resource lease生成immutable handle |
| Runtime91-P1-24 | parent material、property override、texture dependency与PSO invalidation不在同一reverse DAG | material inheritance/dependency graph输出affected prepared-material与pipeline closure |

### 6.4 Pipeline、PSO cache与prewarm（P1-25..32）

| ID | 差距 | 必须重构为 |
|---|---|---|
| Runtime91-P1-25 | render/compute pipeline分别由46/27个文件直接创建，状态不可统一查询 | 所有consumer只申请`PipelineHandle`，backend创建仅在Runtime90 RHI内部 |
| Runtime91-P1-26 | Mesh多张pass HashMap、module map、variant registry无count/bytes/age/device代淘汰 | `PipelineService`统一stable handle、residency budget、LRU/priority和completion-qualified retirement |
| Runtime91-P1-27 | async compiler每实例一个OS线程，只限制pending count，finish recv和Drop join可阻塞 | Runtime11 shared executor、priority、deadline、cancel、supersede与bounded shutdown |
| Runtime91-P1-28 | async默认关闭且只覆盖Base，其他Mesh pass与全部feature走同步创建 | pipeline creation统一异步状态机；startup/cook可显式await receipt，frame thread永不编译 |
| Runtime91-P1-29 | prewarm worker虽有budget contract，执行仍是单串行loop | shared scheduler并行source/variant stage，受CPU/GPU/I/O/resident-byte独立预算约束 |
| Runtime91-P1-30 | prewarm临时device/layout验证不证明runtime exact descriptor与driver entry | prewarm调用同一artifact lowering、PipelineDescriptorId和RHI backend，产出exact readiness receipt |
| Runtime91-P1-31 | disk WGSL/meta为两个文件，metadata version不作为lookup expected contract | 单一transactional artifact envelope或journaled pair，lookup接受完整expected ArtifactId/profile |
| Runtime91-P1-32 | Vulkan-only driver cache只接Mesh，启动读和Drop持久化同步执行 | backend capability化PSO library，异步load/flush ticket、全PipelineService接入和总量/年龄预算 |

### 6.5 Hot reload、lifecycle与产品集成（P1-33..40）

| ID | 差距 | 必须重构为 |
|---|---|---|
| Runtime91-P1-33 | source revision/hash只让新key产生，不能计算受影响的transitive consumers | reverse dependency index从source/import到variant/layout/PSO/material/product view |
| Runtime91-P1-34 | reload没有一个原子generation覆盖module、layout、PSO和prepared binding | `ShaderGenerationCoordinator`后台构建完整required closure后一次compare-and-publish |
| Runtime91-P1-35 | compile failure不能自动保留并证明last-good与当前binding兼容 | last-good generation与compatibility class绑定；不兼容时转error material而非混代 |
| Runtime91-P1-36 | 旧module/PSO/prepared material只累积，没有明确retire条件 | generation lease + Runtime90 submission completion retirement + bounded cache eviction |
| Runtime91-P1-37 | device loss没有把CPU artifact复用与GPU PSO重建拆开 | CPU ShaderArtifact跨device保留，GPU PipelineHandle进入new DeviceGeneration的Queued状态 |
| Runtime91-P1-38 | project/quality/platform/plugin切换没有统一required-set差分 | `ShaderProductSet`计算added/retained/retired artifacts并产生有界transition receipt |
| Runtime91-P1-39 | feature consumer可绕过prewarm/PSO service在首次frame直接创建 | lint/architecture gate禁止production直接WGPU shader/pipeline创建，例外清单仅RHI backend/tests |
| Runtime91-P1-40 | miss/fail观测没有material/entity/view/frame/age/fallback闭环 | `PipelineDemandReceipt`记录demand owner、priority、first-needed frame、terminal state与fallback |

### 6.6 Editor、Graph、testing与performance（P1-41..48）

| ID | 差距 | 必须重构为 |
|---|---|---|
| Runtime91-P1-41 | Material Editor声明资源路径不存在，命令只有descriptor且dist无invoke/bridge | 随包发布真实ZUI/template，命令绑定transactional factory并返回operation receipt |
| Runtime91-P1-42 | Material Graph只有六类节点/String pin，未校验pin schema、multiple incoming和完整拓扑 | typed node/pin registry、topological validation、stage/domain capability和migration version |
| Runtime91-P1-43 | Material compiler只常量折叠为传统MaterialAsset，texture math被拒绝 | `MaterialIR`支持typed expressions、texture/sample/parameter/dependency/source map并生成surface artifact |
| Runtime91-P1-44 | Shader Graph按Vec顺序拼字符串、ID未消毒、missing output写magenta | 删除该编译器或迁到同一typed graph compiler；错误fail-closed并提供structured diagnostic |
| Runtime91-P1-45 | Shader Graph runtime executor为noop，Editor feature只有descriptor | 产品feature必须消费PipelineHandle编码真实pass；未实现则不得发布available capability |
| Runtime91-P1-46 | 三套Graph/Variant DTO并存且没有migration owner | 硬切到canonical document/schema；旧DTO删除，不留compat re-export或双写 |
| Runtime91-P1-47 | 大量测试锁source字符串/局部happy path，缺cold/warm/reload/failure产品矩阵 | artifact-identity、exact hit、last-good、device-loss、cook/export、compile storm集成测试 |
| Runtime91-P1-48 | 没有统一compile CPU、resident、PSO create、cache I/O与frame hitch基线 | source/variant/pipeline各stage telemetry、p50/p95/p99、budget breach与可复现trace artifact |

## 7. P2差距清单

| ID | 差距 | 长期目标 |
|---|---|---|
| Runtime91-P2-01 | compiler与引擎同进程，恶意/畸形source可拖垮产品 | 隔离compiler worker process、heartbeat、crash quarantine与可重放job bundle |
| Runtime91-P2-02 | 没有跨机器content-addressed Shader DDC | signed local/shared DDC，完整input hash与toolchain provenance |
| Runtime91-P2-03 | 没有分布式compile调度与结果验证 | farm scheduler、dedupe、priority、cancel、output hash和不可信worker验证 |
| Runtime91-P2-04 | native shader binary/library策略不完整 | DXIL/SPIR-V/Metal等backend artifact library与compatibility/support window |
| Runtime91-P2-05 | PSO usage capture没有产品级采集/合并/排序 | per-product trace、stable usage masks、merge、sort、coverage和regression governance |
| Runtime91-P2-06 | Material Graph没有subgraph/function library | versioned function/subgraph artifact、incremental invalidation与共享compile cache |
| Runtime91-P2-07 | custom code节点没有沙箱与capability | restricted source domain、resource/stage capability、time/memory budget和security audit |
| Runtime91-P2-08 | permutation pruning只靠人工维度 | build-time reachability、SAT/constraint pruning、runtime usage反馈与可解释strip report |
| Runtime91-P2-09 | pipeline library没有跨版本迁移/patch/merge | signed library manifest、patch layering、plugin library mount和atomic rollback |
| Runtime91-P2-10 | 高级pipeline domain缺统一identity | mesh/task/ray tracing/work graph等在backend支持后进入同一descriptor/artifact模型 |
| Runtime91-P2-11 | shader性能只看compile/PSO时延，缺GPU code quality证据 | ISA/statistics、occupancy/register/bandwidth、vendor regression与场景视觉基线 |
| Runtime91-P2-12 | 没有跨引擎可复现实验说明“优于Unreal” | 固定内容、设备、driver、build、cache state、trace与统计方法的公开benchmark protocol |

## 8. 五套参考引擎的可迁移差异

### 8.1 Unreal：工程上限是全局compile/job/cache/library authority

Unreal的`FShaderCompilingManager`集中追踪outstanding jobs、取消、finish与time-sliced async result processing；`FShaderJobCache`用完整input hash做duplicate single-flight、异步DDC、output hash、memory budget与eviction statistics。`ShaderPipelineCache`明确表达batch size、time/memory budget、pause/resume/save/open、usage mask与统计；RHI PipelineStateCache集中创建、consolidation和eviction。ShaderCodeLibrary则拥有packaged shader maps、preload/release、per-stage object创建、plugin libraries、patch/merge/cook。Zircon当前最缺的不是某个宏，而是这些职责落在共同身份和生命周期上。

### 8.2 Bevy：WGPU同样可以有稳定状态机与反向依赖

Bevy `PipelineCache`以稳定ID公开`Queued/Creating(Task)/Ok/Err`，统一持有layout、bind-group layout、ShaderCache、device、pipeline waiting/new状态。ShaderCache记录processed modules by defs、resolved imports、reverse dependents与waiting-on-import；`set_shader`清理transitive dependents并返回受影响pipeline IDs重新排队。它证明Zircon不能以WGPU限制为理由保留分散创建和无reverse closure。Bevy也明确不自动对pipeline descriptor去重，因此这里只采用状态/依赖owner，不把它当Unreal级PSO上限。

### 8.3 Godot：Shader Version、variant group、reflection和cache是显式对象

Godot `ShaderRD::Version`维护dirty/valid/initialize、variant arrays、group tasks与mutex；base hash包含Godot version/hash、stage sources和debug信息，version hash包含uniform/global、排序code sections与custom definitions。cache schema/API path显式版本化，并行group compile/load；RenderingShaderContainer持久化SPIR-V及binding/specialization/stage reflection、push constants和IO masks。Zircon目前没有等价的原子Version owner和typed reflection container。

### 8.4 Unity Graphics/Shader Graph：Graph不是字符串拼接器

Unity `GraphData`维护nodes、edges与active targets，在mutation后验证；typed `MaterialSlot`定义连接兼容性、stage capability与default value。Generator收集active fields/requirements、targets/passes/permutations/properties/includes/dependencies并确定性生成shader dependencies。Core build pipeline还有可扩展variant stripping scope与report。Zircon的两套graph compiler目前没有这些typed contract，也没有build-time product closure。

### 8.5 Fyrox：小型引擎也已有单一可序列化ShaderDefinition

Fyrox Material shader以一个serializable `ShaderDefinition`拥有resources、passes、draw parameters和source，检查binding唯一性；renderer cache编译pass program并对missing binding严格报错，Editor有真实Material Editor。它不是性能上限，但说明Zircon的raw WGSL固定Surface、多个Graph DTO和空产品executor尚未稳定跨过较小引擎的完整性基线。

## 9. 目标架构与硬切边界

### 9.1 唯一artifact链

```text
ShaderSourceAuthority
  -> ShaderSourceArtifact(source id, owner generation, spans)
  -> ShaderModuleGraphCompiler(import DAG, parse/validate once)
  -> ShaderArtifactService(IR, reflection, compiler profile)
  -> ShaderPermutationDomain(source variant id)
  -> PipelineLayoutCatalog(layout id, ABI compatibility)
  -> PipelineService(pipeline descriptor id, stable handle/state)
  -> Runtime90 RHI backend object + submission-qualified lifetime

MaterialGraphDocument
  -> ShaderAuthoringService typed graph validation
  -> MaterialIR / SurfaceArtifact
  -> Material schema + dependencies
  -> PreparedMaterialService
  -> same PipelineService

ShaderGenerationCoordinator
  -> affected reverse closure
  -> background build
  -> atomic publish / last-good / retire
```

### 9.2 核心所有者

| Owner | 唯一职责 | 禁止拥有 |
|---|---|---|
| `ShaderSourceAuthority` | URI、source bytes、content ID、language、owner generation、source map | WGPU module与PSO |
| `ShaderModuleGraphCompiler` | import DAG、cycle/SCC、parse/validate、module IR | product scene/material state |
| `ShaderArtifactService` | compiler profile、typed reflection、artifact cache与diagnostic | direct frame submission |
| `ShaderPermutationDomain` | static dimensions、canonicalization、constraints、variant ID | arbitrary consumer string key |
| `PipelineLayoutCatalog` | bind group/layout ABI与compatibility class | material instance values |
| `PipelineService` | descriptor ID、stable handle、state、single-flight、residency | source authoring document |
| `PipelineCacheStore` | transactional disk/backend library、quota、provenance | frame-thread I/O |
| `ShaderPrewarmService` | product required set、priority/budget、readiness receipt | alternate compiler/descriptor path |
| `ShaderGenerationCoordinator` | affected closure、atomic publish、last-good、retirement | backend resource creation |
| `PreparedMaterialService` | material revision/layout/resource lease到prepared handle | graph editing transaction |
| `ShaderAuthoringService` | typed graph schema、IR、preview/diagnostic artifact | plugin-local duplicate graph DTO |

### 9.3 硬切规则

- production Rust中禁止在Runtime90 RHI backend之外直接调用`create_shader_module`、`create_render_pipeline`、`create_compute_pipeline`；tests必须使用显式allowlist。
- 删除plugin-local重复Graph/Variant DTO与compat re-export，不做双写、shadow cache或旧新owner并存。
- runtime、cook、prewarm、IDE preview与Editor compile必须调用同一source/module/artifact lowering；允许不同target profile，不允许复制compiler。
- frame thread不得parse WGSL、Naga validate、创建PSO、读写disk cache或等待compile worker。
- `SkipDraw`不能是通用terminal failure策略；每个pipeline domain必须声明last-good/error/defer/fatal policy。
- cache key必须来自完整typed input artifact，metadata不能弥补不完整key。

## 10. 依赖顺序与重构里程碑

### L0：身份、清单与禁止新增债务

定义Source/Module/Artifact/Reflection/Layout/Pipeline/Generation ID和compiler/device profile；生成direct WGPU创建点inventory与allowlist，CI禁止新增绕行。纠正`packed_dims`、raw WGSL kind与readiness false-ready。

### L1：Source、module compiler与reflection

建立`ShaderSourceAuthority`、统一module DAG、parse-once compiler、typed reflection和layout compatibility。先让`.zshader`、raw WGSL、builtin/plugin includes和IDE/prewarm共享artifact，再迁移Graph。

### L2：全renderer PipelineService硬切

把Mesh Base/other passes、post、UI、particle、IBL、plugin compute/render逐域迁移到stable `PipelineHandle`；删除consumer-local module/PSO creation和私有async compiler。Runtime90 RHI负责backend object与device generation。

### L3：Generation、Material与Graph产品闭环

建立reverse dependency closure、atomic generation、last-good和prepared material。合并三套Graph DTO，交付真实ZUI/template/operation、typed MaterialIR、preview与runtime executor。

### L4：Cook、prewarm、disk与driver library

统一exact required product set、transactional disk artifact、backend pipeline library、quota与async load/flush；cold/warm启动必须用同一PipelineDescriptorId证明命中。

### L5：性能、规模与高级能力

完成100k permutation、compile storm、multi-project/plugin churn、device loss、PSO usage capture、variant pruning、DDC/worker isolation和跨设备benchmark，再讨论超过Unreal的性能结论。

## 11. 48个资格门

### 11.1 Source与module graph（G01..G08）

| Gate | 通过条件 |
|---|---|
| G01 | 每个Shader source有canonical URI、content ID、language与owner generation，重复内容可共享 |
| G02 | Raw WGSL无descriptor时不会被猜为Surface；Surface/Compute/Fullscreen按kind contract验收 |
| G03 | module import DAG含typed edge、cycle/SCC、missing/redirected dependency和source span |
| G04 | 同source generation在runtime/cook/prewarm/IDE只parse/Naga validate一次 |
| G05 | plugin include token由generation index O(1)解析，冲突和owner unload fail-closed |
| G06 | generated template/include/compiler revision全部进入artifact identity |
| G07 | source change能列出完整transitive consumer closure并可重放验证 |
| G08 | source/module诊断带stable ID、span、generation、artifact和repair action |

### 11.2 Compiler、reflection与layout（G09..G16）

| Gate | 通过条件 |
|---|---|
| G09 | compiler input manifest包含toolchain/backend/target/optimization且hash确定 |
| G10 | duplicate compile single-flight，取消一个waiter不会取消仍有consumer的job |
| G11 | reflection完整表达bindings、visibility、entry IO、workgroup、push/specialization常量 |
| G12 | material schema与reflection双向匹配，未知/缺失/类型错误不能ready |
| G13 | PipelineLayoutId由typed schema生成，跨generation兼容判断可解释 |
| G14 | compiler failure区分source/schema/backend/device并保存可重放manifest |
| G15 | compile queue有job/byte/resident/priority/deadline/cancel/shutdown预算 |
| G16 | frame/editor UI线程不执行parse、Naga、backend compile或blocking finish |

### 11.3 Permutation与material（G17..G24）

| Gate | 通过条件 |
|---|---|
| G17 | SourceVariant/Layout/PipelineDescriptor三层ID边界唯一，无consumer手拼key |
| G18 | 所有packed/serialized dimension有版本、范围与collision test |
| G19 | 每个permutation维度声明static/specialization/uniform/binding/runtime branch分类 |
| G20 | canonicalization实际复用artifact/PSO，不只增加统计计数 |
| G21 | 100k permutation生成、lookup、evict和diagnostic在预算内且无identity collision |
| G22 | Geometry/Shading plugin ID带owner generation，可安全reload/unload/retire |
| G23 | PreparedMaterialHandle绑定MaterialRevision、LayoutId和resource generation |
| G24 | parent/override/texture变化只重建affected prepared materials与pipelines |

### 11.4 Pipeline、cache与prewarm（G25..G32）

| Gate | 通过条件 |
|---|---|
| G25 | production direct shader/render/compute pipeline创建只剩RHI backend allowlist |
| G26 | 所有consumer通过stable PipelineHandle观察Queued/Creating/Ready/Failed/Retired |
| G27 | pipeline single-flight覆盖Mesh、post、UI、particle、plugin render与compute |
| G28 | frame首次需求不触发同步compile、disk I/O或driver cache persist |
| G29 | memory/disk/backend cache均有count/bytes/age/priority/device-generation预算 |
| G30 | disk artifact单事务写入，crash/corrupt/stale产生typed miss且不伪报成功 |
| G31 | prewarm并行受CPU/GPU/I/O/resident预算约束，支持priority/cancel/supersede |
| G32 | warm startup证明required PipelineDescriptorId全部exact hit且runtime compile miss为0 |

### 11.5 Hot reload与生命周期（G33..G40）

| Gate | 通过条件 |
|---|---|
| G33 | source/import/material/plugin变化生成确定affected closure |
| G34 | required closure全部ready后module/layout/PSO/prepared material一次原子切代 |
| G35 | compile失败保留compatible last-good；不兼容时可见error policy而非混代 |
| G36 | stale async completion不能覆盖较新generation，结果可回收且有receipt |
| G37 | 旧GPU PSO按submission completion退休，CPU artifact按lease/cache policy淘汰 |
| G38 | device loss保留可复用CPU artifact并为新DeviceGeneration重排GPU pipeline |
| G39 | project/quality/platform/plugin切换有required-set差分、deadline与rollback |
| G40 | compile/pipeline shutdown有bounded drain/abort，Drop不join或同步持久化 |

### 11.6 Authoring、产品与性能（G41..G48）

| Gate | 通过条件 |
|---|---|
| G41 | Material Editor所有声明ZUI/template真实存在、随包发布且可从普通Editor打开 |
| G42 | 六个命令绑定operation factory、transaction、undo/redo和terminal receipt |
| G43 | typed Graph验证pin/type/stage/domain/topology/cycle/migration并生成source map |
| G44 | Material Graph生成MaterialIR/SurfaceArtifact并由同一PipelineService preview/runtime |
| G45 | Shader Graph feature不再noop；未实现时capability不可发布available |
| G46 | cold/warm/reload/failure/device-loss/cook/export矩阵在真实产品路径通过 |
| G47 | compile CPU、queue wait、resident bytes、PSO create、cache I/O、frame hitch有p50/p95/p99 |
| G48 | “优于Unreal”比较固定内容、设备、driver、build、cache state、trace和统计方法，可复现 |

## 12. 禁止的临时实现

- 不得新增“局部HashMap + 直接WGPU create”作为某个新feature的pipeline cache。
- 不得用source字符串比较、metadata字符串或revision常量补救不完整artifact key。
- 不得让合法Naga module自动等价于Surface-ready、Material-ready或PSO-ready。
- 不得用`String::contains`、未消毒identifier或Vec存储顺序替代reflection、typed graph和topology。
- 不得让prewarm、IDE、Editor preview拥有与runtime不同的模板/compile/layout实现。
- 不得通过更多私有OS线程解决shader compile并发；必须进入Runtime11 executor与预算。
- 不得把`SkipDraw`、magenta源码或noop executor当作已交付feature的终态。
- 不得保留旧Graph DTO、compat module、双写cache或旧新PipelineService并存。
- 不得在Drop中无期限join worker或同步读取/持久化driver blob。
- 不得只以单测数量、Naga parse成功或WGPU对象创建成功宣称产品工程化完成。

## 13. 本轮完成边界

本轮完成的是Runtime91 current-source静态审查、Runtime09C currentness纠偏、五套参考引擎差异归纳、目标owner、重构层次和资格门设计。没有实现任何production修复，没有运行Cargo、Editor/App、真实GPU、RenderDoc、cook/export、device-loss、compile storm、soak和benchmark。7项继承P0仍由Runtime09C唯一计数，本篇新增0项P0、48项P1与12项P2；所有实现状态均为`not_started`。

Runtime91不能单独宣告Material/Shader系统完成。只有Runtime85/86/88的asset/import/reload generation、Runtime89的Render Graph packet、Runtime90的RHI device/submission/lifetime、Runtime11 job system以及本篇Shader/PSO artifact链共同通过G01-G48，且普通Editor和产品游戏路径使用同一artifact与PipelineService后，才具备进入性能对标的资格。
