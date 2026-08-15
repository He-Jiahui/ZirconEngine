---
related_code:
  - zircon_runtime/src/asset/assets/scene/lighting.rs
  - zircon_runtime/src/asset/assets/scene/post_process.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/extract.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/volumetric.rs
  - zircon_runtime/src/scene/components/scene/lighting.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/render/lights.rs
  - zircon_runtime/src/scene/world/render_post_process.rs
  - zircon_runtime/src/scene/world/project_io/post_process.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting/froxel
  - zircon_runtime/src/graphics/scene/scene_renderer/lighting/light_buffer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/history/scene_frame_history_textures/volumetric_history.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/deferred
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/fallback_mesh.wgsl
  - zircon_plugins/rendering/features/volumetric_fog/runtime
  - zircon_plugins/rendering/features/volumetric_fog/editor
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/rendering/workbench_extension_post_process_workspace.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - docs/tests/runtime/render
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_runtime/09e-direct-lighting-clustered-shadow-review.md
  - docs/plans/optimize/zircon_runtime/09f1-environment-sky-ibl-reflection-probe-review.md
  - docs/plans/optimize/zircon_runtime/09f2-baked-lighting-lightmap-irradiance-volume-review.md
  - docs/plans/optimize/zircon_runtime/09f3-hybrid-global-illumination-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/VolumetricFog.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/VolumetricFogVoxelization.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/VolumetricFogLightFunction.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/LocalFogVolumeRendering.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/LocalFogVolumeRendering.h
  - dev/UnrealEngine/Engine/Shaders/Private/VolumetricFog.usf
  - dev/UnrealEngine/Engine/Shaders/Private/VolumetricFogVoxelization.usf
  - dev/UnrealEngine/Engine/Shaders/Private/VolumetricFogLightFunction.usf
  - dev/godot/scene/3d/fog_volume.cpp
  - dev/godot/scene/3d/fog_volume.h
  - dev/godot/servers/rendering/renderer_rd/shaders/environment/volumetric_fog.glsl
  - dev/godot/servers/rendering/renderer_rd/shaders/environment/volumetric_fog_process.glsl
  - dev/godot/editor/scene/3d/gizmos/fog_volume_gizmo_plugin.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/VolumetricLighting/LocalVolumetricFog.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/VolumetricLighting/HDRenderPipeline.VolumetricLighting.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/VolumetricLighting/VolumetricLighting.compute
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Editor/Lighting/VolumetricLighting/LocalVolumetricFogEditor.cs
  - dev/bevy/crates/bevy_light/src/volumetric.rs
  - dev/Fyrox/fyrox-impl/src/renderer
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 09G1 · Volumetric Fog / Froxel 工程化差距

## 1. 结论

Zircon当前Volumetric Fog不是空壳。产品链已经包含typed全局雾参数、Light的volumetric参与标记、Post Process Volume抽取、三段froxel compute、clustered light与shadow atlas消费、Beer-Lambert前向积分、history texture、forward/deferred/sky最终采样、render graph资源别名，以及runtime/editor plugin注册。High质量下实际创建160x90x96的RGBA16F 3D资源，RenderDoc artifact也能证明媒体注入、散射和积分dispatch曾经真实执行。以上基础应保留并迁移，而不是退回屏幕空间颜色覆盖层。

但当前实现仍是固定网格原型，不是可与Unreal Volumetric Fog或Unity HDRP竞争的工程级产品。Low、Medium、High固定为160x90x48/64/96，XY与viewport、宽高比、dynamic resolution和GPU预算无关；4:3、21:9、4K与低分辨率viewport会得到不同的实际像素覆盖和拉伸质量。RGBA16F current media/scatter/integrated经过alias仍需两份物理3D资源，High temporal再加一份history，分别约10.55、14.06、31.64 MiB；现有设置和Editor都看不到这笔预算。

局部雾体积的scene合同在到达GPU之前已经失真。Sphere被转成AABB，旋转Box被转成world AABB，原始shape、rotation、blend distance和priority全部丢失。shader只判断froxel world position是否落入AABB，然后把所有命中体积的uniform density/albedo直接相加。每个froxel线性遍历全部局部体积，没有tile/binning、上限、overflow策略、纹理/材质、形状SDF、边缘fade、emissive、流动或negative density carving。它不等价于Unreal的volume material voxelization、Godot的shape/material FogVolume，也不等价于HDRP的OBB、mask、face fade、priority与scrolling texture。

全局介质同样过于简化。高度公式使用`max(world_position.y, 0.0)`，把密度硬锚定到绝对世界Y=0；负Y区域保持最大密度，且没有base height、start/max distance、emissive、absorption/extinction分离、density texture、multiple scattering或sky/IBL/GI injection。局部和全局介质无法表达洞穴清雾、云雾层、异质烟尘、移动雾材质或大世界原点迁移。

时域路径有一个需要在实现前立即复测的具体单位错误：`TemporalJitterSample.offset_pixels`被直接加到froxel invocation，再除以160x90 grid，没有从viewport pixel换算成froxel cell。以3840宽viewport和160宽froxel为例，0.25 pixel应是0.0104 froxel，却按0.25 froxel使用，相当于约6个screen pixels。首帧没有有效history时仍可启用jitter；history用nearest `textureLoad`、固定0.9权重，仅以extinction alpha差异拒绝，缺少motion vector、depth/normal/disocclusion、radiance clamp、moments和局部光源/体积变更失效，因此运动镜头、薄遮挡与闪烁光源无法达到产品稳定性。

直接光散射虽然复用了clustered grid和shadow atlas，但物理与功能合同仍断裂。所有普通cluster candidate都先进入遍历，再用volumetric flag提前返回；CPU为每盏灯执行`Vec::contains`判断。point/spot使用简化的`(1-d/r)^2`，Rect Light忽略面积尺寸，cookie metadata已打包却未采样，ambient只来自显式Ambient Light，不接Sky/IBL、baked irradiance或Hybrid GI。shadow质量则继承09E中layer、bias、cache、visibility与面积光缺口。

最终合成对opaque/sky有真实深度采样，但transparent/OIT顺序不成立。forward fragment在alpha blend之前已经加入完整camera-to-fragment in-scattering并乘transmittance；OIT随后把这些已雾化的片元存储和组合到已经雾化的scene color上。多层透明会alpha-weight或重复计算in-scattering，不能表达沿每一段深度的介质传输。SSS/transmission与volumetric的能量/排序合同也未定义，需在09G2共同收口。

Editor侧目前只有能力登记与通用Post Process演示页面。页面硬编码`PPV_CityGlobal`、Cinematic、Bloom/Tonemap示例和“Preview queued / Apply queued”反馈，没有Volumetric Fog参数、Local Fog Volume、gizmo、纹理/材质、质量、VRAM、overflow、froxel slice、history rejection或GPU timing；反馈不修改scene，不进入transaction/undo/save，也没有runtime bridge。这是展示面，不是authoring工具。

现有产品证据不能关闭上述差距。`plan18_volumetric_compiled_scene_window_light_shaft_perf_wgpu_20260711.txt`首行明确为`diagnostic_failed`：brighter pixels为612、darker为23964、window-shaft brighter pixels为0、shaft与shadow-control平均delta都为-2.347、contrast为-0.000。对应384x128 PNG只显示很小的亮矩形，没有可信房间光束。RenderDoc资源统计能证明160x90x96资源和dispatch存在，但media slice近乎平色，scatter/integrated导出在当前显示下接近黑色。多项WGPU测试在无adapter时直接`return`，产品导出测试被`#[ignore]`；小16x8x8 fixture和source-string断言只能作为L1/L2证据。

本轮登记11项P0、22项P1、8项P2。重构顺序必须先冻结真实完成度和唯一数据合同，再建立viewport/scalability驱动的VBuffer、独立Local Fog Volume与GPU culling/material voxelization，随后补全光照/时域/透明合成、资源生命周期、Editor和竞争性验收。不能继续在CPU AABB数组和固定160x90网格上叠字段并宣称工程化完成。

## 2. 审查边界与覆盖

### 2.1 已读范围

| 子域 | 文件 / 物理行 | 本轮判定 |
|---|---:|---|
| 直接production与最终consumer focused set | 50 / 10,855 | E3：typed contract、scene抽取、froxel shader/executor、history、light packing、deferred/forward消费、plugin与Editor surface |
| production focused fingerprint | 50 / 10,855 | `c0b98ade19f50bd6881021787f7a83e1e6cef95e25e40ce0998b477a635e42e2` |
| production文件内test属性 | 32 | E2：多数验证packing、source inclusion、dispatch与小型数学合同 |
| dedicated froxel/plugin tests与fixture | 15 / 4,709 | E2：60个test属性；包含CPU oracle、WGPU readback、ignored PNG/RenderDoc exporter |
| Volumetric/Froxel artifacts | 20 / 28,250,439 bytes | E2：2个RDC、binding/resource stats、PNG和文本报告；产品gate当前失败 |
| Reference engine主链 | Unreal 8、Godot 5、Unity HDRP 4、Bevy 1、Fyrox renderer spot check | E3：runtime、shader、authoring和editor工具对照 |

focused fingerprint按路径排序，对每个文件计算SHA-256，再对UTF-8的`path<TAB>hash<LF>`清单计算SHA-256。范围包括froxel下17个非test production文件、volumetric runtime/editor plugin 6个文件、core/scene/asset contract 12个文件、graph/history/final consumer 11个文件和Editor surface 4个文件。当前其中18个文件存在本轮之外的modified状态，因此该摘要只绑定当前工作区快照，实施前必须重取。

### 2.2 数据链读取深度

本轮从scene asset和`VolumetricFogSettings`开始，追踪Light参与标记、camera layer过滤、Post Process Volume权重与collider转换、`AdvancedLightingExtract`、render graph resource声明、media inject、clustered light scatter、history reprojection、Z-column integrate、history copy，再检查forward/deferred/OIT/sky的最终绑定和应用。shader不仅核对binding，还核对了world reconstruction、slice distribution、height density、volume loop、light attenuation、shadow/cookie入口、history坐标/拒绝、积分公式与透明组合位置。

Editor读取了实际workspace资产、template binding、navigation spec和feedback handler，区分“注册了页面”与“页面能修改、撤销、保存并预览runtime真值”。证据读取同时核对了文本gate、PNG和RenderDoc导出slice，而不是只按文件名认定功能完成。

### 2.3 与相邻审查的owner边界

- 09A拥有render graph version/resource truth、persistent GPU object、pipeline/cache、bind group、queue/async compute、fence、history copy与device-loss。09G1定义VBuffer资源语义、预算和pass依赖，不再私建逐帧GPU对象owner。
- 09B拥有persistent RenderScene、view family、stereo/orthographic/dynamic-resolution view、GPU Scene、bounds和GPU culling。Local Fog Volume和volumetric light demand必须进入同一visibility/binning体系，不能每帧clone CPU Vec后全格扫描。
- 09C拥有material/shader permutation、PSO generation和cache。Volume Material、light cookie/light function与apply permutation必须复用该authority。
- 09D拥有density/mask/noise texture derived artifact、streaming、residency、budget、fallback和cook。09G1只消费ready generation，不能在render submission同步载入体积纹理。
- 09E拥有direct light光度、mobility、layer、cluster、shadow visibility/cache/bias与Rect Light面积模型。volumetric只定义参与、介质散射和专用采样，不复制另一套灯光真值。
- 09F1、09F2、09F3分别拥有Sky/IBL、baked irradiance与Hybrid GI。体积环境/间接散射必须消费versioned输出和明确能量合同，不能用显式Ambient Light替代全部环境能量。
- 09G2拥有cookie产品化、OIT、planar reflection、SSS与transmission。09G1负责介质沿深度的组合规则；跨透明/SSS的最终pass顺序由两份计划共同验收。
- 09H拥有velocity、jitter authority、camera cut、history generation、upscaling和全renderer temporal rejection。09G1不能维护不兼容的第二套pixel jitter与失效规则。

### 2.4 参考引擎边界

- Unreal是主要上限基线。其GridPixelSize/GridSizeZ由scalability和view计算，支持temporal reprojection、Halton 2/3/5、history miss supersampling、emissive permutation、local fog volume、volume material voxelization、local lights、shadow和directional light function。Local Fog Volume还是独立runtime系统，而不是把Post Process Volume collider降成AABB数组。
- Unity HDRP提供更直接的资源/authoring下限。LocalVolumetricFog保持OBB transform、priority、blending、volume mask、texture/material mask、tiling/scrolling、六面fade、distance fade、falloff与indirect rendering；pipeline按viewport、screen fraction和slice count计算体积网格，并通过render graph管理history/filter/APV组合。
- Godot提供独立FogVolume node、World/Box/Ellipsoid/Cone/Cylinder shape、FogMaterial/ShaderMaterial、negative density carving、SDF shape评价、temporal jitter、clustered light/shadow、editor warning与gizmo。它证明轻量引擎也不应在scene extract阶段丢失shape和transform。
- Bevy的体积雾能力更窄，但公开限制、step count/jitter/ambient、density texture与scrolling、absorption/scattering/asymmetry合同较诚实。可参考Rust组件边界，不能作为画质上限。
- Fyrox当前没有与前三者等价的专用volumetric fog系统，只用于传统fog/fallback和Rust renderer ownership抽查；不得因其缺失而降低Zircon目标。

### 2.5 明确未做

本轮没有修改production code，没有运行Cargo、Editor、WGPU或RenderDoc，没有重新生成失败的2026-07-11 artifact，也没有运行Unreal/Godot/HDRP。未执行camera cut、rapid light movement、animated density texture、透明多层、粒子、world-origin rebasing、stereo、orthographic、ultrawide、dynamic resolution、VRAM pressure、device loss或同画质GPU benchmark。PNG与RDC导出只做静态证据检查。

## 3. 可保留并迁移的基础

### 3.1 三段froxel产品链是真实实现

Media Inject、Light Scatter和Integrate都有独立WGSL、typed request、dispatch和3D texture，最终结果进入forward/deferred/sky。重构应保留阶段边界和可检查资源名，将固定资源升级为viewport/scalability VBuffer，而不是合并成不可诊断的单pass。

### 3.2 Beer-Lambert积分公式方向正确

Integrate沿Z列执行front-to-back scattering/transmittance累积，并处理低extinction极限。该数学基础可作为CPU oracle和GPU golden test，后续主要扩展介质参数、采样与并行/缓存策略。

### 3.3 clustered light和shadow atlas复用方向正确

volumetric scatter消费正式light grid、light buffer和shadow atlas，没有单独遍历scene light component重建GPU灯光。应把volumetric participation、cookie、面积光、layer和shadow generation补完整，而不是另建第二套light manager。

### 3.4 history generation、quality失效和末帧copy可迁移

history texture具有persistent owner，quality变化会失效，当前帧完成后再复制到history。它比把history藏在executor局部静态状态更可靠。后续应接入09H统一generation/camera-cut/disocclusion，并保留显式copy/publish边界。

### 3.5 graph lifetime alias已有实际收益

Media与Integrated资源生命周期不重叠时可共享物理slot，High之外不分配history。该方向可继续扩展到typed resource class、预算和alias验证，但不能把logical name相同当成资源正确性的充分条件。

### 3.6 typed settings、schema、scene I/O和layer参与可升级

密度、albedo、phase、height falloff、scattering、Z distribution、temporal、quality以及Light volumetric flag已贯穿serde和extract。需要版本化迁移并补足物理/工程字段，不必废弃所有现有资产。

### 3.7 小型WGPU readback可保留为L2 shader验证

16x8x8 fixture适合验证inject/scatter/integrate数值和binding。它们应明确标为unit/integration，不再替代真实房间、室外、大尺度、运动和性能产品gate。

## 4. P0 差距清单

### P0-1：唯一现有产品光束证据明确失败

`plan18_volumetric_compiled_scene_window_light_shaft_perf_wgpu_20260711.txt`记录`diagnostic_failed`、window-shaft brighter pixels为0、shaft contrast为-0.000；PNG也没有可信体积光束。任何“Volumetric Fog完成”状态必须回退为prototype/path-integrated。先建立可重复场景、指标与当前源码artifact，禁止用RenderDoc中存在3D resource替代画质验收。

### P0-2：Local Fog Volume的shape、transform、blend和priority在抽取时丢失

`render_post_process.rs`只输出world AABB。Sphere、rotated Box、blend distance、priority和原始collider shape无法到达GPU，最终画面与authoring对象不一致。必须建立独立`LocalFogVolumeComponent`/asset contract，保留shape、world-to-local transform、fade、priority、layer和material generation；旧AABB DTO在迁移完成后硬切除。

### P0-3：固定160x90网格不满足viewport、宽高比和scalability合同

Low/Medium/High只改变Z或temporal，不根据render extent、dynamic resolution、FOV、stereo eye、目标pixel size或GPU预算计算XY。必须以view-local extent和screen percentage生成VBuffer，设置可由scalability profile和实时budget controller解析，并在resize/quality变化时原子切换resource generation。

### P0-4：全局介质锚定绝对Y=0且参数不足以表达工程场景

`max(world_position.y, 0.0)`使负Y全部处于最大密度。必须提供base height/world-origin aware transform、start/max distance、extinction/absorption/scattering、emissive、environment contribution、multiple scattering近似和可选heterogeneous density；大世界rebasing后画面不得跳变。

### P0-5：每个froxel遍历全部局部体积，无上限、culling或overflow真值

复杂度为`grid_x * grid_y * grid_z * local_volume_count`，High单视图已有1,382,400个froxel。100个局部体积意味着仅inject就有约1.38亿次volume candidate判断，且没有typed cap/overflow/fallback。必须按view frustum筛选并在GPU构建tile/cluster/brick list或voxelize indirect draw，所有容量都要有counter、overflow reason和last-good策略。

### P0-6：Temporal jitter把screen pixel直接当froxel cell

当前`offset_pixels`直接加入froxel invocation。必须由09H提供统一jitter单位，或显式执行`pixel_offset * grid_xy / viewport_xy`；首帧/history invalid/camera cut不得抖动到未配对的history空间。增加4K、720p、ultrawide和dynamic-resolution数值测试，防止单位再次漂移。

### P0-7：Temporal history只有nearest sample、固定0.9与extinction差异拒绝

没有motion/depth/normal/disocclusion、radiance neighborhood clamp、moments、confidence或光源/介质mutation generation。必须采用filterable/trilinear历史采样、per-froxel confidence、camera cut和resource generation，结合当前/历史深度范围及介质/光照变化拒绝；history miss需要受预算约束的额外采样或空间滤波。

### P0-8：Transparent/OIT的介质传输顺序会重复或错误加权in-scattering

透明片元在OIT存储前已应用完整camera-to-fragment fog，随后再与已经fogged的opaque scene组合。必须定义按depth segment积分的透明合同：至少保留片元深度/coverage并在resolve按层处理transmittance与in-scattering，或使用可证明误差界的近似。两层玻璃、粒子、半透明体积和折射必须进入产品gate。

### P0-9：Volumetric direct/environment lighting低于正式光照合同

Rect Light不是面积光，punctual attenuation不是统一光度模型，cookie/light function未采样，非volumetric灯仍消耗cluster遍历，ambient不接Sky/IBL/Baked/HGI。必须消费09E/F的versioned lighting结果，支持volumetric participation mask、cookie/function、shadow、area approximation与环境/间接散射，并定义能量守恒和fallback。

### P0-10：Editor没有可保存、可撤销、可诊断的Volumetric Fog authoring

plugin只注册能力，通用Post Process页面是硬编码演示。必须提供全局雾profile、Local Fog Volume、shape gizmo、volume material/texture、priority/fade/layer、quality/预算、preview、debug slice和GPU timing；所有操作必须经过transaction、dirty/save、undo/redo、play/runtime同步和validation warning。

### P0-11：GPU缺席会静默通过，产品测试默认不运行

多个froxel/WGPU测试在adapter/device不可用时直接`return`，PNG/RenderDoc exporter被ignore，source-string包含断言又容易在未执行shader时变绿。CI必须区分`passed`、`skipped-capability`和`not-run`，产品lane缺GPU时失败或明确不计入acceptance；当前失败artifact未被新artifact替换前不得关闭P0-1。

## 5. P1 差距清单

### P1-1：介质参数没有物理单位、范围和迁移语义

`density`、`scattering_intensity`和height falloff没有单位说明，albedo/phase只做clamp。定义extinction的世界单位、mean free path换算、color space、phase范围、legacy migration与极端值行为，避免项目尺度变化导致完全不同画面。

### P1-2：Global与Local Fog复用Post Process Volume，owner边界错误

后处理权重体积和参与介质的几何/材质/渲染生命周期不同。保留profile对全局雾的覆盖能力，但Local Fog应成为独立scene component/render primitive；不得继续依赖任意Collider类型猜测雾形状。

### P1-3：局部体积只支持uniform additive AABB

补齐Box/Sphere/Ellipsoid/Cylinder/Cone/World或清晰裁剪的首批shape，保持world-to-local transform；支持face/radial/distance fade、priority/blend mode、density/albedo/emissive texture、tiling、scrolling和material parameters。

### P1-4：不能用negative density或显式operation carve fog

sanitization禁止负density，重叠体积只能相加。定义Add/Replace/Min/Max/Subtract或受控negative extinction authoring，并在物理clamp前完成组合；Editor必须警告不可稳定或非物理配置。

### P1-5：局部体积数据每帧clone，缺少stable identity和generation

`fog_volumes_for_layers`返回新Vec，无法增量更新或追踪哪一个volume变化。接入09B stable render primitive identity，维护persistent GPU record、dirty generation和visibility result，只上传变化range。

### P1-6：volumetric light membership在CPU执行线性`Vec::contains`

light buffer packing对每盏灯扫描volumetric ID Vec。将参与位直接放入light extract/packed light record或按stable ID构建O(1) lookup；更进一步为froxel构建只含参与灯的candidate mask，避免shader早退成本。

### P1-7：volume material/texture没有09C/09D产品链

设计可编译的Volume Material domain与受限shader permutation，明确density/albedo/emissive输出、world/local coordinates、noise/texture residency和fallback。不得从任意surface shader拼接或在submission时同步加载纹理。

### P1-8：Z slice范围绑死camera near/far，缺少可控雾距离

雾体积通常不需要覆盖完整far plane。分离view far与fog max distance，支持start distance、near fade和distribution exponent；对infinite far、reversed Z、orthographic和stereo给出明确定义。

### P1-9：Low/Medium的能力是硬编码削减而非scalability profile

Low禁Local Fog，Medium禁Temporal，High全开，没有预算、设备能力、场景复杂度或可解释fallback。把grid pixel size、slice count、history、filter、local volume/cookie/shadow质量拆成profile字段，并记录resolved reason。

### P1-10：3D资源预算不可见且没有多视图上限

当前两份物理current资源约为Low 10.55 MiB、Medium 14.06 MiB、High 21.09 MiB，High history再加10.55 MiB，总计31.64 MiB/视图，未计alignment与临时资源。建立per-view/per-family/global budget、最大view数、allocation failure fallback和Editor/telemetry展示。

### P1-11：每dispatch创建uniform buffer和bind group

Media还每帧创建local-volume storage buffer，最终apply又在forward/deferred/sky分别创建参数和binding。迁移到09A persistent ring/upload arena、bind group cache和generation-aware resource set；静态volume/light layout变化才重建相关binding。

### P1-12：Compute pipeline明确`cache: None`

三个froxel pipeline虽被`Mutex<Option<_>>`持久化，但没有统一PSO cache、warmup manifest、shader generation或driver cache。接入09C PSO authority，确保quality/permutation切换不在首帧编译卡顿。

### P1-13：AsyncCompute只是声明，executor可回退Graphics且无重叠证据

保留graph pass的AsyncCompute意图，但必须由09A证明queue ownership、resource transition、fence和与shadow/opaque工作重叠；否则准确标记为compute-on-graphics。验收记录GPU timeline而不是只看pass kind字符串。

### P1-14：ambient只累计显式Ambient Light

建立environment radiance输入：Sky/IBL、baked irradiance、APV/irradiance volume和Hybrid GI按质量/模式进入froxel。定义一次散射与multiple-scattering approximation，避免把同一间接能量重复注入。

### P1-15：light cookie/light function metadata没有进入散射采样

09G2完成cookie资源后，volumetric scatter必须按light type和projection采样同一ready generation，包含wrap/fallback、mip和shadow组合。Unreal directional light function路径可作为功能边界参考。

### P1-16：shadow限制被透明地继承但没有volumetric diagnostics

缺少per-light shadow route、slot missing、layer mismatch、cache age与bias在体积中的可视化。增加unshadowed/shadowed scattering debug、shadow generation和fallback reason，避免把shadow缺失误判为雾参数错误。

### P1-17：world-position重建与大世界精度没有专项合同

VBuffer依赖inverse view-projection重建世界位置，绝对Y又参与指数。引入camera-relative坐标或明确origin generation，对大坐标、origin rebase、near/far极端和FP16 storage做误差测试。

### P1-18：最终apply参数在多个consumer中复制

Forward、Deferred、Sky各自构建参数/bind group，容易在viewport region、depth convention和history generation上漂移。建立统一`VolumetricApplyView`/binding contract和generation，consumer只选择是否/何时应用。

### P1-19：SSS与Transmission的能量/顺序未定义

SSS retained target、deferred lighting、transmission和volumetric apply之间需要正式pass ordering。定义介质发生在surface shading前后哪些部分、厚度/背光如何组合、哪些buffer仍是scene-linear pre-exposed值，并加入09G2交叉测试。

### P1-20：缺少froxel/local-volume/light debug views

提供media extinction/albedo、scattering、integrated radiance/transmittance、slice occupancy、local-volume bins、light count、history confidence/rejection、overflow、shadow/cookie route与NaN/Inf heatmap。Editor必须能选slice和probe point，不只导出离线PNG。

### P1-21：统计只证明dispatch，不证明成本或质量

现有`RenderStats`记录3次dispatch、group数和624 upload bytes，但不记录GPU ms、volume/light candidates、history rejection、VRAM、allocation/cache、overdraw或quality reason。建立稳定schema和per-view aggregation，并控制metric cardinality。

### P1-22：scene/profile序列化缺少版本与跨模式迁移测试

新增参数、独立Local Fog component和material后必须提供schema version、legacy Post Process Volume迁移、unknown-field保留/警告、asset dependency与cook测试；不能让旧资产静默变成不同shape或密度。

## 6. P2 差距清单

### P2-1：`FogVolumeData`命名掩盖其AABB-only语义

迁移期间把旧类型明确标记为LegacyAabbFogVolume或在类型中带shape/transform，避免调用者以为它已表达完整局部雾。

### P2-2：缺少平台/设备能力报告

记录3D storage texture格式、filterability、max dimension、memory budget、async capability和fallback quality；Editor和artifact都应带adapter/backend/driver信息。

### P2-3：缺少可复用的质量preset与项目覆盖层

提供平台scalability preset、project default、camera override和runtime budget override，明确优先级和resolved snapshot，不依赖硬编码Low/Medium/High分支。

### P2-4：缺少体积雾asset thumbnail与快速预览

Local Fog material/texture应有稳定thumbnail、小场景preview和编译/驻留状态，帮助作者发现全黑、全白、过高extinction与缺失纹理。

### P2-5：缺少非物理参数警告与修复建议

对极端phase、near-zero mean free path、巨型重叠volume、负密度operation、不可驻留texture、超过bin capacity和world-scale不匹配给出结构化warning。

### P2-6：测试命名没有区分数学、GPU路径和产品验收

按L0数学、L1 shader compile、L2 GPU readback、L3 renderer integration、L4 visual product、L5 performance/soak分类artifact和test name，避免`product`文件中仍是16x8x8 fixture。

### P2-7：参考引擎能力映射尚未沉淀为feature matrix

为shape、material、lighting、shadow、cookie、temporal、transparent、editor和debug建立逐项矩阵，记录Zircon支持/降级/不支持及其证据，不只在计划正文散述。

### P2-8：用户文档没有解释成本和典型失败模式

最终文档需说明slice/grid预算、体积数量、纹理、phase、透明、shadow、history ghosting和dynamic-resolution取舍，并给出诊断入口；不得用营销式“volumetric enabled”代替约束。

## 7. 目标架构与重构范围

### 7.1 唯一authoring与render primitive合同

建立`GlobalVolumetricFogProfile`和独立`LocalFogVolumeComponent`。Local组件至少持有stable identity、shape、world-to-local、density/scattering/absorption/emissive、fade/priority/blend/layer、material/texture handle和generation。Post Process Volume只覆盖全局profile或相机栈，不再充当局部介质几何。

### 7.2 View-local VBuffer contract

由viewport extent、dynamic-resolution fraction、FOV/projection、stereo eye、fog distance和scalability计算`VBufferDesc`：extent、slice count、distribution、format、history/filter、budget和generation。logical resource由graph声明，物理资源由09A池化/别名/退役；allocation failure产生typed fallback。

### 7.3 GPU-local volume demand与voxelization

CPU只发布persistent volume records和dirty generation。09B visibility先做view cull，GPU按tile/cluster/brick生成candidate list或通过indirect draw/material voxelization写入media volume。列表有显式capacity、counter、overflow和deterministic priority；不能回退到无界全格全量扫描。

### 7.4 统一介质与光照输入

Media阶段输出可解释的extinction/scattering/emissive；Light阶段消费09E light/shadow/cookie和09F environment/indirect generation。local volume material复用09C artifact与09D residency。所有输入记录generation，mutation只失效受影响区域/history，而不是隐式使用陈旧或空资源。

### 7.5 统一temporal与final composition

09H提供pixel jitter、motion、camera cut、view/history generation和disocclusion。Volumetric维护per-froxel confidence/rejection并输出current integrated volume。Opaque/Sky/Transparent/SSS/Transmission使用同一apply contract；transparent按depth segment合成或采用经过误差测试的近似。

### 7.6 Editor与diagnostics是产品的一部分

Inspector、gizmo、material preview、quality/budget panel、slice/debug views、GPU timing、overflow/history/shadow/cookie route和artifact capture共用runtime typed diagnostics。所有编辑操作必须可撤销、可保存、可迁移、可在Play/remote runtime同步。

## 8. 需要修改或新建的owner

| Owner | 需要重构的内容 | 硬切除条件 |
|---|---|---|
| `core/framework/render/advanced_lighting/volumetric.rs` | 物理介质、VBuffer desc、Local Volume typed GPU record、budget/fallback/generation | 新schema与迁移通过后移除固定dimension和AABB-only DTO |
| `scene/world/render_post_process.rs` | 仅解析全局profile；Local Fog交给独立component/render primitive | legacy asset迁移完成后删除collider-to-AABB雾转换 |
| `advanced_lighting/froxel/media_inject` | world-origin aware global media、shape/material voxelization、GPU bins与overflow | GPU cull/voxelization通过大场景gate后删除全volume loop |
| `advanced_lighting/froxel/light_scatter` | 正式光度、参与mask、cookie/function、shadow、environment/indirect、temporal confidence | 新输入generation与数值oracle通过后删早退式participation |
| `advanced_lighting/froxel/integrate` | 保留Beer-Lambert，扩展filter/history miss/精度和多视图合同 | 新VBuffer格式稳定后移除固定RGBA16F假设 |
| history/render graph | view generation、pool/alias、persistent bindings、camera-cut/dynamic-res失效 | 09A/09H验收后删除executor局部分配与私有jitter |
| forward/deferred/OIT/sky/SSS/transmission | 统一apply与depth-segment透明组合 | 交叉视觉/数值gate通过后移除片元预雾化OIT路径 |
| volumetric fog plugins | 从capability registration升级为真实runtime/editor贡献 | builtin与plugin authority明确后删除重复/空注册 |
| Editor Post Process/scene tools | 全局profile、Local Fog Inspector/gizmo、material、debug、budget、transaction/save | 新authoring闭环通过后删除硬编码PPV演示反馈 |
| tests/artifacts | 分层测试、非静默GPU skip、当前源码视觉/性能/RenderDoc证据 | 新artifact manifest发布后归档失败旧证据，不覆盖历史 |

## 9. 分层实施里程碑

### M0：冻结完成度和证据真值

- 将Volumetric Fog状态标为prototype/path-integrated，登记P0-1失败artifact。
- 固定可重复GPU/driver/scene/camera/cvar/profile和artifact manifest。
- 重新运行ignored exporter前先让缺GPU成为明确skip/fail，不得静默pass。

退出条件：当前源码能稳定重现旧失败或产生可解释的新基线，所有artifact带source fingerprint与环境。

### M1：定义介质、VBuffer和组合架构

- 写出物理单位、Global/Local schema、VBuffer desc、generation、budget和fallback。
- 定义与09A/B/C/D/E/F/G2/H的owner边界及opaque/transparent/SSS顺序。
- 给legacy asset制定versioned migration和hard-cutover表。

退出条件：架构评审通过，禁止在旧AABB DTO增加长期字段。

### M2：建立独立Local Fog Volume产品合同

- 新增component/asset/stable identity、shape/transform/fade/priority/blend/layer。
- 实现Box/Sphere起步和negative/subtractive明确语义。
- scene I/O、clone/play capture、cook、dependency、migration、validation完整。

退出条件：rotated Box与Sphere在extract/GPU前后形状一致，legacy collider资产自动迁移或明确报错。

### M3：Viewport/scalability驱动VBuffer和资源池

- 按view extent、screen fraction、slice count和fog distance计算网格。
- 建立09A pool/alias/retirement、allocation failure和per-view/global budget。
- 覆盖resize、dynamic resolution、stereo、orthographic与quality切换。

退出条件：720p/1080p/4K/21:9的实际pixel coverage符合profile，资源无泄漏/错用旧generation。

### M4：GPU volume culling/binning与material voxelization

- 先frustum cull，再生成有界tile/cluster/brick candidate list或indirect voxelization。
- 接入Volume Material与density/albedo/emissive texture residency。
- 提供overflow、occupancy、candidate count和last-good diagnostics。

退出条件：100/1,000 volume压力场景不执行全格全量循环，overflow可见且画面确定性降级。

### M5：全局介质与大世界正确性

- 引入base height、camera-relative/world-origin generation、fog start/max distance。
- 扩展scattering/absorption/extinction/emissive和heterogeneous density。
- 建立极端near/far、origin rebase与FP16误差测试。

退出条件：负Y、洞穴、高空、原点迁移和超大坐标场景不出现密度跳变或NaN。

### M6：正式直接光、shadow与cookie/function

- 消费09E统一光度/area/layer/shadow与09G2 cookie generation。
- 构建只含参与灯的froxel candidate mask，移除CPU `Vec::contains`和shader早退浪费。
- 提供shadow/cookie route diagnostics。

退出条件：Directional/Point/Spot/Rect的数值与视觉场景通过，cookie和shadow移动无陈旧帧。

### M7：Environment、baked与Hybrid GI散射

- 接入Sky/IBL、baked irradiance/APV和Hybrid GI的versioned radiance。
- 定义一次/多次散射近似和能量组合，避免双计。
- 完成静态/动态/混合模式与fallback矩阵。

退出条件：室内外、昼夜、无Ambient Light场景仍有正确环境散射，切换模式无能量跳变。

### M8：统一temporal reconstruction

- 修正pixel-to-froxel jitter单位并接入09H jitter/camera cut/motion。
- 实现filterable history、confidence、depth/disocclusion、radiance clamp/moments和mutation generation。
- 受预算控制地处理history miss supersampling/filter。

退出条件：平移、旋转、快速灯光、移动volume、薄遮挡、dynamic resolution和camera cut无明显拖影/闪烁。

### M9：Opaque/Transparent/OIT/SSS/Transmission组合收口

- 建立统一apply binding与scene-linear/pre-exposure合同。
- 按depth segment处理多层透明，或证明近似误差和边界。
- 与09G2共同固定SSS/transmission/refraction排序。

退出条件：两层玻璃、粒子、折射、SSS角色和opaque/sky对照通过数值与视觉gate。

### M10：资源、pipeline与submission产品化

- 迁移per-dispatch buffer/bind group到persistent upload/cache。
- 接入09C PSO cache/warmup和09A真实async/queue/fence。
- 验证device loss、shader reload、allocation failure和多viewport退役。

退出条件：steady-state无不必要GPU对象创建，timeline证明预期重叠或准确标记graphics compute。

### M11：Editor authoring与debug闭环

- 实现Global profile、Local Fog Inspector/gizmo、material/texture与preview。
- 实现slice、occupancy、light count、history rejection、overflow、shadow/cookie和GPU timing视图。
- 接入transaction/undo/save/play/runtime/remote同步。

退出条件：作者无需手改asset即可创建、调试、保存和回放复杂体积场景。

### M12：分层自动化与artifact更新

- L0/L1数学与shader compile；L2 GPU readback；L3 renderer/scene/editor integration。
- L4固定视觉场景；L5性能、VRAM、soak、device-loss和跨backend。
- 所有skip为结构化结果，artifact绑定source/env/settings/scene/camera。

退出条件：旧失败artifact被保留为历史，新artifact在当前源码上通过且可独立复现。

### M13：竞争性画质/性能验收与硬切换

- 在同场景、同分辨率、相近画质预算下对照Unreal/HDRP，并记录Zircon GPU ms、VRAM、稳定性和误差。
- 覆盖室内窗光束、室外高度雾、100/1,000局部体积、动画纹理、透明粒子、stereo与4K。
- 新路径达到gate后删除AABB-only、fixed-grid、private jitter、mock Editor和旧OIT预雾化路径。

退出条件：没有双产品authority；性能和画质结论由可复现artifact支持，不使用功能存在性代替竞争性结果。

## 10. 验收矩阵

| 层级 | 必须覆盖 | 失败判定 |
|---|---|---|
| L0 数学 | HG phase、Beer-Lambert、slice distribution、pixel/froxel jitter换算、shape SDF、fade/blend、光度单位 | 非有限值、边界不连续、分辨率改变导致单位漂移 |
| L1 编译/布局 | WGSL compile、binding layout、struct alignment、permutation、material artifact、schema migration | source-string命中但shader未编译不算通过 |
| L2 GPU readback | media/scatter/integrate、history rejection、cookie/shadow、bin overflow、dynamic extent | 无adapter静默return；只测16x8x8不能关闭产品P0 |
| L3 集成 | scene save/load、Local Fog gizmo/transaction、light/volume mutation、resize、camera cut、多viewport | asset与GPU shape/generation不一致或旧history泄漏 |
| L4 视觉产品 | 房间窗光束、室外高度雾、洞穴carving、动画异质雾、透明多层、SSS/transmission、昼夜 | 只有非黑像素、资源存在或单张无对照PNG |
| L5 性能/稳定 | 720p/1080p/4K/21:9、100/1,000 volume、多灯、VRAM pressure、soak、device loss、跨backend | 无GPU ms/VRAM/candidate/overflow，或降级原因不可见 |

## 11. 最低产品gate

1. 旧窗光束场景的shaft区域必须比shadow control产生显著且方向正确的亮度/对比度，不得继续为0 brighter pixels与-0.000 contrast。
2. Rotated Box、Sphere和重叠/优先级/negative carve的GPU结果必须与authoring shape一致，不接受world AABB替代。
3. 720p、1080p、4K与21:9在同quality profile下保持可解释的screen pixel coverage和稳定性能，不固定160x90冒充自适应网格。
4. Camera cut、dynamic resolution、快速移动光源和volume不得使用不匹配history；jitter单位有跨分辨率数值测试。
5. 两层透明、粒子和折射不得重复加入camera-to-fragment in-scattering；Opaque/Transparent/SSS/Transmission顺序有数值oracle。
6. 100与1,000 Local Fog Volume场景必须有GPU candidate/overflow统计和确定性降级，不执行无界全格扫描。
7. Editor能够创建、编辑、撤销、保存、重载、Play预览和诊断Global/Local Volumetric Fog，不依赖硬编码演示反馈。
8. 产品CI lane缺GPU时明确失败或标记不计入acceptance；ignored exporter必须在发布gate中实际运行。
9. 每份accepted artifact必须包含source fingerprint、adapter/backend/driver、scene/camera、resolved profile、GPU ms、VRAM和关键counter。
10. “优于Unreal”只能由同场景同预算的画质、时域稳定、GPU时间和VRAM结果支持，不能由API数量或pass数量推断。

## 12. 实施前复核清单

- 重算本报告50文件focused fingerprint，并逐项重查18个modified文件。
- 确认09A/09B/09C/09D/09E/09F/09G2/09H的当前owner与接口是否已变化。
- 重新读取2026-07-11/16 artifact对应源码commit、GPU和capture脚本，不能假定当前源码等价。
- 为M0/M1建立canonical implementation plan；本文件保持review-and-refactor-plan，不直接充当执行日志。
- 任何新增代码先补失败测试或可重复artifact，再按support-first顺序实现底层合同。
- 新路径验收后执行hard cutover，禁止保留AABB-only/fixed-grid/mock Editor作为隐式第二产品路径。
