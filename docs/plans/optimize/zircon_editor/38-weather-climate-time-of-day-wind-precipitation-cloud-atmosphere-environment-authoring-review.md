---
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_weather_editor_workspace.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/world_building/volume_and_weather.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/world_building.rs
  - zircon_runtime/src/asset/assets/scene
  - zircon_runtime/src/scene/components/scene
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/core/framework/time
  - zircon_runtime/src/core/runtime/time.rs
  - zircon_runtime/src/core/framework/render/environment
  - zircon_runtime/src/core/framework/render/advanced_lighting/volumetric.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment
  - zircon_plugins/particles/runtime/src
  - zircon_plugins/particles/editor/src
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
  - zircon_plugins/terrain/runtime/src/plugin.rs
  - zircon_plugins/sound/runtime/src/components.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
  - docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/08b-audio-runtime-review.md
  - docs/plans/optimize/zircon_runtime/09e-direct-lighting-clustered-shadow-review.md
  - docs/plans/optimize/zircon_runtime/09f1-environment-sky-ibl-reflection-probe-review.md
  - docs/plans/optimize/zircon_runtime/09g1-volumetric-fog-froxel-review.md
  - docs/plans/optimize/zircon_runtime/09h2-exposure-color-bloom-dof-motion-blur-ssr-terminal-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
  - docs/plans/optimize/zircon_editor/15-material-shader-graph-instance-vfx-particle-preview-compiler-diagnostics-authoring-review.md
  - docs/plans/optimize/zircon_editor/16-terrain-landscape-foliage-scatter-world-partition-level-streaming-authoring-review.md
  - docs/plans/optimize/zircon_editor/17-sound-audio-clip-mixer-routing-effect-spatial-acoustic-timeline-authoring-review.md
  - docs/plans/optimize/zircon_editor/22-render-pipeline-frame-capture-lighting-bake-reflection-probe-post-process-debug-authoring-review.md
  - docs/plans/optimize/zircon_editor/28-spawn-rules-encounter-population-world-state-scenario-quest-flag-authority-simulation-authoring-review.md
  - docs/plans/optimize/zircon_editor/37-volume-zone-trigger-region-gameplay-audio-post-process-environment-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/SkyAtmosphereComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/VolumetricCloudComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/DirectionalLightComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/ExponentialHeightFogComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/WindDirectionalSourceComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/WindDirectionalSource.h
  - dev/godot/scene/resources/environment.h
  - dev/godot/scene/resources/sky.h
  - dev/godot/scene/3d/world_environment.h
  - dev/godot/scene/3d/fog_volume.h
  - dev/bevy/crates/bevy_light/src/atmosphere.rs
  - dev/bevy/crates/bevy_pbr/src/atmosphere
  - dev/bevy/crates/bevy_pbr/src/fog.rs
  - dev/bevy/crates/bevy_light/src/volumetric.rs
  - dev/Fyrox/fyrox-impl/src/scene/skybox.rs
  - dev/Fyrox/fyrox-impl/src/scene/mod.rs
  - dev/Fyrox/fyrox-impl/src/renderer/light.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Sky/VisualEnvironment.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Sky/PhysicallyBasedSky/PhysicallyBasedSky.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Sky/CloudSystem/CloudLayer/CloudLayer.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/VolumetricClouds/VolumetricClouds.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Editor/Lighting/VolumetricClouds/VolumetricCloudsEditor.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 38 · Weather / Climate / Time-of-Day / Wind / Precipitation / Cloud / Atmosphere / Environment Authoring 工程化差距

## 1. 结论

Zircon当前不存在可供项目使用的Weather或Climate系统。产品层只有一张静态Weather Editor Workbench：它固定显示`Weather_Storm`、`Region_Mountains`、`Layer_Clouds`，固定列出Cloud Build、Rain Burst、Wind Gust和Lightning时间段，固定报告8 layers、5 regions、2 warnings。Preview与Build只返回`queued`文本，Preset、Region和Blend Time提交只经过模板route。仓内没有Weather/Climate插件目录、第一方catalog registration、源资产、Scene component、compiler、artifact、runtime manager或安装回执。因此这张页面不是未完成的天气编辑器，而是第二套假authority。

现有底层也不是全空。Runtime有real/virtual/fixed clock、可暂停和缩放的游戏时间、Scene DirectionalLight、typed Post Process/Volumetric Fog、程序化天空参数、Source Cubemap/PMREM/SH9、双缓冲且generation-aware的realtime IBL时间片、真实CPU/GPU Sprite Particle以及Sound、Terrain和共享Region的局部基础。这些都应保留并通过typed adapter接入Weather，不能为赶功能再复制一套Light、Fog、Particle、Audio或Region系统。

但这些基础尚未形成Environment产品链。Scene schema没有Sky、Atmosphere、Cloud、Weather、Wind、Precipitation或TimeOfDay字段；`World::build_environment_extract()`只读取viewport的`preview_skybox`布尔值，返回固定默认gradient或disabled。`ProceduralSkyParams`的sun direction/intensity与Scene DirectionalLight互不关联，默认gradient的sun intensity还是0。Scene方向光本身只有direction/color/intensity/volumetric，没有atmosphere sun identity、太阳角半径、色温、云阴影或天体时钟绑定。

所谓Cloud路径尤其需要纠正。Realtime IBL scheduler确实声明`CaptureSky`和`CaptureCloud`两个operation，也有双slot、generation、retry/stale和分帧publish语义；但WGPU recorder把两者都调用到同一个`record_capture()`，使用同一套三色gradient加sun disk shader，并写入同一个source cubemap mip。`CaptureCloud`没有云密度、材质、天气图、光照、transmittance或composite输入，本质是重复覆盖和重复GPU工作，不能作为体积云实现或Weather集成点对外宣称。

Particle同样只能算可复用执行器，不是降水系统。它有shape、lifetime、velocity、gravity、drag、CPU/GPU backend、世界/局部空间和可选physics external force；但没有wind field、turbulence、weather snapshot、rain/snow/hail语义、camera-relative precipitation volume、surface impact、splash/decal/puddle或accumulation。CPU路径在physics capability存在时会消费资产内固定`external_force`，GPU frame layout只编码gravity与drag，连该external force都没有进入GPU参数，因此不能通过把雨粒子preset接到静态按钮来完成Weather。

真正需要建立的是一条单一、确定、可编译、可持久化的环境状态链：`ClimateProfile/WeatherPreset/WeatherTimeline/Region Binding source -> validator/compiler -> generation-qualified WeatherProgramArtifact -> WorldWeatherService + CelestialClock -> immutable WeatherFrameSnapshot -> Sky/Atmosphere/Cloud/IBL/Fog/Wind/Precipitation/Surface/Lighting/Audio/Gameplay adapters -> Editor document/preview/diagnostics`。Climate描述慢变量与地理基线，Weather描述可复现状态迁移，Time-of-Day描述天体时钟，Region只提供空间权重；各渲染和游戏子系统仍拥有自己的执行、资源和质量策略。

参考源码也支持这一分层。Unreal把SkyAtmosphere、VolumetricCloud、DirectionalLight、ExponentialHeightFog和WindSource拆成可序列化组件，并显式绑定Atmosphere Sun与cloud shadow；它并没有一个万能Weather property bag。Unity HDRP用VisualEnvironment、PhysicallyBasedSky、VolumetricClouds和Volume参数表达可混合环境，甚至公开global/custom/additive/multiply wind与可网络同步的cloud animation data，但它仍不是Gameplay天气authority。Godot把Environment/Sky/FogVolume做成资源和场景节点；Bevy提供物理大气介质、LUT和ECS提取；Fyrox的SkyBox虽简单，也有Reflect/Visit、资源验证、Scene持久化和render消费。Zircon当前连这些较低基线的项目authoring闭环都未达到。

本报告登记5个P0、70个P1、12个P2、M0-M11重构路线和32个验收门。它只做review，不修改Runtime、Editor、plugin、interface、App生产代码或tests。

## 2. 审查边界与证据

### 2.1 当前工作树物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| Weather Editor静态产品面 | 11 / 4,835 / 236,083 | E3逐control/route/feedback/catalog test：固定资产、固定timeline、固定queued、字段提交只更新模板状态；11个test attributes |
| Scene、Light与Environment入口 | 16 / 4,097 / 152,217 | E3逐schema/load-save/extract：Scene无环境组件，方向光与天空太阳分裂，viewport bool是唯一World environment source；15个test attributes |
| Sky/IBL/Cloud/Fog选取链 | 24 / 6,986 / 248,673 | E3逐scheduler/graph/recorder/shader：gradient、双缓冲、generation、Cloud重复覆盖、Volumetric Fog基础；73个test attributes、1个ignored |
| Runtime clocks | 10 / 966 / 29,806 | E3逐real/virtual/fixed/frame/timer：duration、pause、scale和fixed budget真实，但无calendar/geography/celestial authority；9个test attributes |
| Particle可复用执行器 | 24 / 4,748 / 168,762 | E3逐asset/CPU/GPU/service/editor contribution：gravity/drag/固定external force真实，无wind/weather/precipitation adapter，Editor menu disabled；24个test attributes |
| Catalog、Terrain与Sound边界 | 6 / 772 / 28,746 | E2/E3核对第一方装配与下游descriptor：无Weather注册，Terrain无天气输入，Sound无风雨雷语义；1个test attribute |
| selected combined scope | 91 / 22,404 / 864,287 | 当前工作树fingerprint `7c2378aec5cfff40dea096c2b99cc56b43d873694cba64d0f07de3242fb779ce`；133个test attributes、1个ignored、8个在途文件 |

唯一ignored项是`realtime_ibl_wgpu_recorder/tests.rs`中的完整PMREM+SH9 WGPU product validation。它被标记为manual ignored，且adapter创建失败时仍可提前退出；不能把73个environment test属性等同于真实Weather、Cloud或产品GPU验收。

8个在途文件为`world_building.rs` template binding、`volumetric.rs`及其tests、`environment/extract.rs`、`ibl_bake_artifact.rs`、`environment/mod.rs`、`skybox.rs`和`source_cubemap.rs`，均非本轮产生。本报告按读取时当前工作树事实编写；实施前必须重导91文件manifest、重算fingerprint并复核这些文件的最终schema、extract、artifact和binding合同。

### 2.2 Weather Editor静态事实

1. Weather workspace只有Layers、Curves、Timeline三个tab按钮，没有对应document pane model或typed tab content。
2. 左侧固定显示`Weather_Storm`、`Region_Mountains`和`Layer_Clouds`，没有project asset query、Scene selection或empty/error状态。
3. 时间线固定显示Cloud Build `00:00-02:00`、Rain Burst `02:00-04:00`、Wind Gust `03:20-05:00`、Lightning `04:00-04:30`。
4. 输出固定为`8 layers 5 regions 2 warnings`；它不包含job ID、source revision、artifact digest或diagnostic identity。
5. Preset固定为Storm/Clear/Fog/Snow，Region固定为Mountains/Coast/City/Interior，Blend Time是字符串`12.0`。
6. template binding只把Change/Submit/Click映射到route；没有Editor operation factory、command handler或domain controller。
7. navigation spec只负责显示workspace、选择control和切tab，不能读写资产或运行runtime simulation。
8. feedback对Open、Preview、Build、Rain Burst和Lightning返回固定字符串；`queued`没有进入Editor09 job authority。
9. Workbench preview action allow-list证明这些route可被演示，不证明存在Weather业务执行器。
10. 第一方Runtime catalog没有Weather或Climate registration；第一方Editor catalog只注册Navigation和Neural。
11. `zircon_plugins/weather`与`zircon_plugins/climate`目录都不存在。
12. 源码中的`weather.Component.CloudLayer`、`weather.Component.Wind`和`plugins/weather`全部位于generic component/plugin loader测试fixture，不是生产Weather类型或package。

### 2.3 Scene、太阳与Time-of-Day静态事实

1. `SceneEntityAsset`可持久化Camera、Mesh、五类Light、Post Process Volume、Physics、Animation、Terrain、TileMap、Prefab和Script binding，但没有Environment或Weather引用。
2. Scene component模块没有Sky、Atmosphere、Cloud、Wind、Precipitation、Climate或TimeOfDay component。
3. `build_environment_extract()`只调用`EnvironmentExtract::from_preview_skybox_enabled(request.settings.preview_skybox)`。
4. preview_skybox为true时固定使用`ProceduralSkyParams::default_gradient()`；false时完全disabled。
5. Source Cubemap environment虽有真实PMREM/SH/IEM/upload artifact，但普通Scene没有字段能引用它。
6. `ProceduralSkyParams`提供horizon/zenith/ground、sun direction/color/intensity/angular radius、environment intensity、rotation和source revision。
7. 默认程序天空的sun direction固定为上方，sun intensity为0；Scene方向光默认另有一组direction与intensity。
8. DirectionalLight只有direction、color、intensity和volumetric；没有太阳/月亮identity或Atmosphere/Cloud coupling字段。
9. World light extract直接复制DirectionalLight字段，不查询sky、clock或weather generation。
10. Runtime time有real/virtual/fixed三套duration clock、frame index、pause、relative speed、max delta与fixed-step budget。
11. Runtime time没有date、calendar、day length、latitude、longitude、timezone、season、axial tilt、solar elevation、moon phase或celestial body identity。
12. 因而`02:00-04:00`既不能解释为游戏内日期时间，也不能确定它跟virtual time、real time、sequence time或预览scrub的关系。

### 2.4 Sky、Cloud、IBL与Fog静态事实

1. 程序天空fragment shader只在horizon/zenith/ground三色间插值，并叠加smoothstep sun disk。
2. 源码没有Rayleigh、Mie、ozone、turbidity、transmittance LUT、multi-scattering LUT、sky-view LUT或aerial perspective LUT的Zircon生产实现。
3. Realtime IBL以`IblBakeKey`比较sky source revision/参数，变更时申请rebake。
4. scheduler用A/B slot、generation token和published/pending key阻止旧提交覆盖新状态。
5. 首次更新在单帧安排全部六面Sky、六面Cloud、source mips、全部PMREM和SH9，不执行时间切片。
6. 后续更新按每帧两面capture与分阶段prefilter切片，共有Retry、Stale、Advanced与Published终态。
7. `CaptureCloud`有独立graph pass和executor ID，但recorder将它与`CaptureSky`合并到同一分支。
8. 两个operation调用相同`record_capture()`、相同`ProceduralSkyParams`和相同output source mip。
9. capture shader没有cloud输入，所以Cloud pass只是再次写入相同gradient。
10. scheduler没有weather importance、camera demand、changed-subsystem dirty mask或GPU time budget自适应。
11. publish只有slot切换，没有旧/新环境radiance的时域crossfade合同。
12. Volumetric Fog有density、albedo、phase_g、height falloff、scattering intensity、depth distribution和temporal开关。
13. Fog参数没有humidity/aerosol visibility/rain extinction等Weather语义；Local Fog的shape/priority问题归Runtime09G1与Editor37。
14. Exposure、Fog、Sky、Cloud与DirectionalLight没有共享Environment generation或依赖图。

### 2.5 Wind、Precipitation、Lightning与下游静态事实

1. 全仓生产源码没有WindField、WeatherSnapshot、Precipitation、rain intensity、wetness、snow accumulation或snow mask产品类型。
2. Terrain runtime只注册Terrain asset reference与diagnostic-only heightfield importer，没有风、积雪、湿润或侵蚀输入。
3. Material/Graphics生产搜索没有wetness/snow/rain/weather参数owner或global surface-state buffer。
4. Sound component可表达source/listener/volume，但没有weather ambience、rain loop、wind gust或thunder event类型。
5. Particle emitter有固定gravity、drag、initial velocity、shape、rate、burst、color/size curve和CPU/GPU backend。
6. CPU physics capability允许使用资产中固定external force；它不是按位置或时间查询的风场。
7. GPU emitter layout只编码gravity与drag，没有`ParticlePhysicsOptions.external_force`，CPU/GPU在该能力上不对等。
8. Particle shape只有Point/Sphere/Box/Cone，没有camera frustum precipitation volume、screen-space spawn budget或world streaming policy。
9. Particle runtime没有surface collision hit stream、splash/decal/puddle emitter bridge或snow accumulation receipt。
10. Weather没有调用Particle manager的instantiate/tick/build_extract，也没有把天气强度转成emission rate的typed adapter。
11. 仓内没有Lightning scheduler、seed、bolt geometry、flash light、exposure impulse、cloud illumination、thunder delay或Gameplay strike authority。
12. Region dropdown没有接Editor37的SpatialRegion source/geometry/generation，也没有global/local/altitude/weather front混合规则。

### 2.6 动态证据边界

本轮是review-only，没有修改Runtime、Editor、plugin、interface、App生产代码或tests，也没有运行新的动态测试。静态ZUI/feedback、Scene environment断点、Cloud重复capture、Particle CPU/GPU输入差异和Weather类型缺失都可由当前源码直接证明。

此前`zircon_editor --lib`测试编译在617.2秒后被239个既有错误和122个warning阻断，当前源代码没有解除该阻断条件，本轮没有重复相同lane。后续实施必须先恢复可编译基线，再运行真实项目roundtrip、deterministic simulation、GPU cloud/IBL、particle precipitation、audio/lightning、network/save与Editor交互组合验收。

## 3. 目标架构

### 3.1 单一Weather authority与typed adapters

```text
Project / Scene authoring sources
  -> ClimateProfileAsset
  -> CelestialProfileAsset
  -> WeatherPresetAsset
  -> WeatherTransitionGraph / TimelineAsset
  -> WeatherRegionBinding(SpatialRegionId, layer, priority, weight policy)
  -> WeatherCompiler + validators
  -> WeatherProgramArtifact(schema, source digest, dependencies, deterministic tables)
  -> WorldWeatherService(world generation, authority mode, simulation clock, seed)
  -> immutable WeatherFrameSnapshot
       -> Celestial/Sun/Moon adapter
       -> SkyAtmosphere/AerialPerspective adapter
       -> Cloud/IBL/Fog adapter
       -> WindField adapter
       -> Precipitation/VFX adapter
       -> SurfaceWetness/Snow/Terrain/Vegetation/Water adapter
       -> Lightning/Lighting/Exposure adapter
       -> Sound ambience/Thunder adapter
       -> Gameplay/AI/Navigation/Streaming adapter
  -> Editor document / timeline / map / preview / contributor / diagnostics
```

Weather runtime只能发布immutable、generation-qualified snapshot；Render、Physics、Particle、Sound和Gameplay线程不得直接读取Editor document、可变HashMap或任意JSON。每个adapter显式声明所需字段、update cadence、fallback、quality tier、last-good generation和budget。

### 3.2 Climate、Weather、Celestial与Region的职责

| Owner | 应拥有 | 不应拥有 |
|---|---|---|
| ClimateProfile | 纬度/海拔/季节基线、温湿度/风/云/降水统计、允许状态与概率参数 | 每帧GPU资源、粒子实例、声音voice |
| CelestialProfile/Clock | calendar/day length、地理坐标、太阳/月亮轨迹、时间跳变和scrub语义 | Weather随机转移、云材质、IBL资源 |
| WeatherPreset/Graph | 状态参数、transition、hysteresis、duration、seed policy、layer outputs | Region geometry、Renderer内部pass、DSP node |
| SpatialRegion | stable geometry/transform/filter/priority/source revision | 天气字段万能property bag、每域执行逻辑 |
| WorldWeatherService | authority、deterministic tick、state transition、regional resolve、snapshot publish | Editor UI、GPU command、音频线程可变状态 |
| Domain adapter | snapshot到本域typed command/proxy、budget/fallback/receipt | 重新解释timeline或自行产生Weather真值 |

Climate是慢变量和约束，不应每帧采样复杂统计模型；Weather是可复现的离散/连续状态；CelestialClock是独立、可跳变的时间源；Region只产生空间影响。四者分离可防止“把所有字段塞进Storm preset”再次形成不可迁移的临时实现。

### 3.3 建议核心类型

1. `ClimateProfileId`、`WeatherPresetId`、`WeatherProgramId`、`WeatherRegionBindingId`使用稳定内容身份，不使用display name或ECS index。
2. `WeatherSourceRevision`、`WeatherArtifactDigest`、`WeatherInstallGeneration`、`WorldGeneration`和`SimulationEpoch`贯穿source到runtime receipt。
3. `CelestialTime`保存calendar tick与可解释的day fraction；显示时区不得改变simulation identity。
4. `WeatherStateId`与`WeatherTransitionId`稳定，可记录from/to、start tick、duration、seed、reason和authority sequence。
5. `WeatherLayerValue<T>`表达global、region-weighted、override/add/multiply/min/max等经过schema许可的组合，不用字符串运算符。
6. `WindFieldSnapshot`表达global vector、gust/turbulence参数与可选bounded local contributors，查询有LOD和budget。
7. `PrecipitationSnapshot`按类型、强度、粒径、fall velocity、temperature phase、visibility和surface deposition输出。
8. `CloudSnapshot`表达layer altitude/thickness、coverage/density/type、weather map/material、wind offset、lighting/shadow与history generation。
9. `AtmosphereSnapshot`表达planet geometry、scattering/absorption介质、aerial perspective和LUT generation。
10. `WeatherFrameSnapshot`包含tick、state、region contributors、dirty domains、snapshot bytes和overflow/quality disposition。
11. `WeatherAdapterReceipt`记录accepted/rejected generation、applied domains、fallback、budget和diagnostics。
12. `WeatherPreviewSession`绑定document revision、preview world、camera、clock、quality、random seed和last-good artifact。

### 3.4 更新频率与失效边界

1. Celestial transform可按simulation tick更新，Atmosphere LUT只在介质/行星/关键太阳阈值变化时失效。
2. DirectionalLight direction与sky sun disk必须来自同一celestial sample；light shadow仍由Runtime09E拥有。
3. Cloud advection可每帧更新offset，cloud map/material/quality变化才重建重资源。
4. IBL不能随每个Weather浮点变化全量rebake；需要阈值、dirty domain、capture cadence、GPU time budget和旧/新radiance blend。
5. Wind global/local field按固定Weather tick发布，Particle、Vegetation、Cloth、Water和Audio按自己的采样频率消费同一generation。
6. Precipitation强度可连续变化，但emitter capacity、spawn distribution和surface deposition应有独立预算。
7. Lightning事件使用deterministic schedule和stable strike identity；visual flash、light、thunder和Gameplay receipt引用同一event。
8. Region进出只改变contributor set和blend state，不直接创建无界粒子/音频/Render资源。
9. 时间jump、scrub、load、network correction、world origin shift和quality change都必须产生明确history reset或reprojection policy。
10. snapshot发布与adapter apply在frame/tick boundary完成；旧generation late result不能覆盖当前世界。

### 3.5 Editor产品边界

Weather Editor应是versioned asset toolkit，不是静态dashboard。它必须读取project asset registry和Scene region binding，使用Editor02 transaction/save/recovery，Editor03/05 selection/Inspector/gizmo，Editor09 job与Editor11 diagnostic journal。Layers、Curves和Timeline应是同一document的不同投影；Preview只消费compiler artifact和PreviewWorld snapshot，不得直接改Runtime可变状态。

Build必须返回`WeatherBuildReceipt`：source revision、artifact digest、dependency manifest、diagnostic set、duration、cache disposition和publish结果。Preview必须返回session/generation/frame identity。Region map必须展示Editor37编译geometry和逐层贡献。Output面板只过滤typed diagnostics，不维护固定计数或私有日志。

## 4. P0 阻断项

### P0-1：Weather Editor是静态第二authority

页面、timeline、preset、region、warning和feedback全部硬编码；Preview/Build不进入job、compiler或runtime。必须在真实domain完成前硬切Unavailable/empty状态，删除固定成功与业务样例，不能继续用`queued`冒充执行。

### P0-2：不存在Weather/Climate产品owner与source-to-runtime链

没有插件、catalog registration、asset/schema/component/compiler/artifact/service或Scene引用；测试中的weather名字只是generic fixture。必须先确定core/plugin ownership、server/client/editor模块和versioned package contract，再开放产品入口。

### P0-3：Scene Environment、太阳与Time-of-Day没有共同真值

Scene只通过viewport bool得到固定gradient，程序天空太阳与DirectionalLight分裂，Runtime duration clock没有celestial语义。必须建立CelestialClock与Environment generation，让sun disk、方向光、大气、云、雾、曝光和IBL共享可追溯sample。

### P0-4：Cloud/Precipitation/Wind/Lightning只存在名字或不完整原语

`CaptureCloud`重复执行sky gradient；生产代码没有Cloud medium、WindField、Precipitation或Lightning chain。Particle固定gravity/external force不能替代风雨雪，Volumetric Fog不能替代云。必须给每个域建立真实typed source、runtime adapter和失败/降级状态。

### P0-5：没有确定性Weather simulation、Region resolve、save/network/cook闭环

静态timeline没有时间源、transition语义、seed、authority、artifact或runtime install；Region下拉没有geometry。必须建立deterministic program、fixed tick、state receipt、regional blending、save/replay/replication和cook dependency，才能允许Build、Preview或Play。

## 5. P1 工程化重构项

### 5.1 产品装配、schema与生命周期（P1-01 至 P1-10）

| ID | 差距 | 重构要求 |
|---|---|---|
| P1-01 | `RuntimePluginId`、project manifest和first-party catalog没有Weather/Climate身份 | 定义唯一package/module/capability IDs及EditorHost/Client/DedicatedServer目标矩阵 |
| P1-02 | Weather workspace随Editor模板存在，不受插件readiness控制 | surface由reason-coded capability snapshot驱动，缺owner时Unavailable而不是展示样例 |
| P1-03 | 没有Climate/Weather/Celestial source asset kind | 注册versioned asset schema、type presentation、creation template、toolkit和dependency collector |
| P1-04 | 没有Scene级Environment/Weather binding | Scene保存stable asset refs、region bindings、clock source与override，支持Prefab/unknown roundtrip |
| P1-05 | 没有schema version与migration | 每类source和artifact独立version，提供upgrade/downgrade report、backup和rollback |
| P1-06 | dynamic component测试名可能被误认为产品能力 | capability/report只接受真实registration+provider+consumer，不从fixture字符串推断 |
| P1-07 | 没有plugin unload/reload语义 | 卸载撤销service/adapters/proxies，保留未知source数据，late result按generation拒绝 |
| P1-08 | 没有Dedicated Server裁剪规则 | server保留deterministic state/Gameplay/replication，裁剪GPU/VFX/Audio且仍产出相同权威事件 |
| P1-09 | 没有world/scene lifecycle owner | create/load/activate/deactivate/unload/destroy/Play exit必须原子安装或撤销Weather generation |
| P1-10 | 没有public query/command ABI | 提供bounded typed snapshot/query/override接口，禁止外部插件借用内部可变Weather service |

### 5.2 Climate、Celestial Clock与Time-of-Day（P1-11 至 P1-22）

| ID | 差距 | 重构要求 |
|---|---|---|
| P1-11 | Runtime time只有duration和frame index | 新增独立CelestialClock，不污染Real/Virtual/Fixed clock基础 |
| P1-12 | Weather timeline未声明使用哪个clock | 每个program显式选择simulation/celestial/sequence/preview clock与pause/scale政策 |
| P1-13 | 没有calendar、day length和epoch | 定义整数tick calendar、day fraction、rounding、overflow与序列化语义 |
| P1-14 | 没有纬度、经度、海拔、北向与up-axis | CelestialProfile定义地理/坐标约定，并与large-world/origin shift解耦 |
| P1-15 | 没有太阳位置算法与精度合同 | 实现可复现ephemeris或项目曲线，记录近似范围、单位、误差和测试reference |
| P1-16 | 没有月亮、星空与多天体identity | 先建立Sun/Moon stable IDs和light/disk policy，再扩展多星体，禁止匿名第二方向光 |
| P1-17 | DirectionalLight不能声明Atmosphere Sun | 增加typed celestial binding、index/role、angular radius、illuminance/color temperature接口 |
| P1-18 | sun disk与方向光参数可漂移 | 由同一CelestialSample生成方向、disk、光度和invalidate key，Editor只编辑source |
| P1-19 | 没有season/climate baseline | ClimateProfile按calendar/region输出慢变量与allowed weather envelope，不每帧随机生成 |
| P1-20 | 时间跳转/scrub没有history语义 | 输出Jump/Continuous/Correction disposition，Cloud/Fog/Exposure/TAA/IBL各自决定reset或reproject |
| P1-21 | pause/time-scale与Weather transition关系未定义 | 明确暂停是否冻结Weather、Lightning、Cloud offset和audio；Editor preview可独立控制 |
| P1-22 | 浮点累计会损害长时确定性 | authoritative clock和transition使用整数tick，渲染插值使用局部float sample |

### 5.3 Atmosphere、Sky、Cloud、IBL与Fog（P1-23 至 P1-38）

| ID | 差距 | 重构要求 |
|---|---|---|
| P1-23 | Scene只能保存preview skybox bool | 增加Environment asset/component引用、fallback和project default，并接source cubemap |
| P1-24 | 三色gradient被当作默认完整天空 | 将其标为Low/Artistic fallback；正式路径实现physical atmosphere或明确provider unavailable |
| P1-25 | 无Rayleigh/Mie/absorption介质 | 定义物理单位、planet radii、ground albedo、density falloff、phase和spectral/RGB approximation |
| P1-26 | 无transmittance/multi-scattering/sky-view/aerial LUT | 建立缓存、quality tier、dirty dependency、GPU/CPU reference与last-good publish |
| P1-27 | `CaptureCloud`重复覆盖sky | 在真实Cloud完成前删除该operation；完成后输入cloud radiance/transmittance而非复用gradient |
| P1-28 | 无Cloud source schema | 定义layer altitude/thickness、coverage/type/density、weather map、material/noise和wind binding |
| P1-29 | 无Cloud tracing/quality | 定义view/light/reflection sample budget、distance、early exit、half/quarter resolution和fallback |
| P1-30 | 无Cloud光照与shadow | 接Atmosphere transmittance、Sun/Moon、ambient probe、ground contribution和cloud shadow artifact |
| P1-31 | 无Cloud temporal reconstruction | history绑定view/cloud/weather/quality generations，处理motion、camera cut、jump和ghost rejection |
| P1-32 | Realtime IBL首次更新单帧全量 | 首次也接受admission/time budget，使用fallback/last-good，不制造首帧GPU尖峰 |
| P1-33 | IBL只按完整BakeKey触发 | 引入domain dirty mask、阈值和cadence，区分sky介质、sun、cloud、exposure与纯offset变化 |
| P1-34 | IBL publish没有radiance crossfade | 发布双generation与blend window，处理cancel/stale/device loss和memory budget |
| P1-35 | IBL scheduler固定每帧两面 | 由GPU timestamps、frame budget、camera demand和quality profile自适应，保留确定上限 |
| P1-36 | Fog与Weather完全分离 | Weather只通过typed adapter驱动humidity/aerosol/visibility目标，不直接写froxel资源 |
| P1-37 | Fog、Cloud、Sky、Exposure缺共同generation | EnvironmentFrameSnapshot携带各domain generation和history reset原因，禁止按名称猜同步 |
| P1-38 | 多camera/view family没有环境策略 | 明确global/region/view override、reflection capture、editor preview和split-screen资源共享边界 |

### 5.4 Wind、Precipitation、Lightning与Surface（P1-39 至 P1-54）

| ID | 差距 | 重构要求 |
|---|---|---|
| P1-39 | 没有Wind source/field | 定义global directional、point/local、gust、turbulence、altitude profile和stable contributor IDs |
| P1-40 | 没有空间查询与LOD | 构建bounded WindFieldSnapshot/clipmap或grid，支持位置/高度查询、region blend和overflow |
| P1-41 | 下游各域可能复制风参数 | Particle/Vegetation/Cloth/Water/Audio只消费同一wind generation和domain-specific filtered sample |
| P1-42 | Particle CPU external force是资产常量 | 增加runtime force provider接口，区分asset baseline和per-frame WindField sample |
| P1-43 | Particle GPU不编码external force | 统一CPU/GPU emitter force合同，增加parity fixture和unsupported capability诊断 |
| P1-44 | 无Rain/Snow/Hail source | Precipitation schema表达phase、intensity、drop/flakes、fall velocity、visibility、temperature和budget |
| P1-45 | 无camera-relative precipitation volume | 以view/camera需求生成bounded emitter proxy，处理teleport、multi-camera和indoor mask |
| P1-46 | 无surface impact链 | Physics/query提供batched hit或depth/heightfield方案，产出splash/ripple/decal receipt和预算 |
| P1-47 | 无wetness/puddle/snow accumulation | 建立SurfaceWeatherState与material/terrain/water typed adapter、persistence和evaporation/melt policy |
| P1-48 | Terrain插件没有Weather输入 | Terrain按chunk/cell消费deposition/temperature/wind，不让Weather直接改heightfield或layer asset |
| P1-49 | 无Vegetation/foliage风响应owner | 待Editor16/runtime植被系统落地后接Wind adapter；能力缺失时明确Unsupported而非静态摆动 |
| P1-50 | 无Lightning deterministic schedule | strike由program seed、tick、region/candidate与authority sequence生成，支持cancel/correction |
| P1-51 | 雷电视觉效果彼此无identity | bolt VFX、flash Directional/Local Light、cloud illumination和exposure impulse引用同一StrikeId |
| P1-52 | 无Thunder传播 | 根据strike/listener位置和speed-of-sound调度Sound event，处理listener切换、pause和late join |
| P1-53 | 无Gameplay lightning/temperature/hazard事件 | 通过Runtime08G typed command/effect执行，server authority与cosmetic adapter分离 |
| P1-54 | 无indoor/occlusion/cover语义 | 以Region/visibility/physics typed mask驱动降水、音频和surface影响，不用名称或单ray临时判断 |

### 5.5 Weather状态、Region、网络、保存与规模（P1-55 至 P1-64）

| ID | 差距 | 重构要求 |
|---|---|---|
| P1-55 | 没有Weather transition graph | 定义state、edge、condition、duration distribution、hysteresis、cooldown和priority，编译前验证 |
| P1-56 | 没有deterministic RNG | 使用program seed + world/region/state/transition identity派生流，禁止调用全局随机或wall clock |
| P1-57 | Region dropdown不接空间authority | 绑定Editor37 SpatialRegionId/compiled geometry/generation并显示实际contributors |
| P1-58 | global/local/nested Weather无组合政策 | 对每层字段声明override/add/multiply/min/max/normalized blend及tie-break，compiler拒绝非法组合 |
| P1-59 | Region移动/流送无状态连续性 | contributor add/remove按stable binding和blend history处理，cell unload不丢权威state或重复事件 |
| P1-60 | 没有server authority/replication | 复制state/tick/transition/seed/correction而非每帧全部float；客户端重建并记录误差 |
| P1-61 | 没有prediction/late join | 区分authoritative gameplay与cosmetic prediction，late join从snapshot+event cursor恢复 |
| P1-62 | 没有save/load/replay | 保存program digest、clock、state、transition progress、RNG streams、regional overrides和event sequence |
| P1-63 | 没有bounded snapshot/queue | 限制regions、contributors、events、adapter diffs和diagnostics的count/bytes/time，定义overflow终态 |
| P1-64 | 没有大世界/多world预算 | 每World独立service/generation，按cell demand增量resolve；1/1k/100k region基准可重复 |

### 5.6 Editor document、compiler、preview与诊断（P1-65 至 P1-70）

| ID | 差距 | 重构要求 |
|---|---|---|
| P1-65 | Layers/Curves/Timeline没有document authority | 建立WeatherDocumentController、selection、dirty/history、shared/unique profile与multi-edit语义 |
| P1-66 | 字段只是字符串Change/Submit | schema生成typed editor、单位/范围/enum/curve/asset reference，commit形成Editor02 transaction |
| P1-67 | Build没有compiler/job/artifact | 通过Editor09执行validate/compile/cook-preview/publish，支持cancel、progress、LKG和stale fence |
| P1-68 | Preview没有世界、相机、时钟或seed | 创建隔离PreviewWorld/session，绑定source revision、artifact generation、camera、clock、quality和seed |
| P1-69 | Output和warning是固定文本 | 所有诊断进入Editor11 journal，带code/severity/owner path/source span/generation/fix action |
| P1-70 | 测试只验证route和固定反馈 | 建立source roundtrip、compiler golden、determinism、region/network/save、GPU visual/perf和真实UI workflow层级 |

## 6. P2 高阶能力

| ID | 高阶能力 | 进入条件 |
|---|---|---|
| P2-01 | 多行星、多太阳、日食/月食与轨道系统 | 单Sun/Moon CelestialClock、physical atmosphere和authoring闭环通过 |
| P2-02 | 光谱大气、极光、空气辉光与天文星表 | RGB物理大气达到性能/画质门且有reference capture |
| P2-03 | Mesoscale climate、pressure front与orographic precipitation | deterministic Weather graph和大世界region预算稳定 |
| P2-04 | 云体积守恒、对流与多层微物理 | 基础Volumetric Cloud、shadow、temporal和Weather map已产品化 |
| P2-05 | 水循环、土壤含水、积雪压实与融雪径流 | SurfaceWeatherState与Terrain/Water adapter完成 |
| P2-06 | 局部天气雷达、forecast与概率可视化 | Climate/Weather统计模型有可解释数据和误差合同 |
| P2-07 | 飞行高度风切变、热气流与航空湍流 | WindField 3D query、LOD和Physics consumers完成 |
| P2-08 | 高级镜头雨滴、结霜、污渍和水膜光学 | 真实降水/表面状态和Post Process dependency完成 |
| P2-09 | Weather对AI感知、导航成本、生态与人群行为的系统影响 | Gameplay/AI/Nav typed adapter与server authority完成 |
| P2-10 | 分布式云/大气LUT与Weather artifact构建 | 本地cook/DDC key、原子publish和LKG完成 |
| P2-11 | 多人协作Weather authoring与冲突合并 | document transaction、stable IDs、diff/merge schema完成 |
| P2-12 | 与Unreal/Unity同场景自动画质-性能Pareto回归 | 同内容、同分辨率、同硬件、同质量定义和采集工具完成 |

P2不能用来推迟P0/P1的基本真实性。物理大气、真实云、确定性状态、网络/保存和Editor闭环属于P0/P1，不是“以后再做的高级效果”。

## 7. 参考引擎差异矩阵

| 参考 | 可复用原则 | Zircon当前差距 | 不应照搬 |
|---|---|---|---|
| Unreal | SkyAtmosphere公开planet/Rayleigh/Mie/absorption/multi-scattering；DirectionalLight显式Atmosphere Sun与cloud shadow；VolumetricCloud有layer/material/tracing/lighting；WindSource可按位置查询 | Zircon只有gradient、独立sun和重复Cloud capture，无scene component或Weather authority | Unreal Wind源码说明默认主要影响SpeedTree，不能把这种有限consumer当统一风场；Actor/UObject层级也不直接复制 |
| Unity HDRP Graphics | VisualEnvironment选择Sky/Cloud并提供global wind；PhysicallyBasedSky有planet/air/aerosol/ozone；VolumetricCloud有map/LUT/curve/wind/temporal/shadow/quality与可同步animation data | Zircon没有Volume-backed环境资产、Cloud参数、history或同步接口 | HDRP Volume是渲染参数系统，不替代Gameplay Weather、server authority、save或region simulation |
| Godot | Environment统一background/sky/ambient/reflection/fog；Sky有processing/radiance；WorldEnvironment和FogVolume把资源接入Scene | Zircon Scene没有Environment引用，Source Cubemap和Fog基础无法从项目authoring闭环使用 | Godot的传统环境能力是完整性下限，不是性能/画质上限，也没有统一气候模拟 |
| Bevy | Atmosphere以ECS component + ScatteringMedium表达planet、absorption、scattering、falloff、phase，并用LUT/raymarch渲染sky/aerial perspective | Zircon缺介质、LUT、scene extract和物理光照共同真值 | Bevy没有Zircon所需的完整Editor、Weather timeline、network/save产品，不能作为终点 |
| Fyrox | SkyBox具备Reflect/Visit、Scene field、异步资源等待、cubemap构建验证和render/IBL消费 | Zircon虽有更强IBL数据模型，却没有Scene可保存Sky引用 | 六面SkyBox只是最低authoring基线，不能替代physical atmosphere/cloud/weather |

参考结论不是“复制某一个引擎的Weather面板”。五套源码共同证明：可序列化source、Scene owner、typed runtime proxy、资源generation、渲染消费和Editor投影必须闭环；而天气状态、网络、保存与跨域适配仍需Zircon自己定义统一架构。

## 8. 分层实施路线

| 里程碑 | 内容 | 前置 | 退出条件 |
|---|---|---|---|
| M0 Truthfulness与基线 | Weather workspace硬切Unavailable/empty；恢复Editor编译；冻结gradient/IBL/Fog/Particle/Light当前fixture和失败证据 | 无 | 不再显示固定Storm/timeline/warnings/queued；现有底座行为可重复 |
| M1 Domain与Schema | 确定Weather package/core边界；Climate/Celestial/Weather/Region source schema、stable IDs、version/migration | M0、Runtime04/05 | old/new source roundtrip、unknown保留、invalid在compile前拒绝 |
| M2 Compiler与Artifact | transition/curve/region dependency compiler、deterministic tables、diagnostics、artifact digest、DDC/LKG | M1、Editor09/11 | 相同source产生相同artifact，失败不覆盖LKG，cancel/stale不可publish |
| M3 Celestial与Environment authority | CelestialClock、Sun/Moon sample、DirectionalLight/Sky/Exposure/Fog/IBL generation dependency | M2、Runtime09E/F/H | 时间连续/jump下所有consumer引用同一sample和reset reason |
| M4 Physical Atmosphere | planet/scattering/absorption/LUT/aerial perspective、quality/cache/fallback | M3、Runtime09F1 | Scene source到pixel闭环，reference与预算通过，gradient降为明确fallback |
| M5 Volumetric Cloud | layer/material/weather map/wind/lighting/shadow/temporal/IBL capture，删除伪CaptureCloud | M3-M4、Runtime09F1/09G1 | Cloud真实输入可见，capture不重复gradient，history/quality/device loss通过 |
| M6 Weather Runtime与Region | fixed tick、RNG、state graph、regional resolve、snapshot publish、bounded events | M2、Editor37 | deterministic replay、nested region、streaming和100k scale预算通过 |
| M7 Wind/Precip/Surface/Lightning | WindField、Particle CPU/GPU adapter、rain/snow/hail、impact/deposition、Strike/Thunder chain | M5-M6、Editor15/16/17 | 同generation跨VFX/Surface/Light/Audio/Gameplay闭环，capability差异可解释 |
| M8 Editor产品 | toolkit、Layers/Curves/Timeline/Map、typed Inspector、transaction/save/recovery、PreviewWorld、jobs/diagnostics | M1-M7、Editor02/03/05/09/11 | 所有显示来自真实document/artifact/snapshot/receipt，undo/save/reopen一致 |
| M9 Network/Save/Cook | authority/replication/late join/correction、save/load/replay、cook dependency、headless裁剪 | M6-M8 | server/client/replay结果一致，package不依赖Editor path，旧generation被拒 |
| M10 Scale/Quality/Fault | GPU/CPU/RAM/VRAM budgets、multi-view、large world、device loss、fuzz/soak、cross-platform | M4-M9、Tooling07/10 | required lanes无ignored，qualification artifact可重现，fallback不伪成功 |
| M11 Hard Cutover | 迁移旧preview bool/静态workspace，删除Cloud假operation与所有旁路，生成maturity/rollback证据 | M10 | 全仓无第二authority或fixture误报，旧项目迁移与rollback通过 |

M0-M3不能跳过。直接新增Rain Particle preset、DayNight脚本或把`sun_direction`暴露到面板，只会制造第三套环境真值，并扩大后续迁移成本。

## 9. 验收门 G01-G32

### G01：Product truth

空项目或未启用Weather provider时不显示Storm、region、timeline、warning或queued成功；UI显示typed Empty/Unavailable/Failed原因。

### G02：Source roundtrip/migration

Climate、Celestial、Weather、Timeline和Region binding可保存、重开、迁移、unknown plugin字段保留；失败有rollback。

### G03：Compiler determinism

相同normalized source/dependencies在不同机器和重复构建中产生相同artifact digest、state IDs、tables和diagnostics顺序。

### G04：Artifact publish/LKG

compile cancel、disk full、corrupt dependency、stale revision和plugin unload不能覆盖last-good artifact；receipt可追溯。

### G05：Clock identity

real/virtual/fixed/celestial/sequence/preview clock不会隐式混用；pause/scale/jump/scrub行为按program声明并可测试。

### G06：Celestial correctness

date/geography/up-axis corpus中Sun/Moon方向与reference在误差内，long-session整数tick无可观察漂移。

### G07：Sun single truth

sky disk、DirectionalLight、atmosphere transmittance、cloud lighting/shadow和IBL key引用同一CelestialSample/generation。

### G08：Environment Scene closure

Scene/Prefab保存Environment source后无需测试手工覆盖snapshot即可从project load到render pixel，Source Cubemap同样可达。

### G09：Physical atmosphere

Rayleigh/Mie/absorption、多重散射、aerial perspective与ground albedo有CPU/GPU reference、quality tier和visual golden。

### G10：Cloud semantic truth

删除重复gradient Cloud pass；Cloud capture含真实cloud radiance/transmittance，coverage/density/material变化产生可见且可测结果。

### G11：Cloud temporal/history

camera movement/cut、weather transition、time jump、quality change、dynamic resolution和device reset得到正确history reset/reprojection，无持续ghost。

### G12：IBL update budget

首次和后续environment更新都受GPU time/dispatch/bytes预算；无整帧尖峰，publish有last-good与crossfade。

### G13：Fog/Exposure coupling

Weather到Fog/Exposure使用typed adapter和generation；humidity/aerosol变化不绕过Runtime09G/H schema或直接写GPU资源。

### G14：Weather transition determinism

state duration、condition、hysteresis、cooldown、RNG和tie-break在save/load/replay/server/client上得到相同transition sequence。

### G15：Region blending

global/local/nested/equal-priority/moving/streamed regions按声明组合；contributor view与runtime snapshot数值一致。

### G16：Region lifecycle

disable、destroy、reparent、cell unload、world destroy、Play exit和plugin unload撤销bindings/proxies；late result被generation拒绝。

### G17：Wind field parity

global/local/gust/turbulence/altitude corpus在CPU reference与GPU consumers中误差受控，position query有LOD和overflow状态。

### G18：Particle CPU/GPU parity

相同Weather wind/precipitation输入下CPU/GPU的force、spawn、lifetime和fallback语义一致；不支持能力显式诊断。

### G19：Precipitation view budget

rain/snow/hail在1/2/4 camera、teleport、indoor/outdoor和高强度下满足particle count、spawn、GPU/CPU和memory预算。

### G20：Surface deposition

wetness/puddle/snow对terrain/material/water的deposit、evaporate/melt、save/load和streaming结果可重现且无跨cell接缝。

### G21：Impact effects

surface hits、splash/ripple/decal由bounded batch产生，overflow可观测，不在每drop创建无界Physics query或entity。

### G22：Lightning identity

每个StrikeId关联bolt、flash light、cloud illumination、exposure impulse、thunder和Gameplay receipt；cancel/correction不重复伤害。

### G23：Thunder timing

多listener、移动listener、pause/time scale、late join和region occlusion下arrival time/attenuation与声明一致。

### G24：Network authority

server只复制必要state/tick/seed/event/correction；客户端cosmetic reconstruction可校验，网络抖动不会改变Gameplay结果。

### G25：Save/load/replay

中transition、雷电待触发、region streaming与time jump状态可保存恢复；replay event cursor无丢失或重复。

### G26：Editor transaction/save

Layers/Curves/Timeline/Region/Inspector的edit/cancel/undo/redo/save/autosave/recovery/conflict使用统一document authority。

### G27：Preview isolation

PreviewWorld绑定document revision/artifact/camera/clock/seed/quality；关闭或失败不会修改authoritative Scene或Play world。

### G28：Diagnostics/jobs

Build/preview/cook返回真实job和receipt；diagnostic带stable code、source span、owner、generation和fix action，无固定计数。

### G29：Bounded data与scale

1/1k/100k region、长timeline、百万粒子候选、多world和长session满足CPU/RAM/VRAM/queue预算，无无界集合。

### G30：Fault/headless/package

NaN/Inf、malformed source/artifact、missing plugin、cancel、disk full、device loss和Dedicated Server均有确定终态且保留LKG。

### G31：Cross-platform/quality

Windows/Linux、各GPU backend与quality tier报告Supported/Degraded/Unavailable；降级画面和成本有资格证据，不伪装相同能力。

### G32：Competitive maturity/rollback

在相同场景、分辨率、硬件和画质定义下完成与Unreal/Unity的CPU/GPU/RAM/VRAM/稳定性对比；版本升级可canary和rollback后才标stable。

## 10. 跨计划边界

- Runtime01/02拥有service lifecycle、event/task和shutdown；Weather service、adapter与event必须复用其generation和bounded execution。
- Runtime04拥有asset/resource/artifact/DDC/residency；本篇定义Weather内容，不另建文件cache或GPU upload owner。
- Runtime05拥有World/ECS/Scene lifecycle；Weather只发布World-scoped service/snapshot，不复制实体存储。
- Runtime09E拥有DirectionalLight、physical units和shadow；本篇只建立Celestial binding与cloud/atmosphere dependency。
- Runtime09F1拥有Sky/IBL/Reflection Probe底层重构；本篇拥有Climate/Weather/Celestial source、Cloud semantic input与更新政策，不能另造第二IBL pipeline。
- Runtime09G1拥有Volumetric Fog/Froxel和Local Fog；Weather Fog adapter只产生typed目标参数和generation。
- Runtime09H2拥有Exposure/Post Process；Lightning flash与Weather exposure变化必须通过其typed stack/history合同。
- Editor15拥有Particle/VFX asset/compiler/preview；本篇只定义Precipitation/Wind adapter和Weather驱动的runtime instances。
- Editor16拥有Terrain/Foliage/World Partition；本篇只定义Weather surface/wind/deposition输入与streaming continuity。
- Editor17拥有Sound authoring/runtime bridge；本篇只定义Weather ambience与Thunder typed events。
- Editor22拥有Render/Lighting/Post Process diagnostics；Weather Environment preview和capture应投影共同runtime snapshot。
- Editor28拥有World State/Scenario authority；Weather状态可选择向World State发布typed keys/events，但不能由字符串Scenario反向成为Weather真值。
- Editor37拥有SpatialRegion identity/geometry/index；Weather region binding必须引用它，不再建Mountains/Coast字符串区域表。
- Editor02/03/05/09/11拥有document、gizmo、Inspector、job和diagnostic基础；Weather Editor只能接入这些owner。

## 11. 禁止的临时修补

1. 禁止新增`WeatherState { kind: String, properties: Json }`万能property bag。
2. 禁止保留`Weather_Storm`、固定timeline、8 layers、5 regions和2 warnings作为产品默认数据。
3. 禁止Preview/Build只写`queued`、toast或Output字符串。
4. 禁止把测试fixture中的`weather.Component.CloudLayer`解释为生产实现。
5. 禁止只新增`zircon_plugins/weather`空manifest或capability descriptor就标记功能存在。
6. 禁止用viewport `preview_skybox` bool作为Scene Environment资产。
7. 禁止把gradient加sun disk称为physical atmosphere。
8. 禁止继续让`CaptureCloud`调用同一gradient capture并算作Cloud pass。
9. 禁止以Volumetric Fog参数重命名方式冒充Cloud。
10. 禁止让程序天空sun direction和Scene DirectionalLight继续独立编辑。
11. 禁止用wall clock或全局随机驱动Weather transition和Lightning。
12. 禁止用逐帧浮点累计作为长期calendar/transition authority。
13. 禁止把Weather timeline的`02:00`隐式解释为virtual/real/sequence time。
14. 禁止每个Weather参数微小变化都全量rebake PMREM/SH9。
15. 禁止首次IBL更新无budget塞入一个frame并隐藏尖峰。
16. 禁止只给Particle emitter设置固定gravity或external force就宣称Wind/Precipitation完成。
17. 禁止忽略Particle CPU/GPU external force差异。
18. 禁止每个雨滴创建entity、Physics query、decal或audio voice。
19. 禁止Weather直接修改Terrain heightfield、Material内部buffer或Sound mixer可变状态。
20. 禁止复制Wind参数到Particle/Vegetation/Water/Audio并各自漂移。
21. 禁止用entity/display name作为Region、Weather state、transition或Strike identity。
22. 禁止另建Weather region AABB并绕过Editor37 geometry/generation。
23. 禁止server复制每帧全部Weather float或把客户端VFX结果当Gameplay authority。
24. 禁止save只保存当前preset名而丢失clock、transition、RNG、region和event cursor。
25. 禁止Editor preview直接改authoritative World或Play session。
26. 禁止UI线程读取可变Weather/Render/Particle/Sound service容器。
27. 禁止无界region contributor、weather event、adapter diff、particle impact或diagnostic队列。
28. 禁止在真实GPU、network、save、fault、scale和同画质benchmark未通过前宣称优于当前Unreal。

## 12. 本轮产出边界

本轮只新增审查与重构计划，没有修改production Runtime、Editor、plugin、interface、App代码或tests。静态证据覆盖91个显式文件、22,404行、864,287 bytes、133个test attributes和1个ignored，读取时fingerprint为`7c2378aec5cfff40dea096c2b99cc56b43d873694cba64d0f07de3242fb779ce`。

8个在途文件均非本轮产生，实施前必须重算物理范围并复核其终态。本轮没有运行动态测试；此前Editor lib测试编译仍被239个既有错误/122个warning阻断。后续实现必须从M0 truthfulness和可编译基线开始，先建立source/compiler/artifact、Celestial/Weather单一authority和generation，再逐域接入，不能用静态面板、DayNight脚本、Rain粒子preset或Cloud空pass替代工程化系统。
