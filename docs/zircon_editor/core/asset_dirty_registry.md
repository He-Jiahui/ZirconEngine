# Editor Asset Dirty Registry

`zircon_editor::core::asset` façade 的 dirty API 将文档关闭询问和后续 save/save_all 所需脏态统一投影到两个
明确来源：Editor03 transaction engine 的 `saved_top`，以及不属于事务历史的 typed external-effect
ledger。Registry 不保存 transaction dirty bool、history top 或平行保存基线。

## 权威边界

- `DirtyRegistry` 持有共享 `Arc<EditorTransactionEngine>`；单文档 snapshot 仍实时调用 owner，整批 consumer
  使用 Editor03 `HistoryDirtyCursor`。编辑、mark saved、undo、redo 和 branch replacement 都由 Editor03
  journal 发布，不需要 Registry 保存 transaction dirty bool。
- external effect 用 `DirtyExternalEffectId` 标识，例如 `ui.source_buffer` 或
  `asset.import_settings`。ID 是开放的 typed key，但只允许小写 ASCII namespace；不能使用散落裸字符串。
- 每次 external mark 产生单调 `DirtyExternalEffectRevision`。重复 mark 同一 effect 会更新 revision；
  `clear_external_effect` 必须携带预期 revision，旧 completion 不能删除更新后的 effect。
- 文档必须显式 register；未知文档的 query/mark/clear 返回 typed error，不会隐式创建幽灵条目。

## 快照和性能

`DirtyDocumentSnapshot` 同时提供 transaction dirty、按 ID 排序的 external effects 及各自 revision；
`is_dirty` 是两来源的 OR。多文档 consumer 使用 registry-lineage-bound `DirtyRegistryCursor` 调用
`changes_since`：首次或 journal 落后时得到 reset，稳定时 snapshots/removals 都为空，增量时只返回 changed
document 和 typed removals。旧 `snapshots/dirty_snapshots` 已删除，不保留兼容别名。

Registry 用 4,096-entry document journal 记录生命周期/external-effect 变化。首次/reset 才遍历全部注册
文档；external delta 只复制 changed document 的 compact effect pairs，并在 Editor03 transaction delta
之前点查这些文档，随后用更晚的 transaction cursor 覆盖竞态。transaction-only delta 只合并当前注册
文档；无关 history 不会伪装成 unregister。任一 external generation 在查询期间变化时只重试 delta
projection，连续 8 次不能稳定返回 typed error，不返回 false-clean。

1/10,000 文档行为合同锁定 initial reset、stable empty 和 single-change-only delta。Registry 不轮询磁盘、
不读取 `.zmeta`、不构建资产 registry，也不缓存 transaction dirty bool。

## 保存边界

本切片不提供不安全的 `mark_saved` wrapper，旧两调用 API 已物理删除。Editor03 当前唯一合法完成路径是
保存前 `capture_save_token`，写盘成功后 `mark_saved_if_unchanged`；对应 source/review 已完成，但受管 Cargo、
fixed return 与 managed SHA 仍由
`docs/plans/zircon_editor/editor/03/failure-2026-07-22-saved-top-compare-and-mark-save-token.md`
保持 open。

调用方在 I/O 成功后必须使用 Editor03 save token 完成 saved-top；不得自行比较 top。Runtime/asset
sidecar、digest 与 artifact 仍由各自 owner 管理。

## 验证

- 静态合同：`python -m unittest tools.tests.test_editor09_dirty_registry_contract -v`。
- Rust 行为合同覆盖 saved_top 的 edit/save/undo/redo 投影、external effect 排序/清除/revision、未知
  document、typed unregister、并发 delta retry 及 10,000 文档 stable/single-change 规模。
- owner 已私有挂载并从 `core::asset` 重导出 typed API；必须使用 Coordinator01 冻结且闭合 Cargo
  local-path manifest graph 的 source copy 运行聚焦 Cargo，静态挂载合同不替代动态验收。
