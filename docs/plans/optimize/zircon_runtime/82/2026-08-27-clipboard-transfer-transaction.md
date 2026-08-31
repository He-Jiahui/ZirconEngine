# Runtime Text clipboard transfer transaction

日期：2026-08-27

状态：`runtime_transfer_contract_implemented_unvalidated / cut_delete_after_write_ack_implemented_unvalidated / paste_result_route_implemented_unvalidated / stale_and_duplicate_fence_implemented / surface_targeted_dynamic_abi_implemented_unvalidated / app_event_loop_backend_implemented_unvalidated / windows_cf_unicodetext_backend_implemented_unvalidated / clipboard_queue_and_payload_bounds_implemented_unvalidated / non_windows_typed_unsupported / managed_validation_pending`

## 问题与参考结论

旧实现的 `Cut` 先执行 `Delete`，再发布一个没有 result route 的 `WriteText` host request。宿主未消费、拒绝或写入失败时，选区已经永久丢失。`Paste` 只有 `ReadText` 请求，读取结果无法与原 owner、编辑状态或请求关联。

Unreal `SlateEditableTextLayout.cpp` 的 cut 在 `FScopedEditableTextTransaction` 内先调用平台 clipboard copy，再删除选区；paste 在同一事务内读取并插入。Zircon 不能照搬同步平台调用，因为平台对象属于 App，但必须保留相同的不变量：clipboard write 成功前不得提交 cut，异步 result 必须由 transfer identity 与编辑 revision 限定。

## 本次实现

- Runtime Interface 新增 UUID `UiClipboardTransferId`、`Copy/Cut/Paste` intent、`expected_edit_revision`、typed read/write/failure outcome 与低基数 receipt status。
- `UiInputEvent::Clipboard` 成为 host result 的中立入站合同；surface route 将成功 paste 送入现有 constraint owner，将成功 cut 送入现有 edit action，失败不改变文档。
- surface 对每个 editable owner 最多保存一个 pending transfer，只保存 owner/property/intent/revision/security classification，不复制完整文本或选区快照。
- value、caret、selection、composition、read-only、secure policy 或 focus 变化只使当前 pending revision stale；重复、未知、错误 owner、错误 outcome、clone/serde 与 detached owner 均 fail closed。
- secure copy/cut 继续禁止；secure paste result 复用 opaque secure event，并由统一 dispatch redactor 清除 read payload、binding 与公开报告中的原文。
- surface input transaction snapshot 显式捕获 ephemeral clipboard store，使通用 effect rollback 不会遗失或复活错误的 pending 状态。
- `UiInputManager` 按 Surface 收集 host request；队列最多 256 行，同 owner 尚未送出的旧行由新 transfer 取代。dynamic ABI 显式携带 `target_viewport + target_surface`，result 只投递给精确 Surface，避免多 Surface 复用局部 node id 时误路由。
- dynamic host output 继续使用既有 transactional page：未 commit 的 batch 不重新收集、不丢行，单页最多 256 request。App 在 winit event-loop owner 上完成平台操作后才回送 `ZrRuntimeClipboardResultV1`。
- Windows backend 参考 Unreal `FWindowsPlatformApplicationMisc::{ClipboardCopy,ClipboardPaste}`，使用目标 winit Window 的真实 Win32 HWND、`CF_UNICODETEXT`、`GMEM_MOVEABLE` 与显式内存 ownership transfer。`OpenClipboard(NULL)` 因 `EmptyClipboard` 后 owner 语义会使 `SetClipboardData` 失败，已从实现中排除。
- 非 Windows backend 当前显式返回 `Unsupported`。正文按 UTF-8 限制 32 KiB；producer、App read/write 与 Runtime result admission 均检查上限，超限 result 转为 typed `PayloadTooLarge` 并结束对应 transaction。

## 算法与常态成本

| 路径 | 复杂度与内存 |
|---|---|
| 无 pending transfer 的普通文本编辑 | `pending.is_empty()` 后直接返回；不写 revision map、不复制文本 |
| 创建 request | `O(log T)`，`T` 为当前 surface 有 pending 的 editable owner 数；每 owner 最多一项 |
| 首次相关编辑/focus 变化 | `O(log T)`，revision 最多推进一次；同一 stale transfer 的后续属性写不再更新 revision |
| completion | `O(log T)` transfer lookup，加既有 edit/constraint 成本 |
| clipboard payload | copy/cut 仅复制实际选区；paste 只持有当前入站 payload，不保存第二份 pending snapshot |
| manager host queue | 同 owner 替换为 `O(Q)`，`Q <= 256`；drain 为 `O(Q)`，固定容量避免无界积压 |
| dynamic collection/output | `O(S + R)`，`S` 为 Surface 数、`R` 为 queued rows；output 以 256 行分页且 commit/rollback 保留所有权 |
| Windows platform body | UTF-16 scan/encode 与 UTF-8 decode 为 `O(B)`；临时内存 `O(B)`，`B <= 32 KiB UTF-8` |

这不是性能优化里程碑，不声明 CPU、RSS、功耗或 Unreal 数值接近。没有受管 profile 证据前不调整容器、阈值或平台调用策略。

## 静态证据与开放项

Rust 2024 formatting、scoped `git diff --check`、queue/source ownership contract 与最坏 `\u0000` JSON envelope contract 已补；App backend 和 dynamic ABI 已实现，但 Cargo、真实 App/Editor/WOC host、系统 clipboard、timeout/fault injection、性能/功耗和 WGPU 产品验收尚未执行。因此当前只能标记 implementation-unvalidated，不能关闭 Runtime82 clipboard 产品项。本切片没有改变渲染输出，不生成策略目标截图。

后续仍需 window/seat/principal/deadline、MIME 扩展策略、真正 `TextDocumentId + Revision`、teardown/timeout，以及 Windows 系统实机与故障注入；macOS/Linux/Web 需要各自平台 backend，不能以当前 typed `Unsupported` 作为产品完成。
