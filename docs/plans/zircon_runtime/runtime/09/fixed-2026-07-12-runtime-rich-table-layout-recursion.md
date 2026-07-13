---
handoff_kind: fixed
status: fixed
created_at: 2026-07-12
resolved_at: 2026-07-12
summary_slug: runtime-rich-table-layout-recursion
origin_plan: docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md
fixing_plan: docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md
origin_child_dir: docs/plans/zircon_runtime/runtime/09
fixing_child_dir: docs/plans/zircon_editor/editor_ui/03
related_code:
  - zircon_runtime/src/ui/text/layout_engine.rs
  - zircon_runtime/src/ui/text/layout_engine/rich_table.rs
  - zircon_runtime/src/ui/text/layout_engine/tests/rich_table/mod.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/render/tests/rich_table.rs
tests:
  - cargo test -p zircon_runtime --lib ui::text::layout_engine::tests::rich_table --locked -- --test-threads=1
  - cargo test -p zircon_runtime --lib graphics::scene::scene_renderer::ui::render::tests::rich_table --locked -- --test-threads=1
  - cargo test -p zircon_runtime --lib --locked -- --test-threads=1
---


# Editor UI 03：Runtime rich-table layout 递归导致栈溢出

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/runtime/09-ui-subsystem-architecture.md`
- 来源执行切片：Runtime 02–15 当前默认 feature 全量 Runtime lib-test 回归
- 修复责任计划：`docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md`
- 交接原因：失败位于正在由 Editor UI 03/Text 07 实现的 rich-table 解析与 `UiResolvedTextLayout.boxes` 布局链，Runtime 09 只消费其 UI layout 结果，不越界重写文本 owner。

## 失败现象与复现证据

2026-07-12 当前源码默认 feature Runtime lib-test 成功编译后，完整测试运行到：

`graphics::scene::scene_renderer::ui::render::tests::rich_table::screen_space_ui_plan_paints_table_cell_background_before_text_and_border_after_text`

发生 Windows 栈溢出（进程状态 `-1073741571`）。精确过滤进一步证明：

- `ui::text::layout_engine::tests::rich_table` 在第一项 `text_rich_bbcode_table_authored_padding_controls_content_origin_and_measure` 即栈溢出；
- `graphics::scene::scene_renderer::ui::render::tests::rich_table` 同样栈溢出；
- 把 `RUST_MIN_STACK` 提高到 128 MiB 仍栈溢出，排除普通深调用，指向无终止递归；
- 相邻 `graphics::text::layout::rich::tests` 4/4 通过。

当前最低可疑链是 `layout_parsed_text_with_provider -> layout_rich_tables_with_provider -> layout_cell_range_with_provider -> slice_parsed_with_table_depth -> layout_parsed_text_with_provider`。`slice_parsed_with_table_depth` 把位于 cell 范围内且 `table.depth > parent_table_depth` 的表重新归一化为 depth 0；对于当前非嵌套表，必须证明当前表不会被重新选入，否则布局会无限递归。

## 最低共享层根因

Rich-table cell 切片的表包含/深度归一化契约未保证严格递归收敛，导致 cell layout 可再次进入同一 table owner。根因应在文本布局 owner 的 table-slice/递归边界修复，不能在 renderer 测试、线程栈大小或 Runtime 09 调用点掩盖。

## 架构修复验收

- table cell 子布局只允许进入严格更深且严格包含于当前 cell 的嵌套 table；当前 table 本身不能重新出现在局部 `UiParsedText.rich.tables`。
- 添加直接锁定单层表终止与嵌套表深度递减的 focused 测试。
- `ui::text::layout_engine::tests::rich_table` 全部通过且无栈溢出。
- renderer rich-table 精确过滤通过。
- Runtime 默认 feature 全量 lib-test 从本失败点继续完成，并由 Runtime 09/02 更新上行结果。

## 禁止临时方案

- 禁止提高线程栈、ignore 测试、捕获进程崩溃或回退为 plain text。
- 禁止添加兼容 shim、静默丢弃所有 nested tables 或在 renderer 单点特殊处理。

## 修复结果与回传

- 根因：Runtime acceptance initially executed a stale 19:05 test binary; current Text 07 table slicing only retains tables strictly deeper than the parent, so the current table cannot re-enter cell layout.
- 架构修复：Keep rich-table recursion in the shared text layout owner with strict depth descent and normalized local depth; no stack-size, ignore, plain-text, or renderer special-case workaround.
- 验证：Current-source binary: original exact crash 1/1; ui::text::layout_engine::tests::rich_table 10/10; graphics::scene::scene_renderer::ui::render::tests::rich_table 1/1. Full serial Runtime run crossed the former crash point but exceeded the 1200-second command limit.
- 回传：Fixed return accepted by Runtime 09; focused lower-layer and renderer filters pass, while full Runtime completion remains pending due command timeout.
