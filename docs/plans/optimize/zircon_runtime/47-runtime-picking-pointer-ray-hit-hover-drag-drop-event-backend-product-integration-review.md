---
related_code:
  - zircon_runtime/src/core/framework/picking/mod.rs
  - zircon_runtime/src/core/framework/picking/backend.rs
  - zircon_runtime/src/core/framework/picking/debug_feed.rs
  - zircon_runtime/src/core/framework/picking/hit_data.rs
  - zircon_runtime/src/core/framework/picking/hit_record.rs
  - zircon_runtime/src/core/framework/picking/hit_target.rs
  - zircon_runtime/src/core/framework/picking/hover_map.rs
  - zircon_runtime/src/core/framework/picking/pickable.rs
  - zircon_runtime/src/core/framework/picking/pipeline.rs
  - zircon_runtime/src/core/framework/picking/pointer_button.rs
  - zircon_runtime/src/core/framework/picking/pointer_event.rs
  - zircon_runtime/src/core/framework/picking/pointer_event_state.rs
  - zircon_runtime/src/core/framework/picking/pointer_hits.rs
  - zircon_runtime/src/core/framework/picking/pointer_id.rs
  - zircon_runtime/src/core/framework/picking/pointer_input.rs
  - zircon_runtime/src/core/framework/picking/pointer_location.rs
  - zircon_runtime/src/core/framework/picking/pointer_phase.rs
  - zircon_runtime/src/core/framework/picking/primitive_backend.rs
  - zircon_runtime/src/core/framework/picking/ray.rs
  - zircon_runtime/src/core/framework/picking/ray_map.rs
  - zircon_runtime/src/core/framework/picking/report.rs
  - zircon_runtime/src/core/framework/picking/schedule_label.rs
  - zircon_runtime/src/core/framework/picking/settings.rs
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/tests/picking/mod.rs
  - zircon_runtime/src/tests/picking/diagnostics.rs
  - zircon_runtime/src/tests/picking/hits_and_hover.rs
  - zircon_runtime/src/tests/picking/pipeline.rs
  - zircon_runtime/src/tests/picking/pointer_events.rs
  - zircon_runtime/src/tests/picking/rays.rs
  - zircon_editor/src/scene/viewport/pointer/runtime_picking_adapter.rs
  - zircon_editor/src/scene/viewport/pointer/viewport_pointer_dispatch.rs
  - zircon_editor/src/scene/viewport/pointer/viewport_pointer_route.rs
  - zircon_editor/src/scene/viewport/pointer/constants.rs
  - zircon_editor/src/scene/viewport/pointer/overlay_router/build_dispatcher.rs
  - zircon_editor/src/scene/viewport/pointer/overlay_router/viewport_overlay_pointer_router_event.rs
  - zircon_editor/src/scene/viewport/pointer/overlay_router/viewport_overlay_pointer_router_debug.rs
  - zircon_editor/src/scene/viewport/pointer/precision/shared_resolution_state.rs
  - zircon_editor/src/scene/viewport/pointer/precision/renderer_visible_spatial_pick_source.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_pointer_route.rs
  - zircon_runtime/src/core/framework/render/backend_types/handles.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime_interface/src/ui/dispatch/input/event.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_runtime/23-coordinate-space-unit-precision-transform-numeric-robustness-large-world-review.md
  - docs/plans/zircon_runtime/runtime/05-scene-editor-boundary-closeout.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
  - docs/plans/performance/01/2026-07-18-runtime-core-framework-picking-static-review.md
reference_engines:
  - dev/bevy/crates/bevy_picking/src/backend.rs
  - dev/bevy/crates/bevy_picking/src/events.rs
  - dev/bevy/crates/bevy_picking/src/hover.rs
  - dev/bevy/crates/bevy_picking/src/pointer.rs
  - dev/bevy/crates/bevy_picking/src/mesh_picking/mod.rs
  - dev/bevy/crates/bevy_picking/src/mesh_picking/ray_cast/mod.rs
  - dev/UnrealEngine/Engine/Source/Runtime/Engine/Public/HitProxies.h
  - dev/UnrealEngine/Engine/Source/Runtime/Renderer/Private/SceneHitProxyRendering.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/EditorViewportClient.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorViewportClient.cpp
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
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 47 · Runtime Picking、Pointer、Ray、Hit、Hover、Drag/Drop、Backend 与产品接入工程化差距

## 1. 结论

`zircon_runtime::core::framework::picking` 已经不是空壳。它具有 pointer/location/input DTO、透视与正交射线、camera-pointer RayMap、backend hit 合并、固定 target priority、hover/blocking 约简、pointer event 状态机、一个 sphere CPU backend、report/debug feed，以及 22 个未忽略的单元测试。近期性能修复还把 hover/report 的命中排序合并为一次 projection，并用 `Arc` 共享普通帧 hover storage。这些基础应保留，不能把当前模块误写成“完全没有 Picking”。

但这套实现仍是 Bevy Picking 外形的 plain-Rust 试验层，不是 Zircon 产品 Picking authority。`run_picking_pipeline` 没有任何 production caller；Editor 只把 precision candidates 手工转成 `PointerHits`，调用 `resolve_picking_outputs` 取得 route/debug，再把构造出的 `PointerInput` 放进返回对象。该 `runtime_input` 除测试外没有消费者，`PickingEventState`、ray stage、backend stage 与 event stream 均未进入 Editor、App、Runtime world 或 UI 的真实帧链。所谓 `PickingScheduleLabel` 也只是五个统计标签，不是 scheduler 中的 set/barrier。

更深的问题不是“接一条调用”就能修好：`PointerHits` 和 hover/event state 只按 `PointerId` 分组，虽然 RayMap 测试允许同一 pointer 同时位于两个 viewport，后续却会把两个 viewport、多个 camera 的 hits 混成一份 hover；`HitTarget` 是硬编码的 HandleAxis/SceneGizmo/Renderable 三分 enum；所有 owner、camera、pointer、viewport 都是无 generation 的裸 `u64`；backend trait 同步、不可失败、只接 `RayMap`，而 `Ui`/`GpuPicking` capability 从未被读取；event state 没有 capture、drag threshold、click duration/count、target invalidation、同帧逐事件 hit snapshot 或事件预算。当前 Editor move 又固定产生 `delta=ZERO`，即使今天把 pipeline 接上，Move/Drag 仍会被状态机直接丢弃。

本报告记录 **0 个新增 P0、48 个 P1、12 个 P2**。可见像素/精确几何、Editor 硬编码 pointer/viewport/camera、box selection 与 visualization registry 继续由 Editor03 拥有；typed coordinate/large-world 由 Runtime23 拥有；输入上游与 UI-first 仲裁由 Runtime12/Runtime09 拥有；重复排序、workspace、全乘积复杂度与 event amplification 的性能资格继续由 PERF-MVP-332 拥有。Runtime47 只拥有通用 Picking 的 frame authority、qualified identity、backend protocol、resolved-hit contract、pointer interaction state、产品接线与端到端资格，避免重复建 P0 或关闭别人的 failure。

本轮没有运行 Cargo、Editor、GPU readback、真实窗口、多 viewport、触摸/笔/XR 或 fault-injection 测试。性能结论只陈述代码复杂度和缺失的预算，不宣称 Zircon 当前快于或慢于 Unreal、Bevy、Godot、Fyrox 或 Unity Graphics。

## 2. 审查边界与可复验证据

### 2.1 物理范围

| 范围 | 文件 / 行 / bytes | 测试 | 状态 |
|---|---:|---:|---|
| `core/framework/picking` production | 23 / 2,069 / 61,154 | production 内无独立 `#[test]` owner | 逐文件完整读取；clean |
| `src/tests/picking` | 6 / 889 / 29,809 | 22 `#[test]`、0 ignored | 逐文件完整读取；clean，未执行 |
| focused total | 29 / 2,958 / 90,963 | 22 / 0 | fingerprint `377c7f27b41bf31a6d7c061ac3feee9ab793c492d6468dbf5b65ba110252b1c3` |
| Editor product bridge | adapter、dispatcher、event wrapper、shared state、renderer-visible source、controller caller | focused source tracing | 只作 integration evidence；Editor03 保持产品 UX owner |

fingerprint 使用按相对路径排序的 `path + NUL + per-file SHA-256` 清单再次计算 SHA-256。它仅标识本次读取集合，不是 ABI、cache key 或 release identity。Picking/test 源码最近一次相关提交为 `7a20f921bb97ed428ae248cbcaf3c2fac5442ddf`（2026-08-16）；本报告基线 HEAD 为 `25e09a23178000f2e783ce2143cf70a8b118d404`。成文时 Picking、专用测试与 Editor pointer 源码均无工作树改动；共享 optimize 文档存在其他会话改动，本报告不覆盖或回退它们。

### 2.2 产品调用链反查

全仓排除 docs 与 picking tests 后，`run_picking_pipeline`、`PickingEventState::default()` 和 `PrimitivePickingBackend::new()` 均没有 production caller。唯一真实复用点是 Editor adapter 的 `resolve_picking_outputs(&outputs)`：

```text
SceneViewportController::route_at_cursor
  -> ViewportOverlayPointerRouter::handle_move / handle_down
  -> UiSurface dispatcher
  -> renderer-visible broad phase + editor precision candidates
  -> editor-owned runtime_pointer_hits_for_candidates
  -> resolve_picking_outputs
  -> first hovered HitTarget -> private ViewportPointerRoute
  -> selection / hover controller
```

同一个 event wrapper 还构造 `PointerInput`，但 controller 只读取 `dispatch.route`。`ViewportPointerDispatch::runtime_input` 的非测试生产读取为零；`handle_up` 与 `handle_scroll` 甚至只在 `#[cfg(test)]` 下作为 convenience method 暴露。因而当前产品路径没有 Runtime Picking hover history、press/release/click、drag/drop、cancel、ray backend、stage report 或 backend diagnostics。

### 2.3 已有 owner 与不重复范围

| 已有 owner | 继续负责 | Runtime47 只负责 |
|---|---|---|
| Editor03 | renderer-visible 精确点选、GPU/CPU pick product、Editor identity 来源、box selection、gizmo/component visualization、selection UX | runtime frame/result/backend/event contract及Editor cutover接收面 |
| Runtime12 / Runtime09 | platform input归一、window/device/pointer ingress、UI先消费与玩法fallback、coalescing | UI仲裁后进入Picking的qualified pointer batch与capture状态 |
| Runtime23 | coordinate/space/unit/precision/origin generation、validated projection | Picking只消费其typed view/ray snapshot，不重建数学authority |
| Runtime24 | stable handle、generation、owner epoch与stale rejection通则 | `PickTargetKey/PickViewKey/BackendId` 的领域组合与失效使用 |
| Runtime09A/Render04 | renderer visibility、draw/instance identity、GPU Scene与readback | renderer-published picking backend contract和receipt |
| PERF-MVP-332 | reusable workspace、sort/clone/allocation、ray×camera、primitive probes、drag×hover event成本 | correctness先行的容量/admission语义和产品trace入口 |

## 3. 当前实现中应保留的工程基础

1. `RayMap` 已把 `(camera, pointer, viewport)` 作为 ray key，并能过滤 inactive camera、viewport mismatch 与越界 location；这比单一全局鼠标射线更接近多视图底座。
2. perspective/orthographic ray 都通过 `PointerRay::new` 做 finite 与非零方向检查，invalid ray 返回 `None` 而不是 panic。
3. hit reduction 已统一 `target priority -> backend order -> depth -> stable insertion` 顺序，并使用 `total_cmp` 避免普通比较直接 panic。
4. `Pickable` 已区分 hoverable 与 block-lower，支持 invisible blocker、non-blocking overlay 与 ignored hit。
5. `resolve_picking_outputs` 让 hover 与 report 共用一次 sorted projection；普通帧 `PickingPipelineOutput` 与 next-frame event state 共享 hover backing storage。
6. event vocabulary 已覆盖 Over/Enter/Move/Leave/Out、Press/Release/Click、DragStart/Drag/DragEnd、DragEnter/Over/Drop/Leave、Scroll 与 Cancel。
7. cancel 会先排除同帧 backend hover，再清理 pointer state；disabled pipeline 也不会保留 stale press/drag map。
8. report/debug feed 能保留 ray-only pointer、raw/hover/block counts，以及 top/blocking target，适合作为后续 typed telemetry 的最小起点。

这些能力应通过 hard cut 迁移到真正的 frame coordinator，而不是另建 `picking2` 或让 Editor/Runtime UI/physics/render 各自继续复制一套 route state。

## 4. P0 裁决

本轮不新增 P0。Picking 当前没有 production event authority，本身不会直接覆盖磁盘或破坏持久化；最严重的可见产品行为仍由 Editor03 的 proxy picking、identity 与错误吞没条目覆盖。实施 Runtime47 时若发现 renderer ID reuse 导致错误对象被 destructive command 选中，必须回到 Editor03/Runtime24 按数据完整性标准升级，而不是在本报告提前虚构 P0。

## 5. P1：产品化前必须闭合的工程差距

### 5.1 Reachability、Composition 与单一 Authority

| ID | 当前差距 | 必须重构的内容 | 唯一 owner / 验收 |
|---|---|---|---|
| PICK-P1-001 | `run_picking_pipeline` 只有定义与测试调用者 | 建立一个 per-world/per-surface `PickingFrameCoordinator`，由真实 frame schedule 调用并发布 generation-bound result | Runtime47；产品trace每个active surface每帧至多一个 authoritative result |
| PICK-P1-002 | Editor 构造 `PointerInput` 后无人消费，只读取私有 route | Editor bridge提交 input batch并消费 resolved target/event receipt；删除只为测试携带的死字段 | Runtime47 + Editor03；非测试 caller 可追到 event/selection command |
| PICK-P1-003 | `PickingScheduleLabel` 只是 output 中的五个枚举项 | 映射到真实 schedule set/barrier，声明 input freeze、view freeze、backend deadline、resolve、event publish 顺序 | Runtime03/Runtime47；并发系统不能越过frame stamp |
| PICK-P1-004 | plain synchronous function同时临时拥有所有 frame containers，却没有 world/surface owner | coordinator持有双buffer workspace、settings generation、backend snapshot和event state；function降级为受控内部阶段 | Runtime47；没有第二套产品state或process-global singleton |
| PICK-P1-005 | `HitTarget` 把 Editor 的三类 route 写死到 Runtime 公共 enum | 改为 qualified `PickTargetKey { domain, owner, subobject, generation }`，具体handle/gizmo/renderable由注册descriptor解释 | Runtime24 + Runtime47；新增component/tool/backend无需改core enum |
| PICK-P1-006 | `PickingBackendInfo`/capabilities 无任何 consumer，`info()` 从未被pipeline调用 | registration时验证唯一BackendId、capability、input kind、view support、latency、ordering与failure policy，冻结backend snapshot | Runtime47；声明与实际request/result可做parity审计 |
| PICK-P1-007 | Editor precision、Runtime UI hit test、physics query、renderer visible query与framework pipeline是平行真值 | 定义一份 `ResolvedPickingFrame`，各backend只贡献hits，route/debug/events/selection都消费同一receipt | Runtime47；同一pointer/frame的top target在所有consumer一致 |
| PICK-P1-008 | disabled pipeline直接clear state且不发布Out/Cancel/DragEnd；shutdown也无协议 | disable/quiesce先发布typed cancellation/retirement frame，再清资源；abrupt loss记录terminal reason | Runtime01 + Runtime47；observer不会看到永久Pressed/Hovered |

### 5.2 Qualified Identity、Multi-View 与 Input Batch

| ID | 当前差距 | 必须重构的内容 | 唯一 owner / 验收 |
|---|---|---|---|
| PICK-P1-009 | `PointerId(u64)` 不含window/device/source/principal/generation | 组合 `QualifiedPointerId`，区分mouse/touch/pen/XR/custom/remote及host generation | Runtime12/24 + Runtime47；跨window同raw id不碰撞 |
| PICK-P1-010 | EntityId、camera和RenderViewportHandle都是裸 `u64`，可被回收后误认 | target/view/backend全部采用owner+generation typed handle，result验证同一world/surface generation | Runtime24 + Runtime47；stale handle fail closed |
| PICK-P1-011 | `PointerHits` 只有pointer、hits、float order，没有viewport/ray/backend/frame | result携 `PickingFrameStamp`、BackendId、PickViewKey、query/ticket id、source generation和completeness | Runtime47；report能定位每个hit来源 |
| PICK-P1-012 | RayMap可存同pointer两个viewport，hover/event却只按pointer分组并用最后一个location | hover/state至少按 `(qualified pointer, surface/view domain)` 分区，跨view合并必须有显式stack/composition policy | Runtime47；双viewport同pointer测试不串hover/location |
| PICK-P1-013 | RayMap是HashMap，equal priority/order/depth最终受迭代产生的output index影响 | deterministic key/order或显式tie key；所有backend output排序必须独立于hash seed与线程完成顺序 | Runtime47；100次随机插入receipt hash一致 |
| PICK-P1-014 | 同一target可由多个camera/backend重复提交，hover/event不dedupe | merge policy按target/subobject/view/backend定义replace/combine/keep-many，冲突产生diagnostic | Runtime47；重复hit不制造重复Enter/Press |
| PICK-P1-015 | `PointerPhase` 公开导出但没有任何生产或测试使用 | 删除死类型或让ingress lifecycle真正驱动spawn/move/end/cancel；禁止保留虚假公共能力 | Runtime12 + Runtime47；public inventory零dead contract |
| PICK-P1-016 | `PointerInput` 无sequence、timestamp、device metadata、modifiers、pressure/tilt/contact或source frame | 采用ordered `PickingInputBatch`，保留edge顺序与时间，motion可合并但不能跨press/release/cancel | Runtime12 + Runtime47；125/500/1000Hz语义与UI仲裁一致 |

### 5.3 Ray、Projection、Hit Data 与排序正确性

| ID | 当前差距 | 必须重构的内容 | 唯一 owner / 验收 |
|---|---|---|---|
| PICK-P1-017 | ray builder忽略 `ViewportCameraSnapshot::projection_override` | 只消费Runtime23 validated inverse view-projection或camera authority的`viewport_to_world`结果 | Runtime23/37 + Runtime47；custom projection parity |
| PICK-P1-018 | perspective ray从camera位置起步，orthographic从camera plane起步，z-near/z-far均未进入query range | `PickRay`显式定义origin convention、min/max distance、near/far、reversed/infinite-Z和clip policy | Runtime23 + Runtime47；perspective/ortho深度同一合同 |
| PICK-P1-019 | `CameraRaySource.active` 与snapshot `is_active` 双authority，重复camera key静默覆盖 | view compiler一次冻结active state和unique PickViewKey；duplicate/stale source返回typed failure | Runtime37 + Runtime47；无last-write-wins |
| PICK-P1-020 | location只按完整viewport size映射，不表达viewport rect、DPI/logical-physical scale、letterbox或render target | PickViewSnapshot携target、logical/physical rect、content transform、resolution/jitter policy | Runtime06/09A/37 + Runtime47；split/subviewport/DPI parity |
| PICK-P1-021 | `HitData.position/normal` 是无space标记的裸Vec3，depth只靠调用者自洽 | 使用typed position/normal/depth space和origin generation，backend必须声明并由resolver验证 | Runtime23 + Runtime47；space mismatch拒绝而非排序 |
| PICK-P1-022 | hit没有triangle/primitive/instance/material/UV/barycentric/shape/face等扩展信息 | 提供bounded typed extra payload或schema-keyed detail handle，clone共享且有bytes预算 | Runtime47；mesh/physics/UI backend可保留subobject证据 |
| PICK-P1-023 | target priority、backend order和depth接受NaN、负值、无限值；`total_cmp`只是给异常值排了确定顺序 | admission校验finite/range/domain；invalid hit隔离并计数，不能成为top/blocker | Runtime23 + Runtime47；NaN/Inf/negative fuzz fail closed |
| PICK-P1-024 | 固定 HandleAxis > SceneGizmo > Renderable 永远压过camera/backend order，无法表达UI layer、tool policy或view stack | ordering由compiled layer policy、view order、backend layer、depth和stable tie key组成；Editor tool priority作为profile输入 | Runtime47 + Editor03；政策可审计，不以enum声明顺序暗控 |

### 5.4 Backend Protocol、World/Render/Physics/UI Product Chain

| ID | 当前差距 | 必须重构的内容 | 唯一 owner / 验收 |
|---|---|---|---|
| PICK-P1-025 | backend trait同步返回Vec且不可失败，任意backend可阻塞frame | request/ticket/completion协议支持sync CPU与async GPU，带deadline/cancel/partial/error和budget | Runtime47；hung/fault backend有bounded terminal |
| PICK-P1-026 | 所有backend只收到RayMap，`Ui`与`GpuPicking` capability却可能不需要ray | 按capability提供Pointer/View/WorldRay/ScreenRegion/RenderReadback输入，backend只获得声明的最小view | Runtime47；UI backend在ray stage关闭时仍可工作 |
| PICK-P1-027 | 唯一backend只支持sphere，并对每条ray线性扫描全部primitive | 明确降级为test/coarse overlay backend；产品geometry接renderer/physics acceleration structure | Runtime47 + Editor03/Runtime08A；不得把sphere命中宣传为mesh picking |
| PICK-P1-028 | primitive backend只有builder式追加，无update/remove/world generation或retire | 若保留overlay backend，使用generation-bound immutable snapshot/delta adoption与target invalidation | Runtime47；scene变化不会靠重建未知全量Vec隐式同步 |
| PICK-P1-029 | 没有mesh、physics、UI、sprite/2D、terrain、particle、gizmo或renderer ID真实backend | 建立最小产品backend矩阵并由各域owner实现；缺失能力在profile/load report中诚实Unavailable | Runtime47协调，各子域实现；default Editor至少一个exact path |
| PICK-P1-030 | hit query没有visibility、render layer、collision mask、locked/hidden/editor-only、alpha/backface policy | `PickQueryPolicy`按surface/tool冻结，backend返回已应用policy receipt | Editor03/Runtime08A/09A + Runtime47；跨backend政策一致 |
| PICK-P1-031 | GPU picking没有request frame、readback latency、camera jitter、resize、ID table generation或stale discard | renderer发布PickId table + frame/view stamp，readback只在matching generation提交；支持latest-known/prediction policy | Render04/Runtime09A + Runtime47；resize/camera cut后旧结果拒绝 |
| PICK-P1-032 | backend无registry、owner/module provenance、reload、quarantine、metrics或shutdown | backend registry纳入module composition，执行panic/no-unwind隔离、health、disable与generation retire | Runtime01/03/46 + Runtime47；插件卸载后无悬空trait object |

### 5.5 Hover、Capture、Click、Drag/Drop 与传播语义

| ID | 当前差距 | 必须重构的内容 | 唯一 owner / 验收 |
|---|---|---|---|
| PICK-P1-033 | `dispatch_move` 对delta ZERO直接返回；Editor所有move恰好固定ZERO | ingress collector基于同pointer generation计算delta，absolute move仍可按position change触发hover/move | Runtime12 + Runtime47；Editor真实move产生Move/Drag |
| PICK-P1-034 | 一个frame先计算一次current hover，再让所有ordered inputs共享previous/current两张图 | input edge绑定对应location/query snapshot；至少press/release不得被frame末位置重解释 | Runtime47；同帧move-down-move-up序列目标正确 |
| PICK-P1-035 | 状态机没有pointer capture、capture owner/reason、implicit/explicit release或loss | 引入per pointer/button capture lease，capture target接收move/up/cancel，hover与capture分开维护 | Runtime09/12 + Runtime47；出viewport拖动仍终结，focus loss cancel |
| PICK-P1-036 | 任何非零move立即对所有pressed hovered targets开始drag | gesture policy声明distance/time/device threshold、axis lock与tool override；越阈值才DragStart | Runtime47 + Editor03；1px jitter不变drag |
| PICK-P1-037 | drag后仍会Click；Click没有duration/count/tolerance，button空state也不retire | press state记录time/position/click series/drag outcome，按policy抑制click并回收empty button slot | Runtime47；single/double/triple、long press、drag-click矩阵 |
| PICK-P1-038 | non-blocking hover链上所有target都Press并同时成为drag source，产生多drag目标 | direct target、propagated ancestors与pass-through listeners分层；capture/handler决定唯一或显式multi-source | Runtime47；overlay+world不默认同时拖动 |
| PICK-P1-039 | Release/Click固定使用previous hover，未按mouse/touch/capture/device policy区分 | 明确press-target、release-under-pointer与capture-target语义；兼容Bevy行为只能作为profile，不是隐藏规则 | Runtime47；mouse/touch/pen各自oracle |
| PICK-P1-040 | event只有`propagate: bool`，没有实际hierarchy resolver、capture/bubble phase、stop/default或receipt | runtime routing bridge基于generation-bound hierarchy生成path，执行capture/target/bubble并记录handled/default action | Runtime09/Runtime47；deleted/reparented path有确定政策 |

### 5.6 Lifetime、Failure、Diagnostics 与资格证据

| ID | 当前差距 | 必须重构的内容 | 唯一 owner / 验收 |
|---|---|---|---|
| PICK-P1-041 | state只能clear pointer/all，target删除或generation变化不能清press/drag/hover | target-retire event按qualified key清理并发布Cancel/Leave；旧owner id不可命中新对象 | Runtime24 + Runtime47；delete/reuse storm无ghost click |
| PICK-P1-042 | previous hover中的pointer若本帧没有location，exit被跳过，随后previous map仍被整体替换 | pointer/view retirement携last valid location与reason；必须产生terminal hover transition或明确silent-retire receipt | Runtime47；window/viewport/pointer disappear矩阵 |
| PICK-P1-043 | event Vec无count/bytes/time admission，multi-drag×hover可无界放大 | correctness层定义事件上限、coalescing与overflow terminal；具体workspace/性能数字回链PERF-MVP-332 | Runtime47 + PERF-MVP-332；压力下不静默截断edge |
| PICK-P1-044 | pipeline返回值永远成功，NoHit、NoView、BackendFault、Stale、Timeout无法区分 | `PickingFrameOutcome`分开complete/partial/unavailable/rejected，保留per-backend typed cause与retryability | Runtime03 + Runtime47；Editor不再把故障当NoHit |
| PICK-P1-045 | report只有count/top target，没有frame/view/backend generation、耗时、drop、invalid或stale原因 | diagnostics读取同一receipt，按bounded cardinality发布stage/backend counters、latency、queue age和failure | Runtime03 + Runtime47；debug不重跑query |
| PICK-P1-046 | 22个测试全部是framework单元，没有一个production caller或App/Editor frame end-to-end gate | 增加真实window/input->UI arbitration->Picking->selection/tool/event链，tests不能只引用helper | Runtime47 + Editor03/App03；产品trace中各stage可见 |
| PICK-P1-047 | 测试缺duplicate target、NaN/Inf、hash determinism、same-frame edges、stale target、capture、backend fault和multiview collapse | 建property/fuzz/fault matrix，先以当前错误行为写RED，再实施hard cut | Runtime47；所有高风险合同有behavior oracle |
| PICK-P1-048 | 模块文档把M2 runner、backend seam与Editor runtime input写成已接入能力，且Bevy参考已新增extra/click count等语义 | 文档按reachable/contract-only/test-only明确标注；每次source fingerprint变化重核参考与产品caller | Runtime47；docs inventory与call graph guard一致 |

## 6. P2：后续能力与维护性债务

| ID | 差距 | 后续处理 |
|---|---|---|
| PICK-P2-001 | pointer source只能从raw id猜测 | 增加mouse/touch/pen/eraser/XR/custom/remote source descriptor，保持core routing不依赖平台枚举细节 |
| PICK-P2-002 | resolved frame对consumer暴露大量owned Vec/BTreeMap | 在正确性完成后提供borrowed frame view与Arc slab，避免鼓励consumer clone完整hit/event表 |
| PICK-P2-003 | backend-specific hit detail无schema与inspection | typed extra registry支持triangle/UV/material/physics shape/UI path，并限制Debug输出和payload bytes |
| PICK-P2-004 | 重叠对象只能固定取first，没有产品级cycle/choose-through | 提供stable overlap stack token；Editor循环选择继续由Editor03决定UX |
| PICK-P2-005 | GPU readback与CPU capture间没有hybrid continuity | 研究hover latest-known、press同步确认、预测校正与mismatch incident，不能让异步结果重放旧click |
| PICK-P2-006 | 没有XR controller ray、gaze、hand joint或stereo view policy | 用custom pointer/view capability扩展，不在core硬编码具体XR SDK |
| PICK-P2-007 | 没有multi-user/remote pointer principal、颜色与权限隔离 | remote input先通过session/auth authority，Picking只接qualified principal和capability |
| PICK-P2-008 | 没有deterministic capture/replay artifact | 记录input batch、view/backend receipt hash与resolved result；禁止把GPU nondeterminism伪装bit-exact |
| PICK-P2-009 | hover/press状态没有accessibility或keyboard focus投影 | UI owner从统一route receipt映射a11y/focus，world picking不直接伪造accessibility node |
| PICK-P2-010 | debug feed固定分配六个metric和rows，缺可查询历史 | devtools使用bounded ring/streaming snapshot；普通产品frame不因无人订阅物化debug DTO |
| PICK-P2-011 | HitTarget没有cursor、tooltip、interaction affordance descriptor | target descriptor可提供安全的cursor/action hints，最终政策由surface/tool owner裁决 |
| PICK-P2-012 | point ray是唯一query shape | 后续支持screen rect/lasso/frustum/sphere/volume query；Editor selection政策仍由Editor03拥有 |

## 7. 参考引擎对照与适用边界

| 参考 | 本地源码事实 | 对 Zircon 的约束 | 不照搬的内容 |
|---|---|---|---|
| Bevy Picking | `PointerId`区分Mouse/Touch/UUID Custom，location绑定NormalizedRenderTarget；backend hits有typed `extra`；plugin有Input/Focus/Backend/Hover/Events阶段；mesh backend使用visibility、RenderLayers、marker filter、early exit和真实ray cast；Click已有duration/count，hierarchy事件真实传播 | Zircon不能只复制DTO名称而缺pointer lifecycle、target extra、stage owner、真实backend和传播executor；现有文档关于click无count已落后于当前reference | Bevy RayMap同样是camera×pointer、order同样为f32；Zircon目标是解决这些扩展性/确定性问题，不以照搬为终点 |
| Unreal Hit Proxy / EditorViewport | renderer写唯一color ID与depth，支持custom/per-instance hit proxy buffer、masked/two-sided/material permutation、Nanite/instance culling；HHitProxy有typed class、refcount lifetime、priority/ortho priority与translucent policy；viewport click/tracking/cursor都消费真实proxy | visible pixel、instance/subobject identity和renderer lifetime必须由render owner发布；input tracking与hit result需在同一viewport lifecycle内 | 不复制C++全局ID表、裸UObject pointer、同步GPU stall或宏RTTI；使用generation、async readback与typed Rust owner |
| Godot Viewport/Physics Picking | viewport exit清mouseover但保留drag focus，window focus loss会drop focus；3D picking使用camera ray和physics space query，支持capture-on-drag、first-only、mouse enter/exit、shape index与handled gate | Zircon必须区分hover和capture，定义focus/viewport loss，允许physics backend与UI-first仲裁，不可把drag等同持续hover | 不把physics collider当所有renderable的唯一selection truth，也不复制其stereo picking禁用限制 |
| Fyrox Editor/Physics | PickingOptions包含editor-only、filter、backface、precise hull/coarse AABB、selection loop与settings；camera pick检查global visibility、prefab root、bounds/precise test；physics query使用broad phase、groups、max length、可复用ArrayVec result | backend request必须携visibility/filter/query policy，精确与fallback要可诊断；overlap cycling和allocation-free query可作为资格参考 | Fyrox editor仍递归graph并有`partial_cmp().unwrap()`等限制，不能作为Zircon极限规模或健壮性终点 |
| Unity Graphics | HDRP/URP提供独立ScenePicking/SceneSelection pass、DOTS/BRG instance ID、selection/object ID输出、alpha clip与non-jittered picking transform修正 | renderer backend必须和真实instancing、material alpha、LOD/变形、camera matrix policy一致，而不是投影owner中心圆 | Graphics仓库主要覆盖render pass，不提供完整input/capture/event authority；只用作GPU picking证据 |

五个参考共同证明，工程 Picking 不是一个 `ray -> Vec<hit>` helper。它至少需要输入/viewport lifecycle、可扩展目标身份、一个或多个真实 query backend、可见性与layer policy、generation-bound result、hover/capture/event状态、故障与诊断、以及Editor/Runtime UI/玩法的明确消费边界。Zircon可以比参考实现更确定、更异步、更少分配，但必须先完整拥有这些语义。

## 8. 目标架构

### 8.1 单帧权威链

```text
Platform/App input + Runtime12 UI arbitration
  -> PickingInputBatch(frame, surface, qualified pointers, ordered edges)
  +  PickViewSnapshotSet(view/camera/viewport/projection/origin generations)
  +  CompiledPickingBackendSet(capabilities/policy/budgets/provenance)
  -> PickingFrameCoordinator
       freeze input/view/backend generations
       build only requested ray/screen/region inputs
       dispatch sync CPU jobs and async GPU tickets
       validate/merge/dedupe backend results
       produce ResolvedPickingFrame
       advance hover/capture/click/drag state
       route capture/target/bubble events
  -> one immutable PickingFrameReceipt
       resolved hits + hover + capture + events
       backend/stage outcomes + diagnostics projection
  -> Editor selection/tools, Runtime UI, gameplay, remote devtools
```

每个consumer只读同一receipt。Editor可在自己的command/selection owner中解释target descriptor，但不能重新排序hits、重新做point test或自行维护第二套hover。Renderer/physics/UI backend只贡献generation-bound结果，不能直接触发selection mutation。

### 8.2 核心 identity 与 frame contract

建议最小typed组合，不要求这些名字原样落地：

```text
PickingFrameStamp = runtime/world + surface + frame sequence + config generation
QualifiedPointerId = principal + window/device/source + pointer id + generation
PickViewKey = surface + viewport + camera/view + generation
PickTargetKey = domain + owner handle + subobject key + generation
PickingBackendId = module owner + local backend key + generation
```

`ResolvedHit` 至少携 target/view/backend/frame、validated depth、typed position/normal space、pickability/layer、stable tie key与optional bounded detail。所有排序字段必须在admission时验证；任何跨generation组合都是typed rejection，而不是debug assertion。

### 8.3 Backend 协议

backend registration应声明：input kind、支持的view/target domains、sync/async latency、visibility/layer/alpha能力、top-only或all-hits completeness、ordering layer、最大results/bytes、thread affinity、cancel、reload和failure policy。每次dispatch返回ticket或immediate result；completion带frame/view/backend generation，过期自动discard并计数。

CPU mesh/physics backend应复用各自broad phase，不在Picking复制scene index。GPU backend由renderer拥有ID allocation、draw/instance映射、depth/material policy和readback。UI backend消费screen-space layout snapshot，不要求RayMap。Primitive sphere backend只保留test、gizmo coarse fallback或小规模overlay，并在capability/report中明确标记Coarse。

### 8.4 Pointer interaction state

状态按qualified pointer + surface/view域保存，至少包含last location/time、hover path、per-button press target、capture lease、gesture threshold、drag sources/drop targets、click series和terminal reason。事件路径来自generation-bound hierarchy snapshot；capture、target与bubble阶段分开，handled/default/stop信息进入receipt。target/view/pointer retirement、focus loss、backend unavailable与pipeline disable都走显式cancel/leave终态。

## 9. 硬切范围与禁止方案

1. 删除无产品caller的public `run_picking_pipeline` 外形，或将其降为coordinator私有stage；不得保留旧runner和新coordinator双authority。
2. `HitTarget` 从固定三分enum硬切到qualified extensible key；Editor私有route在迁移完成后删除，不保留双向shim长期存在。
3. `PointerHits` 必须携frame/view/backend identity；旧三字段constructor不得作为产品兼容入口保留。
4. 删除无consumer的capability装饰字段，改为registration compiler真实验证；不得仅增加更多enum值。
5. backend trait改为typed request/outcome/ticket；不得在sync trait里用内部thread/block_on伪装async GPU readback。
6. UI、physics、renderer、gizmo backend不得复制彼此的acceleration data；每个domain只发布最小immutable snapshot/result。
7. 不得用更大的全量Vec、全局Mutex、永久ID interner、无限event queue或hash iteration“稳定性”掩盖正确性问题。
8. 不得继续以raw `u64` owner/camera/pointer/viewport跨world、window、reload或remote session传递。
9. 不得让debug feed、Editor route和event state各自重新resolve同一hits；同一frame只有一个resolved projection。
10. 不得把sphere/AABB/owner-center fallback称为“精确Picking”；产品UI必须标明fallback/degraded状态。
11. 不得在无真实caller的单元测试中宣告产品完成；至少一个Editor或App end-to-end trace是每个milestone的必要门。
12. 不得以“比Unreal快”为理由删除hit proxy、material alpha、instance identity、capture或failure语义；性能比较只能在等价正确性之后进行。

## 10. 测试先行的重构里程碑

### M0 · Reachability 与行为 RED

先添加产品call-graph guard和behavior RED：Editor runtime input被消费、same-pointer双viewport不合并、duplicate target不重复事件、NaN/Inf不成为top、same-frame move/down/up按各自位置解释、target delete不ghost click、focus loss终结capture。同步修正文档中“已接入”的过度陈述。

### M1 · Qualified Identity 与 Frame Receipt

引入PickingFrameStamp、QualifiedPointerId、PickViewKey、PickTargetKey和BackendId，迁移hit/hover/report/event key。建立immutable `ResolvedPickingFrame/Receipt`，硬切raw三字段PointerHits产品入口。Runtime24负责通用handle构件，Runtime47负责领域组合。

### M2 · View、Projection 与 Ray Contract

接Runtime23 validated projection/origin contract，覆盖viewport rect、DPI、subviewport、custom projection、near/far、perspective/ortho、multi-camera与camera cut。RayMap变成可复用的request-derived view，不再无条件构造全笛卡尔积。

### M3 · Backend Registry 与 Failure Protocol

建立compiled backend set、capability验证、sync/async ticket、deadline/cancel、typed partial/fault、panic quarantine与generation retire。先迁移Primitive backend为Coarse/Test能力，再接一个physics或renderer-visible exact CPU backend。

### M4 · Renderer / Physics / UI 产品 Backend

按profile闭合至少：Editor renderer ID或exact mesh、physics collision、Runtime UI screen-space三类路径；包含visibility/layer/alpha/instance/subobject政策。缺少的profile必须报告Unavailable，不生成空成功。

### M5 · Hover、Capture、Gesture 与 Event Routing

实现input batch逐edge snapshot、capture lease、drag threshold、click time/count/suppression、target retirement与capture-target-bubble传播。对mouse/touch/pen分别建立oracle，确保UI先消费判词不变。

### M6 · Editor Hard Cut 与 Devtools

Editor controller改为提交input/view并消费receipt；删除硬编码identity、dead runtime_input carrier和私有重排序。Selection/Gizmo具体行为仍由Editor03实现。Debug面只投影receipt并显示backend/generation/failure/fallback。

### M7 · Scale、Fault 与竞争性证据

执行PERF-MVP-332矩阵和本报告correctness/fault矩阵：1/8/64 pointers、views、backends；1/100/10k hits；1k/100k primitives；sync/async latency、resize/camera cut/reload、125/500/1000Hz input。记录probes、alloc/realloc、clone bytes、event amplification、queue age/drop、CPU/GPU latency、RSS和p50/p95/p99。只有所有correctness gates通过后，才在同硬件、同scene、同可见性/alpha/instance语义下与参考引擎比较。

## 11. 资格门

| Gate | 验收内容 |
|---|---|
| G01 | 每个active surface/frame只有一个PickingFrameStamp与authoritative receipt |
| G02 | `run_picking_pipeline`、Editor resolver、UI/physics/render route不存在平行产品authority |
| G03 | Editor production input从host进入Picking并产生可消费event/selection receipt |
| G04 | same raw pointer id跨window/device/principal/generation不碰撞 |
| G05 | same pointer跨两个viewport/view的hover、location、capture完全隔离 |
| G06 | camera/view/target/backend stale generation全部fail closed并有diagnostic |
| G07 | hash seed、backend completion顺序与input insertion顺序不改变等价receipt hash |
| G08 | duplicate target按compiled merge policy只产生规定数量事件 |
| G09 | NaN、Inf、negative/out-of-range order/depth不能成为top/blocking target |
| G10 | custom projection、viewport rect、DPI、perspective/ortho、near/far ray parity通过 |
| G11 | typed hit position/normal/origin mismatch在resolve前拒绝 |
| G12 | mesh detail可保留instance/primitive/triangle/material等bounded证据 |
| G13 | backend capability实际控制request输入和registration admission |
| G14 | UI backend不依赖RayMap，ray disabled不使screen-space picking假Unavailable |
| G15 | sync backend error、panic、timeout、cancel都有bounded terminal且不越过API |
| G16 | async GPU结果只在matching frame/view/ID-table generation采用 |
| G17 | renderer picking覆盖visibility、instance、alpha/masked、LOD/deformation政策 |
| G18 | physics picking覆盖layer/mask、shape/subobject与capture-on-drag使用场景 |
| G19 | coarse sphere/AABB fallback在receipt和UI中明确标识 |
| G20 | zero-delta absolute move、coalesced motion与高频输入不丢hover/move语义 |
| G21 | same-frame move/down/move/up按各edge位置和时间选择目标 |
| G22 | capture后pointer离开viewport仍收到move/up；focus/window loss产生Cancel |
| G23 | drag threshold前不Drag，DragStart后按policy抑制Click |
| G24 | click duration/count、long press、multi-button、touch与pen矩阵通过 |
| G25 | non-blocking overlay与world target不会默认同时成为drag source |
| G26 | target delete/reparent/world reload不会产生ghost hover/click/drop |
| G27 | capture/target/bubble path generation一致，stop/default receipt可审计 |
| G28 | disable/quiesce/shutdown发布terminal transitions，不静默clear |
| G29 | NoHit、NoView、Unavailable、Partial、Stale、Fault、Timeout在产品UI可区分 |
| G30 | report/debug读取同一receipt，不重跑backend、不二次排序 |
| G31 | event/result有count+bytes+time admission，overflow不丢press/release/cancel edge |
| G32 | 22个既有测试迁移后保留语义，并增加production end-to-end与fault/property/fuzz矩阵 |
| G33 | PERF-MVP-332的workspace/probe/allocation/event amplification资格通过 |
| G34 | Editor03、Runtime12/23/24、Render04父finding保持原owner，不在本文重复关闭 |
| G35 | source fingerprint或参考引擎关键contract变化触发recheck |
| G36 | frontmatter路径、链接、severity/portfolio计数、LF/BOM/trailing-space和`git diff --check`通过 |

## 12. Owner 与依赖顺序

| 层 | Runtime47 owner | 依赖/交接 |
|---|---|---|
| L0 identity/frame | PickingFrameStamp、qualified pointer/view/target/backend、receipt | Runtime24 handle/generation；Runtime12 ingress |
| L1 view/math | PickViewSnapshot、request-derived rays、projection validation接收面 | Runtime23、Runtime37、Runtime06 |
| L2 backend | registry、capability、request/ticket/outcome、merge/dedupe、fault lifecycle | Runtime01/03/46；Render04、Runtime08A、Runtime09 |
| L3 interaction | hover、capture、click/drag/drop、retire、event routing | Runtime09 UI hierarchy；Runtime12 UI-first仲裁 |
| L4 product | Editor/App/Runtime UI接线、debug receipt、degraded projection | Editor03、App03、Runtime03 |
| L5 qualification | correctness/fault/end-to-end，性能矩阵回链 | PERF-MVP-332 |

依赖顺序必须是 M0 行为RED -> M1 identity/frame -> M2 view/ray -> M3 backend protocol -> M4 product backend -> M5 interaction -> M6 Editor hard cut -> M7 qualification。不能先把现有runner接进Editor，因为那会把跨viewport collapse、raw identity和不可失败sync backend固化为产品ABI。

## 13. 状态与产出记录

| 审查项 | 状态 | 日期 | 证据 |
|---|---|---|---|
| Picking production 23/23逐文件审查 | review_complete | 2026-08-19 | 2,069行；backend/ray/hit/hover/event/pipeline/report/debug全读 |
| Picking tests 6/6逐文件审查 | review_complete | 2026-08-19 | 889行、22 tests、0 ignored；未执行 |
| 产品caller与Editor bridge反查 | review_complete | 2026-08-19 | full pipeline/event/backend零production caller；仅`resolve_picking_outputs`被Editor消费 |
| Unreal/Bevy/Godot/Fyrox/Unity Graphics对照 | review_complete | 2026-08-19 | 本地源码锚见frontmatter与§7 |
| 工程差距登记 | review_complete | 2026-08-19 | 0 P0 / 48 P1 / 12 P2 / 36 gates |
| 生产重构与动态资格 | pending | - | 本篇不修改Rust或测试；G01-G36均未通过 |
