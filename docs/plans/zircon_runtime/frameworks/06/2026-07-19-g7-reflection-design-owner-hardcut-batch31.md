---
related_code:
  - docs/superpowers/specs/2026-05-08-reflection-type-registry-design.md
  - zircon_runtime/src/scene/reflect/builtin_reflection/registration.rs
  - zircon_runtime/src/scene/reflect/builtin_reflection/hierarchy.rs
  - zircon_runtime/src/scene/reflect/builtin_reflection/active_in_hierarchy.rs
  - zircon_runtime/src/scene/components/scene/mod.rs
  - zircon_runtime/src/scene/components/scene/identity.rs
  - zircon_runtime/src/scene/components/scene/hierarchy.rs
  - zircon_runtime/src/scene/components/scene/transform.rs
  - zircon_runtime/src/scene/components/scene/activation.rs
  - zircon_runtime/src/core/framework/scene/mobility.rs
  - zircon_runtime/src/scene/components/scene/camera.rs
  - zircon_runtime/src/scene/components/scene/mesh_renderer.rs
  - zircon_runtime/src/scene/components/scene/lighting.rs
  - zircon_runtime/src/scene/components/scene/physics.rs
implementation_files:
  - docs/superpowers/specs/2026-05-08-reflection-type-registry-design.md
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - python tools/check_conventions.py --only docs --json
  - git diff --check -- docs/superpowers/specs/2026-05-08-reflection-type-registry-design.md docs/plans/zircon_runtime/frameworks/06/2026-07-19-g7-reflection-design-owner-hardcut-batch31.md
---

# Frameworks06 G7 Reflection Design Owner 硬切 Batch 31

Plan: docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
Milestone: M2
Status: accepted
Files: ["docs/superpowers/specs/2026-05-08-reflection-type-registry-design.md"]
Date: 2026-07-19
Session: `frameworks06-g7-reflection-design-owner-hardcut-batch31-20260719`

## Scope Delivered

- 将 approved Reflection/TypeRegistry design 中两条已删除 `zircon_runtime/src/scene/components/scene.rs` 机器路径硬切到 folder-backed facade 与 current component leaves。
- `related_code` 指向 curated `scene/mod.rs` facade；`implementation_files` 严格跟随当前 `builtin_reflection::registration` 的 fixed-component 输入，记录 hierarchy/active-in-hierarchy 专用 adapter，并只列 identity、hierarchy、transform、activation、mobility、camera、mesh renderer、lighting 与 rigid body/physics component owners。
- 未把 animation、post-process 或其他未由当前 registration owner 注册的 scene leaves无依据扩入实现清单；不修改 approved design 方向、Runtime 行为，也不恢复 alias、shim、兼容 include 或手写 fixed-adapter tree。

## Fresh Testing Evidence

- 修改前 fresh G7：目标 approved design 有 `2` 个 missing-path violations；同一时序全局 `585` violations / `135` documents / `67,526` checked paths。
- 修改后 fresh G7：目标 approved design 与本记录 focused `0` violations；同一时序全局降到 `583` violations / `134` documents / `67,551` checked paths，继续保持 RED。
- current registration inventory 对照通过：`13` 个 derive registration 加 hierarchy/active-in-hierarchy `2` 个专用 adapter，9 个 component-owner symbol families 与 10 个 facade/owner paths 全部存在；exact-scope `git diff --check` 通过，仅输出工作树既有 LF/CRLF 提示。

## Review

- coordinator snapshot `671` exact2 独立双遍复审为 `C0 / I0 / M0 — Ready`；record `a89171bd762ae80aa23b96759acb0360736c6c5c092afc45c8b083e3a5cd034f`、approved design `2cfa529ad256d48547d65f76c14d6ec3f4016733a3e048f6b556cf088faf6eb7` 与 ordinal fingerprint `56cedf00e48f1c62524a77742a6e2e0bfc0c348a3b6cef02279e7bc9f8e8fd36` 两遍稳定无漂移。
- 复审确认 registration inventory、9 个 component family leaves + facade、现有 local-transform/rigid-body field codec owner 与 no-shim 事实全部准确；空的未跟踪 `scene/reflect/fixed` 目录没有文件、wiring 或引用，不构成兼容表面。

## Milestone Decision

本批 focused G7 与独立复审已通过，状态记为 `accepted`，等待通过协调器纳入 Frameworks06 管理提交。Frameworks06 M1/M2、全局 G7 与计划 06 均保持 `in_progress`。
