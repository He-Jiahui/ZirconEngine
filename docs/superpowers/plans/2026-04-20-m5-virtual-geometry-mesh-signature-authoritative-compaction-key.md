---
related_code:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/pending_mesh_draw.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_execution_order.rs
  - docs/assets-and-rendering/render-framework-architecture.md
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/mesh/build_mesh_draws/build/build_shared_indirect_args_buffer.rs
plan_sources:
  - user: 2026-04-20 M5 继续回到更深的 unified indirect / cluster-raster / GPU-generated args authority
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - .codex/sessions/20260417-1415-m5-vg-slot-assignment-ownership.md
tests:
  - cargo test -p zircon_graphics --offline --locked shared_indirect_args_layout_keeps_mesh_signature_in_authoritative_compaction_key -- --nocapture
  - cargo test -p zircon_graphics --offline --locked shared_indirect_args_layout -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_submission_execution_order -- --nocapture
  - cargo check -p zircon_graphics --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Mesh-Signature Authoritative Compaction Key

## Goal

继续把 `Virtual Geometry` 的 unified-indirect authority 往 `GPU-generated args / compaction` 下沉，避免 shared layout 只在排序阶段认 `mesh_signature`、而在 args compaction 阶段又把它丢掉。

## Problem

此前 `build_shared_indirect_args_layout(...)` 已经把 `mesh_signature` 纳入：

- pending draw submission order key
- authoritative draw-ref submission order key

但 draw-ref compaction 仍然只按：

- `mesh_index_count`
- `segment_key`

分组并分配 occurrence rank。

这会留下一个 authority 漏口：

- authoritative draw-ref 列表里更早的 primitive signature 先占用 rank
- 后面 signature 不同、但 `segment_key + mesh_index_count` 相同的 surviving primitive
- 在 drawable subset 里会被错误重映射回前一个 primitive 的 args slot / draw-ref rank

也就是说 shared layout 的排序 truth 和 compaction truth 还没有真正收敛成同一份 primitive identity。

## Delivered Slice

### 1. authoritative compaction key 现在显式包含 `mesh_signature`

`DrawRefGroupKey` 现在扩展为：

- `mesh_index_count`
- `mesh_signature`
- `segment_key`

这条 key 同时用于：

- authoritative occurrence rank
- pending occurrence rank
- indirect args slot reuse / fallback

因此排序阶段和 compaction 阶段终于共享同一份 primitive identity。

### 2. surviving later primitive 不再退回更早 args slot

新增红绿单测：

- `shared_indirect_args_layout_keeps_mesh_signature_in_authoritative_compaction_key`

它证明：

- authoritative 列表里两个 primitive 共享同一 `segment_key + mesh_index_count`
- 但 `mesh_signature` 不同
- 当 drawable subset 只剩后一个 primitive 时
- layout 仍然保留它的 later authoritative draw-ref rank，而不会塌回更早 primitive 的 args slot

### 3. unified-indirect / submission regression 保持绿色

这条修补没有打穿现有 M5 主链：

- repeated primitive GPU args authority
- transparent submission execution order
- unified indirect fallback synthesis

说明 compaction key 继续变得更 authoritative，但并没有把已有 cluster-raster submission contract 打乱。

## Why This Matters

这条修补虽然小，但它把 shared layout 的最后一层 “排序认 signature、compaction 不认 signature” 的内部分裂收掉了。

结果是：

- shared layout 更接近真实的 primitive identity authority
- GPU-generated args 不再只靠 sort key 偶然保持正确
- 后续要继续下沉到更真实的 visibility-owned / GPU-generated args source 时，不会再把 primitive identity 漏回 CPU compaction residue

## Validation

- `cargo test -p zircon_graphics --offline --locked shared_indirect_args_layout_keeps_mesh_signature_in_authoritative_compaction_key -- --nocapture`
- `cargo test -p zircon_graphics --offline --locked shared_indirect_args_layout -- --nocapture`
- `cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect -- --nocapture`
- `cargo test -p zircon_graphics --offline --locked virtual_geometry_submission_execution_order -- --nocapture`
- `cargo check -p zircon_graphics --offline --locked`

## Remaining Route

- 继续把这份 authoritative compaction truth 往更真实的 visibility-owned / GPU-generated args source 推进，减少 `build_mesh_draws(...)` 内仍然存在的 CPU-side per-draw submission bookkeeping。
- 继续推进 deeper cluster-raster submission ownership，让 unified indirect authority 不只在 shared layout / readback 层准确，也更完整地主导实际 indirect execution。
