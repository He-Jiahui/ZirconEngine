---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/09/2026-08-24-background-job-hotpaths.md
related_records:
  - docs/plans/astra/features/editor/926-editor09-background-job-hotpaths.md
  - docs/plans/astra/features/editor/931-editor09-reservation-preflight-borrowed-requests.md
  - docs/plans/astra/features/runtime/696-20260911-runtime-editor-async-validation-admission-log.md
implementation_files:
  - zircon_editor/src/core/jobs/system/state.rs
tests:
  - zircon_editor/src/core/jobs/tests/admission_scaling_contract/indexed.rs
---

# Editor932 Editor09 Enqueue Dependency Capacity

## 完成列表

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Pending dependency admission | `enqueue_pending` 先按 `pending.spec.after.len()` 为未调度依赖 scratch buffer 预留上界，再用 `extend` 过滤 `AwaitingSchedule` 记录；依赖顺序和过滤语义保持不变。 | 新增 admission scaling source contract，要求上界预留与 `extend`，并拒绝回退到 `collect::<Vec<_>>()`。 |
| 分配热路径 | 过滤后的依赖集合不再依靠 `Vec` 默认容量逐步增长，已知的 `after` 上界可一次完成容量准备；终态依赖 pin、pending 插入和调度行为未改变。 | `rustfmt --edition 2021 --check` 与 scoped `git diff --check` 通过；现有 Editor09 本地合同批量 68/68 通过。 |
| 性能门禁 | 该修复减少高依赖批量 admission 的 scratch buffer 扩容/复制；真实吞吐、尾延迟和 Release 门槛仍以当前源码的受管 Editor09 批量结果为准。 | 已提交新的 Runtime/Editor Debug 与 Runtime02/Editor09 Release 四路异步验证；提交回执不等同于 Cargo 或性能通过。 |

## 本地验证

- `rustfmt --edition 2021 --check`：`state.rs` 与 admission scaling contract 通过。
- `python -m unittest discover -s tools/tests -p test_editor09_*.py -v`：`Ran 68 tests ... OK`。
- 当前记录不扩展 tooling 范围；tooling 迁移和 tooling 专属合同仍按用户要求延期。

## 当前源码批量验证

Editor932 改动落地后，先前 Editor931 四路 PTY 视为过期；新的四路提交回执和
当前源码状态记录在 696 与 Editor926：Runtime 开发 PTY `81935`、Editor 开发
PTY `42136`、Runtime02 Release PTY `99713`、Editor09 Release PTY `4102`。
所有 wrapper 均按异步约定不轮询，故本记录不提前宣称编译、测试数量、吞吐或
Release 分位数达标。

Editor933 subsequently changed progress projection scratch buffers. The four
PTYs above are therefore stale for the current Editor source; the replacement
batch and its unpolled launch are recorded in Editor933 and the central log.
