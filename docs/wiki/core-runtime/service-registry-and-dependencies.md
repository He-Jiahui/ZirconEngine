---
related_code:
  - zircon_runtime/src/core/runtime/descriptors/mod.rs
  - zircon_runtime/src/core/runtime/descriptors/registry_name.rs
  - zircon_runtime/src/core/runtime/descriptors/module_order.rs
  - zircon_runtime/src/core/runtime/handle/registration/mod.rs
  - zircon_runtime/src/core/runtime/handle/resolution.rs
  - zircon_runtime/src/core/runtime/state/service_entry.rs
  - zircon_runtime/src/engine_module/mod.rs
implementation_files:
  - zircon_runtime/src/core/runtime/descriptors/module_order.rs
  - zircon_runtime/src/core/runtime/handle/registration/mod.rs
  - zircon_runtime/src/core/runtime/handle/resolution.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
tests:
  - zircon_runtime/src/core/runtime/tests/registration
  - zircon_runtime/src/core/runtime/tests/resolution
  - zircon_runtime/src/core/runtime/descriptors/module_order_tests.rs
doc_type: module-detail
status: current
---

# 服务注册表与依赖

## 服务种类

核心支持 `Driver`、`Manager`、`Plugin` 三类 registry service。

| 类型 | 典型职责 | 允许依赖 |
| --- | --- | --- |
| Driver | 平台、设备或底层后端适配 | Driver |
| Manager | 领域服务、状态管理与稳定业务入口 | Driver、Manager |
| Plugin | 可选能力、宿主插件实例 | Driver、Manager、Plugin |

依赖等级防止底层 Driver 反向依赖高层 Manager 或 Plugin。规则在冻结服务图时验证，而不是仅靠开发约定。

## 规范服务名

`RegistryName` 的格式是：

```text
Module.Namespace.ServiceKind.ServiceName
```

最后两个点划分 kind 和 service，前面的完整部分是 module namespace。例如 `Runtime.Core.Manager.WindowManager` 的 module 是 `Runtime.Core`，kind 是 `Manager`，service 是 `WindowManager`。

规范要求：module 的每个点分段非空且无首尾空白；kind 必须是 `Driver`、`Manager` 或 `Plugin`；service 非空、无点、无首尾空白。

```rust
use zircon_runtime::core::{RegistryName, ServiceKind};

let name = RegistryName::from_parts(
    "Gameplay",
    ServiceKind::Manager,
    "AbilityManager",
);
assert_eq!(name.as_str(), "Gameplay.Manager.AbilityManager");
```

来自用户/文件的字符串应用 `RegistryName::new` 获取可恢复错误；`from_parts` 面向受控常量，非法输入会断言失败。

## 描述符

`DriverDescriptor`、`ManagerDescriptor` 和 `PluginDescriptor` 都携带：

- `name: RegistryName`
- `startup_mode: StartupMode`
- `dependencies: Arc<[DependencySpec]>`
- factory（Plugin 使用 `PluginFactory`，其余使用 `ServiceFactory`）

`StartupMode::Immediate` 在所属模块激活时创建；`Lazy` 在首次解析时创建。两者的依赖都在自身 factory 执行前完成解析。

## 声明模块与服务

```rust
use std::sync::Arc;
use zircon_runtime::core::{
    CoreError, ManagerDescriptor, ModuleDescriptor,
    RegistryName, ServiceKind, StartupMode,
};
use zircon_runtime::core::runtime::ServiceObject;
use zircon_runtime::engine_module::factory;

#[derive(Debug)]
struct AbilityManager;

let service_name = RegistryName::from_parts(
    "Gameplay",
    ServiceKind::Manager,
    "AbilityManager",
);

let descriptor = ModuleDescriptor::new("Gameplay", "Gameplay runtime")
    .with_manager(ManagerDescriptor::new(
        service_name,
        StartupMode::Lazy,
        Vec::new(),
        factory(|_core| {
            Ok(Arc::new(AbilityManager) as ServiceObject)
        }),
    ));
```

真实跨模块 manager 更适合包装为 `RegisteredManagerService<dyn Trait>`，详见句柄页面。

## 注册事务

`register_module` 在修改 registry 前完成结构验证和重复检查。Module 名是 module table 的唯一 key，`RegistryName` 是 service table 的唯一 key。成功后，`ModuleEntry` 缓存：

- 完整 owner service 列表。
- 已按依赖过滤的 immediate startup 列表。
- 预先反转的 shutdown 列表。

这些缓存让激活和卸载无需每次扫描全局服务表。

## 依赖图冻结

第一次需要激活顺序时，运行时把注册描述符冻结为模块与服务图，并验证：

- 模块依赖都存在且无环。
- 服务依赖都存在且无环。
- 服务 kind 依赖方向合法。
- 跨模块服务依赖有对应 `ModuleDependencySpec`。
- 服务名的 module 部分与 owner module 一致。

模块激活 order 同时考虑 `InitLevel` 和依赖；服务 order 使用拓扑排序。shutdown order 是 activation order 的逆序。

## 解析与缓存

`CoreRuntimeInner.services` 是实例唯一权威。解析时：

1. 检查名称、kind 和 lifecycle。
2. 返回已缓存实例，或认领 initialization ownership。
3. 必要时激活 owner module。
4. 解析依赖。
5. 在 panic boundary 内执行工厂。
6. 再次核对 index/generation/owner/state 后提交实例。

如果初始化失败，状态回到可重试状态。并发线程不会各自创建第二个实例。

## 直接类型解析

```rust
let manager: Arc<AbilityManager> = runtime.resolve_manager(
    "Gameplay.Manager.AbilityManager",
)?;
```

直接解析要求注册表中的实际对象能 `Arc::downcast` 为精确具体类型。跨 crate 公共契约更推荐 manager trait handle，以避免调用方依赖实现类型。

## 常见错误

- `MissingModule` / `MissingService`：名字不存在。
- `InvalidRegistryName`：名字不规范。
- `DuplicateModule` / duplicate service 类错误：注册冲突。
- `ModuleDependencyCycle` / `ServiceDependencyCycle`：静态图有环。
- `DependencyCycle`：动态/并发解析发现循环等待。
- `InvalidServiceDependencyKind`：低层服务依赖高层种类。
- `UndeclaredCrossModuleServiceDependency`：服务跨模块依赖未在模块层声明。
- `ServiceDowncast`：请求的具体 Rust 类型与 registry 对象不一致。
- `ServiceFactoryPanicked`：工厂 panic 已隔离并转为错误。

## 实现状态

上述规则均有当前源码实现和 registration/resolution/module-order 测试。公开 API 尚不提供运行中任意替换 descriptor 的通用热更新；插件热替换需使用插件体系定义的 slot/protocol，而不是直接篡改 service registry。
