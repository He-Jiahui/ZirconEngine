# Viewport 交互边界重构计划

## 摘要
- 这轮只处理已经确认“明显错包”的 viewport interaction 集合：`ViewportController`、`ViewportState`、`ViewportInput`、`ViewportFeedback`、`GizmoAxis`，以及 `zircon_graphics/src/viewport/**`。
- 主归属按你的选择收口到 `zircon_editor`；`zircon_entry` 不再复用这套 editor 交互，而是改成私有的 runtime 相机控制。
- `zircon_graphics` 只保留渲染职责：frame 输出、offscreen/shared-texture 服务、scene overlay 渲染、render server 桥接都不搬。

## 关键改动
- 在 `zircon_editor/src/editing/viewport/interaction/` 新建 editor-owned 交互子树，承载 `gizmo_axis.rs`、`viewport_input.rs`、`viewport_feedback.rs`、`viewport_state.rs` 和 `mod.rs`。
- 更新 `zircon_editor/src/editing/viewport/mod.rs` 与现有 `controller/handles/pointer`，把所有 `use zircon_graphics::{...}` 的 viewport 相关引用切到本地 interaction 模块。
- 在 `zircon_editor/src/lib.rs` 增加对 `ViewportState`、`ViewportInput`、`ViewportFeedback`、`GizmoAxis` 的根级 re-export；同步更新 `EditorState::viewport_state()`、`EditorEventRuntime::viewport_state()` 等公开返回类型。
- 从 `zircon_graphics` 删除这组非渲染 API：
  - 删掉 `src/viewport/**`
  - 删掉 `src/types/gizmo_axis.rs`、`viewport_input.rs`、`viewport_feedback.rs`、`viewport_state.rs`
  - 从 `src/types/mod.rs`、`src/lib.rs` 移除对应 re-export
  - 更新 crate 顶层注释，不再宣称拥有 viewport control
- 保留在 `zircon_graphics` 的内容：
  - `EditorOrRuntimeFrame`
  - `ViewportFrame`
  - `ViewportFrameTextureHandle`
  - `RenderService` / `SharedTextureRenderService`
  - `RuntimePreviewRenderer`
  - `SceneRenderer` / `ViewportIconSource`
- 在 `zircon_entry/src/entry/runtime_entry_app/` 下新增私有 `camera_controller/` 子树，替换当前对 `zircon_graphics::ViewportController` 的依赖。
- 新 runtime 控制器只实现：
  - viewport size 管理
  - orbit
  - pan
  - zoom
  - orbit target 与当前 selected node 的位置同步
- runtime 明确移除旧的隐式对象编辑行为：左键不再沿 XYZ 轴拖动选中节点，也不再依赖 gizmo 轴判定。
- 从 `zircon_editor/Cargo.toml` 移除 `zircon_graphics` 依赖；`zircon_entry` 继续保留 `zircon_graphics` 依赖，因为启动模块集仍要注册 graphics module。

## 公共接口变化
- 删除 `zircon_graphics::{ViewportController, ViewportState, ViewportInput, ViewportFeedback, GizmoAxis}`。
- 新增 `zircon_editor::{ViewportState, ViewportInput, ViewportFeedback, GizmoAxis}`。
- `zircon_entry` runtime 不再暴露或复用共享 viewport controller；改为 crate-private `RuntimeCameraController`。
- 本轮不做 `GizmoAxis -> zircon_scene::OverlayAxis` 的统一，`GizmoAxis` 先作为 editor-owned 类型保留。

## 测试与验收
- `cargo test -p zircon_editor`
  - 现有 viewport/editor state/workbench snapshot 测试全部改到新类型路径
  - 增加边界回归：确认 editor 源码与 `Cargo.toml` 不再通过 `zircon_graphics` 获取 viewport interaction 类型
- `cargo test -p zircon_entry`
  - 给新的 `camera_controller` 补 unit tests，覆盖 `resize`、`orbit`、`pan`、`zoom`
  - 增加回归测试，确认 runtime 左键输入不再修改 selected node transform
- `cargo test -p zircon_graphics`
  - 删除旧 `ViewportController` 相关测试
  - 保证 render server、scene overlay、project render 等现有测试仍通过
- 最后执行受影响 crate 的联测或至少 `cargo check`，确认跨 crate API 断点都已修复

## 假设
- 本次“找全”按“证据充分的明显错包模块”执行；目前只有 viewport interaction 这一组达到迁移标准。
- `RenderService`、`SharedTextureRenderService`、`RuntimePreviewRenderer`、`ViewportIconSource` 等虽然面向 editor 场景也能被用到，但本质仍是渲染/GPU 服务，不在这轮迁移范围。
- 不保留 `zircon_graphics` 里的兼容 re-export；直接完成边界切换。
