# Runtime72 批量生命周期事务守卫架构与验证计划

> 日期：2026-08-26
> 所属审查：`72-runtime-core-lifecycle-registry-concurrency-shutdown-current-source-review.md`
> 覆盖问题：`RCL-P0-004`、`RCL-G08`、`RCL-G09`
> 状态：`source_implemented_static_passed_managed_validation_pending`

## 1. 范围与结论

本项只修复 `CoreHandle::activate_registered_modules_with_ready_timeout` 的 owner token
所有权漏洞，不调整 lifecycle coordinator 的 waiter 算法、状态枚举、公共 API 或锁拓扑。

当前批量激活按 frozen graph 顺序取得 module lifecycle token。若前 `k` 个 module 已取得
`Owner(token)`，第 `k + 1` 个 module 的 acquire 直接返回错误，循环中的 `?` 会提前退出，
但前 `k` 个 token 没有进入 `complete_module_lifecycle_transition`。这些 module 将永久保留
`InFlight` 记录；同线程后续操作得到 reentrant 错误，其他线程则可能永久等待。

一次失败最多泄漏 `V - 1` 个 token，其中 `V` 是 frozen graph 中的 module 数。该问题是
生命周期事务正确性缺陷，不需要性能假设即可修复。

## 2. 当前失败序列

确定性复现不依赖 sleep 或线程调度：

1. frozen graph 的批量顺序为独立 module `A`、module `B`；
2. 单独激活 `B`，使当前线程先持有 `B` 的 activate token；
3. `B.build()` 在同一线程重入 `activate_registered_modules()`；
4. 内层 batch 成功取得 `A` 的 token；
5. 内层 batch 取得 `B` 时返回 `ModuleLifecycleCommandReentrant`；
6. 旧实现经 `?` 返回，遗漏 `A` 的完成；
7. 外层 `B` 正常完成后，单独激活 `A` 仍错误地报告 reentrant。

## 3. 参考引擎与 Zircon 约束

主要参考：

- Unreal Engine `FModuleManager::AddModuleToModulesList` 在
  `dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Modules/ModuleManager.cpp` 中先在
  `ModulesCriticalSection` 下提交 map，再在锁外广播 `ModulesChangedEvent`；
- `LoadModuleWithFailureReason` 先建立可发现但未 ready 的 module record，成功启动后才广播
  `ModuleLoaded`。这支持“状态所有权提交”和“外部可见发布”分阶段，而不是让半完成记录静默遗留。

Zircon 不复制 UE 的 process-global singleton。事务集合由当前 `CoreHandle` 私有持有，并继续
通过现有 `LifecycleCoordinator` 完成 token；callback、module build 和通知仍不在 coordinator
mutex 内执行。

## 4. 目标结构

在 `handle/activation/batch/transaction_set.rs` 中增加 batch 私有
`LifecycleTransactionSet<'a>`：

- 构造时借用 `CoreHandle`，按 graph cardinality 预留 token 容量；
- 每个 `Owner(token)` 立即登记，形成单一所有权集合；
- 任意显式 `Result` 通过 `finish(result)` 完成全部 token；
- 未调用 `finish` 即发生 panic/drop 时，`Drop` 以 typed callback-panic 结果中止全部 token；
- 完成后清空集合，使 `Drop` 幂等且不会重复提交；
- 不公开 token、不增加跨模块抽象、不改变 coordinator record 语义。

批量 API 对 acquire 的 `Err`、`Completed(Err)`、意外 `Wait` 和 owned activation 的最终结果
统一走 `finish`。因此新增 return 分支不再需要人工记住清理前序 token。

## 5. 锁、复杂度与性能判断

本轮保留当前完成路径：每个 token 调用一次
`complete_module_lifecycle_transition`，其内部短暂取得 coordinator mutex，提交 record，锁外
`notify_all`。

| 项目 | 修复前 | 修复后 |
|---|---:|---:|
| acquire 遍历 | `O(V)` | `O(V)` |
| owner token 存储 | `O(V)` | `O(V)` |
| 正常完成 | `O(K)` 次 coordinator 提交 | `O(K)` 次 coordinator 提交 |
| acquire 错误后的遗留 token | 最多 `V - 1` | `0` |
| panic/drop 后的遗留 token | 最多 `V` | `0` |

这里的 `K <= V` 是本次 batch 实际取得的 owner token 数。新增守卫没有额外渐进复杂度，
只增加一个空集合检查和显式清空。逐 token mutex/notify 成本可能是后续性能问题，但在没有
module-cardinality、waiter-count、lock-wait 与 CPU/功耗动态基线前，不把它改成批量 coordinator
提交，也不宣称性能收益。

后续若优化 completion，应先记录 `V/K` 分布、P50/P95/P99、coordinator lock wait、notify
唤醒数、CPU time、allocation 与 RSS；只有证实逐 token 完成是瓶颈，才设计一次锁内批量提交、
一次锁外通知，并用 waiter 线性化测试证明语义等价。

## 6. 验证计划

本轮非 Cargo 可执行验证：

- 确定性 Rust 行为测试：`B.build -> batch -> A owner -> B reentrant` 后，`A` 可再次激活；
- scoped `rustfmt --check`；
- owned path `git diff --check` 与 trailing-whitespace 检查；
- 静态确认 batch 的所有退出统一经过 transaction set。

受管 Cargo compile/test 与动态 profile 仍由 coordinator validation 执行；在 receipt 终态前，
本项状态不得写成 `accepted`，不得提交 milestone commit，也不得发送手工企微消息。

### 6.1 2026-08-26 源码验证结果

- scoped `rustfmt --check`：4/4 Rust 文件通过；
- transaction-set 源码结构断言：8/8 通过；
- 5 个 owned 路径 trailing whitespace：0；
- tracked owned diff `git diff --check`：通过，仅有 Git 的 LF/CRLF 工作区提示；
- 新增确定性 Rust 行为测试：1 项已挂载，尚未取得受管 Cargo 执行回执；
- 动态性能/功耗样本：0，本项没有性能改善声明。

当前实现从所有权结构上把 acquire 错误和 panic/drop 的遗留 token 上限由 `V - 1`/`V`
收敛为 0；该结论仍需上述 Rust 行为测试和原有 lifecycle 测试集的受管执行确认。

## 7. 完成定义

- `RCL-G08` 的 batch token set 在显式 return、panic 和 drop 路径都完成或中止全部已取得 token；
- `RCL-G09` 的 callback 重入返回 typed error，随后所有涉及 module 仍可继续 lifecycle 操作；
- 不编辑当前其他会话持有的 `activation.rs`、`core_runtime_state.rs`、`error.rs`；
- 静态验证通过，受管 Cargo 验证状态被如实记录。
