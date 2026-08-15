---
related_code:
  - zircon_plugins/hybrid_gi/plugin.toml
  - zircon_plugins/hybrid_gi/runtime/src/lib.rs
  - zircon_plugins/hybrid_gi/runtime/src/provider.rs
  - zircon_plugins/hybrid_gi/runtime/src/render_pass_executors
  - zircon_plugins/hybrid_gi/runtime/src/hybrid_gi
  - zircon_plugins/hybrid_gi/editor
  - zircon_plugins/hybrid_gi/dist
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/graphics/visibility/context/from_extract_with_history/construct.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/update_stats/hybrid_gi_stats.rs
  - zircon_editor/src/ui/retained_host/viewport/editor_viewport_render_defaults.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/pane_payload_builders/runtime_diagnostics.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_runtime/09e-direct-lighting-clustered-shadow-review.md
  - docs/plans/optimize/zircon_runtime/09f1-environment-sky-ibl-reflection-probe-review.md
  - docs/plans/optimize/zircon_runtime/09f2-baked-lighting-lightmap-irradiance-volume-review.md
  - docs/assets-and-rendering/hybrid-gi-lumen-scene-representation.md
  - docs/zircon_plugins/hybrid_gi/usage.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenScene.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenSceneData.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenMeshCards.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenSceneCardCapture.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenSurfaceCache.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenSurfaceCacheFeedback.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenMeshSDFCulling.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenScreenProbeGather.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenScreenProbeTracing.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenScreenProbeFiltering.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenRadianceCache.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenRadianceCacheHardwareRayTracing.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/Lumen/LumenHardwareRayTracingCommon.cpp
  - dev/UnrealEngine/Engine/Shaders/Private/Lumen/LumenScreenProbeTracing.usf
  - dev/UnrealEngine/Engine/Shaders/Private/Lumen/LumenRadianceCacheUpdate.usf
  - dev/UnrealEngine/Engine/Shaders/Private/Lumen/SurfaceCache/LumenSurfaceCacheSampling.ush
  - dev/godot/servers/rendering/renderer_rd/shaders/environment/sdfgi_preprocess.glsl
  - dev/godot/servers/rendering/renderer_rd/shaders/environment/sdfgi_integrate.glsl
  - dev/godot/servers/rendering/renderer_rd/shaders/environment/sdfgi_debug.glsl
  - dev/godot/editor/scene/3d/voxel_gi_editor_plugin.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Lighting/ProbeVolume/ProbeBrickPool.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Lighting/ProbeVolume/ProbeReferenceVolume.Streaming.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Lighting/ProbeVolume/ProbeReferenceVolume.Debug.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Lighting/ProbeVolume/ProbeVolumeLightingTab.cs
  - dev/bevy/crates/bevy_pbr/src/light_probe
  - dev/Fyrox/fyrox-impl/src/renderer
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 09F3 · Hybrid Global Illumination 工程化差距

## 1. 结论

Zircon当前Hybrid GI已经不是空目录。它有独立runtime/editor/dist plugin、四pass feature graph、typed quality/mode/profile/debug view、scene representation状态、Surface Cache page/residency雏形、voxel fallback、screen-probe/radiance-cache DTO、GPU completion/readback、软件追踪能力图，以及正在开发的Mesh SDF/Global SDF GPU build与trace route。按instance持久化的Global SDF和Radiance Cache GPU state、generation compare-and-commit、bounded in-flight readback、typed overflow/fallback diagnostics、当前帧RC consume先于probe trace执行，都是可以迁移的基础。

但现有产品画面不是工程级Hybrid GI。render graph中的最终GI只由固定8x8 tile grid产生：scene handoff用单个8x8 workgroup把整屏压成64条记录，trace schedule只容纳16个Surface Cache page、4个voxel clipmap和64个voxel cell，resolve再把64个tile铺回全分辨率。每个像素最终取得的是8x8 tile之一的低频颜色，而不是按屏幕深度/法线自适应放置、方向追踪、积分和重建的screen-probe gather。

名为Surface Cache的资源没有捕获真实surface。每个mesh只产生一张card，card中心是transform translation，半径是最大scale的一半；card material永远取CPU纹理`uv=[0.5,0.5]`，用手写PBR近似和CPU light loop计算一个RGBA8颜色，再把整张64x64 atlas/capture tile填成相同常量。depth tile同样不是几何深度，而是由card center/radius公式产生一个8-bit常量。atlas没有albedo/normal/emissive/opacity/depth等独立层、真实card view raster、mip residency或反馈驱动更新，因此不能表达几何边缘、材质细节、遮挡、薄片、背面、纹理变化或视差。

Radiance Cache也只是阶段命名完整。CPU先从上述单色Surface Cache或4x4x4 voxel clipmap取得一个RGB8 sample；GPU所谓trace阶段把同一个packed `radiance_confidence`写进每个4x4 probe tile的内部2x2，filter/border/mip对常量取平均，最多32个resident slot。`cs_consume`用8个slot的mip2值插值后写入当帧resident probe buffer，随后completion/trace可以消费；但最终cache、probe completion、irradiance、trace lighting、diagnostics、tile、indirect args和每个card slot sample仍全部回读CPU，用于下一帧runtime state/prepare DTO。它不是在GPU上追踪并保持方向radiance的Lumen式RC。

正在加入的Mesh/Global SDF比旧voxel proxy更接近真实功能，但仍未完成产品闭环。Mesh SDF artifact会被Global SDF build shader三线性采样，这是实质进展；然而Global SDF只有4层clipmap、64 cells/axis、全局最多128页、每页最多32 object candidate与每object 8 payload。trace只有16步，Global SDF field采用nearest cell sample，hit后不查询hit material或Surface Cache；没有probe lineage时直接制造蓝灰色。Hardware Ray Tracing只存在于enum/capability mask，生产dispatch明确设置false，没有TLAS/BLAS、RayQuery或ray pipeline。

更严重的是GI authority重复。插件graph生成`hybrid-gi-lighting`与history，core post-process又逐像素遍历最多16个CPU probe，并在每个probe内遍历trace region，投影圆形splat、重建第二份GI和第二套history，再与插件8x8 texture按luminance混合；core还独立加入baked ambient。这样同一帧存在plugin resolve、CPU-readback probe proxy、core post history和baked baseline多个owner，容易双计、延迟一帧、错误失效，也使性能复杂度达到`pixels * probes * trace_regions`。

Editor能力是登记而非工具。editor plugin注册了`plugins://hybrid_gi/editor/authoring.zui`，仓内没有该文件；测试只确认template path/command/view字符串已登记。没有Surface Cache、SDF、probe、RC、residency、overflow、history、backend route可视化，也没有质量/预算/参与规则的实际Inspector。core editor反而对所有viewport强制启用HGI，并用环境变量与硬编码32/64/16预算改写0值；这不是可保存、可撤销、可诊断的project/scene authoring。

现有视觉证据应降级为路径回归。抽查的scene-representation图是771x257的单三角形/简单体色斑矩阵，M4 profile图是646x242的两个方块，multi-direction图是257x128的8x4色块；代码扫描到34处`test_device()`调用，至少10处无adapter时`return`并静默通过。文档引用的2026-08-10五debug-view PNG当前不存在。旧PNG/RenderDoc仍可证明特定历史版本有pass和非黑像素，不能证明当前源码的大场景质量、时域稳定、GPU residency或优于Unreal。

本轮登记14项P0、23项P1、8项P2。重构不是继续给8x8 proxy添加字段，而是先确定唯一GI owner，建立persistent RenderScene与GPU resource lifetime，真实生成card/Surface Cache和SDF tracing，再完成adaptive screen probes、directional Radiance Cache、denoiser、baked integration、Editor和竞争性验收。旧core probe-splat/history与固定8x8 graph在新路径可验收时必须硬切除，不保留双产品路径。

## 2. 审查边界与覆盖

### 2.1 已读范围

| 子域 | 文件 / 物理行 | 本轮判定 |
|---|---:|---|
| Hybrid GI runtime production Rust/WGSL | 221 / 29,714 | E3：plugin/provider、state、prepare、GPU resources/readback、graph executors、Surface Cache、SDF、RC、trace/resolve shader |
| dedicated tests / fixtures / integration | 39 / 12,021 | E2：138个test属性；包含CPU oracle、source contract、WGPU readback和ignored PNG exporter |
| editor / dist / manifest | 9 / 317 | E3：capability、registration、native ABI与authoring template断链 |
| core/editor integration spot check | 26 / 2,109 | E3：settings、visibility stub、post-process双owner、stats与viewport default |
| HGI architecture / usage evidence docs | 2 / 3,254 | E2：历史进展与验收声明交叉核对 |
| production focused fingerprint | 221 / 29,714 | `86d113db0a5e67cf23f77b3747f15cd5674748bd8af87506738736d8b278aa5b` |

production fingerprint覆盖`zircon_plugins/hybrid_gi/runtime/src`下所有非tests/test_sources/test_support的`.rs`与`.wgsl`文件，并包含production文件内的95个inline test属性。当前HGI有大量modified与untracked拆分文件，包括Global SDF、Radiance Cache、trace shader、readback与runtime collector；该指纹只是本轮证据快照，实施前必须重取。

### 2.2 数据链读取深度

本轮从`RenderHybridGiExtract`与profile resolution开始，追到provider、feature graph、scene representation同步、prepare DTO、GPU buffer/texture创建、RC/trace/resolve dispatch、readback collect、runtime completion、core post-process和Editor diagnostics。shader不只检查binding声明，还核对了workgroup、dispatch、容量、循环、采样坐标、hit lighting、history metadata与最终像素组合。

对Mesh/Global SDF新增代码按候选实现审查，明确区分“当前工作区有实质进展”和“已成为accepted product baseline”。未跟踪代码可以说明设计方向，不能用于关闭既有产品差距。

### 2.3 与相邻审查的owner边界

- 09A拥有render graph version/resource truth、persistent GPU object、queue/fence、async compute、readback与device-loss生命周期。09F3定义GI资源和pass语义，不再私建逐帧buffer/texture/pipeline和graph外submission owner。
- 09B拥有persistent RenderScene、visibility、GPU Scene、bounds、view family与stable instance identity。09F3不能继续从frame extract clone和mesh translation重建第二份scene，也不能让visibility把HGI probe/update/feedback固定为空。
- 09C拥有material/shader/PSO generation。Surface Cache capture必须复用正式material evaluation与pipeline cache，不维护CPU手写PBR或每帧创建layout/pipeline/bind group。
- 09D拥有Mesh SDF derived artifact、texture/mesh residency、upload、eviction、budget和cook。09F3定义SDF/card/page/probe的consumer与专用residency feedback，不同步cook或全量上传asset。
- 09E拥有direct light、mobility、layer、shadow和光度真值。card lighting与GI injection必须消费同一light list/shadow result，不复制衰减、BRDF和static/dynamic规则。
- 09F1拥有sky/IBL/reflection。GI miss/environment与emissive injection消费其ready generation，不能用固定neutral ambient或hashed color伪造命中。
- 09F2拥有baked static indirect、Light Build generation与probe hierarchy。Hybrid模式必须按明确能量合同组合baked/static与dynamic delta，不能只看一个`baked_lighting_available`布尔值。

### 2.4 参考引擎边界

- Unreal Lumen是主要工程基线。其card capture渲染真实mesh draw/Nanite shading到Albedo、Normal、Emissive、Depth atlas；Surface Cache还有direct/final/indirect lighting、history、page/tile feedback与压缩。Screen Probe Gather默认按屏幕下采样放置并支持adaptive probe、octahedral directional traces、screen/HZB、Mesh/Global SDF和Hardware RT。Radiance Cache有clipmap、adaptive probes、trace-tile importance、occlusion/depth、filter、mip、scroll与GPU graph更新。Zircon当前同名类型不能视为等价实现。
- Godot SDFGI以最多8 cascades、3D SDF/light/aniso texture、probe history/scroll/average/ambient texture执行真实cascade integration，并提供debug shader与VoxelGI editor plugin。它不是最终性能上限，但已证明动态GI至少需要可视化、cascade state和方向/历史数据，而非单色card表。
- Unity Graphics的Adaptive Probe Volume不是Lumen替代品，但其brick pool、cell streaming、scenario、baking/placement/dilation/virtual offset/sky occlusion、debug/editor workflow可作为probe residency和工具下限。Zircon当前32-slot RC和缺失authoring panel低于该工程面。
- Bevy/Fyrox没有可直接照搬的Lumen等价实现，价值在于能力边界诚实、资源/scene ownership清楚。不得用它们较小的GI表面降低目标；它们只用于Rust ECS/resource integration和传统renderer fallback对照。

### 2.5 明确未做

本轮没有修改production code，没有运行Cargo、Editor、WGPU、RenderDoc或reference engine；没有执行运动场景、昼夜、skinned mesh、foliage、translucency、world partition、device loss、VRAM pressure、性能采样或同画质Unreal benchmark。PNG仅做静态视觉抽查。`source_recheck_required`为true，因为审查期间HGI源码仍在外部变化。

## 3. 可保留并迁移的基础

### 3.1 插件package与neutral renderer sideband方向正确

runtime/editor/dist分包、capability status、provider输出和neutral DTO减少了core对具体HGI实现的编译依赖。重构应让插件真正拥有资源、pass和authoring，不应把算法重新塞回core post-process。

### 3.2 typed quality/mode/profile/debug view可升级

serde枚举、fallback reason和resolved settings提供了稳定配置表面。需要补充明确的Auto/Disabled/Inherited语义、可扩展backend/scalability policy和migration；保留类型，不保留当前硬编码预算含义。

### 3.3 generation、participation与compare-and-commit值得保留

scene representation、RC和Global SDF已有generation/participation epoch、dirty page与stale completion保护雏形。目标系统应将其提升为GPU resource generation与immutable publish contract，而不是退回无代际的全清空。

### 3.4 Mesh SDF imported artifact与Global SDF真实采样是正向进展

当前Global SDF build shader会三线性采样validated Mesh SDF payload，而不是仅用AABB填格。candidate overflow、payload/upload限制和fallback reason也已有类型。应迁移到09D derived-artifact/residency与09A graph lifetime，不应删除重写成另一套临时格式。

### 3.5 capability graph和diagnostics可以成为正式route controller

Surface Cache/Global SDF/Voxel/Hardware RT的intersection backend枚举，以及intersection/lighting source/overflow统计，适合扩展为真实硬件能力、质量、预算和per-ray fallback controller。当前mask的声明不能继续冒充backend已实现。

### 3.6 bounded readback ring与last-good输出可保留为诊断通道

异步readback和in-flight上限比同步map好，但产品光照不得依赖整套readback往返。目标应只回读低频counter、sampled diagnostics和debug capture；GPU lighting、page table、probe atlas与history必须留在GPU。

### 3.7 现有小型CPU/WGPU fixtures可转成oracle层

trilinear weight、generation rejection、buffer packing、route selection和小shader readback仍适合unit/integration。它们应明确标成L1/L2，不再充当L4产品完成证据。

## 4. P0 差距清单

### P0-1：最终产品GI只有固定8x8 tile proxy

`scene_depth_handoff.wgsl`固定8x8 tile grid并以一个workgroup处理整屏；`trace_schedule_handoff.wgsl`固定576 words，`resolve_trace_depth_source.wgsl`把每个full-resolution pixel映射回64个trace tile。必须以viewport分辨率、质量和GPU预算生成screen-probe lattice/adaptive probes和directional ray samples，禁止固定`dispatch=[1,1,1]`成为产品路径。

### P0-2：Surface Cache没有捕获surface

card capture只取中心UV、一个CPU算出的RGBA8，并把64x64 tile全部填同色；depth是bounds公式常量。必须建立真实card view raster/material capture，至少分离albedo、normal、emissive、opacity/depth与lighting layers，支持mip、gutter、compression、page table、resident/dirty state和last-good publish。

### P0-3：scene representation退化为one mesh/one card/one probe

每个mesh只有一张card和一个位于mesh origin/bounding sphere的“screen probe”；没有多方向card generation、coverage selection、view-dependent probe placement、screen depth/normal classification或adaptive density。必须由persistent RenderScene生成per-primitive card set，并由当前view的GBuffer生成screen probes，两者不能复用同一DTO冒充。

### P0-4：visibility到HGI的产品桥被硬编码为空

`from_extract_with_history/construct.rs`把active probes、update plan、feedback和requested probes全部设为empty/default。插件scene representation随后旁路visibility自建状态，造成两套residency/update authority。必须由09B visibility/GPU Scene产生可见card/probe demand、occlusion与feedback，删除legacy空桥和旁路owner。

### P0-5：plugin graph与core post-process同时拥有GI和history

插件生成8x8 lighting/history，core post又从CPU probe/trace region重建splat和另一套history，再按luminance混合并加入baked ambient。必须选择唯一composite owner：core只消费一张versioned HGI diffuse/specular/AO output或插件直接在正式lighting stage合成；删除probe-splat、重复history和命名为history却携带current-frame texture的资源。

### P0-6：产品数据每帧GPU回读、CPU重打包、再上传

prepare路径回读cache、completed probes/traces、irradiance、trace lighting、diagnostics、RC counters、每个atlas/capture/depth slot、trace tiles和indirect args。backpressure会停止新工作。必须让page table、probe atlas、trace result、denoiser history和completion queue保持GPU resident，只回读有界统计/调试抽样；CPU不能成为GI frame-to-frame truth中转站。

### P0-7：Radiance Cache“trace”只缓存CPU RGB8常量

CPU从单色Surface Cache或4x4x4 voxel挑一个sample，GPU向4x4 tile内部2x2写相同packed值，最多32 slots。必须在GPU生成directional radiance/depth/visibility，按world/view clipmap mark/allocate/trace/filter/mip/scroll，支持足够probe数量、格式动态范围和budgeted incremental update。

### P0-8：Global SDF命中没有可信surface lighting

Global SDF hit只在存在probe lineage时复用旧probe颜色，否则制造蓝灰色；field nearest采样且只有16步。必须能从hit position/normal/material identity查询Surface Cache或fallback material/emissive/environment，计算可解释的radiance；没有lighting data应返回typed miss/low confidence，禁止伪造颜色。

### P0-9：Hardware Ray Tracing是公开假表面

enum与capability mask包含Hardware RT，但production dispatch把`prefer_hardware_ray_tracing`和capability都设为false，没有TLAS/BLAS、instance/material mapping、RayQuery/ray pipeline、compaction/update、fallback或validation。实现前必须报告Unavailable/NotImplemented并从可选route移除；实现后按设备能力与场景类型真实选择。

### P0-10：固定小容量和silent truncation不能承载产品场景

graph packet只容纳16 pages/4 clipmaps/64 cells，RC 32 slots，core post最多16 probes；Global SDF全局128 pages、32 candidates/page、8 payloads/object。虽然部分新路径有统计，最终画面仍可在容量外降成低频fallback。必须有可配置budget、需求优先级、page/trace feedback、稳定退化和overload UI，并用城市/室内/植被场景验证，而非提高几个常量。

### P0-11：GI失效不是geometry/material/light的精确依赖图

全部light变化可使所有page dirty，card/voxel/RC依赖多次clone snapshot和revision比较；deforming/skinned、material texture residency、emissive、LOD、Nanite/VG、foliage、heightfield和world-partition cell没有完整失效语义。必须建立primitive/card/page/SDF/probe dependency与dirty-region传播，stale generation保持last-good，局部变化只更新受影响资源。

### P0-12：BakedStaticDynamic建立在不完整baked contract上且可能双计能量

profile只检查baked availability、过滤部分static lights，再与core baked ambient/动态GI多个路径组合。09F2已经证明baked producer/generation/static-light语义不完整。必须定义baked static indirect与dynamic delta、emissive/sky/direct contribution、shadowmask和history identity，缺失或stale artifact时明确降级，禁止同一能量由baked、plugin和core重复加入。

### P0-13：Editor authoring、诊断与debug view没有产品实现

注册的`authoring.zui`不存在，Editor插件没有实际控件或viewport overlay；普通用户不能检查card coverage、Surface Cache page、Mesh/Global SDF、probe、RC、trace backend、overflow或history rejection。必须建立可保存/撤销的project/scene settings、per-object participation和真实debug workspace，所有数值来自runtime snapshot而非静态标签。

### P0-14：验收和文档允许proxy proof冒充产品完成度

大量PNG exporter默认ignored，10个WGPU调用可静默跳过，无同场景reference、动态序列、规模、VRAM或性能门；进展文档接近3,000行changelog并混用历史证据与当前能力。必须建立test taxonomy与current-source evidence manifest，missing adapter是skip/fail而不是pass，产品门要求可重放场景、图像序列、counter、capture和同预算reference对比。

## 5. P1 差距清单

### P1-1：CPU scene owner每帧clone/sort/比较大量Vec和BTree集合

settings、cards、lights、page contents、voxel cells、probe demands和snapshot多次复制；稳定帧仍有heap和排序工作。迁移到persistent indexed state、dirty ranges与arena/SoA，稳定帧应接近零CPU scene rebuild。

### P1-2：runtime collector在全局Mutex内完成投影、同步和GPU encoding

锁域覆盖prepared frame clone、Mesh SDF projection、Global SDF residency和prepare dispatch，多个viewport/instance相互串行。拆分immutable input snapshot、per-instance state和render-thread command recording，锁只保护短生命周期registry mutation。

### P1-3：旧voxel fallback只有每层4x4x4 cells且以scene bounds为中心

cell只保存dominant card和RGB8，CPU按bounds sphere/AABB填充并手写光照。目标fallback应为camera-relative clipmap、可扩展分辨率/砖块、occupancy/material/radiance层与GPU update；在Global SDF未ready时也要有空间连续性和明确质量级别。

### P1-4：Radiance/Surface Cache使用RGBA8和8-bit depth损失动态范围

HDR emissive、强光、曝光变化、低亮间接光和confidence被压入8-bit并经手写tonemap。正式capture/radiance使用可证明的HDR格式、独立confidence/moment/depth，并按平台压缩；debug/readback再量化。

### P1-5：CPU card shading复制了一套不一致的PBR与light attenuation

它与09C material shader、09E direct-light/shadow、09F1 environment不共享BRDF或资源ready语义，lookup失败还返回hashed debug RGBA。删除production CPU shading；missing material输出typed invalid page和可视化，不得污染正常画面。

### P1-6：card bounds与orientation不足

center取translation、radius取scale，normal取transform forward，忽略真实mesh bounds、section、negative/nonuniform scale、deformation和遮挡方向。card builder应从geometry/cluster/LOD生成OBB与多方向coverage，并与stable primitive identity绑定。

### P1-7：Surface Cache没有真实feedback与page replacement policy

当前一个card一个page、按小预算分slot，没有view coverage、ray hit feedback、mip demand、age/cost、update priority和thrash控制。应建立GPU feedback compact、CPU/GPU budget controller或GPU-driven residency，并暴露miss/eviction/dirty latency。

### P1-8：screen probe没有depth discontinuity与adaptive placement

probe来自card列表而不是屏幕tile/GBuffer，ray budget只是总budget除probe数。需要uniform grid、depth/normal discontinuity补点、thin geometry/edge处理、importance sampling、wave/coherent tracing与per-probe confidence。

### P1-9：trace route缺少screen trace与层级fallback闭环

产品应按screen/HZB、Mesh SDF、Global SDF、Hardware RT、voxel/environment的可配置顺序执行，并记录每ray hit source。当前8x8 graph与新增GDF prepare路径没有统一route/resource owner。

### P1-10：Global SDF field采样和normal重建质量不足

trace用nearest cell、固定step和max distance，没有gradient normal、surface bias、thin feature、heightfield或translucent classification。至少需要trilinear field sample、robust gradient、conservative distance、clipmap blend、object SDF near-field与leak tests。

### P1-11：Global SDF build仍受readback completion驱动commit

page build完成要回读请求再由CPU commit，readback backpressure可冻结build。应使用GPU generation/page-table state与indirect work queue，CPU只异步观察统计；失败/overflow在GPU page metadata中可见。

### P1-12：没有deforming geometry、foliage、heightfield与translucency策略

缺少skinned SDF/card update、World Position Offset、masked/two-sided foliage、landscape/heightfield、water/translucency和hair分类。必须定义每类geometry的representation、update cost、fallback和unsupported diagnostic。

### P1-13：emissive propagation与multi-bounce语义不完整

单个CPU capture颜色和lineage复用不能处理小型强emissive、远场聚合、radiosity history或能量守恒。应有emissive injection、Surface Cache lighting/radiosity或RC feedback的明确路径，以及firefly/clamp策略。

### P1-14：Radiance Cache缺少occlusion/depth/relocation与方向重要性

32个4x4 RGBA8 tile没有visibility moments、probe offset、near geometry rejection、BRDF/ray direction importance、trace tile LOD或neighbor filter。目标RC需要这些数据并能被screen probe/translucency等多个consumer共享。

### P1-15：temporal/spatial filter只在8x8 tile metadata上做启发式

history只记录depth、6-bit oct normal+source、10-bit support signature与confidence，阈值硬编码；运动向量、disocclusion、reactive material、variance、firefly和camera cut处理不完整。09H应拥有统一temporal contract，HGI提供radiance/moments/confidence而非第二套私有history。

### P1-16：graph executor在执行时创建pipeline layout/pipeline/bind group/uniform buffer

scene、trace、resolve handoff的pipeline资源没有进入09C cache，RC每stage也创建bind group。必须按shader/layout/device generation缓存，per-frame只分配动态参数或使用ring/arena。

### P1-17：每帧重新创建大量buffer/texture并用heap Vec清零

cache、probe、trace、irradiance、diagnostics、atlas/capture/depth等都在prepare创建。使用persistent capacity-managed pools、transient graph allocator和suballocation；统计creation/upload/peak/retired bytes并设验收阈值。

### P1-18：AsyncCompute声明没有真实并行/依赖收益证据

fixed tiny dispatch、graph外prepare、readback与串行collector使queue标签不等于重叠执行。需用09A timestamp/queue/cross-queue dependency证明card/SDF/trace/filter与graphics的overlap，未证明前按普通compute调度。

### P1-19：profile名字与实际算法/预算不匹配

FullyDynamic/IndoorStatic/OpenWorld/Cinematic主要映射quality和三个计数预算，不能表达resolution、trace distance、backend、update fraction、SDF/page/RC/history成本。建立scalability policy asset和设备/帧预算控制，profile必须可解释且有benchmark目标。

### P1-20：Editor viewport默认强制启用实验性HGI

editor defaults无条件`with_hybrid_global_illumination(true)`并插入enabled settings，可能把不可用provider、额外readback和伪画面变成默认体验。实验期应显式opt-in或按project setting/provider readiness启用，并展示fallback/成本。

### P1-21：diagnostics数量多但缺少闭环操作

runtime stats已有Global SDF CPU timing、page/overflow/upload/resource计数，Editor只显示少量probe/settings文本。需要按view/instance展示route、page heatmap、dirty latency、RC occupancy、history rejection、VRAM、readback/backpressure，并能从异常跳到对象/设置。

### P1-22：源码树混入两种shader cache目录

插件runtime下同时出现`.zircon/cache`和`.zircon-cache`生成物，容易污染审查、打包与变更。统一cache root、gitignore/cook规则和清理验证；source package不得依赖工作区残留cache。

### P1-23：文档是历史日志而非稳定能力合同

`hybrid-gi-lumen-scene-representation.md`累计多轮实现日志、旧命令、旧数字和阶段声明，`usage.md`同时写已通过与开放总门。拆为architecture、data contract、scalability、debug/operations、known limitations和current evidence manifest，历史记录移入plan/evidence archive。

## 6. P2 差距清单

### P2-1：命名使proxy获得了错误完成度

`CardCapture`、`SurfaceCache`、`RadianceCacheTrace`、`ScreenProbe`和`HardwareRayTracing`名称目前高于实现。迁移期间为proxy加`Prototype`/`CpuSeed`/`NotImplemented`诊断，正式名称只由产品owner持有。

### P2-2：超大owner文件降低审查和演进可靠性

representation、voxel state、RC、trace/resolve executor与测试文件多处500-800行。按scene representation、residency、capture、trace backend、filter、readback diagnostics拆分，并保持public boundary最小。

### P2-3：大量source-string contract test易锁定实现文本

`.contains()`/`include_str!`适合少量结构守卫，不能证明shader binding、dispatch和pixel。迁移为Naga/reflection ABI test、CPU oracle、WGPU behavior和product sequence，结构测试只保留关键禁止依赖。

### P2-4：visual exporter文件名携带日期而无source identity

PNG/RDC名称不能证明对应当前commit、shader、settings或adapter。生成sidecar manifest：source fingerprint、binary hash、adapter/driver/backend、scene hash、settings、capture tool、pixel metrics和accepted baseline。

### P2-5：配置schema缺少版本化migration和unknown-field策略

新增profile/mode可serde roundtrip，但project setting升级、removed enum、platform override和unsupported backend的迁移未定义。建立schema version、migration diagnostic与last-good load。

### P2-6：跨平台manifest声明缺少真实backend矩阵证据

plugin声明Windows/Linux/macOS，当前关键证据集中在WGPU/DX12/RenderDoc。至少建立D3D12/Vulkan/Metal feature/correctness matrix，并对无SDF/HRT能力的平台给出明确scalability path。

### P2-7：缺少可复现benchmark工具和参考场景包

需要室内、小型emissive、开放世界、植被、动态几何、多视口、camera cut、昼夜、溢出与VRAM压力场景；输出GPU时间、CPU prepare、VRAM、upload/readback、page/trace hit和质量误差。

### P2-8：没有公开限制清单和shipping gate

experimental/partial状态是正确的，但用户文档仍需列出不支持geometry/material/platform、预算上限、fallback和debug方法。shipping profile在P0门关闭前不得默认依赖HGI。

## 7. 目标架构

### 7.1 单一owner与数据流

```text
World / Asset / Material / Light truth
  -> persistent RenderScene + stable primitive/material generations       [09B/09C/09D/09E]
  -> HGI scene representations
       card set + Surface Cache page table / layered atlases
       Mesh SDF residency + Global SDF clipmaps
       optional hardware RT scene
  -> per-view screen probe placement + trace scheduling
  -> screen/HZB -> Mesh SDF -> Global SDF -> HWRT -> environment fallback
  -> directional probe radiance + Radiance Cache
  -> temporal/spatial reconstruction                                  [09H]
  -> one HGI output generation
  -> one lighting composite owner
```

CPU只提交scene deltas、settings和低频budget/readback diagnostics。card page、SDF page、probe atlas、trace queues、RC、history与denoised output保持GPU resident。所有pass进入09A graph并使用09C pipeline cache；resource generation在device loss/hot reload时原子替换。

### 7.2 Scene representation

- `HgiPrimitiveId`绑定stable RenderScene primitive、mesh/material/transform/deformation generation。
- imported/offline card builder为每个mesh/section/LOD生成多方向card OBB、coverage和material slot；runtime只做可见性、page demand和必要recapture。
- Mesh SDF是09D derived artifact，包含schema、mesh revision、scale/deform limitations和streaming chunks。
- Global SDF是camera-relative clipmap/brick/page table，dirty region由primitive变化传播，build/commit保持GPU generation。
- unsupported geometry必须有typed representation/fallback，不得用hashed color或bounds sphere静默代替。

### 7.3 Surface Cache

- card capture复用正式material shader/vertex factory/geometry path；支持masked/two-sided和明确定义的WPO/skinned fallback。
- layered physical atlases至少保存albedo、normal、emissive、opacity/depth、direct/final lighting或等价压缩合同。
- virtual page table、mips、gutter、feedback、LRU/cost priority、dirty recapture、last-good generation与VRAM budget统一管理。
- lighting update消费09E light/shadow与09F1 environment，不实现第二套CPU BRDF。

### 7.4 Trace与Radiance Cache

- per-view GBuffer产生uniform+adaptive screen probes，方向域使用octahedral tile或等价结构。
- trace route按每ray可用性执行screen/HZB、Mesh/Global SDF、HWRT和environment；hit返回position/normal/material/radiance/confidence/source。
- RC使用world clipmap、directional radiance/depth/visibility、importance-driven trace tile、scroll/reprojection/filter/mip，容量由scalability与VRAM控制。
- indirect dispatch、compaction、sorting和update fraction在GPU生成；CPU不枚举所有probe/tile。

### 7.5 Reconstruction与组合

- HGI输出radiance、moments/confidence/source和必要的bent-normal/AO，不复用模糊的history texture名称。
- 09H统一motion vector、depth/normal disocclusion、camera cut、reactive mask、variance和history lifetime。
- 只有一个owner把dynamic indirect与09F2 baked static indirect、09F1 environment和09E direct lighting组合；能量合同可被debug view逐项显示。

### 7.6 Editor与运营面

- Project/scene settings：enable、quality/scalability、backend policy、budget、trace distance、baked integration、per-object participation。
- Debug workspace：card coverage、Surface Cache layers/page residency、Mesh/Global SDF slice、screen probes/rays、RC、backend source、history rejection和overflow。
- Diagnostics：CPU/GPU time、VRAM、transient creation、upload/readback、page/probe/trace hit、dirty latency、fallback、device capability和last-good generation。
- 所有控件通过同一command/transaction/state snapshot，支持undo/save/reload；缺失template或provider直接显示错误。

## 8. 硬切换原则

1. 新HGI product owner可输出可验证画面前，旧路径保持`experimental`且显式命名proxy，不继续扩张公共依赖。
2. 新Surface Cache完成后删除CPU center-UV shading、uniform tile/depth和hashed color fallback，不保留compatibility switch。
3. 新screen-probe/trace/RC完成后删除固定8x8 scene/trace/resolve packet以及32-slot constant RC，不把它们留作低质量档。
4. 新composite接入后删除core post-process probe/trace-region nested splat、第二份history和旧payload source；debug fixture通过专用test provider注入。
5. Hardware RT未实现时从admitted capability移除；实现时不保留只改mask不执行ray query的路径。
6. 真实Editor面板落地时删除缺失`authoring.zui`登记和环境变量私有override，配置迁移到versioned project setting。
7. 历史PNG/RDC移入evidence archive；current acceptance只读取带source manifest的新证据。

## 9. 依赖顺序与里程碑

### M0：冻结能力声明与证据分级

- plugin继续`experimental/partial`；docs列出8x8、CPU card、无HRT、容量和readback限制。
- 定义L0 source guard、L1 CPU oracle、L2 GPU unit、L3 renderer integration、L4 product sequence、L5 competitive benchmark。
- GPU unavailable时测试报告skip/fail reason，CI required lane不得静默pass。

### M1：建立HGI scene/setting/identity合同

- versioned project/scene settings、per-object participation、stable primitive/material/deformation generation。
- 删除Editor hardcoded默认覆盖或改为显式project default。
- visibility输出真实card/probe demand与feedback owner。

### M2：GPU lifetime与单一feature owner

- 把persistent HGI resource set、device generation、graph pass/resource和pipeline cache接入09A/09C。
- 消除每帧pipeline/atlas/buffer创建与全局collector长锁。
- 只保留bounded diagnostic readback。

### M3：离线/导入card builder

- 多方向card OBB、coverage、section/material/LOD identity、validation与derived artifact。
- 支持reimport/cook/version migration和unsupported geometry diagnostics。

### M4：真实Surface Cache capture

- raster真实geometry/material到layered HDR/depth atlas。
- page table、mip/gutter、resident/dirty/last-good generation。
- 使用正式light/shadow/environment，不再CPU手写shading。

### M5：Surface Cache feedback与增量更新

- GPU hit/coverage feedback、page priority/eviction、dirty recapture和budget controller。
- 大场景稳定帧CPU/上传接近零；局部材质/灯光/transform变化只更新受影响page。

### M6：Mesh SDF与Global SDF产品化

- 把当前artifact/build候选迁入正式residency和graph lifetime。
- trilinear field、gradient normal、clipmap blend、dirty page、GPU commit和overflow recovery。
- 验证thin geometry、adjacent contributor、scaled mesh、streaming和stale completion。

### M7：软件追踪route

- screen/HZB、Mesh SDF、Global SDF、voxel/environment按ray fallback。
- hit material/Surface Cache lighting与typed confidence/source。
- indirect dispatch、compaction/sort和quality-dependent ray budget。

### M8：Hardware Ray Tracing

- BLAS/TLAS build/update/compaction、instance/material mapping、RayQuery或ray pipeline。
- device capability、memory/update budget、software fallback、validation和capture。
- 没有硬件的shipping profile仍可运行完整software path。

### M9：Adaptive Screen Probe Gather

- GBuffer-driven uniform/adaptive placement、depth/normal edge、octahedral directions和importance sampling。
- full viewport/half resolution/scalability策略，不再固定64 probes。

### M10：Directional Radiance Cache

- world clipmap、probe mark/allocate/scroll、directional radiance/depth/visibility、trace tile、filter/mip。
- GPU resident、multi-view policy、capacity/VRAM/fallback诊断。

### M11：Temporal/spatial reconstruction

- 接入09H motion/depth/normal/reactive/camera-cut history contract。
- variance/confidence/source-aware filter、disocclusion、thin geometry和fast-motion sequence。
- 删除旧8x8 resolve/history。

### M12：Baked/dynamic能量整合

- 消费09F2可信generation，定义static indirect、dynamic delta、emissive/sky/direct和shadowmask。
- DynamicOnly/BakedStaticDynamic在同场景有逐分量debug与像素oracle。
- 删除core/plugin/baked重复加光。

### M13：Editor/debug/operations

- 实际authoring template、Inspector、viewport overlays、runtime diagnostics与对象定位。
- provider missing、overflow、stale generation、device loss、unsupported geometry可操作。

### M14：性能、内存和稳态验收

- scene sizes、GPU/CPU frame、VRAM、upload/readback、page/RC residency、shader compilation和async overlap基准。
- camera traverse、昼夜、动态几何、多viewport、device loss、OOM和24h soak。
- stable frame禁止持续大Vec clone、全buffer重建和整GI readback。

### M15：产品与竞争性验收后硬切换

- 室内、开放世界、植被、强emissive、移动遮挡、薄几何和baked hybrid场景通过L4 sequence。
- 相同硬件、分辨率、场景、质量目标和内存预算与当前Unreal Lumen对比画质/性能；记录不能等价的feature差异。
- 删除所有旧proxy/core splat/重复history，更新manifest maturity只在门关闭后进行。

## 10. 验收矩阵

| 维度 | 最低验收 | 不可接受替代 |
|---|---|---|
| Surface Cache | 真实几何/纹理/法线/emissive/depth capture；局部dirty page变化可见 | uniform RGBA tile、center UV、bounds depth |
| Screen probes | probe数随分辨率/质量变化，depth/normal edge有adaptive probes | one mesh/one probe、固定8x8 |
| Software trace | screen/Mesh SDF/Global SDF hit source与material radiance可读 | hit后固定蓝灰色或lineage旧颜色 |
| Hardware RT | capture中有真实AS build/update与ray dispatch/query | enum、capability bit或shader字符串 |
| Radiance Cache | directional HDR/depth/visibility、scroll、filter、GPU resident | 32个RGBA8常量tile |
| History | camera cut、disocclusion、fast motion、moving emissive序列稳定 | 两张静态PNG有差异 |
| Residency | 大场景page/probe/SDF预算、eviction、last-good和overflow可观测 | 提高固定数组常量 |
| Editor | settings保存/撤销、debug overlay、异常定位、provider fallback | capability/view字符串注册 |
| Performance | GPU/CPU/VRAM/upload/readback稳定帧与峰值门 | pass存在、非黑像素、单帧CPU计时 |
| Product | 当前源码场景包、manifest、image sequence、capture、reference compare | 历史日期PNG/RDC或ignored exporter |

## 11. 参考源码映射

| Zircon目标 | 主要参考 | 应学习的边界 |
|---|---|---|
| Card generation/capture | Unreal `LumenMeshCards.cpp`, `LumenSceneCardCapture.cpp` | 多card coverage、真实draw/material capture、card view |
| Surface Cache | Unreal `LumenSurfaceCache.cpp`, feedback/shaders | layered atlas、page/tile feedback、lighting/history、compression |
| Screen Probe Gather | Unreal gather/tracing/filtering C++与USF | adaptive placement、octahedral direction、screen/SDF/HRT、filter |
| Radiance Cache | Unreal RC C++/USF/HRT | clipmap、trace tile、importance、depth/occlusion、scroll、GPU update |
| Software SDF GI | Unreal Mesh SDF/Global SDF；Godot SDFGI | distance field hierarchy、cascade/clipmap、directional integration/history |
| Hardware RT | Unreal common/screen-probe/RC HRT | AS lifecycle、material mapping、software fallback、budget |
| Probe residency/editor | Unity APV runtime/editor/debug | brick/cell streaming、scenario、debug、bake/placement operations |
| Rust ownership | Bevy/Fyrox render/resource integration | honest capability boundary、ECS extraction、resource lifetime；非质量上限 |

## 12. 完成定义

09F3文档完成只表示current-source静态审查和重构路线已经登记，不表示Hybrid GI实现完成。实现完成至少要求M0-M15全部关闭、旧proxy硬切除、current-source证据manifest可复现、跨平台required lane不静默skip、Editor真实可操作，并在同等条件下完成与Unreal的质量/性能对照。

在这些条件之前，允许的产品描述是“experimental Hybrid GI prototype with partial Mesh/Global SDF and GPU state work”；不允许描述为“Lumen-style product GI已完成”，更不允许声称性能或表现优于当前Unreal。
