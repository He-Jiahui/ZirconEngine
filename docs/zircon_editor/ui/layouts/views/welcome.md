---
related_code:
  - zircon_editor/assets/ui/editor/welcome.zui
  - zircon_editor/src/ui/layouts/views/welcome.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/content.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/welcome/main_column.rs
  - zircon_editor/src/tests/ui/welcome/bootstrap_assets.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/welcome_visual_screenshot.rs
implementation_files:
  - zircon_editor/assets/ui/editor/welcome.zui
  - zircon_editor/src/ui/layouts/views/welcome.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/docks/pane/content.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_workbench_renderer/welcome/main_column.rs
plan_sources:
  - docs/plans/zircon_editor/editor_layout/15-component-standardization-from-primitives.md
  - docs/plans/zircon_editor/editor_layout/15d-composite-density-and-alignment.md
  - docs/plans/zircon_editor/editor_layout/15e-domain-breakpoint-adaptation.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/ui-and-layout/editor-workbench-designs/welcome-workbench.png
  - docs/ui-and-layout/editor-workbench-designs/welcome-new-project-focus.png
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Styling/StarshipCoreStyle.cpp
tests:
  - zircon_editor/src/tests/ui/welcome/bootstrap_assets.rs
  - zircon_editor/src/tests/host/retained_menu_pointer/welcome_visual_screenshot.rs
  - docs/tests/editor/editor-window-m3-welcome-mvp-actions-640x520.png
---

# Welcome view

`welcome.zui` 是编辑器欢迎页的布局真值，`welcome_pane_nodes` 只负责把该资产投影为 retained mount nodes。项目创建或打开的业务动作必须在首屏可达，装饰性 Hero、状态、路径预览和开发启动选择器不得把主动作挤出短窗口。

## MVP 排序契约

主列按以下依赖顺序排列：

1. New Project 标题；
2. 项目名 TextField；
3. 项目位置 TextField；
4. 校验反馈；
5. Open / Create Project 动作组；
6. Hero、引擎状态、路径预览和开发启动入口。

该顺序由同一棵 `VerticalBox` 自适应布局树计算，不使用窗口 painter 中的绝对坐标补丁。640×420 与 900×520 回归测试要求动作行底边始终位于主列底边以内，并要求所有可选内容从动作行之后开始。

## Starship 原子密度

- 项目输入框的可视高度和布局行高统一为 44、边框为 1、圆角为 4。
- Project name / Location 使用独立 `Label` 原子，9px muted 文本由共享 Runtime Text painter 绘制；标签与输入框之间使用 4 的紧邻间距。
- Open / Create Project 按钮高度为 32、圆角为 4，替代旧的 999 pill 圆角。
- 输入、校验、动作和可选内容使用 8/12 的间距节奏；主内容两侧使用 28 的 slot padding。
- 主内容宽度上限为 680。窄档由父级可用宽度收缩，宽档不让表单无限扩张。
- Recent 列使用 220/320/320 的 min/preferred/max 弹性约束，项目主列声明 280 最小宽；640 窗口扣除 Workbench chrome 后约 560 的真实 pane 仍优先保证主业务栏，再让历史列表从 320 收缩，900/1260 档恢复 Recent 的 320 上限。
- Recent 顶部节奏为 18 top inset / 46 双行 header / 8 list gap；资产投影与 native fallback 使用同一语义，fallback list 高度由 panel bottom 相对计算，不保留旧 26/54/14 镜像值。
- Recent header 使用共享 body/small 字号与派生行高，两行内容作为一个相对块在 46 高 header 内垂直居中；水平 inset 读取 `gap_l`，不保留 `+18/+6/+30` 文本坐标。fallback list 的水平 padding/list gap 分别读取 `gap_l/gap_m`，列表外表面用共享 control radius 与 border width。
- Recent 空状态同样使用 Runtime Text body/small 字号与派生行高；两行文案按实测宽度作为一个内容块在 list frame 内相对水平/垂直居中，不保留 `+14/+16/+40` 空状态坐标。
- Recent 行采用 Unreal 列表式平面层级：普通行只绘制 surface 与共享 `border_width` 底部分隔，不再为每行叠加四边“卡片”轮廓；无效行才增加 warning 语义轮廓。Open/Remove 操作框由共享行几何 owner 投影，表面填充与边框共同读取宿主 `radius_control`/`border_width` 设计令牌。按钮标签由 Runtime Text 实测宽度，并按按钮框相对居中；Remove 使用紧凑 `×` 字形，不保留 `+10/+8/+6` 固定偏移、ASCII `X` 或 native 局部字体常量。
- Recent 双行文本读取宿主 body 字号和派生 line-height。第一行状态槽按 Runtime Text 实测宽度与 `text_clip_guard` 分配，并从项目名槽中扣除共享 `gap_m`；长项目名因此省略在状态之前，不再与 `Missing/Today` 叠字。路径行保留完整文本列宽，不复用旧固定 76px 状态预算。

这些数值对应 Unreal Starship 的 4px input/button radius 与紧凑工具型控件风格。字体字族、shaping、fallback 和 raster 仍由 Runtime Text 统一拥有；Welcome 不得声明局部 concrete font family 或复制文本栅格逻辑。

## 布局、绘制与命中一致性

`WelcomeActionsRow`、两个按钮和两个 TextField 都来自模板投影。旧路径先绘制共享模板节点，随后 native Welcome 背景覆盖整块 body，最后再用方形 `draw_rect` 重画输入框和按钮；它同时制造了样式双 owner、遮蔽圆角以及绘制/命中几何漂移。

当前路径只对 Welcome 采用 `native foundation → shared template controls → debug overlay` 层序；Hierarchy、Assets、AssetBrowser 等 native hover/scrollbar overlay 仍保持模板之后。native main column 不再挂载 `actions.rs` 或 `form/field.rs`，这两个重复 painter 及其四个 action-only 色常量已硬删除；按钮 surface、圆角、文本、disabled 状态和输入焦点都回到通用组件 owner，绘制框与 hit node 因而共享同一模板几何。

动作行在真实 640 窗口档主栏满足 280 最小宽后，经过 28 双侧 padding 至少可用 224；两个固定动作和两个 box gap 总计 218，剩余空间由前置 fill 吸收。

## 验证状态

- 资产 TOML 解析、布局顺序、Runtime Text 字段标签、三档列宽约束、旧 10/999 圆角与 56/64 高度零残留、native standard-control overlay 零命中、Rust 格式和 scoped diff 检查已通过。
- focused Cargo、交互截图和最终像素审查仍为 `validation_pending`。Runtime Text01 已由 commit `a7607a30` 闭合 `sys-locale` 根/插件双 lock 漂移；Layout15 的 FIFO 作业 `a541cec23a0a44b1be906b635e243053` 已实际启动并在进入 editor 编译前暴露 Runtime10 reactive-wake V3 硬切迁移中的 2 个 E0432（runtime export/FFI 仍引用 interface 已删除的 V2/ConfigV1）。Layout15 不添加兼容别名或局部绕过，等待 Runtime10 原子迁移后重跑当前源码门。
- 12:36 已编译 editor test binary 的 Recent 共享几何精确用例为 2/2 通过；同一二进制的两条截图守卫都在写文件前被其旧 `CommandPalette/QueryChanged` 绑定缺口拒绝，因此没有把旧宿主状态输出为视觉证据。当前源码已包含模板事件与 builtin binding，须由 Runtime10 恢复上行编译后生成新二进制再拍摄。
- Runtime10 V3 上行随后已恢复；受管 job `29b7d520398245e0a831c066733e3d6f` 成功编译 Runtime 并进入 editor test crate，但被 EditorUI06 paged keyboard、Editor05/Render04 viewport picking 与 Performance render-framework capture 的 10 个 current-source 迁移错误阻断。Layout15 文件不在诊断集合，作业以 exit 101、live PIDs 0 自然释放；当前源码截图与新 Recent surface 录制守卫仍不得计为通过。
- 验收截图只允许写入 `docs/tests/editor/editor-window-m3-welcome-mvp-actions-640x520.png`，并必须扫描 repository、D/E/F target roots，确认不存在同名或 Welcome 验证截图。
- 640×520 截图使用独立 `welcome_visual_screenshot.rs` ignored test；它复用真实 Welcome fixture 和窗口 snapshot helper，并在写入后断言父目录正是仓库 `docs/tests/editor`，不继续扩张 2250 行的旧截图总集文件。
