---
related_code:
  - zircon_runtime/src/ui/tests/hit_grid.rs
  - zircon_runtime/src/ui/tree/hit_test.rs
  - zircon_runtime/src/ui/surface/arranged_tree.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
  - docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Private/Input/HittestGrid.cpp
tests:
  - 10 behavioral hit-grid tests reviewed plus one new source guard
  - exact-query borrowed-cell guard observed RED then GREEN
  - rustfmt and scoped source predicate passed
  - current-source Cargo and dense-radius scale counters pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI hit grid测试逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/ui/tests/hit_grid.rs`，当前1/1个tracked Rust文件、649行、11个测试，其中10个行为测试覆盖disabled/virtual window/focus/capture/cursor radius/world/scope/borrowed-grid/dirty rebuild，另1个是本轮新增的源码性能守卫。

## PERF-MVP-316：每pointer query的候选scratch

原`hit_test_grid_arranged_with_query`即使`cursor_radius=0`且只访问一个cell，也依次分配cell indices、deduped/sorted entry indices、exact hits、radius hits和最终stacked五个Vec；随后每candidate还走PERF-MVP-277的linear arranged get/ancestor input-policy。半径查询跨cells时以`entries.contains`去重，密集重叠候选最坏O(K²)，并再排序radius hits。

## 已直接止损：exact query借用单cell

新增源码守卫先观察RED。本轮为最常见零半径查询增加borrowed single-cell fast path：直接反序遍历cell已按z/paint/node排序的entry indices，复用同一clipped-frame与input-policy过滤，只构造API返回必需的stacked/route；不再创建或排序四个中间候选Vec。守卫转GREEN并通过`rustfmt`。半径、world/scope和debug路径保持原逻辑，避免一次改动多项排序合同。

## 剩余架构与验收

EditorUI01应为radius query使用generation-stamped dense mark/scratch或等价近线性去重，并复用frame-local buffer；EditorUI02完成node-id→dense arranged index和继承input-policy，消除每candidate线性get/ancestor walk。1/100/1k/10k overlapping entries、radius 0/8/64、120 Hz连续100k queries记录cells/entries/contains probes、sort、temp allocations、arranged/ancestor probes和CPU p95。exact query中间candidate Vec alloc=0，radius dedupe近O(K)，命中顺序/route/scope/world parity通过。current-source Cargo和F4 pointer trace完成前，本文件留在`pending.md`。
