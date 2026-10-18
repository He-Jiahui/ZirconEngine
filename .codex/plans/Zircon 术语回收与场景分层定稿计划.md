# Zircon 术语回收与场景分层定稿计划

## Summary

- 把 `server` 全部回收为你路线图里的 `manager` 语义：`zircon_server` 重命名为 `zircon_manager`，所有稳定 façade trait 统一改成 `*Manager`。
- 把 `app` 全部回收为你路线图里的 `entry` 语义：`zircon_app` 重命名为 `zircon_entry`，所有公共入口类型统一改成 `Entry*`。
- 把场景层正式定稿为 `LevelManager -> LevelSystem -> World`：`World` 只是 ECS 数据容器，不再被当作 `WorldSystem`；`LevelSystem` 才是系统实例层，子系统挂在它下面。
- 先做一个 Phase 0 编译门槛恢复：当前 `zircon_editor` 的 Slint 构建错误必须先清掉，否则后续 rename 和边界收口没有稳定验收基线。

## Implementation Changes

### 1. Manager Façade 层

- 工作区 crate 重命名为 `zircon_manager`，根 `Cargo.toml`、各 `zircon_*` 依赖和 `use` 路径统一替换，不保留长期 `zircon_server` 别名。
- 公共稳定接口定稿为：
  - `RenderingManager`
  - `LevelManager`
  - `AssetManager`
  - `ResourceManager`
  - `InputManager`
  - `ConfigManager`
  - `EventManager`
  - `WorldHandle`
  - `AssetHandle`
  - `HostHandle`
  - `PluginSlotId`
  - `CapabilitySet`
- 原 `SceneServer` 不保留，直接改为 `LevelManager`，因为你的路线图里“运行中的场景实例”归 `LevelManager/LevelSystem` 管，不归抽象 `SceneServer`。
- `server` 术语从公共 API、文档、registry name 中彻底移除；字符串命名统一回到 `ModuleName.Manager.ServiceName` 语义。
- 为了避免 trait 名和 concrete type 冲突，`zircon_manager` 只定义 trait、handle 和 resolver；具体实现类型在各子系统 crate 里统一采用 `Default*Manager`、`Wgpu*Manager`、`Project*Manager` 这类名字，不再与 façade trait 同名。
- `zircon_manager` 增加 façade resolver/holder 机制，替代当前 `core.resolve_manager::<ConcreteType>(...)` 用法。editor/runtime/plugin 只能通过 `zircon_manager` 暴露的 resolver helper 或 holder type 获取 `Arc<dyn *Manager>`，禁止依赖 `zircon_graphics::GraphicsManager`、`zircon_scene::WorldManager` 之类内部实现类型。

### 2. Entry 层

- 工作区 crate 重命名为 `zircon_entry`，不再把它定义成独立于 `Entry` 的额外架构层。
- 公共入口类型定稿为：
  - `EntryProfile::{Editor, Runtime, Headless}`
  - `EntryConfig`
  - `EntryRunner`
  - `BuiltinEntryModuleSet`
- `zircon_entry` 只负责：
  - 选择 profile
  - 注册/激活模块
  - 启动 editor/runtime/headless shell
  - 接通平台事件循环
- `zircon_entry` 不再直接解析或持有子系统 concrete manager 类型；它只依赖 `zircon_manager` 的 façade resolver 和 shell 公开接口。
- 当前 [zircon_app/src/lib.rs](/E:/Git/ZirconEngine/zircon_app/src/lib.rs) 里的 `AppMode`、`bootstrap_core()`、runtime preview 组装逻辑，迁移后都收敛到 `EntryProfile` 和 `EntryRunner` 命名下。

### 3. Scene 层

- `zircon_scene` 公共模型定稿为：
  - `World`
  - `LevelSystem`
  - `LevelManager`
  - `EntityId`
  - `Schedule`
  - `SystemStage`
  - `Hierarchy`
  - `LocalTransform`
  - `WorldTransform`
  - `RenderExtractPacket`
  - `SceneAssetSerializer`
- `World` 的地位固定为 ECS 存储与调度容器，只承载：
  - entities/components
  - hierarchy/transform
  - schedule/system stage
  - render extract 输入
  - scene asset 序列化数据映射
- `LevelSystem` 是 manager 创建出来的系统实例，拥有：
  - 一个 `World`
  - 该 world 的 `WorldHandle`
  - 关卡/场景资源元数据
  - 生命周期状态
  - 子系统注册表
- `WorldSession` 从公共 API 中移除，职责并入 `LevelSystem`。
- 现有 `WorldManager` 语义改成 `LevelManager` façade；`zircon_scene` 内部 concrete 实现命名改成 `DefaultLevelManager`。
- `LevelSystem` 下允许托管子系统；首版先把这个结构定出来，不要求一次把全部音频/导航/UI 子系统做完。子系统生命周期严格跟随 `LevelSystem`。
- `WorldSystem` 不作为首版单独公共类型引入。你的“系统实例层”由 `LevelSystem` 承担；`World` 保持纯数据容器，避免把 ECS storage 和系统生命周期再次揉在一起。
- `SceneAssetSerializer` 负责 `SceneAsset <-> World` 转换；运行中的 level 由 `LevelSystem` 托管，序列化边界仍然落在 `World`。

### 4. Naming 与迁移规则

- 不保留长期兼容别名。`*Server`、`App*`、`WorldSession` 都应在同一轮重构中从公共表面移除。
- registry name 同步改名：
  - `ServerModule` -> `ManagerModule`
  - `SceneModule.Manager.WorldManager` -> `SceneModule.Manager.LevelManager`
  - `AppMode` -> `EntryProfile`
- 文档和计划同步改名，尤其是 `.codex/plans/全系统重构方案.md`、`.cursor/plans/基本路线图.md` 对应的落地文档，不能继续混用旧术语。
- 依赖边界固定为：
  - editor/runtime/plugin 只依赖 `zircon_manager` 和必要的公共系统实例类型，例如 `zircon_scene::LevelSystem`
  - 不允许 editor/runtime/plugin 直接依赖任何子系统 concrete manager 类型
  - `zircon_entry` 只做 entry，不直接变成业务 orchestrator

## Test Plan

- Phase 0 先恢复 `cargo check --workspace`，清掉当前 `zircon_editor` Slint 构建错误，作为后续每步重构的统一门槛。
- crate rename 后验证工作区能完整解析依赖，`Cargo.toml` 成员、crate 名和 `use` 路径无残留 `zircon_server`/`zircon_app`。
- façade 边界测试：
  - `zircon_manager` 的每个 resolver 能返回正确的 `Arc<dyn *Manager>`
  - editor/runtime/plugin 侧不存在对 concrete manager 类型的直接解析与导入
  - Config/Event/Asset/Resource/Level façade 都能完成基本 roundtrip
- scene 语义测试：
  - `LevelManager` 能创建、加载、保存 `LevelSystem`
  - `LevelSystem` 持有 `World`，`World` 仍是 ECS authority
  - `LevelSystem` 销毁时其子系统一并销毁
  - `RenderExtractPacket` 从 `LevelSystem.world()` 提取结果，不直接依赖 editor 状态
- 命名一致性测试：
  - registry name、模块常量、公开 trait/type、文档标题均不再出现 `Server`/`AppMode`/`WorldSession`
  - `SceneModule.Manager.LevelManager` 等字符串命名与核心层解析规则一致

## Assumptions

- `LevelSystem` 是首版唯一显式系统实例层；`WorldSystem` 不单独落类型，避免和 `World` 形成重复抽象。
- `WorldHandle` 继续表示 `LevelSystem` 所托管的 `World`，首版不额外引入 `LevelHandle`。
- 外部插件方向不变，仍为 VM-only；`HostHandle`、`PluginSlotId`、`CapabilitySet` 继续保留在 façade 层。
- 子系统托管先把结构与生命周期定稿，首版不要求一次实现所有 `AudioSubsystem/NavigationSubsystem/ClockSubsystem` 的完整业务细节。
