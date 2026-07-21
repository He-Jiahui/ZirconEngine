---
related_code:
  - docs/plans/zircon_editor/editor/01/fixed-2026-07-12-collider-shape-consumer-exhaustiveness.md
  - docs/plans/zircon_editor/editor/08/fixed-2026-07-12-rigid-body-sleep-policy-consumer-cutover.md
  - zircon_runtime/src/scene/components/scene/physics.rs
implementation_files:
  - docs/plans/zircon_editor/editor/01/fixed-2026-07-12-collider-shape-consumer-exhaustiveness.md
  - docs/plans/zircon_editor/editor/08/fixed-2026-07-12-rigid-body-sleep-policy-consumer-cutover.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - docs/plans/zircon_plugins/03-physics.md
tests:
  - python tools/check_conventions.py --only docs --json
  - git diff --check -- docs/plans/zircon_editor/editor/01/fixed-2026-07-12-collider-shape-consumer-exhaustiveness.md docs/plans/zircon_editor/editor/08/fixed-2026-07-12-rigid-body-sleep-policy-consumer-cutover.md docs/plans/zircon_runtime/frameworks/06/2026-07-19-g7-physics-fixed-record-owner-hardcut-batch30.md
---

# Frameworks06 G7 Physics Fixed Record Owner 硬切 Batch 30

Plan: docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
Milestone: M2
Status: accepted
Files: ["docs/plans/zircon_editor/editor/01/fixed-2026-07-12-collider-shape-consumer-exhaustiveness.md", "docs/plans/zircon_editor/editor/08/fixed-2026-07-12-rigid-body-sleep-policy-consumer-cutover.md"]
Date: 2026-07-19
Session: `frameworks06-g7-physics-fixed-record-owner-hardcut-batch30-20260719`

## Scope Delivered

- 将两份 canonical Physics fixed handoff frontmatter 中指向已删除 `zircon_runtime/src/scene/components/scene.rs` 的 `related_code` 机器路径硬切到 `scene/components/scene/physics.rs` current owner。
- `physics.rs` 同时拥有 ColliderShape/ColliderComponent 与 RigidBodyComponent/PhysicsSleepPolicy 所需的场景组件合同；不再用退役聚合文件冒充 Physics schema owner。
- 保留两份 fixed handoff 的 lifecycle、复现、根因、验收、验证和回传正文原样；没有创建第二份 failure/fixed 记录，也没有恢复 alias、shim、兼容 include 或旧组件字段。

## Fresh Testing Evidence

- 修改前 fresh G7：两份目标 fixed handoff 共 `2` 个 missing-path violations；同一时序全局 `587` violations / `137` documents / `67,521` checked paths。
- 修改后 fresh G7：两份目标 fixed handoff 与本记录 focused `0` violations；同一时序全局降到 `585` violations / `135` documents / `67,526` checked paths，继续保持 RED。
- `physics.rs` 中 ColliderShape、ColliderComponent、RigidBodyComponent 与 PhysicsSleepPolicy current owner 锚点全部存在；两份 canonical handoff 的 `status: fixed` 保持不变；exact-scope `git diff --check` 通过，仅输出工作树既有 LF/CRLF 提示。
- `audit_plan_output_records.py` 全库审计仍为 RED：报告 `15` 个既有 missing-notice/direct-record-limit 基线问题，均不命中本 exact3；本批没有修改 foreign-dirty Frameworks06 parent 或其他 child plan，也不宣称全库记录治理通过。

## Review

- coordinator snapshot `663` exact3 独立双遍复审为 `C0 / I0 / M0 — Ready`；Editor01 fixed `412f96f7471ebde6a5d9367c698d219b00084da6c20bba97fe24150806176599`、Editor08 fixed `e29b05ccabb990a4d8c727dc4f3e9326dd8037b62eedaffea43b87200d643def`、record `3930369607803d37e50c5dae26c7730470c3d6636566423b80aa0a5888564a1a` 与 ordinal fingerprint `6caa405873cdf62b21c0d66034966219b99aa76a4c232a90f4b9199e295ef31b` 两遍稳定无漂移。
- 复审确认两份 fixed artifact 各仅修改 frontmatter owner 一行，历史正文保持原样；current Physics schema owner、生产 `can_sleep` 清零与 no alias/shim 全部通过。

## Milestone Decision

本批 focused G7 与独立复审已通过，状态记为 `accepted`，等待通过协调器纳入 Frameworks06 管理提交。两份 origin fixed handoff 继续保持 `fixed`，Frameworks06 M1/M2、全局 G7、plan-output 全库审计与计划 06 均保持 `in_progress`。
