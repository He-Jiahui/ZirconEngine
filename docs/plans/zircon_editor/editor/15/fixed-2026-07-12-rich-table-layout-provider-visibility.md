---
handoff_kind: fixed
status: fixed
created_at: 2026-07-12
summary_slug: rich-table-layout-provider-visibility
origin_plan: docs/plans/zircon_editor/editor/15-build-export-and-publishing.md
fixing_plan: docs/plans/zircon_runtime/text/07-rich-text-html-bbcode.md
origin_child_dir: docs/plans/zircon_editor/editor/15
fixing_child_dir: docs/plans/zircon_runtime/text/07
related_code:
  - zircon_runtime/src/ui/text/layout_engine/rich_table/layout.rs
tests:
  - cargo test -p zircon_editor core::export::tests --lib --offline -- --nocapture
resolved_at: 2026-07-12
---


# Text 07：Rich table provider layout visibility 编译失败

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/15-build-export-and-publishing.md`
- 来源执行切片：Editor 15 M1 core export focused gate
- 修复责任计划：`docs/plans/zircon_runtime/text/07-rich-text-html-bbcode.md`
- 交接原因：E0364/E0603 位于 Runtime Text 07 rich-table layout 子模块的 provider 可见性边界。

## 失败现象与复现证据

Editor 15 M1 focused test 曾被 Runtime Text 07 阻断：rich-table layout 子模块函数的可见性不足，
`layout_engine.rs` 无法调用重导出的 provider layout function，产生 E0364/E0603。

## 最低共享层根因

`layout_rich_tables_with_provider` 的定义可见性小于父级 `layout_engine` 的合法调用边界，导致内部 owner
重导出不可用。无需扩大为 crate/public API。

## 架构修复验收

- 函数只开放到 `layout_engine` 所需的最窄模块边界。
- Editor 15 `core::export::tests` 原始复现命令完成编译并全绿。
- 不新增兼容 re-export 或公共 API。

## 禁止临时方案

- 禁止把内部 helper 扩为 `pub`、增加旧路径 alias 或绕过 Text 07 owner。
- 禁止跳过 Editor 15 focused tests。

## 产出记录与时间

| 时间 | 状态 | 产出 |
| --- | --- | --- |
| 2026-07-12 21:24 +08:00 | 未通过，已转交 | Runtime rich-table provider layout 可见性产生 E0364/E0603；失败归档 Text 07。 |
| 2026-07-12 21:42 +08:00 | 已修复，待回传 | Text 07 owner 使用受限 `pub(in super::super)` 修复边界；Editor 15 focused tests 13/13 通过。 |

## 修复结果与回传

- 根因：layout_rich_tables_with_provider 的定义可见性小于父级 layout_engine 的合法调用边界。
- 架构修复：Text07 将 helper 收敛为 pub(in super::super)，只开放到 layout_engine owner，不扩大 crate/public API。
- 验证：cargo test -p zircon_editor core::export::tests --lib --offline -- --nocapture：13/13 通过，E0364/E0603 未重现。
- 回传：Editor15 M1 core export focused gate 可继续；canonical artifact 返回 origin/15。
