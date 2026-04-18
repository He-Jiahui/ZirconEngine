---
related_code:
  - zircon_graphics/src/runtime/virtual_geometry/prepare_frame/pending_page_requests.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/shaders/uploader.wgsl
  - zircon_graphics/src/types/virtual_geometry_prepare/request.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_gpu.rs
implementation_files:
  - zircon_graphics/src/runtime/virtual_geometry/prepare_frame/pending_page_requests.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/shaders/uploader.wgsl
plan_sources:
  - user: 2026-04-18 仍然是 Virtual Geometry 更深的 split-merge frontier policy / residency-manager cascade
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/sessions/20260417-1415-m5-vg-slot-assignment-ownership.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-hot-frontier-runtime-residency-cascade.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_gpu.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_keeps_frontier_recycle_preference_for_later_requests_without_assigned_slots -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_gpu_uploader_readback_preserves_frontier_recycle_preference_after_stale_requests_are_skipped -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_frontier_runtime -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_gpu -- --nocapture
  - cargo test -p zircon_graphics --offline --locked visibility -- --nocapture
  - cargo check -p zircon_graphics --lib --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Fallback Recycle Preference Uploader Cascade

## Goal

把 `Virtual Geometry` 的 split-merge frontier truth 再向真实 GPU uploader fallback path 下沉一层，补齐下面这条缺口：

- runtime prepare 已经能为更早 request 生成 frontier-aware recycle slot
- 但当这些更早 request 在 GPU 侧因为 stale contract 被跳过时
- 更晚、当下没有 `assigned_slot` 的 request 仍可能退回 raw `evictable_slots` 顺序

这样会让 fallback submission 脱离当前 frontier / lineage 语义，甚至回收错误的 slot。

## Non-Goal

- 本轮不改 page-table 数据格式。
- 本轮不重写 `VisibilityContext` 的 hierarchy refine / request scoring。
- 本轮不引入新的 indirect buffer、page residency allocator 或 GPU-driven compaction pass。

## Delivered Slice

### 1. Prepare request 现在会保留 fallback recycle preference

`pending_page_requests(...)` 现在不只为“当前能立刻拿到 slot”的 request 生成 `recycled_page_id`。

如果 request 因为前面的 request 已经暂时占满当前 `evictable_pages`，导致本帧 `assigned_slot == None`，runtime host 仍会把：

- 按当前 frontier 顺序
- active request lineage 保护
- ancestor / descendant distance
- hot frontier hold

得到的最优 recycle candidate 继续写进 `VirtualGeometryPrepareRequest.recycled_page_id`。

这意味着 later request 不会因为当下没 slot，就把自己的 frontier-aware recycle 真值完全丢掉。

### 2. GPU uploader 现在会先尝试这条 preferred recycled page

`uploader.wgsl` 现在把 `recycled_page_id` 扩成两种语义：

- `assigned_slot + recycled_page_id` 同时存在时：仍然是显式 recycle-slot contract
- 只有 `recycled_page_id` 存在时：它表示 implicit fallback path 的 preferred recycled page

因此当 request 没有显式 `assigned_slot` 时，GPU uploader 会先尝试：

- 从当前 page table 找到 `recycled_page_id` 还在占用的 slot
- 如果还存在，就优先复用这个 frontier-preferred slot
- 只有这条 preferred page 已经不再 resident 时，才回退到 raw evictable-slot scan

### 3. Raw evictable fallback 现在会跳过本帧已经被更早 completion 复用过的 slot

`uploader.wgsl` 新增了 completed-slot 检查。

这解决了一个更深的 submission-level 问题：

- 更早 request 可能用显式 contract 成功复用某个 evictable slot
- 更晚 request 若仍按 raw `evictable_slots[0]` 回退
- 就可能再次拿到刚刚被更早 completion 占用的 slot

现在 raw fallback 会跳过这些已经在当前 completion pass 中被写入的 slot，避免 later request 反向踢掉更早、语义上更靠前的 frontier upload。

## Why This Slice Exists

上一轮 hot-frontier runtime residency cascade 已经把：

- `hot_resident_pages`
- current-frame completion recycle order
- next-frame prepare recycle plan

三者接通。

但 frontier truth 仍然在一个分支上会断掉：

- prepare 只能为前面几条 request 显式分配 recycle slot
- 更晚 request 一旦拿不到 `assigned_slot`
- GPU uploader 遇到前面 request 被 stale contract 跳过时
- fallback path 会退回 raw `evictable_slots` 顺序

结果是：

- request 虽然还处在同一 visibility-owned frontier 下
- 但真实 GPU submission 可能回收到错误的 resident lineage
- 甚至在同一 completion pass 内复用已经被更早 request 刚占走的 slot

本轮补上的就是这条 “prepare frontier truth -> GPU fallback slot selection” 的断层。

## Validation Summary

- `virtual_geometry_runtime_state_keeps_frontier_recycle_preference_for_later_requests_without_assigned_slots`
  - 证明 runtime prepare 会为 `assigned_slot == None` 的 later request 保留 frontier-aware `recycled_page_id` 偏好，而不是直接丢失 recycle 真值。
- `virtual_geometry_gpu_uploader_readback_preserves_frontier_recycle_preference_after_stale_requests_are_skipped`
  - 证明当前面 stale explicit request 被 GPU 跳过时，later fallback request 仍会复用 preferred recycled page 对应的 slot，而不会退回 raw evictable-slot 顺序。
- `virtual_geometry_frontier_runtime`
  - 证明 hot-frontier / active-lineage / frontier-order 的 runtime cascade 仍然成立。
- `virtual_geometry_runtime`
  - 证明 prepare snapshot、GPU completion host merge、residency cascade 与新的 fallback recycle preference 一起通过。
- `virtual_geometry_gpu`
  - 证明 uploader 的 explicit contract、implicit replacement、budget arbitration 与新 fallback path 兼容。
- `visibility`
  - 证明 visibility-side split/merge / collapse / hysteresis 行为未被本轮 request contract 语义扩展破坏。
- `cargo check -p zircon_graphics --lib --offline --locked`
  - 证明 `zircon_graphics` 当前库面仍保持 compile closure。

## Remaining Route

- 把同一套 frontier truth 继续推进到更深的 unified indirect / cluster raster / draw-ref ownership，而不只停在 uploader fallback slot 选择。
- 继续收敛更完整的 split-merge frontier policy，让 page residency、prepare contract、GPU completion 与 indirect submission 共用更少分叉语义。
- 继续推进更深层 residency-manager cascade，包括更真实的 page-table / uploader / submission authority 一致化。
