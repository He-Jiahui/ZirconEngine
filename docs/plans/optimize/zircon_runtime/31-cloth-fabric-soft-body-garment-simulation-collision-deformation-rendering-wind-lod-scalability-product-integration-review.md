---
related_code:
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime/src/asset/assets/mesh
  - zircon_runtime/src/asset/assets/scene
  - zircon_runtime/src/scene/components/scene
  - zircon_runtime/src/core/framework/animation
  - zircon_runtime/src/core/framework/physics
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/advanced_lighting/material_features.rs
  - zircon_runtime/src/core/framework/render/material/lighting_model.rs
  - zircon_runtime/src/graphics/feature/builtin_render_feature
  - zircon_runtime/src/graphics/scene/gpu_scene
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity
  - zircon_runtime/src/graphics/shader/wgsl
  - zircon_plugins/physics/runtime/src
tests:
  - zircon_runtime/src/asset/tests/assets/mesh/morph_targets.rs
  - zircon_runtime/src/core/framework/physics/tests.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache.rs
  - zircon_runtime/src/graphics/tests/render_product_mesh_cache/morph.rs
  - zircon_plugins/physics/runtime/src/backend/tests/jolt_contract.rs
  - zircon_plugins/physics/runtime/src/skeletal/tests.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/08a-physics-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_runtime/09e-direct-lighting-clustered-shadow-review.md
  - docs/plans/optimize/zircon_runtime/09g2-advanced-surface-lighting-review.md
  - docs/plans/optimize/zircon_runtime/09h1-temporal-aa-velocity-history-upscaling-review.md
  - docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
  - docs/plans/optimize/zircon_runtime/23-coordinate-space-unit-precision-transform-numeric-robustness-large-world-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/28-hardware-ray-tracing-blas-tlas-ray-query-pipeline-sbt-denoising-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_editor/32-model-mesh-skeleton-geometry-import-lod-collision-retarget-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/38-weather-climate-time-of-day-wind-precipitation-cloud-atmosphere-environment-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/ChaosClothAsset/Source
  - dev/UnrealEngine/Engine/Source/Runtime/ClothingSystemRuntimeInterface
  - dev/UnrealEngine/Engine/Source/Runtime/ClothingSystemRuntimeCommon
  - dev/godot/scene/3d/physics/soft_body_3d.cpp
  - dev/godot/modules/godot_physics_3d/godot_soft_body_3d.cpp
  - dev/godot/modules/jolt_physics/objects/jolt_soft_body_3d.cpp
  - dev/godot/thirdparty/jolt_physics/Jolt/Physics/SoftBody
  - dev/bevy/crates/bevy_mesh/src/skinning.rs
  - dev/bevy/crates/bevy_mesh/src/morph.rs
  - dev/bevy/crates/bevy_pbr/src/render/morph.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Material/Fabric
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 31 · Cloth、Fabric、Soft Body、Garment、Simulation、Collision、Deformation、Rendering、Wind、LOD、Scalability 与 Product Integration 工程化差距

## 1. 结论

Zircon 当前没有 Cloth/Fabric/Soft Body/Garment 运行时产品。排除 tests、文档和路径示例后，production 对 `Cloth/SoftBody/Garment/Fabric` 的精确标识搜索为零；`ResourceKind`、`SceneEntityAsset`、Scene component、`RenderFrameExtract`、`BuiltinRenderFeature`、Physics capability 和 Jolt bridge 都没有布料资源、实例、软体句柄、模拟输出或渲染 feature。仓内 `cloth` 只出现在 shader import path 测试字符串中，不能构成任何产品能力。

现有蒙皮、Morph、Ragdoll 与通用 PBR 是真实但有限的基础。Mesh 支持四个 joint index/weight、Morph position/normal/tangent/color，GPU Scene 保留当前/上一帧 skin palette 和 morph weights，velocity pass 能计算骨骼与 Morph 运动；Physics 有刚体 shape/body/constraint/query/fixed-step，Ragdoll 也能把骨骼映射到刚体并回写模拟 pose。它们都没有 simulation mesh、seam/fabric/constraint topology、pin/max-distance map、self collision、continuous collision、render-to-sim mapping、vertex output buffer、cloth bounds 或 per-LOD state。普通 GPU mesh buffer只有 `VERTEX`/`INDEX` usage，既不能作为 storage output，也没有 generation-qualified 动态顶点发布合同。

本篇登记 **0 P0 / 64 P1 / 14 P2**。0 P0 不是完成度认可：Runtime08A 已明确登记“车辆、软体、布料、破坏无 owner”，09H1 已登记 cloth previous deformation 输入缺失；当前产品 catalog 又没有宣称 Cloth Ready，因此不重复制造 truthfulness P0。本篇接管独立运行时 owner，把占位结论展开为 `ClothSourceAsset -> ClothBuildArtifact(sim/render/mapping/LOD) -> ClothRuntimeInstance -> Solver Provider -> qualified DeformationOutput -> Render/Physics/Animation/Wind adapters -> typed receipt`。若未来 catalog、Editor 或示例在这条链闭合前把普通 skinned mesh、Morph、double-sided material、Ragdoll 或 Fabric shader 称为 Cloth Ready，应沿全局 capability truth P0 直接阻断。

## 2. 审查边界、方法与 currentness

### 2.1 冻结输入

主冻结语料为 121 个文件、24,663 行、1,069,738 bytes：70 个 Zircon production 文件为 10,207 行、393,979 bytes；9 个 focused test 文件为 3,290 行、121,743 bytes；42 个参考文件为 11,166 行、554,016 bytes。指纹算法为按 forward-slash 相对路径排序，逐文件计算小写 SHA-256，形成 `path|file_sha256` 行，以单个 LF 连接且无末尾 LF，再对 UTF-8 payload 计算 SHA-256；结果为 `ce2fbebacb7814a4b3dac120cad7c7bc5ba965f167ae574304244a42b6337c94`。

另外补读 `gpu_mesh_resource.rs` 与 `gpu_mesh_resource_from_asset.rs` 两个 GPU upload 文件，共 107 行、3,860 bytes，补充指纹为 `d8f902d5881d26539339a10e8de5ad955bdb5cf5978ec957e2dc5adb33cf12e7`。它们用于确认 mesh vertex/index buffer 的 usage、source bounds、wire segments 与不可更新事实；主语料和补充语料合计 123 个显式输入。

冻结基线为 `main@25e09a23178000f2e783ce2143cf70a8b118d404`。主语料中 `advanced_lighting/material_features.rs` 与 `frame_extract.rs` 被 Git 状态标为 modified，但二者 working blob 与 HEAD blob 完全相同，`git diff --numstat` 为空；本篇按读取时 current bytes 冻结，不把行尾/索引状态归因于本轮。实施前仍必须重导 manifest、重算两个指纹并复核所有在途文件。

### 2.2 纵向检查链

本轮逐层检查 resource identity -> source/import schema -> sim/render topology -> constraints/weight maps -> cooked artifact/DDC -> Scene persistence -> runtime lifecycle -> animation pose/teleport -> fixed step/solver -> rigid/self/world collision -> wind/external fields -> current/previous deformation -> bounds/visibility -> depth/GBuffer/forward/shadow/velocity/RT -> Fabric shading -> LOD/streaming -> cache/replay/network -> diagnostics/tests/product evidence。9 个 focused test 文件共有 45 个 `#[test]`/`#[ignore]` 属性，全部覆盖既有 Mesh/Morph/skinning velocity、刚体 Jolt contract、Ragdoll/profile 或 physics DTO；没有 Cloth schema、solver、collision、render pixel 或产品测试。

### 2.3 动态验证限制

本轮是 E3 source-level review，没有运行 Cargo、Jolt soft-body、WGPU 或产品场景。原因不是把动态门省略为完成：仓内没有 Cloth 类型、provider 或 pass，运行普通 Morph/Ragdoll/mesh tests 无法证明不存在的产品；同一工作树已有 Editor 编译被大量既有错误阻断，本篇不重复无关 lane。实现阶段必须从 deterministic CPU oracle 和 source/compiler golden 起步，再进入真实 WGPU/Jolt、设备、fault、soak 与跨引擎同口径验证。

## 3. 当前可保留的真实基础

1. `MeshAsset` 已有 typed attribute map、topology/index validation、skin、Morph target、bounds 与 management summary，可作为 Cloth compiler 的输入之一；布料 source 仍需独立 schema，不能把任意自定义 attribute 名直接当长期合同。
2. 四权重 linear skinning、CPU fallback、shader skinning、Morph 与 current/previous weights/palette 已形成可工作的基础，可用于 sim mesh 初始 pose、render mesh skin blend 和 velocity；它们不是 cloth solver。
3. `RenderFrameExtract` 已分 geometry、animation pose、lighting、sprites、particles 与 visibility，说明新 Cloth extract 应作为 typed sideband/geometry provider接入，而不是塞进 `MeshRenderer::morph_weights`。
4. Physics 已有 stable handle pool、shape/body/constraint、command buffer、query、event、fixed step 和 backend selection；Cloth 应消费 rigid collision snapshot或扩展 soft-body provider contract，不创建第二个互不一致的刚体世界。
5. Ragdoll profile 已有骨骼路径验证、shape/mass/blend、拓扑排序、spawn rollback和模拟 pose feed，可复用 skeleton/collider attachment语义；逐骨刚体不能替代逐顶点/约束模拟。
6. Standard PBR 已有 anisotropy、transmission 和 subsurface 扩展，custom lighting model registry也提供插件入口；Fabric compiler可复用材质/PSO基础，但需要明确 cloth simulation 与 fabric optical model是两个 owner。
7. Runtime22 的 fixed-step、Runtime23 的 space/unit、Runtime24 的 generation/epoch 计划提供了共同治理方向；Cloth 的 solver time、teleport、origin shift、instance/output generation必须接入它们。
8. Editor38 已定义 WindField/Weather generation的authoring owner，未来 Cloth 只消费qualified wind snapshot；不得在 garment component 中复制天气真值。

## 4. 当前代码事实与断路

### 4.1 Source、Scene 与资源身份

1. `ResourceKind` 没有 Cloth、Garment、Fabric、SoftBody、ClothCache 或 ClothProfile。
2. `SceneEntityAsset` 只有 mesh、rigid body、collider、joint、animation、terrain/tilemap等固定字段，没有 cloth component或plugin payload。
3. `MeshRenderer` 只保存 model/mesh/material、Morph weights、LOD、queue、tint与alpha mode，没有 cloth artifact、solver、collision、wind、teleport、simulation blend或cache binding。
4. `MeshAssetUsage` 只有 `main_world/render_world` 两个 bool，无法表达 simulation/readback/storage/RT update/cpu-access/cook role。
5. Mesh 自定义 attribute 可以保存未知名字，但 built-in validation/renderer只认识 position/normal/tangent/UV/color/joint；这不是 typed seam、fabric、weight-map或render mapping schema。
6. 生产 import/cook 没有 garment pattern、2D panel、3D sim mesh、seam、pin/max-distance/backstop、fabric、collision layer或 LOD mapping artifact。

### 4.2 Animation、Physics 与 Solver

1. `AnimationPoseOutput` 只有骨骼 local transforms；没有 cloth root/reference bone、pre/post skin pose snapshot、teleport epoch或simulation reset request。
2. `SkeletalPoseTargets`/`SimulatedPoseFeed` 以 entity -> bone targets工作，Ragdoll按骨骼生成刚体；没有 vertex state或render deformation feed。
3. `PhysicsColliderShape` 只有刚体 primitive/convex/triangle mesh/heightfield/compound；triangle mesh/heightfield还被限制为 static body。
4. `PhysicsBackend` trait 只支持 shape/body/constraint、body commands、step、active rigid states、queries和events；没有 soft body creation/state/force/collision/query。
5. Jolt bridge只 import `JPC_Body*`、rigid shape、PhysicsSystem等接口；没有 Jolt SoftBody settings/body interface/vertex state或contact listener投影。
6. 当前 fixed clock默认 60 Hz、最多 4 substeps，服务整个 physics world；没有 cloth solver iterations、substep policy、adaptive quality、interpolation output或 overload degradation。
7. 没有 stretch/shear/bend/area/tether/volume/seam constraints，没有 compliance/stiffness model或material-to-constraint编译。
8. 没有 self collision、adjacency exclusion、continuous collision、friction/thickness、initial penetration、kinematic collider skinning或 collision budget。
9. 没有 solver provider、CPU/GPU parity、deterministic baseline、task graph、cancel、generation、panic/failure isolation或 LKG output。

### 4.3 Deformation、Renderer 与时域

1. `BuiltinRenderFeature` 没有 Cloth；`RenderFrameExtract` 没有 cloth instances、simulation output、dynamic bounds或previous deformation handle。
2. `GpuMeshResource` 只有 vertex/index buffer、静态 source bounds/wire segments；buffer usage为 `VERTEX`/`INDEX`，没有 `STORAGE`/`COPY_DST`、surface pool、fence或 release callback。
3. CPU skinning会新建完整 primitive，shader skin/morph则从静态source和palette/weight读取；两条路径都没有可供 depth/shadow/RT/visibility共享的 cloth-deformed geometry owner。
4. velocity pass已读 current/previous skin和Morph，但 material/WPO/cloth/wind previous deformation没有统一 ABI；缺输入时不能区分静止、cut、reset和unsupported。
5. source bounds在上传时计算，Cloth 大幅摆动后会被 stale bounds裁剪；没有predicted/conservative/dynamic bounds或GPU reduction/readback策略。
6. 没有 normal/tangent重建、double-sided normal policy、thin surface shadow、TAA reactive/motion、RT BLAS refit、GI/card/SDF更新或 virtual geometry fallback。
7. Standard PBR anisotropy不等于 Fabric BRDF；当前没有 sheen/fuzz/Charlie、cotton-wool与silk分型、fabric energy compensation或 Fabric lighting model。

## 5. 参考实现给出的工程边界

### 5.1 Unreal Chaos Cloth：source、simulation topology、render mapping与runtime proxy必须分层

冻结的 Chaos Cloth Asset 文件明确区分 Sim Pattern 与 Render Pattern。collection包含2D/3D sim positions、sim faces、fabric、seam stitches、bone indices/weights、tethers、render vertices/material，以及 render vertex到sim triangle的barycentric/deformer mapping；这证明高质量 garment不能把render mesh顶点直接当solver particle，也不能靠Morph target保存动态状态。

`ClothComponent`/simulation model/proxy又把 per-LOD model、runtime property collection、collision source、parallel task、suspend/resume、teleport/reset、bounds、cache与skeletal integration分开。可吸收原则是清晰的 source/artifact/runtime边界、sim/render mapping、generation和并行收口；不复制 UObject/Dataflow 类层次，也不把 Unreal 默认参数当 Zircon 真值。

### 5.2 Godot 与 Jolt：更小的 SoftBody 也需要真实 server/mesh/render更新闭环

Godot `SoftBody3D` 仍提供 mesh、mass、pressure、stiffness、damping/drag、pinned points、collision layer/mask、simulation precision和renderer更新；Godot Physics与Jolt模块各有soft-body object桥。Jolt upstream又区分 creation settings、shared settings、motion properties、vertex/contact和update context。即使目标小于 garment系统，也至少需要 typed soft-body identity、backend lifetime、vertex output和render同步；Zircon当前 rigid-only JoltC bridge低于该基线。

### 5.3 Bevy：skinning/Morph是 deformation基础，不是 Cloth capability

Bevy参考树在本次精确搜索中没有 engine-owned Cloth/SoftBody模块，但 skinning/Morph仍建立了component/extract、current/previous buffer、platform uniform/storage fallback和GPU descriptor。这个结果有两层意义：Zircon现有 skin/morph基础可保留；同时不能因一个参考引擎没有 cloth就把普通 deformation称为 cloth。Zircon的目标是工程级完整引擎，Cloth必须有独立资格门。

### 5.4 Unity HDRP Fabric：光学材质与物理模拟是两个不同问题

Unity Graphics参考包提供 Cotton/Wool 与 Silk等 Fabric material、Charlie/fuzz/anisotropy、preintegration/IBL和Shader Graph subtarget，但该包不是 Unity Cloth solver源码。它适合检验 Cloth render output如何进入高质量 Fabric BRDF，不适合证明simulation topology、collision或determinism。Zircon必须分别交付 Cloth solver和Fabric shading；只完成其中一侧都不能宣称 garment产品闭环。

### 5.5 Fyrox：参考树缺失不是降级许可

对当前 Fyrox参考树精确搜索没有 cloth/soft-body生产模块。报告保留这一负证据，不虚构比较对象；它只说明本域主要参考 Unreal/Godot/Jolt，不能把“中型Rust引擎也没有”变成 Zircon 的产品标准。

## 6. 目标架构与唯一 Owner

```text
Garment/Cloth Source
  -> schema migration + import diagnostics
  -> Cloth Compiler
       -> Simulation Mesh/Topology Artifact
       -> Fabric/Constraint/Weight-map Artifact
       -> Render Mesh Mapping Artifact
       -> Collision/LOD/Cache Metadata
  -> generation-qualified ClothRuntimeInstance
  -> admitted Solver Provider (deterministic CPU baseline, optional GPU)
  -> ClothDeformationOutput(current/previous position-normal-tangent-bounds)
  -> Render / Physics / Animation / Wind / Replay adapters
  -> terminal lifecycle and execution receipts
```

| 领域 | 唯一 owner | Cloth31 只消费/提供 |
|---|---|---|
| Resource/artifact/DDC | Runtime04 | Cloth typed source、dependency、artifact与install receipt |
| Scene/ECS/world lifecycle | Runtime05 | Cloth component identity、instance lifecycle与world generation |
| Rigid physics/query/fixed world | Runtime08A + Runtime22 | collider/kinematic snapshot、fixed clock admission；Cloth不私建刚体世界 |
| Skeleton/pose/Morph | Runtime08C | qualified pre/post pose与root/teleport epoch |
| Cloth domain | 新 Runtime Cloth owner | topology/compiler/solver/instance/output/cache与domain diagnostics |
| Renderer/GPU Scene/visibility | Runtime09B | 消费deformation output、dynamic bounds、draw/visibility receipt |
| Material/PSO/Fabric shading | Runtime09C/09G2 | Fabric model与variant；Cloth不私建材质编译器 |
| Temporal/history | Runtime09H1 | current/previous output、reset/cut epoch与reactive policy |
| Ray tracing/GI/shadow | Runtime28/09F/09E | deformed geometry update/fallback与cost receipt |
| Wind/weather source | Editor38对应runtime owner | qualified WindField generation；Cloth只做aerodynamic sampling |
| Model/garment authoring | Editor32后续 Cloth authoring owner | source edits、paint、preview、transaction；runtime compiler/evaluator保持共享 |

Cloth与通用 SoftBody可以共享 solver kernel、constraint primitives和backend admission，但必须拥有不同 source profile与组件语义。Garment需要2D/3D pattern、seam、skin/render mapping；generic SoftBody可能只有tetra/surface topology。禁止用一个巨大 `SoftBodyComponent { json }` 混合全部产品合同。

## 7. P1：Source、Schema、Compiler 与 Artifact

| ID | 差距 | 必须重构 |
|---|---|---|
| CLOTH-P1-001 | 无 Cloth/Garment/SoftBody资源身份 | 定义稳定kind、marker、schema/version、dependency与capability owner |
| CLOTH-P1-002 | 无独立 source asset | 建`ClothSourceAsset`，分离authoring source与derived runtime artifact |
| CLOTH-P1-003 | render mesh被误认为可直接模拟 | 明确2D pattern、3D sim mesh、render mesh三种拓扑及映射 |
| CLOTH-P1-004 | 无 stable panel/pattern/vertex/face/seam ID | 所有可编辑/诊断对象使用持久ID，禁止数组下标长期身份 |
| CLOTH-P1-005 | 无 seam/stitch schema | 定义端点、方向、rest length、weld/stitch policy与验证 |
| CLOTH-P1-006 | 无 Fabric profile | 定义密度、stretch/shear/bend/buckle、damping、friction、thickness与单位 |
| CLOTH-P1-007 | 无 typed weight maps | max distance、backstop、anim drive、self-collision、tether等使用stable map ID与范围 |
| CLOTH-P1-008 | 无 pin/kinematic attachment schema | 定义bone/socket/vertex selection、space、weight、break与missing target policy |
| CLOTH-P1-009 | 无 collision source schema | physics asset/entity/layer/mask、primitive filter、thickness和owner generation类型化 |
| CLOTH-P1-010 | 无 LOD source与映射 | 每LOD独立sim/render topology、transition map、quality和fallback |
| CLOTH-P1-011 | Mesh custom attributes无domain validation | compiler显式导入允许属性，未知/重复/长度错误给stable diagnostic |
| CLOTH-P1-012 | 无 deterministic compiler/DDC artifact | 编译拓扑、constraints、maps、acceleration、mapping和digest，支持migration/LKG |

## 8. P1：Runtime Instance、Solver、Time 与 Determinism

| ID | 差距 | 必须重构 |
|---|---|---|
| CLOTH-P1-013 | 无 Cloth runtime component/instance | 建artifact handle、owner、generation、state、blend、quality与lifecycle |
| CLOTH-P1-014 | 无严格生命周期 | Requested/Preparing/Active/Suspended/Retiring/Failed/Cancelled各有唯一终态receipt |
| CLOTH-P1-015 | 无 solver provider合同 | 定义CPU/GPU能力、limits、determinism、memory、async与fallback admission |
| CLOTH-P1-016 | 无 deterministic CPU oracle | 先交付可复算baseline，用于compiler、GPU differential与headless |
| CLOTH-P1-017 | 无 particle state布局 | current/previous position、velocity、inverse mass、normal及scratch按generation管理 |
| CLOTH-P1-018 | 无 stretch/shear constraints | 选择XPBD/等价模型，定义compliance、iteration、warm start与误差门 |
| CLOTH-P1-019 | 无 bend/buckle/area constraints | 编译邻接和rest state，退化三角形必须拒绝或typed repair |
| CLOTH-P1-020 | 无 tether/long-range attachment | 从kinematic region构建bounded tethers，避免stretch drift且可诊断 |
| CLOTH-P1-021 | 无 seam constraint执行 | seam顺序、重复、断裂、跨LOD映射和stale topology必须确定 |
| CLOTH-P1-022 | 无 substep/iteration policy | 物理fixed tick上建立cloth-specific substep/iteration budget与overload degrade |
| CLOTH-P1-023 | 无 teleport/reset语义 | 区分preserve pose/velocity、reset、origin shift、large jump和cut epoch |
| CLOTH-P1-024 | 无 pause/suspend/visibility policy | freeze、sleep、offscreen tick、resume warmup和network authority显式化 |
| CLOTH-P1-025 | 无 async task ownership | task带world/instance/artifact/tick generation，cancel/stale/panic不发布 |
| CLOTH-P1-026 | 无 bounded memory/scratch | pool按particle/constraint/instance预算，OOM进入typed degrade或failure |
| CLOTH-P1-027 | 无 NaN/energy fail-safe | finite检查、energy/strain阈值、rollback/LKG和disable reason可观察 |
| CLOTH-P1-028 | 无 cache/replay state | 记录artifact、tick、pose、fields、state/output digest，支持seek/reset/differential |

## 9. P1：Collision、Animation、Wind 与 Gameplay Integration

| ID | 差距 | 必须重构 |
|---|---|---|
| CLOTH-P1-029 | 无 kinematic collider extraction | 从qualified skeleton/physics snapshot生成sphere/capsule/convex并保持bone generation |
| CLOTH-P1-030 | 无 world collision | 消费Physics owner的broad phase/query snapshot，禁止每particle全场扫描 |
| CLOTH-P1-031 | 无 self collision | 建spatial acceleration、adjacency exclusion、thickness/friction和pair budget |
| CLOTH-P1-032 | 无 continuous collision | 对高速骨骼/顶点定义sweep/TOI或明确quality fallback，防tunneling |
| CLOTH-P1-033 | 无 initial overlap恢复 | 提供pushout、iteration cap、stuck diagnostic与reset policy |
| CLOTH-P1-034 | 无 collision layer/filter | cloth/world/self/character/accessory按typed channel和owner解析 |
| CLOTH-P1-035 | 无 friction/contact persistence | 定义static/dynamic friction、warm contact、drop/overflow和event边界 |
| CLOTH-P1-036 | 无 animation pose phase合同 | 明确pre-skin pose、post-animation、physics、cloth、render extract顺序 |
| CLOTH-P1-037 | 无 skin-to-sim/anim-drive blend | reference pose、bone skin、max-distance和anim-drive按同一artifact计算 |
| CLOTH-P1-038 | 无 cloth-to-skeleton反馈边界 | 默认单向；需要反馈时走显式force/pose adapter和authority policy |
| CLOTH-P1-039 | 无 wind field输入 | 消费Editor38对应runtime generation，定义space、sampling、stale与fallback |
| CLOTH-P1-040 | 无 aerodynamic模型 | drag/lift/pressure按triangle normal/area/relative velocity计算并有quality tier |
| CLOTH-P1-041 | 无 gameplay impulse/field ingress | typed force/impulse/explosion接口有owner、tick、capacity、drop reason和receipt |
| CLOTH-P1-042 | 无 network authority政策 | 区分server sim、client cosmetic、replicated cache、correction/reset与带宽预算 |

## 10. P1：Deformation Output、Rendering、Fabric 与 Temporal

| ID | 差距 | 必须重构 |
|---|---|---|
| CLOTH-P1-043 | 无 render-to-sim deformer | 用barycentric/skin blend mapping生成render position/normal/tangent，禁止同拓扑假设 |
| CLOTH-P1-044 | 无 qualified deformation output | 发布current/previous buffer、bounds、tick/artifact/output generation和fence |
| CLOTH-P1-045 | GPU mesh不可作为solver output | 建动态surface pool与STORAGE/VERTEX/COPY合同，资源retire由fence/generation控制 |
| CLOTH-P1-046 | 无 CPU/GPU output parity | 相同artifact/input在误差门内，unsupported平台走明确CPU/skin fallback |
| CLOTH-P1-047 | 无 dynamic bounds | CPU conservative/GPU reduction/readback分层，stale bounds不得误裁剪 |
| CLOTH-P1-048 | 无 normal/tangent重建 | area/angle weighted或GPU kernel有degenerate policy、cost与previous一致性 |
| CLOTH-P1-049 | 无 depth/GBuffer/forward统一消费 | 所有pass读取同一output generation，无base mesh与deformed mesh混帧 |
| CLOTH-P1-050 | 无 shadow/velocity时域闭环 | current/previous deformation与reset epoch进入shadow、velocity、TAA/motion blur |
| CLOTH-P1-051 | 无 Fabric lighting model | 建cotton/wool/silk或批准模型、sheen/fuzz/anisotropy、energy与IBL一致性 |
| CLOTH-P1-052 | double-sided/thin surface不完整 | 定义backface normal、thickness、shadow、SSS/transmission与ray any-hit policy |
| CLOTH-P1-053 | 无 RT/GI/SDF更新策略 | BLAS refit/rebuild、card/SDF/lightmap invalidation按quality/cost显式降级 |
| CLOTH-P1-054 | 无 virtual geometry兼容策略 | dynamic cloth与VG分离或专用deformation path，禁止静态cluster假设 |
| CLOTH-P1-055 | 无 material/texture residency联动 | cloth render artifact声明material/pages依赖，pressure/eviction不产生半安装 |
| CLOTH-P1-056 | 无 multi-view/XR/capture合同 | simulation共享边界、view-specific cull/render/history和offline cache采样明确 |

## 11. P1：LOD、Scalability、Diagnostics、Tests 与 Product Qualification

| ID | 差距 | 必须重构 |
|---|---|---|
| CLOTH-P1-057 | 无 simulation/render LOD policy | screen error、distance、importance、visibility、budget与hysteresis共同选择 |
| CLOTH-P1-058 | 无 LOD transition state | position/velocity/constraint/render mapping可迁移，切换无爆跳或能量注入 |
| CLOTH-P1-059 | 无 update-rate/quality scalability | substep、iteration、collision、normal、RT按profile分层且输出degrade reason |
| CLOTH-P1-060 | 无 global budget/admission | 以particle/constraint/pair/bytes/CPU/GPU time限额选择active/frozen/skin fallback |
| CLOTH-P1-061 | 无 telemetry与debug snapshot | 记录active、particles、constraints、iterations、error、contacts、memory、CPU/GPU分位 |
| CLOTH-P1-062 | 无 fault与fuzz矩阵 | malformed topology、NaN、stale callback、OOM、device loss、provider crash和unload覆盖 |
| CLOTH-P1-063 | 无真实产品测试 | 建cape/skirt/flag/layered garment、collision/wind/teleport/LOD像素与数值golden |
| CLOTH-P1-064 | 无跨引擎超越基准 | 同资产/碰撞/风/画质/硬件比较误差、稳定性、CPU/GPU、memory和stutter原始receipt |

## 12. P2：完整性与长期竞争力

| ID | 后续能力 | 前置条件 |
|---|---|---|
| CLOTH-P2-001 | tearing/fracture与dynamic topology | stable topology IDs、constraint lifecycle、render remap和network policy已完成 |
| CLOTH-P2-002 | sewing/dressing runtime assembly | seam compiler、collision、attachment和transaction稳定 |
| CLOTH-P2-003 | multilayer garment/contact ordering | self/world collision、layer/filter、budget和deterministic tie-break稳定 |
| CLOTH-P2-004 | volumetric/tetrahedral soft body | surface cloth kernel与generic soft-body owner边界批准 |
| CLOTH-P2-005 | GPU async compute solver | CPU oracle、resource barriers、fence/output generation和device loss完成 |
| CLOTH-P2-006 | reduced-order/ML deformation | authoritative fallback、训练数据provenance、error bound和platform admission完成 |
| CLOTH-P2-007 | cloth cache compression/streaming | cache schema、seek、error metric、residency和atomic publication完成 |
| CLOTH-P2-008 | deterministic rollback cloth | fixed tick、input/state digest、checkpoint和cross-platform float policy完成 |
| CLOTH-P2-009 | wetness/ice/burning/damage material coupling | Water/Weather/Gameplay adapters与Fabric material authority稳定 |
| CLOTH-P2-010 | accessibility motion reduction | gameplay/cosmetic区分、quality profile和render fallback完成 |
| CLOTH-P2-011 | large-world partitioned cloth | origin/rebase、streaming owner、跨cell attachment与cache continuity完成 |
| CLOTH-P2-012 | plugin constraint/deformer nodes | ABI/version/capability/budget/unload sandbox与compiler extension完成 |
| CLOTH-P2-013 | collaborative garment authoring | stable panel/seam/map IDs、semantic merge和transaction/recovery完成 |
| CLOTH-P2-014 | distributed simulation qualification farm | deterministic fixtures、artifact digest、raw receipt和自动差异定位完成 |

## 13. 分层重构里程碑

### M0 · Truth、Owner 与 Baseline

- 冻结identifier/caller/capability清单，维持 Cloth Unsupported；禁止普通 mesh/Morph/Ragdoll/Fabric false-ready。
- 批准 Cloth/SoftBody owner、术语、单位、性能预算、CPU oracle与参考场景。

### M1 · Source Schema 与 Compiler

- 建 Cloth source、stable IDs、sim/render topology、seam/fabric/maps/collision/LOD schema。
- 完成migration、validation、deterministic compiler、artifact/DDC/LKG与malformed fuzz。

### M2 · Scene、Instance 与 CPU Solver

- 接Scene persistence、runtime component、generation lifecycle、fixed-step admission和deterministic CPU solver。
- 实现particles、stretch/shear/bend/area/tether/seam、pause/teleport/reset与bounded memory。

### M3 · Animation、Collision 与 Wind

- 接qualified skeleton pose、skin/anim drive、kinematic/world/self/continuous collision。
- 接WindField、aerodynamic、gameplay force ingress、filter/budget和stale rejection。

### M4 · Render Mapping 与 Dynamic GPU Geometry

- 实现render-to-sim mapping、current/previous output、dynamic bounds、normal/tangent与surface pool。
- depth/GBuffer/forward/shadow/velocity统一消费同一generation，完成CPU/GPU output parity。

### M5 · Fabric、Temporal、RT/GI 与 Residency

- 接Fabric shading、double-sided/thin-surface、TAA/motion、RT BLAS和GI/SDF显式策略。
- 完成material/texture residency、device loss、fence retirement和unsupported fallback。

### M6 · LOD、Scalability 与 Cache/Replay

- 实现simulation/render LOD、state transition、update-rate、global budget与quality profile。
- 完成cache record/playback、replay/network authority和multi-view/offline capture。

### M7 · Editor/Product Integration

- 为后续Editor Cloth authoring提供shared compiler/runtime preview、paint/selection/gizmo/debug snapshot合同。
- 交付cape/skirt/flag/layered garment真实产品scene、save/reopen/play/export证据链。

### M8 · Reliability 与性能超越门

- 完成fault/fuzz/OOM/device loss/provider crash/unload、长时间soak和规模矩阵。
- 在同硬件、同资产、同画质和同输入下与Unreal/Godot/Jolt基线比较，领先声明可由原始receipt复算。

## 14. 验收门

1. Cloth source对NaN/Inf、退化面、non-manifold topology、非法seam/map/fabric和超预算输入给typed diagnostic。
2. panel/pattern/vertex/face/seam/map/LOD stable ID经save/reopen/reimport/migration不漂移。
3. 相同source/dependency/compiler/settings生成byte-identical artifact与相同digest。
4. sim/render topology与mapping可独立验证，缺面、越界barycentric和orphan seam在compile阶段阻断。
5. provider缺失时Scene/catalog/Editor/App统一报告Unsupported且保留source，不创建空instance。
6. instance lifecycle每个ticket唯一终态，world replace/unload后stale task/output不能发布。
7. CPU oracle在固定输入、tick、平台政策下state/output digest可复算。
8. stretch/shear/bend/area/tether/seam各有analytic/golden误差与收敛门。
9. substep/iteration超预算进入可解释degrade/freeze/skin fallback，不发生spiral或NaN扩散。
10. pause/suspend/resume/teleport/reset/origin shift分别保留或清理pose/velocity，行为由typed epoch解释。
11. kinematic collision跟随正确bone generation，missing/stale collider不使用旧transform。
12. world/self/continuous collision覆盖高速、thin collider、initial overlap、layer/filter与contact overflow。
13. collision broad phase与pair budget有p50/p95/p99、drop reason和无每particle全场扫描证据。
14. animation -> physics -> cloth -> render phase顺序固定，两个local player/world不共享pose或instance状态。
15. WindField相同generation在不同update rate下结果满足误差门，stale field不混入新tick。
16. aerodynamic drag/lift/pressure对triangle winding、area、relative velocity和zero-area输入数值稳定。
17. gameplay force ingress有owner/tick/capacity/drop receipt，world teardown后无残留。
18. render mapping在不同topology、LOD、skin blend与极端形变下position/normal/tangent连续。
19. dynamic output携current/previous、artifact/tick/output generation和fence，consumer拒绝错代。
20. depth/GBuffer/forward/shadow/velocity同帧读取同一deformation generation，无base/deformed混帧。
21. dynamic bounds覆盖真实形变且不过度膨胀超预算，GPU readback延迟有保守fallback。
22. velocity/TAA/motion blur区分连续形变、teleport/reset/cut和unsupported previous state，无ghost爆线。
23. Fabric BRDF在cotton/wool/silk批准参数下满足energy、IBL、anisotropy和double-sided visual golden。
24. 只启用Fabric material不会创建solver；只启用solver而无Fabric model时有明确standard-PBR fallback。
25. RT/GI/shadow对deformed cloth执行批准的refit/rebuild/fallback，unsupported不能静默使用stale geometry。
26. material/texture/output resources作为原子bundle安装/retire，pressure/device loss不产生半安装。
27. LOD切换迁移position/velocity/constraints/mapping，无爆跳、塌陷或额外能量峰值。
28. quality profile真实改变substep、iteration、collision、normal、RT和update rate，并记录成本差异。
29. global budget在1/10/100/1000 instances下稳定选择active/frozen/skin fallback，无无界allocation或队列。
30. hidden/offscreen/dedicated server/XR/capture各有明确simulation/render policy和资源证明。
31. cache录制/seek/loop/reset与live simulation在误差门内，损坏/缺帧/版本不匹配明确失败。
32. replay/network区分authoritative与cosmetic cloth，correction不污染gameplay physics truth。
33. malformed/fuzz、cancel、stale、panic、OOM、provider crash、device loss和plugin unload均不泄漏handle/task/GPU resource。
34. debug snapshot能解释artifact、solver、LOD、constraints、error、contacts、wind、memory和CPU/GPU time，关闭读者时无全量trace。
35. cape、skirt、flag、layered garment场景通过source roundtrip、CPU oracle、WGPU pixel/frame capture和长时间soak。
36. 同口径benchmark同时记录形变误差、穿透、能量、CPU/GPU、memory、stutter与画质；领先声明由原始receipt复算。

## 15. Finding 到里程碑映射

| Finding | 主里程碑 |
|---|---|
| CLOTH-P1-001..012 | M0-M1 |
| CLOTH-P1-013..028 | M2、M6 |
| CLOTH-P1-029..042 | M3 |
| CLOTH-P1-043..056 | M4-M5 |
| CLOTH-P1-057..064 | M6-M8 |
| CLOTH-P2-001..014 | 对应P1与验收门完成后独立立项，不得提前并入MVP |

## 16. 禁止的临时修补

1. 禁止给`MeshRenderer`增加`cloth: bool`或几个stiffness字段后直接逐顶点积分。
2. 禁止把render mesh拓扑直接当sim mesh，或用Morph target数组保存每帧cloth state。
3. 禁止用Ragdoll逐骨刚体、double-sided material、vertex animation或wind WPO冒充Cloth。
4. 禁止让CPU、GPU、Editor preview和cache playback各自实现不同constraint/solver语义。
5. 禁止把任意Mesh custom attribute名字当seam/fabric/weight map长期schema。
6. 禁止在Cloth里私建第二个刚体world、broad phase、clock或weather authority。
7. 禁止每帧重建完整GPU mesh/bounds/wire data或同步GPU readback后再提交。
8. 禁止depth、shadow、velocity、RT分别读取不同代的base/deformed geometry。
9. 禁止无界self-collision pair、scratch、task、debug trace或gameplay force队列。
10. 禁止碰撞失败、NaN、OOM、device loss或provider缺失时静默回到“看起来还在动”的假成功。
11. 禁止把Unity HDRP Fabric shader或Bevy skin/Morph代码当成Cloth solver参考完成度。
12. 禁止在没有同资产、同画质、同硬件和原始receipt时宣称性能或表现超过Unreal。

## 17. 实施前重查清单

1. 重导123个主/补充输入manifest并重算两个指纹。
2. 复核`material_features.rs`与`frame_extract.rs`是否仍仅为blob-equal状态或已有语义改动。
3. 重跑production exact identifier与resource/scene/feature/catalog查询，确认没有新Cloth owner并入。
4. 取得Runtime04/05、08A/08C、09B/09C/09H1、22/23/24、28与Editor32/38 owner确认。
5. 先建立CPU oracle/source compiler测试，再选择Jolt soft body扩展、独立XPBD或GPU provider；不得由依赖便利性倒推架构。
6. 动态lane按Windows优先，先core/compiler/headless，再Jolt/WGPU产品场景、fault/soak和跨引擎benchmark。

## 18. 本轮产出边界

本轮只新增静态review与分层重构计划，没有修改production Runtime、Editor、Interface、Plugin、App代码或tests，没有运行Cargo、Jolt soft body或WGPU。报告不表示Cloth已经可用，也不授权从P2高级能力开工；实现必须从M0 truth/owner与M1 source/compiler开始，以CPU oracle、generation-qualified output和真实产品证据逐层收敛。
