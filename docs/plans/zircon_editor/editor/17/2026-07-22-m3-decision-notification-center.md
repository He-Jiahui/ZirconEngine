# Editor17 M3.2 typed Decision notification center

## 目标与状态

- 状态：核心源码完成、静态契约/模块 Rust/格式门通过、独立复审 0/0/0；共享挂载、受管 Cargo 与 retained UI adapter 待办。
- 目标：建立无需 UI callback 的 typed Decision authority，为恢复、删除阻断、Play pending edits 等需要显式选择的流程提供单源 pending/snapshot/receipt 契约。
- 非目标：本切片不实现 Toast/Progress、activity 呈现迁移、Editor04 apply/discard adapter，也不越权修改当前由其它会话持有的 `core/mod.rs`。

## 完成项目

- 新增 folder-backed `core/notifications/decision/` owner；两层 `mod.rs` 只做挂载与精选导出，行为分别落在 `center/error/id/model/receipt` 叶子。
- `DecisionNotification` 使用 validated `NotificationId`、typed option IDs、builtin/plugin source、i18n title/message/label keys，以及显式 default/cancel policy；字段私有，调用方不能构造绕过验证的模型。
- `publish` 返回不可由外部构造的 `DecisionTicket { center_instance, notification_id, incarnation }`。同一逻辑 ID 在旧 receipt 淘汰后可重新发布，但旧 UI ticket 返回 typed `StaleTicket`；replacement center 的 ticket/cursor 返回 `ForeignTicket/ForeignCursor`，不会跨实例 ABA。
- `DecisionNotificationCenter` 使用 `Mutex<BTreeMap<...>> + VecDeque`：pending 和 receipt 容量均有界，snapshot 顺序稳定，receipt sequence/cursor 显式，过期 cursor 返回可直接重试且包含 oldest retained receipt 的 `resume_cursor`。
- 所有外部 payload 有硬边界：notification ID 192 bytes、option ID 64、source ID 128、i18n key 256、options 16；`DecisionNotification` 以 `Arc` 共享 immutable payload，snapshot 不深拷贝外部字符串/vector。
- resolve 同 ticket/option 幂等返回原 receipt，冲突 option 返回 `AlreadyResolved`；cancel 只走生产者声明的 cancel option。center 不保存/执行 callback，consumer 只通过 receipt cursor 驱动外部命令。
- receipt 淘汰同步退休对应 resolved entry；pending/receipt/ticket sequence 的失败都返回 typed error，不使用生产路径 `expect/panic`，不产生半插入。

## TDD、验证与开放门

- 初始 RED：`python -m unittest tools.tests.test_editor17_decision_notification_center_contract -v` 为 4 errors，命中 owner/model/center/行为矩阵均未存在；首轮实现后 GREEN 4/4。
- 自审 RED：私有 invariant 与重复 cancel 修正后新增旧 ticket 回归，命令为 2 failures，分别命中 `DecisionTicket` 和 `stale_ticket_cannot_resolve_reused_notification_id` 缺失；实现代际票据后 GREEN 4/4。
- 独立模块 Rust harness：`rustc +1.94.1 --edition 2021 --test zircon_editor/src/core/notifications/decision/mod.rs` 编译通过；初版 8/8，review 修复后 15/15 passed。新增覆盖 cursor 无损恢复、foreign center authority、payload 上限，以及同选项/冲突选项/publish capacity/cancel-vs-resolve 四类并发线性化。该证据只验本 folder owner，不替代整 crate Cargo。
- 独立首审为 `0 Critical / 3 Important / 1 Minor`：发现 cursor gap 不可恢复、ticket/cursor 跨 center ABA、payload 实际无界，以及并发测试缺口。review-driven 静态 RED 为 3 failures；上述 center epoch、resume cursor、payload bounds/Arc 与并发矩阵落地后静态 5/5、Rust 15/15。独立 re-review 为 `0/0/0`，确认四项均关闭且未发现新问题。
- `rustfmt --edition 2021`（exact Rust scope）与 `git diff --check` 通过；静态扫描确认生产文件无 `unwrap/expect/panic/todo/unimplemented`。
- 尚未声明独立复审和 Cargo：模块暂未挂载到共享 `zircon_editor/src/core/mod.rs`；共享 Coordinator Cargo 队列与外部基线仍在变动。待 owner scope 可用后需挂载，再创建 current-source snapshot 并执行 focused/broad `zircon_editor --lib` 受管门。
- `failure-2026-07-22-play-pending-edit-decision-notification-contract.md` 保持 open：本切片只完成 Decision core，retained UI 与 Editor04 receipt adapter 尚未闭环。

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-07-22 | M3.2 Decision core authority | 核心源码完成 / static 5/5 / Rust模块15/15 / review 0/0/0 / 挂载与Cargo待办 | typed notification/option/source、center-bound incarnation ticket/cursor、recoverable cursor gap、payload byte/count 上限与 Arc snapshot、bounded pending/receipt、显式 cancel、幂等 receipt 与四类并发线性化已落地；Python 初始 RED 4 errors -> GREEN 4/4，自审 RED 2 failures -> GREEN 4/4，review RED 3 failures -> GREEN 5/5，standalone Rust 初版8/8 -> 修复后15/15，exact rustfmt/diff-check 通过。独立首审 0/3/1，修复后复审 0/0/0；共享根挂载、retained UI adapter、受管 Cargo 和 failure fixed return 尚未完成。 |

2026-07-22性能补充：publish的pending capacity判断原需扫描全部entries；现维护`pending_count`并在首次resolve递减，新增`resolving_a_notification_releases_pending_capacity`。源码守卫/rustfmt/diff通过，current-source独立`rustc --test`为16 passed / 0 failed；受管Cargo仍pending。该O(1)止损不改变上述切片状态，也不把低频decision中心提升为MVP主热点。
