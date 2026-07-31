---
related_code:
  - zircon_runtime/src/core/runtime/diagnostics/animation.rs
  - zircon_runtime/src/core/runtime/diagnostics/devtools.rs
  - zircon_runtime/src/core/runtime/diagnostics/frame_diagnostics.rs
  - zircon_runtime/src/core/runtime/diagnostics/mod.rs
  - zircon_runtime/src/core/runtime/diagnostics/physics_backend.rs
  - zircon_runtime/src/core/runtime/diagnostics/physics.rs
  - zircon_runtime/src/core/runtime/diagnostics/render_stats_store.rs
  - zircon_runtime/src/core/runtime/diagnostics/render.rs
  - zircon_runtime/src/core/runtime/diagnostics/snapshot.rs
  - zircon_runtime/src/core/runtime/diagnostics/store.rs
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
tests:
  - ten root production Rust files reviewed
  - two source-level RED to GREEN performance guards added
  - rustfmt and scoped diff checks passed
  - current-source Cargo, diagnostic allocation counters and product trace pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime core diagnostics root逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`zircon_runtime/src/core/runtime/diagnostics`根目录10/10个生产Rust文件，当前937行/7个inline测试。范围覆盖DiagnosticStore、runtime/devtools snapshot、render/physics/animation facade与render stats分发根；`profiling/**`和`render_stats_store/**`均已另行逐文件登记。

## PERF-MVP-324：静态render metrics原先走通用分配路径

`render_stats_store`的count/bytes/microseconds/bool helper被13个render diagnostics family逐帧调用，却使用通用`DiagnosticStore::record`：即使series已存在，每条也先构造owned DiagnosticPath，并把静态unit/tags转String、查重、排序。Time diagnostics已有`record_static`快路。新增RED→GREEN守卫让4个helper全部使用该快路，稳定metadata时只追加measurement；path/unit/tag owner保持不变。

Devtools snapshot原先在module/service registry guard仍存活时对owned snapshots排序，放大activation/resolution等待；tag汇总先clone所有重复String再sort/dedup。第二项RED→GREEN止损显式在sort前drop两把registry guard，并对tag借用`&str`排序去重后仅分配唯一输出。

## 剩余根因

通用`record`仍为动态path/metadata每次执行owned转换和tags线性查重+排序；snapshot会为全部series深clone path/unit/tags及最多64条history，devtools再全量复制modules/services/dependencies/hooks/catalog并排序。若编辑器每帧拉取快照，成本与全部历史/registry规模相关，而不是变化量。Render stats叶模块还需逐文件确认实际每帧metric数量和unchanged record频率。

Runtime07/Render17应建立generation/delta snapshot与静态series registration token：record hot path只用dense ID写ring/current，metadata注册一次；消费者按generation/cursor取delta，完整导出按需执行且不持runtime registry锁。

## 验收要求

对series 1/100/10k、metrics/frame 1/100/10k、history 1/64/1k、modules/services/deps各1/100/10k记录path/unit/tag/history clone bytes、metadata probes/sorts、store/registry lock hold/wait、snapshot bytes和frame p95：static render metric metadata alloc=0，snapshot idle delta bytes/visits近0，sort在registry锁外，tag只为唯一输出分配；完整snapshot保持排序/内容parity。current-source Cargo、F2/F4 diagnostic UI trace通过前，10文件留在`pending.md`。
