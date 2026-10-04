---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/09/2026-08-26-borrowed-batch-admission-validation.md
  - docs/plans/optimize/zircon_editor/09/2026-08-24-background-job-hotpaths.md
related_records:
  - docs/plans/astra/features/editor/926-editor09-background-job-hotpaths.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/core/jobs/system/admission_ledger.rs
  - zircon_editor/src/core/jobs/system/pending.rs
  - zircon_editor/src/core/jobs/system/state.rs
tests:
  - zircon_editor/src/core/jobs/tests/admission_scaling_contract/indexed.rs
  - zircon_editor/src/core/jobs/tests/admission_scaling_contract/reservation.rs
---

# Editor931 Editor09 Reservation Preflight Borrowed Requests

## 完成列表

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Batch admission preflight | `reserve_batch_admission` 直接把 `requests.iter()` 传入 admission ledger；删除仅用于首次预检的 `Vec<&EditorJobAdmissionRequest>` 临时分配。 | `PendingJobQueue` 与 `PendingAdmissionLedger` 暴露同一受限的 `Clone + ExactSizeIterator` 借用接口，保留空批次、年龄、条目数和字节预算检查。 |
| 行为边界 | 预检仍在分配 reservation id、job id 和 reservation entries 之前执行；后续 materialization、commit、release 和 shutdown 语义不变。 | admission scaling contract 锁定无 `requests_for_preflight` 临时向量，并要求 state/pending/ledger 的借用迭代器链路。 |
| 性能门禁 | 批量预检不再为已有请求数组复制一份引用 slice，降低高频批量 admission 的一次分配和复制成本。 | 受管 Editor09 Debug/Release Cargo 及 release admission workload 尚待当前源码批量验证；不提前宣称吞吐或分位数达标。 |

## 本地验证

- `rustfmt --edition 2021 --check`：四个 Rust 文件通过。
- `git diff --check`：通过，仅保留仓库 CRLF checkout 提示。
- 现有 Editor09 Python 合同批量：68/68 通过（与本次 Rust source-contract 变更相邻的静态门禁）。

本记录只覆盖首次 reservation preflight 的借用路径；Editor09 的 owner、resource
lease、deadline、observer fan-out、ProcessSupervisor 和完整 shutdown P0/P1 仍由父计划继续追踪。

## 当前源码批量验证

Editor931 源码已与 Runtime/Editor Debug 与 Runtime02/Editor09 Release 四路
一次性提交：Runtime 开发 PTY `81169`、Editor 开发 PTY `98412`、Runtime02
Release PTY `26311`、Editor09 Release PTY `68990`。上一轮 Editor930 PTY 在
本次 source change 后视为过期；本批 wrapper 也按异步约定不轮询，因此尚无
Cargo 编译、测试数量或性能阈值回执。

Editor932 subsequently changed the pending dependency scratch buffer, so the
four PTYs above are stale for the current source. The replacement batch is
recorded in Editor932 and the central admission log; no result is inferred
from its unpolled launch.
