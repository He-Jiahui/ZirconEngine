---
related_code:
  - zircon_runtime/src/ui/tests/taffy_layout_pass.rs
  - zircon_runtime/src/ui/tests/taffy_layout_pass
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
tests:
  - 6 tracked Rust files and 35 test definitions statically reviewed
  - Taffy/fallback/slot pixel and routing semantics covered
  - persistent-tree, slot-index, diagnostic-budget, and large-tree counters pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI Taffy layout tests逐文件性能静态审查（2026-07-18）

本批逐文件完整阅读`tests/taffy_layout_pass.rs`与其目录6/6个tracked Rust文件、1,367行、35个测试。累计UI tracked source从530/783增至536/783。覆盖flex/wrap/grid、linear/grid slot sizing/padding/alignment、fallback policy与selection diagnostics。

功能断言细致，但所有夹具只有少量nodes/slots；没有tree create、node insert/style/children set、slot probes或stable compute计数，无法验收PERF-MVP-260/261。routing diagnostics测试用2次filesystem读取源码做字符串断言，并有9次`surface_frame()`调用；fallback selection为每container保留完整记录，没有PERF-MVP-263要求的release reason-count/first-example预算测试。

EditorUI02验收需在100/1k/10k nodes/slots和稳定300帧记录slot probes、Taffy create/insert/style/children/compute、selection entries/bytes与CPU p95：slot edge近O(1)，stable create/insert=0，release完整selection默认关闭而reason counts保留。现有fallback/pixel/slot语义必须保持；Cargo、规模counter与产品layout trace/像素完成前保持pending。
