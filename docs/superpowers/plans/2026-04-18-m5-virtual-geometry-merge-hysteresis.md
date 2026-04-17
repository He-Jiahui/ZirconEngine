---
related_code:
  - zircon_graphics/src/visibility/planning/build_virtual_geometry_plan/build.rs
  - zircon_graphics/src/tests/visibility.rs
implementation_files:
  - zircon_graphics/src/visibility/planning/build_virtual_geometry_plan/build.rs
  - zircon_graphics/src/tests/visibility.rs
plan_sources:
  - user: 2026-04-17 continue M5
  - user: 2026-04-17 Virtual Geometry still needs deeper split-merge hysteresis
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-split-hysteresis.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-page-owned-cluster-raster-consumption.md
tests:
  - zircon_graphics/src/tests/visibility.rs
  - cargo test -p zircon_graphics --offline --locked visibility_context_holds_resident_parent_one_frame_after_requested_children_become_resident
  - cargo test -p zircon_graphics --offline --locked visibility
  - cargo test -p zircon_graphics --offline --locked virtual_geometry
  - cargo check -p zircon_graphics --lib --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Merge Hysteresis

**Goal:** 把 `Virtual Geometry` 的 split/merge 稳定层再往前推进一步，让 coarse parent 在真正 split 到 resident children 之后仍然额外保留一帧 non-evictable，而不是在 split 落地帧立刻进入回收集。

**Non-Goal:** 本轮仍然不实现真正的 budget-oversubscription merge hold、GPU-driven indirect compaction、Nanite-like cluster rasterizer 或完整 residency-manager 策略树。

## Delivered Slice

- `build_virtual_geometry_plan(...)` 现在除了已有的 `split_hold_protected_pages` 外，又新增了一层 merge-side coarse-parent 保护：
  - parent page 当前 resident
  - 上一帧 `previous_visible_cluster_ids` 里确实还在渲染这个 parent
  - 当前帧则已经真正 split 到它的 resident children
  - 那么当前帧不会立刻把 parent page 写进 `evictable_pages`
- 这层保护只持续一个过渡帧：
  - split 落地帧保护 parent
  - 下一帧如果 children 仍然稳定可见，parent 才重新回到 evictable 集

## Why This Slice Exists

- 之前的 `split hysteresis` 只解决了 “children 刚上传完成时不要立刻替掉 parent”。
- 但真正 split 到 children 之后，coarse parent 仍会在同一帧马上进入 `evictable_pages`，这会让 merge-back 路径没有缓冲。
- 如果 finer pages 紧接着发生 churn、budget 变化或 residency 抖动，runtime host 可能已经把 coarse parent 提前回收。
- 本轮加上一帧 merge-side parent hold 后，当前架构终于具备了最小可用的 split-then-keep-coarse-one-more-frame 稳定层。

## Validation Summary

- `visibility_context_holds_resident_parent_one_frame_after_requested_children_become_resident`
  - 证明上传完成帧会先 hold coarse parent，真正 split 到 children 后 parent 仍再保留一帧 non-evictable，下一帧才重新变成 evictable
- `cargo test -p zircon_graphics --offline --locked visibility`
  - 证明新增 coarse-parent merge hysteresis 没有破坏 Virtual Geometry / Hybrid GI 的统一 visibility 规划回归
- `cargo test -p zircon_graphics --offline --locked virtual_geometry`
  - 证明这层 residency 保护与 prepare snapshot、GPU uploader/readback、page-table truth、indirect raster 主链兼容
- `cargo check -p zircon_graphics --lib --offline --locked`
  - 证明 visibility planning contract 仍然闭合

## Remaining Route

- 把当前 coarse-parent eviction hysteresis 继续推进到更完整的 split-merge policy，而不是只保护 parent eviction 一帧
- 把真正的 visibility-owned unified indirect ownership 继续下沉到更靠前的 prepare/visibility contract
- 继续朝 GPU-driven indirect compaction、cluster streaming 与 Nanite-like cluster raster execution 迈进
