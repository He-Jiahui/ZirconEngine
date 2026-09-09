---
related_code:
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/lifecycle.rs
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/handle/activation/startup.rs
  - zircon_runtime/src/core/runtime/handle/activation/blocked_unload.rs
  - zircon_runtime/src/core/runtime/state/core_runtime_state.rs
  - zircon_runtime/src/core/runtime/module_lifecycle_observer.rs
implementation_files:
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/state/core_runtime_state.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/plans/zircon_runtime/runtime/02-core-spine-and-root-surface.md
  - docs/plans/zircon_runtime/frameworks/02-module-kernel-and-lifecycle-unification.md
tests:
  - zircon_runtime/src/core/runtime/tests/activation
  - zircon_runtime/src/core/runtime/tests/registration
  - zircon_runtime/src/tests/runtime_absorption/service_registry_lifecycle.rs
doc_type: module-detail
status: current
---

# 运行时与模块生命周期

## `CoreRuntime` 构造

| API | 用途 | 错误/限制 |
| --- | --- | --- |
| `CoreRuntime::new()` | 默认任务图、系统 monotonic 时钟、默认随机服务 | 任务图初始化失败时 panic，产品宿主更适合 fallible API |
| `try_new()` | 默认配置的可失败构造 | 返回 `EngineTaskGraphInitError` |
| `try_with_task_graph_options(options)` | 显式线程/worker budget | 多 runtime 进程必须显式预算，不能假定全局池 |
| `with_random_seed(seed)` | 确定性 master seed | 空随机流 registry |
| `with_random_service_state(state)` | 从 seed authority 状态恢复 | 不恢复已注册流进度 |
| `with_random_service_checkpoint(checkpoint)` | 恢复 seed 与流进度 | checkpoint 非法时返回 `RandomServiceError` |
| `with_clock_source(source)` | 注入可控 monotonic source | 只影响外层 frame delta，不替代 deadline/profiling/file watcher 时钟 |

`CoreRuntime` 内部持有一个已存在的 `CoreHandle`，其 clone 不会新建 runtime。`handle()` 克隆强引用；`weak()` 返回不会延长 runtime 生命周期的 `CoreWeak`。

## Module 描述符

`ModuleDescriptor` 包含名称、说明、`InitLevel`、模块依赖、生命周期对象和三类服务描述符。默认 `InitLevel` 是 `Post`，默认 lifecycle 是 `NoopModuleLifecycle`。

初始化层级按以下顺序参与模块排序：

```text
Kernel -> Services -> Scene -> Editor -> Post
```

显式模块依赖仍优先保证 dependency-first；层级用于没有冲突的全局排序和越层规则验证。

## 生命周期回调

自定义模块 lifecycle 实现 `ModuleLifecycle`：

```rust
use std::sync::Arc;
use zircon_runtime::core::{CoreResult, ModuleContext, ModuleDescriptor, ModuleLifecycle};

#[derive(Debug)]
struct GameplayLifecycle;

impl ModuleLifecycle for GameplayLifecycle {
    fn build(&self, context: &ModuleContext) -> CoreResult<()> {
        let _core = context.core.upgrade();
        Ok(())
    }

    fn ready(&self, _context: &ModuleContext) -> CoreResult<bool> {
        Ok(true)
    }

    fn finish(&self, _context: &ModuleContext) -> CoreResult<()> {
        Ok(())
    }

    fn cleanup(&self, _context: &ModuleContext) -> CoreResult<()> {
        Ok(())
    }
}

let descriptor = ModuleDescriptor::new("Gameplay", "Gameplay services")
    .with_lifecycle(Arc::new(GameplayLifecycle));
```

`cleanup_until` 默认转调 `cleanup`；需要 deadline-aware 关闭的模块应覆盖它。`ready` 可暂时返回 `false`，宿主使用 `activate_module_with_ready_timeout` 或批量 timeout API 为等待设置上限。

## 生命周期状态

`Registered -> Initializing -> Running -> Stopping -> Unloaded` 是规范状态集合。重新激活允许 `Unloaded` 回到准备态并重新构造服务，但不会让旧句柄重新有效。

并发保障：

- 同一模块同一命令由一个线程执行，等待者接收相同结果。
- 同线程在 lifecycle 回调中重入同一模块命令会被拒绝。
- 生命周期回调不在 module/service registry lock 内执行。
- 服务 factory 只有一个 initialization owner；其他解析线程等待状态改变。

## 激活 API

- `activate_module(name)`：激活模块及其完整依赖闭包。
- `activate_module_with_ready_timeout(name, timeout)`：带 ready deadline。
- `activate_registered_modules()`：按冻结图激活所有已注册模块。
- `activate_registered_modules_with_ready_timeout(timeout)`：批量带 timeout。

一旦依赖图被冻结，继续注册会破坏已建立的组合语义，因此应用应先完成所有注册，再开始激活。

## 卸载 API

- `deactivate_module(name)`：使用默认 drain timeout。
- `deactivate_module_with_drain_timeout(name, timeout)`：关闭 admission，并等待 in-flight 调用。
- `shutdown_registered_modules_with_drain_timeout(timeout)`：在一个总预算内按激活逆序关闭全部模块。

若另一个 active 模块或服务仍依赖目标，卸载会被阻止。失败必须保持事务性，不能留下“前几个服务已经清掉，后续服务仍在”的半卸载模块。

## Runtime lifecycle observer

上层运行时域可以安装一个 `RuntimeModuleLifecycleObserver`，在 core 模块成功激活后接收通知，并在 core 开始卸载前执行检查。observer 可用 `RuntimeModuleLifecycleBlock` 阻止仍不安全的卸载。这是跨层生命周期挂钩，不是第二套模块生命周期。

## 限制与状态

- `CoreRuntime` 没有自动 Drop 即完整停机的文档保证；宿主应调用显式 shutdown。
- 强 `CoreHandle` 会延长 runtime 生命周期。registry-owned 服务只能保留 `CoreWeak`。
- 生命周期和并发协调已实现并有专项测试；外部模块自己的 cleanup 完整性仍由模块实现者负责。
