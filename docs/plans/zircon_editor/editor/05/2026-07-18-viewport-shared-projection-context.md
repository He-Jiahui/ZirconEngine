---
status: in_progress
plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
failure: docs/plans/zircon_editor/editor/05/failure-2026-07-18-viewport-pointer-candidate-regeneration.md
session: editor05-viewport-shared-projection-context-20260718
---

# Viewport shared projection context

## 产出记录与时间

| 日期 | 状态 | 完成项目与验证证据 |
| --- | --- | --- |
| 2026-07-18 | 前置归属明确 | `projected_ring_segments.rs` 已存在来源性能会话的 `Vec::with_capacity(48)` 止损且无活动租约；本 Session 在 exact8 中显式接管其 current hash，不把该一行冒充本轮原创，也不吸收同 failure 的其他脏文件。 |
| 2026-07-18 | RED 已确认 | 先加入 2 个缺失合同：透视 context 必须稳定投影 viewport 中心并复用 camera scale；pointer candidate root 必须恰好构造一次 `ViewportProjectionContext`，4 个叶投影器构造次数必须为 0。静态 RED 为 `tests=2 / context impl=0 / root ctor=0`。 |
| 2026-07-18 | 源码完成，受管验收待屏障 | `projection.rs` 新增借用 camera 的 `ViewportProjectionContext`，一次预计算 projection × view 并统一 world-units-per-pixel；precision root 构造一次，handle/gizmo/renderable/ring 全部改借用 context，旧 free `projected_point` 硬删除。6 个叶文件 rustfmt、静态合同 13/13、旧三参数候选调用扫描 0、exact 8/8、`git diff --check`、staged 0 通过。 |
| 2026-07-18 | 未完成项明确保留 | 未声明 Cargo、matrix counter、1k/10k node visits、CPU p95、独立 review、failure fixed 或 commit。Coordinator01 immutable full-input snapshot failure 关闭后需受管验证；同一 viewport failure 仍需 shared render/pointer gizmo extract 与 runtime-visible/BVH candidate backend。 |
