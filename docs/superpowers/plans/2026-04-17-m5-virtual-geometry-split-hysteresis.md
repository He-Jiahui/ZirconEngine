---
related_code:
  - zircon_graphics/src/visibility/declarations/visibility_history_snapshot.rs
  - zircon_graphics/src/visibility/context/from_extract_with_history/build_history_snapshot.rs
  - zircon_graphics/src/visibility/context/from_extract_with_history/construct.rs
  - zircon_graphics/src/visibility/planning/build_virtual_geometry_plan/build.rs
  - zircon_graphics/src/visibility/planning/build_virtual_geometry_plan/refine_visible_cluster_frontier.rs
  - zircon_graphics/src/tests/visibility.rs
implementation_files:
  - zircon_graphics/src/visibility/declarations/visibility_history_snapshot.rs
  - zircon_graphics/src/visibility/context/from_extract_with_history/build_history_snapshot.rs
  - zircon_graphics/src/visibility/context/from_extract_with_history/construct.rs
  - zircon_graphics/src/visibility/planning/build_virtual_geometry_plan/build.rs
  - zircon_graphics/src/visibility/planning/build_virtual_geometry_plan/refine_visible_cluster_frontier.rs
  - zircon_graphics/src/tests/visibility.rs
plan_sources:
  - user: 2026-04-17 continue M5
  - user: 2026-04-17 Virtual Geometry still needs deeper split-merge hysteresis
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-streaming-aware-refine-frontier.md
tests:
  - zircon_graphics/src/tests/visibility.rs
  - cargo test -p zircon_graphics --offline --locked visibility_context_holds_resident_parent_one_frame_after_requested_children_become_resident
  - cargo test -p zircon_graphics --offline --locked visibility
  - cargo test -p zircon_graphics --offline --locked virtual_geometry
  - cargo test -p zircon_graphics --offline --locked render_server_bridge
  - cargo check -p zircon_graphics --lib --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Split Hysteresis

**Goal:** 把 `Virtual Geometry` 的 resident-gated refine frontier 再推进一层，在 children page 刚从 request 变成 resident 的上传完成帧上保留 coarse parent 一帧，并同步保护这些 child page 不被立即标记成 `evictable`。

**Non-Goal:** 本轮仍然不实现真正的 Nanite-like cluster raster、GPU-driven occlusion、visibility-owned unified indirect buffer 或更深层 cluster split/merge policy。

## Delivered Slice

- `VisibilityHistorySnapshot` 现在会记录上一帧 `virtual_geometry_visible_cluster_ids`，而不再只保存 `virtual_geometry_requested_pages`。
- `refine_visible_cluster_frontier(...)` 在 render frontier 路径上新增了一条 upload-completion split hysteresis 规则：
  - 当前 coarse parent page 仍 resident
  - replacement children page 本帧已经全部 resident
  - 且这些 child page 正好来自上一帧 `requested_pages`
  - 那么当前帧继续保留 coarse parent，不在 upload 完成帧立刻跳到 children
- 下一帧如果这些 children 仍然 resident，但上一帧已经不再把它们记作 request，新 frontier 会自然 split 到 children，因此 hysteresis 只持续一个过渡帧。
- `build_virtual_geometry_plan(...)` 同时会保护这批“刚从 request 变 resident 但因 hysteresis 暂未消费”的 child page：
  - 当前帧不会把它们写进 `evictable_pages`
  - 避免 render frontier 还在 hold coarse parent 时，residency manager 却立刻把新上传页当作可回收页

## Why This Slice Exists

- 之前的 streaming-aware refine 已经把 request frontier 和 render frontier 拆开，但一旦 children page 全部 resident，render frontier 仍会在同一帧立刻从 parent 切到 children。
- 这会带来两个不稳定点：
  - coarse -> fine frontier 会在上传完成帧瞬时跳变
  - 新完成上传的 child page 还没真正参与一帧 raster，就会因为 parent 仍然被保留或 budget/visibility 变化而被误判成 `evictable`
- 加上一帧 split hysteresis 后，当前架构终于具备了最小可用的“upload completes, keep coarse one frame, then split”稳定层。

## Behavior Contract

- hysteresis 只作用于 render frontier；streaming target frontier 仍会继续请求更细 hierarchy。
- 只有“上一帧确实请求过、这一帧刚变 resident、并且 coarse parent 当前仍 resident”的 replacement children 才会被 hold。
- hysteresis 结束后，resident children 会像之前一样进入正常 render frontier，parent page 也会重新进入 `evictable_pages`。

## Validation Summary

- `visibility_context_holds_resident_parent_one_frame_after_requested_children_become_resident`
  - 证明 children page 刚变 resident 时，当前帧仍会保留 coarse parent，下一帧才真正 split 到 children。
- `cargo test -p zircon_graphics --offline --locked visibility`
  - 证明新的 history-aware split hysteresis 没有破坏 Virtual Geometry 既有的 visibility、Hybrid GI、batching 与 instance 上传回归。
- `cargo test -p zircon_graphics --offline --locked virtual_geometry`
  - 证明 render frontier hold + `evictable_pages` 保护与 prepare snapshot、runtime host、GPU uploader/readback、indirect raster 主链兼容。
- `cargo test -p zircon_graphics --offline --locked render_server_bridge`
  - 证明 render-server runtime 主链继续保持稳定。

## Remaining Route

- 更深层的 split-merge hysteresis，而不是只做 upload-completion 这一跳
- `Virtual Geometry` 的 deeper cluster raster consumption / unified indirect ownership / GPU-driven indirect compaction
- 更完整的 residency manager policy，与 Nanite-like cluster streaming/page residency 主链对齐
