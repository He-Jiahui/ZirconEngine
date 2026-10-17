---
status: design_recorded_acceptance_open
requirement: Editor02-P1-08
date: 2026-10-14
---
# Editor02 历史保留字节预算：归属与提交协议

Editor02 的 P1-08 仍未完成。原始评审 7.2、8.M2 与验收 8 要求数量、单文档字节和全局预算，保护 pinned/barrier，验证巨型命令、长会话及逐出后的 saved-top/dirty；原有 count 基线为 128 条；原文没有定义字节公式或默认字节额度。本记录由 gpt-6.1-sol max 核心审计形成，仅记录设计与缺口，没有新增 API、修改默认编辑行为或关闭性能门槛。

当前 HistoryStore 只限制记录数量。DeleteNode 在 apply 后接管 move-only 的 detached payload，包含 Any 组件与不透明的实体回调；Animation/Graph 的交换，以及 Navigation/Neural 的 inverse 捕获，也会改变 apply/undo/redo 各状态持有的数据。结构体大小、JSON 编码长度及 journal 的单条/文件上限，均不足以证明内存历史有界。未知成本记零会漏算；直接默认拒绝未知成本，会破坏现有合法编辑。

## 最小归属迁移

- Runtime 在组件尚未擦除为 Any 的注册点，由实际 owner 提供保守的 retained 上界。实体 observer 注册也要声明捕获数据上界；未声明是 Unaccounted。
- World 的 prepare 汇总 table、sparse、动态 JSON 和容器 owned capacity；detach/restore 转移同一成本声明。Editor 不猜测 Any 内部数据。
- Animation 覆盖旧/新 asset 可达状态的最大值；公开 GraphModel/Target 需给出 forward/inverse 的界，不能只针对测试中的 i32。
- Navigation 的 prepare 要约束旧状态及最大发布输出；Neural 在修改前 bounded-read 并保留真实旧输出。
- Engine 同时计入 command、label、participants、selection、容器与记账结构。生产组件、回调、内建命令和插件协议须完成兼容性迁移，才能统一启用默认预算。

## 事务协议

每个 command 先在有限 staging 配额内 prepare，再预留额度，之后才能 apply。根 scope 对多命令 checked-add；merge 保留保守的成本和，nested 转移额度。rollback/compensation/fault 仍持有的数据继续占额；commit 消费已有 reservation，不能在 apply 后首次因预算拒绝。

Undo/redo 沿用覆盖全生命周期的额度。仅回收目标 history 中允许截断的 redo 和已应用前缀；不自动逐出其他 history。active rollback/group、replay/recovery 及 pinned/barrier 保护的记录不提供回收额度。剩余额度不足时，当前 command 在 apply 前返回有类型的错误，不解除保护、不跨 barrier，也不丢弃已应用的 inverse。

记录在锁内移出，锁外 finalize 和 Drop 完成后再释放 retiring charge。数量/字节逐出必须保持 saved-top 可达性、generation 与 dirty 发布正确。Unaccounted、PinnedCapacity、PerHistory、Global 是拟议错误原因，尚无对应实现。

## 初始策略建议，尚未采用

建议保留数量 128；单 record 和单 history 的 committed 额度各 64MiB；engine committed 额度 256MiB，另设 active/preparation 64MiB。这样账面合计上界是 320MiB。它不是 engine RSS 或测得的峰值；若全局预算定义包含 staging，则必须从总额中划出 staging，不能同时声称总上界只有 256MiB。

128 条沿用原有明确的 count 基线。上述字节数是核心审计的工程初值，不是原计划字节值或 journal 限值，也没有加入现有冻结验收门槛。完整迁移至少涉及 Runtime ECS/observer/World、animation/navigation owner、Editor 内建与 generic/plugin command、engine 生命周期与真实 fixtures。本轮没有加入无生产 caller 的孤立预算 API。

## 待完成与证据

已有 tests::editing::transaction_engine::history:: 的 redo 截断、FIFO 逐出、保存点可达、分支 dirty 与 save token 失效用例可复用。缺口是新增字节/全局/未知成本/保护额度的证明，不能把现有语义测试说成不存在。已有 hash-index、snapshot 和 dirty-journal producer 分别测 lookup/projection/dedup，也不能替代内存预算验收。

- [x] 确认原始要求、实际保留数据归属和当前默认启用的兼容性问题。
- [x] 明确提交前预留、pinned/barrier、retiring charge 与 saved-top/dirty 的设计约束。
- [ ] 完成有版本的生产 owner/插件成本合同迁移和默认预算实现。
- [ ] 验证超大命令、全局/保护记录不足、merge/nested/失败恢复、undo/redo/clear 的真实额度守恒。
- [ ] 完成精确源码绑定的原生测试与巨型命令/长会话性能和内存验证。P1-08、原有性能门槛与总目标保持开放。

[原始评审](../02-document-transaction-save-autosave-recovery-review.md)、
[HistoryStore](../../../../../zircon_editor/src/core/editing/engine/history.rs)、
[EditCommand](../../../../../zircon_editor/src/core/editing/engine/command.rs)、
[生命周期](../../../../../zircon_editor/src/core/editing/engine/transaction/lifecycle.rs)、
[设计证据及原始字节哈希](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/editor02-history-retention-budget-design-evidence.json)。

本阶段没有编译、运行测试、创建源码副本或原生 fixture；实际释放空间 0B。保留最小设计、优化记录及完成列表供后续迁移与验收。
