---
handoff_kind: failure
status: open
created_at: 2026-07-18
summary_slug: ui-surface-pairwise-overlap-batching
origin_plan: docs/plans/performance/01-mvp-performance-audit-and-optimization.md
fixing_plan: docs/plans/zircon_runtime/render/17-performance-and-profiling.md
origin_child_dir: docs/plans/performance/01
fixing_child_dir: docs/plans/zircon_runtime/render/17
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/rhi_wgpu/ui_surface.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/batching.rs
  - zircon_runtime/src/rhi_wgpu/ui_surface/batching/dependency_depths.rs
  - zircon_runtime/src/rhi/ui_surface.rs
---

# UI surface pairwise overlap batching

## 来源执行者

- 来源计划：`docs/plans/performance/01-mvp-performance-audit-and-optimization.md`
- 来源执行者：`20260717-0515-performance-mvp-audit`
- 来源执行切片：RHI/WGPU UI surface聚焦5/5 Rust文件
- 修复责任计划：`docs/plans/zircon_runtime/render/17-performance-and-profiling.md`
- 交接原因：runtime GPU UI batch compilation、profiling counters与产品RenderDoc验收属于Render17，不能由editor painter建立第二套backend cache。

## 失败现象与复现证据

原batch planner每present对N个visible items执行N(N-1)/2 rect checks；1k disjoint rows固定499,500，10k固定49,995,000。FullRedraw对`damage=None`仍深clone完整draw list/RGBA；随后draw-item extraction、stats、image upload与text prepare重复扫描。性能切片已落adaptive-axis interval index与无damage full-redraw borrow止损。

## 最低共享层根因

WGPU presenter没有generation-owned compiled batch/spatial plan；batch dependency、visible stats、image upload与text preparation各自从owned command list重新派生事实。damage patch也没有共享前一generation的batch topology。

## 架构修复验收

- command/presentation generation未变时batch plan与interval index build=0；damage只过滤/patch受影响range，不重编全表。
- 一次visible projection同时产出draw items、visible command/image/upload stats与text/image preparation rows；每present command visibility scan≤1。
- 1/100/1k/10k rows/columns/mixed clips/all-overlap记录axis candidates、exact dependencies、CPU p50/p95、alloc与clone bytes；稀疏规模近O(N log N + candidates)。
- 1k disjoint candidates≤1k且dependency=0；all-overlap保持N(N-1)/2 dependencies和原painter order。
- no-damage FullRedraw clone bytes=0；cache bootstrap patch仍绘制完整frame并正确初始化retained cache。
- current-source Cargo、GPU/Softbuffer像素和RenderDoc pass/draw/resource对拍通过。

## 禁止临时方案

- 不得关闭non-overlap painter-order约束、减少visible commands或错误合并overlap items来降低CPU/draw calls。
- 不得把每present全表batch rebuild无界投递worker；先用generation消除稳定工作，再评估有界并行。
- 不得由editor维护与runtime draw list不一致的私有spatial/batch cache，或省略clip/damage后的effective rect。

## 修复结果与回传

Open state: `generation-owned compiled full projection/batch/spatial plan、稳定generation stats cache、首次damage compile单次command traversal、稳定image resource-key preparation fast path及1/100/1k/10k rows/columns确定性scale counter矩阵已落地；stats与WGPU geometry共享finite-positive rect可见性契约，防止NaN/Infinity绕过区间与damage判定；待current-source managed Windows validation、p50/p95与alloc/clone预算、GPU/Softbuffer像素及RenderDoc parity后回传`。
