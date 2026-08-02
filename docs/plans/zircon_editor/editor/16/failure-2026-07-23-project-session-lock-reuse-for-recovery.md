---
handoff_kind: failure
status: open
created_at: 2026-07-23
summary_slug: project-session-lock-reuse-for-recovery
origin_plan: docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
fixing_plan: docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md
origin_child_dir: docs/plans/zircon_editor/editor/17
fixing_child_dir: docs/plans/zircon_editor/editor/16
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/recovery/autosave.rs
tests:
  - cargo test -p zircon_editor --lib --locked core::recovery::tests -- --test-threads=1
  - zircon_editor integration crash-restart recovery fixture
---

# Editor 16：缺少可供恢复层复用的项目会话锁

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md`
- 来源执行者：`editor17-recovery-autosave-core-r1-20260723`
- 来源执行切片：Editor17 M2.1 自动保存与崩溃恢复基础层
- 修复责任计划：`docs/plans/zircon_editor/editor/16-cli-args-and-hub-integration.md`
- 交接原因：Editor17 的计划明确要求复用 Editor16 的单实例项目会话锁；当前 `zircon_editor/src` 不存在该锁的 owner 或可调用契约。

## 失败现象与复现证据

Editor17 的自动保存快照在一个共享 `AutosaveStore` 内以进程内序号保留防止跨扩展名和并发任务复用同一 `<seq>`；保留在任务终态释放，进程崩溃时由 OS 自动回收，不落持久 reservation/claim。原子写的临时 `.tmp` 在崩溃时可能保留，但它不参与序号判定或轮换，后续写入可继续。跨进程排他不能由 autosave 层自行假设：现物中没有 Editor16 提供的 `<root>/.zircon/session.lock`（PID + 心跳）契约，也没有启动路径阻止第二编辑器同时写同一项目。

Editor17 没有自行创建锁文件、没有按超时删除快照或隐藏声明，也不会把内存 dirty 状态变成第二套持久化事实源。在 Editor16 锁接通前，自动保存基础层只声明进程内序号安全；真实项目级恢复启动不得接入。

## 最低共享层根因

Editor16 的 CLI/Hub 启动责任尚未收口为项目级单实例会话生命周期：没有唯一 lock owner、没有持久 PID/心跳、没有正常退出清理，也没有“检测到陈旧前会话后才授权恢复清理”的受限能力。因此 Editor17 若自行补锁会分裂项目进程与恢复安全的 owner。

## 架构修复验收

- Editor16 在唯一项目启动 owner 中维护 `<root>/.zircon/session.lock`，锁内容至少包含 PID、进程身份/创建时间与心跳；正常 shutdown 按既定顺序删除锁。
- 启动时先取得项目独占，再区分活动锁与陈旧锁；只有确认陈旧会话时，`session_guard.rs` 才能启动 Editor17 M2.2 的恢复检测与对话流程。
- 活动会话的锁或 autosave 快照不得被超时或目录扫描删除；并发第二编辑器启动必须得到明确拒绝/转交结果，不能绕过 lock owner。
- 异常退出后的重启夹具证明：已有 autosave 快照保留、恢复流程可继续；正常并发第二编辑器夹具证明不会写入同一项目。
- 该锁同时是 Editor17 M2.2 恢复对话和后续 Editor14 关停顺序的唯一项目会话事实来源，不新增平行 lock、全局静态或兼容文件路径。

## 禁止临时方案

- 禁止在 `autosave.rs`、UI 面板或 commandlet 中自行写第二个 `session.lock`、PID 文件或布尔标志。
- 禁止依据文件时间、固定 TTL 或“目录里没有 Cargo job”删除 autosave 快照或自行推断项目没有活动编辑器。
- 禁止将旧锁路径、旧 preferences 路径或双写兼容层重新引入启动流程。

## 修复结果与回传

Open state: `Editor17 自动保存基础层仅在进程内保证序号单飞，真实项目会话锁 owner 缺失。Editor16 必须完成唯一锁契约、启动拒绝/授权与 crash-restart 证据后，按 lifecycle key 回传 fixed；Editor17 再接通 session_guard 和 M2.2 恢复流。`

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据 |
|---|---|---|---|
| 2026-07-23 17:38 +08:00 | `失败已交接-等待Editor16会话锁契约` | 将 autosave 的跨进程排他前置条件路由至 Editor16；Editor17 仅保留进程内序号单飞，不创建第二套锁或持久声明。 | `git grep` 覆盖 `zircon_editor/src` 未发现项目会话锁/`session_guard` 实现；后续需受管 recovery 单测与 crash-restart 集成夹具。 |
