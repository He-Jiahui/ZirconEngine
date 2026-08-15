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
  - zircon_editor/src/core/recovery/session_guard/guard.rs
  - zircon_editor/src/core/recovery/autosave_catalog/catalog.rs
  - zircon_editor/src/ui/host/editor_host_startup.rs
  - zircon_editor/src/ui/host/editor_manager_project.rs
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

Editor17 的自动保存快照以每文档/序号的隐藏文件系统 marker 执行原子 create-new，防止独立 `AutosaveStore` 实例或进程跨扩展名复用同一 `<seq>`；marker 仅覆盖写入与轮转，并在正常终态释放。原子写的临时 `.tmp` 或 marker 在崩溃时可能保留，二者均不参与候选发现或轮换；残留 marker 只保守阻止该序号的重用。当前 `core/recovery/session_guard/` 已提供唯一 `<root>/.zircon/session.lock` 记录、OS ownership lease、残锁精确替换与正常 release；`autosave_catalog/` 也已把每个快照不可改绑地映射到项目内源文档。仍缺失的是 Editor16 唯一项目启动 owner 对该 guard 的 liveness/admission 裁决和在项目打开前的安装，因此第二编辑器仍没有产品启动路径上的明确拒绝/接管结果。

Editor17 没有自行创建项目锁文件、没有按超时删除快照或隐藏声明，也不会把内存 dirty 状态变成第二套持久化事实源。在 Editor16 锁接通前，自动保存基础层只声明每序号写入安全；真实项目级恢复启动不得接入。

## 最低共享层根因

Editor16 的 CLI/Hub 启动责任尚未收口为项目级单实例会话 admission：现有 guard 已有持久 PID/心跳、正常 release 和残锁精确替换能力，但 `EditorHostStartupSession::open_with_prepared_project`、`EditorManager::open_project` 与 `open_prepared_project_and_remember` 均未调用 `SessionGuard::inspect/acquire/replace_residual_at` 或 `AutosaveStore::recovery_candidates()`。因此项目会先被打开，之后无唯一的位置确认活动会话、授权残锁接管、保有 guard 直到 shutdown，或把恢复候选投影到 Decision 通知。Editor17 若在 autosave 或 UI 面板自行补这条 admission 会再次分裂项目进程与恢复安全的 owner。

## 2026-08-15 架构修复契约（实现中，非验收）

- **唯一 owner**：`EditorManager` 持有 `Mutex<Option<SessionGuard>>`。`open_project` 与 `open_prepared_project_and_remember` 必须进入同一个 project-admission helper；先取得 guard，项目激活失败时显式 `release`，成功后才存入 manager。工程切换与 `close_project` 在已提交关闭后显式释放。任何 autosave、Hub、UI pane 或 commandlet 都不得创建第二把项目锁。
- **活动实例**：`SessionGuard::inspect` 得到 active record 时，第二实例不能替换或删除 lock。Hub 启动将向 `<project>/.zircon/hub/focus/<instance_id>.json` 原子写入独立 focus signal；第一实例只消费匹配自身 `instance_id` 的请求，再请求原生窗口前台激活。该 signal 不写入、扩展或宽松解析 `session.lock`，因此保持 Editor17 recovery record 的严格未知字段拒绝语义。
- **UI 线程边界**：`core::hub_link::focus_signal` 只负责文件协议与事件通知；`UiHostWindow` 提供可跨线程传递的、合并式 `HostWindowAttention` callback，native event loop 在自己的线程调用 `Window::focus_window()`。不能让 core 直接依赖 retained UI，也不能由每帧文件扫描完成聚焦。
- **Hub 终态**：Hub 仅在 `BackgroundTask` 中对 project launch 等待一次 v1 mailbox。首次实例在项目 admission、retained host 初始化、plugin/template 安装、首个 UI refresh 以及 focus watcher 就绪后写 `ready { pid, project }`；上述任一前置失败写 `failed { reason }`。活动实例已成功收到 focus signal 时也回 `ready`，但 PID 必须来自 lock record 而非第二子进程。Hub 在收到匹配规范化 project 的 `ready` 前不得写 recent/action success；空 editor launch 保持无 mailbox 的原有行为。
- **存活裁决**：`dev/UnrealEngine/Engine/Source/Runtime/Core/Private/GenericPlatform/GenericPlatformProcess.cpp` 的 `IsApplicationRunning` 是平台 liveness 的参考，但 Zircon 不以 PID 单独作为锁所有权依据。现有 `SessionGuard` 的 OS ownership lease、严格 record 与 Editor17 residual-recovery policy 保持为权威，这一收紧是刻意差异。
- **结构与性能**：`dev/Fyrox/editor/` 与 `fyrox-*` 的 editor/runtime 分隔支持把协议消费置于 editor host，而不是 runtime core；Zircon 复用 `notify` 的 event ingress 模式（现有 asset watcher）而不加 idle timer。Hub 端现有 250 ms / 10 s 轮询只存在于单次后台 launch；实现后应记录 handshake elapsed、mailbox read attempts、focus event count、UI attention wake count 和 idle wake count，先采集 31 次 Windows 样本的 p50/p95、CPU 与 power baseline，再评估轮询参数，不能凭经验修改阈值。
- **验证顺序**：先覆盖 guard acquire/rollback/release、active-lock focus signal、residual lock 不接管、focus signal 一次消费、错误 mailbox 与 project mismatch；再覆盖 Hub ready/fail/timeout 与 recent writeback；最后执行本 failure 声明的受管 recovery/crash-restart fixture、M3 DTO batch 和 Hub-to-Editor product flow。UI12 的 workbench-menu test split 是独立所有权阻塞，不能被此修复绕过或改写。

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

Open state: `Editor17 自动保存基础层已保证跨实例的单序号写入，但真实项目会话 admission owner 缺失。Editor16 必须完成唯一锁契约、启动拒绝/授权与 crash-restart 证据后，按 lifecycle key 回传 fixed；Editor17 再接通 session_guard 和 M2.2 恢复流。`

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据 |
|---|---|---|---|
| 2026-07-23 17:38 +08:00 | `失败已交接-等待Editor16会话锁契约` | 将 autosave 的跨进程排他前置条件路由至 Editor16；Editor17 仅保留进程内序号单飞，不创建第二套锁或持久声明。 | 历史 `git grep` 覆盖 `zircon_editor/src` 未发现项目会话锁/`session_guard` 实现；后续需受管 recovery 单测与 crash-restart 集成夹具。 |
| 2026-08-05 | `open / forward_repair_required` | 现物已前向收敛为单一 `SessionGuard`、snapshot-to-source recovery catalog 和跨实例的单序号 autosave marker，未回滚这些已集成基础层；failure 精确收窄为 Editor16 项目启动 admission/liveness、guard 生命周期保有、残锁授权和 Decision 投影接线。 | 当前 `git grep`：`RestoreFlow`/`SessionGuard`/`recovery_candidates` 在 `core/recovery` 外无生产消费；`editor_host_startup.rs` 和项目打开入口没有 guard 或恢复候选调用。此行不替代受管 crash-restart/第二实例夹具，也不关闭 failure。 |
