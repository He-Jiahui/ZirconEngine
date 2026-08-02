---
related_code:
  - zircon_runtime/src/ui/tests/asset_binding.rs
  - zircon_runtime/src/ui/template/asset/binding/validation.rs
  - zircon_runtime/src/ui/component/catalog
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
  - docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
tests:
  - 23 asset binding semantic tests reviewed
  - target/ref/operator/scope/descriptor/payload diagnostic parity present
  - shared-registry expression/index/pass scale counters pending
  - current-source Cargo and F4 template compile product trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI asset binding测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/ui/tests/asset_binding.rs` 1/1个tracked Rust文件、799行、23个测试。范围覆盖prop/class/visibility/enabled/payload target、boolean/operator、param/control refs、component scope、descriptor authority及compiler precondition diagnostics。

## PERF-MVP-264：owned registry重复取得

多数test case调用`UiComponentDescriptorRegistry::editor_showcase()`取得完整owned registry，compiler case还clone registry。测试隔离可接受，但产品compiler/validator必须使用shared generation handle；不得让每次asset validation深clone descriptor/String/Vec/BTreeMap。PERF-MVP-264已提供shared入口，后续需保证所有compiler consumer采用它。

## PERF-MVP-311：每case小图不证明validator复杂度

`collect_asset_binding_report`为scope建立control property图并递归parse expressions；本文件通常只有1到4 nodes/bindings，没有tree passes、path String、expression parse、control index build或diagnostic bytes计数。EditorUI05 compiled validation pipeline应共享node/path/control/property indexes与parsed expression，单generation同表达式/descriptor parse<=1。

## 验收要求

nodes/bindings/scopes/control refs各1/100/10k，registry descriptors 1/100/1k，记录registry clone bytes、tree passes、index builds、expression parses、path/message bytes和compile p95。shared registry clone=0；control index每scope/generation<=1；完整validator共享tree walk有明确上限。保留当前diagnostic code/severity/node/binding/message和compiler error优先级。current-source Cargo与F4 template compile trace完成前，本文件留在`pending.md`。
