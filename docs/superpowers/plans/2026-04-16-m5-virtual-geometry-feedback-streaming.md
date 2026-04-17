---
related_code:
  - zircon_graphics/src/runtime/virtual_geometry/mod.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/mod.rs
  - zircon_graphics/src/runtime/virtual_geometry/residency_management/mod.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/mod.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/record_submission/mod.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/submit/mod.rs
  - zircon_graphics/src/visibility/declarations/visibility_virtual_geometry_feedback.rs
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - docs/assets-and-rendering/srp-rhi-render-server-architecture.md
  - docs/assets-and-rendering/index.md
implementation_files:
  - zircon_graphics/src/runtime/virtual_geometry/mod.rs
  - zircon_graphics/src/runtime/virtual_geometry/pending_completion/mod.rs
  - zircon_graphics/src/runtime/virtual_geometry/residency_management/mod.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/mod.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/record_submission/mod.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/submit/mod.rs
plan_sources:
  - user: 2026-04-16 continue next M5 Virtual Geometry slice after prepare consumption
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-16-m5-virtual-geometry-runtime-host.md
  - docs/superpowers/plans/2026-04-16-m5-virtual-geometry-prepare-consumption.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_runtime.rs
  - cargo test -p zircon_graphics virtual_geometry_runtime_state_consumes_feedback_and_promotes_requested_pages --locked
  - cargo test -p zircon_graphics virtual_geometry_runtime_state_leaves_requests_pending_without_evictable_budget --locked
  - cargo test -p zircon_graphics virtual_geometry_runtime_state_builds_prepare_frame_with_resident_pending_and_missing_clusters --locked
  - cargo test -p zircon_graphics virtual_geometry_prepare_filters_mesh_fallback_to_allowed_entities --locked
  - cargo test -p zircon_graphics --lib --locked
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_graphics
doc_type: milestone-detail
---

# M5 Virtual Geometry Feedback Streaming Plan

**Goal:** 把此前未消费的 `VisibilityVirtualGeometryFeedback` 接进 `VirtualGeometryRuntimeState`，让 pending page request 在帧后按 resident budget 和 evictable page 列表推进到下一帧 residency，而不是永远停在 request sink 里。

**Non-Goal:** 本轮仍然不实现真实 GPU page upload、DMA/readback、磁盘 streaming、cluster hierarchy refine、Nanite raster 或 indirect draw integration。

## Delivered Slice

- `VirtualGeometryRuntimeState` 现在记录一个 runtime resident budget，当前由 `RenderVirtualGeometryExtract.page_budget` 提供，并至少覆盖 extract 基线 resident page 数量。
- runtime host 新增 `consume_feedback(&VisibilityVirtualGeometryFeedback)`：
  - 只消费当前帧 feedback 中仍处于 pending 的 requested page
  - resident 数达到 budget 时，只能回收 feedback 提供的 `evictable_pages`
  - 没有可回收 budget 时，request 会继续保持 pending，而不是无上限扩 resident cache
- `WgpuRenderServer::submit_frame_extract(...)` 现在会在 render 完成后调用 `consume_feedback(...)`，再把更新后的 runtime host 写回 viewport record，并用更新后的 snapshot 作为 façade stats 的来源。
- `runtime/virtual_geometry/{pending_completion,residency_management}/` 与 `runtime/server/submit_frame_extract/{submit,record_submission}/` 当前都已经下沉成 root-only wiring + helper 子模块，feedback completion、resident-slot bookkeeping 与提交后 runtime host 回写不再继续堆放在单脚本里。

## Runtime Rules

- `RenderVirtualGeometryExtract.page_budget`
  - 当前 CPU fallback baseline 同时充当 requested-page 预算与 runtime resident-page 预算
  - 如果 extract 自己已经声明了更多 `resident = true` 的 page，runtime host 会自动把 budget 抬到至少能容纳这些基线 resident page
- `consume_feedback(...)`
  - 只会处理 `feedback.requested_pages` 中当前仍处于 `pending` 的 page
  - 按 feedback 顺序尝试 promote；如果当前 resident 数已经达到 budget，则按 feedback 的 `evictable_pages` 顺序回收 resident page
  - 无法回收足够 slot 时，本帧剩余 request 会继续留在 pending queue
- 下一帧 `build_prepare_frame(...)`
  - 会直接观察到上一帧 feedback 消费后的 resident/pending 状态
  - 因此 `PendingUpload -> Resident` 的推进已经不需要手工测试 helper 才能出现

## Why This Slice Exists

- prepare-consumption slice 已经让 `virtual-geometry-prepare` 真正消费 runtime host，但 runtime host 里的 pending request 仍然没有真实 consumer。
- 如果不补这层，runtime host 只会不断累积 pending request，`evictable_pages` 也永远停留在结构占位，后续 uploader/readback/refine 都没有可替换的演进点。
- 这次实现故意保持 CPU-only，并把 resident budget / eviction policy 固定在 runtime host 内部，好让未来 GPU uploader 只替换“如何 fulfill request”，而不是重拆 façade 或 render thread 的边界。

## Validation Summary

- `virtual_geometry_runtime_state_consumes_feedback_and_promotes_requested_pages`
  - 证明 feedback request 会在 budget 内消费，并优先复用 evictable resident page 的 slot
- `virtual_geometry_runtime_state_leaves_requests_pending_without_evictable_budget`
  - 证明在没有可回收 budget 时，runtime host 不会偷偷无限扩 resident cache
- `virtual_geometry_runtime_state_builds_prepare_frame_with_resident_pending_and_missing_clusters`
  - 证明 prepare snapshot 的 resident/pending/missing 分类在新反馈消费逻辑下保持兼容
- `virtual_geometry_prepare_filters_mesh_fallback_to_allowed_entities`
  - 证明当前 prepare-driven fallback filtering 没有被新的反馈消费路径破坏

## Remaining Route

- 真实 GPU page upload backend / copy queue plumbing
- page residency feedback readback / request completion source
- cluster hierarchy refine / split-merge / screen-space-error driven selection
- indirect draw / occlusion / BVH update 与 Virtual Geometry 的更深层耦合
- 真正替换当前 mesh fallback，而不是继续让 mesh fallback 承担 prepare baseline 的最终消费
