---
status: in_progress
local_validation_receipt: "runtime task-failure contract 4/4; batched Runtime audit 14/14; task-graph/recent-record Rustfmt checks pass; expanded non-tooling Runtime/Editor source-contract batch 4072/4072 across 962 files in 139.499s (performance-or-contract filename filter, tooling/export/coordinator excluded); managed Windows Cargo/Release and product gates pending"
plan_sources:
  - docs/plans/optimize/zircon_runtime/301-runtime-core-lifecycle-taskgraph-session-shutdown-review.md
  - docs/plans/optimize/zircon_runtime/11/2026-09-20-task-lock-poison-guard-repair.md
  - docs/plans/astra/features/01-product-correctness.md
---

# Runtime 任务失败诊断修复

## 已确认问题

`TaskGraphScopeInner::run` 和 `TaskNode::run_detached` 将 `catch_unwind` 的
`Box<dyn Any + Send>` 以 `&payload` 传给接收 `&dyn Any` 的函数。实际 downcast
检查的是 Box，字符串 panic 因而退化为 `non-string panic payload`，丢失根因。
这是 Runtime301 的任务错误保留局部修复，不关闭完整 shutdown receipt 缺口。

## 实施与验收

- 消费 boxed payload，保留静态字符串、owned String 和非字符串 fallback；owned
  String 直接移动，避免消息长度相关的额外复制。
- 同批覆盖 scoped、detached、dependency failure 终态及确切消息；确认 panic 后 worker 可排空。
- 与 [Editor 进度派发](../editor/01-progress-dispatch.md) 组成一个验证批次。
- Windows coordinator 验证当前源码快照；源码修复和静态检查不能代替测试通过。
- owned String 转换保留原缓冲地址作为零复制门槛；产品停机与性能仍需各自实测。

## 计划完成列表

| 批次 | 内容 | 状态 | 验证证据 |
|---|---|---|---|
| M1 | 保留任务 panic 消息，移动 owned String，补终态回归 | implemented_pending_validation | 请求 `astra-m1-task-progress-20260905-v1` 在封存前拒绝：`validation_ticket_external_worktree_dirty`，尚无编译或性能结果 |
| M2 | Runtime15/Runtime11 锁毒化 guard 同步 TaskNode 与 pending 所有者 | implemented_pending_validation | [Runtime11 task lock-poison guard repair](../../../optimize/zircon_runtime/11/2026-09-20-task-lock-poison-guard-repair.md)；Exact Rustfmt/source-anchor/diff checks pass，managed Cargo pending |

## 验证交接

Windows Rust/Cargo 1.94.1；命令为
`cargo test --locked --release --no-default-features -p zircon_runtime -p zircon_editor --lib astra_m1 -- --include-ignored --nocapture --test-threads=1`。
本轮 7 个修改/新增文件连同既有 tasks 模块依赖形成 40 项 manifest；既有 TaskNode
重构仅是验证输入，不冒认为本轮交付。封存因外部 `E:/Git/zr_vm` 的源码与子模块 dirty
被拒绝，没有 ticket。依赖恢复可封存后重提当前 hash；不轮询、不回滚外部工作树。
独立复审已完成，Editor 测试的缓冲身份假设与预热实例问题已修正。
