# Runtime03 原性能矩阵盘点完成列表（2026-10-11）

对应[原性能差距记录](../../../optimize/zircon_runtime/03/2026-10-11-original-runtime03-performance-matrix-gap-record.md)与[Runtime03 原计划](../../../optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md)。本列表记录静态源码盘点，不表示功能、native 或性能验收完成。

- [x] 核对原计划 §9.4 的 profiling、DiagnosticStore 与 Config persistence 规模及报告指标。
- [x] 确认现有 profiling Release producer 只覆盖 active 1/8/64 threads × 0/100/10K events 形状；inactive 对照和原计划部分指标缺失。
- [x] 确认 diagnostic store 的实际 series/history/consumer 组合矩阵缺失；现有 4,096 个独立 series 容量 probe 与合成 100,000 条 devtools tag projection 不覆盖实际 10K store。
- [x] 确认 ConfigManager 的 snapshot 1B/1MiB/100MiB 与 1/1K key burst producer 缺失；64-sample percentile helper 不代表 config persistence workload。
- [x] 记录 producer 的静态 cap 冲突：最大 case 保留完整 640,000 scope 写入，当前 hard cap 是 131,072，现断言仍要求全部 retained；Core Wake cap-aware 修复与 review/采用 pending。
- [ ] Review 并采用 cap-aware producer 修复，保持 64 × 10K workload 不变。
- [ ] 实现并运行真实 DiagnosticStore 矩阵 producer，提交原始样本与 source/environment fingerprint。
- [ ] 实现并运行 ConfigManager snapshot/burst persistence producer，覆盖 serialize/write/replace、coalescing 与 shutdown flush。
- [ ] 在 Windows Release 产品条件下完成原计划要求的矩阵和数值验收；native/performance/product acceptance 当前 pending。

盘点中发现的合成 100,000 条 tag snapshot、每样本 128 个 nested leaves，以及其他小型 helper 数量均未作为原产品规模或通过证据。本文没有运行 Cargo、native 测试或性能 producer。

## 后续状态增量：原 producer 容量修复已采用

上面的未完成项记录首次静态盘点的状态。之后原 producer 的 cap-aware 修复已通过 Root 独立 review 并实际采用；具体范围与回执见[容量修复记录](../../../optimize/zircon_runtime/03/2026-10-11-original-producer-normalized-capacity-repair-record.md)。该状态仅覆盖一个 producer 的源码修正。

- [x] 实际采用 producer cap-aware 修复，保留 64 × 10K workload、131,072 hard cap、原测试名/ignore、45 个样本、`RUNTIME07_PROFILE_RECORDER_BASELINE_V1` key 前缀与 profile 配置；逐样本加入 written/overwritten/retained 守恒核对。
- [ ] 执行原 Release producer 的 45 个 samples，review requested/effective/written/overwritten 数据及 allocation medians。
- [ ] 运行并验收原生与产品性能矩阵；未执行，不得据源码采用勾选完整 Runtime03 验收。
- [ ] 补齐并验收真实 DiagnosticStore series/history/consumer 与 Config snapshot/burst 矩阵。

V11 后续批次仍计划为 15 arms / 14 producers，仅此 producer 绑定新的实际 source hash；筛选下限为 #2 至少 22、#15 至少 191。此批次仍未执行。完整 Runtime03 native/performance/product acceptance 保持 pending。
