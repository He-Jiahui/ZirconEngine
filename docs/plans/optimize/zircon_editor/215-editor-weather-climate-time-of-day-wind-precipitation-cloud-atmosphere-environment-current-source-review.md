---
title: Editor Weather、Climate、Time-of-Day、Wind、Precipitation、Cloud、Atmosphere 与 Environment 当前源码复核
category: zircon_editor
report_id: Editor215
review_date: 2026-08-29
baseline_head: a2d8d811c4a3a1fc1db6f5375c491e7e4502533f
verification_head: a2d8d811c4a3a1fc1db6f5375c491e7e4502533f
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
canonical_owner: Editor38
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/38-weather-climate-time-of-day-wind-precipitation-cloud-atmosphere-environment-authoring-review.md
  - docs/plans/optimize/zircon_editor/112-editor-weather-climate-time-of-day-wind-precipitation-cloud-atmosphere-current-source-review.md
  - docs/plans/optimize/zircon_editor/159-editor-weather-climate-time-of-day-wind-precipitation-cloud-atmosphere-environment-current-source-review.md
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/world/workbench_extension_weather_editor_workspace.zui
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_feedback.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/world_building/volume_and_weather.rs
  - zircon_editor/src/ui/template_runtime/builtin/workbench_extension_module_template_bindings/world_building.rs
  - zircon_editor/src/core/asset/type_registry
  - zircon_runtime/src/asset/assets/scene
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world_time
  - zircon_runtime/src/core/framework/time
  - zircon_runtime/src/core/runtime/time.rs
  - zircon_runtime/src/core/framework/render/environment
  - zircon_runtime/src/core/framework/render/advanced_lighting/volumetric.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment
  - zircon_plugins/rendering/features/volumetric_fog
  - zircon_plugins/particles
  - zircon_plugins/terrain
  - zircon_plugins/sound/runtime/src/engine/source_environment
  - zircon_plugins/first_party_runtime_catalog/src/lib.rs
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
  - zircon_app/Cargo.toml
tests:
  - zircon_runtime/src/core/framework/time
  - zircon_runtime/src/scene/world_time
  - zircon_runtime/src/core/framework/render/environment
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_wgpu_recorder/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_runtime/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/environment/realtime_ibl_time_slice/tests.rs
  - zircon_plugins/particles/runtime/src/tests
  - zircon_plugins/rendering/features/volumetric_fog/runtime/src/tests.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/extension_module_navigation/specs/world_building/volume_and_weather.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/38-weather-climate-time-of-day-wind-precipitation-cloud-atmosphere-environment-authoring-review.md
  - docs/plans/optimize/zircon_editor/112-editor-weather-climate-time-of-day-wind-precipitation-cloud-atmosphere-current-source-review.md
  - docs/plans/optimize/zircon_editor/159-editor-weather-climate-time-of-day-wind-precipitation-cloud-atmosphere-environment-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zx-runtime-weather-climate-celestial-time-of-day-wind-precipitation-cloud-atmosphere-surface-state-determinism-network-save-scalability-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/96-runtime-environment-sky-atmosphere-cloud-ibl-reflection-probe-capture-convolution-sh-pmrem-cache-residency-submission-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/214-editor-volume-zone-trigger-region-gameplay-audio-post-process-environment-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/SkyAtmosphereComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/VolumetricCloudComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/WindDirectionalSourceComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/WindDirectionalSource.cpp
  - dev/UnrealEngine/Engine/Plugins/Experimental/DaySequence/Source/DaySequence
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Sky
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/VolumetricClouds
  - dev/godot/scene/resources/environment.cpp
  - dev/godot/scene/3d/world_environment.cpp
  - dev/godot/scene/3d/fog_volume.cpp
  - dev/bevy/crates/bevy_light/src/atmosphere.rs
  - dev/bevy/crates/bevy_pbr/src/atmosphere
  - dev/Fyrox/fyrox-impl/src/scene/skybox.rs
finding_status:
  p0_open: 5
  p0_partial: 0
  p0_closed: 0
  p1_open: 32
  p1_partial: 38
  p1_closed: 0
  p2_open: 12
  p2_partial: 0
  p2_closed: 0
gate_status:
  fail: 18
  partial: 14
  pass: 0
---

# 215 · Editor Weather / Climate / Time-of-Day / Wind / Precipitation / Cloud / Atmosphere / Environment 工程化差距

## 1. 结论

Editor38 的 canonical 结论仍成立：Zircon 当前没有可供项目使用的 Weather/Climate authoring 产品。对 **19,584** 个 `zircon_*` 生产 Rust/TOML/ZUI 文件做一次读取的精确合同扫描后，`ClimateProfileAsset`、`CelestialProfileAsset`、`WeatherPresetAsset`、`WeatherTimeline`、`WeatherProgramArtifact`、`WorldWeatherService`、`WeatherFrameSnapshot`、`WindField`、`SurfaceWeatherState`、`StrikeId`、`WeatherRegionBinding`、`CloudProgramArtifact`、`PrecipitationProfile` 与 `CelestialClock` 全部为零命中。没有 Weather/Climate 首方 package、ResourceKind、Scene binding、compiler、World service、network/save codec 或 typed domain adapter。

Weather workspace 继续充当第二 authority：生产 ZUI 固定 `Weather_Storm`、`Region_Mountains`、`Layer_Clouds`、四段时间线、8 layers、5 regions、2 warnings、Preset/Region/Blend Time；callback 固定返回 opened/preview queued/build queued/Rain Burst/Lightning 文本。navigation、template binding 与 allow-list 只证明 route 可达，不证明 document、transaction、job、artifact、runtime snapshot 或 terminal receipt 存在。

当前可保留的底座继续增强，但没有改变产品判定。每个 World 有 virtual/fixed clock、pause/rate、fixed debt、transaction 与 immutable `WorldTimeSnapshot`；Environment 有 source cubemap、PMREM/SH9/IEM、reflection probe、generation-aware realtime IBL 与 last-good 方向；Fog 有 typed froxel/history；Particle 有 CPU/GPU backend；通用 asset/compiler/DDC、operation、network、save、diagnostics 与 Editor document 基础可复用。这些基础使 38 项 P1 和 14 个门禁维持 `Partial`，却没有 Weather identity、program generation 与跨域同代真值。

本轮重新确认两个容易误判的边界。第一，`EnvironmentExtract` 仍只有 skybox、probes、baked lighting 和 probe grid，`build_environment_extract()` 仍只读取不可序列化的 `preview_skybox`；Scene entity schema没有 Environment/Weather。第二，Particle CPU仍叠加 asset gravity与 physics external force，GPU emitter只上传 gravity，CPU/GPU 外力语义不一致。历史伪 `CaptureCloud` 已删除，但 Cloud source/medium/lighting/shadow/history/artifact 仍为零，不能将删除错误路径记作 Cloud 产品完成。

目标硬边界保持为：

```text
Climate/Celestial/Weather/Region source documents
  -> deterministic WeatherCompiler + dependency/target admission
  -> immutable WeatherProgramArtifact
  -> per-World generation-qualified WorldWeatherService + CelestialClock
  -> atomic WeatherFrameSnapshot + bounded query/event cursor
  -> typed Atmosphere/Cloud/IBL/Fog/Wind/Precipitation/Surface/
     Particle/Sound/Lightning/Gameplay adapters
  -> transactional Editor toolkit + qualified preview/build/apply receipts
```

Weather拥有状态、确定性和跨域同代真值；render、particle、terrain、material、sound 与 gameplay owner 拥有执行资源和质量策略。不得建立万能 Environment property bag，也不得让 DirectionalLight、Fog、Particle 或 Editor 控件成为第二天气权威。

## 2. 审查范围与方法

### 2.1 当前物理选择集

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 当前指纹 |
|---|---:|---|
| Zircon Editor/Runtime/Plugin/App selected | **283 / 49,316 / 45,139 / 1,778,761 / 387 / 0** | `75ad8195ecee2269ed81d71cf73f026c609caab026eaa821462431e2f8e5e4c6` |
| Unreal/Unity Graphics/Godot/Bevy/Fyrox selected | **111 / 29,661 / 24,864 / 1,213,662 / 0 / 0** | `2cf53274c6ee73ca175368e8176a33a902d468eb83fd0416044536c34aa2f625` |
| 去重并集 | **394 / 78,977 / 70,003 / 2,992,423 / 387 / 0** | `75618ee53dfa034753c46db1f9acc04728f0df3b9cffe991989b2d63c8cbcd32` |

Zircon 选择集按 frontmatter 显式路径递归展开，根分布为 `zircon_runtime=150`、`zircon_plugins=112`、`zircon_editor=20`、`zircon_app=1`。指纹按规范化路径与当前文件 SHA-256 聚合；tests/ignored 只统计 Rust 属性。产品合同扫描限定 `zircon_*` roots，排除 `dev/`、`docs/`、`tools/`、`target/`、`.codex/` 和临时/worktree副本。Tooling 依用户要求不作为产品证据。

### 2.2 判定规则与动态边界

1. `Open` 表示 Weather 目标 contract、owner、artifact、consumer 或产品链不存在，或行为与目标冲突。
2. `Partial` 表示有可执行、可测试的通用/域内底座，但 Weather identity、generation、consumer 或资格证据未闭合。
3. `Closed/Pass` 必须有当前源码、产品装配、runtime执行与动态证据；本轮没有项目达到该等级。
4. 类型、descriptor、capability、ZUI、test fixture、固定 feedback 与 optional feature 不单独证明产品存在。
5. 本轮是静态 review，没有运行 Cargo、Editor、WGPU、audio、network、save、headless、fault、scale、soak 或跨引擎 benchmark。

共享工作树在 Environment、Time、Particle、Fog、Terrain、Sound、catalog/App 与 Weather ZUI 路径存在用户或其它 Session 的在途改动。本报告读取当前磁盘作为 authority，不归因、不覆盖、不回退；实施前必须重新冻结选择集与 generation contract。

## 3. 当前 Zircon 产品链

### 3.1 Package、Asset、Scene 与产品真相

1. `zircon_plugins`没有 Weather/Climate/Atmosphere/Cloud package；first-party runtime/editor catalog没有Weather provider。
2. builtin ResourceKind没有 Climate/Celestial/Weather/Preset/Timeline/Region Binding；App没有 Weather feature closure。
3. `SceneEntityAsset`固定保存 camera、mesh、lights、post process、physics、animation、terrain、tilemap、prefab与script bindings，没有Environment/Weather/Climate/Cloud/Wind。
4. script JSON、dynamic plugin ID 与测试中的 weather manifest 只能证明通用扩展机制，不能充当首方 runtime authority。
5. UI中出现的 Mountains/Coast/City/Interior 只是字符串选项，没有 Editor214 shared Region identity、compiled geometry 或 generation binding。

### 3.2 Editor：静态演示面，不是 authoring product

1. workspace固定 `Weather_Storm`、`Region_Mountains`、`Layer_Clouds`，没有project query、stable selection、loading/error/conflict state。
2. Timeline固定 Cloud Build、Rain Burst、Wind Gust、Lightning 与时间字符串；没有typed key/curve/event、seed、clock domain、scrub、loop或branch。
3. Preset/Region/Blend Time提交只穿过模板route；callback只改status/output text。
4. Preview/Build不产生operation ticket、source revision、artifact digest、World generation、diagnostic identity或terminal disposition。
5. 没有Weather document provider、operation factory、controller、background job、viewport overlay、preview scene、runtime bridge或catalog registration。

### 3.3 Clock 与 Celestial：时间底座不是天体权威

1. Runtime time区分 MonotonicReal、WallUtc、WorldVirtual、WorldFixed、Input、Render、Audio、Network、Media与EditorPreview domain。
2. `WorldTimeController`支持pause/rate、fixed debt、begin/commit/abort；`WorldTimeSnapshot`携带多个clock stamp、policy generation与discontinuity。
3. 生产源码没有calendar、logical day/year/season、timezone、latitude/longitude、planet center、axial tilt、ephemeris、solar elevation、moon phase或celestial identity。
4. 因此Clock只能作为Weather输入；它无法解释UI时间线，也不能自动把程序sun、DirectionalLight、IBL、Fog和Gameplay绑定到同一代。

### 3.4 Environment、Atmosphere、Cloud、IBL 与 Fog

1. `EnvironmentExtract`只有skybox/probes/baked lighting/probe grid，不含Weather/Atmosphere/Cloud identity、source revision、generation或dirty mask。
2. `build_environment_extract()`只把 `preview_skybox` 映射为默认gradient或disabled；普通Scene不能持久化Environment source。
3. `ProceduralSkyParams`只有horizon/zenith/ground、sun direction/color/intensity/angular radius、rotation和source revision；没有planet、Rayleigh、Mie、ozone、turbidity或aerial perspective。
4. DirectionalLight与程序sun均可执行但互不共享celestial body ID、photometry、temperature、shadow revision或clock generation。
5. source cubemap、PMREM/SH9/IEM、reflection probes、realtime IBL generation/time slice/last-good是真实底座，但没有Weather dirty mask、Cloud generation或跨代radiance crossfade。
6. 当前没有Cloud density/coverage/erosion/weather map/height profile/wind animation/lighting/shadow/transmittance/history/composite产品。
7. Volumetric Fog有typed settings、froxel passes与temporal history，但没有humidity/aerosol/visibility/rain/Weather generation输入；它只能是consumer。

### 3.5 Wind、Precipitation、Surface、Sound 与 Lightning

1. 没有Wind contributor、position/height/time query、gust/turbulence/region/LOD/overflow合同。
2. Particle CPU消费gravity + physics external force；GPU只消费gravity。既没有Weather adapter，也没有CPU/GPU frame schema parity。
3. 没有Rain/Snow/Hail profile、camera-relative spawn、streaming/LOD、surface hit、splash、decal、puddle、wetness或snow artifact。
4. Terrain没有snow/wetness/erosion/material mask输入；Material没有surface-state buffer owner。
5. Sound没有rain/wind ambience、listener/source Weather region、lightning PTS或thunder propagation adapter。
6. 没有deterministic StrikeId、bolt/flash/exposure/cloud illumination共享事件，也没有gameplay/network/replay/save receipt。

## 4. 五套参考源码对照

| 参考 | 当前可验证边界 | Zircon差距与采用边界 |
|---|---|---|
| Unreal Atmosphere/Cloud/Wind/DaySequence | Atmosphere与Cloud持有完整参数并管理scene proxy/render state；Wind随Scene add/update/remove并提供position query；DaySequence有day length、preview/pause/static time、modifier与replicated playback | 学习component/proxy/service/lifecycle/sequence分责；不复制历史耦合，不把各域塞进Weather bag |
| Unity HDRP | VisualEnvironment以versioned Volume参数选择sky/cloud/ambient；SkyManager维护per-camera context与lighting sky；physical sky有planet/air/aerosol/ozone；Cloud有map/shape/erosion/wind/quality/history/shadow | 学习versioned profile、context hash、history/fallback；Volume不是deterministic gameplay Weather authority |
| Godot | WorldEnvironment把可持久化Environment资源安装到World/RenderingServer并处理enter/exit/update与warning；FogVolume有shape/material/AABB/gizmo/lifetime | 学习resource-to-World/render闭环与authoring warning；不照搬单World限制为全部Weather模型 |
| Bevy | Atmosphere/settings为typed ECS component；render extract建设transmittance、multi-scattering、sky-view与aerial LUT，能力/尺寸显式 | 学习typed extract、LUT ownership与capability admission；Bevy大气不覆盖Climate/Network/Save |
| Fyrox | SkyBox具Reflect/Visit/UUID、TextureResource、cubemap生成、validation与builder，闭合Scene持久化到renderer | 保留Zircon更强IBL底座，补资源identity/validation/Scene闭环；不退化为固定六面图 |

最低工程闭环是 `persistent source -> lifecycle owner -> compiled/prepared representation -> real consumer -> observable failure`。五套参考都不能单独覆盖 Zircon 目标中的 deterministic Climate、regional transition、network/save 与 gameplay adapters；应组合其边界，再以 immutable artifact、per-World authority、generation-qualified snapshot 与 receipt 补齐跨域真值。

## 5. Authority 与目标架构

| 层 | 唯一 owner | 必须拥有 | 禁止拥有 |
|---|---|---|---|
| Climate/Celestial Source | Asset/Scene | stable ID、schema、geography/calendar/body、slow variables、unknown fields | runtime cursor、GPU LUT、widget state |
| Weather Source | Asset/Scene | preset/state/transition/timeline/region refs/seed/override provenance | World instance、particle emitter、render resource |
| Weather Compiler | neutral Runtime build service | validation、dependency、target variant、deterministic key、program、diagnostics | Editor selection、GPU object、domain mutable state |
| Weather Runtime | per-World service | program/clock/seed/region generation、snapshot、query/event cursor、rollback | fog froxel、particle pool、terrain texture、sound mixer |
| Domain Adapter | each domain owner | typed target、same-generation apply、fallback、budget、terminal receipt | duplicate Weather state/RNG/region authority |
| Editor Toolkit | Editor38 provider | document/transaction、timeline/curve、region binding、preview/build/diagnostics | fixed facts、direct Runtime mutation |
| App/Catalog | composition owner | provider/factory/service/toolkit closure、capability truth | descriptor-only success |

运行序列必须是：

`EditTransaction -> SourceRevision -> CompileRequest -> WeatherProgramArtifact -> WorldInstallGeneration -> WeatherFrameSnapshot -> DomainApplyReceipt -> QualifiedObservation`

任何编译、安装或adapter失败都保留上一完整generation；不允许半代Weather被不同consumer各自补默认值。

## 6. P0：必须先关闭的断路（5 Open）

| ID | 状态 | 当前证据 | 完整重构出口 |
|---|---|---|---|
| P0-1 | Open | 无Weather/Climate/Celestial source、compiler、artifact、World service或install receipt | 建立versioned source、deterministic compiler、immutable program与per-World service |
| P0-2 | Open | Scene无Environment/Weather；World只读preview_skybox；程序sun与DirectionalLight/clock分裂 | 建立CelestialClock、Scene binding、same-generation celestial sample与持久化/网络边界 |
| P0-3 | Open | 伪CaptureCloud已删除，但Cloud source/medium/render/history/artifact为零 | 建立真实Cloud field/render adapter；无能力平台显式Unsupported |
| P0-4 | Open | Particle CPU/GPU force不对称；Wind/Precipitation/Surface/Sound/Lightning无adapter | 先建snapshot与typed adapters，再闭合CPU/GPU parity和消费receipt |
| P0-5 | Open | workspace/timeline/counts/warnings/queued固定，且无provider/job/runtime caller | M0改为Fixture/Unavailable；真实产品链接入后恢复命令 |

## 7. P1：产品与 Editor 闭环（32 Open / 38 Partial）

| ID | 状态 | 当前证据与需要重构的内容 |
|---|---|---|
| P1-1 | Partial | generic versioned asset/migration/unknown-field基础可复用；仍需ClimateProfile schema/migration/loss report |
| P1-2 | Open | 无CelestialProfile、body identity、orbit/tilt/geography/timezone source |
| P1-3 | Partial | per-World clocks与generation真实；仍需CelestialClock、calendar及network/replay authority |
| P1-4 | Partial | WorldTimeSnapshot真实；仍需同代sun/moon/day/season CelestialFrameSnapshot |
| P1-5 | Open | Scene/Prefab/World无Weather/Climate/Celestial refs、revision或override provenance |
| P1-6 | Open | 无versioned WeatherPreset、state variables、seed或transition policy |
| P1-7 | Open | 无WeatherTimeline/TransitionGraph、typed key/curve/event或scrub/loop/branch |
| P1-8 | Open | 无WeatherRegionBinding，未引用Editor37 shared Region identity/generation |
| P1-9 | Partial | generic compiler/validator/dependency/digest基础可复用；Weather compiler规则为零 |
| P1-10 | Open | 无WeatherProgramArtifact及source/tool/schema/algorithm/platform provenance |
| P1-11 | Partial | World/Level可安装generation-qualified service；WorldWeatherService/lease/hot reload缺失 |
| P1-12 | Partial | immutable snapshot primitive存在；无原子WeatherFrameSnapshot或跨consumer一致性 |
| P1-13 | Partial | generation/budget/stale drop/rollback primitive分散；Weather dirty/crossfade/receipt未接 |
| P1-14 | Partial | diagnostics基础存在；缺source/region/clock/weather generation/subsystem维度 |
| P1-15 | Partial | Environment支持disabled/gradient/cubemap；无physical sky、exposure source与Scene authoring |
| P1-16 | Partial | 程序sun与DirectionalLight可执行但分裂；需celestial identity/photometry/shadow revision |
| P1-17 | Open | 无Rayleigh/Mie/ozone/turbidity/ground albedo Atmosphere source |
| P1-18 | Open | 无transmittance/multi-scattering/sky-view/aerial LUT compiler/cache |
| P1-19 | Open | 无Atmosphere LUT resolution/precision/platform tier/rebuild artifact metadata |
| P1-20 | Open | 无Cloud density/coverage/erosion/weather map/height/seed source |
| P1-21 | Open | 无Cloud lighting/shadow/transmittance/composite/temporal history |
| P1-22 | Partial | CaptureCloud已删除；独立Cloud graph/resource/input/artifact仍不存在 |
| P1-23 | Partial | realtime IBL有generation/A-B/stale/last-good/device基础；Cloud/sky同代crossfade未闭合 |
| P1-24 | Partial | IBL有key与time slicing；缺Weather dirty policy、GPU-time自适应与Cloud输入 |
| P1-25 | Partial | PMREM/SH9/IEM artifact与recipe真实；缺Environment generation/platform/consumer receipt |
| P1-26 | Partial | Fog有typed/froxel执行；humidity/aerosol/visibility/rain adapter为零 |
| P1-27 | Partial | Fog history与camera基础存在；未绑定Weather crossfade与同代viewport generation |
| P1-28 | Open | 无Wind direction/speed/gust/turbulence/altitude/region/seed/time field |
| P1-29 | Open | 无global/local/volumetric/terrain Wind provider、query snapshot或budget |
| P1-30 | Open | 无Weather-to-particle deterministic adapter |
| P1-31 | Open | CPU有external force而GPU只有gravity；force/collision/surface frame schema不对齐 |
| P1-32 | Open | 无rain/snow/hail type、rate/size/velocity/temperature/visibility profile |
| P1-33 | Open | 无camera-relative precipitation spawn、streaming、LOD、budget/drop generation |
| P1-34 | Open | 无hit/splash/decal/puddle/wetness/snow surface artifact |
| P1-35 | Open | Terrain无snow/wetness/erosion/material-mask incremental provider |
| P1-36 | Open | Material/global surface-state buffer与Weather generation为零 |
| P1-37 | Open | Sound无rain/wind ambience、listener/source region、Weather reverb/occlusion adapter |
| P1-38 | Open | 无seeded lightning bolt/flash/exposure/cloud/thunder PTS scheduler |
| P1-39 | Open | 无lightning gameplay/audio/network/replay/save同代receipt |
| P1-40 | Partial | Net replication/RPC/prediction可复用；无Weather digest/tick/state/seed/correction codec |
| P1-41 | Partial | save/archive/migration/checkpoint可复用；无Weather state/RNG/event cursor payload |
| P1-42 | Partial | streaming/generation/cancel基础存在；无weather region lease/prefetch/rollback |
| P1-43 | Partial | multi-world/view/camera/listener基础存在；Weather资源/RNG/snapshot隔离未验证 |
| P1-44 | Partial | capability/quality基础存在；无Atmosphere/Cloud/Fog/Particle平台矩阵 |
| P1-45 | Partial | finite validation/budget/fuzz基础可复用；Weather profile/curve/LUT/particle矩阵为空 |
| P1-46 | Partial | artifact cache/atomic publication/key/LKG存在；无Weather GC/platform/equivalence证据 |
| P1-47 | Partial | Operation具bounded/cancel/deadline/terminal基础；无Weather compile/bake/preview handler |
| P1-48 | Open | 无1/1k/100k regions、particles、cloud tiles、transitions与GPU规模基线 |
| P1-49 | Partial | diagnostics能承载clock/generation/budget；无Weather state/dirty/cloud/IBL/drop snapshot |
| P1-50 | Open | Editor无Climate/Celestial/Weather/Preset/Timeline/Region Binding AssetType/ResourceKind |
| P1-51 | Partial | generic document revision/save/recovery/conflict/undo存在；无Weather document |
| P1-52 | Partial | schema Inspector基础存在；无resolved clock/region/curve/capability/diagnostic schema |
| P1-53 | Open | Timeline固定行，不能操作真实document或typed key/event/seed |
| P1-54 | Open | Region dropdown未接shared spatial source/gizmo/weight/priority/query snapshot |
| P1-55 | Partial | viewport能显示gradient/cubemap/IBL/Fog；无artifact-aware Atmosphere/Cloud preview |
| P1-56 | Open | 无deterministic clock/seed Weather preview或frame/subsystem receipt |
| P1-57 | Open | rain/snow/wind/fog/surface/audio/lightning未通过runtime adapter预览 |
| P1-58 | Open | Build/Preview只返回固定文本，不提交真实job或receipt |
| P1-59 | Partial | catalog/App机制与若干domain provider真实；Weather provider/factory/toolkit为零 |
| P1-60 | Partial | plugin admission有通用边界；无Weather operation/controller/service closure |
| P1-61 | Open | 固定Weather_Storm、timeline、counts、regions、warnings仍在production workspace |
| P1-62 | Partial | logging/diagnostic/export基础可复用；无Weather source/clock/region/generation filters |
| P1-63 | Partial | Scene/PIE/network/save/reimport/hot reload有通用基础；Weather roundtrip为零 |
| P1-64 | Partial | Environment/IBL/Fog/Particle有局部tests；无跨Weather visual/audio/data golden |
| P1-65 | Partial | cache/device/cancel/late publish有generic failure基础；无whole-chain fault oracle |
| P1-66 | Partial | headless/platform/package基础存在；无Weather client/server/editor clean matrix |
| P1-67 | Partial | GPU timing/budget/render stats存在；无Weather p50/p95/p99/VRAM/hitch基线 |
| P1-68 | Partial | migration/LKG/generation pin/rollback primitive存在；无Weather canary |
| P1-69 | Partial | package/release manifest基础存在；未列Climate/Weather artifacts与provenance |
| P1-70 | Open | 无同内容/同质量/同平台/同统计口径的五参考竞争benchmark |

## 8. P2：长期能力（12 Open）

| ID | 状态 | 能力 |
|---|---|---|
| P2-1 | Open | physically based atmosphere、multi-scattering、ozone、aerial perspective与LUT streaming |
| P2-2 | Open | compute volumetric cloud、sparse weather fields、shadow maps与temporal reprojection |
| P2-3 | Open | global fronts、pressure、humidity、thermodynamics与climate data import |
| P2-4 | Open | terrain snow/ice/wetness/erosion accumulation与streaming sparse masks |
| P2-5 | Open | ocean/wave/fog/rain interaction、splash/foam、wind water与shoreline weather |
| P2-6 | Open | multi-body celestial、eclipse、moon phase、starfield、aurora与calendar localization |
| P2-7 | Open | procedural lightning/thunder propagation、acoustic delay与network synchronization |
| P2-8 | Open | remote forecast provider、cache、permission与deterministic fallback |
| P2-9 | Open | neural/upscale/weather denoiser provider纳入artifact/quality/fallback contract |
| P2-10 | Open | collaborative weather timeline editing、field merge、lock与review annotation |
| P2-11 | Open | program schema/algorithm migration、canary、rollback与replay compatibility |
| P2-12 | Open | cross-engine weather benchmark与公开reference scenes/methodology |

## 9. 分层重构顺序

### M0 Truthfulness 与第二 authority 清理

Weather capability保持Unavailable/Prototype；workspace明确标为fixture，删除固定成功、计数与warning语义。missing provider、missing asset、unsupported platform必须可见，M1前不再增加静态Weather controls。

### M1 Source、Clock、Celestial 与 Region

建立versioned Climate/Celestial/Weather/Preset/Timeline sources、stable IDs、CelestialClock、Editor37 SpatialRegion binding及lossless Scene/Prefab/save/net/replay边界。先闭合source authority，再建设效果。

### M2 Compiler、Artifact 与 World Authority

建立validator/dependency/compiler、immutable WeatherProgramArtifact、per-World service、atomic WeatherFrameSnapshot、dirty/crossfade/budget、bounded query/event cursor与diagnostics。失败保留上一完整generation。

### M3 Environment Render Adapters

由render owners实现Atmosphere LUT、真实Cloud medium/render/history、Sky/DirectionalLight binding、IBL dirty/crossfade与Fog adapter。Weather只输出同代typed parameters，不拥有GPU资源或froxel。

### M4 Wind、Precipitation、Surface、Sound 与 Lightning

建立WindField、CPU/GPU particle parity、camera-relative precipitation、impact/surface accumulation、Terrain/Material/Sound/Gameplay adapters与deterministic Lightning receipts。

### M5 Transactional Editor Product

接入AssetType、document/transaction、schema inspector、timeline/curve、region gizmo、artifact-aware preview、background jobs、catalog/App/provider closure。所有命令返回job/source/artifact/World generation与terminal disposition。

### M6 Qualification 与竞争基线

完成roundtrip、determinism、network/save/replay、visual/audio/data golden、fault、device-loss、scale、headless、platform matrix、upgrade/rollback与同质量五参考benchmark。通过前不得发布Stable/Complete。

## 10. 验收门禁（18 Fail / 14 Partial / 0 Pass）

| # | 状态 | 门禁 |
|---:|---|---|
| 1 | Fail | Climate/Celestial/Weather/Region source、artifact、instance、generation identity完整 |
| 2 | Partial | World clocks具pause/rate/generation；Celestial geography/season/network/replay未实现 |
| 3 | Fail | Weather compiler/program key、dependency、seed、migration、platform variant正确 |
| 4 | Partial | immutable/generation/budget primitive存在；Weather snapshot/dirty/crossfade未闭合 |
| 5 | Fail | Scene/Prefab/PIE/Save/Network/Replay roundtrip保持Weather authority |
| 6 | Fail | Sky/DirectionalLight/sun/moon identity、photometry、shadow与clock一致 |
| 7 | Fail | Atmosphere LUT numeric/visual/precision/cache/rebuild/fallback golden |
| 8 | Fail | Cloud density/erosion/lighting/shadow/transmittance/history/composite有效 |
| 9 | Fail | CaptureCloud已删除，但独立Cloud graph/input/resource/artifact不存在 |
| 10 | Partial | PMREM/SH/IEM、generation/time-slice/last-good存在；Weather crossfade/receipt未闭合 |
| 11 | Partial | Fog typed/froxel/history存在；Weather humidity/aerosol/rain/generation未接 |
| 12 | Fail | Wind field/query/gust/turbulence/altitude/region/seed deterministic |
| 13 | Fail | CPU/GPU Particle Weather/external-force schema parity、LOD、budget、cancel无分叉 |
| 14 | Fail | rain/snow/hail spawn/streaming/impact/surface accumulation golden |
| 15 | Fail | Terrain/Material wetness/snow/ice/roughness masks与generation正确 |
| 16 | Fail | Sound ambience/reverb/listener/source/lightning PTS与output receipt正确 |
| 17 | Fail | Lightning geometry/flash/cloud/thunder/network/replay/save deterministic |
| 18 | Partial | multi-world/view/camera/listener存在；Weather isolation/late-generation未验证 |
| 19 | Partial | generic validation/fuzz/budget存在；Weather malformed/LUT/curve/particle矩阵为空 |
| 20 | Partial | atomic artifact/cache/LKG存在；Weather GC/platform/equivalence/rollback未验证 |
| 21 | Partial | bounded/cancel/deadline/job存在；Weather retry/shutdown/device恢复未验证 |
| 22 | Partial | generic document/transaction/undo/recovery存在；Weather source未接入 |
| 23 | Fail | timeline/curve/region/sky/cloud preview使用真实document/artifact/runtime snapshot |
| 24 | Fail | Build/Preview/Apply/Playtest返回job/source/generation/artifact/diagnostic receipt |
| 25 | Fail | fixed Weather workspace facts由真实state替代，missing/error可见 |
| 26 | Partial | catalog/admission机制存在；Weather provider/factory/controller/service为零 |
| 27 | Partial | 各domain有局部tests；跨Weather visual/audio/data golden不存在 |
| 28 | Fail | 1/1k/100k regions/particles/cloud tiles/transitions与GPU/VRAM/hitch达标 |
| 29 | Partial | platform/headless/capability基础存在；Weather完整矩阵未运行 |
| 30 | Partial | diagnostics/export基础存在；Weather同代过滤与receipt未接 |
| 31 | Partial | migration/LKG/rollback primitive存在；Weather canary/pin/replay未验证 |
| 32 | Fail | Stable/Complete由compile、registration、runtime、Editor、fault、scale证据派生 |

## 11. 禁止的临时修补

1. 禁止把固定Weather_Storm、timeline、counts、regions、warning或queued feedback当产品状态。
2. 禁止用preview_skybox、gradient、DirectionalLight或Fog字段拼接Environment authority。
3. 禁止恢复CaptureCloud gradient pass，或把source cubemap/IBL更新称为Cloud实现。
4. 禁止只增加Weather/Cloud/Wind/Rain类型、manifest或UI，没有compiler/artifact/service/adapter。
5. 禁止让CPU/GPU Particle继续接受不同force/weather inputs而没有typed schema与receipt。
6. 禁止在render/particle/audio thread同步执行Weather compile、LUT/Cloud bake或无界复制。
7. 禁止让Weather直接写Fog froxel、Particle pool、Terrain texture、Material buffer或Sound mixer。
8. 禁止用脚本JSON、display name、control ID或下拉字符串作为stable Weather identity。
9. 禁止编译/安装失败时发布半代state，或用默认Clear/Storm静默替换失败source。
10. 禁止以缺失Atmosphere/Cloud/Precipitation后的低耗时宣称性能优于Unreal/Unity。

## 12. 跨计划 Owner 与实施边界

| 领域 | 唯一 owner | Editor215只登记的边界 |
|---|---|---|
| Weather authoring/product truth | Editor38 | document/toolkit/transaction/preview/build/diagnostic；实施前保持Unavailable |
| Weather executable runtime | Runtime149（历史Runtime36） | program/service/snapshot/query/adapter runtime合同与first-party实现 |
| Spatial Region | Editor37 / Runtime Region owner | stable source、compiled geometry、index、generation；Weather只持binding/policy |
| Environment/IBL/Atmosphere/Cloud | Runtime96与render owners | GPU artifacts、graph、history、quality、device lifecycle；消费Weather adapter |
| Volumetric Fog | Runtime Fog owner | froxel/history/render资源；只接typed Weather target |
| Particle/Surface/Terrain/Material/Sound | 各domain owner | 执行资源与output receipt；不得复制Weather authority |
| Time/Scene/Net/Save/Operation | 各Runtime owner | 通用primitive与persistence/transport，不复制Weather policy |
| App/Catalog/Admission | composition owners | 显式装配provider/factory/service/toolkit及能力闭包 |

Editor38 仍是5 P0、70 P1、12 P2和32门的唯一canonical owner；Editor112/159/215只刷新currentness，不重复增加finding。Runtime149拥有可执行Weather runtime细化项，Editor215只拥有authoring/product truth与实施依赖。

## 13. 本轮产出边界

本轮只新增current-source review、状态重判、参考差异、架构边界、分层里程碑与门禁；没有修改Runtime、Editor、Plugin、Interface、App生产代码或tests。`review_complete`只表示冻结的394文件范围完成静态取证，不表示Editor38实施完成，也不表示任何动态资格门通过。

实施前必须重导283个Zircon文件manifest/fingerprint并核对shared working tree drift，重点复查Scene schema、EnvironmentExtract、World time、Particle CPU/GPU frame schema、Fog/IBL generation、Weather workspace/callback与catalog/App closure。Tooling继续按用户要求排除，也不得通过等待或轮询协调器阻塞其它审查里程碑。
