---
handoff_kind: fixed
status: fixed
created_at: 2026-07-14
summary_slug: variable-shaping-visibility-compilation
origin_plan: docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
fixing_plan: docs/plans/zircon_runtime/text/02-shaping-unicode-and-bidi.md
origin_child_dir: docs/plans/zircon_editor/editor/07
fixing_child_dir: docs/plans/zircon_runtime/text/02
related_code:
  - zircon_runtime/src/text/shaping/horizontal/mod.rs
  - zircon_runtime/src/text/shaping/horizontal/direct.rs
  - zircon_runtime/src/text/shaping/cosmic.rs
tests:
  - cargo test -p zircon_editor --lib --locked tests::editor_event::runtime::when_evaluation::typed_document_focus_tracks_floating_activation_and_focused_close -- --exact --test-threads=1 --nocapture
resolved_at: 2026-07-14
---


# Text 02：variable shaping helper 可见性阻断共享 Runtime 编译

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|
| `FIXED / 已回传来源计划` | 2026-07-14 | 原始 Text02 E0364/E0603 已通过 folder-backed horizontal shaping owner 硬切修复；managed Runtime `text_horizontal_` 5/5，并由 Windows coordinator job `15ed7b7df026474486615df05a7abc37` 完成来源 Editor07 exact 1/1（3172 filtered out）。本 lifecycle 已验证关闭。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md`
- 来源执行切片：focused document kind 与 animation indexed fixture 的 current-source Windows exact gate
- 修复责任计划：`docs/plans/zircon_runtime/text/02-shaping-unicode-and-bidi.md`
- 交接原因：最低失败位于 Text02 所有的 horizontal variable shaping owner/consumer 可见性；Editor07 尚未开始编译，不能在 Editor 侧绕过 Runtime 文本整形边界。

## 失败现象与复现证据

- `zircon_runtime/src/text/shaping/variable/mod.rs:7`：E0364，private `apply_horizontal_variable_shaping` 不能 `pub(crate) use`。
- `zircon_runtime/src/text/shaping/cosmic.rs:20`：E0603，consumer 无法导入 private helper。
- 同轮另有 Plugins08 reflection 与 Runtime04 reference resolver 错误，均已由各自 failure artifact 拥有，本交接只负责 Text02 两项。

## 最低共享层根因

旧 folder-backed variable shaping 切分后，helper 可见性低于稳定 shaping owner 的 re-export/consumer 边界。当前架构已把该职责硬切到 `shaping::horizontal::{backend,direct}`；`direct` 仅在 `crate::text::shaping` 内消费 backend 输出并投影 canonical shaped run，不是 cosmic consumer 的调用特判。旧 `projection` 名称仅属于这次已关闭失败的历史上下文。

## 架构修复验收

- `apply_horizontal_variable_shaping` 仅向 `shaping` 所需 owner 可见，crate 外保持不可见。
- Text02 variable shaping 聚焦测试与 Runtime lib 编译不再出现 E0364/E0603。
- 向上重跑 Editor07 current-source exact，确认编译继续推进到 Editor 代码。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- Do not weaken tests or plan acceptance criteria to hide the failure.
- 禁止改成无边界 `pub`、复制 helper 到 `cosmic.rs` 或关闭变量字体轴路径。

## 修复结果与回传

- 根因：Horizontal variable-font secondary shaping was implemented in a private flat helper and re-exported across the text shaping boundary, so the Editor upward build hit Rust E0364/E0603 visibility errors; the Editor paint fixture also lacked the new ShapedGlyph font_instance_id field.
- 架构修复：Hard-cut the helper into the folder-backed text/shaping/horizontal backend and direct modules, keep backend shaping scoped to the shaping subsystem, and update the Editor fixture to construct the canonical ShapedGlyph contract with font_instance_id.
- 验证：Windows coordinator job d4821ebfeef1445eb743515c9439948c passed text_horizontal_ 5/5; Windows coordinator job 15ed7b7df026474486615df05a7abc37 passed the originating Editor07 exact test 1/1 with 3172 filtered out. No E0364/E0603 visibility error remained.
- 回传：Variable-font shaping now compiles through the real Editor07 consumer and the exact floating-focus test passes; return the verified fix to Editor07 and close this Text02 failure lifecycle.
