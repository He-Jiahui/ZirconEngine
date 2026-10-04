---
title: Runtime Material、Shader Artifact、Variant、Pipeline、PSO 与 Publication 当前工作树复核
category: zircon_runtime
report_id: Runtime225
review_date: 2026-09-02
baseline_head: 9963f8eb72e2d725d2536eb50b393b30387a1ffa
related_code:
  - zircon_runtime/src/asset/artifact/cache_payload/material_shader
  - zircon_runtime/src/asset/assets/material
  - zircon_runtime/src/asset/assets/shader
  - zircon_runtime/src/asset/importer/ingest/import_shader_package.rs
  - zircon_runtime/src/core/framework/render/material
  - zircon_runtime/src/core/framework/render/shader
  - zircon_runtime/src/graphics/material
  - zircon_runtime/src/graphics/pipeline
  - zircon_runtime/src/graphics/shader
  - zircon_runtime/src/graphics/scene/resources
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh
  - zircon_runtime/src/dynamic_api/shader_prewarm.rs
  - zircon_runtime/src/bin/zircon_shader_prewarm
plan_sources:
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/91-runtime-material-shader-module-graph-permutation-compiler-reflection-layout-pipeline-pso-cache-prewarm-hot-reload-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/189-runtime-material-shader-artifact-variant-pipeline-pso-cache-publication-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/224-runtime-resource-authority-asset-residency-current-working-tree-review.md
  - docs/plans/optimize/zircon_editor/249-editor-material-shader-graph-instance-toolkit-preview-compiler-current-working-tree-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/Shader.h
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/ShaderCompilerCore.h
  - dev/UnrealEngine/Engine/Source/Runtime/RHI/Public/PipelineStateCache.h
  - dev/UnrealEngine/Engine/Source/Runtime/RHI/Public/PipelineFileCache.h
  - dev/bevy/crates/bevy_render/src/render_resource/pipeline_cache.rs
  - dev/bevy/crates/bevy_shader/src/shader_cache.rs
  - dev/godot/scene/resources/material.h
  - dev/godot/servers/rendering/renderer_rd/shader_rd.h
  - dev/godot/servers/rendering/renderer_rd/shader_rd.cpp
  - dev/Fyrox/fyrox-material/src/lib.rs
  - dev/Fyrox/fyrox-material/src/shader/mod.rs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Data/Graphs/GraphData.cs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Generation/Processors/Generator.cs
doc_type: current_working_tree_recheck
review_status: complete
implementation_status: not_started
source_recheck_required: true
tooling_scope: excluded_by_user_request
---

# Runtime Material、Shader Artifact、Variant、Pipeline、PSO 与 Publication 当前工作树复核

## 1. 结论

Runtime189 的核心判断仍成立，近期工作树没有形成新的全 Renderer artifact authority。当前可复现选集为 **651 个文件、153,977 行、142,247 非空行、5,666,957 bytes、1,459 个 test attributes、74 个 ignored attributes、1 个 lexical unsafe token**；其中 **90 个文件处于 tracked dirty 或 untracked 状态**。选集不仅覆盖 Material、Shader、Pipeline、ResourceStreamer 与完整 Mesh 链，还把 `graphics` 下所有生产候选的直接 shader module/render pipeline/compute pipeline 创建文件纳入交叉审查。审查期间新增的residency work queue及其测试已计入最终快照；它们属于Runtime224资源链进展，不改变本报告的shader/program/PSO authority结论。

工作树包含真实但局部的进展：Material/schema 查询改为按规模建立借用 HashMap/HashSet，Shader IDE entry name改为move，module registry预分配容量，Mesh resolver新增configuration epoch并在策略改变后清空cached draw commands。这些改动减少局部clone、线性扫描或陈旧command复用，但没有改变authority、identity、scheduler、publication和device-generation边界。

按排除test目录及主文件首个`#[cfg(test)]`之后代码的词法口径，当前仍有：

| 直接WGPU调用 | 生产候选调用点 | 文件数 | 主要分布 |
|---|---:|---:|---|
| `.create_shader_module(...)` | 59 | 53 | post process 18、Mesh 12、advanced lighting 9，其余分布于UI、overlay、environment、particle、HZB、mipgen等 |
| `.create_render_pipeline(...)` | 47 | 44 | post process 21、Mesh 8、advanced lighting 4、UI/overlay各3，其余分布于particle、scene clear、sprite等 |
| `.create_compute_pipeline(...)` | 14 | 14 | advanced lighting 5、post process 5，其余为environment、graph execution、HZB、mipgen |

该计数是“直接dot-call交叉审查入口”，不是对唯一绕过点的夸大：其中包含局部factory实现和backend adapter；但它足以证明所有创建尚未硬切到唯一`PipelineArtifactService`。`RenderPassGpuResourceFactory`只统一了调用形状与创建计数，其`wgpu::Device`/native实现仍直接创建设备对象，不拥有canonical key、single-flight、artifact state、budget、generation、retirement或persistent provenance。

因此09C七项父P0仍为 **3 Open / 4 Partial / 0 Closed**；Runtime189的36项P1仍为 **28 Open / 8 Partial / 0 Closed**，14项P2仍全部Open，30项资格门仍为 **20 Fail / 10 Partial / 0 Pass**。本报告不复制或重编号canonical finding，而是给出当前树的逐链证据、需要重构的具体落点和硬切顺序。

## 2. 范围、口径与工作树边界

### 2.1 可复现选择集

选择集由以下目录完整递归去重组成：

1. Material/Shader asset、artifact cache payload、framework material/shader。
2. `graphics/material`、`graphics/pipeline`、`graphics/shader`。
3. `graphics/scene/resources`与`graphics/scene/scene_renderer/mesh`完整目录。
4. Shader package importer、project shader record、dynamic prewarm API与prewarm CLI。
5. `zircon_runtime/src/graphics`中含三种直接WGPU创建dot-call的全部Rust文件。

统计读取当前物理文件，包含tracked dirty与untracked文件；不把文件存在、测试注册、descriptor或metrics wrapper等同于产品闭环。旧Runtime189的175文件统计与本轮651文件统计范围不同，不能用文件数或直接调用数做趋势比较。

### 2.2 脏工作树分类

| 当前修改族 | 当前变化 | 对工程状态的影响 |
|---|---|---|
| Material dependency/schema/projection | 线性去重/查找改为按阈值构建HashSet/HashMap，Editor projection也改为HashSet | 局部CPU优化；不改变schema authority、artifact identity、document/compiler或preview状态 |
| Shader IDE/module/prewarm | move entry names、容量预分配、借用source id | 局部clone/allocation优化；不改变production reflection publication或compile scheduler |
| Mesh variant/draw cache | environment-only PBR策略变化推进configuration epoch并清空command cache | 修复一种resolver策略变化后的陈旧复用；仍是单策略、全清、进程内、可wrap的局部epoch |
| ResourceStreamer/residency | 大量在途residency、texture、upload与semantic executor修改 | 已由Runtime224单独审查；尚未把shader/program/PSO和draw consumer切到统一artifact owner |
| Mesh draw command/materialization | command arena、parallel preparation、统计与cache调整 | 不改变PSO creation authority和generation-qualified GPU install合同 |

### 2.3 证据等级

- **E3**：当前物理文件逐链读取、direct-create生产候选调用交叉扫描、脏文件核对和owner/caller追踪。
- **E2**：对照Runtime09C/91/189/224、Editor249以及本地Unreal、Bevy、Godot、Fyrox、Unity Graphics源码。
- **E1**：测试、ignored benchmark、profile counter和descriptor只证明局部意图；没有据此提升产品状态。
- **E0**：本轮未运行Cargo、WGPU/DX12/Vulkan、Editor、热重载、device loss、RenderDoc、scale、soak或竞争benchmark，不能证明性能或表现优于Unreal。

## 3. 当前真实链路

```text
.zshader/.wgsl
  -> import_shader_package
       -> ShaderAsset + validation_diagnostics
       -> validate_wgsl_captures (当前为字符串contains)
  -> ResourceStreamer::ensure_shader_source_recursive
       -> PreparedShader { source/imports/generated WGSL, revision, dependency_revision }
       -> visiting/completed遍历集合（cycle当前静默早退）
  -> ShaderModuleRegistry + Naga validation/reflection
       -> entry/resource/stage I/O/layout hashes/sampling pairs
  -> MeshPipelineCache private validation worker
       -> ShaderVariantCacheDisk (压缩WGSL source cache)
       -> shader module map + per-pass PSO maps + Vulkan driver cache
       -> private Base pipeline worker
  -> PreparedMaterial staged/published/previous/rejected
       -> viewport/Mesh requirement ledger
       -> PublishedMaterialDrawProxy + cached mesh draw command

旁路：post process / advanced lighting / UI / overlay / particle / HZB / mipgen / graph compute
  -> 各自直接create shader module / render pipeline / compute pipeline
```

这条链的关键问题不是“没有高级类型”，而是高级类型只在Mesh局部形成闭环。其它renderer consumer没有被同一program artifact、pipeline state machine、target profile、publication generation与retirement合同约束。

## 4. 可保留的工程底座

1. `ShaderTemplateReflection`从同一个Naga `Module`和`ModuleInfo`推导entry stage、stage I/O、resource binding、sampling pair、type/interface/resource layout hash及specialization dependency；这应成为canonical program artifact的一部分。
2. `ShaderModuleRegistry`对模板include图有显式`visiting` stack和cycle error。应保留该实现，并把同等SCC/diagnostic语义前移到asset dependency graph；不能把ResourceStreamer的静默早退误认为同等能力。
3. Mesh shader validation能把entry、vertex/fragment接口、resource layout与attachment contract做fail-closed admission；应抽离为renderer-neutral contract validator。
4. `PreparedMaterial`的staged/published/previous/rejected和`MaterialPipelineGenerationAdmissionLedger`已经区分candidate、last-good和已解析PSO；这是统一publication service的原型。
5. Base pipeline的typed admission、queue saturation、pending/failure原因和bounded miss report是真实底座；应扩展到所有pass/compute target，而不是继续为每个pass新增HashMap。
6. source disk key已经包含canonical variant、source hash、include content hash、template revision与Naga/WGPU version；它适合作为source artifact cache输入的一部分，但不能当作compiled program或PSO命中。
7. runtime Vulkan pipeline cache有64 MiB读取上限、magic/length/digest校验和原子写helper；应迁入显式persistent cache provider，而不是在Drop中静默持久化。
8. resolver configuration epoch修复了environment-only PBR策略切换后的cached command陈旧复用，是正确性局部修复；后续应由qualified configuration/artifact generation取代裸计数器。

## 5. 当前源码差距与必须重构内容

### 5.1 Source、dependency、validation 与 reflection

| 映射owner | 状态 | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| MSP4-P1-025 | Open子边界 | `validate_wgsl_captures`只执行`source.contains(name)`；注释、子串或无关标识符即可伪造capture，且结果直接进入shader import diagnostics。 | 从已验证Naga module/reflection解析canonical property/texture binding；按稳定slot ID、group/binding、type和visibility逐项匹配，保留source span。 |
| MSP4-P1-026 | Open | `ensure_shader_source_recursive`在`!traversal.enter(shader_id)`时返回`Ok`，将正在访问和已完成两种情况折成同一早退；asset import cycle没有SCC路径诊断。 | 构建不可变dependency graph，Tarjan/Kosaraju SCC；非法环fail-closed，允许环必须由显式module policy声明并产生稳定diagnostic。 |
| MSP4-P1-025/034 | Partial | Naga reflection很强，但只在Mesh validation/cache内部发布；`ShaderReadinessReport::is_ready`不要求reflection、selected constants、GPU module、pipeline或install ready，甚至不要求`has_pipeline_layout`。 | 分离`SourceReady / Validated / Specialized / PipelineReady / Installed / Degraded`；consumer声明最低阶段，receipt携带artifact/device generation。 |
| MSP4-P1-008 | Partial | authored `pipeline_layout` DTO仍进入asset management/readiness统计，实际Mesh布局来自另一owner。 | 删除生产死DTO；布局只来自specialized reflection + pass contract，旧数据只通过versioned migrator读取。 |
| MSP4-P1-027 | Open | ShaderAsset、PreparedShader、assembled source、disk payload和module cache继续保存多份长字符串或clone。 | content-addressed immutable source blob、include slice/provenance和artifact handle；限制source bytes/import depth/node count。 |

### 5.2 Variant identity、program 与 PSO authority

| 映射owner | 状态 | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| MSP4-P1-001/002 | Open | 没有进程级`ShaderArtifactId`/`ProgramArtifactId`/`PipelineArtifactId`；filtered census仍覆盖59/47/14个直接创建点。 | 唯一`ShaderArtifactService`与RHI-facing `PipelineArtifactService`；旧direct create callsite按产品族硬切删除。 |
| MSP4-P1-003/004 | Open | `MeshPipelineCache`同时拥有module、多个pass PSO map、variant registry、layouts/fallback、两个worker、source cache、driver cache、publication ledger和diagnostic。 | 分拆source/program authority、shared scheduler、pipeline registry、material publication、fallback bundle与metrics projection。 |
| MSP4-P1-006/007 | Open | `ShaderVariantKey`虽含material revision/layout/options/geometry/shading/pass/features/quality，但生产platform token固定`wgpu-runtime`。 | `ShaderTargetProfileId`覆盖backend、adapter/device、features/limits、binding model、shader compiler ABI、quality、selected constants；PSO key再加入entries/layout/render state/attachments/device generation。 |
| MSP4-P1-005 | Open | variant interner仍单调增长，ID耗尽走panic；新configuration epoch不提供retirement或bounded identity。 | generation-qualified bounded interner，带tombstone、pin、retirement和typed exhaustion admission。 |
| MSP4-P1-002 | Open | `RenderPassGpuResourceFactory`对device/native context仍直接调用WGPU，只附加metrics。 | factory只能消费artifact descriptor/handle；真正creation由唯一registry执行并返回state/receipt，禁止wrapper成为第二authority。 |

### 5.3 编译调度、状态机与frame stall

| 映射owner | 状态 | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| MSP4-P1-009/010 | Open | 每个`PipelineAsyncCompiler`创建私有OS thread；request bounded而completion使用无界`channel()`；Mesh至少有source validation和Base pipeline两实例。 | 接入Runtime共享executor，job/result/source/IR byte统一预算，single-flight按artifact key去重。 |
| MSP4-P1-011 | Open | FIFO没有priority、deadline、cancel、supersede、owner/device affinity或view/project fairness。 | compile ticket携带owner、stage、generation、priority、deadline、cancel token与device affinity；旧代完成不得发布。 |
| MSP4-P1-012/013 | Open | `finish_pending*`调用阻塞`recv`，prewarm/error proxy路径可同步等待；compiler Drop无期限join，driver cache Drop同步persist且忽略错误。 | frame/render路径只poll；bootstrap走显式loading gate；shutdown提供bounded drain/abort/persist receipt，析构不等待worker/I/O/GPU。 |
| MSP4-P1-014/015 | Partial | Base有typed admission，但其它Mesh pass及post/compute多为局部同步create、Option/skip/panic或各自失败语义。 | 全target统一`Queued -> Preprocess -> Compile -> Create -> Ready/Failed -> Retired`状态机和stable error taxonomy。 |

### 5.4 Cache、persistence 与预算

| 映射owner | 状态 | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| MSP4-P1-017/018 | Partial/Open | `ShaderVariantCacheDisk`只缓存压缩WGSL；lookup/decompress/write仍在调用线程同步I/O。 | source/IR/program/PSO分层cache，由scheduler异步读取/验证/写入，miss reason和bytes可观测。 |
| MSP4-P1-019 | Open | payload与metadata分别atomic write；`rename`失败但目标存在时删除temp并返回成功，不验证现有目标内容。 | content-addressed immutable payload + 单一manifest原子提交；冲突后重新读取并exact verify，损坏不得伪成功。 |
| MSP4-P1-019/020 | Open | primary root损坏时返回Error并只删除primary pair，不能证明继续尝试fallback roots；无全局entry/byte/age/project quota或orphan sweep。 | 分层fallback lookup policy、quarantine、budget、pin/lease、eviction、orphan sweep和recovery receipt。 |
| MSP4-P1-021/022/023 | Partial/Open | Vulkan driver key只含backend/vendor/device，缺driver/OS/build/compiler/capability；构造同步读，Drop同步写且吞错。 | backend provider产生compatibility key和load/persist receipt，异步保存last-good，明确unsupported/incompatible/corrupt/failed。 |
| MSP4-P1-024 | Partial | `CompiledGraphCache`只有16-entry局部LRU，compute/graph/Mesh cache仍各自定义key、状态、预算和retirement。 | 局部cache只持统一artifact handle；entry/bytes/device generation/pin/retirement由共享registry拥有。 |

### 5.5 Material publication、draw currentness 与 device generation

| 映射owner | 状态 | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| MSP4-P1-028/029 | Partial/Open | `MaterialAsset`同时保留固定PBR字段和动态property/texture maps；projection同步能缓解但不是唯一可写authority。 | shader/schema生成canonical typed property table与stable slot ID；legacy固定字段只作read-old migration projection。 |
| MSP4-P1-030/031 | Open | material draw generation是进程内序号，publication requirements主要来自Mesh viewport；post/particle/sprite/UI不共享原子admission。 | content-derived `MaterialArtifactId` + renderer-neutral publication transaction，收集全部consumer required artifacts。 |
| MSP4-P1-032/035 | Partial/Open | previous只保留一代；PSO pin有局部ledger，但module/layout/bind group/texture/device object未由同一submission fence和device generation保护。 | immutablepublished bundle覆盖program/PSO/layout/bindings/textures，submission ticket pin完整集合，device loss按新generation重装。 |
| MSP4-P1-033 | Open | error proxy requirement可调用`finish_pending_shader_source_validations`同步等待；error material/pipeline/layout/binding不是预建的全target bundle。 | 启动阶段预建并验证每target error bundle；运行时切换仅选择已安装generation，不等待compiler。 |
| MSP4-P1-005/030 | Open | 新configuration epoch用`wrapping_add`且变化时全量clear cached commands；它不表达target/device/profile/artifact。 | qualified resolver configuration hash/generation进入command key，按依赖精确失效，wrap/alias由generation handle防止。 |

### 5.6 Editor边界当前性

Editor249在2026-09-02复核后仍是当前owner，不新建重复Editor273：

1. `plugins://material_editor/editor/graph.zui`与`plugins://material_editor/templates/default_material_graph.toml`仍物理缺失。
2. Material dist仍为`is_stateless: true`、`state_schema_version: 0`、`invoke_command: None`、`bridge_methods: []`。
3. Material Graph仍只编译base color/单texture到传统MaterialAsset；第二套ShaderGraph仍按Vec拼接WGSL，runtime executor仍直接`Ok(())`。
4. 当前Editor material projection的HashSet优化只是已有structural projection的局部查找优化，没有新增operation factory、transactional document、durable save、artifact receipt、runtime preview或product caller。

因此Editor137五项父P0与Editor249的34项P1、12项P2、26门状态均不变。

## 6. 父P0当前重判

| Canonical owner | 状态 | 2026-09-02重判 |
|---|---|---|
| `09C-P0-1` 唯一Shader artifact / PSO authority | Open | direct create和局部factory仍跨renderer分散；Mesh cache不是全局owner。 |
| `09C-P0-2` 完整cache identity | Partial | source key较强；target/device/driver/compiler/constants/layout/state/attachment的program/PSO identity仍未统一。 |
| `09C-P0-3` 共享编译调度与非阻塞render path | Open | 两个Mesh私有worker、无界completion、同步finish/Drop，以及其它pass同步create仍在。 |
| `09C-P0-4` 原子generation/reverse dependency/LKG | Partial | Material三槽、PSO requirement pin和resolver epoch是局部进展；跨consumer/device/submission原子性仍无证据。 |
| `09C-P0-5` readiness/reflection/ABI | Partial | Naga reflection成熟；source readiness、字符串capture、静默cycle和Mesh私有消费仍造成false-ready边界。 |
| `09C-P0-6` Material/Shader Graph产品与唯一schema | Open | 两套graph、缺资源/operation/preview/save和no-op executor均未改变。 |
| `09C-P0-7` 可见failure/fallback policy | Partial | Base typed admission可保留；其它target和error bundle仍没有统一状态与原子fallback。 |

## 7. Runtime189 P1逐组当前性

| Runtime189条目 | 当前状态 | 当前树复核摘要 |
|---|---|---|
| MSP4-P1-001..007 | 7 Open | artifact authority、direct create、Mesh大对象、per-pass maps、variant retirement、target profile、canonical PSO key均未闭合。 |
| MSP4-P1-008 | 1 Partial | authored layout DTO的迁移inventory存在，生产删除和唯一layout authority未完成。 |
| MSP4-P1-009..013 | 5 Open | private worker、unbounded completion、无priority/cancel、同步finish、Drop等待均仍在。 |
| MSP4-P1-014 | 1 Partial | Base异步typed admission存在，覆盖面未扩至全pass/compute。 |
| MSP4-P1-015..016 | 2 Open | error/retry策略与全链correlation仍分散。 |
| MSP4-P1-017 | 1 Partial | source cache identity可保留，但不是compiled artifact/PSO cache。 |
| MSP4-P1-018..020 | 3 Open | 同步I/O、双文件提交、无预算/eviction仍在。 |
| MSP4-P1-021 | 1 Partial | Vulkan driver cache有bounded seed校验，provider/identity/receipt未工程化。 |
| MSP4-P1-022..023 | 2 Open | compatibility key不完整，Drop persist吞错。 |
| MSP4-P1-024..025 | 2 Partial | 局部LRU与source readiness存在，但不等于统一artifact/pipeline/install状态。 |
| MSP4-P1-026..027 | 2 Open | asset dependency SCC/cycle fail-close和source blob去重未完成。 |
| MSP4-P1-028 | 1 Partial | typed schema/projection存在，双authority仍在。 |
| MSP4-P1-029..031 | 3 Open | migration、content artifact identity、跨consumer publication未完成。 |
| MSP4-P1-032 | 1 Partial | material LKG/PSO pin存在，完整GPU bundle fence retirement未完成。 |
| MSP4-P1-033..036 | 4 Open | error bundle、selected constants、device reinstall和全链E2E/scale/visual证据未完成。 |

合计保持 **28 Open / 8 Partial / 0 Closed**。近期micro-optimization、profile counter、ignored benchmark或configuration epoch没有资格改变以上状态。

## 8. 参考引擎差异

| 参考源码合同 | Zircon当前差异 | 应吸收的边界，不照搬实现 |
|---|---|---|
| Unreal `FShaderMapResource`分离code/resource，shader key含material map、pipeline、vertex factory、permutation和platform；PSO precache有priority、request ID、Complete/Missed/TooLate等结果。 | Zircon key的平台只有`wgpu-runtime`，私有FIFO worker无priority/cancel/deadline，prewarm与draw miss没有同一artifact request。 | qualified shader/program/PSO identity、usage-driven precache ticket、priority提升、可审计miss/too-late结果。 |
| Bevy单一`PipelineCache`维护Queued/Creating/Ok/Err，`ShaderCache`等待imports、记录依赖pipeline并在shader变化后requeue。 | Zircon只有Mesh Base形成类似状态，其它target和shader dependency invalidation不经过同一queue/registry。 | descriptor作为状态机输入、shader reverse dependency到pipeline requeue、统一render/compute owner。 |
| Godot `ShaderRD::Version`拥有dirty、variant/group、compile task、cache、valid和RID install；Material/ShaderMaterial参数与RID lifecycle关联。 | Zircon source readiness、Mesh reflection/module/PSO、material publication和device install由不同owner判断。 | shader version/artifact/install一体化状态、variant group、明确dirty/valid/install和资源lifecycle。 |
| Fyrox ShaderDefinition定义pass/resource bindings，Material用typed property/resource binding消费同一schema。 | Zircon固定PBR字段、动态TOML值、shader property layout、Editor projection和runtime binding仍可漂移。 | 单一typed schema、pass/resource contract和共享material resource；legacy字段仅迁移。 |
| Unity Shader Graph `GraphData`保存active/unknown target并做validation/version，Generator按target/subshader/pass收集property/keyword并产出真实shader。 | Zircon两个graph模型没有统一target/pass/diagnostic/artifact，optional executor仍no-op。 | canonical graph source、target/subtarget合同、deterministic compiler、diagnostic source map和真实preview artifact。 |

参考源码只证明成熟系统需要哪些明确合同，不证明其具体数据结构适合Zircon。Zircon可继续使用Rust、Naga、WGPU和现有三槽publication，但必须达到同等可证伪的identity、state、lifecycle、failure与规模边界。

## 9. 目标架构与不变量

```text
ShaderSourceAuthority
  -> immutable source blobs + dependency DAG/SCC + source spans
  -> ShaderCompileScheduler
       { single-flight, priority, deadline, cancel, generation, byte budget }
  -> SpecializedProgramArtifact
       { exact target profile, constants, validated IR/code, reflection, provenance }
  -> PipelineArtifactService
       { entries, layout, render/compute state, attachments, device generation }
  -> MaterialArtifactService
       { typed values, textures, required pipeline handles, fallback bundle }
  -> MaterialPublicationTransaction
       { candidate, LKG, all-consumer admission, installed device generation }
  -> SubmissionArtifactPins
       { program, pipeline, layout, bindings, textures, completion fence }
```

必须同时满足以下不变量：

1. hash只用于索引；canonical descriptor exact compare才是身份。
2. source/build、artifact、device install、live publication四类generation不可互换。
3. `Ready`必须带阶段；SourceReady绝不能投影成PipelineReady或Installed。
4. render/frame路径不等待compiler、cache I/O、driver persist或worker shutdown。
5. fallback是预先验证的完整bundle，不以skip draw、几何消失或旧局部对象掩盖失败。
6. 任何旧direct creation API删除前必须有caller/deletion matrix；删除后用结构门禁止回流。

## 10. 依赖顺序与实施切片

1. **M225.0 关闭false-ready**：为字符串capture、dependency cycle、source-ready冒充pipeline-ready、ShaderGraph no-op建立RED tests和stable diagnostic；产品capability在闭环前明确Unavailable。
2. **M225.1 冻结identity与artifact schema**：定义source/program/pipeline/material/target profile descriptor、规范序列化、exact compare、migration和write-current-only；删除死authored layout authority。
3. **M225.2 共享compile scheduler**：迁出Mesh两个私有worker，建立single-flight、priority/deadline/cancel/supersede、request/result byte budget和显式shutdown receipt。
4. **M225.3 统一pipeline registry并硬切caller**：先迁Mesh所有pass，再按post process、advanced lighting、UI/overlay/particle、graph compute/HZB/mipgen顺序删除direct create；local cache只持artifact handle。
5. **M225.4 原子material publication**：canonical typed property table、全部renderer consumer requirement、prebuilt error bundle、device generation install和完整submission pin。
6. **M225.5 Editor产品闭环**：只保留一个Material/Shader Graph source；补齐package资源、operation factory、transaction/save、artifact receipt、runtime preview和cook，不在Editor复制compiler/PSO owner。
7. **M225.6 资格验收**：1/100/10,000 variant/material，cold/warm/reload/corrupt/queue saturation/device loss；DX12/Vulkan/Metal、visual capture、CPU/GPU/RSS/VRAM/I/O/power全部形成可重复报告。

任何里程碑的完成标准都是旧authority删除、唯一状态机可观测、失败可恢复且动态门通过；新增wrapper、HashMap、epoch、counter或ignored benchmark不能代替硬切完成。

## 11. 资格门当前状态

Runtime189的30门逐组复核如下：

| Gate组 | 状态 | 当前结论 |
|---|---|---|
| RT-MSP-G01..G04 | 4 Fail | 唯一source/program/pipeline authority与完整PSO identity未形成。 |
| RT-MSP-G05..G06 | 2 Partial | Naga reflection/contract较强，但authored DTO和Mesh私有覆盖仍在。 |
| RT-MSP-G07 | 1 Fail | asset dependency graph没有SCC fail-close。 |
| RT-MSP-G08 | 1 Partial | source readiness存在，阶段化artifact/install readiness缺失。 |
| RT-MSP-G09..G17 | 9 Fail | 统一状态机、scheduler、nonblocking、shutdown、预算、cache atomicity/compatibility/retirement均未完成。 |
| RT-MSP-G18 | 1 Partial | source cache identity较强，产品价值和异步/预算未证明。 |
| RT-MSP-G19 | 1 Fail | compiled program/PSO persistent provenance未形成。 |
| RT-MSP-G20..G22 | 3 Partial | schema、typed projection、三槽LKG有底座，migration/唯一authority/retirement不完整。 |
| RT-MSP-G23 | 1 Fail | 跨consumer required pass不能原子admit。 |
| RT-MSP-G24..G26 | 3 Partial | error proxy、PSO pin、reload generation只有Mesh局部证据。 |
| RT-MSP-G27..G30 | 4 Fail | device recovery、10k规模、visual capture和综合性能基线均无动态证据。 |

合计 **20 Fail / 10 Partial / 0 Pass**。

## 12. Review-only交付边界

本轮只新增Runtime review文档并更新索引/coverage/Editor249当前性说明；没有修改Runtime、Editor、plugin、Cargo、ABI、WGSL或ZUI实现。没有运行Cargo、真实Editor、GPU、热重载、device loss、RenderDoc、cook/export、fault/scale/soak或benchmark。Tooling继续按用户要求排除，也没有查询、轮询、等待或实时跟踪协调器。
