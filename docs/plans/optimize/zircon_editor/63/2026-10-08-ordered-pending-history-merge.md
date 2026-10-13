# Editor63：有序合并 pending history

对应[原优化计划](../63-editor-authoring-transaction-command-history-undo-redo-merge-group-savepoint-dirty-document-scope-object-generation-async-operation.md)。生产代码已接入，原生测试与性能门槛仍开放。

## 已接入的修复

- `changed_histories_after` 已返回按 `HistoryContextId` 排序的唯一 journal IDs，Reset 来自有序 `BTreeMap` keys。
- 对 cursor 记住的上一 pending ID 与当前 pending ID，最多进行两次 binary search；只有缺失时有序插入。替代追加后第二轮排序/去重，重复 pending 不会先增长 Vec 再丢弃。
- 保留 dirty state、generation、Reset/Delta、取消与切换 pending、volatile 过滤、cursor 验证和 journal visit 语义。没有修改跨 history scope 准入。
- 原 7 个公共用例原样保留，新增 `pending_history_transitions_preserve_sorted_unique_dirty_batches`，用真实不同顺序提交、pending/cancel/switch 验证排序、唯一性、dirty/generation、visit count 和取消后的实际值。

独审通过；补丁 SHA256 `a82ab8db999d99cfb05a130effbeb6940f650cc39fbac529eca683f057b8279e`。实际接入回执 `editor-next-core-v1/actual-canonical-apply.json` 与独审 `editor63-core-review-v1/review.json`，位于 `E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/`。格式/diff 检查退出码 0，内容符合审查哈希，索引与控制文件未变。

原 `optimization_batch_20260826o_editor63_dirty_journal_hash_dedup_performance_evidence`、标记 `EDITOR63_DIRTY_JOURNAL_HASH_DEDUP_BENCH_V1` 与 P95 ≤ 60% 的原门槛完整保留，仍需原生 Release 执行。源码操作次数减少不证明延迟达标，也不关闭原计划的 1/10/100 文档、100K 命令、适用的 1GB history、恢复/崩溃/产品整体验收。

## 异步验证状态

上一 16 命令批次的 `sourcechanged-refusal` 历史记录保留：首条命令在调用 Cargo 前被拒绝，其余命令未调用；该批没有原生测试通过证据。

后继请求 `runtime-editor-source-repair-batch-20261009-02` 的 `batch-native-v2` 已落盘真实失败终态：首条 `three-package-check` Cargo 原生编译以退出码 `101` 失败，结束时间为 `2026-10-09T04:10:34.317492+00:00`；`terminal-receipt.json` 的 `sourceUnchanged=true`。`actual-compile-errors.json` 共 42 条 JSON entries：24 个迁移后缺失 `.zui` 资源的 `include_str!` 诊断与 4 个 E0282 推断级联构成 `zircon_editor` lib test 的 28 个错误，`editor_world_sync_watch_map` integration test 有 12 个原始 source errors，另含 2 条 crate summary，合计 40 条原始 source diagnostics。后续 11 个命令均为 `not-invoked-prerequisite-failed`；`nativeTestsRun=false`，native、性能、里程碑与 Jenkins 验收均保持开放。

证据：`E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/batch-native-v2/terminal-receipt.json`、`actual-compile-errors.json`、`01-three-package-check.log`（log SHA256 `f3c8c06d419a5f7a9a338e49195498d311b80536ccf53a1e84b224d947f26455`）。下一批次待修复再绑定：先交付并审查两个最小 core 修复（Editor assets 迁移引用、`ServiceHandle`/`Result` test 契约），再用保持 `sourceUnchanged` 的回执重新调用原生批次；在此之前不把本失败批次当作原生或性能通过证据。

- [x] 有序唯一 pending 合并、公共回归、独审、格式/diff 与实际接入。
- [ ] 相关原事务、observation 及新回归原生通过。
- [ ] 原 Release producer 的原 P95 门槛实际通过。
- [ ] 原 Editor63 全规模、所有权、恢复与产品性能指标达标。
