---
handoff_kind: failure
status: open
created_at: 2026-07-14
summary_slug: variable-shaping-visibility-compilation
origin_plan: docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md
fixing_plan: docs/plans/zircon_runtime/text/02-shaping-unicode-and-bidi.md
origin_child_dir: docs/plans/zircon_editor/editor/07
fixing_child_dir: docs/plans/zircon_runtime/text/02
related_code:
  - zircon_runtime/src/graphics/text/shaping/horizontal/mod.rs
  - zircon_runtime/src/graphics/text/shaping/horizontal/projection.rs
  - zircon_runtime/src/graphics/text/shaping/cosmic.rs
tests:
  - cargo test -p zircon_editor --lib --locked tests::editor_event::runtime::when_evaluation::typed_document_focus_tracks_floating_activation_and_focused_close -- --exact --test-threads=1 --nocapture
---

# Text 02：variable shaping helper 可见性阻断共享 Runtime 编译

## 产出记录与时间

| 状态 | 记录日期 | 完成项目与当前门禁 |
|---|---|---|
| `OPEN / 修复已落地，来源 exact 待复验` | 2026-07-14 | Editor07 current-source exact 在进入 `zircon_editor` 前编译 Runtime 失败；原始 Text02 分组为 2 项：旧 `variable::axis::apply_horizontal_variable_shaping` 不能从 `variable/mod.rs` 重新导出（E0364），`shaping/cosmic.rs` 不能消费该 private import（E0603）。当前已硬切为 `horizontal::projection::apply_horizontal_backend_shaping` 的最窄 shaping 可见性；managed Runtime compile 已不再报告 E0364/E0603，但来源 Editor07 exact 尚待活动的非文本编译漂移稳定后复验。原始日志 `.codex/tmp/editor07-focused-document-current-exact-20260714.log`。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/07-domain-editors-and-graph-foundation.md`
- 来源执行切片：focused document kind 与 animation indexed fixture 的 current-source Windows exact gate
- 修复责任计划：`docs/plans/zircon_runtime/text/02-shaping-unicode-and-bidi.md`
- 交接原因：最低失败位于 Text02 所有的 horizontal variable shaping owner/consumer 可见性；Editor07 尚未开始编译，不能在 Editor 侧绕过 Runtime 文本整形边界。

## 失败现象与复现证据

- `zircon_runtime/src/graphics/text/shaping/variable/mod.rs:7`：E0364，private `apply_horizontal_variable_shaping` 不能 `pub(crate) use`。
- `zircon_runtime/src/graphics/text/shaping/cosmic.rs:20`：E0603，consumer 无法导入 private helper。
- 同轮另有 Plugins08 reflection 与 Runtime04 reference resolver 错误，均已由各自 failure artifact 拥有，本交接只负责 Text02 两项。

## 最低共享层根因

旧 folder-backed variable shaping 切分后，helper 可见性低于稳定 shaping owner 的 re-export/consumer 边界。当前架构已把该职责硬切到 `shaping::horizontal::{backend,projection}`，`projection` helper 仅以 `pub(in crate::graphics::text::shaping)` 暴露给 cosmic owner；不是 cosmic consumer 的调用特判。

## 架构修复验收

- `apply_horizontal_variable_shaping` 仅向 `shaping` 所需 owner 可见，crate 外保持不可见。
- Text02 variable shaping 聚焦测试与 Runtime lib 编译不再出现 E0364/E0603。
- 向上重跑 Editor07 current-source exact，确认编译继续推进到 Editor 代码。

## 禁止临时方案

- Do not add aliases, compatibility shims, silent fallback, duplicated truth, test-only bypasses, or call-site exceptions.
- Do not weaken tests or plan acceptance criteria to hide the failure.
- 禁止改成无边界 `pub`、复制 helper 到 `cosmic.rs` 或关闭变量字体轴路径。

## 修复结果与回传

Implementation state: `Text02 修复与 focused exact 已通过，来源 exact 待复验`。managed Windows job `d4821ebfeef1445eb743515c9439948c` 在当前共享源码上完成 37m04s 首链，`text_horizontal_` 实际运行 5 个测试并以 5 passed / 0 failed 结束（26.98s），原 E0364/E0603 未复现；ephemeral target 已由 coordinator 删除。仍不在 Editor07 original exact 实际运行前把 handoff 迁为 fixed。
