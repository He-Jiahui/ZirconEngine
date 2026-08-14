---
handoff_kind: fixed
status: fixed
created_at: 2026-08-13
summary_slug: host-api-abi-decode-target-cache-rmeta-missing
origin_plan: docs/plans/zircon_plugins/01-plugin-architecture-core.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_plugins/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
related_code:
  - tools/session_coordinator/cargo_jobs.py
  - tools/session_coordinator/cargo_runner.py
  - tools/session_coordinator/cleanup.py
  - tools/session_coordinator/server.py
  - tools/session_coordinator/tests/test_cargo_jobs.py
  - tools/session_coordinator/tests/test_cleanup.py
  - .codex/state/session-coordinator/cargo-runs/784cbb25cd9148b5aa40c3029826a3f1/0c318642355048afb0fd306e615b5d57/stderr.log
  - .codex/state/session-coordinator/cargo-runs/a790e2f9673c462fafb02b3558628d47/3c32d521abc4427c9663ce39a6a6005e/stderr.log
tests:
  - python -m unittest tools.session_coordinator.tests.test_cargo_jobs tools.session_coordinator.tests.test_cleanup -v
  - cargo test -p zircon_runtime --lib host_api_adapter --no-default-features --features core-min --locked --jobs 1 --target-dir <fresh-coordinator-managed-target> --message-format short --color never -- --test-threads=1 --nocapture
resolved_at: 2026-08-13
---


# Coordinator01：managed Cargo target 在依赖物化阶段丢失

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/01-plugin-architecture-core.md`
- 来源执行切片：M4 `host_api_adapter` 五域拆分与上层 typed host boundary 的 current-source focused gate
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：两次失败均发生在插件源码编译之前，最低已证边界是 coordinator-managed target 的依赖物化与压力清理生命周期；Plugins01 只消费受管 Cargo lane，不能在插件源码中修复或绕过该不变量。

## 失败现象与复现证据

第一次受管验证 job `784cbb25cd9148b5aa40c3029826a3f1` / run
`0c318642355048afb0fd306e615b5d57` 在 `syn 2.0.117` 编译期间以 `101`
终止。Coordinator 分配的 fresh target
`D:\cargo-targets\zircon-engine\plugins01-host-api-abi-decode-20260813` 缺少
`libproc_macro2-3925eafbdf1fd01b.rmeta`；`zircon_runtime` 与目标测试均未运行。

第二次使用不同盘符、不同 fresh target，reservation
`25af0f9160ec4fc39d824a2c143c71ba` 绑定 immutable 42-file source manifest
（fingerprint `77262958f759188a1d0f3d8e128f7b6b0470aaf173d622644b72f6144a39364c`），
并受管启动 job `a790e2f9673c462fafb02b3558628d47` / run
`3c32d521abc4427c9663ce39a6a6005e`。该 run 在
`2026-08-13T03:30:03.551129Z` 启动，于
`2026-08-13T03:31:04.349770Z` 以 `101` 完成；Cargo 在编译
`cfg-if 1.0.4` 时无法写入
`E:\cargo-targets\zircon-engine\plugins01-host-boundary-m4-20260813\debug\.fingerprint\cfg-if-709781a0f9096a35\dep-lib-cfg_if`，
报 `os error 3`。同样没有进入 `zircon_runtime`，零个目标测试执行。

只读账本审计确认：

- 同一 E target 只有上述一个 job；没有第二 job、显式 cleanup plan 或 cleanup reservation。
- job 与 reservation 均正常记录 `started_at`、terminal `exit_code=101`、空 live PID 集及 release；不是 never-started 或悬挂任务。
- job 的 `cleanup_policy=retained`，最终 `cleanup_status=deleted`，磁盘目录在诊断时已整体不存在。
- E 盘当时只余约 `18.25 GB`，低于 `CleanupService` 的 `50 GiB` 压力阈值；`cargo.consume_cpu_reservation` 会异步调用 `evict_idle_pools_under_pressure()`。
- 当前 durable events 只有 `cargo.start_pending`、`cargo.start_accepted`、`cargo.start_registered`，没有能把目录删除动作、执行线程和被删 target 绑定起来的事件。

旧 D target 与本次 E target 均不得复用。外部 validation copy
`5945e3ef29d74bd69602adca02e243b5` 不属于本 failure 的修复路径，必须保持未触碰。

## 最低共享层根因

已证明的最低共享故障边界是：fresh coordinator-managed target 连续两次在 proc-macro/
基础依赖物化阶段失去所需产物或目录，导致上层 current-source gate 在源码编译前终止。
第二次失败与 reservation 消费触发的磁盘压力清理并发，且 target 最终被标记为
`deleted`；但现有事件模型无法证明是压力清理竞态还是 coordinator 外部删除。
最终根因必须由 Coordinator01 的可重复并发 TDD 和 durable deletion provenance 收敛，
不得把当前相关性提前写成确定因果。

## 架构修复验收

- 先新增 RED 测试：在磁盘压力淘汰与 reservation consume/run 并发时，任何
  `leased`/`running` job 的 target 及其父子重叠目录都不可被删除；测试必须覆盖
  acquire-to-start 窗口、running 窗口与 finish/release 边界。
- 所有 prompt cleanup、pressure eviction 与显式 cleanup 删除都写入 durable event，
  至少包含 target identity、触发源、owner job、删除前 job/process 状态与结果；仅用
  `cleanup_status=deleted` 补记目标缺失不算删除 provenance。
- Coordinator01 focused tests 先通过，再以 fresh coordinator-managed target 重跑本
  handoff 的 Plugins01 原始 focused gate；必须完成基础依赖物化、进入
  `zircon_runtime` 编译并实际运行目标测试。
- Plugins01 后续 ignored benchmark、broad gate 与独立 review 仍绑定同一 current-source
  manifest；只有全部终态证据齐备后才可把 M4 标为 accepted。

## 禁止临时方案

- 不得在 Plugins01 源码中添加重试、备用 target、测试旁路、兼容层或静默回退。
- 不得通过手工预建 `.fingerprint`、复制旧 rmeta、复用两个失败 target，或单纯提高/
  关闭压力阈值来掩盖生命周期竞态。
- 不得把静态 `24/24`、历史 warmed binary 或 `0 tests` 的构建终态记为动态 GREEN。
- 不得重建、重试、清理或修改 validation copy
  `5945e3ef29d74bd69602adca02e243b5`。

## 修复结果与回传

- 根因：The acquire/start cleanup-reservation protocol already serialized overlapping leased and running targets, as the new deterministic window tests prove. The actual Coordinator defect was that prompt, pressure, and explicit deletion paths lacked one durable two-phase identity and crash settlement, so an externally missing or interrupted target could be recorded only as cleanup_status=deleted without proving who deleted it; failed deletion could also be reopened or remain permanently blocked.
- 架构修复：All three deletion paths now persist one deletion_id with trigger, canonical target identity, owner job/Session, pre-delete target/job/PID state and executor identity before removal, then persist deleted, already_missing, failed, or recovered terminal evidence. Unknown outcomes keep the overlapping reservation and fail closed; startup resumes the same deletion_id, failed exact parent/child targets block Cargo admission, and maintenance retry atomically settles all exact-target history only after successful deletion.
- 验证：python -X dev -W error::ResourceWarning -m unittest tools.session_coordinator.tests.test_cargo_jobs tools.session_coordinator.tests.test_cleanup -v (latest broad run plus post-review focused additions: acquire/start/running overlap, RuntimeError provenance, restart retry, succeeded-unreleased retry, and mixed pending/failed exact-target settlement)
- 回传：Coordinator cleanup now preserves active Cargo targets and emits durable, crash-recoverable deletion provenance; Plugins01 may rerun its focused gate only on a new managed target after FIFO admission, while failed D/E targets and validation copy 5945e3ef29d74bd69602adca02e243b5 remain untouched.
