---
title: Runtime Weather、Climate、Celestial、Atmosphere、Cloud 与 Wind 当前工作树复审
category: zircon_runtime
report_id: Runtime184
review_date: 2026-08-30
baseline_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
verification_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/99zx-runtime-weather-climate-celestial-time-of-day-wind-precipitation-cloud-atmosphere-surface-state-determinism-network-save-scalability-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/36-weather-climate-celestial-time-of-day-wind-precipitation-cloud-atmosphere-surface-state-determinism-network-save-scalability-product-integration-review.md
related_editor_owner:
  - docs/plans/optimize/zircon_editor/244-editor-weather-climate-atmosphere-current-working-tree-review.md
related_code:
  - zircon_runtime/src/scene/world_time
  - zircon_runtime/src/scene/world
  - zircon_runtime/src/scene/dynamic_scene
  - zircon_runtime/src/core/framework/render/environment
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment
  - zircon_plugins/rendering/features/volumetric_fog
  - zircon_plugins/particles
  - zircon_plugins/terrain
  - zircon_plugins/sound
  - zircon_plugins/net
  - zircon_runtime/src/plugin/runtime_plugin
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_weather_editor_workspace.zui
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/SkyAtmosphereComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/VolumetricCloudComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/WindDirectionalSourceComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/SkyAtmosphereRendering.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/VolumetricCloudRendering.cpp
  - dev/UnrealEngine/Engine/Plugins/Experimental/DaySequence/Source/DaySequence
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Sky
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/VolumetricClouds
  - dev/godot/scene/resources/environment.cpp
  - dev/godot/scene/3d/world_environment.cpp
  - dev/godot/scene/3d/fog_volume.cpp
  - dev/bevy/crates/bevy_light/src/atmosphere.rs
  - dev/Fyrox/fyrox-impl/src/scene/skybox.rs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime184 · Weather/Climate/Atmosphere 当前工程化差距

## 1. 结论

当前 Zircon 没有 Weather/Climate runtime 产品。没有 Weather source kind、ClimateProfile、CelestialClock、WeatherProgramArtifact、WorldWeatherService、WeatherFrameSnapshot、WindField、SurfaceWeatherState、network codec、SaveGame participant 或 concrete weather plugin。运行时拥有 per-World virtual/fixed time、程序 sky/IBL、volumetric fog、particles、terrain、sound、net 与 plugin loader 底座，但它们没有共享 Weather authority。

工作树里 `weather.runtime`、`weather.Component.CloudLayer`、`weather.seasonal` 等主要位于 plugin loader/extension tests；没有 weather production package。`World::build_environment_extract` 仍以 preview skybox/默认 gradient 作为环境输入，`RenderFrameExtract` 没有 Weather snapshot。Weather Workbench 的 `Weather_Storm`、Cloud Build、Rain Burst、Wind Gust、Lightning 和 `8 layers / 5 regions / 2 warnings` 是 collapsed 静态 ZUI fixture。

历史 Runtime149/36 的“无 Weather owner”结论仍成立。本次登记 **0 项新 P0、30 项 P1、12 项 P2、26 道资格门**；P1 30 Open，P2 12 Open，资格门 23 Fail、3 Partial、0 Pass。目标架构：

```text
Climate/Celestial/Weather/Region Sources
  -> deterministic WeatherCompiler + dependency admission
  -> immutable WeatherProgramArtifact
  -> per-World WeatherService(program/clock/seed/region generations)
  -> atomic WeatherFrameSnapshot + bounded query/event cursor
  -> Celestial/Atmosphere/Cloud/Fog/Wind/Precipitation/Surface/
     Lightning/Sound/Gameplay/Network/Save adapters
```

## 2. 当前源码证据

- `WorldTimeState`/`WorldFixedStep` 提供虚拟时间和固定步进，但没有 calendar/day/season/climate/weather program、tick authority 或 weather generation。
- Environment extract/probe/IBL 与 sky preview 是 renderer 输入；没有 physical atmosphere parameters、cloud map/coverage/density、wind/precipitation source、celestial role 或 Weather-to-fog/IBL adapter。
- Volumetric Fog 是真实 consumer 底座，Particle GPU/CPU 仍只接 gravity/external force，Terrain/Sound/Net 没有 Weather typed adapter；不能把 fog/particle/noise 组件当 Weather。
- plugin runtime loader 能解析 weather/climate 测试 manifest，但没有 concrete package、resource kind、compiler 或 per-World provider。
- Weather ZUI `props.visibility = "collapsed"`，静态样例和按钮 routes 没有 operation factory/document/transaction/job/artifact/receipt。

## 3. 参考引擎差异

Unreal SkyAtmosphere/VolumetricCloud/WindDirectionalSource 与 DaySequence 分开管理 planet/atmosphere/cloud/wind/celestial time，并把 rendering、shadow、fog、IBL、world partition 与 editor component 连接；Unity HDRP Sky/VolumetricClouds 提供 physical sky、cloud layer/noise, lighting/history/budget；Godot WorldEnvironment/FogVolume 与 Bevy atmosphere 提供 scene resource/render extraction 对照，Fyrox 仅是 skybox 数据。Zircon 需要先建立 Weather authority，再让这些 consumers 接入。

## 4. P1 重构任务

| ID | 差异 | 必须完成 |
|---|---|---|
| RT-WTH-01 | 无资源 taxonomy | ClimateProfile/Celestial/WeatherProgram/Region/WindField/Cloud/Precipitation/SurfaceState 类型。 |
| RT-WTH-02 | 无 source/artifact | source 保存曲线/规则/区域；artifact 固化 dependencies、transitions、seed、consumer targets、hash/version。 |
| RT-WTH-03 | 无 compiler | deterministic condition/duration/hysteresis/priority graph、units、calendar、diagnostics/source map。 |
| RT-WTH-04 | 无 clock authority | calendar/epoch/day/season/timezone、rational cycle、rounding/overflow/serialization/migration。 |
| RT-WTH-05 | 无 celestial | sun/moon/stars stable IDs、ephemeris/curve、photometry/angular radius/planet frame。 |
| RT-WTH-06 | 无 World service | per-World WeatherService、program/clock/seed/region generations、replace/unload/drain。 |
| RT-WTH-07 | 无 region | spatial region/cell identity、overlap/priority/blend、partition residency、late result rejection。 |
| RT-WTH-08 | 无 snapshot/query | atomic WeatherFrameSnapshot、temperature/humidity/visibility/wind/precip/cloud query、bounded cursor。 |
| RT-WTH-09 | 无 atmosphere | Rayleigh/Mie/absorption/aerial parameters、LUT residency、camera altitude/quality/device policy。 |
| RT-WTH-10 | 无 cloud | layer/coverage/density/noise/weather map、lighting/shadow/transmittance/history and render artifact。 |
| RT-WTH-11 | 无 wind field | global/local/directional/gust/turbulence profile、position/time query、shared generation。 |
| RT-WTH-12 | 无 precipitation | rain/snow/hail schema、camera-relative bounded instances、phase/intensity/size/fall/temperature。 |
| RT-WTH-13 | 无 surface state | snow/wetness/puddle/melt/evaporation/deposition per cell/material channel、save/replay。 |
| RT-WTH-14 | 无 renderer adapters | sky/atmosphere/cloud/fog/IBL/exposure/shadow/particle/terrain/water consumers share generation。 |
| RT-WTH-15 | 无 temporal policy | continuous/jump/correction/discontinuity、history reset/crossfade/last-good per view。 |
| RT-WTH-16 | 无 lightning/events | strike identity、server authority、flash/impulse/occlusion/audio/VFX events、dedupe/cursor。 |
| RT-WTH-17 | 无 gameplay/audio | weather query/effects、surface impact/wake, ambience, gameplay tags and deterministic receipts。 |
| RT-WTH-18 | 无 network/save | tick/state/transition/seed digest codec、join-in-progress、save slot/checkpoint、migration。 |
| RT-WTH-19 | 无 scalability | cloud/fog/wind/precip quality tiers、distance/resolution/budget/memory/GPU time and fallback。 |
| RT-WTH-20 | 无 failure policy | invalid program/provider/LUT/device/region loss、budget overflow、stale generation terminal state。 |
| RT-WTH-21 | 无 diagnostics | layer/region/query/event/history/LUT/GPU/CPU/memory/bandwidth/fallback metrics。 |
| RT-WTH-22 | 无 tests | calendar/ephemeris/compiler, deterministic transitions, query, render parity, net/save, fault/soak/scale。 |
| RT-WTH-23 | 测试 manifest 误充产品 | weather/climate test IDs 不得 publish capability；concrete provider 才能 admission。 |
| RT-WTH-24 | gradient 误充 sky | gradient 只能显式 fallback，physical sky/atmosphere generation 必须可观测。 |
| RT-WTH-25 | cross-domain authority | Particle/Cloth/Hair/Water/Fog/IBL/Audio 只消费 immutable Weather snapshot。 |
| RT-WTH-26 | 大世界/精度缺失 | origin rebasing、planet frame、region partition、large coordinate/query precision。 |
| RT-WTH-27 | editor/runtime 断裂 | Editor source/operation -> runtime compile/service/snapshot receipt。 |
| RT-WTH-28 | 线程所有权不明 | snapshot/lease、render/physics/audio consumers、event ordering and cancellation。 |
| RT-WTH-29 | product integration absent | Scene/PIE/standalone/save/reopen/network plus atmosphere/cloud/fog/wind/precip cases。 |
| RT-WTH-30 | quality gates absent | deterministic CPU oracle、GPU capture、visual diff、fault/scale/soak/perf budgets。 |

## 5. P2 增强任务

| ID | 差异 | 工程化方向 |
|---|---|---|
| RT-WTH-P2-01 | 缺天气历史 | persisted weather history、replay scrubbing、branch/correction 与 migration。 |
| RT-WTH-P2-02 | 缺空间插值 | high-quality region blending、altitude bands、front propagation 与 boundary hysteresis。 |
| RT-WTH-P2-03 | 缺太阳/月亮高级模型 | eclipse、aerial visibility、seasonal orbit、starfield and celestial calibration。 |
| RT-WTH-P2-04 | 缺云体细节 | multi-scale noise、erosion、self-shadow、phase function、cloud tracing cache。 |
| RT-WTH-P2-05 | 缺降水碰撞 | surface impact、splash/snow accumulation、shelter/occlusion and gameplay query。 |
| RT-WTH-P2-06 | 缺湿度/积雪材质 | material channel packing、virtual texture feedback and partial terrain updates。 |
| RT-WTH-P2-07 | 缺天气音频空间化 | ambience zones、occlusion、wind/precip one-shots、distance and mix priorities。 |
| RT-WTH-P2-08 | 缺 weather-driven VFX | particle graph inputs、lightning/impact events、deterministic effect lifetime。 |
| RT-WTH-P2-09 | 缺移动/低端降级 | LUT resolution、cloud mode、precip budget and explicit unsupported feature states。 |
| RT-WTH-P2-10 | 缺观测导出 | frame/region/layer/query/event traces、capture provenance and offline comparison。 |
| RT-WTH-P2-11 | 缺 canonical scenarios | clear/overcast/storm/snow/dust scenes with visual and state golden baselines。 |
| RT-WTH-P2-12 | 缺安全约束 | bounded rule graph、noise dimensions、event rate、memory and network payload limits。 |

## 6. 资格门

| 门 | 结果 | 关闭证据 |
|---|---|---|
| time substrate | Partial | WorldTime/fixed-step exists, but no calendar/season/weather authority or shared generation。 |
| environment substrate | Partial | environment/IBL/fog paths exist, but no physical weather/atmosphere/cloud provider。 |
| adjacent consumers | Partial | particles/terrain/sound/net can be adapters, but no Weather snapshot ingress。 |
| resource taxonomy | Fail | Climate/Celestial/Weather/Region/Wind/Cloud/Precipitation/SurfaceState kinds。 |
| source schema | Fail | curves/rules/units/seed/dependencies/unknown fields round-trip。 |
| compiler artifact | Fail | deterministic program artifact、hash、diagnostics、source map and migration。 |
| world service | Fail | per-World program/clock/seed/region generation and unload/drain lifecycle。 |
| clock/calendar | Fail | rational epoch/day/season/timezone semantics with overflow and correction policy。 |
| celestial | Fail | sun/moon/stars/ephemeris/photometry and planet frame receipts。 |
| region/query | Fail | spatial region/cell blending and bounded snapshot/query cursor。 |
| atmosphere | Fail | Rayleigh/Mie/absorption parameters, LUT residency and quality policy。 |
| cloud | Fail | cloud layer/noise/coverage/transmittance/shadow/history artifact and execution。 |
| wind | Fail | global/local field, gust/turbulence, position/time query and shared generation。 |
| precipitation | Fail | rain/snow/hail bounded instances, phase/impact/temperature and budget。 |
| surface state | Fail | wetness/snow/puddle/melt/deposition per cell/material with save/replay。 |
| renderer adapters | Fail | sky/atmosphere/cloud/fog/IBL/exposure/shadow/particle/terrain consumers。 |
| temporal policy | Fail | continuous/jump/correction/cut/teleport history and last-good behavior。 |
| events | Fail | lightning/transition/enter-exit/impact identity, authority, dedupe and cursor。 |
| gameplay/audio | Fail | typed weather queries/effects, ambience and deterministic receipts。 |
| network/save | Fail | state digest codec, join-in-progress, checkpoint, replay and migration。 |
| diagnostics | Fail | layer/region/query/event/history/LUT/GPU/CPU/memory/fallback metrics。 |
| failure/device loss | Fail | invalid provider/LUT/region/device/world loss and stale generation terminal states。 |
| editor bridge | Fail | source/operation -> compiler/service/snapshot with provenance and receipt。 |
| scalability/performance | Fail | cloud/fog/wind/precip quality tiers, large-world budgets and soak evidence。 |
| product integration | Fail | Scene/PIE/standalone plus atmosphere/cloud/fog/wind/precip/network/save cases。 |
| benchmark corpus | Fail | clear/storm/snow/dust scenarios with deterministic state and visual baselines。 |

本轮仅写审查文档，未修改生产代码、测试、Cargo、ABI 或 ZUI，也未运行 Weather/atmosphere/GPU/PIE 动态验证。
