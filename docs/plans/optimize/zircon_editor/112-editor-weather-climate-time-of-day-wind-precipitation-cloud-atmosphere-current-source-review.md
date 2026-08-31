---
title: Editor Weather、Climate、Time-of-Day、Wind、Precipitation、Cloud 与 Atmosphere 当前源码复核
category: zircon_editor
report_id: Editor112
review_date: 2026-08-26
baseline_head: 590376671b8745a0d230304c94432857c669bfbd
baseline_epoch: 524
canonical_owner: Editor38
refreshes:
  - docs/plans/optimize/zircon_editor/38-weather-climate-time-of-day-wind-precipitation-cloud-atmosphere-environment-authoring-review.md
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
tests:
  - zircon_runtime/src/core/runtime/time.rs
  - zircon_runtime/src/core/framework/time
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_wgpu_recorder/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_runtime/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_time_slice/tests.rs
  - zircon_plugins/particles/runtime/src/tests
  - zircon_plugins/particles/editor/src/tests.rs
  - zircon_plugins/first_party_runtime_catalog/src/tests.rs
  - zircon_plugins/first_party_editor_catalog/src/tests.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/world_building/volume_and_weather.rs
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

# 112 · Editor Weather / Climate / Time-of-Day / Wind / Precipitation / Cloud / Atmosphere 工程化差距

## 1. 结论

当前 Zircon 没有可供项目使用的 Weather 或 Climate 产品。Editor 只有一张静态 Weather Workbench：固定 `Weather_Storm`、`Region_Mountains`、`Layer_Clouds`，固定 Cloud Build/Rain Burst/Wind Gust/Lightning 时间段，固定 `8 layers / 5 regions / 2 warnings`，Preview/Build/Preset/Region/Blend 提交只经过模板 route 并返回 queued 文本。仓内没有 Weather/Climate plugin、source asset、Scene component、compiler、artifact、runtime service、catalog registration 或 install receipt。

Runtime 底层并非空白，但必须保持准确命名：real/virtual/fixed clock、DirectionalLight、typed Post Process/Volumetric Fog、程序天空、Source Cubemap/PMREM/SH9、generation-aware realtime IBL time slicing、CPU/GPU particle、Sound/Terrain 和共享 Region 都有局部实现。这些是可复用 adapter/executor，不是 Weather authority。

关键断路包括：Scene 没有 Sky/Atmosphere/Cloud/Wind/Precipitation/Climate/TimeOfDay 字段，`World::build_environment_extract()` 只读取 `preview_skybox` bool 并返回固定 gradient/disabled；ProceduralSky sun 与 DirectionalLight 不绑定；runtime time 没有日期/地理/季节/天体权威；`CaptureCloud` 和 `CaptureSky` 进入相同 gradient `record_capture()` 与 source cubemap mip；Particle 没有 wind field、rain/snow/hail、surface impact、wetness 或 accumulation，且 CPU external force 与 GPU frame layout不一致；Weather 没有连接 Sound、Terrain、Material、Light、Fog、Particle 或 Editor38/37 region。

正确目标不是万能 Weather property bag，而是一条可编译、可持久化、可复现的环境状态链：

`ClimateProfile + CelestialProfile + WeatherPreset + WeatherTimeline + RegionBinding -> WeatherCompiler -> WeatherProgramArtifact -> WorldWeatherService/CelestialClock -> immutable WeatherFrameSnapshot -> Sky/Atmosphere/Cloud/IBL/Fog/Wind/Precipitation/Surface/Lighting/Audio/Gameplay adapters -> truthful Editor toolkit`

Climate 描述慢变量与地理基线，Weather 描述状态迁移，Time-of-Day 描述天体时钟，Region 只提供空间权重。Unreal 拆分 SkyAtmosphere/VolumetricCloud/DirectionalLight/Fog/Wind 组件，Unity HDRP 用 VisualEnvironment/PhysicallyBasedSky/VolumetricClouds/VolumeStack，Godot 使用 Environment/Sky/FogVolume，Bevy/Fyrox 也有可持久化或可验证的天空/雾基础；Zircon 当前尚未达到这一最低产品闭环。

## 2. 审查范围与证据

### 2.1 当前工作树物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / tests | 指纹 |
|---|---:|---|
| Zircon Runtime/Editor/Plugin selected | **215 / 42,557 / 38,835 / 1,527,992 / 318** | Weather workspace、Scene/time/environment/IBL/fog、particles、catalog、terrain/sound；`b5de338f24eae26f3f603bc8ee167dcbc909ee5f9db9319c483c55c15142edb3` |
| Unreal/Godot/Bevy/Fyrox/Unity reference | **35 / 11,413 / 9,977 / 492,418 / 0** | Sky/Atmosphere/Cloud/Light/Fog/Wind、Godot Environment/Sky/FogVolume、Bevy atmosphere/fog/volumetric、Fyrox sky/light、Unity HDRP sky/cloud；`3af6c816c30892ba482adcd07d9484b84de4104f751897e0d3f2c2927024bb5a` |
| Zircon selected union | **250 / 53,970 / 48,812 / 2,020,410 / 318** | current physical working tree union；`afed896f26a0ceadb60e0b06e4be5586decbe07b8aef7c20266ad155877541b1` |

统计按 selected root 去重、相对路径排序后以 UTF-8 内容计算 SHA-256；test 数只表示属性数。当前 baseline epoch 524，工作树含无关在途修改；实施前必须重导 manifest/fingerprint。`realtime_ibl_wgpu_recorder/tests.rs` 中有一个 ignored WGPU product test，不能把 environment test attributes 等同于 Weather/Cloud 完成。本轮没有运行 Cargo、天气模拟、GPU cloud/IBL、粒子降水、Audio/Lightning、Scene roundtrip、network/save 或跨平台动态验证。

### 2.2 Weather Editor 事实

1. Weather workspace 只有 Layers/Curves/Timeline 按钮，没有 typed document pane、asset query、Scene selection 或 empty/error state。
2. 左栏固定 Weather_Storm/Region_Mountains/Layer_Clouds，时间线固定 Cloud Build/Rain Burst/Wind Gust/Lightning。
3. Preset/Region 下拉固定 Storm/Clear/Fog/Snow 与 Mountains/Coast/City/Interior，Blend Time 是字符串 `12.0`。
4. `8 layers / 5 regions / 2 warnings` 和 Preview/Build/Rain/Lightning feedback 是静态业务事实。
5. template binding 只做 route/navigation；没有 operation factory、Weather document、compiler、job、generation 或 receipt。
6. first-party runtime/editor catalog 没有 Weather/Climate；`zircon_plugins/weather` 与 `zircon_plugins/climate` 不存在。
7. generic loader fixture 中出现 weather component/plugin 字符串，不代表 production type/package。

### 2.3 Scene、太阳、时钟事实

1. SceneEntityAsset 保存 Camera、Mesh、Light、PostProcess、Physics、Terrain、TileMap、Prefab、Script 等，没有 Environment/Weather/Climate/TimeOfDay。
2. `build_environment_extract()` 只读取 viewport `preview_skybox`，true 使用固定 `ProceduralSkyParams::default_gradient()`，false disabled。
3. Source Cubemap/PMREM/SH/IEM artifact 有真实 pipeline，但普通 Scene 没有字段可引用它。
4. ProceduralSky 提供 horizon/zenith/ground、sun direction/color/intensity/angular radius、environment intensity/rotation，但与 Scene DirectionalLight 无 identity/revision/clock binding。
5. 默认 gradient sun intensity 为 0；DirectionalLight 另有 direction/color/intensity/volumetric，World light extract 不查询 Sky/Weather generation。
6. real/virtual/fixed time 具备 pause、speed、delta、frame index、fixed-step budget，但没有 date/calendar/day length/latitude/longitude/timezone/season/axial tilt/solar elevation/moon phase。
7. `02:00-04:00` 无法证明是 real/virtual/sequence/celestial time；Editor timeline 没有 clock source 或 scrub receipt。

### 2.4 Sky、Cloud、IBL、Fog 事实

1. sky shader 是 horizon/zenith/ground gradient 加 smoothstep sun disk，不是 Rayleigh/Mie/ozone multi-scattering atmosphere。
2. 没有 transmittance/multi-scattering/sky-view/aerial perspective LUT 的 production compiler/artifact。
3. realtime IBL scheduler 有 source revision key、A/B slot、generation、retry/stale/advance/publish 和 PMREM/SH time slicing，是应保留的局部工程化链。
4. 首次更新会安排 Sky/Cloud 六面 capture、mip、PMREM、SH；没有 weather dirty-subsystem mask 或 GPU deadline 自适应。
5. `CaptureCloud` 与 `CaptureSky` 都进入相同 `record_capture()`、相同 ProceduralSkyParams、相同 gradient shader 和 source mip。
6. Cloud capture 没有 density/material/weather map/lighting/transmittance/composite 输入，不能称为 volumetric cloud。
7. publish 只有 slot switch，没有旧/新环境 radiance 时域 crossfade、weather snapshot 或 consumer receipt。
8. Volumetric Fog 有 density/albedo/phase/height/temporal 参数，但没有 humidity/aerosol/rain extinction/weather binding。
9. Exposure/Fog/Sky/Cloud/DirectionalLight 没有共享 Environment generation 或 dependency graph。

### 2.5 Wind、Precipitation、Particles、Surface、Audio 事实

1. production 中没有 WindField、WeatherFrameSnapshot、Precipitation、rain/snow/hail intensity、wetness、snow accumulation 或 snow mask type。
2. Particle emitter 有 shape、lifetime、velocity、gravity、drag、rate/burst、CPU/GPU backend、local/world space，是可复用 executor。
3. CPU 可消费 asset 固定 external_force；它不是按位置、时间、altitude 或 Weather snapshot 查询的风场。
4. GPU frame layout 只编码 gravity/drag，没有 `ParticlePhysicsOptions.external_force`，CPU/GPU 语义不一致。
5. 没有 camera-relative precipitation volume、surface collision/splash/decal/puddle、snow accumulation、streaming budget。
6. Weather 没有调用 particle instantiate/tick/build_extract，也没有把 intensity/temperature/wind 转成 typed emission/material。
7. Terrain 没有 weather/wetness/snow/erosion input；Material 没有 surface-state buffer owner。
8. Sound 有 source/listener/volume 底座，没有 rain/wind/thunder ambience、weather snapshot 或 timed lightning event。
9. 没有 Lightning scheduler/seed/bolt geometry/flash/exposure impulse/cloud illumination/thunder delay/gameplay authority。
10. Region dropdown 没有绑定 Editor37 SpatialRegion source/geometry/generation、altitude/front/priority blend。

## 3. P0：必须先关闭的断路（5 项，全部 Open）

### P0-1：Weather/Climate source、compiler、service、artifact 全部缺失

没有 source asset、Scene component、first-party owner、compiler、artifact、runtime service 或 install receipt。必须先建立 `ClimateProfile`、`CelestialProfile`、`WeatherPreset/Timeline`、region binding 与 deterministic program artifact，不能继续做静态 workspace。

### P0-2：Scene environment 只有 preview bool，太阳/时钟不一致

World 只读取 `preview_skybox` 并返回固定 gradient；ProceduralSky sun 与 DirectionalLight 分裂，runtime clock 没有日历/地理/天体权威。必须先定义 CelestialClock/EnvironmentFrameSnapshot 和 Scene/PIE/save/network/replay 读写边界。

### P0-3：Cloud capture 是重复 gradient，不是云系统

`CaptureCloud` 与 `CaptureSky` 复用同一 recorder/shader/source mip，没有 cloud density、shadow、transmittance、weather input 或 composite。必须先建立真正的 cloud source/field/render artifact，或明确 capability unsupported。

### P0-4：Particle/Sound/Surface 没有 Weather adapter

粒子没有 wind/rain/snow/surface accumulation，CPU/GPU external force 不一致；Terrain/Material/Sound 没有 wetness/snow/ambience/lightning consumer。必须先建立 immutable weather snapshot 与 typed adapters，不能把粒子 preset 接到按钮上。

### P0-5：Weather Editor 是第二 authority

固定资产、region、timeline、layer、warnings、queued feedback 不读 Runtime/Scene/IBL/Fog/Particle/Sound/Terrain/Region。必须删除或改为真实 document/compiler/runtime preview/toolkit，所有 build/preview 产生 receipt。

## 4. P1：Runtime、Environment、Weather 与 Editor（70 项，全部 Open）

1. 定义 versioned `ClimateProfileAsset` 与 unknown-field-preserving migration。
2. 定义 `CelestialProfileAsset`、天体 identity、轨道/轴倾角/地理坐标/时区。
3. 定义 `CelestialClock`、calendar/day length、pause/rate、network/replay/offline authority。
4. 输出 immutable `CelestialFrameSnapshot`，携带 generation、sun/moon direction/elevation、day/season。
5. Scene/Prefab/World 保存 Weather/Climate/Celestial references、revision、override provenance。
6. 定义 versioned `WeatherPresetAsset`、state variables、seed、transition policy。
7. 定义 `WeatherTimeline/TransitionGraph`、key/curve/event、scrub/loop/branch semantics。
8. 定义 `WeatherRegionBinding`，引用 Editor37 SpatialRegionId、layer、priority、weight/combine policy。
9. 建立 Weather compiler、validator、dependency graph、deterministic artifact digest。
10. 生成 `WeatherProgramArtifact`，记录 source/tool/schema/algorithm/platform dependencies。
11. 建立 generation-qualified `WorldWeatherService`、authority/lease、load/unload/hot reload。
12. 输出 immutable `WeatherFrameSnapshot`，禁止各 subsystem 读取不同步的散乱 fields。
13. 建立 changed-subsystem dirty mask、budget、priority、crossfade、rollback receipt。
14. Weather diagnostic code 关联 source/region/clock/generation/subsystem/receipt。
15. Sky component 支持 source cubemap/procedural/physical sky mode、rotation、intensity、exposure。
16. Sky sun/moon 与 DirectionalLight identity、color/temperature/intensity、shadow revision 绑定。
17. 建立 atmospheric optical properties：Rayleigh/Mie/ozone/turbidity/ground albedo。
18. 实现 transmittance/multi-scattering/sky-view/aerial perspective LUT compiler/cache。
19. 记录 LUT/artifact resolution、precision、platform tier、invalid/rebuild state。
20. Cloud source 支持 density/coverage/erosion/weather map/height profile/animation seed。
21. Cloud render 支持 shadow map/lighting/transmittance/composite/temporal history。
22. 将 CaptureSky/CaptureCloud 拆为不同 graph/resource/inputs，禁止共享 gradient recorder。
23. Cloud/sky/IBL publish 支持 A/B generation、crossfade、stale drop、device loss recovery。
24. IBL dirty policy 由 Weather snapshot/sky/cloud change 驱动，按 GPU time budget 分片。
25. PMREM/SH/IEM artifact 记录 environment generation、platform variant、consumer receipt。
26. Fog 绑定 humidity/aerosol/visibility/rain/height/weather state，并保持独立 typed evaluator。
27. Fog temporal history、froxel budget、camera/viewport generation 与 weather crossfade 对齐。
28. Wind field 定义 direction/speed/gust/turbulence/altitude/region/seed/time sampling。
29. Wind provider 支持 global/local/volumetric/terrain channel、query snapshot、budget。
30. Particle adapter 将 weather intensity/temperature/wind 转为 deterministic emitter params。
31. CPU/GPU particle frame schema 对齐 external force、wind、collision、surface response。
32. Precipitation profile 支持 rain/snow/hail type、rate/size/velocity/temperature/visibility。
33. Camera-relative spawn volume、world streaming、LOD、budget、drop policy 与 generation。
34. Surface adapter 支持 hit/impact/splash/decal/puddle/wetness/snow accumulation artifact。
35. Terrain provider 支持 snow/wetness/erosion/material mask与 incremental update。
36. Material/global surface-state buffer 记录 weather generation、wetness/snow/ice/roughness。
37. Sound adapter 支持 rain/wind/ambient layer、listener/source region、reverb/occlusion。
38. Lightning scheduler 支持 seed、bolt graph、flash light/exposure、cloud illumination、thunder PTS。
39. Lightning gameplay/audio/network/replay/save receipt 与 Celestial/Weather generation 对齐。
40. Weather network authority、prediction、rollback、replication、late client join。
41. Weather save/load/migration、checkpoint、replay、deterministic seed 与 offline bake。
42. World Partition/cell streaming 绑定 weather region lease、prefetch、priority、cancel、rollback。
43. Multi-world/multi-viewport/camera/listener/player weather isolation。
44. Platform capability matrix：atmosphere/cloud/fog/particles/HDR/compute/temporal fallback。
45. malformed profile/curve/seed/NaN/overflow/huge LUT/particle budget fuzz。
46. clean/cold/warm artifact cache、atomic publication、GC、size/age/platform key。
47. background compile/bake/preview jobs bounded/cancellable/retry/shutdown drain。
48. scale baseline 1/1k/100k regions、particles、cloud tiles、weather transitions、GPU budget。
49. Runtime diagnostics expose clock, weather generation, dirty subsystem, cloud/IBL state、drops。
50. Editor AssetType/ResourceKind for Climate/Celestial/Weather/Preset/Timeline/Region binding。
51. Source document revision、dirty/save/autosave/recovery/conflict/undo transaction。
52. schema-driven inspector shows resolved clock/region/curves/subsystem capabilities/diagnostics。
53. timeline/curve editor manipulates real Weather document with typed key/event/seed。
54. Region binding editor uses Editor37 spatial source/gizmo/weight/priority/query snapshot。
55. Sky/atmosphere/cloud preview consumes real artifacts, generation, camera and time scrub。
56. Weather preview simulates deterministic clock/seed and displays frame/subsystem receipt。
57. rain/snow/wind/fog/surface/audio/lightning preview consumes real runtime adapters。
58. Build/Preview/Apply/Playtest routes submit jobs and return job/generation/artifact receipts。
59. first-party runtime/editor catalog/App target explicitly assembles each provider/factory/toolkit。
60. plugin admission validates module, resource URI, operation/controller/service and capability。
61. Workbench removes fixed Weather_Storm/8 layers/5 regions/2 warnings fixture。
62. Editor diagnostics can filter source/clock/region/subsystem/generation and export evidence。
63. Scene/prefab/PIE/network/save/reimport/hot reload roundtrip preserves weather identity and overrides。
64. visual/audio/data golden for sky/cloud/fog/IBL/particle/surface/lightning/clock transitions。
65. failure injection for cache corruption, device loss, cancel, disk full, worker panic, late publish。
66. cross-platform clean headless cook/package and client/server/editor startup matrix。
67. environment quality/performance tests record p50/p95/p99 build/GPU/VRAM/frame hitch。
68. provider/algorithm/schema upgrades support canary, old generation pin, rollback and migration。
69. release manifest includes actual climate/weather/environment artifacts and dependency provenance。
70. compare methods with Unreal/Godot/Bevy/Fyrox/Unity HDRP for fidelity, memory, latency and determinism。

## 5. P2：长期能力（12 项，全部 Open）

1. physically based atmosphere with multi-scattering, ozone, aerial perspective and LUT streaming。
2. hardware/compute volumetric cloud with sparse weather fields, shadow maps and temporal reprojection。
3. global weather simulation fronts, pressure, humidity, thermodynamics and climate data import。
4. terrain snow/ice/wetness/erosion accumulation with streaming sparse surface masks。
5. ocean/wave/fog/rain interaction, splash/foam, wind-driven water and shoreline weather。
6. multi-body celestial system, eclipse, moon phase, starfield, aurora and calendar localization。
7. procedural lightning/thunder propagation, acoustic delay, network synchronized spectacle。
8. remote weather provider/live forecast ingestion with cache, permission, deterministic fallback。
9. neural/upscale/weather denoiser provider within artifact/quality/fallback contract。
10. collaborative climate/weather timeline editing, field merge, lock, review annotations。
11. weather program schema/algorithm migration, canary rollout, rollback and replay compatibility。
12. cross-engine climate/weather benchmark and public reference scenes/methodology。

## 6. 分层重构顺序

### M0：Truthfulness 与第二 authority 清理

冻结 Weather stable capability；将 Weather workspace、Cloud operation、weather fixture strings 与 catalog false positives 标为 fixture/unsupported；保留 Sky/IBL/Fog/Particle/Clock 底座但删除“Weather complete”声明。

### M1：Clock、Climate、Celestial 与 Region source

建立 versioned Climate/Celestial/Weather source、CelestialClock、SpatialRegion binding、Scene/World persistence、network/replay authority。

### M2：Weather compiler 与 immutable snapshot

建立 validator/compiler/program artifact、dependency/provenance、generation service、immutable WeatherFrameSnapshot、dirty/crossfade/budget/diagnostics。

### M3：Sky、Atmosphere、Cloud、IBL、Fog adapters

拆分 Sky/Cloud graph，建立 physical LUT/cloud input/shadow/transmittance/composite、IBL generation/crossfade 与 fog/weather binding；禁止重复 gradient capture。

### M4：Wind、Precipitation、Surface、Particle、Audio、Lightning

建立 typed adapters、CPU/GPU schema parity、surface accumulation、wetness/snow material state、audio/lightning/network/save/replay receipts。

### M5：Editor toolkit/preview/build

接入 AssetType、document/transaction、timeline/curves/regions、artifact-aware preview、background jobs、catalog/App/provider closure；所有 workspace feedback 来自真实 receipt。

### M6：Platform、Fault、Scale、Release

完成 malformed/fault/determinism/visual/audio/data/scale/cross-platform/headless package/rollback 门禁；未通过前 capability 不得 Stable。

## 7. 验收门禁（32 门，当前全部 Fail）

1. Climate/Celestial/Weather/Region source、revision、artifact、instance、generation identity 完整。
2. CelestialClock date/geography/season/day length/pause/rate/network/replay deterministic。
3. Weather compiler/program artifact key、dependency、seed、migration、platform variant 正确。
4. immutable WeatherFrameSnapshot 与 dirty subsystem/crossfade/budget/diagnostics 可追踪。
5. Scene/Prefab/PIE/Save/Network/Replay roundtrip 保持 environment/weather authority。
6. Sky/DirectionalLight/sun/moon identity、color/temperature/shadow 与 clock 一致。
7. atmosphere LUT numeric/visual quality、precision、cache/rebuild/fallback 通过 golden。
8. cloud density/coverage/erosion/lighting/shadow/transmittance/temporal/composite 真实有效。
9. CaptureSky 与 CaptureCloud graph/input/resource 不重复且 artifact generation 正确。
10. IBL PMREM/SH/IEM generation、crossfade、stale/device loss、consumer receipt 正确。
11. Fog humidity/aerosol/visibility/rain/height/temporal 与 weather generation 正确。
12. Wind direction/speed/gust/turbulence/altitude/region/seed query deterministic。
13. CPU/GPU Particle weather/external-force schema parity、LOD、budget、cancel 无分叉。
14. rain/snow/hail spawn/velocity/camera-relative/streaming/impact/surface accumulation golden。
15. Terrain/Material wetness/snow/ice/roughness masks与generation更新正确。
16. Sound weather ambience/reverb/listener/source/lightning PTS 与 output receipt 正确。
17. Lightning geometry/flash/exposure/cloud/thunder/network/replay/save deterministic。
18. world/cell/player/camera/listener multi-context isolation 与 late generation fence。
19. malformed/NaN/overflow/huge LUT/profile/particle/curve fuzz 无 panic/OOM。
20. cache atomic publication/GC/size/age/platform key/rollback 不留下半工件。
21. jobs bounded/cancel/retry/shutdown/worker panic/disk full/device loss 可恢复。
22. Editor AssetType/document/transaction/undo/save/autosave/recovery/conflict 保持 source。
23. timeline/curve/region/sky/cloud preview 使用真实 document/artifact/runtime snapshot。
24. Build/Preview/Apply/Playtest 返回 job/source/generation/artifact/diagnostic receipt。
25. fixed Weather workspace facts 全部由真实 state 替代，missing/error 状态可见。
26. plugin/catalog/App/provider/factory/controller/service admission 闭合。
27. visual/audio/data golden 覆盖 sky/cloud/fog/IBL/particle/surface/lightning/time transitions。
28. 1/1k/100k regions/particles/cloud tiles/transitions、GPU/VRAM/frame hitch 达标。
29. desktop/mobile/web atmosphere/cloud/fog/compute/temporal capability matrix headless 验证。
30. diagnostics 可按 source/clock/region/subsystem/generation/drop/receipt 导出。
31. schema/algorithm/provider upgrades 支持 canary、old generation pin、rollback、replay compatibility。
32. Stable/Complete 只能由 compile、registration、runtime、Editor、fault、platform、scale evidence 派生。

## 8. 禁止的临时修补

1. 禁止把固定 Weather_Storm、时间线、层数、region、warning、queued feedback 当作 Weather 产品。
2. 禁止用 `preview_skybox` bool、gradient shader 或 DirectionalLight 字段拼接成 Environment authority。
3. 禁止把重复 CaptureSky/CaptureCloud gradient 写入 source cubemap 称为 cloud/IBL integration。
4. 禁止把静态 simulation clock/delta、particle external_force 或 Sound volume strongest 称为 Weather state。
5. 禁止只增加 weather/cloud/wind/rain ResourceKind、manifest 或 UI fields 而没有 compiler/artifact/service/adapter。
6. 禁止 CPU/GPU particle 继续接受不同 physics/weather inputs 而无 actual receipt。
7. 禁止在 render/particle/audio thread 同步执行 weather compile、LUT/cloud bake 或无界数据复制。
8. 禁止把 Terrain/Material/Sound/Gameplay 固定参数当作 wetness/snow/lightning/weather consumer。
9. 禁止用 test attribute、ignored GPU test、手工截图替代 32 门资格。
10. 禁止在重新导出 215-file manifest/fingerprint 前实施本报告假设，或通过 lockfile drift 绕过 `--locked`。

## 9. 本轮产出边界

本轮只新增 Editor112 review、索引与分层计划，没有修改 Runtime、Editor、Interface、Plugin、App 或 tests production code，也没有运行 Cargo、weather simulation、GPU cloud/IBL、particle/audio/lightning、Scene roundtrip 或跨平台动态验证；未查询或实时跟踪协调器。实施必须从 M0 开始，先恢复编译基线并建立 Climate/Celestial/Weather/Region owner inventory，再实现任何 Weather UI 或 Cloud capability。
