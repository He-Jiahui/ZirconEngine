# Zircon UI 增量布局、增量重绘与控件池优化计划

## Summary

按保留式 UI 树改造当前逻辑：初次加载或显式热重载才从描述文件构建；普通尺寸、文本、状态、子节点变化只在现有 `UiSurface` / `UiTree` 上增量传播 dirty、局部布局、局部渲染缓存更新。

参考方向：
- Unreal/Slate：显式 invalidation reason、invalidation root、布局与绘制分离、WidgetPool 复用。
- Bevy UI：保留节点树，按组件/资源变化触发布局，避免无变化节点参与刷新。
- Fyrox UI：节点池/句柄式 UI 图，消息驱动修改，运行时不靠描述文件重建。

## Key Changes

- 增加 UI dirty 传播模型：区分 `SelfGeometry`、`ChildLayout`、`ParentGeometry`、`TextMeasure`、`Visual`、`State`、`Input`、`Structure`。
- 把当前全局 `rebuild_dirty(root_size)` 改为“最小 dirty 范围”处理：
  - 子节点位置/大小、文本测量、子节点增删等自下而上变化，只向 `ContentDriven` 或自动编排容器传播。
  - `Free`、`Overlay`、`Fixed`、`ParentDirected` 这类父容器不因普通子布局变化重新布局。
  - 根画布或父元素尺寸变化自上而下 arrange；子元素最终 frame/clip 未变化时不标记 render dirty。
- 保留现有全量 `compute_layout_tree` 作为初次构建、显式强制重排、调试回退路径；新增增量 layout pass 作为默认路径。
- 为 render extract 增加节点级缓存：
  - 用 `UiNodeId` 记录上一帧 geometry fingerprint、visual fingerprint、text fingerprint、render generation。
  - frame/clip/style/text/image/opacity 全部不变时复用上一条绘制命令，不进入重绘 damage。
  - 只把旧 frame 与新 frame 的并集加入 damage region。
- 增加控件池：
  - 在 runtime UI 层实现 `UiControlPool` / `UiSurfaceNodePool`，按稳定模板 key、控件类型、组件路径复用节点。
  - 删除子树时 detach 到池中；新增同类节点时优先复用，清理瞬态输入/焦点状态，保留可安全复用的容量和缓存结构。
  - 普通属性变化、尺寸变化、文本变化不得重新加载 `.ui.toml` 或重建整棵树。
- 改造编辑器预览路径：
  - `UiAssetPreviewHost::rebuild_with_size` 只更新根尺寸并跑增量 layout。
  - `ui_asset_editor_node_projection(size)` 改为使用缓存的编译结果和保留式 surface session；只有文件内容变更或显式热重载才重新读取描述文件。

## Public APIs / Interfaces

- 扩展 `UiSurfaceRebuildReport` / debug stats，新增：
  - layout 访问节点数、几何变化节点数、跳过布局节点数。
  - render 命令复用数、重建数、damage rect 数。
  - 控件池 created / reused / recycled / discarded 计数。
- 在 shared UI interface 中增加 render cache stamp 或 command-level generation，让 `UiPaintElement.cache_generation` 能从生产路径填充。
- 增加自动布局判断接口，例如 `UiContainerKind::is_auto_layout_container()`；`Group` 按“内容驱动或自动编排容器”的语义处理。

## Test Plan

- Runtime layout tests：
  - 子控件大小变化时，自动布局父容器重新 measure/arrange，非自动布局父容器不刷新。
  - 父容器大小变化但子控件最终 frame 不变时，子控件不触发 render dirty。
  - 文本长度变化只传播到需要重新测量的祖先。
  - 子控件增删只刷新对应结构范围，不全树重建。
- Render tests：
  - drawable 控件 frame/style/text/state 全不变时，render command generation 复用，damage 为空。
  - 仅 visual state 变化时，只重建该节点绘制命令。
  - 仅 layout 变化时，只重绘旧/新 frame 覆盖区域。
- Pool tests：
  - 删除再新增同类型控件复用节点池对象。
  - 复用节点不携带旧焦点、输入、hover、临时 dirty 状态。
  - 普通 resize/property mutation 不触发描述文件 reload。
- Validation：
  - 先跑相关 crate 测试，再跑 `cargo test --workspace --locked --verbose`。
  - 同步更新 `docs/` 中 runtime UI、editor host session、render cache 相关文档，记录新缓存/增量布局规则和验证结果。

## Assumptions

- “Group 等自动编排控件”落到现有 Zircon 语义时，按 `ContentDriven` 或线性/网格/包裹/滚动等自动布局容器处理；自由定位和父驱动容器不向上刷新。
- 初次打开 UI、显式热重载、描述文件实际变更允许重新编译模板；运行期交互、尺寸变化、文本变化、属性变化不允许重新加载描述文件或全量重建。
- 优先改 runtime UI 核心，再接入 editor 预览和渲染路径，避免只修编辑器表面现象。
