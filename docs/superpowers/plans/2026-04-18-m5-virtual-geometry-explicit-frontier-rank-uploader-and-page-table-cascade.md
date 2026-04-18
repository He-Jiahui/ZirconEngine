---
related_code:
  - zircon_graphics/src/types/virtual_geometry_prepare/request.rs
  - zircon_graphics/src/runtime/virtual_geometry/prepare_frame/pending_page_requests.rs
  - zircon_graphics/src/types/virtual_geometry_prepare/frame.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/gpu_pending_request_input/gpu_pending_request_input.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/execute_prepare/pending_requests.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/virtual_geometry_uploader_params/virtual_geometry_uploader_params.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/shaders/uploader.wgsl
  - zircon_graphics/src/tests/virtual_geometry_gpu.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
implementation_files:
  - zircon_graphics/src/types/virtual_geometry_prepare/request.rs
  - zircon_graphics/src/runtime/virtual_geometry/prepare_frame/pending_page_requests.rs
  - zircon_graphics/src/types/virtual_geometry_prepare/frame.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/gpu_pending_request_input/gpu_pending_request_input.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/gpu_resources/execute_prepare/pending_requests.rs
  - zircon_graphics/src/scene/scene_renderer/virtual_geometry/shaders/uploader.wgsl
  - zircon_graphics/src/tests/virtual_geometry_gpu.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
plan_sources:
  - user: 2026-04-18 下一步是更深的 unified-indirect / residency-manager cascade，把同一套 frontier truth 继续推进到真实 GPU uploader / page-table / split-merge frontier policy
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/sessions/20260417-1415-m5-vg-slot-assignment-ownership.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-frontier-rank-gpu-submission-and-cluster-raster-consumption.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_gpu.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_gpu -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_render -- --nocapture
  - cargo check -p zircon_graphics --lib --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Explicit Frontier Rank Uploader And Page-Table Cascade

**Goal:** 把 `Virtual Geometry` 的 frontier truth 从 “runtime 先排好 `pending_page_requests`” 继续推进到真实 GPU uploader / page-table completion path，让 residency-manager 的下一层扩展不再依赖输入 Vec 位置这种隐式约定。

**Non-Goal:** 本轮不实现更深的 split-merge hysteresis，不做真正的 GPU-generated indirect compaction，也不把 uploader 升级成完整 page residency manager。

## Delivered Slice

### 1. Frontier truth 变成显式 prepare request contract

- `VirtualGeometryPrepareRequest` 新增 `frontier_rank`
- `pending_page_requests(...)` 不再只输出按 frontier 排序后的 Vec
- 它现在会把当前 `requested_pages` 真值同时固化为显式 `frontier_rank`

这意味着：

- runtime queue order 只是一个投影
- 真正的 frontier priority 已经成为 frame-local prepare contract 的一部分

后续 uploader、page-table cascade、split-merge policy 都可以复用这同一字段，而不需要继续猜测“当前输入第几个元素更重要”。

### 2. GPU uploader 现在按 explicit frontier rank 选请求，不再只吃输入顺序

- `GpuPendingRequestInput` 新增 `frontier_rank`
- `pending_requests(...)` 会把 prepare request 里的显式 rank 继续传给 GPU
- `uploader.wgsl` 不再线性消费 `pending_requests[0..n]`

新的 uploader 行为是：

- 在当前 `remaining_bytes`、slot budget、`page_budget` 约束下扫描全部 pending requests
- 跳过已经完成过的 request
- 选择 **最小 `frontier_rank`** 的可执行 request
- 同 rank 时才回退到输入顺序

所以当前 GPU uploader 已经真正消费 explicit frontier truth，而不是继续依赖 CPU 先把 Vec 排好。

### 3. Page-table completion 现在和 unified-indirect / cluster-raster 共用同一 frontier source

- `VirtualGeometryPrepareFrame::unified_indirect_draws()` 不再从 `pending_page_requests` 的枚举顺序重新推导 frontier
- 它现在直接读 `VirtualGeometryPrepareRequest.frontier_rank`

这让两条路径收敛到同一份 request contract：

- GPU uploader / page-table completion
- unified indirect / cluster-raster submission

结果是 runtime residency path 与 renderer submission path 已经共享同一套 frontier truth，而不是各自从不同的“Vec 顺序暗约定”里再推一遍。

### 4. 回归现在明确覆盖 “truth 不等于输入顺序”

- `virtual_geometry_gpu_uploader_readback_prioritizes_explicit_frontier_rank_over_input_order`
  - 证明 slot 只剩一个时，GPU uploader 会选 `frontier_rank` 更早的页，而不是 pending input buffer 里排第一的页
- `virtual_geometry_unified_indirect_keeps_pending_request_frontier_rank_in_gpu_submission_and_indirect_args`
  - 证明 unified-indirect / GPU submission 现在也按显式 `frontier_rank` 消费，而不是按 request Vec 位置偷取 rank
- `virtual_geometry_prepare_cluster_raster_output_changes_when_pending_request_frontier_rank_changes`
  - 证明最终离屏 raster output 也继续跟随同一条 explicit frontier truth

## Why This Slice Exists

上一轮已经把 frontier/order 压进了：

- GPU submission segment buffer
- indirect args
- cluster-raster output

但 uploader / page-table 一侧仍然有一个显著裂缝：

- runtime host 只是在 CPU 上把 request Vec 先排好
- GPU uploader 自己并不知道“为什么这个请求排在前面”

这会让更深的 residency-manager cascade 很难扩：

- 一旦 request buffer 在后续阶段被 regroup / merge / dedup
- frontier priority 就会退化成“位置语义”，无法稳定传到 page-table / split-merge policy

本轮补上的，就是这条 runtime-to-GPU-residency 的显式 contract。

## Validation Summary

- `virtual_geometry_gpu`
  - 证明 GPU uploader 现在会优先完成更早 `frontier_rank` 的请求
  - 既有 `streaming bytes / available slot / evictable slot / page-table snapshot` 回归没有退化
- `virtual_geometry_runtime`
  - 证明 runtime prepare 现在会输出显式 `frontier_rank`
  - frontier rank 可以与最终 request order 分离存在
- `virtual_geometry_unified_indirect`
  - 证明 unified-indirect / GPU submission 消费的是 explicit frontier truth，而不是 request Vec 位置
- `virtual_geometry_prepare_render`
  - 证明 cluster-raster 最终帧输出继续跟随同一 frontier source
- `cargo check -p zircon_graphics --lib --offline --locked`
  - 证明 uploader/request contract 变化没有破坏 crate 编译闭环

## Remaining Route

- 继续把 explicit `frontier_rank` 与 `split-merge hysteresis`、`lineage_distance`、`page_table` truth 收束成统一的 residency-manager cascade，而不只停在 uploader request selection。
- 继续把 `evictable_pages` / slot recycling 也变成显式 GPU-side policy input，而不是只在 CPU runtime 排序后把 slot id 列表下发。
- 继续向更完整的 visibility-owned unified indirect / cluster streaming / Nanite-like raster consumption 推进。
