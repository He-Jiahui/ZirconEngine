---
related_code:
  - zircon_runtime/src/runtime_diagnostics
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - zircon_runtime/src/core/runtime/diagnostics
  - zircon_runtime/src/graphics/runtime/render_framework
tests:
  - zircon_runtime/src/runtime_diagnostics
  - current-source Windows Cargo and F2/F4 diagnostics traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime diagnostics facade逐文件性能静态审查（2026-07-19）

## 范围与覆盖

`zircon_runtime/src/runtime_diagnostics/**`当前源 **4/4** 个Rust文件、**261** 行、**3** 条测试已逐文件阅读，覆盖render/physics/animation/store/profile聚合、devtools projection及physics feature双分支。4个文件当前均为其他会话未跟踪改动，本轮只读保留。

## 性能结论

- 每次`collect_runtime_diagnostics`无条件收集render、physics、animation、diagnostic store和完整profiling snapshot；consumer只需单一domain也承担全部所有权。
- render query返回宽owned RenderStats；`query_virtual_geometry_debug_snapshot`只为`is_some()`仍可能复制巨大debug payload，分别复用PERF-MVP-324/416/418。
- `collect_diagnostic_store_snapshot`每次把RenderStats展开约541条series后深snapshot store/history；同generation的log、profile和可见pane会重复，归PERF-MVP-324的domain generation/packed delta/snapshot cache。
- physics enabled分支复制backend/status/detail Strings，animation复制settings；相较render payload次要，随domain generation一并量化，不单建任务。

## 验收要求

hidden/render-only/physics-only/full、0/30/60/120Hz及same/changed generation记录manager resolve、stats/VG query、series writes、store/profile snapshot clone bytes与p95：hidden全0、同generation每domain build≤1、boolean availability不得深复制VG payload。current-source Cargo、F2/F4产品trace及RenderDoc可见debug验证完成前留在`pending.md`。
