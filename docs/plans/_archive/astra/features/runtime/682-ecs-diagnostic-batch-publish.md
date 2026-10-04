---
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_runtime/03/2026-08-24-ecs-diagnostic-batch-publish.md
related_code:
  - zircon_runtime/src/scene/ecs/frame_performance_diagnostics.rs
  - zircon_runtime/src/core/runtime/diagnostics/store.rs
---

# Runtime03 ECS 诊断批量发布

`EcsFramePerformanceDiagnostics::publish` 现在通过一次
`CoreHandle::update_diagnostic_store` 写入整帧 58 项诊断；固定元数据路径使用
`record_static` 复用已发布的 path/unit/tag 存储。诊断顺序、帧号、数值和标签保持不变，
没有改变诊断 store 的公开快照合同。

## 计划完成列表

| 批次 | 内容 | 状态 | 验证证据 |
|---|---|---|---|
| Runtime03 | ECS 诊断单锁批量发布与静态元数据复用 | `implemented_pending_validation` | `optimization_wave_20260824p_runtime03_ecs_publish_matches_direct_store_recording` 与单次 store-update 源码守卫已落源；Runtime/Editor 静态合同批次 `1727/1727`（`4.095s`）通过，Rustfmt 与 scoped diff-check 通过。旧路径每次发布 58 次锁获取，当前为 1 次（25,000 次发布模型为 1,450,000 → 25,000，98.2759% 结构性下降）。受管 Windows Cargo、Release 计时与产品 p50/p95/p99 仍待异步验证。 |

本记录不宣称 ECS 诊断 schema、基数/字节预算、快照复制或产品帧性能已完成。
