---
handoff_kind: failure
status: open
created_at: 2026-07-14
summary_slug: mutation-queue-finish-lease-stall
origin_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
fixing_plan: docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
origin_child_dir: docs/plans/zircon_editor/editor/02
fixing_child_dir: docs/plans/zircon_tooling/session_coordinator/01
related_code:
  - tools/session_coordinator/server.py
  - tools/session_coordinator/cargo_jobs.py
  - tools/session_coordinator/leases.py
  - tools/session_coordinator/watch.py
  - tools/session_coordinator/client.py
tests:
  - python -m unittest tools.session_coordinator.tests.test_server tools.session_coordinator.tests.test_cargo_jobs tools.session_coordinator.tests.test_leases -v
  - python -m unittest discover -s tools/session_coordinator/tests -v
---

# Session Coordinator 01：mutation queue 阻塞 finish 与 lease

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 来源执行切片：M1 测试阶段
- 修复责任计划：`docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md`
- 交接原因：失败同时阻塞多个无关 Session 的 `cargo finish`、`lease claim`、heartbeat/register，最低共享边界是 coordinator mutation command queue，不属于 Editor02 WorldSync 功能。

## 失败现象与复现证据

2026-07-14 07:19–07:29（Asia/Shanghai），只读 `tools/zircon-session.ps1 --json status` 与 SQLite 只读审计持续可用，supervision 报 `healthy`；与此同时多个 mutation client 长时间无响应：

- Shader04 job `57672013572a4efbbecf7909f25441e0` 的 Cargo/rustc/测试进程实际 exit 0，随后 `cargo finish ... --exit-code 0` 自 07:19 起挂起；维护线程最终于 07:28:41 将该 job 记为 `orphaned`、`exit_code=null`，完成事实丢失且池未由正常 finish/release 路径关闭。
- Editor02 的 `lease claim` 连续两次超过 60 秒未落库；同时间还能观察到其他 Session 的 `cargo finish`、heartbeat、session register 与 lease claim client 等待。
- mutation queue 恢复后，同一 Editor02 lease claim 用时 49.7 秒才成功；正常期同类请求约 4–11 秒。

预期：完成命令必须优先、可靠登记实际 exit code 并释放池；lease/heartbeat/register 应有有界延迟，不能被 watcher/baseline/maintenance 工作饿死。原 Editor02 验证器严格等待唯一兼容池，没有创建 fallback target。

## 最低共享层根因

当前已证明的最窄边界是 `CoordinatorApplication` mutation command serialization 与后台 watcher/maintenance 并发：只读命令继续响应，所有写命令跨 Session 同时停滞，随后 orphan reconciliation 先于排队中的正常 `cargo finish` 生效。尚未证明是 `_mutation_lock` 饥饿、长事务、watch apply，还是 finish/reconcile 优先级反转；修复计划必须用并发回归定位，不得把 Editor02 或 Shader04 调用方作为根因。

## 架构修复验收

- 新增并发回归：长 watcher/baseline apply 与 `cargo finish`、`lease claim`、heartbeat 同时发生时，finish 保留调用方 exit code，所有 mutation 请求在明确上界内完成。
- `cargo finish(exit=0)` 后 job 必须进入成功终态并可 release/reuse，绝不能被 reconciliation 改成 `orphaned`/`exit_code=null`。
- foreground mutation 与后台维护具有明确优先级或分段锁边界；不得在 mutation lock 内执行无界 filesystem/git 扫描。
- 原 Editor02 M1 的两个 Windows Cargo 门禁可通过同一兼容池正常 acquire/start/finish/release。

## 禁止临时方案

- 不得创建第二个兼容 Cargo 池、直接修改 coordinator SQLite、延长无限超时或跳过 lease/finish。
- 不得以 orphan reconciliation 伪装正常完成，也不得把实际 exit 0 丢成 unknown。
- 不得新增别名、兼容 shim、静默 fallback、重复真相、测试专用 bypass 或调用点例外。
- 不得削弱测试或计划验收条件来隐藏失败。

## 修复结果与回传

待修复；不宣称 coordinator mutation queue 已通过并发验收。Editor02 在服务暂时恢复后可继续当前门禁，但本工具故障保持 open，直至 fixing plan 返回上述证据。
