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
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-explicit-replacement-runtime-host-and-stats-closure.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-stale-explicit-recycle-slot-contract-guard.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_gpu.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_gpu_uploader_readback_reports_actual_recycled_page_for_implicit_evictable_slot_reuse -- --nocapture
doc_type: milestone-detail
---

# M5 Virtual Geometry Implicit Slot Replacement Readback

## Goal

把 `Virtual Geometry` GPU uploader 的 replacement truth 再收紧一层：即使 request 没有显式携带 `recycled_page_id`，只要本帧 uploader 实际复用了一个 occupied evictable slot，readback 也必须回传真实被替掉的 resident page。

## Non-Goal

- 本轮不修改 runtime prepare 的 slot 规划算法。
- 本轮不强行要求所有 request 都预先带显式 `recycled_page_id`。
- 本轮不重写 page-table snapshot decode；只补 completion 输出真值。

## Delivered Slice

### 1. Implicit evictable-slot reuse 现在也会产出 explicit replacement truth

`uploader.wgsl` 在真正选定 `assigned_slot` 之后，新增了这条逻辑：

- 如果 request 自己已经带了 `recycled_page_id`
  - 继续保留这条显式 contract
- 如果 request 没带 `recycled_page_id`
  - 则从当前 GPU page table 里查询 `assigned_slot` 的真实 owner
  - 并把它写回 `completed_pages[page_id, slot, recycled_page_id]`

因此 completion readback 不再只在“prepare 已显式分配 recycle slot”这一路有 replacement truth。

### 2. Runtime host 不再只能靠 page-table aliasing 推断 fallback recycle

上一轮已经把 `completed_page_replacements` 接到了：

- runtime host completion
- façade stats

但 implicit slot reuse 之前仍然会留下一个空洞：

- page-table snapshot 知道 slot 被谁替掉了
- completion replacement 列表却是空的

这意味着 runtime host 只能重新回到：

- slot aliasing 推断
- page-table diff 间接回推

现在这条 fallback 路径也开始携带显式 recycled-page truth，completion 主链不必再依赖推断。

### 3. Explicit contract guard 与 implicit fallback truth 现在拼成完整 uploader 语义

两条 slice 合起来后，uploader 当前的语义已经收敛成：

- explicit `assigned_slot + recycled_page_id` 失效时，跳过 stale request
- implicit evictable-slot reuse 成功时，回传真实 recycled page

也就是：

- 不再接受错误的显式 replacement truth
- 也不再丢掉实际发生的隐式 replacement truth

## Why This Slice Exists

只做 stale explicit guard 还不够，因为 uploader 仍然存在另一条真实路径：

- request 未显式声明 recycle target
- GPU 仍然从 evictable slot 里挑中了一个 occupied slot
- 真实 replacement 已经发生
- 但 readback 不回传对应 `recycled_page_id`

这样 runtime host 虽然可以继续靠 page-table snapshot 存活，但整条 “GPU completion -> runtime host -> stats” 的 explicit truth 链仍然不完整。

本轮补上的就是这条 implicit replacement source。

## Validation Summary

- `virtual_geometry_gpu_uploader_readback_reports_actual_recycled_page_for_implicit_evictable_slot_reuse`
  - 证明当 pending request 没有显式 `recycled_page_id`、但 uploader 实际复用 occupied evictable slot 时，readback 现在会回传真实 `completed_page_replacements(page_id, recycled_page_id)`

## Remaining Route

- 继续把 uploader/page-table/shared frontier truth 收进更深的 split-merge frontier policy 与 residency-manager cascade。
- 继续推进 unified indirect / cluster raster 的 owner truth，让 request / recycle / raster 在更深 hierarchy 切换时共享同一套 authoritative source。
