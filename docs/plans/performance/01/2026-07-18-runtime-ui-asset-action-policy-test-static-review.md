---
related_code:
  - zircon_runtime/src/ui/tests/asset_action_policy.rs
  - zircon_runtime/src/ui/template/asset/action_policy/validate.rs
  - zircon_runtime/src/ui/template/asset/document.rs
  - zircon_runtime/src/ui/template/asset/compiler/package/validate.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
tests:
  - three action-policy semantic tests reviewed
  - one source-level RED to GREEN root-frontier capacity guard added
  - rustfmt and scoped diff checks passed
  - current-source Cargo and package scale counters pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI asset action policy测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/ui/tests/asset_action_policy.rs` 1/1个tracked Rust文件；原始103行、3个语义测试，本轮加入1个源码性能守卫后为117行、4个测试。测试覆盖runtime local action、runtime拒绝asset I/O以及editor允许asset I/O但拒绝network的policy parity。

## PERF-MVP-311：package validation多轮tree pass

`compile_package`依次执行preconditions、cache-key/invalidation、compile、localization、dependency manifest及action-policy；`validate_document_action_policy`又独立遍历全部nodes/bindings。三个fixture各只有一个node/binding，没有记录各validator tree passes、binding visits或diagnostic bytes，不能证明10k-node package compile成本。完整single-parse/shared-index收敛继续归PERF-MVP-311和EditorUI05，不新建重复根因。

本轮先处理独立且语义安全的局部扩容：`UiAssetDocument::iter_nodes()`原先用零容量`Vec`再推入全部component roots，每个validator pass在多组件资产上重复grow/copy frontier。新增源码守卫先确认RED，再改为按`components.len() + root`预留，守卫转GREEN；遍历顺序、节点所有权和diagnostic结果不变。

## 验收要求

对1/100/1k components及1/100/10k nodes/bindings记录iterator allocation/grow、tree passes、binding visits、diagnostic bytes和package p95；完整pipeline应共享generation node/binding index，且单validator不得复制node tree。运行当前源码4项聚焦Cargo和F4 package compile trace前，本文件留在`pending.md`。
