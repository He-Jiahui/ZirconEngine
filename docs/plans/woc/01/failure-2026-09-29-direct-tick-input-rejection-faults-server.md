---
handoff_kind: failure
status: open
created_at: 2026-09-29
summary_slug: direct-tick-input-rejection-faults-server
origin_plan: docs/plans/woc/01-woc-zrvm-one-to-one-replication.md
fixing_plan: docs/plans/woc/01-woc-zrvm-one-to-one-replication.md
origin_child_dir: docs/plans/woc/01
fixing_child_dir: docs/plans/woc/01
failure_scope: local
plan_link_mode: child_record_only
related_code:
  - examples/woc/native/plugins/woc_runtime/src/transaction.rs
  - examples/woc/native/plugins/woc_runtime/tests/transaction.rs
tests:
  - cargo +1.94.1 test --locked --manifest-path examples/woc/native/Cargo.toml -p woc_runtime --test transaction direct_invalid_input_rejection_preserves_server_transaction -- --exact --nocapture
  - cargo +1.94.1 test --locked --manifest-path examples/woc/native/Cargo.toml -p woc_runtime --test transaction -- --nocapture
  - cargo +1.94.1 test --locked --manifest-path examples/woc/native/Cargo.toml -p woc_runtime --features backend-zr-vm --test zr_vm_project_vm real_retained_project_adapter_contract -- --exact --nocapture
---

# WOC 01: direct tick input rejection faults the server

## 来源执行者

- 来源计划：`docs/plans/woc/01-woc-zrvm-one-to-one-replication.md`
- 来源执行切片：M2 transactional runtime direct-input fault audit; the MVP 00 baseline remains in progress.
- 修复责任计划：`docs/plans/woc/01-woc-zrvm-one-to-one-replication.md`
- 交接原因：输入编码和运行状态转换均由 WOC 事务适配器直接拥有，这是计划内的本地 failure。

## 失败现象与复现证据

`WocTransactionalRuntime::prepare_tick` 在调用 VM checkpoint 或 fixed tick 前编码直接传入的命令。原始复现中，未知命令 ID 导致 `EncodeInput` 后仍调用 `transition_failure`；Server 角色因此变成 `Faulted`，虽然 VM 和已提交快照尚未改变。当前源码已在此处直接返回结构化 `EncodeInput`，不进入失败状态转换。现有 wire admission 覆盖不了这条直接运行时入口。新增的精确回归检查 `Running`、快照不变、零 VM 调用及随后合法 tick 可提交；受管动态复现尚未执行。

## 最低共享层根因

事务适配器将前置输入拒绝误分类为执行失败，把尚未进入 VM 的错误交给角色故障状态转换。

## 架构修复验收

- 直接无效输入返回结构化 `EncodeInput`，保持 Server `Running` 和已提交快照不变，且不调用任何 VM 方法；下一次合法 tick 仍能提交。
- checkpoint 之后的 VM、解码、预算和回滚失败维持原有事务故障语义；完整 `transaction` 测试批次通过。
- M2 的真实 `backend-zr-vm` 事务测试及所需当前源码闭包通过后，才能回传并恢复向上的 MVP 验收。

## 禁止临时方案

- 不在 Server 调用点补状态重置，不以 wire-only 校验代替直接运行时边界，不放宽故障或事务测试。

## 修复结果与回传

Open state: `待验证`; no managed Cargo, real VM, return, or closeout pass is claimed.
