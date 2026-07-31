---
related_code:
  - zircon_runtime/src/core/framework/project
  - zircon_runtime/src/core/framework/time
  - zircon_runtime/src/core/framework/tasks
  - zircon_runtime/src/core/framework/state
  - zircon_runtime/src/core/framework/window
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/runtime/03-schedule-and-frame-loop-alignment.md
  - docs/plans/zircon_runtime/runtime/07-runtime-performance-hotpath.md
tests:
  - project twelve of twelve Rust files reviewed
  - time six of six Rust files reviewed
  - tasks ten of ten Rust files reviewed
  - state eleven of eleven Rust files reviewed
  - window fourteen of fourteen Rust files reviewed
  - fixed-step large catch-up regression test added
  - source-level RED to GREEN performance guard passed
  - rustfmt and scoped diff checks passed
  - current-source Cargo and frame-loop product traces pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# Runtime core framework MVP contracts逐文件性能静态审查（2026-07-18）

## 范围与覆盖

已完整阅读`core/framework/{project,time,tasks,state,window}`当前Rust文件53/53：project 12、time 6、tasks 10、state 11、window 14。Project/export与window目录是serde数据、policy/builder、尺寸验证和显式diagnostic formatting，没有线程、锁、I/O或隐藏逐帧入口；plugin selection线性更新和owned crate-name只在项目/导出边界，window diagnostics分配只在显式报告。Tasks定义compute/async/io池、取消策略和默认每帧100次main-thread poll预算，真实调度成本继续归Runtime11/PERF-MVP-317。

StateMachine复核并归并到PERF-MVP-320：每次transition clone event进无界Vec，查询全量clone；enter/exit/transition三表分别线性filter并为匹配hook clone Arc/分配Vec。此处不在公开history retention尚未裁决时擅自截断。

## PERF-MVP-328：fixed-step计划原先按step空循环

`Time<Fixed>::drain_steps(max_steps)`原先在真正执行任何simulation system前，先以`while`逐step减overstep，并逐次更新delta/elapsed/frame index。正常2至8步成本小，但暂停恢复、headless catch-up、测试或错误配置把timestep降到ns且max_steps放大时，时钟计划本身就可在主线程执行百万次循环。

本轮新增百万step行为/源码RED→GREEN守卫，并用精确Duration整数除法一次计算`min(overstep/timestep, max_steps)`，批量扣除consumed并饱和更新elapsed/frame index；delta、remaining overstep、cap、zero-step和既有3/2-step结果等价。该修改只删除时钟层空循环，不跳过调用方真正要求的fixed systems。

## 剩余计划与验收

Runtime03/07仍需让产品frame loop显式记录requested/executed/capped/dropped-or-deferred fixed steps、remaining overstep与simulation CPU；max step cap必须按client/editor/headless profile配置，长stall后不能无限追帧，也不能静默丢时间。对timestep 1 ns/1 ms/16.667 ms、delta 0/16/250/10k ms、cap 0/1/8/1M及60/120 Hz运行记录clock iterations、system executions、overstep、frame p95与input latency：clock plan iterations=O(1)，system executions≤cap，editor stall后有明确defer/drop策略；pause/speed/max-delta/interpolation parity、Cargo/F0/F2/F4 trace通过前，五目录留在`pending.md`。
