---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: command-palette-paged-keyboard-navigation
origin_plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
fixing_plan: docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
origin_child_dir: docs/plans/zircon_editor/editor/08
fixing_child_dir: docs/plans/zircon_editor/editor_ui/06
related_code:
  - zircon_runtime/src/ui/component/state_reducer/command_palette.rs
  - zircon_runtime/src/ui/surface/input/keyboard_action.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/keyboard.rs
  - zircon_editor/src/ui/retained_host/app/command_palette_actions.rs
  - zircon_editor/assets/ui/editor/windows/workbench_window.zui
tests:
  - cargo test -p zircon_runtime --lib --locked command_palette
  - cargo test -p zircon_editor --lib --locked command_palette
---

# EditorUI06：CommandPalette 缺少分页窗口键盘导航契约

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md`
- 来源执行切片：M3.1 command palette generation-owned catalog 与 typed query window
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/06-component-library-mui.md`
- 交接原因：命令目录、模糊匹配、when 与 MRU 属于 Editor08；`CommandPalette` 的键盘状态机、焦点与 host 语义事件契约属于 EditorUI06/runtime component reducer。

## 失败现象与复现证据

Editor08 已把 Workbench 查询收敛为 8 个可见行加 4 个 overscan，并投影 `catalog_generation`、`total_match_count` 与 `window_offset`。打开面板和 query change 当前都请求 `offset = 0`，这是查询重置时的正确初始页。

runtime `command_palette.rs::apply_keyboard_action` 会在组件内部消费 `Next/Previous/First/Last`；`navigate_filtered_commands` 只读取当前 `filtered_commands`，并在当前窗口首尾之间循环。到达第 12 行后继续 Next 不会向 host 请求下一窗口，Previous/First/Last 也只作用于局部页。`LargeIncrement/LargeDecrement` 没有 CommandPalette 分页语义出口。由于事件已经在 runtime reducer 内消费，editor host 无法可靠观察边界并推进 `window_offset`。

因此当前 1,000 条命令目录虽然不会整表 clone/paint，但键盘只能访问首个查询窗口；该结果不能宣称命令面板 MVP 深页导航完成。

## 最低共享层根因

缺口是 runtime 组件与外部数据源之间的虚拟窗口协议，而不是查询实现。组件 reducer 目前假定 `filtered_commands` 是完整集合；Editor08 已将它合法改为可见窗口后，组件没有“请求前一页/后一页/首尾目标窗口”的语义事件，也没有全局 focused offset 与局部 focused index 的守恒规则。

## 架构修复验收

- `CommandPalette` 继续只持有当前窗口，不恢复完整 command catalog 到 `UiValue`。
- reducer 在 Next/Previous 越过页边界以及 PageUp/PageDown、Home/End 需要跨页时，发出类型化 window request；payload 至少包含查询代际、当前 offset、目标 offset/方向与焦点意图。
- host 用 Editor08 的 `command_palette_query_window(context, query, offset, limit)` 响应请求，并原子回写 commands、filtered ids、window offset、total count、selected id 与局部 focused index。
- 查询文本或 catalog generation 改变时拒绝陈旧 window response；新查询重置到 offset 0。
- disabled command 跳过、selection/focus 可见性、Enter commit 与 pointer commit 在换页前后保持一致；不得出现首尾循环回当前局部页的假导航。
- 回归覆盖 1、12、13、1,000 条目录，至少包含 Next/Previous 跨页、PageUp/PageDown、Home/End、陈旧 generation 丢弃、disabled 边界与换页后 commit。

## 禁止临时方案

- 禁止为了键盘导航把完整命令目录重新序列化到 `.zui`/`UiValue` 或每次 paint 全量构造行。
- 禁止 editor host 通过监听物理键复制第二套 CommandPalette reducer；runtime 组件必须输出中性的语义请求。
- 禁止以当前 12 行首尾循环冒充完整结果导航，禁止用焦点丢失触发隐式重新查询。

## 修复结果与回传

Resolving failure。runtime reducer 已输出 current/target offset、focus 与 catalog generation 的窗口请求状态；retained native keyboard 将 Arrow/Page/Home/End 区分为行内移动与跨窗请求，末页不再局部循环。Editor08 host 会校验当前 offset、复用现有 generation-owned catalog 查询目标 12 行窗口，并拒绝 generation 变化的响应；查询变更仍重置到 offset 0。Coordinator01 full compile-input immutable snapshot failure 仍阻止受管 Cargo，disabled 边界、跨页 commit、像素与产品交互证据尚未完成，因此不改名为 `fixed-*`。

## 产出记录与时间

| 日期 | 事项 | 状态 | 证据与后续 |
| --- | --- | --- | --- |
| 2026-07-18 | CommandPalette paged keyboard contract failure handoff | open / 已交接 EditorUI06 | 静态确认 runtime reducer 只导航当前 `filtered_commands` 并消费 Next/Previous/First/Last，Editor08 host 只在 open/query reset 请求 offset 0；需新增类型化 window request、代际校验与 1/12/13/1,000 条深页回归，禁止恢复整表 UI 投影。 |
| 2026-07-18 | CommandPalette typed paged keyboard source implementation | resolving_failure / 源码完成，待受管验收 | reducer 与 retained native path 已实现有界行内导航、Next/Previous 跨窗、PageUp/PageDown、Home/End、current/target offset 与 generation 守恒；Editor08 `WindowRequested` route 原子重查已有 catalog 并拒绝 stale offset/generation。Python contract 3/3、Workbench ZUI TOML 解析、focused rustfmt 与 scoped diff check 通过；1/12/13/1,000 Rust 回归源码已落地但未执行，Cargo、disabled/commit 产品门与独立 review 仍待 Coordinator01 屏障解除。 |
