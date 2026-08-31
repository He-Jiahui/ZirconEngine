---
title: Editor Weather、Climate、Celestial、Atmosphere、Cloud 与 Wind 当前工作树复审
category: zircon_editor
report_id: Editor244
review_date: 2026-08-30
baseline_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
verification_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
canonical_owner: Editor244
related_runtime_owner:
  - docs/plans/optimize/zircon_runtime/184-runtime-weather-climate-atmosphere-current-working-tree-review.md
related_code:
  - zircon_editor/src/core/asset/type_registry
  - zircon_editor/src/core/editing
  - zircon_editor/src/scene
  - zircon_editor/src/scene/viewport
  - zircon_editor/src/ui/asset_editor
  - zircon_editor/src/ui/preview_scene
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_weather_editor_workspace.zui
  - zircon_editor/src/core/extension/inspector.rs
  - zircon_plugins/first_party_editor_catalog/src
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/WorldBrowser
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/SkyAtmosphereComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/VolumetricCloudComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/WindDirectionalSourceComponent.h
  - dev/UnrealEngine/Engine/Plugins/Experimental/DaySequence/Source/DaySequence
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Editor/Lighting
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Sky
  - dev/godot/scene/3d/world_environment.cpp
  - dev/godot/scene/3d/fog_volume.cpp
  - dev/bevy/crates/bevy_light/src/atmosphere.rs
  - dev/Fyrox/fyrox-impl/src/scene/skybox.rs
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Editor244 · Weather/Climate/Atmosphere authoring 当前工程化差距

## 1. 结论

当前 Editor 没有 Weather/Climate/Celestial/Atmosphere/Cloud/Wind/Precipitation/SurfaceState asset factory、document、timeline compiler 或 runtime provider。`builtin.rs`/`toolkit.rs` 没有 Weather 类型；`workbench_extension_weather_editor_workspace.zui` 的根节点为 `collapsed`，并硬编码 `Weather_Storm`、`Region_Mountains`、`Layer_Clouds`、Cloud Build、Rain Burst、Wind Gust、Lightning、`8 layers / 5 regions / 2 warnings`。

通用 time controls、environment preview、viewport/capture、Scene transaction、Inspector 与 diagnostics 只能作为宿主。它们没有 calendar/season/region identity、celestial curve、cloud/wind/precip layers、surface state、weather query、provider generation、compiler artifact、PreviewWorld、save/reopen/network mirror 或 failure receipt。旧 Editor215/159/112/38 的 fixture/authoring 差距在当前树仍存在；本报告登记 **0 项新 P0、28 项 P1、10 项 P2、24 道资格门**，P1 28 Open，P2 10 Open，资格门 21 Fail、3 Partial、0 Pass。

## 2. 当前源码证据

- editor asset registry 没有 ClimateProfile/WeatherProgram/Region/WindField/Cloud/Precipitation/SurfaceState 类型、factory、thumbnail 或 toolkit。
- Weather Workbench 的 tabs、rows、ranges、status 和 Preview/Build routes 是静态组件，未连接 operation factory、document revision、transaction/undo、compiler job 或 artifact generation。
- Inspector 中可出现 weather component IDs/fields，但缺少 runtime provider admission、stable entity/region ID、field schema、save/reopen 与 generation-qualified mirror；测试 extension registration 不能当产品。
- PreviewScene/viewport 没有 celestial/atmosphere/cloud volume、wind vectors、precipitation bounds、surface wetness/snow、lightning event 或 query overlay。
- Render/Runtime Diagnostics 没有 weather layer/region/clock/cloud/fog/wind/precipitation/LUT/GPU/CPU metrics，Capture 也没有 weather provenance。

## 3. 参考引擎差异

Unreal 的 SkyAtmosphere/VolumetricCloud/WindDirectionalSource/DaySequence 与 World Browser 提供组件 authoring、curve/sequence、planet/region、preview、transactions 和 renderer handoff；Unity HDRP lighting/sky/cloud editor 有 physical parameters、cloud maps、quality/volume/history；Godot WorldEnvironment/FogVolume、Bevy atmosphere、Fyrox skybox 是较小的 scene resource 对照。Zircon 当前只是 fixture。

## 4. P1 重构任务

| ID | 差异 | 必须完成 |
|---|---|---|
| ED-WTH-01 | 无 asset types | ClimateProfile/Celestial/WeatherProgram/Region/WindField/Cloud/Precipitation/SurfaceState 类型。 |
| ED-WTH-02 | 无 provider/catalog | editor provider/manifest、runtime capability、unsupported state、receipt-driven status。 |
| ED-WTH-03 | 无 document | stable weather/layer/region/curve/event IDs、revision、dirty/save/reopen/LKG/migration。 |
| ED-WTH-04 | 无 calendar editor | epoch/day/year/season/timezone、cycle/rounding/overflow/units fields and validation。 |
| ED-WTH-05 | 无 celestial editor | sun/moon/stars, ephemeris/curves, photometry, planet/up-axis/origin preview。 |
| ED-WTH-06 | 无 region authoring | region/cell polygons, overlap/priority/blend, partition and dependency visualization。 |
| ED-WTH-07 | 无 layer authoring | cloud/fog/wind/rain/snow/hail/lightning/surface layers with typed fields。 |
| ED-WTH-08 | 无 wind authoring | vector/gust/turbulence/altitude profile, field gizmo and query samples。 |
| ED-WTH-09 | 无 cloud/atmosphere preview | physical sky, cloud maps/noise, transmittance/shadow/history and fallback status。 |
| ED-WTH-10 | 无 precipitation preview | bounded camera-relative particles, phase/intensity/impact/temperature visualization。 |
| ED-WTH-11 | 无 surface state | wetness/snow/puddle/melt/deposition channels and terrain/material map preview。 |
| ED-WTH-12 | 无 timeline/transition | condition/duration/hysteresis/priority/cooldown curve editor with deterministic ordering。 |
| ED-WTH-13 | 无 compiler job | dependency graph、source spans、progress/cancel、artifact generation/install/rollback。 |
| ED-WTH-14 | 无 PreviewWorld | runtime artifact/provider install、fixed-step、pause/step/reset/seek/device/world generation。 |
| ED-WTH-15 | 无 query debugger | point/region query、temperature/humidity/wind/visibility/precip/cloud result、generation/provenance。 |
| ED-WTH-16 | 无 event debugger | lightning/transition/enter-exit/impact event cursor、authority、dedupe、replay。 |
| ED-WTH-17 | 无 runtime mirror | world/clock/program/region/layer generation、provider/status/terminal reason。 |
| ED-WTH-18 | 无 renderer diagnostics | atmosphere/cloud/fog/IBL/exposure/shadow/particle/wind LUT/GPU metrics。 |
| ED-WTH-19 | 无 commands | layer/curve/region/asset/transition changes via operation factory/history/dirty participant。 |
| ED-WTH-20 | 静态 fixture 风险 | Weather_Storm/Cloud Build/Rain Burst/Wind Gust/Ready 文案必须由 runtime receipt 驱动。 |
| ED-WTH-21 | 无 roundtrip | source/artifact/settings save/reopen/migrate preserve IDs, curves, unknown fields。 |
| ED-WTH-22 | 无 product scene | Scene/PIE/standalone/save/reopen with sky/cloud/fog/wind/precip/surface/gameplay/audio。 |
| ED-WTH-23 | 无 network/save | server authority, state digest, checkpoint/replay, join-in-progress and conflict UI。 |
| ED-WTH-24 | 无 collaboration | document lease、external change/rebase、operation provenance。 |
| ED-WTH-25 | 无 quality controls | cloud/fog/precip/wind quality tiers, budget, memory/GPU time and fallback。 |
| ED-WTH-26 | 无 fault UI | invalid profile, missing LUT, provider/device/world loss, stale region and rollback。 |
| ED-WTH-27 | 无 tests | authoring/property/roundtrip/compiler/preview/query/visual/fault/scale/soak。 |
| ED-WTH-28 | ABI boundary absent | versioned neutral descriptors; editor cannot write renderer/physics/audio state directly。 |

## 5. P2 增强任务

| ID | 差异 | 工程化方向 |
|---|---|---|
| ED-WTH-P2-01 | 缺天气历史 | persisted weather history、replay scrubbing、branch/correction 与 migration。 |
| ED-WTH-P2-02 | 缺空间插值 | region blending、altitude bands、front propagation 与 boundary hysteresis authoring。 |
| ED-WTH-P2-03 | 缺 celestial 高级模型 | eclipse、aerial visibility、seasonal orbit、starfield calibration。 |
| ED-WTH-P2-04 | 缺云体细节工具 | multi-scale noise、erosion、self-shadow、phase、tracing cache controls。 |
| ED-WTH-P2-05 | 缺降水碰撞 | surface impact、splash/snow accumulation、shelter/occlusion preview。 |
| ED-WTH-P2-06 | 缺湿度/积雪材质 | material channel packing、virtual texture feedback、terrain partial update。 |
| ED-WTH-P2-07 | 缺天气音频空间化 | ambience zones、occlusion、wind/precip one-shots、distance/mix priorities。 |
| ED-WTH-P2-08 | 缺 weather-driven VFX | particle inputs、lightning/impact events、deterministic effect lifetime。 |
| ED-WTH-P2-09 | 缺移动/低端降级 | LUT/cloud/precip quality controls and explicit unsupported state。 |
| ED-WTH-P2-10 | 缺观测导出 | frame/region/layer/query/event traces、capture provenance、offline comparison。 |

## 6. 资格门

| 门 | 结果 | 关闭证据 |
|---|---|---|
| type/provider/catalog | Fail | types/provider/capability/unavailable state consistent。 |
| plugin extension | Fail | weather editor provider, panels, overlays and lifecycle registration。 |
| document identity | Fail | weather/layer/region/curve/event IDs, revision and unknown-field migration。 |
| calendar editor | Fail | epoch/day/year/season/timezone/cycle fields and unit validation。 |
| celestial editor | Fail | sun/moon/stars, ephemeris, photometry, planet/up-axis preview。 |
| region authoring | Fail | region/cell polygons, overlap/priority/blend and partition dependencies。 |
| layer authoring | Fail | cloud/fog/wind/rain/snow/hail/lightning/surface typed fields。 |
| wind/query tools | Fail | vector/gust/turbulence gizmo and generation-qualified query samples。 |
| compiler job | Fail | dependency graph, source spans, progress/cancel and artifact rollback。 |
| preview world | Fail | runtime provider install, fixed-step pause/seek/reset and device/world generation。 |
| cloud/atmosphere preview | Fail | physical sky, cloud maps/noise, transmittance/shadow/history and fallback。 |
| precipitation/surface preview | Fail | bounded particles, impact, wetness/snow/puddle/melt channels。 |
| timeline/transition | Fail | condition/duration/hysteresis/priority/cooldown curves with deterministic order。 |
| query debugger | Fail | point/region values with generation, provenance and bounded cursor。 |
| event debugger | Fail | lightning/transition/enter-exit/impact authority, dedupe and replay。 |
| runtime mirror | Fail | world/clock/program/region/layer/provider status and terminal reason。 |
| renderer diagnostics | Partial | generic render/diagnostic panels exist, but no weather layer/LUT/GPU metrics。 |
| operation/transaction | Partial | generic Scene transaction host exists, but no weather operation factory/history。 |
| round-trip | Fail | source/artifact/settings save/reopen/migrate preserve IDs, curves and unknown fields。 |
| product scenes | Fail | Scene/PIE/standalone/save/reopen sky/cloud/fog/wind/precip/surface cases。 |
| quality controls | Fail | cloud/fog/precip/wind tiers, budgets, memory/GPU time and fallback。 |
| fault handling | Fail | invalid profile/LUT/provider/device/world loss and stale region rollback。 |
| backend host | Partial | generic time/environment/viewport/PreviewScene host exists, weather provider absent。 |
| test coverage | Fail | authoring/property/roundtrip/compiler/preview/query/visual/fault/scale/soak。 |

本轮仅写审查文档，未修改生产代码、测试、Cargo、ABI 或 ZUI，也未运行 Editor/Weather/GPU/PIE 动态验证。
