---
related_code:
  - zircon_graphics/src/runtime/virtual_geometry/declarations/virtual_geometry_runtime_state.rs
  - zircon_graphics/src/runtime/virtual_geometry/extract_registration.rs
  - zircon_graphics/src/runtime/virtual_geometry/plan_ingestion.rs
  - zircon_graphics/src/runtime/virtual_geometry/prepare_frame/pending_page_requests.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/ordered_evictable_pages_for_target.rs
  - zircon_graphics/src/tests/mod.rs
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
  - zircon_graphics/Cargo.toml
implementation_files:
  - zircon_graphics/src/runtime/virtual_geometry/declarations/virtual_geometry_runtime_state.rs
  - zircon_graphics/src/runtime/virtual_geometry/extract_registration.rs
  - zircon_graphics/src/runtime/virtual_geometry/plan_ingestion.rs
  - zircon_graphics/src/runtime/virtual_geometry/prepare_frame/pending_page_requests.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/ordered_evictable_pages_for_target.rs
  - zircon_graphics/src/tests/mod.rs
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
  - zircon_graphics/Cargo.toml
plan_sources:
  - user: 2026-04-18 切到 Virtual Geometry，当前活动任务是 unified indirect / residency-manager cascade / split-merge frontier policy
  - user: 2026-04-18 继续缺漏内容补充 / 继续深入 / 继续完成后续里程碑
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/sessions/20260417-1415-m5-vg-slot-assignment-ownership.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-pending-cascade-descendant-hold.md
  - docs/superpowers/plans/2026-04-18-m5-virtual-geometry-lineage-depth-and-eviction-distance.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_frontier_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_frontier_runtime -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect -- --nocapture
  - cargo check -p zircon_graphics --lib --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Frontier Priority And Active Request Lineage

## Goal

继续推进 `Virtual Geometry` 的 `unified indirect / residency-manager cascade / split-merge frontier policy`，把 visibility 已经确定下来的当前 request frontier 顺序和 active-lineage 压力真正下沉到 runtime uploader / eviction 排序，而不是继续只靠 runtime 本地的 descendant 数量启发式或输入列表顺序。

## Non-Goal

- 本轮不实现真正的 GPU-owned unified indirect buffer。
- 本轮不重写 `build_virtual_geometry_plan(...)` 的 visibility request 评分。
- 本轮不引入新的 page-table / uploader GPU pass。

## Delivered Slice

### 1. Runtime 现在缓存当前 visibility request frontier 顺序

- `VirtualGeometryRuntimeState` 新增 `current_requested_page_order`
- `ingest_plan(...)` 每帧都会按 `VisibilityVirtualGeometryPageUploadPlan.requested_pages` 刷新这份顺序真值
- `register_extract(...)` 也会在 scene 收缩时裁掉失效 page，避免旧 frontier 顺序穿过场景边界继续污染下一帧

### 2. Pending uploader queue 先服从当前 frontier，再回退旧启发式

- `prepare_frame/pending_page_requests.rs` 不再只按：
  - resident descendant count
  - descendant count
  - page depth
- 现在会先按当前 `requested_pages` 的顺序给 pending upload 排序
- 只有当 request frontier rank 打平时，才回退到原来的 hot-descendant / hierarchy depth 启发式

这意味着 runtime prepare 不会再把一个“上一帧排得更靠前、但当前已经退居次要”的 pending request 仅仅因为 descendant 更多，就重新抢回当前 upload 队首。

### 3. Eviction 排序现在识别其他 active request lineages

- `ordered_evictable_pages_for_target(...)` 仍然保留现有的 target-lineage 策略：
  - unrelated
  - ancestor
  - descendant
  - 且 ancestor / descendant 内部继续优先回收更远 lineage distance
- 在此之上，又新增一层 active-request-lineage 保护：
  - 先回收与所有 active request 都无关的页
  - 如果候选页属于别的 active request lineage，则优先回收较晚 request 的那条 lineage
  - 同一条 active request lineage 内，再优先回收离该 frontier 更远的页

这样 runtime 在为新 frontier page 腾 slot 时，不会再因为 `evictable_pages` 输入顺序碰巧排在前面，就先踢掉还在支撑其他 active request 恢复路径的 resident page。

## Why This Slice Exists

在这之前，M5 的 `Virtual Geometry` 已经完成了：

- visibility-owned lineage segment boundaries
- prepare-owned unified indirect authority
- GPU submission segment / draw-ref readback
- lineage-distance eviction ordering
- pending-cascade descendant hold

但 runtime 仍然有两个真实缺口：

1. 当前 visibility planner 已经决定好的 `requested_pages` frontier 顺序，没有继续进入 runtime pending uploader queue。
2. runtime eviction 只知道 “和当前 target 是不是同 lineage”，却不知道 “这个页是否还在支撑别的 active request lineage”。

这会导致两种明显的抖动：

- 当前 frame 的更重要 frontier request 会被旧 pending queue 的 descendant-count 启发式重新压回后面。
- 一个仍在支撑其他 pending frontier 的 resident page，会因为只是排在 `evictable_pages` 输入靠前位置而被过早回收。

本轮补上的就是这两条 runtime-host 级缺口。

## Validation Summary

- `virtual_geometry_runtime_state_uses_current_visibility_request_order_for_pending_uploads`
  - 证明 runtime prepare 会优先保留当前 visibility request frontier 顺序，而不是让更旧、descendant 更多的 pending request 抢回 upload 队首。
- `virtual_geometry_runtime_state_evicts_unrelated_pages_before_active_request_lineages`
  - 证明 runtime 在为新 frontier page 腾 resident budget 时，会先回收与所有 active request 都无关的页。
- `virtual_geometry_runtime_state_evicts_later_active_request_lineage_before_earlier_one`
  - 证明当只剩 active request lineages 可回收时，runtime 会优先回收较晚 request 的那条 lineage，而继续保热更早的 active frontier。
- `virtual_geometry_runtime`
  - 证明既有的 pending ancestor hold、lineage-distance eviction、slot recycling guard、prepare-owned segment contract 没有回归。
- `virtual_geometry_unified_indirect`
  - 证明本轮 runtime ordering 下沉没有破坏当前 unified indirect / GPU submission baseline。

## Remaining Route

- 把当前 runtime-host 的 frontier/order truth 继续向真实 GPU uploader/readback authority 下沉，而不只停在 CPU-side pending queue 与 eviction 排序。
- 继续把 deeper cluster raster consumption 与 unified indirect authority 合并到更真实的 visibility-owned indirect submission。
- 继续推进更宽的 residency-manager cascade / split-merge frontier policy，包括更深层 hierarchy refine、streaming frontier 与 residency frontier 的联合策略。
