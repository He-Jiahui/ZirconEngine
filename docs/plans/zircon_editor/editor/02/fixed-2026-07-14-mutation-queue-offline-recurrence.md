---
handoff_kind: fixed
status: fixed
created_at: 2026-07-14
summary_slug: mutation-queue-offline-recurrence
origin_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_editor/editor/02
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
related_code:
  - tools/session_coordinator/server.py
  - tools/session_coordinator/leases.py
  - tools/session_coordinator/cargo_jobs.py
  - tools/session_coordinator/control_plane/actions/service.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_server tools.session_coordinator.tests.test_cargo_jobs tools.session_coordinator.tests.test_action_execution
resolved_at: 2026-07-14
---


# Tooling01：mutation queue 在已修复记录后再次卡死并返回 offline

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 来源执行切片：M1 测试阶段 / 受管 Cargo 与 attribution 生命周期
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：失败发生在 coordinator mutation mutex/请求队列，不属于 Editor02、Runtime02 或 Cargo 编译行为；既有 `mutation-queue-finish-lease-stall` 已标记 fixed，本次是重启后可重复的新复发。

## 失败现象与复现证据

第一次复发发生在 daemon instance `d57f039fa23c48a6a9b185a53e3ee184`：

1. Runtime02 Cargo acquire 排队约 210 秒，在客户端终止等待的同一瞬间才创建 job `9f938a4f41f14c9597c4dae6a825f385`；包装 PID 随即消失，Cargo 未实际存活。
2. 随后的 `cargo finish` 连续等待官方五分钟客户端窗口，最终返回 `Coordinator service is offline`，job 仍停留 `running`。
3. 受控 stop/start 创建 instance `6d8be3ffef4944d59a8a3e68942fae9f` 后，同一 finish/release 立即成功，证明 job 本身与数据库记录可恢复。

第二次复发发生在 instance `6d8be3ffef4944d59a8a3e68942fae9f`：一个普通 Session note `baseline attribute` 在有其他会话事务时排队约 258 秒，随后再次返回 offline。第二次受控 stop/start 创建 instance `3b167e727b514623bcb1b5196fb7bd36`（schema 29），相同 claim/attribute 在 4.3 秒内成功。

## 最低共享层根因

最低已证明边界是 coordinator 的前台 mutation serialization/recovery：一个前台事务结束或客户端消失后，后续 mutation 请求可能长期无法取得互斥，健康/只读查询仍短暂可用，最终客户端报告 offline。重启可恢复，说明不是业务文件 lease 冲突，也不是 Cargo 子进程仍存活。

## 架构修复验收

- 客户端在 acquire/finish/attribute 等待期间取消或 PID 消失，服务端必须释放前台 mutation ownership，并把 job 归并为 typed orphan/cancelled 终态。
- 一个长事务不能阻塞无关 Session heartbeat/lease/attribute 到五分钟 offline；需要有可观测 owner/action id、超时与恢复原因码。
- stop/start 后的恢复不能依赖人工 `cargo finish` 才清除无存活 PID 的 running job。
- 增加并发回归：controlled action + Cargo finish + baseline attribute + 客户端断开，验证后续 mutation 在有界时间内成功。

## 禁止临时方案

- 不通过无限延长客户端超时、隐藏 offline、跳过 lease/attribution 或直接改数据库掩盖死锁。
- 不把未实际运行的 Cargo job 记作业务测试失败。
- 不要求业务 Session 反复重启协调器作为正常流程。

## 修复结果与回传

- 根因：全局 mutation mutex 将显式 baseline scan 的逐文件 Git 读取与 HTTP 请求绑定；客户端超时后仍等待该锁，因而把可用服务误报为 offline。
- 架构修复：将基线扫描和验证副本物化移出前台 mutex，使用固定 HEAD 的单归档流与后台作业；长操作改用自身 SQLite/Git/cleanup 协调，客户端超时改为 typed command_timeout。
- 验证：8 项定向并发、归档、过滤与验证副本回归通过；在线 materialize Cargo.toml 在 1.15 秒受理、完成并清理，且未阻塞受管 Cargo。
- 回传：Editor02 可恢复受管 Cargo 与 attribution 生命周期；后续 timeout 需按 typed command_timeout 查询状态，不能当作 offline 重启服务。
