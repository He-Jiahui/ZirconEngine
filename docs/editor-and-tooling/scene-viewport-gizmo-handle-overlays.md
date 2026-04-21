---
related_code:
  - zircon_scene/src/lib.rs
  - zircon_scene/src/components/mod.rs
  - zircon_framework/src/render/camera.rs
  - zircon_framework/src/render/frame_extract.rs
  - zircon_framework/src/render/overlay.rs
  - zircon_framework/src/render/scene_extract.rs
  - zircon_scene/src/world/render.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_graphics/src/scene/mod.rs
  - zircon_graphics/src/scene/resources/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/viewport_overlay_renderer/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/viewport_overlay_renderer/record/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/viewport_overlay_renderer/record/scene_content/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/viewport_overlay_renderer/record/overlays/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/passes/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/passes/scene_gizmo_pass/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/icons/viewport_icon_atlas/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/shaders/icon.wgsl
  - zircon_graphics/src/scene/scene_renderer/primitives/mod.rs
  - zircon_graphics/src/tests/project_render.rs
  - zircon_graphics/src/tests/scene_overlay.rs
  - zircon_scene/tests/render_frame_extract.rs
  - zircon_scene/tests/viewport_packet.rs
  - zircon_editor/build.rs
  - zircon_editor/src/builtin_assets.rs
  - zircon_editor/src/ui/binding/mod.rs
  - zircon_editor/src/tests/ui/binding/mod.rs
  - zircon_editor/src/tests/ui/binding/viewport.rs
  - zircon_editor/src/core/editing/state/mod.rs
  - zircon_editor/src/scene/viewport/mod.rs
  - zircon_editor/src/scene/viewport/controller/mod.rs
  - zircon_editor/src/scene/viewport/handles/mod.rs
  - zircon_editor/src/scene/viewport/pointer/mod.rs
  - zircon_editor/src/scene/viewport/projection.rs
  - zircon_editor/src/ui/slint_host/app.rs
  - zircon_editor/src/ui/slint_host/ui.rs
  - zircon_editor/src/ui/slint_host/viewport/mod.rs
  - zircon_editor/src/ui/binding_dispatch/mod.rs
  - zircon_editor/src/core/editor_event/types.rs
  - zircon_editor/src/core/editor_event/runtime.rs
  - zircon_editor/src/ui/workbench/fixture/mod.rs
  - zircon_editor/src/ui/workbench/reflection/mod.rs
  - zircon_editor/src/ui/workbench/snapshot/mod.rs
  - zircon_editor/assets/viewport_gizmos/camera.pbm
  - zircon_editor/assets/viewport_gizmos/directional_light.pbm
  - zircon_editor/ui/workbench.slint
  - zircon_editor/src/tests/host/binding_dispatch.rs
  - zircon_editor/src/tests/editing/viewport.rs
implementation_files:
  - zircon_scene/src/lib.rs
  - zircon_scene/src/components/mod.rs
  - zircon_framework/src/render/camera.rs
  - zircon_framework/src/render/frame_extract.rs
  - zircon_framework/src/render/overlay.rs
  - zircon_framework/src/render/scene_extract.rs
  - zircon_scene/src/world/render.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_graphics/src/scene/mod.rs
  - zircon_graphics/src/scene/resources/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mod.rs
  - zircon_graphics/src/scene/scene_renderer/core/mod.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/viewport_overlay_renderer/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/viewport_overlay_renderer/record/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/viewport_overlay_renderer/record/scene_content/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/viewport_overlay_renderer/record/overlays/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/passes/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/passes/scene_gizmo_pass/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/icons/viewport_icon_atlas/mod.rs
  - zircon_graphics/src/scene/scene_renderer/overlay/shaders/icon.wgsl
  - zircon_graphics/src/scene/scene_renderer/primitives/mod.rs
  - zircon_graphics/src/tests/scene_overlay.rs
  - zircon_scene/tests/render_frame_extract.rs
  - zircon_scene/tests/viewport_packet.rs
  - zircon_editor/build.rs
  - zircon_editor/src/builtin_assets.rs
  - zircon_editor/src/ui/binding/mod.rs
  - zircon_editor/src/core/editing/state/mod.rs
  - zircon_editor/src/scene/viewport/mod.rs
  - zircon_editor/src/scene/viewport/controller/mod.rs
  - zircon_editor/src/scene/viewport/handles/mod.rs
  - zircon_editor/src/scene/viewport/pointer/mod.rs
  - zircon_editor/src/scene/viewport/projection.rs
  - zircon_editor/src/ui/slint_host/app.rs
  - zircon_editor/src/ui/slint_host/ui.rs
  - zircon_editor/src/ui/slint_host/viewport/mod.rs
  - zircon_editor/src/ui/binding_dispatch/mod.rs
  - zircon_editor/src/core/editor_event/types.rs
  - zircon_editor/src/core/editor_event/runtime.rs
  - zircon_editor/src/ui/workbench/fixture/mod.rs
  - zircon_editor/src/ui/workbench/reflection/mod.rs
  - zircon_editor/src/ui/workbench/snapshot/mod.rs
  - zircon_editor/assets/viewport_gizmos/camera.pbm
  - zircon_editor/assets/viewport_gizmos/directional_light.pbm
  - zircon_editor/ui/workbench.slint
plan_sources:
  - user: 2026-04-15 Scene Viewport Gizmos/Handle/Overlay 规范化方案
  - user: 2026-04-15 Section 1-4 最终规格与 proposed_plan 收束
  - user: 2026-04-15 PLEASE IMPLEMENT THIS PLAN
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
tests:
  - zircon_scene/tests/viewport_packet.rs
  - zircon_graphics/src/tests/project_render.rs
  - zircon_graphics/src/tests/scene_overlay.rs
  - zircon_editor/src/tests/ui/binding/viewport.rs
  - zircon_editor/src/tests/host/binding_dispatch.rs
  - zircon_editor/src/tests/host/slint_builtin_assets.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/viewport/pointer_bridge.rs
  - zircon_editor/src/tests/host/slint_callback_dispatch/viewport/typed_command.rs
  - zircon_editor/src/tests/editing/state.rs
  - zircon_editor/src/tests/editing/viewport.rs
  - zircon_editor/src/ui/slint_host/ui.rs
  - cargo test -p zircon_scene --test viewport_packet -- --nocapture
  - cargo test -p zircon_graphics project_render -- --nocapture
  - cargo test -p zircon_graphics --lib -- --nocapture
  - cargo test -p zircon_editor --lib --locked
  - cargo test -p zircon_editor --lib --locked
  - cargo test -p zircon_editor --lib viewport_ --offline -- --nocapture
  - cargo check -p zircon_editor --lib --offline
  - cargo test -p zircon_editor --lib slint_builtin_assets -- --nocapture
  - cargo test -p zircon_editor --lib editing::state -- --nocapture
  - cargo test -p zircon_editor --lib scene_document_pane_projects_viewport_toolbar_state -- --nocapture
  - cargo test -p zircon_editor --lib typed_viewport_command_dispatch_updates_render_packet_without_pointer_bridge -- --nocapture
  - cargo check -p zircon_editor --lib --locked --target-dir target/codex-shared-b
  - cargo test -p zircon_editor --lib --no-run --locked --target-dir target/codex-shared-b
  - cargo test -p zircon_editor --lib viewport --locked --target-dir target/codex-shared-b
  - cargo test -p zircon_graphics --lib project_render --locked --target-dir target/codex-shared-b
  - cargo check --workspace --locked --target-dir target/codex-shared-b
doc_type: module-detail
---

# Scene Viewport Gizmo, Handle, And Overlay Pipeline

## Purpose

这组改动把 Scene 视图从“基础场景 + 零散 gizmo 特判”推进到分层 packet 模式：

- `zircon_editor` 负责 viewport 状态、typed command、editor-owned camera/handle 交互，以及 selection/grid/gizmo/handle overlay 组装。
- `zircon_scene` 负责把运行时世界抽成基础 scene/preview packet，不生成 editor-owned overlay。
- `zircon_graphics` 只消费 packet，把 sky/base/outline/wireframe/grid/gizmo/handle 叠层渲染出来。

目标不是补一个临时 Camera/Light 图标，而是固定 Scene 视图后续继续长功能时的边界。

## Related Files

- `zircon_scene/src/components/mod.rs`
- `zircon_framework/src/render/camera.rs`
- `zircon_scene/src/world/render.rs`
- `zircon_editor/src/scene/viewport/render_packet.rs`
- `zircon_graphics/src/scene/resources/mod.rs`
- `zircon_graphics/src/scene/scene_renderer/core/mod.rs`
- `zircon_graphics/src/scene/scene_renderer/mesh/mod.rs`
- `zircon_graphics/src/scene/scene_renderer/overlay/mod.rs`
- `zircon_graphics/src/scene/scene_renderer/overlay/viewport_overlay_renderer/mod.rs`
- `zircon_graphics/src/scene/scene_renderer/overlay/passes/*.rs`
- `zircon_graphics/src/scene/scene_renderer/overlay/icons/*.rs`
- `zircon_graphics/src/scene/scene_renderer/overlay/shaders/*.wgsl`
- `zircon_graphics/src/scene/scene_renderer/primitives/mod.rs`
- `zircon_editor/build.rs`
- `zircon_editor/src/builtin_assets.rs`
- `zircon_editor/src/ui/binding/mod.rs`
- `zircon_editor/src/core/editing/state/mod.rs`
- `zircon_editor/src/scene/viewport/controller/mod.rs`
- `zircon_editor/src/scene/viewport/handles/mod.rs`
- `zircon_editor/src/scene/viewport/pointer/mod.rs`
- `zircon_editor/src/scene/viewport/projection.rs`
- `zircon_editor/src/ui/binding_dispatch/mod.rs`
- `zircon_editor/src/core/editor_event/runtime.rs`
- `zircon_editor/src/ui/workbench/reflection/mod.rs`
- `zircon_editor/src/ui/slint_host/viewport/mod.rs`

## Behavior Model

### Viewport Settings Authority

`SceneViewportSettings` 现在是 Scene viewport 的唯一配置模型，字段覆盖：

- tool: `Drag | Move | Rotate | Scale`
- transform space: `Local | Global`
- projection: `Perspective | Orthographic`
- view orientation: `User | PosX | NegX | PosY | NegY | PosZ | NegZ`
- display mode: `Shaded | WireOverlay | WireOnly`
- grid mode: `Hidden | VisibleNoSnap | VisibleAndSnap`
- translate / rotate / scale snap step
- preview lighting / skybox
- gizmos enabled

2026-04-19 的 boundary cutover 进一步固定了导入边界：`SceneViewportSettings`、`SceneViewportExtractRequest`、`SceneViewportTool`、`TransformSpace`、`ViewOrientation`、`GridMode` 这组 editor authoring / viewport request 类型现在统一从 `zircon_framework::render` 直接导入，`zircon_scene` 根 crate 不再替它们做转发 re-export。

同日继续往 runtime/editor 边界再压一层之后，`SceneViewportExtractRequest` 也不再直接携带 selection。它现在只吃中性的 `ViewportRenderSettings + active_camera_override + camera + viewport_size`；`projection_mode / display_mode / preview_lighting / preview_skybox` 这四项通过 `ViewportRenderSettings` 从 editor state 投影出来，而 tool/grid/selection/gizmo 仍留在 editor authoring 状态里。

同一轮继续推进到 graphics 侧之后，`DisplayMode`、`ProjectionMode`、`ViewportCameraSnapshot`、`RenderFrameExtract`、`RenderSceneSnapshot`、`ViewportIconId`、`OverlayBillboardIcon`、`OverlayLineSegment`、`OverlayWireShape`、`HandleElementExtract` 等剩余 render/overlay DTO 也已经全部由 `zircon_framework::render` 直接提供；`zircon_graphics` 生产代码里的 scene 语义入口也已切到 `zircon_framework::scene`，`zircon_scene` 在 graphics crate 内只剩 tests fixture 依赖。

继续收尾到 `zircon_scene` 自身之后，`SceneGizmoKind`、`OverlayWireShape`、`RenderFrameExtract`、`RenderWorldSnapshotHandle` 这批 crate-local tests 先前还在通过 `zircon_scene` 根级入口取值的 render 类型也已经切到 `zircon_framework::render`；`zircon_scene/src/lib.rs` 不再保留这组 framework-owned render re-export。

`ViewportCommand` 不再只有 pointer 输入；toolbar、右上角方向、显示切换、snap、preview、Gizmos 开关都走同一套 typed payload。

### Scene Packet Layout

`RenderSceneSnapshot` 现在等价于 `SceneViewportRenderPacket`，固定拆成三块：

- `scene`: 基础几何、灯光、当前 viewport camera snapshot
- `overlays`: selection highlight、selection anchor、grid、scene gizmo、handle、display mode
- `preview`: lighting/skybox override 与 fallback sky 策略

这些 packet 类型不再堆在 `zircon_scene` 的历史组件文件里；当前目录树固定为：

- `framework/render/camera.rs`: camera snapshot、viewport settings、extract request、aspect ratio helper
- `framework/render/scene_extract.rs`: geometry/light/preview extract 与 `SceneViewportRenderPacket`
- `framework/render/overlay.rs`: overlay DTO 与 pick/icon/handle packet 契约
- `zircon_editor/src/scene/viewport/render_packet.rs`: editor-owned selection/grid/gizmo/handle overlay 组装

本轮之后，`World::build_viewport_render_packet(...)` 仍然消费 `SceneViewportExtractRequest`，但它只负责 runtime-owned 的基础 scene extract；scene/editor/graphics 的调用点也已经不再经由 `zircon_scene` 根级入口旁路这些契约类型。

具体到 renderer 内部，`scene_renderer/overlay/**`、`primitives/**`、`post_process/**`、`visibility/**`、`runtime/render_framework/**` 与 `src/tests/**` 这一整条 graphics 消费链现在都只看 `zircon_framework::render` 的中性 packet/overlay DTO；这把 gizmo/icon/pick/camera/display mode ownership 彻底压回 framework layer，而不是继续藏在 `zircon_scene` 根 crate 的 re-export 里。

`World::build_viewport_render_packet(...)` 现在只负责 scene 侧基础抽取：

- runtime camera snapshot
- mesh / light geometry extract
- preview lighting / skybox packet
- display mode 透传到 render overlay packet

selection highlight / selection anchor / grid / scene gizmo / handle overlay 已经全部转到 `zircon_editor::scene::viewport::render_packet` 这一层基于 runtime world + editor state 再组装，不再由 runtime world 直接生成。

### Handle And Scene Gizmo Split

Scene gizmo 和 handle 现在是物理分层：

- scene gizmo DTO: `zircon_framework::render::SceneGizmoOverlayExtract`
- scene gizmo 生成: `zircon_editor::scene::viewport::render_packet`
- handle: `zircon_editor::editing::viewport::HandleToolRegistry`

行为边界固定为：

- renderable object 选中时始终走 outline + 可选 tint
- non-renderable object 选中时靠 scene gizmo / selection anchor / handle 保持可见
- `Gizmos Off` 只清空 scene gizmo，不清空 handle
- `Drag` 工具下不生成 handle，`Move / Rotate / Scale` 下才生成

### Editor-Owned Camera And Handle Interaction

`SceneViewportController` 现在在 editor 内部维护：

- viewport size
- editor-owned camera override (`ViewportCameraSnapshot`)
- orbit target
- hover axis
- drag session

它直接产出 render request 所需 camera snapshot，而不是再把 scene 内 active camera 当成 editor 交互状态存储区。

当前 controller 支持：

- right mouse orbit
- middle mouse pan
- wheel zoom
- `AlignView(...)`
- projection switch
- `FrameSelection`
- move / rotate / scale handle overlay 构建
- move / rotate / scale handle drag 的 preview transform
- snap 对 translate / rotate / scale 的量化

### Shared Pointer Route And Selection Semantics

`zircon_editor/src/scene/viewport/projection.rs` 现在把 viewport 内共用的 CPU 投影计算单独收口出来：

- world -> screen projection
- world-units-per-pixel 估算
- 线段/圆环屏幕距离计算

`zircon_editor/src/scene/viewport/pointer/mod.rs` 现在在这层之上建立 `ViewportOverlayPointerRouter`，把 viewport overlay 命中真源前移到 shared retained surface：

- bridge 会把 handle overlay、scene gizmo `pick_shapes` 和 renderable 候选先投成一棵最小 `UiSurface`
- retained tree 里保留 root、viewport，以及每个 coarse candidate 的 frame 和 z-order
- 真正的精确判定不在节点构建阶段完成，而是在 viewport 节点上的 `UiPointerDispatcher` 基于 `route.stacked` 做二次筛选
- dispatcher 会统一产出 `ViewportPointerRoute::{HandleAxis, SceneGizmo, Renderable}`
- 路由优先级固定为 `HandleAxis > SceneGizmo > Renderable`，同优先级再按屏幕距离分数和 depth 决定

这让 Scene 视图终于具备了规格里要求的点击规则：

- `Drag` 模式下，单击对象仍然提交 selection
- 主按钮拖拽只有超过阈值才切换为导航
- `Move / Rotate / Scale` 下，未命中 handle 时按普通对象选择处理
- `Gizmos On` 时，non-renderable node 可以通过 scene gizmo pick shape 被选中

`zircon_editor/src/scene/viewport/controller/mod.rs` 现在只负责同步 `ViewportPointerLayout` 并消费 `ViewportPointerRoute`，而不再持有一份本地 overlay hit cache 或额外的 `picking.rs` 选择语义。这样 viewport 里“谁接住 hover/down”的判断，也已经和 menu、dock target、asset list 一样进入 shared `UiSurface + UiPointerDispatcher` 的统一边界。

### Handle Tool Registry

`HandleToolRegistry` 现在不再只是一个大函数集合，而是按工具分层的 trait 注册表：

- `MoveHandleTool`
- `RotateHandleTool`
- `ScaleHandleTool`

registry 统一负责：

- `build_overlay(...)`
- `begin_drag(...)`
- `update_drag(...)`
- `end_drag(...)`

这把 move / rotate / scale 的 overlay 构建和拖拽更新从 controller 内部拆回 handle 层，和 scene gizmo provider 的职责边界保持一致。

### Single-Command Undo For Handle Drag

viewport pointer-driven handle drag 现在不再只改 scene transform 而不进历史栈。

`EditorState::handle_viewport_input(...)` 会根据 `SceneViewportController::is_handle_drag_active()` 的生命周期边界自动：

- 在 handle drag 开始时调用 `EditorHistory::begin_drag(...)`
- 在 handle drag 结束时调用 `EditorHistory::end_drag(...)`
- 将整次拖拽折叠成一条 `UpdateNodeCommand`

这样 Scene 视图里的 pointer 拖拽 finally 满足了“单次拖拽 = 单条 undoable 命令”的验收口径，而不再只是旧的测试专用 intent 辅助路径。

## Design And Rationale

### Why Packetize Now

这轮设计的关键不是多画几根线，而是禁止 renderer 继续持有 editor 语义。`zircon_graphics` 只需要知道：

- 该渲染哪些 mesh
- 哪些对象要 outline
- 当前要不要 wireframe
- 当前有没有 grid / gizmo / handle / sky fallback

这样后续新增 scene gizmo provider、viewport 预览选项、右上角方向控件时，不需要再把 UI 逻辑塞回 renderer。

### Why Editor Owns Camera State

Scene 视图里的 orbit/pan/zoom、orthographic size、方向对齐都属于 editor 观察状态，不应该写回世界数据。  
`SceneViewportController` 通过 `request.camera` 覆盖 scene 侧默认相机抽取，避免把 editor 观察相机和场景内真实 camera 节点混成一件事。

### Why Keep Reflection Action IDs

`workbench/reflection/mod.rs` 现在为 viewport 相关动作保留了稳定 action id 映射，例如：

- `tool_move`
- `projection_orthographic`
- `align_neg_z`
- `display_wire_only`
- `grid_snap`
- `gizmos_off`
- `frame_selection`

这让 shell / reflection / headless 自动化能共享同一套命令命名，不必再发匿名字符串再让 runtime 猜语义。

### Why Split The Graphics Renderer Again

`zircon_graphics` 这轮不是停留在“行为上已经像分层”。`scene_renderer` 已经被切成真正的目录化子模块：

- `core/mod.rs`: renderer 生命周期入口，具体 core/history/target/render 行为继续下沉到 `core/*`
- `mesh.rs`: mesh draw 抽取与 pipeline cache
- `overlay.rs`: 纯结构入口，只保留 `mod/pub use`
- `viewport_overlay_renderer/mod.rs`: `ViewportOverlayRenderer` façade，`record/scene_content/` 与 `record/overlays/` 负责分域录制，buffer 准备和 pass orchestration 继续保持子树分层
- `overlay/passes/*.rs`: 每个 render pass 各自单文件
- `overlay/icons/*.rs`: atlas / sprite / slot / entry 分层
- `primitives/mod.rs` + `primitives/*`: selection / wireframe / grid / gizmo / handle 顶点构建按声明、buffer、geometry helper、feature builder 继续拆分

这样 renderer 不再继续向一个 1000+ 行大文件堆状态和绘制规则，后续再加 scene gizmo provider、selection anchor 变体、icon 批处理时不会重新塌回“巨型函数 + helper 区块”。

### Viewport Icon Loading

viewport scene gizmo icon 现在有了独立的 builtin icon source 管线：

- `zircon_editor/build.rs` 生成 `viewport_gizmo_icon_manifest.rs`
- `zircon_editor/src/builtin_assets.rs` 把 `ViewportIconId -> &'static [u8]` 固化成 typed source
- `SlintViewportController` 在 attach renderer 时把 editor builtin source 注入 shared texture render service
- `zircon_runtime::graphics` 的 `SceneGizmoPass` 通过 crate-private `ViewportIconSource` + `ViewportIconAtlas` 懒加载纹理 icon

如果 icon source 缺失，graphics 仍会回退到线框 glyph，不会让 gizmo 图标完全消失；但 Scene editor 正常路径现在已经会优先走 editor builtin 资源，而不是继续把 Camera/Light 图标硬编码在 renderer 里。

## Control Flow

### Binding To Runtime

统一命令流现在是：

1. `EditorUiBindingPayload::ViewportCommand(...)`
2. `dispatch_viewport_binding(...)`
3. `EditorState::apply_viewport_command(...)`
4. `SceneViewportController`
5. `EditorState::render_snapshot()`
6. `SceneRenderer::render(...)`

`EditorEventRuntime` 的 viewport 事件分支也已经改成同一条命令流，避免 UI 绑定和 runtime replay 走两套状态机。

### Workbench Projection

这轮把 Scene viewport 的 shell 投影也补到了真正可用状态，而不再停留在“只有 typed payload，没有真实 chrome”的中间态：

- `EditorDataSnapshot` / `EditorChromeSnapshot` 直接携带 `SceneViewportSettings`
- `slint_host/ui.rs` 把它映射成 `PaneData.viewport: SceneViewportChromeData`
- `workbench.slint` 在 `Scene` pane 内渲染独立 `SceneViewportToolbar`

当前 Scene-only toolbar 固定分成两组：

- 左组：`Drag / Move / Rotate / Scale`、`Local / Global`、display mode、grid mode、translate / rotate / scale snap、preview lighting、preview skybox、Gizmos、frame selection
- 右组：`Perspective / Orthographic` 与 `+X / -X / +Y / -Y / +Z / -Z`

`Game` pane 明确不吃这组 editor-only chrome，只继续显示 viewport 内容本身，避免把 Scene 的编辑控件扩散到非作用域页面。

### Slint Host Callback Path

`slint_host/app.rs` 现在不再只接 pointer 类 viewport 回调，还会把 Scene toolbar/right-top 的 callback 参数解析成 typed `ViewportCommand`，再统一走：

1. `callback_dispatch::dispatch_viewport_command(...)`
2. `EditorEventRuntime`
3. `EditorState::apply_viewport_command(...)`
4. `SceneViewportController`

这样 Scene toolbar、right-top 方向控件、headless binding 测试和 runtime replay 共享同一套 typed command 语义，不需要再引入一套 Slint-only 字符串命令解释器。

### Packet Build Path

`EditorState::render_snapshot()` 的职责现在是：

1. 让 `SceneViewportController` 准备 viewport camera snapshot
2. 从 `SceneViewportSettings` 投影出 `ViewportRenderSettings`，连同 camera override / viewport size 构造 `SceneViewportExtractRequest`
3. 调用 `World::build_viewport_render_packet(...)` 取得基础 scene packet
4. 在 editor 层补齐 selection / grid / scene gizmo / handle overlay

### Slint Viewport Render Attachment

当前 `zircon_editor/src/ui/slint_host/viewport/**` 只负责消费 render framework 输出并导入 frame image；Scene viewport 的 gizmo overlay 只通过 render packet 里的 `ViewportIconId` 进入 graphics。真正的 icon bytes source 仍是 graphics 内部 seam，而不是 editor 对外可调用的 `BuiltinViewportIconSource` 注入 API；当没有绑定 icon source 时，renderer 会继续回退到线框 glyph。

### Graphics Pass Expectations

`ViewportOverlayRenderer::pass_order()` 现在把 Scene viewport pass 顺序固定成：

1. `PreviewSkyPass`
2. `BaseScenePass`
3. `SelectionOutlinePass`
4. `WireframePass`
5. `GridPass`
6. `SceneGizmoPass`
7. `HandlePass`

每个 stage 都是独立 pass object，而不是再由一个大 render function 临时拼顺序。

`project_render` 测试专门验证了：

- scene gizmo overlay 真实出像素
- project shader 颜色能驱动输出
- `WireOnly` 会压掉大部分 shaded fill 像素

`scene_overlay` 测试额外把 pass 顺序本身钉成编译期契约，避免后续再把 pass 重排回去。

## Edge Cases And Constraints

- `Gizmos Off` 时，scene gizmo extract 必须为空，但已选中 camera/light 仍保留 selection anchor，且 `Move / Rotate / Scale` handle 继续生成。
- `Drag` 工具下 renderable selection 只保留 highlight，不生成 handle。
- `preview_skybox = true` 且场景无 sky 时，graphics 必须稳定回退到 procedural sky。
- `preview_lighting = true` 且场景无灯时，renderer 必须回退到 preview key light，避免全黑。
- 当前 handle drag 仍按当前 editor 场景切片的 root-space transform 工作；更复杂的 parented/world-space 精细变换仍可继续增强。
- 当前 UI 层已经具备 typed viewport command 协议和 reflection action id，但更完整的 Scene toolbar/right-top shell 组合仍可以继续在 Slint 视图上做视觉和交互投影收尾。

## Test Coverage

### Scene

- `zircon_scene/tests/viewport_packet.rs`
  - renderable selection highlight
  - camera / directional light scene gizmo extraction
  - grid / preview / gizmo visibility packet mapping
  - 当前已切成 runtime-owned 断言：`cargo test -p zircon_scene --test viewport_packet --locked --target-dir target/codex-shared-b` 通过，并固定 runtime world 不再生成 selection / grid / scene gizmo overlay，只保留基础 preview / display packet
- `zircon_scene/tests/render_frame_extract.rs`
  - 已改为直接从 `zircon_framework::render` 导入 `RenderFrameExtract` / `RenderWorldSnapshotHandle`
  - `cargo test -p zircon_scene --test render_frame_extract --locked --target-dir target/codex-shared-b` 通过

### Graphics

- `zircon_graphics/src/tests/project_render.rs`
  - scene + gizmo overlay 出图
  - shader pipeline color 输出
  - `WireOnly` 抑制 shaded fill
- `cargo check -p zircon_graphics --lib --locked --target-dir target/codex-shared-b`
  - 当前 graphics import ownership cutover 编译通过
- `cargo test -p zircon_graphics --lib project_render --locked --target-dir target/codex-shared-b`
  - 当前 project render baseline 在直接消费 `zircon_framework::render` 后仍保持 7 个断言通过
- `zircon_graphics/src/tests/scene_overlay.rs`
  - `ViewportOverlayRenderer` pass 顺序固定为规格要求的 7 段

### Editor UI / Editor Runtime

- `zircon_editor/src/tests/ui/binding/viewport.rs`
  - typed viewport toolbar command native binding roundtrip
- `zircon_editor/src/tests/host/binding_dispatch.rs`
  - toolbar commands 更新 `SceneViewportSettings`
  - display/grid/preview/gizmos 切换进入 render packet
  - `Gizmos Off` 不影响 selected camera 的 handle
  - `Drag` 工具下 renderable selection 只有 highlight 没有 handle
- `zircon_editor/src/tests/editing/state.rs`
  - `Drag` 模式点击 renderable 仍然会提交 selection
  - scene gizmo 点击会选中 non-renderable directional light
  - pointer-driven handle drag 会折叠成单条 undo/redo 命令
- `zircon_editor/src/tests/host/slint_callback_dispatch/mod.rs`
  - typed `ViewportCommand` 不经过 pointer bridge 也能更新 render packet
- `zircon_editor/src/tests/host/slint_builtin_assets.rs`
  - viewport gizmo builtin icon manifest 同时暴露 camera / directional light 两种 icon bytes
- `zircon_editor/src/ui/slint_host/ui.rs`
  - Scene document pane 会投影 viewport toolbar/right-top 状态到 `PaneData.viewport`

最新验证证据：

- `cargo check -p zircon_editor --lib --locked --target-dir target/codex-shared-b`
- `cargo test -p zircon_editor --lib --no-run --locked --target-dir target/codex-shared-b`
- `cargo test -p zircon_editor --lib viewport --locked --target-dir target/codex-shared-b`
- `cargo test -p zircon_graphics --lib project_render --locked --target-dir target/codex-shared-b`
- `cargo check --workspace --locked --target-dir target/codex-shared-b`

## Plan Sources

- 用户在 2026-04-15 明确要求 Scene 视图按 Godot/Fyrox 风格做 Handle / Scene Gizmo / Outline / Wireframe / Preview 分层，而不是继续堆一个临时 renderer 特判链。
- 同一轮规格还明确要求 `ViewportCommand` 扩展到 toolbar / right-top / display / preview / snap 设置，并把 `Camera` / `DirectionalLight` 收束到 provider/registry。

## Open Issues Or Follow-up

- 当前 Scene toolbar 的 snap 精度采用紧凑 cycle chip，而不是自由数值输入；如果后续要做更精细 authoring，可以再加 numeric popup 或 direct text field。
- 右上角方向控件目前固定为六向轴 + 透视/正交切换，不包含自由旋转 mini-widget；这符合本期规格，但仍可作为后续增强项。
- 当前 handle drag 已覆盖 move / rotate / scale 的单对象编辑与 snap；更精确的 parented world-space 变换、GPU picking、multi-selection handle 仍然属于后续增强项。

