---
related_code:
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime/src/asset/assets/material
  - zircon_runtime/src/asset/assets/project_document
  - zircon_runtime/src/asset/assets/scene/entity.rs
  - zircon_runtime/src/scene/components/scene
  - zircon_runtime/src/scene/world/project_io
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/frame_extract/geometry.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting
  - zircon_runtime/src/core/framework/render/post_process
  - zircon_runtime/src/graphics/feature
  - zircon_runtime/src/graphics/shader
  - zircon_runtime/src/graphics/scene/scene_renderer/advanced_lighting
  - zircon_runtime/src/graphics/scene/scene_renderer/post_process
  - zircon_plugins/physics/runtime/src
  - zircon_plugins/navigation/runtime/src
  - zircon_plugins/first_party_runtime_catalog
  - zircon_plugins/rendering/runtime
  - zircon_plugins/rendering/features/oit/runtime
  - zircon_plugins/rendering/features/planar_reflections/runtime
  - zircon_plugins/rendering/features/post_process/runtime
tests:
  - zircon_runtime/src/core/framework/render/advanced_lighting/extract/tests.rs
  - zircon_runtime/src/core/framework/render/post_process/stack/tests/screen_space_reflection.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/graph_execution/render_pass_execution_context/gpu/post_process/screen_space_reflection/tests.rs
  - zircon_runtime/src/graphics/tests/render_product_planar_reflection.rs
  - zircon_runtime/src/graphics/tests/pipeline_compile/feature_descriptors.rs
  - zircon_runtime/src/asset/tests/assets/gltf_importer/woc_required_extensions.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge/diagnostics.rs
  - zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge/lifecycle.rs
  - zircon_plugins/rendering/features/oit/runtime/src/tests.rs
  - zircon_plugins/rendering/features/planar_reflections/runtime/src/tests.rs
  - zircon_plugins/physics/runtime/src/manager/tests.rs
  - zircon_plugins/navigation/runtime/src/tests/bake.rs
  - zircon_plugins/navigation/runtime/src/tests/crowd.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/08a-physics-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08d-navigation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_runtime/09g2-advanced-surface-lighting-review.md
  - docs/plans/optimize/zircon_runtime/09h2-exposure-color-bloom-dof-motion-blur-ssr-terminal-review.md
  - docs/plans/optimize/zircon_runtime/23-coordinate-space-unit-precision-transform-numeric-robustness-large-world-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/29-terrain-landscape-heightfield-quadtree-lod-material-layer-foliage-world-partition-physics-navigation-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_editor/38-weather-climate-time-of-day-wind-precipitation-cloud-atmosphere-environment-authoring-review.md
  - docs/plans/optimize/zircon_editor/39-spline-path-road-river-decal-brush-geometry-authoring-review.md
  - docs/plans/optimize/zircon_plugins/04-rendering-umbrella-feature-bundles-solari-native-provider-product-integration-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/Experimental/Water
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Water
  - dev/godot/scene/resources/material.cpp
  - dev/godot/servers/rendering/renderer_rd/shaders/effects/screen_space_reflection.glsl
  - dev/bevy/examples/3d/ssr.rs
  - dev/bevy/assets/shaders/water_material.wgsl
  - dev/bevy/crates/bevy_pbr/src/pbr_material.rs
  - dev/Fyrox/fyrox-graphics-gl/src/shaders/shared.glsl
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 30 · Water、Ocean/Lake/River Surface、Wave/FFT、Shallow Water、Rendering、Underwater、Buoyancy、Query、Scalability 与 Product Integration 工程化差距

## 1. 结论

Zircon 当前没有 Water 运行时子系统。生产 Rust 源码对 `water/ocean/river/buoyancy/underwater/caustic/gerstner` 的精确词边界搜索为零；`ResourceKind`、`SceneEntityAsset`、`SceneNode/NodeRecord`、`BuiltinRenderFeature`、第一方 runtime catalog 与 rendering feature catalog 均没有 Water 资源、组件、feature 或 provider。仓内存在的 PBR transmission、SSR、Planar Reflection、OIT、ApplyForce 和 Recast Navigation 都是真实的通用基础，但没有任何 owner 把它们组合为水体 source、runtime instance、surface geometry、wave/query、underwater 或 buoyancy 闭环。

两个看似反例的测试也不能提高结论等级。`ocean.query.v1` 是 plugin extension bridge 测试内声明的虚构 trait/provider，用于验证 slot、generation 和 unload；没有生产接口或注册。`water_elemental.glb` 只验证 glTF 材质能导入 `ior=1.333`、specular transmission、thickness 与 attenuation；它仍是普通 mesh/material，没有 Water Body、位移、流场、查询或浮力。通用材质甚至以 `ior=1.5` 为默认值，screen-space transmission 只按 normal.xy、`(ior-1)`、thickness 和固定 `0.02` 偏移 LOD0 scene color，无法替代深度相交、真实光程吸收、水线或水下介质。

本篇登记 **0 P0 / 62 P1 / 14 P2**。0 P0 不是完成度认可：Editor39 已拥有“没有空间 Spline/River authority、静态 River/蓝色 mesh 不得伪装产品完成”等 5 个 P0，Runtime09G2 已拥有 transmission/transparent compositor/产品证据 P0，Runtime08A 已记录 buoyancy/field 无 owner。本篇避免重复计数，负责把下游运行时目标收敛为 `WaterBodySource -> WaterBuildArtifact -> WaterRuntimeInstance -> Surface/Wave/Query/Physics/Nav/Audio/VFX adapters -> typed receipt`。任何产品若把 WOC 材质测试、SSR/Planar/OIT pass、静态蓝色 mesh、`ApplyForce` 或虚构 `ocean.query.v1` 展示为 Water Ready/Executed，应直接沿既有产品真实性 P0 升级。

## 2. 审查边界、方法与 currentness

### 2.1 冻结输入

本篇冻结 144 个输入、55,079 行、2,187,890 bytes：46 个 Zircon production 输入为 10,300 行、382,137 bytes；14 个 Zircon test 输入为 4,193 行、145,312 bytes；84 个参考实现输入为 40,586 行、1,660,441 bytes。组合指纹按相对路径排序，对每个文件计算 SHA-256，再对 `path<TAB>hash` 的 LF 拼接文本计算 SHA-256，结果为 `c8478ec84d2888516a64863c31699108b1f5e1c6c146e483150105db7ac430d8`。

冻结基线为 `main@25e09a23178000f2e783ce2143cf70a8b118d404`。选定 Zircon 输入中 8 个文件被 Git 标为 modified；其中 `advanced_lighting/extract.rs`、`material_features.rs`、`screen_space_transmission.rs`、`frame_extract.rs`、`frame_extract/geometry.rs`、SSR settings 与 SSR stack test 的 working blob 和 HEAD blob 完全相同，`git diff --numstat` 为空。`asset/assets/project_document/material.rs` 有已有的 98 additions/9 deletions，内容是 material texture reference flatten/roundtrip 修复；本篇按当前 working bytes 冻结并保留该改动，不把它归因于 Water review。

### 2.2 纵向生产链

本轮逐层核对：resource/source schema -> dependency/cooked artifact -> Scene persistence -> ECS/runtime lifecycle -> body/zone/priority -> geometry/LOD/culling -> depth/GBuffer/forward/shadow/motion/picking -> reflection/refraction/transparency -> wave/spectrum/simulation/current -> CPU/GPU query -> underwater/waterline/foam/caustics -> buoyancy/drag/collision -> navigation/audio/VFX/weather/terrain adapters -> partition/HLOD/residency -> quality/diagnostics/product evidence。生产搜索排除了 tests、build scripts 与 WOC 内容名字，没有发现 facade 后隐藏的第二套 Water 执行实现。

### 2.3 证据等级与限制

本轮达到 E3 source-level review。没有运行 Cargo 或 GPU 产品测试：仓库没有 Water source/component/provider/pass，运行通用 transmission、SSR、Planar、OIT 或普通 mesh 测试不能提高 Water 结论强度；同一工作树已有 `zircon_editor --lib` 在 617 秒后被 239 个既有 test-build errors 阻断，本轮不重复该动态 lane。Editor39 拥有 Spline/River authoring、compiler 与跨域 receipt；本篇只定义其 artifact 被 Water runtime 消费后的算法、状态、adapter 与验收合同。

## 3. 当前可保留的工程基础

1. Standard PBR 已有 clearcoat、anisotropy、diffuse/specular transmission、thickness、IOR、attenuation 字段和 forward/transmission queue 资格，可作为 Water material compiler 的底层参数，不应复制第二套 PBR ABI。
2. Transmission 已有 scene-color copy 资源与 draw stage，SSR、Planar Reflection 和 OIT 也各有真实 pipeline/resource/test；Water 应通过统一 reflection/transparent compositor 消费它们，而不是在专用 shader 内绕开 Runtime09G2。
3. RenderFrameExtract、GeometryExtract、GPU Scene/visibility、render graph、material/PSO 与 streaming/residency 已有 owner，Water 需要新增 typed extract/provider，而不是建立不可观测的私有 renderer。
4. Physics command buffer 已有 ApplyForce/ApplyImpulse、generation handle、fixed-step 和 backend-neutral shape/body DTO，适合承载 buoyancy/current/drag 计算结果；Water 不应成为第二个 physics world。
5. Navigation 已有 Recast/Detour、tiled bake、Crowd、dirty rebuild 和 query owner方向，未来 Water 只提供 water-domain geometry/cost/locomotion adapter，不能复制 pathfinder。
6. Plugin bridge 测试已证明 typed interface slot、generation、weak import 与 owner unload 语义可用；未来 `water.query` 可以复用该机制，但 test trait 本身必须删除或迁移为正式公共合同，不能当实现。
7. WOC water elemental 资产可保留为 material migration fixture，用于证明 IOR/attenuation roundtrip；它不应升级成 Water 产品验收样例。
8. Editor39 已定义 River source、flow artifact 与跨域 receipt 边界，Editor38 已定义 WindField/Weather generation；Water runtime 应只消费 accepted generation，避免复制 River/Weather authoring truth。

## 4. 参考实现给出的工程边界

### 4.1 Unreal：Water Body、Mesh/Quadtree、Query、Wave、Buoyancy 与 World 集成是不同 owner

Unreal Experimental Water 插件虽不是完美上限，但已形成真实纵向链。`WaterBodyComponent` 区分 Ocean/Lake/River，提供 typed `TryQueryWaterInfoClosestToWorldLocation`，返回 surface location/normal、velocity、depth 与失败原因；同时暴露 wave info/max wave height、collision/navigation、Water Zone、HLOD、terrain interaction 和 baked shallow-water query资格。`WaterQuadTree`/GPU builder管理 tile、LOD selection、morph、far mesh、bounds与多view draw；Gerstner source/evaluator分离 wavelength、amplitude、steepness、direction、speed和octave；Buoyancy component/manager与 baked shallow-water simulation各有生命周期。

可迁移原则不是复制 UObject 层次，而是：WaterBody source、zone resolve、surface mesh proxy、wave state、query acceleration、physics adapter、underwater volume与HLOD artifact必须拥有不同 identity/generation；同一 accepted generation再向各consumer原子发布。单个透明 mesh无法承担这些职责。

### 4.2 Unity HDRP：Water rendering、FFT simulation、CPU search 与 underwater分层

Unity Graphics 的 HDRP Water 参考树包含 118 个文件；本篇冻结其中 28 个关键输入。`WaterSurface`区分 surface/geometry type、CPU simulation、ripples、tessellation与质量参数；Water System 分离 GPU simulation、CPU simulation/search、underwater和decals；shader侧独立实现 Fourier transform、simulation/evaluation、current、deformation、foam、lighting、water line、underwater、tile classification与vertex tessellation。它证明 Water 不只是一个材质 preset，也证明 visual GPU displacement与gameplay CPU query必须有误差、同步和fallback合同。

Unity Graphics主要拥有 SRP rendering/simulation，不提供完整 gameplay physics/buoyancy owner。本篇只借鉴其 Water 渲染、模拟、查询和scalability分层，不把缺失的 gameplay 层错误补记为已存在。

### 4.3 Godot：真实通用 refraction/SSR，但没有引擎级 Water owner

Godot material会为 refraction申请 screen texture并结合 normal/depth discontinuity，Environment和RD renderer有真实 SSR设置与Hi-Z shader；当前参考树没有通用3D Water Body、Ocean、Wave Query或Buoyancy子系统。因此Godot适合作为通用surface/reflection集成参考，不是Water完成度基线。Zircon同样不能因已有通用transmission/SSR就声称Water存在。

### 4.4 Bevy：Water shader示例证明扩展性，不证明内建系统

Bevy SSR示例用 `ExtendedMaterial`和四层滚动normal map构造水面，并启用通用SSR；PBR material文档列出water IOR约1.33。它展示第三方material如何进入正式render asset/pipeline，但没有Water Body schema、zone、wave query、shallow simulation或buoyancy。Zircon应吸收可插拔路径，不应把示例shader的视觉近似当系统架构。

### 4.5 Fyrox：本轮只找到通用 Fresnel，不虚构 Water 能力

Fyrox冻结输入中的共享GLSL有Fresnel等通用光照原语，生产源码没有与Unreal/HDRP等价的Water子系统。本篇不以参考引擎名单强行制造错误类比；其适用价值仅是Rust renderer/shader组织方式。

## 5. Owner 裁决与非重复边界

| Owner | 本篇拥有 | 本篇不重复拥有 |
|---|---|---|
| Runtime30 | Water source/artifact/runtime contract、body/zone、surface geometry、wave/simulation/query、underwater、physics/nav/audio/VFX adapter、scalability/product evidence | River/Spline authoring、通用RHI/material/SSR/OIT/physics/nav实现 |
| Editor39 | SpatialSpline、River source/compiler、surface/bank/bed/flow/terrain carve artifact与authoring receipt、现有产品真实性P0 | Water renderer、wave solver、query、underwater与buoyancy算法 |
| Editor38 | Weather/Climate/TimeOfDay/Wind source、simulation generation、`WindFieldSnapshot`与surface weather state | Water current/wave truth；只发布输入generation |
| Runtime05 | Scene/ECS/world lifecycle、component attach/detach与plugin component persistence底座 | Water component schema与runtime状态机 |
| Runtime08A | Physics world/body/shape/step/query/force command与backend | submersion、buoyancy、hydrodynamic drag/current模型 |
| Runtime08D | Recast/Detour nav world、tile/query/crowd与locomotion owner | Water/swim layer、water cost和surface-generation adapter |
| Runtime09A-D | GPU lifetime、visibility/GPU Scene、material/PSO、streaming/residency | Water专属resource配方、tile/LOD、simulation与消费规则 |
| Runtime09G2 | OIT/transparent compositor、Planar、Transmission、surface refraction | Water body/material/simulation；Water只声明consumer要求 |
| Runtime09H2 | SSR与post-process/history/terminal | Water reflection policy、waterline与underwater medium |
| Runtime23/24 | 坐标/单位/大世界、stable identity/generation | Water zone/tile/simulation generation组合规则 |
| Runtime29 | Terrain/foliage/partition runtime与terrain consumer adapter | shoreline、水深、carve和Water-side attach逻辑 |
| Plugins04 | rendering umbrella、feature package/profile/capability装配真实性 | Water backend和算法实现 |

Editor39 P1-44..49继续拥有 River metadata、geometry、flow artifact、collision/query adapter输入与跨cell continuity；Runtime30从typed artifact边界开始消费。Runtime30不复制 River spline/compiler，也不让Water renderer回写authoring source。Virtual texture/page pool归Runtime09D，reflection/transparency归Runtime09G2/H2，Water只定义按body/view/generation使用这些底座的资格和receipt。

## 6. P0 裁决与升级条件

本篇没有新增P0。Editor39 P0-02/P0-05已覆盖无空间/River authority和项目静态反馈伪装完成；Runtime09G2 P0-02/P0-08/P0-12已覆盖feature truth、非工程级transmission和证据不足；Runtime08A P2-2记录buoyancy/field无owner。以下条件出现时应回写既有P0 owner或新增跨owner failure handoff，而不是在本篇重复累计：

1. 产品catalog、Editor、Hub或样例把WOC `water_elemental`材质参数、静态蓝色mesh或normal滚动展示为Water runtime Ready。
2. `ocean.query.v1`测试fixture、trait可编译或bridge registration被解释为正式Water query provider。
3. SSR、Planar Reflection、OIT或screen-space transmission启用成功被解释为Ocean/Lake/River闭环。
4. ApplyForce/ApplyImpulse存在被解释为buoyancy、current或hydrodynamic drag已实现。
5. River source/flow artifact在没有Water consumer generation与typed receipt时被展示为已渲染、可查询或可游泳。

## 7. P1：Source、Artifact、Scene、Body/Zone 与 Runtime Lifecycle

| ID | 当前差距 | 重构要求 |
|---|---|---|
| WAT-P1-001 | `ResourceKind`没有Water/WaterMaterial/WaveSpectrum/ShallowSimulation等owner | 选择core kind或可验证plugin kind，建立canonical asset URI、marker、schema version与provider owner |
| WAT-P1-002 | Scene asset、SceneNode/NodeRecord没有Water component或plugin payload | 通过统一plugin component persistence建立typed reference/policy；禁止继续为每个body类型增加平行静态Option |
| WAT-P1-003 | 没有world-scoped Water runtime owner和状态机 | 建`WaterRuntimeService`，覆盖requested/preparing/resident/attached/retiring/failed/cancelled与world replace/unload |
| WAT-P1-004 | first-party catalog和rendering feature catalog没有Water package/capability | 在正式provider存在前明确Unsupported；落地后发布backend/profile/readiness/reason/generation，不以crate存在推断能力 |
| WAT-P1-005 | 没有Ocean/Lake/River/Transition/Custom body语义 | 定义sealed body kind与各自geometry/query/collision资格；共享WaterBody base不能吞并River source或Ocean无限面特性 |
| WAT-P1-006 | 没有WaterBody/Zone/Tile/Wave/Simulation stable ID与generation | 复用Runtime24规则建立world/slot/generation/owner epoch，所有ticket/result拒绝stale apply |
| WAT-P1-007 | 没有source、cooked artifact、live instance分层 | 硬切`WaterBodySource`、`WaterBuildArtifact`、`WaterRuntimeInstance`，authoring数据不得被GPU/physics/nav直接修改 |
| WAT-P1-008 | 没有compiler version、target profile、dependency hash或build receipt | artifact key覆盖source/material/spectrum/terrain/quality/compiler版本并支持可复现cook与精确失效 |
| WAT-P1-009 | 多body重叠、zone覆盖、priority与transition没有唯一resolve规则 | 建空间索引和deterministic resolve，返回chosen body、overlap stack、reason与generation，非法重叠在build阶段诊断 |
| WAT-P1-010 | 没有finite/unit/extent/depth/flow/material依赖校验或migration | versioned validator拒绝NaN/Inf、负深度、零extent、非法spectrum/priority与超预算输入；migration无损且fail-close |

## 8. P1：Geometry、LOD、Culling、Pass 与 Large World

| ID | 当前差距 | 重构要求 |
|---|---|---|
| WAT-P1-011 | `BuiltinRenderFeature`没有Water，rendering umbrella也没有Water feature | 建真实Water render provider/descriptor/pass graph；无provider时不可借Transparent Mesh伪装feature receipt |
| WAT-P1-012 | RenderFrameExtract/GeometryExtract只携带通用mesh/light等，没有Water body/surface输入 | 新增immutable WaterExtract，携带body/tile/generation/material/simulation handles与view decisions，不复制大数组 |
| WAT-P1-013 | 没有body kind到surface topology的正式geometry policy | Ocean、Lake polygon、River strip与custom mesh分别编译稳定artifact，明确winding、UV、bounds、holes和shore edge |
| WAT-P1-014 | 无限Ocean没有camera-relative grid、far mesh、horizon或world extent策略 | 建clipmap/quadtree/far mesh与curvature/sky horizon合同，防止超大静态plane精度和overdraw失控 |
| WAT-P1-015 | River/Lake artifact没有typed runtime consumer | 消费Editor39 accepted surface/flow artifact，验证generation、cell continuation和confluence，不重新解释Spline source |
| WAT-P1-016 | 没有screen-error LOD、tessellation、tile resolution或quality profile | CPU reference与GPU route使用同一误差目标，按view/body/profile选择LOD并记录原因/cost |
| WAT-P1-017 | 没有相邻tile stitch、morph、skirt或wave displacement边界规则 | 定义neighbor constraint与crack-free transition，surface/normal/velocity在LOD和cell边界连续 |
| WAT-P1-018 | 没有frustum/HZB/occlusion、GPU preprocessing或indirect draw | 接入Runtime09B GPU Scene/visibility，发布visible/culled/dropped/overflow计数和确定性fallback |
| WAT-P1-019 | 没有depth/GBuffer/forward/shadow/motion/picking pass资格 | 同一accepted geometry generation服务全部pass，displacement与previous state一致，禁止beauty-only水面 |
| WAT-P1-020 | 没有Water mask、excluder、interior hole、island或receiver规则 | build mask/zone/exclusion artifact，GPU与CPU query共享边界，overlap priority和debug可解释 |
| WAT-P1-021 | 没有Water HLOD、partition cell、streaming attach/evict contract | 生成surface/far mesh/simulation/query bundle，按cell/generation原子attach；partial failure rollback |
| WAT-P1-022 | 没有large-world origin shift、负坐标和多view稳定性 | 所有tile/body/query使用local origin+stable grid key；origin shift不改变identity、phase或可见接缝 |

## 9. P1：Material、Reflection/Refraction、Transparency、Foam、Caustics 与 Underwater

| ID | 当前差距 | 重构要求 |
|---|---|---|
| WAT-P1-023 | Standard PBR字段是真实通用材质，但没有Water material domain/compiler | 定义WaterSurfaceMaterial输入并编译到09C统一ABI，明确opaque/depth/transparent/waterline/underwater permutations |
| WAT-P1-024 | 通用默认IOR为1.5；1.333只在WOC测试材质出现 | Water profile显式提供介质参数/provenance/单位，测试材质不得成为全局默认或Water capability证据 |
| WAT-P1-025 | transmission仅按normal.xy和固定系数偏移scene color LOD0 | 在09G2 physical transmission基础上实现depth-aware intersection、roughness mip、edge validity与off-screen fallback |
| WAT-P1-026 | attenuation使用author thickness而非相机到surface/底部的真实光程 | 建surface/depth/bottom contract，按world-space path length做absorption/scattering并处理未知底深fallback |
| WAT-P1-027 | SSR、Planar、probe/environment没有Water reflection resolve policy | 按view/body/roughness/validity融合provider，定义energy conservation、priority、fallback与receipt，不重复实现reflection owner |
| WAT-P1-028 | SSR缺失、屏幕边缘、遮挡和camera cut时Water无稳定fallback | 使用09H2 validity/history与09F/09G2 fallback，防止黑洞、泄漏、双反射和temporal ghost |
| WAT-P1-029 | 通用transparent compositor本身仍有09G2 P0，Water未定义depth segment顺序 | 与mesh/sprite/particle/fog/transmission统一排序、吸收、散射和OIT资格；Water不得私自重放背景 |
| WAT-P1-030 | 没有位移几何、normal、velocity、motion vector的同源规则 | wave evaluator同时生成position/normal/velocity/previous state，CPU/GPU误差受门禁，禁止只滚normal贴图 |
| WAT-P1-031 | 没有shore/crest/interaction foam source、history、decay与budget | 建typed foam emitters/field/artifact，区分白沫视觉与物理含气量，支持LOD、overflow和deterministic reset |
| WAT-P1-032 | 没有caustics生成、projection、receiver mask或temporal filter | 建surface-to-receiver caustics pass与quality fallback，terrain/mesh接收资格、能量和GPU cost可审计 |
| WAT-P1-033 | 没有underwater volume、camera containment、medium resolve或post-process | 以Water body/zone query决定介质，处理overlap、camera cut、surface crossing、多view和缺depth，不靠碰撞猜测唯一真值 |
| WAT-P1-034 | 没有waterline、top/bottom face、near-plane crossing与半入水相机合同 | 独立waterline/clip方案处理波峰穿越、MSAA/temporal、反射/折射两侧和UI/terminal顺序 |
| WAT-P1-035 | 没有shore wetness、puddle、depth color、terrain bottom与bank blend | 通过Editor38 SurfaceWeatherState和Runtime29 Terrain adapter消费generation，禁止shader读取未版本化全局参数 |
| WAT-P1-036 | 没有current/deformation/foam/water mask decal统一注入点 | 建Water decal/influence provider与tile classification，局部输入有bounds、priority、lifetime、generation和overflow receipt |

## 10. P1：Wave、Spectrum/FFT、Shallow Simulation、Current 与 Query

| ID | 当前差距 | 重构要求 |
|---|---|---|
| WAT-P1-037 | 没有Wave source、spectrum、band、seed、time/phase或quality schema | 定义versioned wave program，区分analytic、spectral、baked、shallow与external provider，参数单位和范围明确 |
| WAT-P1-038 | 没有Gerstner等analytic CPU/GPU reference evaluator | 建单一公式/coordinate contract，CPU query、GPU displacement、normal、velocity与max bound通过跨平台误差门 |
| WAT-P1-039 | 没有FFT/spectral initialization、dispersion、frequency bands或inverse transform | 建可复现spectrum artifact、GPU simulation resource、band budget与CPU降级，不用随机normal贴图替代波高 |
| WAT-P1-040 | 没有simulation clock、fixed step、pause/seek/reset、history或world lifecycle | 消费Runtime22唯一时间域，按body/generation维护state；camera FPS不改变波相位，world unload清理全部资源 |
| WAT-P1-041 | 没有浅水高度/速度场、边界条件、source/sink或baked/live路线 | 建shallow-water solver contract、stable grid、CFL/step budget、terrain/river boundary和typed instability/degrade状态 |
| WAT-P1-042 | 没有current/flow field查询、blend、depth profile或confluence resolve | 消费Editor39 flow artifact并生成runtime field，位置/深度/时间采样有generation、LOD和deterministic tie-break |
| WAT-P1-043 | 没有局部deformer、wake、impulse、rain/boat interaction或反馈队列 | 建bounded influence ingress、tile routing、admission/backpressure与feedback artifact，禁止任意线程直接改GPU texture |
| WAT-P1-044 | 没有正式Water query contract | 定义body/surface/depth/normal/velocity/flow/wave/containment查询、flags、typed miss/error、generation与provenance |
| WAT-P1-045 | 没有query acceleration、batch/async、caller buffer或overflow | 建body/zone BVH和tile field view，支持Any/Closest/AllSorted、batch与bounded scratch，热路径不分配大Vec |
| WAT-P1-046 | 没有CPU/GPU/query determinism、save/load、network或replay边界 | 规定source/seed/time/generation和authoritative query路线；非确定GPU视觉与gameplay truth明确分层并可回放 |

## 11. P1：Physics、Navigation、Audio/VFX、Weather、Terrain 与 Gameplay Integration

| ID | 当前差距 | 重构要求 |
|---|---|---|
| WAT-P1-047 | ColliderShape没有Water volume/submersion语义，Physics world不消费Water generation | 建Water-side broad volume/query adapter；Physics仍拥有body/shape，不把水面做成巨大碰撞平面 |
| WAT-P1-048 | ApplyForce存在但没有buoyancy owner、sample policy或stability model | 建`BuoyancyComponent/Service`，按pontoon/volume采样surface/depth/normal/velocity，输出有budget和force receipt |
| WAT-P1-049 | 没有linear/angular drag、current relative velocity、slamming或planing基础 | 先实现有单位/上限/能量检查的hydrodynamic force model，逐级扩展；所有力只经Physics command buffer提交 |
| WAT-P1-050 | 没有enter/exit/submerge fraction、water body changed或surface crossing事件 | 在fixed step按generation生成去抖、排序、bounded事件；world replace和body overlap不产生幽灵事件 |
| WAT-P1-051 | Navigation只拥有地面navmesh/crowd，没有swim/water locomotion layer | 由Runtime08D定义多运动域后接Water layer/cost/entry/exit links；缺能力时typed Unsupported，不把水面quad烘成可走地面 |
| WAT-P1-052 | 没有shore/wave/current/underwater audio emitter或acoustic adapter | 消费Water query/zone generation产生bounded emitters与listener medium状态，Sound owner负责voice/mix/occlusion |
| WAT-P1-053 | 没有splash/spray/wake/foam/impact VFX事件与GPU interop | 向Particles/VFX发布typed bounded stream和surface sample handle，禁止每粒子同步Water query或复制simulation grid |
| WAT-P1-054 | 没有WindField、precipitation、storm和surface weather消费 | 只消费Editor38 accepted generation，定义wind-to-spectrum、rain impulse、evaporation/wetness适配与fallback原因 |
| WAT-P1-055 | 没有Terrain bottom/shoreline、River artifact与partition bundle代际协调 | 与Runtime29/Editor39通过artifact ID+generation原子attach，terrain/river更新只失效相交Water tiles和consumer outputs |
| WAT-P1-056 | 没有script/gameplay/network公开capability、authority或安全预算 | 暴露read-only/bounded query与受控influence command；server authoritative gameplay不依赖client GPU texture readback |

## 12. P1：Scalability、Observability、Product Evidence 与 Competitive Gate

| ID | 当前差距 | 重构要求 |
|---|---|---|
| WAT-P1-057 | 没有Water quality/scalability profile或platform capability matrix | profile覆盖geometry LOD、simulation bands/resolution、reflection、foam、caustics、underwater与query路线，降级reason-coded |
| WAT-P1-058 | 没有CPU/GPU/memory/IO budget、residency、eviction或pressure policy | 每body/tile/resource记录bytes/time/priority；pressure按确定规则降band/LOD或evict，gameplay query truth不得随视觉evict消失 |
| WAT-P1-059 | 没有多view、reflection capture、VR、camera stack和temporal history合同 | view family共享immutable simulation，独立持有view resources/history；camera cut/resize/device loss按generation失效 |
| WAT-P1-060 | 没有body/tile/wave/query/buoyancy/pass统计、diagnostic或execution receipt | 发布source/artifact/runtime/view generation、visible tiles、query latency、force samples、GPU timings、fallback与drop原因 |
| WAT-P1-061 | 没有Ocean/Lake/River/underwater/buoyancy产品fixture与自动gate | 建小型数值fixture、跨域scene、WGPU像素/帧捕获、headless query/physics、save/reopen/export和soak证据层级 |
| WAT-P1-062 | “表现/性能优于Unreal”没有同口径Water benchmark | 固定硬件、分辨率、场景、视图路径、质量和参考commit，联合比较画质误差、CPU/GPU、内存、stutter与query吞吐 |

## 13. P2：MVP 闭环后的扩展能力

| ID | 扩展项 | 前置条件 |
|---|---|---|
| WAT-P2-001 | 船体planing、wake谱、推进器、舵和多浮体耦合 | P1 buoyancy/current/query稳定且Physics车辆/constraint合同成熟 |
| WAT-P2-002 | 双向刚体/浅水耦合与大物体位移体积 | shallow solver、bounded feedback与能量/稳定性门通过 |
| WAT-P2-003 | 破浪、spray、mist、bubble与白沫物理耦合 | 基础foam/VFX stream和透明介质预算完成 |
| WAT-P2-004 | 河网洪水、蓄泄、潮汐、流量守恒与水位传播 | Editor39 River network与shallow simulation artifact稳定 |
| WAT-P2-005 | 侵蚀、泥沙、河床变化与terrain双向更新 | Runtime29局部artifact invalidation和原子bundle成熟 |
| WAT-P2-006 | 冰冻、融化、冰面碰撞与相变材质 | Weather/temperature authority和Water lifecycle完成 |
| WAT-P2-007 | 高级水下体积光、参与介质、色散与体积caustics | Runtime09G1介质、09G2 transmission与基础underwater闭环 |
| WAT-P2-008 | Path/Ray Tracing Water反射、折射与caustics | Runtime28正式RT管线/SBT/denoise和Water material ABI完成 |
| WAT-P2-009 | 海岸浪破碎、bathymetry感知spectrum与shoaling | terrain bottom query、analytic/spectral parity和shore mask稳定 |
| WAT-P2-010 | 分布式/网络化Ocean phase与大世界simulation streaming | authoritative time/seed、partition和replay门通过 |
| WAT-P2-011 | 水下AI、aquatic navigation volume与群体行为 | Runtime08D多运动域与Water volume query完成 |
| WAT-P2-012 | 水生生态、污染、温度/盐度与gameplay field | 基础Water query/field schema和save/network contract完成 |
| WAT-P2-013 | Procedural shoreline、island fill、harbor与wave obstacle自动生成 | source/compiler/diagnostic和deterministic geometry门通过 |
| WAT-P2-014 | ML辅助spectrum/shore拟合 | 只输出可审查source和deterministic compiler输入，不直接写live simulation |

P2不能替代基础真实度。真实Water source、surface render、wave/query、underwater、buoyancy、跨域generation和产品证据全部属于P1工程基线，不得以“高级效果”名义延期。

## 14. 目标架构与数据流

```text
WaterBodySource (Ocean/Lake/River/Custom)
  + WaterMaterialSource / WaveProgram / Zone policy
  + accepted RiverFlowArtifact / TerrainBottomArtifact / Weather generation
  -> WaterCompiler
       validate + resolve body/zone/overlap
       surface/far-mesh/mask/shore artifact
       wave/spectrum/shallow/query artifact
       partition/HLOD/consumer dependency bundle
       deterministic build receipt
  -> WaterBuildArtifact { id, generation, profile, hashes, costs }
  -> WaterRuntimeService (world owner)
       body registry + spatial index + lifecycle
       immutable CPU query view
       GPU surface/simulation resources
       view LOD/culling/reflection/underwater plan
       fixed-step current/shallow state
  -> Render / Physics / Navigation / Audio / VFX / Gameplay adapters
  -> typed frame/build/query/force/execution receipts
```

关键裁决是 gameplay query truth不得依赖GPU readback或view residency。Analytic/baked/CPU shallow representation可作为authoritative query view；GPU spectrum可提供视觉高频细节，但必须声明是否影响buoyancy/gameplay。所有consumer绑定同一Water artifact generation，视觉可降级但不能静默跨代。

## 15. 分层实施路线

### M0 · Truth Cutoff 与 Owner RFC

- 在catalog/profile/product UI明确Water Unsupported，列出WOC材质与`ocean.query.v1`仅为fixture。
- 批准Water、River、Weather、Terrain、Render、Physics、Navigation owner边界和公共命名。

### M1 · Source、Artifact、Scene 与 Migration

- 建WaterBody/Material/Wave/Zone schema、validator、compiler key和typed artifact。
- 完成plugin component Scene roundtrip、旧静态mesh/material迁移fixture与fail-close诊断。

### M2 · Runtime Lifecycle 与 CPU Query Reference

- 建world-owned service、stable ID/generation、body/zone空间索引和immutable query view。
- 实现analytic surface/depth/normal/velocity/current CPU oracle与headless路线。

### M3 · Surface Geometry、LOD 与 Render Pass

- 完成Ocean/Lake/River surface artifact、clipmap/quadtree、stitch/morph、culling和GPU resources。
- 接入depth/GBuffer/forward/shadow/motion/picking与真实execution receipt。

### M4 · Water Material、Reflection、Refraction 与 Underwater

- 在09G2/H2 owner上完成depth-aware refraction、reflection resolve、transparent ordering和absorption。
- 完成foam、caustics、waterline、underwater medium与shore mask基础闭环。

### M5 · Wave、FFT、Current 与 Shallow Simulation

- 建analytic CPU/GPU parity、spectral bands/FFT、fixed-step simulation和quality fallback。
- 接入River flow、local deformation、bounded influence与deterministic reset/replay。

### M6 · Physics、Navigation 与 Gameplay Query

- 完成buoyancy、drag/current、submersion events和Physics command integration。
- 定义swim/water nav layer、script/server bounded query与authority边界。

### M7 · Terrain、River、Weather、Audio 与 VFX Adapters

- 原子消费Editor39/38和Runtime29 generation，完成shore/current/wind/precipitation适配。
- 发布bounded audio/VFX streams和跨域failure/degrade receipt。

### M8 · Partition、Residency、Scalability 与 Reliability

- 完成cell/HLOD bundle、multi-view、budget/pressure、device loss、cancel和soak。
- 建quality matrix、telemetry、overflow/degrade与headless/VR/capture差异门。

### M9 · Product Qualification 与性能超越门

- 建Ocean/Lake/River/underwater/buoyancy真实产品scene和自动证据链。
- 同硬件、同画质、同视图路径与固定Unreal/HDRP参考比较；原始receipt不可复算时禁止领先声明。

## 16. 验收门

1. Water source对NaN/Inf、负深度、零extent、非法priority/spectrum和超预算输入全部typed拒绝。
2. WaterBody/Material/Wave/Zone schema save/reopen/cook保持stable ID、单位、body kind和依赖，无静默字段丢失。
3. source/profile/compiler version相同产生byte-identical artifact和相同composite key。
4. body/zone/tile/wave/simulation handles在world replace、slot reuse和unload后拒绝stale访问。
5. provider缺失时catalog、Scene load、Editor和App统一报告Unsupported及reason，不创建空pass或假receipt。
6. runtime lifecycle覆盖requested/preparing/resident/attached/retiring/failed/cancelled且每个ticket有终态。
7. Ocean/Lake/River/Transition各自只接受合法geometry/query/collision policy，非法组合在build阶段失败。
8. 多Water Body/Zone overlap按稳定priority/tie-break解析，query返回选择原因和generation。
9. plugin component Scene clone/prefab/snapshot/save/reopen/play/export均保留Water引用与policy。
10. headless world可加载Water query/physics artifact，不创建GPU resource且结果与图形world CPU oracle一致。
11. Water render feature拥有非空extract/phase/pass/execution receipt，不复用Transparent Mesh名字伪装完成。
12. Ocean/Lake/River/custom surface在holes、负坐标、非均匀extent和极端scale下winding/UV/bounds正确。
13. screen-error LOD/tessellation满足误差预算，相邻tile在任意LOD组合无裂缝和phase跳变。
14. frustum/HZB/GPU culling和indirect overflow不丢合法tile，visible/culled/fallback计数可解释。
15. Ocean camera-relative grid、far mesh和horizon在长距离移动/origin shift时无抖动、缝或精度崩溃。
16. depth/GBuffer/forward/shadow/motion/picking共享同一displacement generation和previous state。
17. Water mask/excluder/island/interior hole在GPU surface、CPU containment和physics/nav adapter上一致。
18. partition/HLOD attach/evict对surface/simulation/query/consumer bundle原子，partial failure完整rollback。
19. origin shift、负world grid和多view不改变body/tile stable identity或query结果。
20. Water material显式使用约1.33 IOR或author value，默认provenance可见且不污染通用PBR默认。
21. refraction使用depth-aware intersection/validity/roughness，屏幕边缘和缺depth走typed fallback。
22. absorption/scattering使用实际或明示fallback光程，浅/深水颜色变化通过数值和golden门。
23. SSR/Planar/probe/environment融合满足能量与priority合同，invalid/history reset无黑洞、双反射或ghost。
24. Water与mesh/sprite/particle/fog/transmission在统一transparent compositor中顺序正确，每深度段介质只应用一次。
25. GPU displacement、normal、velocity、motion与CPU query oracle在各quality profile的约定误差内一致。
26. shore/crest/interaction foam具有稳定emit/decay/history、LOD、overflow与reset行为。
27. caustics只作用于合法receiver，能量、深度、temporal和quality fallback可量化。
28. underwater containment、水线和半入水相机跨波峰、camera cut、多view、near plane时无错误介质切换。
29. shore wetness、terrain bottom、bank blend和Water decal/influence使用accepted generation且局部失效。
30. analytic wave CPU/GPU对height/normal/velocity/max bound通过跨平台误差门。
31. spectral/FFT initialization、band组合、dispersion和inverse transform可复现，降band有明确质量/cost receipt。
32. shallow solver满足稳定性/质量守恒预算，非法CFL或资源压力进入typed degrade/failure而非NaN扩散。
33. River current、confluence、deformer/wake/impulse在segment/cell边界连续且ingress有backpressure。
34. Water query覆盖body/surface/depth/normal/velocity/flow/wave/containment、typed miss/error与provenance。
35. batch/async query使用persistent acceleration和bounded scratch，记录p50/p95/p99、overflow及分配量。
36. save/load/replay/network在相同source/seed/time/generation产生相同authoritative query结果。
37. buoyancy在静水、波浪、斜面、多pontoon、部分浸没和body切换下稳定，force/torque receipt可复算。
38. drag/current/submersion enter/exit事件在fixed step排序、去抖、bounded，world replace不泄漏旧事件。
39. Water nav能力缺失时明确Unsupported；落地后swim layer/entry/exit link与地面nav不互相误烘焙。
40. Audio/VFX/Weather adapters消费同一Water/Wind generation，stream有容量、drop原因和lifetime。
41. River/Terrain/Water跨cell更新只失效相交artifact，surface height、flow、shore和consumer generation原子一致。
42. quality/profile切换真实改变tile、band、reflection、foam、caustics、underwater资源与GPU cost receipt。
43. Ocean/Lake/River/underwater/buoyancy产品scene通过roundtrip、WGPU pixel/frame capture、headless query/physics和soak证据链。
44. 同口径benchmark同时记录image error、CPU/GPU、memory、IO、stutter、query和physics吞吐；领先声明可由原始receipt复算。

## 17. Finding 到里程碑映射

| Finding | 主里程碑 |
|---|---|
| WAT-P1-001..010 | M0-M2 |
| WAT-P1-011..022 | M3、M8 |
| WAT-P1-023..036 | M4 |
| WAT-P1-037..046 | M2、M5 |
| WAT-P1-047..056 | M6-M7 |
| WAT-P1-057..062 | M8-M9 |
| WAT-P2-001..014 | 对应P1 owner与验收门完成后独立立项，不得替代P1执行 |

## 18. 本轮验证与未执行项

- 已验证144个冻结输入全部存在且唯一，分组行数/bytes与组合SHA可复算。
- 已验证排除tests/build scripts后的Zircon production精确Water领域词搜索为零；ResourceKind、Scene、BuiltinRenderFeature、catalog均无Water owner。
- 已核对`ocean.query.v1`只存在于plugin bridge tests，`water_elemental`只验证普通glTF/PBR volume material参数。
- 已核对screen-space transmission固定UV偏移/LOD0采样/author thickness attenuation边界，以及SSR、Planar、OIT是真实但通用的独立owner。
- 已核对Physics只有通用shape/body/force命令，没有Water volume/submersion/buoyancy/current；Navigation只有ground navmesh/crowd，没有swim/water layer。
- 已核对Unreal Water与Unity HDRP Water关键source/query/render/simulation分层；没有把Bevy示例、Godot/Fyrox通用shader误记为Water系统。
- 本轮未修改production/test/Cargo/workflow，未运行无法增加Water执行证据的Cargo/GPU lane；实施必须从M0 capability truth和owner RFC开始。
