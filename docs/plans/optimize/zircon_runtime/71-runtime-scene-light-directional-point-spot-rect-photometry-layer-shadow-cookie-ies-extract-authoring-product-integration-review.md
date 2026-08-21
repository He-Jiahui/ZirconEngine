---
title: Runtime Scene Light、Directional/Point/Spot/Rect、Photometry、Layer、Shadow、Cookie、IES、Extract、Authoring 与 Product Integration 工程化差距
category: zircon_runtime
report_id: Runtime71
review_date: 2026-08-20
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime/src/asset/assets/scene/defaults.rs
  - zircon_runtime/src/asset/assets/scene/entity.rs
  - zircon_runtime/src/asset/assets/scene/lighting.rs
  - zircon_runtime/src/asset/artifact/cache_payload/scene.rs
  - zircon_runtime/src/scene/components/scene
  - zircon_runtime/src/scene/world/bootstrap.rs
  - zircon_runtime/src/scene/world/records.rs
  - zircon_runtime/src/scene/world/project_io
  - zircon_runtime/src/scene/world/property_access
  - zircon_runtime/src/scene/world/compiled_binding
  - zircon_runtime/src/scene/reflect/builtin_reflection/registration.rs
  - zircon_runtime/src/scene/inspection/snapshot.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/render/lights.rs
  - zircon_runtime/src/core/framework/render/light
  - zircon_runtime/src/core/framework/render/advanced_lighting/extract.rs
  - zircon_runtime/src/core/framework/render/frame_extract.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_editor/src/core/commands/defaults.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/build.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_editor/src/ui/workbench/event/node_kind_id.rs
  - zircon_editor/src/ui/workbench/event/node_kind_from_id.rs
tests:
  - zircon_runtime/src/asset/tests/assets/scene/lights.rs
  - zircon_runtime/src/scene/tests/asset_scene/product_fields.rs
  - zircon_runtime/src/scene/tests/ecs_reflect/foundation/fixed_lights_name.rs
  - zircon_runtime/src/scene/tests/ecs_schedule/render_extract.rs
  - zircon_runtime/src/scene/tests/ecs_typed_api/persistent_lighting.rs
  - zircon_runtime/src/scene/tests/render_extract/direct_sections.rs
  - zircon_runtime/src/scene/tests/render_extract/lighting_postprocess.rs
  - zircon_runtime/src/scene/tests/world_basics/render_extract.rs
  - zircon_runtime/src/scene/world/compiled_binding/tests.rs
  - zircon_editor/src/tests/editing/editor_projection.rs
  - zircon_editor/src/tests/editing/viewport.rs
  - zircon_runtime/src/graphics/tests/project_render/project_scenes/pbr_matrix.rs
  - zircon_runtime/tests/runtime_shader_pbr_hdri_export/scene_fixtures.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09c-material-shader-pipeline-pso-review.md
  - docs/plans/optimize/zircon_runtime/09d-render-asset-streaming-residency-review.md
  - docs/plans/optimize/zircon_runtime/09e-direct-lighting-clustered-shadow-review.md
  - docs/plans/optimize/zircon_runtime/09f1-environment-sky-ibl-reflection-probe-review.md
  - docs/plans/optimize/zircon_runtime/09f2-baked-lighting-lightmap-irradiance-volume-review.md
  - docs/plans/optimize/zircon_runtime/09f3-hybrid-global-illumination-review.md
  - docs/plans/optimize/zircon_runtime/09g1-volumetric-fog-froxel-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/61-runtime-scene-world-level-registry-lifecycle-project-io-snapshot-clone-serialization-schema-transaction-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/62-runtime-scene-hierarchy-transform-propagation-reparent-activation-mobility-visibility-bounds-render-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/63-runtime-scene-reflection-type-schema-registry-dynamic-component-property-address-inspection-artifact-subscription-editor-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/64-runtime-resource-authority-asset-handle-load-request-state-machine-version-lease-cache-dependency-reload-cancellation-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/65-runtime-scalability-quality-profile-device-profile-capability-tier-dynamic-resolution-frame-budget-lod-feature-fallback-product-integration-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/22-render-pipeline-frame-capture-lighting-bake-reflection-probe-post-process-debug-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/LightComponentBase.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/LightComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/DirectionalLightComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/PointLightComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/SpotLightComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/RectLightComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Components/LightComponent.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Components/PointLightComponent.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Components/RectLightComponent.cpp
  - dev/bevy/crates/bevy_light/src/ambient_light.rs
  - dev/bevy/crates/bevy_light/src/directional_light.rs
  - dev/bevy/crates/bevy_light/src/point_light.rs
  - dev/bevy/crates/bevy_light/src/spot_light.rs
  - dev/bevy/crates/bevy_light/src/rect_light.rs
  - dev/bevy/crates/bevy_light/src/gizmos.rs
  - dev/bevy/crates/bevy_light/src/cluster/test.rs
  - dev/Fyrox/fyrox-impl/src/scene/light/mod.rs
  - dev/Fyrox/fyrox-impl/src/scene/light/directional.rs
  - dev/Fyrox/fyrox-impl/src/scene/light/point.rs
  - dev/Fyrox/fyrox-impl/src/scene/light/spot.rs
  - dev/Fyrox/editor/src/light.rs
  - dev/Fyrox/fyrox-impl/src/utils/lightmap.rs
  - dev/godot/scene/3d/light_3d.h
  - dev/godot/scene/3d/light_3d.cpp
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Utilities/LightUnitUtils.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Tests/Runtime/LightUnitTests.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/Light/HDAdditionalLightData.Types.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/Light/HDAdditionalLightData.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Lighting/Light/HDAdditionalLightData.Migration.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Tests/Editor/HDAdditionalLightDataTests.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Editor/Lighting/HDLightUI.Handles.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 71 · Runtime Scene Light、Photometry、Extract、Authoring 与 Product Integration 工程化差距

## 1. 结论

Zircon的Scene Light并非空壳。五类light component与asset字段已经能经typed ECS、clone、serde、project I/O、reflection和RenderFrameExtract流动；render framework有稳定light ID、layer、mobility与`LightShadowSettings` DTO；Editor有五类创建命令、Inspector反射投影和方向光gizmo；graphics已有GPU light ABI、cluster grid与shadow算法底座。这些能力应被保留并收敛，不能再造第二个World、第二套property系统或第二套renderer light ABI。

但当前数据模型仍是为了让画面“出现一盏灯”的早期实现，而不是工程级光源产品。颜色、无单位`intensity`、范围、方向和少数形状字段之间没有物理光度、色温、衰减、源几何、参与策略、shadow、cookie、IES、版本迁移或预算合同。方向光与聚光保存独立`direction`并忽略节点/父节点旋转，矩形光却使用world transform forward；同一旋转操作对不同光种产生不同结果。extract又按view重复全量扫描、分配、排序五个component store，把volumetric拆成ID sideband，并把四类shadow全部硬编码为`None`。

本轮确认2项新增P0。第一，`SceneSpotLightAsset`允许缺字段时静默采用`+Y`、1,000,000强度、20范围和0/0锥角，而runtime `SpotLight::default()`是`-Y`、8、12和0.3/0.55；合法的部分资产会被接纳为方向相反、能量相差125,000倍且退化为零锥角的光，没有迁移或诊断。第二，`SceneEntityAsset`和`NodeRecord`允许五种light component同时存在，load只按Ambient -> Directional -> Point -> Rect -> Spot优先级选择一个`NodeKind`，但仍保留并渲染全部component；Editor以单一kind呈现节点，用户可得到不可见的额外发光源。两项都是本篇Scene Light产品对象独有的静默错误，不重复Runtime61的通用持久化P0。

此外登记48项P1、12项P2与48项资格门。Runtime09E继续唯一拥有shadow authoring断链、light layer shader失效、物理光度renderer消费、RectLight非真实面积光、cluster/shadow/readiness等既有P0；本篇只定义Scene Light source/schema/Editor/extract必须提供的适配合同，绝不重复累计。完成同硬件、同场景、同采样、同画质的正确性、CPU/GPU、RAM/VRAM、fault与soak证据前，不能宣称性能或表现达到或超过当前Unreal。

## 2. 审查边界与物理冻结

### 2.1 Owner边界

| 领域 | Canonical owner | Runtime71责任 | 不得重复登记 |
|---|---|---|---|
| Direct light、cluster、shadow renderer | Runtime09E | 提供守恒、可验证的Scene Light source与extract输入 | cluster算法、light WGSL、shadow atlas/cache/contact shadow父P0 |
| Material、texture residency、GI/fog | Runtime09C/09D/09F1-09F3/09G1、Runtime64 | 提供cookie/IES/参与策略的typed dependency与generation | shader/PSO、GPU residency、IBL/bake/GI/froxel实现 |
| Scene schema、hierarchy、reflection | Runtime61/62/63 | 定义light-specific不变量、迁移、transform authority与field conservation | 通用save事务、hierarchy传播、reflection catalog/property grammar父问题 |
| Identity、budget、readiness | Runtime24/65 | 采用stable light identity、generation、budget input与typed outcome | 通用handle耗尽、device/quality/scalability父问题 |
| Editor authoring | Editor03/05/22 | Scene Light create/inspect/gizmo/preview adapter及save/reopen资格 | 通用selection/undo/picking、Inspector框架、Lighting Bake工具父问题 |

`zircon_runtime::scene`拥有持久Scene Light实例、组合不变量和world-space effective source；`zircon_runtime::core::framework::render`只发布中立light delta/descriptor；`zircon_runtime::graphics`拥有packed generation、cluster、shadow与GPU资源；`zircon_editor`拥有authoring document、控件、gizmo与preview。不得让graphics DTO反向成为资产schema，不得在Editor保存renderer私有slot，也不得用另一个root package或compat shim掩盖现有边界。

### 2.2 Zircon物理冻结

本轮聚焦53个Zircon文件，共17,796行、659,163 bytes。按相对路径小写、排序去重，以`path|lowercase SHA-256`逐行LF连接且末尾无LF计算，指纹为`70b95e0afedbb10f285f3f37ec7a096b2680766b48543575f0971b7758b056b9`。入选范围含80个Rust test attribute；冻结时9个入选路径dirty，结论绑定当前共享working copy。

| 范围 | 文件 / 行 / bytes | 本轮证据 |
|---|---:|---|
| Scene component、asset、store、project I/O、property | 27 / 9,606 / 370,668 | 默认值、组合不变量、save/reopen、reflection/compiled path逐字段核对 |
| Framework DTO、readiness、world extract | 8 / 2,901 / 102,436 | identity/layer/mobility/shadow、per-view扫描、sideband与降级语义 |
| Editor、tests、product fixture | 18 / 5,289 / 186,059 | create/inspect/gizmo、显式roundtrip、render fixture与缺失的异常/规模资格 |
| 去重合计 | **53 / 17,796 / 659,163** | fingerprint如上；80个test attribute；9个路径dirty |

本轮不修改production/tests，不运行Cargo、Editor、GPU capture、pixel、fault、soak或benchmark。Runtime09E已经深入审查graphics direct-light/shadow物理范围，本篇不重扫其13,250行renderer实现来制造第二owner；实施前必须同时重算两篇指纹并复核共享ABI。

### 2.3 参考物理冻结

五类参考共31个文件、14,862行、593,772 bytes，指纹为`9718752e66737ede52fe7ae327ea99d7083d588806a6ca9066852fcee3126acb`。参考用于确定工程合同和资格下限，不要求Zircon复制对象模型、宏、RenderingServer或SRP API。

| 参考 | 可采用证据 | 对Zircon的直接约束 |
|---|---|---|
| Unreal | common LightComponent有温度、IES、lighting channels、indirect/volumetric、static/dynamic shadow、draw distance；point/spot/rect有source radius/length、cone clamp、width/height与barn door；单位覆盖candela/nits/lumen/EV | Scene source必须有common descriptor、type-specific shape、单位/迁移/validation与参与策略，不能继续靠裸float和百万默认值 |
| Bevy | directional以entity transform forward为权威并使用lux，punctual使用lumens；shadow/bias/near-z/lightmap participation在component；point/spot/rect/directional都有transform-aware gizmo | transform authority、单位、shadow source和Editor形状反馈必须同一数据链；无shadow时不生成对应frustum/work |
| Fyrox | `BaseLight`统一color/intensity/scatter，type node拥有bounds、transform、shadow bias、spot cookie与setter validation；lightmap记录参与light避免double lighting | common/type-specific字段、setter validation、world bounds、bake participation与cookie dependency必须成套存在 |
| Godot | Light3D有lumens/lux、Kelvin、indirect/volumetric/specular、projector、bake mode、cull/caster mask、shadow参数、distance fade及typed Editor hints | schema metadata必须带单位、范围、条件与真实validator；参与、fade、shadow、projector不可只存在renderer内部 |
| Unity Graphics | LightUnitUtils按light type转换lumen/candela/lux/nits/EV并有数值测试；HDRP用versioned migration迁移shape/unit/spot/radius/shadow，提供IES/cookie/barn door/update mode与完整handles | unit conversion必须是测试过的共享服务；schema演进必须有版本和迁移receipt；Editor要有range/cone/area交互handles |

## 3. 可保留的真实底座

1. 五类component、asset和固定ECS访问已经贯通clone/serde/record/query，可作为hard cutover的迁移输入。
2. `RenderDirectional/Point/Spot/RectLightSnapshot`已有stable entity/light ID、layer与部分mobility，`LightShadowSettings`已有bias、strength、resolution和PCF类型，可收敛为中立descriptor，而不是删除重做。
3. World active、render layer、world transform与component index足以建立change-driven light registry；现有全量collector应被替换，不必再发明一套Scene graph。
4. derived reflection能自动看见`volumetric`，Editor Inspector已消费reflection artifact；需要统一generic/compiled access与metadata，不应退回手写每字段UI。
5. Editor create command、node kind映射、overlay DTO和方向光gizmo是接入点；应扩展成完整light authoring adapter，不应在graphics renderer里实现Editor状态。

## 4. 两项新增P0

### RSL-P0-001：Spot资产默认值与runtime默认值相反且退化，合法部分资产被静默误读

`SceneSpotLightAsset`为缺失字段使用`default_vec3_up`、`default_rect_light_intensity`、`default_rect_light_range`和两个serde零值；其`Default`同样得到`+Y / 1,000,000 / 20 / 0 / 0`。`SpotLight::default()`却是`-Y / 8 / 12 / 0.3 / 0.55`。project load不校验、不迁移，直接复制这些值；因此同一个“默认聚光灯”根据创建入口或资产是否省略字段产生完全不同且零锥角的结果。

必须为Scene Light schema建立显式版本、canonical defaults与逐版本migration。reader先解析版本和presence，再迁移到validated descriptor；方向必须finite/non-zero，范围与强度有限且非负，满足`0 <= inner <= outer < pi/2`。旧文档要么确定性迁移并给receipt，要么以typed error拒绝；不得继续把缺字段解释成另一个光种的百万强度默认值。必须加入empty/partial/legacy/malformed asset -> World -> save/reopen测试。

### RSL-P0-002：单一NodeKind隐藏多个同时生效的light component

`SceneEntityAsset`、`SceneNode`和`NodeRecord`各自保存五个独立`Option<Light>`，typed API测试还明确把五类灯全部附着到一个entity。load选择`NodeKind`时使用Ambient、Directional、Point、Rect、Spot优先级，却继续复制全部component；render collector按component store独立扫描，所以一个在Outliner中显示为Ambient Light的节点仍可同时产生方向光、点光、矩形光和聚光。Editor只给Point/Rect/Spot零gizmo，额外发光源更不可见。

必须选择唯一且可验证的composition contract：推荐每个light entity只有一个`SceneLightComponent`，内部由common descriptor加受约束的type-specific shape组成；若保留ECS多component能力，也必须以mutually-exclusive component set做transactional admission。load、add/remove/replace、clone、undo、save/reopen与reflection写入都在preflight拒绝或原子转换非法组合，并返回旧/新kind与affected component receipt。不得靠优先级隐藏冲突，也不得只在Editor菜单阻止而允许脚本/property旁路。

## 5. 目标架构

| 组件 | 所属 | 责任 |
|---|---|---|
| `SceneLightCommon` | Runtime Scene | stable identity、color/temperature、photometric value/unit、participation、mobility、layer、fade、priority与source generation |
| `SceneLightShape` | Runtime Scene | Directional/Point/Spot/Rect的受约束type-specific source geometry；只有一个active variant |
| `SceneLightShadowSource` | Runtime Scene | casts/method/bias/strength/resolution/update/cascade/near/fade/cache policy，编译到09E contract |
| `SceneLightTextureBindings` | Runtime Scene/Resource | cookie、IES与profile handle、sampler/UV/projection policy及dependency generation |
| `ValidatedSceneLightDescriptor` | Runtime Scene | schema migration、finite/range/unit/shape/participation校验后的canonical immutable value |
| `SceneLightMutation` | Runtime Scene | create/replace/edit/remove的preflight/CAS/commit/receipt，维护kind/component/property守恒 |
| `SceneLightDeltaExtract` | Scene -> Framework | created/changed/removed、world transform/bounds、effective descriptor、dependency与source generation |
| `PreparedSceneLightGeneration` | Graphics | 09E拥有的packed index、cluster/shadow/cookie/IES resolution、last-good与failure状态 |
| `SceneLightSubmissionReceipt` | Framework/diagnostics | requested/effective unit、shape、participation、accepted/rejected/degraded reason、cost与consumer generations |
| `SceneLightAuthoringAdapter` | Editor | unit-aware Inspector、range/cone/area handles、gizmo/pick、preview、undo与save/reopen，复用runtime compiler |

方向权威必须统一为world transform basis；若产品确需独立aim vector，必须是显式aim mode并定义parent/scale/constraint行为，不能让Directional/Spot用字段、Rect用transform。所有consumer只读同一validated/effective descriptor，不再各自归一化、clamp或猜单位。

## 6. P1差距与重构定义

### 6.1 Source identity、schema与migration

| ID | 当前差距 | 重构完成定义 |
|---|---|---|
| RSL-P1-001 | 五类struct复制color/intensity等字段，没有共同light contract | common descriptor统一identity、generation、color/photometry/participation；shape只保存差异字段 |
| RSL-P1-002 | Scene Light asset没有schema version、presence或migration receipt | versioned reader/writer与逐版本migration，unknown/newer版本fail closed且旧字段可审计 |
| RSL-P1-003 | Directional/Point字段无serde defaults，Ambient/Rect/Spot却各自使用不同策略 | canonical defaults由单一definition生成component/asset/Editor模板，字段presence与默认来源可追踪 |
| RSL-P1-004 | NodeKind、component set与asset字段是三套可漂移truth | transactional light mutation同时维护kind、component、inspection和persistence invariant |
| RSL-P1-005 | clone、bundle staging和generic add/remove没有light-specific admission | 所有入口经过同一preflight；非法组合无partial mutation并返回typed conflict |
| RSL-P1-006 | asset cache payload未表达light schema/dependency generation | artifact记录schema/source/dependency generation与migration provenance，stale payload不得发布 |

### 6.2 Photometry、color与shape

| ID | 当前差距 | 重构完成定义 |
|---|---|---|
| RSL-P1-007 | `intensity`没有单位，不同光种默认值不可比较 | directional用lux，punctual可选lumen/candela/lux-at-distance/EV，area可选lumen/nits/EV；内部canonical单位明确 |
| RSL-P1-008 | 没有共享单位转换或数值资格 | 按solid angle/source area/distance转换并覆盖round-trip、极值、非法组合和参考数值测试 |
| RSL-P1-009 | color只是Vec3，无working color space、temperature或tint语义 | linear working-space color、Kelvin/tint enable policy与conversion generation明确，Editor显示真实单位 |
| RSL-P1-010 | Point/Spot只有range，没有inverse-square/source radius/length/falloff policy | physically based falloff为默认，cutoff只做bounded influence；source geometry与legacy unitless模式显式迁移 |
| RSL-P1-011 | Rect只有size/range，无basis、barn door、two-sided、source texture语义 | width/height、orientation、barn door、sidedness与emissive/source texture形成validated area shape |
| RSL-P1-012 | Directional没有angular diameter/soft source，Spot没有source radius且cone无约束 | type-specific shape含sun angle、radius/length、inner/outer cone并在setter/compiler统一clamp或拒绝 |

### 6.3 Transform、bounds、participation与budget

| ID | 当前差距 | 重构完成定义 |
|---|---|---|
| RSL-P1-013 | Directional/Spot忽略world rotation，Rect使用world forward | 单一transform/aim authority，parent/reparent/negative scale/zero scale有确定规则与测试 |
| RSL-P1-014 | raw direction可为NaN、Inf、zero或非归一化 | validation在mutation/load边界完成；extract不再临时normalize或fallback |
| RSL-P1-015 | light没有canonical influence bounds或spatial revision | Point sphere、Spot cone、Rect oriented bounds、Directional global scope进入Runtime62 spatial delta |
| RSL-P1-016 | Ambient/Rect snapshot缺mobility，Ambient还缺identity/layer | 所有family保留stable ID、layer、mobility、source generation与适用scope，不靠数组位置识别 |
| RSL-P1-017 | 只有ambient的lightmapped布尔，其他light没有bake/static/dynamic/indirect策略 | common participation表达Disabled/Static/Stationary/Movable、direct/indirect/bake/reflection/GI/volumetric |
| RSL-P1-018 | 没有max draw/fade、priority、cost class或overload policy | source给出距离fade、importance、quality eligibility；Runtime65/09E仲裁并返回effective receipt |

### 6.4 Shadow、cookie、IES与resource dependency

| ID | 当前差距 | 重构完成定义 |
|---|---|---|
| RSL-P1-019 | component/asset/property没有`LightShadowSettings`入口 | 按Runtime09E P0-1接入versioned shadow source并逐字段守恒到extract，不在本篇复制renderer P0 |
| RSL-P1-020 | shadow source缺update mode、near/fade/cascade/cache/mobility关系 | typed policy覆盖EveryFrame/Cached/OnDemand及type-specific参数，非法组合在authoring admission拒绝 |
| RSL-P1-021 | 没有cookie/projector字段、UV/projection或dependency | qualified texture handle、projection transform、sampler/color policy与generation进入resource graph |
| RSL-P1-022 | 没有IES profile、brightness/unit或type compatibility | qualified IES artifact、point/spot/rect布局、normalization和unit conversion明确且可预览 |
| RSL-P1-023 | cookie/IES缺失、加载、reload、eviction没有typed state | Runtime64提供async ticket/version lease/last-good；extract和receipt区分Waiting/Ready/Failed/Degraded |
| RSL-P1-024 | layer只按camera过滤，receiver/caster/volumetric没有共同mask source | source定义lighting与shadow-caster mask；实际shader消费与P0资格仍由Runtime09E唯一拥有 |

### 6.5 Reflection、property、compiled binding与mutation

| ID | 当前差距 | 重构完成定义 |
|---|---|---|
| RSL-P1-025 | reflection可见`volumetric`，entries/read/write却全部遗漏 | schema生成reflection、enumeration、read/write与compiled field，字段集合差异测试为0 |
| RSL-P1-026 | compiled light enum/writer同样遗漏所有`volumetric` | compiled plan覆盖完整public schema并绑定schema generation；旧handle在migration后拒绝 |
| RSL-P1-027 | generic writer直接写NaN、负range、零direction和逆序cone | 每个property action调用同一descriptor validator，失败不改变World且返回字段地址/原因 |
| RSL-P1-028 | reflection metadata没有unit/range/conditional visibility或shape hint | Runtime63 schema发布可执行metadata；validator是权威，hint不只是UI装饰 |
| RSL-P1-029 | 多字段编辑逐字段提交，中间可出现非法light | photometry/unit、inner/outer、width/height等以atomic patch preflight/CAS/commit |
| RSL-P1-030 | write结果只有changed bool，缺source/effective generation和consumer invalidation | mutation receipt携before/after revision、affected bounds/dependencies/consumers与recompile reason |

### 6.6 Extract、lifecycle、readiness与performance

| ID | 当前差距 | 重构完成定义 |
|---|---|---|
| RSL-P1-031 | viewport packet和frame extract各自调用五个collector | 单一prepared SceneLightGeneration被所有view/frame consumer只读复用 |
| RSL-P1-032 | 每个view按family全量扫描、分配、排序component store | change-driven registry维护stable order与bounds；view只查询候选并复用scratch/capacity |
| RSL-P1-033 | 没有created/changed/removed delta或removal receipt | extract携entity/source generation的增删改，zero-change帧产生0 repack/0 upload |
| RSL-P1-034 | Ambient extract丢`affects_lightmapped_meshes`且无identity/layer/mobility | common/effective descriptor逐字段守恒，任何drop必须是显式compile policy和receipt |
| RSL-P1-035 | volumetric只以独立`Vec<u64>` sideband发布 | participation属于每盏灯同一generation，froxel只消费qualified packed index/descriptor |
| RSL-P1-036 | readiness对Directional/Point/Spot按count全标ready | readiness来自最终validated/prepared/submitted generation，覆盖dependency、budget、renderer与device状态 |

### 6.7 Editor authoring、gizmo与preview

| ID | 当前差距 | 重构完成定义 |
|---|---|---|
| RSL-P1-037 | Inspector只显示derive字段，裸float/vector没有单位、范围或条件 | unit-aware controls、temperature、shadow/cookie/IES、participation与validation message由schema驱动 |
| RSL-P1-038 | 创建模板沿runtime默认，asset默认却漂移 | create command调用canonical descriptor factory，立即save/reopen后逐字段相同 |
| RSL-P1-039 | Point没有radius/range gizmo或pick shape | sphere/source radius、range、fade handles可交互并进入单一transaction |
| RSL-P1-040 | Spot没有cone/range/radius gizmo或pick shape | inner/outer cone、source radius、range与orientation handles使用world transform authority |
| RSL-P1-041 | Rect没有area/range/barn-door gizmo或pick shape | rectangle、normal、range、barn door与sidedness可视化并有stable picking |
| RSL-P1-042 | Directional gizmo优先raw direction，旋转节点无效 | gizmo、renderer和Inspector共享effective direction；transform/aim切换产生明确undo与receipt |

### 6.8 Diagnostics、tests与product qualification

| ID | 当前差距 | 重构完成定义 |
|---|---|---|
| RSL-P1-043 | diagnostics只报告family count或Rect固定degraded string | per-light receipt可追踪source/effective unit、validation、dependency、budget、shadow/cluster接受原因 |
| RSL-P1-044 | tests只覆盖显式字段roundtrip，未覆盖partial/legacy/malformed | empty/partial/version skew/NaN/Inf/negative/zero/angle/order/default parity形成table-driven tests |
| RSL-P1-045 | `persistent_lighting`把五灯同entity当成功，没有非法组合测试 | composition invariant覆盖所有add/remove/replace/clone/deserialize/undo入口及rollback |
| RSL-P1-046 | 没有transform/parent/reparent方向与bounds测试 | Directional/Spot/Rect在rotation、parent、scale、reparent后authoring/gizmo/extract一致 |
| RSL-P1-047 | shadow/cookie/IES/volumetric/layer/bake只靠手工snapshot或零覆盖 | asset -> World -> Editor -> extract -> renderer产品场景覆盖load/reload/failure/degrade/save/reopen |
| RSL-P1-048 | 没有many-light、多view、长时间变更或allocation/byte基准 | 固定workload验证steady/dirty CPU、allocation、extract bytes、GPU upload、budget稳定性与soak |

## 7. P2演进项

| ID | 演进项 | 启动前提 |
|---|---|---|
| RSL-P2-001 | spectral/SPD light source与wavelength-aware material/medium | 物理单位、working color space和基线能量守恒先通过 |
| RSL-P2-002 | calibrated fixture/IES measurement import与真实灯具库 | IES artifact、unit conversion、license/provenance和Editor preview闭环 |
| RSL-P2-003 | Disk、Tube、Sphere、Polygon与Mesh emitter source shape | Rect/Point source geometry、09E area integration与bounds先合格 |
| RSL-P2-004 | artist light linking group、include/exclude collection与component filter | 基础lighting/caster mask在所有consumer真实生效 |
| RSL-P2-005 | animated gobo、flicker、DMX/timecode与Sequencer binding | stable property address、atomic patch和deterministic time owner成立 |
| RSL-P2-006 | emissive geometry与visible source mesh能量联动 | material emissive、area source和bake/GI避免double lighting |
| RSL-P2-007 | celestial sun/moon、sky atmosphere与geolocation binding | Runtime09F1的environment owner和directional authority完成 |
| RSL-P2-008 | ray/path tracing inclusion、sampling weight与MIS source | HRT capability、shared photometric descriptor和fallback parity完成 |
| RSL-P2-009 | adaptive light LOD、stochastic many-light source admission | Runtime09E/65有正确稳定baseline和同画质性能证据 |
| RSL-P2-010 | multi-user light authoring lock、presence与conflict merge | Editor multi-user transaction和stable property identity完成 |
| RSL-P2-011 | prefab/class-default light override与schema-aware propagation | Runtime61/63及Editor44的default/override authority完成 |
| RSL-P2-012 | 自动光度校准实验室、reference render与跨后端差分 | 产品场景、capture、pixel metric、hardware matrix和currentness成立 |

## 8. 实施里程碑

### M71-1：Schema hard cutover与P0关闭

- 建立versioned `SceneLightCommon`、`SceneLightShape`和canonical defaults；
- 迁移五个独立Option与NodeKind优先级，所有mutation入口执行互斥不变量；
- 用partial/legacy/malformed/save-reopen测试关闭RSL-P0-001/002。

### M71-2：Photometry、transform与participation

- 建共享unit conversion、temperature/color与type-specific source geometry；
- 统一transform/aim authority、bounds和mobility/bake/GI/volumetric策略；
- 所有setter/property/asset load消费同一validator。

### M71-3：Shadow、cookie、IES与resource generation

- 接入Runtime09E shadow source合同，不复制shadow renderer；
- 建cookie/IES artifact dependency、version lease、last-good与typed failure；
- layer/caster/volumetric mask在source到最终consumer逐字段守恒。

### M71-4：Delta extract、readiness与budget

- 建change-driven light registry、stable index、created/changed/removed delta；
- 多view复用prepared generation，zero-change不全扫/不重pack；
- readiness和diagnostics由最终generation/receipt产生。

### M71-5：Editor产品闭环

- schema驱动unit-aware Inspector与conditional controls；
- 完成Point/Spot/Rect/Directional gizmo、handles、picking、preview和degradation；
- create/edit/undo/save/reopen/play round-trip逐字段一致。

### M71-6：Correctness、fault、scale与performance资格

- 建asset -> World -> Editor -> extract -> GPU的产品测试与reference pixel场景；
- 覆盖reload/device loss/budget rejection/version skew/many-light/multi-view/soak；
- 同场景同画质基准后才允许任何性能或表现比较结论。

## 9. 资格门

| Gate | 通过条件 |
|---|---|
| RSL-GATE-001 | component、asset、Editor create使用同一canonical defaults |
| RSL-GATE-002 | empty/partial Spot资产不再产生+Y、百万强度、零锥角静默结果 |
| RSL-GATE-003 | legacy schema有确定migration与receipt，newer unknown schema fail closed |
| RSL-GATE-004 | 任一entity最多一个effective light shape，所有入口同样执行 |
| RSL-GATE-005 | 非法composition失败后World、kind、inspection与asset逐字节不变 |
| RSL-GATE-006 | clone/undo/save/reopen保持light identity、shape、fields与generation |
| RSL-GATE-007 | 每类光的native/allowed photometric units明确 |
| RSL-GATE-008 | lumen/candela/lux/nits/EV转换与reference数值、round-trip通过 |
| RSL-GATE-009 | color/temperature在working color space有确定conversion与范围 |
| RSL-GATE-010 | physical falloff、cutoff range与source geometry语义不混淆 |
| RSL-GATE-011 | Point/Spot source radius/length与Rect width/height/barn door可持久化 |
| RSL-GATE-012 | Directional angular source与soft shadow输入可持久化 |
| RSL-GATE-013 | Directional/Spot/Rect方向均来自同一transform/aim authority |
| RSL-GATE-014 | parent/reparent/rotation/negative scale规则有行为测试 |
| RSL-GATE-015 | NaN/Inf/zero direction、负range/intensity/size与逆序cone被拒绝 |
| RSL-GATE-016 | 每类光有canonical bounds/global scope与spatial revision |
| RSL-GATE-017 | Ambient/Rect保留stable ID、layer、mobility与source generation |
| RSL-GATE-018 | bake/direct/indirect/reflection/GI/volumetric参与策略显式 |
| RSL-GATE-019 | Scene shadow source逐字段到09E extract，不再静默`None` |
| RSL-GATE-020 | shadow update/cache/near/fade/cascade非法组合在admission拒绝 |
| RSL-GATE-021 | cookie使用qualified handle并携projection/sampler/generation |
| RSL-GATE-022 | IES按light type校验布局、unit、normalization与artifact generation |
| RSL-GATE-023 | cookie/IES load/reload/evict有Waiting/Ready/Failed/Degraded状态 |
| RSL-GATE-024 | lighting/caster/volumetric mask在最终consumer有逐对象资格证据 |
| RSL-GATE-025 | reflection/enumeration/read/write/compiled public field集合完全相同 |
| RSL-GATE-026 | `volumetric`可经generic、compiled、Inspector与save/reopen守恒 |
| RSL-GATE-027 | property invalid write不改变World并返回完整typed address/reason |
| RSL-GATE-028 | unit/range/condition metadata与runtime validator一致 |
| RSL-GATE-029 | multi-field light patch只有一个commit point和revision |
| RSL-GATE-030 | mutation receipt携before/after、bounds/dependency/consumer invalidation |
| RSL-GATE-031 | viewport/frame/multi-view消费同一prepared light generation |
| RSL-GATE-032 | steady frame不全扫五个store、不重新分配/排序全部light |
| RSL-GATE-033 | created/changed/removed delta与entity generation可验证 |
| RSL-GATE-034 | Ambient的lightmapped、layer、mobility等字段不在extract丢失 |
| RSL-GATE-035 | volumetric不是裸ID sideband，而是同代participation contract |
| RSL-GATE-036 | readiness来自validated/prepared/submitted状态而非family count |
| RSL-GATE-037 | Inspector显示并写入正确单位、范围、条件与validation error |
| RSL-GATE-038 | Editor create后立即save/reopen与创建前descriptor一致 |
| RSL-GATE-039 | Point radius/range/fade gizmo和pick shape可交互 |
| RSL-GATE-040 | Spot inner/outer/range/radius/orientation gizmo可交互 |
| RSL-GATE-041 | Rect area/normal/range/barn-door gizmo可交互 |
| RSL-GATE-042 | Directional gizmo、node rotation与renderer effective direction一致 |
| RSL-GATE-043 | per-light diagnostics解释validation/dependency/budget/renderer状态 |
| RSL-GATE-044 | partial/legacy/malformed/极值测试覆盖所有light type |
| RSL-GATE-045 | composition测试覆盖add/remove/replace/clone/load/undo全部入口 |
| RSL-GATE-046 | transform/bounds/gizmo/extract跨parent/reparent行为一致 |
| RSL-GATE-047 | shadow/cookie/IES/layer/bake/volumetric有真实产品链和fault测试 |
| RSL-GATE-048 | many-light/multi-view/soak记录CPU、allocation、bytes、GPU与budget证据 |

## 10. 禁止的临时修法

1. 禁止只把Spot asset默认改成runtime数值，却不建立schema版本、presence和migration测试。
2. 禁止只在Editor菜单阻止多light component，而让serde、script、reflection或typed API继续制造隐藏发光源。
3. 禁止继续保存独立direction并在不同consumer选择字段或transform；不得用normalize-or-zero掩盖非法source。
4. 禁止把`LightShadowSettings`简单塞进snapshot或测试fixture而不贯通asset/property/World/extract。
5. 禁止用字符串cookie/IES路径、同步磁盘读取、永久强引用或renderer私有texture handle旁路Runtime64。
6. 禁止把light unit作为UI label而renderer仍按裸float猜测；转换、canonical storage与测试必须共同落地。
7. 禁止用更多per-view `Vec`、hash scan、sort或source-string测试冒充incremental generation和性能资格。
8. 禁止在本篇复制Runtime09E的cluster/shadow/area-light finding，或以文档完成宣称renderer已经实现。

## 11. 状态

- `review_status`: `review_complete`
- `implementation_status`: `pending`
- `source_recheck_required`: `true`
- 本轮新增：**2 P0 / 48 P1 / 12 P2 / 48 gates**
- 共享最高优先级阻断继续路由Runtime09E、Runtime61/62/63/64/65与Editor03/05/22，不在本篇重复计数。
- 下一步不是继续堆字段，而是先完成M71-1的versioned schema、canonical defaults与composition hard cutover；随后才能接photometry、resource、delta extract和Editor产品链。
