---
doc_type: feature-completion
status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor/09/2026-08-24-background-job-hotpaths.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/131-editor-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-current-source-review.md
implementation_files:
  - zircon_editor/src/core/jobs/system/scheduling.rs
tests:
  - zircon_editor/src/core/jobs/tests/scheduling_contract.rs
---

# Editor930 Editor09 Promotion Dependency Capacity

## 完成列表

| 范围 | 修复 | 证据与门禁 |
| --- | --- | --- |
| Promotion hot path | `promote` 现在按显式 dependency 数量加可选 mutex-group tail 一次性预留依赖向量容量，再扩展显式依赖；保持原有顺序、重复项和 tail 语义。 | 源码合同锁定 `Vec::with_capacity(after + optional tail)` 与 `extend`，避免带 mutex group 的二次扩容。 |
| 行为边界 | 没有改变 admission、依赖 pin、mutex 串行化、promotion 批量上限或 runtime scheduler 调用。 | 现有 scheduling contract 继续覆盖依赖顺序、失败依赖、mutex group、取消和 shutdown；Editor09 Python 合同批量 68/68 通过。 |
| 性能门禁 | 结构上将带 mutex tail 的 dependency-vector growth 从潜在二次扩容收敛为一次预留。 | 受管 Editor09 Debug/Release Cargo 及 Release promotion workload 尚待当前源码批量验证；不提前宣称吞吐或分位数达标。 |

## 本地验证

- `rustfmt --edition 2021 --check`：`scheduling.rs` 与 `scheduling_contract.rs` 通过。
- `git diff --check`：通过，仅保留仓库 CRLF checkout 提示。
- `python -m unittest discover -s tools/tests -p 'test_editor09_*.py' -v`：68/68 通过。

本记录只覆盖这一项无语义变化的容量优化；Editor09 的 owner、resource lease、deadline、
observer fan-out、ProcessSupervisor 和完整 shutdown P0/P1 仍由父计划继续追踪。

## 当前源码批量验证

Editor930 变更已与 Runtime/Editor 开发和 Release 四路一起提交：Runtime
开发 PTY `2746`、Editor 开发 PTY `6182`、Runtime02 Release PTY `32549`、
Editor09 Release PTY `34609`。按异步验证约定未轮询 wrapper；提交输出没有
Cargo 或性能回执，故本记录仍保持 `implemented_pending_validation`。
