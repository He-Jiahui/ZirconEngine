---
title: Runtime Volumetric Fog、Froxel、Local Fog Volume、Lighting、Shadow、History、Temporal Reprojection 与 Product Integration 当前源码工程化差距
category: zircon_runtime
report_id: Runtime99
review_date: 2026-08-22
baseline_head: be5a281c96b6dc9d33b5c9d0f2699a8bf75afcf1
baseline_epoch: 336
related_code:
  - zircon_plugins/rendering/features/volumetric_fog/runtime
  - zircon_plugins/rendering/features/volumetric_fog/editor
  - zircon_plugins/rendering/plugin.toml
  - zircon_plugins/rendering/runtime/src/lib.rs
  - zircon_runtime/src/asset/assets/scene/post_process.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/volumetric.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/extract.rs
  - zircon_runtime/src/core/framework/render/post_process
  - zircon_runtime/src/scene/components/scene/post_process.rs
  - zircon_runtime/src/scene/world/render_post_process.rs
  - zircon_runtime/src/scene/world/render/lights.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/resource_descriptors.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/oit_buffers
  - zircon_runtime/src/graphics/scene/scene_renderer/history/scene_frame_history_textures/volumetric_history.rs
  - zircon_runtime/src/graphics/shader/wgsl/zr_volumetric.wgsl
  - zircon_runtime/src/graphics/shader/includes/zr_oit.wgsl
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store/volumetric_fog.rs
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/rendering/workbench_extension_post_process_workspace.zui
tests:
  - zircon_plugins/rendering/features/volumetric_fog/runtime/src/tests.rs
  - zircon_plugins/rendering/features/volumetric_fog/runtime/src/wgpu_product_tests.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/volumetric/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/integrate/tests
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/light_scatter/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel/media_inject/tests.rs
  - zircon_runtime/tests/runtime_volumetric_shading_contract.rs
  - zircon_runtime/tests/runtime_volumetric_temporal_wgpu_contract.rs
  - docs/tests/runtime/render/plan18_volumetric_compiled_scene_window_light_shaft_perf_wgpu_20260711.txt
  - docs/tests/runtime/render/plan18_af_m3_volumetric_media_dx12_renderdoc_20260716_resource_stats.json
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/09e-direct-lighting-clustered-shadow-review.md
  - docs/plans/optimize/zircon_runtime/09f1-environment-sky-ibl-reflection-probe-review.md
  - docs/plans/optimize/zircon_runtime/09f2-baked-lighting-lightmap-irradiance-volume-review.md
  - docs/plans/optimize/zircon_runtime/09f3-hybrid-global-illumination-review.md
  - docs/plans/optimize/zircon_runtime/09g1-volumetric-fog-froxel-review.md
  - docs/plans/optimize/zircon_runtime/09g2-advanced-surface-lighting-review.md
  - docs/plans/optimize/zircon_runtime/09h1-temporal-aa-velocity-history-upscaling-review.md
  - docs/plans/optimize/zircon_runtime/89-runtime-render-graph-builder-compiler-resource-lifetime-pass-culling-transient-aliasing-barrier-queue-scheduling-execution-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/95-runtime-direct-lighting-photometry-light-grid-clustered-forward-plus-shadow-atlas-cascade-point-spot-rect-cookie-ies-submission-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/96-runtime-environment-sky-atmosphere-cloud-ibl-reflection-probe-capture-convolution-sh-pmrem-cache-residency-submission-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/97-runtime-baked-lighting-lightmap-probe-volume-bake-job-artifact-residency-sampling-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/98-runtime-hybrid-global-illumination-scene-representation-surface-cache-global-sdf-screen-probe-radiance-cache-product-integration-current-source-review.md
  - docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/VolumetricFog.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/VolumetricFogVoxelization.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/VolumetricFogLightFunction.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/LocalFogVolumeRendering.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/LocalFogVolumeComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/LocalFogVolumeSceneProxy.cpp
  - dev/UnrealEngine/Engine/Source/Editor/ComponentVisualizers/Private/LocalFogVolumeComponentVisualizer.cpp
  - dev/UnrealEngine/Engine/Shaders/Private/VolumetricFog.usf
  - dev/UnrealEngine/Engine/Shaders/Private/LocalFogVolumes
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/AtmosphericScattering/VolumetricFog.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/VolumetricLighting
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Material/VolumetricMaterial/VolumetricMaterial.compute
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Editor/Lighting/VolumetricLighting
  - dev/godot/scene/3d/fog_volume.cpp
  - dev/godot/servers/rendering/renderer_rd/environment/fog.cpp
  - dev/godot/servers/rendering/renderer_rd/shaders/environment/volumetric_fog.glsl
  - dev/godot/servers/rendering/renderer_rd/shaders/environment/volumetric_fog_process.glsl
  - dev/godot/editor/scene/3d/gizmos/fog_volume_gizmo_plugin.cpp
  - dev/bevy/crates/bevy_light/src/volumetric.rs
  - dev/bevy/crates/bevy_pbr/src/volumetric_fog
  - dev/Fyrox/fyrox-impl/src/renderer
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Runtime Volumetric Fog、Froxel、Local Fog Volume、Lighting、Shadow、History、Temporal Reprojection 与 Product Integration 当前源码工程化差距

## 1. 结论

当前Zircon Volumetric Fog不是纯占位。first-party runtime plugin已经声明Media Inject、Light Scatter、Integrate三段compute pass，使用RGBA16F 3D纹理、clustered light grid、shadow atlas、Henyey-Greenstein phase、前向Beer-Lambert积分和跨帧history；Forward、Deferred、Sky及普通透明mesh也接入了最终`color * transmittance + scattering`。RenderDoc artifact进一步证明High档确实创建160x90x96资源并执行三段dispatch。这些路径可以保留为characterization oracle。

但现有实现仍是固定规模研究原型，不是工程级体积介质系统。Local Fog不是独立Scene primitive，而是把任意Post Process Volume与Collider临时解释为雾；抽取后Sphere、旋转Box、blend distance、priority和原始shape全部退化为world AABB。Media shader让1,382,400个High froxel逐个遍历全部局部体积，没有view culling、tile/bin、容量、overflow或persistent generation。全局介质又固定锚定绝对Y=0，负Y始终是最大密度。

时序和合成仍有正确性阻断。screen pixel jitter被直接当作froxel cell offset；history只做nearest `textureLoad`、固定0.9、单一extinction阈值且不感知光、雾体积、参数或dynamic-resolution generation。OIT在存储片元前已经应用camera-to-fragment完整雾，再把RGBA压成UNORM8并与已经fogged的scene color排序混合，多个透明层会重复或错误加权in-scattering；OIT sprite路径又完全不应用同一介质合同。

产品证据没有通过。唯一compiled-scene窗光束报告仍是`diagnostic_failed`，4,398个shaft sample中0个变亮，平均亮度差为-2.347，shaft/shadow control对比为-0.000；本轮重新查看PNG只看到亮窗和整体变暗，没有可信光束。RenderDoc slice证明资源存在，但media slice近乎均匀，scatter/integrated slice肉眼近黑，不能替代画质gate。20个artifact共约26.94 MiB，其中两份RDC不能证明跨设备、动态场景或透明正确性。

旧09G1的 **11项P0全部保持开放**。本轮新增 **1项独立P0**：启用插件但Scene没有任何Volumetric Fog authoring时，executor仍从`RenderResolvedPostProcessSettings`取得`density=0.02`的默认全局介质并执行；`AdvancedLightingExtract::volumetric == None`和空Scene不代表关闭。本文另登记 **36项P1、8项P2与44个资格门**。在Scene truth、独立Local Fog、GPU resident voxelization、正式lighting/environment、history validity、透明分段合成、Editor authoring和竞争性验收闭合前，不得声称该功能达到或优于当前Unreal、Unity HDRP或Godot Forward+。

## 2. 审查边界、currentness与证据

### 2.1 冻结语料

| 范围 | 文件 / 行 / bytes / test attributes / ignored | 证据等级 | fingerprint |
|---|---:|---|---|
| Volumetric symbol-bearing current-source总语料 | **146 / 42,278 / 1,676,278 / 280 / 5** | E3覆盖asset、Scene、neutral contract、plugin、froxel、history、OIT、shader、Editor与tests | `c09adacfa7d853420ee7d85f4be75970ffa4b20424a4fdc368f5fe927757f591` |
| production-like源码 | **119 / 32,405 / 1,301,627 / 149 / 0** | E3覆盖产品数据链和内嵌characterization tests | `6f1f3169327d922e53e8faa73c7cce5cb6aa0c4e4ea16fd58b3e2cfa5a9bf7dc` |
| focused tests与test support | **27 / 9,873 / 374,651 / 131 / 5** | E3读取CPU、Naga、WGPU、product exporter、compiled scene和temporal contract | `d17a848a3ac31160fcc661a445d67c2bd7b061b9510ed8c3bb351149ad110f34` |
| Editor authoring表面 | **8 / 2,296 / 98,451 / 0 / 0** | E3覆盖Editor plugin与通用Post Process workspace/action wiring | `6232aac0be053dbec14dc7766a73939df00cb8d7f805d7aea96e0ab3f0d3f97c` |
| 当前产品/RenderDoc artifacts | **20 / 28,250,439 bytes** | E2读取4组TXT/PNG、2个RDC、bindings/resource stats及8张slice/scene PNG | `5cccf51e9d49ecda25d85c9789f2fbfb9dbc40955bdfe94b3b3f3cffcc2f5066` |
| 五引擎参考切片 | **47 / 21,664 / 929,669** | E2/E3读取Unreal、Unity HDRP、Godot、Bevy和Fyrox owner边界 | `41eb85cca7dfe8b5ecd80c3974cffb65a3b4ce9bdc69a64792c7f2072e58d316` |

fingerprint算法为：相对路径与每文件SHA-256组成排序manifest，以TAB分隔字段、LF分隔记录，再对UTF-8 manifest执行SHA-256。行数按PowerShell `Get-Content`逻辑统计。冻结对象是2026-08-22共享working tree，不是只读HEAD；基线HEAD为`be5a281c96b6dc9d33b5c9d0f2699a8bf75afcf1`，coordinator epoch为336。

Bevy、Fyrox、Godot与Unity Graphics revision分别为`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`与`a7e4c051d256a781ab362c64316b125a1e104694`。Unreal不是独立Git checkout；其`VolumetricFog.cpp`冻结SHA-256为`3a7c245ff748c97c2af79a96b390d0c40e877b3812b95bb02476eee4c1b7c8b5`，不得把父仓revision伪装为Unreal revision。

冻结时`advanced_lighting/extract.rs`、`volumetric.rs`、部分post-process文件在共享working tree显示为modified，但相关scope的`git diff --stat`没有文本差异。本报告读取current blob，不接管、不还原这些文件；实现前必须重新计算fingerprint并检查并发Session结果。

### 2.2 机械扫描与测试语义

production-like 119文件中有0个TODO/FIXME/HACK/XXX，6个`panic!/todo!/unimplemented!`、31个`.unwrap(`、104个`.expect(`、14个`#[allow(`和121个`include_str!`。相关路径至少直接创建10个texture、6个pipeline、9个bind group并执行6次Mutex lock；三段froxel PSO均明确`cache: None`，Media/Scatter/Integrate每dispatch仍创建uniform/bind group，Media另创建local-volume storage buffer。零TODO不是完整度证据，只说明临时行为被固化为正式代码形态。

focused tests有131个test attributes和5个ignored exporter。至少6个非ignored WGPU contract在adapter缺失时直接`return`；ignored exporter同样可skip。focused测试语料有251处`.contains(`，其中大量是source-string结构断言。它们适合锁ABI和shader拼装，不足以证明shader执行、画质、性能或产品链。

### 2.3 Owner边界

| 边界 | Runtime99要求的owner | 禁止的越界 |
|---|---|---|
| Scene truth | `zircon_runtime::scene`拥有versioned global medium、独立Local Fog primitive、light participation和持久化quality policy | 不由插件启用状态暗中制造默认雾；不借Post Process Collider猜测介质几何 |
| Neutral contract | `core::framework::render`携typed、generation-qualified、backend-neutral view/volume/light/history descriptor | 不携WGPU资源；不把world AABB当完整Local Fog contract |
| Runtime graphics | graphics唯一拥有froxel layout、GPU culling/voxelization、lighting、history、composition、residency和fault recovery | 不让每个consumer各建apply bind group；不允许CPU逐froxel/逐volume权威 |
| Feature package | Volumetric Fog plugin拥有算法实现和graph contribution，但必须尊重Scene显式enable与prepared generation | capability启用不等于Scene全局介质启用；descriptor不得冒充authoring |
| Editor product | `zircon_editor`拥有Inspector、gizmo、material、debug slice、budget、capture、transaction和validation | 不以硬编码Post Process演示页和queued字符串冒充工具 |
| 历史P0 | 旧09G1唯一计数11项父P0；Runtime99做current-source重验并只新增不同根因 | 不因同一AABB/固定网格问题跨层可见而重复累计P0 |

### 2.4 明确未做

本轮只做静态review和现有artifact复核，没有修改Rust、WGSL、Cargo、plugin manifest、Scene asset或Editor UI，没有运行Cargo、Editor/App、真实GPU、RenderDoc重放、cook/export、device loss、OOM、large-world、XR、dynamic resolution、透明多层golden、24小时soak或同画质性能基准。静态源码足以证明数据丢失、复杂度、默认语义和测试skip，不能替代最终动态验收。

## 3. 当前应保留的真实基础

1. first-party runtime plugin和三段compute graph是正确feature边界起点，应升级而非退回core硬编码。
2. `VolumetricFogSettings`、`FogVolumeData`、`AdvancedLightingExtract`、render layer与light participation已形成typed characterization contract。
3. `FroxelGridParams`的非线性Z重建、RGBA16F D3资源和High history resource提供了可迁移的VBuffer底座。
4. Media/Scatter/Integrate WGSL真实通过Naga和部分WGPU fixtures，前向Beer-Lambert积分公式可保留。
5. Scatter已消费统一clustered light data与shadow atlas，而不是完全独立的测试灯列表。
6. `zr_volumetric_apply`已接入Forward、Deferred与Sky，可收敛为唯一surface/volume composition contract。
7. history texture具有quality变更与资源重建生命周期入口，可扩展为完整generation validity。
8. RenderStats已有dispatch/group/upload三个稳定指标，适合作为正式telemetry schema的种子。
9. 两份RenderDoc capture、bindings和resource stats证明真实DX12资源/dispatch存在，可保留作回归样本但不能充当产品通过证据。

## 4. P0 current-source重验

| finding | 当前状态 | current-source证据与关闭要求 |
|---|---|---|
| 09G1-P0-1 唯一产品光束证据失败 | 开放 | TXT仍为`diagnostic_failed`；shaft brighter 0/4398、平均差-2.347、对比-0.000，本轮PNG复看也无可信光束。关闭需当前源码、非ignored、required-GPU的可重复光束golden与性能证据 |
| 09G1-P0-2 Local Fog shape/transform/blend/priority丢失 | 开放 | Post Process Box/Sphere经`fog_volume_from_extract`转world AABB；rotation、sphere、blend distance、priority均不进入`FogVolumeData`，其他Collider直接None。关闭需独立component、world-to-local、shape/fade/priority/material generation |
| 09G1-P0-3 固定160x90 XY网格 | 开放 | Low/Medium/High均为160x90，只改变Z=48/64/96；不消费render extent、宽高比、DRS、FOV、XR或budget。关闭需view-local计算、atomic generation切换和scalability curve |
| 09G1-P0-4 全局介质绝对Y=0与参数不足 | 开放 | shader仍执行`max(world_position.y, 0.0)`；没有base height、start/max distance、absorption、emissive、heterogeneous density或origin generation。关闭需物理系数、camera-relative/large-world和完整环境介质参数 |
| 09G1-P0-5 每froxel遍历全部Local Fog | 开放 | High每view 1,382,400 froxel，shader对`volume_count`无界循环；无culling/binning/cap/overflow。关闭需CPU view cull + GPU tile/cluster/brick list或indirect voxelization及overflow receipt |
| 09G1-P0-6 pixel jitter被当froxel cell | 开放 | `offset_pixels.xy`直接写入`jitter_and_history`并加到invocation；单测甚至在history unavailable时仍断言0.25/-0.125。关闭需统一像素/UV/froxel单位和首帧/resize/cut规则 |
| 09G1-P0-7 history过滤和拒绝不足 | 开放 | nearest `textureLoad`、固定0.9、`max(0.02,current.a*0.25)`；只混RGB，不看motion/depth/normal/light/volume/parameter generation。关闭需filter、clamp、confidence、disocclusion、history-miss策略和全失效图 |
| 09G1-P0-8 Transparent/OIT重复介质组合 | 开放 | mesh `fs_oit`存入已经fogged的`zr_fs_main_impl`结果，RGBA再`pack4x8unorm`；resolve与已经fogged scene color混合。关闭需depth-segment volume integration、HDR fragment payload及粒子/折射合同 |
| 09G1-P0-9 direct/environment lighting低于正式合同 | 开放 | Rect只做方向cosine，cookie不采样，ambient只来自显式Ambient snapshot，未接Sky/IBL/Baked/HGI；light participation仍靠ID membership。关闭需消费Prepared Lighting generation及统一photometry/shadow/cookie/environment |
| 09G1-P0-10 Editor无真实authoring | 开放 | editor plugin只有descriptor/capability；通用Post Process workspace硬编码Bloom/Tonemap/LUT和City样例，搜索不到volumetric authoring字段。关闭需transactional Inspector、gizmo、material、quality、debug、save/reopen和undo/redo |
| 09G1-P0-11 GPU缺席静默通过且产品测试默认不运行 | 开放 | 5个exporter ignored，至少6个非ignored WGPU测试无adapter直接return，失败artifact尚未被当前通过artifact替换。关闭需capability-aware required lane、skip receipt、current artifact和设备矩阵 |
| Runtime99-P0-12 空Scene在插件启用后自动获得默认全局雾 | 新增、开放 | `resolved_volumetric_fog_settings`在extract `None`时解析post-process stack，而`RenderResolvedPostProcessSettings::new`固定填`VolumetricFogSettings::DEFAULT`，density=0.02；graph无per-frame admission。关闭需显式Scene enable/absence语义，empty extract不dispatch且有save/reopen/product test |

## 5. 当前产品数据链与断点

```text
Scene PostProcessVolume + Collider
        | profile.volumetric_fog
        +--> global volume evaluation ----------------------+
        |                                                   |
        +--> local volume -> Box/Sphere shape ----X----> world AABB DTO
                                                        |
plugin enabled -> three passes always compiled          v
empty Scene ----X----> resolved DEFAULT density 0.02 -> Media Inject
                                                        |
          every froxel x every local AABB ---------------+
                                                        v
clustered light grid + shadow atlas + Ambient snapshots -> Light Scatter
                                                        |
nearest fixed-0.9 history -------------------------------+
                                                        v
front-to-back Integrate -> RGBA16F VBuffer
                |              |              |
             Forward        Deferred         Sky
                |
       transparent color already fogged
                v
     OIT RGBA8 pack + depth sort + fogged scene resolve

Editor plugin descriptor ----X----> Inspector / gizmo / material / debug / save
failed ignored artifact -----X----> required product acceptance
```

High档current两份物理3D texture约21.09 MiB；history再加约10.55 MiB，合计约31.64 MiB/单view，尚未计alignment、bind/upload、shadow/light资源和多view。Low约10.55 MiB、Medium约14.06 MiB。当前没有per-view family/global budget、allocation failure策略或Editor内存展示。

## 6. P1重构项

### 6.1 Scene truth、contract与identity

| ID | 差距与要求 |
|---|---|
| P1-1 | 为extinction、scattering、absorption、mean free path、emission、world unit和color space定义物理单位、合法域及迁移规则，禁止继续让`density`随项目尺度任意变化。 |
| P1-2 | 新建独立`LocalFogVolumeComponent`/asset；Post Process Volume只覆盖global/profile参数，不再兼任参与介质几何。 |
| P1-3 | 将global fog的`enabled/disabled/inherit`变成显式versioned字段；空Scene、无profile、禁用plugin和unsupported backend必须有不同receipt。 |
| P1-4 | Local Fog contract保留shape、world-to-local、size、face/radial/distance fade、priority、blend operation、layer和material handle；AABB只用于粗剔除。 |
| P1-5 | volume、light和view使用stable identity、revision、prepared generation与last-good resident record；不得每帧clone整组DTO或按裸ID猜生命周期。 |
| P1-6 | 提供从旧Post Process Volume+Collider到新Local Fog asset的显式迁移、unknown-field处理、cook schema、roundtrip及拒绝报告；迁移完成后硬删除旧路径。 |

### 6.2 Froxel layout、介质与voxelization

| ID | 差距与要求 |
|---|---|
| P1-7 | 按render extent、screen percentage、FOV、projection、view rect、XR eye和quality budget计算XY/Z；resize与DRS必须原子切换resource generation。 |
| P1-8 | 分离camera far与fog start/max distance，支持base height/world origin、near fade、orthographic、infinite far和camera-relative大世界。 |
| P1-9 | 将介质存储定义为明确的scattering/extinction或albedo/extinction合同，处理FP16范围、pre-exposure、NaN/Inf和能量clamp。 |
| P1-10 | 建立Volume Material domain与3D density/noise texture、tiling、scrolling、emissive、temperature/color及Add/Overwrite/Multiply/Min/Max/Subtract组合。 |
| P1-11 | Scene粗剔除后在GPU构建tile/cluster/brick candidate list，或按visible OBB/sphere/volume material执行indirect slice voxelization；禁止全froxel全volume循环。 |
| P1-12 | 所有visible volume、tile list、material、texture和dispatch容量必须有typed cap、priority、overflow counter、degradation reason与last-good策略。 |

### 6.3 Lighting、shadow与environment

| ID | 差距与要求 |
|---|---|
| P1-13 | 每盏light提供独立volumetric scattering intensity、shadow dimmer、participation/layer mask和quality，而不是只有`bool volumetric`。 |
| P1-14 | Scatter消费Runtime95的统一photometry和attenuation；Rect必须按面积或有误差界的LTC/representative-point近似，不得退化为point+cosine。 |
| P1-15 | cookie、light function、IES和projector texture必须按同一resident generation进入体积采样，并有missing/unsupported fallback。 |
| P1-16 | direction/point/spot/rect shadow使用正式cascade/atlas/cache generation、bias和per-light quality；体积debug能区分无shadow、slot miss与layer mismatch。 |
| P1-17 | Sky/IBL、baked probe/APV、HGI/Lumen-like radiance和volumetric cloud shadow进入明确的environment/multiple-scattering合同，禁止重复注入能量。 |
| P1-18 | cluster candidate直接排除不参与体积的light，或使用独立compact mask/list；layer、view family和multi-camera选择必须在CPU/GPU一致。 |

### 6.4 Temporal、history与composition

| ID | 差距与要求 |
|---|---|
| P1-19 | 统一TAA与froxel jitter的pixel/UV/cell单位，记录sample sequence、current/previous view rect和resource scale；history invalid时不得无配对抖动。 |
| P1-20 | history保存prev grid size、viewport scale/limit、depth encoding、pre-exposure、camera origin和view identity，支持DRS、resize、stereo与camera stack。 |
| P1-21 | 使用filterable history、neighborhood/variance clamp、moments/confidence、depth/normal/motion/disocclusion验证，并提供受预算约束的history-miss supersampling或空间滤波。 |
| P1-22 | camera cut、projection/quality变化、volume/light/material/texture/shadow/environment generation变化必须形成可解释的局部或全局history invalidation。 |
| P1-23 | 建立按depth segment积分的透明介质合同；排序透明、OIT、粒子、折射、water和volumetric material必须共享transmittance/in-scattering定义。 |
| P1-24 | OIT payload保持scene-linear HDR和足够depth/coverage，解决RGBA8 quantization与overflow；sprite/particle路径不得绕过Volumetric Apply。 |

### 6.5 GPU lifetime、调度与scalability

| ID | 差距与要求 |
|---|---|
| P1-25 | 使用persistent upload ring、storage capacity manager和bind-group/resource-set cache；只在view/volume/light generation变化时更新range。 |
| P1-26 | 三段PSO接入统一shader/PSO cache、warmup manifest、pipeline generation和driver cache；quality切换不得首帧同步编译。 |
| P1-27 | 建立per-view/per-family/global VRAM与bandwidth budget、多view上限、allocation failure降级、device-loss重建和Editor可见memory receipt。 |
| P1-28 | `AsyncCompute`必须有真实queue ownership、barrier、fence和GPU timeline overlap；否则准确标记compute-on-graphics并按该成本预算。 |
| P1-29 | 删除`fog_volumes_for_layers`整Vec clone、volumetric ID `Vec::contains`和重复light collect/sort/dedup，改为prepared compact tables和dirty range upload。 |
| P1-30 | 明确FP16 overflow、large-world precision、zero/negative density、NaN/Inf、device feature缺失及shader fault的fail-closed/last-good行为。 |

### 6.6 Editor、diagnostics与产品证据

| ID | 差距与要求 |
|---|---|
| P1-31 | Editor提供global fog与Local Fog Inspector、shape gizmo、priority/fade/layer、quality、preview，所有改动走transaction、dirty、undo/redo、save/reopen和PIE同步。 |
| P1-32 | 提供Volume Material/3D texture/noise authoring、asset dependency、residency状态、无效mask/material warning和cook预检。 |
| P1-33 | 提供extinction/albedo/scattering/integrated slice、volume bins、light count、shadow/cookie route、history confidence/rejection、overflow与NaN heatmap。 |
| P1-34 | 稳定记录GPU ms、候选volume/light、tile occupancy/peak、history acceptance、VRAM、upload、PSO/cache、async overlap和resolved quality reason。 |
| P1-35 | product lane把adapter缺失标为`skipped-capability/not-run`且不计通过；PNG/RenderDoc exporter必须由当前源码required job生成并校验artifact lineage。 |
| P1-36 | 建立同场景、同分辨率、同画质、同硬件的Unreal/Unity/Godot竞争基准，分开报告画质、GPU ms、VRAM、CPU submit和动态稳定性。 |

## 7. P2治理项

| ID | 治理要求 |
|---|---|
| P2-1 | 发布Volumetric Media参数单位、坐标、pre-exposure、alpha语义和推荐范围文档，并与schema version绑定。 |
| P2-2 | 为每个quality preset提供机器可读budget、fallback顺序和平台覆盖，不用Low/Medium/High隐藏能力硬切。 |
| P2-3 | 固化RenderDoc/PIX capture事件命名、资源标签、slice选择、shader hash和artifact lineage recipe。 |
| P2-4 | 规范telemetry metric名称、单位、view cardinality、retention和性能阈值，避免只累计dispatch总数。 |
| P2-5 | 为shader/CPU ABI生成layout manifest与compatibility test，避免手写padding和字符串断言成为唯一保护。 |
| P2-6 | Editor字段、warning、debug view和颜色图例进入本地化、可访问性与自动化命令合同。 |
| P2-7 | golden、RDC、TXT和benchmark artifact设保留期限、current-source hash、设备/driver元数据和失败替换策略。 |
| P2-8 | 维护正式feature maturity矩阵；在44个资格门通过前，manifest不得标Stable或在产品文档中宣称competitive。 |

## 8. 参考引擎给出的最低约束

### 8.1 Unreal Engine

Unreal的VBuffer XY由view resolution与grid pixel size计算，Z、history、jitter、history miss supersampling、emissive、conservative depth、light function、soft shadow等都有独立scalability cvar。history使用bilinear sampler，跟踪prev view/resource UV scale/limit、conservative depth和pre-exposure；reprojection miss可执行额外采样。Lighting path连接Sky Light、DFAO、Lumen translucency GI volume、MegaLights、VSM/RT shadow和volumetric cloud interaction。

Local Fog是独立Scene component/scene proxy，保存transform、radial/height extinction、height falloff/offset、phase、albedo、emissive、start distance和sort priority。其当前形状主要是sphere，不应误写为任意shape；但renderer有CPU view cull、GPU tiled cull、per-tile count/index、indirect draw、debug tile和analytical radial/height integral。一般体积材质则由Volumetric Fog voxelization另行处理。

### 8.2 Unity HDRP

HDRP按camera actual width/height、screen fraction、slice budget和viewCount/XR alignment创建VBuffer，并保存current/previous viewport size、scale、limit及distance encoding。history用linear sampler、显式valid frame、exposure compensation和约6/7权重；Gaussian filter、APV diffuse GI、ambient probe、directional/local/area light、shadow、water caustics均有正式路径。

`LocalVolumetricFog`是独立MonoBehaviour，保存OBB、mean free path、albedo、blend mode、priority、3D mask或Fog Volume material、tiling/scroll、六面fade、distance fade和falloff。AABB只用于culling，OBB进入voxelization；visible list、chunk-resized persistent buffers和indirect slice draw避免全VBuffer扫描全部volume。Editor有serialized inspector、mask/material validation、Undo与Scene handle/gizmo。

### 8.3 Godot

Godot提供World、Box、Ellipsoid、Cone、Cylinder五种Fog Volume shape与FogMaterial/ShaderMaterial；shader通过world-to-local和SDF保留真实shape，density原子累加允许负值先参与组合，再在process阶段clamp总density。它有16帧Halton jitter、linear 3D history、temporal blend、direction/omni/spot/area clustered light、shadow atlas、sky/ambient、VoxelGI与SDFGI注入、三轴Gaussian filter和前向积分。

Editor暴露size/shape/material、Forward+与environment enable warning，并有可撤销size gizmo。Godot也存在固定quality与近似，不是性能上界；但其Scene truth、shape保真、material、cluster lighting和Editor闭环已显著高于Zircon当前AABB DTO。

### 8.4 Bevy与Fyrox

Bevy是screen-space raymarch而非3D froxel VBuffer，可作为较低复杂度基线。它仍提供独立camera `VolumetricFog`、light marker、`FogVolume`、3D density texture/offset、absorption/scattering/asymmetry、step count与jitter；当前WGSL实际支持directional、clustered point和spot light及shadow，模块顶部“只支持directional”的注释已经落后，不能用旧注释否认当前shader。

本地Fyrox语料没有可比的专用Volumetric Fog/Fog Volume实现。本报告只用其renderer/resource owner边界作为下限旁证，不虚构其不存在的功能，也不把“Fyrox未实现”当成Zircon简化的许可。

## 9. 目标架构

```text
Scene
  VolumetricEnvironmentComponent(enabled, physical medium, quality policy)
  LocalFogVolumeComponent(shape, transform, fade, priority, material, layers)
  LightComponent(volumetric intensity/shadow/cookie generation)
       |
       v
VolumetricSceneCompiler
  -> VolumetricSceneGeneration
  -> visible volume/light/material handles + dirty ranges + overflow policy
       |
       v
Runtime graphics
  FroxelLayoutService(render extent / DRS / XR / budget)
  ParticipatingMediaVoxelizer(view cull -> GPU bins/indirect voxelization)
  VolumetricLightingService(prepared lighting/environment/shadow/cookie)
  VolumetricHistoryService(validity / reprojection / filter / confidence)
  VolumetricCompositionService(opaque / transparent segment / OIT / particle)
  VolumetricResourceService(residency / PSO / memory / device recovery)
       |
       +--> telemetry + debug views + capture lineage
       v
Editor VolumetricAuthoringService
  Inspector + gizmo + material + budget + preview + operation + validation
```

Public package形态不变：Scene truth留在`zircon_runtime::scene`，backend-neutral descriptor留在`core::framework::render`，WGPU资源和算法留在graphics，feature graph contribution留在Volumetric Fog plugin，Editor只拥有authoring/operation而不复制运行时算法。不得为旧AABB DTO、默认隐式enable或Post Process Collider路径留下长期compat shim。

## 10. 硬切换与里程碑

| Milestone | 交付内容 | 退出条件 |
|---|---|---|
| M0 Truth与证据止血 | 显式enable/absence语义、空Scene不dispatch、required GPU lane、失败artifact currentness | P0-1、P0-11、P0-12有可复验关闭证据 |
| M1 Scene/contract硬切换 | 独立global/local assets、shape/transform/fade/priority/material、light settings、schema migration | 删除Post Process Collider到`FogVolumeData`旧转换；roundtrip/cook/undo通过 |
| M2 GPU介质表示 | view-dependent VBuffer、persistent resources、view cull、GPU bins/indirect voxelization、overflow | 1/100/1K volume scale曲线和所有overflow receipt通过 |
| M3 Lighting/environment | Prepared Lighting、photometry、area、shadow、cookie/function/IES、Sky/IBL/Baked/HGI | light/environment golden、energy和fallback矩阵通过 |
| M4 Temporal/composition | 统一jitter、history validity/filter、DRS/XR/cut、transparent/OIT/particle segment integration | ghosting、resize、multi-layer transparent和HDR OIT gates通过 |
| M5 Editor/operations | Inspector、gizmo、material、debug slice、capture、budget、transaction与validation | save/reopen、undo/redo、PIE同步、invalid asset warning通过 |
| M6 Competitive acceptance | GPU/VRAM/CPU/quality、fault、device loss、long soak与同画质对照 | 44个资格门全部通过，方可调整maturity和竞争性声明 |

每个Milestone只允许hard cutover：新Scene contract落地后删除旧DTO与consumer；新history generation落地后删除固定0.9旁路；新transparent composition落地后删除片元预fog+RGBA8 OIT路径。禁止以`pub use`、compat module、fallback wrapper或双owner长期保留旧行为。

## 11. 资格门

### 11.1 Scene与contract

1. G01 空Scene+启用plugin时三段dispatch为0，画面与plugin关闭逐像素一致。
2. G02 global enable/disable/inherit经过Scene/Prefab/save/reopen/cook保持一致。
3. G03 Box、Sphere/Ellipsoid、Cylinder/Cone或首批承诺shape在旋转、非均匀scale后与gizmo/GPU一致。
4. G04 priority、blend、face/radial/distance fade、layer和material generation有重叠golden。
5. G05 旧Post Process Volume迁移生成明确receipt，unknown/unsupported Collider不静默丢失。
6. G06 volume/light stable identity在spawn/despawn/reparent/reload后无stale record或错误复用。

### 11.2 Froxel与介质

7. G07 720p、1080p、4K、ultrawide、orthographic、DRS与XR得到可解释的grid/budget结果。
8. G08 resize/quality切换只发布完整resource generation，不混绑旧history或旧view scale。
9. G09 base height、start/max distance、world origin rebase和大坐标下密度连续。
10. G10 mean free path/extinction/scattering/albedo/emission在不同world scale下符合单位测试。
11. G11 3D density texture、scroll/tiling和resident fallback有静态/动态golden。
12. G12 1/100/1K visible volume的GPU候选数、peak tile、overflow和GPU ms受预算约束。
13. G13 全volume重叠、negative/subtractive或Multiply/Min/Max操作无NaN/Inf且顺序语义稳定。

### 11.3 Lighting与environment

14. G14 direction/point/spot/rect在统一photometry下有单灯能量golden。
15. G15 每灯volumetric intensity=0完全不进入候选；非零值单调且不影响surface direct light。
16. G16 shadowed/unshadowed、cascade edge、point六面、spot atlas、rect approximation有差分golden。
17. G17 cookie/light function/IES移动、reload、missing和generation切换无stale采样。
18. G18 Sky/IBL、baked/APV、HGI与ambient分别可开关，组合无重复能量。
19. G19 layer mask、camera stack、multi-view和XR eye只消费各自visible light/volume集合。
20. G20 shadow/cloud/GI unavailable时fallback reason、画面和history invalidation一致。

### 11.4 Temporal与history

21. G21 pixel jitter到froxel jitter在720p/4K/ultrawide数值正确，history invalid帧无未配对jitter。
22. G22 camera cut、teleport、projection、near/far、DRS、view rect与quality变化正确失效history。
23. G23 volume/light/material/texture/shadow/environment generation变化只保留安全history区域。
24. G24 static camera 256帧收敛且无固定pattern；slow pan无明显拖影。
25. G25 thin geometry、disocclusion、high-frequency shadow和moving light达到ghosting阈值。
26. G26 NaN/Inf、out-of-bounds reprojection和history allocation failure不污染current frame。

### 11.5 Composition

27. G27 opaque Forward、Deferred与Sky在相同depth处使用同一VBuffer结果。
28. G28 两层/八层排序透明按depth segment组合，结果不重复camera-to-near-layer in-scattering。
29. G29 OIT在HDR强光下不因RGBA8 quantization/clamp改变能量，overflow有receipt。
30. G30 particle/sprite、refraction、water和transmission均进入定义明确的volume segment合同。
31. G31 MSAA、TAA/upscale、viewport subrect和camera stack不混用错误depth/UV。

### 11.6 GPU lifetime与性能

32. G32 steady-state帧不新建per-pass uniform/storage buffer、bind group或PSO。
33. G33 High单view和4-view/XR的VRAM、upload、bandwidth在预算内且可降级。
34. G34 AsyncCompute有timestamp/timeline重叠证据；无独立queue平台准确走graphics budget。
35. G35 device loss、OOM、shader/pipeline failure和resource recreation恢复到last-good或明确禁用。
36. G36 10K lights/1K volumes压力下CPU prepare、GPU cull、dispatch和overflow均有界。
37. G37 24小时动态camera/light/volume/DRS soak无资源增长、stale generation或history污染。

### 11.7 Editor与operations

38. G38 Inspector/gizmo所有字段走transaction、undo/redo、dirty、save/reopen和PIE同步。
39. G39 invalid material/texture/shape/backend/budget在authoring时给出可操作warning。
40. G40 debug slice、bins、light/shadow/cookie、history、overflow与GPU timing可按view选择并可capture。
41. G41 cook/export包含全部Volume Material依赖并在缺失时fail closed。

### 11.8 产品与竞争性

42. G42 compiled-scene窗光束current artifact通过亮度/对比/阴影控制指标，替换20260711失败报告。
43. G43 required GPU matrix缺adapter时失败或明确not-run，ignored exporter与source-string断言不计acceptance。
44. G44 同硬件、同分辨率、同画质对Unreal/Unity/Godot报告quality、GPU ms、VRAM、CPU和稳定性；只有证据优于目标时才允许“优于当前Unreal”声明。

## 12. 完成定义

Runtime99完成不是“三个compute pass存在”或“RenderDoc里有3D纹理”，而是：Scene能够显式保存global/local介质和light participation；Local Fog保留真实shape/transform/material；VBuffer随view和budget解析；GPU candidate/voxelization有界且resident；lighting、shadow、cookie与environment共享Prepared Lighting generation；history在cut/DRS/scene mutation下可信；opaque/transparent/OIT/particle按depth segment正确合成；Editor可创建、撤销、保存、诊断与cook；fault/scale/soak/required GPU和竞争基准全部通过。

本轮没有实施代码修正。实现阶段必须从M0开始，先关闭默认隐式雾、失败产品证据与测试skip语义，再推进Scene hard cutover；不得在当前AABB+固定网格上继续堆叠材质或画质选项。
