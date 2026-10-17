---
status: design_recorded_acceptance_open
requirement: Editor02-P1-08
date: 2026-10-14
---
# Editor02 历史保留字节预算完成列表

本次仅完成归属与提交协议审计。P1-08 的字节/全局预算实现和验收均未完成，现有合法编辑行为保持原状。

- [x] 确认原计划要求 count、单文档 bytes、global budget、pinned/barrier 和 saved-top/dirty；原有 count 基线为 128；原文未给字节公式或默认字节额度。
- [x] 确认 Any 组件、回调捕获和可变 inverse 缺少统一成本合同；不能按零、sizeof 或 journal 长度宣称有界。
- [x] 记录 Runtime owner 成本迁移、提交前 reservation、保护记录拒绝和锁外 finalize/Drop 后释放额度的设计。
- [x] 保留原有 128 条 count 基线；将 64MiB record/history、256MiB committed 加 64MiB staging 标为未采用的字节工程初值；账面合计 320MiB，不声称 RSS。
- [x] 确认已有 transaction_engine history 语义用例可复用；lookup/projection/dedup producer 不代表字节预算完成。
- [ ] 完成生产组件/observer/World、内建及 generic/plugin command 的有版本成本合同。
- [ ] 实现默认预算并保留合法编辑兼容性；超限必须在当前 apply 前拒绝。
- [ ] 通过真实 merge/nested/rollback/fault、undo/redo/truncation/clear 和 saved-top/dirty 回归。
- [ ] 通过巨型命令和长会话的原生、性能、内存与暂态额度验证。
- [ ] 关闭 P1-08 和总目标验收。

[优化设计记录](../../../optimize/zircon_editor/02/2026-10-14-history-retention-budget-contract-design-record.md)、
[设计证据](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/editor02-history-retention-budget-design-evidence.json)。

核心决策：gpt-6.1-sol max。本阶段无源代码变更、编译或性能测量；释放 0B，保留必要设计与待验收记录。
