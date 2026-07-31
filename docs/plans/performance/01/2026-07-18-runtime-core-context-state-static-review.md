---
related_code:
  - zircon_runtime/src/core/runtime/contexts
  - zircon_runtime/src/core/runtime/state
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
tests:
  - seven production Rust files reviewed
  - current-source Cargo and lifecycle scale counters pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime core context/state逐文件性能静态审查（2026-07-18）

已完整阅读`runtime/contexts/**` 3/3与`runtime/state/**` 4/4个生产Rust文件，合计180行。`CoreRuntimeInner`把modules/services/config/events/tasks/time/diagnostics/state各自隔离为锁或独立owner，没有单一全局runtime锁；风险集中在ModuleContext/PluginContext每次复制String/PathBuf/Weak，以及ModuleEntry保留descriptor同时拥有三套RegistryName列表、ServiceEntry再次拥有dependency/factory投影。它们已分别纳入PERF-MVP-321/322。

`ServiceEntry`卸载/失败会推进generation并清空instance，避免stale handle误用；此处未发现可独立安全修改的新热路径。验收需在1/100/10k modules/services与反复activate/unload中记录context/name/path/dependency clone bytes、lock wait、generation wrap/stale handle和RSS；冻结arena/context方案完成且current-source Cargo/F0/F2 trace通过前，7文件留在`pending.md`。
