---
related_code:
  - zircon_runtime/tests/runtime_profiling_recorder_performance.rs
  - zircon_runtime_interface/src/profiling.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/recorder.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
  - docs/plans/optimize/zircon_runtime/03/2026-10-11-original-runtime03-performance-matrix-gap-record.md
doc_type: source-repair-record
title: Runtime03 original producer normalized capacity repair
date: 2026-10-11
implementation_status: actual_source_adopted
validation_status: native_and_performance_pending
performance_status: release_producer_not_run
---

# Runtime03 原 producer normalized-capacity 修复记录

日期：2026-10-11。只记录原多线程 profiling producer 的 cap-aware 测量与断言修复。Root 已独立 review 并采用源码；没有运行 Cargo、native 测试或 Release/performance producer。该源码修复不是性能通过或 Runtime03 完成验收。

## 修复范围与保留合同

原 ignored 测试 [`active_profile_recorder_reports_multithread_scaling_and_allocation_cost`](../../../../../zircon_runtime/tests/runtime_profiling_recorder_performance.rs) 保留原 `#[ignore]`、测试名与完整 workload：threads `1/8/64` × events/thread `0/100/10,000` × 5 samples，共 45 个样本。最大 case 仍执行 64 × 10,000 = 640,000 scope 写入；recorder hard capacity 仍为 131,072，没有通过削减 workload 或提高 cap 改变负载。

现有 `RUNTIME07_PROFILE_RECORDER_BASELINE_V1` 输出 key 前缀、profile feature、Release profile 与 worker/allocation 测量窗口均保持。实现增补 requested/effective span capacity、written、overwritten 与既有 retained 字段；配置请求是 `event_count.max(1)`，effective capacity 从 normalized config 读取。每个样本在测量窗口外使用 retention snapshot 验证 capacity、written、retained、snapshot span 数与 overwritten。

最大 case 的容量语义为：

```text
written 640,000 = retained 131,072 + overwritten 508,928
```

此等式描述 ring 的写入与留存守恒；retained 131,072 不代表 workload 只执行了 131,072 次写入。

五个样本的 `allocation_count` 与 `allocated_bytes` 各自复制到独立数组并排序，分别取各自中位数，不再借用 elapsed 排序后 p50 样本的 allocation 值。数组复制、排序及新增 retention 断言均位于测量窗口之外。既有 elapsed 报告名义上的 `p95` 仍取 5 个样本最大值；原计划 §9.4 所需性能阈值、inactive 对照和其它 profiling 指标没有由此补齐。

## 实际采用与审查证据

- [实际 canonical apply](../../../../../.codex/outbox/runtime03-original-producer-normalized-capacity-root-adoption-v1/actual-apply/actual-canonical-apply.json)：SHA256 `8d9f746b596de0372982399c49c6780da98d786deb955265a05b4b03656c5bfb`；记录单一受管源码路径采用，未执行 Cargo/native/performance/Jenkins。
- [Root independent review](../../../../../.codex/outbox/runtime03-original-producer-normalized-capacity-root-adoption-v1/review.json)：SHA256 `1efb9c22641be1812fd2016df8ba1dad63530bb0f53b9d1a2f705ef86708dae3`；source-only review 接受，0 blocking findings。
- 采用后的 [`runtime_profiling_recorder_performance.rs`](../../../../../zircon_runtime/tests/runtime_profiling_recorder_performance.rs)：raw SHA256 `5b3f4d8d55c778039af601bd6cbbb4773c6bde12aeb589ba986e3bed992d28b1`；LF SHA256 `daad399c0962fe3166c8b715f867e70b06b6549ebb2408ec88edf03c3636599f`。

## 后续验证与范围边界

源码 review 接受的是 producer 断言与输出语义，不是采样结果。以下仍 pending：

1. 运行原 Release producer 的全部 45 个样本，检查 requested/effective/written/overwritten/retained 和独立 allocation medians 的实际输出。
2. review 原始样本、源指纹、硬件与环境，并由计划 owner 确定缺失的产品性能阈值。
3. 补齐实际 DiagnosticStore series/history/consumer 矩阵和 Config snapshot/burst 矩阵；producer cap 修复不覆盖这些差距。
4. 按 Runtime03 计划完成 Windows Release 产品矩阵及 native/performance/product acceptance。

V11 后续批次仍为 15 arms / 14 producers；只对此 producer 更新 source hash 并绑定实际采用记录。候选筛选规模下限计划为 #2 至少 22、#15 至少 191。该 successor 未执行，本记录不宣称其验证通过。
