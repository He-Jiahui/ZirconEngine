---
related_code:
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/core/runtime/mod.rs
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/manager/mod.rs
  - zircon_runtime/src/core/math/mod.rs
  - zircon_runtime/src/core/resource/mod.rs
  - zircon_runtime/src/engine_module/mod.rs
implementation_files:
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/core/manager/resolver.rs
  - zircon_runtime/src/core/manager/service.rs
  - zircon_runtime/src/core/math/mod.rs
  - zircon_runtime/src/core/resource/mod.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
tests:
  - zircon_runtime/src/core/framework/tests.rs
  - zircon_runtime/src/core/manager/tests.rs
  - zircon_runtime/src/tests/runtime_absorption
doc_type: module-detail
status: current
---

# 分层与所有权

## Core 主干

`zircon_runtime::core` 是运行时内部的共享主干。它的五个子域不是同义 facade，而是各自拥有不同类型的事实。

| 子域 | 拥有什么 | 不拥有什么 |
| --- | --- | --- |
| `runtime` | 注册表、生命周期、工厂执行、任务、事件、时间、状态、诊断 | 领域业务 trait 的语义定义 |
| `manager` | 稳定服务名、类型化 manager 句柄、resolver | manager 具体实现和业务 DTO |
| `framework` | 跨模块 trait、枚举、描述符、快照和协议 DTO | 锁、线程、服务实例、文件写入等具体行为 |
| `math` | 坐标/单位/精度 schema，数学类型与 render narrowing | 场景组件或渲染后端 |
| `resource` | 资源身份、句柄、状态、事件和持久化 I/O 投影 | 具体资产导入器与编辑器资产 UI |

## `core::runtime`

这是进程级核心权威。`CoreRuntime` 对外提供拥有式门面，`CoreHandle` 是可克隆强句柄，`CoreWeak` 是供 registry-owned 对象反向访问核心的弱句柄。

内部 `CoreRuntimeInner` 分别锁住模块表、服务表、冻结依赖图、激活顺序、生命周期协调器、事件总线、配置存储、任务调度、帧时钟、随机数服务、状态注册表和诊断存储。调用模块生命周期回调和服务工厂时不得长期持有注册表锁。

## `core::manager`

Manager 层解决“上层怎样稳定访问一项服务”，而不是“服务怎样实现”。其核心类型是：

- `ManagerServiceHandle<T>`：包含 index、generation、`RegistryName`、runtime 弱身份和类型 marker。
- `RegisteredManagerService<T>`：把 `Arc<dyn Trait>` 包装为可进入通用注册表的服务对象。
- `ManagerResolver`：只持有 `CoreWeak`，提供领域化 handle 创建与解析。
- `*_MANAGER_NAME`：单一规范服务名，避免各调用方重复字符串常量。

句柄只能交回创建它的 `CoreRuntime`，并且 generation 必须与当前服务槽一致。模块卸载会递增 generation，使之前取得的句柄失效。

## `core::framework`

Framework 层面向领域定义中性契约。当前公开域包括 animation、asset、audio、bridge、camera controller、channel、events、foundation、gizmos、input、navigation、picking、platform、project、random、render、scene、script、tasks、text、time、ui、window，以及受 feature 控制的 AI、network、physics、sound。

正确的依赖方向是：

```text
framework trait/DTO
       ^
       | 被具体模块实现
runtime module implementation
       ^
       | 经 manager handle 暴露
app/editor/plugin consumer
```

如果一个 framework 文件开始执行磁盘 I/O、创建线程、持有 GPU 对象或修改 World，它就越过了契约边界。

## `core::math`

Math 层重新导出规范数学 crate 的类型，并把运行时接口中的 schema 汇聚到同一个入口。重要边界包括：

- `Real` 与 `RenderScalar` 明确区分引擎精度和渲染后端精度。
- `to_render_*` / `try_to_render_*` 负责窄化，而不是在调用点裸 `as` 转换。
- `CoordinateSchema`、`UnitSchema`、`PrecisionProfile` 固定跨模块空间约定。
- `ValidatedTransform`、`ValidatedPerspective` 等类型承担输入验证。

## `core::resource`

Resource 层统一资源 UUID、locator、scheme、typed marker、状态、事件、readiness generation 和 registry 操作。`core::resource::io` 对外仅公开原子写入等稳定能力；恢复事务等装配接口为 crate-private，不属于普通模块 API。

## `engine_module`

`zircon_runtime::engine_module` 是在核心描述符之上的开发便利层：

- `EngineModule` 统一 `module_name`、`module_description`、`descriptor`。
- `factory` / `plugin_factory` 把闭包转换为核心工厂类型。
- `qualified_name` / `dependency_on` 生成规范服务名和依赖。
- `driver_contract` / `manager_contract` / `plugin_contract` 生成可检查的声明合同。

它不是第二套注册表。最终注册、激活、解析仍由 `core::runtime` 执行。

## 边界检查清单

新增能力时应逐项回答：

1. trait/DTO 是否属于 `framework`，具体行为是否留在运行时模块？
2. 上层是否通过 manager handle、descriptor 或命令接口访问，而不是知道实现类型？
3. registry-owned 实例是否只持有 `CoreWeak`？
4. 数据是 runtime authority、editor authority，还是派生快照？
5. 服务依赖是否在描述符中声明，跨模块依赖是否同时声明模块 dependency？
6. 资源和数学类型是否复用了规范 `core` 投影？

## 实现状态

五域目录和主要公开面均已实现。需要注意的是，`framework` 内部领域较多，各领域的功能完整度不同；本页只保证其所有权边界，不承诺每个可选后端均在所有 feature 组合中可用。
