---
handoff_kind: failure
status: open
created_at: 2026-07-17
summary_slug: command-palette-catalog-clone-and-full-row-paint
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_editor/editor/08
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/commands/registry.rs
  - zircon_editor/src/ui/retained_host/app/command_palette_actions.rs
  - zircon_editor/src/ui/retained_host/host_contract/paint_template_nodes/template_command_palette/commands.rs
---

# Command palette catalog clone and full-row paint

> 深页键盘导航不是本记录中查询目录的剩余算法补丁；runtime 组件缺少分页窗口语义出口，已交接到 [EditorUI06：CommandPalette 分页键盘导航契约](../../editor_ui/06/failure-2026-07-18-command-palette-paged-keyboard-navigation.md)。该交接禁止恢复完整目录 UI 投影。

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：`template_command_palette*` 39/39 个 Rust 文件及已审查 command registry/open-state 入口聚焦回查
- 修复责任计划：`docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md`
- 交接原因：command catalog generation、enabled evaluation、typed search index/result 与 entry ownership 属于 Editor08；EditorUI08 只负责 visible-row consumption/paint。

## 失败现象与复现证据

Palette open先收集完整entry Vec，再完整转换为commands UiValue，并再次clone全部id生成filtered_commands。Painter随后对全部structured rows执行row_data，leaf才clip。大catalog和query burst因此可能同时放大catalog owned bytes与offscreen paint work。

## 最低共享层根因

Command registry没有发布可共享的immutable catalog generation和typed query result；UI边界以多个完整owned DTO表示同一catalog，也没有可直接消费的visible/top-K结果预算。

## 架构修复验收

- Stable catalog重复open不深clone完整entries；catalog generation只在descriptor/when依赖变化时更新。
- Query使用typed index/result并有明确top-K/visible预算；1,000 keystrokes报告visited/comparisons/allocations与input p95。
- EditorUI08只clone/visit visible+overscan handles；offscreen row_data/text/build为零。
- Enabled/when、selection/focus/commit、search/empty、row detail、ordering和pixels等价。

## 禁止临时方案

- 不得在painter建立第二份command catalog或不受registry generation约束的cache。
- 不得保留commands与filtered ids两份完整owned catalog再仅优化其中一份。
- 不得以截断结果静默改变keyboard selection/commit语义；top-K/virtualization必须保留完整可检索集合。

## 修复结果与回传

Resolving failure（2026-07-18）：Editor08 已交付 generation-owned shared catalog、typed paged query、
权威 `QueryChanged` route/binding/host intercept 与 bounded bridge update；旧全量 entry/value API
扫描为 0，open/query edit 均收敛为 8 visible + 4 overscan，1,000-query 测试锁定 retained
handles 不超过窗口预算。EditorUI08 painter 也已从全行 `row_data` 改为 clip-derived visible +
1-row overscan borrowed access。EditorUI06/Editor08 现已接通 typed 深页 keyboard window request、
stale offset/generation 拒绝和 bounded host requery。当前仍保持 `resolving_failure`：受管 current-source
Cargo、disabled/commit 产品门、像素等价、1,000 输入 p95 与独立 review 尚未完成；Coordinator01 immutable
full-input snapshot failure 仍阻断有效验收。不得把本次源码阶段记录改名为 `fixed-*` 或提前
回传来源计划。

## 产出记录与时间

| 日期 | 事项 | 状态 | 证据与后续 |
| --- | --- | --- | --- |
| 2026-07-18 | generation-owned query、bounded paint 与深页 keyboard source closure | resolving_failure / 源码完成，待受管验收 | catalog 查询、visible+overscan paint 与 typed keyboard window request 已形成闭环，host 不恢复全量 UI catalog，并对 stale offset/generation 无副作用拒绝。Python contract 3/3、Workbench ZUI TOML、focused rustfmt、scoped diff check 通过；Cargo、1,000 输入 p95、像素/产品交互与独立 review 仍开放。 |

2026-07-22逐文件复核：generation-owned catalog和bounded window继续成立，因此旧“open时全量clone”根因不恢复；remaining query每次仍两遍扫描全部search documents，enabled行每遍按String id回查registry BTreeMap，window UiValue再clone字段。failure保持`resolving_failure`，PERF-MVP-211验收补descriptor slot/enablement index、增量候选或等价top-K+count证据，不能只以retained handles≤window关闭。
