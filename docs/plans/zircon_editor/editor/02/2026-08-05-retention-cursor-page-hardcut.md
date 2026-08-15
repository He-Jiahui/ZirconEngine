# Editor02 retention cursor page hardcut

## 范围与不变量

- 修复 `failure-2026-07-17-editor-event-journal-listener-unbounded-retention.md` 的当前源码 P0：listener polling 不得先物化三类 queue 的全部记录、排序并投影为 owned DTO。
- 控制请求硬切为 `QueryDeliveriesPage { listener_id, after_delivery_cursor, max_deliveries }`；旧 `QueryDeliveries` 与 `QueryDeliveriesSince` 不保留 compatibility variant、forwarder 或默认无界页。
- `max_deliveries` 只接受 `1..=256`。非空响应总是返回最后一项的 `next_delivery_cursor`，空页才返回 null；配合 `has_more`，caller 即使在页尾之后才收到迟到事件，也能以已见 watermark 连续拉取而不重扫。cursor 是 listener-local delivery arrival ordinal，不能用可能乱序抵达的 event sequence 伪装为 continuation token。
- `AckDeliveriesThrough` 同步硬切为 `delivery_cursor`，只移除该 listener 已确认 watermark 之前的 delivery；不得以 event sequence 确认并删除迟到、尚未读取的低 sequence 记录。
- 三类 retention queue 以 delivery cursor 保存，并以 event sequence 辅助 journal/replay 排序；ack 只按 delivery cursor 截断。page 用三路 k-way merge 只访问 cursor 之后至多页大小的记录。latest-state 的 key 到 delivery cursor 索引替代线性查找和中段移除。
- 一条 `SharedEditorEventRecord` 在构造期只 JSON 编码一次并立即释放临时 bytes，其长度是所有 journal/listener budget 的唯一 accounting 值。listener fanout 仍只复制 `Arc`；ABI/JSON 控制响应边界才产生 owned delivery 投影。

## 完成项

- [x] 将 retention 存储从 `VecDeque` 硬切为 delivery-cursor `BTreeMap`、age/event-sequence `BTreeSet` 与 latest-state `HashMap` 索引；ack 按 delivery cursor 截断，journal/replay 继续读取 event-sequence 索引，诊断直接读取索引首末。
- [x] 新增 `EditorEventListenerDeliveryPage`，删除两条旧无界 query request 与 registry API，所有现有 event runtime consumer tests 一次性迁到显式 256 或更小页。
- [x] registry mutex 内只返回共享 `EditorEventRetentionPage`；service 在 guard 析构后才投影 owned delivery/JSON，并以 source-topology regression 锁定该顺序。
- [x] acknowledgement 从 event sequence 硬切到 delivery cursor；覆盖“先确认 cursor 1、后到达 event sequence 1”仍可被 cursor 1 后的 page 拉取。
- [x] 增加混合 durable/frame-local/latest-state 的两页连续测试与非法零页大小拒绝测试；修复两个 `EditorEventRecord` fixture 缺失的 `binding_path`、`transaction_id`、`save_generation` 字段。
- [x] immutable route/filter generation 与锁外 per-owner enqueue：registry 配置变更重建 frozen route snapshot；`record` 取 snapshot 后才 filter/enqueue；status/page/ack 取 listener-owned handle 后运行 retention。
- [ ] source-bound managed Cargo、1k/10k listener/page contention、failure fixed return 和受管提交仍待 coordinator materialize current-source snapshot。

## Route P0 设计与实现

- listener registry 仅在 register/unregister/enable/filter 变更时重建 `Arc<[EditorEventListenerRoute]>`；route 固化 enabled/filter 与一个 listener-owned inbox handle，发布路径不再逐 event 重新遍历或克隆 descriptor/filter。
- `EditorEventService::record` 只在 registry mutex 内获取一份 route snapshot，随后在锁外对 snapshot 执行 filter，并逐 route 锁定自己的 inbox 后 enqueue。journal push 与任一 slow listener 不持有全局 registry mutex。
- controls 保持 registry 为 descriptor authority；query/status/ack 取得 inbox handle 后在 registry guard 之外运行 retention 读取或变更。unregister 与正在飞行的旧 snapshot 的线性化点为 snapshot 获取时刻。
- 测试先锁定四个语义：config mutation 重建 generation、禁用/过滤后的新 snapshot 不投递、两个 listener 的 inbox handle 不共享 mutex、`record` 的 route snapshot guard 在 filter/enqueue 前析构；managed gate 再覆盖 0/1/1k/10k listeners 与 0/50/100% filters 的 lock wait/hold 和 p95。
- source-topology 回归锁定 route snapshot 在 filter/enqueue 之前析构；行为回归持有旧 snapshot 后切换为 failure-only/disabled/unregister，证明在飞行 route 维持其获取时语义而后续 snapshot 不再投递；detached inbox handle 断言旧 route 仍实际递增投递数。route module 与 service exact paths 已取得 lease 并完成当前源码修复。独立二审已通过，受管 Cargo/1k-10k contention 仍待后续门禁。

## 静态验证

- `rustfmt --edition 2024 --config skip_children=true` 覆盖本切片 11 个 Rust 文件通过。
- scoped `git diff --check` 通过。
- source grep 确认旧 `QueryDeliveries`/`QueryDeliveriesSince` request 不存在；内联结构回归同时锁定 ACK 不再依赖 `retained_by_event_sequence`。
- 最终独立二审：Critical / Important / Minor = `0 / 0 / 0`；复审覆盖 ACK delivery cursor、迟到低 sequence、三路 page merge、索引同步清理与锁外 DTO/JSON 投影。
- 未运行 Cargo；本记录不将格式或静态审计表述为运行时 GREEN。

## 产出记录与时间

| 时间 | 切片 | 状态 | 完成项目与后续门禁 |
| --- | --- | --- | --- |
| 2026-08-05 CST | PERF-MVP-067 cursor page / latest-state index forward repair | `source_forward_repair_static_green / independent_second_review_green / managed_validation_pending` | 已 hard-cut 旧无界 listener query，建立 delivery-cursor k-way page、一次 encoded-length accounting、latest-state O(1) key index，并使 mutex 仅覆盖 shared page；ACK 亦以 delivery cursor 截断，迟到低 sequence 不会被已确认高 sequence 删除。独立二审 `0/0/0`；Coordinator01 下一步必须基于当前 exact source manifest 调度 managed Cargo 和 contention/stress evidence。 |
| 2026-08-05 CST | PERF-MVP-067 immutable route / per-owner inbox forward repair | `source_forward_repair_static_green / independent_second_review_green / managed_validation_pending` | register/unregister/enable/filter 重建 `Arc<[EditorEventListenerRoute]>`；`record` 在 registry guard 析构后才 filter 并锁定各 route 的 inbox，control 的 status/page/ack 同样先取得 handle。in-flight snapshot 与新 filter/disable/unregister 行为、detached inbox 的旧 route 实际投递、静态 topology/legacy-path 合同均已覆盖；最终独立二审 `0/0/0`，下一步由 Coordinator01 对 current source 调度 managed Cargo 和 0/1k/10k contention evidence。 |
