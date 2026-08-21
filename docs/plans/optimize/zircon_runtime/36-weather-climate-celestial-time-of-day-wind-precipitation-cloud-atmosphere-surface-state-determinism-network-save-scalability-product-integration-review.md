---
related_code:
  - zircon_runtime/src/core/framework/time
  - zircon_runtime/src/core/runtime/time.rs
  - zircon_runtime/src/asset/assets/scene
  - zircon_runtime/src/scene/components/scene
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/core/framework/render/environment
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/volumetric.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment
  - zircon_plugins/particles/runtime/src
  - zircon_plugins/terrain/runtime/src/plugin.rs
  - zircon_plugins/sound/runtime/src/components.rs
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_weather_editor_workspace.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/world_building/volume_and_weather.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/world_building.rs
  - zircon_plugins/plugin_sdk/src/registration.rs
  - zircon_plugins/plugin_sdk_examples/editor/src/extensions.rs
  - examples/woc/native/apps/woc_client/src/preferences/settings
  - examples/woc/contracts/m4_abilities.json
tests:
  - zircon_runtime/src/core/framework/render/environment/source_cubemap/tests.rs
  - zircon_runtime/src/core/framework/render/environment/source_cubemap_upload/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_graph_plan/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_runtime/tests.rs
  - zircon_plugins/particles/runtime/src/tests
  - zircon_runtime/src/asset/importer/native.rs
  - zircon_runtime/src/plugin/native_plugin_loader/candidate_from_manifest.rs
  - zircon_runtime/src/plugin/native_plugin_loader/discover_load_manifest.rs
  - zircon_runtime/src/plugin/native_plugin_loader/discovery_refresh/work.rs
  - zircon_runtime/src/plugin/native_plugin_loader/host_api_adapter/bridge_scope/tests.rs
  - zircon_runtime/src/scene/ecs/component/registry.rs
  - zircon_runtime/src/scene/world/component_type_registry.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/resources_events.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
  - docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/08b-audio-runtime-review.md
  - docs/plans/optimize/zircon_runtime/09e-direct-lighting-clustered-shadow-review.md
  - docs/plans/optimize/zircon_runtime/09f1-environment-sky-ibl-reflection-probe-review.md
  - docs/plans/optimize/zircon_runtime/09g1-volumetric-fog-froxel-review.md
  - docs/plans/optimize/zircon_runtime/09h2-exposure-color-bloom-dof-motion-blur-ssr-terminal-review.md
  - docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
  - docs/plans/optimize/zircon_runtime/26-particle-vfx-system-emitter-cpu-gpu-simulation-rendering-scalability-determinism-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/29-terrain-landscape-heightfield-quadtree-lod-material-layer-foliage-world-partition-physics-navigation-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/30-water-ocean-lake-river-surface-wave-fft-shallow-water-rendering-underwater-buoyancy-query-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/31-cloth-fabric-soft-body-garment-simulation-collision-deformation-rendering-wind-lod-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/34-vegetation-tree-foliage-grass-species-instancing-wind-animation-billboard-impostor-lod-streaming-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_editor/37-volume-zone-trigger-region-gameplay-audio-post-process-environment-authoring-review.md
  - docs/plans/optimize/zircon_editor/38-weather-climate-time-of-day-wind-precipitation-cloud-atmosphere-environment-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/SkyAtmosphereComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/VolumetricCloudComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/WindDirectionalSourceComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/DirectionalLightComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Engine/WindDirectionalSource.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/SkyAtmosphereRendering.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/VolumetricCloudRendering.cpp
  - dev/UnrealEngine/Engine/Plugins/Experimental/DaySequence/Source/DaySequence/Public/DaySequenceActor.h
  - dev/UnrealEngine/Engine/Plugins/Experimental/DaySequence/Source/DaySequence/Private/DaySequenceActor.cpp
  - dev/UnrealEngine/Engine/Plugins/Experimental/DaySequence/Source/DaySequence/Public/Actors/SunMoonDaySequenceActor.h
  - dev/UnrealEngine/Engine/Plugins/Experimental/DaySequence/Source/DaySequence/Public/Actors/DaySequenceModifierVolume.h
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Sky/VisualEnvironment.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Sky/VisualEnvironment.Migration.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Sky/SkyManager.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Sky/PhysicallyBasedSky/PhysicallyBasedSky.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Sky/CloudSystem/CloudLayer/CloudLayer.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/VolumetricClouds/VolumetricClouds.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/VolumetricClouds/HDRenderPipeline.VolumetricClouds.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/VolumetricClouds/HDRenderPipeline.VolumetricCloudsAccumulation.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/VolumetricClouds/HDRenderPipeline.VolumetricCloudsShadows.cs
  - dev/godot/scene/resources/environment.h
  - dev/godot/scene/resources/environment.cpp
  - dev/godot/scene/resources/sky.h
  - dev/godot/scene/3d/world_environment.h
  - dev/godot/scene/3d/fog_volume.h
  - dev/bevy/crates/bevy_light/src/atmosphere.rs
  - dev/bevy/crates/bevy_pbr/src/atmosphere
  - dev/bevy/crates/bevy_pbr/src/fog.rs
  - dev/Fyrox/fyrox-impl/src/scene/skybox.rs
  - dev/Fyrox/fyrox-impl/src/scene/mod.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 36 · Weather、Climate、Celestial、Time-of-Day、Wind、Precipitation、Cloud、Atmosphere、Surface State、Determinism、Network、Save、Scalability 与 Product Integration 工程化差距

## 1. 结论

Zircon当前没有可执行的Weather/Climate runtime。没有first-party Weather package、capability、source asset、Scene binding、compiler、artifact、World service、state machine、region resolver、snapshot、query ABI或product fixture。Editor38已经把静态Weather Workbench、缺失产品owner、太阳与Environment分裂、Cloud/Wind/Precipitation/Lightning伪表面以及determinism/save/network断链登记为5项P0，并以P1-01..70建立跨runtime/editor父要求。本篇不复制这些顶层阻断，登记 **0个新P0 / 72个runtime子P1 / 16个P2**；Editor38仍是父owner，只有其作者工具门和本篇对应runtime子门同时关闭，父finding才能完成。

可保留基础真实存在，但彼此没有Weather authority。Runtime拥有real/virtual/fixed clock与fixed-step预算，working tree还增加了RuntimeTimeAdvance.virtual_delta；EnvironmentExtract拥有skybox/probe/baked lighting，程序天空新增sun direction/color/intensity/angular radius；realtime IBL有generation与分片调度；Volumetric Fog有typed settings；Particle有CPU/GPU simulation；Terrain、Sound、Material和Scene有各自owner。这些能力只能作为adapter consumer，不能把preview skybox、程序sun、固定particle force、fog density或WOC天气画质开关组合后称为Weather。

当前项目链的决定性断点仍是 zircon_runtime/src/scene/world/render.rs 的 build_environment_extract：它只把viewport settings.preview_skybox布尔值映射为disabled或procedural default。Scene asset/component没有Environment、Atmosphere、Cloud、Weather、Climate、Wind或Precipitation字段；EnvironmentExtract只有skybox、probes、baked_lighting和probe_grid；RenderFrameExtract没有Weather snapshot。Particle CPU只累加asset gravity与asset physics.external_force，GPU params只上传gravity；Terrain、Sound、Material范围对weather/wetness/puddle/snow/humidity/precipitation/temperature/wind的独立单词搜索均为零。

目标链必须是：

Climate/Celestial/Weather/Region source -> deterministic compiler -> immutable WeatherProgramArtifact -> WorldWeatherService(world/program/clock/seed generations) -> WeatherFrameSnapshot -> typed Celestial/Atmosphere/Cloud/IBL/Fog/Wind/Precipitation/Surface/Lightning/Sound/Gameplay adapters -> per-domain execution receipts。Runtime线程不得读取Editor document或字符串property bag；render、particle、sound、terrain和gameplay只消费同代snapshot/adaptation result。

## 2. 审查边界、方法与 currentness

### 2.1 冻结语料

本轮冻结228个文件、70,785行、2,727,838 bytes：135个production文件为28,313行、992,696 bytes；43个focused test文件为11,559行、422,964 bytes；12个产品/控制面证据文件为8,653行、269,676 bytes；38个参考文件为22,260行、1,042,502 bytes。43个focused test文件共有230个test/tokio::test属性和1个ignore。

指纹算法为按forward-slash相对路径排序，逐文件计算小写SHA-256，形成path、TAB、file_sha256行，以单个LF连接且无末尾LF，再对UTF-8 payload计算SHA-256；结果为 cdf27abd370ed3f7dca546f1464f89a22de56eb8b8e282679d1cb549b0800e3f。

冻结基线为 main@25e09a23178000f2e783ce2143cf70a8b118d404，按读取时working bytes计算。所选Zircon范围有22个Git status条目：20个environment/volumetric/frame-extract路径的working hash与HEAD完全相同；zircon_runtime/src/core/runtime/time.rs有其他Session的6行语义diff，增加virtual_delta保存与访问；world_building.rs只有import排序diff。两项都不是本轮产生，本篇不回退或接管。实施前必须重导manifest、重算指纹并复核这些owner。

### 2.2 纵向检查链

本轮按package/capability -> source asset/schema -> compiler/artifact/DDC -> Scene binding -> World lifecycle -> clock/calendar/celestial -> deterministic state/transition/RNG -> region resolve/streaming -> immutable snapshot -> sky/atmosphere/cloud/IBL/fog -> wind/particle/precipitation/surface -> lightning/sound/gameplay -> network/save/replay/headless -> diagnostics/scalability/tests/product evidence逐层读取。

精确词法扫描区分了产品语义与fixture名字。Runtime native loader、component registry和plugin tests大量使用weather/climate作为generic package/type/interface样例，所有首个命中都位于cfg(test)之后；SDK Weather只是扩展窗口示例。WOC的Rain/Lightning是技能名，客户端weather只是0..1画质设置。它们没有Weather source、provider或consumer，不能升级capability。

### 2.3 动态证据边界

本轮是E3 source-level review，没有运行Cargo、Editor、真实GPU、网络双进程、save/replay、headless server或产品窗口。既有zircon_editor --lib lane此前在617.2秒后被239个已有错误和122个warning阻断，当前源码没有解除该条件，因此没有重复同一已知失败lane。源码足以证明owner与数据链缺失，但未来云画质、确定性、网络纠正和性能必须由实现后的动态receipt证明。

## 3. 当前可保留的真实基础

1. Runtime20的real/virtual/fixed clock、pause/scale、fixed-step plan和overstep budget可作为simulation clock输入；Celestial calendar必须是独立domain，不污染通用Time类型。
2. Runtime05的World generation、dynamic component registry、transactional spawn方向可承载Weather service安装和Scene binding；generic fixture不能代替真实schema。
3. Runtime09F1的EnvironmentExtract、source cubemap、PMREM/SH9、realtime IBL generation和last-good方向可作为Weather environment adapter consumer。
4. 程序天空已具sun direction/color/intensity/angular radius与stable bake key，可作为Low/Artistic fallback；它不是physical atmosphere或celestial authority。
5. Runtime09G1的typed VolumetricFogSettings、local volume与history基础可接受humidity/aerosol目标；Weather不得直接写froxel资源。
6. Runtime26的particle service、CPU/GPU backend、stable handle和snapshot可承载降水实例；必须先增加runtime force/provider与CPU-GPU parity。
7. Terrain、Vegetation、Cloth、Water、Material、Sound和Gameplay已有独立owner报告，可通过adapter消费wind/deposition/temperature/event，Weather不反向拥有其内部资源。
8. Editor37已有SpatialRegion目标边界，Weather region只能引用stable compiled region identity，不另建字符串Mountains/Coast表。

## 4. 当前代码事实与断路

| 层级 | 当前事实 | 工程断路 |
|---|---|---|
| package/catalog | zircon_plugins无weather/climate目录，首方runtime/editor catalog无对应ID | 无安装、版本、profile、capability与产品装配真相 |
| source/schema | Scene asset只有camera/light/mesh/physics/post-process等已知字段 | 无Climate/Weather/Celestial/Region source与unknown roundtrip |
| environment extract | EnvironmentExtract只有skybox/probes/baked lighting/probe grid | 无atmosphere/cloud/weather generation、dirty mask或history disposition |
| Scene-to-render | build_environment_extract只读取preview_skybox bool | 普通项目无法保存、重开、cook或运行environment/weather |
| sky/sun | ProceduralSkyParams有独立sun，DirectionalLight另有Scene identity | disk、light、shadow、atmosphere、IBL不共享CelestialSample |
| time | RuntimeTimeClocks只有real/virtual/fixed duration与frame/fixed plan | 无calendar、epoch、day length、geography、celestial tick或jump policy |
| cloud | SkyboxMode只有Disabled/ProceduralGradient/SourceCubemap | 无cloud medium/source/material/work/history/shadow；旧CaptureCloud仍由09F1治理 |
| fog | typed density/albedo/phase/height/temporal | 无humidity/aerosol adapter和Weather generation |
| wind/particle | CPU读取asset gravity+external_force，GPU只上传gravity | 无WindField、runtime provider、空间query或CPU-GPU一致性 |
| surface | Terrain/Sound/Material关键词审计为零 | 无wetness/puddle/snow/deposition、thunder或ambience消费 |
| state/network/save | 无Weather service、state graph、RNG、region resolver、codec | 无authority、late join、replay、headless parity或persistence |
| product | 静态Weather Workbench、SDK窗口、WOC画质/技能词汇 | 无save-play-export-capture、双进程或规模证据 |

## 5. 唯一 Owner、父子 Finding 与目标合同

Editor38继续拥有P0-1..5和P1-01..70父要求，其中P1-65..70是Editor document/compiler job/preview/diagnostic作者工具面。本篇只拥有runtime executable child；它不重复Runtime09F1天空/IBL算法、09G1 fog、20通用时间、26粒子内核、29/30/31/34各domain实现。

推荐owner形状为：zircon_runtime只定义稳定的WeatherProgramArtifact、WeatherFrameSnapshot、query/adapter ABI与World生命周期合同；首方zircon_plugins/weather/runtime实现compiler/service/transition/region resolve和domain adapters，zircon_plugins/weather/editor消费同一schema/compiler。最终路径可在M0架构评审中调整，但不得把feature实现塞进Scene render.rs、EnvironmentExtract构造器或Editor callback。

每个snapshot至少绑定WorldGeneration、ProgramDigest、WeatherTick、ClockGeneration、ClimateGeneration、CelestialGeneration、RegionGeneration、StateId、TransitionId、RngCursor、EventSequence和AdapterDirtyMask。每个consumer回报Accepted/Degraded/Unsupported/Stale/Failed及source/snapshot/consumer generations，禁止静默成功。

## 6. P1：Package、Source、Schema、Compiler 与 Artifact

| ID | 差距 | 重构要求 |
|---|---|---|
| WTH-P1-001 | 无唯一package/module/capability identity | 定义runtime/editor/client/server IDs、dependency、version和maturity，不从UI或fixture推导Ready |
| WTH-P1-002 | 首方catalog无Weather | catalog只在provider、artifact install、consumer和qualification同代时升级能力 |
| WTH-P1-003 | 无Climate/Celestial/Weather/Region asset kinds | 注册versioned source schema、asset refs、dependency collector和type presentation |
| WTH-P1-004 | 字段无stable property ID | field ID、unit、range、enum、default、nullable和unknown policy成为唯一schema |
| WTH-P1-005 | 无physical/finite validation | 经纬海拔、速度、温度、湿度、概率、duration与curve在compile前fail-close |
| WTH-P1-006 | 无source dependency manifest | profile、curve、texture/noise、region、VFX、sound、material依赖进入BuildSet |
| WTH-P1-007 | 无deterministic compiler | 相同source/dependency/toolchain/target产生相同program tables、diagnostics和digest |
| WTH-P1-008 | 无WeatherProgramArtifact | artifact携schema/compiler版本、state/edge/curve/region tables、seed policy和adapter declarations |
| WTH-P1-009 | 无DDC/LKG/publication | compile失败保留同源last-good，stale/rollback/cache disposition可观察 |
| WTH-P1-010 | Scene无stable Weather binding | 保存program/profile refs、clock source、region bindings、override与fallback，支持prefab/unknown roundtrip |
| WTH-P1-011 | 无migration/downgrade | 每类source/artifact独立version，输出loss/backup/rollback report，不静默丢字段 |
| WTH-P1-012 | 无target admission | EditorHost/Client/DedicatedServer按provider/consumer/capability选择Supported/Degraded/Unsupported |

## 7. P1：Clock、Calendar、Climate 与 Celestial

| ID | 差距 | 重构要求 |
|---|---|---|
| WTH-P1-013 | 通用Time没有celestial语义 | 建独立CelestialClock，以integer tick保存authority，render只消费插值sample |
| WTH-P1-014 | program未声明clock | 明确simulation/virtual/celestial/sequence/preview clock、pause、scale和correction政策 |
| WTH-P1-015 | 无calendar与epoch | 定义day/year/season、epoch、timezone语义、rounding、overflow和codec |
| WTH-P1-016 | 无day-length/cycle contract | source指定logical day length与real-time cycle，禁止隐式24h或wall clock |
| WTH-P1-017 | 无geography/space约定 | 纬度、经度、海拔、北向、up-axis、planet center与large-world origin分离 |
| WTH-P1-018 | 无ephemeris/曲线精度合同 | 可复现算法或项目曲线有适用范围、单位、reference dataset和误差门 |
| WTH-P1-019 | 无celestial stable identity | Sun/Moon/其它天体使用stable ID、role、disk、light与visibility policy |
| WTH-P1-020 | DirectionalLight无atmosphere binding | typed binding声明celestial role/index、illuminance、temperature、angular radius和shadow |
| WTH-P1-021 | sun disk与light可漂移 | 同一CelestialSample生成direction/disk/photometry/atmosphere invalidate key |
| WTH-P1-022 | 无Climate慢变量 | ClimateProfile按calendar/region产生temperature/humidity/wind envelope与allowed weather |
| WTH-P1-023 | jump/scrub无history disposition | 输出Continuous/Jump/Correction，cloud/fog/exposure/TAA/IBL逐域reset/reproject |
| WTH-P1-024 | pause/scale长期累计不确定 | authority以tick和rational policy推进，暂停、快进、回放和preview有确定结果 |

## 8. P1：World Authority、Transition、Region、Lifecycle、Network 与 Save

| ID | 差距 | 重构要求 |
|---|---|---|
| WTH-P1-025 | 无WorldWeatherService | 每World唯一owner，安装program/clock/region并发布immutable snapshot |
| WTH-P1-026 | 无service lifecycle | Requested/Preparing/Active/Suspended/Retiring/Failed/Cancelled每ticket唯一终态 |
| WTH-P1-027 | 无deterministic RNG streams | program/world/region/state/transition/strike identity派生流，禁止global RNG/wall clock |
| WTH-P1-028 | 无transition graph semantics | state/edge/condition/duration/hysteresis/cooldown/priority/tie-break在artifact中固定 |
| WTH-P1-029 | 无stable state/transition identity | reload/save/network/replay通过ID与generation迁移，拒绝display name authority |
| WTH-P1-030 | region dropdown无runtime geometry | 只消费Editor37 SpatialRegionId、compiled geometry/index/generation |
| WTH-P1-031 | nested contributors无组合政策 | 每字段声明override/add/multiply/min/max/normalized blend和stable tie-break |
| WTH-P1-032 | streaming导致状态断裂 | cell add/remove保留authority/history/event cursor，late result不能复活旧region |
| WTH-P1-033 | 无原子snapshot publish | state、clock、region、events与adapter dirty set在单一generation commit point冻结 |
| WTH-P1-034 | 无server authority/replication | 复制program digest/tick/state/transition/seed/correction，不逐帧复制全部float |
| WTH-P1-035 | 无save/load/replay | 持久化clock/state/progress/RNG/regions/event sequence，digest mismatch有migration或失败 |
| WTH-P1-036 | 无bounded query/command ABI | snapshot query、override、forecast和event cursor有limit/generation/timeout/disposition |

## 9. P1：Environment、Atmosphere、Cloud、IBL、Fog 与 Multi-View Adapter

| ID | 差距 | 重构要求 |
|---|---|---|
| WTH-P1-037 | EnvironmentExtract无Weather identity | 通过typed EnvironmentAdapterSnapshot携同代celestial/atmosphere/cloud/fog generations |
| WTH-P1-038 | preview bool冒充Scene environment | product Scene读取source binding；gradient只作为显式Low/Preview fallback |
| WTH-P1-039 | 无physical atmosphere adapter | 输出planet/ground/Rayleigh/Mie/absorption/aerial参数给09F1，不复制LUT owner |
| WTH-P1-040 | 无atmosphere dirty mask | 区分profile、sun angle、aerosol、camera altitude、quality与device invalidation |
| WTH-P1-041 | 无Cloud source/runtime payload | 编译layer/coverage/density/weather map/noise/material/wind binding和resource dependencies |
| WTH-P1-042 | 无Cloud work/quality policy | view/light/reflection sample、resolution、distance、early exit和fallback受联合预算 |
| WTH-P1-043 | 无Cloud lighting/shadow adapter | 消费Celestial/Atmosphere/IBL并发布cloud transmittance/ambient/shadow generation |
| WTH-P1-044 | 无Cloud temporal contract | history绑定view/cloud/weather/quality generations并处理motion、jump、cut和ghost rejection |
| WTH-P1-045 | Weather变化会无差别触发IBL | domain dirty/cadence/threshold决定capture，last-good/crossfade/cancel受09F1预算 |
| WTH-P1-046 | Fog与Weather分离 | 只向09G1输出humidity/aerosol/visibility目标和generation，不直接写froxel |
| WTH-P1-047 | Exposure/lightning无共同事件 | lightning impulse、cloud occlusion和day transition通过typed 09H2 adapter/history disposition |
| WTH-P1-048 | 无multi-view/capture政策 | camera、stereo、reflection、scene capture、preview各有region/sample/history/resource sharing规则 |

## 10. P1：Wind、Precipitation、Surface、Lightning、Sound 与 Gameplay Adapter

| ID | 差距 | 重构要求 |
|---|---|---|
| WTH-P1-049 | 无Wind source/contributors | global/local/directional/gust/turbulence/altitude profile使用stable contributor IDs |
| WTH-P1-050 | 无WindField snapshot/query | bounded grid/clipmap或analytic set支持position/height/time query、LOD和overflow |
| WTH-P1-051 | 各域可能复制风参数 | Particle/Vegetation/Cloth/Water/Audio消费同一wind generation的filtered sample |
| WTH-P1-052 | Particle CPU force为资产常量 | Runtime26增加per-instance force provider，asset baseline与Weather sample分离 |
| WTH-P1-053 | Particle GPU不消费external force | GPU program编码同一provider contract，unsupported时显式degrade并做CPU oracle parity |
| WTH-P1-054 | 无Rain/Snow/Hail schema与view volume | phase/intensity/size/fall speed/temperature/budget驱动camera-relative bounded instances |
| WTH-P1-055 | 无surface impact batching | depth/heightfield/physics batch输出splash/ripple/decal/audio requests和drop receipt |
| WTH-P1-056 | 无wetness/puddle/snow state | SurfaceWeatherState按cell/material channel持久化deposition/evaporation/melt |
| WTH-P1-057 | 无跨domain surface adapter | Terrain/Material/Vegetation/Water/Cloth只消费typed diff，不被Weather直接写内部buffer |
| WTH-P1-058 | 无deterministic Lightning identity | StrikeId由seed/tick/region/candidate/authority sequence生成，视觉与gameplay共用 |
| WTH-P1-059 | 无Thunder/ambience schedule | 根据strike/listener/speed-of-sound和occlusion调度Sound event，支持late join/pause |
| WTH-P1-060 | 无Gameplay/AI/Nav/indoor contract | server发布typed hazard/visibility/cover事件，cosmetic与authority分离 |

## 11. P1：Scalability、Reliability、Diagnostics、Tests 与 Product Qualification

| ID | 差距 | 重构要求 |
|---|---|---|
| WTH-P1-061 | 无联合budget/admission | regions/events/cloud/IBL/wind/particles/impacts/surface/audio CPU/GPU/memory统一决策 |
| WTH-P1-062 | snapshot/queue无边界 | contributor/event/diff/diagnostic count、bytes、age、time有hard limit与overflow终态 |
| WTH-P1-063 | 无large-world/multi-world隔离 | 每World state/generation/RNG/resource独立，按cell demand增量resolve |
| WTH-P1-064 | 无async cancellation/unload drain | compile/stream/cloud/IBL/particle/sound work按owner generation撤销并拒绝late publish |
| WTH-P1-065 | 无fault/device-loss/headless政策 | GPU缺失保留deterministic authority，adapter降级可见，device restore不改变gameplay state |
| WTH-P1-066 | 无runtime diagnostics | 报告program/tick/state/region/RNG/events/adapter dirty/budget/degrade/stale/failure |
| WTH-P1-067 | 无schema/compiler golden | validation、migration、dependency、digest、bad graph、cycle与LKG测试为空 |
| WTH-P1-068 | 无deterministic simulation matrix | pause/scale/jump/rewind、1/1k regions、thread schedule、reload重复运行hash一致 |
| WTH-P1-069 | 无network/save/replay tests | late join/correction/digest skew/rollback/duplicate event/restart恢复为空 |
| WTH-P1-070 | 无adapter CPU/GPU/pixel tests | celestial/atmosphere/cloud/fog/wind/particle/surface/lightning逐代oracle为空 |
| WTH-P1-071 | 无真实产品fixture | 建clear-day/storm/snow/front/region-crossing/lightning六类save-play-export-capture场景 |
| WTH-P1-072 | 无竞争性基准 | 同场景/硬件/画质对照Unreal/Unity的CPU/GPU/RAM/VRAM/stutter/image error raw receipt |

## 12. P2：高阶能力

| ID | 高阶能力 | 进入条件 |
|---|---|---|
| WTH-P2-001 | spectral multi-layer atmosphere | RGB physical baseline、reference oracle和预算完成 |
| WTH-P2-002 | multi-planet/multi-star celestial | single-planet Sun/Moon identity、space/precision和lighting contract完成 |
| WTH-P2-003 | physically coupled climate solver | deterministic profile/state baseline及离线reference dataset完成 |
| WTH-P2-004 | data-assimilated forecast/import | source provenance、license、schema migration和offline fallback完成 |
| WTH-P2-005 | volumetric storm cell dynamics | cloud medium、wind field、lightning与scale receipts完成 |
| WTH-P2-006 | tornado/hurricane structured flow | WindField query、physics/particle/vegetation adapters和safety budget完成 |
| WTH-P2-007 | hydrology/puddle flow coupling | Terrain/Water/SurfaceState stable cell identity与persistence完成 |
| WTH-P2-008 | deformable persistent snow | material/terrain/character interaction、save/network和streaming完成 |
| WTH-P2-009 | ocean-atmosphere energy coupling | Runtime30 wave/wind/temperature adapters与deterministic exchange完成 |
| WTH-P2-010 | cloud-ground electrical field | StrikeId、gameplay authority、visual/audio timing和reference模型完成 |
| WTH-P2-011 | urban canopy/microclimate | region/material/geometry inputs有bounded acceleration和validation |
| WTH-P2-012 | forecast-driven AI/gameplay planning | typed forecast uncertainty、authority和AI consumer budget完成 |
| WTH-P2-013 | cross-server weather fronts | shard clock/region ownership、handoff、dedup和reconciliation完成 |
| WTH-P2-014 | authoritative rollback prediction | network/save/replay baseline与deterministic adapter event contract完成 |
| WTH-P2-015 | third-party Weather provider SDK | ABI/version/trust/budget/unload/artifact compatibility完成 |
| WTH-P2-016 | distributed visual/performance qualification farm | frozen BuildSet、GPU/driver matrix、capture/diff/raw receipt与promotion完成 |

## 13. 参考引擎差异矩阵

| 参考 | 可复用工程事实 | Zircon当前差异 | 适用边界 |
|---|---|---|---|
| Unreal | SkyAtmosphere/VolumetricCloud/Wind是可持久化组件并创建scene proxy；物理单位、材质、trace/shadow/scalability字段完整；DaySequence有day length、time-per-cycle、preview、modifier volume、replicated playback | Zircon无组件/source/service，sun与light分裂，wind/cloud/sequence均无runtime | 学组件生命周期、proxy、DaySequence与网络表面；不复制UObject/RDG结构 |
| Unity HDRP | VisualEnvironment以VolumeParameter选择sky/cloud/ambient；WindParameter类型化；VolumetricClouds含map/shape/erosion/local/wind/temporal/shadow；SkyManager有per-camera context、hash/update与ambient probe | Zircon只有bool gradient、固定IBL计划，无cloud history/wind parameter/environment hash | 学typed volume、per-camera generation、cloud history；不把Unity Volume当Weather simulation |
| Godot | WorldEnvironment持有Environment资源，Environment统一sky/reflection/fog，FogVolume是持久Scene node | Zircon Scene无Environment引用，render extract靠preview bool | 作为资源/Scene闭环下限；Godot没有完整deterministic climate authority |
| Bevy | Atmosphere是Component，camera有AtmosphereSettings；ExtractSchedule同步render world，GPU capability不足显式警告；LUT与WGSL链真实 | Zircon无Atmosphere component/extract/capability disposition | 学ECS extract与fail-visible capability；不把nearest-atmosphere规则直接当region policy |
| Fyrox | SkyBox具Reflect/Visit/UUID、texture validation与scene persistence | Zircon程序天空不在Scene asset | 作为Rust资源/序列化下限；其skybox不代表weather/cloud上限 |

五套参考中没有一套同时提供Zircon目标所需的deterministic climate、regional transition、network/save和所有render/gameplay adapter。实现必须组合其成熟边界，并由Zircon自己的artifact/snapshot/receipt合同补足，不能以“参考引擎也没有统一Weather”降低目标。

## 14. 分层重构里程碑

### M0 · Truth、Owner 与 Parent Closure

冻结Editor38父finding、Runtime36子finding和228项manifest；Weather UI在真实provider前保持Unavailable/Prototype。决定core neutral contract与first-party plugin owner，不创建空package。

### M1 · Source、Schema、Compiler 与 Artifact

完成WTH-P1-001..012：四类source、stable schema、dependency graph、deterministic compiler、artifact/DDC/LKG、Scene binding与target admission。

### M2 · Clock、Climate 与 Celestial

完成WTH-P1-013..024：integer celestial tick、calendar/geography/ephemeris、Sun/Moon identity、DirectionalLight binding、Climate envelope与jump history。

### M3 · World Service、Transition 与 Region

完成WTH-P1-025..036：World lifecycle、RNG streams、compiled transition、SpatialRegion resolve、atomic snapshot、network/save/replay和query ABI。

### M4 · Environment、Atmosphere 与 Cloud

完成WTH-P1-037..048：Scene environment、physical atmosphere adapter、cloud payload/work/lighting/history、IBL dirty cadence、fog/exposure与multi-view。09F1/09G1保持算法owner。

### M5 · Wind、Precipitation 与 Surface

完成WTH-P1-049..060：WindField、particle CPU/GPU provider、precipitation instances、impact batch、SurfaceWeatherState、Lightning/Thunder和Gameplay events。

### M6 · Reliability、Scale 与 Product

完成WTH-P1-061..071：联合预算、bounded queues、multi-world、unload/device loss/headless、diagnostics、compiler/simulation/network/GPU测试和六类产品fixture。

### M7 · Competitive Qualification

完成WTH-P1-072：在相同资产、天气序列、相机、画质、硬件、warmup和采样窗口下对照Unreal/Unity，归档可复跑raw receipt；不能只比较截图或平均FPS。

## 15. 验收门

| Gate | 必须证明 |
|---|---|
| WTH-G01 | Weather capability不从静态UI、SDK示例、fixture名称或画质开关推导 |
| WTH-G02 | package/source/compiler/artifact/service/consumer/qualification同代才可标Ready |
| WTH-G03 | source roundtrip、unknown字段、migration/downgrade和rollback无静默数据丢失 |
| WTH-G04 | 相同source/dependency/toolchain/target重复compile digest一致 |
| WTH-G05 | compile失败保留同源LKG且stale/publish disposition可观察 |
| WTH-G06 | Scene save/reopen/prefab/cook保留Weather refs、clock和region bindings |
| WTH-G07 | Celestial authority使用integer tick，长时、暂停、缩放和回放无浮点漂移 |
| WTH-G08 | ephemeris/项目曲线在批准范围内通过reference误差门 |
| WTH-G09 | sun disk、DirectionalLight、atmosphere和shadow消费同一CelestialSample |
| WTH-G10 | jump/scrub/correction向所有history consumer发布一致disposition |
| WTH-G11 | transition RNG在重启、线程调度、平台和late join后结果一致 |
| WTH-G12 | state/edge/contributor排序有stable ID tie-break |
| WTH-G13 | SpatialRegion generation变化不会重复事件、丢state或接受stale result |
| WTH-G14 | snapshot在单一commit point冻结clock/state/regions/events/dirty set |
| WTH-G15 | plugin/world unload后旧task、adapter和resource不能复活 |
| WTH-G16 | server复制state/tick/seed/correction而非每帧全量浮点 |
| WTH-G17 | save/load/replay恢复program digest、RNG cursor和event sequence |
| WTH-G18 | headless与client产生相同authoritative weather/gameplay events |
| WTH-G19 | project Scene不再以preview_skybox bool作为Environment authority |
| WTH-G20 | gradient明确标Low/Preview，physical atmosphere缺失时不冒充成功 |
| WTH-G21 | atmosphere/cloud/fog generations与Weather snapshot同代且stale拒绝 |
| WTH-G22 | cloud disabled调度零工作，enabled必须采样真实cloud medium/radiance |
| WTH-G23 | cloud history在motion/jump/cut/quality/region变化时正确reset或reproject |
| WTH-G24 | IBL dirty mask/cadence避免每个Weather微变全量rebake |
| WTH-G25 | multi-view/reflection/preview不共享错误region或history generation |
| WTH-G26 | WindField CPU oracle与GPU/consumer samples在误差内一致 |
| WTH-G27 | Particle CPU/GPU消费相同weather force合同或明确Unsupported |
| WTH-G28 | precipitation按view有hard instance/pixel/impact budget和drop receipt |
| WTH-G29 | surface deposition按stable cell/material channel持久化并可stream |
| WTH-G30 | Terrain/Material/Vegetation/Water/Cloth不被Weather直接写私有buffer |
| WTH-G31 | bolt/light/cloud/exposure/thunder/gameplay共享同一StrikeId |
| WTH-G32 | thunder delay、listener切换、pause与late join通过时序oracle |
| WTH-G33 | all queues/snapshots/diagnostics有count/bytes/age/time上限 |
| WTH-G34 | device loss只降级visual adapters，不改变authority state |
| WTH-G35 | malformed source、cycle、missing asset、OOM、overflow、unload、skew矩阵通过 |
| WTH-G36 | 六类fixture通过save/reopen/play/export/capture和frame/state inspection |
| WTH-G37 | 1/1k/100k regions及storm峰值有CPU/GPU/RAM/VRAM/stutter raw receipts |
| WTH-G38 | Editor38父项只有在对应Runtime36子门和Editor authoring门均关闭后完成 |
| WTH-G39 | source/build/device/driver变化使旧accepted visual/perf receipt自动过期 |
| WTH-G40 | 优于Unreal/Unity的结论绑定同场景/序列/硬件/画质和可复跑raw receipt |

## 16. Finding 到里程碑与父项映射

| Runtime36 finding | 里程碑 | Editor38父范围 |
|---|---|---|
| WTH-P1-001..012 | M0-M1 | P0-1..2、P1-01..10 |
| WTH-P1-013..024 | M2 | P0-3、P1-11..22 |
| WTH-P1-025..036 | M3 | P0-5、P1-55..64 |
| WTH-P1-037..048 | M4 | P0-3..4、P1-23..38 |
| WTH-P1-049..060 | M5 | P0-4、P1-39..54 |
| WTH-P1-061..072 | M6-M7 | P0-2..5、P1-08、P1-63..64、P1-70 |
| WTH-P2-001..016 | 独立后续 | 对应父P2与基础门完成后立项 |

## 17. 禁止的临时修补

1. 禁止新增WeatherState { kind: String, properties: Json }万能property bag。
2. 禁止只建zircon_plugins/weather空manifest、descriptor或catalog行。
3. 禁止继续让preview_skybox bool承担Scene Environment。
4. 禁止把gradient sun、fog density或CaptureCloud命名当作Weather。
5. 禁止用wall clock、global RNG或逐帧float累计驱动authority。
6. 禁止程序天空sun与DirectionalLight独立编辑。
7. 禁止Weather直接写render texture、particle pool、terrain heightfield、material buffer或sound mixer。
8. 禁止每个雨滴创建entity、physics query、decal或audio voice。
9. 禁止复制一套Weather Region AABB绕过Editor37。
10. 禁止server逐帧复制所有visual float或客户端VFX反向成为gameplay truth。
11. 禁止save只保存preset名字而丢clock、transition、RNG、region和event cursor。
12. 禁止用registration/unit/source-string测试替代GPU、双进程、save和产品证据。

## 18. 实施文件与职责蓝图

| Owner候选 | 应新增/重构 | 不应放入 |
|---|---|---|
| zircon_runtime neutral contract | artifact/snapshot/query/adapter IDs、World lifecycle与receipt DTO | weather算法、UI状态、GPU资源 |
| zircon_plugins/weather/runtime | source/compiler/service/transition/region resolve与adapter orchestration | 私有Time、Scene、IBL、particle、sound owner |
| zircon_plugins/weather/editor | document/toolkit/preview/build/diagnostic projection | 第二份schema/compiler或mock authority |
| Runtime20/05 | clock/fixed-step与World install/publish primitives | Celestial/Weather domain policy |
| Runtime09F1/09G1/09H2 | atmosphere/cloud/IBL/fog/exposure真实consumer | Weather state machine |
| Runtime26/29/30/31/34/08B | particle/terrain/water/cloth/vegetation/sound typed adapters | Weather source authority |
| Editor37/38 | SpatialRegion authoring与Weather parent/product workflow | runtime mutable service |

M0必须先用architecture decision record固定最终crate/module路径、public ABI、feature ownership和hard-cutover顺序；本表是职责约束，不授权在现有巨型文件里就地堆实现。

## 19. 本轮产出边界

本轮只新增review/refactor计划，没有修改production Runtime、Editor、Plugin、App、Interface、Hub、tests、Cargo manifest或workflow。没有运行动态验证，也没有把既有239-error Editor compile、静态Workbench、generic Weather fixture、WOC技能名、路径存在或参考引擎功能当作Zircon完成证据。

退出本审查阶段只表示Runtime Weather/Climate执行链的差异、owner、72个子P1、16个P2、40个gate和父子关闭关系已登记；implementation_status仍为pending。实施顺序必须先关闭truth/source/compiler/authority，再接domain adapters，最后做产品、规模与竞争性资格，不能从Rain particle preset或DayNight脚本开始堆临时功能。
