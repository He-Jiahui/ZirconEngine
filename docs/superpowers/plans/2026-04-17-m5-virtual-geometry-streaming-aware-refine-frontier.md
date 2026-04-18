---
related_code:
  - zircon_graphics/src/visibility/planning/build_virtual_geometry_plan/build.rs
- zircon_graphics/src/visibility/planning/build_virtual_geometry_plan/frontier/refine_visible_cluster_frontier.rs
  - zircon_graphics/src/tests/visibility.rs
implementation_files:
  - zircon_graphics/src/visibility/planning/build_virtual_geometry_plan/build.rs
- zircon_graphics/src/visibility/planning/build_virtual_geometry_plan/frontier/refine_visible_cluster_frontier.rs
  - zircon_graphics/src/tests/visibility.rs
plan_sources:
  - user: 2026-04-17 continue the remaining M5 route without waiting for confirmation
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-17-m5-virtual-geometry-cluster-refine.md
tests:
  - zircon_graphics/src/tests/visibility.rs
  - cargo test -p zircon_graphics visibility_context_keeps_parent_virtual_geometry_cluster_visible_while_requesting_nonresident_children --locked
  - cargo test -p zircon_graphics visibility_context_keeps_resident_virtual_geometry_children_visible_while_requesting_nonresident_grandchildren --locked
  - cargo test -p zircon_graphics visibility --locked
  - cargo test -p zircon_graphics virtual_geometry --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Streaming-Aware Refine Frontier

**Goal:** 把 `Virtual Geometry` 的 hierarchy refine 从“只看 `cluster_budget` 的可见 frontier”推进到真正能服务 cluster streaming 的双前沿规则：request 侧继续追更细 hierarchy，而 render 侧只有在替换 children page 已经 resident 时才真正下沉。

**Non-Goal:** 本轮仍然不实现 visibility-owned unified indirect buffer、GPU-driven residency manager、Nanite-like cluster raster 或更深的 split-merge hysteresis。

## Delivered Slice

- `build_virtual_geometry_plan(...)` 现在不再只生成一份 frontier。
  它会先从同一批 cluster 可见集里导出两条不同职责的结果：
  - `streaming_target_clusters`：继续沿用 budget-aware refine，专门服务 page request 计划
  - `visible_clusters`：同样遵守 `cluster_budget`，但只有在 replacement children 的 page 全部 resident 时才真正替换 parent
- `requested_pages` 现在来自 `streaming_target_clusters`，所以请求链仍然会向更细 cluster 继续推进，不会因为 render 还停在 coarse parent 就停止 streaming。
- `virtual_geometry_visible_clusters`、`visible_cluster_ids` 与 `evictable_pages` 现在来自 resident-gated render frontier，因此：
  - coarse parent page 在 children 还没 resident 时会继续留在 raster 路径里
  - parent 只有在 render frontier 真正切到 resident children 之后才会进入 `evictable_pages`
- hierarchy 递进现在支持更深层链路：
  - parent 可在 children resident 后先被替换
  - grandchildren 仍可继续作为 request 目标
  - 但只要 grandchildren page 还没 resident，render frontier 就会稳定停在 children，而不是过早跳到 pending grandchildren

## Behavior Contract

- request frontier 和 render frontier 共享同一份 cluster frustum filtering、`screen_space_error / lod_level / cluster_id` 排序和 `cluster_budget` 约束。
- resident gating 只影响“能否把 parent 替成 children”，不影响 request 侧继续向更细层级推进。
- `page_budget` 仍然只约束真正发起的 page request 数量；它不会强迫 render frontier提前丢掉当前 coarse resident page。
- `evictable_pages` 现在显式代表“已经不再属于当前 render frontier 的 resident page”，不再把“仍在 coarse 渲染但 children 正在请求中”的 parent 提前标成可回收。

## Why This Slice Exists

- 之前的 hierarchy refine 虽然已经 budget-aware，但完全不看 page residency。
  这会让系统在 children 还没 stream 到位时就把 coarse parent 从可见 frontier 里拿掉，导致 request 面和 raster 面都过早跳向 pending children。
- 这种行为不适合真正的 cluster streaming，因为：
  - coarse resident data 无法继续稳定承担 fallback raster
  - residency/eviction 规划会过早把 coarse parent 当成可回收页面
  - deeper hierarchy request 很难和当前帧实际可 raster frontier 解耦
- 把 request frontier 与 render frontier 分离后，当前架构终于有了真正的 “stream while still rendering coarse data” 基线，后续 visibility-owned indirect、cluster raster、residency manager 才有可信的继续落点。

## Validation Summary

- `visibility_context_keeps_parent_virtual_geometry_cluster_visible_while_requesting_nonresident_children`
  - 证明 children 还没 resident 时，render frontier 会保留 coarse parent，但 request 侧会继续请求 children page。
- `visibility_context_keeps_resident_virtual_geometry_children_visible_while_requesting_nonresident_grandchildren`
  - 证明 hierarchy 已经能形成更深层 streaming：render frontier 停在 resident children，而 request 继续追到 nonresident grandchildren。
- `visibility_context_refines_virtual_geometry_parent_cluster_into_visible_children_when_budget_allows`
  - 证明一旦 replacement children page 全部 resident，render frontier 仍会按原计划真正下沉到 finer clusters。
- `cargo test -p zircon_graphics visibility --locked`
  - 证明新的双前沿规则没有破坏现有 visibility、history、Hybrid GI 与 instancing/culling 规划。
- `cargo test -p zircon_graphics virtual_geometry --locked`
  - 证明新的 streaming-aware hierarchy frontier 与 runtime host、GPU uploader/readback、indirect raster 基线兼容。

## Remaining Route

- 把当前 request/render 双前沿继续推进到真正的 visibility-owned indirect / cluster raster consumption，而不是继续只停在 prepare fallback consumption
- 把 coarse/fine frontier 与 residency manager、feedback hysteresis、GPU-driven indirect compaction 连接起来
- 引入更完整的 split-merge hysteresis / SSE threshold policy，而不是只靠当前帧 resident gating
