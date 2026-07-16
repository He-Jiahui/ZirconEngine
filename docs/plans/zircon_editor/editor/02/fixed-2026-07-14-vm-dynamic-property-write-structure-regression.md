---
handoff_kind: fixed
status: fixed
created_at: 2026-07-14
summary_slug: vm-dynamic-property-write-structure-regression
origin_plan: docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
fixing_plan: docs/plans/zircon_plugins/08-zr-vm.md
origin_child_dir: docs/plans/zircon_editor/editor/02
fixing_child_dir: docs/plans/zircon_plugins/08
related_code:
  - zircon_runtime/src/scene/world/dynamic_components.rs
  - zircon_runtime/src/scene/tests/ecs_dynamic_components_structure.rs
  - zircon_runtime/src/scene/reflect/dynamic_json.rs
tests:
  - cargo test -p zircon_runtime --lib scene::tests::ecs_dynamic_components_structure::dynamic_component_property_writes_split_and_insert_only_at_map_boundaries --locked -- --exact --nocapture
  - cargo test -p zircon_runtime --lib scene:: --locked
resolved_at: 2026-07-14
---


# Plugins08：VM 动态属性写入结构回归复发

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md`
- 来源执行切片：Editor02 M1 fresh 默认特性 runtime scene 验收门禁
- 修复责任计划：`docs/plans/zircon_plugins/08-zr-vm.md`
- 交接原因：失败由 Plugins08 最新 VM reflection/dense-call-site 提交重新引入 dynamic component 写入 closure 链；Editor02 不拥有 VM-backed reflection commit 路径。
- 受管证据：Windows job `acfc6c19219441e498a6af33ce4b5e7a`，日志 `E:\ZirconBuilds\editor02-m1-runtime-scene-final-20260714.log`。

## 失败现象与复现证据

`dynamic_component_property_writes_split_and_insert_only_at_map_boundaries` 再次失败。当前 `set_dynamic_component_property` 的 VM 分支在候选写入后使用 `.get_mut(...).and_then(...).map(...).ok_or_else(...)`；结构合同要求显式成功/错误分支，并只在 map insertion 边界创建 owned component/property strings。`git log` 显示该生产路径最后由 Plugins08 提交 `8b273faa`（`feat(zircon_plugins): unify VM reflection registry and dense call sites`）更新。

此前 `dynamic-reflection-json-projection-regression` lifecycle 已 fixed 回传，且其记录明确把同名结构失败列入验收。本次是 fixed 之后的新提交复发，必须使用新的 lifecycle，不能篡改旧 fixed 结果。

## 最低共享层根因

Plugins08 为 VM-backed declared-value 写入增加事务候选替换时，把 shared dynamic-component map 更新重新写成 closure 链，导致 Runtime15 的直接控制流和分配边界合同失效。最低修复点仍是 VM dynamic property commit 分支，不是 Editor02 或测试过滤。

## 架构修复验收

- VM 分支以显式 `let Some(...) else`/直接分支完成 entity map、component 与 candidate commit，保留 typed `ReflectError`/`SceneError` 上下文。
- owned component/property string 只在实际 map insertion 或错误 payload 需要拥有值时分配；成功更新现有 component 不重复构造 key。
- exact 结构测试、Plugins08 dynamic-components 合同和 fresh Editor02 默认 scene 门禁通过。

## 禁止临时方案

- 禁止删除 `.ok_or_else` 文本但保留等价隐式 closure 链，或在测试中排除 VM 分支。
- 禁止恢复旧 reflection facade、双轨 dynamic JSON owner 或 test-only bypass。
- 禁止修改旧 fixed artifact 来掩盖本次复发。

## 修复结果与回传

- 根因：Plugins08 M1 replaced the shared VM-backed dynamic component commit with a get_mut/and_then/map/ok_or_else closure chain, violating the Runtime15 direct control-flow and allocation-boundary contract.
- 架构修复：Restored explicit let-Some-else lookup for the entity component map and component slot, retained schema validation before commit, preserved typed ReflectError::MissingComponent paths, and commits only through direct candidate replacement.
- 验证：Coordinator jobs `eff1cea101b44d0eb0944950352d14fe` exact structure test 1/1 PASS；`b62ac55d650b4a2ebf9b630598637361` full `ecs_dynamic_components_structure` suite 13/13 PASS。Fresh broad runtime jobs `fbc4d6eb1fc34af681e523f33cc0de36` 与 `3bb7d91fe54d4bc68616e5abf09801f4` 均已越过本 owner，当前只被其他活跃会话的 Render18 `advanced_lighting` 字段迁移阻断，因此本修复路径不再残留失败。
- 回传：Plugins08 returned the lowest-layer VM dynamic property write fix to Editor02; the source-structure regression is closed with exact and complete owner-suite evidence, while unrelated broad-workspace render churn remains outside this lifecycle.
