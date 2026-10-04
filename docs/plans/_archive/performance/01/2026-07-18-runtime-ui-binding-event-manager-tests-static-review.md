---
related_code:
  - zircon_runtime/src/ui/tests/binding.rs
  - zircon_runtime/src/ui/tests/event_manager.rs
  - zircon_runtime/src/ui/binding
  - zircon_runtime/src/ui/event_ui
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_editor/editor_ui/06-component-library-mui.md
tests:
  - 7 binding/router/reflection tests reviewed
  - parse route update-report diff and error parity present
  - slow/dead subscriber queue/bytes/backpressure and mutation transaction counters pending
  - current-source Cargo and remote-control product trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI binding/event manager测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/ui/tests/{binding,event_manager}.rs`，共2/2个tracked Rust文件、329行、7个测试。范围覆盖native binding roundtrip、exact router、binding update dirty classification、route invocation、reflection diff/property/action request与missing-route error。

## PERF-MVP-252：测试没有慢/死订阅

event manager测试subscribe后立即阻塞`recv()`一条notification，无法覆盖remote `SubscribeDiffs`丢弃receiver却保留sender、send failure不清理、slow consumer无界queue或每subscriber notification clone。1/100/10k subscribe/disconnect与100k reflection/invocation storm必须记录live/dead senders、queue entries/bytes、clone bytes、send failures、drop/coalesce/age。

## PERF-MVP-265：update report规模与事务

binding helper测试先clone四个update再组成owned report，锁定source/target/status/dirty union语义；它没有证明一个logical widget action只构造一次report/transaction，也没有String/UiValue clone和unchanged write预算。EditorUI06 typed patch应直接产出changed-field report，不让alias逐字段重复update。

## 验收要求

1/100/10k bindings/routes/nodes/subscribers、100k updates/diffs记录parse/index probes、report/update/value clone bytes、transactions、queue/backpressure和CPU p95。stable compiled binding不reparse；action transaction=1；disconnect/send failure后dead sender下一broadcast清零；slow consumer内存有界且最终generation正确。current-source Cargo与remote-control/diagnostic product trace完成前，2/2留在`pending.md`。
