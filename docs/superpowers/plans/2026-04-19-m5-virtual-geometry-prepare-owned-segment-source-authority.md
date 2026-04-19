---
related_code:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_authority.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_gpu.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_authority.rs
plan_sources:
  - user: 2026-04-18 下一条剩余主链仍然是把这套 prepare-owned ordering + renderer-side compaction 继续下沉到真正的 visibility-owned / GPU-generated args source
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-prepare-owned-indirect-order-authority.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_submission_authority.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_gpu.rs
  - cargo test -p zircon_graphics --offline virtual_geometry_segment_buffer_keeps_prepare_owned_segments_when_some_entities_do_not_emit_pending_draws -- --nocapture
  - cargo test -p zircon_graphics --offline virtual_geometry_submission_authority -- --nocapture
  - cargo test -p zircon_graphics --offline virtual_geometry_unified_indirect -- --nocapture
  - cargo test -p zircon_graphics --offline virtual_geometry_prepare_render -- --nocapture
  - cargo test -p zircon_graphics --offline virtual_geometry_runtime -- --nocapture
  - cargo test -p zircon_graphics --offline virtual_geometry_gpu -- --nocapture
  - cargo check -p zircon_graphics --lib --offline
doc_type: milestone-detail
---

# M5 Virtual Geometry Prepare-Owned Segment Source Authority

## Goal

把 shared indirect segment buffer 的 authority 再往上提一层：不再只从 renderer `pending_draws` 反推出 unique segment 列表，而是先直接消费 prepare/visibility 已经固定好的 segment truth，再让 renderer 只负责 drawable subset 的 draw-ref / args compaction。

## Delivered Slice

### 1. shared segment buffer 现在先吃 prepare-owned authoritative segments

`build_mesh_draws(...)` 现在会先从 `build_context.virtual_geometry_cluster_draws` 扁平化出一份 authoritative `VirtualGeometryIndirectSegmentKey` 列表，再把它传给 `build_shared_indirect_args_buffer(...)`。

这意味着：

- segment buffer 的第一来源不再是 `pending_draws`
- prepare/visibility 已经决定的 unified-indirect segment truth 会先进入真实 GPU submission source
- renderer 不再需要靠“先看到哪个 pending draw”来推断 segment 是否存在

### 2. renderer 仍然保留 pending-only fallback，但只作为补集

`build_shared_indirect_args_buffer(...)` 现在会把 authoritative segment 列表与 `pending_draws` 里的 segment key 合并，再统一排序和去重。

因此：

- prepare/visibility authoritative segments 会被无条件保留
- 没有 prepare-owned segment、但 renderer 仍然需要 fallback 的 pending-only segment 也不会丢
- renderer 侧继续只为实际 drawable pending draws 生成 draw-ref / indirect args offsets

### 3. 真实 submission 现在允许“segment 比 draw 多”

这一刀刻意允许 segment buffer 保留比 draw-ref 更多的 entries。

当前语义变成：

- `segment_buffer`: authoritative visibility/prepare truth
- `draw_ref_buffer`: 当前帧真实可提交 draw 的子集映射
- `indirect args`: 只为被 draw-ref 引用的 drawable subset 编码

这样即使某些实体在当前 mesh filtering 下没有生成 pending draw，GPU-submitted segment buffer 仍然会保留它们的 prepare-owned visibility truth。

## Why This Slice Matters

上一刀已经把“第一份排序”提到了 `prepare.unified_indirect_draws()`，但 segment source 本身仍然是 renderer 从 pending draws 重建出来的。

这会留下一个剩余裂缝：

- prepare 拥有 ordering truth
- renderer 仍拥有 segment existence truth

本轮把 segment existence 也前移到 prepare/visibility 之后，renderer 末端就进一步收缩成：

- 消费 authoritative segments
- 为 drawable subset 建 draw-ref
- 做 args compaction

这正好为下一轮继续推进到真正的 visibility-owned / GPU-generated args source 打平地基。

## Validation Summary

- red -> green
  - `virtual_geometry_segment_buffer_keeps_prepare_owned_segments_when_some_entities_do_not_emit_pending_draws`
- regressions
  - `virtual_geometry_submission_authority`
  - `virtual_geometry_unified_indirect`
  - `virtual_geometry_prepare_render`
  - `virtual_geometry_runtime`
  - `virtual_geometry_gpu`
  - `cargo check -p zircon_graphics --lib --offline`

## Remaining Route

- 把当前 prepare-owned segment source authority 继续下沉到真正的 visibility-owned / GPU-generated unified indirect args source，让 args record 本身也不再主要由 renderer compaction 生成
- 继续推进 deeper cluster raster consumption，让更深的 page/slot/frontier truth 不只停在 segment buffer，而是真正控制更完整的 GPU-driven raster submission
- 或切回更深 split-merge frontier residency cascade，把同一套 page-table / completion truth 继续推进到 residency-manager policy
