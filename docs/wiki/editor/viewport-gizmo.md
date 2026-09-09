---
related_code:
  - zircon_editor/src/scene/viewport/mod.rs
  - zircon_editor/src/scene/viewport/settings.rs
  - zircon_editor/src/scene/viewport/controller/mod.rs
  - zircon_editor/src/scene/viewport/handles/mod.rs
  - zircon_editor/src/scene/viewport/pointer/mod.rs
  - zircon_editor/src/ui/retained_host/viewport/mod.rs
implementation_files:
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller.rs
  - zircon_editor/src/scene/viewport/handles/handle_tool_registry.rs
  - zircon_editor/src/scene/viewport/handles/move_handle_tool.rs
  - zircon_editor/src/scene/viewport/handles/rotate_handle_tool.rs
  - zircon_editor/src/scene/viewport/handles/scale_handle_tool.rs
  - zircon_editor/src/scene/viewport/pointer/runtime_picking_adapter.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 编辑器详细 Wiki
  - docs/editor-and-tooling/scene-viewport-gizmo-handle-overlays.md
  - docs/editor-and-tooling/viewport-interaction-boundary-split.md
tests:
  - zircon_editor/src/scene/viewport
  - zircon_editor/src/ui/retained_host/viewport/tests
doc_type: module-detail
---

# Scene Viewport 与 Gizmo

## 概览

Scene Viewport 是编辑器作者态交互与 runtime renderer 的交界面。编辑器拥有相机操作意图、工具模式、选择、Gizmo drag 和 chrome 设置；runtime/graphics 拥有世界抽取、可见性、渲染产品、GPU picking 和最终帧。

## Viewport 设置

`SceneViewportSettings` 是可序列化的视口配置：

| 字段 | 可选值/含义 |
| --- | --- |
| `transform_space` | `Local` / `Global` |
| `projection_mode` | Perspective/Orthographic 等 runtime render 枚举 |
| `view_orientation` | User、±X、±Y、±Z |
| `gizmos_enabled` | 是否构建作者态 gizmo overlay |
| `display_mode` | Shaded/Wireframe 等显示模式 |
| `grid_mode` | Hidden、VisibleNoSnap、VisibleAndSnap |
| `preview_lighting` | 预览光照开关 |
| `preview_skybox` | 预览天空盒开关 |

默认是 Local、Perspective、User、Gizmo 开启、Shaded、网格可见但不吸附、预览光照和天空盒开启。

`render_settings()` 只投影 renderer 需要的字段。`SceneViewportChromeSettings` 额外组合当前 mode、pivot 与来自 settings authority snapshot 的 translate/rotate/scale snap step，供工具栏展示；snap step 不是 Scene 资产内容。

```rust
use zircon_editor::scene::viewport::{
    GridMode, SceneViewportSettings, TransformSpace, ViewOrientation,
};

let mut settings = SceneViewportSettings::default();
settings.transform_space = TransformSpace::Global;
settings.grid_mode = GridMode::VisibleAndSnap;
settings.view_orientation = ViewOrientation::PosY;
let renderer_settings = settings.render_settings();
```

## 输入路由

`ViewportInput` 包含 pointer move、三键 press/release、scroll、resize。Left press 同时携带 `SelectionMutation`。

桌面输入路径：

```text
native window event
 -> shared UiSurface hit-test/capture
 -> viewport binding command
 -> ViewportInput
 -> SceneModeStack / handle tool / navigation
 -> ViewportFeedback + editor event effects
```

Pointer capture 必须保证 drag 过程中光标离开 viewport 后仍能收到 move/up。失焦或取消会终止交互并恢复未提交的 interactive transform。

## 相机导航

控制器把 right/middle drag、scroll 和 frame-selection 转成 editor-owned camera 意图，再投影为 runtime viewport/camera request。固定方向视图使用 `ViewOrientation`；切回 User 后恢复自由导航语义。

视口尺寸最小钳制为 `1x1`。Workbench 中真实 viewport content frame 由 pane surface 扣除 toolbar 后得到，host presentation 和 renderer 提交必须共用这一尺寸，不能用窗口大小替代。

## 拾取

拾取优先通过 `EditorRuntimeViewportPickRoute`：

1. 捕获当前 gateway lease 与完整 `GatewaySessionIdentity`。
2. 调用 `request_viewport_pick`，获得 opaque ticket。
3. 后续帧 `poll_viewport_pick`。
4. 完成后把 renderable/entity 映射为 selection candidate，或在取消时调用 `cancel_viewport_pick`。

Ticket 的整个生命周期必须留在同一 endpoint。项目切换或 runtime 重建后，旧 ticket 不可提交到新 gateway。

## Gizmo 与 Handle

公开工具类型 `TransformHandleKind` 表示 Move、Rotate、Scale。内部 `HandleToolRegistry` 持有对应 tool；各 tool 负责：

- 根据 selection、pivot、transform space 构造可视 handle。
- 生成 overlay line/wire/pick shape。
- hit-test 后建立 `HandleDragSession`。
- 把屏幕 delta 转成平移、旋转或缩放 delta。
- 应用 snap step。
- 更新 interactive transform，结束时提交事务。

`GizmoAxis::{X,Y,Z}` 表示轴向 handle。Local space 使用 subject/local basis，Global space 使用世界轴。旋转手柄需要把屏幕移动转换为角度；缩放应保护退化轴和非法数值。

## Pivot 和多选

`PivotMode` 决定多选操作的 pivot 语义。构建 handle 时使用 primary selection 和所有选择项的快照；拖动期间不能因为 projection rebuild 而换掉目标集合。若 selection/world generation 变化，交互应取消或拒绝提交。

## Overlay 分层

Viewport overlay 不是 UI 直接画到 GPU 命令流的任意列表。编辑器构建 typed extract：

- scene gizmo（camera/light 等图标）。
- transform handle element、line、wire 与 pick shape。
- selection anchor/highlight。
- grid 和 preview environment。

这些数据进入 runtime render framework 的 overlay contracts，与 scene mesh snapshot 分层。`DisplayMode::Wireframe` 影响 scene rendering，不应把 gizmo overlay 也改成 world wireframe。

## Highlight

选择 outline/tint 通过 `EditorRuntimeHighlightSet` 提交，包含 viewport、generation、entity 集合和渲染属性。Generation 用于避免旧 selection 覆盖新 selection；空或无效 viewport set 会被 gateway 拒绝。

## 帧获取

Serialized/runtime host 可使用 `bind_viewport_surface` + `present_viewport` 直接呈现，或 `capture_frame` 获取 `EditorRuntimeFrame` RGBA。捕获帧带 ABI version、尺寸和 generation；使用后调用 `release(self)`，对外部 ABI buffer 尤其重要。

若 renderer 没有新 frame，宿主不重复发布缓存图像。`EditorRuntimeFrameDemand` 告诉 event loop 下一次应 OnDemand、SleepUntil 或 Continuous。

## 状态与限制

- **已实现**：typed settings、导航、renderer picking、Move/Rotate/Scale、snap、overlay、highlight、frame capture。
- **内部实现**：`SceneViewportController`、session registry、handle registry 和大部分 render packet。
- GPU picking 取决于 endpoint capability；缺失时返回 `CapabilityMissing`，不能假设总有 CPU fallback。
- `GridMode::VisibleAndSnap` 才同时表达显示与吸附；`VisibleNoSnap` 不应用 snap。
- Gizmo drag 必须作为单个可撤销操作提交，cancel 不得污染历史。
