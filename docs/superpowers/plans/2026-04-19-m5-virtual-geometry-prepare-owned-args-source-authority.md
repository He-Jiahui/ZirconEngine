---
related_code:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_graphics/src/tests/mod.rs
  - zircon_graphics/src/tests/virtual_geometry_args_source_authority.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_authority.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_gpu.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_graphics/src/tests/mod.rs
  - zircon_graphics/src/tests/virtual_geometry_args_source_authority.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_authority.rs
plan_sources:
  - user: 2026-04-19 是把这套 authority 继续压进真正的 visibility-owned / GPU-generated args source
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-19-m5-virtual-geometry-prepare-owned-segment-source-authority.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_args_source_authority.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_authority.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_gpu.rs
  - cargo test -p zircon_graphics --offline virtual_geometry_args_source_authority -- --nocapture
  - cargo test -p zircon_graphics --offline virtual_geometry_submission_authority -- --nocapture
  - cargo test -p zircon_graphics --offline virtual_geometry_unified_indirect -- --nocapture
  - cargo test -p zircon_graphics --offline virtual_geometry_prepare_render -- --nocapture
  - cargo test -p zircon_graphics --offline virtual_geometry_runtime -- --nocapture
  - cargo test -p zircon_graphics --offline virtual_geometry_gpu -- --nocapture
  - cargo check -p zircon_graphics --lib --offline
doc_type: milestone-detail
---

# M5 Virtual Geometry Prepare-Owned Args Source Authority

## Goal

把 unified indirect 的 authority 再往下压一层：不仅 segment buffer 先吃 prepare/visibility truth，连 `draw_ref_buffer / indirect args` record 的 existence truth 也尽量先由 prepare-owned visibility contract 决定，而不是继续只从 renderer `pending_draws` 里反推。

## Delivered Slice

### 1. authoritative draw-ref records 现在先从 scene mesh + prepare-owned cluster draws 生成

`build_mesh_draws(...)` 现在除了扁平化 authoritative `segment_key`，还会先按：

- `frame.scene.scene.meshes`
- `streamer.model(...).meshes`
- `build_context.virtual_geometry_cluster_draws`

生成一份 authoritative `VirtualGeometryIndirectDrawRef` 列表。

这条列表不再依赖某条 mesh draw 最终有没有进入 `pending_draws`。

### 2. shared indirect args build 现在先保留 authoritative args records

`build_shared_indirect_args_buffer(...)` 现在先把 authoritative draw refs 排序、去重并写入 `draw_ref_buffer`，随后才把 `pending_draws` 中仍然不存在的 key 作为 fallback 补进去，并为真实 pending draws 回填 `indirect_args_offsets`。

因此当前 contract 变成：

- `segment_buffer`: prepare-owned authoritative segment truth
- `draw_ref_buffer`: prepare-owned authoritative draw-ref truth，再并上 pending-only fallback
- `indirect_args_offsets`: 真实 mesh draw 仍然只绑定 drawable subset

### 3. 真实提交数和真实 args source 现在可以继续分离

这一刀以后，允许出现：

- `last_virtual_geometry_indirect_draw_count == 1`
- `last_virtual_geometry_indirect_args_count == 2`

这种“实际提交 draw 数仍然是 drawable subset，但 shared args source 已经保留更宽的 prepare-owned visibility truth”的状态。

这正是下一步把 args source 继续推向真正 GPU-generated / visibility-owned execution 的过渡形态。

## Why This Slice Matters

上一刀只把 segment existence 从 renderer `pending_draws` 里剥离出来，但 `draw_ref / args` existence 仍然属于 renderer。

这会留下另一条裂缝：

- prepare 拥有 segment truth
- renderer 仍拥有 args truth

本轮把 draw-ref / args existence 也往 prepare-owned visibility contract 靠拢之后，renderer 在 unified indirect 链路里的剩余职责继续缩小成：

- 为真实 drawable subset 回填 offsets
- 保留 pending-only fallback
- 消费 GPU-generated args buffer

## Validation Summary

- red -> green
  - `virtual_geometry_args_source_keeps_prepare_owned_draw_refs_when_some_entities_do_not_emit_pending_draws`
- regressions
  - `virtual_geometry_submission_authority`
  - `virtual_geometry_unified_indirect`
  - `virtual_geometry_prepare_render`
  - `virtual_geometry_runtime`
  - `virtual_geometry_gpu`
  - `cargo check -p zircon_graphics --lib --offline`

## Remaining Route

- 继续把当前 prepare-owned args source authority 下沉到真正的 visibility-owned / GPU-generated args compaction，让 args cardinality 与 actual draw submission 都不再主要由 CPU-side pending draws 驱动
- 继续推进 deeper cluster-raster consumption，让同一套 args truth 更直接控制 GPU-driven raster execution
- 或切回更深 split-merge frontier residency cascade，把 final page-table / completion 真值推进到更完整的 residency-manager policy
