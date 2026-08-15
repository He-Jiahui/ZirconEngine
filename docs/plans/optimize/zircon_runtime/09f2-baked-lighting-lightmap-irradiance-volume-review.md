---
related_code:
  - zircon_runtime/src/core/framework/render/environment/lightmap.rs
  - zircon_runtime/src/core/framework/render/environment/extract.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/irradiance_volume.rs
  - zircon_runtime/src/asset/assets/texture/lightmap_asset.rs
  - zircon_runtime/src/asset/assets/mesh/mesh_asset.rs
  - zircon_plugins/gltf_importer/runtime/src/lib.rs
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
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_runtime/09e-direct-lighting-clustered-shadow-review.md
  - docs/plans/optimize/zircon_runtime/09f1-environment-sky-ibl-reflection-probe-review.md
  - docs/assets-and-rendering/environment-lightmap-probe-consumption.md
  - docs/zircon_runtime/graphics/scene/scene_renderer/environment/lightmap-binding.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/MapBuildDataRegistry.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/LightMap.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/PrecomputedVolumetricLightmap.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/PrecomputedVolumetricLightmapStreaming.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/LightMapRendering.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/LightMapDensityRendering.cpp
  - dev/UnrealEngine/Engine/Plugins/Experimental/GPULightmass/Source/GPULightmass/Public/GPULightmassSettings.h
  - dev/UnrealEngine/Engine/Plugins/Experimental/GPULightmass/Source/GPULightmass/Private/LightmapRenderer.cpp
  - dev/UnrealEngine/Engine/Plugins/Experimental/GPULightmass/Source/GPULightmass/Private/LightmapStorage.cpp
  - dev/UnrealEngine/Engine/Plugins/Experimental/GPULightmass/Source/GPULightmass/Private/VolumetricLightmap.cpp
  - dev/bevy/crates/bevy_pbr/src/lightmap/mod.rs
  - dev/bevy/crates/bevy_pbr/src/lightmap/lightmap.wgsl
  - dev/bevy/crates/bevy_pbr/src/light_probe/irradiance_volume.rs
  - dev/bevy/crates/bevy_pbr/src/light_probe/irradiance_volume.wgsl
  - dev/godot/scene/3d/lightmap_gi.h
  - dev/godot/scene/3d/lightmap_gi.cpp
  - dev/godot/editor/scene/3d/lightmap_gi_editor_plugin.cpp
  - dev/godot/modules/lightmapper_rd/lightmapper_rd.h
  - dev/Fyrox/fyrox-impl/src/utils/lightmap.rs
  - dev/Fyrox/fyrox-impl/src/utils/uvgen.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/PathTracing/BakeLightmapDriver.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/PathTracing/Lightmapping/UVOverlapDetection.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Lighting/ProbeVolume/ProbeVolumeBakingSet.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Lighting/ProbeVolume/ProbeReferenceVolume.Streaming.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Lighting/ProbeVolume/ProbeGIBaking.Placement.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Lighting/ProbeVolume/ProbeGIBaking.Dilate.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Lighting/ProbeVolume/ProbeGIBaking.VirtualOffset.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 09F2 · Baked Lighting / Lightmap / Irradiance Volume / Offline Bake 工程化差距

## 1. 结论

Zircon当前已经有一组值得保留的baked-lighting消费基础：`LightmapBakeRequest`、`LightmapBakeOutput`与`LightmapConsumeContract`具备版本、request/scene/generation、atlas page、stable instance slot和SH9 probe grid校验；`EnvironmentExtract::try_with_baked_lighting`会拒绝lightmap与probe grid的generation不一致；glTF能够把`TEXCOORD_1`导入为mesh `uv1`；GPU Scene可以按static instance写入atlas page与UV rect；forward、deferred和fallback shader都有真实lightmap/probe sampling；irradiance volume也有world-to-volume、priority、layer、normal transform和ambient-cube采样。这些结构可以迁移到目标系统，不应重新发明一套互不兼容的数据。

但这里尚不存在工程意义上的静态光照系统。公开API `offline_bake_frame`不生成lightmap、不追踪光线、不计算间接光、不构建probe grid，也不保存任何artifact。它只是累加DirectionalLight intensity，从前N个mesh的translation和scale制造球形`ReflectionProbeData`，并且没有设置`baked_cubemap`。Reflection Probe GPU admission明确过滤掉没有baked cubemap的probe，所以名为`offline_bake_outputs_reflection_probe_data_without_fake_baked_ambient`的M4测试与真实consumer静态矛盾，不能证明离线烘焙改变了像素。仓内除该测试外没有调用者。

Lightmap部分只有DTO、内存纹理转换和手工消费。没有scene component、scene build settings、Light mobility/bake contribution、自动UV1展开、chart packing、重叠/边距验证、baker、job scheduler、取消/进度、artifact importer、build-data registry、cook dependency或Editor command。唯一产品测试从JSON fixture反序列化已经烘好的RGBA16F页，在内存AssetManager注册TextureAsset，再手工构造`EnvironmentExtract`。它证明“已知正确的字节和slot能被shader采样”，没有证明项目能产生、保存、重载或增量更新这些字节。

Editor和插件表面进一步放大了完成度错觉。Workbench的“Bake queued 87 lightmaps”“4 warnings”“6 texels”是静态字符串反馈；Baked Lighting与Irradiance Volumes editor plugin只注册descriptor/capability。Baked Lighting runtime plugin默认启用，贡献一个读写`scene-color`的post pass，但executor是`noop_render_executor`。实际lightmap模块始终拼入mesh shader，GPU Scene也无条件读取environment contract；`baked_lighting_enabled`只控制一个两条分支都返回default的退休post参数。Irradiance Volume同样不由插件拥有：core render在执行graph前无条件选择和prepare volume，插件bind executor随后重复选择、分配position Vec并写同一uniform。移除或禁用插件并不会移除真实采样。

消费正确性也未闭环。CPU把`light_set_generation`写进GPU instance和probe buffer header，但WGSL从不读取generation word；atlas resource cache只以AssetId命中，同ID hot reload或descriptor变化会继续持有旧`Arc<GpuTextureResource>`；probe grid cache只看generation，相同generation内容变化会静默保持旧buffer。`scene_snapshot.content_hash`只检查非零而不校验payload，`LightmapBakeOutput`也不携带snapshot/recipe/backend hash，故数字ID碰撞或错误复用不能被内容身份阻止。当前generation更像DTO标签，而不是atlas、slot、probe、shader共同原子发布的资源代际。

着色合同不足以承载Static/Stationary工程语义。所有direct lights、ambient和environment IBL继续作用于lightmapped surface；`AmbientLight.affects_lightmapped_meshes`未到达最终shader；没有light bake contribution、stationary shadow channel、shadowmask、directional lightmap、bent normal、sky visibility或specular occlusion。如果外部baker输出包含direct light，运行时会重复照明；如果只包含indirect，又没有artifact channel/schema明确这一约束。Deferred路径还把baked diffuse加进GBuffer emissive，SSS路径因此把它归入retained而非diffuse，调试视图也无法区分真实emissive与静态间接光。

Irradiance Volume只允许整view选择一个volume。选择依据是所有extract mesh的translation，不是可见bounds或per-object assignment；最高priority volume一旦被选中，shader在其范围外直接fallback，不会继续查找另一个相邻/重叠volume。layer只与camera layer相交，不与当前object layer比较。ResourceStreamer会为extract里的全部volume同步ensure完整3D LUT，错误被忽略；最终只绑定一张texture。没有adaptive placement、brick/cell、validity、visibility、relocation、dilation、leak prevention、streaming或lighting scenario。

本轮登记12项P0、20项P1、5项P2。P0先删除伪bake/伪plugin readiness，建立scene/editor/job/build-data/cook闭环、UV与atlas生产、物理静态灯光语义、原子generation发布和多volume空间消费；P1再解决cache、格式、shader语义、热路径分配、采样质量和测试证据；P2才进入分布式path tracing、超大世界virtual lightmap、lighting scenario和高阶重建。完成相同场景、相同静态光照质量、相同内存预算与相同硬件的Unreal/Godot/Unity对照前，不能声称性能或表现优于当前Unreal。

## 2. 审查边界与覆盖

### 2.1 已读范围

| 子域 | 文件 / 物理行 | 本轮判定 |
|---|---:|---|
| core lightmap / probe ABI | 5 / 1,312 | E3：request/output/consume、generation、SH9 grid、environment attachment与volume selection |
| asset / mesh UV / glTF import | 5 / 1,344 | E3：RGBA16F array转换、UV1保存/缺省、TEXCOORD_1导入及fixture |
| offline bake / streamer / renderer / WGSL | 22 / 3,967 | E3：伪bake、resource prepare、GPU Scene slot、forward/deferred、volume bind与最终采样 |
| baked/volume plugins与Editor表面 | 19 / 1,932 | E3：manifest、capability、executor、Workbench command feedback与控件绑定 |
| 合计 | 51 / 8,555 | 63个inline test属性；focused fingerprint `4b3864bdc34debc0d4f1fa8269921a5e758123df2fdf84cf279be908a7c18463` |

另抽查7个独立产品、integration或Python contract test文件、2,503行、21个test属性，包括`render_product_baked_lighting.rs`、M4 behavior layer、advanced-lighting irradiance fixture和Hybrid GI baked baseline。它们未计入focused fingerprint。跨owner调用点还核对了World mesh snapshot、stable instance key、render pipeline compile、resource revision和Reflection Probe admission；这些大文件只读与09F2直接相关的调用链，不重复计入上表。

“E3”表示静态读到数据构造、调用链、缓存/线程边界、shader消费与失败语义，不表示真实bake、GPU或Editor动态验收已经完成。

### 2.2 本轮归属与后续边界

- 09A拥有GPU submission、queue、fence、device loss和render graph resource truth；bake job与runtime upload必须使用该生命周期，不能用graph external-texture名字伪装一份未被graph拥有的uniform/texture。
- 09B拥有persistent RenderScene、visibility、bounds、stable object identity和GPU Scene；09F2依赖它提供可持久化bake subject identity、可见volume assignment和world partition cell，不继续用extract mesh translation猜测view内容。
- 09C拥有shader module、feature variant、PSO generation和GBuffer contract；09F2定义baked diffuse/shadowmask/probe channel，禁止再借用emissive保存静态间接光。
- 09D拥有asset handle、revision、residency、upload、streaming、budget和cook artifact；09F2定义LightmapSet/ProbeBrick专用内容与generation，不在render路径同步ensure全部体积纹理。
- 09E拥有direct light、mobility、shadow和光度真值；09F2增加Static/Stationary bake contribution、shadowmask和失效依赖，但不复制第二套Light component。
- 09F1拥有sky/IBL/reflection probe；09F2必须把sky contribution、Reflection Capture build data与lightmap/probe artifact放进同一scene build generation，不能让`offline_bake_frame`继续冒充两者的共同入口。
- 09F3单独审查Hybrid GI。09F2提供可信的static indirect、probe hierarchy和history identity；Hybrid GI不能把手工fixture lightmap当作真实bake输入。

### 2.3 参考引擎边界

- Unreal用`UMapBuildDataRegistry`按GUID持有mesh、light、reflection capture、precomputed light volume与volumetric lightmap build data，并提供分类型invalidate。GPULightmass有Full/Bake What You See、tile pool、GI/stationary shadow samples、denoise、irradiance cache、ray guiding、progress/start/stop/save。Volumetric Lightmap包含indirection texture、brick atlas、SH、sky bent normal、directional shadowing和sublevel streaming。Zircon当前没有对应owner。
- Bevy明确说明自身不内置lightmap/irradiance baker，这种边界比Zircon名不副实的`offline_bake_frame`更诚实。它仍提供entity `Lightmap` component、visibility extraction、pending readiness、bindless slab、bicubic option，以及支持时的multiple irradiance-volume binding；可作为Rust runtime消费架构参考，不作为最终离线质量上限。
- Godot `LightmapGI`/`LightmapGIData`把scene node、可序列化user mapping、texture array、shadowmask、probe生成、bake质量/反弹/denoise和Editor错误处理接通；`LightmapperRD::bake`有真实geometry/light/environment输入与progress callback。其传统限制只应视为产品闭环下限。
- Fyrox的CPU lightmapper规模更小，但已经有scene light/geometry extraction、自动TexCoord1生成与vertex patch、并行计算、取消、阶段进度和可序列化node/surface lightmap mapping。Zircon不能以“仍在早期”为由低于这一基础工程面。
- Unity Graphics可见源码中，PathTracing lightmap包含chart identification/rasterization、UV overlap detection、direct/indirect/AO/shadowmask/validity integration；Adaptive Probe Volume包含placement、subdivision、dilation、virtual offset、sky occlusion、rendering layer、scenario、serialization与disk-to-GPU cell streaming。仓内不含Unity全部native lightmapper，本报告只引用可见SRP contract。

### 2.4 明确未做

本轮没有修改production code，没有运行Cargo、真实baker、Editor、cook、WGPU、RenderDoc、device loss、VRAM pressure、视觉golden或对照引擎benchmark。`lightmap_asset.rs`、irradiance volume module、lightmap binding及相邻environment代码存在其他Session修改，因此标记`source_recheck_required`；实施前必须重算fingerprint并复核行级结论。

## 3. 当前必须保留并迁移的基础

### 3.1 versioned request/output/consume DTO可以升级为正式协议

现有合同已经拒绝零generation、重复instance、非法UV rect、atlas page缺失/重复/越界、RGBA16F payload尺寸错误、probe维度溢出、SH9数量与非有限值，以及request/output数字身份不一致。应把它升级为content-addressed bake manifest和artifact schema，不应退回无版本二进制blob。

### 3.2 EnvironmentExtract的单generation校验方向正确

`try_with_baked_lighting`要求Lightmap与Probe Grid共享`light_set_generation`，这是原子Light Build Set的雏形。目标实现应把atlas、slot table、shadowmask、directionality、probe bricks、sky contribution与light revisions全部纳入同一不可变generation，并由资源系统一次发布。

### 3.3 glTF UV1与GPU vertex ABI已经贯通

Importer读取`TEXCOORD_1`，MeshAsset保存`uv1`，GpuMeshVertex和material shader template都传递第二UV通道。重构需要补producer/validator和missing-channel诊断，不应改掉现有vertex ABI或再增加一套重复字段。

### 3.4 stable instance slot和Static mobility gate可作为迁移起点

GPU Scene会按stable instance key关联atlas page/rect，只有`Mobility::Static`实例取得lightmap。这个原则正确，但identity必须从entity+primitive ordinal升级为可重导入、prefab、LOD、world partition与cook稳定的BuildSubjectId，并建立失效映射。

### 3.5 SH9 CPU/GPU布局和trilinear oracle可保留

Probe grid已经有CPU trilinear sampler、GPU header/SH9布局和有限值测试，可作为新brick/probe hierarchy的低阶reference oracle。目标系统需要加入visibility/validity/leak correction、分层streaming和平台压缩，而不是删除数学基线。

### 3.6 irradiance volume transform与normal处理有有效测试

World-to-volume、非均匀缩放normal matrix、priority/tie break、inside test和`affects_lightmapped_meshes`已经有针对性测试。应把这些语义迁移到per-object、多volume assignment，而非继续保留single-view winner。

### 3.7 forward/deferred产品fixture可转成新pipeline回归

现有WGPU fixture至少覆盖一张真实RGBA16F array在forward/deferred可见。实施时将其输入改为真实scene bake、artifact reload与resource generation publish；小型手工fixture仍可作为shader unit oracle，但不得继续充当端到端产品证明。

## 4. P0 差距清单

### P0-1：`offline_bake_frame`是错误命名和错误能力表面

函数只按DirectionalLight intensity与前N个mesh生成无`baked_cubemap`的球形Reflection Probe metadata；不生成本文任何lightmap/probe-grid成果。必须删除公开导出，或在迁移期明确重命名为test-only probe-layout heuristic；正式入口只能是返回typed job handle、progress、diagnostics和artifact generation的Bake Service。

### P0-2：scene、prefab、World和project asset没有静态光照authoring

没有BakedLightingSettings、LightmapSettings、IrradianceVolume component、LightmapImportance/Probe Density volume、per-renderer scale/index或build-data reference。普通项目无法save/load、duplicate、undo、script、prefab override或cook这些功能。必须先建立versioned scene schema和migration，再允许实现baker。

### P0-3：Editor烘焙工作区和两个editor plugin都是能力占位

Workbench只回写静态字符串；plugin只提供descriptor/capability，没有command、selection、validation、preview、cancel、progress、result import、dirty/undo或error table。必须把UI接到同一Bake Job API，并让显示的数量、耗时、warning、UV和artifact来自后端快照，禁止保留伪成功反馈。

### P0-4：没有UV展开、chart packing、gutter与可烘焙性验证

glTF只保留外部`TEXCOORD_1`；OBJ/缺失通道回落到零UV，MeshAsset甚至不保存全零UV1。没有自动unwrap、vertex split、chart ID、overlap、inverted/degenerate UV、padding、texel density、max atlas或LOD一致性验证。目标pipeline必须产生可版本化geometry patch或derived mesh，不允许shader静默在`uv1=(0,0)`采整片mesh。

### P0-5：没有真实Lightmap/Probe bake backend和可恢复job系统

`LightmapBakeRequest`无人构造、无人消费，`LightmapBakeOutput`只在测试中手写。必须实现typed scene snapshot、geometry/material/light/environment extraction、CPU reference与GPU/path-traced backend、阶段进度、取消、暂停/恢复、OOM/设备丢失、确定性seed、失败artifact保留和headless cook worker。

### P0-6：没有Build Data Registry、artifact identity与增量失效图

`scene_snapshot.payload`是无schema opaque bytes，content hash只要求非零；output不记录snapshot hash、recipe、backend、engine/shader version、platform、material/mesh/light dependency或producer。必须用content-addressed BuildManifest与per-subject dependency graph持久化LightmapSet、ShadowMask、ProbeBrick和diagnostics，并能按geometry/material/light/sky/settings变化精确invalidate。

### P0-7：Static/Stationary灯光、shadowmask与双重照明语义不存在

Direct light仍全部实时着色，ambient/IBL也不排除lightmapped surface；外部bake到底包含direct还是indirect没有schema。必须在09E Light contract上增加bake contribution、Static/Stationary/Movable行为、stationary shadow channel allocation和冲突诊断，并在shader明确组合baked diffuse、dynamic direct、shadowmask、environment与specular。

### P0-8：Baked Lighting与Irradiance Volume插件不拥有真实能力开关

Baked Lighting默认插件运行noop post pass，实际mesh采样始终存在；Irradiance Volume core无条件prepare，插件又重复执行。必须硬切换到唯一owner：关闭feature时extract、resource、shader variant与pass都不消费数据；开启时由真实feature注册资源/依赖/执行器。删除noop composite和core/plugin双写，不保留旧名称别名。

### P0-9：Light Build generation没有贯穿GPU资源原子发布

atlas按AssetId早退、probe grid只按generation缓存，instance/probe header中的generation又不被shader读取。必须发布不可变`BakedLightingSetGeneration`，其中所有GPU handle和lookup table一起ready；shader/bind group只能看到同一代，hot reload使用last-good直到新代完整，错误代必须拒绝而不是混采。

### P0-10：Irradiance Volume的single-view winner算法在重叠和大场景中错误

renderer选择一个包含任意mesh origin的最高priority volume并全屏绑定。相邻volume不能同屏，低priority overlap不会作为逐像素fallback，大mesh bounds与origin不一致，offscreen extract mesh也能影响选择。必须由09B visibility/spatial structure生成per-object/cluster volume list，支持多个volume、priority/blend/fade和确定性fallback。

### P0-11：Probe Volume没有producer、有效性、遮挡、去漏光或streaming

当前volume只是任意D3 post-process LUT的ambient-cube解释；global grid是完整SH9 Vec。没有placement、adaptive subdivision、geometry validity、backface/visibility moments、relocation、classification、dilation、sky occlusion、brick/cell、world partition和disk-to-GPU streaming。目标最低线应达到Unreal VLM或Unity APV的分层数据与leak-control能力。

### P0-12：没有统一readiness、诊断、预算和竞争性验收

Lightmap atlas未准备会返回render error，volume ensure错误被丢弃，Editor却显示固定成功。没有bake ETA、failed subject、stale reason、generation、atlas occupancy、probe validity、upload/VRAM、fallback、stream queue或per-frame采样成本。必须建立同一diagnostic snapshot并制定与Unreal相同场景、质量、分辨率、静态灯数、内存预算和硬件的benchmark协议。

## 5. P1 差距清单

### P1-1：slot DTO公开可变且每帧重建HashMap

`LightmapConsumeContract.slots`是public Vec，`slot_for_instance`线性查找；GPU Scene sync每帧按整表重新分配HashMap。应在validated generation内部持有不可变、排序或hash-indexed lookup，稳定scene不产生heap work；mutation只能生成新generation。

### P1-2：stable key依赖entity与primitive ordinal，缺少asset revision映射

`(entity << 16) | primitive_ordinal`在primitive重排、model reimport、LOD变化、prefab实例化与world partition迁移时可能指向不同surface。Build Data Registry应使用稳定scene object UUID、mesh subasset identity、section/LOD key和geometry revision，迁移时显式报告orphan/missing slot。

### P1-3：output可静默漏掉request中的instance

`validate_against`拒绝未请求slot，却不要求每个requested static instance都有结果。若某对象不可烘焙，应输出typed skipped/failed diagnostic及原因；若要求complete build，则缺失必须使generation不可发布。

### P1-4：atlas descriptor校验不含设备与内容约束

绑定只检查format、D2和array layer count，不核对width/height等于`page_size`、mip count、sample count、resource revision或内容hash。Bake/cook还应校验device page/layer limits、总字节预算、平台格式能力与空页策略。

### P1-5：atlas cache只按AssetId命中会保留hot-reload旧资源

同一AssetId替换TextureAsset后，ResourceStreamer可以准备新revision，但`SceneLightmapResources`因`atlas_asset == contract.atlas`提前返回并继续持有旧Arc。cache key必须包含resource revision、descriptor和BakedLightingSet generation，并有hot-reload回归测试。

### P1-6：probe grid cache只看generation且整buffer重建

相同generation但内容改变会保持旧buffer；generation变化则用`create_buffer_init`重建全部SH9数据，没有staging budget、partial cell update或last-good fence。目标brick streamer应按cell/chunk revision更新，并对同generation内容漂移报错。

### P1-7：GPU已上传的generation字段是死数据

`GpuInstanceData.lightmap_params.zw`和probe header generation没有任何WGSL consumer。要么由bind set identity保证绝不需要逐像素比较并删除死字段，要么在debug/validation variant比较atlas/slot/probe generation；不能保留看似安全但无效果的ABI。

### P1-8：RGBA16F atlas转换复制、排序并长期保留raw page

`texture_asset_from_lightmap_bake_output`先收集引用、排序，再flat-map复制整份payload；调用方通常还持有原`LightmapBakeOutput` pages。大场景会出现bake output、container bytes、decoded upload与GPU texture多份峰值。应由artifact writer流式编码page/chunk，并允许move/zero-copy staging和及时释放CPU bake buffer。

### P1-9：lightmap只有单mip raw RGBA16F，无平台压缩或virtualization

一页固定8 bytes/texel，没有mip chain、BC6H/ASTC HDR等平台格式、directionality分离、texture streaming或virtual lightmap。远距离会alias，超大场景只能扩大常驻array。格式recipe必须进入cook identity和画质profile。

### P1-10：atlas采样没有gutter、texel-center或chart边界处理

WGSL只把UV1乘scale加offset后全局clamp到0..1；linear filter可跨相邻chart取样，单mip也无法用正确LOD。Atlas packer必须记录interior/padded rect，dilate边缘并按texel center/mip gutter采样；bleed regression要覆盖多page和极端缩放。

### P1-11：Deferred把baked diffuse编码进emissive

`zr_template_deferred_gbuffer.wgsl`将baked indirect加到`output.emissive`。Deferred SSS随后把它归入retained，绕过diffuse profile；GBuffer debug、material diagnostics和任何依赖emissive语义的后续效果也被污染。应增加独立baked-diffuse channel，或在deferred lighting阶段用surface/lightmap inputs重建，不能继续复用emissive。

### P1-12：bake payload只有无语义RGB irradiance

当前atlas只采RGB并乘base color/metallic diffuse scale/AO。没有encoding/exposure scale、directional dominant light、shadowmask、bent normal、validity、sky visibility、specular occlusion或per-light metadata。Artifact schema与shader ABI必须显式声明每个channel的物理含义和组合顺序。

### P1-13：global SH9 grid每个动态fragment读取72个vec4

Trilinear插值访问8 probes，每个probe读取9个vec4，另有header与循环开销；没有brick locality、cache-aware packing、L1/L2 quality tier或compute preclassification。应以可见对象/cluster选择短地址，按平台选择压缩SH/ambient cube，并用GPU counter验证带宽。

### P1-14：grid边界外直接归零且没有层级fallback

动态物体越过grid最大坐标会突然失去全部baked irradiance；没有border cell、fade、sky probe、coarser cascade或neighbor volume。需要连续的fallback hierarchy和越界diagnostic，验收移动物体穿越cell/volume/world partition边界。

### P1-15：volume selection在core和plugin各分配一次mesh-position Vec

存在volume时，core render收集全部extract mesh translation并prepare；graph executor再重复collect/select/clone/write。应由visibility阶段一次生成不可变view assignment，feature executor消费句柄；稳定帧不得有重复Vec、clone或uniform write。

### P1-16：volume layer只按camera过滤，不按object过滤

volume selection和binding都只知道selected camera layers；shader入口没有当前object layer。一个camera内不同render layer的对象会采同一volume。需要在GPU Scene携带lighting layer并在per-object/cluster assignment阶段相交，和09E direct-light layer共用ABI。

### P1-17：ResourceStreamer预载全部volume且吞掉失败

`ensure_scene_resources`遍历extract中每个volume，调用完整3D LUT ensure，并丢弃`Result`；最终renderer只用一张。大场景会造成无空间优先级的I/O、CPU decode与VRAM常驻。应按view/cell importance发residency ticket，失败记录原因并采last-good/fallback。

### P1-18：Irradiance Volume复用post-process LUT类型，编码校验过弱

绑定只要求D3、height为2的倍数、depth为3的倍数；不检查专用schema/version、linear color、filterability、format、mips、hash或producer。任意3D LUT都可能被解释成ambient cube。应引入typed `IrradianceVolumeAsset`与cook validator。

### P1-19：volume uniform每次prepare都写，且single texture限制无能力分级

即使volume、transform和resource revision未变，prepare仍写params buffer。硬件支持binding arrays时也没有multiple path，低能力平台也没有明确closest-only quality downgrade。应按revision跳过稳定写入，并提供capability profile、最大volume数和可见降级提示。

### P1-20：测试数量掩盖producer、feature gate和artifact闭环缺失

多数测试是DTO validation、WGSL source-string或手工snapshot注入。M4 `offline_bake`测试生成的probe会被真实GPU admission过滤；没有missing UV1、atlas bleed、hot reload、generation mismatch at GPU、多volume overlap、scene roundtrip、Editor cancel、artifact corruption、device loss和large-world streaming。必须按验收矩阵重建测试层级。

## 6. P2 差距清单

### P2-1：分布式与多GPU path-traced baking

在单机可取消、可恢复、确定性job与artifact cache稳定后，再增加remote worker、multi-GPU tile分配、结果校验、失败重试和带宽感知调度。

### P2-2：自适应采样、ray guiding与高质量denoise

以CPU reference和固定seed path tracer建立ground truth，再加入variance-guided sampling、first-bounce guiding、OIDN/OptiX或自有denoiser；必须保存sample count、albedo/normal auxiliary和误差统计。

### P2-3：Virtual Lightmap与超大世界分层streaming

在普通atlas/page residency正确后，进入virtual texture page table、world partition build cells、camera/importance feedback、disk-to-GPU direct upload和跨cell连续probe hierarchy。

### P2-4：Lighting Scenario、day/night与多状态混合

目标可支持同一geometry上的多个静态光照scenario、异步预取、受控blend和memory budget；scenario identity必须进入build set，不允许直接混合不同recipe或geometry revision。

### P2-5：硬件光追reference、神经重建与动态增量更新

HWRT可以用于preview/reference和局部重烘，神经重建只能在误差、稳定性、跨GPU和fallback合同明确后采用，不能替代基础artifact、visibility和streaming工程。

## 7. 目标架构

### 7.1 Authoring层：可序列化组件与唯一Light truth

新增versioned `BakedLightingSettings`、`LightmapRendererSettings`、`IrradianceVolume`、`ProbeDensityVolume`与scene-level `BakedLightingDataRef`。Light component继续由09E拥有，但增加明确的bake contribution、stationary channel与invalidate classification。Mesh renderer公开generate/use UV1、resolution scale、receive baked lighting和LOD policy。

### 7.2 Bake Input层：typed、content-addressed scene manifest

`BakeSceneManifest`应列出稳定BuildSubjectId、geometry/material/light/sky/resource hash、transform/static state、lighting layer、recipe/backend/platform/tool version和world cell。Payload必须有正式schema，不再接受“非空Vec+任意非零hash”。每个dependency变化都能解释为何某subject/cell失效。

### 7.3 Geometry层：derived UV与atlas planning

独立UV pipeline负责缺失通道生成、chart split、overlap/degenerate检测、padding/dilation、texel density和multi-page packing。输出`DerivedLightmapMesh`与`AtlasPlan`，保留source-to-derived vertex/triangle映射；Editor能可视化chart、density、overlap和waste。

### 7.4 Bake Job层：后端无关、可取消、可恢复

统一`BakeJobHandle`、`BakeProgressSnapshot`、`BakeDiagnostic`和`BakeArtifactWriter`。阶段至少包括snapshot、UV、acceleration structure、direct、indirect、shadowmask、probe placement、denoise、encode、import/publish。CPU reference、GPU path tracer与remote worker共享manifest和output validator。

### 7.5 Build Data层：不可变generation与分类型artifact

`BakedLightingBuildData`按scene/world cell保存LightmapPageSet、InstanceAllocationTable、ShadowMaskSet、ProbeBrickSet、LightBuildMapping和diagnostics。每一代有完整content hash与dependency manifest；发布只在所有required artifact校验并可驻留后发生，旧代保持last-good。

### 7.6 Runtime资源层：09D residency与空间流式

Lightmap page、shadowmask和probe brick都是typed streaming resource，拥有revision、memory cost、priority、last-used fence和failure reason。View根据09B visibility请求object page与附近probe cells；upload走staging budget，禁止render线程同步decode整套volume。

### 7.7 Assignment层：per-object / cluster多volume列表

Persistent scene维护volume/probe spatial index。Visibility输出每个object或cluster的短列表，包含priority、blend、layer、generation和brick address；高能力平台走binding array/atlas，低能力平台明确closest-N降级。相邻与重叠volume必须连续可预测。

### 7.8 Shading层：物理channel分离

Forward/Deferred共享同一`BakedLightingSample`，至少区分diffuse irradiance、directionality/validity、shadowmask、bent normal/sky visibility和generation。Deferred不得借emissive传输；Static/Stationary/Movable direct light、ambient、IBL、baked diffuse和specular occlusion按唯一公式组合。

### 7.9 Editor层：真实命令、事务、诊断与preview

Lighting Build面板从Bake Service读取scene validity、jobs、ETA、memory、atlas/probe statistics、warnings和stale reasons。Bake/Cancel/Save/Clear/Invalidate是undo/dirty/cook-aware command；preview与headless cook使用同一manifest，不能各自实现隐式规则。

### 7.10 Telemetry与benchmark层

统一记录snapshot/UV/trace/denoise/encode时间、rays/s、samples/texel、atlas occupancy、artifact bytes、load/upload、resident pages/bricks、fallback count、shader fetch和GPU cost。竞争性验收保存场景、commit、driver、quality recipe和图像误差，禁止只报单一FPS。

## 8. Hard Cutover 规则

1. 删除公开`offline_bake_frame`、`OfflineBakeSettings`和`OfflineBakeOutput`伪表面；没有实际artifact的heuristic不得继续使用“offline bake”命名。
2. 删除Baked Lighting noop composite pass与退休post参数；actual sampling必须由真实feature owner开启/关闭。
3. 删除core render与Irradiance Volume plugin的双重prepare，保留唯一graph/resource owner。
4. `LightmapConsumeContract.slots`改为不可变generation内的validated mapping；不保留public mutation兼容层。
5. 新artifact schema发布后，不让raw `LightmapBakeOutput`直接充当长期asset；旧测试fixture通过显式migration tool转换，不在runtime双读。
6. 禁止同AssetId/数字generation绕过resource revision/content hash；所有cache改用完整Build Set identity。
7. Deferred baked diffuse迁出emissive后删除旧shader路径和source-string断言，不维护双GBuffer语义。
8. Workbench静态成功文本改为后端状态；没有job时必须显示Unavailable/Not configured，而非伪queue结果。
9. 手工`EnvironmentExtract`注入只保留在低层unit test，产品测试必须从scene asset、build、save/reload到pixel。

## 9. 分层实施里程碑

### M0：冻结失败证据、目标ABI与对照基准

先添加伪bake被GPU过滤、feature disable仍采样、same-ID hot reload、same-generation grid漂移、deferred-emissive alias和single-volume overlap的失败测试；保存当前CPU/GPU/RAM/VRAM与视觉基线。

### M1：建立scene authoring与Light bake语义

完成settings/components/property path/prefab/save-load/migration，接入09E mobility、bake contribution、lighting layer与shadowmask channel需求；尚未bake时明确NotBuilt状态。

### M2：建立BuildSubjectId、BakeManifest与Build Data Registry

定义typed snapshot、content hash、dependency graph、recipe/backend/platform identity和分类型invalidate；完成scene roundtrip、orphan mapping与artifact corruption测试。

### M3：完成UV生成、验证和atlas planner

实现derived UV1、chart packing、padding/dilation、density/overlap诊断、多page预算和Editor visualization；用Fyrox/Godot级别fixture验证missing/degenerate/overlap/large mesh。

### M4：实现确定性CPU reference baker

先完成可测试的小场景direct+indirect+environment+probe reference，支持cancel/progress和固定seed；它是GPU backend parity oracle，不要求首版性能领先。

### M5：实现GPU/path-traced bake backend与job scheduler

加入tile/queue budget、pause/resume、device loss、denoise和headless mode；Bake What You See只能作为显式preview模式，不能替代完整build。

### M6：完成artifact encode/import/cook与原子generation发布

输出mip、平台压缩、directional/shadowmask/probe chunks，接入09D async I/O/residency；新代完整ready后一次替换last-good，same-ID hot reload正确。

### M7：重构runtime lightmap lookup与GPU Scene

使用persistent immutable mapping和稳定BuildSubjectId，消除每帧HashMap；处理LOD/instance/mobility/world-cell变化，generation/debug validation贯通GPU。

### M8：修正Forward/Deferred静态光照组合

建立公共BakedLightingSample，移除emissive alias，接入Static/Stationary shadowmask、ambient/IBL排除与specular occlusion；验证Standard PBR、SSS、Unlit、透明边界和volumetric组合。

### M9：完成Probe Volume bake与artifact质量

实现placement、adaptive brick、validity、visibility、relocation/dilation、sky occlusion和leak tests；保留SH9/ambient-cube质量tier并以ground truth比较。

### M10：实现多volume assignment与cell streaming

由persistent scene/visibility生成per-object/cluster列表，支持overlap/blend/layer、world partition和disk-to-GPU streaming；低能力平台有明确closest-N降级。

### M11：硬切换plugin owner与Editor真实工作流

删除noop/duplicate executor，feature flag真正控制extract/resource/shader；Lighting Build面板接通job、cancel、save、clear、warning、UV/probe debug和undo/dirty。

### M12：资源预算、故障与soak

覆盖OOM、corrupt/missing artifact、device loss、hot reload、rapid scene edit、取消重启、多camera、长时间streaming和无可见内存增长；诊断能解释每次fallback。

### M13：同画质竞争性验收

在室内、室外、stationary shadow、多楼层、角色移动、超大世界和压力场景中，与Unreal GPULightmass/VLM及可见Unity/Godot实现对照build time、误差、frame time、RAM/VRAM、disk size、hitch和stream latency。只有全部门槛通过才允许“优于”结论。

## 10. 验收矩阵

| 维度 | 必须提供的证据 | 拒绝条件 |
|---|---|---|
| Scene roundtrip | 真实project中的settings、renderer、volume和build ref保存/重载一致 | 测试手工覆盖snapshot |
| UV与atlas | 自动UV、overlap/padding/density、多page、LOD和bleed golden | 缺UV静默采(0,0) |
| Bake correctness | CPU reference与GPU backend同recipe误差统计，direct/indirect/environment分项 | 只看一张主观截图 |
| Static/Stationary | 无双重照明，shadowmask channel稳定，Movable行为正确 | 通过关闭direct light规避 |
| Artifact identity | snapshot/recipe/dependency hash、corrupt/stale拒绝、增量失效解释 | 仅比较数字request id |
| Atomic publish | atlas/slot/shadowmask/probe同generation，hot reload保留last-good | mixed generation可见 |
| Forward/Deferred | PBR/SSS结果一致，GBuffer emissive不含baked diffuse | source-string断言代替像素 |
| Multi-volume | 同屏相邻/重叠、priority/blend、per-object layer、camera切换 | single global winner |
| Streaming | page/brick budget、world cell、disk-to-GPU、无同步render I/O | 预载全部volume |
| Editor job | Bake/Cancel/Resume/Save/Clear、progress、warnings、undo/dirty真实 | 固定成功文本 |
| Fault handling | missing/corrupt/OOM/device loss/cancel/restart有终态与诊断 | silent zero或吞Result |
| Performance | p50/p95/p99 build与frame CPU/GPU、RAM/VRAM、disk、hitch | 降质量/减灯却称更快 |

## 11. 参考实现映射

| Zircon目标owner | Unreal | Bevy | Godot | Fyrox | Unity Graphics |
|---|---|---|---|---|---|
| Build Data Registry | `UMapBuildDataRegistry` | 外部baker，entity component消费 | `LightmapGIData` | scene/node lightmap map | `ProbeVolumeBakingSet` / streamable assets |
| Bake scheduler | GPULightmass subsystem/tile renderer | 明确无内置baker | LightmapperRD + editor plugin | CPU parallel + cancel/progress | BakeLightmapDriver / APV bake pipeline |
| UV/atlas | Lightmap coordinate/density/VT | 依赖外部atlas/uv_rect | bake atlas与错误诊断 | `uvgen` surface patch | chart/overlap/rasterizer |
| Static light/shadow | light build data、stationary channel | runtime diffuse lightmap | shadowmask texture/mode | baked-light masking | direct/indirect/shadowmask kernels |
| Dynamic-object GI | Volumetric Lightmap bricks | irradiance volumes | generated probes | 无同级上限 | Adaptive Probe Volume |
| Streaming | VLM sublevel/world partition | asset readiness/bindless slab | texture arrays | 传统resource | cell/chunk disk-to-GPU |
| Runtime assignment | lightmap resource cluster/VLM indirection | per-entity lightmap、多probe binding | user mapping/probe data | node handle map | brick index/index-of-indices |
| Debug/diagnostics | density/VLM visualize/progress | readiness/logging | editor bake errors | progress stage/warn | probe debug/dilation/offset views |

参考代码用于确认owner、数据流、失效、调度和质量门槛，不要求复制其API。Unreal/Unity的复杂性也不能成为首版堆叠所有功能的理由；里程碑按依赖从可信producer、artifact、runtime到高级质量递进，但每层必须是可替换、可验证的工程实现，不接受临时假表面。

## 12. 证据缺口与风险

### 12.1 当前测试不能证明真实烘焙存在

`render_product_baked_lighting`读取fixture并手工注册TextureAsset；Hybrid GI测试也手写output；M4 `offline_bake`只生成会被GPU过滤的probe metadata。没有任何测试调用真实scene baker，因为仓内没有该实现。

### 12.2 没有动态验证伪bake测试是否当前可执行

静态调用链确定`baked_cubemap == None`会被Reflection Probe admission丢弃，因此该测试的像素差断言没有合法probe来源。本轮未运行Cargo/GPU，不能把“静态矛盾”扩写成已观察的失败日志；M0必须在Windows目标目录单独执行并记录adapter/feature状态。

### 12.3 reference版本不是直接性能结论

Bevy明确无内置baker，Fyrox CPU lightmapper不是质量上限，Godot与Unity/Unreal各有不同格式和平台假设。它们证明工程边界与缺失功能，不直接证明某算法一定适合Zircon；最终选择必须由同recipe误差和profile支撑。

### 12.4 源码正在被其他Session修改

本轮相关asset/environment/renderer文件有未提交变化，focused fingerprint只描述审查时内容。实现者领取M0前必须重跑状态、diff、fingerprint和关键symbol search；若false surface已被其他计划移除，应更新报告而不是恢复旧代码。

### 12.5 GPU/内存成本目前只有静态推断

72 vec4 SH读取、整volume preload、raw RGBA16F、每帧HashMap/Vec与uniform write均由代码路径可见，但尚无PIX/RenderDoc/WPR计时。优先级不因缺profile而取消，具体预算和优化方案要在M0/M12测量后冻结。

## 13. 完成定义

09F2只有在以下条件同时满足时才可关闭：

1. 普通项目能创作、保存、重载、prefab、script、undo和cook静态光照settings、renderer与volume。
2. 真实Bake Service从scene manifest产生UV/atlas、lightmap、shadowmask与probe artifact，支持progress/cancel/failure/restart。
3. Build Data Registry以content/dependency identity管理增量失效，并原子发布完整generation。
4. Runtime不在render线程同步decode，不预载全部volume，page/brick residency有预算、last-good与诊断。
5. Feature plugin真实拥有extract/resource/shader/pass开关，noop pass、重复prepare和伪Editor反馈全部删除。
6. Forward/Deferred/SSS使用明确的baked channel，无emissive alias、双重direct/ambient/IBL或silent missing UV。
7. 同屏多volume、layer、overlap、world cell与移动物体边界连续正确，probe有有效性与去漏光证据。
8. Scene-to-bake-to-artifact-reload-to-pixel、fault、hot reload、device loss、large-world soak测试全部通过。
9. 在相同画质与预算下完成Unreal/Godot/Unity参考场景的build/render/RAM/VRAM/disk/误差报告，竞争性结论有可复现实验支撑。

在此之前，现有代码应描述为“baked-lighting consumption ABI与实验性shader路径”，不能描述为完整Light Baking、Lightmap GI或Irradiance Volume产品能力。下一份图形审查为09F3 Hybrid GI。
