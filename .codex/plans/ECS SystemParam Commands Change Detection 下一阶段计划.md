# ECS SystemParam / Commands / Change Detection 下一阶段计划

## Summary

目标是把当前“可运行基础切片”推进到接近 Bevy ECS 日常可用形态，但仍保持 Zircon 的 runtime world authority，不引入外部 `bevy_ecs` 依赖。

当前基线已有：`SystemState<P>`、`QueryState`、`Res/ResMut`、`Local`、`Commands`、`Added<T>`、`Changed<T>`、`ChangeTick`。下一阶段补齐缺口：更多系统参数、命令构建器、移除检测、`Ref/Mut` 变更包装、系统调度集成和 `ApplyDeferred`。

参考来源：
- Bevy：`dev/bevy/crates/bevy_ecs/src/system/system_param.rs`
- Bevy：`dev/bevy/crates/bevy_ecs/src/system/commands/mod.rs`
- Bevy：`dev/bevy/crates/bevy_ecs/src/change_detection/params.rs`
- Bevy：`dev/bevy/crates/bevy_ecs/src/schedule/auto_insert_apply_deferred.rs`
- Zircon 当前基线：`zircon_runtime::scene::ecs`

## Key Changes

- **SystemParam 扩展**
  - 新增 `Option<ResParam<T>>` / `Option<ResMutParam<T>>` 语义，缺资源时返回 `None`，普通 `ResParam<T>` 继续保持缺资源错误。
  - 新增 `ParamSet<(P0, P1, ...)>`，允许同一 system 内声明潜在冲突参数，但运行时只能通过 `p0()` / `p1()` 分段借用。
  - 新增 `EventReaderParam<T>` / `EventWriterParam<T>`，基于现有 `EventStore`，读 current queue，写 next queue。
  - 保持 tuple param 支持上限先到 8 项；不做无限泛型或宏复杂化。

- **Commands 完整化**
  - 增加 `Command` trait，并让 `CommandQueue` 存储 typed `Command`，保留 closure command 作为 `FnCommand`。
  - 增加 `EntityCommands`：`commands.entity(entity).insert(...)`、`remove::<T>()`、`despawn()`。
  - 增加 `spawn_empty()`、`spawn(bundle).id()`、`entity_or_spawn(entity)`。
  - `apply_deferred()` 保持唯一同步点；schedule 后续自动插入 `ApplyDeferred`，手动 `World::apply_deferred()` 继续可用。

- **Change Detection 完整化**
  - 新增 `Ref<T>` / `Mut<T>` 查询项，提供 `is_added()`、`is_changed()`、`last_changed()`。
  - `&mut T` 查询继续可用，但计划内部改走 `Mut<T>` 的 tick 写入路径。
  - 新增 `RemovedComponents<T>` 资源式 reader；`World::remove<T>`、`despawn`、`Commands::remove/despawn` 都记录 removed component event。
  - 保留 Zircon 当前 `u64 ChangeTick`，不复刻 Bevy 的 wrapped `u32` tick 维护；文档记录为有意差异。

- **Schedule 集成**
  - 让 `SceneSystemDescriptor` 支持 runtime ECS system entry，而不仅是当前 internal enum built-ins。
  - 增加 `ApplyDeferred` internal system，并在同一 stage 内检测到前序 system 使用 `CommandsParam` 时插入同步点。
  - 第一阶段只支持串行执行和确定性 order；并行调度留给后续 borrow graph milestone。

## Test Plan

- `scene::tests::ecs_systems`
  - `ParamSet` 允许声明冲突 query/resource，并验证分段访问不 panic。
  - `Option<ResParam<T>>` 缺失返回 `None`，普通 `ResParam<T>` 缺失仍报错。
  - `EventReader/EventWriter` 同帧写入 next queue，下一次 event update 后可读。
  - `EntityCommands` 在 `apply_deferred()` 前不可见，之后按排队顺序生效。

- `scene::tests::ecs_change_detection`
  - `Ref<T>` / `Mut<T>` 对 added/changed 判断正确。
  - `Changed<T>` 包含 newly added component，保持 Bevy 对齐。
  - `RemovedComponents<T>` 能观察 direct remove、despawn、deferred remove、deferred despawn。
  - 同一个 system 修改组件后，自己的下一次 run 不误报旧变化；其他旧窗口 system 可观察变化。

- `scene::tests::ecs_schedule`
  - 使用 `CommandsParam` 的 system 后自动执行 `ApplyDeferred`，后续 system 能看到 command 结果。
  - 未使用 deferred commands 的 system 不插入多余 sync。
  - stage order、existing render extract dirty flush 不回退。

- 验证命令：
  - `cargo test -p zircon_runtime --lib scene::tests::ecs_systems --locked --message-format short`
  - `cargo test -p zircon_runtime --lib scene::tests::ecs_query --locked --message-format short`
  - `cargo test -p zircon_runtime --lib scene::tests::ecs_change_detection --locked --message-format short`
  - `cargo test -p zircon_runtime --lib scene::tests::ecs_schedule --locked --message-format short`
  - `cargo test -p zircon_runtime --lib scene::tests --locked --message-format short`

## Assumptions

- 这一阶段只改 `zircon_runtime::scene::ecs` 和 `World` 相关 runtime 层，不触碰 editor authoring state。
- 不引入 `bevy_ecs` 依赖；Bevy 只作为语义和测试参照。
- 不实现并行调度、archetype-level 性能优化、proc macro derive、observer/lifecycle hooks；这些放到后续 milestone。
- 文档更新落在 `docs/zircon_runtime/scene/ecs.md` 和 `tests/acceptance/ecs-to-render-chain.md`。
