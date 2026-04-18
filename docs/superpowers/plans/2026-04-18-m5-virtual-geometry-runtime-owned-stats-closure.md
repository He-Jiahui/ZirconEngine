---
related_code:
  - zircon_render_server/src/types.rs
  - zircon_render_server/src/tests.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/submission_record_update.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/record_submission/update_virtual_geometry_runtime.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/update_stats/virtual_geometry_stats.rs
  - zircon_graphics/src/tests/render_server_bridge.rs
implementation_files:
  - zircon_render_server/src/types.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/submission_record_update.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/record_submission/update_virtual_geometry_runtime.rs
  - zircon_graphics/src/runtime/server/submit_frame_extract/update_stats/virtual_geometry_stats.rs
plan_sources:
  - user: 2026-04-18 列出后续所有 tasks，把它们作为 todo，然后继续深入
  - user: 2026-04-18 Virtual Geometry 的 unified indirect ownership 下沉 / residency-manager cascade 继续推进
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/sessions/20260417-1415-m5-vg-slot-assignment-ownership.md
tests:
  - zircon_render_server/src/tests.rs
  - zircon_graphics/src/tests/render_server_bridge.rs
  - cargo test -p zircon_render_server --offline --locked stable_render_handles_and_frame_types_are_constructible
  - cargo test -p zircon_graphics --offline --locked headless_wgpu_server_exposes_current_m5_flagship_baselines_without_rt_capabilities
doc_type: milestone-detail
---

# M5 Virtual Geometry Runtime-Owned Stats Closure

**Goal:** 把 `Virtual Geometry` 的 prepare-owned indirect authority 和 GPU completion/readback truth 再往 `RenderServer` façade 推一层，让 `RenderStats` 直接暴露这条 M5 主链的关键 ownership 指标，而不是继续只看 renderer 末端私有计数。

> Repository reality note: 当前 `zircon_graphics` 内的实际 façade/runtime 提交路径已经迁移到 `runtime/render_framework/*`，这份文档里提到的“render server submit path”应读作 `WgpuRenderFramework -> runtime/render_framework/submit_frame_extract/*`，而不是旧的 `runtime/server/*` 磁盘布局。

## Delivered Slice

- `RenderStats` 新增两项稳定可观测字段：
  - `last_virtual_geometry_completed_page_count`
  - `last_virtual_geometry_indirect_segment_count`
- `last_virtual_geometry_indirect_draw_count` 的来源也已切到 `PreparedRuntimeSubmission.virtual_geometry_prepare.unified_indirect_draws()`，不再继续只信 renderer 末端记录的 draw 数。
- `last_virtual_geometry_completed_page_count` 直接取自本帧 GPU completion 的 `completed_page_assignments.len()`，因此 façade 现在能看见真实 uploader/readback 的 streaming 完成规模。
- `last_virtual_geometry_indirect_segment_count` 直接取自 prepare-owned `cluster_draw_segments` 的非 `Missing` submission 数，因此 façade 现在能看见 visibility/runtime 持有的 unified indirect authority 边界。
- `last_virtual_geometry_indirect_buffer_count` 仍然保留 renderer-local 统计，因为共享 args buffer 复用仍然是 renderer 内部资源分配事实，而不是 runtime host/prepare contract。

## Why This Slice Exists

- 之前 `RenderStats` 已经能暴露：
  - visible/requested/dirty cluster/page 规模
  - runtime host 的 `page_table / resident / pending` 规模
  - renderer 末端的 indirect draw/buffer 计数
- 但中间仍有一个所有权断层：
  - facade 看不到本帧到底完成了多少真实 page upload
  - facade 也看不到 prepare-owned unified indirect segment contract
  - `last_virtual_geometry_indirect_draw_count` 仍然默认相信 renderer 私有记录
- 这会让后续的 unified indirect ownership 下沉、residency-manager cascade、GPU-driven compaction 很难从 façade 侧确认“当前究竟是谁在持有 submission 真值”。
- 本切片把关键统计重新绑回 `prepare + GPU completion` 两条 authority source 后，`RenderServer` 对 M5 Virtual Geometry 的观测面就更接近真实 runtime/readback contract，而不是 renderer side-effect。

## Validation Summary

- `stable_render_handles_and_frame_types_are_constructible`
  - 证明 `zircon_render_server` 的稳定 façade 默认统计快照已经包含新字段，并保持零值初始化。
- `headless_wgpu_server_exposes_current_m5_flagship_baselines_without_rt_capabilities`
  - 证明 pure `wgpu` baseline 下的真实 M5 提交会把：
    - `last_virtual_geometry_completed_page_count`
    - `last_virtual_geometry_indirect_segment_count`
    - prepare-owned `last_virtual_geometry_indirect_draw_count`
    一起写入 façade 统计。

## Remaining Route

- `Virtual Geometry`
  - 继续推进 unified indirect ownership 下沉，让更多 submit/readback/stats 直接围绕 prepare/runtime authority 组织。
  - 当前 follow-on 已经继续落下两层：其一，`VisibilityVirtualGeometryDrawSegment.lineage_depth` 已经沿 `prepare -> unified indirect -> GPU submission -> indirect args shader` 下沉；其二，runtime residency completion 已经会在 ancestor / descendant 内部优先回收更远 lineage distance，保护更近的 split-merge frontier page。
  - 继续推进 deeper cluster raster consumption，把更深 hierarchy / streaming frontier 对真实 raster submit 的影响扩展开。
  - 继续推进 residency-manager cascade 与 split-merge hysteresis，减少 slot/page/frontier 在深层切换时的抖动。
- `Hybrid GI`
  - 继续推进更完整的 scene-driven screen-probe hierarchy / RT hybrid lighting continuation。
