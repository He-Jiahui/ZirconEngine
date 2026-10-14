# Runtime03 绝对 P95 预算计划完成列表

状态：预算定义完成；Native 与性能验收待回执。用户已授权由 agent 联网研究并给定数值，无待用户填写的预算数值。

- [x] 已采用 [预算记录](../../../optimize/zircon_runtime/03/2026-10-13-config-persistence-absolute-p95-budget-record.md)的 12 格上限：1 B × 1 为 flush/关闭 50/75 ms，1 B × 1000 为 75/100 ms，1 MiB × 1 为 75/100 ms，1 MiB × 1000 为 100/125 ms，100 MiB × 1 为 1500/1600 ms，100 MiB × 1000 为 1750/1850 ms。
- [x] 同 cap 约束 derived burst + completion 与 API completion；完整计时边界和排除项明确，未称为连续完整 session wall-clock。
- [x] 每格各 n=5、nearest-rank P95=max、整数纳秒判断；60 原始样本、原 producer、原 marker/字段和所有历史门保留。
- [x] 已写入 [机器契约](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime03-absolute-p95-criteria.json)，工程假设与外部历史基准、实际测量区分明确。
- [ ] 随下一批 Runtime/Editor 修复交接，保持 batch09、各 attempt/source/raw 身份和 receipt lifecycle。
- [ ] Native exit0、60 行完整且守恒、磁盘 JSON、background writer、worker 与 task-graph real join 全部通过。
- [ ] 固定 Windows Release/local SSD NTFS 环境回执绑定完整输入、实际路径和原始样本，逐格两指标 P95 不超 cap。
- [ ] 原 helper 比例、资源、容量和矩阵门全部通过；保持失败样本，不事后放宽预算。
- [ ] Runtime03 产品、Jenkins 与 milestone 验收完成。

未记录实际时延或性能通过；5 样本观测门不作为稳定总体 P95 或覆盖/置信保证。
