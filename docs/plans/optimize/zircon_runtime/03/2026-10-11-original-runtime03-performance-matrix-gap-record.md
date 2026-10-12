---
related_code:
  - zircon_runtime/tests/runtime_profiling_recorder_performance.rs
  - zircon_runtime/tests/runtime03_nested_scope_allocation_profile.rs
  - zircon_runtime/src/core/runtime/diagnostics/store.rs
  - zircon_runtime/src/core/runtime/diagnostics/tests/store.rs
  - zircon_runtime/src/core/runtime/diagnostics/tests/devtools.rs
  - zircon_runtime/src/runtime_diagnostics/collect.rs
  - zircon_runtime/src/runtime_diagnostics/tests/collect_performance_tests.rs
  - zircon_runtime/src/foundation/runtime/config_manager/state/tests/percentile_stack_tests.rs
  - zircon_runtime_interface/src/profiling.rs
  - zircon_runtime/src/core/runtime/diagnostics/profiling/recorder.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md
doc_type: performance-gap-inventory
title: Runtime03 original performance matrix gap record
date: 2026-10-11
implementation_status: static_inventory
validation_status: native_and_performance_pending
performance_status: original_matrix_not_run
---

# Runtime03 原性能矩阵差距记录

日期：2026-10-11。对应[原 Runtime03 计划](../03-core-runtime-diagnostics-profiling-config-review.md) §9.4。本文是静态源码盘点：没有运行 Cargo、Rust/native 测试、Release producer 或产品性能采样；以下不构成任何通过证据。

## 原矩阵与当前 producer

| 范围 | 原计划维度与指标 | 当前最接近的 producer | 静态结论 |
|---|---|---|---|
| Profiling scope | inactive/active；threads 1/8/64；events/frame 0/100/10K；ns/event、allocations、lock wait、drop rate、retained bytes、workload disturbance | [`active_profile_recorder_reports_multithread_scaling_and_allocation_cost`](../../../../../zircon_runtime/tests/runtime_profiling_recorder_performance.rs) 在 active capture 下遍历三个 thread 值与三个事件值，做 5 个样本，打印 elapsed 与 allocations | 覆盖 active 维度的形状，但没有 inactive 对照、lock wait/drop/workload disturbance/RSS 指标，也没有 latency/allocation 数值门槛。它另有 retention 等值断言，见下文 cap 冲突。 |
| Diagnostic store | series 1/541/10K；history 1/64/1K；consumer 0/30/60/120 Hz；frame p50/p95/p99 与 writer/reader CPU | [`optimization_wave_20260824b_runtime03_history_ring_capacity_evidence`](../../../../../zircon_runtime/src/core/runtime/diagnostics/tests/store.rs) 使用 4,096 次单独构造的 `DiagnosticSeries`、history 64、每条 128 writes；[`optimization_wave_20260824e_runtime03_devtools_tag_projection_evidence`](../../../../../zircon_runtime/src/core/runtime/diagnostics/tests/devtools.rs) 对合成的 100,000 条 snapshot 投影 tags | 没有覆盖实际 `DiagnosticStore` 的 series/history 组合、并发读写或 consumer cadence。环形容量 producer 测单条 series 的容量节省与总耗时；devtools producer 测 tags helper，均不是 store matrix。 |
| Config persistence | snapshots 1B/1MiB/100MiB；key burst 1/1K；serialize/write/replace latency、coalescing、shutdown deadline | [`animation_playback_settings_use_config_manager_persistence`](../../../../../zircon_runtime/src/foundation/tests/cases.rs) 写入并 reload 一个设置；[`optimization_batch_20260826gq_runtime237_config_percentile_stack_buffer_bench`](../../../../../zircon_runtime/src/foundation/runtime/config_manager/state/tests/percentile_stack_tests.rs) 对 64 个 flush latency 样本重复计算 percentile，并要求优化版 p95 不超过 legacy 的 70% | 已有 caller 持久化正确性与 percentile helper 的局部门槛，没有端到端 snapshot 大小、burst、文件 replace 或 shutdown 矩阵。 |

生产采集入口 [`collect_runtime_diagnostic_store` / `collect_runtime_diagnostic_current_store`](../../../../../zircon_runtime/src/runtime_diagnostics/collect.rs) 已存在；对应的 [`collect_performance_tests`](../../../../../zircon_runtime/src/runtime_diagnostics/tests/collect_performance_tests.rs) 核对 current/full snapshot 与 history 语义，但没有按原计划创建 1/541/10K store、1/64/1K history 或 0/30/60/120 Hz consumer workload。physics 的 `fixed_hz=60/120` 是被采集的 domain 值，不是 snapshot consumer 频率。

其他相邻 probe 也不补齐产品规模：[`runtime03_nested_scope_allocation_profile`](../../../../../zircon_runtime/tests/runtime03_nested_scope_allocation_profile.rs) 每个样本 128 个 leaves，单线程测 nested scope 分配路径；devtools probe 的 100,000 series 是合成 snapshot 输入。两者都不等于原计划的 10K 实际诊断 store 或产品级 100K workload。配置 percentile probe 的 16,384 次重复也只是 64-sample percentile helper，不是配置写入 burst。

## 静态识别的 producer 失败风险

[`runtime_profiling_recorder_performance.rs`](../../../../../zircon_runtime/tests/runtime_profiling_recorder_performance.rs) 把 `max_spans` 请求设为 `event_count.max(1)`，并在每个 case 后断言 `snapshot().spans.len() == event_count`。最大 case 仍会执行完整的 64 × 10,000 = 640,000 次 scope workload。配置归一化在 [`ProfileCaptureConfig::normalized`](../../../../../zircon_runtime_interface/src/profiling.rs) 将 `max_spans` 限制到 `PROFILE_CAPTURE_MAX_SPANS = 131_072`；[`ProfileRecorder::start_capture` 与 `record_span`](../../../../../zircon_runtime/src/core/runtime/diagnostics/profiling/recorder.rs) 使用该归一化容量并在环形队列满后覆盖旧样本。

因此源码推导该最大 case 在全部写入后会有 640,000 written、508,928 overwritten、131,072 retained，而现有断言要求 retained 为 640,000。此风险尚未通过运行确认。保留的 131,072 个样本反映 retention cap，不表示 workload 从 640,000 次写入缩小。Core Wake 正在处理 cap-aware 原断言；修复的源码 review 与采用状态仍 pending，本记录不宣称已修复。

当前 recorder producer 还把五个排序样本的末项标作 `p95`，即该小样本集的最大值；代码输出 elapsed 与 allocation 数值，但没有原计划要求的统一性能预算。§9.4 给出了矩阵和报告指标，未列出绝对 latency/allocation 限值。不能据静态 coverage 或局部 helper 门槛宣称性能达标。

## 可直接落地的后续工作

1. 完成 Core Wake 的单 producer cap-aware 断言修复并 review：保留完整 64 × 10K scope workload，按 recorder retention 语义检查 written/overwritten/retained；不得通过降低负载或提升 hard cap 消除冲突。
2. 增加真实 `DiagnosticStore` Release producer，覆盖 series 1/541/10K、history 1/64/1K 与 consumer 0/30/60/120 Hz；记录 writer/reader CPU 和每帧 p50/p95/p99，并保留原始样本、source fingerprint 与运行环境。现有单条 series 和 tag projection helper 继续只作为局部 microbench。
3. 增加实际 `ConfigManager` persistence producer，覆盖 1B/1MiB/100MiB snapshot 与 1/1K key burst，并测 serialize/write/replace、coalescing 和 shutdown flush。先记录可复现的基线与原始样本，再由计划 owner 确认产品阈值，不从局部 70% helper 门槛外推。
4. 按计划在 Windows Release 产品环境收集 source fingerprint、CPU/GPU/driver、profile、raw samples 与 noise interval；在这些材料 review 前，Runtime03 产品性能验收保持 pending。

## 2026-10-11 状态增量：原 producer cap-aware 修复已采用

上文“修复仍 pending”准确记录首次静态盘点时的状态。随后该单一 producer 修复已由 Root 独立 review 并实际采用，详见[新修复记录](2026-10-11-original-producer-normalized-capacity-repair-record.md)、[实际采用回执](../../../../../.codex/outbox/runtime03-original-producer-normalized-capacity-root-adoption-v1/actual-apply/actual-canonical-apply.json) SHA256 `8d9f746b596de0372982399c49c6780da98d786deb955265a05b4b03656c5bfb` 与[独立 review](../../../../../.codex/outbox/runtime03-original-producer-normalized-capacity-root-adoption-v1/review.json) SHA256 `1efb9c22641be1812fd2016df8ba1dad63530bb0f53b9d1a2f705ef86708dae3`。采用后的 [`runtime_profiling_recorder_performance.rs`](../../../../../zircon_runtime/tests/runtime_profiling_recorder_performance.rs) raw SHA256 为 `5b3f4d8d55c778039af601bd6cbbb4773c6bde12aeb589ba986e3bed992d28b1`，LF SHA256 为 `daad399c0962fe3166c8b715f867e70b06b6549ebb2408ec88edf03c3636599f`。

该增量只关闭原 producer 的静态容量断言冲突，不改变首次盘点中其余矩阵差距：active-only profiling 仍缺 inactive 对照及部分指标/性能门槛；实际 DiagnosticStore series/history/consumer 与 Config snapshot/burst 产品矩阵仍缺；原生 Release 执行和性能证据仍未运行。cap-aware 修复已 review/采用不等于测试、native 或 Runtime03 产品性能验收通过。

后续 V11 命令批次仍按原 15 arms / 14 producers 规划，只允许此 producer 更新 source hash 并绑定上述实际采用；筛选规模下限按 #2 至少 22、#15 至少 191 规划。该 successor 仍是待执行计划，不是执行收据或通过事实。
