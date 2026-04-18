---
related_code:
  - zircon_graphics/src/types/virtual_geometry_prepare/frame.rs
  - zircon_graphics/src/types/virtual_geometry_prepare/indirect_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build_virtual_geometry_cluster_raster_draws.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/virtual_geometry_cluster_raster_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/shaders/virtual_geometry_indirect_args.wgsl
  - zircon_graphics/src/tests/virtual_geometry_submission_authority.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
implementation_files:
  - zircon_graphics/src/types/virtual_geometry_prepare/frame.rs
  - zircon_graphics/src/types/virtual_geometry_prepare/indirect_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build_virtual_geometry_cluster_raster_draws.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/virtual_geometry_cluster_raster_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/shaders/virtual_geometry_indirect_args.wgsl
  - zircon_graphics/src/tests/virtual_geometry_submission_authority.rs
plan_sources:
  - user: 2026-04-18 把 fallback slot authority 继续下沉到 unified indirect / draw-ref / submission ordering，不只停在 uploader fallback 选槽
  - user: 2026-04-18 把同一套 frontier / lineage truth 继续推进到 deeper cluster raster consumption
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/sessions/20260417-1415-m5-vg-slot-assignment-ownership.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-fallback-recycle-preference-uploader-cascade.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_submission_authority.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_submission_authority -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_render -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_gpu -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_frontier_runtime -- --nocapture
  - cargo check -p zircon_graphics --lib --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Fallback Slot Submission Authority Cascade

## Goal

把上一轮已经进入 prepare request / uploader fallback 的 `recycled_page_id` 偏好继续下沉到真正的 renderer submission authority。

这一轮要补的不是“GPU uploader 最终会选哪个 slot”本身，而是下面这条更深的断层：

- prepare request 已经知道更偏好的 recycle target
- uploader fallback 也已经会优先尝试这条 page-table truth
- 但 unified indirect、segment ordering、draw-ref 映射与 pending cluster-raster consumption 仍然把这条 truth 当成不存在

结果就是：

- 真实 submission 仍然会按 raw segment insertion order 编排
- draw-ref 仍然只映射到“谁先被 push 进去”
- pending cluster-raster output 也不会跟随 fallback slot authority 改变

## Non-Goal

- 本轮不改 page-table readback 格式。
- 本轮不重写 GPU uploader selection policy；uploader fallback slot 选择仍然沿用上一轮 contract。
- 本轮不实现真正的 GPU-generated cluster compaction、cluster raster 或 residency manager。

## Delivered Slice

### 1. Prepare 现在会把 request contract 投影成 submission slot authority

`VirtualGeometryPrepareFrame::unified_indirect_draws()` 现在不只继续投影：

- `page_id`
- `frontier_rank`
- `resident_slot`

还会额外投影出 `submission_slot`：

- resident cluster 时，直接沿用当前 `resident_slot`
- pending request 若已有 `assigned_slot`，直接信显式 slot contract
- 否则从 `recycled_page_id -> current resident page slot` 继续解析出 fallback slot authority

这样 `assigned_slot / recycled_page_id / current page-table snapshot` 三条 truth 开始在 prepare projection 层收束成一条统一的 submission-slot contract。

### 2. Unified indirect segment buffer 现在按 submission slot 排序

`build_shared_indirect_args_buffer(...)` 不再把 unique segment buffer 固定绑死在 first-seen insertion order 上。

现在 unique segment 会按：

- `submission_slot`
- `frontier_rank`
- 其余稳定 segment key

排序后再生成实际 GPU-submitted segment buffer。

这意味着：

- fallback slot authority 现在会真实改变 segment ordering
- draw-ref buffer 会把“固定 draw 顺序”映射到“按 slot authority 排好的 segment 索引”
- unified indirect 的 authority 不再只停在 prepare Vec 顺序或 cluster push 顺序

### 3. Pending cluster-raster consumption 现在也消费 submission slot

`virtual_geometry_indirect_args.wgsl` 过去对 pending cluster 只会吃：

- `state`
- `frontier_rank`

而对 fallback slot authority 完全无感。

现在 compute-generated indirect args 会继续消费 `submission_slot`：

- resident path 仍保持原来的 slot-aware resident trim
- pending path 新增 submission-slot trim / offset

因此不同 recycle-slot authority 不只会改变 segment/draw-ref 排序，还会真实改变：

- indirect args
- first/index range
- 最终离屏 cluster-raster coverage

### 4. 回归从 uploader contract 扩到了 submission/read-ref/raster 三层

本轮新增 `virtual_geometry_submission_authority.rs`，覆盖两条关键回归：

- `virtual_geometry_unified_indirect_uses_fallback_recycle_slot_authority_for_submission_order_and_draw_refs`
  - 证明 fallback recycle target 会真实改变 GPU-submitted segment order 与 draw-ref segment index，而不是只停在 uploader fallback path
- `virtual_geometry_prepare_cluster_raster_output_changes_when_fallback_slot_authority_changes`
  - 证明同一 page/state/frontier 下，仅 fallback slot authority 改变，就会真实改变 indirect args 与最终 raster output

## Why This Slice Exists

上一轮 `Fallback Recycle Preference Uploader Cascade` 已经解决了：

- later request 在没有即时 `assigned_slot` 时也能保留 recycle preference
- stale explicit request 被跳过后，uploader fallback 仍会优先尝试 preferred recycled page

但更下游的 renderer 仍然处在“旧世界”里：

- prepare 只把 pending cluster 看成 `resident_slot = None`
- shared indirect segment 仍按原始 push 顺序建 buffer
- pending cluster-raster 仍不消费 fallback slot authority

所以 request / uploader / renderer 三层仍然不是同一条 authority 链。

本轮补上的，就是这条 “prepare request truth -> unified indirect -> draw-ref -> pending cluster-raster” 的 submission closure。

## Validation Summary

- `virtual_geometry_submission_authority`
  - 直接证明 fallback slot authority 现在会进入 GPU-submitted segment ordering、draw-ref mapping 与 raster output
- `virtual_geometry_runtime`
  - 证明 prepare projection 在 resident/pending 旧路径上保持闭环，同时已有 recycle preference / frontier-order 行为没有回退
- `virtual_geometry_unified_indirect`
  - 证明既有 page-owned / lineage-depth / frontier-rank 的 unified-indirect closure 仍然成立
- `virtual_geometry_prepare_render`
  - 证明 cluster-raster fallback/indirect 输出没有因新增 submission-slot consume 而退化
- `virtual_geometry_gpu`
  - 证明 uploader/page-table/replacement 旧路径未被本轮 renderer-side authority 下沉破坏
- `virtual_geometry_frontier_runtime`
  - 证明 frontier-order / hot-page recycle / active-request-lineage 的 runtime cascade 仍然稳定
- `cargo check -p zircon_graphics --lib --offline --locked`
  - 证明 crate 当前 compile closure 仍然成立

## Remaining Route

- 继续把同一条 `submission_slot / page-table / completion` truth 向更深的 residency-manager cascade 推进，而不只停在 prepare projection 与 renderer-local submission。
- 继续推进 split-merge frontier policy，让 ancestor/descendant hierarchy 切换时的 request / recycle / submission / page-table truth 共享更少分叉语义。
- 继续向更真实的 GPU-driven cluster raster / residency manager / visibility-owned indirect execution 演进。
