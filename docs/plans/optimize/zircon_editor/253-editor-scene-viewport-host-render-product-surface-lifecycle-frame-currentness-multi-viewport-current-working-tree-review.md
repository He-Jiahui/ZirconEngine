---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle
  - zircon_editor/src/ui/retained_host/app/viewport
  - zircon_editor/src/ui/retained_host/app/play_preview_redraw.rs
  - zircon_editor/src/ui/retained_host/app/play_viewport_pick.rs
  - zircon_editor/src/ui/retained_host/app/viewport_image_redraw.rs
  - zircon_editor/src/ui/retained_host/app/native_windows
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/viewport.rs
  - zircon_editor/src/ui/retained_host/viewport
  - zircon_editor/src/ui/retained_host/host_contract/data/viewport_image.rs
  - zircon_editor/src/ui/retained_host/host_contract/globals/pane_context
  - zircon_editor/src/ui/retained_host/host_contract/globals/state.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/viewport.rs
  - zircon_editor/src/ui/retained_host/host_contract/window
  - zircon_editor/src/core/play
  - zircon_editor/src/core/gateway
  - zircon_editor/src/scene/viewport
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_runtime_host/src/viewport_surface.rs
  - zircon_runtime_interface/src/runtime_api/session
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
plan_sources:
  - docs/plans/mvp/index.md
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/58-editor-scene-viewport-host-render-product-surface-lifecycle-frame-currentness-multi-viewport-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/179-editor-scene-viewport-host-render-product-surface-lifecycle-frame-currentness-multi-viewport-current-source-review.md
  - docs/plans/optimize/zircon_editor/250-editor-camera-viewport-pilot-preview-cut-capture-current-working-tree-review.md
  - docs/plans/optimize/zircon_editor/252-editor-render-graph-frame-debugger-capture-lighting-bake-reflection-probe-post-process-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/191-runtime-platform-host-window-registry-display-event-loop-application-lifecycle-surface-command-current-working-tree-review.md
  - docs/plans/zircon_runtime/render/16/failure-2026-07-17-editor-viewport-synchronous-readback.md
  - docs/plans/zircon_runtime/render/17/failure-2026-07-29-scene-viewport-surface-projection-drift.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-viewport-fallback-scene-rebuild-under-live-frame.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-viewport-toolbar-surface-rebuild-storm.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-18-runtime-ui-surface-frame-full-copy-and-ecs-reprojection.md
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
report_id: Editor253
refreshes:
  - docs/plans/optimize/zircon_editor/58-editor-scene-viewport-host-render-product-surface-lifecycle-frame-currentness-multi-viewport-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/179-editor-scene-viewport-host-render-product-surface-lifecycle-frame-currentness-multi-viewport-current-source-review.md
review_status: current_working_tree_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# 253 · Editor Scene Viewport / Host Render Product / Surface Lifecycle / Frame Currentness / Multi-Viewport 当前工作树复核

## 1. 结论

当前 Scene Viewport 已有若干真实底座，不能再描述为纯占位：`HostViewportImageSet`分离 Scene/Simulate/Game；authoring Scene 有direct GPU product与显式CPU capture fallback；Play frame保留`PlayInstanceId + GatewaySessionIdentity + size + generation`；Simulate pick会绑定实际显示帧；Runtime在publish/finish前执行post-render viewport generation资格检查；`ViewportSurfaceBindings`也具有bind/release transition guard和确定性teardown。

但这些局部能力仍没有组成工程级viewport产品。决定性问题不是某个按钮或某条render pass缺失，而是 **layout `ViewInstanceId`、native floating `MainPageId/UiHostWindow`、Runtime `RenderViewportHandle`三套身份从未进入同一个session/request/product/present transaction**。当前Host只有一个`RetainedViewportController`、一个`viewport_size`和一个`render_dirty`；图像只按pane kind查询，Game/Simulate固定抓child runtime default viewport；pointer callback保留了source window，却丢弃pane instance、surface generation和displayed product identity。

本轮新增的关键current-working-tree证据是：native floating window确实拥有独立`UiHostWindow`、presentation、callback source和focus，但每个`HostContractState::new`也会初始化一份独立空`HostViewportImageSet`；authoring/Play图像只通过`self.ui.global::<PaneSurfaceHostContext>()`写入主Host state。`NativeWindowPresenterStore`的生产patch路径只覆盖structure、viewport chrome和UI Asset pane，没有显式的per-window viewport image/product publication。因此不能继续把“独立native presenter存在”等同于“floating Scene/Game已有独立或共享live frame产品”；静态结论只能是 **没有找到显式图像发布路径**，动态是否显示旧图、空图或fallback仍需真实窗口测试。

失败与能力真实性仍不合格：resize先销毁旧target；create/quality失败会失去active target；render submit `Err`会消费dirty且不重试；direct/capture共用一个generation cursor；world-space UI仍把world DTO投影成screen-space debug Quad。故Editor58的canonical finding不新增、不改号，按当前工作树重判仍为：

| 等级 | Open / Fail | Partial | Closed / Pass |
|---|---:|---:|---:|
| P0 | 2 | 1 | 1 |
| P1 | 33 | 23 | 0 |
| P2 | 12 | 0 | 0 |
| Gate | 28 Fail | 15 Partial | 3 Pass |

`ED58-P0-04`保持Closed；其余P0和全部产品化里程碑不得因direct texture、三图像槽、native window壳或Gateway ABI存在而提前关闭。当前也没有任何动态证据可支持“性能或表现优于Unreal”。

## 2. 审查冻结点

### 2.1 当前磁盘选择集

| 范围 | files | lines | non-empty | bytes | test markers | ignored | fingerprint |
|---|---:|---:|---:|---:|---:|---:|---|
| Editor host/viewport/play/gateway + App/Runtime Interface/Host/render/RHI focused closure | **1,000** | **142,602** | **130,033** | **5,095,346** | **1,500** | **0** | `dd07985f398990274fa969444d8d2fcb9bc51c86fd00f2c8f01aa425139f55f4` |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics selected references | **14** | **31,013** | **26,550** | **1,231,129** | n/a | n/a | `e9817f7e4b082b86e77e0f0886e6b1b57689abb25116533b98fd58f919c2e8e6` |

统计对去重后的normalized relative path按ordinal排序，将`lowercase path + NUL + raw bytes + NUL`串联后计算SHA-256；test/ignored是词法marker，不是执行receipt。冻结时Git HEAD为`cc5cadbd597c3707954ebd6109fad0fd5643a152`，但共享工作树含大量其他修改，因此本报告以当前磁盘fingerprint为审查锚点，实施前必须重算。

### 2.2 当前产品链真实性

| 链路 | 当前真实能力 | 工程级缺口 |
|---|---|---|
| Pane/layout | builtin pane拥有`ViewInstanceId`，floating surface拥有`MainPageId` | viewport controller、callback、request、product和image map均不携`ViewInstanceId` |
| Native window | 每个floating target创建独立`UiHostWindow`，保留source-window callback与focus | window不是Runtime191-qualified `WindowId/SurfaceLease`；没有显式per-window viewport image publication |
| Authoring Scene | 单controller能submit world-qualified extract，direct/capture均有产品 | 所有Scene共享handle、size、camera、cursor、last error和last-good image |
| Play/Simulate | child gateway capture保留instance/gateway/size/frame generation；Simulate pick拒绝非显示帧 | capture/input/camera/pick固定default viewport；没有pane/session/request/present identity |
| Host image | Scene/Simulate/Game三槽，避免跨kind错误fallback | `for_pane(pane.kind)`是kind-global查询；产品缺document/camera/window/request/format/present receipt |
| Runtime receipt | frame submission和viewport publication有device/submission/frame generation检查 | Editor转换为`HostViewportImageData`后丢失Runtime receipt provenance |
| Surface transport | Gateway和Runtime Host有typed bind/unbind/present与transition guard | Editor Gateway目录外生产caller为0；ABI仍不是Editor per-pane session |
| Recovery | destroy失败可恢复旧handle；lazy backend未ready会保留dirty | create/quality失败丢active target；submit error清dirty；无Stale/Degraded/retry/incident |
| World UI | DTO、merge cache和可见debug绘制存在 | world transform/camera/depth/billboard/ray-UV均未实现，screen rect冒充3D产品 |

### 2.3 精确源码证据

- `app.rs:678,694,748,776`显示Host仍只有一个viewport controller、一个viewport size、一个native window store和一个render dirty bit。
- `viewport_state.rs:15-29`只有单`ActiveViewport`、单`latest_generation`、单字符串error与全局world-UI capture；`poll_viewport_product.rs`和`poll_captured_frame.rs`都读取并更新同一cursor。
- `viewport_lifecycle.rs:32-52`先clear/destroy旧viewport，再create/configure/install；create或quality失败后旧viewport已不可用，新target destroy失败也被忽略。
- `render_submission.rs:35-70`仅`Ok(false)`保留dirty；`Err`记录日志后最终写`render_dirty=false`。现有source-shape test只验证写error，没有验证retry与last-good currentness。
- `viewport_image.rs:10-36`只有Scene/Simulate/Game三槽，`HostViewportImageData`没有pane/window/source/request/present字段；native pane renderer在`viewport.rs:14-16`只按`pane.kind`选择。
- `viewport_image_redraw.rs:4-24`和`play_preview_redraw.rs:9-108`只写主`self.ui`的global state并重绘主viewport frame。每个native `UiHostWindow`在`HostContractState::new`中得到独立default image set。
- `native_windows/store.rs`创建、隐藏并patch独立native windows；生产专用patch只有Scene chrome和UI Asset pane。`native_window_presenters/presentation.rs`只构造presentation/chrome，没有图像/product参数或复制步骤。
- `callback_wiring.rs:6-15`能把source `UiHostWindow`解析为`MainPageId`；但viewport callback签名仅传kind/button/coordinates/modifiers，selected product、pane instance和surface generation均丢失。
- `play/controller/preview_routing.rs:16-43,120-128`抓帧与Simulate camera都硬编码`ZIRCON_RUNTIME_DEFAULT_VIEWPORT_HANDLE_V1`；Game input和Simulate pick同样依赖default viewport。
- 对viewport product/callback focused文件精确检索，`ViewInstanceId`、`MainPageId`、Runtime `WindowId`和`SurfaceLease`均为0；对Editor生产源码排除`core/gateway`与tests后，`bind_viewport_surface`、`unbind_viewport_surface`、`present_viewport`调用文件均为0。
- `world_space_ui.rs:187-247`只读取viewport rectangle，生成`UiRenderCommandKind::Quad`并做矩形topmost hit；world transform、meter size、pixels-per-meter和camera target没有进入render/pick。

### 2.4 必须保留的局部基础

1. `PlayPreviewFrameIdentity`的instance/gateway/size/generation provenance和displayed-frame-qualified Simulate pick。
2. Runtime post-render generation-before-publication顺序，以及`RenderFrameSubmissionReceipt`的device/submission/frame generation验证。
3. direct GPU product作为正常路径、CPU capture作为显式fallback的方向；禁止退回每帧同步readback。
4. `ViewportSurfaceBindings`的transition guard、rollback与deterministic teardown，但只能作为transport adapter。
5. Host image Arc共享、重复resource key no-op、bounded cache/ring与world-UI merge cache等局部分配优化。

这些基础只能降低后续重构成本，不能替代Editor viewport session、产品currentness和presentation lease。

## 3. P0重判

### ED58-P0-01 · Partial · Scene/Game/duplicate/floating仍没有per-instance viewport产品

Scene/Simulate/Game分槽、Play provenance和Simulate displayed-frame pick都是真实进展，故不回退为Open。但Scene仍是单controller/size/submission；同kind duplicate继续命中一个槽；floating window虽有独立Host state，却没有显式image publication。Game固定default viewport，不携active camera、pane、request或present receipt。

必须引入`EditorViewportSessionRegistry + ViewportProductMap`，以`ViewInstanceId + qualified WindowId + source binding + session generation`形成唯一owner。禁止继续增加kind槽来模拟multi-view。

### ED58-P0-02 · Open · resize/create/quality/submit失败仍破坏可用状态并停止重试

destroy失败恢复旧handle只是局部保护。destroy成功后的create/quality失败会留下空active target；submit error又消费dirty。旧图没有`Current/Stale/Degraded/Lost`状态，用户无法区分当前帧、last-good帧与永久停更。

必须采用prepare/create/configure/warmup/first-qualified-frame/commit；旧target和last-good lease在commit前保持有效。所有失败形成typed retry/terminal receipt，包含incident、backoff、deadline和remediation。

### ED58-P0-03 · Open · world-space UI仍由screen-space debug Quad冒充完整3D产品

公开DTO声称支持world transform、physical size、billboard、depth和camera target，生产render却只画screen rectangle，pointer也只做rectangle hit。该能力在真实`WorldUiSurfaceProduct`完成前必须返回Unavailable或明确标为DebugOverlay，不能继续以完整功能命名。

### ED58-P0-04 · Closed · Runtime保持post-render generation资格检查后再发布

direct product、capture finish和registry publication仍在post-render viewport generation资格检查之后，且receipt会验证owner、submission sequence与frame generation。本项保持Closed；后续必须补并发fault行为测试，避免只靠源码顺序断言。

## 4. P1重构账本

### 4.1 Identity、session与owner

| ID | 状态 | 当前证据与必须重构为 |
|---|---|---|
| ED58-P1-01 | Open | controller无`ViewInstanceId/window/document/session`。建立session registry作为pane到target唯一owner。 |
| ED58-P1-02 | Open | runtime handle只代表allocation。以session generation和Runtime surface lease包装handle。 |
| ED58-P1-03 | Open | Scene/Game builtin payload仍未持久化camera/view mode/show flags/realtime/quality/source schema。 |
| ED58-P1-04 | Open | 单`viewport_lifecycle: Arc<Mutex<()>>`扩展多视口会全局串行；改为registry协调和per-session gate。 |
| ED58-P1-05 | Open | cloneable controller没有最终owner和close receipt；改为不可复制owner与受限lease。 |
| ED58-P1-06 | Open | Drop继续忽略destroy error；close需stop、cancel、drain、destroy、deadline和receipt。 |
| ED58-P1-07 | Partial | kind级visibility能扫描dock/floating/native active pane并跳过部分capture；缺per-instance occlusion/background/keep-warm authority。 |
| ED58-P1-08 | Open | 无稳定view/subview identity；需要main/aux/subview概念并绑定Editor session。 |

### 4.2 Request、receipt与frame currentness

| ID | 状态 | 当前证据与必须重构为 |
|---|---|---|
| ED58-P1-09 | Open | `EditorRenderFrameSubmission`仍只有extract/ui；补request id、session generation、reason、deadline、cancel和policy。 |
| ED58-P1-10 | Partial | world/gateway generation真实存在；仍缺document/camera/selection/settings/size/quality统一冻结点。 |
| ED58-P1-11 | Open | `RenderViewportProduct`只有key/size/generation，无法证明当前source；扩成session/request/source-qualified product。 |
| ED58-P1-12 | Partial | Play frame与Runtime receipt provenance局部真实；Host Scene DTO转换后仍丢pane/document/camera/request/present provenance。 |
| ED58-P1-13 | Open | poll只按renderer generation替换，不能拒绝source已过时的成功帧；按expected epochs资格化。 |
| ED58-P1-14 | Partial | invalidation reason与重复设置no-op提供局部coalescing；缺latest-wins/ordered/barrier策略。 |
| ED58-P1-15 | Partial | Runtime有transaction/failure receipt；Editor缺accepted/queued/superseded/rendered/published/presented/dropped/failed状态机。 |
| ED58-P1-16 | Open | visible spatial snapshot在submit成功后另查，未绑定实际presented product receipt。 |

### 4.3 Surface、resize与恢复

| ID | 状态 | 当前证据与必须重构为 |
|---|---|---|
| ED58-P1-17 | Open | 每次有效尺寸变化destroy/create；缺interactive resize debounce/settle/temporary-resolution policy。 |
| ED58-P1-18 | Partial | zero frame局部受控且内部clamp到1；缺Minimized/Occluded/Suspended/Restoring状态机。 |
| ED58-P1-19 | Open | create和quality成功即安装target，没有first-frame-ready gate。 |
| ED58-P1-20 | Open | framework typed error被压成string/status；缺retryability/device-loss/OOM/terminal pane合同。 |
| ED58-P1-21 | Open | 无device loss后active session重建、last-good、fallback和恢复receipt。 |
| ED58-P1-22 | Open | terminal present仍能退出event loop，未先按window隔离并保全authoring状态。 |
| ED58-P1-23 | Partial | presenter upgrade失败可恢复standalone并记录warning；切换仍无prepare/commit与像素连续性receipt。 |
| ED58-P1-24 | Open | `take_error`消费单字符串；缺incident、连续失败、backoff、history和recovery action。 |

### 4.4 Direct GPU、capture与resource lease

| ID | 状态 | 当前证据与必须重构为 |
|---|---|---|
| ED58-P1-25 | Open | direct/fallback仍按Host全局协商，不按window/presenter/session。 |
| ED58-P1-26 | Open | direct product与capture共用`latest_generation`，切换模式可互相压制同代结果；引入per-mode epoch。 |
| ED58-P1-27 | Partial | provider按viewport/presenter确认resident并计consumer；缺pane/window/session generation资格。 |
| ED58-P1-28 | Partial | producer ring与cache可保护局部资源；缺accept/present/release lease和stall deadline。 |
| ED58-P1-29 | Open | cache admission reject未驱动pane degraded/fallback receipt。 |
| ED58-P1-30 | Partial | presenter refcount、per-presenter cache和shared texture ownership存在；缺多window present completion与回收receipt。 |
| ED58-P1-31 | Partial | 最后direct provider退出后capture predicate恢复；缺per-session fallback barrier。 |
| ED58-P1-32 | Open | product缺format/colorspace/HDR/alpha/sample/resolve/dynamic-resolution/display metadata。 |

### 4.5 Scene/Game、multi-view与native window

| ID | 状态 | 当前证据与必须重构为 |
|---|---|---|
| ED58-P1-33 | Partial | Game来自active Play gateway并保留identity；仍固定default viewport，无active camera、pane/request/present receipt和typed unavailable。 |
| ED58-P1-34 | Open | Scene仍只有一个controller/camera/mode/show flags/realtime状态，重复实例不能有不同视角。 |
| ED58-P1-35 | Open | recompute只读取主`viewport_content_frame`并更新global size；建立per-pane measured target epoch。 |
| ED58-P1-36 | Partial | floating window有独立UiHostWindow/presentation/focus；独立Host image set没有显式产品发布路径，更没有独立cadence/currentness。 |
| ED58-P1-37 | Open | 没有1/2/3/4 viewport session layout、split persistence和per-cell restore。 |
| ED58-P1-38 | Partial | Scene/Game/Simulate分槽、mode-specific input和kind visibility真实；source/camera/focus/budget仍非完整隔离。 |
| ED58-P1-39 | Partial | Gateway有session/transport generation、surface ABI和identity-qualified capture；UI无surface caller，pane latency/drop/present receipt缺失。 |
| ED58-P1-40 | Partial | Simulate可路由editor camera且跳过未变化值；缺通用preview/pilot/cinematic binding、return和污染隔离协议。 |

### 4.6 World UI、overlay与input

| ID | 状态 | 当前证据与必须重构为 |
|---|---|---|
| ED58-P1-41 | Open | world submission由Host模板派生，不是runtime component/UI asset qualified product。 |
| ED58-P1-42 | Open | world transform、meter size、pixels-per-meter未参与layout/raster/projection/LOD。 |
| ED58-P1-43 | Open | depth/billboard只改debug颜色；实现真实depth/occlusion和camera-facing orientation。 |
| ED58-P1-44 | Open | `camera_target`未消费；绑定真实camera/eye/layer/target并返回typed invalid-target。 |
| ED58-P1-45 | Open | pointer只做screen rect命中；实现ray-plane/mesh intersection、UV、clip和occlusion qualification。 |
| ED58-P1-46 | Open | capture全局保存克隆submission；缺pointer/button/window/viewport/surface generation与cancel reason。 |
| ED58-P1-47 | Open | overlay直接扩展command list；缺tree/node namespace、z/clip domain、冲突和容量验证。 |
| ED58-P1-48 | Partial | merge cache复用稳定generation/base Arc；仍无surface/pixel/command/text/update/distance预算。 |

### 4.7 Diagnostics、调度与性能

| ID | 状态 | 当前证据与必须重构为 |
|---|---|---|
| ED58-P1-49 | Partial | lazy resolve有ticket/cancel和有界job基础；仍用`JobCategory::Misc`，无startup deadline/priority/degraded product。 |
| ED58-P1-50 | Partial | invalidation mask记录并合并reason；最终submission gate仍是global bool，不能表达per-session policy。 |
| ED58-P1-51 | Partial | Runtime有profile/receipt/UI cache counters；缺per-session CPU/GPU/queue/capture/present-age/stale/drop/recovery指标。 |
| ED58-P1-52 | Partial | structured process diagnostics存在；用户状态仍是全局string且无pane/session/incident。 |
| ED58-P1-53 | Open | Editor私有quality profile继续绕过Runtime qualified profile authority。 |
| ED58-P1-54 | Partial | cache有界且隐藏Play pane可跳capture；缺viewport总VRAM/带宽/并发/background/fairness admission。 |
| ED58-P1-55 | Open | 关键测试仍大量是include_str/source-shape，缺真实presenter/fault/multi-view/source-currentness行为门。 |
| ED58-P1-56 | Partial | gateway identity、displayed-frame pick、Runtime receipt和resource key提供局部追踪；仍不能从pane追到request/frame/resource/present。 |

## 5. P2长期成熟度

| ID | 状态 | 当前差距 |
|---|---|---|
| ED58-P2-01 | Open | 无per-session viewport preset/profile。 |
| ED58-P2-02 | Open | 无per-viewport screenshot/record/frame comparison receipt。 |
| ED58-P2-03 | Open | 无统一pixel/HDR/depth/normal/object-id inspect product。 |
| ED58-P2-04 | Open | 无safe area、resolution/aspect preset、device frame和letterbox语义。 |
| ED58-P2-05 | Open | 无color management、display transform和per-monitor HDR选择。 |
| ED58-P2-06 | Open | 无bookmark、camera history与layout/session migration。 |
| ED58-P2-07 | Open | 无deterministic viewport render request capture/replay。 |
| ED58-P2-08 | Open | 无render product schema/compatibility version。 |
| ED58-P2-09 | Open | 无custom producer/overlay/input/budget extension API。 |
| ED58-P2-10 | Open | 无面向辅助技术的viewport状态、错误、camera和tool feedback。 |
| ED58-P2-11 | Open | 无multi-adapter/multi-GPU/cross-device unsupported或bridge策略。 |
| ED58-P2-12 | Open | 无generation/cache/resize/detach长期soak资格。 |

## 6. 参考引擎约束

| 参考源码 | 直接读取到的机制 | 对Zircon的约束 |
|---|---|---|
| Unreal | `SEditorViewport`逐实例调用`MakeEditorViewportClient`，并以该client创建自己的`FSceneViewport`；`FEditorViewportClient`管理view state、realtime、input、viewport type和invalidate | 每个pane必须消费自己的session/client/target；Host global controller不是可扩展架构 |
| Godot | `Node3DEditorViewport`逐实例持有`SubViewportContainer + SubViewport + Camera3D`；plugin显式维护多达4个viewport及布局状态 | 1/2/3/4布局必须创建同数量真实camera/target/session，不是重复绘制一张图 |
| Fyrox | Scene editor拥有camera controller和scene render target；`PreviewPanel`另建scene、camera、render target并按frame size重建 | Scene/Game/Preview应是独立producer/owner，resize归属于具体产品实例 |
| Bevy | Camera extract携target/viewport/order/output/HDR；`RetainedViewEntity`以main entity、auxiliary entity和subview index唯一标识view | runtime view identity、subview与target metadata必须进入request/receipt，不能只有opaque resource key |
| Unity Graphics | `UniversalCameraData`保存render type、target descriptor/texture、pixel rect、SceneView标识与resolve-final-target | 每camera/per-view frame data需明确format/HDR/size/resolve/stack；本地Graphics语料不代表闭源Unity Editor内部实现 |

采用顺序仍是：Unreal定义Editor per-instance owner主架构；Godot验证真实multi-view实例化；Fyrox验证preview/scene target分离；Bevy和Unity Graphics约束Runtime view identity和frame metadata。Zircon可以保留immutable snapshot、generation guard、direct/capture双路径和typed gateway，但“优于Unreal”必须由自己的多视口吞吐、frame age、故障恢复和视觉正确性证据证明。

## 7. 目标架构与唯一owner

### 7.1 必备合同

| 合同 | 唯一职责 |
|---|---|
| `EditorViewportSessionId` | project/document或play instance、`ViewInstanceId`、qualified `WindowId`、session generation |
| `EditorViewportDefinition` | Scene/Game/Preview/custom kind、source/camera binding、persistence schema、capability |
| `EditorViewportSession` | target/input/visibility/settings/dirty/lifecycle/last-good owner |
| `ViewportRenderRequest` | request id、全部source epochs、target descriptor、deadline、coalescing policy |
| `ViewportRenderReceipt` | accepted到presented/failed的状态、timing、typed failure与完整provenance |
| `ViewportFrameProduct` | session/request、format/colorspace/HDR/size、direct/capture payload与resource lease |
| `ViewportPresentationState` | Starting/Current/Stale/Degraded/Lost/Suspended/Closing与reason/remediation |
| `ViewportPresentationLease` | consumer/window/presenter、resource generation、accept/present/release fence与deadline |
| `ViewportProductMap` | 只按session或`ViewInstanceId + WindowId`查询；禁止global/kind-only authority |
| `WorldUiSurfaceProduct` | UI tree generation、world transform、camera/layer、depth/billboard、texture/mesh与ray-UV endpoint |

### 7.2 Owner边界

- `zircon_app`继续拥有进程和真实platform event loop的执行；Editor不能创建第二套loop。
- Runtime191的PlatformHost/WindowRegistry拥有qualified OS `WindowId/DisplayId/SurfaceLease`及lifecycle；Editor只提交window/pane intent并持有lease，不保存裸native authority。
- Editor53拥有通用tool lease和pointer capture；viewport route携session/window/surface/product generation后消费共享authority。
- Editor03/247拥有authoring world、selection、Gizmo与document；viewport只冻结并消费source revision。
- Editor07拥有Play process/world/pause/eject/recovery；viewport只绑定play instance/camera/frame consumer。
- Editor30/250与Runtime Camera owner拥有camera/rig/director/cut；viewport只消费camera binding revision。
- Runtime render/RHI owner拥有render graph、view execution、resource lifetime与surface present；Editor定义end-to-end consumer currentness，不复制renderer。
- Runtime UI owner生成真实UI texture/mesh product；world UI不能在Editor Host复制第二套UI renderer。

### 7.3 原子产品流

`Pane activate -> resolve qualified Window/Surface lease -> create EditorViewportSession -> freeze source epochs -> prepare target -> render -> post-render qualify -> publish frame product lease -> pane/session/currentness match -> presenter accept -> present receipt -> commit last-good`。

resize、quality、camera/source切换、direct/capture切换和device replacement都必须走prepare/commit。任何失败返回typed receipt；旧产品保持`Current`或明确降为`Stale`。close先停止新request，再cancel/drain queue和leases，最终destroy并发布close receipt。

## 8. 必须硬切的旧路径

1. 删除Host单一controller/size对所有Scene实例的authority，禁止kind-only产品查询。
2. 删除native window独立空`HostViewportImageSet`却无qualified publication的隐式行为；每个native presenter必须订阅session product map。
3. 删除Scene/Game空或无版本payload，改为versioned viewport definition与session restore。
4. 删除direct/capture共享generation cursor，改为per-session、per-mode epoch。
5. 删除resize先destroy再create的owner切换，改为双target prepare/commit。
6. 删除submit error后清空render dirty的行为；retry或terminal state必须保留reason。
7. 删除全局status与一次性string error作为viewport failure authority。
8. 禁止用固定ring/cache寿命代替consumer presentation lease。
9. 删除Editor私有quality authority，接Runtime qualified profile/epoch。
10. 删除world-space UI screen-rect生产入口；真实产品前返回Unavailable或DebugOverlay。
11. 禁止为Game复制authoring Scene或只增加kind槽；必须绑定Play session/camera product。
12. 禁止把Gateway surface ABI当成pane session；它只能是qualified session的transport adapter。
13. 禁止source-shape、截图或resource generation替代source epoch、fault、lease与present receipt证据。

## 9. 依赖有序重构里程碑

### M0 · Capability truth与failure currentness

- 保持P0-04顺序并补真实并发fault test。
- submit error保留dirty并进入typed retry/terminal state。
- world-space UI切Unavailable/DebugOverlay；Game无active product显示typed unavailable。
- 旧图增加Current/Stale/Degraded/Lost状态和incident。

### M1 · Platform identity bridge与session registry

- 由App/Runtime191提供qualified Window/Display/Surface lease给Editor。
- 建立`EditorViewportSessionId/Definition/Registry`，把`ViewInstanceId + WindowId + source`绑定为唯一owner。
- viewport callback必须携pane/session/window/surface generation。

### M2 · Per-pane product map与native presenter cutover

- 引入`ViewportProductMap`，先支持两个Scene实例拥有不同camera/size/input/product。
- main/native floating presenter按session查询image/product并发布accept/present/release receipt。
- 删除global `scene/simulate/game` authority；兼容adapter只能短期只读且不可成为第二事实源。

### M3 · Request/receipt与source currentness

- 冻结document/world/camera/selection/settings/size/quality epochs。
- 实现latest-wins、ordered、barrier、supersede、cancel、deadline和typed failure。
- visible spatial snapshot与同一presented receipt提交。

### M4 · Atomic target与recovery state machine

- prepare/configure/warmup/first-qualified-frame/commit resize和quality change。
- 建立Minimized/Occluded/Suspended/DeviceLost/Restoring/Closing状态。
- presenter upgrade/fallback、surface replacement与shutdown进入同一transaction。

### M5 · Resource lease与direct/capture currentness

- direct/capture独立cursor和mode epoch。
- producer ring、presenter cache与shared registry接accept/present/release lease。
- admission reject、stall和device loss驱动pane fallback/degraded receipt。

### M6 · Play、多视口与持久化

- Play/Simulate从default viewport迁移到session-qualified runtime view/camera。
- 完成Scene+Game同时可见、duplicate/floating和1/2/3/4布局恢复。
- 建立per-session cadence、background throttle、fairness与总VRAM/带宽预算。

### M7 · 真实world UI与资格闭环

- Runtime UI tree生成texture/mesh product，接camera projection、depth、billboard、ray/UV、focus/capture和surface lifecycle。
- 运行真实Windows main/native floating、fault injection、device loss、HDR、多视口scale与30分钟soak。
- 以frame age、CPU/GPU time、queue latency、copy bytes、VRAM、drop/stale/recovery和视觉golden形成性能/正确性基线。

## 10. 46项资格门当前重判

| Gate | 状态 | 当前判定 |
|---:|---|---|
| 1 | Partial | Scene/Game槽和Play gateway identity不同；无独立pane session/request/present receipt。 |
| 2 | Partial | 无active Play frame会清Game槽；无typed unavailable product state。 |
| 3 | Fail | 两个Scene不同camera没有独立像素/metadata证据。 |
| 4 | Fail | duplicate/floating Scene没有独立controller、size、generation与publication。 |
| 5 | Partial | native floating有独立UiHostWindow/presentation/focus；没有显式per-window viewport image publication。 |
| 6 | Fail | 无1/2/3/4 session layout与恢复。 |
| 7 | Partial | kind级visibility可跳部分Play capture；非per-session suspend/throttle。 |
| 8 | Partial | world/gateway/Play generation局部存在；缺document/camera/settings/size/quality request epochs。 |
| 9 | Fail | product没有全部expected epochs，无法资格为Current。 |
| 10 | Partial | Runtime拒绝viewport generation race，Simulate pick绑定显示帧；Editor product仍无source supersede receipt。 |
| 11 | Fail | visible spatial snapshot未绑定同一presented receipt。 |
| 12 | Partial | last-good像素可能继续显示且destroy失败可恢复handle；无Stale/Degraded与自动retry。 |
| 13 | Partial | quality失败的新target不安装，但旧target已销毁且无原子rollback。 |
| 14 | Fail | submit瞬时失败会消费dirty。 |
| 15 | Fail | 无incident-backed terminal Degraded/Lost状态。 |
| 16 | Partial | zero frame局部受控；无显式per-session lifecycle state。 |
| 17 | Fail | 无interactive resize policy。 |
| 18 | Fail | 无device-loss session rebuild资格。 |
| 19 | Pass | post-render viewport generation变化时先validate，direct registry不发布新descriptor。 |
| 20 | Pass | direct/capture publication与success record位于post-render qualification之后。 |
| 21 | Fail | direct/capture共用cursor且无mode epoch。 |
| 22 | Partial | presenter upgrade失败会恢复standalone并记录warning；无qualified product continuity receipt。 |
| 23 | Partial | last direct provider退出后capture predicate恢复；无per-session fallback barrier。 |
| 24 | Partial | ring/cache可持有已接受资源；未接受stall没有lease/deadline/fallback。 |
| 25 | Fail | cache reject未传播pane receipt。 |
| 26 | Partial | per-presenter cache/refcount存在；无multi-window present/release completion。 |
| 27 | Fail | direct product缺format/colorspace/alpha/dynamic-resolution/resolve metadata。 |
| 28 | Fail | 无HDR/SDR/per-monitor display transform currentness。 |
| 29 | Pass | retryable surface acquire保留同一present request与qualified draw list。 |
| 30 | Fail | terminal present仍能退出event loop，未先形成window隔离恢复receipt。 |
| 31 | Fail | close无stop/drain/lease deadline/close receipt。 |
| 32 | Fail | Drop destroy failure被静默吞掉。 |
| 33 | Fail | world-space UI生产入口仍画debug Quad，未Unavailable。 |
| 34 | Fail | world transform/meter/pixels-per-meter不参与产品生成。 |
| 35 | Fail | billboard/depth无camera orientation与occlusion证据。 |
| 36 | Fail | camera target不决定实际projection。 |
| 37 | Fail | pointer不是ray/UV且occluded surface仍可抢input。 |
| 38 | Fail | capture未按pointer/window/viewport/surface generation限定。 |
| 39 | Fail | overlay无稳定namespace和command/pixel budget。 |
| 40 | Fail | 无1/2/4 viewport完整性能门。 |
| 41 | Partial | kind visibility与cache budget提供局部节流；无per-session fairness/总预算。 |
| 42 | Partial | invalidation reason与duplicate no-op提供局部coalescing；无request policy。 |
| 43 | Fail | 无30分钟multi-window resize/mode-switch soak。 |
| 44 | Fail | 无create/quality/submit/cache/surface/device全链fault injection。 |
| 45 | Fail | 无pane到present receipt的端到端audit。 |
| 46 | Fail | 未取得Windows main/native floating主路径完整执行资格。 |

## 11. 验证边界与实施入口

本轮只做current-working-tree静态review、参考源码对照、状态重判与文档记录。没有修改Editor/Runtime/App/plugin/Cargo/ABI/ZUI/assets或测试；没有运行Cargo、Editor、真实native floating window、GPU presenter、Play child process、fault injection、device loss、HDR、multi-view、soak或benchmark。因此本报告不宣称当前工作树可构建、测试green、floating viewport必然黑屏，亦不宣称性能优于Unreal。

实施入口必须从M0开始，先封闭错误后停更与伪world-space能力，再由Runtime191/App提供qualified window/surface identity，建立Editor session registry和per-pane product map。不得先在`HostViewportImageSet`增加更多槽、复制主Host图像到native Host或为每个pane直接new第二套controller，这些做法只会扩大三套identity和两套platform ownership。Tooling不在本轮范围；按用户要求，本轮没有查询、轮询、等待或实时跟踪协调器。

## 12. 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
