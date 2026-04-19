---
related_code:
  - zircon_graphics/src/runtime/virtual_geometry/plan_ingestion.rs
  - zircon_graphics/src/runtime/virtual_geometry/prepare_frame/pending_page_requests.rs
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
  - docs/assets-and-rendering/render-framework-architecture.md
implementation_files:
  - zircon_graphics/src/runtime/virtual_geometry/plan_ingestion.rs
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
plan_sources:
  - user: 2026-04-19 继续完成全部的虚拟几何体任务，不要中途确认
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-frontier-priority-and-active-request-lineage.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_keeps_first_unique_visibility_request_order_when_duplicate_requested_pages_reappear -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_frontier_runtime -- --nocapture
  - cargo check -p zircon_graphics --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry First-Unique Request-Order Closure

## Goal

收掉 `Virtual Geometry` runtime host 在 `current_requested_page_order` 上的一条 duplicate-request residue：当 visibility feedback / plan 给出重复 `requested_pages` 时，runtime 不能继续让 later duplicate 覆盖 first-seen frontier rank。

## Delivered Slice

### 1. `current_requested_page_order` 现在保留 first-seen unique request rank

`plan_ingestion.rs` 原来直接把：

- `requested_pages.iter().enumerate()`

收成 `BTreeMap<page_id, order>`。

这意味着一旦输入出现 duplicate page id，后一次出现的位置会直接覆盖前一次，runtime 看到的 active request frontier 就会被 later duplicate 改序。

现在 runtime host 改成：

- 先清空旧的 `current_requested_page_order`
- 再按输入顺序逐个插入
- 只记录每个 page 的第一次出现位置

因此 duplicate request 现在只会被当成噪声，而不会重写 frontier truth。

### 2. pending upload 排序继续跟随 first-unique visibility frontier

新增回归：

- `virtual_geometry_runtime_state_keeps_first_unique_visibility_request_order_when_duplicate_requested_pages_reappear`

它证明当第二帧 `requested_pages = [200, 800, 200]` 时，runtime prepare 仍然应该把：

- `200` 当作 `frontier_rank = 0`
- `800` 当作 `frontier_rank = 1`

而不是因为尾部 duplicate `200` 把 `200` 反向降成更晚的 request。

## Why This Slice Matters

`Virtual Geometry` 当前已经把 active request frontier 真值一路压进：

- pending upload 排序
- recycle slot 选择
- eviction lineage priority
- prepare-owned indirect submission authority

如果 `current_requested_page_order` 还允许 duplicate request 覆盖 first-seen frontier rank，那么更深层 runtime / residency / indirect ownership 就仍然会被重复 request 噪声改序。

## Validation Summary

- 红:
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_keeps_first_unique_visibility_request_order_when_duplicate_requested_pages_reappear -- --nocapture`
- 绿:
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime_state_keeps_first_unique_visibility_request_order_when_duplicate_requested_pages_reappear -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_frontier_runtime -- --nocapture`
  - `cargo check -p zircon_graphics --offline --locked`

## Remaining M5 Todo After This Slice

- `Virtual Geometry`: 继续把 unified indirect / cluster-raster ownership 往更真实的 GPU-generated args source 下沉。
- `Virtual Geometry`: 继续补更深的 split-merge frontier / residency-manager cascade。
- `Hybrid GI`: 继续推进更完整的 scene-driven screen-probe hierarchy / RT hybrid lighting continuation。
