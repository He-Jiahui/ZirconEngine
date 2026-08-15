---
doc_type: implementation-design
status: in_progress
owner_plan: docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
failure_record: docs/plans/zircon_editor/editor/14/failure-2026-07-17-job-pump-budget-and-pending-scan.md
related_code:
  - zircon_editor/src/core/jobs/system/pending.rs
  - zircon_editor/src/core/jobs/system/mod.rs
  - zircon_editor/src/core/jobs/system/construction.rs
  - zircon_editor/src/core/jobs/system/submission.rs
  - zircon_editor/src/core/jobs/system/lifecycle.rs
  - zircon_editor/src/core/jobs/system/scheduling.rs
  - zircon_editor/src/core/jobs/system/progress_observer.rs
  - zircon_editor/src/core/jobs/system/state.rs
  - zircon_editor/src/core/jobs/spec.rs
  - zircon_editor/src/core/jobs/event_sink.rs
  - zircon_editor/src/core/jobs/progress.rs
  - zircon_editor/src/core/jobs/event.rs
  - zircon_editor/src/core/jobs/tests/scheduling_contract.rs
  - zircon_editor/src/core/jobs/tests/admission_scaling_contract/indexed.rs
reference_sources:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Async/Fundamental/Scheduler.cpp
  - dev/Fyrox/fyrox-core/src/task.rs
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
---

# Editor14 M4：Job 公平准入与系统边界

## 问题与边界

`PendingJobQueue::take_next` 当前总是从 Interactive、Normal、Background 依次选取，持续的高优先级
提交可以使可运行的 Background 永远不被提升。`system/mod.rs` 同时拥有构造、提交、生命周期、调度和
observer 派发，违反叶模块按职责收敛的结构规则。稳定 job label 也在排队、取消和 worker 启动之间重复
转换所有权。

本切片只改变 `zircon_editor::core::jobs` 的编辑器门面准入政策；不修改 runtime scheduler、不创建新的
跨 crate 抽象，也不触碰 Editor02 的 non-consuming lossless producer 前置契约。JobPump 的队首重试和
lifecycle delivery reservation 仍等待该下游 owner。

## 参照与选择

Unreal 的 scheduler 将优先级队列作为明确的调度政策，Fyrox 将通用 task pool 保持为狭窄的执行内核。
相应地，Zircon runtime `JobScheduler` 继续执行任务依赖，编辑器门面只在 ready admission 层实施确定性
优先级选择。

不使用 wall-clock aging：它会把测试结果、调度顺序和性能证据绑定到运行时钟。使用一个私有的固定
`[Interactive, Interactive, Normal, Interactive, Normal, Background]` 轮转槽位。每次成功提升从下一槽位
开始寻找该优先级中任一类别的最早 ready job；槽位无可运行任务时跳过。一个 ready 且类别有容量的
Background job 因而至多等待六次成功提升。若它被类别配额或依赖阻塞，不计入该界限，恢复可运行后从
后续轮转继续参与。相同 priority 内仍按最早 `JobId`，所以保留原 FIFO 语义。

该权重是 `pending.rs` 的私有 helper policy，不是跨模块协议常量；它仅控制此队列实现，按常量收敛规则
保持在 owner 模块。

## 目标模块形状

```
core/jobs/system/
  mod.rs                # 仅声明私有叶模块与有意的 batch reservation re-export
  construction.rs       # EditorJobSystem / inner 构造
  submission.rs         # submit、batch、admission reservation/window
  lifecycle.rs          # pump、progress、cancel、shutdown、join
  scheduling.rs         # promote、finish、state lock、completion guard
  progress_observer.rs  # 有界 observer dispatch 与 panic recovery
  pending.rs            # ready indexes、admission ledger、加权公平选择
  pending/tests/        # admission snapshot 与 fairness/recovery 行为契约
  state.rs              # record、dependency、category/mutex state owner
```

`EditorJobSystem` 和其内部 state 仍是 `system` 私有实现，公开面维持现有精选的 `EditorJobSystem` 与
`EditorJobBatchAdmissionReservation`。不留下旧 `mod.rs` 行为 facade、转发 wrapper 或兼容 alias。

`EditorJobSpec.label` 改为 `Arc<str>`，构造时一次性取得稳定标签；进度快照在需要面向用户的可变
`String` 时才显式拷贝。event sink 接受并 clone 这个 `Arc`，取消和 worker 运行路径不再将 `String`
克隆后再次转换为 `Arc`。

该 label hard-cut 使 `progress.rs` 的快照构造需要显式 `to_string()`，所以它不再匹配 M3 的 snapshot
`1632`。M4 会在新的 source manifest 中一并固定该路径；M3 的既有设计和静态审查不回滚，但其后续
current-source 验证必须以 M4 successor snapshot 为准。

## 当前源码边界

原 23-path M3/M4 candidate 遗漏了 `system/mod.rs` 声明的 M1 leaf owner，已明确降为历史记录。完整的
40-path M1/M3/M4 union 由
[`2026-08-11-m1-m3-m4-current-source-manifest.md`](2026-08-11-m1-m3-m4-current-source-manifest.md)
唯一列举；它包含 M1 admission、M3 progress-generation source/evidence 与本切片所有新增 leaf owner。
任何后续验证、审查和 closeout 都必须以该 successor snapshot 为准，不能从共享 working tree 吸收其他
会话 hunk。

## 契约与验收

1. 连续 Interactive 准入不能饿死一个已 ready、类别可运行的 Background job；测试冻结最多六次成功
   promotion 的上界，并保留同 priority FIFO。
2. 依赖未完成或类别满额的 Background 不被伪造为“公平可运行”；恢复容量后参与下一个轮转；该行为
   契约位于 `pending/tests/fairness.rs`，不再堆积在 production owner 内联测试模块。
3. `system/mod.rs` 只含 module declarations 和精选 re-export；行为分别位于明确 owner 叶模块。
4. 同一 job 的 `EditorJobSpec` 与 emitted event 共享同一 `Arc<str>` 标签 allocation。
5. 快速完成先于 scheduler handle 安装时，terminal record 不得重新安装 mutex-group tail。
6. failure metadata 列出实际调度、标签和契约文件；记录保持 `open`，直到其既有 Editor02 前置条件、
   managed Cargo、规模矩阵、WPR 和二次审查都具备。

验证顺序：先运行新增 Rust 定向契约；随后在协调器中请求 current-source `zircon_editor` 受管验证、
1k/10k/100k admission 矩阵和 Windows WPR。任何受管失败以对应 owner 的 `failure-*.md` 前向交接，不
回滚本切片。
