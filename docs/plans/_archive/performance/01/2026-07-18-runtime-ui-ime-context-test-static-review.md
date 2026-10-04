---
related_code:
  - zircon_runtime/src/ui/tests/widget_text_input_ime_context.rs
  - zircon_runtime/src/ui/surface/input/editable_text/ime_context.rs
  - zircon_runtime/src/ui/surface/render/extract.rs
  - zircon_runtime/src/ui/text
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/03-text-and-font-stack.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
tests:
  - 7 IME context geometry tests reviewed
  - committed surrounding text wrap tab vertical and composition-rect parity present
  - synchronous full-render-extract refresh explicitly required by current test contract
  - indexed layout-generation/full-extract-zero counters and current-source Cargo pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI IME context测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/ui/tests/widget_text_input_ime_context.rs` 1/1个tracked Rust文件、585行、7个测试。范围覆盖preedit/commit surrounding text、cursor/composition rect、soft wrap、tab advances、vertical-rl geometry和platform manager host requests。

## PERF-MVP-296：测试合同显式要求full extract刷新

`text_input_ime_preedit_refreshes_render_extract_before_cursor_update`明确断言旧text command已从`surface.render_extract`消失，再从新command读取layout。这把每次preedit同步`refresh_render_extract_for_current_tree()`固化为正确性前提；产品实现随后还线性扫commands找owner。UI nodes或render commands增多时，单个IME owner的60/120 Hz更新会反复全树extract。

正确合同应改为：text edit提交产生text generation，增量text/layout pipeline发布owner node→layout handle；IME只读取不旧于该edit generation的indexed handle。若pipeline尚未发布，采用有界fallback/延迟host update，不得同步重建全部非目标render commands。

## Text/request ownership缺口

测试要求完整committed surrounding text、composition rects，并多处比较/clone request；helper还clone完整`UiResolvedTextLayout`。这些是测试语义，不代表产品应多份owned正文/layout。surrounding text应共享source+ranges，在平台边界才物化；cursor/composition rect消费borrowed layout。

## 验收要求

1/100/10k UI nodes、1/10k/100k chars、60/120 Hz preedit/commit记录full extract calls、command scans、layout/text/request clone bytes、generation wait/fallback、prefix visits和input p95。input路径full extract=0、layout lookup近O(1)、borrowed layout clone=0；tab/wrap/vertical/composition geometry与platform IME parity通过。current-source Cargo与Windows产品IME trace完成前，本文件留在`pending.md`。
