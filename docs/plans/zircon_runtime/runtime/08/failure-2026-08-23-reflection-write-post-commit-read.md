---
handoff_kind: failure
status: open
created_at: 2026-08-23
summary_slug: reflection-write-post-commit-read
origin_plan: docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md
fixing_plan: docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md
origin_child_dir: docs/plans/zircon_editor/editor/03
fixing_child_dir: docs/plans/zircon_runtime/runtime/08
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/scene/reflect/world_reflection.rs
  - zircon_runtime/src/scene/reflect/reflect_component.rs
  - zircon_runtime/src/scene/reflect/reflect_resource.rs
  - zircon_runtime/src/scene/tests/ecs_reflect/foundation.rs
  - zircon_editor/src/core/editing/command.rs
  - zircon_editor/src/tests/editing/reflected_command.rs
tests:
  - .\\.codex\\skills\\zircon-dev\\scripts\\validate-matrix.ps1 -Package zircon_runtime -LibTests
  - .\\.codex\\skills\\zircon-dev\\scripts\\validate-matrix.ps1 -Package zircon_editor -LibTests
---

# Runtime08: reflection write must not read after publication

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/03-command-transaction-and-undo.md`
- 来源执行切片：scene-write callback outcome and undo/redo compensation
- 修复责任计划：`docs/plans/zircon_runtime/runtime/08-ecs-kernel-data-alignment.md`
- 交接原因：`WorldReflection::reflect_write` owns the shared mutation boundary used by editor, plugin, and remote consumers. Editor history must consume its result; it cannot infer whether an opaque reflection adapter published a write.

## 失败现象与复现证据

`WorldReflection::reflect_write` invokes an adapter `write_field`, then invokes the independently registered `read_field` to construct `ReflectWriteResponse`. A component or resource adapter can publish the write successfully and reject that second read. The method then returns `Err` even though the world has changed.

`SetReflectedSceneFieldCommand` correctly treats a callback `Err` as `Unchanged` only when the runtime mutation contract is atomic. The current post-commit read violates that prerequisite, so history can skip compensation after a real world change. `ReflectComponent` exposes separate read/write function pointers; no contract currently guarantees that the post-write read cannot fail. This is a source-traced P1; no Cargo result is claimed.

Reference evidence supports a setter as a one-way mutation boundary: `dev/godot/core/object/object.cpp` returns after an accepted script, extension, or built-in setter without an implicit readback. `dev/bevy/crates/bevy_reflect/src/reflect.rs` explicitly warns that fallible reflective application may leave partial mutation and recommends a cloned staging value for callers that require rollback. Zircon's editor transaction contract needs the stricter staged/atomic behavior at the runtime adapter boundary.

## 最低共享层根因

The reflection facade couples state publication to a second, arbitrary read operation. It therefore reports the read failure as if the mutation itself had not completed. This mixes two independently fallible operations into one transactional result and forces upper layers to guess state that only Runtime08 owns.

## 架构修复验收

- Define the component and resource write-adapter contract: validation happens before publication, and `Err` leaves the live world unchanged. Existing derived adapters already stage a cloned component before insertion; custom adapters must conform to the same contract.
- `WorldReflection::reflect_write` validates request metadata before dispatch, invokes exactly one write adapter, and never performs a post-publication `read_field` to construct its response.
- Define `ReflectWriteResponse.field` as the accepted request field/value, not an implicit canonical readback. Consumers that need an observed value must issue an explicit `reflect_read` in a separate operation.
- Add a Runtime08 regression adapter whose write changes the world while its read always fails. The write must return `Ok`, expose `changed`, preserve the submitted response field, and not invoke the failing read.
- Add the Editor03 reflected-field apply/revert history regression through the same adapter; a runtime write result must be recoverable by the command transaction without an `Unchanged` false negative.
- Run the two declared managed package test batches after the current shared artifact governance issue is cleared. Record actual command/job evidence before marking this handoff fixed.

## 禁止临时方案

- Do not mark every reflected command callback error as `Applied`; a real validation or adapter write failure must remain `Unchanged` under the atomic adapter contract.
- Do not add editor-only read suppression, type-name branches, compatibility shims, or duplicate reflection state.
- Do not retain an implicit write-readback path beside the new contract.
- Do not report the removed read call as a measured performance gain without a managed benchmark; this handoff is a correctness repair.

## 产出记录与时间

| 日期 | 项目 | 状态 | 证据 |
| --- | --- | --- | --- |
| 2026-08-23 | 反射写入后读失败根因 | `open / source_traced / runtime08_owner` | `world_reflection.rs` 的 write-then-read 调用序列与 `ReflectComponent` 独立读写适配器已复核；Editor03 不建立调用点旁路。受管 Cargo 尚未执行。 |
| 2026-08-24 | Runtime08 write acknowledgement hard cut | `source_updated_static_green_cargo_pending` | `reflect_write` 统一预检 component/resource schema field 与 editable，再只回显已接受 request field，移除两分支写后 read；新增 published-write/read-failure 和 unknown-field/no-dispatch 回归及静态守卫。新增测试与 DTO 文件 `rustfmt --check`、scoped `git diff --check`、RED->GREEN 源码守卫通过；未运行受管 Runtime08/Editor03 Cargo 或性能基准，failure 保持 open。 |

## 修复结果与回传

Open state: `source_updated_static_green_cargo_pending`; the repair is not accepted and
no managed package pass is claimed. `WorldReflection::reflect_write` now constructs
`ReflectWriteResponse.field` from the accepted request after uniform registry
field/editable preflight and before the one adapter write, without a post-publication
component or resource readback. Runtime08 regressions cover a component adapter that
publishes a node rename while every read method fails, plus unknown-field rejection
before adapter dispatch. Editor03 upward history coverage and the declared managed
package gates remain required before a fixed return.
