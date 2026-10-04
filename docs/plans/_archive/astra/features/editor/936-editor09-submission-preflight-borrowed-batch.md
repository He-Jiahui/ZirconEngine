---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/09/2026-08-26-borrowed-batch-admission-validation.md
related_records:
  - docs/plans/astra/features/editor/931-editor09-reservation-preflight-borrowed-requests.md
  - docs/plans/astra/features/editor/935-editor09-scheduling-dependency-capacity.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/core/jobs/system/submission.rs
  - zircon_editor/src/core/jobs/system/state.rs
  - zircon_editor/src/core/jobs/system/pending.rs
  - zircon_editor/src/core/jobs/system/admission_ledger.rs
tests:
  - zircon_editor/src/core/jobs/tests/admission_scaling_contract/indexed.rs
---

# Editor936 Editor09 Submission Preflight Borrowed Batch

## 完成列表

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Batch admission preflight | `submit_batch` 不再收集临时 `Vec<&EditorJobSpec>`；通过 `Clone + ExactSizeIterator` 两次借用遍历完成最老条目、数量与估算字节校验，再直接遍历依赖验证。既有 slice API 保留并委托到 iterator API。 | 新增 admission source contract 要求 `ensure_batch_pending_admissible_iter`、borrowed `submissions.iter().map`，并禁止恢复 `let specs = submissions` 预检向量；既有 admission 行为测试继续覆盖边界和原子性。 |
| 性能门禁 | 4,096 项批量提交的预检不再为引用收集创建额外堆向量；不改变容量、年龄、字节、依赖顺序、锁范围或提交结果。 | 新增 Editor09 ignored marker `EDITOR09_SUBMISSION_PREFLIGHT_BORROWED_BATCH_BENCH_V1`，旧模型 1 次额外引用向量分配、优化模型 0 次；managed Debug/Release 仍待异步回执。 |

## 本地验证

- `rustfmt --edition 2021 --check`：submission/state/pending/admission ledger 与合同文件通过。
- Editor09 Python 合同批量继续覆盖 68 项；Rust source contract 与 marker 随 Editor09 包测试提交。
- tooling 未修改，继续按要求延期。

## 当前源码批量验证

Editor935 的四路批次在本次 Editor936 source change 之前提交，不能覆盖当前
提交预检路径。新批次已尝试提交：Runtime Debug PTY `78084`、Editor Debug
PTY `33986`、Runtime02 Release PTY `82585`、Editor09 Release PTY `98943`。
其中 Editor Debug 与 Runtime02 Release 在入场阶段返回 `request_overloaded`
并未启动 Cargo；另外两路按约定不轮询。该批次不提供编译、测试或性能通过
结论。
