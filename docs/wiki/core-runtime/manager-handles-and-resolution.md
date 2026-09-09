---
related_code:
  - zircon_runtime/src/core/manager/mod.rs
  - zircon_runtime/src/core/manager/service.rs
  - zircon_runtime/src/core/manager/resolver.rs
  - zircon_runtime/src/core/manager/service_names.rs
  - zircon_runtime/src/core/runtime/handle/service_identity.rs
  - zircon_runtime/src/core/runtime/handle/resolution.rs
implementation_files:
  - zircon_runtime/src/core/manager/service.rs
  - zircon_runtime/src/core/manager/resolver.rs
  - zircon_runtime/src/core/runtime/handle/resolution.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
tests:
  - zircon_runtime/src/core/manager/tests.rs
  - zircon_runtime/src/core/runtime/tests/resolution
doc_type: module-detail
status: current
---

# Manager 句柄与解析

## 为什么需要句柄

直接 `resolve_manager::<ConcreteType>` 会把调用者绑定到实现类型。Manager 层提供 `ManagerServiceHandle<dyn Contract>`，把稳定身份与动态 trait 契约结合：调用方知道“这是 LevelManager”，但不知道 registry 里具体是哪一个 struct。

## 句柄内容与校验

每个 handle 内含：

- service slot `index`。
- 当前实例 `generation`。
- 规范 `RegistryName`。
- 创建它的 runtime 的 `CoreWeak` 身份。
- `T` 的零大小类型 marker。

解析时同时验证 runtime identity、service kind、index 与 generation。卸载清空实例并递增 generation，所以旧 handle 会返回 `StaleServiceHandle` 或不可用错误，而不是错误地指向重新创建的新服务。

## 注册 trait manager

```rust
use std::sync::Arc;
use zircon_runtime::core::{
    ManagerDescriptor, ModuleDescriptor, RegistryName,
    ServiceKind, StartupMode,
};
use zircon_runtime::core::manager::RegisteredManagerService;
use zircon_runtime::core::runtime::ServiceObject;
use zircon_runtime::engine_module::factory;

trait GameplayApi: Send + Sync {
    fn active_actor_count(&self) -> usize;
}

#[derive(Debug)]
struct GameplayManager;

impl GameplayApi for GameplayManager {
    fn active_actor_count(&self) -> usize { 0 }
}

let descriptor = ModuleDescriptor::new("Gameplay", "Gameplay runtime")
    .with_manager(ManagerDescriptor::new(
        RegistryName::from_parts(
            "Gameplay", ServiceKind::Manager, "GameplayManager",
        ),
        StartupMode::Lazy,
        Vec::new(),
        factory(|_core| {
            let api: Arc<dyn GameplayApi> = Arc::new(GameplayManager);
            Ok(Arc::new(RegisteredManagerService::new(api)) as ServiceObject)
        }),
    ));
```

`RegisteredManagerService<T>` 是通用 registry object 与 unsized trait object 之间的包装。它不改变 trait 的线程约束：`T` 必须是 `Send + Sync + 'static`。

## 获取与解析句柄

```rust
use zircon_runtime::core::manager::{
    manager_service_handle, ManagerResolver,
};

let core = runtime.handle();
let handle = manager_service_handle::<dyn GameplayApi>(
    &core,
    "Gameplay.Manager.GameplayManager",
)?;

let resolver = ManagerResolver::new(core);
let api = resolver.resolve(handle)?;
assert_eq!(api.active_actor_count(), 0);
```

`manager_service_handle` 只取得已注册身份；真正解析可能触发 lazy 初始化。一个 handle 是一次性的解析参数（解析方法按值接收），需要多次解析时显式 `clone()`。

## 内建领域入口

`ManagerResolver` 为常用领域提供类型化方法：

| 方法 | 契约 | 规范名称 |
| --- | --- | --- |
| `rendering_handle()` | `RenderingManager` | `GraphicsModule.Manager.RenderingManager` |
| `render_framework_handle()` | `RenderFramework` | `GraphicsModule.Manager.RenderFramework` |
| `level_handle()` | `LevelManager` | `SceneModule.Manager.LevelManager` |
| `resource_handle()` | `ResourceManager` | `AssetModule.Manager.ResourceManager` |
| `input_handle()` | `InputManager` | `InputModule.Manager.InputManager` |
| `input_actions_handle()` | `InputActionManager` | `InputModule.Manager.InputActionManager` |
| `config_handle()` | `ConfigManager` | `FoundationModule.Manager.ConfigManager` |
| `animation_handle()` | `AnimationManager` | `animation.runtime.Manager.AnimationManager` |
| `navigation_handle()` | `NavigationManager` | `navigation.runtime.Manager.NavigationManager` |
| `platform_preferences_handle()` | `PreferenceStorage` | `PlatformModule.Manager.PlatformManager` |

AI、network、physics、sound 方法受对应 contract feature 控制。对 feature-gated API 的调用必须同步配置 `Cargo.toml`。

## 所有权规则

`ManagerResolver::new` 接收强 `CoreHandle`，但内部立即降级为 `CoreWeak`，因此 resolver 不会独自阻止 runtime 释放。registry-owned manager 同样应保存 weak core。只有不被 registry 反向拥有的 app/editor host 可以按需要持有强 handle。

## 限制与实现状态

- Handle 稳定的是逻辑服务身份，不保证 Rust 对象地址在卸载/重载后连续。
- `service_name()` 可用于诊断，不应被重新手工解析为三个字符串。
- Manager handle 路径已实现并有 runtime identity、generation 和领域 resolver 测试。
- Driver/Plugin 也有核心 `ServiceHandle<T>`，但本页领域化 `ManagerServiceHandle<T>` 只适用于 Manager kind。
