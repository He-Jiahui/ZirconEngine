---
handoff_kind: fixed
status: fixed
created_at: 2026-07-17
summary_slug: font-database-render-input-equivalence-visibility
origin_plan: docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md
fixing_plan: docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md
origin_child_dir: docs/plans/zircon_editor/editor/01
fixing_child_dir: docs/plans/zircon_runtime/text/01
related_code:
  - zircon_runtime/src/text/font/shared.rs
  - zircon_runtime/src/text/font/database/equivalence.rs
tests:
  - cargo test -p zircon_editor --lib gateway::in_process --locked --jobs 1 -- --test-threads=1
  - cargo test -p zircon_runtime --lib text_font --locked --jobs 1 -- --test-threads=1
resolved_at: 2026-07-17
---


# Runtime Text01: FontDatabase render-input equivalence visibility drift

## 产出记录与时间

| 时间 | 来源门禁 | 状态 | 失败证据 | 修复责任 |
| --- | --- | --- | --- | --- |
| 2026-07-17 | Editor01 M2.1 `InProcessGateway` current-source GREEN | `待修复（open）` | reservation `7b441e22b36342e1b0632b6dd9aaa38c`、job `a93d2db3edd9446b95a102fc3da07efc`、run `9ba1d618f83b41ffbb900e3a8ade7d71` 在编译 `zircon_runtime` 时以 E0624 / exit 101 终止；gateway 的 7 项测试尚未执行。 | Runtime Text01 修复 `FontDatabase` 模块内等价比较的最小合法可见面，再复跑 Text01 聚焦门与来源门禁。 |
| 2026-07-17 | Text01 修复回传复验 | `fixed-returned` | `has_same_render_inputs` 已收束为 `pub(in crate::text::font)`，独立 current-source review 为 0/0/0。Text01 受管 job `e774762f3bdf47ceb9d9636f9a332ebc`、run `43fbd23a72404a78b7ff6bb3e4fb04bb` 实际执行 47 项，字体行为 46 项全绿，唯一失败为 Runtime15 的 `runtime_15_screen_space_ui_text_font_id_report_is_child_owner`；Editor01 上行 job `37b0965d5e7647bb8952c3adb523145d`、run `6b173cb849884a49b827961fdfcb6667` 已通过 gateway 7/7。 | 本 visibility lifecycle 已返回 fixed；Runtime15 的 `font_id_report` 生产挂载继续由独立 failure 跟踪，修复后再把 Text01 聚焦门提升为 47/47。 |

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/01-editor-kernel-and-runtime-interaction.md`
- 来源执行切片：M2.1 `InProcessGateway` 借用式访问基础 current-source GREEN
- 修复责任计划：`docs/plans/zircon_runtime/text/01-font-resource-faces-and-database.md`
- 交接原因：错误发生在 `FontDatabase` 的模块拆分可见性合同，低于 Editor gateway，且 Text01 拥有字体数据库与 shared database 更新策略。

## 失败现象与复现证据

Windows 受管命令：

```text
cargo test -p zircon_editor --lib gateway::in_process --locked --jobs 1 -- --test-threads=1
```

编译在执行任何 gateway 测试前失败：

```text
error[E0624]: method `has_same_render_inputs` is private
  --> zircon_runtime/src/text/font/shared.rs:42:20
  ::: zircon_runtime/src/text/font/database/equivalence.rs:5:5
      pub(super) fn has_same_render_inputs(...)
```

受管 run `9ba1d618f83b41ffbb900e3a8ade7d71` exit 101；该结果不声明 Editor01 GREEN，也不归因于 gateway 行为。

## 最低共享层根因

`FontDatabase::has_same_render_inputs` 被抽到 `text/font/database/equivalence.rs` 后保留 `pub(super)`。该可见性只覆盖 `database` 父模块，无法被同属 `text::font` owner 的 sibling `shared.rs` 调用；调用方与方法的 owner 边界在模块拆分后未同步。

## 架构修复验收

- Text01 在 `text::font` owner 内提供最小合法的等价比较访问面，且不扩大为 crate 外公共 API。
- `cargo test -p zircon_runtime --lib text_font --locked --jobs 1 -- --test-threads=1` 通过。
- 原始 Editor01 gateway 命令重新编译并实际执行全部 7 项测试。

## 禁止临时方案

- 不得把方法改成无约束 `pub`，不得在 `shared.rs` 复制字段比较逻辑。
- 不得恢复旧 database 文件布局、兼容 re-export、静默跳过等价检查或调用点特判。
- 不得弱化 Editor01 或 Text01 测试来隐藏 E0624。

## 修复结果与回传

- 根因：FontDatabase equivalence moved into database/equivalence.rs but retained pub(super), which excluded sibling text/font/shared.rs.
- 架构修复：Constrain has_same_render_inputs to pub(in crate::text::font), preserving the text-font owner boundary without public API or duplicated comparisons.
- 验证：Independent current-source review 0/0/0; Text01 behavior 46/46 with the only 47th failure routed to Runtime15; Editor01 gateway origin gate 7/7, exit 0 (job 37b0965d5e7647bb8952c3adb523145d).
- 回传：FontDatabase sibling visibility is fixed and the Editor01 origin gate passes; Runtime15 font-id report production mount remains open under its own failure.
