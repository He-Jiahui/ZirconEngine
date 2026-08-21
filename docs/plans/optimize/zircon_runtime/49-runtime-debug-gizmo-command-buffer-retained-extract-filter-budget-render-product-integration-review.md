---
title: Runtime Debug Gizmo Command Buffer、Retained Asset、Extract、View Filter、Budget、Render 与 Product Integration 工程化差距
category: zircon_runtime
report_id: Runtime49
review_date: 2026-08-19
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_runtime/src/core/framework/gizmos
  - zircon_runtime/src/core/framework/render/overlay.rs
  - zircon_runtime/src/core/framework/navigation/gizmo.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/scene_gizmo_pass
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/passes/line_pass.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/viewport_overlay_renderer/construct/create_line_pipeline.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/overlay/shaders/line.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/buffers/build_line_buffer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/primitives/scene_gizmo
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/build_runtime_frame.rs
  - zircon_runtime/src/dynamic_api/session/extract_stats.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_plugins/navigation/editor/src/overlay.rs
  - zircon_plugins/ai/editor/src/overlay.rs
tests:
  - zircon_runtime/src/tests/gizmos
  - zircon_runtime/src/core/framework/navigation/tests.rs
  - zircon_runtime/src/graphics/tests/render_product_ui.rs
  - zircon_editor/src/scene/viewport/pointer/tests.rs
  - zircon_editor/src/tests/editing/viewport.rs
  - zircon_plugins/navigation/editor/src/tests.rs
  - zircon_plugins/ai/editor/src/tests.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/23-coordinate-space-unit-precision-transform-numeric-robustness-large-world-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/47-runtime-picking-pointer-ray-hit-hover-drag-drop-event-backend-product-integration-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_plugins/14-first-party-navigation-source-native-runtime-editor-dist-catalog-recast-detour-crowd-tilecache-query-bake-product-integration-review.md
  - docs/plans/optimize/zircon_plugins/15-first-party-ai-source-runtime-editor-dist-catalog-behavior-tree-blackboard-perception-eqs-product-integration-review.md
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/DrawDebugHelpers.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/DrawDebugHelpers.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Components/LineBatchComponent.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Components/LineBatchComponent.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Classes/Debug/DebugDrawService.h
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Private/Debug/DebugDrawService.cpp
  - dev/bevy/crates/bevy_gizmos/src/config.rs
  - dev/bevy/crates/bevy_gizmos/src/gizmos.rs
  - dev/bevy/crates/bevy_gizmos/src/retained.rs
  - dev/bevy/crates/bevy_gizmos_render/src/lib.rs
  - dev/bevy/crates/bevy_gizmos_render/src/retained.rs
  - dev/bevy/crates/bevy_gizmos_render/src/pipeline_3d.rs
  - dev/godot/editor/scene/3d/node_3d_editor_gizmos.h
  - dev/godot/editor/scene/3d/node_3d_editor_gizmos.cpp
  - dev/godot/editor/scene/3d/node_3d_editor_viewport.cpp
  - dev/Fyrox/fyrox-impl/src/renderer/debug_renderer.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Volume/VolumeGizmoDrawer.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugDisplaySettingsUI.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 49 · Runtime Debug Gizmo Command Buffer、Retained Asset、Extract、View Filter、Budget、Render 与 Product Integration 工程化差距

## 1. 结论

`zircon_runtime::core::framework::gizmos` 不是空文件。六个 production 文件已经提供九类 typed command、immediate buffer、颜色策略、retained data、CPU shape tessellation，以及到 `SceneGizmoOverlayExtract` 的转换；十个 dedicated test 覆盖记录顺序、禁用、默认配置、retained copy、颜色覆盖、append、基础形状数量与单次 transform matrix 计算。最终 renderer 也能把 line、wire shape 和 icon 送入 WGPU，并有一项像素测试证明密集 scene overlay 不应覆盖 terminal runtime UI。这些底座应保留其 typed math、稳定 command 顺序和最小 CPU reference，而不是退回散落的匿名 `[f32; N]`。

但该子系统目前不是产品使用的 debug-draw service。对非 `dev/docs` 源码反查后，`GizmoBuffer`、`GizmoAsset`、`RetainedGizmo`、`extract_gizmo_overlay` 与 `append_gizmo_overlay` 的 production consumer 全部为 **0**；Editor camera/light、Runtime Navigation、Navigation plugin、AI plugin和Virtual Geometry均绕过它，直接构造 `SceneGizmoOverlayExtract` 与 `OverlayLineSegment`。六文件 API 当前主要由自己的十个测试自证，不能代表普通项目、Editor插件或renderer已接入。

配置合同也存在明确的 false surface。`GizmoConfig` 声明 group、2 px line width、depth bias、render layer、color policy与screen scale，但 extract 只读取 `enabled/color_policy`；其余字段没有进入 overlay DTO、pipeline key、uniform或GPU。最终 shader是固定 `LineList`，深度恒为 `LessEqual` 且bias为零。`selected` 虽进入 `SceneGizmoOverlayExtract`，renderer也不读取。`GizmoAsset::from_buffer`只复制commands并丢弃源buffer config，`RetainedGizmo`随后恢复默认配置；所谓retained又只是每实例内嵌一份可clone的command Vec，没有asset handle、generation、owner lease、TTL或销毁协议。

几何正确性同样没有闭合。retained transform只正确应用于point-based line/ray/strip与显式matrix rect/cube；Axis只变换origin而不变换direction，Sphere/Circle不缩放radius，AABB只变换min/max再重建axis-aligned corners，旋转或非均匀缩放下结果错误。normal、radius、size、color和坐标没有finite/范围校验，固定32段圆与全CPU展开也没有屏幕误差、bounds、frustum、LOD或预算。

性能主路径已经由 `PERF-MVP-333` 记录为P0：稳定overlay仍重建Vec与GPU buffer，每个icon单独buffer/draw，多个LoadStore pass重复访问attachments。Runtime49不重复计该P0，也不重复Runtime09A/09B的RHI/renderer、Runtime23的空间、Runtime24的qualified identity、Runtime47的picking、Editor03的transform gizmo、Runtime08D/08F与Plugins14/15的Navigation/AI payload问题。本篇只拥有通用 debug gizmo service 的producer/config/lifetime/geometry/extract/budget/diagnostic/product纵向合同，登记 **0项新增P0、56项P1和14项P2**。

本轮只做静态review，没有修改production、tests、Cargo或reference source；没有运行Cargo、Editor、WGPU、RenderDoc、GPU capture、soak或benchmark。本文不能作为Zircon在表现或性能上达到、超过Unreal的证据。

## 2. 审查边界、规模与 currentness

### 2.1 物理冻结

| 范围 | 文件 / 行 / bytes / tests | 结论 |
|---|---:|---|
| `core/framework/gizmos` | 6 / 798 / 20,360 / 0 | 全部逐文件读取；目标目录clean |
| dedicated gizmo tests | 1 / 207 / 6,933 / 10 | 全部逐测试读取；未执行 |
| overlay DTO与WGPU line提交 | 7个focused文件 | 读取extract DTO、prepare/record、line vertex、pipeline、shader与GPU buffer创建 |
| 产品producer | 4个focused文件 | Editor camera/light、Navigation、AI、Virtual Geometry与stats反查 |
| focused fingerprint集合 | 18 / 2,715 / 82,535 / 12 | SHA-256 `bac42e7125ae5e46246b12d50730f8988c2de1bc7a3377dd7748ab964891ae27` |

focused fingerprint按相对路径小写排序，将每项编码为 `path + NUL + per-file SHA-256`，以LF连接后再次计算SHA-256。它标识本次读取集合，不是artifact、ABI或release identity。基线HEAD为 `bea1acf91b909525ab1759e2c800858b0eda6528`，gizmos目录最近可定位提交为 `322a03acfec7c8527cec593a4165af3ae31437b5`（2026-08-01）。

冻结时 `zircon_runtime/src/core/framework/render/overlay.rs` 与 `zircon_editor/src/scene/viewport/render_packet.rs` 已有其他会话/用户改动；两个文件均按当前内容读取并进入fingerprint，但本报告不覆盖、暂存或回退。相邻 selection/wireframe primitive也有共享改动，不在本篇写入范围。由于产品DTO与Editor producer不是clean baseline，`source_recheck_required`保持true。

### 2.2 通用API与真实产品路径

```text
公开但无production caller：
  GizmoBuffer / GizmoAsset / RetainedGizmo
    -> extract_gizmo_overlay()
    -> SceneGizmoOverlayExtract
    -> RenderOverlayExtract.scene_gizmos

当前真实producer：
  Editor camera/light -----------+
  Runtime Navigation -----------+|
  Navigation editor plugin -----+|-> 直接构造 SceneGizmoOverlayExtract
  AI editor plugin -------------+|-> 直接追加 OverlayLineSegment/PickShape
  Virtual Geometry debug -------+

Renderer：
  scene_gizmos
    -> CPU展开 line/wire/icon fallback
    -> 每帧 create_buffer_init
    -> 固定 LineList + LessEqual depth
    -> SceneGizmoPass Load/Store
```

这不是“helper尚未推广”的小问题。每个producer已开始建立自己的分段数、颜色、可选项、pick shape、生命周期与重建策略：AI使用24段圆，generic使用32段；Navigation按每个triangle输出三条边并复制共享边；Virtual Geometry另有box/cross builder；Editor camera/light另有wire/icon路径。继续增加领域会形成多套几何、预算、失效、测试与GPU路径。

### 2.3 当前可保留底座

- command使用`Vec2/Vec3/Vec4/Transform/Real`，不是无类型JSON或裸byte stream；
- `GizmoBuffer`保持插入顺序，disabled时不积累command，clear行为显式；
- shape tessellation为流式写入，单个Transform matrix不会逐点重建；
- color override/multiply在CPU reference中有明确语义；
- overlay DTO把可视line/wire/icon与pick shape分开，允许后续共享identity而不强制共用geometry；
- final overlay pass关闭depth write，并位于scene、selection、wire、grid之后和handle之前；
- runtime UI像素测试至少保护“scene debug overlay不覆盖最终UI”的composition底线。

这些基础只能作为迁移输入，不能证明配置、retained、产品接线或scale已经完成。

## 3. 关键代码事实

### 3.1 配置只消费两项

`GizmoConfig`的七组字段中，`GizmoBuffer::push_command`读取`enabled`，extract再次读取`enabled`并应用`color_policy`。`group`、`line.width`、`depth_bias`、`render_layer`与`screen_scale_policy`没有进入`SceneGizmoOverlayExtract`。line shader输入只有position/color；pipeline只有单一LineList variant、固定LessEqual和零DepthBiasState。

默认测试名为`gizmo_config_defaults_cover_m3_rendering_policy`，但只断言结构体默认值，并未证明任何rendering policy被消费。这是典型的descriptor presence false green。

### 3.2 Retained只有名称，没有retention owner

`GizmoAsset::from_buffer`把commands clone进新Vec，不保存buffer config。`RetainedGizmo`又按值持有整个asset、transform与config；clone一个instance会复制全部command。仓库没有`AssetId/Handle`、共享immutable geometry、revision、dirty range、GPU residency、owner world、entity generation、module lease、expiration、remove token或detach。它既不是Bevy式asset+component retained，也不是Unreal式world line batcher/persistent batcher。

### 3.3 Transform语义按primitive分裂

| Command | 当前retained transform | 风险 |
|---|---|---|
| Line/Ray/LineStrip | 每个point乘matrix | 基本正确；仍无finite/space检查 |
| Rect/Cube | 外层matrix乘command matrix | 可表达旋转缩放 |
| Circle | center乘matrix、normal乘vector matrix、radius不变 | uniform scale也丢失；non-uniform scale应为ellipse |
| Sphere | 只变换center | scale完全丢失 |
| Aabb | 只变换min与max，再生成AABB | 旋转/shear/负scale会错误 |
| Axis | 只变换origin，direction使用全局X/Y/Z | retained rotation/scale无效 |

Circle normal在非均匀scale下也没有inverse-transpose或明确的“保持圆/变椭圆”政策。NaN normal不会可靠进入zero fallback；NaN radius因`radius <= 0`为false，会继续生成NaN vertices。

### 3.4 Renderer是逐帧临时提交

`SceneGizmoPass::prepare`遍历每个icon，逐个创建vertex buffer与draw record；全部line/wire/icon fallback先展开为`Vec<LineVertex>`，随后`build_line_buffer`每帧调用`device.create_buffer_init`。vertex count用`vertices.len() as u32`截断，没有checked conversion。没有persistent arena、dirty upload、range table、generation cache、per-view culling或upload receipt。

scene gizmo还拥有独立Load/Store pass；grid、selection、wireframe与handle分别使用同类line pass。`PERF-MVP-333`已经要求把稳定overlay编译为generation artifact、用persistent arena做dirty upload并合并overlay pass；本篇只补Gizmo侧输入、生命周期、预算和资格合同。

### 3.5 当前diagnostics只估计DTO payload

dynamic extract stats只对`scene_gizmos`及四个Vec调用`slice_bytes`。它不报告capacity slack、producer、group、commands、expanded vertices、culled/dropped数、CPU tessellation时间、GPU upload bytes、buffer create、draw/pass数、last-good generation或budget exhaustion。错误输入、超预算或空产品consumer都没有terminal status。

## 4. 与参考引擎的可迁移差异

| 参考 | 已核对能力 | Zircon应吸收的合同 | 不照搬项 |
|---|---|---|---|
| Unreal | DrawDebug helper带world、persistent、lifetime、depth priority、thickness；LineBatchComponent有BatchID、tick expiry、flush、bounds；DebugDrawService命名注册/注销delegate | world/session owner、即时/定时/持久寿命、批次撤销、深度/粗细、bounds、registration lease | 不复制全局宏、UObject或其线程模型 |
| Bevy | typed GizmoConfigGroup/Store；width/perspective/style/joint/depth bias/RenderLayers真正进入extract/uniform/render phase；retained为asset handle + ECS component | typed category config、per-camera layer、immediate frame storage、retained handle/generation、GPU thick-line pipeline | 不把TypeId作为跨DLL稳定identity，也不要求照搬ECS API |
| Godot | EditorNode3DGizmoPlugin有priority、hide/on-top/select policy、register/unregister；可视mesh、collision segments/triangles、handles分离并维护BVH bounds | plugin contribution lifecycle、visual/pick分离、bounds/spatial query、hidden/selected/on-top policy | 不把Editor Node/Object继承树搬进Runtime |
| Fyrox | DebugRenderer复用DynamicDraw GPU geometry buffer，`set_lines`只更新数据并产出pass statistics | 至少复用GPU owner、显式更新与统计，不能每帧创建新buffer作为唯一实现 | 其单Vec/单pass只是最低基线，不是Zircon最终性能目标 |
| Unity Graphics | `DrawGizmo`按Active/Selected/NonSelected过滤；Volume gizmo区分wire/solid并应用matrix；Debug settings UI注册、延迟初始化、reset、unregister和foldout persistence | selection/view filter、wire/solid、matrix语义、可发现设置与对称注销 | 本地Graphics镜像不是完整Unity runtime/editor，不能外推其全部Gizmo系统 |

共同规律是：debug draw不是“把一些线塞进Vec”。成熟实现至少有producer/category、scope、lifetime、view filtering、geometry/pick分离、render policy、资源复用、注销和可观测性。Zircon可以设计更低开销的typed/compiled方案，但不能删掉这些语义来宣称更快。

## 5. Owner边界

| Owner | 保留职责 | Runtime49只交接什么 |
|---|---|---|
| Runtime09A | RHI resource、GPU fence、device generation、buffer retirement、render graph pass | prepared gizmo artifact与预算，不另建RHI |
| Runtime09B | RenderScene、visibility、per-view extract、GPU Scene与renderer diagnostics | gizmo bounds/view mask/overlay range输入 |
| Runtime23 | coordinate space、transform、large-world narrowing | command必须声明space并走validated transform |
| Runtime24 | qualified owner/generation/stale reference | retained handle与overlay owner使用其identity |
| Runtime47 | pointer/pick backend、capture、hover/drag事件 | visual artifact发布同代pick geometry，不重建pointer系统 |
| Editor03 | selection、transform handle、transaction、tool mode | 消费runtime gizmo descriptor/artifact，不由Runtime拥有编辑行为 |
| Runtime08D/08F、Plugins14/15 | Navigation/AI debug source、domain LOD、reader gating | 发布bounded typed source，不复制通用line builder |
| PERF-MVP-333 | generation compiled overlay、persistent GPU arena、pass合并与scale计数 | Runtime49定义Gizmo专属失效、budget与parity输入 |
| Runtime49 | producer registry、category config、immediate/retained lifetime、geometry compiler、view filter、budget、diagnostics、product qualification | 不拥有领域simulation、Editor transaction或底层RHI |

## 6. P1差距清单

| ID | 已证实差距 | 目标合同 / 验收方向 |
|---|---|---|
| GIZMO-P1-001 | 通用Gizmo API的production caller为0 | 至少一个Runtime产品producer和一个Editor/plugin producer走canonical service；否则标记experimental/unavailable |
| GIZMO-P1-002 | 六类真实producer直接构造overlay DTO | 禁止领域复制tessellation/lifetime/budget；通过typed contribution编译到共同artifact |
| GIZMO-P1-003 | config七组字段只消费enabled/color | 每个公开字段必须有consumer、diagnostic与像素/状态测试，或删除字段 |
| GIZMO-P1-004 | group只是buffer内raw String | 建立qualified category descriptor、注册冲突、owner、default与settings schema |
| GIZMO-P1-005 | `GizmoAsset::from_buffer`丢失源config | 明确asset只含geometry还是连同style；转换不得静默重置可见语义 |
| GIZMO-P1-006 | retained实例按值clone全部commands | asset handle + immutable generation + instance transform/style，静态geometry单owner |
| GIZMO-P1-007 | immediate buffer无frame/schedule生命周期 | frame writer在确定barrier seal，下一帧自动清空；late writer返回receipt |
| GIZMO-P1-008 | retained无TTL、remove token或owner detach | 支持frame/time/persistent lifetime、显式remove、owner teardown与last-use retirement |
| GIZMO-P1-009 | retained无world/session scope | key包含runtime session/world/view domain，world unload后不可泄漏 |
| GIZMO-P1-010 | plugin/module贡献无lease与quiescence | unregister先停止admission、drain writer/artifact引用，再卸载代码与资源 |
| GIZMO-P1-011 | `SceneGizmoKind`硬编码Camera/Light/VG/Nav/AI | 改为qualified kind key + registered descriptor；新增plugin不改Runtime core enum |
| GIZMO-P1-012 | owner只有裸EntityId | 使用world/entity generation qualified identity，拒绝stale overlay与pick result |
| GIZMO-P1-013 | extract无view/camera/eye identity | 编译目标显式携带view family、camera、eye、viewport和frame generation |
| GIZMO-P1-014 | 没有统一config store或profile | project/user/session/view override分层，变更生成新config generation |
| GIZMO-P1-015 | render_layer声明但未消费 | 与camera/view layer求交后才生成/上传；layer改变精确失效 |
| GIZMO-P1-016 | screen scale policy声明但未消费 | world/pixel/constant-screen-size语义进入per-view compiler与测试 |
| GIZMO-P1-017 | 2 px line width声明但GPU恒为LineList | thick-line geometry/shader真正消费width，并定义DPI/MSAA/near-plane行为 |
| GIZMO-P1-018 | depth_bias声明但pipeline恒零bias | 定义depth-tested/xray/on-top与bias范围，进入pipeline/uniform而非死字段 |
| GIZMO-P1-019 | selected字段renderer不读取 | selection style由明确policy消费，或从render DTO删除并交由producer定色 |
| GIZMO-P1-020 | color policy在CPU flatten，后续配置变化要求重建全部line | style与geometry分代，纯颜色/visibility变化可复用geometry artifact |
| GIZMO-P1-021 | 没有occluded/xray/foreground双通道 | 支持可审计depth mode及pass order，避免所有debug线只能被场景遮挡 |
| GIZMO-P1-022 | 无dash/dot/joint/cap/anti-alias policy | 建立line style与quality tier，GPU实现和fallback parity |
| GIZMO-P1-023 | primitive仅九类且缺arrowhead/cone/capsule/frustum/grid/polyline loop | 按实际physics/nav/AI/render debug需求建立可扩展primitive compiler |
| GIZMO-P1-024 | 无2D、screen-space、text/label、arc或image primitive | 明确2D/3D/view/screen域，文字走Text artifact而非私有临时字体路径 |
| GIZMO-P1-025 | generic command不能生成wire/icon/fill/pick geometry | visual与pick共享identity/bounds，但允许独立representation与budget |
| GIZMO-P1-026 | position/vector/color/transform无finite检查 | producer admission或compiler拒绝NaN/Inf并按source计数，不向GPU传播 |
| GIZMO-P1-027 | radius、size、normal、line length无范围/退化政策 | typed validation定义zero/negative/huge/degenerate的drop或error语义 |
| GIZMO-P1-028 | retained Axis只变换origin | axis direction按声明space变换并对scale policy做显式裁决 |
| GIZMO-P1-029 | retained Sphere/Circle不缩放radius | uniform/non-uniform/negative scale分别定义sphere/ellipsoid/circle/ellipse语义 |
| GIZMO-P1-030 | retained AABB只变换min/max | 变换八角或编译为OBB；旋转、shear、negative scale有golden |
| GIZMO-P1-031 | Circle normal对non-uniform scale无正确合同 | 用inverse-transpose/局部plane basis或明确禁止不支持的transform |
| GIZMO-P1-032 | command/retained asset无bounds | 编译并缓存world bounds，接入frustum、spatial selection与BVH |
| GIZMO-P1-033 | 全部gizmo无frustum/distance/occlusion/LOD culling | per-view visibility先于tessellation/upload，domain可提供LOD source |
| GIZMO-P1-034 | Circle/Sphere固定32段，与屏幕尺寸和质量无关 | 以pixel error、quality tier和hard max计算segment，记录clamp |
| GIZMO-P1-035 | stable primitive每帧CPU重新展开 | generation-owned compiled geometry；camera-independent段只在source变化时重建 |
| GIZMO-P1-036 | line vertex每帧`create_buffer_init` | persistent GPU arena/ring + dirty range upload + completion retirement |
| GIZMO-P1-037 | 每个icon单独buffer与draw | atlas instance buffer与batch range；按texture/pipeline合批并受budget约束 |
| GIZMO-P1-038 | scene/grid/selection/wire/handle拆成多个LoadStore pass | 服从PERF-MVP-333合并策略并保留显式子层顺序与pipeline range |
| GIZMO-P1-039 | 无command/primitive/vertex/byte/time预算 | project/profile/category/view四级budget，tessellation前admission |
| GIZMO-P1-040 | LineStrip与serde payload可携任意长度Vec | decode与submit限制items/bytes/depth，拒绝恶意或损坏payload |
| GIZMO-P1-041 | `sum -> reserve`与分配失败没有checked路径 | checked accounting，超限返回typed drop/truncate/error receipt而非panic/OOM |
| GIZMO-P1-042 | GPU vertex count用unchecked `usize as u32` | checked conversion并按draw range拆分；超限不可静默wrap |
| GIZMO-P1-043 | push/extract没有admission或terminal receipt | 每producer获得accepted/dropped/truncated/stale/disabled结果与原因 |
| GIZMO-P1-044 | 无per-category/source diagnostics | 记录commands、expanded vertices、culled、dropped、CPU/GPU bytes/time与generation |
| GIZMO-P1-045 | extract stats只估算DTO slice payload | 加上capacity、compiled artifact、upload、buffer create、draw/pass与retained residency |
| GIZMO-P1-046 | 无capture/export/replay | 可冻结特定frame/view的source/config/artifact摘要，绑定build/schema并可离线重放 |
| GIZMO-P1-047 | API只有调用方私有`&mut GizmoBuffer`，无并行producer模型 | thread-local/chunk writer，barrier确定性merge；禁止全局热锁 |
| GIZMO-P1-048 | merge顺序只取决于调用Vec顺序 | 定义phase/layer/category/owner/stable sequence与overflow tie-break |
| GIZMO-P1-049 | 没有source/config/camera generation失效图 | geometry、style、view-facing、pick与GPU artifact分别按依赖精确失效 |
| GIZMO-P1-050 | Navigation每triangle输出三边并复制共享边 | compiled unique-edge/indexed tile artifact，稳定navmesh不重复O(T)构建 |
| GIZMO-P1-051 | AI、VG与generic各自手写circle/box/cross分段 | 领域发布typed source，复用canonical compiler且保留domain颜色/LOD policy |
| GIZMO-P1-052 | 无multi-camera、stereo、dynamic resolution和large-world方案 | per-view relative transform、eye/layer过滤、resize/rebase generation gate |
| GIZMO-P1-053 | 无headless/shipping/capture权限或编译策略 | BuildSet声明debug draw capability、strip policy、remote principal与数据泄露边界 |
| GIZMO-P1-054 | GizmoAsset/Command可serde但无schema/version/migration | versioned asset schema、unknown command policy、limits、roundtrip与migration |
| GIZMO-P1-055 | Editor只有总开关和领域私有toggle，没有统一category设置面 | 可发现的group tree、search/reset/persist、per-viewport override与状态诊断 |
| GIZMO-P1-056 | dedicated tests未走真实product/GPU路径 | canonical API -> Editor/Runtime producer -> extract -> WGPU pixel/RenderDoc全链资格 |

## 7. P2差距清单

| ID | 差距 | 收敛要求 |
|---|---|---|
| GIZMO-P2-001 | API名`linestrip`不符合仓内常用分词 | 硬切为`line_strip`，不留永久alias |
| GIZMO-P2-002 | width、depth bias、screen scale的单位/范围文档不足 | 文档与schema声明pixel/world/NDC、DPI和合法范围 |
| GIZMO-P2-003 | group ID接受空白、大小写和任意String | canonical qualified ID、长度/字符限制与冲突诊断 |
| GIZMO-P2-004 | `GizmoRenderLayer(pub u32)`为无语义tuple | typed layer mask/key，说明0/default与camera交集语义 |
| GIZMO-P2-005 | color没有linear/sRGB/HDR/premultiply约定 | 统一overlay color contract并做HDR/SDR像素测试 |
| GIZMO-P2-006 | command无source label/callsite/debug tag | 可选interned producer marker，diagnostic/capture可定位来源 |
| GIZMO-P2-007 | request为每次builder持有两个Vec引用 | 支持slice/iterator/small stable list，并以测量决定内联容量 |
| GIZMO-P2-008 | serialized enum/Vec格式没有size估算或compact artifact | source schema与compiled binary分离，避免把Rust enum布局当cook格式 |
| GIZMO-P2-009 | 测试用`include_str!`检查实现文本 | 改为counter/allocator/benchmark行为门，删除源码字符串耦合 |
| GIZMO-P2-010 | 默认配置测试把字段存在误称为rendering policy覆盖 | 分成schema default与consumer tests，命名反映真实证据等级 |
| GIZMO-P2-011 | 无Axis/Sphere/Circle/AABB变换回归测试 | 覆盖rotation、uniform/non-uniform/negative scale、NaN与large coordinate |
| GIZMO-P2-012 | 无serde roundtrip、unknown/version、fuzz与size-limit测试 | 建立fixture corpus和bounded decode property tests |
| GIZMO-P2-013 | 无1/1k/100k primitive、stable/1% changed benchmark | 记录CPU、alloc、RSS、upload、buffer、draw、pass和GPU timestamp分布 |
| GIZMO-P2-014 | 无current-source真实Editor/GPU/多平台证据 | Windows MVP后补WGPU/RenderDoc；Linux/macOS按平台资格，不以adapter缺失算通过 |

## 8. 目标架构

### 8.1 Source、lifetime 与 publication

```text
Module/plugin producer descriptor
  -> GizmoFrameWriter / RetainedGizmoStore
  -> qualified source + owner/world generation + config group
  -> seal barrier and deterministic merge
  -> GizmoSourceSnapshot

GizmoSourceSnapshot + ConfigGeneration + ViewGeneration
  -> validation/admission
  -> bounds + visibility + LOD
  -> canonical primitive compiler
  -> visual artifact + pick artifact + diagnostics receipt
  -> PreparedOverlayArtifact ranges
  -> persistent GPU arena / render phase
```

Immediate writer只活到frame seal；retained store由qualified handle管理，handle包含store/world/owner generation。plugin卸载时先关闭writer和descriptor admission，再等待已发布artifact与callback lease退出。产品不允许保存`&dyn plugin callback`到下一代。

### 8.2 Geometry与style分离

几何generation只依赖source topology和local transform；颜色、粗细、depth mode、visibility等style generation可以重用同一geometry。screen-facing/icon/text或pixel width再依赖view generation。这样稳定帧不做CPU tessellation、不创建GPU buffer；只改变颜色时也不重建全部positions。

### 8.3 Budget与确定性降级

预算至少覆盖source commands、decoded bytes、expanded vertices、retained bytes、per-view visible items、CPU compile time、GPU upload bytes和draw ranges。超限按明确优先级保留selection/active tool/错误诊断，再保留近处或用户pin类别；同一输入必须得到同一截断集合。任何drop都进入receipt和diagnostics，不能假装完整显示。

### 8.4 Product controls

Editor与runtime debug UI消费同一category registry：显示owner、capability、enabled来源、config revision、visible/dropped数、last error与GPU cost。project default、user preference、session override、viewport override按优先级叠加，headless/shipping/profile按BuildSet fail-close。领域插件只贡献descriptor、source与可选settings schema，不复制通用UI、tessellator和GPU owner。

## 9. 分层重构路线

### M0 · Truth freeze

- 将无production consumer的通用API标为experimental，禁止以字段存在宣称M3 rendering policy完成；
- 为现有六类direct producer登记canonical owner、source generation和当前成本；
- 把Runtime49的0/56/14纳入总账，不重复PERF-MVP-333与Runtime09A/09B的P0。

### M1 · Identity、descriptor 与 config

- 建立qualified producer/category/owner/view key；
- 建立typed descriptor/config registry、分层override与对称unregister；
- 删除closed SceneGizmoKind对plugin种类的硬编码依赖。

### M2 · Immediate/retained lifetime

- 实现frame writer、seal barrier、deterministic merge和late-write receipt；
- retained改为handle + immutable asset generation + instance数据；
- 接入world unload、entity retirement、module/plugin quiescence与TTL/remove。

### M3 · Geometry correctness

- 修复Axis/Sphere/Circle/AABB transform；
- 建立finite/degenerate/space/unit validation；
- 补齐常用3D/2D/view primitive、visual/pick共享identity与bounds。

### M4 · Compiler、visibility 与 budget

- source/config/view分代，稳定geometry缓存；
- bounds/frustum/layer/distance/LOD先于tessellation；
- command/byte/vertex/time预算与确定性截断receipt全部可观测。

### M5 · GPU product renderer

- 与PERF-MVP-333合流为persistent arena、dirty upload、icon instancing和overlay pass range；
- 实现thick/dashed/join/depth/xray pipeline与device generation恢复；
- 多camera/stereo/dynamic resolution/large-world通过像素与GPU capture。

### M6 · Domain与Editor cutover

- Editor camera/light、Navigation、AI、VG逐个切到canonical service；
- Editor03保持transform transaction owner，Runtime47保持pick/event owner；
- 建立统一category settings/diagnostic panel与source/native plugin parity。

### M7 · Qualification

- CPU reference、WGPU、Editor、headless、shipping profile、plugin unload、device loss和scale测试进入machine-readable plan；
- artifact绑定HEAD/build/schema/config/workload/GPU adapter；
- correctness、failure、memory、CPU/GPU和产品像素全部通过后才讨论相对引擎性能。

## 10. 资格门

| Gate | 验收内容 |
|---|---|
| GZ01 | canonical Gizmo API至少有一个Runtime与一个Editor/plugin production consumer |
| GZ02 | 全仓不再由领域直接手写通用circle/box/cross tessellation |
| GZ03 | 每个公开config字段有真实consumer与测试，零dead descriptor |
| GZ04 | category/producer identity qualified、可注册、可冲突诊断、可注销 |
| GZ05 | immediate frame seal后late write得到typed结果且不污染下一帧 |
| GZ06 | retained handle支持remove/TTL/world unload/entity generation/module unload |
| GZ07 | plugin unload等待writer/callback/artifact lease清零 |
| GZ08 | stale world/entity/view generation overlay被拒绝 |
| GZ09 | render layer与camera/view layer真实求交 |
| GZ10 | world/pixel/constant-screen-size在perspective/ortho/DPI下符合定义 |
| GZ11 | line width在1/2/8 px与MSAA/无MSAA下像素可验 |
| GZ12 | depth-tested/xray/on-top/bias无z-fight且pass order确定 |
| GZ13 | style、geometry、view-facing generation可独立复用 |
| GZ14 | Axis在retained rotation下方向正确 |
| GZ15 | Sphere/Circle在uniform/non-uniform/negative scale下符合政策 |
| GZ16 | AABB/OBB在rotation/shear下不丢角点或倒置 |
| GZ17 | NaN/Inf/negative/huge输入fail-close且记录source |
| GZ18 | visual与pick artifact同owner/generation但预算可独立 |
| GZ19 | bounds/frustum/layer/distance/LOD发生在CPU展开与GPU上传前 |
| GZ20 | circle/arc segment按pixel error与quality计算且受hard cap |
| GZ21 | stable source/config/view frame的CPU tessellation为0 |
| GZ22 | stable frame GPU buffer create与full upload为0 |
| GZ23 | icon按atlas/pipeline实例化，buffer/draw不随icon一一增长 |
| GZ24 | overlay attachment LoadStore pass满足PERF-MVP-333目标且顺序等价 |
| GZ25 | commands/bytes/vertices/time/residency均有hard budget |
| GZ26 | 超预算集合确定、优先级可审计、dropped receipt非零 |
| GZ27 | usize/u32/byte乘加全部checked，无silent wrap |
| GZ28 | diagnostics按producer/category/view报告source/visible/culled/dropped/upload/draw/time |
| GZ29 | capture可绑定build/schema/config/source generation并离线重放 |
| GZ30 | 1/1k/100k primitive与stable/1% changed有CPU/GPU/RSS分布证据 |
| GZ31 | Navigation稳定mesh复用unique-edge artifact，不每帧3T输出 |
| GZ32 | AI/VG/physics等domain source共享compiler且保持领域LOD/颜色语义 |
| GZ33 | multi-camera/stereo/dynamic resolution/resize/rebase无串view或陈旧artifact |
| GZ34 | headless/shipping/remote debug按BuildSet与principal fail-close |
| GZ35 | Editor category settings可search/reset/persist/per-view override并显示真实status |
| GZ36 | canonical API到WGPU/Editor产品像素、RenderDoc、device loss与unload全链通过 |

## 11. 验证说明

本轮已完成：六个Gizmo production文件与dedicated tests逐行读取；公开符号与全部非reference consumer反查；overlay DTO、WGPU line pipeline/shader/buffer、Editor/Navigation/AI/VG producer和extract stats追踪；Unreal、Bevy、Godot、Fyrox及Unity Graphics对应源码对照；focused fingerprint与共享脏文件冻结。

本轮未完成：Cargo编译/测试、真实Editor窗口、WGPU像素、RenderDoc、GPU timestamp、device loss、multi-view、plugin unload、fault injection、fuzz、scale benchmark与跨平台验证。所有实施状态保持pending，任何历史green结果都不能替代当前HEAD与当前dirty source的重新资格。
