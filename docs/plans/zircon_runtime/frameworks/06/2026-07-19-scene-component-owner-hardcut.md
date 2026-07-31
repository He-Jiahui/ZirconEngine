---
related_code:
  - zircon_runtime/src/scene/components/mod.rs
  - zircon_runtime/src/scene/components/scene/mod.rs
  - zircon_runtime/src/scene/tests/component_structure/runtime_world_domains.rs
  - zircon_runtime/src/scene/tests/ecs_reflect/structure.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/scene_component_structure.rs
implementation_files:
  - zircon_runtime/src/scene/components/scene/activation.rs
  - zircon_runtime/src/scene/components/scene/animation.rs
  - zircon_runtime/src/scene/components/scene/camera.rs
  - zircon_runtime/src/scene/components/scene/hierarchy.rs
  - zircon_runtime/src/scene/components/scene/identity.rs
  - zircon_runtime/src/scene/components/scene/mesh_renderer.rs
  - zircon_runtime/src/scene/components/scene/node.rs
  - zircon_runtime/src/scene/components/scene/physics.rs
  - zircon_runtime/src/scene/components/scene/transform.rs
plan_sources:
  - docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - cargo +1.94.1 test -p zircon_runtime --lib scene::tests::ecs_reflect::structure::builtin_component_metadata_is_owned_by_zr_reflect_derives --locked --jobs 1 -- --exact --nocapture --test-threads=1
---

# Frameworks06 M2 Scene Component Owner Hard-Cut

Plan: docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md
Milestone: M2
Status: accepted
Files: ["docs/plans/zircon_plugins/01/fixed-2026-07-19-scene-component-hardcut-ecs-reflect-guard-drift.md", "docs/plans/zircon_runtime/frameworks/06-development-conventions-and-guardrails.md", "docs/plans/zircon_runtime/frameworks/06/2026-07-19-scene-component-hardcut-ecs-reflect-guard-drift-return.md", "docs/plans/zircon_runtime/frameworks/06/2026-07-19-scene-component-owner-hardcut.md", "zircon_runtime/src/scene/components/mod.rs", "zircon_runtime/src/scene/components/scene.rs", "zircon_runtime/src/scene/components/scene/activation.rs", "zircon_runtime/src/scene/components/scene/animation.rs", "zircon_runtime/src/scene/components/scene/camera.rs", "zircon_runtime/src/scene/components/scene/hierarchy.rs", "zircon_runtime/src/scene/components/scene/identity.rs", "zircon_runtime/src/scene/components/scene/mesh_renderer.rs", "zircon_runtime/src/scene/components/scene/mod.rs", "zircon_runtime/src/scene/components/scene/node.rs", "zircon_runtime/src/scene/components/scene/physics.rs", "zircon_runtime/src/scene/components/scene/transform.rs", "zircon_runtime/src/scene/tests/component_structure/runtime_world_domains.rs", "zircon_runtime/src/scene/tests/ecs_reflect/structure.rs", "zircon_runtime/src/tests/runtime_absorption/structure_convention/test_file_budget/scene_component_structure.rs"]
Date: 2026-07-19
Session: `frameworks06-m2-scene-component-owner-hardcut-r6-20260719`

## Scope Delivered

- 物理删除 796 行 `zircon_runtime/src/scene/components/scene.rs`，将组件声明硬切到按 domain 划分的 folder-backed owner；`scene/mod.rs` 只保留 module/re-export facade。
- 同步两个结构预算 guard 与 ECS reflection guard，不恢复 flat owner、shim、alias 或兼容 include。
- reflection guard 对 9 个 builtin component type path 逐一绑定真实 owner，并验证相邻声明自身携带 `zircon_reflect_derive::ZrReflect`；同一 `activation.rs` 内三个类型不能互相提供假阳性。
- 关闭 Plugins01 upward gate 暴露的旧 `components/scene.rs` compile blocker；Text01 的独立错误不吸收到本里程碑。

## Fresh Testing Evidence

- RED：Plugins01 job `c1fe7621b2bc4aa1b68291f8fa117248` / run `835c9dcd9316494eba57e2f929f1f7df` 在 `ecs_reflect/structure.rs` 读取已删除 `components/scene.rs`，exit 101。
- GREEN：Frameworks06 job `683eb23631aa4364a6cdbc82de80dddd` / run `23f5eb7e782443469c6a7862e305a06d`，source manifest fingerprint `fc564368ad11bf37d5632cc0b04bf3fde3e2c5bfe470e02a49b71e60f423d7d9`，1/1 passed、0 failed、8528 filtered、exit 0；test profile 编译 44m39s。
- `rustfmt +1.94.1 --edition 2024 --check zircon_runtime/src/scene/tests/ecs_reflect/structure.rs` 通过；exact-scope `git diff --check` 通过。

## Review

- 首轮独立只读复审：Critical 0 / Important 2 / Minor 1；发现重复 failure lifecycle、裸函数名 `--exact` 会执行 0 条、derive guard 仅为文件级。
- 修正后第二轮与最终增量复审：Critical 0 / Important 0 / Minor 0；canonical lifecycle 唯一、完整 libtest 路径实际执行 1 条、逐类型 derive 绑定无 activation 假阳性。

## 里程碑判定

Frameworks06 M2 的 scene component owner hard-cut 切片已满足代码、结构 guard、current-source behavior test 与独立复审要求，等待 coordinator milestone 原子提交。Frameworks06 总计划仍为 `in_progress`，本记录不声明 M1/M2 全计划完成。
