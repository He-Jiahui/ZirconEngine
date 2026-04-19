---
related_code:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_graphics/src/tests/virtual_geometry_args_source_authority.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_graphics/src/tests/virtual_geometry_args_source_authority.rs
plan_sources:
  - user: 2026-04-19 把这套 truth 继续压进更真实的 visibility-owned / GPU-generated args source
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-19-m5-virtual-geometry-prepare-owned-args-source-authority.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_args_source_authority.rs
  - cargo test -p zircon_graphics --offline virtual_geometry_fallback_full_mesh_args_follow_prepare_cluster_state_when_segments_are_absent -- --nocapture
  - cargo test -p zircon_graphics --offline virtual_geometry_fallback_full_mesh_uses_most_authoritative_visible_cluster_when_segments_are_absent -- --nocapture
  - cargo test -p zircon_graphics --offline virtual_geometry_ -- --nocapture
  - cargo test -p zircon_graphics --offline bloom_quality_profile_spreads_bright_pixels_when_enabled -- --nocapture
  - cargo check -p zircon_graphics --lib --offline
doc_type: milestone-detail
---

# M5 Virtual Geometry Fallback Full-Mesh Cluster Authority

## Goal

继续把 `Virtual Geometry` 的 unified-indirect authority 从 “prepare-owned args source” 压向更真实的 fallback cluster truth：

- 当 `cluster_draw_segments` 缺失时，full-mesh fallback 不能再回退到 renderer 自造的 `page_id = 0 / submission_slot = 0 / Resident` 默认 key。
- 同一个 entity 有多个 `visible_clusters` 时，fallback 也不能只吃 extract 里的第一个 cluster；它必须挑出当前最 authoritative 的 cluster truth，再把它推到真实 `segment_buffer / draw_ref_buffer / indirect args`。

## Delivered Slice

### 1. full-mesh fallback 不再自造 CPU 默认 indirect key

`build_mesh_draws(...)` 现在会先从 prepare snapshot 生成 `authoritative_fallback_segment_keys`：

- 先排除已经有显式 `cluster_draw_segments` 的 entity
- 再从 `visible_clusters + resident_pages + pending_page_requests` 推出 fallback entity 的 `page_id / frontier_rank / submission_slot / lod_level / state`
- 最后把这份 truth 写成 authoritative full-mesh segment key

这样一来，VG 开启但缺少显式 segment 的 entity，不会再把真实 GPU submission source 退回到 `page 0 / slot 0 / resident`。

### 2. pending draws 现在复用同一份 authoritative fallback key

`extend_pending_draws_for_mesh_instance(...)` 不再在 full-mesh fallback 分支里无条件调用默认 `full_mesh_indirect_draw_ref(...)`。

它现在会优先消费 `authoritative_fallback_segment_keys[entity]`，让：

- `pending_draws.indirect_draw_ref`
- authoritative `draw_ref_buffer`
- shared `segment_buffer`
- compute-generated `indirect args`

共用同一份 fallback segment truth，而不是一边是 prepare authority，一边又在 renderer 里临时造一份不同的 fallback key。

### 3. 同一 entity 的 fallback full-mesh 现在会选择最 authoritative visible cluster

如果 entity 没有显式 `cluster_draw_segments`，但 `visible_clusters` 不止一个，新的 fallback 逻辑会先在 entity 内部比较 cluster authority，再选 winner：

- `submission_slot`
- `frontier_rank`
- `cluster_index`
- `page_id / lod_level / state`

这意味着 fallback full-mesh 已经不再绑定 “第一个 visible cluster”，而是开始服从更深一层的 cluster visibility truth。

## Why This Slice Matters

上一刀已经把 `draw_ref_buffer / indirect args` existence truth 从 `pending_draws` 往 prepare/visibility 侧推了一层，但 full-mesh fallback 仍然留着两条明显泄漏：

- 没有显式 segment 时，renderer 继续自造默认 fallback key
- 同一 entity 有多个 visible clusters 时，renderer 继续凭输入顺序挑 cluster

这两条泄漏会让 unified-indirect authority 在最需要 fallback 的路径上重新滑回 CPU bookkeeping。

本轮补完之后，fallback full-mesh 至少已经和显式 segment 路径共享了同一类 authority contract：

- 先选 visibility/prepare truth
- 再生成 authoritative segment/draw-ref source
- 最后让 actual pending draws 只去绑定这份 truth

## Validation Summary

- red -> green
  - `virtual_geometry_fallback_full_mesh_args_follow_prepare_cluster_state_when_segments_are_absent`
  - `virtual_geometry_fallback_full_mesh_uses_most_authoritative_visible_cluster_when_segments_are_absent`
- regressions
  - `cargo test -p zircon_graphics --offline virtual_geometry_ -- --nocapture`
  - `cargo test -p zircon_graphics --offline bloom_quality_profile_spreads_bright_pixels_when_enabled -- --nocapture`
  - `cargo check -p zircon_graphics --lib --offline`

## Remaining Route

- 继续把这条 fallback authority 从 “CPU mesh draws 绑定 authoritative key” 压向更真实的 visibility-owned / GPU-generated args compaction source
- 继续推进 deeper cluster-raster consumption，让 fallback full-mesh 与显式 segment 路径都更多直接服从 cluster/page frontier truth，而不是主要通过 CPU-side draw bookkeeping 中转
- 再往后才切回更深的 split-merge frontier residency-manager cascade
