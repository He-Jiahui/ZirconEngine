---
title: Runtime Material、Shader Artifact、Variant、Pipeline、PSO Cache 与 Generation Publication 当前工作树工程化差距
category: zircon_runtime
report_id: Runtime189
review_date: 2026-08-30
baseline_head: working-tree
related_code:
  - zircon_runtime/src/asset/assets/material
  - zircon_runtime/src/asset/assets/shader
  - zircon_runtime/src/graphics/material
  - zircon_runtime/src/graphics/pipeline
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset
  - zircon_runtime/src/graphics/scene/resources
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pipeline_cache
  - zircon_plugins/material_editor
  - zircon_plugins/rendering/features/shader_graph
plan_sources:
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/91-runtime-material-shader-module-graph-permutation-compiler-reflection-layout-pipeline-pso-cache-prewarm-hot-reload-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/137-editor-material-shader-graph-material-instance-vfx-particle-preview-compiler-diagnostics-authoring-current-source-review.md
  - docs/plans/optimize/zircon_runtime/166-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-current-source-review.md
  - docs/plans/optimize/zircon_runtime/188-runtime-asset-resource-lifecycle-locator-registry-load-cache-import-cook-current-working-tree-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/Shader.h
  - dev/UnrealEngine/Engine/Source/Runtime/RenderCore/Public/ShaderCompilerCore.h
  - dev/UnrealEngine/Engine/Source/Runtime/RHI/Public/PipelineStateCache.h
  - dev/UnrealEngine/Engine/Source/Runtime/RHI/Public/PipelineFileCache.h
  - dev/bevy/crates/bevy_render/src/render_resource/pipeline_cache.rs
  - dev/bevy/crates/bevy_shader/src/shader_cache.rs
  - dev/godot/scene/resources/material.h
  - dev/godot/servers/rendering/shader_language.h
  - dev/godot/servers/rendering/renderer_rd/shader_rd.h
  - dev/godot/servers/rendering/renderer_rd/shader_rd.cpp
  - dev/Fyrox/fyrox-material/src/lib.rs
  - dev/Fyrox/fyrox-material/src/shader/mod.rs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Data/Graphs/GraphData.cs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Generation/Processors/Generator.cs
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
tooling_scope: excluded_by_user_request
---

# Runtime Material、Shader Artifact、Variant、Pipeline、PSO Cache 与 Generation Publication 当前工作树工程化差距

## 1. 结论

当前工作树已经不再是“一个 WGSL 字符串加一个 RenderPipeline”的样例实现。Shader importer/readiness 能区分 Module、Surface、Compute 与 Fullscreen；Naga reflection 已发布 entry stage、stage I/O、resource binding、min binding size、type layout、sampling pair、specialization dependency 和稳定 hash；Material preparation 已有 `published / previous_published / staged_candidate / rejected_candidate` 三槽状态，并以 material/shader/texture revision 与 draw generation 选择 last-good；Mesh Base pass 也已有 typed `PipelineAdmission::{Ready, Deferred, Failed}`、bounded miss report 和 generation-qualified requirement ledger。这些都是应保留的真实底座。

但工程级权威仍没有形成。当前选集是 175 个 Rust 文件、41,623 行、约 1.70 MiB；`MeshPipelineCache` 一处同时拥有 shader module、八组以上 pass PSO HashMap、variant registry、layout/fallback 资源、两个私有编译线程、driver cache、publication ledger 和诊断。与此同时，graphics/RHI 仍有约 55 个 `create_render_pipeline` 调用分散到 49 个文件、27 个 `create_compute_pipeline` 调用分散到 24 个文件。Shader source disk cache、render-pipeline graph cache、compute cache、Mesh pass cache 与 driver cache各自定义 identity、预算和失败语义，没有唯一 `ShaderArtifact -> ProgramArtifact -> PipelineArtifact` authority。

因此 09C 的七项父 P0 仍不能关闭：P0-2 的 source-cache identity、P0-4 的三槽 material publication、P0-5 的 reflection/ABI admission 和 P0-7 的 Base typed admission已有实质进展，但都只闭合局部子边界；P0-1/P0-3/P0-6 仍为 Open。Runtime189 不重复新增 P0，登记 36 项 P1（28 Open / 8 Partial）、14 项 P2 和 30 个资格门（20 Fail / 10 Partial / 0 Pass），把下一轮实现限定为权威收敛和硬切迁移，而不是继续给 Mesh 大对象增加字段。

## 2. 审查范围与证据边界

### 2.1 逐文件 owner 链

| 链 | 当前文件 | 核验内容 |
|---|---|---|
| Asset/source | `asset/assets/{material,shader}`、shader importer/package/cache payload | source schema、kind、entry、dependency、property、layout、readiness、migration |
| Material runtime | `graphics/material`、`scene/resources/{prepared,runtime,resource_streamer}` | ABI、texture/property binding、fallback、candidate/last-good、revision、GPU resource |
| Shader/variant | `graphics/shader`、`graphics/pipeline` | template、reflection、source cache、compile scheduler、variant identity、failure |
| Pipeline graph | `graphics/render_pipeline_asset`、render graph consumers | declaration/compile、local cache、pipeline construction、device profile |
| Mesh/PSO | `scene_renderer/mesh/mesh_pipeline_cache` | pass caches、admission、publication ledger、error proxy、prewarm、retirement |
| Cross-product | 其它 graphics/RHI 的 render/compute pipeline creation | 是否存在唯一 authority、是否绕过 generation/cache/scheduler |

### 2.2 证据等级

- **E3**：读取当前工作树生产代码、调用点和 owner 状态，不把测试构造或 descriptor 注册视为产品完成。
- **E2**：对照 09C/Runtime91 的历史结论以及本地 Unreal、Bevy、Godot、Fyrox、Unity Graphics 源码。
- **E1**：静态计数和测试文件只证明结构意图；本轮没有运行 Cargo、WGPU/DX12、热重载、device-loss、RenderDoc、scale 或 benchmark。
- **E0**：没有证据支持“性能和表现优于 Unreal”；当前只能确定结构性风险、正确性缺口和验收方法。

## 3. 当前可保留底座

1. `PreparedMaterial` 的 published/previous/staged/rejected 状态把“当前 source”与“可绘制 last-good”分开，方向正确；后续应将其提升为跨 pass、跨 renderer、带 device generation 的 artifact publication，而不是删除。
2. `PublishedMaterialDrawProxy` 以选定 generation 固化 runtime、texture 和 uniform，可作为 render submission 的 immutable draw binding。
3. `ShaderTemplateReflection` 已由同一 Naga module/module-info 推导 entry、resource、stage I/O、layout hash 与 specialization dependency，具备形成 canonical program artifact 的信息基础。
4. `ShaderSourceValidationAdmission` 能对 Mesh entry、vertex/fragment ABI、resource binding 和 attachment contract fail-closed；应抽离 Mesh 私有边界并由所有 pipeline consumer复用。
5. source disk cache v2 已把最终 WGSL hash、include content hash、template revision、Naga/WGPU version 纳入 source identity，并在命中后重算 payload hash。
6. `CompiledGraphCache` 和 graph execution compute cache 都有 capacity=16 的局部 LRU，证明 bounded cache 模式可行；问题是它们没有共享 identity、byte budget 和 device retirement。
7. Base pass typed admission、bounded miss report、resolved-PSO generation pin 与 submission ticket是真实正确性底座，但不能只覆盖 Mesh Base/OIT。

## 4. 父 P0 当前重判

| Canonical owner | 状态 | 当前重判与硬切要求 |
|---|---|---|
| `09C-P0-1` 唯一 Shader artifact / PSO authority | Open | RHI已有真实 native registry，但 renderer 仍在数十文件直接创建 module/pipeline；所有路径必须迁到统一 artifact/pipeline service。 |
| `09C-P0-2` 完整 cache identity | Partial | generated WGSL source key 已加强；compiled program、layout、selected constants、target/device/driver、render state 与 PSO identity 仍未统一。 |
| `09C-P0-3` 共享编译调度与非阻塞 render path | Open | Mesh仍创建两个私有 OS 线程，completion channel无界，`finish_pending*`和 Drop join可阻塞；其它 pass 多为同步创建。 |
| `09C-P0-4` 原子 generation / reverse dependency / last-good | Partial | Material已有三槽发布和resolved PSO pin；作用域仍以 Mesh/viewport为主，不能证明所有 pass、device loss、submission retirement原子一致。 |
| `09C-P0-5` readiness / reflection / ABI | Partial | kind、surface contract、Naga reflection与Mesh admission明显增强；readiness本身仍不等于 specialized artifact/actual pipeline ready，死 `pipeline_layout` DTO仍在。 |
| `09C-P0-6` Material/Shader Graph产品与唯一 schema | Open | canonical MaterialGraph与optional ShaderGraph仍是两套模型；后者生成未验证 WGSL且executor为no-op。 |
| `09C-P0-7` 可见 failure / fallback policy | Partial | Base已有typed admission和generation error proxy；其它pass/compute仍各自返回 Option、panic、skip或同步等待，跨pass fallback未原子化。 |

## 5. P1 工程化差距（36 项）

### 5.1 Artifact authority、identity 与 layout

| ID | 状态 | 当前证据与差距 | 必须重构为 |
|---|---|---|---|
| MSP4-P1-001 | Open | 没有进程级唯一 `ShaderArtifactId`；asset snapshot、assembled source、reflection、module 与 PSO分别缓存。 | versioned `ShaderSourceArtifact`、`SpecializedProgramArtifact`、`PipelineArtifact`，每层只有一个 owner 和明确输入 hash。 |
| MSP4-P1-002 | Open | render/compute pipeline创建仍分散，绕过 Mesh cache、driver cache和publication ledger。 | RHI-facing `PipelineArtifactService` 为唯一 creation入口；旧 direct WGPU callsite硬切删除。 |
| MSP4-P1-003 | Open | `MeshPipelineCache` 聚合 shader/layout/fallback/compiler/cache/publication/diagnostic 多域职责。 | 拆成 artifact resolver、compile scheduler、pipeline registry、material publication、fallback policy 和 metrics projection。 |
| MSP4-P1-004 | Open | Base/OIT/GBuffer/Depth/Hit/Velocity/Shadow/TAA/Mask等各有独立 HashMap，无统一 entry/byte/age/device budget。 | 统一 pipeline registry，key包含 target/pass，value带 state/bytes/last-use/pins/device generation。 |
| MSP4-P1-005 | Open | `MeshPipelineVariantRegistry` 的 HashMap/Vec/HashSet单调增长，ID耗尽会 panic。 | generation handle + bounded interner + tombstone/retirement；耗尽返回typed admission failure。 |
| MSP4-P1-006 | Open | variant key平台 token固定为 `wgpu-runtime`，不足以表达 adapter features/limits/backend/compiler/profile。 | `ShaderTargetProfileId`包含 backend、adapter/device、features/limits、compiler ABI、binding model、quality和selected constants。 |
| MSP4-P1-007 | Open | Render state、attachment ABI、layout、stage entry与specialization值没有一个 canonical PSO key。 | `PipelineArtifactKey`采用规范序列化并逐字段 exact compare，hash只作索引而非身份。 |
| MSP4-P1-008 | Partial | `ShaderAsset.pipeline_layout_descriptor()`仅自身和测试消费，生产布局来自其它 owner；迁移inventory已完成但删除尚未实施。 | 删除死 DTO；actual specialized reflection + pass layout owner 是唯一 ABI authority。 |

### 5.2 编译、调度与失败状态机

| ID | 状态 | 当前证据与差距 | 必须重构为 |
|---|---|---|---|
| MSP4-P1-009 | Open | `PipelineAsyncCompiler`按实例创建 OS thread，Mesh一处就有 shader validation和Base pipeline两套 worker。 | 接 Runtime job executor 的共享 `ShaderCompileScheduler`，按阶段和device affinity调度。 |
| MSP4-P1-010 | Open | request channel bounded，但completion `channel()`无界；调用方停收时结果可持续积压。 | request/completion/result bytes均计入同一 budget，consumer retirement可取消或丢弃旧代结果。 |
| MSP4-P1-011 | Open | FIFO无priority、deadline、cancel、supersede、project/view fairness。 | compile ticket携带priority/deadline/owner/generation/cancel；同key single-flight，旧代自动supersede。 |
| MSP4-P1-012 | Open | `finish_pending`/`finish_pending_through`阻塞，error-proxy requirement也显式同步finish。 | render/frame path只poll；required bootstrap在加载阶段预热，超过deadline进入typed degraded状态。 |
| MSP4-P1-013 | Open | compiler Drop关闭sender后无期限join；driver cache又在Drop同步持久化。 | 显式 shutdown phase，bounded drain/abort receipt；析构不得等待I/O、worker或GPU。 |
| MSP4-P1-014 | Partial | Base可异步且有typed admission，其他Mesh pass和大量post/compute仍同步创建。 | 所有pipeline target共用Queued/Preprocessing/Compiling/Creating/Ready/Failed/Retired状态机。 |
| MSP4-P1-015 | Open | panic、worker unavailable、queue full、WGPU error、stale generation的重试/终态策略分散。 | stable error code + recoverability + retry budget + stage provenance；同错误不按帧刷屏。 |
| MSP4-P1-016 | Open | shader/source、pipeline creation与driver cache没有统一correlation id。 | 一次source change贯穿source/artifact/job/PSO/material/draw receipt，能定位等待与失败阶段。 |

### 5.3 Cache、prewarm 与资源预算

| ID | 状态 | 当前证据与差距 | 必须重构为 |
|---|---|---|---|
| MSP4-P1-017 | Partial | source disk cache v2 identity较强，但只保存压缩WGSL，warm hit仍需Naga/module/PSO。 | 根据profile决定保留source层或升级为target-qualified compiled artifact；不得把source hit当PSO hit。 |
| MSP4-P1-018 | Open | cache lookup/decompress/write在调用线程做文件I/O。 | I/O阶段进入共享scheduler，带byte budget、cancel、deadline和cache miss reason。 |
| MSP4-P1-019 | Open | payload与meta分别atomic write，不能证明成对提交；rename目标已存在时可能被当作成功。 | 单manifest指向content-addressed payload，commit原子发布；冲突需重新验证完整内容。 |
| MSP4-P1-020 | Open | source cache没有entry/byte/age/project/device quota、lease或eviction。 | 多级cache budget + pin/lease + LRU/clock + orphan sweep + eviction receipt。 |
| MSP4-P1-021 | Partial | Vulkan driver cache能加载/保存最多64 MiB payload。 | backend-neutral provider；identity加入driver/API/device profile，load/persist异步且错误可观测。 |
| MSP4-P1-022 | Open | driver cache path主要按backend/vendor/device，缺driver version、OS/build、capability/compiler profile。 | provider生成可审计compatibility key；不兼容是明确miss，不依赖驱动内部静默拒绝。 |
| MSP4-P1-023 | Open | Drop持久化错误被忽略，无法区分未修改、保存失败、损坏和不支持。 | `PipelineCachePersistenceReceipt`记录bytes、duration、entry、reason、atomicity和last-good file。 |
| MSP4-P1-024 | Partial | graph/compute local cache有16-entry LRU，但key/owner不同，compile closure仍同步；compute key还保存完整source String。 | 共享artifact handle作为key，局部cache只保留execution specialization；统一byte/device retirement。 |

### 5.4 Shader readiness、dependency 与 material publication

| ID | 状态 | 当前证据与差距 | 必须重构为 |
|---|---|---|---|
| MSP4-P1-025 | Partial | `ShaderReadinessReport::is_ready`检查kind/runtime WGSL/entries/defs/diagnostic，但不要求specialized reflection与actual pipeline。 | 区分SourceReady、ArtifactReady、PipelineReady、Installed和Degraded；consumer声明需要的阶段。 |
| MSP4-P1-026 | Open | recursive shader import在 `!traversal.enter(shader_id)` 时成功早退，cycle可无SCC诊断地参与发布。 | dependency graph先做SCC；非法循环fail-closed，允许循环必须有显式module policy和稳定诊断。 |
| MSP4-P1-027 | Open | prepared shader从asset snapshot再复制source/import/generated WGSL，长source可能多份常驻。 | content-addressed immutable source blob + slices/Arc；artifact只持handle、offset和provenance。 |
| MSP4-P1-028 | Partial | `MaterialAsset`有固定PBR字段、typed schema和texture slot，也保留`BTreeMap<String,toml::Value>`动态值。 | canonical typed property table由shader reflection/schema生成；legacy fixed fields作为迁移projection而非第二authority。 |
| MSP4-P1-029 | Open | `.zmaterial`写出version 2，但graph/material/property/layout迁移没有单一migrator与write-current-only规则。 | schema registry + read-old/write-current migration receipt；unknown property可opaque保留并带origin。 |
| MSP4-P1-030 | Open | material generation是进程内计数，不能跨artifact store、cook、restart或device generation证明身份。 | `MaterialArtifactId`来自source/dependency/schema/target；draw generation只作live publication序号。 |
| MSP4-P1-031 | Open | 三槽publication主要由Mesh viewport需求驱动，post/particle/sprite/UI等material consumer不共享。 | renderer-neutral `MaterialPublicationService`收集全consumer requirements并原子admit全部required artifacts。 |
| MSP4-P1-032 | Partial | previous只保留一代；retirement已有resolved-PSO pin基础，但texture/bind group/module/device资源没有统一fence ledger。 | submission-qualified artifact pins覆盖program/pipeline/layout/bindings/textures；最后使用完成后才回收。 |
| MSP4-P1-033 | Open | cold/error proxy会同步完成validation；error material、pipeline、layout、binding之间仍可能局部可用。 | bootstrap时预建每target兼容的完整error bundle；切换必须是一个generation transaction。 |
| MSP4-P1-034 | Open | specialization dependency进入reflection hash，但selected override values没有形成所有PSO的exact identity。 | selected constants规范编码进入program/pipeline key、diagnostic和artifact provenance。 |
| MSP4-P1-035 | Open | device-loss后source/material/PSO/cache各自恢复，没有统一device generation reinstall/retire receipt。 | device generation切换触发artifact reinstall graph；旧GPU object不可被新submission引用。 |
| MSP4-P1-036 | Open | 当前测试集中在局部parser/reflection/cache/admission，缺跨pass热重载、1/100/10k variant、fault与visual证据。 | source->compile->publish->draw E2E矩阵，包含invalid reload/LKG/recovery、queue saturation、device loss、cache corruption和capture。 |

## 6. P2 性能、质量与维护（14 项）

1. **MSP4-P2-001**：为source bytes、IR bytes、module、layout、PSO、driver cache和material bindings建立统一resident-byte census。
2. **MSP4-P2-002**：variant key大量String/BTreeMap canonicalization需要interned symbol与stable binary key；先profile allocation与hash p95。
3. **MSP4-P2-003**：shader compilation可选进程隔离以处理编译器崩溃/内存峰值，但必须建立在共享ticket/artifact协议之后。
4. **MSP4-P2-004**：按target/profile支持离线批量编译与分布式worker；runtime仍必须验证provenance与device compatibility。
5. **MSP4-P2-005**：收集实际PSO usage trace、miss/debt和bind count，生成可合并、可裁剪的shipping prewarm set。
6. **MSP4-P2-006**：对pipeline cache采用hot/warm/cold层级，避免所有PSO永久驻留；阈值必须来自产品场景profile。
7. **MSP4-P2-007**：生成WGSL/IR保留node/source/include映射，支持capture、crash、diagnostic和Editor定位。
8. **MSP4-P2-008**：为shader source/include/graph设置byte、node、nesting、import depth与compile-time fuse，拒绝资源消耗攻击。
9. **MSP4-P2-009**：错误与miss诊断使用stable code/parameter，运行时只投影bounded human text，避免热路径格式化。
10. **MSP4-P2-010**：layout/bind group cache采用结构interning并记录collision exact compare，禁止仅靠`DefaultHasher` u64判等。
11. **MSP4-P2-011**：material property upload按dirty ranges/instance sharing/bindless profile优化，不能每次reload重建全部buffer/bind group。
12. **MSP4-P2-012**：把pipeline creation、cache I/O、Naga、driver compilation与first-present GPU bubble关联到同一trace。
13. **MSP4-P2-013**：对DX12/Vulkan/Metal和低规格adapter建立相同artifact输入、不同target profile的parity/fallback矩阵。
14. **MSP4-P2-014**：性能验收同时记录frame CPU/GPU、stutter、RSS/VRAM、I/O、功耗与视觉正确性；只减少某个计数不能宣称优于Unreal。

## 7. 参考引擎对照

| 参考 | 可吸收的工程合同 | Zircon当前差异 |
|---|---|---|
| Unreal RenderCore/RHI | `FShaderMapResource`分离code/resource，shader key携带material map、pipeline、vertex factory、permutation和platform；PSO file cache记录完整initializer、usage、precache result并支持incremental/bound save。 | Zircon的source/reflection/module/PSO/driver cache分散，Mesh key和其它pass又不一致，缺全renderer artifact authority与usage-driven prewarm。 |
| Bevy PipelineCache/ShaderCache | 单一`PipelineCache`维护Queued/Creating/Ok/Err，等待shader依赖并缓存layout/shader/pipeline；descriptor是状态机输入。 | Zircon只有Base形成typed admission，其它pass/compute仍是局部HashMap、Option或同步create。 |
| Godot ShaderRD/Material | shader version拥有dirty/valid/variant/group、compile task和cache路径；Material/ShaderMaterial将shader参数、RID和dirty update纳入resource生命周期。 | Zircon material三槽强于简单替换，但shader version、variant group、GPU install和material publication没有同一owner。 |
| Fyrox Material/Shader | shader定义resource bindings/pass，Material以typed property group/texture binding消费schema并作为共享resource。 | Zircon同时保留固定PBR字段和动态TOML值，schema到binding/inspector/runtime尚未成为唯一typed contract。 |
| Unity Shader Graph | GraphData维护GUID、target、validation、version/unknown target，Generator按target/pass/vertex/surface生成，preview使用独立scene与真实material。 | Zircon两套graph schema均未产出同一canonical artifact；optional ShaderGraph直接拼WGSL且pass executor no-op。 |

## 8. 目标架构

```text
ShaderSourceDocument / MaterialSourceDocument
  -> SourceDependencyGraph (SCC + immutable blobs)
  -> ShaderCompilerService (single-flight + budget + cancel)
  -> SpecializedProgramArtifact
       { source/dependency/compiler/target/constants/reflection/code }
  -> PipelineArtifactService
       { program + entries + layout + render state + attachments + device }
  -> MaterialArtifact / MaterialPublicationService
       { typed values + textures + required pipelines + LKG }
  -> RenderSubmissionArtifactPins
       { material generation + exact PSO/binding/resource + fence }
```

核心规则是四种 generation 不混用：source/build generation证明内容，artifact generation证明编译结果，device generation证明GPU安装，draw/publication generation只证明某次live选择。任何cache只能加速对应authority，不能成为第二事实源。

## 9. 重构顺序

1. **M189.0 清除 false-ready**：关闭09C-P0-6，停用或删除duplicate ShaderGraph runtime/no-op executor；Material Graph在无canonical compiler/artifact前标记Unavailable。
2. **M189.1 定义唯一 identity/artifact**：冻结source/program/pipeline/material key schema，删除死`pipeline_layout` DTO，建立migration inventory和write-current-only规则。
3. **M189.2 共享编译调度**：迁出Mesh私有线程和所有finish/wait/Drop阻塞，建立single-flight、priority、cancel、deadline、bounded completion。
4. **M189.3 统一pipeline registry/cache**：先迁Mesh全部pass，再迁post/compute/UI/particle等direct WGPU callsite；局部cache只能存artifact handle。
5. **M189.4 原子material publication**：将三槽/LKG提升到renderer-neutral owner，requirements覆盖所有consumer，submission pins覆盖完整GPU bundle。
6. **M189.5 hot reload/device loss**：SCC reverse dependency、candidate supersede、cross-pass admission、device generation reinstall与bounded retirement。
7. **M189.6 产品验收**：1/100/10,000 variants/materials，cold/warm/reload/corrupt/device-loss；DX12/Vulkan/Metal、capture、visual、timing、memory、power全部有报告。

## 10. 资格门（30 个）

| Gate | 状态 | 完成条件 |
|---|---|---|
| RT-MSP-G01 | Fail | 所有shader source/reflection/code由唯一artifact owner发布。 |
| RT-MSP-G02 | Fail | 产品代码不再直接创建shader module。 |
| RT-MSP-G03 | Fail | PSO identity覆盖program/entry/layout/state/attachment/constants/target/device。 |
| RT-MSP-G04 | Fail | 所有render/compute PSO通过唯一PipelineArtifactService。 |
| RT-MSP-G05 | Partial | actual layout与specialized reflection逐binding对拍，无死authored DTO。 |
| RT-MSP-G06 | Partial | reflection覆盖所有stage/consumer，不局限Mesh。 |
| RT-MSP-G07 | Fail | import graph先做SCC，非法cycle不能静默ready。 |
| RT-MSP-G08 | Partial | readiness明确区分source/artifact/pipeline/install/degraded。 |
| RT-MSP-G09 | Fail | 每个compile/pipeline有统一queued-to-retired状态机。 |
| RT-MSP-G10 | Fail | 所有编译共享scheduler和single-flight。 |
| RT-MSP-G11 | Fail | priority/cancel/deadline/supersede/fairness动态通过。 |
| RT-MSP-G12 | Fail | render/frame thread不等待编译、文件I/O或driver persist。 |
| RT-MSP-G13 | Fail | Drop不join worker、不持久化cache、不无限等待GPU。 |
| RT-MSP-G14 | Fail | request/result/source/IR/module/PSO均有entry+byte预算。 |
| RT-MSP-G15 | Fail | disk payload/manifest成对原子提交并可恢复。 |
| RT-MSP-G16 | Fail | cache identity包含完整target/device/driver/compiler profile。 |
| RT-MSP-G17 | Fail | cache/PSO有pin、eviction、retirement、orphan和device generation。 |
| RT-MSP-G18 | Partial | source cache identity已加强，产品profile证明其保留价值。 |
| RT-MSP-G19 | Fail | compiled program/PSO persistent cache有可验证provenance。 |
| RT-MSP-G20 | Partial | Material schema版本化、可迁移、unknown property可保留。 |
| RT-MSP-G21 | Partial | property/texture/option由同一typed schema驱动。 |
| RT-MSP-G22 | Partial | current/staged/rejected/LKG状态可证明且旧代可安全退役。 |
| RT-MSP-G23 | Fail | 所有required pass跨consumer原子admit，不出现半更新。 |
| RT-MSP-G24 | Partial | error proxy为完整兼容bundle，不以几何消失掩盖失败。 |
| RT-MSP-G25 | Partial | submission fence pin住program/PSO/layout/binding/texture完整集合。 |
| RT-MSP-G26 | Partial | direct/transitive shader/material/texture reload只发布一个新generation。 |
| RT-MSP-G27 | Fail | device loss/recreate不会复用旧GPU handle并能恢复LKG。 |
| RT-MSP-G28 | Fail | 1/100/10,000 cold/warm/reload/fault/soak无无界增长与stall。 |
| RT-MSP-G29 | Fail | capture/PNG/RenderDoc证明每pass输出、fallback和reload视觉正确。 |
| RT-MSP-G30 | Fail | CPU/GPU/RSS/VRAM/I/O/power基线可重复，才允许声称性能目标。 |

## 11. Review-only 交付边界

本轮只新增审查文档、索引和coverage，不修改Runtime/Editor/plugin/Cargo/ABI/ZUI，也没有执行Cargo、GPU、产品帧或性能验收。实施前必须重新核对当前working tree，因为相关目录已有大量并行修改；每个里程碑都应以旧direct authority删除、唯一artifact状态机可观测、失败可恢复和动态资格门通过为完成条件，不能以新增wrapper、HashMap或测试fixture代替。
