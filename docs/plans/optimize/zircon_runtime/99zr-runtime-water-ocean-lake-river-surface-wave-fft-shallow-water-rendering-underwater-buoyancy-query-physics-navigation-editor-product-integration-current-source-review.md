---
title: Runtime Water、Ocean/Lake/River Surface、Wave/FFT、Shallow Water、Rendering、Underwater、Buoyancy、Query、Physics、Navigation、Editor 与 Product Integration 当前源码工程化差距
report_id: Runtime143
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
related_code:
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime/src/asset/assets/scene
  - zircon_runtime/src/asset/assets/material
  - zircon_runtime/src/scene/components/scene
  - zircon_runtime/src/scene/world/project_io
  - zircon_runtime/src/core/framework/render
  - zircon_runtime/src/graphics/feature
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting
  - zircon_plugins/rendering/features/oit/runtime/src
  - zircon_plugins/rendering/features/planar_reflections/runtime/src
  - zircon_runtime/src/core/framework/physics
  - zircon_plugins/physics/runtime/src
  - zircon_runtime/src/core/framework/navigation
  - zircon_plugins/navigation/runtime/src
  - zircon_runtime/src/core/framework/audio
  - zircon_runtime/src/core/framework/sound
  - zircon_plugins/particles/runtime/src
  - zircon_plugins/first_party_runtime_catalog
  - zircon_app/src
  - zircon_editor/assets/ui/editor/components/workbench/modules/extensions/simulation/workbench_extension_navmesh_ai_workspace.zui
  - examples/woc/scripts/woc_game/src/world
tests:
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge/diagnostics.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge/lifecycle.rs
  - zircon_runtime/src/asset/tests/assets/gltf_importer/woc_required_extensions.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/30-water-ocean-lake-river-surface-wave-fft-shallow-water-rendering-underwater-buoyancy-query-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/99a-runtime-advanced-surface-lighting-light-cookie-oit-planar-reflection-subsurface-scattering-clearcoat-anisotropy-transmission-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zm-runtime-physics-world-body-shape-collider-material-joint-query-contact-trigger-fixed-step-jolt-character-controller-vehicle-ragdoll-debug-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zn-runtime-audio-sound-clip-streaming-device-mixer-bus-effect-spatial-occlusion-reverb-timeline-event-voice-chat-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zp-runtime-navigation-navmesh-recast-detour-tilecache-crowd-query-pathfinding-obstacle-off-mesh-link-bake-streaming-world-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99zq-runtime-terrain-landscape-heightfield-clipmap-quadtree-lod-material-layer-virtual-texture-foliage-world-partition-physics-navigation-editor-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/37-volume-zone-trigger-region-gameplay-audio-post-process-environment-authoring-review.md
  - docs/plans/optimize/zircon_editor/38-weather-climate-time-of-day-wind-precipitation-cloud-atmosphere-environment-authoring-review.md
  - docs/plans/optimize/zircon_editor/39-spline-path-road-river-decal-brush-geometry-authoring-review.md
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
---

# Runtime Water、Ocean/Lake/River、Wave/FFT、Underwater、Buoyancy 与 Product Integration 当前源码工程化差距

## 1. 结论

当前 Zircon 仍然没有 Water 运行时子系统。对 `zircon_runtime`、`zircon_runtime_interface`、`zircon_plugins`、`zircon_editor`、`zircon_app` 和 `examples` 中排除 tests/target 后的 **12,207 个 production Rust 文件**执行精确词边界搜索，`Water/Ocean/River/Lake/Buoyancy/Underwater/Caustic/Gerstner/ShallowWater/WaterBody/WaterZone` 命中文件数为 **0**。`ResourceKind`、Scene schema、World project I/O、`BuiltinRenderFeature`、`RenderProductFeature`、首方 runtime catalog、App composition、Physics shape/command、Navigation area/agent 均没有 Water owner、Water capability 或 Water generation。

仓内并非毫无可用代码。Standard PBR transmission/IOR/attenuation、SSR、Planar Reflection、OIT、render graph、fixed clock、Physics `ApplyForce/ApplyImpulse`、ground Navigation、Sound volume/listener 和 Particles CPU/GPU simulation 都是真实底座；但它们没有被任何 owner 组合为 Water source、build artifact、per-World lifecycle、surface geometry、wave/query、underwater、buoyancy 或跨域 receipt。**通用底座存在不等于 Water Partial Ready，更不等于产品完成。**

产品侧还存在两个会制造错误印象的入口。Navmesh AI Workbench 的 area 下拉框静态列出 `Water`，但 route 只回填固定 rebuild/query feedback，没有 Water 或 Navigation domain handler；该问题已由 Runtime141 `NAV-P0-005` 拥有。WOC 则在脚本里硬编码水位 `-4.5`、五个圆形湖和私有 `hasWaterAt` 线性扫描，游泳、mob motion 与 pathfinding 直接消费这些标量；它是迁移夹具，不是引擎 Water 系统。`ocean.query.v1` 仍仅为 plugin bridge 测试 trait，`water_elemental.glb` 仍仅为 glTF volume-material fixture。

本篇不重复创建已有 owner 的 P0，登记 **0 项新的 Water-owned P0**；历史 62 项 P1 重判为 **49 Open / 13 Partial / 0 Closed**，14 项 P2 全部 Open；44 项资格门为 **40 Fail / 4 Partial / 0 Pass**。13 个 Partial 只表示相邻通用 owner 已有可复用前置，不表示 Water 领域链已经启动。目标必须硬切到：

```text
WaterBodySource
  -> WaterCompiler
  -> WaterBuildArtifact
  -> per-World WaterRuntimeService + accepted generation
  -> Render / Query / Physics / Navigation / Audio / VFX / Gameplay adapters
  -> typed build/frame/query/force/execution receipts
```

## 2. 审查边界、方法与 currentness

### 2.1 冻结范围

冻结基线为 `main@9db1492cffcfd605aa2d68e562555b8adc6d3b8b` 的当前 working bytes。冻结时共享工作树已有 3,220 个 tracked changes、4,385 个含 untracked changes（不含本篇新增报告）；本篇保留所有既有改动，不归因、不覆盖、不回退。用户已明确排除 tooling，因此没有扫描或规划未来将被 Rust 替换的 tooling 实现。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 工作树指纹 |
|---|---:|---|
| Resource、Scene、World、catalog 与 App carrier | **70 / 13,772 / 12,672 / 496,007 / 116 / 0** | `bf0251963caf0289305976298b947686d51a0d8120d17661b30adf690e237b86` |
| Material、feature、Transmission、SSR、OIT 与 Planar 底座 | **124 / 17,370 / 16,153 / 651,657 / 138 / 0** | `2f47cd984dae9df366adb8fcdee5df56530b77c6f0cc520d19ebaacd147d86c7` |
| Physics、Navigation、Audio、VFX 与 Time adapter 底座 | **179 / 16,375 / 14,802 / 542,240 / 80 / 0** | `1c836b3d0892b76a1456291024a5caffd15fbb4c1c6902ccce5dc208dacc088f` |
| WOC、Editor 与测试 fixture | **13 / 6,980 / 6,622 / 216,575 / 23 / 0** | `db189a0c90085da75d3f156b37874b43fb60141dfbb6eabbef81586f1c2eef7c` |
| Zircon selected union | **382 / 54,183 / 49,954 / 1,896,370 / 357 / 0** | `400cf58678600188fcd7b9eda97ca5f1efe18d5c4f1976b9c91b109974765c0f` |
| Unreal Water family | **227 / 50,524 / 41,467 / 1,946,934 / 0 / 0** | `dffc4dd6d3e16b467edd4e11cdda16ae380f8a1360c0ebe34f42b31fc66c47b0` |
| Unity HDRP Water Runtime + Editor | **71 / 15,699 / 13,179 / 716,049 / 0 / 0** | `b107d3052b2643a0e6531b66e1e4d12d0da03cb31a33e53e421842e72c56dbb1` |
| Godot、Bevy、Fyrox 通用对照 | **6 / 7,374 / 6,419 / 281,457 / 0 / 0** | `6eda8ef2f886fa4921743df2fe4b887cbb75f1f36fa2d599a8469fc3a34dd8a4` |
| reference selected union | **304 / 73,597 / 61,065 / 2,944,440 / 0 / 0** | `2cdf4129fac4abc3290533e546f68e4b329b48454d423e83fbb5bb7ae20c3a43` |
| all selected | **686 / 127,780 / 111,019 / 4,840,810 / 357 / 0** | `a9dd6249b30a02edcc2c38066ec6c06c346c56f189437ddbe9fceafd8508d69c` |

指纹算法为：repository-relative path 转 `/` 并小写排序；每个文件取当前 bytes 的 SHA-256；聚合输入为 `path + NUL + lowercase(file_sha256) + LF`，再取 SHA-256。参考树不受 Git 跟踪状态假设约束，按实际文件 bytes 计算。

### 2.2 纵向扫描链

本轮逐层核对：resource kind/source schema -> importer/compiler/cooked artifact -> Scene persistence/prefab/clone -> World load/save/replace/unload -> package/catalog/App selection -> body/zone/priority -> geometry/LOD/culling/pass -> material/reflection/refraction/transparency -> wave/FFT/shallow/current/query -> underwater/waterline/foam/caustics -> physics/buoyancy/navigation -> audio/VFX/weather/terrain/gameplay -> partition/residency/scalability/diagnostics -> Editor/product/benchmark。未发现 facade、feature slot 或插件测试之后隐藏的第二套 Water production owner。

### 2.3 证据等级与限制

本篇达到 E3 source-level review。没有运行 Cargo、App、Editor、WGPU、Jolt、Recast、asset cook、save/reopen 或 GPU capture，因为当前没有 Water source/component/provider/pass 可执行；运行通用 transmission、SSR、Planar、OIT 或 ApplyForce 测试不能提高 Water 结论。实施前必须重取指纹，并从 capability truth、schema roundtrip 和 CPU query oracle 的 RED 证据开始。

## 3. 当前产品链事实

### 3.1 Source、Scene 与 World

1. `ResourceKind` 已包含 Terrain、TerrainLayerStack、NavMesh 等资源，但没有 WaterBody、WaterMaterial、WaveSpectrum、WaterMask 或 ShallowSimulation。
2. `SceneEntityAsset` 只有 camera、mesh、lights、post-process、physics、animation、terrain、tilemap、prefab 与 scripts，没有 Water component，也没有可让任意插件无损持久化的通用 typed component payload。
3. `SceneNode/NodeRecord` 没有 Water。`World::to_scene_asset` 当前仍把 `terrain` 和 `tilemap` 写为 `None`；脚本 binding 与 prefab dynamic component 的局部桥不能推导出 Water roundtrip。
4. 没有 `zircon_plugins/water`、Water importer/compiler/artifact store、first-party package、module factory、provider capability 或 App selection。

### 3.2 Render

1. `BuiltinRenderFeature` 和 `RenderProductFeature` 都没有 Water；advanced slots 即使有 Terrain 等 descriptor-only 入口，也没有 Water slot。
2. Standard PBR 已有 clearcoat、anisotropy、specular/diffuse transmission、thickness、IOR 和 attenuation，但默认 IOR 是通用介质的 1.5；水的 1.333 只在 glTF fixture 中出现。
3. Runtime100 已确认 transmission 仍是 `normal.xy * (ior - 1) * thickness * 0.02` 的单次屏幕偏移，没有 Snell ray、Hi-Z/exit surface、roughness mip、off-screen fallback 或 temporal；scene copy 与 forward fog 还会重复应用体积雾。
4. Planar Reflection 有真实 capture/filter executor，但普通 Scene 没有 producer；所有 probe 共享一张 persistent texture，texture 与最小 `probe_id` 参数可错配。
5. OIT 有真实 fragment store/resolve，但 HDR 被压为 RGBA8、固定 K 溢出无 telemetry，且 descriptor 未声明 executor 读取的 Volumetric/Transmission 资源。Water 不得复制一套私有透明合成来绕开这些 owner 问题。

### 3.3 Physics、Navigation、Audio 与 VFX

1. `PhysicsColliderShape` 只有 Box/Sphere/Capsule/Cylinder/ConvexHull/TriangleMesh/HeightField/Compound；不存在 Water volume/submersion。`BodyCommand` 只有速度、force/impulse、teleport、body type、CCD 与 sleep，Jolt 只是直接转发 force/impulse。
2. Navigation 默认 area 只有 not-walkable、walkable、jump；agent 是 ground NavMesh 模型，off-mesh motion 只有 Linear/Parabolic。静态 UI 中的 `Water` 选项没有对应 area ID、swim locomotion 或 runtime receipt。
3. Sound 有 listener/emitter/volume 与 source-environment 底座，Particles 有 CPU/GPU simulation 和 external force，但没有 underwater medium、shore/wave emitter、splash/wake stream 或 Water sample handle。
4. fixed clock/time owner 可复用，但没有 Water simulation state、phase、history、pause/seek/reset 或 world retirement。

### 3.4 Product 与 fixture

1. `OceanQueryInterface` 仅定义在 plugin extension bridge test，`INTERFACE_ID = "ocean.query.v1"`，sample height 只是温度加二；diagnostics/lifecycle 中的 `ocean.runtime` 只验证 generation、enable/disable 与 unload。
2. `water_elemental.glb` 的测试只验证 IOR、transmission、thickness 与 attenuation 能进入普通 MaterialAsset；它没有 body identity、surface、wave、query 或 buoyancy。
3. Navmesh AI Workbench 固定列出 `Walkable/Jump/Door/Water`，但 action/commit 只是模板 route 和固定 feedback。该假产品问题归 Runtime141，不在本篇重复 P0。
4. WOC `terrain_content.zr` 硬编码一个水位、blend multiplier 与五个圆形湖，`hasWaterAt` 逐个线性判断。`player_motion.zr` 用 `waterHeight - 0.75` 和 ground threshold 决定 swimming/treading；`pathfind_state.zr`、`mob_motion_world.zr` 重复私有水位/可游泳判定。Water normal texture 资产没有 production engine consumer。

## 4. 必须保留的真实底座

1. 保留统一 MaterialAsset/PBR ABI，把 Water material 编译为现有材质与 pipeline key；禁止另建孤立水材质参数系统。
2. 保留 render graph、frame extract、GPU Scene/visibility、SSR/Planar/OIT/Transmission owner；Water 只提供 typed producer、eligibility、resource plan 与 receipt。
3. 保留 Physics world、generation handle、fixed-step 与 command buffer；buoyancy/current/drag 计算结果通过 Physics command 提交，不建立第二个 rigid-body world。
4. 保留 Navigation owner和其未来多运动域边界；Water 发布 geometry/cost/entry/exit 输入，不复制 pathfinder，也不把水面 quad 伪装成地面。
5. 保留 plugin interface slot/generation/weak import/unload 语义；把测试 trait 替换为正式 Water query contract，而不是从测试模块 re-export。
6. 保留 Sound volume/listener、Particles bounded simulation 与 Runtime time owner，新增 Water adapter 而非侵入各域私有状态。
7. WOC 五湖、游泳和路径判定只保留为 migration/characterization fixture，用于证明迁移前后玩法等价；不得作为产品验收终点。

## 5. P0 owner 裁决：本篇 0 项新增，不代表无阻塞

| 既有 owner | 当前仍阻塞 Water 的问题 | 本篇边界 |
|---|---|---|
| Runtime100 / 旧 Runtime09G2 | Scene producer、唯一 capability truth、统一透明合成、工程级 transmission、fog 重复、Planar 多 probe、OIT graph ABI 与产品证据 P0 全未关闭 | Water 只声明 consumer 条件，不复制 reflection/transparency 修复 |
| Runtime141 Navigation | `NAV-P0-005` 固定 Navmesh AI Workbench 状态/feedback；其中 `Water` 选项没有 runtime 执行链 | 入口先标 Demo/Unavailable，直到真实 Water/Nav receipt |
| Runtime142 Terrain | Terrain provider/World/render/product chain 仍断裂，无法提供可信 bottom/shore/partition generation | Water 只消费 accepted Terrain artifact |
| Editor37 | Volume/Zone/Region authoring 与跨域 consumer truth 未闭合 | Water runtime 拥有 medium resolve，不复制通用 authoring tool |
| Editor38 | Weather/Wind/Precipitation authoring与运行时 generation 未闭合 | Water 只消费 accepted weather snapshot |
| Editor39 | 空间 Spline、plugin Scene persistence、River source/compiler/receipt 与静态 River 假产品问题仍开放 | River authoring归 Editor39；Water 从 accepted artifact 开始 |
| Runtime08A / Runtime142 Physics事实 | 通用 Physics 没有 buoyancy/field owner，HeightField/Terrain generation 也未闭合 | 本篇拥有 Water-side sampling/force model，不接管 solver/backend |

以下任一行为必须回写上述 P0 owner或建立 failure handoff：把静态蓝色 mesh、WOC 五湖、normal 滚动、`water_elemental.glb`、`ocean.query.v1`、SSR/Planar/OIT 启用、ApplyForce 存在或 Navmesh UI 的 `Water` 选项展示为 Water Ready/Executed。

## 6. P1：Source、Artifact、Scene、Body/Zone 与 Runtime Lifecycle

| ID | 状态 | 当前源码证据 | 硬切重构要求 |
|---|---|---|---|
| WAT-P1-001 | Open | `ResourceKind` 无任何 Water 资源 | 定义 canonical WaterBody/Material/Wave/Mask/Shallow resource kind、schema version、URI 与 provider owner |
| WAT-P1-002 | Open | Scene 静态字段无 Water，plugin payload 只对脚本/prefab有局部桥 | 建统一 typed plugin component persistence；clone/prefab/snapshot/save/reopen/play/export 全链无损 |
| WAT-P1-003 | Open | 无 world-scoped Water service | 建 `WaterRuntimeService`，覆盖 requested/preparing/resident/attached/retiring/failed/cancelled 与 replace/unload |
| WAT-P1-004 | Open | catalog、App、render profile均无 Water package/capability | 无 provider 时统一 Unsupported；落地后发布 requested/eligible/active/degraded/reason/generation/cost |
| WAT-P1-005 | Open | 无 Ocean/Lake/River/Transition/Custom body 语义 | 定义 sealed body kind及各自 geometry/query/collision资格，River source仍归 Editor39 |
| WAT-P1-006 | Open | 无 body/zone/tile/wave/simulation identity | 复用 Runtime24 stable ID、generation、owner epoch；所有 ticket/result拒绝 stale apply |
| WAT-P1-007 | Open | 无 source/artifact/live instance分层 | 硬切 `WaterBodySource -> WaterBuildArtifact -> WaterRuntimeInstance`，authoring不可被GPU/physics/nav回写 |
| WAT-P1-008 | Open | 无 compiler/version/profile/dependency hash/build receipt | composite key覆盖source/material/spectrum/terrain/river/weather/profile/compiler并支持精确失效 |
| WAT-P1-009 | Open | 无 body overlap、zone priority、transition resolve | 建空间索引与deterministic resolve，返回chosen body、overlap stack、reason、generation |
| WAT-P1-010 | Open | 无单位、finite、extent、depth、flow、budget校验 | validator拒绝NaN/Inf、负深度、零extent、非法priority/spectrum和超预算输入；migration fail-close |

## 7. P1：Geometry、LOD、Culling、Pass 与 Large World

| ID | 状态 | 当前源码证据 | 硬切重构要求 |
|---|---|---|---|
| WAT-P1-011 | Open | `BuiltinRenderFeature`与umbrella均无 Water | 建真实 Water descriptor/extract/phase/pass/executor/receipt；不得借 Transparent Mesh 改名 |
| WAT-P1-012 | Open | Frame/Geometry extract无body/tile/simulation handle | 新增 immutable `WaterExtract`，只传identity/generation/handles/view decision，不复制大网格 |
| WAT-P1-013 | Open | 无 body kind 到 topology 的 compiler | Ocean grid、Lake polygon、River strip、Custom mesh分别编译稳定winding/UV/bounds/hole/shore artifact |
| WAT-P1-014 | Open | 无 camera-relative ocean grid/far mesh/horizon | 建 clipmap/quadtree/far mesh 与大世界局部原点，控制精度、overdraw和sky horizon |
| WAT-P1-015 | Open | Editor39 River artifact无 Water runtime consumer | 只消费accepted surface/flow generation，验证cell continuation/confluence，不重新解释spline source |
| WAT-P1-016 | Open | 无 screen-error LOD/tessellation/tile profile | CPU reference与GPU route共享误差目标；按view/body/profile记录选择reason/cost |
| WAT-P1-017 | Open | 无 stitch/morph/skirt/displacement边界合同 | 规定neighbor约束，保证surface/normal/velocity在LOD/cell边界连续无裂缝 |
| WAT-P1-018 | Partial | 通用GPU Scene/HZB/indirect底座存在，无 Water packet | 接入唯一visibility owner，发布visible/culled/dropped/overflow和确定性fallback |
| WAT-P1-019 | Open | 无 Water depth/GBuffer/forward/shadow/motion/picking资格 | 同一accepted geometry/wave generation服务全部pass与previous state，禁止beauty-only |
| WAT-P1-020 | Open | 无 mask/excluder/island/interior hole | 编译共享GPU/CPU边界artifact；containment、render、physics、nav使用同一generation |
| WAT-P1-021 | Open | 无 Water partition/HLOD/streaming bundle | surface/far mesh/simulation/query/consumer outputs按cell原子attach/evict，partial failure rollback |
| WAT-P1-022 | Open | 无origin shift、负坐标、多view稳定规则 | body/tile/query使用local origin+stable grid key；shift不改变identity、phase和接缝 |

## 8. P1：Material、Reflection/Refraction、Transparency、Foam、Caustics 与 Underwater

| ID | 状态 | 当前源码证据 | 硬切重构要求 |
|---|---|---|---|
| WAT-P1-023 | Partial | PBR transmission/IOR/attenuation真实存在，无Water domain/compiler | 定义 WaterSurfaceMaterial source并编译到统一ABI，明确depth/opaque/transparent/waterline/underwater permutations |
| WAT-P1-024 | Open | 通用IOR默认1.5，1.333只在fixture | Water profile显式携带介质参数、单位与provenance，不污染通用PBR默认 |
| WAT-P1-025 | Partial | transmission只有normal.xy固定偏移与LOD0 scene sample | 依赖Runtime100实现Snell/depth intersection、roughness mip、edge validity、off-screen fallback和typed quality tier |
| WAT-P1-026 | Open | attenuation使用author thickness而非真实光程 | 建surface/depth/bottom合同，以world-space path length做absorption/scattering并声明未知底深fallback |
| WAT-P1-027 | Partial | SSR/Planar/environment provider存在，无Water resolve | 按view/body/roughness/validity融合，定义energy、priority、fallback、history与receipt |
| WAT-P1-028 | Open | Water无SSR miss/cut/edge/occlusion policy | 接Runtime history/validity，防止黑洞、泄漏、双反射和ghost，所有fallback reason-coded |
| WAT-P1-029 | Partial | OIT/transparent基础存在且自身仍有P0，无Water segment顺序 | 统一mesh/sprite/particle/fog/transmission/Water的深度段吸收与散射，禁止私自重放背景 |
| WAT-P1-030 | Open | 无位移、normal、velocity、motion同源 evaluator | wave evaluator同时输出current/previous position、normal、velocity与max bound，CPU/GPU受误差门 |
| WAT-P1-031 | Open | 无shore/crest/interaction foam source/history | 建bounded foam emitter/field、decay、LOD、overflow、reset与deterministic replay合同 |
| WAT-P1-032 | Open | 无caustics生成、投影、receiver或temporal | 建surface-to-receiver pass、mask、energy、depth、quality fallback与GPU cost receipt |
| WAT-P1-033 | Open | 无underwater volume/camera containment/medium resolve | body/zone query解析介质、priority与generation，覆盖camera cut、surface crossing、多view和缺depth |
| WAT-P1-034 | Open | 无waterline/near-plane/半入水合同 | 独立waterline/clip path处理波峰穿越、MSAA/temporal和反射/折射两侧顺序 |
| WAT-P1-035 | Open | 无shore wetness/puddle/depth color/bank blend | 消费accepted Weather/Terrain generation，不读取未版本化全局shader参数 |
| WAT-P1-036 | Open | 无current/deformation/foam/mask统一注入 | 建bounded Water influence/decal provider，含bounds/priority/lifetime/generation/admission/overflow |

## 9. P1：Wave、Spectrum/FFT、Shallow Simulation、Current 与 Query

| ID | 状态 | 当前源码证据 | 硬切重构要求 |
|---|---|---|---|
| WAT-P1-037 | Open | 无wave/spectrum/band/seed/phase/quality schema | 定义versioned wave program，区分analytic、spectral、baked、shallow、external provider |
| WAT-P1-038 | Open | 无Gerstner等CPU/GPU reference evaluator | 单一坐标/公式合同输出height/normal/velocity/max bound并通过跨平台误差门 |
| WAT-P1-039 | Open | 无FFT initialization/dispersion/frequency band/inverse transform | 建可复现spectrum artifact、GPU resource、band budget和CPU降级；normal贴图不能替代波高 |
| WAT-P1-040 | Partial | Runtime有time/fixed clock，无Water state/phase/history | 每body/generation消费唯一时间域，支持pause/seek/reset；camera FPS不改变phase，unload完整退休 |
| WAT-P1-041 | Open | 无浅水高度/速度场、边界或live/baked路线 | 定义stable grid、CFL/step budget、terrain/river boundary、source/sink与typed instability/degrade |
| WAT-P1-042 | Open | 无flow field查询、depth profile、confluence | 消费Editor39 flow artifact，按位置/深度/时间/generation/LOD确定性采样 |
| WAT-P1-043 | Open | 无deformer/wake/impulse/rain/boat反馈队列 | bounded ingress、tile routing、admission/backpressure和feedback artifact；线程不可直写GPU texture |
| WAT-P1-044 | Open | 正式Water query contract为0 | 定义body/surface/depth/normal/velocity/flow/wave/containment flags、typed miss/error、generation/provenance |
| WAT-P1-045 | Open | 无query acceleration/batch/async/scratch预算 | 建body/zone BVH与tile field view，支持Any/Closest/AllSorted和caller buffer，热路径不分配大Vec |
| WAT-P1-046 | Open | 无CPU/GPU/query determinism、save/network/replay边界 | gameplay truth使用analytic/baked/CPU route；视觉GPU高频细节显式声明是否authoritative |

## 10. P1：Physics、Navigation、Audio/VFX、Weather、Terrain 与 Gameplay

| ID | 状态 | 当前源码证据 | 硬切重构要求 |
|---|---|---|---|
| WAT-P1-047 | Open | shape/world无Water volume与generation | Water侧建broad volume/query adapter；Physics仍拥有body/shape，不把水面做巨大碰撞平面 |
| WAT-P1-048 | Partial | ApplyForce/Impulse与fixed command buffer存在，无buoyancy | 建Buoyancy service，按pontoon/volume采样surface/depth/normal/velocity，输出bounded force/torque receipt |
| WAT-P1-049 | Open | 无linear/angular drag、relative current、slamming/planing基础 | 先实现有单位、上限、能量检查的hydrodynamic model，全部经Physics command提交 |
| WAT-P1-050 | Open | 无enter/exit/submerge/body-changed事件 | fixed step按generation去抖、排序和限流；world replace/body overlap不产生幽灵事件 |
| WAT-P1-051 | Partial | ground NavMesh/Crowd存在，无swim/water domain | 等Runtime Navigation多运动域后接Water layer/cost/entry/exit；缺能力时typed Unsupported |
| WAT-P1-052 | Partial | Sound listener/emitter/volume存在，无Water medium | Water发布zone/listener medium与bounded shoreline/wave emitters，Sound拥有voice/mix/occlusion |
| WAT-P1-053 | Partial | Particles CPU/GPU和external force存在，无Water interop | 发布typed bounded splash/spray/wake/impact stream与surface sample handle，禁止逐粒子同步查询 |
| WAT-P1-054 | Partial | Weather历史owner存在但runtime链未闭合，无Water consumer | 只消费accepted Wind/Precipitation generation，定义wind-to-spectrum/rain impulse/wetness fallback |
| WAT-P1-055 | Partial | Terrain carrier存在但World/render仍断，River artifact亦未闭合 | 以artifact ID+generation原子attach，更新只失效相交Water tiles和consumer outputs |
| WAT-P1-056 | Open | script/gameplay/network无Water capability/authority/budget | 暴露read-only bounded query与受控influence command；server truth不依赖client GPU readback |

## 11. P1：Scalability、Observability、Product Evidence 与 Competitive Gate

| ID | 状态 | 当前源码证据 | 硬切重构要求 |
|---|---|---|---|
| WAT-P1-057 | Open | 无Water quality/profile/platform matrix | profile覆盖geometry、bands/resolution、reflection、foam、caustics、underwater与query route，降级reason-coded |
| WAT-P1-058 | Open | 无CPU/GPU/memory/IO/residency/pressure policy | 每body/tile记录bytes/time/priority；视觉可降级，gameplay query truth不可随view eviction消失 |
| WAT-P1-059 | Partial | 通用multi-view/history/capture底座存在，无Water合同 | view family共享immutable simulation，独立view resources/history；cut/resize/device loss按generation失效 |
| WAT-P1-060 | Open | 无Water stats/diagnostics/execution receipt | 发布source/artifact/runtime/view generation、tiles、query latency、force samples、GPU timing、fallback/drop reason |
| WAT-P1-061 | Open | 无Ocean/Lake/River/underwater/buoyancy产品fixture | 建数值oracle、跨域scene、WGPU capture、headless query/physics、roundtrip/export/soak证据链 |
| WAT-P1-062 | Open | 无同口径Unreal/HDRP benchmark | 固定硬件、分辨率、场景、质量、view path和reference commit，对比image error、CPU/GPU、memory、stutter、query吞吐 |

## 12. 历史 Runtime30 台账重判

| 组 | 历史条目 | 当前结果 | 判定依据 |
|---|---:|---:|---|
| Source/Artifact/Scene/Lifecycle | 001..010 | 10 Open | 精确production搜索为0，Scene/World/catalog均无Water owner |
| Geometry/LOD/Pass/Large World | 011..022 | 11 Open / 1 Partial | 仅通用GPU Scene/HZB/indirect是可复用前置 |
| Material/Reflection/Underwater | 023..036 | 10 Open / 4 Partial | PBR、Transmission、SSR/Planar/OIT真实但无Water producer，且Runtime100 P0未关 |
| Wave/FFT/Shallow/Query | 037..046 | 9 Open / 1 Partial | 只有通用time/fixed clock；wave/query领域为0 |
| Physics/Nav/Audio/VFX/Weather/Terrain | 047..056 | 4 Open / 6 Partial | 相邻owner存在但Water adapter/generation/receipt全部缺失 |
| Scalability/Product | 057..062 | 5 Open / 1 Partial | 通用multi-view/history存在，Water profile/telemetry/product/benchmark为0 |
| 合计 | 62 | **49 Open / 13 Partial / 0 Closed** | 无任何Water端到端条目达到关闭条件 |

旧报告的架构方向仍成立，但其冻结基线、输入规模和部分“无普通Scene producer/World persistence”证据已由 Runtime100/141/142 的 current-source 结论具体化。本篇取代 Runtime30 的状态判断，不删除历史报告。

## 13. P2：MVP 工程闭环后的扩展能力

| ID | 状态 | 扩展项 | 前置条件 |
|---|---|---|---|
| WAT-P2-001 | Open | 船体planing、wake谱、推进器、舵和多浮体耦合 | P1 buoyancy/current/query与Physics constraint稳定 |
| WAT-P2-002 | Open | 双向刚体/浅水耦合与大物体位移体积 | shallow solver、bounded feedback、能量/稳定性门通过 |
| WAT-P2-003 | Open | 破浪、spray、mist、bubble与白沫物理耦合 | 基础foam/VFX stream和透明介质预算完成 |
| WAT-P2-004 | Open | 河网洪水、蓄泄、潮汐、流量守恒与水位传播 | River network与shallow artifact稳定 |
| WAT-P2-005 | Open | 侵蚀、泥沙、河床变化与Terrain双向更新 | Terrain局部失效与原子bundle成熟 |
| WAT-P2-006 | Open | 冰冻、融化、冰面碰撞与相变材质 | Weather temperature authority和Water lifecycle完成 |
| WAT-P2-007 | Open | 高级水下参与介质、色散与体积caustics | 统一介质、Transmission和基础underwater闭环 |
| WAT-P2-008 | Open | Path/Ray Tracing Water反射、折射与caustics | 正式RT/SBT/denoise和Water material ABI完成 |
| WAT-P2-009 | Open | 海岸浪破碎、bathymetry spectrum与shoaling | bottom query、CPU/GPU parity与shore mask稳定 |
| WAT-P2-010 | Open | 网络化Ocean phase与大世界simulation streaming | authoritative time/seed、partition、replay门通过 |
| WAT-P2-011 | Open | 水下AI、aquatic nav volume与群体行为 | 多运动域Navigation和Water volume query完成 |
| WAT-P2-012 | Open | 水生生态、污染、温度/盐度gameplay field | Water field schema、save/network contract完成 |
| WAT-P2-013 | Open | Procedural shoreline/island/harbor/wave obstacle | source/compiler/diagnostic/deterministic geometry完成 |
| WAT-P2-014 | Open | ML辅助spectrum/shore拟合 | 只输出可审查source，不直接写live simulation |

P2 不得替代基础真实度。Water source、surface render、wave/query、underwater、buoyancy、generation adapters和产品证据均属于 P1 工程基线。

## 14. 五套参考源码的正确用法

### 14.1 Unreal：采用 owner 分层，不复制 UObject 层级

冻结的 227 个 Unreal Water family 文件明确分开 WaterBody Actor/Component、Ocean/Lake/River/Custom、WaterZone、WaterSubsystem、WaterMesh/QuadTree CPU/GPU builder、WaterInfo rendering、Gerstner source/evaluator/subsystem、Buoyancy component/manager/types、baked shallow simulation、Niagara/EQS、Spline/Editor/HLOD/Terrain integration。`EWaterBodyQueryFlags` 可选择 location、normal、velocity、depth、immersion、waves；query result只允许读取已请求字段，`TryQueryWaterInfoClosestToWorldLocation` 还返回失败原因。Buoyancy 数据包含pontoon、damping、linear/quadratic/angular drag、current/river force、force cap和async输入。

应吸收的是 identity/generation、query flags/result/error、body/zone/mesh/wave/physics owner separation和同一generation跨consumer发布；不应复制 UObject hierarchy、全局subsystem或把 Experimental 插件当性能上限。

### 14.2 Unity HDRP：采用 rendering/simulation/query 分层，不误补 gameplay owner

冻结的 71 个 Water Runtime/Editor source 文件把 `WaterSurfaceType` 分为 OceanSeaLake、River、Pool，把 geometry 分为 Quad、Custom、InstancedQuads、Infinite，并单独维护 GPU simulation、CPU simulation/search、deformation/current/foam、Fourier transform、Water GBuffer、tile classification/tessellation、underwater与waterline。

`WaterSearchParameters` 显式包含target/start position、目标误差、最大迭代、是否含deformation、是否排除simulation和是否输出normal；`WaterSearchResult` 返回projected/candidate position、normal、current direction、迭代数和最终误差。CPU scripting route与GPU readback/资源资格明确分开。Underwater route先按infinite plane或volume bounds与priority选surface，再建立waterline buffer、camera-height输入、scattering/extinction与caustics参数。

Unity Graphics主要提供SRP rendering/simulation，不提供完整gameplay buoyancy/authority。Zircon应借其水面渲染和查询误差合同，再结合自身Physics/World架构，而不是把HDRP类层次照搬进Runtime。

### 14.3 Godot：通用refraction/SSR参考，不是Water系统参考

Godot StandardMaterial refraction会申请screen/depth texture并处理normal；RD SSR shader使用Hi-Z、normal/roughness、step count、distance fade和depth tolerance。当前参考树没有等价的Water Body、Ocean wave query或buoyancy owner。它证明通用surface effect与领域Water是两件事，也直接否定“Zircon已有SSR所以Water部分完成”的说法。

### 14.4 Bevy：插件化材质参考，不是内建Water owner

Bevy `ssr.rs` 用 `ExtendedMaterial`和四层滚动normal map构造示例水面，且明确需要deferred SSR；shader本身只扰动PBR normal。它适合作为material extension与pipeline接入参考，没有body、zone、wave height、underwater、query或buoyancy。

### 14.5 Fyrox：仅采用Rust shader组织证据

本轮只找到共享GLSL中的通用Fresnel等基础，没有与Unreal/HDRP等价的Water production subsystem。报告不因用户列出参考引擎就虚构能力；Fyrox在本域只提供Rust renderer/shader组织对照。

## 15. 目标架构与硬切合同

```text
Editor authoring truth
  WaterBodySource { Ocean | Lake | RiverRef | Custom }
  WaterMaterialSource + WaveProgram + Zone/Exclusion policy
  accepted RiverFlowArtifact / TerrainBottomArtifact / WeatherSnapshot
                          |
                          v
WaterCompiler
  validate units/ranges/budgets/overlap
  compile surface/far-mesh/mask/shore artifacts
  compile analytic/spectral/shallow/query artifacts
  compile partition/HLOD/consumer dependency bundle
  emit deterministic WaterBuildReceipt
                          |
                          v
WaterBuildArtifact { stable id, generation, hashes, profile, costs }
                          |
                          v
WaterRuntimeService (one owner per World)
  body/zone registry + spatial index + lifecycle
  immutable authoritative CPU query view
  GPU surface/simulation resources
  view LOD/culling/reflection/underwater plan
  fixed-step current/shallow state
                          |
        +-----------------+------------------+
        v                 v                  v
 Render adapter      Physics/Nav adapter   Audio/VFX/Gameplay adapter
        +-----------------+------------------+
                          v
 typed generation-bound receipts and bounded observations
```

硬合同：App只选择provider，不拥有Water truth；Editor只拥有authoring和transaction，不拥有runtime simulation；Runtime拥有per-World accepted generation与query truth。视觉GPU高频细节允许非确定和降级，但buoyancy/gameplay查询不得依赖view residency或同步GPU readback。所有consumer必须绑定同一artifact generation，不能静默跨代。

## 16. 依赖有序重构里程碑

### M0：Truth Freeze 与 RED 证据

- catalog/App/Editor把Water明确标为Unsupported；Navmesh `Water`选项、WOC五湖和两个测试fixture不得显示Ready。
- 建立resource/Scene roundtrip、provider缺失、stale generation、query miss、World replace的RED测试。

### M1：Canonical Source、Artifact 与 Persistence

- 建WaterBody/Material/Wave/Zone schema、validator、compiler key和artifact。
- 完成plugin component clone/prefab/snapshot/save/reopen/play/export无损合同。

### M2：per-World Owner 与 CPU Query Oracle

- 建service lifecycle、stable identity/generation、body/zone spatial index。
- 实现analytic surface/depth/normal/velocity/current/containment query与typed error/provenance。

### M3：Surface Geometry、LOD 与 Render Execution

- 实现Ocean/Lake/River surface artifact、clipmap/quadtree、stitch/morph、culling和GPU resource plan。
- 接入depth/GBuffer/forward/shadow/motion/picking及真实execution receipt。

### M4：Material、Reflection、Refraction 与 Underwater

- 依赖Runtime100关闭统一透明合成、physical transmission和Planar/OIT关键P0。
- 实现Water material resolve、foam、caustics、waterline、underwater medium和shore mask。

### M5：Wave、FFT、Current 与 Shallow Simulation

- 实现analytic CPU/GPU parity、spectral bands/FFT、fixed-step state与quality fallback。
- 接入River flow、local deformation、bounded influence、reset/replay。

### M6：Physics、Navigation 与 Gameplay

- 实现buoyancy、drag/current、submersion events与Physics command receipt。
- 在多运动域Navigation上实现swim layer/entry/exit；提供script/server bounded query。

### M7：Terrain、Weather、Audio 与 VFX Adapter

- 原子消费accepted Terrain/River/Weather generation，完成shore/current/wind/precipitation适配。
- 发布bounded audio/VFX streams、medium state与跨域failure/degrade receipt。

### M8：Partition、Residency、Reliability 与 Scale

- 完成cell/HLOD bundle、multi-view、budget/pressure、cancel/device loss、fault/soak。
- 建quality matrix、telemetry、overflow/degrade和headless/VR/capture差异门。

### M9：Product Qualification 与“优于Unreal”门

- 建Ocean/Lake/River/underwater/buoyancy真实产品scene和自动证据链。
- 固定硬件、画质、view path、reference commit和测量方法；原始receipt不可复算时禁止领先声明。

## 17. G01-G44 综合资格门

| Gate | 状态 | 关闭条件 |
|---|---|---|
| G01 | Fail | Water source对NaN/Inf、负深度、零extent、非法priority/spectrum和超预算全部typed拒绝 |
| G02 | Fail | Body/Material/Wave/Zone save/reopen/cook保持stable ID、单位、kind和依赖 |
| G03 | Fail | 相同source/profile/compiler产生byte-identical artifact/composite key |
| G04 | Fail | body/zone/tile/wave/simulation handles在replace/reuse/unload后拒绝stale访问 |
| G05 | Fail | provider缺失时catalog/Scene/Editor/App统一Unsupported且无假pass/receipt |
| G06 | Fail | lifecycle所有ticket进入明确终态并可cancel/retire |
| G07 | Fail | Ocean/Lake/River/Transition只接受合法geometry/query/collision policy |
| G08 | Fail | overlap按稳定priority/tie-break解析并返回reason/generation |
| G09 | Fail | plugin component clone/prefab/snapshot/save/reopen/play/export无损 |
| G10 | Fail | headless加载query/physics artifact且不创建GPU，结果匹配图形World CPU oracle |
| G11 | Fail | Water feature拥有非空extract/phase/pass/execution receipt |
| G12 | Fail | 各body topology在hole、负坐标、极端scale下winding/UV/bounds正确 |
| G13 | Fail | LOD/tessellation满足误差且相邻tile无裂缝/phase跳变 |
| G14 | Fail | HZB/GPU culling/indirect overflow不丢合法tile并可解释 |
| G15 | Fail | Ocean grid/far mesh/horizon在长移动/origin shift无抖动/缝/精度崩溃 |
| G16 | Fail | depth/GBuffer/forward/shadow/motion/picking共享同一generation和previous state |
| G17 | Fail | mask/excluder/hole在GPU、CPU、Physics、Nav上一致 |
| G18 | Fail | partition attach/evict对surface/simulation/query/adapters原子且可rollback |
| G19 | Fail | origin shift、负grid、多view不改变identity或authoritative query |
| G20 | Fail | Water显式使用1.33或author IOR，provenance可见且不污染PBR默认 |
| G21 | Partial | 已有screen-copy transmission；关闭需Snell/depth/roughness/edge/off-screen与Water集成 |
| G22 | Fail | absorption/scattering按真实或明示fallback光程，浅深水通过数值/golden门 |
| G23 | Partial | SSR/Planar基础存在；关闭需Water resolve、energy、priority、history与fallback |
| G24 | Fail | Water与mesh/sprite/particle/fog/transmission在统一compositor正确分段 |
| G25 | Fail | GPU displacement/normal/velocity/motion与CPU oracle在profile误差内一致 |
| G26 | Fail | foam emit/decay/history/LOD/overflow/reset稳定 |
| G27 | Fail | caustics只投合法receiver且energy/depth/temporal/cost可量化 |
| G28 | Fail | underwater containment/waterline跨波峰/cut/multiview/near plane正确 |
| G29 | Fail | wetness/bottom/bank/influence消费accepted generation并局部失效 |
| G30 | Fail | analytic wave CPU/GPU height/normal/velocity/max bound通过跨平台门 |
| G31 | Fail | spectrum/FFT初始化、band、dispersion、inverse可复现且降级有receipt |
| G32 | Fail | shallow solver满足CFL/质量守恒，压力或非法输入typed degrade/fail |
| G33 | Fail | River current/confluence/deformer跨segment/cell连续且ingress有backpressure |
| G34 | Fail | query覆盖body/surface/depth/normal/velocity/flow/wave/containment和typed error |
| G35 | Fail | batch/async query有persistent acceleration/bounded scratch/p50-p99与allocation |
| G36 | Fail | save/load/replay/network保持相同authoritative query结果 |
| G37 | Partial | Physics force command存在；关闭需稳定buoyancy/drag/current与可复算receipt |
| G38 | Fail | submersion事件fixed-step排序/去抖/bounded且replace不泄漏 |
| G39 | Fail | 缺swim能力typed Unsupported；落地后water/ground domain不误烘焙 |
| G40 | Partial | Audio/VFX/Weather相邻owner存在；关闭需同generation bounded adapter stream |
| G41 | Fail | River/Terrain/Water跨cell更新只失效相交artifact并原子发布 |
| G42 | Fail | quality切换真实改变tile/band/reflection/foam/caustics/resources与GPU cost |
| G43 | Fail | 产品scene通过roundtrip、WGPU capture、headless query/physics、export和soak |
| G44 | Fail | 同口径benchmark记录image error、CPU/GPU、memory、IO、stutter、query/physics并可复算 |

## 18. 禁止的临时修补

1. 禁止用静态蓝色mesh、四层normal滚动、SSR或Planar启用冒充Water runtime。
2. 禁止把WOC硬编码水位/五湖直接搬进Runtime作为正式schema或query。
3. 禁止从tests re-export `ocean.query.v1`，或把trait注册成功当provider可用。
4. 禁止每种Water body继续向`SceneEntityAsset`堆平行`Option<T>`而不建立统一plugin persistence。
5. 禁止Water renderer私建scene-color copy、透明排序、reflection或fog管线绕开Runtime100。
6. 禁止buoyancy直接持有Jolt/native world或每帧同步GPU readback；力只能经中立Physics命令提交。
7. 禁止把水面quad烘成walkable navmesh，或在没有swim domain时返回成功空路径。
8. 禁止视觉资源evict后连带丢失server/gameplay authoritative query truth。
9. 禁止用“Experimental Unreal也不完整”降低Zircon工程基线；参考只决定边界，不替代Zircon验收。
10. 禁止在没有固定画质、硬件、场景、版本和原始receipt时声称性能或表现优于Unreal。

## 19. 本轮完成定义

- 已冻结并核对382个Zircon selected文件、304个参考文件和可复算working-tree指纹。
- 已确认12,207个production Rust文件的Water精确领域词命中为0。
- 已确认Resource/Scene/World/catalog/App/render/physics/navigation没有Water owner。
- 已把Transmission、SSR、Planar、OIT、Physics、Navigation、Sound、Particles与time只判为可复用底座，没有误记为Water实现。
- 已核对Navmesh Workbench固定`Water`选项、WOC五湖/游泳/路径私有逻辑以及两个测试fixture的真实边界。
- 已逐条重判Runtime30的62项P1和14项P2，并把P0归还既有owner。
- 已核对Unreal query/body/mesh/wave/buoyancy分层、Unity HDRP rendering/simulation/search/underwater分层，以及Godot/Bevy/Fyrox的适用上限。
- 本轮只写review、索引和coverage；没有修改production/test/Cargo/workflow，没有运行不能提高Water证据等级的动态lane。
