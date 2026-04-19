---
related_code:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
plan_sources:
  - user: 2026-04-19 visibility-owned / GPU-generated args compaction ownership continuation
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
tests:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - cargo test -p zircon_graphics --offline --locked shared_indirect_args_layout_keeps_distinct_gpu_args_slots_for_repeated_draw_refs -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_keeps_one_visibility_owned_segment_but_distinct_gpu_args_for_multi_primitive_model -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_ -- --nocapture
doc_type: milestone-detail
---

# M5 Virtual Geometry Repeated Draw-Ref GPU Args Authority

## Goal

把 `Virtual Geometry` 的 visibility-owned authority 再往真实 GPU args source 压一层：

- shared segment 早已是 prepare-owned truth
- `submission_index` / draw-ref rank 也已经能写进 GPU-generated args
- 但同一 visibility-owned segment 下的重复 primitive draw 仍会被 `(mesh_index_count, segment_key)` compaction 折叠到一条 args record

这会让 repeated primitive submission 继续依赖 CPU `MeshDraw` 排序残留，而不是让 GPU args source 自己保留 per-draw authority。

## Delivered Slice

### 1. repeated draw-ref 不再共用同一条 args slot

`build_shared_indirect_args_layout(...)` 现在会在 `(mesh_index_count, segment_key)` 组内继续跟踪 occurrence rank。

这意味着：

- segment 仍然共享
- repeated primitive draw 仍然属于同一 visibility-owned segment truth
- 但每个 repeated draw-ref 会拿到独立的 indirect args record，而不再机械落到同一 offset

### 2. authoritative / pending 两侧都按 occurrence 对齐

layout 在 authoritative draw-ref 预热和 pending draw-ref 映射两条路径上，都使用同一套组内 occurrence 规则。

这样 shared args 不会因为 “authoritative 先建表、pending 再映射” 而重新把 repeated primitive collapse 回单条 GPU args slot。

### 3. repeated primitive submission token 真值继续进入 first_instance

现在 repeated primitive draw 会得到：

- 独立的 `indirect_args_offset`
- 独立的 `draw_ref_rank_within_segment`
- 独立的 `IndexedIndirectArgs.first_instance`

因此 repeated primitive submission 已经不再只靠 CPU 排序区分；真实 GPU-generated args source 自己就能保住这条 per-draw authority。

### 4. render stats 也同步切到新 submission truth

`RenderStats.last_virtual_geometry_indirect_args_count` 对 multi-primitive identical draw 的观测值现在会从 `1` 提升到真实的 `2`。

这让 façade 侧看到的是更真实的 GPU args ownership，而不是旧的 CPU-compacted 假象。

## Why This Slice Matters

这轮不是把 renderer 直接改成 GPU multi-draw submission，但它把一类仍然停留在 CPU 排序层的 authority 真值继续推进到了：

- shared args layout
- indirect args offset
- first_instance token
- render stats

因此后续继续推进 deeper cluster-raster submission ownership 时，重复 primitive draw 已经不再是明显的 CPU residual gap。

## Validation Summary

- 绿灯
  - `cargo test -p zircon_graphics --offline --locked shared_indirect_args_layout_keeps_distinct_gpu_args_slots_for_repeated_draw_refs -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_keeps_one_visibility_owned_segment_but_distinct_gpu_args_for_multi_primitive_model -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_ -- --nocapture`

## Remaining Route

- 这轮解决的是 repeated draw-ref 的 GPU args authority，不是完整 GPU-driven execution。
- 下一条更自然的主链仍然是更深的 unified indirect / cluster-raster submission ownership，以及 confirmed `submission_slot / page-table / completion` 继续下沉到 deeper residency-manager cascade。
