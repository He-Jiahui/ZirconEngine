---
related_code:
  - zircon_graphics/src/visibility/declarations/visibility_context.rs
  - zircon_graphics/src/visibility/declarations/visibility_virtual_geometry_draw_segment.rs
  - zircon_graphics/src/visibility/planning/build_virtual_geometry_plan/build.rs
  - zircon_graphics/src/runtime/virtual_geometry/prepare_frame/build_prepare_frame.rs
  - zircon_graphics/src/runtime/virtual_geometry/prepare_frame/prepare_visible_clusters.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/prepare_runtime_submission/virtual_geometry/build_virtual_geometry_prepare.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/prepare_runtime_submission/prepare.rs
  - zircon_graphics/src/tests/visibility.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
implementation_files:
  - zircon_graphics/src/visibility/declarations/visibility_context.rs
  - zircon_graphics/src/visibility/declarations/visibility_virtual_geometry_draw_segment.rs
  - zircon_graphics/src/visibility/planning/build_virtual_geometry_plan/build.rs
  - zircon_graphics/src/runtime/virtual_geometry/prepare_frame/build_prepare_frame.rs
  - zircon_graphics/src/runtime/virtual_geometry/prepare_frame/prepare_visible_clusters.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/prepare_runtime_submission/virtual_geometry/build_virtual_geometry_prepare.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/prepare_runtime_submission/prepare.rs
  - zircon_graphics/src/tests/visibility.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
plan_sources:
  - user: 2026-04-18 Virtual Geometry 的 deeper unified indirect / cluster raster / residency-manager cascade
  - user: 2026-04-18 继续列出所有剩余 todo，作为 tasks，然后继续深入
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-prepare-owned-unified-indirect-boundaries.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-page-owned-cluster-raster-consumption.md
tests:
  - zircon_graphics/src/tests/visibility.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - cargo test -p zircon_graphics --offline --locked visibility_context_splits_virtual_geometry_draw_segments_across_parent_lineages_even_when_page_matches
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_preserves_visibility_owned_draw_segments_across_parent_lineages
  - cargo test -p zircon_graphics --offline --locked visibility
  - cargo test -p zircon_graphics --offline --locked virtual_geometry
  - cargo test -p zircon_graphics --offline --locked render_server_bridge
  - cargo check -p zircon_asset --lib --offline --locked
  - cargo check -p zircon_graphics --lib --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Visibility-Owned Lineage Segment Boundaries

**Goal:** 把 `Virtual Geometry` 的 unified indirect segment authority 再从 runtime prepare 下沉一层到 visibility planning，使 `VisibilityContext` 本身就能表达稳定的 draw-segment 边界，而不是继续让 runtime 只靠 `visible_clusters` 做一次“看起来合理”的二次压缩。

**Non-Goal:** 本轮仍然不实现真正的 GPU-owned unified indirect buffer 资产、cluster streaming page uploader 重写、Nanite-like cluster rasterizer 或完整 residency-manager cascade。

## Delivered Slice

- 新增 `VisibilityVirtualGeometryDrawSegment`，作为 `VisibilityContext` 的显式输出：
  - `entity`
  - `cluster_id`
  - `page_id`
  - `cluster_ordinal`
  - `cluster_span_count`
  - `cluster_count`
  - `lod_level`
- `build_virtual_geometry_plan(...)` 现在不只生成 `virtual_geometry_visible_clusters`，还会同步生成 `virtual_geometry_draw_segments`。
- 这批 visibility-owned segment 的 compaction 规则在旧的同 page/ordinal 连续基础上，新增了 hierarchy-aware 边界：
  - 如果两个 refined cluster 虽然落在同一 `page_id`
  - 且 ordinal 连续、LOD 相同
  - 但它们来自不同 `parent_cluster_id` lineage
  - 那么 visibility planning 会保留两条独立 segment，而不是预先把它们并成一条
- `VirtualGeometryRuntimeState` 新增 `build_prepare_frame_with_segments(...)`，runtime prepare 现在可以直接消费 visibility-owned `draw_segments`，把 page residency / slot state 贴回这批 segment，而不是再依据 `visible_clusters` 自行 regroup。
- `submit_frame_extract -> prepare_runtime_submission -> build_virtual_geometry_prepare(...)` 生产路径已经切到这条新 contract，因此 `RenderServer` 主链会真实使用 visibility-owned lineage-aware segment authority。

## Why This Slice Exists

- 之前的 `prepare-owned unified indirect boundaries` 已经解决了 renderer 不能对 prepare segment 再做二次 regroup 的问题。
- 但 runtime prepare 仍然会从 `visible_clusters` 自行压出 `cluster_draw_segments`，authority 其实还停留在 runtime host，而没有真正下沉到 visibility 统一前处理层。
- 对更深的 `unified indirect / cluster raster / residency-manager cascade` 路线来说，这仍然不够，因为：
  - visibility 才真正知道当前 frontier 是如何跨 coarse/child/grandchild lineages 选出来的
  - runtime host 只看 `visible_clusters` 时，会丢掉“为什么这里必须 split”这类 hierarchy 语义
- 本轮用 “同一 resident page、但不同 parent lineage 仍需保持独立 segment” 作为第一条 visibility-owned contract，把 unified indirect authority 从 runtime 再往上推进了一步。

## Behavior Contract

- `VisibilityContext` 现在同时输出：
  - `virtual_geometry_visible_clusters`
  - `virtual_geometry_draw_segments`
- runtime prepare 仍然负责把 residency truth、pending state 和 slot ownership 贴回 segment，但不再拥有最终 segment regroup authority。
- 当 visibility draw segments 为空时，runtime 仍保留旧的 fallback compaction helper，避免无 payload 的辅助路径直接断掉。
- 一旦 visibility 明确给出 segment，runtime prepare 会原样保留这些边界，只做 state/slot/page truth 映射。

## Validation Summary

- `visibility_context_splits_virtual_geometry_draw_segments_across_parent_lineages_even_when_page_matches`
  - 证明 refined frontier 即使落在同一 resident page，也会因为 parent lineage 不同而保留两条 visibility-owned draw segment
- `virtual_geometry_runtime_state_preserves_visibility_owned_draw_segments_across_parent_lineages`
  - 证明 runtime prepare 不会再把这两条 lineage-aware segment 压回一条 prepare segment
- `cargo test -p zircon_graphics --offline --locked visibility`
  - 证明新的 visibility-owned segment 输出没有破坏 Hybrid GI、batching、history、Virtual Geometry frontier 现有回归
- `cargo test -p zircon_graphics --offline --locked virtual_geometry`
  - 证明 visibility -> runtime prepare -> indirect raster 主链仍然保持稳定
- `cargo test -p zircon_graphics --offline --locked render_server_bridge`
  - 证明 `RenderServer` 的 runtime submission 主链继续闭环
- `cargo check -p zircon_asset --lib --offline --locked`
  - 证明本轮顺带修复的 `zircon_asset::pipeline::manager` folder-backed wiring 漂移已经恢复 compile closure
- `cargo check -p zircon_graphics --lib --offline --locked`
  - 在 `E:` 盘空间耗尽后，改用 `D:\\codex-targets\\zircon-m5-vg` + `CARGO_INCREMENTAL=0` 复跑通过，说明当前 crate 代码闭环没有新的编译问题

## Remaining Route

- 把现在的 visibility-owned draw segment contract 继续推进到真正的 unified indirect buffer 资产，而不是仍由 runtime prepare 挂接后再交给 renderer build 层投影
- 让 deeper cluster raster / GPU indirect args / residency-manager cascade 直接消费这份 lineage-aware segment truth
- 继续推进更宽的 split-merge hysteresis、cluster streaming page residency，以及更接近 Nanite-like 的 execution 主链
