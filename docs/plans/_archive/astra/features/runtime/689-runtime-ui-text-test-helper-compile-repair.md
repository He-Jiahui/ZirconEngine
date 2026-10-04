---
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11b-runtime-text-font-shaping-layout-editing-ime-review.md
  - docs/plans/optimize/zircon_runtime/81/2026-08-27-nowrap-clip-width-semantics-review.md
related_code:
  - zircon_runtime/src/ui/text/layout_engine/tests/rich_layout.rs
  - zircon_runtime/src/ui/tests/widget_text_input_ime_context.rs
---

# Runtime UI/Text 测试辅助编译修复

历史受管 Windows UI-feature 编译日志定位到五项当前测试漂移：rich-layout 回归缺少
`measure_text_size` 与 `UiTextRange` 的当前作用域导入（分别在 ellipsis frame 和
glyph-artifact range 断言中使用），而 IME context 回归两处调用的 `int_attr` 本地测试
辅助函数已不存在。

本批次恢复这两个测试模块所需的本地引用：rich-layout 从 layout-engine 当前模块导入
measurement helper，并从 interface surface 导入 range DTO；IME 测试在已有
`text_attr`/`usize_attr` 辅助函数旁恢复只读整数属性读取。它不改变 rich layout、IME
dispatch、attributes、文本选择或生产性能路径。

## 计划完成列表

| 批次 | 内容 | 状态 | 验证证据 |
|---|---|---|---|
| Runtime11A / Runtime11B / Runtime81 | rich-layout 与 IME context 五项测试作用域/辅助函数编译修复 | `implemented_pending_validation` | 历史受管日志定位 `rich_layout.rs` 三项和 `widget_text_input_ime_context.rs` 两项诊断；当前源码守卫 `5/5`、Runtime UI performance-contract 批次 `213/213`（`0.371s`）、Runtime Text contract 批次 `143/143`（`0.766s`）、scoped diff-check 通过。受管 Windows Cargo、原始 Rust 回归和 Release p50/p95/p99 仍待异步批量验证。 |

这两份测试文件的整文件 Rustfmt 检查仍报告修改前已存在、且不在本次 diff hunk 内的
格式差异；本批次没有扩大为格式化重写。上述结果不替代受管编译或产品性能验收。
