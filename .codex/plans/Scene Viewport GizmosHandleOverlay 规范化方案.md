# Scene Viewport Gizmos/Handle/Overlay 规范化方案

## Summary
- 作用域固定为 `Scene` 视图，不扩到 `Game` 视图。
- 采用分层注册式架构：`zircon_editor` 管状态与命令，`zircon_scene` 产出中立 extract packet，`zircon_graphics` 只按 packet 渲染。
- `Handle` 与 `Scene Gizmo` 明确拆成两套系统：`Handle` 负责编辑交互，`Scene Gizmo` 负责场景表达。
- 最终交付必须同时覆盖：对象选中高亮、Camera/Light 纯 Gizmos 表达、右上角方向控件、网格与吸附、线框模式、预览光照/天空盒、Gizmos 开关，以及对应测试链路。
- UI 布局固定采用 `A. Layered Corners` + `A2. Compact Groups`：左上为场景工具组，右上为方向/投影组，显示相关开关收纳为紧凑组。

## Core Spec
### 1. 分层边界
- `zircon_editor`
  - 唯一持有 `SceneViewportState`。
  - 只允许通过 typed `ViewportCommand` 改变状态。
  - 负责工具模式、坐标空间、投影模式、六向视角状态、网格/吸附参数、线框模式、预览光照、预览天空盒、Gizmos 开关、hover、drag session、hit cache。
  - 负责 `HandleToolRegistry`、handle 构建、handle 命中、拖拽会话、undo 聚合。
- `zircon_scene`
  - 不直接绘制 gizmo。
  - 接收 `SceneViewportExtractRequest`，输出基础场景几何、selection highlight extract、scene gizmo extract、preview 环境 extract。
  - 通过 `SceneGizmoRegistry` 以 provider 方式扩展节点类型，不允许在 renderer 内堆积巨型 `match`。
- `zircon_graphics`
  - 只消费 `SceneViewportRenderPacket`。
  - 将 `preview sky / base scene / outline / wireframe / grid / scene gizmo / handle` 拆成独立 pass。
  - 不理解 toolbar、Slint 字符串命令或 editor 内部状态机。
- 过渡约束
  - 当前 `World.selected_entity` 仅视为过渡实现。
  - 目标模型里，selection 来源必须是 editor 传入的 request，不再让 world 自己持有 editor 语义。

### 2. 对象规则
- `Cube / MeshRenderer / 任意有真实表面渲染内容的对象`
  - 未选中：只正常渲染，不显示 scene gizmo 图标。
  - 选中：始终显示外描边 + 轻微选中色偏。
  - `Drag` 模式：保留描边与色偏，不显示 transform handle。
  - `Move / Rotate / Scale` 模式：在描边基础上显示对应 handle。
  - `Gizmos Off`：不影响描边和选中色偏。
- `Camera / Light / 任意无真实表面渲染、依赖编辑器表达的对象`
  - 默认：`Gizmos On` 时 scene gizmo 常显。
  - `Camera` provider：图标 + 视锥线框。
  - `DirectionalLight` provider：图标 + 方向箭头。
  - 选中：scene gizmo 进入高亮态；若工具为 `Move / Rotate / Scale` 则叠加 handle。
  - `Drag` 模式：只强化选中态，不显示变换 handle。
  - `Gizmos Off`：scene gizmo 隐藏，但已选中对象仍需保留最小可识别锚点；若当前工具为 `Move / Rotate / Scale`，对应 handle 继续显示。
- Gizmos 开关边界固定为：
  - `Scene Gizmo` 是场景表达层。
  - `Handle` 是编辑交互层。
  - 两者不能共用一个总开关。

### 3. 工具、方向、网格、线框、预览
- 工具模式固定四态：`Drag | Move | Rotate | Scale`
  - `Drag` = 场景导航模式，不关闭选择。
  - 单击对象仍可选中。
  - 主拖拽超过阈值后进入导航并取消本次点击提交。
  - 右键始终 `Orbit`，中键始终 `Pan`，滚轮始终 `Zoom`。
  - `Move / Rotate / Scale` 下，命中 handle 后拖拽进入对象编辑；右键/中键/滚轮仍保留导航。
- 坐标空间固定二态：`Local | Global`
  - `Move / Rotate / Scale` 共用一个状态。
  - `Drag` 不消费此状态。
  - 顶栏切换，不做每工具独立记忆。
- 右上角方向控件固定为 `View Cube + Projection Chip`
  - `View Cube` 支持 `+X / -X / +Y / -Y / +Z / -Z`。
  - 点击任一轴向时，相机围绕当前 orbit target 对齐到绝对轴向。
  - 不改变 selection，不改变 tool mode。
  - 透视模式保留观察距离；正交模式保留正交尺寸。
  - 投影切换固定为 `Perspective | Orthographic`，仅切投影，不强制切轴向。
  - `view_orientation` 固定为 `User | PosX | NegX | PosY | NegY | PosZ | NegZ`。
- 网格模式固定三态：`Hidden | VisibleNoSnap | VisibleAndSnap`
  - `Hidden`：不显示网格，不启用吸附。
  - `VisibleNoSnap`：显示网格，不量化变换。
  - `VisibleAndSnap`：显示网格，并按当前工具启用量化。
  - 吸附精度固定三项：`translate_step`、`rotate_step_deg`、`scale_step`。
  - `Move` 按空间量化位移，`Rotate` 按角度量化，`Scale` 按步长量化，`Drag` 不量化。
- 显示模式固定三态：`Shaded | WireOverlay | WireOnly`
  - `Shaded`：正常材质渲染。
  - `WireOverlay`：材质正常渲染并叠加线框。
  - `WireOnly`：只渲染线框，不渲染材质填充。
  - `WireOnly` 下 renderable selection 改为“外描边 + 更亮选中线框色”，不再依赖填充色偏。
- 预览光照与天空盒均为 viewport preview override，不写回世界
  - `Preview Lighting On`：优先场景灯光；无灯时回退 editor preview key light。
  - `Preview Lighting Off`：关闭场景灯光与 fallback key light，但保留基础环境亮度，保证可读。
  - `Preview Skybox On`：优先场景天空；无天空时回退内建 procedural gradient sky。
  - `Preview Skybox Off`：不显示天空盒，仅用 editor 背景色清屏。
  - 默认值固定为：
    - `gizmos_enabled = true`
    - `preview_lighting = true`
    - `preview_skybox = true`
    - `fallback sky = procedural gradient sky`

## Interfaces And Data Flow
### 1. 核心状态与公共类型
- 在 `zircon_scene` 定义中立 DTO：
  - `ViewportTool = Drag | Move | Rotate | Scale`
  - `TransformSpace = Local | Global`
  - `ProjectionMode = Perspective | Orthographic`
  - `ViewOrientation = User | PosX | NegX | PosY | NegY | PosZ | NegZ`
  - `GridMode = Hidden | VisibleNoSnap | VisibleAndSnap`
  - `DisplayMode = Shaded | WireOverlay | WireOnly`
- `SceneViewportSettings` 固定字段：
  - `tool`
  - `transform_space`
  - `projection_mode`
  - `view_orientation`
  - `gizmos_enabled`
  - `display_mode`
  - `grid_mode`
  - `translate_step`
  - `rotate_step_deg`
  - `scale_step`
  - `preview_lighting`
  - `preview_skybox`

### 2. Editor 层接口
- `SceneViewportState`
  - 持有 `SceneViewportSettings`
  - 持有 `ViewportHoverState`
  - 持有 `Option<ViewportDragSession>`
  - 持有 `OverlayHitTestCache`
- `ViewportCommand` 必须从仅鼠标输入扩展为：
  - `PointerMoved`
  - `PointerPressed`
  - `PointerReleased`
  - `Scrolled`
  - `Resized`
  - `SetTool`
  - `SetTransformSpace`
  - `SetProjectionMode`
  - `AlignView`
  - `SetDisplayMode`
  - `SetGridMode`
  - `SetTranslateSnap`
  - `SetRotateSnapDegrees`
  - `SetScaleSnap`
  - `SetPreviewLighting`
  - `SetPreviewSkybox`
  - `SetGizmosEnabled`
  - `FrameSelection`
- `SceneViewportController`
  - 唯一负责把 `ViewportCommand` 应用到 `SceneViewportState`
  - 负责从 editor state 生成 `SceneViewportExtractRequest`
- `HandleTool` / `HandleToolRegistry`
  - `Drag` 不注册 handle provider。
  - `Move / Rotate / Scale` 各自一个 `HandleTool`。
  - `HandleTool` 固定职责：`build_overlay`、`begin_drag`、`update_drag`、`end_drag`。
  - handle 构建只读取 `SceneViewportSettings`，不额外隐藏 Local/Global 或 snap 状态。
  - 命中缓存先固定使用 CPU 投影命中，不引入 GPU picking buffer。

### 3. Scene 层接口
- `SceneViewportExtractRequest`
  - `settings: SceneViewportSettings`
  - `selection: SmallVec<[EntityId; 4]>`
  - `active_camera_override: Option<EntityId>`
- `SceneGizmoRegistry`
  - 持有 `Vec<Box<dyn SceneGizmoProvider>>`
  - 第一批 provider 只落地：
    - `Camera`
    - `DirectionalLight`
- `SceneGizmoProvider`
  - `supports(world, entity) -> bool`
  - `build(ctx, out)`
- `SceneGizmoOverlayExtract`
  - `owner`
  - `kind`
  - `selected`
  - `lines`
  - `wire_shapes`
  - `icons`
  - `pick_shapes`
- `ViewportIconId` 第一批固定：
  - `Camera`
  - `DirectionalLight`
- `SelectionHighlightExtract`
  - 只用于 renderable 对象
  - 固定表达“是否描边 + 可选色偏”
- `Gizmos Off` 时
  - `SceneGizmoRegistry` 整体跳过构建
  - 非渲染节点不生成 selection outline，只能依赖 selection anchor / handle 保底可见性

### 4. 统一渲染 packet
- `RenderSceneSnapshot` 升级为三块：
  - `scene: RenderSceneGeometryExtract`
  - `overlays: RenderOverlayExtract`
  - `preview: PreviewEnvironmentExtract`
- `RenderOverlayExtract` 固定包含：
  - `selection: Vec<SelectionHighlightExtract>`
  - `grid: Option<GridOverlayExtract>`
  - `handles: Vec<HandleOverlayExtract>`
  - `scene_gizmos: Vec<SceneGizmoOverlayExtract>`
  - `display_mode: DisplayMode`
- `PreviewEnvironmentExtract`
  - `lighting_enabled`
  - `skybox_enabled`
  - `fallback_skybox`
- `FallbackSkyboxKind`
  - `None`
  - `ProceduralGradient`
- 调用顺序固定为：
  1. `zircon_editor` 接收 `ViewportCommand`
  2. `SceneViewportController` 更新 `SceneViewportState`
  3. `SceneViewportController` 生成 `SceneViewportExtractRequest`
  4. `zircon_scene` 构建 `scene + selection + grid + scene_gizmos + preview`
  5. `zircon_editor` 用 `HandleToolRegistry` 补 `handles`
  6. 合并为完整 `SceneViewportRenderPacket`
  7. `zircon_graphics` 渲染 packet

### 5. 图标加载规范
- viewport 内 scene gizmo 图标不复用 Slint toolbar string key。
- 统一使用 `ViewportIconId`。
- 图标来源固定为 editor 内建资源，不走项目 asset manager。
- 资源目录固定为类似 `zircon_editor/assets/viewport_gizmos/` 的 editor builtin asset 路径。
- `build.rs` 生成 icon manifest，运行时只允许按 `ViewportIconId` 获取资源，不允许路径字符串查找。
- `zircon_graphics` 侧只暴露 `ViewportIconAtlas` / `ViewportIconSource`，负责懒加载和 GPU cache。

## Rendering, Events, Tests
### 1. 渲染顺序
- 每帧 pass 顺序固定，不允许不同模式重排：
  1. `PreviewSkyPass`
  2. `BaseScenePass`
  3. `SelectionOutlinePass`
  4. `WireframePass`
  5. `GridPass`
  6. `SceneGizmoPass`
  7. `HandlePass`
- 约束：
  - `Selection highlight`、`Scene Gizmo`、`Handle` 三层永远独立。
  - `Gizmos Off` 只影响 `SceneGizmoPass`，不影响 `HandlePass`。
  - `Wireframe` 只改变基础渲染表现，不改变 gizmo/handle 是否生成。
  - `PreviewSkyPass` 内单独处理 default procedural sky，不把 fallback sky 散落到 base pass。
  - `WireOnly` 也必须保留可观测 selection 高亮。

### 2. 事件流
- UI 命令流固定：
  - `Slint/UI -> ViewportCommand -> SceneViewportController -> SceneViewportState`
  - 禁止匿名字符串命令，如 `Custom("ToggleWireframe")`
- hover 规则：
  - 优先命中 handle
  - 若未命中 handle 且 `gizmos_enabled = true`，再命中 scene gizmo
  - hover 只更新 transient state，不提交世界修改
- 选择规则：
  - `Drag` 模式下，按下后若未超过导航阈值且释放时命中对象，才提交 selection
  - `Move / Rotate / Scale` 模式下，若未命中 handle，则按正常选择规则提交 selection
  - 非渲染节点在 `Gizmos On` 时允许通过 gizmo pick shape 被选中
- 导航规则：
  - `Drag` 主拖拽优先进入导航
  - 右键恒为 orbit，中键恒为 pan，滚轮恒为 zoom
  - 导航只改 viewport camera state，不改世界对象
- handle 拖拽规则：
  - `begin_drag` 记录初始 transform / snap / space / camera basis
  - `update_drag` 生成 preview delta
  - 拖拽中只写 preview transform
  - `end_drag` 收束为单条 undoable `EditorIntent`
  - 一次拖拽不允许拆成多条历史命令

### 3. 必须满足的状态约束
- 无 selection 时，`HandlePass` 必须为空。
- 当前工具为 `Drag` 时，`HandlePass` 必须为空。
- `gizmos_enabled = false` 时，`SceneGizmoPass` 必须为空，但 `HandlePass` 不能因此变空。
- `preview_skybox = true` 且无场景天空时，必须稳定回退到 procedural sky，不能黑屏闪烁。
- `preview_lighting = true` 且无灯光时，必须稳定回退到 preview key light，不能全黑。
- `Camera / Light` 默认常显 scene gizmo，但必须能被 `Gizmos` 开关统一隐藏。
- 非渲染节点在被选中且 gizmo 被隐藏时，仍必须存在可识别编辑锚点。
- renderable object 的高亮完成标准是“outline 为主，tint 为辅”，不是简单整体改 tint。

### 4. 自动化测试与验收
- `zircon_scene` 单元测试
  - renderable object 选中时生成 `SelectionHighlightExtract`
  - renderable object 未选中时不生成 selection highlight
  - `Camera / DirectionalLight` 在 `gizmos_enabled = true` 时生成 scene gizmo extract
  - `gizmos_enabled = false` 时 scene gizmo extract 为空
  - `grid_mode` 三态正确映射
  - `preview_lighting / preview_skybox` 正确映射
  - `display_mode` 正确进入 render packet
- `zircon_editor` 单元测试
  - 全部 `ViewportCommand` 正确更新 `SceneViewportState`
  - `Drag` 模式主拖拽进入导航，不生成对象 transform 命令
  - `Move / Rotate / Scale` 命中 handle 拖拽后只提交一条 undo 命令
  - `Gizmos Off` 不影响 handle 生成
  - `Local / Global` 切换影响 handle overlay 结果
- `zircon_graphics` 渲染测试
  - `Shaded` 有材质像素
  - `WireOverlay` 同时存在材质与线框像素
  - `WireOnly` 无大面积材质填充
  - 非空 `scene_gizmo` packet 有 overlay 像素
  - 非空 `handle` packet 有 overlay 像素
  - `preview_skybox = true` 且无场景天空时输出不同于纯清背景
  - `preview_lighting = true` 且无灯时 mesh 不全黑
  - renderable selection 时 outline 有可观测边缘变化
- editor 集成测试
  - toolbar 按钮经由 `ViewportCommand` 更新运行时状态
  - 右上角六向轴更新 `view_orientation`
  - 透视/正交切换更新 `projection_mode`
  - `Gizmos` 开关只影响 scene gizmo，不影响 handle
  - `GridMode` 三态全链路正确
  - `DisplayMode` 三态全链路正确
  - `Camera / Light` 默认常显，关闭 Gizmos 后隐藏
  - 选中 mesh 有 outline/highlight；选中 camera/light 有 gizmo 强化 + handle
- 完成判定
  - 所有相关 UI 都通过 typed command 驱动统一状态模型
  - `Handle` 与 `Scene Gizmo` 已物理拆层
  - `Camera / Light` 已走 provider/registry，不是 renderer 特判
  - `Grid / Wireframe / Skybox / Lighting / Gizmos` 状态都收束到 `SceneViewportSettings`
  - 对象编辑拖拽稳定收束为单条 undo 命令
  - 至少存在 `scene/editor/graphics` 三层自动化覆盖核心分支

## Assumptions And Defaults
- 本期只做 `Scene` 视图，不动 `Game` 视图。
- 本期 `SceneGizmoProvider` 首批只实现 `Camera` 与 `DirectionalLight`，其他类型预留注册口。
- 本期右上角方向控件固定为六向轴 + 透视/正交切换，不做自由旋转 mini-widget。
- 本期命中系统固定采用 CPU 投影命中；GPU picking 不在本轮范围。
- 本期天空盒 fallback 固定为 procedural gradient sky，不引入项目级 sky asset authoring。
- 本期预览光照/天空盒均为 viewport override，不修改世界数据或序列化内容。
- 本期 `Drag` 的定义固定为“主按钮优先导航”，不是“禁用选择”的特殊模式。
- 本期不接受的实现形式：
  - 只给 UI 补静态按钮，不接 typed state/command
  - 继续把 gizmo 和 handle 混画或共用开关
  - 仍用简单 tint 代替 renderable outline
  - 仍把 camera/light 作为 renderer 硬编码特判
