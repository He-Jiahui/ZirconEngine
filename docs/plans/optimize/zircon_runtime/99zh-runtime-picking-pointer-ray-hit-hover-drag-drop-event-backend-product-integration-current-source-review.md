---
title: Runtime Picking / Pointer / Ray / Hit / Hover / Drag-Drop / Event / Backend / Product Integration 当前源码复审
category: zircon_runtime
report_id: Runtime133
review_date: 2026-08-24
baseline_head: 5fcf31956e2f35663b5696313d6f760052773a9e
baseline_epoch: 392
verification_head: f73dd740892f9ecc86e0783b31e4cb8660ef0e75
verification_epoch: 393
supersedes_currentness_of:
  - docs/plans/optimize/zircon_runtime/47-runtime-picking-pointer-ray-hit-hover-drag-drop-event-backend-product-integration-review.md
related_code:
  - zircon_runtime/src/core/framework/picking
  - zircon_runtime/src/tests/picking
  - zircon_runtime/src/core/framework/render/visible_spatial_query.rs
  - zircon_runtime/src/graphics/visibility/spatial_query.rs
  - zircon_runtime/src/core/framework/physics/query_interface.rs
  - zircon_runtime/src/ui
  - zircon_runtime/src/dynamic_api/session/input_events.rs
  - zircon_runtime/src/dynamic_api/session/runtime_ui.rs
  - zircon_runtime_interface/src/ui/picking.rs
  - zircon_app/src/entry/runtime_entry_app/pointer_input
  - zircon_editor/src/scene/viewport/pointer
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_pointer_route.rs
  - zircon_plugins/physics/runtime/src
tests:
  - zircon_runtime/src/tests/picking
  - zircon_editor/src/scene/viewport/pointer/tests.rs
  - zircon_editor/src/scene/viewport/pointer/precision/renderer_visible_spatial_pick_source.rs
  - zircon_editor/src/scene/viewport/pointer/overlay_router/viewport_overlay_pointer_router_visible_spatial_query.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
  - docs/plans/optimize/zircon_runtime/23-coordinate-space-unit-precision-transform-numeric-robustness-large-world-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime/47-runtime-picking-pointer-ray-hit-hover-drag-drop-event-backend-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/77-runtime-ui-input-dispatch-routing-focus-navigation-pointer-capture-gesture-drag-drop-ime-window-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/99r-runtime-input-device-event-frame-state-action-map-focus-gamepad-recording-replay-host-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/59-editor-scene-viewport-interaction-controller-input-picking-selection-highlight-gizmo-transaction-cancel-generation-product-integration-current-source-review.md
  - docs/plans/zircon_editor/editor/05/failure-2026-07-18-viewport-pointer-candidate-regeneration.md
  - docs/plans/performance/01/failure-2026-08-01-picking-repeated-hit-projection-false-green.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/HitProxies.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/SceneHitProxyRendering.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/EditorViewportClient.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorViewportClient.cpp
  - dev/bevy/crates/bevy_picking/src/backend.rs
  - dev/bevy/crates/bevy_picking/src/events.rs
  - dev/bevy/crates/bevy_picking/src/hover.rs
  - dev/bevy/crates/bevy_picking/src/pointer.rs
  - dev/bevy/crates/bevy_picking/src/mesh_picking/mod.rs
  - dev/bevy/crates/bevy_picking/src/mesh_picking/ray_cast/mod.rs
  - dev/godot/scene/main/viewport.cpp
  - dev/godot/scene/3d/camera_3d.cpp
  - dev/godot/scene/3d/physics/collision_object_3d.cpp
  - dev/godot/editor/scene/3d/node_3d_editor_viewport.cpp
  - dev/Fyrox/editor/src/camera/mod.rs
  - dev/Fyrox/fyrox-impl/src/scene/graph/physics/mod.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Editor/Material/BRGPicking.shader
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Editor/ShaderGraph/Includes/SelectionPickingPass.hlsl
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Shaders/BRGPicking.shader
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/ShaderLibrary/PickingSpaceTransforms.hlsl
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Runtime133 · Picking / Pointer / Ray / Hit / Hover / Drag-Drop / Event / Backend / Product Integration 当前源码复审

## 1. 结论

当前 `zircon_runtime::core::framework::picking` 不是空目录：23个生产文件定义了ray map、backend trait、hit排序、hover map、pointer event state、pipeline report、debug feed和一个sphere primitive backend；6个专用测试文件有22个非ignored测试声明。其局部算法也有应保留价值，包括有限类型的hit priority、backend order + depth稳定排序、previous/current hover diff、press/release/click/drag事件生成，以及无需再次查询就能从report投影debug feed。

但它尚未成为产品 Picking 系统。`run_picking_pipeline` 在整个产品源码中没有非测试调用者；`PickingScheduleLabel` 只是report里的枚举值，不对应真实schedule barrier；backend同步、不可失败并返回无界 `Vec`；唯一实现对每条ray线性扫描sphere。pointer、camera、viewport、target均主要是裸 `u64`，result没有frame/view/backend/world generation。Ray builder忽略projection override、viewport rect、DPI、jitter与near/far；resolver按pointer而不是pointer+surface/view聚合，HashMap迭代与output index会参与平局裁决。状态机没有capture、输入edge快照、阈值、时间、click count、target retirement、fault outcome或容量预算。

Editor 当前路径比 Runtime47 记录时更诚实，但不是闭环。旧的 `ViewportPointerDispatch::runtime_input` 及其固定zero delta合成已被硬删除，测试甚至锁定“不得暴露synthetic runtime input”；这消除了一个假桥接，却没有把App/Dynamic input送入runtime Picking。现行链路是 `UiSurface -> renderer-visible ray broad phase -> Editor screen-space Circle/Line/Ring score -> PointerHits -> resolve_picking_outputs -> private ViewportPointerRoute + PickingDebugFeed`。它固定 `PointerId(1)`、camera 0、order 0，只做route/debug，不运行pipeline、不更新 `PickingEventState`、不产生capture/click/drag receipt。renderer snapshot已有world/viewport/frame generation与BVH/grid可见球体宽相，这是重要底座；最终renderable仍按transform translation和经验半径投影成屏幕圆，不能称为精确mesh、instance、masked-material或GPU-ID picking。

Runtime UI已有独立surface routing/capture，physics插件已有ray/shape query，renderer已有可见空间查询；三者都没有注册为framework Picking backend，也没有共同 `ResolvedPickingFrame`。因此这些是可复用substrate，不是能力闭合。模块文档仍声称Editor dispatch携带 `runtime_input`，边界文档仍声称Cancel会映射为runtime `PointerAction::Cancel`，均与当前源码相反。

Runtime47的48项P1本轮重判为 **39 Open、9 Partial、0 Closed**；12项P2为 **10 Open、2 Partial、0 Closed**；36项资格门为 **23 Fail、10 Partial、3 Pass**。本轮不新增P0，也不覆盖Editor59已拥有的3项P0。`viewport-pointer-candidate-regeneration` 与 `picking-repeated-hit-projection-false-green` 两个failure继续保持open；后者没有真实完成focused Cargo证据，不能以静态源码替代。

## 2. 审查边界、方法与currentness

### 2.1 冻结物理范围

统计口径：物理UTF-8行、非空行、文件bytes；test declaration匹配 `#[test]`，ignored匹配 `#[ignore`；fingerprint为按normalized lowercase relative path排序后的 `path + NUL + lowercase(file SHA-256) + LF` 集合再做SHA-256。相邻产品集包含Editor viewport pointer完整目录，并沿App ingress、Dynamic UI、Runtime UI、renderer visible query、physics query和Editor controller追踪实际连接。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | fingerprint |
|---|---:|---|
| Picking完整owner | **23 / 2,069 / 1,873 / 61,154 / 0 / 0** | `1e8c3fdb85e8b4913984d715df669667ea620cbd525967443a66f1fba4f475a8` |
| Picking专用测试 | **6 / 889 / 811 / 29,809 / 22 / 0** | `391f1ecb75a2c6bd3eb7036b77833e7e4c72feb082849d8b7d0237445448977c` |
| 相邻产品与owner边界 | **61 / 4,925 / 4,537 / 175,563 / 28 / 0** | `c998fc48e02d9482ecaf171160058e577773cf18f9b8692759f235480d1be77e` |
| 五引擎参考选择集 | **20 / 32,732 / 27,975 / 1,226,410 / 5 / 0** | `b2a75ce5298e099d2306ed66f47889f3fc4d90d4f5df73737f207a00f8899d76` |

参考仓库revision：Bevy `fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`、Godot `8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、Fyrox `8d815db36494f1badb347547dfc7094bf4fbbdf8`、Unity Graphics `a7e4c051d256a781ab362c64316b125a1e104694`。`dev/UnrealEngine` 没有独立Git元数据，`git -C` 会向上解析到Zircon workspace；因此Unreal及整套参考证据以所列20个物理文件和集合fingerprint冻结，不伪造独立revision。

### 2.2 检查方法

1. 逐文件读取23个Picking owner文件和6个专用测试文件，复核所有公开类型、构造器、排序、状态转移、clear路径、report/debug投影与primitive backend。
2. 对 `run_picking_pipeline`、`resolve_picking_outputs`、`PointerHits::new`、`PickingScheduleLabel` 做全产品caller反查；前者仍只有定义/re-export/test，Editor只使用后两者。
3. 逐文件读取Editor viewport pointer的41个Rust文件及controller route，确认renderer-visible source、UI surface、候选构造、精度shape、route/debug和测试事实。
4. 沿App winit pointer、Dynamic session input/UI、Runtime UI capture、renderer visible spatial snapshot及physics query/plugin backend检查能否形成同一frame authority。
5. 对Runtime47的48项P1、12项P2与36项gate原编号逐项重判；不关闭Runtime12/23/24、Runtime77、Render04、Editor03/59或Performance01的父owner。
6. 对照Unreal hit proxy/editor tracking、Bevy picking backend/event/mesh ray cast、Godot viewport physics capture、Fyrox editor precise/coarse picking与physics query、Unity Graphics selection-ID pass的物理源码。

### 2.3 currentness与共享工作树

- baseline为 `5fcf31956e2f35663b5696313d6f760052773a9e` / epoch 392；共享 `main` 在注册时已有3,272个未accept变化并标记degraded。
- 最终verification时共享 `main` 已前进到 `f73dd740892f9ecc86e0783b31e4cb8660ef0e75` / epoch 393（`fix(coordinator): isolate finalize temp files`）；Picking owner、专用测试和61个相邻产品文件的三组fingerprint均未变化，结论仍对应最终物理源码。
- Picking 23个owner文件与6个专用测试相对Runtime47审查baseline没有源码差异；Editor pointer/controller有21个文件变化，主要是删除synthetic runtime input、引入typed UI error、render-published interaction extract与renderer-visible spatial query。
- Editor viewport树、Dynamic UI、renderer/physics相邻边界正在被其他会话修改。本文记录最终重扫时点的物理fingerprint，不回退、不覆盖任何外部修改，也不承诺共享工作树随后停止变化。
- 本轮是review-only，只写本报告与索引；不修改Rust/Cargo，不运行Cargo、Editor、GPU capture或产品交互测试，不提交commit。

## 3. 当前拓扑、可达性与断路

```text
winit pointer event
  -> ZrRuntimeEventV1
  -> Dynamic InputState / RuntimeUiState (独立UI surface capture)
  -X-> PickingFrameCoordinator                         [不存在]

Editor viewport pointer
  -> private UiSurface hit stack
  -> renderer-visible spatial query (visible sphere broad phase)
  -> Editor Circle / Line / Ring screen-space scoring
  -> synthetic PointerHits(pointer=1, camera=0, order=0)
  -> resolve_picking_outputs
  -> private ViewportPointerRoute + PickingDebugFeed
  -X-> run_picking_pipeline / PickingEventState / selection receipt

Physics query + renderer visibility + Runtime UI picking policy
  -X-> PickingBackendRegistry                          [不存在]
```

| 层 | 当前事实 | 工程缺口 |
|---|---|---|
| frame owner | `run_picking_pipeline` 是同步plain function，临时创建五个stage report | 无per-world/per-surface owner、frame stamp、workspace、schedule barrier或publication receipt |
| ingress | App和Dynamic session有mouse/touch/wheel/focus事件；Editor synthetic Picking input已删除 | 无ordered `PickingInputBatch`、UI arbitration结果、pointer generation与edge-time query绑定 |
| view/ray | core从camera transform/FOV/ortho size生成ray；Editor另有projection context | core忽略override/rect/DPI/jitter/near-far；两套投影真值没有contract |
| backend | sphere backend与physics/render/UI独立查询均存在 | 无registry、capability admission、typed fault、async ticket、deadline或backend generation |
| resolve | target priority、group order、depth可排序，Editor route/debug共用一次resolve | target enum封闭；无qualified view/frame/source、dedupe/merge policy和invalid-hit admission |
| interaction | hover diff及基础press/release/click/drag事件存在 | 无capture、threshold/time/count、edge snapshot、hierarchy propagation、retirement或bounded event queue |
| product | Editor能返回private route与debug feed | 无runtime event/selection/tool receipt，无App/gameplay caller，无完整frame authority |

## 4. 必须保留的工程基础

1. 保留Picking按Ray、Backend、Resolve、Hover、Event分阶段的概念边界，但改为真实schedule set与generation-bound receipt。
2. 保留backend `PointerHits` 分组及order/depth排序意图，补齐qualified identity、finite admission、stable tie key、merge policy和source provenance。
3. 保留previous/current hover diff算法，把key扩为qualified pointer+surface/view，并加入retire/cancel原因。
4. 保留 `PickingEventState` 的per-pointer/button隔离方向，扩展为capture lease、press snapshot、gesture policy、click series与bounded output。
5. 保留report到debug feed的单向投影；产品diagnostics只能读authoritative receipt，不能像 `debug_feed_at` 那样重新查询。
6. 保留primitive sphere backend作为明确标注的test/coarse overlay backend，不作为默认产品mesh picking证明。
7. 保留Editor renderer-visible snapshot的world/viewport/frame identity、查询统计与event-time broad phase，接入exact backend而不是继续在Editor私有路由中二次造真值。
8. 保留Editor handle/gizmo的Line/Ring交互shape与屏幕像素容差，但政策、owner generation和target descriptor必须显式化。
9. 保留Runtime physics ray/shape query已有的filter、normal/position/feature/toi底座，新增adapter而不是复制物理宽窄相。
10. 保留Runtime UI自己的surface routing/capture owner；Picking消费UI仲裁receipt，不复制UI focus/IME/gesture状态机。

## 5. P0裁决与父owner

本轮没有发现需要从Runtime47提升为新的P0。Editor59已拥有“事件到完整interaction transaction未闭合、generation/selection/cancel产品链未闭合”等3项P0；Runtime117拥有physical-first input与UI/capture仲裁的P0。本文只记录Picking侧依赖，不重复计数或越权关闭。

## 6. P1当前源码重判

### 6.1 Reachability、Composition 与单一Authority

| ID | 状态 | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| PICK-P1-001 | Open | `run_picking_pipeline` 只有定义、re-export和测试调用者 | per-world/per-surface `PickingFrameCoordinator` 由真实frame schedule调用并发布唯一receipt |
| PICK-P1-002 | Partial | Editor已删除无人消费的 `runtime_input` 假字段，但只返回private route/debug | host input batch进入Picking，Editor消费resolved event/selection receipt |
| PICK-P1-003 | Open | `PickingScheduleLabel` 只写进五个stage report | 映射真实schedule set/barrier，冻结input/view/backend并约束publish顺序 |
| PICK-P1-004 | Open | plain function每次创建RayMap、outputs、hover、events、report | coordinator持有双buffer workspace、settings/backend snapshot与interaction state |
| PICK-P1-005 | Open | `HitTarget` 公共enum仍写死HandleAxis/SceneGizmo/Renderable | qualified `PickTargetKey` + domain descriptor registry，具体Editor类型留在Editor |
| PICK-P1-006 | Open | pipeline从不调用backend `info()`，capabilities无人消费 | registration校验BackendId、输入、view、latency、failure和ordering，冻结snapshot |
| PICK-P1-007 | Partial | Editor route/debug已共用一次runtime resolve；UI、physics、renderer和framework仍是平行authority | 各backend只贡献hits，所有consumer读同一 `ResolvedPickingFrame` |
| PICK-P1-008 | Open | disabled仍直接clear hover/event state，无Out/Cancel/DragEnd receipt | disable/quiesce/shutdown先发布typed retirement/cancellation frame再释放资源 |

### 6.2 Qualified Identity、Multi-View 与 Input Batch

| ID | 状态 | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| PICK-P1-009 | Open | `PointerId(u64)` 无window/device/source/principal/generation | `QualifiedPointerId` 区分mouse/touch/pen/XR/custom/remote和host generation |
| PICK-P1-010 | Open | Entity/camera/viewport/Editor owner仍大量裸 `u64` | world/surface/view/target/backend generation-qualified typed handle |
| PICK-P1-011 | Open | `PointerHits` 仍只有pointer、hits、float order | 携frame stamp、backend、view、query/ticket、source generation和completeness |
| PICK-P1-012 | Open | RayMap可含同pointer多viewport，hover/event只按PointerId分区 | state key至少为qualified pointer+surface/view domain，跨view合并显式化 |
| PICK-P1-013 | Open | RayMap是HashMap，平局仍可受迭代/output index影响 | deterministic iteration与stable tie key，receipt hash不受hash seed/并发完成顺序影响 |
| PICK-P1-014 | Open | 多camera/backend同target没有dedupe或conflict diagnostic | 按target/subobject/view/backend编译merge policy并记录冲突 |
| PICK-P1-015 | Open | `PointerPhase` 继续公开但无生产/测试消费者 | 删除死contract，或由真实pointer lifecycle驱动spawn/move/end/cancel |
| PICK-P1-016 | Open | Picking `PointerInput` 无sequence/time/device/modifier/pressure/tilt/contact/source frame | ordered `PickingInputBatch`，motion可合并但不能跨edge/cancel |

### 6.3 Ray、Projection、Hit Data 与排序正确性

| ID | 状态 | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| PICK-P1-017 | Open | core ray builder仍忽略 `projection_override` | 只消费Runtime23 validated inverse view-projection或camera authority转换结果 |
| PICK-P1-018 | Open | perspective/ortho起点不一致且near/far不进入query range | `PickRay` 明确origin、min/max、near/far、reversed/infinite-Z与clip policy |
| PICK-P1-019 | Open | `CameraRaySource.active` 与snapshot active双authority，重复key覆盖 | view compiler冻结唯一active状态和PickViewKey，duplicate/stale返回typed failure |
| PICK-P1-020 | Open | core只按完整viewport size映射；无rect、DPI、letterbox、logical/physical或target | PickViewSnapshot携target、logical/physical content transform、resolution/jitter policy |
| PICK-P1-021 | Open | position/normal/depth仍无space和origin generation | typed position/normal/depth space，resolver拒绝space mismatch |
| PICK-P1-022 | Open | 无instance/primitive/triangle/material/UV/barycentric/shape/face细节 | bounded typed extra payload或schema detail handle，clone共享且受bytes预算 |
| PICK-P1-023 | Partial | Editor adapter会把非finite score/depth压成有限大值；core仍接受NaN/Inf/negative order/depth | 统一admission校验并隔离计数，invalid hit不能成为top/blocker |
| PICK-P1-024 | Open | 固定HandleAxis > Gizmo > Renderable仍先于camera/backend order | compiled layer/view/backend/depth/stable-key policy；tool priority只作为profile输入 |

### 6.4 Backend Protocol、World/Render/Physics/UI 产品链

| ID | 状态 | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| PICK-P1-025 | Open | backend同步返回 `Vec` 且不可失败/取消，任意实现可阻塞frame | request/ticket/completion支持sync CPU和async GPU，带deadline/cancel/partial/error/budget |
| PICK-P1-026 | Open | 所有backend只收RayMap，UI/GPU capability并不改变request | capability编译Pointer/View/WorldRay/ScreenRegion/RenderReadback最小输入 |
| PICK-P1-027 | Partial | primitive仍是ray×sphere线性扫描；Editor已接renderer BVH/grid宽相但最终仍是圆形代理 | primitive标成test/coarse，产品geometry接renderer/physics acceleration与exact narrow phase |
| PICK-P1-028 | Open | primitive只有builder append，无update/remove/generation/retire | generation-bound immutable snapshot或delta adoption，支持target invalidation |
| PICK-P1-029 | Partial | UI、physics、renderer query底座存在，但没有Picking backend registry或默认exact path | 建最小backend矩阵并对缺失能力诚实报告Unavailable |
| PICK-P1-030 | Partial | renderer query提供visible-only候选；core仍无render layer、mask、locked/hidden、alpha/backface统一policy | per-surface/tool冻结 `PickQueryPolicy`，backend回报已应用政策 |
| PICK-P1-031 | Open | 无GPU-ID request/readback/ID table generation/stale discard | renderer发布PickId table与frame/view stamp，matching generation才采用readback |
| PICK-P1-032 | Open | 无backend registry、owner/module provenance、reload、health、quarantine或shutdown | 纳入module composition并提供panic/no-unwind隔离、generation retire和metrics |

### 6.5 Hover、Capture、Click、Drag-Drop 与传播

| ID | 状态 | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| PICK-P1-033 | Partial | Editor固定zero delta的假输入已删除；core仍对zero delta立即返回且无真实delta ingress | collector按qualified pointer generation计算delta，absolute position变化也驱动语义 |
| PICK-P1-034 | Open | 一个frame所有ordered input仍共享frame末current hover | 每个edge绑定location/query snapshot，press/release不被后续位置重解释 |
| PICK-P1-035 | Open | Picking state无capture；Runtime UI的surface capture是另一个domain | per pointer/button capture lease，hover与capture分离，loss有typed release/cancel |
| PICK-P1-036 | Open | 任意非zero move立即让所有pressed hovered target开始drag | distance/time/device threshold、axis lock和tool override的gesture policy |
| PICK-P1-037 | Open | drag后仍Click；无duration/count/tolerance，empty button slot不retire | press snapshot、click series、drag outcome和slot lifecycle |
| PICK-P1-038 | Open | non-blocking hover链所有target都Press并成为drag source | direct target、propagated ancestor、pass-through listener分层并显式选择drag source |
| PICK-P1-039 | Open | Release/Click固定取previous hover，不区分mouse/touch/capture policy | press target、release-under-pointer、capture target语义按device/profile明确化 |
| PICK-P1-040 | Open | event只有 `propagate: bool`，没有route phase、stop/default或receipt | generation-bound hierarchy path执行capture/target/bubble并记录handled/default action |

### 6.6 Lifetime、Failure、Diagnostics 与资格证据

| ID | 状态 | 当前源码证据 | 必须重构为 |
|---|---|---|---|
| PICK-P1-041 | Open | 只能clear pointer/all，target删除或generation变化不清press/drag/hover | target retirement按qualified key发布Cancel/Leave并清状态 |
| PICK-P1-042 | Open | 本帧无location时previous hover exit被跳过，随后旧map整体替换 | pointer/view retirement携last location与reason，产生terminal transition或silent receipt |
| PICK-P1-043 | Open | event Vec无count/bytes/time admission，multi-drag×hover可放大 | bounded workspace、coalescing、overflow terminal，性能数值回链PERF-MVP-332 |
| PICK-P1-044 | Open | Picking pipeline无Result/outcome，NoHit与NoView/Fault/Stale/Timeout不可区分 | `PickingFrameOutcome` 区分complete/partial/unavailable/rejected与per-backend cause |
| PICK-P1-045 | Partial | Editor route/debug同一次resolve；report仍无frame/view/backend generation、耗时、drop或failure | diagnostics只读authoritative receipt并发布bounded-cardinality指标 |
| PICK-P1-046 | Partial | 22个专用测试仍全是framework单元；Editor有route/debug/renderer-source测试，但没有host到selection完整链 | 增加真实window/input -> UI arbitration -> Picking -> tool/selection/event端到端gate |
| PICK-P1-047 | Open | 无duplicate/NaN/hash/same-frame edge/stale/capture/backend fault/multiview property-fuzz矩阵 | 先写当前错误行为RED，再执行hard cutover |
| PICK-P1-048 | Open | 两份模块/边界文档仍声称已删除的 `runtime_input` 与Cancel桥存在 | 文档按reachable/contract-only/test-only标注，并以caller/fingerprint guard防漂移 |

## 7. P2当前源码重判

| ID | 状态 | 当前源码证据与后续方向 |
|---|---|---|
| PICK-P2-001 | Partial | App/Dynamic UI已有pointer source概念，Picking仍只有raw id；把source descriptor接入qualified pointer |
| PICK-P2-002 | Open | resolved数据继续暴露owned Vec/BTreeMap；正确性闭合后提供borrowed frame view/Arc slab |
| PICK-P2-003 | Open | backend-specific detail无schema/inspection；建立typed extra registry与payload/debug上限 |
| PICK-P2-004 | Open | point overlap固定first；增加stable overlap stack token，Editor循环选择UX仍归Editor03 |
| PICK-P2-005 | Open | GPU readback与CPU capture无continuity；研究latest-known、press确认、prediction correction |
| PICK-P2-006 | Open | 无XR controller/gaze/hand/stereo policy；通过custom pointer/view capability扩展 |
| PICK-P2-007 | Open | 无remote principal/权限隔离；先经过session/auth authority再进入Picking |
| PICK-P2-008 | Open | 无deterministic capture/replay artifact；记录input/view/backend/result receipt hash |
| PICK-P2-009 | Open | 无a11y/keyboard focus projection；由UI owner从统一route receipt映射 |
| PICK-P2-010 | Open | debug feed按请求物化且 `debug_feed_at` 会重跑query；改成bounded history/streaming snapshot |
| PICK-P2-011 | Open | target无cursor/tooltip/affordance descriptor；由surface/tool最终裁决安全提示 |
| PICK-P2-012 | Partial | Editor已有私有rect selection helper，Picking合同仍只有point ray；后续统一rect/lasso/frustum/volume query |

## 8. 五引擎参考结论

| 参考 | 物理源码事实 | 对Zircon的约束 |
|---|---|---|
| Unreal | `HHitProxy` 有typed class、priority、cursor、translucent policy、lifetime/ref-count；SceneHitProxy rendering有custom/per-instance ID、material permutation、Nanite/instance culling；Editor viewport有GetHitProxy、tracking lifecycle、drag threshold、Escape abort与hit-proxy invalidation | Picking必须同时覆盖renderer ID与Editor interaction生命周期；不能用CPU sphere代理宣称等价 |
| Bevy | pointer identity区分Mouse/Touch/Custom，location绑定NormalizedRenderTarget；PointerHits按pointer/order/depth合并，HitData带camera/position/normal/typed extra；state记录press time/location、click count与drag；mesh backend有visibility/AABB/mesh exact/backface/early exit | 可借鉴backend contribution和event vocabulary，但Zircon还需更强generation、failure、budget与async GPU合同 |
| Godot | Viewport区分mouse exit与focus loss，physics picking使用camera ray/space query、shape id、mouse enter/exit和capture-on-drag；Editor gizmo用BVH宽相、exact intersect、frustum selection与重叠候选菜单 | hover、focus、capture、selection volume和overlap cycle必须分开建模，不能由一次frame末hover替代 |
| Fyrox | Editor picking先AABB再precise hull，带filter/backface/method，按TOI排序并支持同位置候选循环；physics query提供groups、normal/position/feature/toi以及Vec/ArrayVec结果容器 | renderer/physics exact path、过滤和无堆分配查询是最低工程基线；coarse fallback必须明确降级 |
| Unity Graphics | HDRP/URP提供ScenePickingPass/SceneSelectionPass、instance ID传递、selection/object ID输出、alpha clip、custom/non-jittered picking matrices与camera-relative修正 | GPU picking要复用真实渲染几何/材质/实例政策并绑定view generation，不能另建简化相机和材质真值 |

参考不是逐类型照抄。Unreal的hit proxy、Unity的ID pass、Bevy的backend/event、Godot/Fyrox的physics/editor query分别证明不同工程面；Zircon应在统一receipt下组合，而不是选一个参考掩盖其余缺口。当前没有同硬件、同场景、同画质、同输入负载的Zircon/参考引擎benchmark，不能声称性能或表现优于Unreal。

## 9. 目标架构

```text
Host Input Authority
  -> PickingInputBatch(frame, sequence, time, qualified pointer, edges)
  -> UI Arbitration Receipt(captured/blocked/pass-through)
  -> PickingFrameCoordinator(world, surface, frame generation)
       -> Frozen PickViewSnapshot[]
       -> PickingBackendRegistry snapshot
       -> sync CPU requests + async GPU tickets
       -> BackendCompletion[] / typed terminal outcomes
       -> PickingResolver(policy, merge, admission, stable ordering)
       -> ResolvedPickingFrame
       -> PointerInteractionState(capture, hover, press, click, drag)
       -> PickingEventReceipt + diagnostics projection
  -> Editor selection/tool adapter / gameplay consumers / devtools
```

核心合同至少应包含：

- `PickingFrameStamp { host, world, surface, frame, input_generation, view_generation, backend_generation }`。
- `QualifiedPointerId { window/surface, device, source, local_id, principal, generation }`。
- `PickViewSnapshot` 只引用Runtime23验证过的projection、viewport/content transform、target和depth convention。
- `PickTargetKey { domain, owner, subobject, generation }` 与有界descriptor，不在Runtime enum硬编码Editor类别。
- `BackendRequest/Completion` 带capability、ticket、deadline、completeness、fault和provenance。
- `ResolvedPickingFrame` 同时是route、interaction、selection、debug与capture/replay的唯一事实源。
- `PointerInteractionState` 区分hover、capture、press target、release target、drag source/drop target和terminal loss。

## 10. 硬切范围与禁止方案

1. 删除现有public `HitTarget` 封闭enum、裸identity和dead `PointerPhase` 时不保留compat re-export、shim trait或双写bridge。
2. Editor private adapter只能在新receipt consumer切通后删除；不得继续把固定pointer/camera/order的 `PointerHits` 当产品backend输出。
3. primitive sphere backend只允许明确的test/coarse profile，默认Editor/gameplay profile缺exact backend必须报告Unavailable。
4. 不复制Runtime UI capture、physics broad phase、renderer visibility或camera projection；通过owner adapter和immutable snapshot接入。
5. 不在Picking core吸收Editor selection transaction、gizmo UX、UI focus/IME、renderer material实现或physics engine细节。
6. 不用未执行的source-string test、helper-only test、空场景或低质量代理benchmark证明功能/性能完成。

## 11. 测试先行的重构里程碑

| 里程碑 | 先写RED证据 | 实施边界 | 退出条件 |
|---|---|---|---|
| M0 Reachability与行为冻结 | product caller guard、同pointer双view、same-frame edges、NaN/duplicate、disable/loss | 只建立失败oracle和当前receipt snapshot | 当前错误可稳定复现，22个既有语义有迁移清单 |
| M1 Identity与Frame Receipt | raw-id collision、stale target/view/backend、deterministic hash | qualified identities、frame stamp、immutable `ResolvedPickingFrame` | stale fail closed且同输入receipt hash稳定 |
| M2 View与Ray | custom projection、DPI/rect、ortho/perspective、near/far、camera cut | 复用Runtime23 view snapshot，硬删core自造投影真值 | CPU ray与camera authority oracle一致 |
| M3 Backend Registry | duplicate ID、capability mismatch、panic/timeout/cancel/partial、reload retire | registry snapshot与request/ticket/completion，不接具体产品backend | 所有backend terminal bounded且可审计 |
| M4 产品Backend | visible/hidden、instance、masked/backface、physics mask/subobject、UI no-ray | renderer ID/exact、physics adapter、UI contribution；sphere降级 | Editor默认至少一个exact path，缺失能力诚实Unavailable |
| M5 Interaction | capture loss、threshold、click count/time、multi-button、touch/pen、delete/reparent | capture/hover/gesture/event route state machine | press/release/cancel edge不丢，ghost event为零 |
| M6 Editor Hard Cut | App input到viewport selection/tool receipt、debug不重查、error UI | 删除private authority与stale docs，Editor只消费runtime receipt | 非测试caller可追到真实selection/tool transaction |
| M7 Scale与竞争证据 | high-frequency input、backend delay/fault、event amplification、multi-view/instance scene | bounded workspace与profiling，接PERF-MVP-332 | raw artifact证明预算与语义，不以功能缺失换性能 |

## 12. 资格门

| Gate | 状态 | 当前证据 / 通过要求 |
|---|---|---|
| G01 | Fail | 没有active surface/frame唯一PickingFrameStamp与receipt |
| G02 | Fail | framework pipeline、Editor resolver、UI/physics/render query仍是平行authority |
| G03 | Fail | Editor production input未进入Picking event/selection receipt |
| G04 | Fail | raw pointer id跨window/device/principal/generation可碰撞 |
| G05 | Fail | hover/event不按view分区，双viewport会串状态 |
| G06 | Partial | renderer-visible snapshot会按world/view/frame identity拒绝部分stale；target/backend仍无generation |
| G07 | Fail | 无receipt hash，HashMap/output index仍影响平局 |
| G08 | Fail | duplicate target无compiled merge policy |
| G09 | Partial | Editor adapter有限化异常depth；core admission仍不拒绝NaN/Inf/negative |
| G10 | Partial | Editor projection context复用camera/viewport；core custom projection、rect/DPI/near-far仍未闭合 |
| G11 | Fail | hit position/normal/depth无typed space |
| G12 | Fail | mesh hit无instance/primitive/triangle/material等bounded detail |
| G13 | Fail | backend capability不控制registration/request |
| G14 | Fail | 没有真正UI backend，screen-space contribution未进入统一pipeline |
| G15 | Fail | backend error/panic/timeout/cancel没有API terminal |
| G16 | Fail | 没有GPU picking ticket/readback generation |
| G17 | Partial | renderer visible BVH/grid提供可见宽相；无exact instance/material/deformation picking |
| G18 | Partial | physics query有mask/shape/subobject substrate；无Picking adapter和capture场景 |
| G19 | Fail | coarse sphere/AABB/circle fallback没有在receipt/UI明确标识 |
| G20 | Fail | zero-delta move会被core抑制，高频/coalesced语义无产品证据 |
| G21 | Fail | same-frame edge共享frame末hover |
| G22 | Fail | Picking无capture与focus/window-loss cancel链 |
| G23 | Fail | 非zero move立即drag，drag后仍click |
| G24 | Fail | 无duration/count/long-press/multi-button/touch/pen oracle |
| G25 | Fail | non-blocking多个target会同时成为drag source |
| G26 | Partial | Editor visible query有world generation fail-closed；Picking target删除/reparent/reuse仍会ghost |
| G27 | Partial | Runtime UI有独立route authority；Picking event没有capture/target/bubble与stop/default receipt |
| G28 | Fail | disable/quiesce/shutdown仍静默clear |
| G29 | Partial | Editor surface保留typed `UiTreeError`；Picking仍无法区分NoHit/NoView/Stale/Fault/Timeout |
| G30 | Partial | Editor route/debug在event path共用一次resolve，但 `debug_feed_at` 仍重跑query且无frame receipt |
| G31 | Fail | event/result无count+bytes+time admission与edge-preserving overflow |
| G32 | Partial | 22个专用测试和部分Editor测试存在；未运行且无production e2e/fault/property/fuzz矩阵 |
| G33 | Fail | PERF-MVP-332 workspace/probe/allocation/amplification资格未完成 |
| G34 | Pass | 本文保留Runtime12/23/24/77、Render04、Editor03/59与Performance01父owner，不重复关闭 |
| G35 | Pass | 本轮因源码/fingerprint变化完成旧Runtime47、参考源码与产品caller重查 |
| G36 | Pass | 本报告完成frontmatter、路径、计数、索引、LF/BOM/trailing-space与scoped diff检查后方可保持Pass |

Gate统计：**23 Fail、10 Partial、3 Pass**。Pass仅代表owner/复审/文档卫生，不代表产品功能资格。

## 13. Owner、依赖顺序与开放Failure

| Owner | 保留职责 | Runtime133只要求的接口 |
|---|---|---|
| Runtime12 / Runtime117 | physical input、device/window/user/seat、时间/sequence、UI优先仲裁 | ordered qualified input batch + arbitration receipt |
| Runtime23/37 | camera/view/projection/multi-view authority | immutable validated PickViewSnapshot |
| Runtime24 | world/entity/handle generation与retirement | qualified target/view/backend key及retire event |
| Runtime77 | UI route/focus/capture/gesture/IME/window lifecycle | UI contribution与blocked/captured/pass-through outcome |
| Render04/09A | visibility、instance、material、GPU ID/readback | generation-bound renderer picking request/completion |
| Physics owner | scene query、filter、shape/feature与backend lifetime | immutable query snapshot或bounded adapter completion |
| Editor03/59 | selection、gizmo/tool、transaction/cancel与UX | consume authoritative picking/event receipt |
| PERF-MVP-332 | workspace、allocation、probe、event amplification预算 | raw benchmark/profile artifact与门槛 |

保持开放：

- `docs/plans/zircon_editor/editor/05/failure-2026-07-18-viewport-pointer-candidate-regeneration.md`：renderer-visible query/currentness已有局部落地；exact broad phase、scale metrics与managed product capture仍未完成。
- `docs/plans/performance/01/failure-2026-08-01-picking-repeated-hit-projection-false-green.md`：静态COW/单projection改动存在，但focused current-source Cargo 1/1没有成功完成，禁止转成fixed。

## 14. 验证边界与首个实施切片

本轮只执行静态review、caller反查、物理统计、reference source compare、文档结构检查和scoped diff检查。没有运行Cargo、真实Editor/App、GPU ID pass、physics backend、输入设备矩阵、multi-window/multi-view、fault/fuzz/soak/profile或跨引擎同语义benchmark。

首个实现切片应只做M0，不先扩充sphere、再造Editor helper或增加更多公共枚举：先以测试证明产品不可达、同pointer双view串状态、same-frame edge误判、异常hit admission、disable/loss静默清理和非确定平局；随后才能硬切identity/frame receipt。性能目标必须建立在同功能、同场景、同硬件、同画质和raw receipt上，不能用缺失材质/实例/事件语义的简化路径宣称优于Unreal。
