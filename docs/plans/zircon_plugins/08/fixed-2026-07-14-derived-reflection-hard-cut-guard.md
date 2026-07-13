---
handoff_kind: fixed
status: fixed
created_at: 2026-07-14
resolved_at: 2026-07-14
summary_slug: derived-reflection-hard-cut-guard
origin_plan: docs/plans/zircon_plugins/08-zr-vm.md
fixing_plan: docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
origin_child_dir: docs/plans/zircon_plugins/08
fixing_child_dir: docs/plans/zircon_runtime/runtime/15
related_code:
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/scene_fixed_lights.rs
  - zircon_runtime/src/scene/reflect/builtin_reflection/registration.rs
  - zircon_runtime/src/scene/reflect/derived/component_adapter.rs
tests:
  - cargo test -p zircon_runtime --lib runtime_15_scene_fixed_light_reflection_write_fields_are_child_owner -- --nocapture
---


# Runtime 15：派生反射硬切后结构守卫仍指向已删除 owner

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/08-zr-vm.md`
- 来源执行切片：M1-T2 `fixed/` 手写反射迁移为 derive 并删除
- 修复责任计划：`docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md`
- 交接原因：最低共享原因是 Runtime 15 M4 的生产文件预算守卫把一次中间拆分结果固化为永久源码路径；Plugins 08 按既定架构删除该手写 owner 后，守卫需由 Runtime 15 改为验证最终派生反射边界。

## 失败现象与复现证据

Plugins 08 M1-T2 已删除 `scene/reflect/fixed/mod.rs`、`lights.rs` 与
`lights/write_fields.rs`，但
`runtime_15_scene_fixed_light_reflection_write_fields_are_child_owner` 仍通过
`read_runtime_src` 读取后两条旧路径，运行时会在读取不存在文件时失败。

2026-07-14 静态复现结果：`FixedModuleExists = false`，同时
`GuardReadsDeletedParent = true`、`GuardReadsDeletedChild = true`。原测试复现命令为：

```powershell
cargo test -p zircon_runtime --lib runtime_15_scene_fixed_light_reflection_write_fields_are_child_owner -- --nocapture
```

## 最低共享层根因

Runtime 15 的守卫验证的是“手写灯光适配器父/子文件如何拆分”，而不是长期结构不变量。
统一反射计划已经把元数据与字段访问迁入组件上的 `ZrReflect` 派生，并把普通灯光注册
收敛到 `derived_component_registration::<T>`；旧守卫因此成为对退休 owner 的反向依赖。

## 架构修复验收

- Runtime 15 守卫改为验证灯光组件元数据由 `ZrReflect` 派生持有、注册统一走 `derived_component_registration::<T>`，并继续执行当前生产文件预算。
- 上述原复现测试通过，且不再读取任何 `scene/reflect/fixed/**` 路径。
- Plugins 08 M1 的 `ecs_reflect` 既有行为测试与 `reflection_hard_cut_removes_the_manual_fixed_adapter_tree` 通过。

## 禁止临时方案

- 不恢复 `fixed/`、别名、兼容模块、空壳文件或重复注册表。
- 不删除或弱化生产文件预算与派生反射结构验收来隐藏失败。
- 不把旧状态行中的历史证据伪装成当前源码 owner。

## 修复结果与回传

- 根因：Runtime 15 guard encoded retired intermediate scene/reflect/fixed/lights owner paths instead of final derived-reflection invariants.
- 架构修复：Guard now validates derive-owned lighting metadata, generic derived registration, World reinsertion, hard-cut absence, and retained production file budgets; reflection docs mark the handwritten path retired.
- 验证：Runtime15 original reproduction 1/1 passed; scene::tests::ecs_reflect 61/61 passed; VM reverse registration 1/1 passed; handoff validator 87 artifacts with 0 errors.
- 回传：Plugins 08 M1-T2 hard-cut gate is restored and may resume independent M1 acceptance.
