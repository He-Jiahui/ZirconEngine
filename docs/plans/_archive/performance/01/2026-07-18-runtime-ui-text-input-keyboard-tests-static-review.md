---
related_code:
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard
  - zircon_runtime/src/ui/surface/input/editable_text
  - zircon_runtime/src/ui/surface/input/text_state.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
tests:
  - 52 keyboard/text/IME semantic tests reviewed
  - grapheme CRLF word line selection clipboard filter and composition parity present
  - 10k/100k-char clone/prefix/property-transaction counters pending
  - current-source Cargo and F4 editor text-input trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI text input keyboard测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已逐文件完整阅读`zircon_runtime/src/ui/tests/widget_text_input_keyboard.rs`与`widget_text_input_keyboard/**`，共6/6个tracked Rust文件、1,373行、52个测试。范围覆盖basic edit、grapheme deletion/navigation、selection、CRLF/multiline、word/document shortcuts、clipboard、filter/max chars及IME preedit/cancel/commit。

## 强语义门禁

测试对Unicode grapheme、CRLF line boundary、selection anchor/focus、read-only、single-line、clipboard host request和composition restore有细粒度断言，适合作为persistent editable state与atomic patch cutover的回归矩阵。caret-only操作不得产生ValueChanged正文事件，read-only仍允许navigation。

## PERF-MVP-295：短文本没有正文ownership/事务预算

fixture文本通常只有数到十几个bytes；几乎每次Arrow/Home/End/selection操作都要求binding report。产品当前从TOML metadata重建`UiEditableTextState`并复制正文/composition，再分别提交caret/selection/composition等property mutation。测试没有text copied bytes、metadata reads、property transactions、binding updates、dirty commits或unchanged writes计数，不能验收caret-only正文零clone和每event单事务。

## 长文navigation与IME缺口

word/line/grapheme helpers在1/10k/100k chars上的prefix visits和CPU未测；连续preedit/selection也没有60/120 Hz预算。PERF-MVP-296还要求IME读取indexed layout handle而非同步full render extract，并让surrounding text共享source+ranges。

## 验收要求

1/10k/100k chars、连续100k type/caret/selection/word/line/IME operations记录text/composition clone bytes、grapheme/line/prefix visits、metadata reads、transactions、binding reports、dirty stages与CPU p95。caret-only正文clone=0、transaction<=1；line/word navigation采用可观测index/cache且不重复full prefix；IME input full extract=0。current-source Cargo与F4 Inspector/Console文本输入trace及像素完成前，6/6留在`pending.md`。
