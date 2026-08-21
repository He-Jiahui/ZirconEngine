---
title: Editor Scene Viewport Host、Render Product、Surface Lifecycle、Frame Currentness、Multi-Viewport 与 Product Integration 当前源码工程化差距
category: zircon_editor
report_id: Editor58
review_date: 2026-08-21
baseline_head: bea1acf91b909525ab1759e2c800858b0eda6528
baseline_epoch: 335
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle
  - zircon_editor/src/ui/retained_host/app/viewport
  - zircon_editor/src/ui/retained_host/app/viewport_image_redraw.rs
  - zircon_editor/src/ui/retained_host/viewport
  - zircon_editor/src/ui/retained_host/host_contract/data
  - zircon_editor/src/ui/retained_host/host_contract/globals
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/native_panes/viewport.rs
  - zircon_editor/src/ui/retained_host/host_contract/window
  - zircon_editor/src/ui/host/builtin_layout/builtin_shell_view_instances.rs
  - zircon_editor/src/ui/host/builtin_views/activity_views
  - zircon_editor/src/ui/workbench/state/editor_state_render.rs
  - zircon_runtime/src/core/framework/render
  - zircon_runtime/src/graphics/runtime/render_framework
  - zircon_runtime/crates/zr_rhi_wgpu/src/ui_surface
tests:
  - zircon_editor/src/tests/host/retained_window/native_viewport_image.rs
  - zircon_editor/src/ui/retained_host/ui/tests/floating_windows.rs
  - zircon_editor/src/ui/retained_host/viewport/tests
  - zircon_editor/src/ui/retained_host/viewport/viewport_state_job_tests.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/tests.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/optimize/zircon_editor/30-camera-asset-component-rig-controller-director-blend-shake-cinematic-cut-preview-authoring-review.md
  - docs/plans/optimize/zircon_editor/53-editor-interactive-tool-scheduler-resource-lease-input-capture-mode-modal-extension-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/09a-rhi-render-graph-gpu-lifetime-review.md
  - docs/plans/optimize/zircon_runtime/09b-renderer-visibility-gpu-scene-review.md
  - docs/plans/optimize/zircon_runtime/11c-gpu-ui-renderer-atlas-sdf-batch-clip-submit-review.md
  - docs/plans/optimize/zircon_runtime/57-platform-host-window-registry-monitor-display-event-loop-application-lifecycle-surface-command-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/65-runtime-scalability-quality-profile-device-profile-capability-tier-dynamic-resolution-frame-budget-lod-feature-fallback-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/79-runtime-ui-renderer-display-list-paint-order-clip-transform-opacity-atlas-text-glyph-batch-wgpu-submit-product-integration-current-source-review.md
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
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Editor Scene Viewport Host、Render Product、Surface Lifecycle、Frame Currentness、Multi-Viewport 与 Product Integration 当前源码工程化差距

## 1. 结论

Zircon当前编辑器视口不是纯占位。它已经通过`RenderFramework`创建、销毁和按尺寸重建真实runtime viewport，能提交scene extract、overlay UI和quality profile；同设备native presenter可以直接消费GPU texture，softbuffer路径可以异步轮询CPU capture；UI image cache按resource key与generation隔离，native surface对retryable present也有明确重试。这些底座应保留。

但当前产品合同仍是单视口原型。`RetainedEditorHost`只持有一个`RetainedViewportController`和一个`viewport_size`，host contract只保存一个全局`viewport_image`，painter把这张图画进所有kind为`Scene`或`Game`的pane。默认布局虽同时注册`editor.scene#1`与`editor.game#1`，floating window测试也允许更多Scene/Game实例，渲染提交却始终只从一个editor scene controller构造一份snapshot。结果是Scene、Game、复制视口、浮动视口和未来多窗口没有独立产品身份，Game面板也没有play world/camera产品事实。

失败和currentness合同同样不足。resize先清空并销毁旧viewport，再创建新viewport；create、quality或submit错误会消费`render_dirty`，不会自动重试，也不会清除或标记host里的last-good image，因此旧像素可以无限期冒充当前场景。runtime直出GPU产品的两个submit入口又在释放并重新取得state后，先`publish`、再复核viewport generation；一旦代际在间隙变化，调用虽返回`ViewportChanged`，错误产品却已经可被UI轮询。产品DTO只有renderer generation、尺寸和resource key，没有document/view/camera/source/settings/size epoch，宿主无法判断“成功生成但已过时”的帧。

当前所谓world-space UI还是能力真实性错误。DTO公开world transform、meter size、billboard、depth test和camera target，但render path只使用预先填入的screen-space rectangle画固定Quad与`control_id`文本；pointer path也只做矩形hit test并改一条status字符串，不向真实control分发事件。该功能必须在产品面撤下或明确标成Unavailable，不能继续用完整字段名表示已经支持相机投影、遮挡和交互。

本报告登记 **4项P0、56项P1、12项P2与46个资格门**。Editor58唯一拥有viewport instance/session、pane到render product映射、source-to-present currentness、surface/resize恢复、GPU直出与CPU fallback模式、多Scene/Game产品隔离及编辑器world-space UI视口集成。Editor03继续拥有scene语义、selection/picking/Gizmo和相机导航，Editor07拥有PIE/play process与Game world，Editor30拥有Camera资产/director authoring，Editor53拥有通用tool scheduler/input capture，Runtime09a/09b/11c/57/65/79拥有renderer、RHI、window surface、quality与UI renderer父合同。

## 2. 审查边界、currentness与证据等级

### 2.1 冻结语料

| 范围 | 文件 / 行 / 非空行 / bytes / test attributes | 证据等级 | 说明 |
|---|---:|---|---|
| Zircon视口产品链 | **86 / 11,974 / 11,055 / 446,169 / 109** | E3 | retained host/controller、pane presentation、Scene/Game descriptor、runtime viewport/product/capture、submit generation guard、wgpu external image cache及focused tests |
| 五引擎参考切片 | **14 / 31,013 / 26,550 / 1,231,129 / 0** | E2/E3 | Unreal editor viewport主证据，Godot multi-viewport，Fyrox scene/preview target，Bevy retained view identity，Unity Graphics per-camera consumer |

86份Zircon文件按normalized relative path排序，将每个`path + NUL + lowercase file SHA-256 + LF`串联后计算working-tree fingerprint，结果为`737c56f8ec9561d1a25a426a0458bf98d06d5c155344641fa930c6cf3c516b20`。14份参考源码按同一算法计算，fingerprint为`ea01c41e5c8eca9ab76821ddc07d7e6161fed2c157f3e29d9f8e983f89b4e640`。

冻结Git基线为`bea1acf91b909525ab1759e2c800858b0eda6528`，coordinator baseline epoch为335。Godot、Fyrox、Bevy与Unity Graphics参考revision分别为`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`与`a7e4c051d256a781ab362c64316b125a1e104694`；Unreal源码随主仓基线冻结。

聚焦Zircon语料中有10份非本轮产生的dirty文件：`recompute/presentation.rs`、5份viewport focused test/fake、`editor_state_render.rs`及3个runtime submit入口。本轮按working tree审查，只编辑报告和三个索引；实施前必须重算fingerprint，并复核这些在途文件是否改变pane映射、submission gate、world-space overlay或publish/validate顺序。

### 2.2 产品链事实矩阵

| 阶段 | 当前实现 | 当前资格 | 关键缺口 |
|---|---|---|---|
| Pane实例 | Scene/Game有独立`ViewInstanceId`和可浮动layout | UI身份真实 | 身份没有进入viewport controller或render request |
| Extract | 从一个`EditorState.viewport_controller`构造scene snapshot与overlay | 单场景提交真实 | 无view kind、camera binding、document或source revision |
| Viewport lifecycle | controller按尺寸create/destroy并设置quality | happy path真实 | 先破坏旧viewport；无prepare/commit、rollback或retry state |
| Submit | operation gate串行submit并查询visible spatial snapshot | 单viewport happy path真实 | 错误消费dirty；visible snapshot没有与presented receipt绑定 |
| Direct GPU | runtime复制最终texture并发布external image product | 同设备直出真实 | generation复核晚于publish；无request provenance与present fence |
| CPU fallback | 按generation轮询captured RGBA | fallback真实 | 与direct path共用单一cursor；模式切换无epoch |
| Host storage | 一个`HostViewportImageData`，支持GPU key或RGBA | 单图绘制真实 | 全局图同时供所有Scene/Game pane，且无stale/degraded状态 |
| Surface present | retryable acquire/present会保留同一present请求 | surface retry真实 | renderer submit/create/quality错误没有相同恢复合同 |
| World-space UI | 构造含world字段的submission并合并overlay | DTO和可见Quad真实 | world字段、相机、深度、billboard及真实事件均未实现 |

### 2.3 必须保留的工程基础

1. 保留`RenderViewportHandle`的generation guard和create/destroy显式生命周期；重构应把它纳入session receipt，而不是回退到隐式全局surface。
2. 保留direct GPU product与CPU capture双路径；目标是per-consumer mode epoch和资格状态，不是强制所有平台readback。
3. 保留`resource_key + generation`图像缓存键和外部纹理独立copy；它避免renderer复用final target时覆盖UI仍在采样的帧。
4. 保留per-presenter image cache与device-shared static image registry的内存预算、LRU和统计；补充的是lease/fence、拒绝反馈和viewport产品域隔离。
5. 保留retryable native surface acquire/present的退避和同请求重试语义；renderer submission也应达到同级别。
6. 保留controller operation gate，防止同一viewport的resize、submit与destroy无序交错；后续应缩小为per-session gate而非全局串行化所有视口。
7. 保留extract按实际viewport size更新camera aspect ratio，后续将size epoch写入request/receipt。
8. 保留renderer-visible spatial snapshot的generation-aware scene adoption；必须再绑定同一presented frame receipt，供Editor03消费。
9. 保留pane layout对Scene/Game、drawer/document和floating window的通用承载；产品owner应按`ViewInstanceId`注入，不另造第二套窗口系统。
10. 保留host paint-only viewport invalidation，避免每个新产品触发完整业务snapshot重算。

### 2.4 唯一owner边界

| 问题 | 唯一实现owner | Editor58责任 |
|---|---|---|
| scene文档、selection、picking、Gizmo、camera navigation | Editor03 | 只提供qualified viewport session/input/product receipt |
| PIE/play process、play world、pause/eject/recovery | Editor07 | Game View绑定play session与camera，不复制play authority |
| camera asset/rig/director/cut/preview authoring | Editor30、Runtime37 | 消费camera binding/revision，不建立第二套camera model |
| tool lease、modal mode、pointer capture | Editor53 | viewport route携带session/pointer/window身份并消费共享lease |
| renderer/RHI/render graph/visibility/GPU lifetime | Runtime09a/09b | 登记publish-before-validate依赖并验证端到端产品真实性 |
| host window/surface/device lifecycle | Runtime57 | pane产品状态接入window/surface receipt，不私建platform loop |
| quality profile/dynamic resolution/frame budget | Runtime65 | viewport request引用profile/epoch；不在Editor58复制默认常量 |
| GPU UI renderer/cache/clip/batching | Runtime11c/79 | 定义external product lease、admission和present receipt消费合同 |
| retained UI业务snapshot/paint性能 | Editor01 | 只修viewport product map和局部invalidation |

## 3. P0：必须先关闭的能力真实性与产品完整性问题

### ED58-P0-01 · Scene、Game、duplicate与floating pane共享一个全局viewport产品

`RetainedEditorHost`只有一个`RetainedViewportController`和一个`viewport_size`；controller state只有一个`ActiveViewport`和一个`latest_generation`。`HostContractState`与presentation generation也只保存一个`viewport_image`。`draw_viewport_image`对任意kind为`Scene`或`Game`的pane都绘制这张图，而默认布局同时创建`editor.game#1`和`editor.scene#1`，floating window路径还允许更多实例。`EditorState::render_frame_submission`不接收`ViewInstanceId`或view kind，只从一个editor scene controller构造snapshot。

因此当前Game不是play/runtime camera产品，多个pane也不是多个视角，只是把同一像素重复显示。必须建立`EditorViewportSessionRegistry`，让每个可见view instance拥有明确session、source world、camera、target、size/quality epoch和presentation state；pane只能消费与自身instance相符的receipt。修复前不能把Game或多视口标为可用能力。

### ED58-P0-02 · resize/create/quality/submit失败会销毁可用viewport、冻结旧图并停止重试

`ensure_viewport`在尺寸变化时先从shared state移除active viewport并清空latest generation，再destroy旧viewport，然后create并设置quality，最后才安装新viewport。create或quality失败时没有active viewport，也没有last-good target rollback。host中的旧`HostViewportImageData`没有同步清除或stale标记。

`render_submission`仅在`Ok(false)`即render framework尚未解析时保留`render_dirty`；create、quality和submit的`Err`分支只写日志/status，随后把`render_dirty`设为false。瞬时故障因而不会自动重试，旧像素却继续作为普通当前图绘制。必须改为prepare/create/configure/warmup/commit的原子切换，保留旧产品到新产品qualified，错误进入`Stale`或`Degraded`并按typed retry policy重试；不可用时必须显式占位而不是冒充当前帧。

### ED58-P0-03 · `WorldSpaceUiSurfaceSubmission`公开完整3D语义，实际只画screen-space假控件

submission含`world_position`、`world_rotation`、`world_scale`、world width/height、pixels-per-meter、billboard、depth test与camera target，但`world_space_ui_render_command`完全不读取这些空间字段，只用`viewport_x/y/width/height`生成固定颜色Quad，把`control_id`当文本。pointer route同样只做screen rectangle hit test；Down/Up/Scroll最终只改status字符串，没有向对应UI tree/control派发事件、focus、capture或action。

必须立即从可见产品面移除或标为Unavailable。真正实现必须由world UI surface生成独立UI render product，经过camera projection/world quad/depth/occlusion/billboard合成，并通过ray-to-quad/UV映射进入真实UI dispatcher；capture必须按pointer、window、viewport与surface generation限定。不能继续靠字段齐全和测试固定矩形来证明功能完成。

### ED58-P0-04 · runtime在viewport generation复核前发布直出GPU产品

`submit.rs`和`submit_runtime_frame.rs`在render后通过`finish_active_capture_and_relock`重新取得framework state，随后若存在direct presenter便调用`viewport_products.publish`，再执行`validate_viewport_generation`。preparation阶段虽提前校验过generation，但state释放期间viewport可被其他owner改变；这时函数最终返回`ViewportChanged`，已经发布的external image descriptor仍可被UI轮询并绘制。

必须把post-render generation/source qualification放在任何capture/direct publication、visible snapshot adoption和success stats之前；publication应是带expected session generation与request id的commit。失败路径必须证明没有新product可见，并撤销或隔离所有临时资源。该缺陷的代码owner在Runtime09a/09b，Editor58负责端到端门禁和回归资格。

## 4. P1：工程级能力差距

### 4.1 Viewport identity、instance与owner（ED58-P1-01至P1-08）

- **ED58-P1-01**：controller没有`ViewInstanceId`、window id、document id或session id；无法判断调用属于哪个pane。
- **ED58-P1-02**：`RenderViewportHandle`只代表runtime allocation，不代表editor view kind、source world或owner lifetime。
- **ED58-P1-03**：Scene/Game descriptor的serializable payload为空，未持久化camera、view mode、realtime、quality、render target和session binding。
- **ED58-P1-04**：一个全局`viewport_lifecycle: Mutex<()>`会串行化未来所有视口；应是registry协调加per-session operation gate。
- **ED58-P1-05**：controller可clone并被presenter factory持有，实际最后owner和shutdown时机不透明。
- **ED58-P1-06**：`Drop`中的destroy错误被丢弃，没有close receipt、drain deadline或资源泄漏诊断。
- **ED58-P1-07**：没有viewport visibility/activation authority；隐藏、遮挡、后台window和inactive tab仍无法分别决定pause、throttle或keep-warm。
- **ED58-P1-08**：没有stable per-view/subview identity支持stereo、thumbnail/preview、camera stack layer或未来XR eye。

### 4.2 Render request、receipt与frame currentness（ED58-P1-09至P1-16）

- **ED58-P1-09**：`EditorRenderFrameSubmission`只有`extract`与`ui`，缺request id、session generation、dirty reason、deadline和cancel token。
- **ED58-P1-10**：submission不携带document/world revision、camera revision、selection/highlight revision、settings revision、size epoch或quality epoch。
- **ED58-P1-11**：`RenderViewportProduct`只有resource key、width、height和renderer generation，不能证明产品来自当前source。
- **ED58-P1-12**：`CapturedFrame`虽含capture/profile数据，仍没有editor source provenance或consumer target。
- **ED58-P1-13**：poll仅按单调renderer generation判新；成功但过期的帧会覆盖较老但仍与当前source匹配的last-good状态。
- **ED58-P1-14**：没有coalescing/backpressure策略；连续编辑、resize和camera move没有latest-wins/ordered/barrier分类。
- **ED58-P1-15**：submission没有receipt区分accepted、queued、superseded、rendered、published、presented、dropped和failed。
- **ED58-P1-16**：renderer-visible spatial snapshot在submit后另行查询并同步，未证明它与实际presented frame、camera和source revision一致。

### 4.3 Surface、resize、device loss与恢复（ED58-P1-17至P1-24）

- **ED58-P1-17**：resize没有debounce/settle policy和interactive resize分辨率策略，可能对每个尺寸变化destroy/create。
- **ED58-P1-18**：零尺寸/minimized/occluded状态没有显式state machine，只有输入尺寸是否合法的局部判断。
- **ED58-P1-19**：new target没有首帧ready gate；allocation成功即可替换owner，尚未证明可渲染和可present。
- **ED58-P1-20**：destroy、create、set quality和submit错误被压成字符串，无法分类retryable、device-lost、out-of-memory、invalid request或terminal。
- **ED58-P1-21**：没有device loss后的session-wide重建顺序、last-good保留、CPU fallback切换或恢复receipt。
- **ED58-P1-22**：native presenter fatal present会直接退出event loop；没有按window隔离、surface recreation或编辑状态保全策略。
- **ED58-P1-23**：presenter从standalone升级runtime时先drop旧presenter再create新presenter，切换没有prepare/commit或像素连续性资格。
- **ED58-P1-24**：错误通过`take_error`一次性消费，没有incident id、连续失败计数、退避状态、恢复动作或历史查询。

### 4.4 Direct GPU、CPU capture与资源驻留（ED58-P1-25至P1-32）

- **ED58-P1-25**：direct/fallback选择是`UiHostWindow`全局bool，不按window、presenter、viewport或产品能力决定。
- **ED58-P1-26**：GPU product与CPU capture共用一个`latest_generation` cursor，模式切换没有mode epoch，可能跳过目标路径已有帧。
- **ED58-P1-27**：runtime producer只在presenter确认某viewport resident后停止其CPU capture，但确认没有绑定具体pane/window/session生命周期。
- **ED58-P1-28**：三代producer ring是固定数量而非显式consumer lease；presenter本地cache可保住已接收纹理，但接收前的stall没有deadline/fence合同。
- **ED58-P1-29**：共享image registry和presenter cache各自有64 MiB/256项预算，admission reject只进统计，不回传pane的degraded/fallback状态。
- **ED58-P1-30**：没有跨多个native window的消费计数、present completion和安全回收receipt；`confirm_resident`只证明cache接收。
- **ED58-P1-31**：last presenter drop会清空全部direct product，没有对仍活跃viewport发布mode transition或强制CPU capture barrier。
- **ED58-P1-32**：缺少颜色空间、HDR、alpha、format、sample/resolve和dynamic resolution metadata；UI只接收RGBA大小或opaque texture key。

### 4.5 Scene/Game、multi-view与window产品语义（ED58-P1-33至P1-40）

- **ED58-P1-33**：Game View没有绑定Editor07 play session、runtime world、active game camera或no-session unavailable状态。
- **ED58-P1-34**：Scene View没有per-instance camera/view mode/show flags/realtime策略，重复实例无法形成正交/透视等不同产品。
- **ED58-P1-35**：pane resize只更新一个全局viewport size，多pane同时可见时没有各自content frame到target size映射。
- **ED58-P1-36**：floating window有独立native surface和focus target，但没有独立viewport product consumer或present cadence。
- **ED58-P1-37**：没有1/2/3/4 viewport layout产品模型、split ratio持久化和per-cell session恢复。
- **ED58-P1-38**：没有Scene/Game同时显示时的source isolation、camera isolation、input focus和帧预算策略。
- **ED58-P1-39**：没有remote/play-process frame transport、latency/drop/currentness receipt；Game View无法跨进程成为可信consumer。
- **ED58-P1-40**：没有preview/pilot/cinematic camera临时绑定与归还协议，容易让camera authoring状态污染普通Scene session。

### 4.6 World-space UI、overlay composition与input（ED58-P1-41至P1-48）

- **ED58-P1-41**：world submission由host scene template节点生成，不是runtime scene component/UI asset实例的qualified产品。
- **ED58-P1-42**：world transform、meter size和pixels-per-meter没有参与layout、rasterization、projection或LOD。
- **ED58-P1-43**：`depth_test`和`billboard`只改变debug颜色，不产生深度、朝向或遮挡行为。
- **ED58-P1-44**：`camera_target`完全未消费，无法决定投影相机、layer、eye或render target。
- **ED58-P1-45**：pointer命中没有ray、plane intersection、UV、clip或occlusion qualification，屏幕矩形可在物体背后抢占输入。
- **ED58-P1-46**：capture保存整个cloned submission且全局唯一，没有pointer id、button、window、viewport、surface generation和cancel reason。
- **ED58-P1-47**：合并overlay时直接`extend`command vector，没有tree/node namespace、z-domain、clip、容量预算或冲突验证。
- **ED58-P1-48**：每帧clone/merge所有surface，未设surface数、像素面积、command数、文本、update rate和远距降级预算。

### 4.7 Diagnostics、调度、性能与跨报告边界（ED58-P1-49至P1-56）

- **ED58-P1-49**：framework lazy resolve使用通用`JobCategory::Misc`，没有viewport启动deadline、优先级、取消和terminal degraded状态。
- **ED58-P1-50**：render dirty只有bool，不能解释scene/camera/overlay/size/quality/source切换，也不能精确合并或丢弃。
- **ED58-P1-51**：没有per-session CPU/GPU time、queue latency、capture bytes、present age、stale duration、drop/supersede和recovery指标。
- **ED58-P1-52**：用户可见status只是一条全局字符串；多个viewport并发错误会互相覆盖，且无法定位pane。
- **ED58-P1-53**：默认quality配置由editor viewport私有常量直接下发，未引用Runtime65的qualified profile id/capability decision。
- **ED58-P1-54**：没有viewport总显存、capture带宽、并发render、后台节流与公平性预算；多窗口扩展会无准入地放大成本。
- **ED58-P1-55**：测试大量验证源码字符串和happy path，缺真实presenter、fault injection、multi-view和source-currentness证据。
- **ED58-P1-56**：没有端到端产品审计记录关联`view instance -> request -> runtime viewport -> frame -> resource -> present`，线上问题无法重放。

## 5. P2：成熟度与长期演进差距

- **ED58-P2-01**：缺viewport preset/profile，无法保存透视/正交、show flags、camera speed、quality与overlay组合。
- **ED58-P2-02**：缺per-viewport截图、录制和frame comparison receipt；不能把全窗口capture冒充viewport capture。
- **ED58-P2-03**：缺像素取样、HDR inspect、depth/normal/object-id buffer inspect的统一产品选择。
- **ED58-P2-04**：缺安全区、分辨率/aspect preset、device frame和letterbox产品语义。
- **ED58-P2-05**：缺color management/display transform与不同monitor HDR能力的per-window选择。
- **ED58-P2-06**：缺viewport bookmark、camera history和跨layout恢复时的session migration。
- **ED58-P2-07**：缺deterministic render request capture/replay，难以复现偶发stale或surface race。
- **ED58-P2-08**：缺render product compatibility/schema version，未来跨进程或remote consumer无法协商。
- **ED58-P2-09**：缺extension API声明自定义viewport producer、overlay layer、input route和resource budget。
- **ED58-P2-10**：缺辅助功能语义，把viewport状态、错误、camera和tool feedback暴露给screen reader/keyboard workflow。
- **ED58-P2-11**：缺多adapter、多GPU和跨device copy的显式unsupported/bridge策略。
- **ED58-P2-12**：缺长期soak中generation exhaustion、cache churn、resize storm和window detach/reattach资格。

## 6. 参考引擎对照与采用结论

| 参考 | 当前源码证据 | Zircon应采用 | 不应机械复制 |
|---|---|---|---|
| Unreal | `SEditorViewport`逐实例创建client与`FSceneViewport`；`FEditorViewportClient`持有自己的view state、input/draw/realtime/invalidation；`SLevelViewport`恢复per-instance show flags、FOV、exposure与visualization | 以viewport client/session为产品owner，widget只消费该session；realtime与invalidated驱动按实例调度 | 不复制Slate或全局editor singleton，保留Zircon Rust合同和retained host |
| Godot | `Node3DEditorViewport`逐实例拥有SubViewportContainer、SubViewport和Camera3D；plugin构造固定数组并支持1/2/3/4布局 | 建立真实multi-view cell和持久化split，每个cell有独立camera/target/input | 不复制固定数组上限，Zircon用registry与typed session |
| Fyrox | scene editor与PreviewPanel分别拥有scene/camera/render target，并在尺寸变化时替换target | preview、scene、game必须是不同producer/session，target生命周期由owner管理 | 不把preview scene结构直接作为主scene文档authority |
| Bevy | camera extract按target/order排序；`RetainedViewEntity`组合main/auxiliary/subview形成稳定view identity | runtime request携带stable view/subview identity、target与order，支持多个consumer隔离 | Bevy不是Editor UX参考，不用ECS entity替代document/view instance合同 |
| Unity Graphics | URP按camera构造`UniversalCameraData`，包含target descriptor、pixel rect、render type、SceneView标识和resolve target；Base/Overlay stack有明确约束 | 每camera/per-view frame data与camera stack资格进入request/receipt | 本地语料不含Unity Editor SceneView源码，只能作为render consumer证据 |

结论是以Unreal的per-instance editor viewport owner作为主架构，Godot证明multi-view不是重复绘图，Fyrox校验preview/scene target ownership，Bevy与Unity Graphics约束runtime view identity和per-camera frame data。性能目标必须通过Zircon自己的qualification matrix证明，不能从参考结构推导“优于虚幻”。

## 7. 目标架构与硬切边界

### 7.1 核心合同

| 合同 | 必备字段/职责 |
|---|---|
| `EditorViewportSessionId` | project、document/play session、window、`ViewInstanceId`、session generation |
| `EditorViewportDefinition` | Scene/Game/Preview/custom kind、source binding、camera binding、persistence schema、capabilities |
| `EditorViewportSession` | lifecycle state、target lease、input route、settings、visibility、dirty reasons、last-good receipt |
| `ViewportRenderRequest` | request id、session id/generation、source/camera/selection/settings/size/quality epoch、target、deadline、policy |
| `ViewportRenderReceipt` | accepted/queued/superseded/rendered/published/presented/failed状态、完整source provenance、timing与failure |
| `ViewportFrameProduct` | session/request identity、format/colorspace/size、renderer generation、direct/capture payload、resource lease |
| `ViewportPresentationState` | Starting、Current、Stale、Degraded、Lost、Suspended、Closing及可见reason/action |
| `ViewportPresentationLease` | consumer/window/presenter、resource generation、accept/present fence、release/deadline |
| `ViewportProductMap` | 按`ViewInstanceId`/window查询产品；禁止全局`viewport_image` |
| `WorldUiSurfaceProduct` | UI tree generation、world transform、camera/layer、depth/billboard、texture/mesh、ray-UV input endpoint |

### 7.2 原子状态流

`Pane activates -> registry resolves/creates session -> request freezes source epochs -> runtime prepares target -> render -> post-render qualification -> publish product lease -> host matches pane/session/request -> presenter accepts -> present receipt -> last-good commit`。

任一阶段失败都必须产生typed receipt。resize和mode switch使用双缓冲式prepare/commit：旧产品保持`Current`或明确降为`Stale`，直到新target成功present首帧；失败不得清空可恢复owner，也不得把旧像素标成当前。关闭流程先停止新request、取消/排空队列、等待present lease或deadline、销毁target，最后发布closed receipt。

### 7.3 Scene、Game与world-space UI硬切

Scene session只能消费authoring document/world及自己的editor camera。Game session必须消费Editor07提供的play session/process与qualified game camera；没有play session时显示明确Unavailable/Stopped状态。duplicate、split和floating Scene都是新的session，不得复用全局camera/target/cursor。

world-space UI在真实runtime/editor UI surface product完成前必须从生产入口硬切Unavailable。完成后它不再把host template rectangle伪装成world surface，而是消费真实UI tree render target，以world quad/mesh参与depth和camera projection，并把pointer ray转换为qualified UV事件交给统一UI dispatcher。

## 8. 分层重构里程碑

### M0 · 能力真实性止血与证据锁定

撤下或禁用伪world-space UI；Game View在没有独立play产品时显示Unavailable；为旧图增加Stale/Degraded状态；修正runtime publish-before-validate。冻结fault tests和产品审计字段。

### M1 · Viewport session registry与per-pane product map

引入session id/definition/registry，按`ViewInstanceId + window`建立owner、size、visibility和last-good state；删除全局`viewport_image`与单controller消费假设。先支持两个Scene实例输出不同camera，再接Game。

### M2 · Request/receipt/currentness合同

为submission/product/capture补齐source/camera/settings/size/quality epoch、request id、状态和typed failure；实现latest-wins、ordered barrier、supersede与visible spatial snapshot同receipt提交。

### M3 · Atomic target/surface lifecycle与恢复

实现prepare/configure/warmup/commit resize，零尺寸/suspend、device loss、presenter upgrade/fallback和shutdown state machine；所有retryable错误保留dirty reason并按deadline退避。

### M4 · Direct GPU/capture lease与present receipt

将producer ring、presenter cache和shared registry纳入resource lease/fence；direct与capture使用独立cursor/mode epoch；admission reject驱动可见fallback/degraded，不只计数。

### M5 · Scene/Game/multi-window产品硬切

接入Editor07 play product，支持Scene+Game并显、duplicate/floating和1/2/3/4 layout；保存per-session camera/settings/split；建立后台window节流和总预算。

### M6 · 真实world-space UI

在Runtime UI父合同上实现UI render target、world projection/depth/billboard、ray/UV input、focus/capture与surface lifecycle；删除旧screen-rect debug path及兼容shim。

### M7 · 资格、性能与长期soak

完成fault/device/resize/multi-window/remote-play测试、资源与延迟预算、连续soak和产品审计回放；只有门槛全部通过后才能宣称工程级viewport。

## 9. 缺失测试与故障矩阵

| 维度 | 必测场景 | 当前缺口 |
|---|---|---|
| Identity | Scene/Game并显、两个Scene不同camera、duplicate/floating/multi-window | 只有layout/floating身份测试，没有像素/receipt隔离 |
| Currentness | edit/camera/resize/quality快速交错，旧request晚到 | 只按renderer generation轮询 |
| Resize | create失败、quality失败、submit失败、首帧失败、rollback | 只有happy resize order |
| Runtime race | render期间destroy/recreate，publish前generation变化 | 无fault injection，且当前顺序错误 |
| Direct/fallback | presenter升级失败、last presenter drop、mode往返、CPU capture恢复 | 只有单路径poll测试 |
| Resource lifetime | consumer stall超过3代、64 MiB拒绝、跨window LRU、present fence延迟 | 只有局部ring/LRU单元测试 |
| Surface/device | minimize、zero size、occlusion、surface lost、device lost、OOM | 无editor session恢复资格 |
| World UI | transform、billboard、depth遮挡、camera target、ray/UV、真实button action | 当前测试只证明矩形Quad和status |
| Shutdown | queued submit、capture、present lease、window detach、Drop error | 无close receipt与deadline测试 |
| Performance | 1/2/4视口、4K/HDR、resize storm、background throttle、cache churn | 无预算或benchmark门 |

## 10. 资格门

1. Scene与Game同时可见时必须拥有不同session id、source binding、camera binding、request和product receipt。
2. 没有active play session时Game View显示typed unavailable，不得显示Scene像素。
3. 两个Scene pane使用不同camera时，自动像素/metadata证据能区分输出。
4. duplicate/floating Scene不共享controller state、size cursor、input capture或last generation。
5. 主窗口与浮动窗口按各自presenter/window id消费产品，关闭一窗不清空另一窗产品。
6. 1/2/3/4 layout创建相应session数量并恢复split与per-view设置。
7. pane隐藏或遮挡后按策略suspend/throttle，重新显示能恢复而不误用别的pane产品。
8. 每个request携带document/world、camera、settings、size和quality epoch。
9. 产品只有在所有expected epoch匹配时才能进入`Current`。
10. 旧request晚到时被记录为superseded/dropped，不能替换last-good current产品。
11. renderer-visible spatial snapshot与同一presented receipt绑定，Editor03可验证一致性。
12. resize create失败保留旧viewport或last-good产品，并显示Stale/Degraded原因。
13. quality apply失败不提交半配置target，retry成功后原子切换。
14. submit瞬时失败保留dirty reason并自动重试，不需要无关用户输入。
15. repeated terminal失败进入有incident id的Degraded/Lost状态，不无限忙重试。
16. zero-size/minimized不会create非法target，也不会丢失可恢复session。
17. interactive resize有明确debounce/temporary resolution策略和最终精确尺寸barrier。
18. device loss能重建所有active session或逐pane报告失败，authoring文档不丢失。
19. runtime generation在render期间变化时，direct product registry没有任何新descriptor可见。
20. capture publication、visible snapshot和success stats都发生在post-render qualification之后。
21. direct GPU与CPU capture各自维护cursor/mode epoch，切换不会漏帧或倒退。
22. runtime presenter升级失败时softbuffer继续显示qualified产品且状态可诊断。
23. last direct presenter退出后，每个active viewport都收到fallback barrier并恢复capture。
24. consumer stall超过三代时，已接受产品由lease/cache安全持有；未接受产品触发明确重取或fallback。
25. 64 MiB/256项cache admission reject会传播pane级degraded/fallback receipt。
26. 两个native window消费同一产品时，资源在两者present/release前不被回收。
27. direct product包含format、colorspace、alpha、size、dynamic-resolution与resolve metadata。
28. HDR/SDR或不同monitor切换不会用错误display transform显示旧产品。
29. presenter retryable surface acquire保持相同qualified product与present request。
30. terminal present错误先保存编辑状态与incident，再按window隔离/恢复策略处理。
31. session close停止新request、排空或取消队列、等待lease deadline并返回close receipt。
32. controller/drop期间的destroy失败进入诊断，不被静默吞掉。
33. world-space UI不可用阶段生产入口明确Unavailable，不能再画debug Quad冒充控件。
34. 真实world UI使用world transform、meter size与pixels-per-meter生成产品。
35. billboard与depth test分别有相机朝向、遮挡和像素证据。
36. camera target决定实际投影相机/eye/layer，错误target返回typed failure。
37. pointer通过ray-plane/mesh交点和UV进入真实UI dispatcher，遮挡面不抢输入。
38. pointer capture按pointer/window/viewport/surface generation限定，并正确处理Cancel/close。
39. overlay合成有稳定tree/node/z/clip namespace和command/像素预算。
40. 1/2/4视口性能门记录CPU extract、GPU render、queue、capture、present age和显存。
41. background/inactive viewport有公平性与总预算，不能饿死active viewport。
42. resize storm、camera movement和scene edit的request coalescing符合声明策略。
43. 30分钟多窗口resize/mode-switch soak无资源增长、stale冒充或generation错配。
44. fault injection覆盖create、quality、submit、publish、cache reject、surface lost和device lost。
45. 端到端审计能从pane追踪到request、runtime viewport、frame、resource和present receipt。
46. 所有资格在Windows native主路径通过；只有明确Linux要求才进入WSL验证。

## 11. 禁止的临时修补

1. 禁止继续把一个global image按pane kind复制到Scene/Game来伪装多视口。
2. 禁止用`latest_generation + 1`、随机UUID或pane title代替session/request/source epoch。
3. 禁止在错误后简单清空图像而不保留last-good、状态和retry receipt。
4. 禁止为resize加入固定sleep或无限重试循环代替原子state machine。
5. 禁止只把runtime `publish`代码下移而不建立request qualification和失败不可见测试。
6. 禁止把三代ring增大为更大常量来代替consumer lease/present fence。
7. 禁止让direct和capture继续共享cursor，再用模式切换时清零掩盖丢帧。
8. 禁止把world字段传入shader但仍不实现真实UI product、depth和input dispatch。
9. 禁止为Game View复制一份editor world snapshot代替Editor07 play session产品。
10. 禁止建立第二套window loop、camera model、tool capture、quality profile或renderer cache。
11. 禁止保留旧global viewport/controller API作为长期compat shim；迁移完成后必须硬切删除。
12. 禁止以截图“看起来正常”替代身份、currentness、fault、lease和present receipt证据。

## 12. 本轮状态与后续入口

本轮完成current-source静态审查与参考引擎对照，未修改Rust、Cargo、assets或tooling，未运行Cargo、Editor、真实GPU presenter、fault injection、device loss、multi-window、soak或benchmark。工具优化按用户要求暂不纳入。

实施前第一动作是重取10份dirty路径和86文件fingerprint。优先顺序为：先关闭P0-04 publication顺序与失败不可见，再为P0-01建立per-pane session/product map，同时以P0-02原子lifecycle替换旧controller；P0-03在真实world UI父合同完成前保持Unavailable。任何实现不得越过Editor03/07/30/53与Runtime09a/09b/11c/57/65/79的唯一owner边界。
