---
related_code:
  - docs/engine-architecture/runtime-foundation-precision-and-scene-authority.md
  - zircon_runtime_interface/src/math.rs
  - zircon_runtime_interface/src/reflect/reflected_value.rs
  - zircon_runtime_interface/src/ui/surface/hit.rs
  - zircon_runtime/src/core/math
  - zircon_runtime/src/core/framework/animation/parameter_value.rs
  - zircon_runtime/src/core/framework/ai/perception.rs
  - zircon_runtime/src/core/framework/picking/ray.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/view_matrix_pair.rs
  - zircon_runtime/src/core/framework/render/mesh/bounds.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/core/framework/render/visible_spatial_query.rs
  - zircon_runtime/src/core/framework/sound/components.rs
  - zircon_runtime/src/core/framework/sound/emission.rs
  - zircon_runtime/src/navigation
  - zircon_runtime/src/scene/components/scene/transform.rs
  - zircon_runtime/src/scene/world/transform_validation.rs
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/world/project_io/transform.rs
  - zircon_runtime/src/scene/world/compiled_binding/property_path.rs
  - zircon_runtime/src/scene/world/property_access/value_conversion.rs
  - zircon_runtime/src/scene/world/property_access/write.rs
  - zircon_runtime/src/asset/assets/scene
  - zircon_runtime/src/asset/formats/obj
  - zircon_runtime/src/asset/importer
  - zircon_runtime/src/graphics/scene/gpu_scene
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform
  - zircon_runtime/src/graphics/scene/scene_renderer/temporal/velocity
  - zircon_runtime/src/graphics/visibility
  - zircon_runtime/src/graphics/shader
  - zircon_plugins/physics/runtime/src/backend/builtin/step.rs
  - zircon_plugins/physics/runtime/src/backend/jolt
  - zircon_plugins/particles/runtime/src/asset.rs
  - zircon_plugins/particles/runtime/src/simulation
  - zircon_plugins/gltf_importer
  - zircon_plugins/obj_importer
  - zircon_editor/src/scene/viewport/projection.rs
  - zircon_editor/src/scene/viewport/handles
  - zircon_editor/src/scene/viewport/pointer
tests:
  - zircon_runtime/src/scene/tests/world_basics
  - zircon_runtime/src/scene/tests/property_paths
  - zircon_runtime/src/graphics/scene/gpu_scene/gpu_scene/tests.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_uniform/from_frame.rs
  - zircon_runtime/src/tests/runtime_absorption
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/08a-physics-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08b-audio-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08c-animation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/08d-navigation-runtime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/09h1-temporal-aa-velocity-history-upscaling-review.md
  - docs/plans/optimize/zircon_runtime/17-woc-world-terrain-collision-locomotion-spawn-spatial-targeting-runtime-review.md
  - docs/plans/optimize/zircon_runtime/22-time-clock-domain-fixed-step-determinism-rng-replay-scheduling-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_tooling/22-magic-constant-sentinel-threshold-timeout-capacity-budget-policy-convergence-review.md
  - docs/plans/optimize/zircon_tooling/23-failure-contract-panic-unwind-error-propagation-poison-recovery-result-observability-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/LargeWorldCoordinates.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/LargeWorldRenderPosition.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Math/DoubleFloat.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/SceneView.h
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition-config/Runtime/ShaderConfig.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/ShaderLibrary/SpaceTransforms.hlsl
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/RenderPipeline/Camera/HDCamera.cs
  - dev/godot/core/math/math_defs.h
  - dev/godot/core/math/vector3.h
  - dev/godot/core/math/basis.h
  - dev/godot/core/math/transform_3d.h
  - dev/bevy/crates/bevy_transform/src/components/transform.rs
  - dev/bevy/crates/bevy_transform/src/components/global_transform.rs
  - dev/bevy/crates/bevy_math/src/direction.rs
  - dev/Fyrox/fyrox-impl/src/scene/transform.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 23 · Coordinate Space、Unit、Precision、Transform、Numeric Robustness 与 Large World 工程化差距

## 1. 结论

Zircon 的数学基础不是临时散落的 `glam` 调用。`zircon_runtime_interface::math` 已经成为真实共享 seam：`Real/Vec*/Quat/Mat4/Transform` 由单点导出，runtime、plugins、editor、app 与 hub 基本遵守该入口；render 侧另有固定 `f32` 的 `Render*` alias 和 `to_render_*` helper；scene transform 写入会拒绝非有限 translation/rotation/scale、零长 quaternion 和精确零 scale。坐标约定也能从 `-Z forward`、`+Y up`、`perspective_rh` 与 camera/view helper中推导为右手系。这些基础应保留，不能因为要做大世界就让每个 subsystem 自己引入 `DVec3`、cell 或私有单位。

但现有架构文档把未来 `f64` 迁移描述得过于局部。`Real` 仍固定为 `f32`，没有 precision profile、Cargo feature 或 CI lane；reflection value、animation parameter、sound spatial DTO、render bounds和若干跨域数组仍硬编码 `f32`。`to_render_scalar` 只检查转换前有限，未来 `f64` 超出 `f32` 范围时可以在 cast 后成为 infinity。Scene persistence、plugin capability、wire/schema identity也没有记录precision、coordinate convention或unit system。因此把alias改为double不会是局部切换，而会同时触发序列化、反射、资产、插件、编辑器、网络/回放和render extraction迁移。

更关键的是，仓内没有大世界坐标authority。CPU world translation本身是absolute `f32`；GPU scene上传absolute `world_from_local`和`prev_world_from_local`的`f32`矩阵，SceneUniform上传absolute camera position，shader在absolute world space相减。对`world_origin/origin_offset/relative_origin/camera_relative/floating_origin/rebase/large_world`的11个production-like命中逐条分类后，只剩Editor popup origin和temporal参数兼容命名，没有world rebase、origin generation、tile/cell坐标或per-view relative origin实现。远离原点时，精度在CPU scene阶段已经丢失，之后的render downcast helper无法补救。

空间和单位合同同样缺失。公开边界大量复用裸`Vec3/[f32; 3]`表达local/world position、direction、normal、velocity、bounds center和ray；没有`SpaceKind`或typed position/direction/normal。builtin physics将`Vec3(0,-9.81,0)`直接当gravity，material又单独提供`subsurface_world_unit_scale`，但项目/scene/import artifact没有全局长度单位或source unit receipt。glTF、OBJ、physics、navigation、audio、AI、particles与Editor gizmo之间只能依靠命名和隐含约定保持一致。

当前transform validator也只证明“不是NaN/Inf、不是精确零”。它接受任意非单位quaternion与任意接近零scale；层级矩阵传播不检查有限性、determinant、condition或误差增长；`affine_inverse`对奇异矩阵直接调用`inverse()`；`looking_at`在eye==target或up共线时静默构造退化basis；perspective只夹住aspect和near的一部分输入。机器`EPSILON`又被多个几何域当作业务容差，无法随世界单位、对象尺度和算法条件调整。

本篇拥有引擎级Coordinate/Unit/Precision schema、typed space boundary、large-world CPU representation、origin generation/rebase transaction、render-relative extraction、precision narrowing receipt和通用transform/numeric validity policy。Runtime05继续拥有ECS hierarchy/dirty frontier/World lifecycle；Runtime08A/08B/08D分别拥有physics/audio/navigation具体backend；Runtime09B/09H1拥有GPU Scene/temporal实现；Runtime17拥有WOC terrain/world业务；Editor03拥有gizmo transaction。本文不重复这些报告的局部finding。本轮登记 **0项P0、40项P1和12项P2**，均未实施。

## 2. 审查边界、方法与 currentness

### 2.1 物理扫描

本轮对`zircon_runtime`、`zircon_runtime_interface`、`zircon_plugins`、`zircon_editor`、`zircon_app`与`zircon_hub`的tracked Rust执行production-like词法inventory，排除显式tests目录、`tests.rs`和examples路径。该集合有11,936个文件；`Real`信号2,544处/339文件，`Vec3` 2,792处/408文件，原始`[f32; 3]` 206处/84文件，`f64` 949处/255文件，`is_finite` 1,582处/544文件，`.inverse()` 28处/19文件，normalize信号311处/156文件，`EPSILON` 634处/255文件。

这些数量只用于定位：`f64`包含时间、统计与业务标量，不代表world precision；`[f32; 3]`大量是mesh/GPU格式，本来就应保持f32；`EPSILON`不是自动缺陷。确认项来自逐owner和数据流阅读。直接`glam`引用的结果反而是正证据：runtime主体1处/1文件，runtime interface 10处/1文件，plugins/editor/app/hub均为0，说明共享alias cutover真实生效。

world-origin相关11个命中经人工分类没有产品大世界实现；unit相关49个命中主要是viewport world-units-per-pixel、subsurface profile、tangent handedness、shadow texel与font units-per-em，不构成World Unit System。Cargo/workflow扫描也没有发现precision/double/large-world build profile或f64 qualification lane。

### 2.2 深读调用链

1. `math::Transform -> compose_trs -> LocalTransform -> validate_transform_for_write -> WorldMatrix propagation -> matrix_to_transform`。
2. `TransformAsset/reflection/property path -> Quat::from_array -> scene validator -> persistence roundtrip`。
3. `Scene extract/RenderMeshBounds -> GPU Scene current/previous matrices -> SceneUniform camera -> WGSL world-space lighting/velocity`。
4. Physics gravity、navigation direction、sound position/velocity、AI perception、particles与UI world ray对空间/单位类型的消费。
5. core/plugin glTF/OBJ importer、mesh tangent/normal与asset artifact中source coordinate/unit provenance的存在性。
6. precision alias、reflection/animation/wire schema、plugin capability与BuildSet identity之间的迁移边界。

本轮源revision为`ae2be3d865a937b9ed368bf965592045346c64e3`。共享math、reflection、bounds、scene transform validator/project conversion、GPU layout和SceneUniform路径在检查时没有工作区差异；scene tests与schedule、Editor等其他区域仍有其他Session在途修改，所以标记`source_recheck_required: true`，实施前必须重取fingerprint。既有Editor、Hub、WOC和plugin动态验证阻断没有变化，本轮没有重复运行不能抵达coordinate/large-world产品语义的Cargo/npm lane。

## 3. 当前可保留的工程基础

| 基础 | 当前证据 | 保留条件 |
|---|---|---|
| 单一数学seam | runtime/interface集中导出`Real/Vec/Quat/Mat4/Transform`，下游直接glam引用极少 | 所有precision profile仍从一个owner生成，不允许subsystem私设double alias |
| render precision分层 | `RenderScalar/RenderVec/RenderMat4`固定f32并有conversion helper | 补range、post-cast finite、relative origin和receipt，不把GPU全链盲目改成f64 |
| finite写入校验 | scene拒绝transform非有限分量、零长quat、精确零scale | 升级为unit quaternion、conditioning和统一typed validator |
| local/world分离 | `LocalTransform`与derived `WorldMatrix`职责已分开 | world representation要能携origin/cell generation，避免退回每组件缓存双真值 |
| 稳定层级传播 | iterative hierarchy traversal避免深链递归栈 | 与Runtime05 dirty frontier合并，并在compose处发布numeric failure/delta |
| 明确相机约定 | RH perspective、+Y up、-Z forward/right/up helper一致 | 固化为versioned CoordinateSchema和shader/import conformance test |
| animation quat归一化 | 多个采样/pose路径显式normalize输入quat | 把同一invariant提升到scene/import/reflection authority，不靠每消费者补救 |
| ordered数据结构基础 | scene/asset/runtime广泛使用稳定容器和显式generation | origin/precision/unit schema进入snapshot/hash/replay，而不是只作为运行时全局变量 |

## 4. 参考实现给出的边界

### 4.1 Unreal：CPU double与render-relative不是同一层

Unreal的Large World Coordinates让默认`FVector`等real类型使用double，同时保留显式float/double类型；`UE_REAL_TO_FLOAT`及clamped变体让narrowing可搜索。render侧没有把所有shader直接改成FP64，而是用tile+offset、relative world matrix、translated world/view、high+low double-float和current/previous origin承接GPU精度。`FSceneView`同时保存ViewOrigin、PreViewTranslation、TranslatedWorldToClip和前后帧origin信息。

Zircon应吸收两层合同：CPU world coordinate有足够range/precision；GPU按view建立相对坐标且前后帧origin可追踪。只把`Real`改为f64却继续上传absolute f32 matrix，或只做floating origin却让save/network/replay保存未标epoch的local值，都不是完整方案。

### 4.2 Unity Graphics：camera-relative必须贯穿current/previous frame

Unity Graphics的HDRP配置显式启用Camera Relative Rendering；`SpaceTransforms.hlsl`提供absolute与camera-relative world position转换，HDCamera会处理current/previous camera displacement和矩阵。应借鉴的是per-view相对空间、history correction和shader helper统一入口，而不是把camera position随意从每个object translation中减一次。

### 4.3 Godot：precision是build contract，数学对象暴露validity语义

Godot用`REAL_T_IS_DOUBLE`控制`real_t`为float或double；Vector/Basis/Transform API明确提供finite、normalized、orthogonal、orthonormal、conformal、orthonormalize和affine inverse等语义。Zircon不需要复制宏方案，但必须让precision profile成为可构建、可序列化、可测试的产品身份，并把数学前置条件从调用者猜测升级为typed result/validator。

### 4.4 Bevy与Fyrox：强类型方向、affine world和单位quaternion

Bevy的`Dir3`把单位方向与任意`Vec3`区分，构造时拒绝zero/NaN/infinite，并提供误差再归一化；`GlobalTransform`以`Affine3A`表达world affine，而不是声称总能无损还原TRS。Fyrox transform用`UnitQuaternion<f32>`保存local/pre/post rotation，并明确pivot、offset和矩阵缓存/分解限制。

这些参考并不证明Bevy或Fyrox已经解决大世界；它们提供的是方向/旋转invariant、affine与TRS语义边界。Zircon应结合Unreal/Unity的大世界模型，而不是从任一引擎表面API单独推导完整设计。

## 5. Owner裁决与非重复边界

| Owner | 本篇拥有 | 邻接报告继续拥有 |
|---|---|---|
| Coordinate Schema | handedness、axis、forward/up、winding、clip/depth、space taxonomy与版本 | asset报告拥有artifact/cache/import生命周期 |
| Unit System | length/angle/time派生单位、project/source unit与conversion receipt | physics/audio/navigation拥有具体solver/DSP/query语义 |
| Precision Profile | CPU real/world position、storage/reflection/wire identity、narrowing policy | Tooling Cargo/BuildSet报告拥有通用feature/resolved graph |
| Transform Numeric Policy | unit quat、scale/reflection/shear、finite/condition/inverse/look-at/perspective failure | Runtime05拥有hierarchy topology、dirty frontier和World lifecycle |
| Large World Authority | cell/origin/generation、rebase transaction、cross-subsystem shift contract | Runtime17拥有WOC terrain/streaming/spawn业务；Runtime09B拥有GPU Scene实现 |
| Render Relative Space | per-view origin、current/previous correction、CPU-to-GPU relative extraction | Runtime09H1拥有velocity/history算法与reset policy |

必须避免两种错误合并。第一，BuildSet identity、precision profile与world-origin generation相关但不等价：前者标识构建格式，后者标识一次运行中的坐标基准。第二，`WorldPosition`、`LocalPosition`、`Direction`与`Normal`都可能由三个数表示，但转换律、允许运算和单位不同，不能只加字段名继续复用裸数组。

## 6. P1：Precision、Persistence、Reflection 与 ABI

| ID | 当前差距 | 需要重构 |
|---|---|---|
| COORD-P1-001 | `Real=f32`是唯一可构建profile，CPU world在远离原点前已丢精度 | 建立versioned `PrecisionProfile`，至少明确CurrentF32与LargeWorld方案，先由产品需求和benchmark裁决f64或cell+offset |
| COORD-P1-002 | 没有f64/large-world compile lane，架构文档的future-ready无法证明 | 增加compile/test/schema migration矩阵；禁止只改alias后以局部unit test宣称完成 |
| COORD-P1-003 | `ReflectedValue::Scalar/Vec*/Quaternion`硬编码f32 | 引入明确numeric kind/precision和schema版本；旧值迁移、范围错误与roundtrip必须可诊断 |
| COORD-P1-004 | `AnimationParameterValue`硬编码f32数组且可承载transform相关参数 | 区分dimensionless/render参数与runtime-real/typed transform参数，compiler决定存储和转换 |
| COORD-P1-005 | sound position/velocity仍是`[f32;3]`，不经过共享world math schema | 用typed spatial snapshot接收world-relative值，DSP边界再显式narrow并记录origin generation |
| COORD-P1-006 | `RenderMeshBounds`以f32存储，却直接接收runtime `Transform`做world变换 | 分离local asset bounds与world/relative bounds；downcast只发生在命名清晰的render extraction boundary |
| COORD-P1-007 | `to_render_scalar`只验证cast前finite，future f64超range可cast为Inf | 检查representable range、cast后finite和误差policy，返回`NarrowingReceipt`或typed failure |
| COORD-P1-008 | Scene/asset persistence没有precision、coordinate、unit schema identity | document/artifact header记录schema IDs和migration owner；未知组合fail-close或保留opaque数据 |
| COORD-P1-009 | plugin/runtime interface没有协商precision与space capability | capability manifest声明accepted coordinate/precision ABI，adapter负责转换，禁止插件猜`Real`布局 |
| COORD-P1-010 | BuildSet、save、network、replay和cache key没有precision profile digest | 把profile/schema digest纳入兼容性、cache、snapshot和replay admission，跨profile加载必须迁移或拒绝 |

## 7. P1：Coordinate Space、Unit 与 Import Contract

| ID | 当前差距 | 需要重构 |
|---|---|---|
| COORD-P1-011 | RH、+Y up、-Z forward等只由代码推导，没有versioned schema | 定义`CoordinateSystemId`，覆盖handedness、axes、front、winding、matrix/vector convention和clip/depth |
| COORD-P1-012 | cross-system边界用裸Vec/array，space仅藏在字段名 | 建立`SpaceKind/SpaceId`与typed boundary，至少区分Local、Parent、World、ViewRelative、View、Clip、Screen |
| COORD-P1-013 | position/vector/direction/normal共用`Vec3`，非法相加和错误normal变换可编译 | 提供`Position3/Vector3/UnitDirection3/Normal3`语义或checked wrapper，normal使用inverse-transpose合同 |
| COORD-P1-014 | gravity 9.81隐含米制，但项目/scene没有Length Unit authority | 建立`UnitSystem`，固定canonical runtime unit并记录project/source display conversion；physics常量由typed acceleration生成 |
| COORD-P1-015 | degrees/radians主要靠命名约束，反射/JSON/插件边界可混淆 | 使用`Angle<Radians/Degrees>`或schema unit tag，UI显示单位与storage unit分离 |
| COORD-P1-016 | glTF/OBJ/import artifact不发布source coordinate/unit/conversion receipt | importer先识别source schema，再一次性转换到canonical asset space并保存source、transform、warnings和version |
| COORD-P1-017 | axis flip、winding、tangent handedness、mirrored transform没有统一转换规则 | `GeometryConversionPlan`原子处理position/normal/tangent/winding/inverse bind/animation/collider并做parity test |
| COORD-P1-018 | physics/navigation/audio/AI/particles各自消费位置/速度，没有space adapter | 每个subsystem声明accepted space/unit/origin generation，由中央adapter生成snapshot，mismatch直接拒绝 |
| COORD-P1-019 | parent non-uniform scale产生shear后仍可被`matrix_to_transform`近似分解 | 明确TRS与Affine合同；需要保真时保留affine，分解必须返回quality/residual而不是静默成功 |
| COORD-P1-020 | negative scale/reflection与near-zero scale只有零值检查，跨render/physics/gizmo语义不一 | 建立TransformPolicy：允许域、mirror/winding、collider支持、normal修正、sign crossing与最小condition阈值 |

## 8. P1：Large World 与 Render-relative Pipeline

| ID | 当前差距 | 需要重构 |
|---|---|---|
| COORD-P1-021 | CPU `WorldPosition`等于absolute f32 translation | 选择并固化double或`CellId + local offset`表示，定义range、resolution、canonicalization和运算 |
| COORD-P1-022 | 没有world cell/tile/sector、origin state或generation | 建`WorldCoordinateAuthority`与`WorldOriginState { anchor, generation }`，所有relative snapshot携generation |
| COORD-P1-023 | 没有origin rebase transaction/event/rollback | rebase在frame/tick barrier prepare/validate/commit，发布old/new delta、affected owners和failure receipt |
| COORD-P1-024 | GPU Scene上传absolute f32 current/previous object matrix | extract生成view-relative current/previous transform，absolute值不得直接进入GPU instance ABI |
| COORD-P1-025 | SceneUniform上传absolute camera position且shader在absolute world space相减 | 建per-view render origin与统一shader space helper；camera-relative space中camera通常接近零但保留absolute恢复接口 |
| COORD-P1-026 | previous matrix不记录previous origin，rebase会制造全场速度 | temporal payload携current/previous origin high/low或delta，camera cut/rebase/history policy共同处理 |
| COORD-P1-027 | bounds、visibility、spatial query没有cell/origin identity | broad phase以world cell/absolute authority存储，view snapshot转换为relative bounds并验证generation |
| COORD-P1-028 | physics/navigation没有world shift/rebuild/refit合同 | adapter声明native backend能力：shift origin、double world、tile rebuild或拒绝；操作进入同一rebase transaction |
| COORD-P1-029 | audio、particles、trails、decals和历史buffer没有rebase处理 | 分类为persistent absolute state、relative simulation state或view history，并逐owner实现shift/reset/preserve策略 |
| COORD-P1-030 | save/network/replay坐标没有cell/origin schema，relative值可能失去上下文 | 只持久化canonical absolute/cell坐标；packet/snapshot携schema、origin generation和quantization profile |

## 9. P1：Transform 与 Numeric Robustness

| ID | 当前差距 | 需要重构 |
|---|---|---|
| COORD-P1-031 | scene validator接受所有非零长度quaternion，project/reflection路径不normalize | 构造`UnitQuat`或原子normalize+误差阈值；过大偏差拒绝，微小漂移修正并记录receipt |
| COORD-P1-032 | scale只拒绝精确0，极小有限值可制造病态矩阵 | 按unit/scene magnitude定义minimum singular value/condition policy，authoring时阻止不可逆transform |
| COORD-P1-033 | `affine_inverse`和多个consumer直接`inverse()`，奇异输入返回无效矩阵 | 提供`try_affine_inverse`，验证finite/determinant/condition并让caller选择reject、fallback或disable |
| COORD-P1-034 | `looking_at`对eye==target或up共线用zero vector继续构造basis | 返回`Result<Transform, LookAtError>`或显式fallback axis，退化原因进入diagnostic |
| COORD-P1-035 | perspective只夹aspect/near，未统一验证fov、far、finite和near<far | 建`ValidatedProjection`，构造时检查单位、范围、finite、depth mode和reversed/infinite-Z policy |
| COORD-P1-036 | `Real::EPSILON/f32::EPSILON`被当作几何长度、方向和ray阈值 | 禁止machine epsilon充当domain tolerance；每个算法使用有单位、与尺度相关的policy |
| COORD-P1-037 | 1e-4/1e-6等局部阈值没有world unit、profile或误差模型 | `NumericPolicyRegistry`按geometry/physics/navigation/render域提供abs/relative/ULP/condition规则和版本 |
| COORD-P1-038 | world matrix层级传播不检查compose后finite、condition或深度误差 | 在dirty frontier compose时验证并产出首个坏ancestor/path；稳定场景不增加全树扫描 |
| COORD-P1-039 | 没有near/far-origin、deep hierarchy、extreme scale与rebase资格矩阵 | 建range × scale × depth × velocity × platform/backend测试和benchmark，覆盖current/previous frame |
| COORD-P1-040 | diagnostics没有space/unit/origin/condition/narrowing/fallback上下文 | 统一`NumericFailureEvent`，携owner、operation、values摘要、schema IDs、origin generation、policy和recovery |

## 10. P2：后续能力

| ID | 能力 | 进入条件 |
|---|---|---|
| COORD-P2-001 | fixed-point或严格deterministic numeric profile | Runtime22 replay/determinism合同和真实跨平台需求先成立 |
| COORD-P2-002 | geospatial/ECEF/latitude-longitude/projected coordinates | 产品有GIS需求，并建立与canonical world的明确投影/误差合同 |
| COORD-P2-003 | planetary scale、multi-origin与orbital frame hierarchy | 单origin大世界通过资格后，以独立reference-frame owner实现 |
| COORD-P2-004 | mixed-precision SIMD/SoA优化 | 正确性profile稳定且profile-guided benchmark证明收益 |
| COORD-P2-005 | Editor coordinate/space inspector | typed schema与origin generation可查询后展示，不从字段名猜测 |
| COORD-P2-006 | precision heatmap与camera-distance error overlay | 能从真实CPU/GPU conversion receipt计算误差后接入render debug |
| COORD-P2-007 | origin/rebase timeline debugger | rebase transaction、current/previous origin和subsystem ack完成后提供 |
| COORD-P2-008 | unit-aware Inspector/curve/table显示与批量换算 | canonical unit和schema migration先稳定，显示单位不改存储真值 |
| COORD-P2-009 | legacy asset precision/unit migration tool | format版本与loss policy冻结后支持preview/diff/rollback |
| COORD-P2-010 | numeric fuzz、metamorphic和adversarial corpus | M1 validator/transform API硬切后纳入required CI |
| COORD-P2-011 | shader high-low/double-float utility library | camera-relative仍不足的真实render workload证明需要后实现 |
| COORD-P2-012 | 跨RHI/backend numerical conformance dashboard | RHI authority与GPU evidence链完成后聚合，不用静态shader检查冒充结果 |

## 11. 目标架构

### 11.1 Schema与类型

```text
BuildSet
  -> PrecisionProfileId
  -> CoordinateSystemId
  -> UnitSystemId
  -> NumericPolicyId

Canonical World
  WorldPosition(cell/absolute high precision)
  LocalPosition / Vector / UnitDirection / Normal
  LocalTransform(validated TRS) or LocalAffine(explicit)
  WorldOriginState(anchor, generation)
```

`WorldPosition - WorldPosition`得到有单位Vector；`WorldPosition + Vector`得到WorldPosition；两个position不能相加。Direction构造保证有限且单位长度；Normal不能用普通vector transform。类型可以采用newtype、generic marker或受控DTO，但跨crate/ABI必须有稳定schema而不是暴露Rust generic布局。

### 11.2 Frame数据流

```text
Canonical World Snapshot
  -> WorldCoordinateAuthority seals origin generation
  -> subsystem adapters build physics/nav/audio/AI snapshots
  -> per-view RenderOrigin selects anchor
  -> relative current/previous transforms + bounds
  -> checked narrowing + NarrowingReceipt
  -> GPU Scene / SceneUniform / shader relative-space ABI
  -> optional absolute reconstruction helper
```

RenderOrigin可以与simulation WorldOrigin不同：前者按view选择，后者用于需要整体rebase的backend。两者必须带各自generation和转换receipt，禁止共享一个可变全局`Vec3`。多view不得为了方便修改canonical world；每个view只构建相对projection。

### 11.3 Rebase transaction

1. Prepare：选择新anchor，冻结tick/frame boundary，枚举必须ack的subsystem和history owner。
2. Validate：确认physics/nav/backend能力、pending async query、particle/audio/history策略和预算。
3. Apply：更新origin generation；需要native shift的owner应用同一delta；canonical world保持不丢精度。
4. Publish：current/previous origin、changed ranges与typed receipt原子可见。
5. Recover：任一required owner失败则不发布新generation，或进入明确degraded/fatal状态，不能部分系统已shift。

### 11.4 Transform validation

目标不是每次矩阵运算都panic，而是把失败点前移。Asset/import/editor/property/script路径统一构造`ValidatedTransform`；unit quaternion、finite、scale和policy在写入前验证。世界compose可返回`WorldTransformStatus::{Valid, IllConditioned, Invalid}`和首个ancestor；inverse/look-at/projection全部是fallible constructor。render/physics可以对invalid object选择skip/disable，但必须产生一次有owner/generation的diagnostic，不能用identity/zero静默伪造成功。

## 12. Hard Cutover与迁移规则

1. 先建立schema/profile和静态negative tests，再改存储类型；禁止先把`Real`替换为f64后逐个修编译错误。
2. `ReflectedValue`、asset、scene、plugin与wire格式必须有显式旧版migration；不允许serde默认把旧f32文档伪装成新profile。
3. 所有world-to-render转换收敛到一个extract adapter；删除调用方私有`as f32`和absolute GPU upload，不长期保留双路径。
4. relative位置没有origin generation即为无效；debug build和boundary validator必须拒绝跨generation组合。
5. physics/nav/audio/particles等未实现rebase ack前，large-world profile必须fail-close，不以“不移动到那么远”作为能力声明。
6. negative scale、shear和near-singular transform按component/backend capability裁决；不能全局强行禁止，也不能默认全支持。
7. Editor显示单位、gizmo snapping与asset source unit只是view/authoring policy，不得改写canonical runtime单位含义。
8. 旧架构文档中“f64主要只改math alias/helper”的表述必须在M0修订，保留历史背景但禁止继续作为验收结论。

## 13. 里程碑

| 里程碑 | 内容 | 退出条件 |
|---|---|---|
| M0 Truth Freeze | 冻结current source，生成precision/space/unit/narrowing/inverse inventory，修订架构声明 | 所有跨域numeric字段有owner/space/unit/storage分类；报告与fingerprint绑定 |
| M1 Schema与Validator | Coordinate/Unit/Precision/Numeric schema，UnitQuat/Direction/ValidatedTransform，fallible math API | invalid quat/scale/look-at/projection/inverse先红后绿；旧asset migration可回滚 |
| M2 CPU World Precision | 引入WorldPosition和canonical storage，scene/asset/reflection/script/plugin boundary迁移 | 近原点行为兼容；远场position/velocity/hierarchy误差达到预算；无无标记relative值 |
| M3 Subsystem Adapters | physics/nav/audio/AI/particles/picking接typed snapshot与origin ack | 每个required owner有space/unit/generation验证、failure和shutdown/reload测试 |
| M4 Render-relative | per-view origin、relative current/previous transform/bounds/uniform/shader helper | rebase不制造全屏velocity；多view不同origin正确；absolute upload static guard归零 |
| M5 Product Workflow | Editor unit/coordinate显示、import conversion receipt、migration/diagnostics | glTF/OBJ轴/单位/tangent/animation/collider parity；authoring save/reopen无损 |
| M6 Qualification | range/scale/depth/rebase、backend、platform、performance与soak | E4/E5证据绑定BuildSet/profile/schema；超预算或unsupported profile fail-close |

## 14. 验证矩阵

| 层 | 必须验证 | 失败标准 |
|---|---|---|
| Unit/property | UnitQuat、Dir3、WorldPosition运算、checked narrowing、try inverse/look-at/projection | NaN/Inf/zero/collinear/near-singular/overflow被静默接受 |
| Serialization | f32旧scene到新profile迁移、unknown schema、roundtrip、cache/replay identity | 数据静默截断、profile不匹配仍加载或opaque字段丢失 |
| Import | RH/LH、Y/Z up、unit scale、winding、tangent、skin/animation/collider | visual mesh正确但collision/animation轴错或mirror未记录 |
| Hierarchy | 深链、non-uniform/negative scale、affine/shear、dirty frontier | 单点变化全树扫描，或TRS分解静默丢shear且无quality |
| Large world | 1m到目标最大range的静态/运动/交互、跨cell、连续rebase | 抖动、穿透、query mismatch、audio跳变、particle/trail断裂 |
| Temporal/render | camera/object移动、rebase、cut、多view、current/previous origin | 全屏错误速度、history ghost、shadow/visibility/bounds空间不一致 |
| Backend | DX12/Vulkan/Metal目标lane、Jolt/nav/audio adapter能力 | backend不支持却报告large-world ready，或转换failure被identity替代 |
| Performance | CPU double/cell算术、hierarchy、extract、culling、upload bytes | 无baseline声称优于参考引擎，或relative conversion重复per-pass全场执行 |
| Soak/replay | 长时间移动与多次rebase、save/load、record/replay | origin generation泄漏、累计漂移、跨运行digest不可解释分叉 |

## 15. 实施约束与最终判断

- 不把所有数值都升级为f64。mesh vertex、GPU buffer、颜色、UV和多数DSP/render参数可继续f32，但必须在有语义的边界转换。
- 不把large world简化成“相机超过阈值就减去一段坐标”。需要canonical position、origin generation、current/previous correction、subsystem transaction和持久格式共同成立。
- 不机械把634个`EPSILON`命中替换成统一常量。先识别算法、单位、尺度和误差模型，Tooling22管理placement，本篇管理numeric semantics。
- 不在Runtime05重复实现第二套transform hierarchy。WorldCoordinate/ValidatedTransform必须成为其derived-state pipeline的输入和状态，而不是旁路cache。
- 不以参考引擎代码形状作为验收。性能和表现要优于Unreal，必须由相同world range、entity/view规模、误差预算、frame time、memory和artifact evidence证明。

最终判断：Zircon已经完成了“共享数学类型与基础scene transform校验”的重要收敛，但尚未完成工程引擎所需的coordinate/unit/precision contract，更没有large-world runtime/render闭环。现状可以支撑近原点、单precision的功能开发，不能据此宣称f64-ready或大型开放世界ready。正确顺序是先固化schema和fallible numeric boundary，再迁移CPU world representation，随后接subsystem adapter与render-relative current/previous pipeline，最后用产品范围和误差预算验收。
