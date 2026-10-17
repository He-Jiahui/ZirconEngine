# Editor182：已观测 Scene 版本保存完成列表

日期：2026-10-15。任务：`optimize-20261013-01a13451`。对应 `ED182-P0-01 / ED182-M0`；[优化记录及完整证据](E:/Git/ZirconEngine/docs/plans/optimize/zircon_editor/182/2026-10-15-observed-scene-save-revision-record.md)。

- [x] open/startup/reload 的 Scene 解析与 source revision 来自同一个字节快照；不以缓存 artifact 配合稍后观察的 baseline。
- [x] 普通 Scene 保存沿现有 generation guard、owner lock、journal 传递 mandatory expected revision。
- [x] Missing 使用 create-only 发布；receipt 来自 staged 文件对象；普通无条件写入不增加身份/摘要工作。
- [x] 一次性 `OverwriteLease` 绑定提示观察的版本、文档与 activation；过期决策被拒绝，保留外部内容及 dirty/checkpoint。
- [x] 仅 Durable Persisted 发布 Saved、推进 clean checkpoint；ProjectionFailed 保留 typed receipt/磁盘 baseline 与 dirty；RecoveryDeferred 保留旧 baseline。
- [x] 69 文件源码冻结与增量正反向 raw 证明：60 份原始 before、9 份原始不存在状态、66 changed、3 unchanged，保留原换行与 index。
- [x] scoped guard actual exit 0、findings 为空；独立 `gpt-6-luna max` 复核 `PASS_static_only`，失败和 UNKNOWN 证据保留。
- [x] 登记 20 个新增源码病例及原病例库存：18 Windows、2 纯逻辑；尚非 native passing floor。
- [ ] 通过统一 Jenkins 批次的真实编译、cfg/test listing、全部新增与受影响原有测试。
- [ ] 验证 Windows metadata witness、完整文件身份、`ReplaceFileW`、真实 symlink/reparse 与发布故障矩阵。
- [ ] 关闭非协作外部 writer 的 check-to-replace、namespace substitution 与 recovery 条件发布竞态。
- [ ] 满足原有性能、样本、RSS、Runtime03 绝对 P95 和产品验收。

`ED182-P0-01`、完整跨进程 CAS、native、performance 和产品验收仍为 **OPEN**。不得把本列表中的源码/static 完成项当作 P0 关闭或整体任务完成。

核心证据：[binding](E:/Git/ZirconEngine/.codex/outbox/reports/optimize-20261013-01a13451/editor182-observed-scene-save-revision-v1-source-binding.json)、[patch](E:/Git/ZirconEngine/.codex/outbox/reports/optimize-20261013-01a13451/editor182-observed-scene-save-revision-v1.patch)、[root static](E:/Git/ZirconEngine/.codex/outbox/reports/optimize-20261013-01a13451/editor182-observed-scene-save-revision-v1-static.json)、[产物清单](E:/Git/ZirconEngine/.codex/outbox/reports/optimize-20261013-01a13451/artifacts.json)。

原生验证由后续合批承担，继续独立源码工作，不等待或持续查询协调器状态。当前无本切片 native 产物/夹具待清理，实际释放 0B；保留证据到验收结束，未知历史归属不宣称已关闭。
