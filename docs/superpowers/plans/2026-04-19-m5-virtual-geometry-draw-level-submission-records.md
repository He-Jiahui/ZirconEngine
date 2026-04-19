---
related_code:
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_records.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/mod.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_execution_order.rs
  - zircon_graphics/src/tests/virtual_geometry_args_source_authority.rs
implementation_files:
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/read_mesh_draw_submission_records.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_virtual_geometry/last_state/mod.rs
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_execution_order.rs
  - zircon_graphics/src/tests/virtual_geometry_args_source_authority.rs
plan_sources:
  - user: 2026-04-19 Virtual Geometry deeper unified indirect / GPU-generated args / cluster-raster submission ownership
  - .codex/plans/Zircon SRP_RHI Rendering Architecture Roadmap.md
  - docs/superpowers/plans/2026-04-19-m5-virtual-geometry-visibility-owned-submission-execution-order.md
  - docs/superpowers/plans/2026-04-19-m5-virtual-geometry-submission-index-gpu-args-authority.md
tests:
  - zircon_graphics/src/tests/virtual_geometry_unified_indirect.rs
  - zircon_graphics/src/tests/virtual_geometry_submission_execution_order.rs
  - zircon_graphics/src/tests/virtual_geometry_args_source_authority.rs
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_renderer_submission_records_preserve_draw_level_tokens_for_repeated_primitives -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_submission_execution_order -- --nocapture
  - cargo test -p zircon_graphics --offline --locked virtual_geometry_args_source_authority -- --nocapture
  - cargo check -p zircon_graphics --offline --locked
doc_type: milestone-detail
---

# M5 Virtual Geometry Draw-Level Submission Records

## Goal

继续把 `Virtual Geometry` 的 unified-indirect authority 从 GPU buffer/readback 往 renderer 自身的 submission truth 再压一层。

前几刀已经证明：

- shared indirect args 会保留 repeated primitive draw-ref
- GPU-generated args 会保留 `submission_index` 与 `draw_ref_rank`
- renderer 的 coarse submission order 也会随 visibility-owned unified indirect 翻转

但 renderer last-state 这边仍然只稳定暴露粗粒度的：

- `(entity, page_id)`

这对 repeated primitive / same-segment compaction 不够。到了这一步，buffer/shader 已经拥有 draw-level truth，但 renderer side 自己还拿不出对应的 draw-level submission record。

## Delivered Slice

### 1. 红灯锁定 “renderer submission record 仍然只剩 coarse truth”

新增回归：

- `virtual_geometry_renderer_submission_records_preserve_draw_level_tokens_for_repeated_primitives`

测试构造的是：

- 单 entity
- 单 visibility-owned segment
- 双 primitive model

也就是最典型的 repeated draw-ref / repeated indirect args authority 场景。

这条测试要求 renderer last-state 不只回报两条 `(2, 300)`，而要能继续区分：

- `submission_index`
- `draw_ref_rank`

实现前 helper 不存在，说明 renderer side 还没有把这份 finer-grained truth 保住。

### 2. last-state 新增 draw-level submission record helper

新增：

- `read_last_virtual_geometry_mesh_draw_submission_records_with_tokens()`

它会继续消费真实 GPU readback 的 `submission_tokens`，再把 renderer 已保存的：

- `entity`
- `page_id`
- `indirect_args_offset`
- `original_index`

重新收束成 draw-level record：

- `(entity, page_id, submission_index, draw_ref_rank)`

因此 repeated primitive 场景现在不再只剩 coarse renderer order；renderer side 自己也能继续观察到 unified-indirect compaction 后的 finer-grained submission truth。

### 3. 同步修掉 VG 回归套件里的 project import 漂移

在继续跑更宽的 `Virtual Geometry` 套件时，还暴露了两份现有测试文件的下层支持漂移：

- `virtual_geometry_submission_execution_order.rs`
- `virtual_geometry_args_source_authority.rs`

它们还在从 `zircon_asset` root 导入：

- `ProjectManager`
- `ProjectManifest`
- `ProjectPaths`

而当前仓库 reality 已经迁到：

- `zircon_asset::project::*`

这轮也顺手把这些 import / call site 一起收口掉，避免 VG 套件在本轮 slice 外围继续卡在旧导出面上。

## Why This Slice Matters

如果 repeated primitive / same-segment compaction 的 finer-grained truth 只存在于：

- `draw_ref_buffer`
- `submission_buffer`
- GPU-generated indirect args

而 renderer 自己最后保存的 submission record 仍然只有 `(entity,page)`，那么 unified-indirect authority 到 renderer 这一层仍然是半断开的。

补上这层以后：

- buffer/readback 有 draw-level truth
- renderer last-state 也开始有 draw-level truth

这样后续继续往更深的 cluster-raster submission ownership 或 render-framework/runtime-host 统计闭环推进时，就不需要再把 repeated draw / compaction authority 重新从 GPU buffer 侧临时猜回来了。

## Validation Summary

- 红灯
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_renderer_submission_records_preserve_draw_level_tokens_for_repeated_primitives -- --nocapture`
- 绿灯
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_renderer_submission_records_preserve_draw_level_tokens_for_repeated_primitives -- --nocapture`
- 回归
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_unified_indirect -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_submission_execution_order -- --nocapture`
  - `cargo test -p zircon_graphics --offline --locked virtual_geometry_args_source_authority -- --nocapture`
  - `cargo check -p zircon_graphics --offline --locked`

## Remaining Route

- `Virtual Geometry` 下一条仍然更像真实行为主链，而不是 observability：
  - 继续把 unified-indirect / submission token / draw-ref rank 的 authority 往更深的 cluster-raster execution 或 runtime stats/host contract 下沉
  - 或切回更深的 residency-manager cascade / split-merge frontier policy
