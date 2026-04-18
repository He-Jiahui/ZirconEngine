---
related_code:
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/shaders/uploader.wgsl
  - zircon_graphics/src/tests/virtual_geometry_gpu.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/shaders/uploader.wgsl
plan_sources:
  - user: 2026-04-18 下一步是更深的 unified-indirect / residency-manager cascade，把同一套 frontier truth 继续推进到真实 GPU uploader / page-table / split-merge frontier policy
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/sessions/20260417-1415-m5-vg-slot-assignment-ownership.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-explicit-frontier-rank-uploader-and-page-table-cascade.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-explicit-replacement-runtime-host-and-stats-closure.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_gpu.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_gpu_uploader_readback_rejects_stale_explicit_recycle_slot_contract -- --nocapture
doc_type: milestone-detail
---

# M5 Virtual Geometry Stale Explicit Recycle-Slot Contract Guard

## Goal

把 `Virtual Geometry` 已经显式化的 `assigned_slot + recycled_page_id` frontier truth 再向真实 GPU uploader 下沉一层：如果 prepare contract 声称“回收 page X 的 slot Y”，而当前 GPU page table 里 `slot Y` 已经不再属于 `page X`，uploader 必须拒绝这条请求，而不能偷偷替换掉另一个 resident page。

## Non-Goal

- 本轮不重写 runtime prepare 的 assigned-slot 规划。
- 本轮不引入新的 GPU residency scoring tree。
- 本轮不把 implicit fallback slot path 全部改成显式 replacement source；这里只收紧 explicit contract。

## Delivered Slice

### 1. GPU uploader 现在会校验 explicit recycle-slot contract

`uploader.wgsl` 新增了两层显式校验：

- `slot_owner_page_id(...)`
- `request_matches_explicit_slot_contract(...)`

当 `PendingRequestInput` 自带：

- `assigned_slot != u32::MAX`
- 且 `recycled_page_id != u32::MAX`

时，shader 会先确认当前 page-table 里这个 slot 的真实 owner 仍然等于该 `recycled_page_id`；如果不一致，这条 request 不再进入本轮 completion 选择。

### 2. Stale contract 不再污染真实 page-table completion

旧行为是：

- request 显式指定 `slot 2`
- 即便 `slot 2` 当前 owner 已经不是 request 记录里的 recycled page
- GPU 仍会直接替换 `slot 2`
- readback 同时还会回传旧的 `recycled_page_id`

这会让三个真值发生背离：

- 当前 GPU page-table 真正被替掉的是谁
- readback 回来的 explicit replacement truth
- runtime host 随后消费的 completion truth

现在 stale request 会被跳过，因此 page-table、readback 和 runtime 后续消费链不再因为 slot owner 漂移而互相打架。

### 3. Frontier continuation 现在优先推进“仍然有效”的下一条 request

本轮没有把 stale request 直接硬失败到整轮 uploader 停摆，而是让 selection 继续扫描下一条仍满足：

- byte budget
- page budget
- explicit slot contract

的 pending request。

这意味着更深的 residency cascade 现在开始具备一条更可靠的原则：

- 显式 frontier truth 一旦失效，就宁可跳过该 request
- 也不允许用错误 replacement truth 继续推进 page-table completion

## Why This Slice Exists

上一轮已经补齐了两条关键闭环：

- explicit `recycled_page_id` 会从 GPU readback 进入 runtime host 与 façade stats
- requested lineage / streaming-target lineage 已经在 visibility collapse policy 上开始精确分流

但 GPU uploader 本身仍然有一个断口：

- explicit `assigned_slot + recycled_page_id` 只是“被写进了请求”
- shader 还没有验证这对 contract 是否仍然和当前 page-table 一致

一旦 slot owner 在 prepare 和真正 dispatch 之间发生漂移，系统就会：

- 用错误 slot 继续完成上传
- 覆盖错误 resident page
- 回传错误 replacement truth

本轮补上的就是这条 prepare contract -> GPU submission validity 的最后一道 guard。

## Validation Summary

- `virtual_geometry_gpu_uploader_readback_rejects_stale_explicit_recycle_slot_contract`
  - 证明当 request 声称“回收 page 800 的 slot 2”，但当前 page-table 的 `slot 2` 实际已经属于 `page 900` 时，uploader 会跳过这条 stale request，并继续完成下一条仍然有效的 request

## Remaining Route

- 继续把 unified indirect / residency-manager cascade 的 shared frontier truth 下沉到更完整的 GPU submission authority。
- 继续推进 split-merge frontier policy，让更深 hierarchy 切换时的 request / recycle / raster authority 共享同一套真实 owner truth。
