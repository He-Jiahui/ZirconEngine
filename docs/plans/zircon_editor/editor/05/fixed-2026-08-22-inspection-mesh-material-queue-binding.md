---
handoff_kind: fixed
status: fixed
failure_scope: local
created_at: 2026-08-22
resolved_at: 2026-08-22
summary_slug: inspection-mesh-material-queue-binding
origin_plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
fixing_plan: docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md
origin_child_dir: docs/plans/zircon_editor/editor/05
fixing_child_dir: docs/plans/zircon_editor/editor/05
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/tests/inspection.rs
---

# Editor05: inspection-mesh-material-queue-binding 验证失败回写

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md`
- 来源执行切片：Render11 Shader06 realtime IBL managed library validation
- 修复责任计划：`docs/plans/zircon_editor/editor/05-scene-editing-hierarchy-and-gizmos.md`
- 交接原因：同一编号计划拥有已集成快照及其前向修复。

## 失败现象与复现证据

- 验证回写：`Render11 Shader06 realtime IBL managed library validation` — Windows managed validate-matrix compilation of zircon_runtime with text_oversized_run_keeps_one_logical_shaped_line reaches zircon_runtime/src/scene/tests/inspection.rs:96 and fails E0425 because mesh_material_queue is read from an undeclared inspection binding.

## 最低共享层根因

The inspection test fixture changed its queue assertion without preserving or reintroducing the inspection value that owns mesh_material_queue.

## 架构修复验收

- The inspection test binds the canonical inspection result before reading mesh_material_queue, preserves the intended queue assertion, and the originating managed zircon_runtime validation advances past inspection.rs E0425.

## 禁止临时方案

- 不回滚已集成快照来掩盖普通测试失败；应通过前向修复返回 `fixed-*` 记录。
- 不得添加别名、兼容垫片、静默回退、测试旁路或调用点特例。

## 修复结果与回传

- 根因：测试在 `WorldInspection` hard cut 后保留了 `mesh_material_queue` 断言，却未绑定其新的权威 `WorldInspectionFieldsArtifact`。
- 架构修复：测试先以 `World::inspection_artifact()` 读取 hierarchy，再以 `World::inspection_fields_artifact(child)` 绑定字段 artifact；所有 MeshRenderer field 断言均从同一 `fields` binding 读取。没有恢复旧 snapshot API、别名或测试旁路。
- 验证：`rustfmt --edition 2024 --check zircon_runtime/src/scene/tests/inspection.rs` 通过；fixture binding/source guard 通过；Render11 Shader06 的 Windows managed zircon_runtime 编译已越过原 `inspection.rs:96` E0425。
- 回传：该编译阻塞已解除；上游 Render11/Shader06 验证可继续其余 runtime/WGPU gate，Editor05 其余 failure 仍保持 open。

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 验证与未完成项 |
|---|---|---|---|
| 2026-08-22 +08:00 | `fixed / returned` | `mesh_material_queue` 断言改为消费 canonical field artifact，保留字段契约并移除对已删除 snapshot 绑定的依赖。 | rustfmt、source guard 通过，受管 runtime 编译已越过原 E0425；仅关闭此 fixture compile blocker，不代表 Editor05 整体验收。 |

## 2026-08-27 structured-path normalization

The structured `related_code` entry now names the tracked Rust file without the
diagnostic line suffix. The original `inspection.rs:96` location remains dated
evidence in the body; no foreign inspection test bytes or fixed result changed.
