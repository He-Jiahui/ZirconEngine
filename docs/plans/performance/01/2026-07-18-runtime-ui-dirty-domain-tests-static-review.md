---
related_code:
  - zircon_runtime/src/ui/tests/surface_dirty_domains.rs
  - zircon_runtime/src/ui/tests/surface_dirty_domains
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_editor/editor_ui/02-layout-taffy-and-containers.md
tests:
  - 5 tracked Rust files and 15 test definitions statically reviewed
  - visited/skipped/reused/rebuilt/damage deterministic counters covered on tiny trees
  - 100/1k/10k hidden-stage counters and product trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime UI dirty-domain tests逐文件性能静态审查（2026-07-18）

本批逐文件完整阅读`tests/surface_dirty_domains.rs`与其目录5/5个tracked Rust文件、1,056行、15个测试。累计UI tracked source从520/783增至525/783。覆盖dirty summary单pass源码守卫、domain→stage映射、render cache reuse、damage、incremental layout route merge与removed-node清理。

现有小树测试断言`layout_visited/skipped/geometry_changed_node_count`、`render_command_reused/rebuilt_count`、damage rect以及layout/arranged/hit/render stage开关；render-only mutation不触发layout/hit，单dirty leaf在Free parent下报告visited=1/skipped=2。这些是PERF-MVP-259/281的可靠功能基线。

但1到4节点夹具不能证明`visited_node_count`涵盖responsive/root discovery/arranged/hit的隐藏全树扫描，也没有100/1k/10k nodes、1% dirty与stable generation测试。EditorUI02联动EditorUI08需把各stage真实visits/generation计入同一report：stable=0，single leaf工作随changed boundary，render cache只在结构/owner失效清空。Cargo、规模counter、F4 layout/paint trace与像素完成前保持pending。
