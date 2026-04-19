---
related_code:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_mesh_draw_build_context.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build_virtual_geometry_cluster_raster_draws.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_graphics/src/tests/virtual_geometry_args_source_authority.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_execution_order.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_graphics/src/tests/virtual_geometry_args_source_authority.rs
plan_sources:
  - user: 2026-04-19 把 virtual_geometry_cluster_draws authority 继续下沉到更真实的 visibility-owned / GPU-generated args source 和 deeper cluster-raster execution
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-19-m5-virtual-geometry-fallback-unified-indirect-downshift.md
  - docs/superpowers/plans/2026-04-19-m5-virtual-geometry-mesh-build-authority-convergence.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_args_source_authority.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_execution_order.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_missing_explicit_segments_do_not_resurrect_cpu_full_mesh_fallback_draws -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_args_source_authority -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_submission_execution_order -- --nocapture
doc_type: milestone-detail
---

# M5 Virtual Geometry Missing-Segment Authority Closure

## Goal

把 `Virtual Geometry` 在 renderer mesh-build 末端残留的 “explicit `Missing` segment entity 仍被 CPU full-mesh fallback 复活” 彻底关掉，让 `prepare -> unified indirect -> cluster-raster draws -> shared segment/draw-ref/args -> actual submission` 在“没有 draw”这件事上也只认同一份 authority。

## Delivered Slice

### 1. `virtual_geometry_cluster_draws == None` 现在代表 authoritative no-draw truth

`extend_pending_draws_for_mesh_instance(...)` 过去在 `Virtual Geometry` 打开但 entity 没有 `cluster_raster_draws` 时，还会补一条 renderer 私造的 `full_mesh_indirect_draw_ref(...)`。  
这条路径现在被删除了。

新的规则是：

- `build_mesh_draw_build_context(...)` 已经把 `prepare` 的 `visible_entities` 收成 VG draw 白名单
- `build_virtual_geometry_cluster_raster_draws(...)` 已经把 `prepare.unified_indirect_draws()` 投影成 authoritative per-entity cluster-raster truth
- 如果某个 visible entity 最终没有 `cluster_raster_draws`，renderer 必须把它当成 “authoritative no-draw”，而不是再发明一条私有 full-mesh fallback draw

### 2. explicit `Missing` segment 不再在 submission 尾端被复活

`VirtualGeometryPrepareFrame::unified_indirect_draws()` 本来就会过滤掉 `state == Missing` 的显式 `cluster_draw_segments`，同时也不会再为这类 entity 合成 fallback slices。  
本轮把这条 contract 继续压到真实 mesh submission：

- `last_virtual_geometry_indirect_draw_count()` 不再为 `Missing` entity 增加 draw
- `read_last_virtual_geometry_indirect_segments()` 不再出现 page `0` / slot `0` / resident 的幽灵 segment
- `read_last_virtual_geometry_indirect_draw_refs()` 与 `read_last_virtual_geometry_indirect_args()` 也不再出现这条 CPU 补出来的 fallback record

### 3. 旧的 renderer-side fallback helper 退出当前 M5 主链

随着这条 closure 落地，`pending_mesh_draw.rs` 里只剩 cluster-draw driven 的 indirect key / input builder。  
旧的 full-mesh fallback key helper 已经不再参与当前 VG submission authority。

## Why This Slice Matters

前一刀已经把 missing-segment fallback truth 前移到 `prepare.unified_indirect_draws()`；但只要 mesh-build 末端还能在 “authoritative source 为空” 时偷偷补一条 draw，这条 authority 就仍然会重新分叉。

这会直接破坏两件事：

- `visibility-owned / GPU-generated args source` 无法真正拥有 “draw exists / draw absent” 的最终决定权
- deeper cluster-raster execution 看起来已经跟随 unified indirect truth，实际却仍有一条 CPU resurrection side-path

这一刀收掉之后，renderer 不再拥有把 `Missing` entity 复活成 full-mesh draw 的权限，VG args/submission ownership 才真正闭环。

## Validation Summary

- focused red/green
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_missing_explicit_segments_do_not_resurrect_cpu_full_mesh_fallback_draws -- --nocapture`
- broader regressions
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_args_source_authority -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_submission_execution_order -- --nocapture`

