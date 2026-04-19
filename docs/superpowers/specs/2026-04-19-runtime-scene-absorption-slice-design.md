---
related_code:
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/sessions/20260418-1910-runtime-absorption-boundary-cutover.md
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/module.rs
  - zircon_scene/src/lib.rs
  - zircon_scene/src/module.rs
  - zircon_scene/src/module/default_level_manager.rs
  - zircon_scene/src/module/world_driver.rs
  - zircon_scene/src/level_system.rs
implementation_files:
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/module.rs
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/module/mod.rs
  - zircon_runtime/src/scene/module/default_level_manager.rs
  - zircon_runtime/src/scene/module/world_driver.rs
  - zircon_scene/src/lib.rs
  - zircon_scene/src/module.rs
plan_sources:
  - user: continue pushing more modules into zircon_runtime while keeping the directory structure clean
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/sessions/20260418-1910-runtime-absorption-boundary-cutover.md
tests:
  - cargo test -p zircon_runtime --lib --locked --offline
  - cargo test -p zircon_scene --lib --locked --offline
  - cargo test -p zircon_editor --lib --no-run --locked --offline
  - cargo check --workspace --locked --offline
doc_type: milestone-detail
---
# Runtime Scene Absorption Slice Design

## 背景

当前 `zircon_runtime::scene` 只拥有模块注册入口，而真正的 runtime 编排对象仍然来自 `zircon_scene`：

- `zircon_runtime/src/scene/module.rs` 直接解析和构造 `DefaultLevelManager`、`WorldDriver`
- `zircon_runtime/src/scene/mod.rs` 仍然通过 `pub use zircon_scene::*;` 暴露整块 legacy surface
- `zircon_scene/src/lib.rs` 同时导出 world authority、runtime orchestration、level lifecycle 三类职责

这和计划要求不一致。计划要求 `zircon_runtime` 成为高层运行时业务的统一注册面，而 `zircon_scene` 收窄为 scene domain authority，不继续承担 runtime 根编排职责。

## 本轮批准范围

用户已确认本轮同时执行两个切面：

1. 吸收 `zircon_scene` 的 runtime orchestration 模块
2. 吸收 `LevelSystem`

因此本轮固定把以下内容物理迁入 `zircon_runtime::scene`：

- `DefaultLevelManager`
- `WorldDriver`
- `LevelSystem`
- `LevelLifecycleState`
- `LevelMetadata`
- `module/**` 目录下为上述类型服务的支撑模块

## 不在本轮范围

以下内容继续留在 `zircon_scene`，不顺手并入 `zircon_runtime`：

- `World`
- `components/**`
- `render_extract`
- `semantics`
- `serializer`
- `SceneAssetSerializer`

原因是这些仍然属于 scene domain authority 或 world data/model surface，不应在这一步与 runtime orchestration 一起搅混。

## 目标结构

本轮完成后，结构目标固定为：

```text
zircon_runtime/src/scene/
  mod.rs
  level_system.rs
  module.rs
  module/
    mod.rs
    core_error.rs
    default_level_manager.rs
    level_display_name.rs
    level_manager_facade.rs
    level_manager_lifecycle.rs
    level_manager_project_io.rs
    service_names.rs
    world_driver.rs

zircon_scene/src/
  lib.rs
  world/
  components/
  render_extract.rs
  semantics.rs
  serializer.rs
  world.rs
```

其中 `zircon_runtime/src/scene/module.rs` 只允许作为极薄 façade；任何真实实现都必须放进 `module/` 子树，不允许再次把根 wiring 文件堆成实现热点。

## 边界规则

本轮之后边界固定为：

- `zircon_runtime::scene` 负责 scene runtime service registration、level lifecycle object、level manager facade、world driver
- `zircon_scene` 负责 world model、component model、scene serialization、scene render extract
- `zircon_runtime::scene` 可以依赖 `zircon_scene::World` 与 `SceneAssetSerializer`
- `zircon_scene` 不反向依赖 `zircon_runtime`

关键约束是避免形成 `zircon_scene -> zircon_runtime -> zircon_scene` 反向结构依赖。物理移动只能发生在当前已经被 `zircon_runtime` 消费、且不被 `zircon_scene` 根逻辑反向依赖的 runtime 编排对象上。

## Root Surface 收束要求

本轮必须同时收紧两个 root surface：

- `zircon_runtime/src/scene/mod.rs`
  - 不再用 `pub use zircon_scene::*;` 把整个 legacy crate 根表面原样透传
  - 改成显式导出 runtime-owned items，再按需导出仍留在 `zircon_scene` 的 domain items
- `zircon_scene/src/lib.rs`
  - 不再导出已经迁入 runtime 的 `DefaultLevelManager`、`WorldDriver`、`LevelSystem`
  - crate 根只保留 scene domain authority 所需的权威导出

目标是让两个 crate 根都回到“短 wiring + 清晰领域归属”的状态。

## 迁移顺序

执行顺序固定为：

1. 先在 `zircon_runtime::scene` 建立 `level_system.rs` 和 `module/` 子树
2. 迁移 `LevelSystem` 及相关 metadata/lifecycle 类型
3. 迁移 `DefaultLevelManager` 与 `WorldDriver`
4. 改写 `zircon_runtime::scene` 根导出，移除整块 `pub use zircon_scene::*`
5. 改写 `zircon_scene::lib` 根导出，删除被 runtime 吸收的 orchestration surface
6. 修复所有消费者导入路径到新归属

这个顺序的目的是先稳定 runtime 物理落点，再删旧根导出，避免在迁移中间态制造额外的命名混乱。

## 验收标准

- `zircon_runtime::scene` 拥有 `LevelSystem`、`DefaultLevelManager`、`WorldDriver` 的物理定义
- `zircon_runtime/src/scene/mod.rs` 不再包含 `pub use zircon_scene::*;`
- `zircon_scene/src/lib.rs` 不再导出 `LevelSystem`、`DefaultLevelManager`、`WorldDriver`
- `zircon_runtime` 与 `zircon_editor` 侧 scene 编排调用全部改走 `zircon_runtime::scene::*`
- `zircon_scene` 仍能独立提供 `World`、`SceneAssetSerializer`、render extract 相关 API
- `cargo` 验证若失败，失败原因必须是已知外部环境问题或本轮无关回归，而不是 namespace/ownership 迁移错误

## 默认假设

- `LevelSystem` 虽然包裹 `World`，但其职责属于 runtime lifecycle object，因此与 `World` 的物理位置允许分离
- `SceneAssetSerializer` 先继续留在 `zircon_scene`，因为它更接近 scene domain persistence，而不是 runtime manager orchestration
- 如果 `zircon_editor` 仍保留对 `zircon_scene::LevelSystem` 旧路径的依赖，本轮直接切到新路径，不保留兼容 re-export
