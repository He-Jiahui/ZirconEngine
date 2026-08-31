# Runtime revision-bound text snapshot lease (2026-08-27)

## 状态

`revision_bound_snapshot_lease_implemented_unvalidated /
single_flatten_per_requested_revision_implemented_unvalidated /
source_index_secondary_source_copy_removed_unvalidated /
document_debug_source_redacted / product_registry_open /
snapshot_budget_and_managed_profile_pending`

## Current-source 缺陷

内部 `TextDocument` 已有 piece storage 和 revisioned edit receipt，但连续正文只能通过
`snapshot() -> String` 获取。每个调用都会重新展平全部 piece；grapheme source index 随后又以该临时
`String` 扫描。未来 layout、IME、accessibility 和 host consumer 若分别调用，会让同一 revision
发生多次 `O(N)` 复制，并且没有对象能证明自己持有的是哪个 revision。派生 `Debug` 还会输出
original/addition 正文，不满足 secure text 的日志边界。

## 已实现基础设施

- document 每个 revision 持有一个惰性 `OnceLock<Arc<str>>` 连续快照。
- 初始 revision 直接复用 original `Arc`，不复制。
- document authority 自行签发 public UUID；`snapshot_lease()` 返回 UUID + typed revision +
  internal cache key + shared source，同 revision 重复请求只 clone `Arc`。
- 真实 edit 在所有可失败 prepare 通过后使新 revision 的连续快照槽失效；旧 lease 继续稳定持有旧
  `Arc`。typed no-op 不失效当前 lease。
- source index 直接借用 lease 构建 grapheme boundaries，不再先复制临时 `String`。
- document/lease 自定义 `Debug` 只输出 identity、revision、长度和块计数，不输出正文。

## 算法与成本

piece edit 的局部存储算法不变。只有 consumer 明确请求连续快照时，新 revision 才首次执行一次
`O(N)` 展平并保留一份 `N` byte `Arc<str>`；后续 lease 为 `O(1)` 引用计数操作。source index
仍需 `O(N)` 扫描 grapheme boundaries，但不再额外分配/复制一份 N-byte 正文。

这是一项结构性复杂度收敛，尚无运行测量。不得据此声称大文档编辑、RSS、功耗或与 Unreal 对标
已经达标。

## 开放项

- `TextDocument` 尚无产品 registry、Surface/widget consumer 或 secure owner；UUID 已绑定到
  authority，但还没有 service lifecycle/admission。
- lease 同时保留 cache-oriented `TextDocumentKey` 和 public UUID/revision；cache key 不得穿过
  产品边界。
- 尚无 snapshot byte budget、lease count/deadline、old revision retention policy 或 zeroization。
- grapheme index 仍全文重建；stable marker 和 incremental index repair 未实现。
- legacy `snapshot() -> String` 仍为显式复制 API，待 consumer 迁移后再评估删除。
- Cargo、allocation/RSS profile、功耗与 WGPU 产品验证均待执行。
