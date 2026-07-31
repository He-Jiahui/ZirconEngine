---
related_code:
  - zircon_runtime/src/ui/tests/v2_asset.rs
  - zircon_runtime/src/ui/tests/v2_asset
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/04-style-theme-and-painter-selector.md
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/FastUpdate/SlateInvalidationWidgetList.cpp
  - dev/slint/internal/compiler/llr/lower_to_item_tree.rs
tests:
  - 60 test definitions statically reviewed across 13 tracked and 1 untracked Rust files
  - no sleep, Instant, thread spawn, ignored test, or timing assertion found
  - current-source Cargo guard batch pending in the shared CPU FIFO
  - product-scale counters, F4 trace, and pixel evidence pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI v2 tests逐文件性能静态审查（2026-07-18）

## 范围与覆盖

本批逐文件完整阅读`zircon_runtime/src/ui/tests/v2_asset.rs`及`tests/v2_asset/**`：Git tracked 13/13，连同外部未跟踪`performance_guards.rs`为current 14/14，共4,166行、60个测试。累计UI tracked source从454/783增至467/783。该外部文件只纳入current静态事实，不改变17,706基线分母。

静态扫描命中8处clone、3处collect、41处filesystem调用、8处cache-hit断言与23处`rebuild_dirty`。没有sleep、wall-clock、线程创建或ignored测试，现有测试以确定性状态断言为主；这是合适的回归基础，但还不是动态性能验收。

## 已有性能契约

`demo_and_builder.rs`用10,000节点深链验证template/surface构建不依赖递归栈；style runtime覆盖最高512层伪状态传播；range controls明确断言pointer/keyboard改变只重建render，不触发布局、arranged tree或hit grid。file cache覆盖同进程Arc compiled artifact复用、跨cache实例persistent hit及dependency变更失效，asset loading/composite/default controls覆盖产品`.zui`投影和控件交互契约。

这些断言保留了正确的domain边界，但没有暴露node/rule/edge visits、序列化字节、文件stat/read、cache clone bytes、rebuild节点数或CPU/RSS分位数。10,000节点用例只能证明完成，不能证明O(N)；cache-hit只能证明结果复用，不能证明命中路径没有重新序列化或扫描。

## PERF-MVP-313：功能测试不能替代规模预算

当前未跟踪`performance_guards.rs`只有两个`include_str!`源码片段否定断言，可防止已知clone文本回归，但会受格式变化影响，也不能测量等价的另一种复制。EditorUI04/05需为compiled generation、selector probes、tree visits、cache I/O/serialize/clone、dirty domains提供可注入计数器；保留小型确定性单测，并另建1/100/10,000规模的current-source benchmark/产品trace gate。

验收要求：深树build的node/edge visits近O(N)且栈深有界；stable cache hit的parse/serialize/full-tree clone/stat/read为明确零预算；单节点伪状态与range事件只访问受影响path/domain；模板/样式/资源变更分别记录dirty nodes与root fallback原因。三次稳定运行报告p50/p95/p99、RSS和工件路径，Cargo与F4产品操作通过后本模块才可进入`review.md`。

## 责任计划

实现根因继续由EditorUI05的validated prototype DAG、persistent cache与hot-reload handoff负责，selector/runtime pseudo增量化由EditorUI04负责；本证据新增的是验收层，不另建重复架构根因。共享CPU FIFO中的current-source guard、规模counter、F4 load/hover/range/hot-reload trace和像素证据未完成，因此保持pending。
