---
related_code:
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/lifecycle.rs
  - zircon_runtime/src/core/runtime/descriptors/module_descriptor.rs
  - zircon_runtime/src/core/runtime/descriptors/driver_descriptor.rs
  - zircon_runtime/src/core/runtime/descriptors/manager_descriptor.rs
  - zircon_runtime/src/core/runtime/descriptors/plugin_descriptor.rs
implementation_files:
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/descriptors
  - zircon_runtime/src/core/runtime/lifecycle.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/engine-architecture/core-runtime-service-registry.md
tests:
  - zircon_runtime/src/core/runtime/tests/activation
  - zircon_runtime/src/core/runtime/tests/registration
doc_type: module-detail
---

# 模块、服务与生命周期

## 模块模型

一个 `ModuleDescriptor` 是运行时对某个功能域的可验证声明。它包含名字、描述、`InitLevel`、模块依赖、`ModuleLifecycle` 以及 Driver/Manager/Plugin 的 descriptor 集合。声明不是实例：它允许 core 在启动副作用前检查重复名、依赖缺失、依赖环和服务种类规则。

```rust
use std::sync::Arc;
use zircon_runtime::core::{InitLevel, ModuleDescriptor, NoopModuleLifecycle};

let module = ModuleDescriptor::new("Example", "Example runtime module")
    .with_init_level(InitLevel::Post)
    .with_lifecycle(Arc::new(NoopModuleLifecycle));
```

实际模块还会调用 `with_module_dependency`、`with_driver`、`with_manager` 或 `with_plugin`。descriptor 中服务工厂只能获取规定的 `ModuleContext`/`PluginContext`，不能假设某个未声明的模块已经可用。

## 初始化级别

| `InitLevel` | 适合放置的能力 |
| --- | --- |
| `Kernel` | 最低层内核和必须先于一切产品能力的服务 |
| `Services` | foundation、platform、输入、任务、配置等共享服务 |
| `Scene` | 资源、World、文本、图形、UI 等运行时域 |
| `Editor` | 仅编辑器宿主使用的作者态模块 |
| `Post` | 可选插件、后接扩展和不应抢占基础层的功能 |

同一 level 仍由显式 `ModuleDependencySpec` 拓扑排序。不要依赖 `Vec` 插入顺序或名称的字典序表达运行语义。

## ModuleLifecycle

`ModuleLifecycle` 是 `Send + Sync` trait，具有四个主要钩子：

```rust
use zircon_runtime::core::{CoreResult, ModuleContext, ModuleLifecycle};

struct ExampleLifecycle;

impl ModuleLifecycle for ExampleLifecycle {
    fn build(&self, context: &ModuleContext) -> CoreResult<()> {
        // 注册模块需要的 runtime-owned 行为。
        let _ = context;
        Ok(())
    }

    fn ready(&self, _context: &ModuleContext) -> CoreResult<bool> {
        // false 表示仍等待异步依赖；core 在 ready timeout 内重试。
        Ok(true)
    }

    fn finish(&self, _context: &ModuleContext) -> CoreResult<()> {
        Ok(())
    }

    fn cleanup(&self, _context: &ModuleContext) -> CoreResult<()> {
        Ok(())
    }
}
```

- `build`：建立模块本身的注册和基础服务，不把外部 UI、GPU 或异步资源当成已就绪。
- `ready`：返回 `false` 时继续等待；错误或超时终止激活。
- `finish`：所有前置条件满足后做最终连线，服务开放调用 admission。
- `cleanup`/`cleanup_until`：释放模块资源；带 deadline 的版本用于产品关闭预算。

## 激活与卸载

```rust
use std::time::Duration;
use zircon_runtime::core::CoreRuntime;

let runtime = CoreRuntime::try_new()?;
runtime.register_module(module)?;
runtime.activate_registered_modules_with_ready_timeout(Duration::from_secs(10))?;
// ... product work ...
runtime.shutdown_registered_modules_with_drain_timeout(Duration::from_secs(5))?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

激活会计算目标模块的依赖 closure，先执行 `build`，创建 `StartupMode::Immediate` 服务，轮询 `ready`，然后 `finish`。lazy service 则在首次解析时创建，但仍受 owner module lifecycle 保护。

卸载按成功激活顺序的逆序执行。core 会先拒绝新服务调用、等待 in-flight `ServiceCallGuard` 排空，再清理服务、执行 lifecycle cleanup 并递增服务 generation。旧 handle 之后不能再次取得新实例。

## 状态与失败语义

`LifecycleState` 依次可见为 `Registered`、`Initializing`、`Running`、`Stopping`、`Unloaded`。核心为同一模块的并发命令做协调：重复激活共享结果，重入或循环解析返回 `CoreError`，不能依赖锁死或 `panic`。常见失败包括：

- 未注册模块、缺失/循环模块依赖；
- descriptor 服务种类的非法依赖；
- `build`/`ready`/`finish`/`cleanup` 错误或超时；
- service factory panic（会被转换为结构化错误）；
- 外部依赖仍在运行导致无法卸载。

## 扩展规则

新模块应把 trait 和 DTO 放在 `core::framework`，把实例化和业务实现放在所属 runtime 域，把稳定访问入口放在 `core::manager`。不要为快速接入新增第二套 global registry 或在 editor/app 直接持有具体 manager 实现。
