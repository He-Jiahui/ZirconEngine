# Runtime versioned document edit receipt hard cut (2026-08-27)

## 状态

`public_text_edit_snapshot_event_removed_unvalidated /
versioned_document_edit_receipt_contract_implemented_unvalidated /
internal_document_receipt_projection_implemented_unvalidated /
runtime_document_service_gateway_open / product_snapshot_lease_integration_open /
m1_document_authority_open`

## 结构性缺陷

旧公共 `UiTextEdit` 把 raw `UiTextEditAction` 以及完整
`before/after UiEditableTextState` 一并序列化进 `UiWidgetEvent`。该 DTO 没有稳定
document identity、revision fence 或 schema version；一次编辑事件会复制两份正文和 composition
状态，成本随文档长度线性增长，同时扩大普通/secure 编辑内容的公共暴露面。current-source 没有
runtime、App 或 Editor producer，因此该 DTO 也没有形成可用的产品 document transaction。

## 已实现合同

- `TextEditChange` 只携带 `UiTextEditReceipt`。
- receipt 包含 schema version、document UUID、previous/current revision、node、typed
  source/kind、old/new byte range 和带 focus affinity 的最终 byte selection。
- 旧 `UiTextEdit` 以及事件中的 `edit` snapshot 字段硬删除，不保留 deprecated alias。
- `UiTextEditAction` 只作为进程内输入 intent，不穿过公共 notification 边界。

## 验证语义

接口层拒绝未知 schema、nil document id、revision 跳变或耗尽、old/new 反向 range。内部
document authority 现自行签发并持有 UUID；changed receipt 的 `O(1)` 投影不再接受调用方提供的
document id，并校验内部 owner 不变、revision 相邻、length delta 一致、`usize -> u32` 无溢出、
old/new range 与最终 selection 不越界。产品 document service 后续仍必须依据指定 revision
snapshot 验证 grapheme policy 与 source equality；公共 DTO 不能替代 document owner 的 authority。

## 成本与性能边界

receipt 大小和发布成本对 document length 为 `O(1)`，不复制或 hash 正文，不附加动态 label。
UUID 每个 document session 只签发一次，revision 使用 checked consecutive transition。该变更关闭
结构性线性事件复制，不是已经测量的优化；当前没有 CPU、allocation、RSS 或功耗 profile 数据。

## 开放集成

- 内部 document 已自带 public UUID；`TextDocumentKey { owner, revision }` 只保留为现有 cache
  reuse key，不得作为产品 identity。
- changed receipt 已能投影公共 receipt，但 property transaction 仍以 Surface metadata 为
  authority，尚未由产品 gateway 生产该 receipt。
- 内部 revision-bound lease 尚未接 product registry/event；product edit session、history grouping、
  undo/redo、focused binding rebase 未接入。
- secure opaque value reference 尚不是 document session/snapshot lease。
- ABI、runtime/app/editor producer、managed Cargo、profile、功耗与 WGPU 截图仍待验证。

不得把非 Clone 的 `TextDocument` 直接放进 clone/serde `UiSurface`，也不得创建第二份全文 cache
冒充唯一产品 document owner。
