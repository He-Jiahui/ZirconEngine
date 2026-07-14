# Text 02 SH-M2 variable shaping visibility acceptance

Plan: docs/plans/zircon_runtime/text/02-shaping-unicode-and-bidi.md
Milestone: M2
Status: accepted
Files: ["docs/plans/zircon_runtime/text/02/2026-07-09-shaping-unicode-and-bidi-output-records.md", "docs/plans/zircon_runtime/text/02/2026-07-14-sh-m2-variable-shaping-visibility-acceptance.md", "docs/plans/zircon_runtime/text/02/failure-2026-07-14-variable-shaping-visibility-compilation.md"]

> Owner：[`../02-shaping-unicode-and-bidi.md`](../02-shaping-unicode-and-bidi.md) · 日期：2026-07-14 · Session：`runtime-text-sh-m2-records-commit-20260714`

## Scope delivered

| 里程碑 | 切片 | 状态 | 完成项目 |
|---|---|---|---|
| M2 | SH-M2 variable shaping owner visibility hard cut | 完成 | 旧 private flat helper 已硬切到 folder-backed `graphics/text/shaping/horizontal/{backend,projection}`；`apply_horizontal_backend_shaping` 只向 shaping 子系统开放。Editor paint fixture 使用含 `font_instance_id` 的规范 `ShapedGlyph`，未增加 public shim、重复 helper 或调用点绕行。 |
| M2 | SH-M2 failure lifecycle return | 完成 | `variable-shaping-visibility-compilation` 已从 Text02 open handoff 原子回传为 Editor07 fixed artifact，并同步更新来源计划、修复计划和 Text02 产出归档。Text02 整体仍为 `in_progress`，本记录只关闭该 SH-M2 failure 切片。 |

## Fresh testing evidence

| 里程碑 | 切片 | 状态 | 验证证据 |
|---|---|---|---|
| M2 | SH-M2-T lower horizontal shaping testing | 通过 | Windows managed job `d4821ebfeef1445eb743515c9439948c` 运行 `text_horizontal_`，5 passed / 0 failed。 |
| M2 | SH-M2-T originating Editor07 upward testing | 通过 | Windows managed job `15ed7b7df026474486615df05a7abc37` 运行来源 exact，1 passed / 0 failed / 3172 filtered out；E0364/E0603 未复现。 |
| M2 | SH-M2-T failure artifact validation | 通过 | `validate_plan_failure_handoffs.py` 校验 91 个 artifact，0 errors；scoped `git diff --check` 通过。failure graph 中仍有三组外部计划 cycle 诊断，与本 Text02 lifecycle 无关。 |

## Review

- 架构边界：helper 保持在 horizontal shaping owner 内，crate 外不可见；未恢复旧 `variable` flat owner。
- 结构优先项：本切片没有新增超大文件、根级兼容导出或 production `dead_code`；符合 `engine-code-structure-convention.md` 与 2026-06 review findings 的硬切和模块边界要求。
- 验收边界：本记录不宣称 Text02 整体完成，也不代替后续 mixed-script backend segmentation、完整 BIDI/竖排和产品帧门禁。
