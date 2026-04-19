---
related_code:
  - zircon_graphics/src/scene/scene_renderer/mesh/shaders/virtual_geometry_indirect_args.wgsl
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/mesh/shaders/virtual_geometry_indirect_args.wgsl
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
plan_sources:
  - user: 2026-04-19 把 visibility-owned authority 压进更真实的 GPU-generated args source / compaction ownership
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-19-m5-virtual-geometry-submission-index-gpu-args-authority.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect_propagates_multi_primitive_draw_ref_compaction_into_gpu_args -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_render -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_submission_authority -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_submission_execution_order -- --nocapture
  - cargo check -p zircon_graphics --lib --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Draw-Ref Compaction GPU Source

## Goal

继续削掉 `Virtual Geometry` unified-indirect 路径里残留的 CPU residue：

- `submission_index` 已经能改变真实 GPU-generated indirect args
- 但当同一 visibility-owned segment 下存在多个真实 draw-ref 时
- WGSL 仍然只吃 segment-level truth，而看不到 shared args 自己的 compaction rank

这会导致 later primitive draw-ref 虽然已经进入真实 `draw_ref_buffer`，却仍然复用同一份 segment-level `first_index` 起点。

## Delivered Slice

### 1. 红灯先锁定 multi-primitive compaction 漏洞

新增 `virtual_geometry_unified_indirect_propagates_multi_primitive_draw_ref_compaction_into_gpu_args`：

- 构造一个单 entity、单 segment、双 primitive 的 glTF
- primitive 共享同一 visibility-owned segment，但 `mesh_index_count` 不同，因此会保留两条真实 draw-ref
- 断言第二条 draw-ref 的 indirect args 不应继续停在 `(first_index=0, index_count=6)`
- 期望它改成沿 shared args compaction rank 收缩后的 `(first_index=3, index_count=3)`

实现前这条测试稳定失败，证明真实 GPU source 仍然没有消费 draw-ref compaction truth。

### 2. WGSL 现在直接从 shared draw-ref buffer 计算 same-segment compaction rank

`virtual_geometry_indirect_args.wgsl` 新增三条 helper：

- `draw_ref_count_for_segment(...)`
- `draw_ref_rank_within_segment(...)`
- `draw_ref_compaction_cluster_offset(...)`

compute shader 现在不再只相信 `segment.submission_index`，而会直接扫描真实 `draw_ref_buffer`：

- 统计当前 segment 在 shared args 里实际保留了多少条 draw-ref
- 计算当前 invocation 在同 segment 内的 compaction rank
- 把这条 rank 继续映射到真实 `first_index / index_count`

这意味着同一个 segment 下的 later primitive draw-ref，终于开始服从 shared args 自己的 GPU source truth。

### 3. 影响范围被刻意压窄到 “同 segment 的多 draw-ref”

这轮没有重新发明新的 CPU-side sort key，也没有扩大到所有 indirect draw：

- 单 primitive segment 不受影响
- 不同 segment 之间的 order 仍然继续由 prepare/visibility authority 主导
- 只有 “shared args 中同一 segment 保留多条 draw-ref” 的 case 才会进入新的 compaction offset

这样可以把 slice 控制在真正的 GPU source authority 下沉，而不去打乱已经验证过的 slot/page/frontier/lineage 行为层。

## Why This Slice Matters

M5 当前的 unified-indirect 主链已经做到：

- prepare-owned unified indirect order
- authoritative segment source
- authoritative draw-ref/args source
- visibility-owned submission execution order
- submission-index GPU args authority

但仍有最后一层 compaction 漏洞：

- shared args 里的 draw-ref cardinality 已经是真的
- shared args 里的 draw-ref order 也已经是真的
- WGSL 却还只消费 segment-level metadata

这让 “GPU-generated args source” 还不是真正意义上的 source。现在这条 truth 已经继续压进 compute shader 自己消费的 `draw_ref_buffer`。

## Validation Summary

- 红灯
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect_propagates_multi_primitive_draw_ref_compaction_into_gpu_args -- --nocapture`
- 绿灯
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect_propagates_multi_primitive_draw_ref_compaction_into_gpu_args -- --nocapture`
- 回归
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_prepare_render -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_submission_authority -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_submission_execution_order -- --nocapture`
  - `cargo check -p zircon_graphics --lib --offline --locked`

## Remaining Gaps

- 这轮已经把 same-segment multi-draw-ref compaction 压进真实 GPU source，但 renderer 仍然保留 “每条 pending draw 各发一条 `draw_indexed_indirect(...)`” 的提交形态；更深层的 GPU-driven submission ownership 仍然要继续推进到更真实的 visibility-owned args source / compaction / submission authority。
- split-merge / residency-manager cascade 还需要继续把 final completion / page-table / hot-frontier truth 往 prepare recycle 与 hierarchy hysteresis 深处压。
