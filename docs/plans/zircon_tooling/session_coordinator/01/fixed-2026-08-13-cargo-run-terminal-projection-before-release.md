---
handoff_kind: fixed
status: fixed
failure_scope: local
created_at: 2026-08-13
summary_slug: cargo-run-terminal-projection-before-release
origin_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_tooling/session_coordinator/01
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
plan_link_mode: child_record_only
resolved_at: 2026-08-13
related_code:
  - tools/session_coordinator/cargo_jobs.py
  - tools/session_coordinator/cargo_run_registration.py
  - tools/session_coordinator/cargo_runner.py
  - tools/session_coordinator/reserved_starts.py
  - tools/session_coordinator/windows_job_process.py
  - tools/session_coordinator/tests/test_cargo_jobs.py
  - tools/session_coordinator/tests/test_cargo_runner.py
  - tools/session_coordinator/tests/test_reserved_starts.py
  - tools/session_coordinator/tests/test_windows_job_process.py
tests:
  - python -W error::ResourceWarning -m unittest -v tools.session_coordinator.tests.test_cargo_jobs
  - python -W error::ResourceWarning -m unittest -v tools.session_coordinator.tests.test_reserved_starts
  - python -W error::ResourceWarning -m unittest -v tools.session_coordinator.tests.test_cargo_runner
  - python -W error::ResourceWarning -m unittest -v tools.session_coordinator.tests.test_windows_job_process
---


# Coordinator01: Cargo run terminal projection precedes release

## 来源执行者

- 来源计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 来源执行切片：snapshot 1688 combined Failure closeout managed validation
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：同一编号计划拥有 Cargo runner 的持久化终态与 reservation 释放协议。

## 失败现象与复现证据

托管运行 `d90bafe197e248e79c2b6f1f0cbf8ee3` 的 181 项回归中，
`test_runner_releases_bound_cpu_reservation_after_owner_becomes_stale` 在 run 已报告
`completed` 时仍观察到 job=`succeeded`，后台线程随后持有 SQLite 文件并导致
`TemporaryDirectory.cleanup` 触发 WinError 32。事件屏障 RED 稳定得到
`completed != running`。

## 最低共享层根因

`CargoJobRunner.status()` 会让 `reconcile_terminal_runs()` 恢复本实例仍在
`_finish` collector 中的 run。`_finish` 先持久化 job terminal，再释放 reservation，
最后才持久化 run terminal；恢复路径因此越过 collector 的剩余工作，发布假终态。
同时，`_running` 还保存启动清理未能证明停止的进程句柄，这类条目没有 collector，
不能被同一个恢复排除条件永久跳过；恢复也不能依赖查询前拍摄的 collector 快照，
否则 collector 可在查询等待期间注册并重新打开提前投影窗口。
更早的窗口存在于 `cargo_jobs.start()`/run INSERT 已提交而 collector 尚未登记之间；
此时 `CargoJobService.reconcile_orphans()` 看不到 runner 本地状态，可把正常启动误判为
orphaned，随后提前完成 run 投影。
反向窗口同样危险：若 `Popen` 已成功而 durable start 被守卫拒绝，随后 kill/wait 又
不能证明终止，旧实现只保留进程内句柄。超时恢复或协调器重启会丢失真实 writer 的
PID/creation identity，允许同 target 被重新取得。

## 架构修复验收

- 本地 collector 仍持有 job 时，status 必须保持 run=`running`，直到 release 与 run terminal 写入完成。
- managed run 对外变为 running 与 collector intent 登记必须相对同进程 orphan reconcile 原子化。
- spawn 后任何 cleanup-unproven 状态必须持久化 job/reservation/run 与 PID/creation identity，重启后仍阻止 target 复用。
- Session、reservation command fingerprint、target owner 与 CPU FIFO 必须在 `Popen` 前由同一持久化事务授权；拒绝路径不得让命令获得执行机会。
- 授权后、spawn 前崩溃留下的 `pid=NULL` 启动意图必须由新实例终结，且不得重放命令。
- Windows child 必须在 kill-on-close Job 中 suspended 创建，PID/creation identity/run 与 collector/log reader 就绪均先于 resume；日志 reader 不得拥有裸 Job handle。
- 日志读取失败必须由 collector 在等待 root 前终止完整 Job tree；日志写入失败必须继续排空 pipe，且两者都投影为 `finish_blocked`。
- 新 runner 在重启后仍能恢复 released、orphaned 与 exitless run 投影。
- 重新运行 Cargo jobs 全模块、reserved-start 生命周期及 combined closeout 托管验证。

## 禁止临时方案

- 不得以 sleep、重复重跑或放宽 ResourceWarning 隐藏竞态。
- 不得添加测试旁路、调用点特例或重复终态来源。

## 修复结果与回传

- 根因：CargoJobRunner.status 在本实例仍拥有活跃 finish collector 时恢复 terminal job projection，导致 reservation release 前发布 run completed；`_running` 又混合保存无 collector 的 cleanup-unproven 进程句柄。仅修 runner 本地检查仍留下 run INSERT 与 collector 注册之间的服务级孤儿回收窗口。
- 架构修复：新增独立 `_collecting` 集合，仅让确有活跃 collector 的 job 跳过 terminal projection 恢复；`_running` 继续保存所有未证明终止的本地进程句柄。恢复在每条 UPDATE 前持 `_running_lock` 重验当前 collector并完成投影。`CargoJobService` 以同一生命周期门闩串行 authorization/spawn/managed registration/cleanup 补偿与 orphan reconcile，并跳过已登记 collector 的作业；collector 终态、数据库异常及启动失败均从最外层 `finally` 注销。新 `cargo_run_registration.py` 在 child 创建前原子消费 Session、reservation command fingerprint、target owner 与 CPU FIFO 准入，记录 `running,pid=NULL` 的不可重放启动意图。生产 runner 随后通过 `windows_job_process.py` 在 kill-on-close Job Object 中原子创建 suspended child，把 PID/creation identity 与 `cargo_job_runs` 在一个事务中提交，登记 collector 并等待两条 pipe reader 打开日志后才 resume；因此可执行指令前不存在 PID-only 或 run-only 半状态。reader 只发布 read/write failure，collector 独占 Job handle，并以短周期可中断查询在 root wait 前终止失败的完整进程树；write failure 仍持续排空 pipe。spawn 失败则回滚许可；若 cleanup 无法证明终止，同一模块原子记录 job/reservation/run、PID/creation identity 与原拒绝码，保持请求失败事实但让 target fail-closed。兼容恢复遇到旧版 live-PID/no-run 状态也保持 job/reservation 占用，不再把它标成 before-spawn 后释放。
- 验证：事件屏障 RED 复现 completed-before-release 与 WinError 32；cleanup-unproven RED 复现 orphaned job 永久不恢复；陈旧快照 RED 复现 reconcile 在 collector 注册后仍提前投影；注册窗口 RED 复现正常零时长 run 被 orphan/reconcile；spawn-before-guard RED 复现维护持锁期间已有活进程；start-rejection + kill failure RED 复现 job=`leased,pid=NULL` 且可被回收复用；stale Session 与 reservation command mismatch RED 均证明拒绝发生时 `Popen` 已被调用；after-spawn 两个 RED 交错分别复现 PID 丢失和 live PID 被恢复释放；日志 RED 分别复现 open failure 后错误 resume、write failure 后 pipe 阻塞，以及 read failure 与启动回滚竞争裸 Job handle。修复后完整 `test_cargo_jobs` 64/64、`test_reserved_starts` 11/11、`test_cargo_runner` 5/5、`test_windows_job_process` 8/8 通过，均提升 ResourceWarning 为错误；真实 Windows 集成验证 suspended/resume、stdout、exit code 与完整 Job tree release。
- 回传：Managed Cargo run terminal state now waits for the owning collector to finish job and reservation release; restart-only reconciliation remains available.
