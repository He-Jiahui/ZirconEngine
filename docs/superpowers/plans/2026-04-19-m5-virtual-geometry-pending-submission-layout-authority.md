---
related_code:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
plan_sources:
  - user: 2026-04-19 deeper visibility-owned unified indirect / cluster-raster submission ownership
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-19-m5-virtual-geometry-visibility-owned-submission-execution-order.md
tests:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - cargo test -p zircon_graphics --offline --locked shared_indirect_args_layout_preserves_authoritative_pending_submission_order_even_when_offsets_collapse -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_submission_execution_order -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_render -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_submission_authority -- --nocapture
  - cargo check -p zircon_graphics --lib --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Pending Submission-Layout Authority

## Goal

把 `Virtual Geometry` 里 shared indirect args 已经 authoritative 之后还残留的 CPU submission reconstruction，再往前收一层：

- 之前 `build_shared_indirect_args_buffer(...)` 只回填 `indirect_args_offsets`
- `build_mesh_draws(...)` 再靠 `offset` 排序，把最终 `MeshDraw` 顺序从 compaction 结果里反推回来
- 一旦多个 pending draw 因为 shared compaction 落到同一 offset，这条顺序就只剩 “offset + original_index” 这种 CPU 派生真值

这轮目标不是再做一套新的 GPU path，而是先把 shared indirect 的 CPU 规划拆成显式布局层，直接导出“每条 pending draw 的 authoritative submission order”，让 mesh-build 消费 direct authority，而不是只靠 compacted offset 回推。

## Delivered Slice

### 1. 红灯锁定 “compaction 后顺序只剩 offset”

新增纯布局红测：

- `shared_indirect_args_layout_preserves_authoritative_pending_submission_order_even_when_offsets_collapse`

它构造了两条拥有相同 `(mesh_index_count, segment_key)` 的 pending draw：

- 这两条 draw 仍应保留各自的 authoritative pending submission rank
- 但它们又必须继续共享同一条 compacted indirect args offset

实现前根本没有独立的 layout 层，也没有 `pending_draw_submission_orders` 可导出，因此测试直接红掉。

### 2. shared indirect build 拆成纯布局层

`build_shared_indirect_args_buffer(...)` 现在不再一边算布局一边直接创建 GPU buffer，而是先走：

- `build_shared_indirect_args_layout(...)`

这层纯 CPU layout 会统一产出：

- `segment_inputs`
- `draw_refs`
- `indirect_args_offsets`
- `pending_draw_submission_orders`

因此 shared indirect 的 CPU 规划终于不再只剩 “buffer side-effect + offset 回填”，而是显式暴露出一份可测试、可直接消费的 submission truth。

### 3. mesh-build 改为按 direct submission order 排序

`build_mesh_draws(...)` 现在会优先消费：

- `pending_draw_submission_orders`
- 再以 `indirect_args_offset`
- 最后才退到 `original_index`

也就是说 renderer 末端的 `MeshDraw` 顺序已经从“靠 compacted offset 反推”变成“先按 visibility-owned pending submission order 直排，再用 offset 只做 secondary stability”。

这条变化虽然对现有通过的离屏行为大多保持兼容，但它把剩余 CPU 排序残留明显收窄到了 direct authority 主链里，而不再让 compaction offset 本身继续充当第一排序真值。

## Why This Slice Matters

当前 `Virtual Geometry` 的 unified indirect authority 已经一路下沉到了：

- prepare-owned unified indirect order
- shared segment / draw-ref / args source
- GPU-generated args
- cluster-raster coverage

但 `MeshDraw` 列表自身仍然主要靠 `indirect_args_offset` 重建。

这会让 renderer 的最终 submission order 仍然对 compaction 结果过度耦合，尤其在“多个 pending draw 共用同一 args record”时，剩下的排序真值又会退回 CPU 原始插入顺序。

补上显式 `pending_draw_submission_orders` 之后：

- shared indirect 的 CPU 规划有了单独可测的 authority layer
- compaction 只负责“哪条 draw 共享哪条 args”
- submission order 则继续由 layout 里显式导出的 visibility-owned rank 主导

这正是继续把 unified-indirect truth 压进更深 submission ownership 前，必须先补齐的一层结构收口。

## Validation Summary

- 红灯
  - `cargo test -p zircon_graphics --offline --locked shared_indirect_args_layout_preserves_authoritative_pending_submission_order_even_when_offsets_collapse -- --nocapture`
- 绿灯
  - `cargo test -p zircon_graphics --offline --locked shared_indirect_args_layout_preserves_authoritative_pending_submission_order_even_when_offsets_collapse -- --nocapture`
- 回归
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_submission_execution_order -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_render -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_submission_authority -- --nocapture`
  - `cargo check -p zircon_graphics --lib --offline --locked`

## Remaining Gaps

- 这轮把 CPU-side submission reconstruction 收窄成显式 layout authority，但 render passes 仍然是一条 `MeshDraw` 一次 `draw_indexed_indirect(...)` 的 CPU-issued submission；更深的 GPU-generated args source / cluster-raster execution ownership 仍然值得继续推进。
- `Virtual Geometry` 仍然还差更深的 unified indirect / cluster raster / residency-manager cascade，尤其是把这套 direct submission authority 继续压进更真实的 GPU-driven cluster-raster consumption，而不只停在 current mesh-build layout 层。
