---
related_code:
  - zircon_runtime/src/prelude.rs
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/engine_module/mod.rs
  - zircon_runtime/src/core/manager/mod.rs
  - zircon_runtime/src/builtin/runtime_modules.rs
implementation_files:
  - zircon_runtime/src/prelude.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/engine_module/mod.rs
  - zircon_runtime/src/core/manager/resolver.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
tests:
  - zircon_runtime/src/core/runtime/tests.rs
  - zircon_runtime/src/engine_module/tests.rs
  - zircon_runtime/src/core/manager/tests.rs
  - zircon_app/src/tests/prelude.rs
doc_type: workflow-detail
status: current
---

# Rust API 使用指南

## 导入策略

应用与普通模块优先从 `zircon_runtime::prelude` 导入稳定常用类型。编写核心扩展、需要 feature-gated contract 或诊断细节时，再从 `zircon_runtime::core::{runtime, manager, framework}` 精确导入。

不建议使用大范围 glob 穿透内部子模块；`core::runtime::state` 等 crate-private owner 不是扩展 API。

## 产品宿主：按 Profile 组装

```rust
use std::time::Duration;
use zircon_runtime::prelude::{
    runtime_modules_for_runtime_profile,
    CoreError, CoreRuntime, RuntimeProfileId,
};

fn run_runtime() -> Result<(), CoreError> {
    let runtime = CoreRuntime::try_new()
        .map_err(|error| CoreError::Initialization(
            "CoreRuntime".to_owned(),
            error.to_string(),
        ))?;

    let composition = runtime_modules_for_runtime_profile(
        RuntimeProfileId::Client3d,
    );

    // 组合结果包含 modules 与 availability/diagnostics；实际字段以
    // RuntimeModuleCompositionResult 当前定义为准。
    for module in composition.modules() {
        runtime.register_module(module.descriptor())?;
    }

    runtime.activate_registered_modules()?;

    // host loop ...

    runtime.shutdown_registered_modules_with_drain_timeout(
        Duration::from_secs(2),
    )?;
    Ok(())
}
```

`zircon_app` 已实现更完整的配置、插件注册、清理与产品控制器流程。外部产品宿主应尽量复用它，而不是复制一份简化 bootstrap。上例的 `modules()` 访问器必须以 composition 类型当前实现为准，属于流程示意，不是承诺固定字段名。

## 模块作者：实现 `EngineModule`

```rust
use zircon_runtime::core::{InitLevel, ModuleDescriptor};
use zircon_runtime::engine_module::EngineModule;

#[derive(Clone, Copy, Debug)]
pub struct GameplayModule;

impl EngineModule for GameplayModule {
    fn module_name(&self) -> &'static str { "Gameplay" }

    fn module_description(&self) -> &'static str {
        "Gameplay runtime systems"
    }

    fn descriptor(&self) -> ModuleDescriptor {
        ModuleDescriptor::new(
            self.module_name(),
            self.module_description(),
        )
        .with_init_level(InitLevel::Scene)
    }
}
```

生产模块通常还要添加 module dependency、lifecycle、driver/manager/plugin 描述符。`EngineModule` 自身只是生成描述符的开发合同，不会自动注册。

## 服务工厂中的 Core 访问

`factory` 收到 `&CoreWeak`。仅在一次操作边界升级，并把 weak clone 存进长生命周期服务：

```rust
use zircon_runtime::engine_module::factory;

let service_factory = factory(|core_weak| {
    let core = core_weak
        .upgrade()
        .ok_or(zircon_runtime::core::CoreError::RuntimeUnavailable)?;

    let service = MyService::new(core.downgrade());
    Ok(std::sync::Arc::new(service)
        as zircon_runtime::core::runtime::ServiceObject)
});
```

不要把升级后的强 `CoreHandle` 存进 registry-owned `MyService`，否则形成引用环。

## 通过领域 Manager 访问能力

```rust
use zircon_runtime::core::manager::{
    ManagerResolver, ManagerServiceResolver,
};

let resolver = ManagerResolver::new(runtime.handle());
let level_handle = resolver.level_handle()?;
let level_manager = resolver.resolve(level_handle)?;

// 使用 core::framework::scene::LevelManager trait API
```

这条路径适用于 app、editor 和其他 runtime module。若只为测试具体实现细节，才考虑 `resolve_manager::<ConcreteType>`。

## 事件循环

```rust
use zircon_runtime::core::framework::events::{
    EngineEventDeliveryPolicy,
    EngineEventTryReceiveError,
};

let events = runtime.subscribe_events(
    "gameplay.command",
    EngineEventDeliveryPolicy::Latest,
);

runtime.publish_event(
    "gameplay.command",
    serde_json::json!({ "command": "pause" }),
);

match events.try_recv() {
    Ok(event) => println!("{}: {}", event.topic, event.payload),
    Err(EngineEventTryReceiveError::Empty) => {}
    Err(EngineEventTryReceiveError::Disconnected) => return Ok(()),
}
```

## 每帧时间

```rust
const MAX_FIXED_STEPS: u32 = 4;

loop {
    let frame = runtime.tick_time(MAX_FIXED_STEPS);
    let frame_index = frame.outer_frame_index();
    let delta = frame.raw_real_delta();

    // 将 frame 交给拥有各 World 的 LevelSystem；不要在 Core 中
    // 自行维护 World virtual/fixed time。

    runtime.record_diagnostic(
        "host.frame_delta_ms",
        frame_index,
        delta.as_secs_f64() * 1000.0,
        Some("ms"),
        ["host"],
    );

    break; // 示例
}
```

## 错误处理

核心 API 以 `CoreError`/`CoreResult` 为主。产品宿主应至少区分：

- 组合/注册错误：启动前终止并输出依赖诊断。
- 初始化/ready timeout：尝试按宿主策略清理已激活模块。
- service unavailable/stale handle：重新获取 handle 或停止对应功能，不复用旧句柄。
- cleanup/drain timeout：报告未完成项，不能静默当作成功退出。
- task graph shutdown 错误：graph 保持 closing，应进入终止路径。

## API 稳定边界

推荐依赖：`prelude`、`core` 顶层 re-export、`core::framework` 领域合同、`core::manager` 句柄、`engine_module` helpers。

谨慎依赖：具体领域模块的 implementation struct，除非调用者就是其 owner 或测试。

禁止依赖：`core::runtime::state`、注册表锁、crate-private assembly、测试模块、文件布局中的私有 helper。

## 示例准确性说明

本页以 2026-09-09 工作树公开 API 为依据。Profile 组合结果的遍历代码在产品中应参考 `zircon_app/src/entry/engine_entry.rs` 当前实现；因为 composition report 同时携带诊断和可用性，本页没有把它简化成承诺长期存在的 `Vec` API。
