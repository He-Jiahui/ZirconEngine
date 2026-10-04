---
related_code:
  - zircon_runtime/crates/zr_rhi/src/tests
plan_sources:
  - docs/plans/performance/01-mvp-performance-audit-and-optimization.md
  - docs/plans/zircon_runtime/render/17-performance-and-profiling.md
reference_sources:
  - zircon_runtime/crates/zr_rhi/src
  - zircon_runtime/crates/zr_rhi_wgpu/src
tests:
  - current-source Windows zircon_runtime RHI tests pending
doc_type: implementation-evidence
status: static_complete_dynamic_pending
---

# RHI contract tests逐文件性能静态审查（2026-07-18）

## 范围与覆盖

`zircon_runtime/crates/zr_rhi/src/tests/**`当前源26/26个Rust文件、5,746行已逐文件阅读，覆盖boundary、capabilities/descriptors、command list、debug markers/status、resource/device lifecycle、bind groups、pipelines、render pass clear/lifetime/view/resolve/state、buffer/texture copy及usage/range错误合同。

这些测试全部驱动CPU deterministic `WgpuRenderDevice` test double，资源与command规模很小。它们能保护PERF-MVP-226的distinct/self-overlap buffer copy、row-stride/mip/layer/cube texture copy及validation错误语义，但不能证明真实adapter、queue、fence、GPU memory、readback等待、timestamp或RenderDoc结果。

## 性能结论

测试面没有1k/10k command、resource churn或allocation/lock counter。`boundary.rs`与`device_contract/framework_boundary.rs`重复递归扫描framework/app/editor Rust源树；前者范围更强且还覆盖interface/manifests，后者形成随仓库增长的重复filesystem I/O。该问题只影响测试反馈速度，回链P2 PERF-MVP-226；在产品F4热点与Cargo blocker关闭前不做删除/合并。

`debug_status.rs`断言CPU test double支持graphics debugger capture，而实现并无真实capture；这进一步证明capability/profile不能直接用于产品基线，边界已交接Render17。其余tests是必要的错误合同，不应为了吞吐关闭validation。

## 动态验收

受管scene Cargo job `aefa636dfd58408bb195716eacb771ba`在编译`zircon_runtime`时被两处资产模块错误挡住，未执行任何RHI/scene test。资产owner修复后运行`cargo test -p zircon_runtime --lib rhi --locked`、`texture_copy`和`ui_surface` focused filters；记录test count/elapsed与source-scan I/O，再补1/100/10k deterministic commands。真实GPU验收仍由native graphics/UI path的marker/timestamp/RenderDoc承担，故本目录继续留在`pending.md`。
