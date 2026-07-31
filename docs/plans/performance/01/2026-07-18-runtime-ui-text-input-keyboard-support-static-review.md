---
related_code:
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard_clipboard.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard_hard_line.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard_text.rs
  - zircon_runtime/src/ui/surface/input/editable_text
  - zircon_runtime/src/ui/surface/input/editable_text/ime_context.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
tests:
  - 11 clipboard/hard-line/keyboard-text tests reviewed
  - read-only modifier text payload and active IME context parity present
  - full render-extract surrounding-text/request clone and long-text counters pending
  - current-source Cargo and platform IME product trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI text input keyboard支撑测试静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/ui/tests/{widget_text_input_keyboard_clipboard,widget_text_input_keyboard_hard_line,widget_text_input_keyboard_text}.rs`，共3/3个tracked Rust文件、793行、11个测试。范围覆盖read-only paste、Super/Alt hard-line navigation、printable keyboard payload、selection/filter/max、stale owner、Tab navigation、single-line newline和active IME context refresh。

## PERF-MVP-295：hard-line与caret-only仍是短文

hard-line测试使用不足20 bytes文本并要求每次navigation生成binding report；没有long-text line index/prefix visits、正文clone或atomic selection transaction计数。persistent editable state必须让caret-only正文owner不变，并以同generation line/grapheme index定位而非每key全字符串扫描。

## PERF-MVP-296：active IME同步上下文成本被显式消费

printable payload在active input method owner下要求`UpdateCursor`、完整surrounding text、cursor/anchor、cursor rect，并把host request clone后存入`surface.input.input_method_request`。产品路径当前为取得layout同步刷新full render extract再线性找target，且request/committed text多份所有权。测试锁语义但没有full extract calls、command scans、layout/text/request clone bytes或100k-char prefix成本。

## 验收要求

1/10k/100k chars、连续100k hard-line/caret/text payload及60/120 Hz IME update记录line/prefix visits、full extract calls、command scans、text/layout/request clone bytes、transactions和CPU p95。input路径full extract=0、layout lookup近O(1)、surrounding text共享source+ranges、同request generation不重复owned clone。current-source Cargo、Windows IME与F4编辑器文本输入trace完成前，3/3留在`pending.md`。
