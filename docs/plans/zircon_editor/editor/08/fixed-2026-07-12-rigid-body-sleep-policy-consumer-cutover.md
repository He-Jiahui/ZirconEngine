---
handoff_kind: fixed
status: fixed
created_at: 2026-07-12
summary_slug: rigid-body-sleep-policy-consumer-cutover
origin_plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
fixing_plan: docs/plans/zircon_plugins/03-physics.md
origin_child_dir: docs/plans/zircon_editor/editor/08
fixing_child_dir: docs/plans/zircon_plugins/03
related_code:
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/scene/reflect/builtin_reflection/registration.rs
  - zircon_runtime/src/scene/world/property_access/entries/physics.rs
  - zircon_runtime/src/scene/world/property_access/write/physics.rs
tests:
  - cargo test -p zircon_editor --lib --locked
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_physics_runtime --locked
resolved_at: 2026-07-12
---


# Physics 03：RigidBody SleepPolicy 硬切后旧 can_sleep 消费者未迁移

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md`
- 来源执行切片：Editor08 M1 统一测试阶段
- 修复责任计划：`docs/plans/zircon_plugins/03-physics.md`
- 交接原因：`RigidBodyComponent` 已由布尔 `can_sleep` 硬切为类型化 `PhysicsSleepPolicy`，但反射与 Scene property 消费者仍读取、写入已删除字段；该迁移属于 Physics M2-T2 的刚体契约完整化，不应由命令系统恢复旧字段或兼容别名。

## 失败现象与复现证据

2026-07-12 Windows 受管 job `e2958b598b5249c38cc3d098149675c1` 在 `D:\cargo-targets\editor08-m1-rerun2-20260712` 执行：

```text
cargo test -p zircon_editor --lib --locked --jobs 1 --message-format short
```

编译 `zircon_runtime` 时稳定出现 6 个 `E0609`：

- `scene/reflect/fixed/rigid_body_component.rs` 三处访问 `component.can_sleep`；
- `scene/world/property_access/entries/physics.rs` 一处访问 `rigid_body.can_sleep`；
- `scene/world/property_access/write/physics.rs` 两处访问/写入 `rigid_body.can_sleep`。

当前真实组件只拥有 `sleep_policy: PhysicsSleepPolicy`，默认值为 `Allow`；因此这不是 Editor08 命令代码错误，而是 Physics M2-T2 已开始硬切、下游消费者未原子迁移造成的共享编译阻塞。job 以 exit 101 结束并由 coordinator 正常释放。

## 最低共享层根因

Physics M2-T2 将 Scene 刚体 schema 改为 `PhysicsSleepPolicy::{Allow,Never}`，但反射字段合同和通用 Scene property path 仍停留在旧布尔 `can_sleep`。最低修复层是刚体 schema 的所有生产消费者与序列化/反射/property tests；不能在 `RigidBodyComponent` 重新增加旧字段。

## 架构修复验收

- 反射与 property access 使用类型化 `sleep_policy` 权威字段；字段名、编辑器 hint、读取与写入语义由 Physics 计划统一裁决。
- 若保留场景文本中的旧 `can_sleep` 数据迁移，只能在明确的版本化迁移层一次性转换，生产组件/API 不保留同名字段、getter、setter、re-export 或双写。
- 补齐 `Allow/Never` 反射读写、property visit/set、序列化 round-trip 与默认值测试。
- 先通过 Physics M2-T2 的定向测试，再复跑 `cargo test -p zircon_editor --lib --locked`，并向 Editor08 回传解除阻塞证据。

## 禁止临时方案

- 禁止给 `RigidBodyComponent` 恢复 `can_sleep: bool`，禁止添加兼容 getter/setter 或双字段同步。
- 禁止在 Editor08、inspector 或 property layer 把缺失字段静默默认成 `true`。
- 禁止删除/弱化反射和 property 测试来绕过编译失败。

## 修复结果与回传

- 根因：Physics M2-T2 hard-cut RigidBodyComponent from can_sleep to typed PhysicsSleepPolicy, while reflection and scene property consumers still accessed the removed field, producing six E0609 errors in the Editor08 compile gate.
- 架构修复：Converged the authoritative scene/sync/asset contracts on PhysicsSleepPolicy::{Allow,Never}, exposed typed sleep_policy plus CCD and mass-property fields through reflection and property paths, updated project IO and tests, and retained no production can_sleep field, alias, getter, setter, or dual-write path.
- 验证：Windows managed Physics backend-jolt library job 1d33853ff25e449d83ce7c7603942eed passed 27/27; Runtime reflection job ee73fcf057b24dc98ccae030f58ddc78 passed 1/1; direct Runtime property mutation and project round-trip tests passed 2/2; Editor compile-gate job c349b8ccfed047a0b23b0c33f9993584 succeeded and its fresh test host reported 0 failed with 3042 filtered; tracked Rust can_sleep scan, scoped rustfmt/diff hygiene, production added panic/allow scan, and plugin structure audit passed.
- 回传：Editor08 can resume its full suite: the old can_sleep E0609 blocker is removed by the typed SleepPolicy consumer hard cut, without compatibility aliases.
