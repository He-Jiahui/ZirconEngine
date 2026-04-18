---
related_code:
  - zircon_graphics/src/runtime/hybrid_gi/extract_registration.rs
  - zircon_graphics/src/runtime/virtual_geometry/extract_registration.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
implementation_files:
  - zircon_graphics/src/runtime/hybrid_gi/extract_registration.rs
  - zircon_graphics/src/runtime/virtual_geometry/extract_registration.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
plan_sources:
  - user: 2026-04-18 Hybrid GI 的 resolve/runtime-host 侧更完整 scene-driven hierarchy 闭环
  - user: 2026-04-18 Virtual Geometry 的更深 residency-manager cascade / split-merge policy
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-16-m5-virtual-geometry-runtime-host.md
  - docs/superpowers/plans/2026-04-16-m5-hybrid-gi-runtime-host.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - cargo test -p zircon_graphics --offline --locked hybrid_gi_runtime
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime
doc_type: milestone-detail
---

# M5 Runtime Scene Authority Prune

**Goal:** 把 `Hybrid GI` 与 `Virtual Geometry` 的 runtime host 再向 scene-driven authority 收紧一层，让当前 extract 已移除的 probe/page 不会继续以 stale resident、stale pending 或 stale cached truth 的形式留到下一帧。

**Non-Goal:** 本轮不改写 hierarchy resolve shader、GPU completion 算法或 uploader/readback 数据格式。

## Delivered Slice

- `HybridGiRuntimeState::register_extract(...)` 现在会：
  - 在 `extract = None` 时整状态 reset
  - 清理当前 extract 已经不再包含的 probe id
  - 同步修剪 `resident_slots / pending_probes / pending_updates / probe_ray_budgets / probe_irradiance_rgb`
  - 保留仍属于当前场景、但暂时 `nonresident` 的 live probe，避免错误回收真实还在 hierarchy 里的缓存
- `VirtualGeometryRuntimeState::register_extract(...)` 现在会：
  - 在 `extract = None` 时整状态 reset
  - 清理当前 extract 已经不再包含的 page id
  - 同步修剪 `resident_slots / pending_pages / pending_requests / page_sizes`
  - 释放 stale resident page 占用的 slot，避免旧 page-table truth 穿透到下一帧

## Why This Slice Exists

- 之前两条 runtime host 主链都更偏向“只增长，不收口”：
  - 新 scene id 会进入 resident/pending/cache
  - 但 scene hierarchy 收缩、feature 关闭或 extract 删除分支后，旧 id 仍可能残留
- 这会让后续更深的 M5 行为建立在错误 scene truth 上：
  - `Hybrid GI` 会继续带着已经脱离场景的 probe irradiance / pending update
  - `Virtual Geometry` 会继续带着已经脱离场景的 page slot / pending upload

## Validation Summary

- `hybrid_gi_runtime_state_drops_stale_scene_probes_and_pending_updates_when_extract_shrinks`
  - 证明 removed probe 不会继续保留 resident slot、pending update 或 irradiance cache。
- `virtual_geometry_runtime_state_drops_stale_scene_pages_and_pending_requests_when_extract_shrinks`
  - 证明 removed page 不会继续保留 resident slot、pending request 或 page-size bookkeeping。
- `cargo test -p zircon_graphics --offline --locked hybrid_gi_runtime`
- `cargo test -p zircon_graphics --offline --locked virtual_geometry_runtime`

## Remaining Route

- 把这层 scene authority 继续前推到多帧 `render_server` 集成回归。
- 对 `Hybrid GI`，继续把 scene-driven hierarchy continuity 推进到 runtime-host / resolve 共用的数据源。
- 对 `Virtual Geometry`，继续把 scene authority 与 deeper residency cascade / cluster raster ownership 收拢到同一条执行链。
