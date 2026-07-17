---
handoff_kind: fixed
status: fixed
created_at: 2026-07-17
summary_slug: screen-space-ui-text-font-id-report-mount-drift
origin_plan: docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
fixing_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
origin_child_dir: docs/plans/zircon_runtime/text/01
fixing_child_dir: docs/plans/zircon_runtime/runtime/15
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/text/font_id_report.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/render_ui_text_font_id_report.rs
tests:
  - cargo test -p zircon_runtime --lib runtime_15_screen_space_ui_text_font_id_report_is_child_owner --locked --jobs 1 -- --test-threads=1
  - cargo test -p zircon_runtime --lib text_font --locked --jobs 1 -- --test-threads=1
resolved_at: 2026-07-17
---


# Runtime15: screen-space UI text font-id report mount drift

## 产出记录与时间

| 时间 | 来源门禁 | 状态 | 失败证据 | 修复责任 |
| --- | --- | --- | --- | --- |
| 2026-07-17 | Text01 `text_font` 聚焦门 | `待修复（open）` | reservation `9b08c3232aae4265a13ff3e312f65fe5`、job `e774762f3bdf47ceb9d9636f9a332ebc`、run `43fbd23a72404a78b7ff6bb3e4fb04bb` 完成 14m19s 编译后执行 47 项：46 passed / 1 failed，唯一失败为 `runtime_15_screen_space_ui_text_font_id_report_is_child_owner`；exit 101。 | Runtime15 恢复 `font_id_report` child owner 的真实生产挂载并保持父/子文件预算与结构锚一致。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md`
- 来源执行切片：Text01 `FontDatabase` render-input equivalence failure return gate
- 修复责任计划：`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
- 交接原因：Text01 字体数据库 46 项行为均通过，唯一失败是 Runtime15 所属的 screen-space UI text child-owner 结构守卫与生产挂载缺失。

## 失败现象与复现证据

Windows 受管命令：

```text
cargo test -p zircon_runtime --lib text_font --locked --jobs 1 -- --test-threads=1
```

结果为 46 passed / 1 failed / 8136 filtered out。失败断言：

```text
screen-space UI text parent keeps native/SDF orchestration and child mount
missing required anchors: ["use self::font_id_report::{", "accumulate_text_font_id_report("]
```

当前 `font_id_report.rs` 定义 `accumulate_text_font_id_report`，但生产源码全仓只有定义、没有调用；`text.rs` 仅导入 `ScreenSpaceUiTextFontIdReport` 类型，因此不是可通过删除守卫解决的纯文本漂移。

## 最低共享层根因

screen-space UI text 报告拆到 folder-backed child 后，父 orchestrator 未保留实际 backend glyph face-id 汇总挂载，导致 child 中的累计函数成为 dead code，同时 Runtime15 的结构守卫准确报出 mount 缺失。该根因属于模块 owner/生产调用边界，不属于 Text01 `FontDatabase` 查询或 Editor gateway。

## 架构修复验收

- `text.rs` 或其确定的 orchestration child 实际调用 `accumulate_text_font_id_report`，报告继续读取 backend layout glyph 的真实 face id，不复制累计逻辑。
- `runtime_15_screen_space_ui_text_font_id_report_is_child_owner` 聚焦守卫通过，父/子生产文件继续低于 800 行。
- `cargo test -p zircon_runtime --lib text_font --locked --jobs 1 -- --test-threads=1` 达到 47/47。
- Text01 failure return gate恢复后，再上行复跑 Editor01 gateway 7 项。

## 禁止临时方案

- 不得删除/放宽结构守卫、给 dead function 加豁免或把累计逻辑复制回父文件。
- 不得恢复旧单文件布局、兼容 re-export、伪造 font id 或退化成 family-name 推断。
- 不得从 `text_font` filter 排除 Runtime15 守卫来制造绿色结果。

## 修复结果与回传

- 根因：font_id_report child existed, but the parent screen-space UI text orchestrator neither mounted nor called it; native_buffer also rebuilt a duplicate report from a drifted query and skipped unresolved primary faces.
- 架构修复：The parent now mounts one font_id_report child; native shaping returns the authoritative primary face from native_font_query; the child counts actual layout glyph face ids; the duplicate DTO, method, root re-export, and compatibility surfaces were hard-cut.
- 验证：Windows managed job c263b6875a644947a390abb6ba5b8203 / run dd1c6d7b32824d59a10b8696e4ef6a33 passed text_font 47/47 including runtime_15_screen_space_ui_text_font_id_report_is_child_owner; canonical archive sources 3/3, rustfmt, scoped diff-check, and independent review 0/0/0 passed.
- 回传：Runtime15 restored the real production mount and converged font-id reporting to a single shaping-query/actual-glyph owner without aliases or shims; Text01 may resume its upper gateway gate.
