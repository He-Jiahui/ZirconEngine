---
related_code:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/virtual_geometry_indirect_args.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_authority_records.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_execution_indices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_execution_records.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_execution_segments.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_records.rs
  - zircon_runtime/src/graphics/tests/virtual_geometry_submission_execution_order.rs
implementation_files:
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/shaders/virtual_geometry_indirect_args.wgsl
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_authority_records.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_execution_indices.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_execution_records.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/gpu_readback/read_indirect_execution_segments.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_records.rs
plan_sources:
  - user: 2026-04-20 continue M5 Virtual Geometry without pausing and keep pushing actual execution truth below host-built execution records
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-20-m5-virtual-geometry-gpu-authority-submission-records.md
tests:
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_execution_records_survive_with_execution_indices_and_gpu_authority_buffer_only --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_execution_segments_survive_with_execution_indices_and_gpu_authority_buffer_only --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_submission_execution_order --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_unified_indirect --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_args_source_authority --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo test -p zircon_runtime --locked --offline virtual_geometry_execution_stats --target-dir D:/cargo-targets/zircon-workspace-compile-recovery -- --nocapture
  - cargo check -p zircon_runtime --locked --offline --target-dir D:/cargo-targets/zircon-workspace-compile-recovery
doc_type: milestone-detail
---

# M5 Virtual Geometry GPU Authority Execution Records

## Goal

继续把 `Virtual Geometry` 的 actual execution subset 从 renderer host 自己缓存的 execution records 往下压。上一刀已经让 execution indices 可以配合 GPU authority sidecar 恢复 submission records，但 `read_last_virtual_geometry_indirect_execution_records()` 和 `read_last_virtual_geometry_indirect_execution_segments_with_entities()` 仍然要求 host-built `execution_records_buffer` 存活。

这一刀的目标是让 GPU authority buffer 自己携带足够完整的 execution template，使 execution indices 可以直接索引它恢复：

- execution records
- execution segments

而不再要求 renderer host 再持有一份专门的 execution-record mirror。

## What Changed

### GPU authority sidecar 从 submission tuple 扩成完整 execution template

`virtual_geometry_indirect_args.wgsl` 现在写出的 `SubmissionAuthorityRecord` 不再只包含：

- `draw_ref_index`
- `entity`
- `page_id`
- `submission_token`

而是直接扩成和 execution record 对齐的模板：

- `draw_ref_index`
- `cluster_start_ordinal`
- `cluster_span_count`
- `cluster_total_count`
- `page_id`
- `submission_slot`
- `state`
- `lineage_depth`
- `lod_level`
- `frontier_rank`
- `submission_index`
- `draw_ref_rank`
- `entity_lo`
- `entity_hi`

这意味着 GPU compute pass 现在已经不只是写一份 “submission token sidecar”，而是在 draw-ref 粒度上写出一份可以被 execution indices 直接索引的 cluster-raster authority template。

### Execution readback 新增 authority fallback

当前吸收后的 `zircon_runtime/src/graphics/**` 路径里：

- `read_last_virtual_geometry_indirect_execution_records()`
- `read_last_virtual_geometry_indirect_execution_segments_with_entities()`

现在都会在 `last_virtual_geometry_indirect_execution_records_buffer` 缺失时：

1. 读取 `last_virtual_geometry_indirect_execution_buffer` 得到 actual execution order 的 draw-ref index 列表
2. 读取 GPU-generated authority records
3. 用 execution indices 对 authority template 做 gather
4. 恢复出真实 execution records / execution segments

因此 host-built `execution_records_buffer` 已经从 “唯一 execution readback source” 退回到 “优先但可缺失的缓存层”。

### Recursion edge case 也一起封住

`read_last_virtual_geometry_indirect_execution_draw_ref_indices()` 之前在 `execution_buffer` 缺失时会回退到 `read_last_virtual_geometry_indirect_execution_records()`。这在 execution-record fallback 引入后会形成自递归风险。

现在的规则变成：

- 只有 `execution_records_buffer` 还在时，`execution indices` 才会回退到 `execution records`
- 两者都缺失时直接返回空集

因此 deeper fallback 不会再互相套娃。

## Regression Coverage

本轮新增两条红绿回归：

- `virtual_geometry_execution_records_survive_with_execution_indices_and_gpu_authority_buffer_only`
- `virtual_geometry_execution_segments_survive_with_execution_indices_and_gpu_authority_buffer_only`

它们都会故意清掉：

- host-built `execution_records_buffer`
- indirect submission buffer
- indirect args buffer
- draw-ref buffer
- segment buffer

只保留：

- actual execution indices
- expanded GPU authority buffer

修补前 `execution records / segments` 直接掉回空集；修补后都能恢复正确的 execution truth。

## Why This Matters

这一刀比上一刀更靠近真正的 GPU-driven consume source：

- submission readback 已经可以不依赖 CPU submission records
- execution readback 现在也可以不依赖 CPU execution records
- GPU authority sidecar 已经从 token mirror 变成 cluster-raster authority template

因此当前 Virtual Geometry 剩余最薄的残点已经进一步收窄为：

- actual execution subset 的最终 “哪几条 draw 真的执行了” 仍然主要靠 renderer host 维护 `execution indices`
- 更深的 unified indirect / cluster-raster execution ownership 还没完全变成 GPU-generated consume source

也就是说，下一刀如果继续优先 Virtual Geometry，最值当的方向已经不是再补 host mirror，而是继续把 actual execution subset 本身往更真实的 GPU-generated args / cluster-raster execution ownership 压。
