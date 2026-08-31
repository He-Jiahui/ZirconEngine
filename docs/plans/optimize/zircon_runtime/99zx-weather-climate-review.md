---
title: Runtime Weather、Climate、Celestial、Time-of-Day、Wind、Precipitation、Cloud、Atmosphere、Surface State、Determinism、Network、Save、Scalability、Editor 与 Product Integration 当前源码工程化差距
report_id: Runtime149
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
historical_refresh_of: Runtime36
related_code:
  - zircon_runtime/src/builtin/runtime_modules
  - zircon_runtime/src/plugin/runtime_plugin
  - zircon_runtime/src/core/runtime/time.rs
  - zircon_runtime/src/scene/world_time
  - zircon_runtime/src/scene/world
  - zircon_runtime/src/scene/dynamic_scene
  - zircon_runtime/src/core/framework/scene/component_type_descriptor
  - zircon_runtime/src/asset/assets/scene
  - zircon_runtime/src/core/framework/render/environment
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment
  - zircon_plugins/rendering/features/volumetric_fog
  - zircon_plugins/particles
  - zircon_plugins/terrain
  - zircon_plugins/sound
  - zircon_plugins/net
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_weather_editor_workspace.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/36-weather-climate-celestial-time-of-day-wind-precipitation-cloud-atmosphere-surface-state-determinism-network-save-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_editor/38-weather-climate-time-of-day-wind-precipitation-cloud-atmosphere-environment-authoring-review.md
  - docs/plans/optimize/zircon_runtime/96-runtime-environment-sky-atmosphere-cloud-ibl-reflection-probe-capture-convolution-sh-pmrem-cache-residency-submission-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99-runtime-volumetric-fog-froxel-local-fog-volume-lighting-shadow-history-temporal-reprojection-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99j-runtime-scene-world-level-registry-lifecycle-project-io-snapshot-clone-serialization-schema-transaction-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zd-runtime-save-game-checkpoint-slot-participant-capture-serialization-migration-platform-cloud-async-network-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99ze-runtime-operation-service-handler-registry-admission-prepare-apply-progress-cancel-deadline-harvest-retention-shutdown-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zo-runtime-network-transport-socket-tls-http-websocket-reliable-udp-session-rpc-replication-prediction-rollback-content-download-editor-product-integration-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/SkyAtmosphereComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/VolumetricCloudComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/WindDirectionalSourceComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/WindDirectionalSource.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/SkyAtmosphereRendering.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/VolumetricCloudRendering.cpp
  - dev/UnrealEngine/Engine/Plugins/Experimental/DaySequence/Source/DaySequence
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Sky
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/VolumetricClouds
  - dev/Graphics/Tests/SRPTests/Projects/HDRP_Tests/Assets/GraphicTests/Scenes/5x_SkyAndFog
  - dev/godot/scene/resources/environment.cpp
  - dev/godot/scene/3d/world_environment.cpp
  - dev/godot/scene/3d/fog_volume.cpp
  - dev/godot/tests/scene/test_sky.cpp
  - dev/bevy/crates/bevy_light/src/atmosphere.rs
  - dev/bevy/crates/bevy_pbr/src/atmosphere
  - dev/Fyrox/fyrox-impl/src/scene/skybox.rs
---

# Runtime Weather、Climate、Celestial 与 Product Integration 当前源码工程化差距

## 1. 结论

当前 Zircon **没有 Weather/Climate runtime 产品**。没有首方 Weather package、builtin ID、capability、source kind、compiler、`WeatherProgramArtifact`、`WorldWeatherService`、`WeatherFrameSnapshot`、`CelestialClock`、`WindField`、`SurfaceWeatherState`、`StrikeId`、Scene binding、网络 codec、SaveGame payload或产品fixture。`RuntimePluginId`已经可以承载动态外部ID，plugin/asset/compiler/World/operation/network/save/render基础也有可复用部分；这说明可以建设正式系统，不说明 Weather 已经存在。

当前最危险的问题不是“算法还不够漂亮”，而是多个临时表面容易被误读成实现：Weather Workbench把 `Weather_Storm`、Mountains、Cloud Build、Rain Burst、Wind Gust、Lightning、`8 layers / 2 warnings` 和 Preview/Build 成功文案硬编码在ZUI/callback中；`WeatherQueryInterface::temperature()`、native loader里的weather/climate manifest、`weather.Component.CloudLayer`与`plugin.weather`都位于测试fixture；WOC的weather值只是客户端画质偏好，Rain/Lightning其余命中是技能名。它们都没有source -> compiler -> artifact -> World authority -> consumer receipt链。

渲染侧也不是 Weather authority。`World::build_environment_extract`仍只把`preview_skybox`布尔值映射为默认程序天空；`EnvironmentExtract`只有skybox/probes/baked lighting/probe grid，`RenderFrameExtract`没有Weather snapshot。程序天空虽有sun direction/color/intensity/angular radius并能驱动真实IBL，但它只是艺术化gradient source；DirectionalLight、太阳圆盘、物理大气、Cloud与Time-of-Day没有共同真值。Volumetric Fog是可保留的真实froxel/temporal基础，Particle CPU仅消费asset-local gravity + `external_force`，GPU只消费gravity；Terrain、Material、Physics、Sound与Net生产路径没有Weather typed adapter。

历史Editor38关于`CaptureCloud`重复执行gradient的具体事实已经过时：当前生产代码中`CaptureCloud/CAPTURE_CLOUD`为0，realtime IBL graph test还明确断言两者不得出现。该子问题视为已修复的历史证据，但Cloud medium、Weather map、wind binding、lighting/shadow/history与Weather authority仍全部缺失，因此Editor38 P0-4不能关闭。

历史Runtime36的72项P1按当前working bytes重判为 **27 Open / 45 Partial / 0 Closed**；16项P2全部Open；40项资格门为 **23 Fail / 17 Partial / 0 Pass**。Partial只表示通用clock、artifact、World、operation、network/save、render graph、IBL、fog、particle、budget或diagnostic基础可复用，不表示Weather领域合同已贯通。目标必须硬切到：

```text
Climate/Celestial/Weather/Region Sources
  -> deterministic WeatherCompiler + dependency/target admission
  -> immutable WeatherProgramArtifact
  -> per-World WorldWeatherService(program/clock/seed/region generations)
  -> atomic WeatherFrameSnapshot + bounded query/event cursor
  -> typed Celestial/Atmosphere/Cloud/IBL/Fog/Wind/Precipitation/
     Surface/Lightning/Sound/Gameplay adapters
  -> generation-qualified execution/fallback/overflow/terminal receipts
```

## 2. 审查边界、方法与 currentness

### 2.1 冻结范围

本文记录读取时 `main@1b2684b40ae3eba7abfcdfae3fe7e341b4906ec8` 的selected working bytes。冻结附近共享工作树有 **3,359个tracked changes、2,152个untracked paths**；这些改动属于用户或其他Session，本文未归因、覆盖或回退。用户已明确暂不优化tooling，本轮没有扫描或规划未来将迁移到Rust的tooling实现。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 工作树指纹 |
|---|---:|---|
| Zircon package、authoring与product truth | **650 / 47,960 / 43,821 / 1,805,534 / 567 / 0** | `35f377f416db523b750c638cd1b36b00445bb724bc6295d7dfbba1215c301f56` |
| Zircon time、World、Net、Save与Operation基础 | **903 / 62,588 / 57,132 / 2,265,559 / 301 / 0** | `5cb9809903efc35b417f4cf5d04a20140dff223b3e354cdfa3b020effe957678` |
| Zircon environment、render与domain adapter基础 | **2,421 / 211,374 / 194,698 / 7,768,482 / 2,116 / 3** | `7d9bc0b4beed08dea8aa511ed4852c6c85840b6146269bf86019e519586f40d0` |
| Unreal Atmosphere、Cloud、Wind与DaySequence | **63 / 20,472 / 16,809 / 833,095 / 0 / 0** | `075583579a19c61c24e49fa1e14b499c0e208f1b90295fd4c70f56f9bdfc3a70` |
| Unity HDRP Environment、Sky、Cloud与图形测试场景 | **70 / 67,504 / 65,088 / 2,164,301 / 0 / 0** | `976df8e0103fb942d739e353c0ea4274ebcbd24dce049710f4da99688d8a70cc` |
| Godot Environment、Sky、Fog与test | **11 / 5,131 / 4,280 / 212,239 / 9 / 0** | `0f765d50388c3fe0c828a6f18674b6a6870ee1eb0fca805171e4364334b4c86e` |
| Bevy Atmosphere与Fyrox Skybox | **19 / 4,633 / 4,139 / 176,499 / 0 / 0** | `7cf283a751d152db37f3004d882b13571a5a0af69d3c82a15b9cb83fd8decb24` |

指纹算法为：repository-relative path转`/`并小写排序；每文件取当前bytes的lowercase SHA-256；聚合输入为`path|file_sha256`以单个LF连接且末尾无LF，再对UTF-8 payload取SHA-256。tests/ignored静态统计Rust、NUnit、Godot与Unreal test声明。选择集是证据边界，不应解释为仓库唯一相关文件。

### 2.2 纵向扫描链

本轮按 package/capability -> source/schema/dependency -> compiler/artifact/DDC -> project Scene -> World/Level lifecycle -> real/virtual/fixed/celestial time -> transition/RNG/region -> snapshot/query -> environment/atmosphere/cloud/IBL/fog -> wind/particle/precipitation/surface -> lightning/sound/gameplay -> network/save/replay/headless -> budget/diagnostics/tests/App/Editor/product evidence逐层读取。对Weather、Climate、Cloud、Wind等词执行了production/test分区；fixture名称和UI文案没有被当作产品类型。

### 2.3 动态证据边界

本文达到E3 source-level review，没有运行Cargo、WGPU、Editor/App、PIE、asset cook、Scene roundtrip、双进程网络、save/replay、GPU capture、device-loss、scale/soak或竞争benchmark。源码足以证明owner和数据链缺失、CPU/GPU风力不一致、静态UI false surface及参考边界；未来确定性、网络纠正、云像素、surface persistence和性能必须由实现后的oracle与raw receipt证明。

## 3. 当前可保留的真实基础

1. `RuntimeTimeAuthority`拥有单调real clock和默认world policy；`WorldTimeController`拥有独立virtual/fixed clock、pause/scale、fixed debt、begin/commit/abort step；`WorldTimeSnapshot`携clock stamps、policy generation和discontinuity。这是Weather simulation输入，不是calendar/celestial authority。
2. World、LevelSystem、dynamic component registry、DynamicScene preflight/commit、session archive和atomic writer提供per-World安装、事务、持久化与generation基础；canonical project Scene仍不能保存Weather/plugin payload。
3. runtime plugin ID可动态扩展，catalog有provider admission；asset/import/compiler/DDC/LKG与target profile存在通用方向。首方catalog和`ResourceKind`没有Weather/Climate/Celestial/Cloud/Wind/Precipitation项。
4. Operation service已有bounded registry、prepare/apply/cancel/deadline/harvest语义；Net已有transport/RUDP/RPC/replication模型；它们没有Weather handler、codec或产品consumer。
5. `EnvironmentExtract`、source cubemap、PMREM/SH9、realtime IBL generation/time slicing、reflection probes和程序天空是真实渲染基础；普通Scene不能author Environment，物理大气和Cloud为空。
6. Volumetric Fog有typed settings、froxel integration与temporal history，可消费humidity/aerosol adapter；Weather不得直接拥有或写froxel。
7. Particle有CPU/GPU backend和stable emitter基础；CPU asset-local `external_force`与GPU gravity-only暴露了必须修复的provider/parity边界。
8. scalability、bounded queue、multi-world、generation cancellation、device-loss、diagnostic和test infrastructure可复用；没有任何Weather qualification receipt。

## 4. 当前代码事实与断路

### 4.1 Package、Asset、Scene与产品真相

1. `zircon_plugins`没有weather/climate package；first-party runtime/editor catalog没有Weather provider。动态`RuntimePluginId::new("third_party.weather_sim")`测试只证明ID容器可扩展。
2. `ResourceKind`和known project assets没有ClimateProfile、WeatherProgram、Celestial、Cloud、Wind、Precipitation或SurfaceState类型。
3. `ComponentPropertyDescriptor`仍只有`name/value_type/editable`；没有stable field ID、default/range/unit/version/migration/asset-kind/validation。
4. `SceneEntityAsset`是camera/mesh/light/post-process/physics/animation/terrain/tilemap/prefab/scripts固定字段集合，没有Environment/Weather/Cloud/Wind。script JSON map不能充当runtime authority。
5. `weather.Component.CloudLayer/Wind`、`plugin.weather`和native weather manifest位于`cfg(test)`之后；`WeatherQueryInterface`只在plugin bridge测试里验证provider替换、weak/strong import与temperature调用。
6. Weather Workbench的preset、region、timeline、warnings和preview/build结果全是静态文案。callback只回写status/message/output属性，没有operation ticket、compiler、artifact、runtime provider或terminal receipt。

### 4.2 Clock、World、Transition、Region、Network与Save

1. real/virtual/fixed `Duration`与clock stamps已经工程化，但没有epoch、calendar、day/year/season、timezone、geography、planet center、ephemeris或integer celestial tick。
2. 每个LevelSystem有WorldTimeController；没有每World Weather service、program/seed/region generation、state graph、transition identity、RNG cursor或event sequence。
3. 仓库没有`SpatialRegionId`和compiled Weather region geometry。Mountains/Coast/City/Interior只存在于ZUI文本。
4. DynamicScene/session archive和generic replication是可复用基础；没有Weather digest/state/tick/seed/correction codec，也没有save/load/replay payload或late-join contract。
5. operation/query基础能承载future handler，但当前没有forecast、override、event cursor、bounded Weather query或service lifecycle。

### 4.3 Environment、Atmosphere、Cloud、Fog与IBL

1. `EnvironmentExtract`只含`skybox/probes/baked_lighting/probe_grid`，没有Weather/Atmosphere/Cloud identity、generation或dirty mask。
2. `build_environment_extract`仍调用`EnvironmentExtract::from_preview_skybox_enabled(request.settings.preview_skybox)`；project Scene没有environment source ref。
3. `SkyboxMode`只有Disabled/ProceduralGradient/SourceCubemap。`ProceduralSkyParams`是可执行艺术天空和IBL key，不含planet/Rayleigh/Mie/absorption/aerial perspective。
4. Zircon production范围没有物理`Atmosphere`领域类型，也没有Cloud medium、weather map、density/noise/material、lighting/shadow/history work。
5. realtime IBL的generation、last-good与time slicing是真实基础；它没有Weather dirty mask，且不能替代Cloud或Atmosphere。
6. 历史`CaptureCloud`伪operation已删除；当前唯一命中是测试断言source不得包含`CAPTURE_CLOUD`/`CaptureCloud`。这关闭旧重复工作，不创建Cloud实现。

### 4.4 Wind、Precipitation、Surface、Lightning与下游

1. 没有Wind contributor、WindField snapshot、position/height/time query、gust/turbulence/LOD/overflow合同。
2. CPU particle每步累加`asset.gravity + asset.physics.external_force`；GPU emitter params只有gravity，WGSL只执行`velocity += gravity * dt`。这既不是Weather provider，也不是CPU/GPU parity。
3. 没有Rain/Snow/Hail schema、camera-relative precipitation volume、impact batching、drop receipt、wetness/puddle/snow deposition或stable surface cell。
4. Terrain、Material和Physics生产路径的精确Weather领域词边界为0；Sound/Net命中来自通用类型、测试或无关文本，没有typed Weather consumer。
5. 没有deterministic `StrikeId`、bolt/cloud/exposure/gameplay共同事件、thunder delay、listener late join或server-authoritative hazard。

## 5. 参考实现给出的工程边界

### 5.1 Unreal：组件、Scene proxy、渲染阶段与DaySequence分责

`USkyAtmosphereComponent`持planet transform、ground radius/albedo、atmosphere height、Rayleigh/Mie/absorption、multi-scattering、trace sample scale和aerial参数，并以scene proxy、static-lighting GUID与render command维护生命周期。`UVolumetricCloudComponent`持layer altitude/height、trace distance、material、sample/shadow/reflection scalability并建立cloud proxy；renderer分别建设transmittance/multi-scattering/sky-view/aerial LUT和cloud tracing/history/shadow路径。`UWindDirectionalSourceComponent`持strength/speed/gust/radius，向Scene添加、更新、移除proxy并支持position query。DaySequence把day length、time-per-cycle、preview、play/pause、static time、modifier volume和replicated playback建成独立产品面。Zircon不能把这些职责压进程序天空、固定force或UI timeline。

### 5.2 Unity HDRP：typed Volume、migration、per-camera context与Cloud质量矩阵

`VisualEnvironment`是versioned `VolumeComponent`，以typed parameters选择sky/cloud/ambient并迁移planet radius单位；`SkyManager`维护visual/lighting sky、cached rendering contexts、per-camera state、cubemap与ambient/volumetric/cloud probes。Physically Based Sky持planet/air/aerosol/ozone等物理参数。VolumetricClouds分离cloud map、shape/erosion、local clouds、wind animation、quality、temporal accumulation、full/low resolution、shadow trace/filter和RenderGraph资源；5x_SkyAndFog有Cloud Layer、Volumetric Clouds、shadow、relative clouds、exposure与banding图形场景。Zircon需要学习context/hash/history与迁移，不应把Unity Volume当deterministic Weather simulation。

### 5.3 Godot、Bevy与Fyrox：较小引擎也闭合持久化到runtime

Godot `WorldEnvironment`持有`Environment`资源，Environment统一sky/reflection/fog并通过RenderingServer RID更新；FogVolume是有shape/material/AABB/gizmo/configuration warning的持久Scene node，Sky tests覆盖资源行为。Bevy把Atmosphere和AtmosphereSettings定义为ECS component，extract到render world并真实生成transmittance/multiscattering/sky-view/aerial LUT及environment map，能力/尺寸异常显式处理。Fyrox SkyBox具Reflect/Visit/UUID、六面texture validation、cubemap生成和builder。它们不是完整Weather上限，但都高于“静态名字+默认gradient”。

### 5.4 组合结论

五套参考没有一套同时给出确定性Climate、regional transitions、network/save和全部render/gameplay adapters。Zircon应组合Unreal的组件/proxy/sequence、Unity的typed volume/context/history、Godot/Fyrox的资源持久化下限、Bevy的ECS extract/LUT，并用自己的immutable artifact、per-World authority、generation-qualified snapshot与receipt补齐跨域真值。

## 6. 唯一Owner与父P0复核

| 领域 | 唯一owner | 本篇边界 |
|---|---|---|
| Weather authoring/product truth | Editor38 | document/toolkit/transaction/preview/build/diagnostic；runtime未闭合前保持Unavailable/Prototype |
| Weather executable runtime | Runtime149（历史Runtime36） | neutral artifact/snapshot/query/adapter合同与first-party service实现要求 |
| Time/World/Scene/Save/Net/Operation | 对应Runtime owner | 提供通用primitive，不复制Weather policy |
| Atmosphere/Cloud/IBL/Fog/Exposure | Runtime96/99及render owners | 消费同代Weather adapter，不拥有state machine |
| Particle/Terrain/Water/Vegetation/Cloth/Sound/Gameplay | 各domain owner | 消费typed diff/event，不让Weather写私有buffer |
| Spatial region authoring | Editor37及Scene/streaming owner | 提供stable compiled region identity；Weather不复制字符串区域系统 |

本文 **0项新P0**。Editor38五项父P0当前均Open：静态第二authority、无产品owner/source-runtime链、Environment/sun/Time-of-Day分裂、Cloud/Precipitation/Wind/Lightning无真实域、determinism/region/save/network/cook断链。P0-4中的旧`CaptureCloud`重复gradient子项已经Closed；其余Cloud合同仍Open，不能据此关闭父项。

## 7. P1：Package、Source、Schema、Compiler与Artifact

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| WTH-P1-001 | Partial | 动态`RuntimePluginId`可承载外部ID，但无首方Weather package/module/capability；定义runtime/editor/client/server IDs、version、dependency、maturity与unload contract |
| WTH-P1-002 | Partial | catalog有generic provider admission但无Weather entry/provider；只有provider、artifact、consumer与qualification同代才发布Ready |
| WTH-P1-003 | Open | `ResourceKind`无Climate/Celestial/Weather/Region；注册versioned source kinds、refs、dependency collector与presentation |
| WTH-P1-004 | Partial | generic component descriptor仅name/type/editable；建立stable field ID、unit/range/default/nullable/enum/unknown policy |
| WTH-P1-005 | Partial | asset/compiler存在通用validation，但无Weather finite/physical rules；经纬海拔、温湿度、概率、速度、duration、curve必须fail-close |
| WTH-P1-006 | Partial | generic asset dependency/BuildSet可复用；Weather profile/curve/noise/texture/region/VFX/sound/material manifest不存在 |
| WTH-P1-007 | Partial | deterministic artifact/compiler基础可复用；实现Weather compiler并固定source/dependency/toolchain/target到table/diagnostic/digest映射 |
| WTH-P1-008 | Open | `WeatherProgramArtifact`精确类型为0；定义schema/compiler版本、state/edge/curve/region table、seed policy与adapter declarations |
| WTH-P1-009 | Partial | DDC/LKG/publication基础存在，无Weather key/source equivalence；失败保留同源last-good并暴露stale/rollback disposition |
| WTH-P1-010 | Open | canonical Scene无Weather/Environment binding；保存program/profile refs、clock、regions、override/fallback并支持unknown/prefab roundtrip |
| WTH-P1-011 | Partial | generic migration/archive可复用；四类source/artifact分别版本化并产生loss/backup/rollback report |
| WTH-P1-012 | Partial | generic target/capability admission存在；为EditorHost/Client/DedicatedServer声明Supported/Degraded/Unsupported及缺失consumer |

## 8. P1：Clock、Calendar、Climate与Celestial

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| WTH-P1-013 | Partial | real/virtual/fixed clocks真实存在但无celestial；独立`CelestialClock`以integer tick保存authority，render只读插值sample |
| WTH-P1-014 | Partial | World time policy有pause/scale/fixed plan，无program clock declaration；明确simulation/virtual/celestial/sequence/preview来源和correction政策 |
| WTH-P1-015 | Open | 无calendar/epoch/day/year/season/timezone codec；定义rounding、overflow、serialization与migration |
| WTH-P1-016 | Open | 无logical day length/real-time cycle合同；source必须显式声明，禁止隐式24h或wall clock |
| WTH-P1-017 | Partial | transform/large-world基础可复用；补纬度/经度/海拔/北向/up-axis/planet center/origin rebasing契约 |
| WTH-P1-018 | Open | 无ephemeris或项目曲线精度合同；建立单位、适用范围、reference dataset和误差门 |
| WTH-P1-019 | Open | 无celestial stable identity；Sun/Moon/其它天体需stable ID、role、disk/light/visibility policy |
| WTH-P1-020 | Partial | DirectionalLight真实存在但无atmosphere role/index；增加typed celestial binding、photometry、temperature、angular radius和shadow policy |
| WTH-P1-021 | Partial | 程序sun与DirectionalLight都存在但互不绑定；由单一`CelestialSample`生成direction/disk/photometry/invalidation key |
| WTH-P1-022 | Open | 无ClimateProfile慢变量；按calendar/region输出temperature/humidity/wind envelope与allowed weather |
| WTH-P1-023 | Partial | WorldTimeSnapshot已有discontinuity和render history基础；发布Continuous/Jump/Correction并让Cloud/Fog/Exposure/TAA/IBL逐域决策 |
| WTH-P1-024 | Partial | virtual pause/scale/fixed基础存在但scale经f64到Duration；authority改为tick+rational policy并覆盖快进/回放/preview |

## 9. P1：World Authority、Transition、Region、Lifecycle、Network与Save

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| WTH-P1-025 | Partial | World/Level lifecycle可安装service但无`WorldWeatherService`；每World唯一owner并发布immutable snapshot |
| WTH-P1-026 | Partial | Operation service有prepare/apply/cancel/deadline终态；映射Weather install/compile/override/retire ticket且每ticket唯一终态 |
| WTH-P1-027 | Partial | generic deterministic identity/RNG测试基础可复用；按program/world/region/state/transition/strike派生独立stream |
| WTH-P1-028 | Open | 无transition graph；把condition/duration/hysteresis/cooldown/priority/stable tie-break编译进artifact |
| WTH-P1-029 | Partial | asset/entity/generation identity基础存在；Weather state/transition ID必须跨reload/save/net/replay稳定，拒绝display name authority |
| WTH-P1-030 | Open | 无`SpatialRegionId`或compiled Weather geometry；只消费唯一region owner的ID/index/generation |
| WTH-P1-031 | Partial | render/volume领域有generic blend policy；Weather每字段固定override/add/multiply/min/max/normalized及stable tie-break |
| WTH-P1-032 | Partial | streaming/generation基础可复用；cell add/remove必须保留authority/history/event cursor并拒绝late region result |
| WTH-P1-033 | Partial | immutable snapshot/transaction primitive存在；clock/state/region/events/dirty set需单commit generation冻结 |
| WTH-P1-034 | Partial | Net有replication/RPC基础但无Weather codec；复制digest/tick/state/transition/seed/correction而非逐帧float |
| WTH-P1-035 | Partial | DynamicScene/session archive/migration存在，无Weather SaveGame payload；持久化clock/state/progress/RNG/regions/event sequence |
| WTH-P1-036 | Partial | Operation/query ABI有bounded/cancel基础；实现snapshot query、override、forecast、event cursor的limit/generation/timeout/disposition |

## 10. P1：Environment、Atmosphere、Cloud、IBL、Fog与Multi-View

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| WTH-P1-037 | Partial | `EnvironmentExtract`是typed carrier但无Weather identity；增加同代celestial/atmosphere/cloud/fog generations的adapter snapshot |
| WTH-P1-038 | Open | `preview_skybox`仍是Scene-to-render唯一环境输入；project Scene必须读取source binding，gradient仅显式Low/Preview fallback |
| WTH-P1-039 | Open | 无physical atmosphere type；输出planet/ground/Rayleigh/Mie/absorption/aerial参数给render owner，不在Weather复制LUT资源 |
| WTH-P1-040 | Partial | IBL/environment已有generation/invalidation基础；定义profile/sun/aerosol/camera altitude/quality/device dirty mask |
| WTH-P1-041 | Open | 无Cloud source/runtime payload；编译layer/coverage/density/map/noise/material/wind/resource dependencies |
| WTH-P1-042 | Partial | render graph、quality与budget可复用；Cloud view/light/reflection sample、resolution、distance、early exit和fallback仍需实现 |
| WTH-P1-043 | Open | 无Cloud lighting/shadow adapter；消费Celestial/Atmosphere/IBL并发布transmittance/ambient/shadow generation |
| WTH-P1-044 | Partial | generic temporal history/cut/reset基础存在；Cloud history绑定view/cloud/weather/quality generation并拒绝ghost/stale |
| WTH-P1-045 | Partial | realtime IBL有generation/time slicing/last-good；Weather dirty/cadence/threshold/crossfade/cancel尚未接入 |
| WTH-P1-046 | Partial | Volumetric Fog是真实consumer基础；仅接受humidity/aerosol/visibility target+generation，Weather不写froxel |
| WTH-P1-047 | Partial | exposure/history/event primitive可复用；把lightning impulse、cloud occlusion、day transition接成typed同代事件 |
| WTH-P1-048 | Partial | view family、stereo/reflection/capture基础存在；各view需region/sample/history/resource sharing政策 |

## 11. P1：Wind、Precipitation、Surface、Lightning、Sound与Gameplay

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| WTH-P1-049 | Open | 无Wind source/contributor；定义global/local/directional/gust/turbulence/altitude profile与stable contributor ID |
| WTH-P1-050 | Open | 无WindField snapshot/query；实现bounded grid/clipmap或analytic set、position/height/time query、LOD与overflow |
| WTH-P1-051 | Open | 没有共享wind generation；Particle/Vegetation/Cloth/Water/Audio只能消费统一filtered sample，禁止各自复制参数 |
| WTH-P1-052 | Partial | CPU particle支持asset-local `external_force`，无runtime provider；分离asset baseline与per-instance Weather force |
| WTH-P1-053 | Open | GPU只上传gravity；编码同一force provider，unsupported显式degrade并用CPU oracle做parity |
| WTH-P1-054 | Open | 无Rain/Snow/Hail schema/view volume；phase/intensity/size/fall speed/temperature/budget驱动bounded camera-relative instances |
| WTH-P1-055 | Partial | physics/depth/query与effect request primitive可复用；建立surface impact batch、splash/ripple/decal/audio request及drop receipt |
| WTH-P1-056 | Open | 无`SurfaceWeatherState`；按stable cell/material channel持久化deposition/evaporation/melt/puddle/snow |
| WTH-P1-057 | Open | 无跨domain surface adapter；Terrain/Material/Vegetation/Water/Cloth消费typed diff，不被Weather写私有buffer |
| WTH-P1-058 | Open | 无deterministic `StrikeId`；由seed/tick/region/candidate/authority sequence生成并跨visual/gameplay共享 |
| WTH-P1-059 | Partial | Sound scheduling/voice基础可复用；按strike/listener/speed-of-sound/occlusion调度并处理pause/late join |
| WTH-P1-060 | Partial | Gameplay/AI/Nav事件基础可复用；server发布typed hazard/visibility/cover事件，cosmetic与authority分离 |

## 12. P1：Scalability、Reliability、Diagnostics、Tests与Product Qualification

| ID | 状态 | 当前证据与必须重构 |
|---|---|---|
| WTH-P1-061 | Partial | generic quality/budget/admission存在；统一决策region/event/cloud/IBL/wind/particle/impact/surface/audio CPU/GPU/memory |
| WTH-P1-062 | Partial | bounded queue基础存在；所有contributor/event/diff/diagnostic需count/bytes/age/time hard limit与overflow终态 |
| WTH-P1-063 | Partial | multi-world/Level隔离基础存在；Weather state/generation/RNG/resource必须per-World并按cell demand增量resolve |
| WTH-P1-064 | Partial | generation/cancel/unload primitive存在；compile/stream/cloud/IBL/particle/sound work需拒绝late publish |
| WTH-P1-065 | Partial | device-loss/headless generic路径存在；GPU缺失不改变authority，restore不重放或改写gameplay state |
| WTH-P1-066 | Partial | runtime diagnostics基础存在；新增program/tick/state/region/RNG/event/dirty/budget/degrade/stale/failure快照 |
| WTH-P1-067 | Open | 无Weather schema/compiler golden；validation/migration/dependency/digest/bad graph/cycle/LKG矩阵为空 |
| WTH-P1-068 | Partial | time/world deterministic tests可复用；补pause/scale/jump/rewind、1/1k regions、thread schedule、reload hash matrix |
| WTH-P1-069 | Open | 无Weather network/save/replay test；late join/correction/digest skew/rollback/duplicate event/restart恢复为空 |
| WTH-P1-070 | Open | 无Weather adapter CPU/GPU/pixel oracle；celestial/atmosphere/cloud/fog/wind/particle/surface/lightning逐代验证为空 |
| WTH-P1-071 | Open | 无真实产品fixture；建立clear-day/storm/snow/front/region-crossing/lightning六类save-play-export-capture场景 |
| WTH-P1-072 | Open | 无竞争性raw benchmark；冻结相同场景/硬件/画质/序列后对照Unreal/Unity CPU/GPU/RAM/VRAM/stutter/image error |

## 13. P2：高阶能力

| ID | 状态 | 高阶能力与进入条件 |
|---|---|---|
| WTH-P2-001 | Open | spectral multi-layer atmosphere；RGB physical baseline、reference oracle与预算完成后进入 |
| WTH-P2-002 | Open | multi-planet/multi-star celestial；single-planet Sun/Moon、space/precision与lighting合同完成后进入 |
| WTH-P2-003 | Open | physically coupled climate solver；deterministic profile/state baseline与离线reference dataset完成后进入 |
| WTH-P2-004 | Open | data-assimilated forecast/import；provenance/license/schema migration/offline fallback完成后进入 |
| WTH-P2-005 | Open | volumetric storm cell dynamics；Cloud medium、WindField、Lightning与scale receipts完成后进入 |
| WTH-P2-006 | Open | tornado/hurricane structured flow；WindField及physics/particle/vegetation adapter和safety budget完成后进入 |
| WTH-P2-007 | Open | hydrology/puddle flow coupling；Terrain/Water/SurfaceState stable cell identity与persistence完成后进入 |
| WTH-P2-008 | Open | deformable persistent snow；material/terrain/character interaction、save/network/streaming完成后进入 |
| WTH-P2-009 | Open | ocean-atmosphere energy coupling；Water wave/wind/temperature adapter与deterministic exchange完成后进入 |
| WTH-P2-010 | Open | cloud-ground electrical field；StrikeId、authority、visual/audio timing与reference model完成后进入 |
| WTH-P2-011 | Open | urban canopy/microclimate；region/material/geometry input具bounded acceleration与validation后进入 |
| WTH-P2-012 | Open | forecast-driven AI/gameplay planning；typed uncertainty、authority与AI budget完成后进入 |
| WTH-P2-013 | Open | cross-server weather fronts；shard clock/region ownership、handoff/dedup/reconciliation完成后进入 |
| WTH-P2-014 | Open | authoritative rollback prediction；network/save/replay与deterministic adapter event合同完成后进入 |
| WTH-P2-015 | Open | third-party Weather provider SDK；ABI/version/trust/budget/unload/artifact compatibility完成后进入 |
| WTH-P2-016 | Open | distributed visual/performance qualification farm；frozen BuildSet、GPU/driver matrix、capture/diff/raw receipt完成后进入 |

上述能力不能反向替代P1 baseline，也不能用来提前发布Weather capability。

## 14. 分层重构里程碑

| 里程碑 | 内容 | 退出条件 |
|---|---|---|
| M0 Truth与Owner | 冻结Editor38/Runtime149父子关系、neutral ABI与first-party package owner；UI保持Unavailable/Prototype | 无静态Ready、无第二schema/compiler、架构与hard-cutover顺序批准 |
| M1 Source/Compiler | WTH-P1-001..012：source kinds、schema、dependency、compiler、artifact/DDC/LKG、Scene binding、target admission | roundtrip/migration/digest/LKG/target tests通过 |
| M2 Clock/Celestial | WTH-P1-013..024：integer celestial tick、calendar/geography/ephemeris、Sun/Moon、light binding、Climate envelope | reference accuracy、long-run、pause/scale/jump/history门通过 |
| M3 World/Region | WTH-P1-025..036：service、RNG、transition、region、atomic snapshot、network/save/replay/query | restart/thread/late join/stream/unload重复hash与receipt通过 |
| M4 Render Adapters | WTH-P1-037..048：Scene Environment、Atmosphere、Cloud、IBL/Fog/Exposure、多视图 | real medium/LUT/shadow/history、stale rejection和fallback通过 |
| M5 Weather Effects | WTH-P1-049..060：WindField、Particle parity、precipitation、impact/surface、Lightning/Thunder/gameplay | CPU/GPU oracle、surface persistence、StrikeId与timing通过 |
| M6 Reliability/Product | WTH-P1-061..071：联合预算、bounded data、multi-world、fault/headless、diagnostics、tests和六fixture | save/reopen/play/export/capture、1/1k/100k scale receipts通过 |
| M7 Competitive | WTH-P1-072：冻结BuildSet与同场景对照 | 可复跑raw receipt证明目标指标，而非截图/平均FPS宣称 |

## 15. 资格门

| Gate | 状态 | 当前判定与必须证明 |
|---|---|---|
| WTH-G01 | Fail | 静态Workbench/fixture仍形成Weather外观；capability不得从UI、SDK、测试名或画质开关推导 |
| WTH-G02 | Partial | generic provider admission存在；Weather package/source/compiler/artifact/service/consumer/qualification尚未同代 |
| WTH-G03 | Partial | generic roundtrip/migration基础存在；Weather unknown/downgrade/rollback未验证 |
| WTH-G04 | Fail | 无Weather compiler，无法证明重复compile digest一致 |
| WTH-G05 | Partial | generic LKG/publication存在；无Weather source-equivalent LKG receipt |
| WTH-G06 | Fail | Scene save/reopen/prefab/cook无法保留Weather refs/clock/region |
| WTH-G07 | Partial | 通用Duration/stamp稳定但无integer celestial tick和长期rational推进 |
| WTH-G08 | Fail | 无ephemeris/reference误差门 |
| WTH-G09 | Fail | sun disk、DirectionalLight、atmosphere、shadow不消费同一sample |
| WTH-G10 | Partial | discontinuity/history primitive存在；无Weather跨consumer disposition |
| WTH-G11 | Fail | 无transition RNG重启/线程/平台/late-join determinism |
| WTH-G12 | Partial | generic stable identity/tie-break基础存在；Weather state/edge/contributor未定义 |
| WTH-G13 | Fail | 无SpatialRegion generation与事件连续性 |
| WTH-G14 | Partial | snapshot/transaction基础存在；Weather clock/state/regions/events/dirty未原子冻结 |
| WTH-G15 | Partial | generic cancellation/generation存在；Weather task/adapter/resource late publish未验证 |
| WTH-G16 | Partial | replication基础存在；无Weather digest/tick/seed/correction codec |
| WTH-G17 | Partial | archive/migration基础存在；无Weather RNG/event cursor恢复 |
| WTH-G18 | Partial | headless/runtime基础存在；无authoritative Weather/gameplay parity |
| WTH-G19 | Fail | project Scene仍以`preview_skybox`布尔值构造Environment |
| WTH-G20 | Fail | gradient未作为Weather Low/Preview fallback受正式capability/receipt约束 |
| WTH-G21 | Fail | 无atmosphere/cloud/fog同代Weather generation与stale rejection |
| WTH-G22 | Fail | 无Cloud medium或enabled/disabled work oracle |
| WTH-G23 | Partial | generic temporal reset/reproject存在；无Cloud history generation规则 |
| WTH-G24 | Partial | realtime IBL有generation/time slicing；无Weather dirty threshold/cadence |
| WTH-G25 | Partial | multi-view/capture基础存在；无Weather region/history sharing规则 |
| WTH-G26 | Fail | 无WindField CPU/GPU oracle |
| WTH-G27 | Fail | Particle CPU asset force与GPU gravity-only合同不一致 |
| WTH-G28 | Fail | 无precipitation instance/pixel/impact budget与drop receipt |
| WTH-G29 | Fail | 无stable surface cell/material channel persistence |
| WTH-G30 | Fail | 无typed cross-domain adapter，不能证明不直写私有buffer |
| WTH-G31 | Fail | 无共享StrikeId |
| WTH-G32 | Fail | 无thunder delay/listener/pause/late-join时序oracle |
| WTH-G33 | Partial | generic bounded queues存在；Weather snapshot/event/diagnostic hard limits未定义 |
| WTH-G34 | Partial | generic device-loss路径存在；Weather authority/visual降级隔离未验证 |
| WTH-G35 | Fail | malformed/cycle/missing/OOM/overflow/unload/skew Weather矩阵为空 |
| WTH-G36 | Fail | 六类fixture的save/reopen/play/export/capture与inspection为空 |
| WTH-G37 | Fail | 1/1k/100k region和storm峰值raw receipts为空 |
| WTH-G38 | Fail | Editor38五个父P0仍Open；Runtime与authoring门未同时关闭 |
| WTH-G39 | Partial | generic generation/receipt基础可复用；source/build/device/driver过期Weather receipt未实现 |
| WTH-G40 | Fail | 没有同场景/序列/硬件/画质的Unreal/Unity可复跑竞争证据 |

## 16. 禁止的临时修补

1. 禁止新增`WeatherState { kind: String, properties: Json }`万能bag或以script JSON成为authority。
2. 禁止只建空Weather manifest、descriptor、catalog行或`WeatherQueryInterface`就发布capability。
3. 禁止继续让`preview_skybox`布尔值承担project Environment。
4. 禁止把程序gradient sun、fog density、Particle `external_force`、WOC画质设置或fixture名称称为Weather。
5. 禁止用wall clock、global RNG、逐帧float累计或display name驱动authority。
6. 禁止太阳圆盘、DirectionalLight、atmosphere与shadow各自编辑方向/强度。
7. 禁止Weather直接写render texture、particle pool、terrain/material/water/cloth/vegetation私有buffer或sound mixer。
8. 禁止每个雨滴创建entity、physics query、decal或audio voice；必须批处理并有drop receipt。
9. 禁止复制Mountains/Coast字符串区域系统绕过唯一SpatialRegion owner。
10. 禁止server逐帧复制全部visual float、客户端VFX反向成为gameplay truth或save只存preset名。
11. 禁止用registration/unit/source-string test替代GPU pixel、双进程、save/replay、product与scale evidence。
12. 禁止因旧`CaptureCloud`已删除就宣称Cloud完成；删除伪工作只恢复capability truth。

## 17. 实施职责蓝图

| Owner候选 | 应新增/重构 | 不应放入 |
|---|---|---|
| zircon_runtime neutral contract | artifact/snapshot/query/adapter IDs、World lifecycle与receipt DTO | Weather算法、Editor状态、GPU资源 |
| zircon_plugins/weather/runtime | source/compiler/service/transition/region resolve和adapter orchestration | 私有Time/Scene/IBL/Particle/Sound owner |
| zircon_plugins/weather/editor | document/toolkit/preview/build/diagnostic projection | 第二份schema/compiler或mock authority |
| Time/World/Scene owners | clock/fixed-step、service install、Scene binding和atomic publish primitives | Celestial/Weather policy |
| Render owners | atmosphere/cloud/IBL/fog/exposure真实consumer与GPU receipts | Weather transition/RNG authority |
| Domain owners | particle/terrain/water/cloth/vegetation/sound/gameplay typed adapters | Weather source和跨域直接写入 |

M0必须先以architecture decision固定最终crate路径、public ABI、provider admission、Scene/schema owner和hard-cutover顺序。不得把实现继续堆进`scene/world/render.rs`、`EnvironmentExtract`构造器或Editor callback。

## 18. 本轮产出边界

本轮只新增current-source review和索引记录，没有修改Runtime、Editor、Plugin、App、Interface、Hub、tests、Cargo manifest或workflow，也没有实施任何Weather代码。退出本审查切片只表示历史Runtime36已按当前源码刷新、旧`CaptureCloud`事实已纠正、72项P1/16项P2/40项Gate及owner/里程碑已重新登记；`implementation_status`仍为pending，整个引擎优化目标仍处于`in_progress`。
