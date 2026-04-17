---
related_code:
  - zircon_scene/src/components.rs
  - zircon_scene/src/lib.rs
  - zircon_scene/src/render_extract.rs
  - zircon_graphics/src/runtime/mod.rs
  - zircon_graphics/src/runtime/hybrid_gi/mod.rs
  - zircon_graphics/src/runtime/server/create_viewport/mod.rs
  - zircon_graphics/src/runtime/server/viewport_record/mod.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/mod.rs
  - zircon_graphics/src/visibility/declarations/mod.rs
  - zircon_graphics/src/visibility/declarations/visibility_context.rs
  - zircon_graphics/src/visibility/declarations/visibility_history_snapshot.rs
  - zircon_graphics/src/visibility/declarations/visibility_hybrid_gi_probe.rs
  - zircon_graphics/src/visibility/declarations/visibility_hybrid_gi_feedback.rs
  - zircon_graphics/src/visibility/declarations/visibility_hybrid_gi_update_plan.rs
  - zircon_graphics/src/visibility/planning/mod.rs
  - zircon_graphics/src/visibility/planning/build_hybrid_gi_plan/mod.rs
  - zircon_graphics/src/visibility/context/from_extract_with_history/mod.rs
  - zircon_graphics/src/tests/hybrid_gi_visibility.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
  - zircon_graphics/src/tests/render_server_bridge.rs
  - zircon_render_server/src/types.rs
  - zircon_render_server/src/tests.rs
  - docs/assets-and-rendering/srp-rhi-render-server-architecture.md
  - docs/assets-and-rendering/index.md
implementation_files:
  - zircon_scene/src/components.rs
  - zircon_scene/src/lib.rs
  - zircon_graphics/src/runtime/mod.rs
  - zircon_graphics/src/runtime/hybrid_gi/mod.rs
  - zircon_graphics/src/runtime/server/create_viewport/mod.rs
  - zircon_graphics/src/runtime/server/viewport_record/mod.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/mod.rs
  - zircon_graphics/src/visibility/declarations/mod.rs
  - zircon_graphics/src/visibility/declarations/visibility_context.rs
  - zircon_graphics/src/visibility/declarations/visibility_history_snapshot.rs
  - zircon_graphics/src/visibility/declarations/visibility_hybrid_gi_probe.rs
  - zircon_graphics/src/visibility/declarations/visibility_hybrid_gi_feedback.rs
  - zircon_graphics/src/visibility/declarations/visibility_hybrid_gi_update_plan.rs
  - zircon_graphics/src/visibility/planning/mod.rs
  - zircon_graphics/src/visibility/planning/build_hybrid_gi_plan/mod.rs
  - zircon_graphics/src/visibility/context/from_extract_with_history/mod.rs
  - zircon_render_server/src/types.rs
plan_sources:
  - user: 2026-04-16 continue M5 Hybrid GI runtime representation slice
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-16-m5-flagship-capability-slots.md
tests:
  - zircon_graphics/src/tests/hybrid_gi_visibility.rs
  - zircon_graphics/src/tests/hybrid_gi_runtime.rs
  - zircon_graphics/src/tests/render_server_bridge.rs
  - zircon_render_server/src/tests.rs
  - cargo test -p zircon_graphics visibility_context_builds_hybrid_gi_probe_and_trace_plan --locked
  - cargo test -p zircon_graphics visibility_context_with_history_tracks_hybrid_gi_requested_probes --locked
  - cargo test -p zircon_graphics hybrid_gi_runtime_state_tracks_cache_residency_pending_updates_and_trace_schedule --locked
  - cargo test -p zircon_graphics hybrid_gi_runtime_state_deduplicates_probe_updates_and_reuses_evicted_slots --locked
  - cargo test -p zircon_graphics headless_wgpu_server_capability_gate_blocks_m5_flagship_opt_in_features --locked
  - cargo test -p zircon_graphics --lib --locked
  - cargo test -p zircon_render_server --locked
  - cargo test -p zircon_scene --locked
  - ./.codex/skills/zircon-dev/scripts/validate-matrix.ps1 -Package zircon_graphics
doc_type: milestone-detail
---

# M5 Hybrid GI Runtime Host Plan

**Goal:** 把 `Hybrid GI` 从“只有 feature descriptor + capability/profile gate + budget placeholder”推进到真正的 preprocess + CPU runtime host baseline，让它拥有可见 probe/trace 计划、radiance-cache 风格的 probe residency 宿主，以及 façade 可观测统计。

**Non-Goal:** 本轮不实现真实 radiance cache shading、screen probe gather、hardware RT tracing、surface cache、Lumen scene representation 或任何最终 GI 光照求解。

## Delivered Slice

- `zircon_scene::RenderHybridGiExtract` 现在不再只有两个 budget 数值，还携带：
  - `probes: Vec<RenderHybridGiProbe>`
  - `trace_regions: Vec<RenderHybridGiTraceRegion>`
- `VisibilityContext` 新增了 `Hybrid GI` 统一前处理输出：
  - `hybrid_gi_active_probes`
  - `hybrid_gi_update_plan`
  - `hybrid_gi_feedback`
- `zircon_graphics::runtime::HybridGiRuntimeState` 现在作为 viewport 级 runtime host，维护 probe slot、resident probe、pending update 与 trace schedule。
- `RenderStats` 新增 `Hybrid GI` 观测字段：
  - `last_hybrid_gi_active_probe_count`
  - `last_hybrid_gi_requested_probe_count`
  - `last_hybrid_gi_dirty_probe_count`
  - `last_hybrid_gi_cache_entry_count`
  - `last_hybrid_gi_resident_probe_count`
  - `last_hybrid_gi_pending_update_count`
  - `last_hybrid_gi_scheduled_trace_region_count`

## Visibility Contract

- `RenderHybridGiProbe` 定义：
  - `entity`
  - `probe_id`
  - `position`
  - `radius`
  - `resident`
  - `ray_budget`
- `RenderHybridGiTraceRegion` 定义：
  - `entity`
  - `region_id`
  - `bounds_center`
  - `bounds_radius`
  - `screen_coverage`
- `build_hybrid_gi_plan(...)` 当前规则：
  - 只消费统一 visibility 里的 `visible_entities`
  - probe 再做一次 sphere frustum test
  - active probe 按 `ray_budget` 优先级排序，再按 `probe_id` 稳定排序
  - trace region 按 `screen_coverage` 排序，再按 `region_id` 稳定排序
  - `probe_budget` 限制本帧真正发起的 non-resident probe request
  - `tracing_budget` 限制本帧真正进入 trace schedule 的 region 数
  - `dirty_requested_probe_ids` 只记录相对上一帧新增的 probe request
  - resident 但不再 active 的 probe 进入 `evictable_probe_ids`

## Runtime Host Contract

- `register_extract(Some(&RenderHybridGiExtract))`
  - 记录 probe 的 `ray_budget`
  - 为 extract 中标记 `resident = true` 的 probe 分配稳定 slot
- `ingest_plan(generation, &VisibilityHybridGiUpdatePlan)`
  - 让 `resident_probe_ids` 保持 resident
  - 只对 `dirty_requested_probe_ids` 里的 probe 生成一次 pending update
  - 刷新当前帧 `scheduled_trace_region_ids`
  - 刷新当前帧 `evictable_probe_ids`
- `apply_evictions(...)`
  - 回收 resident probe slot
  - 空闲 slot 会被后续 fulfill 的 probe 复用
- `fulfill_updates(...)`
  - 把 pending probe 升格为 resident
  - 清除对应 update request

## Capability And Server Behavior

- `submit_frame_extract(...)` 只有在 compiled pipeline 真正启用了 `BuiltinRenderFeature::GlobalIllumination` 时才会维护 `HybridGiRuntimeState`。
- 当前 headless `wgpu` 基线依然会把 `Hybrid GI` capability gate 关闭，因此 façade 统计仍然稳定回落到 `0`。
- 这条路径现在已经预埋好了 scene contract、visibility preprocess、runtime host 与 stats 边界；未来真正的 radiance cache / screen probe / RT hybrid lighting 可以继续沿这条边界向下生长，而不需要重拆 render server façade。

## Validation Summary

- `visibility_context_builds_hybrid_gi_probe_and_trace_plan`
  - 证明可见 probe、requested probe、trace schedule 与 feedback 都能稳定生成
- `visibility_context_with_history_tracks_hybrid_gi_requested_probes`
  - 证明跨帧 dirty request 只记录新增 probe
- `hybrid_gi_runtime_state_tracks_cache_residency_pending_updates_and_trace_schedule`
  - 证明 runtime host 能维护 stable slot、pending update 与 trace schedule
- `hybrid_gi_runtime_state_deduplicates_probe_updates_and_reuses_evicted_slots`
  - 证明重复请求不会重复入队，evict 后 slot 会被复用
- `headless_wgpu_server_capability_gate_blocks_m5_flagship_opt_in_features`
  - 证明 capability gate 关闭时，Hybrid GI 与 Virtual Geometry 的新统计都保持 `0`

## Remaining Route

- radiance cache shading / reservoir update
- screen probe gather / resolve
- RT hybrid lighting / trace backend
- scene representation / surface cache / card atlas
- 与 visibility、BVH/AS、history resolve、post-process 的更深层协同
