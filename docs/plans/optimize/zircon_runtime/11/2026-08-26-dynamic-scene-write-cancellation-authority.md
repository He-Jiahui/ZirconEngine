# Runtime11 Dynamic Scene Write Cancellation Authority 架构与验证计划

> 日期：2026-08-26
> 所属 failure：`runtime/11/failure-2026-07-22-dynamic-scene-session-bounded-async-io.md`
> 状态：`source_implemented_static_passed_managed_validation_pending`

## 1. 当前源码结论

`RuntimeSessionArchiveWriter` 已把 archive 写入接入 Runtime11 `BoundedKeyedIoLane`，submission
也返回可等待 terminal 的 `BoundedKeyedIoTicket`。但是 writer 在 activation 前没有保存
`BoundedKeyedIoAdmission` 生成的 cancel authority，调用者只能等待，无法按 failure 合同显式撤销
尚未开始的写入。

这不是 active filesystem I/O 的抢占问题。Runtime11 当前共享合同只允许取消尚未开始、未被 global
fence 固定的 work；一旦 worker 已经 linearize `started`，写入必须走到真实 filesystem terminal，外围
publication 不能伪造取消成功。

## 2. Unreal Engine 对照与本仓边界

主要参考 Unreal Engine
`dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Async/AsyncWork.h` 的 `FAsyncTaskBase::Cancel`：

- 首先通过 thread pool `RetractQueuedWork` 撤回尚未执行的任务；
- 撤回成功后完成任务 bookkeeping，调用者可以安全处理该 terminal；
- work 已经开始时，`Cancel` 不伪造“已经撤回”的结果，而只允许 task 自身接收 cancel 标记；
- shutdown abandon 与调用者 cancel 是不同所有权路径。

Zircon 不复制 UE 的裸 task lifetime 或 active-task cancel flag。Runtime11 已有更严格的 typed
`WrongAuthority / AlreadyStarted / FencePinned` 结果，因此 archive writer 应保存 lane 发放的 capability，
而不是暴露可由任意 ticket observer 构造或借用的取消权。

## 3. 目标所有权与状态机

`RuntimeSessionArchiveWriteSubmission` 是一次提交的 mutation owner，私有持有：

- terminal `BoundedKeyedIoTicket`；
- 与该 ticket ID 绑定的 `BoundedKeyedIoCancelAuthority`；
- 只有 worker 真正执行 filesystem write 时才产生的 domain outcome。

提交顺序保持：

1. path intent reservation；
2. lane `try_admit`；
3. 保存 admission 的 cancel authority；
4. path intent admit；
5. 发布 worker path ticket；
6. lane `activate`；
7. 返回同时拥有 ticket 与 capability 的 submission。

`cancel_before_start` 只委托给同一 ticket/capability 对。成功后共享 terminal 唯一为
`CancelledBeforeStart`，重复调用幂等；若已经开始或被 fence 固定，原样返回 Runtime11 typed error。
pre-start cancel 不运行 filesystem closure，因此 domain `take_outcome()` 保持 `None`；调用者必须以 ticket
terminal 判断“未执行”，不能把缺失 filesystem outcome 解释为成功。

## 4. 复杂度、资源与性能假设

| 路径 | 变化前 | 变化后 |
|---|---:|---:|
| submission 常驻状态 | ticket + outcome | ticket + cancel capability + outcome |
| cancel check | 无 domain API | 单 ticket mutex，`O(1)` |
| queued cancellation | 只能等待 | before-start terminal，`O(1)` |
| active filesystem I/O | 不可抢占 | 不变 |
| terminal truth | Runtime11 ticket | 不变 |

新增成本为 submission 中一个小型 ticket ID capability；不新增线程、channel、轮询、map 或 domain terminal
状态。它收敛的是所有权和可控 shutdown 前置能力，不宣称吞吐、CPU、RSS 或功耗改善。

在继续做 active cooperative cancellation 或 queue 算法优化前，必须先用 Runtime11 diagnostics 与 Tracy
采集：queued age P50/P95/P99、cancel success/AlreadyStarted/FencePinned 计数、cancel-to-terminal latency、
same-path supersede rate、write service latency、pending bytes、worker utilization、CPU、RSS 和平台功耗。
只有 active write 明确占据 shutdown 尾延迟后，才评估 chunk boundary cancellation 和 safe commit fence；
不能在 atomic rename 临界区引入不可恢复的半提交状态。

## 5. 确定性验证计划

回归不使用 sleep：

- 单 I/O worker 先被 channel gate 占用；
- 提交一项 archive write，确保 work 只能处于 queued/before-start；
- submission 调用 `cancel_before_start` 两次，均须成功；
- ticket 必须报告 `CancelledBeforeStart`，filesystem outcome 必须为 `None`；
- 释放 worker 并 drain shutdown 后，目标文件必须不存在；
- fixture 仅位于仓库当前盘 `.codex/tmp`，不向 C 盘写产物。

静态阶段执行 scoped `rustfmt --check`、源码契约断言、owned trailing whitespace 与 diff check。受管
Cargo、slow-I/O、burst、CPU、RSS 与功耗矩阵保持 pending；未取得真实回执前不关闭 failure、不提交
milestone commit、不发送手工企微。

## 6. 本切片完成定义

- archive submission 持有但不泄漏唯一 cancel capability；
- `cancel_before_start` 完整保留 Runtime11 typed 结果与幂等语义；
- terminal 继续由 Runtime11 ticket 单点拥有，不复制状态机；
- 确定性回归证明取消的 queued write 不触碰 filesystem；
- failure 只记录源码与静态证据，不冒充受管测试或性能数据。

## 7. 2026-08-26 源码验证结果

- `RuntimeSessionArchiveWriteSubmission` 已私有持有 `BoundedKeyedIoCancelAuthority`，并提供 typed
  `cancel_before_start`；authority 在 path admit 与 lane activate 前取得，失败回滚仍由 armed admission
  Drop 负责；
- 确定性 cancellation Rust 回归已挂载，覆盖重复取消、`CancelledBeforeStart`、filesystem outcome
  保持 `None` 以及目标文件不存在；
- scoped `rustfmt --check`：2/2 Rust 文件通过；
- capability/state-machine 源码断言：9/9 通过；
- 3 个本切片 owned 路径 trailing whitespace：0；
- tracked owned diff check：通过，仅有 Git 的 LF/CRLF 工作区提示；
- Rust 文件规模：writer 178 行，私有 tests 136 行；没有接近 800 行 production soft limit；
- 受管 Cargo 回归、slow-I/O、burst、CPU、RSS 与功耗样本：0，保持 pending。

本切片把 archive submission 的显式 queued cancellation 从“capability 丢失、无法调用”收敛为一次
`O(1)` typed operation；没有改变 active filesystem I/O 的不可抢占边界，也不据此声明性能瓶颈消失。
