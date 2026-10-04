---
related_code:
  - zircon_runtime/src/ui/tests/asset_compile_cache.rs
  - zircon_runtime/src/ui/template/asset/compiler
  - zircon_runtime/src/ui/template/asset/compiler/package
  - zircon_runtime/src/ui/template/asset/document/validation.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
tests:
  - 18 compile-cache tests reviewed
  - current-source Windows template_asset_hot_paths source guards passed 5/5
  - compile-cache hit/miss and persistent-store scale counters pending
  - F4 asset preview, edit and hot-reload product trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI asset compile cache测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/ui/tests/asset_compile_cache.rs` 1/1个tracked Rust文件、735行、18个测试。15个行为测试覆盖exact hit、document/import/contract/resource/descriptor invalidation、diagnostic miss、persistent round-trip、stale/corrupt miss及按asset清除；末尾3个源码守卫覆盖cache/package单次前置校验和tree authority借用。

## PERF-MVP-304/305：局部修复已有current-source动态门禁

该文件中的3个源码守卫连同同一过滤项命中的另外2个模板热路径守卫，已由shared Cargo coordinator执行`cargo test -p zircon_runtime --lib template_asset_hot_paths --locked --jobs 1 --color never -- --test-threads=1`并通过5/5。它们证明cache miss/package不再重复完整precondition validation、authority map不再复制每棵subtree，以及此前style/slot借用止损仍存在；这只验收局部源码契约，不等于完整cache产品路径验收。

## PERF-MVP-308/313：功能覆盖不证明缓存规模成本

行为测试使用1个document、至多2个artifact/version和临时目录。它们没有记录compile key序列化字节、import closure遍历、cache-hit artifact clone、filesystem calls/bytes、目录候选扫描、容量上限、eviction p95或RSS；`evict_asset`只用两个文件，不能证明persistent manifest/LRU已避免递归目录读取与逐记录反序列化。既有PERF-MVP-308/313及EditorUI05 handoff继续负责generation-owned `Arc` artifact、dependency fingerprint、manifest/LRU和1/100/10k资产预算，无需新增重复根因。

## 验收要求

对1/100/10k documents/import edges/cache records分别测量key serialized bytes、graph visits、hit clone bytes、filesystem calls/read bytes、eviction candidates、cache bytes/RSS及compile/load/evict p50/p95/p99。稳定generation的cache hit要求document/import重新序列化=0、compiled artifact深复制=0；evict单asset只访问manifest索引命中的records。完成整组行为Cargo、规模counter以及F4 asset preview/edit/hot-reload产品trace前，本文件留在`pending.md`。
