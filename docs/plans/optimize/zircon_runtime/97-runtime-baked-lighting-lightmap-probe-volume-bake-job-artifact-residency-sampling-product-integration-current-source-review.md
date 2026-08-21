---
title: Runtime Baked Lighting、Lightmap、Probe Volume、Bake Job、Artifact、Residency、Sampling 与 Product Integration 当前源码工程化差距
category: zircon_runtime
report_id: Runtime97
review_date: 2026-08-21
baseline_head: be5a281c96b6dc9d33b5c9d0f2699a8bf75afcf1
baseline_epoch: 336
related_code:
  - zircon_runtime/src/core/framework/render/environment/lightmap.rs
  - zircon_runtime/src/core/framework/render/environment/extract.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/irradiance_volume.rs
  - zircon_runtime/src/asset/assets/texture/lightmap_asset.rs
  - zircon_runtime/src/asset/assets/scene/lighting.rs
  - zircon_runtime/src/asset/assets/mesh/mesh_asset.rs
  - zircon_runtime/src/asset/importer/ingest/import_gltf.rs
  - zircon_runtime/src/scene/components/scene/lighting.rs
  - zircon_runtime/src/scene/world/render/lights.rs
  - zircon_runtime/src/graphics/runtime/offline_bake
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/lightmap_binding.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/irradiance_volume
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/gpu_scene_sync.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_lightmap.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_irradiance_volume.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_forward.wgsl
  - zircon_runtime/src/graphics/shader/wgsl/zr_template_deferred_gbuffer.wgsl
  - zircon_plugins/rendering/features/baked_lighting
  - zircon_plugins/rendering/features/irradiance_volumes
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/rendering/workbench_extension_lighting_bake_workspace.zui
tests:
  - zircon_runtime/src/core/framework/render/environment/lightmap/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/lightmap_binding/tests.rs
  - zircon_runtime/src/graphics/tests/render_product_baked_lighting.rs
  - zircon_runtime/src/graphics/tests/fixtures/plan11_baked_lighting_v1.json
  - zircon_runtime/src/graphics/tests/m4_behavior_layers.rs
  - zircon_runtime/src/graphics/tests/render_product_advanced_lighting/af_m2.rs
  - zircon_runtime/src/asset/tests/assets/gltf_importer/vertex_channels.rs
  - zircon_runtime/src/asset/tests/assets/mesh/conversion_import.rs
  - zircon_editor/src/tests/host/manager/minimal_host_contract/optional_features.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_runtime/09f2-baked-lighting-lightmap-irradiance-volume-review.md
  - docs/plans/optimize/zircon_runtime/89-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/91-runtime-material-shader-module-graph-permutation-compiler-reflection-layout-pipeline-pso-cache-prewarm-hot-reload-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/92-runtime-texture-image-cubemap-array-volume-format-sampler-mip-compression-upload-streaming-residency-budget-eviction-virtual-texture-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/94-runtime-visibility-spatial-index-bounds-frustum-occlusion-hzb-culling-batching-instancing-gpu-scene-indirect-submission-instance-lifecycle-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/96-runtime-environment-sky-atmosphere-cloud-ibl-reflection-probe-capture-convolution-sh-pmrem-cache-residency-submission-product-integration-current-source-review.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
  - docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/MapBuildDataRegistry.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/LightMap.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/LightMapRendering.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/PrecomputedVolumetricLightmap.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/PrecomputedVolumetricLightmapStreaming.cpp
  - dev/UnrealEngine/Engine/Plugins/Experimental/GPULightmass/Source/GPULightmass
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/PathTracing/BakeLightmapDriver.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/PathTracing/Lightmapping/UVOverlapDetection.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Lighting/ProbeVolume
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Lighting/ProbeVolume
  - dev/bevy/crates/bevy_pbr/src/lightmap
  - dev/bevy/crates/bevy_pbr/src/light_probe/irradiance_volume.rs
  - dev/godot/scene/3d/lightmap_gi.h
  - dev/godot/scene/3d/lightmap_gi.cpp
  - dev/godot/editor/scene/3d/lightmap_gi_editor_plugin.cpp
  - dev/godot/modules/lightmapper_rd/lightmapper_rd.h
  - dev/Fyrox/fyrox-impl/src/utils/lightmap.rs
  - dev/Fyrox/fyrox-impl/src/utils/uvgen.rs
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Runtime Baked Lighting、Lightmap、Probe Volume、Bake Job、Artifact、Residency、Sampling 与 Product Integration 当前源码工程化差距

## 1. 结论

当前Zircon已经具备一条可由测试手工装配的baked-lighting消费路径：glTF `TEXCOORD_1`能够进入`MeshVertex::uv1`和GPU vertex ABI；`LightmapBakeOutput`能够被反序列化为RGBA16F array texture、instance atlas slot与SH9 uniform probe grid；Static mobility mesh可在GPU scene中获得lightmap参数；Forward和Deferred shader都能采样lightmap、uniform probe grid与单个irradiance volume。`render_product_baked_lighting.rs`也用外部JSON fixture证明了这些消费算子在手工场景中可见。这些不是空壳，应保留为characterization oracle。

但完整产品链仍不存在。普通scene/prefab/project不能保存lightmap build settings、静态灯光贡献、Stationary/shadowmask、build-data引用或irradiance-volume source；`World`不会创建`LightmapBakeRequest`，不会把持久化build data装入`EnvironmentExtract`，生产代码也没有`LightmapBakeOutput` producer。唯一名为`offline_bake_frame`的函数只把方向光强度求和，再为前N个mesh制造没有`baked_cubemap`的reflection probe；它既不生成lightmap，也不生成probe volume。其M4测试期望结果改变像素，而reflection-probe资源准备明确丢弃没有cubemap的probe，因此当前源码存在静态可证明的测试/实现矛盾。

Editor表面不是实际工具。Lighting Bake `.zui`硬编码`City_Block_A`、87 assets、4 warnings、02:30等样例数据；Bake与Preview action只由通用feedback函数把状态字符串改成“queued”。没有任务句柄、进度、取消、失败、重试、scene dirty、undo/redo、artifact写入或runtime preview发布。Baked Lighting插件只注入一个读写scene-color的no-op postprocess pass；Irradiance Volume插件是可选executor，但core renderer仍独立选择和准备单个volume，mesh shader include也始终存在。关闭插件不等于关闭真实采样路径。

消费ABI本身也不能直接升级为工程产物。`LightmapConsumeContract`公开可变`Vec`并在每帧GPU scene同步时重建`HashMap`；atlas resident cache只按`ResourceId`命中，probe buffer只按数值generation命中，shader却不读取上传的generation header。`LightmapBakeSceneSnapshot.content_hash`不复算，payload完全不透明；output允许漏掉request中要求的instance。atlas只有单mip raw RGBA16F，没有chart/gutter/dilation/压缩/cook/streaming；shader对UV做全局clamp，Deferred把baked diffuse加进emissive。uniform probe grid每个fragment读取8个probe乘9个SH coefficient，并在边界外突变为零。

旧`09F2`登记的 **12项P0全部保持开放**。本轮没有重复新增父级P0，而是对当前源码登记 **36项P1、8项P2与44个资格门**，并修正两处历史事实边界：Zircon并非没有UV1通道，而是只有通道搬运、没有lightmap UV工程；Zircon并非没有任何mobility，而是有通用Static/Dynamic mobility、没有Static/Stationary baked-light语义。未完成scene/editor/job/build-data/cook/residency/shader/diagnostic闭环及同硬件同画质对照前，不得声称baked lighting达到或优于当前Unreal。

## 2. 审查边界、currentness与证据

### 2.1 冻结语料

| 范围 | 文件 / 行 / 非空行 / bytes / test markers | 证据等级 | fingerprint |
|---|---:|---|---|
| Zircon baked-lighting产品语料 | **77 / 15,735 / 14,606 / 690,650 / 79** | E3逐文件覆盖DTO、scene、mesh import、asset、offline bake、GPU scene、binding、shader、plugin、Editor表面与owner计划 | `f53750186ac3870c75f5f1a90fe723a069f86d28a66b0733719daa9e181a26fd` |
| focused tests与fixture | **9 / 2,554 / 2,355 / 90,731 / 35** | E3读取lightmap contract/binding、glTF UV1、外部fixture、advanced-lighting与Editor capability测试 | `b2bb4cca780e36957c3b2c6b87a10b4dc8fa06be29bb7a82432db47d0b3b715b` |
| 五引擎参考切片 | **31 / 26,018 / 21,742 / 1,133,143 / 8** | E2/E3读取Unreal build registry/GPULightmass/VLM、Unity APV、Godot LightmapGI、Fyrox baker与Bevy消费侧 | `0990fd47f4a46704ca27172d9fc178831dfad09aa46a746b4a950c40ac7c1141` |

fingerprint算法为：规范化小写相对路径与每文件SHA-256组成排序manifest，再对UTF-8 manifest执行SHA-256。冻结对象是2026-08-21共享working tree，不是只读HEAD。基线HEAD为`be5a281c96b6dc9d33b5c9d0f2699a8bf75afcf1`，coordinator epoch为336。

Bevy、Fyrox、Godot与Unity Graphics revision分别为`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`与`a7e4c051d256a781ab362c64316b125a1e104694`。Unreal目录不是独立Git checkout，本文只用参考切片aggregate fingerprint冻结，禁止把父仓revision伪装成Unreal revision。

冻结时`environment/lightmap.rs`与`environment/extract.rs`存在其他Session或用户的working-tree修改。本文审查并冻结的是这些当前版本，没有拥有或修改它们；实施或动态验收前必须重算主语料fingerprint，并重新审计DTO validation、generation与scene attach路径。

### 2.2 Owner边界

| 边界 | Runtime97要求的owner | 禁止的越界 |
|---|---|---|
| Authoring source | `zircon_runtime::scene`拥有可序列化light build settings、mesh/light contribution、probe volume、scenario与build-data引用 | 不把Editor sample state或graphics DTO当scene truth；不让graphics反向持有World对象 |
| Neutral contract | `core::framework::render`只发布typed、versioned、generation-qualified handle/descriptor/readiness | 不携带raw atlas page、opaque scene payload或WGPU资源；不允许公开可变slot列表 |
| Bake producer | rendering plugin通过稳定service拥有bake manifest、backend、job、progress、cancel、resume与diagnostic | 不用`offline_bake_frame`伪装跨帧baker；不把fixture deserialize当生产管线 |
| Build data | asset/derived-data owner拥有content-addressed recipe、依赖图、atomic publish、cook与migration | 不把数值generation或`ResourceId`当内容identity；不在frame路径同步生产产物 |
| Runtime graphics | graphics拥有resident generation、upload、streaming、assignment、shader binding、fence与device恢复 | 不在core与plugin重复选择/prepare；不吞掉streaming失败；不把baked diffuse写成emissive |
| Editor product | `zircon_editor`拥有真实命令、事务、进度、取消、preview、诊断、dirty与save/reopen | 不以硬编码`.zui`和状态字符串宣称工具完成 |
| 历史P0 | 旧09F2继续唯一计数12项父P0；Runtime97只重验并细化 | 不因同一根因在多个层可见而重复累计P0 |

### 2.3 明确未做

本轮只做静态review，没有修改Rust、WGSL、Cargo、plugin manifest、scene asset或Editor UI，没有运行Cargo、Editor、WGPU/GPU capture、RenderDoc/PIX、bake job、cook、streaming、device loss、OOM、large-world、visual golden或性能基准。静态源码可以证明生产调用链缺失、feature gate失真、cache key不足、shader语义与测试矛盾，不能替代真实硬件的最终画质、耗时与内存证据。

## 3. 当前应保留的真实基础

1. glTF `TEXCOORD_1`到`MeshVertex::uv1`、mesh attribute与GPU vertex layout的贯通应保留；新UV pipeline必须在此基础上增加derived channel与validation，不应破坏已有author UV。
2. `LightmapBakeRequest`、`LightmapBakeOutput`、`LightmapConsumeContract`的三阶段意图正确，可升级为typed manifest/artifact/resident contract，但必须hard cutover掉opaque payload、公开Vec和弱identity。
3. `EnvironmentExtract::try_with_baked_lighting`对lightmap/probe generation一致性的拒绝方向正确，应扩展为artifact generation与resident generation的原子发布门。
4. Static mobility才绑定lightmap slot的gate、stable instance key与GPU scene布局可作为迁移起点，但build subject identity必须独立于临时entity ordinal。
5. SH9 CPU sampler与WGSL布局已有数值测试，可以保留为uniform-grid reference oracle，不应把当前O(72 fetch)实现当最终GPU方案。
6. Irradiance volume world-to-local transform、normal变换、priority、layer与ambient-cube packing已有测试，适合保留为单volume correctness oracle。
7. Forward/Deferred产品fixture能证明消费ABI可见，应转成真实bake产物的端到端回归，并增加物理通道与feature-off断言。
8. Bevy明确也是外部baker消费侧，适合对照Rust extraction、bindless slab与capability fallback，不可用它证明Zircon已经拥有Unreal级producer。

## 4. 历史09F2 P0 current-source重验

| 父finding | 当前状态 | current-source证据与关闭要求 |
|---|---|---|
| 09F2-P0-1 `offline_bake_frame`是假bake能力 | 开放 | 当前输出仍只有`Vec<ReflectionProbeData>`；没有lightmap/probe volume，且probe无cubemap会被资源准备丢弃。关闭需删除错误公共表面并接入真实job/service |
| 09F2-P0-2 scene/prefab/World无静态光照authoring | 开放 | scene light只有普通参数与通用mobility；World无build data attach。关闭需save/reopen/cook/runtime roundtrip |
| 09F2-P0-3 Editor与editor plugin是占位 | 开放 | Lighting Bake workspace硬编码样例，action只改字符串；插件只注册descriptor/capability。关闭需真实事务、任务、进度、取消、失败和产物发布 |
| 09F2-P0-4 无UV展开、packing与可烘焙验证 | 开放但修正文案 | UV1传输已存在；没有unwrap、chart、overlap、density、gutter、dilation或atlas planner。关闭需author/derived UV策略与故障诊断 |
| 09F2-P0-5 无真实baker与可恢复job | 开放 | 没有生产`LightmapBakeRequest`消费者或`LightmapBakeOutput`producer。关闭需CPU reference、GPU backend、scheduler、cancel/resume与determinism |
| 09F2-P0-6 无Build Data Registry与失效图 | 开放 | 只有内存DTO/texture转换；无recipe/backend/platform/dependency identity、atomic artifact或增量invalidations |
| 09F2-P0-7 无Static/Stationary/shadowmask/双重照明语义 | 开放但修正文案 | 通用mobility存在；无Stationary与shadowmask，ambient的`affects_lightmapped_meshes`在render snapshot被丢失，shader仍叠加direct+baked |
| 09F2-P0-8 plugin不拥有真实开关 | 开放 | baked plugin executor为no-op；关闭quality feature只影响零值postprocess参数，不移除mesh shader采样或core volume准备 |
| 09F2-P0-9 generation未贯穿GPU原子发布 | 开放 | atlas key只看asset ID，probe key只看generation；shader不验证generation，无法证明同一frame不会混用新slot/旧texture |
| 09F2-P0-10 single-view irradiance winner错误 | 开放 | core与plugin都按整个view的mesh origin选一个volume；重叠volume、大世界和不同object无法正确分配 |
| 09F2-P0-11 Probe Volume无producer/validity/leak/streaming | 开放 | scene无source，只有手工D3 texture；无placement、visibility、validity、dilation、virtual offset、brick/cell residency |
| 09F2-P0-12 无readiness/diagnostic/budget/竞争性验收 | 开放 | streamer吞掉volume ensure错误，Editor数据为假的，测试只证明fixture消费；无bake/resident/streaming telemetry和同画质基准 |

## 5. 当前产品链与断点

```text
glTF TEXCOORD_1 ------------------------------> MeshVertex.uv1 -> GPU vertex
                                                    |
external JSON fixture -> LightmapBakeOutput --------+-> TextureAsset + ConsumeContract
                                                          |
manual EnvironmentExtract.try_with_baked_lighting --------+-> GPU scene slot
                                                          |-> atlas/probe bindings
                                                          `-> Forward/Deferred sample

Scene/Prefab authoring ----X----> BakeManifest
Light/mesh build settings -X----> BakeJob/backend
Build Data Registry -------X----> versioned/cooked artifact
World load ----------------X----> EnvironmentExtract attach
Editor Bake action --------X----> job/progress/cancel/publish
Quality/plugin disable ----X----> actual shader/resource removal
```

上图中的实线只对手工fixture成立。产品目标必须是“scene truth -> deterministic manifest -> resumable bake -> immutable artifacts -> atomic registry publish -> cooked/streamed resident generation -> per-object assignment -> physically separated shading”，而不是继续给当前DTO增加更多optional字段。

## 6. P1重构项

### 6.1 Contract、identity与artifact

| ID | 当前差距 | 必须重构 |
|---|---|---|
| P1-01 | `LightmapConsumeContract.slots`为公开可变`Vec`，lookup线性；GPU scene每帧复制成`HashMap` | hard cutover为不可变、已验证、排序或perfect-hash/packed range的resident assignment table，增量更新而非逐帧重建 |
| P1-02 | slot用`render_mesh_stable_instance_key(entity, primitive ordinal)`，没有build subject、asset revision、submesh topology identity | 建立`BuildSubjectId`与运行时instance mapping，mesh topology/material/transform policy变化可精确失效 |
| P1-03 | `validate_against`只拒绝额外instance，不拒绝request中instance缺失 | 输出必须携带per-subject terminal status，并对完整性、partial build policy和fallback显式建模 |
| P1-04 | snapshot接受调用者提供的非零hash，不从canonical payload复算；payload无schema | 使用canonical typed manifest与content hash，记录scene/mesh/material/light/environment依赖和schema migration |
| P1-05 | request缺少backend、quality recipe、seed、bounce、denoiser、directionality、shadowmask、platform/cook target | 建立版本化`BakeRecipe`与capability negotiation，所有影响结果的参数进入artifact key |
| P1-06 | output只有atlas/raw pages/slot/optional uniform grid，没有producer版本、dependency graph、section checksum或兼容范围 | 定义分section不可变artifact、checksum、schema/backend版本、platform variant与可局部加载目录 |

### 6.2 Atlas、resident generation与hot reload

| ID | 当前差距 | 必须重构 |
|---|---|---|
| P1-07 | atlas descriptor只校验非零page size/count和单一RGBA16F格式 | 校验设备上限、尺寸、layers、mips、row pitch、format capability、byte budget与artifact section一致性 |
| P1-08 | atlas cache只按`ResourceId`早退；同ID内容修订不会重建 | key包含artifact content/revision、device generation与view format；完成upload/fence后原子换代 |
| P1-09 | probe buffer只按数值`light_set_generation`早退，同generation内容漂移不可见 | 使用不可伪造artifact generation/content key，并加入stale completion拒绝 |
| P1-10 | generation写进GPU header与instance params，但WGSL不读取 | shader/assignment必须验证同一resident generation，或删除死字段并由bind-group generation保证一致性 |
| P1-11 | texture转换复制并排序所有raw RGBA16F page，CPU/GPU两份生命周期不清 | artifact decoder输出prepared upload sections，使用staging scheduler、byte accounting与可释放CPU backing |
| P1-12 | 单mipRGBA16F array没有平台压缩、mip policy、streaming或virtualization | 建立directional/occlusion格式族、mip/encoding/cook variants、residency budget及fallback tier |

### 6.3 UV、sampling与物理通道

| ID | 当前差距 | 必须重构 |
|---|---|---|
| P1-13 | importer缺失UV1时填零；asset只验证类型/长度，不判重叠、退化或越界 | 区分author UV、derived UV与invalid UV；实现unwrap、chart validation、overlap heatmap和actionable diagnostic |
| P1-14 | slot只有`uv_rect`和page，没有chart border、texel density、rotation、gutter或mip-safe bounds | atlas planner输出chart metadata、padding/dilation、texel-center transform与LOD-safe sampling bounds |
| P1-15 | WGSL对变换后UV做全局`clamp(0,1)`，不能阻止相邻chart/mip bleed | 依据chart inner rect与mip clamp采样，加入seam/dilation/bicubic capability及visual regression |
| P1-16 | Deferred把baked diffuse写入GBuffer emissive | 为indirect diffuse、baked direct、shadowmask/occlusion建立独立物理channel，deferred lighting阶段统一组合 |
| P1-17 | lightmap只存无语义RGB irradiance，不记录方向性、dominant direction、sky occlusion或参与灯光 | 定义可演进encoding与decode，按目标画质提供directional lightmap、occlusion和light identity |
| P1-18 | ambient authoring的`affects_lightmapped_meshes`在`RenderAmbientLightSnapshot`中丢失 | 统一所有light contribution truth并贯穿manifest、runtime direct-light mask与debug视图 |
| P1-19 | fallback shader无条件叠加ambient、direct、environment与baked；无per-light排除 | 以Static/Stationary/Dynamic及baked light set决定direct contribution，防止double lighting |

### 6.4 Feature、plugin与产品可达性

| ID | 当前差距 | 必须重构 |
|---|---|---|
| P1-20 | `baked_lighting_enabled`的两个分支都返回零值postprocess extract | 删除伪参数；feature必须控制真实pipeline permutation、resource request、assignment与readiness |
| P1-21 | fallback与template assembly始终拼接lightmap/volume WGSL | 建立compiled feature permutation与layout contract；feature-off不得分配、绑定或采样相关资源 |
| P1-22 | baked plugin添加scene-color no-op pass，不能拥有bake或runtime消费能力 | plugin改为注册bake service/backend与runtime capability，不再用空pass制造feature presence |
| P1-23 | World/scene没有生产调用`try_with_baked_lighting`，只有测试手工注入 | scene load解析build-data handle，由asset/residency service发布ready generation并进入render extract |
| P1-24 | `offline_bake_frame`产生无cubemap reflection probe，命名、输出与测试预期都错误 | 删除错误公共API；reflection capture归Runtime96，light build归独立bake job，不互相伪装 |
| P1-25 | Editor workspace的队列、warning、时间与场景名都是静态文本 | UI投影真实job/query model，显示可追踪subject、阶段、吞吐、ETA、warning source和artifact状态 |
| P1-26 | Preview/Bake action只改feedback字符串 | 命令连接事务化service，支持cancel/retry/resume、dirty/save、preview generation与失败详情 |

### 6.5 Probe grid、irradiance volume与streaming

| ID | 当前差距 | 必须重构 |
|---|---|---|
| P1-27 | uniform SH9 grid每fragment手工取8 probe × 9 coefficient | 依据能力选择hardware-filterable encoding、brick pool/indirection或压缩SH，建立fetch与带宽预算 |
| P1-28 | grid只支持一个规则AABB，出界直接零，且无validity/visibility/occlusion | 引入分层cell、validity、geometry-aware placement、leak reduction与空间fallback/blend |
| P1-29 | core按view内任一mesh origin选择单个最高优先volume | assignment改为per-object/per-cluster候选列表，支持overlap、blend、large-world origin与deterministic tie-break |
| P1-30 | core renderer与irradiance-volume plugin各自收集mesh translation并选择/prepare | 指定唯一assignment/resource owner，删除双重Vec分配和双重prepare |
| P1-31 | volume layer compatibility用于view级候选，不保证每个object layer正确 | assignment记录object/volume layer交集，shader只访问该object获批列表 |
| P1-32 | resource streamer遍历所有volume，复用LUT texture store并丢弃ensure错误 | 建立typed irradiance artifact/resource class、visibility/priority驱动请求和结构化失败/readiness |
| P1-33 | D3资源只检查尺寸可被`1×2×3`整除，不检查format、encoding、mip、color space或hash | artifact descriptor精确定义ambient cube/SH encoding、format、dimensions、checksum与decode version |
| P1-34 | 单texture/uniform每次prepare写入，无brick/cell streaming、budget、eviction或device recovery | 接入09D/92资源系统，支持cell residency、LRU/priority、fence-safe eviction与device generation重建 |

### 6.6 Readiness、tests与资格证据

| ID | 当前差距 | 必须重构 |
|---|---|---|
| P1-35 | 没有bake queue、artifact、resident、assignment、streaming的统一readiness/telemetry | 发布typed counters、reason code、memory/IO/GPU budget、stale/fallback状态并接入Editor diagnostics |
| P1-36 | 产品测试由JSON fixture、手工texture注册与手工EnvironmentExtract注入组成；feature-off与真实producer未覆盖 | 增加save/reopen -> bake -> publish -> cook -> load -> stream -> Forward/Deferred的端到端测试、fault injection与golden |

## 7. P2演进项

| ID | 演进项 | 前置条件 |
|---|---|---|
| P2-01 | 分布式、多GPU与远程worker baking | manifest/artifact确定性、可恢复job与安全CAS先完成 |
| P2-02 | 自适应采样、ray guiding、firefly rejection与高质量denoise | CPU reference、GPU backend parity与质量metric先稳定 |
| P2-03 | Virtual Lightmap、稀疏页表与超大世界分层streaming | artifact section、resident generation与GPU feedback先完成 |
| P2-04 | Lighting Scenario、day/night、多状态切换与区域混合 | build-data registry、scenario identity与原子发布先完成 |
| P2-05 | 硬件光追reference baker、software fallback与跨vendor determinism | backend contract、capability与benchmark corpus先完成 |
| P2-06 | 神经重建/压缩与感知质量驱动encoding | 物理channel、golden、误差指标和传统fallback先完成 |
| P2-07 | 增量局部rebake、交互式Bake What You See与viewport priority | dependency invalidation、tile/cell job和partial publish先完成 |
| P2-08 | 跨机器共享DDC、产物签名、审计与farm quota | content-addressed artifact、schema migration和权限模型先完成 |

## 8. 目标架构

### 8.1 Scene truth

Scene保存`LightBuildSettings`、mesh `StaticLightingContribution`、light `BakedLightingMode`、probe-volume source、scenario和`LightBuildDataHandle`。Editor只通过command/transaction修改这些source，不能保存runtime texture或队列状态。

### 8.2 Canonical BakeManifest

Asset/build owner从冻结scene revision生成typed manifest：subject IDs、canonical geometry/material/light/environment hashes、UV policy、quality/backend/cook target、dependency edges和deterministic seed。manifest自身content-addressed，任何结果变化都能解释到具体输入。

### 8.3 Geometry preparation

专用pipeline完成author UV validation、derived unwrap、chart packing、gutter/dilation、texel density和atlas/page planning，产出可复用geometry artifact及Editor heatmap/diagnostic，不把临时patch写回source mesh。

### 8.4 Bake service与backend

Rendering plugin注册backend；runtime service拥有job graph、progress、cancel、resume、checkpoint、memory/GPU budget和fault recovery。先实现确定性CPU reference，再实现GPU/path-traced backend；二者共享manifest、artifact与quality metric。

### 8.5 Build Data Registry

Registry以scene/partition/scenario和stable subject identity映射不可变artifact generations，负责dependency invalidation、atomic commit、garbage collection、migration、cook与World Partition/large-world定位。禁止用裸generation integer替代content identity。

### 8.6 Runtime residency与assignment

Asset streaming把artifact sections异步转换为device-generation-qualified resident resources；完成upload/fence后一次性发布atlas、slot、probe cells和shader descriptor。GPU scene或cluster assignment生成per-object lightmap/probe引用，禁止view winner与逐帧重建HashMap。

### 8.7 Physical shading

Surface/Deferred明确分离emissive、baked indirect diffuse、baked direct、shadowmask、sky occlusion和dynamic direct。Static/Stationary/Dynamic的light identity决定哪些实时灯光被排除或用shadowmask调制；Forward与Deferred共享同一decode oracle。

### 8.8 Editor与observability

Lighting Bake workspace查询真实scene settings、job、artifact和resident状态；支持选中问题subject、UV/texel/probe visualization、preview generation、cancel/retry/resume、save/reopen、日志与可导出报告。所有状态来自typed service，不来自硬编码模板。

## 9. Hard Cutover规则

1. 删除`offline_bake_frame`错误公共能力；不保留compat alias、空wrapper或“以后实现”的同名surface。
2. 删除baked-lighting no-op scene-color pass；feature readiness必须来自真实service/resource/pipeline。
3. `LightmapConsumeContract`切到不可变resident descriptor后，删除公开`slots Vec`与双轨lookup，不维护旧新两套写法。
4. scene/build-data schema只保留一个authoritative版本迁移入口，不在Editor、plugin和graphics各复制一套settings。
5. irradiance-volume assignment只保留一个owner；core与plugin当前重复路径必须一次切除。
6. Deferred物理channel切换后删除“把baked diffuse塞入emissive”的兼容decode。
7. artifact identity统一为content/backend/schema/platform key后，删除只按`ResourceId`或裸generation命中的cache。
8. Editor接入真实job后删除所有City_Block_A、87 assets、4 warnings和02:30等演示常量。

## 10. 分层实施里程碑

### M97-0：冻结characterization与失败证据

固定当前fixture、UV1 import、SH sampler、Forward/Deferred差异、feature-off反例、fake offline bake矛盾和Editor hardcoded行为；建立同场景Unreal/Godot/Unity/Fyrox参考数据集。

### M97-1：Scene authoring与静态灯光语义

完成mesh/light/environment/probe/scenario source schema、migration、property path、save/reopen与undo/redo；定义Static/Stationary/Dynamic和shadowmask truth。

### M97-2：BuildSubjectId、manifest与registry

建立stable subject identity、canonical manifest、dependency graph、artifact key、registry atomic commit和精确invalidations。

### M97-3：UV与atlas preparation

实现author UV validator、derived unwrap、chart pack、gutter/dilation、texel density、diagnostic和deterministic geometry artifact。

### M97-4：确定性CPU reference baker

支持最小物理通道、bounce、environment、directional/point/spot、probe placement与可比较reference output；提供cancel/progress/checkpoint。

### M97-5：GPU/path-traced backend与job scheduler

实现预算化GPU backend、tile/cell任务、checkpoint/resume、device loss恢复、denoise与CPU parity；禁止阻塞frame loop。

### M97-6：Artifact、cook与atomic publish

完成分section编码、checksum、compression、platform variants、import/cook、CAS与registry generation发布。

### M97-7：Runtime resident generation

接入异步I/O、prepared upload、fence、device generation、budget、eviction和stale拒绝；atlas/slot/probe一次发布。

### M97-8：GPU scene与physical shading

删除逐帧slot HashMap，建立per-object assignment；修正Deferred channel、double lighting、Stationary/shadowmask与feature permutation。

### M97-9：Probe Volume producer与质量

实现geometry-aware placement、validity、virtual offset/dilation、leak reduction、brick/cell artifact、fallback与quality visualization。

### M97-10：多volume assignment与streaming

实现per-object/cluster候选、overlap blend、layer、large-world、cell streaming、budget与fault recovery，删除single-view winner。

### M97-11：Plugin与Editor产品闭环

让plugin注册真实backend/service，让Editor完成设置、Bake/Preview、进度、取消、重试、诊断、dirty/save与resident preview。

### M97-12：Fault、scale与soak

覆盖invalid UV、oversized atlas、partial artifact、corruption、cancel、OOM、device loss、hot reload、World Partition、成千上万instance和长时soak。

### M97-13：同画质竞争性验收

固定场景、硬件、分辨率、bounce、采样、denoise、误差阈值、内存与warm/cold条件，对照Unreal/Godot/Unity；只有统计上成立才允许性能/表现声明。

## 11. 资格门

| Gate | 必须通过的证据 |
|---|---|
| G97-01 | Scene可保存并重开light build settings、mesh/light contribution、probe source与build-data handle |
| G97-02 | Prefab/scene migration不会丢失或静默默认关键bake语义 |
| G97-03 | Static/Stationary/Dynamic及shadowmask有单一文档化truth并贯穿authoring到shader |
| G97-04 | Editor修改settings支持undo/redo、dirty与save/reopen |
| G97-05 | World加载真实build data并自动形成render extract，不需要测试手工注入 |
| G97-06 | 缺失、过期或不兼容build data产生typed readiness与actionable diagnostic |
| G97-07 | BakeManifest覆盖所有影响结果的geometry/material/light/environment/backend/platform输入 |
| G97-08 | 相同manifest/seed/backend产生bitwise或定义误差内确定结果 |
| G97-09 | request中的每个subject都有成功、跳过或失败terminal status，不可静默漏项 |
| G97-10 | job支持真实progress、cancel、resume/checkpoint与失败重试 |
| G97-11 | bake不阻塞frame/Editor event loop，并受CPU/GPU/内存预算约束 |
| G97-12 | artifact含schema/backend/platform/content identity、section checksum与兼容范围 |
| G97-13 | registry commit是原子的，失败不会暴露半套atlas/slot/probe generation |
| G97-14 | dependency变化只失效受影响subject/cell，并有可解释invalidations报告 |
| G97-15 | author UV1有效时可保留，缺失/非法时可生成derived UV且不污染source mesh |
| G97-16 | overlap、degenerate、out-of-range、nan和zero-area chart均被定位到具体asset/submesh |
| G97-17 | atlas packing确定、支持多page、设备上限与byte budget，失败不会截断输出 |
| G97-18 | gutter/dilation与mip-safe bounds通过高反差seam visual golden |
| G97-19 | texel density策略可按mesh/level覆盖并在Editor可视化 |
| G97-20 | atlas format/mips/compression/cook variants通过decode parity与平台capability测试 |
| G97-21 | resident cache key包含artifact revision和device generation，hot reload不会保留旧texture |
| G97-22 | atlas、assignment和probe在同一fence-qualified generation原子发布 |
| G97-23 | shader不会把baked diffuse写进emissive，Forward/Deferred物理channel一致 |
| G97-24 | baked light identity阻止Static/Stationary direct double lighting |
| G97-25 | feature/plugin关闭后不分配、不绑定、不采样baked-lighting资源 |
| G97-26 | GPU scene不逐帧重建全量slot HashMap；变更成本随delta而非scene size增长 |
| G97-27 | generation字段要么被端到端验证，要么从ABI删除，无死协议字段 |
| G97-28 | device loss后resident generation可重建，stale upload completion被拒绝 |
| G97-29 | probe placement对墙体/薄几何/室内外边界有validity与leak-reduction证据 |
| G97-30 | dynamic object在volume边界和重叠区平滑blend，无出界突黑 |
| G97-31 | assignment为per-object/per-cluster并尊重layer，不再使用single-view winner |
| G97-32 | brick/cell streaming有priority、budget、eviction、prefetch和fallback |
| G97-33 | volume artifact encoding/format/dimensions/hash严格校验，corruption可诊断 |
| G97-34 | large-world origin shift与partition load/unload不破坏probe定位或generation |
| G97-35 | Editor Bake/Preview绑定真实job ID、scene revision和artifact generation |
| G97-36 | Editor显示真实阶段、吞吐、ETA、warning、失败、cancel/retry/resume状态 |
| G97-37 | UV、texel density、chart、probe validity、cell residency和light contribution有debug view |
| G97-38 | bake完成后scene dirty/save/registry publish与runtime preview次序可测试 |
| G97-39 | 不再存在City_Block_A、87 assets、4 warnings、02:30等生产演示常量 |
| G97-40 | 端到端测试覆盖save/reopen -> bake -> artifact -> cook -> load -> stream -> render |
| G97-41 | fault suite覆盖cancel、corruption、partial write、OOM、device loss、hot reload与retry |
| G97-42 | 规模测试覆盖大atlas、多page、数万instance、多volume与长时residency churn |
| G97-43 | 同画质golden包含间接光、shadowmask、seam、leak、dynamic object与Forward/Deferred parity |
| G97-44 | 同硬件同场景同画质对照记录bake time、runtime GPU、CPU、内存、IO与统计置信区间 |

## 12. 参考引擎映射与适用性

| 参考 | 本轮采用的工程事实 | Zircon应吸收 | 不应照抄 |
|---|---|---|---|
| Unreal | `UMapBuildDataRegistry`按mesh/light/level保存LightMap、ShadowMap、irrelevant lights、VLM与reflection data，提供invalidate与World Partition registry定位；VLM使用indirection/brick data与streaming；GPULightmass有settings、tile/sample workload、denoise/transcode、preview与commit | build registry、light identity、separate shadow data、atomic build commit、brick residency、production bake job | UObject/RHI宏和整套UE package结构不直接移植 |
| Unity Graphics | 可见源码包含PathTracing `BakeLightmapDriver`、UV overlap检测，以及APV BakingSet、placement、serialization、dilation、virtual offset、cell/brick streaming和Editor tests | backend driver contract、UV validation、scenario/baking set、probe validity/leak reduction、cell streaming与测试分层 | 仓库不含Unity全部native lightmapper，不能据此宣称完整backend细节 |
| Godot | `LightmapGIData`保存分层light/shadowmask textures、user UV/slice、probe points/SH/tetrahedra/BSP与hash；`LightmapGI`提供quality、bounces、texel scale、max texture、shadowmask、probe generation、progress/abort和`.lmbake`保存；LightmapperRD有raster/trace/denoise/dilate | 紧凑而完整的scene resource、Editor bake闭环、shadowmask、probe interpolation与明确错误码 | 单一节点/resource模型和Godot rendering server接口不直接复制 |
| Fyrox | `LightmapInputData::from_scene`分离提取与重任务，CPU baker使用Rayon，内建UV generation、progress、cancellation，LightmapEntry记录参与灯光防止double lighting | Rust侧job separation、取消/进度、UV生成、light identity与CPU reference思路 | 源码自己标注单texture上限TODO，不能作为最终大世界/性能上限 |
| Bevy | 明确声明不内建baker；Lightmap component、render-world extraction、bindless slab/fallback、UV rect和bicubic option清晰；irradiance volume文档量化encoding/fetch/bytes取舍并按capability降级 | Rust ECS extraction、resource slab、capability fallback、encoding成本量化 | Bevy是消费侧参照，不能拿它替代Unreal/Godot/Unity producer目标；其低能力single-view fallback也不是Zircon高端目标 |

## 13. 禁止的临时修法

1. 不得给`offline_bake_frame`返回值再塞一张假texture并继续称为offline bake。
2. 不得仅让Editor按钮调用`LightmapBakeOutput::default()`或写JSON fixture。
3. 不得用更多optional字段继续扩张opaque snapshot/output，必须先建立typed manifest与artifact schema。
4. 不得保留旧公开slot Vec，再额外缓存一份HashMap形成双重truth。
5. 不得通过每帧自增generation掩盖缺少content identity与atomic publish。
6. 不得在Deferred继续借用emissive channel并靠后续pass减去baked light。
7. 不得以camera/view winner替代per-object probe assignment。
8. 不得吞掉streaming或resource ensure错误后回退黑色而不进入readiness。
9. 不得用固定超大volume texture、全量预载或无限cache假装streaming完成。
10. 不得只增加单元测试数量而不增加真实scene/bake/artifact/cook/load端到端证据。
11. 不得把Bevy“外部baker”现状当成Zircon producer可以缺失的理由。
12. 不得在没有同硬件同画质统计证据时写“优于Unreal”的性能或表现结论。

## 14. 状态

- 当前源码审查：完成。
- 旧09F2 P0重验：12项全部开放，零项关闭；本文不重复累计父P0。
- 本轮细化：36项P1、8项P2、44个资格门。
- 实现：未开始。
- 动态验证：未执行；没有伪造Cargo、GPU、Editor、cook或benchmark结果。
- source recheck：需要；主语料包含共享working-tree中的`lightmap.rs`与`extract.rs`修改。
- 完成定义：只有G97-01至G97-44全部由当前源码、测试、artifact receipt、visual/performance evidence证明通过，旧09F2才可归档；局部fixture可见、按钮可点击或插件可加载都不构成完成。
