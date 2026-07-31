# Editor04 M1.3 Play 编辑保护与 pending intents

## 目标与状态

- 状态：源码完成、静态契约与格式门通过、独立复审 0/0/0、受管 Cargo 待稳定基线。
- 目标：落实 Playing 三档编辑策略、可审计 pending intent 队列和退出 apply/discard 提示；未决队列必须阻断下一次 Play。
- 非目标：本切片不实现 M4 play-domain volatile history、运行时实体回写或资产热重载放宽，也不复制 Editor03 transaction owner。

## 完成项目

- 新增 folder-backed `edit_policy/` owner：Edit 模式下 edit-domain 即时放行、play-domain 无会话拒绝；Playing 时 play-domain 放行、运行文档锁定、其它文档/工作区操作进入 pending。
- 新增 folder-backed `pending_edits/` owner：存储现有 `EditorOperationInvocation`、单调 `PendingEditId` 与原始 target；无闭包、无第二套 command DTO。
- `apply_all` 在内部锁外逐项调用 dispatcher，失败不中断后项并返回完整失败 intent；`discard_all` 返回完整丢弃清单；并发新项通过 `remaining_count` 明示，不被旧决策吞并。
- `PlayEditProtection` 以专用 gate 串行策略升降与 route 判定；`PlaySessionController::route_edit` 复用 lifecycle transition gate，避免 stop 与 enqueue 交错丢失提示。
- resolution 权威收回 `PlaySessionController`：不暴露 raw protection/queue；Playing 中 apply/discard 返回 typed `PlayActive` 且队列不变。apply 开始先置显式 `resolving`，RAII barrier 跨越 drain 与全部 callbacks；期间新 Play 返回 `PendingEditResolutionInProgress`，回调不持 controller transition lock。
- 进入顺序硬化为 plugin activation -> edit protection begin -> backend start；backend start 失败先落闸再 activation rollback。成功 stop/crash 才落闸，cleanup 失败继续保持 Playing 和保护态。
- `PlayStartRequest::with_running_document` 携带运行文档锚；`PlayTransitionReport.pending_edit_prompt` 在退出后投影 typed decision prompt；pending 未决时下一次 `request_play` 返回 `PendingEditDecisionRequired`。
- `mod.rs` 仅挂载和精选导出；行为分别归属 policy、queue、protection 叶子，符合根 façade 零行为与 owner-module 规则，所有新增生产/测试文件远低于 800 行软预算。

## TDD、验证与开放门

- RED：`python -m unittest tools.tests.test_editor04_play_edit_protection_contract -v` 初次 4 errors，分别命中 policy/pending 模块缺失、controller 未升闸、退出报告无 prompt。
- GREEN：同命令 4/4 passed，覆盖三档策略、typed invocation 队列、逐项失败继续、discard 全量返回、pending 阻断下一次 Play 与 controller 顺序静态契约。
- Rust 行为测试已落盘：Playing 三档矩阵、Edit/play-domain 边界、apply 失败继续并保留 intent、discard、stop prompt 与 next-start guard。
- 首轮独立复审为 `0 Critical / 2 Important / 0 Minor`：发现 Playing 可经 raw queue 绕过决策，以及 apply drain 后 callback 窗口可抢跑新 Play。两项经 controller-owned resolution + 显式 RAII barrier 修复；review-driven RED 为 2 failures，GREEN 4/4。双 Barrier Rust 测试证明 Playing resolve 被拒且队列保持、callback 阻塞期间 request_play typed 拒绝且 mode 保持 Edit。
- 独立 re-review 为 `0/0/0`，确认 raw accessor 已删除、resolution barrier 覆盖 drain 到 callback 结束、preflight 后 `begin_play` 同锁复核覆盖竞态，未发现新问题。
- `cargo fmt --all -- --check` 通过。
- 未声明 Cargo：本切片建立时 Coordinator01 有 Runtime02 受管 Cargo 正在运行，且共享 HEAD 仍等待 Text 基线 owner 的 managed commit；不得用污染源整库结果充当验收。基线稳定后需为本 19 路径重建 current-source snapshot，执行 focused/broad `zircon_editor --lib` 门并独立复审。

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-07-22 | M1.3 `PlayEditPolicy` + `pending_edits` + exit decision | 源码完成 / static green / review 0/0/0 / Cargo待办 | 三档策略、运行文档 typed lock、可序列化 operation intent 队列、锁外逐项 apply/失败报告、discard、stop/crash prompt、未决/Resolving 双状态阻断下一次 Play 已落地；Python TDD RED 4 errors -> GREEN 4/4，review RED 2 failures -> GREEN 4/4，Editor04 静态回归 12/12，Rust 双 Barrier 行为测试落盘，`cargo fmt --all -- --check` 与 diff-check 通过。独立首审 0/2/0、修复后复审 0/0/0。共享 Cargo/Text 基线未稳定，未声明 Cargo。 |

2026-07-22性能补充：上述行为合同成立不代表queue有界；`PendingEditQueue`仍无entry/bytes/age上限、snapshot全clone、`apply_all`单次无预算。PERF-MVP-551与`failure-2026-07-22-play-pending-edit-unbounded-queue.md`保持open，须补typed retention和budgeted apply后再做规模验收。
