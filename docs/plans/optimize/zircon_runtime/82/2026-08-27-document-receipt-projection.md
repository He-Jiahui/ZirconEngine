# Runtime document receipt projection preflight (2026-08-27)

## 状态

`document_authority_uuid_bound_unvalidated /
content_free_changed_receipt_projection_implemented_unvalidated /
wire_receipt_deserialization_validation_implemented_unvalidated /
runtime_document_service_gateway_open / surface_consumer_open /
managed_validation_pending`

## Defect

仅有公共 receipt DTO 仍不足以形成正确 producer。若 projection 由调用方传入 document UUID 或新文档
长度，同一内部 change 可以被误标为另一个 document，错误长度还会让 range/selection bounds 检查
失真。只提供 `validate()` 也依赖每个 wire consumer 主动调用；无效 nested widget event 可先被
反序列化并进入业务代码。

## Implemented

- 非 Clone `TextDocument` 在构造时自行签发并持有 public UUID。
- revision-bound snapshot lease 暴露该 UUID 与 typed revision；内部 `u64 owner + revision`
  只作为现有 cache reuse key。
- changed receipt 持有 authority UUID、previous/current key、previous/current byte length、dirty
  ranges 与 length delta。
- `project_public` 不接受 document id 或 document length 参数；调用方只提供 node、typed
  source/kind 和最终 selection。
- projection 检查 owner 不变、revision 严格相邻、dirty-range delta 与 document-length delta
  一致、offset 可精确收窄为 `u32`、old/new range 及 selection 分别落在对应 revision 长度内。
- public byte selection 保留 focus affinity。
- `UiTextEditReceipt` 反序列化通过同一 `validate()`；无效 schema、nil UUID、revision 和
  range 在 nested `UiWidgetEvent` 入口 fail closed。

## Cost

projection 只读取固定数量标量并构造固定大小 DTO，对 document length 为 `O(1)`；不读取、hash
或复制 source，不创建动态 label。wire validation 同样为固定字段检查。

## Open

- 尚无 bounded `TextDocumentService` registry/session lifecycle 或 Surface edit gateway。
- changed receipt 的 source equality 和 grapheme-boundary 真实性仍由 document owner 的 edit
  preflight保证；wire consumer无法仅凭无正文 receipt重验，必须使用 revision-bound snapshot。
- public offsets 当前为 `u32`，超过 4 GiB 的 document projection明确拒绝；最终产品 byte
  admission 应远低于该上限并由 profile/内存预算校准。
- history/undo/redo、focused binding rebase、secure document owner、ABI/Cargo/profile/WGPU均开放。
