Plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
Milestone: M4.1
Status: implementation_complete_isolated_gate_green_full_workspace_blocked
Files: ["docs/plans/zircon_editor/editor/03/2026-08-28-transaction-history-data-source.md", "docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md", "zircon_editor/src/ui/workbench/snapshot/data/transaction_history_snapshot.rs", "zircon_editor/src/ui/workbench/snapshot/data/mod.rs", "zircon_editor/src/ui/workbench/snapshot/mod.rs", "zircon_editor/src/ui/workbench/state/transaction_history_projection.rs", "zircon_editor/src/ui/workbench/state/mod.rs", "zircon_editor/src/ui/host/editor_event_runtime_access/snapshot.rs", "zircon_editor/src/tests/workbench/transaction_history_snapshot.rs", "zircon_editor/src/tests/workbench/mod.rs"]

# Editor03 M4.1 transaction history data source

## 当前源码结论

- `EditorTransactionEngine` 继续作为唯一历史 owner。新增的 `TransactionHistorySnapshot::query` 只消费已有 `history_details`，不复制 `HistoryStore`，不增加 UI 私有 undo/redo 栈。
- 数据源保留 context、generation、总条目数、dirty/can-undo/can-redo、saved-top 可达性，以及 UI 行所需的 transaction id、label、frame、command/participant 数量、significant、applied/top/saved-top 标记；selection snapshot 和 command object 不进入 UI 投影。
- `EditorState::active_scene_transaction_history_snapshot` 只解析当前 scene document 或 PIE play-session context；无活动场景返回 `None`，engine busy/faulted 等错误保持 typed `EditCommandError`。host API 名显式包含 `scene`，不伪装已经接管 animation editor 的焦点路由。
- animation editor 可直接复用通用 `TransactionHistorySnapshot::query`，其焦点选择仍由 animation owner 负责；本切片不建立第二套焦点 authority。

## 参照与结构复核

- 主要参照 `dev/UnrealEngine/Engine/Source/Developer/UndoHistory` 与 `dev/UnrealEngine/Engine/Source/Editor/UndoHistoryEditor/Private/SUndoHistory.cpp`：UI 观察权威 transaction buffer，并在 buffer/state delegate 变化后刷新 list/detail；UI 不拥有事务历史。
- 当前 Zircon 已有 canonical `TransactionMessage` lifecycle bus 和 generation-aware page cursor，因此本次只补缺失的 read model。没有在右侧 History tab 内创建临时数组 authority，也没有修改事务核心、journal 锁域或 Inspector 虚拟行实现。
- 右侧 History 页的视觉模板、动画焦点组合和消息驱动刷新仍属于后续 layout/host 消费层；M4.1 只交付计划明确要求的数据源。

## 性能边界

- 查询是显式按需调用，不加入每帧 `EditorDataSnapshot`/`EditorChromeSnapshot` 构建；静止 History 页不会产生新轮询。
- 单次最多读取 `MAX_HISTORY_DETAIL_PAGE_SIZE = 128` 条，时间与临时内存均为 `O(min(history_len, 128))`；超过上限只设置 `truncated`，不继续无界分页复制。
- applied 区间按权威 `top` 在首个稳定历史页中的位置计算，不依赖 transaction id 连续性；当 `top` 位于截断后的后续页时，当前可见前缀才整体标记为 applied。
- `rows` 以 `Arc<[TransactionHistoryRowSnapshot]>` 发布，克隆 UI snapshot 只复制 handle；投影不保留 selection before/after 的 Arc，也不持有 engine mutex 或 shell lock 到调用方。

## 验证与边界

- `rustfmt --edition 2021 --check --config skip_children=true`：本切片 8 个 Rust 文件通过。
- scoped `git diff --check`：通过，仅输出仓库既有 LF/CRLF 提示。
- D 盘隔离夹具 `D:\zt\plan03-history-projection` 直接 `include!` 产品 `transaction_history_snapshot.rs` 与 `transaction_history_projection.rs`；`cargo test --manifest-path D:\zt\plan03-history-projection\Cargo.toml --offline` 为 `4 passed / 0 failed / 0 ignored`，覆盖 128 页上限、applied/redo/saved-top、无 top、top 位于后续页、活动 scene context 与无场景 `None`。
- 同一目标会话已确认完整 workspace 当前先在共享 `zircon_runtime` 停止（146 errors，日志 `D:\zt\plan10-reference-runtime-check-20260828.log`）；该门未进入本切片 editor owner，不能作为本切片编译失败，也不能据此宣称 M4.2 通过。
- 未创建 git commit、未同步协调器、未发送企微。必须等完整行为门和 milestone review 接受后再执行这些动作。

## 未关闭项

- M4.2 仍需在可编译 current-source workspace 上运行 transaction lifecycle sequence、journal round-trip 与 workbench history projection 集成测试。
- History tab UI 消费需要组合 scene/PIE 与 focused animation history，并用 canonical transaction message 做 dirty refresh；不得把本数据源塞进 display-cadence 全量 Chrome snapshot。
- journal production command payload 和锁外序列化性能债务继续由 typed journal 记录及 `2026-08-23-journal-lock-domain-performance-review.md` 追踪，本切片不越权优化。

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 证据 |
|---|---|---|---|
| 2026-08-28 22:21 +08:00 | `implementation_complete` | 完成唯一事务 owner 上的有界 history read model、scene/PIE context 路由、typed host 查询入口与三组仓库测试；未修改事务内核或创建兼容层。 | 产品 owner 与测试路径见 Files；UE UndoHistory owner/list 结构已复核。 |
| 2026-08-28 22:21 +08:00 | `isolated_gate_green_full_workspace_blocked` | D 盘真实源码夹具 4/4 通过，格式与 whitespace 门通过；完整 editor 行为门保持开放。 | `cargo test --manifest-path D:\zt\plan03-history-projection\Cargo.toml --offline`: `4 passed / 0 failed / 0 ignored`；共享 runtime 既有 146-error 日志见上文。 |
