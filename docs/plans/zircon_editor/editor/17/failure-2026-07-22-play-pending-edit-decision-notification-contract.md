---
handoff_kind: failure
status: open
created_at: 2026-07-22
updated_at: 2026-08-10
summary_slug: play-pending-edit-decision-notification-contract
origin_plan: docs/plans/zircon_editor/editor/04-pie-and-simulation.md
origin_workflow_node: M1
fixing_plan: docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
origin_child_dir: docs/plans/zircon_editor/editor/04
fixing_child_dir: docs/plans/zircon_editor/editor/17
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/play/transition_report.rs
  - zircon_editor/src/core/play/controller.rs
  - zircon_editor/src/ui/host/editor_event_execution/menu_action.rs
  - zircon_editor/src/core/notifications/decision
  - zircon_editor/src/core/notifications/presentation.rs
  - zircon_editor/src/ui/host/play_pending_decision
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/notifications.rs
tests:
  - pending-edit Decision notification publish and receipt roundtrip
  - apply/discard callback routing and repeated receipt idempotence
  - stop/crash prompt parity and next-Play blocking integration
---

# Editor17：Play pending edit Decision 通知与回执契约缺失

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/04-pie-and-simulation.md`
- 来源执行者：`editor04-m1-play-edit-protection-pending-r1-20260722`
- 来源执行切片：M1.3 `PlayEditPolicy`、`pending_edits` 与退出决策提示
- 修复责任计划：`docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md`
- 目标里程碑：Editor17 M3.2 notifications 三类契约
- 交接原因：Decision 的数据源、呈现生命周期、用户回执与去重属于 Editor17 通知中心统一 owner，Editor04 不得在菜单路径私造临时对话框或 status-line 兼容语义。

## 失败现象与复现证据

Editor04 current-source snapshot `856` 已实现并复审通过 typed `PendingEditDecisionPrompt`：成功 stop/crash 后报告 pending 数量，未决或 resolving 状态会阻断下一次 Play，controller 提供 typed apply/discard resolution API。

当前 `MenuAction::ExitPlayMode` 在 `request_stop()` 后只消费 activation diagnostics，未消费 `transition.pending_edit_prompt`。现有 `WorkbenchNotification` 只有 severity、toast queue 与 notification history；仓库内没有 Editor17 计划要求的 `Decision` kind、选项 payload、receipt ID、选择回调或幂等回执 API。因此内核提示无法合法呈现和回传 apply/discard 选择。

这不是 Editor04 policy 缺陷：把 pending 数量写入 status line 不能产生用户选择；在 `menu_action.rs` 直接调用原生 message box 会绕开 retained UI、headless、测试与通知中心单源契约。

## 最低共享层根因

Editor17 M3.2 仍为 planned。`core/notifications/{center,kinds}` 的 `Toast / Progress / Decision` 三类注册契约尚未落地，现有 retained `WorkbenchNotification` 是呈现雏形而非可回执 Decision authority。

## 架构修复验收

- 在 Editor17 owner 建立 typed `DecisionNotification`：稳定 notification/receipt ID、标题/正文 key、typed option IDs、默认/取消选项、pending/resolved 生命周期和 producer ownership。
- receipt 必须幂等；重复点击、UI 重建或 replay 不得重复执行 apply/discard。关闭窗口必须按显式取消策略返回，不能静默清空队列。
- Editor04 adapter 消费 `PendingEditDecisionPrompt` 并发布一个 Decision；Apply 回执调用 `PlaySessionController::apply_pending_edits`，Discard 回执调用 `discard_pending_edits`，逐项 apply failure 汇入结果通知并保留完整失败 intent 诊断。
- stop 与 backend crash 两条退出路径复用同一 producer；pending 未决期间下一次 Play 保持 typed 阻断，回执终结后才放行。
- retained UI、headless 测试与通知中心 history 消费同一数据源，不在 `menu_action.rs` 或平台窗口层建立第二套状态。
- 契约测试覆盖 publish -> present -> receipt -> apply/discard -> next Play；重复 receipt 幂等；apply callback 阻塞期间 resolving barrier 继续生效。

## 禁止临时方案

- 不得用 status line、toast 自动消退或日志条目替代必须选择的 Decision。
- 不得在 Editor04 直接调用 Win32/平台 message box、保存 bool 或维护第二个 pending 列表。
- 不得默认 apply 或默认 discard，也不得为了 UI 解锁绕过 `PendingEditResolutionInProgress` / `PendingEditDecisionRequired`。

## 修复结果与回传

Open state：`forward_repair_complete / independent_second_review_green / managed_validation_pending`。Editor17 已落地 typed Decision + receipt authority，并把 Editor04 的 typed producer 接入同一数据源；Editor04 的 resolution barrier 仍是唯一的 next-Play 门禁。当前 expiry recovery 只前移到 Decision center 给出的 retained resume boundary：纯 foreign gap 可继续，仍保留的 owned receipt 必须正常消费，已淘汰的 owned choice 必须重发且在无 prompt 时保持错误与旧 cursor。2026-08-10 的 35-path queued ticket 和后续 snapshot `1565` 均已在验收前被独立审查否决；当前 source 已前向修复 `1565` 的 `Critical/Important/Minor = 2/2/0`，在剥离最后一个 formatting-only 路径后收敛为 37-path manifest，并通过最终独立复审 `Critical/Important/Minor = 0/0/0`，但仍必须以新的 immutable snapshot 完成受管验证，因此不得写 `fixed-*` return 或宣称产品验收。

2026-08-04 的前向修复将 retained-host 回归改为真实交互链路：`PlaySessionController` 在 Play 期间接收 deferred edit，stop 返回真实 prompt，`WorkbenchNotificationCenter` 的 option callback 发布并消费 discard receipt，随后 modal 关闭、队列清空且下一次 Play 放行。该测试不再注入伪造 prompt，也不直接调用 resolution API。共享 `NotificationId` / `NotificationSource` / `NotificationIdentityError` 已从 Decision 子域硬切至 `notifications/identity/`；Decision 不再保留旧 identity 类型或把 shared identity error 回压为 Decision error。

同一 `EditorNotificationService` 现有 Toast 与 Progress 核心契约：Toast 使用显式时钟、到期时释放容量并拒绝 live duplicate；Progress 从 `JobTicket` 只提取 `JobId`，以 active progress snapshot 投影且在终态/陈旧绑定时删除，不持有结果 receiver。生产者清单收口、`ui/activity/` 全面呈现迁移及受管动态验证仍属于 M3.2 后续工作，不在本 failure 中提前完成声明。

2026-08-05 前向修复移除了 Play adapter 内旧的硬编码英文 DTO 行文案。`PendingEditDecisionPrompt` 的 pending 数量、payload bytes 与最久 age 现在作为受限的不可变 Decision message facts 发布；`present_decision` 在单一 locale 快照中替换匹配占位符，adapter 仅将该 core message 与本地化 action label 投影为可选择行。受控 `PlayPendingDecisionSelection` 构造器保持 ticket/option identity，避免跨模块访问私有字段。该修复已完成静态与独立二次审查，但仍须以当前 source manifest 的受管 Cargo 验证决定 failure return。

Coordinator validation routing note：2026-08-05 的首个 M3 workflow run `c5ee167c2a6d4ca791598c8f2109f001` 在 Cargo 启动前错误地从多行 `Files:` 记录绑定了 manifest 自身。2026-08-10 snapshot `1559` / ticket `333acb65056445bd912a5f0c304c047b` 又绑定了被二次审查淘汰的 35-path 候选；它只作为历史 receipt，不得验收或提交。当前单行 JSON array 已收敛为精确 37 路径，fresh validation 必须绑定新的 manifest hash。源码只做前向修复，不回滚已集成快照。

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-07-22 | Editor04 M1.3 -> Editor17 M3.2 failure handoff | open | 实地确认 Exit Play 未消费 `pending_edit_prompt`，现有 WorkbenchNotification 无 Decision kind/option/receipt；Editor04 snapshot 856 已提供 typed prompt、apply/discard API 与 resolving barrier。缺口路由到 Editor17，禁止 status-line/平台对话框临时双轨。 |
| 2026-07-26 | Editor17 M3.2 P0 Decision authority | `source_complete_static_green / managed_validation_blocked` | 已新增 `core/notifications/decision` 的稳定 ID、typed option、pending/resolved receipt 与幂等重放；`EditorContext` 持有通知 authority；stop 与 backend crash 共用 producer；保留式 NotificationCenter 投影为 Apply/Discard 行，关闭/ Escape 不会静默消解，Apply failure 写入完整 intent 诊断通知。模块与保留式测试、`rustfmt --check`、`git diff --check` 均通过。受管 Cargo/原子 commit 暂不能冻结：共享 `core/mod.rs` 还引用未提交的 `recovery/settings`（本计划）及 `script_build/sync`（Editor13/Editor02）子树。外部 façade 闭包已由 Editor13 `failure-2026-07-22-script-build-facade-validation-copy-closure.md` 跟踪；不得删除 façade、吸收外部业务文件或把共享工作树当验证输入。 |
| 2026-08-04 | Decision failure forward repair + notification identity convergence | `source_complete_static_green / independent_second_review_green / managed_validation_pending` | 前向修复 retained test 为真实 `queue -> stop -> WorkbenchNotificationCenter option callback -> discard receipt -> next Play` 链路；消除了跨 mutable host sync 的 controller 借用。`NotificationId`、`NotificationSource` 与 identity error 已硬切至共享 `identity/`，旧 Decision identity-error 回压已删除。新增 Toast 显式 expiry/capacity/duplicate 与 Progress ticket-to-id snapshot 生命周期核心契约及边界测试。局部 `rustfmt`、`git diff --check`、identity/interaction/lifecycle 静态守卫通过；两轮独立复审最终 `Critical/Important/Minor = 0/0/0`。未运行 Cargo，待协调器提交 current-source managed validation。 |
| 2026-08-05 | Decision message-fact presentation forward repair | `source_complete_static_green / independent_second_review_green / managed_validation_pending` | 修复二次审查的私有字段构造与提示信息丢失：Decision message facts 仅接受受限命名的 `u64`、上限 8 项；单 locale `present_decision` 投影完成占位符替换；Play 行保留 core summary、本地化 action label 与稳定 ticket/option identity。回归覆盖参数边界、en/zh-CN 重投影和 receipt 路由；逐文件 `rustfmt --check`、完整旧 DTO/API 符号检索、双语占位符与 `git diff --check` 通过。独立二次审查 `Critical/Important/Minor = 0/0/0`。未运行 Cargo，等待协调器受管 current-source 验证后才能 return。 |
| 2026-08-05 | Coordinator M3 validation manifest routing | `managed_validation_rebind_required` | 首个 M3 run `c5ee167c2a6d4ca791598c8f2109f001` 在受管 Cargo 前只绑定 manifest 自身，原因是记录的多行 `Files:` 不符合 Coordinator JSON-array 解析契约。Progress live producer、bounded center、retained-host projection 和 ID replacement-race 前向修复后，记录已更新为精确 35 路径 JSON manifest；既有 binding 不可变，必须由 Coordinator01 fresh run 重新绑定。未发起 Cargo、未回滚源代码或删除 failure。 |
| 2026-08-05 | M3.2 Progress current-source forward repair | `source_complete_static_green / independent_second_review_green / managed_validation_rebind_required` | Job scheduler admission/terminal 事件现驱动真实 Progress observer；中心限制为 64 条，自动 job source 可被同 JobID 的手工 source 原子替换；retained-host activity/history 同步 Progress 并在 overflow 状态改变时刷新。captured snapshot 使用 `(NotificationId, JobId)` 绑定，避免稳定 ID 重用时清除新绑定。局部 rustfmt、范围 diff、JSON manifest 解析均通过；独立复审最终 `Critical/Important/Minor = 0/0/0`。当前 35 路径 manifest 等待 Coordinator01 创建新的受管验证，不可复用旧 run。 |
| 2026-08-05 | M3.2 Decision receipt consumer + current-snapshot hard cut | `source_complete_static_green / independent_second_review_green / managed_validation_rebind_required` | 前向修复本轮审查的两项 Important：Play adapter 现在以受控 cursor 消费 core receipt，host tick、retained UI、headless/replay 复用同一 apply/discard 执行路径；批次后项失败时已成功 effect 在 cursor commit 前被记忆，重试不会重复执行。retained notification bridge 删除私有 `RetainedNotificationHistory`，仅从当帧 Decision/Toast/Progress core snapshot 生成控件投影，快照消失立即清除 UI 行。本轮二审再前向修复两条 Decision 交错：adapter 状态锁内捕获 core pending snapshot，barrier 回归证明两个并发发布只保留一条 live Decision；对 headless/replay 已 resolve 但尚未消费的 receipt，adapter 保留该 ticket 的提示所有权，只有 cursor 成功提交后才可重发，避免 retained projection 产生第二条选择。二审最后发现 Decision facade 仍以 `pub(crate)` 转发共享 `NotificationId`/`NotificationSource`，已硬切为 production owner 对 `notifications` canonical identity 的直接 import，Decision 仅在自身 test module 私有导入；不保留 compatibility re-export 或 API widening。最终独立复审 `Critical/Important/Minor = 0/0/0`，5 组/19 条静态契约、`py_compile`、限定 `rustfmt --check` 与范围 `git diff --check` 通过；未运行本地 Cargo，必须由 coordinator 创建 fresh managed validation，failure 继续保持 open。 |
| 2026-08-05 | M3.2 receipt-expiry forward recovery | `source_complete_static_green / independent_second_review_green / managed_validation_rebind_required` | 独立二次审查发现 `CursorExpired` 会越过被 FIFO 淘汰但尚未执行的 Play 回执。前向修复改为 typed expiry 分支：adapter 在 receipt gate 内先冻结 publication 前的 stale cutoff，controller 必须确认 `reconcile` 实际发布 replacement Decision 后才安装该 cutoff，并返回明确错误要求用户重新选择；`Ok(false)` 或 republish 失败均不推进 cursor。新增 one-receipt-capacity 交错回归证明旧 Apply/Discard 不会被猜测执行、replacement 在 cutoff 安装间隙被 resolve 时仍留给下一次正常消费、未发布 replacement 时 cursor 保持 expired。逐文件 `rustfmt --check` 和范围 `git diff --check` 通过；最终独立 re-review `Critical/Important/Minor = 0/0/0`。未运行本地 Cargo，failure 仍为 open，待 fresh managed validation。 |
| 2026-08-05 | M3.2 current-source manifest provenance repair | `source_complete_static_green / independent_second_review_green / managed_validation_rebind_required` | `milestone prepare` 拒绝旧 38-path manifest 中三个相对当前 Session baseline 无差异的 retained-host 路径：`retained_event_bridge/workbench_notifications.rs`、`retained_host/event_bridge.rs` 与 `retained_host/mod.rs`。三者没有工作区或暂存修改，故不吸收为本轮归属；manifest 前向收窄为 35 个实际 current-attributed 路径并释放其租约。源码不回滚，最终独立 re-review `Critical/Important/Minor = 0/0/0` 和静态检查仍有效；必须创建 fresh managed validation。 |
| 2026-08-05 | M3.2 managed-validation submission receipt | `submission_unconfirmed / failure_open` | 收窄后的 35-path manifest 已通过 `rustfmt --check`、范围 `git diff --check` 与硬切守卫；随后 `milestone prepare` 在客户端 33 秒超时前未返回 coordinator receipt。该记录不推断服务端是否接受，也不重试、轮询或把 pending 当作验证通过；源码保持前向修复，failure 仍为 open，等待 coordinator 的后续受管回执或显式恢复动作。 |
| 2026-08-10 | M3.2 35-path receipt rejection + 38-path forward repair | `implementation_complete / independent_re_review_pending / managed_validation_pending` | 受管 snapshot `1559` / queued ticket `333acb65056445bd912a5f0c304c047b` 在验收前被独立审查否决。当前 38 路径已补齐 Decision owner/contract 并剥离 logging/world-sync/hierarchy 外部 hunk；CursorExpired 只提交 retained resume boundary，foreign-only gap 不要求 Play prompt，retained/post-boundary owned receipt 不被跳过，evicted owned choice 在无 replacement 时不前移 cursor。Progress lifecycle 使用 1024 上限、overflow 单一 resync、panic 隔离与权威快照修复。Python contract 8/8、edition-2021 exact-manifest rustfmt、范围 diff-check 通过。待 fresh independent re-review 和 source-bound managed Cargo；failure 保持 open。 |
| 2026-08-10 | M3.2 malformed 38-path snapshot routing | `rejected_before_validation / failure_open` | Snapshot `1564` 因 PowerShell 动态数组未 splat，被协调器记录为一个空的长路径 tombstone；未提交 validation ticket、未产生 Cargo 或集成动作。该 snapshot 禁止复用，fresh snapshot 必须逐项绑定 38 个路径哈希。 |
| 2026-08-10 | M3.2 snapshot 1565 second-review repair | `implementation_complete / independent_second_review_green / managed_validation_pending` | Snapshot `1565` 的独立审查结果为 `Critical/Important/Minor = 2/2/0`：精确 overlay 仍混入 logging/autosave/hierarchy/viewport 外部依赖，Progress delivery guard 被 `catch_unwind` 结果遮蔽导致编译失败，不可逆 Apply/Discard effect 后的 reconcile 失败会重放 receipt，双重 observer panic 会遗留无人唤醒的 resync。当前 source 已剥离外部 hunk、修复变量与 guard、让常规 event pump 重试 resync，并将 fallible prompt reconcile 移到 receipt cursor 提交之后；新增真实 controller capacity-failure/recovery 与 event-pump resync 回归。最后一个 formatting-only dispatch 路径已从 owner 清单移除，当前 manifest 为 37 路径。Python Decision contract `8/8`、edition-2021 exact Rust rustfmt 与范围 diff-check 通过，最终独立复审 `Critical/Important/Minor = 0/0/0`。待 source-bound managed Cargo，failure 继续 open。 |
