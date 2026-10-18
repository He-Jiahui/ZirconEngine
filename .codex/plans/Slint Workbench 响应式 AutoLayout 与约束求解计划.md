# Slint Workbench 响应式 AutoLayout 与约束求解计划

## Summary
- 彻底停止当前 Slint 里基于 `left/right/bottom extent` 的手写锚点推导，改为由 Rust 端统一 AutoLayout 求解，再把结果帧回灌给 Slint。右侧 `Inspector` 和底部 `Console` 的错误位置，本质上就是当前几何链路分裂造成的。
- 继续以 `WorkbenchLayout` 作为唯一布局拓扑与预设资产模型，不引入第二套布局格式；新增的是约束默认值和布局覆盖值。
- 只保留 Slint 路径作为运行时真源，不再沿用旧 `iced` 的减法估算思路。viewport 尺寸改为直接使用求解后的文档内容区域，而不是 UI 反推。

## Public Interfaces And Types
- 新增轴向约束类型，供 pane、region、solver 共用：
  - `StretchMode { Fixed, Stretch }`
  - `AxisConstraint { min, max, preferred, priority, weight, stretch_mode }`
  - `ResolvedAxisConstraint`
- 约束语义固定：
  - `min == 0` 表示无最小值
  - `max == -1` 表示无最大值
  - `preferred` 为显式字段；若覆盖层未提供，则回退到描述符默认值，再回退到现有内建 preset 的 extent
- `ViewDescriptor` 增加 pane 默认约束，分别提供宽/高两个轴的默认值。
- `WorkbenchLayout` 增加响应式覆盖层：
  - `region_overrides`，键为 `ShellRegionId { Left, Document, Right, Bottom }`
  - `view_overrides`，键为 `ViewInstanceId`
  - 现有 `drawer.extent` 保留，并定义为 drawer 主轴上的持久化 `preferred`
- 新增共享求解结果模型：
  - `WorkbenchShellGeometry { window_min_width, window_min_height, region_frames, splitter_frames, viewport_content_frame }`
  - `ShellFrame { x, y, width, height }`
- Slint 壳体接口改为消费几何结果：
  - 不再让 `WorkbenchShell` 自己算 `document_zone_x`、`right_stack_x`、`bottom_panel_y`
  - 改为接收 host 下发的 frame、最小窗体尺寸、splitter 命中区

## Implementation Changes
- 新增共享 `autolayout` 求解模块，固定 shell 树为：
  - 纵向：`TopBar`、`HostBar`、`CenterBand`、`BottomBand`、`StatusBar`
  - `CenterBand` 横向：`LeftRegion`、`DocumentRegion`、`RightRegion`
- 约束合并顺序固定：
  - 内建 region 默认值
  - layout/preset 的 region override
  - descriptor 的 pane 默认值
  - instance 的 view override
  - `drawer.extent -> primary-axis preferred`
  - 最终再叠加 rail、tab header、border、splitter 等 chrome 开销
- stack 参与外层布局的规则固定为：
  - 只看活动 tab
  - 非活动 tab 不参与外层 min/max/preferred 求解
  - pane 空状态随 pane 走，不随槽位变化
- 求解算法固定：
  - 容器最小值按轴聚合：横向 `sum(min_width)` / `max(min_height)`，纵向反之
  - 初值先取各子项 `preferred`，再按 `min/max` clamp
  - 变大时只允许 `Stretch` 子项吸收剩余空间
  - 变大时按 `priority` 从高到低分组；同组内按 `weight` 分配；触顶 `max` 后在同组内继续重分配，再落到下一优先级
  - 变小时允许所有仍高于 `min` 的子项收缩
  - 变小时按 `priority` 从低到高分组；高优先级受保护；同组内按 `weight` 分摊压缩；触底 `min` 后继续重分配
  - `Fixed` 不吸收额外空间，但在空间不足时允许缩到自己的 `min`
- 默认约束固定：
  - `Document` 宽/高优先级 `100`，`Stretch`
  - 左/右/底工具区宽/高优先级 `50`，`Stretch`
  - 顶栏、宿主栏、状态栏是 `Fixed`
  - 现有内建 preset 的 `312 / 308 / 164` 等 extent 继续作为初始 `preferred`
- Slint 壳体重构为“渲染结果”而不是“推导布局”：
  - 删除当前以 `root.width`、`drawer_extent`、`document_zone_width` 连锁推导的位置公式作为真源
  - region、pane、splitter、drag overlay 全部绑定求解后的 frame
  - 顶层 `Window` 直接绑定 `min-width` / `min-height` 到求解后的总最小尺寸
  - 顶层组件导出真实 `shell_width_px` / `shell_height_px`，host 以此作为窗口实际尺寸输入
- Slint host 更新流固定：
  - 监听真实窗口尺寸变化，设脏 `WindowMetrics`
  - layout/preset 加载、tab 切换、drawer 展开折叠、view 打开关闭、splitter 拖拽、约束覆盖变化都设脏 `Layout`
  - 每 tick 或事件批次末统一 `recompute_if_dirty()`
  - 求解后先下发布局 frame，再把 `viewport_content_frame.size` 送入 `ViewportInput::Resized`
- splitter 与手感修正固定：
  - splitter 命中区直接来自求解后的边界，不再二次计算
  - 拖拽过程中只改内存中的 `preferred`，实时重算
  - 释放鼠标后再提交到 `drawer.extent` / override，避免跳变和抖动
- 兼容规则固定：
  - 用户把 `Project` 放右边、`Console` 放左边，内容和空状态完全不变，只换 host frame
  - 不再保留一条独立 `iced` 布局逻辑作为并行实现

## Test Plan
- 求解器单测：
  - `min=0`、`max=-1` 归一化正确
  - 横向/纵向容器的最小值聚合符合既定规则
  - 扩张时先看 `priority` 再看 `weight`
  - 压缩时高优先级受保护
  - `Fixed` 不增长但可在不足时压到 `min`
  - 命中 `min/max` 后能继续重分配，不出现负尺寸、重叠或空洞
- 集成测试：
  - 默认 `1440x900` 下，右侧 `Inspector` 贴右锚，底部 `Console` 贴底锚
  - 缩小窗口时会被总最小尺寸阻止，中央文档区不会被挤没
  - 加载 alternate preset 后，region 锚点变化但 pane 语义和空状态不变
  - `Scene/Game` 仍保持中心主导，因为 `Document` 优先级更高
- Host/UI 测试：
  - viewport 尺寸来自 solver，不再来自 UI 反测公式
  - splitter 命中区与显示边界一致，拖拽中途改变窗体大小也不漂移
  - preset 资产中的 override 能驱动不同 region frame，而不改 Slint DOM 结构
- 手工验收：
  - 左侧继续正确，右侧和底部在启动、缩放、拖拽后都落在正确位置
  - 不能把窗体缩到小于总最小值
  - 点击、拖拽、缩放手感稳定，没有跳边、滞后、错位

## Assumptions And Defaults
- `WorkbenchLayout` 继续是唯一持久化布局模型；响应式约束作为其补充字段持久化。
- `PreferredWidth` / `PreferredHeight` 必须新增；否则 `Fixed` 无法稳定求解。
- 活动 tab 是 stack 对外唯一参与者。
- 若 descriptor 和 preset 都未提供某项约束，则回退到内建 shell 默认值；若 `preferred` 也未提供，则回退到当前内建 preset 的 extent。
- 当前这轮只收束“布局求解、设脏、最小窗体、viewport 尺寸来源、splitter 手感”这条链路；不在本轮引入新的视觉风格决策。
