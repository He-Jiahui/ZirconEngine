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

Open state: `待 Editor08 回传 catalog/query generation、owned bytes和keystroke counters，并由 EditorUI08 回传 visible-row clone/build evidence`。
