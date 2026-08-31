---
title: Editor Weather、Climate、Time-of-Day、Wind、Precipitation、Cloud、Atmosphere 与 Environment 当前源码复核
category: zircon_editor
report_id: Editor159
review_date: 2026-08-27
baseline_head: 331668a00d93771f0e22ec7db8538d5d809e3a9d
verification_head: 2f684e191c5252775b4a192f3ffa77e2f48c6757
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
canonical_owner: Editor38
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/38-weather-climate-time-of-day-wind-precipitation-cloud-atmosphere-environment-authoring-review.md
  - docs/plans/optimize/zircon_editor/112-editor-weather-climate-time-of-day-wind-precipitation-cloud-atmosphere-current-source-review.md
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
plan_sources:
  - docs/plans/optimize/zircon_editor/38-weather-climate-time-of-day-wind-precipitation-cloud-atmosphere-environment-authoring-review.md
  - docs/plans/optimize/zircon_editor/112-editor-weather-climate-time-of-day-wind-precipitation-cloud-atmosphere-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zx-runtime-weather-climate-celestial-time-of-day-wind-precipitation-cloud-atmosphere-surface-state-determinism-network-save-scalability-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/96-runtime-environment-sky-atmosphere-cloud-ibl-reflection-probe-capture-convolution-sh-pmrem-cache-residency-submission-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/158-editor-volume-zone-trigger-region-gameplay-audio-post-process-environment-current-source-review.md
  - docs/plans/zircon_runtime/render/01/failure-2026-07-23-disabled-forward-volumetric-cache-field-anchor-drift.md
  - docs/plans/zircon_runtime/render/01/failure-2026-08-16-render01-realtime-ibl-recording-pass-import.md
  - docs/plans/zircon_runtime/render/13/failure-2026-07-17-environment-ibl-parallel-staging.md
  - docs/plans/zircon_runtime/render/17/failure-2026-07-29-deferred-volumetric-params-buffer-lifetime.md
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

# 159 · Editor Weather / Climate / Time-of-Day / Wind / Precipitation / Cloud / Atmosphere / Environment 工程化差距

## 1. 结论

Editor38 的核心结论仍成立：Zircon 当前没有可供项目使用的 Weather/Climate authoring 产品。生产源码中没有 `ClimateProfileAsset`、`CelestialProfileAsset`、`WeatherPresetAsset`、`WeatherTimeline`、`WeatherProgramArtifact`、`WorldWeatherService`、`WeatherFrameSnapshot`、`WindField`、`SurfaceWeatherState` 或 `StrikeId`；没有 Weather/Climate 首方 package、resource kind、catalog provider、App feature closure、Scene binding、compiler、runtime service、network/save codec 或 typed consumer adapter。

当前 Weather workspace 仍是第二 authority：它固定显示 `Weather_Storm`、`Region_Mountains`、`Layer_Clouds`、四段时间线、8 layers、5 regions、2 warnings，Preset/Region/Blend Time 也是固定字符串选项；Preview/Build 只回写 `queued` 文本。navigation、template binding 和 preview allow-list只证明route可达，不证明存在document、transaction、job、artifact、runtime snapshot或terminal receipt。`inspector.rs`中的 Weather/CloudLayer 名称位于 `#[cfg(test)]` 后，也不是生产插件。

本轮同时纠正 Editor112 的时效差：历史 `CaptureCloud` 重复写 gradient 的生产路径已经删除，当前唯一命中是 graph test 明确断言不得出现 `CaptureCloud/CAPTURE_CLOUD`。这关闭了一个错误操作，不会自动生成 Cloud 产品。Cloud source/medium、density、weather map、lighting、shadow、transmittance、composite、history和artifact generation仍全部缺失，因此 canonical P0-3 继续 Open，但证据从“错误重复实现”更新为“真实实现为零”。

应保留的底座比旧 Editor112 更完整。每个 World 已有独立 virtual/fixed clock、pause/scale、fixed debt、begin/commit/abort与immutable `WorldTimeSnapshot`；Environment已有source cubemap、PMREM/SH9/IEM、reflection probe、generation-aware realtime IBL、time slicing和last-good思路；Volumetric Fog有typed settings、froxel graph与temporal history；Particle有CPU/GPU backend；通用asset/compiler/DDC、operation、network、save、diagnostic、platform和Editor document基础可复用。这些基础使38项P1与14个门禁达到Partial，但没有一项Weather领域闭环达到Closed/Pass。

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

Weather拥有状态、确定性和跨域同代真值；render、particle、terrain、material、sound和gameplay owner仍拥有各自执行资源与质量策略。不得创建万能Environment property bag，也不得让Editor、DirectionalLight、Fog或Particle中的任一个局部字段成为第二天气权威。

## 2. 审查范围与方法

### 2.1 当前工作树选择集

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 指纹与边界 |
|---|---:|---|
| Zircon Editor/Runtime/Plugin/App selected | **371 / 57,963 / 52,726 / 2,081,567 / 459 / 0** | Weather UI/routing、asset/catalog/App、Scene/time、environment/IBL/fog、particle、terrain、sound environment与builtin registration；`883ed0b6b6cf1c8a249d4deef4cbbee480d29bc708cbaa081464d4dc4bfa4d38` |
| Unreal/Unity Graphics/Godot/Bevy/Fyrox reference | **46 / 23,710 / 20,180 / 1,061,859 / 9 / 0** | Atmosphere、Cloud、Wind、DaySequence、Environment、Fog、Sky persistence/render链；`a5cb37a65914af6ee46ec940578ccbcc48cc543312fbfbd5f5581c1790290249` |
| 全部选择集 | **417 / 81,673 / 72,906 / 3,143,426 / 468 / 0** | 当前共享working tree去重物理语料；`11f2073460ade5da90a91b5530b14509634476295b3359194c86e3c87aec9463` |

统计以当前物理文件为准，明确包含选择前缀内未跟踪源码，排除`target`和`.zircon/cache`；按规范化repository-relative path、当前file SHA-256聚合。Tooling按用户要求排除。本轮是E3静态review，没有运行Cargo、Editor、WGPU、audio、network、save、headless、fault、scale、soak或跨引擎benchmark，因此源码可证明owner/数据链缺失，不能证明未来确定性、像素质量或性能达标。

共享工作树存在大量其它Session的在途修改；本报告未归因、覆盖或回退它们。四份关联failure记录分别涉及volumetric buffer结构守卫、realtime IBL导入、IBL并行staging和deferred volumetric buffer lifetime，均属于各自render owner的实现/动态验证状态，不阻塞本轮只读差距审查，也不被本报告声明为已修复。

### 2.2 判定规则

1. `Open`：Weather目标contract、owner、artifact、consumer或产品链不存在，或现有行为与目标冲突。
2. `Partial`：存在可执行、可测试且可保留的通用/局部子链，但Weather identity、generation、consumer或资格证据未闭合。
3. `Closed/Pass`：必须有当前源码、产品装配、runtime执行和相应动态证据；本轮没有任何项达到该等级。
4. 类型名、descriptor、capability、ZUI、test fixture、固定feedback和可选feature不单独证明产品存在。
5. 负证据限定在生产roots；`dev/`、`docs/`和`#[cfg(test)]`命中不反推Zircon已实现。

## 3. 当前 Zircon 产品链

### 3.1 Package、Asset、Scene 与产品真相

1. `zircon_plugins`没有Weather/Climate/Atmosphere/Cloud package；first-party runtime catalog可装Sound、Particles、Rendering、Navigation等provider，Editor catalog只有Navigation与Neural，没有Weather。
2. Editor builtin `ResourceKind`列出Data、Model、Mesh、Material、Texture、Scene、Sound、Terrain、Animation和UI等26类，没有Climate/Celestial/Weather/Preset/Timeline/Region Binding。
3. `SceneEntityAsset`固定保存camera、mesh、五类light、post process、physics、animation、terrain、tilemap、prefab和script binding，没有Environment、Weather、Climate、Cloud、Wind或Time-of-Day引用。
4. `target-editor-host`装配advanced render runtime、Navigation runtime/editor、Neural editor及若干contracts，没有Weather runtime/editor feature closure。
5. 精确类型检索在生产roots为零；1,516个未跟踪Rust/TOML/ZUI生产候选同样没有上述Weather产品类型。

### 3.2 Weather Editor：静态演示面而非authoring product

1. ZUI固定 `Weather_Storm`、`Region_Mountains`、`Layer_Clouds`，没有project asset query、selection identity、empty/error/loading或conflict state。
2. Timeline固定Cloud Build `00:00-02:00`、Rain Burst `02:00-04:00`、Wind Gust `03:20-05:00`、Lightning `04:00-04:30`；没有typed key、curve、event、seed、clock domain或scrub semantics。
3. 输出固定`8 layers / 5 regions / 2 warnings`；Preset固定Storm/Clear/Fog/Snow，Region固定Mountains/Coast/City/Interior，Blend Time是`12.0`字符串。
4. navigation spec和template binding只把tab/field/click映射到control/action；preview allow-list只放行route。
5. callback对Open、Preview、Build、Rain Burst、Lightning返回固定文本，不产生operation ticket、source revision、artifact digest、World generation、diagnostic identity或terminal disposition。
6. 没有Weather document provider、operation factory、controller、background job、viewport overlay、preview scene、runtime bridge或catalog registration。

### 3.3 Clock 与 Celestial：工程化时间底座，不是天体权威

1. `RuntimeTimeAuthority`拥有monotonic real clock和policy generation；`WorldTimeController`拥有per-World virtual/fixed clocks、pause、relative speed、fixed debt、begin/commit/abort step。
2. `SimulationTickId`绑定world generation、fixed epoch和tick index；`WorldTimeSnapshot`冻结outer frame、raw/virtual delta、elapsed、pause/speed、policy generation、discontinuity、fixed plan和clock stamps。
3. `ClockDomainRegistry`区分MonotonicReal、WallUtc、WorldVirtual、WorldFixed、Input、Render、Audio、Network、Media和EditorPreview。
4. 生产代码没有calendar、epoch、logical day/year/season、timezone、latitude/longitude、planet center、axial tilt、ephemeris、solar elevation、moon phase或celestial identity。
5. 因此强时间底座只能让P1-3/4达到Partial；它不能解释Weather UI中的时间段，也不能让DirectionalLight和程序sun自动同源。

### 3.4 Scene、Sky、Environment 与 DirectionalLight：authority仍分裂

1. `World::build_environment_extract()`仍只用`preview_skybox`映射`EnvironmentExtract::procedural_default()`或disabled；`preview_skybox`又被authoring-boundary test明确禁止序列化。
2. `EnvironmentExtract`只有skybox、reflection probes、baked lighting和probe grid，没有Weather/Atmosphere/Cloud identity、source revision、generation或dirty mask。
3. Editor viewport又单独构造`PreviewEnvironmentExtract` fallback；普通Scene没有可持久化Environment source。
4. 当前sky shader支持source cubemap或程序天空，程序路径是horizon/zenith/ground插值加sun disk；没有Rayleigh/Mie/ozone/turbidity/aerial perspective物理参数。
5. DirectionalLight与程序sun都是真实局部数据，但没有celestial body ID、共同photometry/temperature/shadow revision或clock generation。

### 3.5 Atmosphere、Cloud、IBL 与 Fog：IBL/Fog可保留，Cloud仍为零

1. Environment已有HDR/EXR source cubemap、upload artifact、PMREM/SH9/IEM recipe、reflection probe、bounded readback和realtime IBL资源/graph/time-slice基础。
2. realtime IBL具generation、pending/published key、A/B资源和stale/last-good方向，但没有Weather dirty mask、Cloud generation、跨代radiance crossfade或consumer receipt闭环。
3. 当前生产源码没有`CaptureCloud/CAPTURE_CLOUD`；test显式阻止伪operation回归。真实Cloud source、density、coverage、erosion、weather map、height profile、wind animation、lighting/shadow/history仍不存在。
4. `VolumetricFogSettings`拥有density、albedo、phase、height falloff、scattering、depth distribution和temporal；volumetric plugin建立media inject、light scatter、integrate froxel passes及persistent history。
5. Fog没有humidity、aerosol、visibility、rain extinction或Weather generation输入。它应是typed consumer，不应成为Weather state owner。

### 3.6 Wind、Precipitation、Surface、Sound 与 Lightning：typed adapter全缺

1. 没有Wind contributor、`WindField`、position/height/time query、gust/turbulence/region/LOD/overflow合同。
2. Particle CPU每步累加`asset.gravity + physics.external_force`；GPU emitter只上传gravity，WGSL只执行`velocity += gravity * dt`。这既没有Weather adapter，也存在现有CPU/GPU语义不对称。
3. 没有Rain/Snow/Hail profile、camera-relative spawn volume、world streaming/LOD/drop policy、surface hit/splash/decal/puddle/wetness/snow artifact。
4. Terrain有typed heightfield/layer asset与plugin边界，但没有snow/wetness/erosion/material mask输入；Material没有surface-state buffer owner。
5. Sound有source/listener/acoustic volume与调度基础，但没有rain/wind ambient、Weather region、lightning PTS或thunder propagation adapter。
6. 没有deterministic `StrikeId`、bolt/flash/exposure/cloud illumination共享事件，也没有gameplay/network/replay/save receipt。

## 4. 五套参考源码对照

| 参考 | 可验证的工程边界 | Zircon当前差异 | 采用边界 |
|---|---|---|---|
| Unreal Atmosphere/Cloud/Wind/DaySequence | SkyAtmosphere保存planet、Rayleigh/Mie/absorption/aerial参数并管理scene proxy/render state；VolumetricCloud保存layer/material/sample/shadow/reflection并管理proxy；Wind随Scene add/update/remove且支持position query；DaySequence拥有day length、cycle、preview、pause、static time、modifier volume和replicated playback | Zircon只有gradient/cubemap、独立DirectionalLight、Fog与IBL基础；无Atmosphere/Cloud/Wind/DaySequence source、proxy/service或统一generation | 学习component/proxy/lifecycle/sequence分责；不复制Unreal历史耦合，也不把四域塞进Weather property bag |
| Unity HDRP | `VisualEnvironment`以typed Volume参数选择sky/cloud/ambient并支持migration；`SkyManager`维护visual/lighting sky、per-camera context、cubemap/probe lifecycle；PhysicallyBasedSky有planet/air/aerosol/ozone；VolumetricClouds有map/shape/erosion/wind/quality/history/shadow矩阵 | Zircon无typed Environment source、per-camera Weather context、physical sky或Cloud质量/历史系统 | 学习versioned profile、context hash、history和fallback；Unity Volume不是deterministic gameplay Weather authority |
| Godot | `WorldEnvironment`将可持久化Environment资源安装到World并在enter/exit/update时维护唯一实例与warning；Environment通过RenderingServer RID更新sky/reflection/fog；FogVolume有shape/material/AABB/gizmo/warning/lifetime | Zircon普通Scene不能保存Environment，Fog没有Weather source或Editor gizmo，preview bool是唯一入口 | 学习资源到World/renderer的最低闭环与authoring warning；不把Godot单World Environment限制照搬为全部Weather模型 |
| Bevy | `Atmosphere`/settings是ECS component，最近大气按camera使用；render world extract真实建设transmittance、multi-scattering、sky-view、aerial LUT与environment map，尺寸/能力有显式处理 | Zircon没有physical Atmosphere component、LUT资源或render extract | 学习typed ECS extract、LUT ownership和capability admission；Bevy大气不是Climate/Network/Save完整参考 |
| Fyrox | SkyBox具Reflect/Visit/UUID、六面TextureResource、cubemap生成、validation和builder，形成Scene持久化到renderer的简单闭环 | Zircon有更强IBL算法，但Scene authoring/persistence入口反而缺失 | 保留Zircon artifact能力，补齐资源身份、validation和Scene闭环；不退化为六张图的固定实现 |

五套参考共同给出的最低门槛是`persistent source -> lifecycle owner -> prepared/render representation -> real consumer -> observable failure`。它们没有一套单独覆盖Zircon目标所需的deterministic Climate、regional transitions、network/save和全部gameplay adapter；Zircon应组合这些边界，并用immutable artifact、per-World authority、generation-qualified snapshot与receipt补齐跨域真值。

## 5. Authority 与目标架构

| 层 | 唯一owner | 必须拥有 | 禁止拥有 |
|---|---|---|---|
| Climate/Celestial Source | Asset/Scene | stable ID、schema version、geography/calendar/body/slow-variable、unknown fields | runtime clock cursor、GPU LUT、Editor widget状态 |
| Weather Source | Asset/Scene | preset/state/transition/timeline/region refs/seed policy/override provenance | World instance、particle emitter或render resource |
| Weather Compiler | Runtime neutral build service | validation、dependencies、target profile、deterministic digest、program tables、diagnostics | Editor fallback、per-World mutable state |
| WorldWeatherService | each World | program/clock/seed/region generations、authority lease、atomic snapshot、bounded events | render/fog/particle/sound私有资源 |
| Domain Adapter | each subsystem owner | typed same-generation input、budget、fallback、apply/terminal receipt | 复制Weather state machine或读取散乱UI字段 |
| Editor Toolkit | Weather Editor provider | document/transaction、Inspector、timeline、region、preview、jobs、diagnostics | 固定业务事实、直接写World map、control-local success |
| App/Catalog | product composition | provider/factory/service closure、capability truth、activation receipt | 由descriptor或ZUI猜测产品可执行 |

关键运行序列必须是：

`EditTransaction -> SourceRevision -> CompileTicket -> WeatherProgramArtifact -> WorldInstallGeneration -> WeatherFrameSnapshot -> AdapterApplyReceipt -> QualifiedObservation`

任何阶段失败保留上一完整generation；Editor只显示authoritative state或typed Unavailable/Failed，不显示预设成功文本。

## 6. P0：必须先关闭的断路（5 Open）

| ID | 状态 | 当前证据 | 完整重构出口 |
|---|---|---|---|
| P0-1 | Open | 无Weather/Climate/Celestial source kind、compiler、artifact、World service或install receipt | 建立versioned source、deterministic compiler、immutable program与per-World service，再允许产品能力进入catalog |
| P0-2 | Open | Scene没有Environment/Weather引用；World只读不可序列化的`preview_skybox`；程序sun与DirectionalLight/clock分裂 | 建立CelestialClock、Environment/Weather binding、same-generation celestial sample及Scene/PIE/save/net/replay边界 |
| P0-3 | Open | 历史伪`CaptureCloud`已删除，但Cloud source/medium/render/history/artifact为零 | 建立真实Cloud source/field/render adapter；无能力平台显式Unsupported，禁止恢复gradient伪operation |
| P0-4 | Open | Particle CPU/GPU force不对称；Wind/Precipitation/Surface/Terrain/Material/Sound/Lightning无Weather adapter | 先建立immutable snapshot与typed adapters，再补CPU/GPU parity和各消费回执 |
| P0-5 | Open | Weather workspace、timeline、counts、warnings与queued feedback仍固定，且无provider/job/runtime caller | M0改为Fixture/Unavailable；只在真实document/compiler/runtime preview/toolkit接入后恢复命令 |

## 7. P1：Runtime、Environment、Weather 与 Editor（32 Open / 38 Partial）

| ID | 状态 | 当前证据与完整出口 |
|---|---|---|
| P1-1 | Partial | generic versioned asset/migration/unknown-field基础可复用；仍需`ClimateProfileAsset` schema、migration与loss report |
| P1-2 | Open | 无CelestialProfile、body identity、orbit/tilt/geography/timezone source |
| P1-3 | Partial | per-World virtual/fixed clock、pause/rate/generation真实；仍需CelestialClock、calendar和network/replay/offline authority |
| P1-4 | Partial | immutable `WorldTimeSnapshot`真实；仍需同代sun/moon/day/season的`CelestialFrameSnapshot` |
| P1-5 | Open | Scene/Prefab/World没有Weather/Climate/Celestial refs、revision或override provenance |
| P1-6 | Open | 无versioned WeatherPreset、state variables、seed或transition policy |
| P1-7 | Open | 无WeatherTimeline/TransitionGraph、typed key/curve/event或scrub/loop/branch语义 |
| P1-8 | Open | 无WeatherRegionBinding，也未引用Editor37 SpatialRegion identity/generation |
| P1-9 | Partial | generic compiler/validator/dependency/digest基础可复用；Weather compiler和领域规则为零 |
| P1-10 | Open | 无`WeatherProgramArtifact`及source/tool/schema/algorithm/platform dependency记录 |
| P1-11 | Partial | World/Level lifecycle能安装generation-qualified service；`WorldWeatherService`、lease与hot reload未实现 |
| P1-12 | Partial | immutable snapshot primitive存在；没有原子`WeatherFrameSnapshot`或跨consumer一致性 |
| P1-13 | Partial | generation、budget、stale drop和rollback primitive分散存在；Weather dirty mask/crossfade/receipt未接入 |
| P1-14 | Partial | runtime/editor diagnostic基础存在；没有source/region/clock/weather generation/subsystem receipt维度 |
| P1-15 | Partial | Environment支持disabled/procedural/source cubemap、rotation/intensity方向；无physical sky、exposure source和Scene authoring |
| P1-16 | Partial | 程序sun与DirectionalLight都可执行但互不绑定；需stable celestial identity、photometry/temperature/shadow revision |
| P1-17 | Open | 无Rayleigh/Mie/ozone/turbidity/ground albedo Atmosphere source |
| P1-18 | Open | 无transmittance/multi-scattering/sky-view/aerial LUT compiler/cache |
| P1-19 | Open | 无Atmosphere LUT resolution/precision/platform tier/invalid/rebuild artifact metadata |
| P1-20 | Open | 无Cloud density/coverage/erosion/weather map/height/seed source |
| P1-21 | Open | 无Cloud lighting/shadow/transmittance/composite/temporal history |
| P1-22 | Partial | shared-gradient `CaptureCloud`已删除；但独立Cloud graph/resource/input/artifact仍不存在 |
| P1-23 | Partial | realtime IBL有generation、A/B资源、stale/last-good/device恢复基础；Cloud/sky同代crossfade未闭合 |
| P1-24 | Partial | IBL有key比较与time slicing；没有Weather snapshot dirty policy、GPU-time自适应与Cloud输入 |
| P1-25 | Partial | PMREM/SH9/IEM artifact与recipe真实；缺Environment generation、platform variant和consumer receipt闭环 |
| P1-26 | Partial | Fog有typed evaluator/froxel执行；humidity/aerosol/visibility/rain/Weather state adapter为零 |
| P1-27 | Partial | Fog temporal history、froxel与camera基础存在；未绑定Weather crossfade和同代viewport generation |
| P1-28 | Open | 无Wind direction/speed/gust/turbulence/altitude/region/seed/time field |
| P1-29 | Open | 无global/local/volumetric/terrain Wind provider、query snapshot或budget |
| P1-30 | Open | 无Weather-to-particle deterministic adapter |
| P1-31 | Open | CPU消费external force而GPU只消费gravity；wind/collision/surface frame schema不对齐 |
| P1-32 | Open | 无rain/snow/hail type、rate/size/velocity/temperature/visibility profile |
| P1-33 | Open | 无camera-relative precipitation spawn、streaming、LOD、budget/drop generation合同 |
| P1-34 | Open | 无hit/splash/decal/puddle/wetness/snow surface artifact |
| P1-35 | Open | Terrain无snow/wetness/erosion/material-mask incremental provider |
| P1-36 | Open | Material/global surface-state buffer及Weather generation为零 |
| P1-37 | Open | Sound无rain/wind ambient、listener/source region、Weather reverb/occlusion adapter |
| P1-38 | Open | 无seeded lightning bolt/flash/exposure/cloud/thunder PTS scheduler |
| P1-39 | Open | 无lightning gameplay/audio/network/replay/save同代receipt |
| P1-40 | Partial | Net replication/RPC/prediction基础可复用；无Weather digest/tick/state/seed/correction codec |
| P1-41 | Partial | save/archive/migration/checkpoint基础可复用；无Weather state/RNG/event cursor payload |
| P1-42 | Partial | streaming/generation/cancel基础存在；无weather region lease/prefetch/rollback |
| P1-43 | Partial | multi-world/view/camera/listener基础存在；Weather资源、RNG、snapshot隔离未验证 |
| P1-44 | Partial | platform/capability/quality基础存在；无Atmosphere/Cloud/Fog/Particle HDR/compute/temporal矩阵 |
| P1-45 | Partial | generic finite validation、budget和fuzz基础可复用；Weather profile/curve/seed/LUT/particle矩阵为空 |
| P1-46 | Partial | artifact cache、atomic publication、key/LKG方向存在；无Weather cold/warm/GC/platform/equivalence证据 |
| P1-47 | Partial | Operation service具bounded/cancel/deadline/terminal基础；无Weather compile/bake/preview handler |
| P1-48 | Open | 无1/1k/100k regions、particles、cloud tiles、transitions与GPU budget规模基线 |
| P1-49 | Partial | runtime diagnostics能承载clock/generation/budget；无Weather state/dirty/cloud/IBL/drop snapshot |
| P1-50 | Open | Editor没有Climate/Celestial/Weather/Preset/Timeline/Region Binding AssetType/ResourceKind |
| P1-51 | Partial | generic document revision、dirty/save/autosave/recovery/conflict/undo基础存在；无Weather document |
| P1-52 | Partial | schema inspector/customization基础存在；无resolved Weather clock/region/curve/capability/diagnostic schema |
| P1-53 | Open | Timeline只显示固定行，不能操作真实document或typed key/event/seed |
| P1-54 | Open | Region dropdown未接Editor37 spatial source/gizmo/weight/priority/query snapshot |
| P1-55 | Partial | viewport能显示程序天空/source cubemap/IBL/Fog基础；无artifact-aware Atmosphere/Cloud document preview |
| P1-56 | Open | 无deterministic clock/seed Weather preview或frame/subsystem receipt |
| P1-57 | Open | rain/snow/wind/fog/surface/audio/lightning未通过Weather runtime adapter预览 |
| P1-58 | Open | Build/Preview routes只返回固定文本，不提交真实job或receipt |
| P1-59 | Partial | first-party catalog/App装配机制和若干domain provider真实；Weather provider/factory/toolkit为零 |
| P1-60 | Partial | plugin admission能检查module/capability/URI等通用边界；无Weather operation/controller/service closure |
| P1-61 | Open | 固定Weather_Storm、timeline、counts、regions、warnings仍在production workspace |
| P1-62 | Partial | Editor logging/diagnostic/export基础可复用；无Weather source/clock/region/subsystem/generation filters |
| P1-63 | Partial | Scene/prefab/PIE/network/save/reimport/hot reload有各自通用基础；Weather identity/override roundtrip为零 |
| P1-64 | Partial | Environment/IBL/Fog/Particle有局部unit/GPU/product tests；无跨Weather visual/audio/data golden |
| P1-65 | Partial | cache/device/cancel/late publish等generic failure基础存在；无Weather whole-chain fault oracle |
| P1-66 | Partial | headless/platform startup与package基础存在；无Weather client/server/editor clean matrix |
| P1-67 | Partial | GPU timing、budget和render统计基础存在；无Weather p50/p95/p99 build/GPU/VRAM/hitch基线 |
| P1-68 | Partial | schema migration、LKG、generation pin/rollback primitive存在；无Weather provider/algorithm canary |
| P1-69 | Partial | package/release manifest基础存在；未列实际Climate/Weather/Environment artifacts和provenance |
| P1-70 | Open | 没有同内容/同质量/同平台/同统计口径的五参考竞争benchmark |

## 8. P2：长期能力（12 Open）

| ID | 状态 | 长期能力 |
|---|---|---|
| P2-1 | Open | physically based atmosphere、multi-scattering、ozone、aerial perspective与LUT streaming |
| P2-2 | Open | compute volumetric cloud、sparse weather fields、shadow maps与temporal reprojection |
| P2-3 | Open | global weather fronts、pressure、humidity、thermodynamics与climate data import |
| P2-4 | Open | terrain snow/ice/wetness/erosion accumulation与streaming sparse masks |
| P2-5 | Open | ocean/wave/fog/rain interaction、splash/foam、wind water与shoreline weather |
| P2-6 | Open | multi-body celestial、eclipse、moon phase、starfield、aurora与calendar localization |
| P2-7 | Open | procedural lightning/thunder propagation、acoustic delay与network synchronized spectacle |
| P2-8 | Open | remote weather provider/live forecast ingestion、cache、permission与deterministic fallback |
| P2-9 | Open | neural/upscale/weather denoiser provider纳入artifact/quality/fallback contract |
| P2-10 | Open | collaborative climate/weather timeline editing、field merge、lock与review annotation |
| P2-11 | Open | weather program schema/algorithm migration、canary、rollback与replay compatibility |
| P2-12 | Open | cross-engine climate/weather benchmark与公开reference scenes/methodology |

## 9. 分层重构顺序

### M0：Truthfulness 与第二 authority 清理

将Weather capability保持Unavailable/Prototype；把workspace明确降为fixture，删除固定成功/计数/警告语义，missing provider、missing asset和unsupported platform必须可见。禁止在M1前新增更多静态Weather controls。

### M1：Source、Clock、Celestial 与 Region

建立versioned Climate/Celestial/Weather/Preset/Timeline sources、stable IDs、CelestialClock、Editor37 SpatialRegion binding及lossless Scene/Prefab/save/net/replay边界。先闭合source authority，再做渲染效果。

### M2：Compiler、Artifact 与 World Authority

建立validator/dependency/compiler、immutable WeatherProgramArtifact、per-World service、atomic WeatherFrameSnapshot、dirty/crossfade/budget、bounded query/event cursor和diagnostics。失败保留上一完整generation。

### M3：Environment Render Adapters

由render owners实现Atmosphere LUT、真实Cloud medium/render/history、Sky/DirectionalLight binding、IBL dirty/crossfade与Fog adapter。Weather只输出同代typed parameters，不拥有GPU资源或froxel。

### M4：Wind、Precipitation、Surface、Sound 与 Lightning

建立WindField、CPU/GPU particle parity、camera-relative precipitation、impact/surface accumulation、Terrain/Material/Sound/Gameplay adapters及deterministic Lightning receipts。

### M5：Transactional Editor Product

接入AssetType、document/transaction、schema inspector、timeline/curve、region gizmo、artifact-aware preview、background jobs、catalog/App/provider closure。所有命令返回job/source/artifact/World generation和terminal disposition。

### M6：Qualification 与竞争基线

完成roundtrip、determinism、network/save/replay、visual/audio/data golden、fault、device-loss、scale、headless、platform matrix、upgrade/rollback与同质量五参考benchmark。未通过前不得发布Stable/Complete。

## 10. 验收门禁（18 Fail / 14 Partial / 0 Pass）

| # | 状态 | 门禁 |
|---:|---|---|
| 1 | Fail | Climate/Celestial/Weather/Region source、revision、artifact、instance、generation identity完整 |
| 2 | Partial | World clocks具pause/rate/generation；Celestial date/geography/season/network/replay determinism未实现 |
| 3 | Fail | Weather compiler/program key、dependency、seed、migration、platform variant正确 |
| 4 | Partial | immutable/generation/budget primitive存在；Weather snapshot/dirty/crossfade/diagnostic未闭合 |
| 5 | Fail | Scene/Prefab/PIE/Save/Network/Replay roundtrip保持Weather authority |
| 6 | Fail | Sky/DirectionalLight/sun/moon identity、photometry、shadow与clock一致 |
| 7 | Fail | Atmosphere LUT numeric/visual/precision/cache/rebuild/fallback golden |
| 8 | Fail | Cloud density/erosion/lighting/shadow/transmittance/history/composite真实有效 |
| 9 | Fail | 伪CaptureCloud已删除，但独立Cloud graph/input/resource/artifact仍不存在 |
| 10 | Partial | PMREM/SH/IEM、generation/time-slice/last-good基础存在；Weather crossfade/device/consumer receipt未闭合 |
| 11 | Partial | Fog typed/froxel/history基础存在；Weather humidity/aerosol/rain/generation未接入 |
| 12 | Fail | Wind field/query direction/speed/gust/turbulence/altitude/region/seed deterministic |
| 13 | Fail | CPU/GPU Particle Weather/external-force schema parity、LOD、budget、cancel无分叉 |
| 14 | Fail | rain/snow/hail spawn/streaming/impact/surface accumulation golden |
| 15 | Fail | Terrain/Material wetness/snow/ice/roughness masks与generation更新正确 |
| 16 | Fail | Sound ambience/reverb/listener/source/lightning PTS与output receipt正确 |
| 17 | Fail | Lightning geometry/flash/cloud/thunder/network/replay/save deterministic |
| 18 | Partial | multi-world/view/camera/listener基础存在；Weather isolation/late-generation fence未验证 |
| 19 | Partial | generic validation/fuzz/budget基础存在；Weather malformed/NaN/LUT/curve/particle矩阵为空 |
| 20 | Partial | atomic artifact/cache/LKG基础存在；Weather GC/platform/equivalence/rollback未验证 |
| 21 | Partial | bounded/cancel/deadline/job primitive存在；Weather retry/shutdown/panic/disk/device恢复未验证 |
| 22 | Partial | generic Editor document/transaction/undo/recovery基础存在；Weather source未接入 |
| 23 | Fail | timeline/curve/region/sky/cloud preview使用真实document/artifact/runtime snapshot |
| 24 | Fail | Build/Preview/Apply/Playtest返回job/source/generation/artifact/diagnostic receipt |
| 25 | Fail | fixed Weather workspace facts全部由真实state替代，missing/error可见 |
| 26 | Partial | catalog/admission机制存在；Weather provider/factory/controller/service closure为零 |
| 27 | Partial | 各domain有局部tests；跨Weather visual/audio/data golden不存在 |
| 28 | Fail | 1/1k/100k regions/particles/cloud tiles/transitions与GPU/VRAM/hitch达标 |
| 29 | Partial | generic platform/headless/capability基础存在；Weather完整矩阵未运行 |
| 30 | Partial | generic diagnostics/export基础存在；Weather同代过滤与receipt证据未接入 |
| 31 | Partial | generic migration/LKG/rollback primitive存在；Weather canary/pin/replay compatibility未验证 |
| 32 | Fail | Stable/Complete由compile、registration、runtime、Editor、fault、platform、scale evidence派生 |

## 11. 禁止的临时修补

1. 禁止把固定Weather_Storm、时间线、层数、region、warning或queued feedback继续当作产品状态。
2. 禁止用`preview_skybox`、gradient shader、DirectionalLight字段或Fog参数拼接Environment authority。
3. 禁止恢复`CaptureCloud` gradient pass，或把source cubemap/IBL更新称为Cloud实现。
4. 禁止只增加Weather/Cloud/Wind/Rain ResourceKind、manifest、descriptor或UI字段而没有compiler/artifact/service/adapter。
5. 禁止让CPU/GPU Particle继续接受不同force/weather inputs而没有typed schema与actual receipt。
6. 禁止在render/particle/audio thread同步执行Weather compile、LUT/Cloud bake或无界复制。
7. 禁止让Weather直接写Fog froxel、Particle pool、Terrain texture、Material buffer或Sound mixer私有状态。
8. 禁止用脚本JSON、display name、control ID或region下拉字符串作为stable Weather identity。
9. 禁止在编译/安装失败时发布半代state，或用默认Clear/Storm静默替换失败source。
10. 禁止以缺失Atmosphere/Cloud/Precipitation后的低耗时宣称性能优于Unreal/Unity。

## 12. 跨计划 Owner 与实施边界

| 领域 | 唯一owner | Editor159只登记的边界 |
|---|---|---|
| Weather authoring/product truth | Editor38 | document/toolkit/transaction/preview/build/diagnostic；实现前保持Unavailable/Prototype |
| Weather executable runtime | Runtime149（历史Runtime36） | program/service/snapshot/query/adapter runtime合同与first-party实现 |
| Spatial Region | Editor37 / Runtime Region owner | stable region source、compiled geometry、index、generation；Weather只持binding和combine policy |
| Environment/IBL/Atmosphere/Cloud | Runtime96及render owners | GPU artifacts、render graph、history、quality、device lifecycle；消费Weather同代adapter |
| Volumetric Fog | Runtime Fog owner | froxel/history/render资源；只接typed Weather target |
| Particle/Surface/Terrain/Material/Sound | 各domain owner | 执行资源与输出receipt；不得复制Weather authority |
| Time/Scene/Net/Save/Operation | 各Runtime owner | 提供通用primitive与persistence/transport，不复制Weather policy |
| App/Catalog/Admission | product composition owners | 显式装配provider/factory/service/toolkit及能力闭包 |

canonical finding计数仍归Editor38，本篇不新增或重复累计5 P0、70 P1、12 P2和32门。Runtime149拥有可执行Weather runtime细化项；Editor159只刷新Editor38 authoring/product truth的当前状态与实施依赖。

## 13. 本轮产出边界

本轮只新增current-source review并更新索引/覆盖记录，不修改Runtime、Editor、Plugin、Interface、App生产代码或tests，不运行Cargo或动态产品验收。最终attestation时共享HEAD由`331668a00d93771f0e22ec7db8538d5d809e3a9d`移动到`2f684e191c5252775b4a192f3ffa77e2f48c6757`，但371个Zircon文件、46个参考文件及联合选择集的统计和三个指纹均保持不变；本报告证据范围无源码漂移。
