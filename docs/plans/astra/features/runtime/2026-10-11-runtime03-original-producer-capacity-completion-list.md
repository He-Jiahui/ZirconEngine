# Runtime03 原 producer 容量修复完成列表（2026-10-11）

对应[实际源码修复记录](../../../optimize/zircon_runtime/03/2026-10-11-original-producer-normalized-capacity-repair-record.md)与[原 Runtime03 矩阵盘点](../../../optimize/zircon_runtime/03/2026-10-11-original-runtime03-performance-matrix-gap-record.md)。本列表只关闭单一 producer 的源码容量语义修正，不表示测试或性能验收通过。

- [x] 保留原 ignored test 名称、完整 `1/8/64 × 0/100/10K × 5`、45 样本、640,000 最大 scope workload、131,072 hard cap、baseline key 前缀与 profile 配置。
- [x] 为每个样本增加 normalized requested/effective/written/overwritten/retained 容量报告与窗口外守恒断言；最大 case 期望 `640000 = 131072 + 508928`。
- [x] allocation count 与 allocated bytes 各自对五个样本单独取中位数；新增复制、排序和断言均不计入原 workload 测量窗口。
- [x] Root 对实际 canonical source 做独立 review 并采用；见修复记录所引 apply 与 review receipt。
- [ ] 执行原 Release producer 并核对全部 45 个实际样本及 cap 守恒结果。
- [ ] 完成原计划要求的性能数据审查和数值门槛确认。
- [ ] 补齐原 Runtime03 DiagnosticStore、Config 与 Windows Release 产品矩阵，完成 native/performance/product acceptance。

V11 successor 计划保留 15 arms / 14 producers，只更新并绑定此 producer 的实际 source hash；筛选规模下限为 #2 至少 22、#15 至少 191。后续批次未执行。不能把源码修复标记为完整 Runtime03 验收。
