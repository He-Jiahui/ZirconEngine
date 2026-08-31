---
title: Runtime Water、Ocean、Lake、River、Wave、Underwater、Buoyancy 与 Query 当前工作树复审
category: zircon_runtime
report_id: Runtime178
review_date: 2026-08-30
baseline_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
verification_head: 248b1d484a16f3826933ddb12ad0c75f0ae06223
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/99zr-runtime-water-ocean-lake-river-surface-wave-fft-shallow-water-rendering-underwater-buoyancy-query-physics-navigation-editor-product-integration-current-source-review.md
related_editor_owner:
  - docs/plans/optimize/zircon_editor/238-editor-water-ocean-lake-river-surface-wave-underwater-buoyancy-authoring-current-working-tree-review.md
related_code:
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime/src/asset/assets/scene
  - zircon_runtime/src/asset/assets/material
  - zircon_runtime/src/scene/components/scene
  - zircon_runtime/src/scene/world/project_io
  - zircon_runtime/src/core/framework/render
  - zircon_runtime/src/graphics/feature
  - zircon_runtime/src/core/framework/physics
  - zircon_plugins/physics/runtime/src
  - zircon_runtime/src/core/framework/navigation
  - zircon_plugins/navigation/runtime/src
  - zircon_runtime/src/core/framework/audio
  - zircon_plugins/particles/runtime/src
  - zircon_plugins/first_party_runtime_catalog
  - zircon_app/src
  - examples/woc/scripts/woc_game/src/world
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/Experimental/Water/Source
  - dev/UnrealEngine/Engine/Plugins/Experimental/Buoyancy/Source
  - dev/UnrealEngine/Engine/Plugins/Experimental/WaterAdvanced/Source
  - dev/UnrealEngine/Engine/Plugins/Experimental/MeshPartitionWater/Source
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Water
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Editor/Water
  - dev/godot/scene/resources/material.cpp
  - dev/godot/servers/rendering/renderer_rd/shaders/effects/screen_space_reflection.glsl
  - dev/bevy/assets/shaders/water_material.wgsl
  - dev/bevy/examples/3d/ssr.rs
  - dev/bevy/crates/bevy_pbr/src/pbr_material.rs
  - dev/Fyrox/fyrox-graphics-gl/src/shaders/shared.glsl
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime178 · Water/Ocean/Lake/River 当前工程化差距

## 1. 结论

当前 Zircon 仍没有 Water runtime 子系统。按当前工作树逐层检查 ResourceKind、Scene/World persistence、asset/import/cook、runtime catalog/App、render feature、material/reflection、time、physics、navigation、audio、particles、terrain/weather、WOC 与 plugin bridge，生产路径没有 `WaterBody`、`WaterZone`、`Ocean`、`River`、`WaveSpectrum`、`WaterQuery`、`Buoyancy` 或 `Underwater` owner。旧 Runtime143 的“无 Water production owner”结论仍成立，当前版本没有出现可关闭缺口的实现。

存在的只是可复用相邻底座：PBR transmission/IOR/attenuation、SSR/Planar/OIT、GPU Scene/visibility、fixed clock、Physics force/impulse、ground Navigation、Sound listener/emitter、Particles CPU/GPU、Terrain carrier。它们没有被统一编译成 Water source -> artifact -> per-World service -> render/query/physics/navigation/audio/VFX/gameplay adapters 的闭环。WOC 的水位、五个圆形湖和 `hasWaterAt` 是示例玩法私有数据；测试里的 `ocean.query.v1` 不是正式 query API。

因此本报告刷新 Runtime143 当前性，**不新增 P0**；新增 **28 项 Water-owned P1（28 Open）**、**12 项 P2（12 Open）**、**24 道资格门（21 Fail / 3 Partial / 0 Pass）**。相邻 owner 的 P0 不在此重复计数，必须通过 owner handoff 解决后才能关闭 Water 依赖门。

## 2. 当前源码证据

### 2.1 Source、Scene、World 与装配

- `zircon_runtime_interface/src/resource/marker.rs` 的 ResourceKind 有 Terrain/NavMesh/Texture/Animation/Sound 等类型，没有 WaterBody/WaterMaterial/Wave/Mask/Simulation。
- `zircon_runtime/src/asset/assets/scene` 与 `scene/world/project_io` 没有 Water component carrier；World 的 terrain/tilemap 等局部字段不能承载水体 geometry、zone、wave 或 generation。
- `zircon_runtime/src/graphics/feature`、first-party runtime catalog 与 App composition 没有 Water module/feature/provider。不存在 Water importer/compiler/build artifact 或 per-World lifecycle。
- `examples/woc/scripts/woc_game/src/world` 的 water height、湖泊 circles、swimming threshold、pathfinding 判定分散在脚本中，无法跨场景、网络、保存、replay 或 renderer 共享。

### 2.2 Render、Wave 与 Underwater

Transmission/SSR/Planar/OIT 是通用渲染基础，不是 Water。当前没有 Ocean clipmap/quadtree、Lake polygon、River strip、shoreline/holes、LOD stitch、wave evaluator、FFT spectrum、shallow-water solver、foam/caustics、waterline、underwater volume 或 depth/velocity field。没有同一 accepted generation 同时输出 current/previous position、normal、velocity、query bounds 和 motion vectors。

### 2.3 Physics、Navigation、Audio 与 Gameplay

Physics shape 没有 Water volume/submersion，command 只有 force/impulse 等通用操作；没有 pontoon、buoyancy、drag/current/slamming、enter/exit/submerge receipt。Navigation 是 ground domain，没有 swim volume/entry/exit/cost；UI 中的 Water area 选项属于 Runtime141 owner。Sound/Particles 有 listener/emitter、external force、CPU/GPU simulation，却没有 shoreline/wake/splash/underwater medium 的 typed stream。没有 server-authoritative Water query/influence contract。

### 2.4 参考引擎差异

Unreal Water/Buoyancy/WaterAdvanced/MeshPartitionWater 把 WaterBody、WaterZone、mesh/quadtree、Gerstner、shallow simulation、query flags/result、Buoyancy pontoon 与 editor/world partition 分开；Unity HDRP Water 把 Ocean/Lake/River/Pool、CPU/GPU search、Fourier/simulation、underwater/waterline、tile classification/tessellation 分开。Godot/Bevy 的 SSR/material 只证明通用折射/法线效果，不能替代 Water domain；Fyrox 仅提供 shader 组织对照。

## 3. P1 重构任务

| ID | 当前问题 | 必须完成 |
|---|---|---|
| RT-WATER-01 | 无 Water resource taxonomy | 增加 WaterBody/WaterMaterial/WaveProgram/WaterMask/ShallowSimulation/WaterZone 类型、serde、asset kind、editor/catalog/ABI 映射。 |
| RT-WATER-02 | 无 source/artifact/instance 分层 | 建 `WaterBodySource -> WaterBuildArtifact -> WaterRuntimeInstance`，source 不被 GPU/physics 回写。 |
| RT-WATER-03 | 无 compiler/dependency closure | 编译 geometry/wave/query/consumer bundle，记录 terrain/river/weather/material/profile/compiler hashes 与 diagnostics/source map。 |
| RT-WATER-04 | 无 WaterService owner | 建 per-World lifecycle、generation、replace/unload/retire、capacity、shutdown/drain 和 fail-closed admission。 |
| RT-WATER-05 | 无 body/zone identity | Ocean/Lake/River/Pool/Custom body、zone priority、overlap stack、tile key、owner epoch 均需 stable identity。 |
| RT-WATER-06 | 无 geometry artifact | 分别编译 ocean grid、lake polygon、river strip、custom mesh、holes/shore/bounds/UV/winding 与 HLOD。 |
| RT-WATER-07 | 无 large-world LOD | clipmap/quadtree/far mesh、camera-relative origin、stitch/morph/skirt、screen-error budget 与 origin shift。 |
| RT-WATER-08 | 无 render producer | Water descriptor/extract/pass/executor 接入唯一 visibility/render graph owner，输出 visible/culled/overflow/fallback receipt。 |
| RT-WATER-09 | 无 material contract | water surface/volume/underwater permutations 接入统一 PBR ABI，显式 water IOR/absorption/scattering/depth 单位。 |
| RT-WATER-10 | 无 reflection/refraction policy | Snell/depth intersection/roughness mip/SSR/Planar/environment fallback、history validity 与 energy budget。 |
| RT-WATER-11 | 无 displacement evaluator | analytic/Gerstner/spectral/baked evaluator 同时输出 height/normal/velocity/max bound/current+previous。 |
| RT-WATER-12 | 无 FFT artifact | spectrum seed/dispersion/bands/inverse transform/CPU fallback/precision与GPU resource generation。 |
| RT-WATER-13 | 无 shallow simulation | stable grid、CFL/step budget、boundary/source/sink、terrain/river coupling、instability/degrade receipt。 |
| RT-WATER-14 | 无 query API | typed surface/depth/normal/velocity/current/immersion/containment flags、miss/error、generation/provenance。 |
| RT-WATER-15 | 无 query acceleration | BVH/tile field、Any/Closest/AllSorted、batch/async/scratch budget；热路径禁止大 Vec 分配。 |
| RT-WATER-16 | 无 underwater/waterline | camera containment、priority、surface crossing、near-plane clip、scattering/extinction/caustics、多 view。 |
| RT-WATER-17 | 无 foam/shore interaction | bounded emitter/history/decay/reset/LOD/overflow，shore wetness/depth color/bank blend。 |
| RT-WATER-18 | 无 caustics | surface-to-receiver pass、receiver mask、depth/energy/temporal/fallback 与 GPU cost receipt。 |
| RT-WATER-19 | 无 physics adapter | Water volume/submersion/pontoon sampling，buoyancy/drag/current/slamming 通过 Physics command/receipt。 |
| RT-WATER-20 | 无 navigation adapter | swim layer/cost/entry/exit/agent mode 与 accepted Water geometry generation，缺 provider typed Unsupported。 |
| RT-WATER-21 | 无 audio/VFX adapter | underwater medium、shore/wave/splash/wake/spray bounded streams，Sound/Particles 各自拥有消费。 |
| RT-WATER-22 | 无 weather/terrain coupling | 只消费 accepted Weather/Terrain/River artifact generation；局部失效要原子 rollback。 |
| RT-WATER-23 | WOC 私有水位 | 迁移脚本到 bounded read-only query/influence API，禁止 server truth 读 render/GPU。 |
| RT-WATER-24 | 无 network/save/replay | body generation、wave seed/time、query policy、swim state、force/event receipt 可序列化/复制/重放。 |
| RT-WATER-25 | 无 scale/residency policy | body/tile/simulation/query/render bytes、GPU time、priority、evict/fallback 和 gameplay truth 隔离。 |
| RT-WATER-26 | 无 diagnostics | source/artifact/runtime/view generation、query latency、LOD/fallback/overflow、force sample、GPU timing。 |
| RT-WATER-27 | plugin/catalog 壳 | provider capability、native/backend ABI、missing backend、load/unload/reload 和 first-party App closure。 |
| RT-WATER-28 | 无质量与故障证明 | Ocean/Lake/River/Underwater/Buoyancy fixture、CPU/GPU parity、device loss、origin shift、1000 bodies、P99/soak/benchmark。 |

## 4. P2 完整度任务

| ID | 必须补齐 |
|---|---|
| RT-WATER-P2-01 | 船体 planing、wake spectrum、推进器/舵与多浮体耦合。 |
| RT-WATER-P2-02 | 双向刚体/浅水耦合、洪水/蓄泄/潮汐与守恒。 |
| RT-WATER-P2-03 | spray/mist/bubble/破浪与粒子物理耦合。 |
| RT-WATER-P2-04 | 侵蚀/泥沙/河床变化与 Terrain 双向更新。 |
| RT-WATER-P2-05 | 冰冻/融化/冰面碰撞与相变材质。 |
| RT-WATER-P2-06 | RT/path tracing 水体反射、折射、caustics 与 denoise。 |
| RT-WATER-P2-07 | aquatic navigation/AI、多运动域群体与水下 gameplay。 |
| RT-WATER-P2-08 | 盐度/温度/污染/生态 field、network/save schema。 |
| RT-WATER-P2-09 | procedural shoreline/island/harbor/wave obstacle。 |
| RT-WATER-P2-10 | 360/VR/stereo water、projection 与 late-latch。 |
| RT-WATER-P2-11 | virtualized water artifacts、CDN/range IO、proxy/cache。 |
| RT-WATER-P2-12 | 跨平台 visual/query/physics golden 与 reproducible benchmark。 |

## 5. 资格门

| Gate | 当前结果 | 通过条件 |
|---|---|---|
| RT-WATER-G01 | Fail | Water source/artifact/instance 可创建、导入、保存、重开、cook、卸载且 identity 稳定。 |
| RT-WATER-G02 | Fail | per-World WaterService 具有 generation/lifecycle/shutdown receipt。 |
| RT-WATER-G03 | Fail | Ocean/Lake/River/Pool geometry 与 large-world LOD 有 CPU/GPU artifact。 |
| RT-WATER-G04 | Fail | render graph/visibility/material/reflection/refraction/underwater 真正消费 Water extract。 |
| RT-WATER-G05 | Fail | analytic/spectral/shallow wave可 deterministic evaluate，且有误差/预算门。 |
| RT-WATER-G06 | Fail | query flags/result/error/acceleration/batch/async contract。 |
| RT-WATER-G07 | Fail | waterline/underwater/foam/caustics 与 multi-view/device-loss。 |
| RT-WATER-G08 | Fail | buoyancy/current/drag/enter-exit 通过 Physics receipt。 |
| RT-WATER-G09 | Fail | swim navigation/Audio/VFX adapters 共享 generation。 |
| RT-WATER-G10 | Fail | WOC 迁移后不再持有私有水位 truth。 |
| RT-WATER-G11 | Fail | network/save/replay 可重现 Water seed/time/query/event。 |
| RT-WATER-G12 | Fail | 1K bodies/large ocean/streaming/residency P99 与 fault tests。 |
| RT-WATER-G13 | Partial | 通用 PBR/SSR/Planar/OIT 底座存在，但无 Water producer 与正确介质参数。 |
| RT-WATER-G14 | Partial | Physics force、fixed clock、Terrain/Sound/Particles carrier 可复用，但没有 Water adapter。 |
| RT-WATER-G15 | Fail | 没有 first-party Water catalog/provider/backend closure。 |
| RT-WATER-G16 | Fail | 无 shader/material/query/physics golden vectors。 |
| RT-WATER-G17 | Fail | 无 origin shift、device loss、partial tile、stale generation fault evidence。 |
| RT-WATER-G18 | Fail | 无 underwater multi-camera/VR/camera-cut output。 |
| RT-WATER-G19 | Fail | 无 security/authority boundary，client GPU 不应成为 gameplay truth。 |
| RT-WATER-G20 | Fail | 无 diagnostics/telemetry for query/render/simulation pressure。 |
| RT-WATER-G21 | Fail | 无 migration from WOC and spline/terrain/weather accepted artifacts。 |
| RT-WATER-G22 | Fail | 无 cross-platform benchmark against Unreal/HDRP reference scenes。 |
| RT-WATER-G23 | Fail | 无 networked replay/save compatibility fixture。 |
| RT-WATER-G24 | Fail | 无 editor/runtime integration receipt，静态 Water UI 不得算完成。 |

## 6. 推荐重构顺序

1. 先冻结 resource/source/artifact/instance、WaterService、query flags/result 和 plugin/catalog capability。
2. 接着完成 geometry/LOD、analytic wave/CPU query，再把 Render/Physics/Navigation 适配器接入同一 generation。
3. 再增加 GPU spectral/shallow simulation、underwater/foam/caustics、Sound/Particles/Weather/Terrain adapters。
4. 最后迁移 WOC 和 River/Spline/Terrain authoring，补 network/save/replay、residency、fault/scale/benchmark。
5. 通用 transmission/SSR/OIT/force 等底座只能作为依赖，不能以“存在”关闭 Water gate。
