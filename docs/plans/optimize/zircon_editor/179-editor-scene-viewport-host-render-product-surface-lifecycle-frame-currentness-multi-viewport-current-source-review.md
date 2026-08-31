---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle
  - zircon_editor/src/ui/retained_host/app/viewport
  - zircon_editor/src/ui/retained_host/app/play_preview_redraw.rs
  - zircon_editor/src/ui/retained_host/app/play_viewport_pick.rs
  - zircon_editor/src/ui/retained_host/app/simulate_camera_sync.rs
  - zircon_editor/src/ui/retained_host/viewport
  - zircon_editor/src/ui/retained_host/host_contract/data/viewport_image.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/pane_context
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/viewport.rs
  - zircon_editor/src/ui/retained_host/host_contract/window
  - zircon_editor/src/ui/host/builtin_layout/builtin_shell_view_instances.rs
  - zircon_editor/src/ui/host/builtin_views/activity_views
  - zircon_editor/src/ui/workbench/state/editor_state_render.rs
  - zircon_editor/src/ui/workbench/view/view_instance_id.rs
  - zircon_editor/src/core/editor_event/workbench/view_instance_id.rs
  - zircon_editor/src/core/play/controller.rs
  - zircon_editor/src/core/play/mod.rs
  - zircon_editor/src/core/play/preview_frame.rs
  - zircon_editor/src/core/play/live_link.rs
  - zircon_editor/src/core/gateway
  - zircon_editor/src/scene/viewport
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_runtime_host/src/viewport_surface.rs
  - zircon_runtime_interface/src/runtime_api/session/requests.rs
  - zircon_runtime_interface/src/runtime_api/session/viewport.rs
  - zircon_runtime_interface/src/runtime_api/session/session_identity.rs
  - zircon_runtime/src/core/framework/render
  - zircon_runtime/src/graphics/runtime/render_framework
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface
tests:
  - zircon_editor/src/core/gateway/session/tests.rs
  - zircon_editor/src/core/play/tests.rs
  - zircon_editor/src/scene/viewport
  - zircon_editor/src/tests/host/retained_window/native_viewport_image.rs
  - zircon_editor/src/ui/retained_host/ui/tests/floating_windows.rs
  - zircon_editor/src/ui/retained_host/viewport/tests
  - zircon_editor/src/ui/retained_host/viewport/viewport_state_job_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/tests.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/optimize/zircon_editor/30-camera-asset-component-rig-controller-director-blend-shake-cinematic-cut-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/53-editor-interactive-tool-scheduler-resource-lease-input-capture-mode-modal-extension-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_editor/58-editor-scene-viewport-host-render-product-surface-lifecycle-frame-currentness-multi-viewport-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/120-editor-runtime-gateway-session-event-consumer-world-sync-generation-backpressure-reconnect-shutdown-current-source-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/11c-gpu-ui-renderer-atlas-sdf-batch-clip-submit-review.md
  - docs/plans/optimize/zircon_runtime/57-platform-host-window-registry-monitor-display-event-loop-application-lifecycle-surface-command-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/65-runtime-scalability-quality-profile-device-profile-capability-tier-dynamic-resolution-frame-budget-lod-feature-fallback-product-integration-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/EditorViewportClient.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/SEditorViewport.cpp
  - dev/UnrealEngine/Engine/Source/Editor/LevelEditor/Private/SLevelViewport.cpp
  - dev/godot/editor/scene/3d/node_3d_editor_viewport.h
  - dev/godot/editor/scene/3d/node_3d_editor_viewport.cpp
  - dev/godot/editor/scene/3d/node_3d_editor_plugin.h
  - dev/godot/editor/scene/3d/node_3d_editor_plugin.cpp
  - dev/Fyrox/editor/src/scene/mod.rs
  - dev/Fyrox/editor/src/preview.rs
  - dev/bevy/crates/bevy_render/src/camera.rs
  - dev/bevy/crates/bevy_render/src/view/mod.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/FrameData/UniversalCameraData.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/UniversalAdditionalCameraData.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.universal/Runtime/UniversalRenderPipeline.cs
doc_type: review-and-refactor-plan
refreshes:
  - docs/plans/optimize/zircon_editor/58-editor-scene-viewport-host-render-product-surface-lifecycle-frame-currentness-multi-viewport-product-integration-current-source-review.md
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# 179 · Editor Scene Viewport / Render Product / Surface Lifecycle / Frame Currentness / Multi-Viewport 当前源码复核

## 1. 结论与状态

Editor58定义的工程级视口产品链仍未闭合，但当前源码已经不再是“所有Scene和Game面板画同一张图”的原始状态。`HostViewportImageSet`现在分离`scene`、`simulate`和`game`三个图像槽；Play/Simulate会通过带`PlayInstanceId`的child-runtime gateway抓取默认viewport帧，Play输入只进入Play runtime，Simulate相机也能路由到复制世界。runtime的post-render viewport generation资格检查已前移到direct product publication和capture finish之前，`RenderFrameSubmissionReceipt`还会校验device generation、submission顺序和frame generation。这些是必须保留的真实基础。

决定性缺口仍在Editor产品边界。`RetainedEditorHost`只有一个`RetainedViewportController`、一个`viewport_size`和一个authoring submission；Scene图像仍按pane kind而不是`ViewInstanceId + window`选择，两个Scene、duplicate和floating pane继续共用相机、尺寸、target、generation cursor与last-good图像。Game/Simulate虽有独立像素槽，Play frame也已保留完整`GatewaySessionIdentity`、size和frame generation，却仍固定抓child runtime的default viewport，产品不携pane、camera、size epoch、request或present receipt。新增viewport-surface bind/unbind/present ABI在App/Gateway层是真实运输底座，但Editor UI和Play路径没有生产caller，不能把它当成per-pane session已经完成。

失败语义和能力真实性仍不合格。resize继续先销毁旧viewport，再create/configure新viewport；create/quality/submit失败会把`render_dirty`消费为false，旧host image没有`Stale/Degraded`状态或自动重试。`WorldSpaceUiSurfaceSubmission`继续公开world transform、meter size、billboard、depth和camera target，却只按screen rectangle生成Quad并做矩形hit test。因此本轮不增加或重排Editor58的canonical finding，只按当前源码重判其 **4项P0、56项P1、12项P2和46个资格门**：

| 等级 | Open / Fail | Partial | Closed / Pass |
|---|---:|---:|---:|
| P0 | 2 | 1 | 1 |
| P1 | 33 | 23 | 0 |
| P2 | 12 | 0 | 0 |
| Gate | 28 Fail | 15 Partial | 3 Pass |

本轮确认的最大进展是`ED58-P0-04`关闭；`ED58-P0-01`因Scene/Game/Simulate图像槽、Play/gateway identity和displayed-frame-qualified Simulate pick降为Partial。它们不改变目标架构：仍需以`ViewInstanceId + window + source binding`建立viewport session registry，以request/receipt冻结source epoch，以present lease证明pane看到的是哪一帧。

## 2. 冻结语料与currentness

### 2.1 物理选择集

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 本轮证据 |
|---|---:|---|
| Zircon Editor host/viewport/play/gateway、Runtime Interface/Host/App、render framework、RHI UI surface与聚焦测试 | **514 / 69,991 / 64,076 / 2,545,916 / 696 / 0** | 当前磁盘pane到render/present依赖闭包；fingerprint `36ed38d3ee1ba2db835f1b61a4ab448928326e3268e4eb0c68f25541757f3bd1` |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics | **14 / 31,013 / 26,550 / 1,231,129 / 0 / 0** | per-instance viewport、multi-view、preview target、retained view identity与per-camera frame data；fingerprint `0111f755b9352b3dc4cb9057f9f81c0de3c49da2df102ae127e836ef941c9bd7` |

统计按normalized relative path的ordinal顺序，将`lowercase path + NUL + raw bytes + NUL`串联后计算SHA-256；tests/ignored为词法属性计数，不是执行receipt。冻结时Git HEAD为`e6bfb5c0240fb62434c4ba86a1dc2525c0434d96`。共享工作树持续存在其他会话修改，因此本报告以当前磁盘fingerprint为审查锚点；实施前必须重算选择集并重判状态。

### 2.2 当前产品链真实性矩阵

| 产品链 | 当前事实 | 判定 |
|---|---|---|
| Pane identity | builtin Scene/Game各有`ViewInstanceId`，floating/native window能承载独立active pane | UI identity真实；未进入controller/request/product |
| Authoring Scene | 一个Editor controller生成world-generation-qualified extract和overlay，再提交到一个runtime viewport | 单Scene happy path真实；不是per-instance Scene产品 |
| Play/Simulate | child runtime gateway按完整gateway identity抓default viewport；frame保存instance/identity/size/generation，Play input、Simulate camera与displayed-frame-qualified pick有typed route | 独立source/currentness局部真实；固定viewport且无pane/camera/request/present receipt |
| Host image | `scene/simulate/game`三槽，Scene不会回退到Game，Game无帧时会清空 | kind级隔离真实；同kind所有实例仍共享一槽 |
| Runtime frame receipt | device/poll/scene/product/present ticket和frame generation有一致性校验 | renderer transaction真实；Editor产品DTO没有继承receipt provenance |
| Direct/capture | direct product与CPU capture都可用，runtime按resident consumer决定capture需求 | 路径真实；Editor共用一个generation cursor且无mode epoch |
| Surface ABI | gateway支持bind/unbind/present，host binding有transition guard和teardown release | 运输底座真实；Editor UI/Play生产路径无caller |
| Resize/recovery | destroy失败可恢复旧handle；zero input不create非法target；present retry保留请求 | 局部防护；create/quality/submit失败仍无原子切换和retry |
| World-space UI | DTO、Quad可见、capture和merge cache存在 | screen-space debug产品；3D、depth、camera和真实control均不真实 |

### 2.3 当前源码新增或修正证据

- `app.rs:679,693,774`仍只有一个viewport controller、一个size和一个render dirty；`render_submission.rs:36-70`只提交一份`EditorRenderFrameSubmission`，错误分支不保留dirty，成功后另行查询visible spatial snapshot。
- `viewport_image.rs:7-30`已经建立Scene/Simulate/Game三槽，`native_panes/viewport.rs:15`仍只按`pane.kind`选择；`builtin_shell_view_instances.rs:84-95`的Game/Scene payload仍是`Null`。
- `play_preview_redraw.rs:9-102`按Play/Simulate mode和kind级visibility抓帧或清槽；使用的尺寸仍是全局`self.viewport_size`。`preview_frame.rs:10-95`把`PlayInstanceId + GatewaySessionIdentity + size + frame generation`保存在frame identity中，Host image也保留该identity。
- `play/controller.rs:210-237`在transition gate内验证active Play domain，并用`capture_frame_at_identity`抓default viewport；`:240`之后只向Play runtime派发输入，Simulate相机仍只路由到default viewport。
- `app/play_viewport_pick.rs`会以当前显示Simulate帧的gateway identity、size和frame generation捕获pick route，并在replacement/session变化时cancel或retire；这是可信consumer基础，但仍不建立独立pane viewport产品。
- `core/gateway/session/viewport.rs`和`zircon_runtime_host/src/viewport_surface.rs`具备bind/release transition guard；`ZrRuntimeFrameRequestV1`仍只有ABI、viewport和size，Editor UI及Play源码对bind/unbind/present调用为0。
- `viewport_lifecycle.rs:32-52`先clear/destroy旧target，再create和set quality；quality失败会忽略新target destroy错误。`viewport_state_drop.rs:13`继续静默忽略destroy failure。
- `poll_viewport_product.rs`与`poll_captured_frame.rs`继续读取和更新同一个`latest_generation`；`RenderViewportProduct`仍只有resource key、width、height和generation。
- `submit.rs:183-207`、`submit_runtime_frame.rs:182-206`及`present_frame_extract.rs:226-247`均先post-render validate，再publish/finish；`viewport_product_registry.rs:45-55`还用frame receipt复核publication，因此旧P0-04已关闭。
- `world_space_ui.rs:187-246`只消费screen rect、用`depth_test/billboard`换颜色并执行矩形命中；world transform、meter、pixels-per-meter和camera target没有进入render/pick。merge cache只减少稳定帧重复分配，不使语义变真。
- `lifecycle.rs:218-307`会等待runtime presenter ready，升级失败后重建standalone presenter并记录warning；由于同一HWND约束仍先drop旧presenter，尚无prepare/commit或像素连续性receipt。

### 2.4 目标架构符号缺席

对生产Rust源码检索`EditorViewportSession`、`ViewportProductMap`、`ViewportRenderRequest`、`ViewportRenderReceipt`、`ViewportFrameProduct`、`ViewportPresentationState`、`ViewportPresentationLease`和`WorldUiSurfaceProduct`，定义与使用均为0。现有`GatewaySessionIdentity`、`RenderFrameSubmissionReceipt`、`ViewportSurfaceBindings`和`HostViewportImageSet`是可组合的局部基础，不能替代这些Editor产品合同。

## 3. P0：当前状态

### ED58-P0-01 · Partial · Scene、Game、duplicate与floating仍没有per-instance viewport产品

Scene/Game/Simulate不再无条件绘制同一张图，Play frame也携完整gateway origin，Simulate pick还能拒绝不属于当前显示帧的route，所以旧结论不能继续描述为“所有kind共享同一像素”。但Scene图像只有一个槽，所有Scene实例仍按kind命中；Host只有一个controller/size/submission，floating window只增加独立surface/focus，不增加camera/target/cursor。Game固定抓child runtime default viewport，既没有pane identity，也没有active camera、render request和present receipt。

必须建立`EditorViewportSessionRegistry`和`ViewportProductMap`，让每个`ViewInstanceId + window`拥有source、camera、size、quality、visibility、input与last-good状态。Scene/Game/duplicate/floating必须以session-qualified receipt消费产品，禁止继续用kind级三槽扩容来模拟multi-view。

### ED58-P0-02 · Open · resize/create/quality/submit失败仍破坏可用状态并停止重试

destroy失败会恢复旧handle是进展，但destroy成功后create/quality失败仍没有active viewport；host旧图没有stale/degraded标记。submit的`Err`只写diagnostic/status，随后`render_dirty=false`。这使瞬时失败停止重试，旧图继续以普通当前图绘制。

必须采用prepare/create/configure/warmup/first-frame/commit原子切换：旧target和last-good receipt保持有效，失败产生typed retry policy和pane状态；只有新target首帧qualified后才能替换。shutdown/destroy错误必须进入close receipt，不能丢弃。

### ED58-P0-03 · Open · world-space UI继续以screen-space Quad冒充完整3D产品

公开DTO仍包含world position/rotation/scale、meter size、pixels-per-meter、billboard、depth test和camera target，render却只读取viewport rectangle；depth/billboard只选择debug颜色，pointer只做rectangle hit并返回control id。没有world quad/mesh、camera projection、depth/occlusion、ray-to-UV、真实UI dispatcher或surface-qualified capture。

在真实`WorldUiSurfaceProduct`完成前，生产入口必须明确Unavailable。完成后必须从runtime UI tree生成独立纹理/mesh产品，以camera/layer参与合成，用ray/UV进入统一pointer/focus/capture服务，并按surface generation处理关闭与取消。

### ED58-P0-04 · Closed · runtime已在发布GPU产品前完成post-render generation资格检查

三条submit/present路径现在先捕获rendered frame generation，再调用`validate_viewport_generation`，之后才publish direct product或finish capture；registry publication还用`RenderFrameSubmissionReceipt::validate_viewport_product_publication`验证owner、sequence和frame generation。源码顺序测试也锁定validate在publish/finish之前。

关闭仅代表旧race已修复，不代表Editor currentness闭环完成。后续仍要把document/camera/settings/size/quality epoch和session request id写入Editor request/product/present receipt，并增加真实并发fault test，不能只保留source-shape断言。

## 4. P1：当前状态与重构要求

### 4.1 Viewport identity、instance与owner

| ID | 状态 | 当前证据与必须重构为 |
|---|---|---|
| ED58-P1-01 | Open | controller仍无`ViewInstanceId`、window、document或session id。让session registry成为pane到target的唯一owner。 |
| ED58-P1-02 | Open | runtime handle只代表allocation，未绑定editor kind/source/owner。以session generation包装handle lease。 |
| ED58-P1-03 | Open | Scene/Game builtin payload仍为`Null`。持久化camera、view mode、show flags、realtime、quality和source binding schema。 |
| ED58-P1-04 | Open | 一个`viewport_lifecycle: Arc<Mutex<()>>`只保护单controller，扩展多视口会全局串行。改为registry协调和per-session gate。 |
| ED58-P1-05 | Open | cloneable controller及presenter factory没有显式最终owner/close receipt。采用不可复制owner加受限lease。 |
| ED58-P1-06 | Open | Drop仍忽略destroy错误。close必须stop request、drain lease、typed destroy、deadline和receipt。 |
| ED58-P1-07 | Partial | host能按pane kind判断Game/Scene是否在主/浮动窗口active并跳过部分Play capture；没有per-instance occlusion/background/keep-warm authority。 |
| ED58-P1-08 | Open | 没有稳定view/subview identity；复用Bevy式main/aux/subview概念但绑定Zircon session。 |

### 4.2 Render request、receipt与frame currentness

| ID | 状态 | 当前证据与必须重构为 |
|---|---|---|
| ED58-P1-09 | Open | `EditorRenderFrameSubmission`仍只有extract/ui。补request id、session generation、dirty reason、deadline、cancel和policy。 |
| ED58-P1-10 | Partial | authoring extract携world generation，gateway也有session/transport generation；仍缺camera/selection/settings/size/quality epoch和统一冻结点。 |
| ED58-P1-11 | Open | `RenderViewportProduct`仍只有key/size/generation，无法证明当前source。扩成session/request/source-qualified frame product。 |
| ED58-P1-12 | Partial | runtime capture有profile/report/generation，Play frame保留完整gateway/instance/size/generation；仍缺editor pane/document/camera/request/present provenance。 |
| ED58-P1-13 | Open | poll仍只按renderer generation替换，不能拒绝成功但source过时的帧。按expected epoch资格化。 |
| ED58-P1-14 | Partial | Host invalidation mask和重复设置no-op提供局部reason/coalescing；未定义latest-wins、ordered和barrier request策略。 |
| ED58-P1-15 | Partial | runtime已有transaction/failure receipt和submission tickets；Editor没有accepted/queued/superseded/rendered/published/presented/dropped/failed完整状态机。 |
| ED58-P1-16 | Open | visible spatial snapshot仍在submit成功后另查，未与pane实际presented receipt绑定。 |

### 4.3 Surface、resize、device loss与恢复

| ID | 状态 | 当前证据与必须重构为 |
|---|---|---|
| ED58-P1-17 | Open | 每次有效尺寸变化仍destroy/create，无interactive debounce/settle/temporary resolution策略。 |
| ED58-P1-18 | Partial | zero frame不会生成size，controller内部又clamp到1；没有Minimized/Occluded/Suspended/Restoring状态机。 |
| ED58-P1-19 | Open | allocation和quality成功即安装target，没有first-frame-ready gate。 |
| ED58-P1-20 | Open | framework内部有typed error，host却压成字符串和单status；缺retryability/device-loss/OOM/terminal pane合同。 |
| ED58-P1-21 | Open | 没有device loss后active session重建、last-good、fallback和恢复receipt。 |
| ED58-P1-22 | Open | terminal present仍报告fatal并退出event loop，未按window隔离和保全authoring状态。 |
| ED58-P1-23 | Partial | presenter会等待runtime ready，失败时重建standalone并记录warning；切换仍先drop旧presenter且无像素连续性commit。 |
| ED58-P1-24 | Open | `take_error`仍消费单字符串，无incident、连续失败、backoff、history和recovery action。 |

### 4.4 Direct GPU、CPU capture与资源驻留

| ID | 状态 | 当前证据与必须重构为 |
|---|---|---|
| ED58-P1-25 | Open | direct/fallback仍是Host全局状态，不按window/presenter/session协商。 |
| ED58-P1-26 | Open | direct product和capture继续共用`latest_generation`，没有mode epoch。 |
| ED58-P1-27 | Partial | provider按viewport/presenter确认resident并计数consumer；没有pane/window/session generation生命周期资格。 |
| ED58-P1-28 | Partial | producer三代ring和presenter/cache持有能保护局部资源；仍无显式accept/present/release lease和stall deadline。 |
| ED58-P1-29 | Open | 64 MiB/256项cache拒绝只进入统计或被忽略，未驱动pane degraded/fallback。 |
| ED58-P1-30 | Partial | presenter refcount、per-presenter cache和shared texture ownership存在；缺多window present completion与安全回收receipt。 |
| ED58-P1-31 | Partial | 最后direct provider退出后capture predicate会恢复，但registry先清空全部产品且没有per-session fallback barrier。 |
| ED58-P1-32 | Open | product仍缺format/colorspace/HDR/alpha/sample/resolve/dynamic-resolution/display metadata。 |

### 4.5 Scene/Game、multi-view与window产品语义

| ID | 状态 | 当前证据与必须重构为 |
|---|---|---|
| ED58-P1-33 | Partial | Game frame已来自active Play gateway并保留完整gateway/instance identity，不再复制authoring Scene；仍固定default viewport，无active game camera、pane/request/present receipt和typed unavailable state。 |
| ED58-P1-34 | Open | Scene仍只有一个controller/camera/mode/show flags/realtime状态，重复实例无法不同视角。 |
| ED58-P1-35 | Open | recompute只读取一个main `viewport_content_frame`并更新全局size。建立per-pane measured target epoch。 |
| ED58-P1-36 | Partial | floating window有独立native presenter、surface和focus；仍按kind消费共享Scene/Game image，没有独立cadence。 |
| ED58-P1-37 | Open | 没有1/2/3/4 viewport session layout、split persistence和per-cell restore。 |
| ED58-P1-38 | Partial | Scene/Game/Simulate有分图像槽、mode-specific input和kind visibility；source/camera/focus/budget仍非完整隔离。 |
| ED58-P1-39 | Partial | gateway有session/transport generation、surface ABI和identity-qualified child-runtime capture；Play frame及Simulate pick保留gateway/instance/size/generation。仍没有pane request、latency/drop/present receipt，也没有UI surface caller。 |
| ED58-P1-40 | Partial | Simulate可把editor camera路由到child runtime且跳过未变化值；没有通用preview/pilot/cinematic binding、return和污染隔离协议。 |

### 4.6 World-space UI、overlay composition与input

| ID | 状态 | 当前证据与必须重构为 |
|---|---|---|
| ED58-P1-41 | Open | submission仍由host template派生，不是runtime component/UI asset qualified产品。 |
| ED58-P1-42 | Open | world transform、meter size和pixels-per-meter没有参与layout/raster/projection/LOD。 |
| ED58-P1-43 | Open | depth/billboard只改变debug颜色。实现真实depth attachment/occlusion和camera-facing orientation。 |
| ED58-P1-44 | Open | `camera_target`未消费。绑定真实camera/eye/layer/target并返回typed invalid-target。 |
| ED58-P1-45 | Open | pointer只做screen rect命中。实现ray-plane/mesh intersection、UV、clip和occlusion qualification。 |
| ED58-P1-46 | Open | capture仍保存克隆submission且全局唯一，缺pointer/button/window/viewport/surface generation/cancel reason。 |
| ED58-P1-47 | Open | overlay仍直接`commands.extend`，缺tree/node namespace、z/clip domain、冲突和容量验证。 |
| ED58-P1-48 | Partial | `WorldSpaceUiMergeCache`会复用稳定generation/base Arc，减少无变化帧重建；仍无surface/pixel/command/text/update/distance预算。 |

### 4.7 Diagnostics、调度、性能与跨报告边界

| ID | 状态 | 当前证据与必须重构为 |
|---|---|---|
| ED58-P1-49 | Partial | lazy resolve已有job ticket、cancel token和有界job系统基础；仍使用`JobCategory::Misc`，无startup deadline/priority/terminal degraded产品状态。 |
| ED58-P1-50 | Partial | Host invalidation mask记录render/presentation等原因并合并重建；最终submission gate仍是一个bool，不能表达source/camera/size/quality request policy。 |
| ED58-P1-51 | Partial | runtime有frame profile、submission receipt、UI presenter/cache counters；没有per-session端到端CPU/GPU/queue/capture/present-age/stale/drop/recovery指标。 |
| ED58-P1-52 | Partial | process diagnostics、structured log和warning存在；用户状态仍是全局字符串且不含pane/session/incident。 |
| ED58-P1-53 | Open | editor私有`editor-viewport-default`、环境变量和固定GI预算继续绕过Runtime65 qualified profile authority。 |
| ED58-P1-54 | Partial | runtime/UI cache有有界预算，隐藏Game/Simulate可跳过capture；没有viewport总VRAM/带宽/并发/background/fairness admission。 |
| ED58-P1-55 | Open | 选择集虽有大量单元测试，关键链仍多用include_str/source-shape且缺真实presenter、fault、multi-view和source-currentness行为门。 |
| ED58-P1-56 | Partial | gateway identity、Play displayed-frame pick、runtime transaction receipt和resource key提供局部追踪；仍无法从pane定位request、runtime viewport、frame、resource和present receipt。 |

## 5. P2：长期成熟度

| ID | 状态 | 当前差距与方向 |
|---|---|---|
| ED58-P2-01 | Open | 没有per-session viewport preset/profile。 |
| ED58-P2-02 | Open | 没有per-viewport screenshot/record/frame comparison receipt。 |
| ED58-P2-03 | Open | 没有统一pixel/HDR/depth/normal/object-id inspect产品。 |
| ED58-P2-04 | Open | 没有safe area、resolution/aspect preset、device frame和letterbox语义。 |
| ED58-P2-05 | Open | 没有color management、display transform和per-monitor HDR选择。 |
| ED58-P2-06 | Open | 没有bookmark、camera history与layout/session migration。 |
| ED58-P2-07 | Open | 没有deterministic viewport render request capture/replay。 |
| ED58-P2-08 | Open | 没有render product schema/compatibility version。 |
| ED58-P2-09 | Open | 没有custom producer/overlay/input/budget extension API。 |
| ED58-P2-10 | Open | 没有向辅助技术暴露viewport状态、错误、camera和tool feedback。 |
| ED58-P2-11 | Open | 没有multi-adapter/multi-GPU/cross-device明确unsupported或bridge策略。 |
| ED58-P2-12 | Open | 没有generation/cache/resize/detach长期soak资格。 |

## 6. 本地参考源码对照

| 参考 | 直接读取到的工程机制 | 对Zircon的约束 |
|---|---|---|
| Unreal | `SEditorViewport`逐实例调用`MakeEditorViewportClient`并创建`FSceneViewport`；`FEditorViewportClient`不可复制，持有自己的view state、realtime override、input、viewport type与invalidate | Editor widget/pane必须消费自己的client/session，camera、realtime、input和present target不能是Host全局字段 |
| Godot | `Node3DEditorViewport`逐实例持有`SubViewportContainer`、`SubViewport`和`Camera3D`；plugin明确维护4个viewport并支持状态保存/恢复 | 1/2/3/4布局必须创建对应数量的真实camera/target/session，不是把一张图画四次 |
| Fyrox | scene editor拥有camera controller和scene render target；`PreviewPanel`另建scene、camera与render target并按frame size重建 | Scene、Game、Preview必须是不同producer/owner；target resize归属于具体产品实例 |
| Bevy | camera extract携target、viewport、order、output mode、HDR；`RetainedViewEntity`用main/auxiliary/subview index形成稳定identity，`ViewTarget`按normalized target管理attachment | runtime view identity、target metadata和subview必须进入request/receipt，不能只传opaque resource key |
| Unity Graphics | `UniversalCameraData`含render type、target texture/descriptor、pixel rect、SceneView标识和resolve-final-target；Base/Overlay stack有明确约束 | 每camera/per-view frame data应明确format、HDR、size、resolve与stack关系；本地语料不代表闭源Unity Editor内部实现 |

采用结论仍是Unreal per-instance editor viewport owner为主架构，Godot验证multi-view实例化，Fyrox验证preview/scene target ownership，Bevy和Unity Graphics约束runtime view identity与frame metadata。Zircon可保留Rust immutable snapshot、generation guard、direct/capture双路径和typed gateway优势，但必须用自己的性能、故障和currentness门证明“优于虚幻”，不能从参考API形状推导结论。

## 7. 目标架构与唯一owner

### 7.1 必备合同

| 合同 | 唯一职责 |
|---|---|
| `EditorViewportSessionId` | project/document或play instance、window、`ViewInstanceId`、session generation |
| `EditorViewportDefinition` | Scene/Game/Preview/custom kind、source/camera binding、persistence schema、capability |
| `EditorViewportSession` | target/input/visibility/settings/dirty/lifecycle/last-good owner |
| `ViewportRenderRequest` | request id、全部source epoch、target、deadline、coalescing policy |
| `ViewportRenderReceipt` | accepted到presented/failed的状态、timing、typed failure和完整provenance |
| `ViewportFrameProduct` | session/request、format/colorspace/HDR/size、direct/capture payload和resource lease |
| `ViewportPresentationState` | Starting/Current/Stale/Degraded/Lost/Suspended/Closing及reason/remediation |
| `ViewportPresentationLease` | consumer/window/presenter、resource generation、accept/present/release fence和deadline |
| `ViewportProductMap` | 只按`ViewInstanceId + window`查询产品，禁止global/kind-only image authority |
| `WorldUiSurfaceProduct` | UI tree generation、world transform、camera/layer、depth/billboard、texture/mesh和ray-UV endpoint |

### 7.2 Owner边界

- Editor03继续拥有authoring world、selection/picking/Gizmo和camera navigation；Editor58只提供session-qualified input/product receipt。
- Editor07继续拥有Play process/world/pause/eject/recovery；Editor58只绑定play instance/camera/frame consumer。
- Editor30/Runtime37拥有camera asset/rig/director/cut；viewport只消费camera binding revision。
- Editor53拥有通用tool lease和pointer capture；viewport route必须携session/window/surface generation并消费共享authority。
- Runtime09a/09b拥有renderer/RHI/render graph/visibility和GPU lifetime；Editor58定义端到端currentness与consumer资格。
- Runtime57拥有window/surface/device lifecycle；Editor不能建立第二套platform loop。
- Runtime65拥有quality/device profile/dynamic resolution/frame budget；Editor request只能引用qualified profile/epoch。
- Runtime11c/79拥有UI renderer/cache/clip/batching；world UI必须建立在该产品上，不复制第二套UI renderer。

### 7.3 原子状态流

`Pane activate -> resolve/create session -> freeze source epochs -> prepare target -> render -> post-render qualify -> publish product lease -> pane/session match -> presenter accept -> present receipt -> commit last-good`。

任何失败都必须返回typed receipt。resize、quality、device replacement和direct/capture switch使用prepare/commit，旧产品保持`Current`或明确降为`Stale`，直到新产品首帧presented。close必须先停止新request，再cancel/drain queue和lease，最后destroy并发布close receipt。

## 8. 必须硬切的旧路径

1. 删除Host单一controller/size对所有Scene实例的authority，迁移后禁止kind-only产品查询。
2. 删除Scene/Game descriptor的空payload，改为versioned viewport definition和session restore。
3. 删除direct/capture共享generation cursor，改为per-session per-mode epoch。
4. 删除resize先destroy再create的owner切换，改为双target prepare/commit。
5. 删除submit error后清空render dirty的行为，typed retry或terminal state必须保留原因。
6. 删除全局status和一次性string error作为viewport failure authority。
7. 删除固定三代ring充当consumer lease的假设，保留ring作为cache实现细节。
8. 删除`editor-viewport-default`私有quality authority，接入Runtime65 profile decision。
9. 删除world-space UI screen-rect debug生产入口和相关“完整功能”命名；未实现前返回Unavailable。
10. 禁止为Game复制authoring Scene或仅增加更多kind槽，必须绑定Editor07 play product。
11. 禁止把gateway surface ABI当作pane session；它只能是session产品的transport adapter。
12. 禁止以source-shape、截图或resource generation代替source epoch、fault、lease和present receipt证据。

## 9. 分层重构里程碑

### M0 · 能力真实性与currentness封口

- 保持P0-04顺序修复并补真实并发fault test。
- world-space UI生产入口切Unavailable。
- Game无active play product时显示typed unavailable。
- 旧图增加Stale/Degraded和retry状态。

### M1 · Session registry与per-pane product map

- 引入session id/definition/registry，以`ViewInstanceId + window`创建owner。
- 先通过两个Scene实例不同camera/size/input/product，再接Game和floating。
- 删除global/kind-only image authority。

### M2 · Request/receipt/source currentness

- 冻结document/world/camera/selection/settings/size/quality epoch。
- 实现latest-wins、ordered、barrier、supersede和typed failure。
- visible spatial snapshot与同一presented receipt提交。

### M3 · Atomic target与恢复状态机

- prepare/configure/warmup/first-frame/commit resize和quality change。
- 建立zero/minimized/occluded/device-lost/closing状态。
- presenter upgrade/fallback和shutdown纳入同一receipt。

### M4 · Resource lease与present completion

- direct/capture独立cursor/mode epoch。
- producer ring、presenter cache、shared registry接入accept/present/release lease。
- admission reject驱动pane fallback/degraded。

### M5 · Scene/Game/multi-window产品硬切

- 接Editor07 play session和camera，支持Scene+Game同时可见。
- 支持duplicate/floating和1/2/3/4布局恢复。
- 建立per-session cadence、后台节流和总预算。

### M6 · 真实world-space UI

- runtime UI tree生成texture/mesh产品。
- camera projection、depth、billboard、ray/UV、focus/capture与surface lifecycle闭环。
- 删除旧screen rectangle实现及兼容shim。

### M7 · 资格、性能与长期soak

- fault/device/resize/multi-window/remote play矩阵。
- 1/2/4视口CPU/GPU/queue/capture/present/VRAM预算。
- 30分钟及更长soak、审计回放与跨平台资格。

## 10. 46个资格门当前重判

| Gate | 状态 | 当前判定 |
|---:|---|---|
| 1 | Partial | Scene/Game槽和Play gateway identity不同，但没有独立pane session/request/present receipt。 |
| 2 | Partial | 无active Play帧时会清Game槽；没有typed unavailable产品状态。 |
| 3 | Fail | 两个Scene不同camera没有独立像素/metadata证据。 |
| 4 | Fail | duplicate/floating Scene仍共享controller、size和generation。 |
| 5 | Partial | 主/浮动窗口有独立presenter和surface；仍按kind消费共享图像。 |
| 6 | Fail | 无1/2/3/4 session layout和恢复。 |
| 7 | Partial | kind级active visibility可跳过部分Play capture；非per-session suspend/throttle。 |
| 8 | Partial | world、gateway和Play frame generation局部存在，缺document/camera/settings/size/quality request epochs。 |
| 9 | Fail | 产品没有全部expected epoch，无法资格为Current。 |
| 10 | Partial | runtime拒绝viewport generation race，Simulate pick也绑定显示帧identity；Editor render product仍无source supersede receipt。 |
| 11 | Fail | visible spatial snapshot未绑定同一presented receipt。 |
| 12 | Partial | last-good像素可能继续显示且destroy失败可恢复handle；没有Stale/Degraded和自动retry。 |
| 13 | Partial | quality失败的新target不会安装，但旧target已经销毁且无原子rollback/retry。 |
| 14 | Fail | submit瞬时失败会消费dirty。 |
| 15 | Fail | 无incident-backed terminal Degraded/Lost状态。 |
| 16 | Partial | zero frame不create非法target并保留局部active状态；没有显式session state。 |
| 17 | Fail | 无interactive resize policy。 |
| 18 | Fail | 无device-loss session rebuild资格。 |
| 19 | Pass | post-render viewport generation变化时先validate，direct registry不会发布新descriptor。 |
| 20 | Pass | direct/capture publication和success record/stats位于post-render qualification之后。 |
| 21 | Fail | direct/capture仍共用cursor且无mode epoch。 |
| 22 | Partial | upgrade失败会恢复standalone并记录warning；无qualified product连续性receipt。 |
| 23 | Partial | last provider退出后capture predicate恢复；没有per-session fallback barrier。 |
| 24 | Partial | ring/cache可持有已接受资源；未接受stall没有lease/deadline/fallback。 |
| 25 | Fail | cache reject未传播pane receipt。 |
| 26 | Partial | per-presenter cache/refcount存在；无多window present/release completion。 |
| 27 | Fail | direct product缺format/colorspace/alpha/dynamic-resolution/resolve metadata。 |
| 28 | Fail | 无HDR/SDR/per-monitor display transform currentness。 |
| 29 | Pass | retryable surface acquire保留同一present request和qualified draw list。 |
| 30 | Fail | terminal present仍退出event loop，未先形成window隔离恢复receipt。 |
| 31 | Fail | close无stop/drain/lease deadline/close receipt。 |
| 32 | Fail | Drop destroy失败仍被静默吞掉。 |
| 33 | Fail | world-space UI生产入口仍画debug Quad，未Unavailable。 |
| 34 | Fail | world transform/meter/pixels-per-meter不参与产品生成。 |
| 35 | Fail | billboard/depth无相机朝向和遮挡证据。 |
| 36 | Fail | camera target不决定实际投影。 |
| 37 | Fail | pointer不是ray/UV且遮挡面仍可抢输入。 |
| 38 | Fail | capture未按pointer/window/viewport/surface generation限定。 |
| 39 | Fail | overlay无稳定namespace和command/pixel预算。 |
| 40 | Fail | 无1/2/4视口完整性能门。 |
| 41 | Partial | kind visibility和cache预算提供局部节流；无per-session fairness/总预算。 |
| 42 | Partial | invalidation reason和重复设置no-op提供局部coalescing；无request policy。 |
| 43 | Fail | 无30分钟多window resize/mode-switch soak。 |
| 44 | Fail | 无create/quality/submit/cache/surface/device全链fault injection。 |
| 45 | Fail | 无pane到present receipt的端到端审计。 |
| 46 | Fail | 未取得Windows native主路径完整执行资格。 |

## 11. 本轮验证边界与后续入口

本轮只做current-source静态review、参考源码对照、状态重判和文档记录，未修改Rust、Cargo、assets或Tooling。共享工作树在复核期间新增Play frame provenance和Simulate displayed-frame pick，本报告已按最终fingerprint重新审查；没有运行Cargo、Editor、native GPU presenter、Play child process、fault injection、device loss、multi-window、HDR、soak或benchmark，因此不宣称当前基线可构建、测试green或性能优于Unreal。按用户要求，本轮没有查询、轮询、等待或实时跟踪协调器。

实施入口按依赖顺序为：先保持P0-04的runtime资格顺序并补行为fault test，同时把伪world-space UI切Unavailable；随后建立per-pane session/product map和Stale/Degraded状态，再做原子resize/request receipt；最后接Game/multi-window、resource lease和真实world UI。任何实现不得越过Editor03/07/30/53与Runtime09a/09b/11c/57/65/79的唯一owner边界。
