---
related_code:
  - zircon_app/src/lib.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/core/mod.rs
  - zircon_runtime/src/scene/mod.rs
  - zircon_editor/src/lib.rs
implementation_files:
  - zircon_app/src/entry/engine_entry.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_runtime/src/scene/mod.rs
  - zircon_editor/src/lib.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - .codex/plans/全系统重构方案.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
tests:
  - zircon_app/src/tests
  - zircon_runtime/src/tests/runtime_absorption
  - zircon_runtime/src/tests/scene_boundary
  - zircon_editor/src/tests
doc_type: module-detail
status: current
---

# 系统总览

## 概览

ZirconEngine 把“进程如何启动”“运行时能力在哪里执行”“编辑器如何修改运行时世界”拆成明确的所有权边界。该设计的直接目的，是让 Editor、Client Runtime、Server Runtime 和导出产物共享同一个核心生命周期，而不共享彼此的产品状态。

## 三个顶层产品包

### `zircon_app`：进程宿主

`zircon_app` 负责解析产品配置、选择目标模式与 Profile、组装模块、创建 `CoreRuntime`、注册描述符、激活模块，并把已启动的运行时交给 editor/runtime/headless 控制器。它可以持有进程级对象，但不应实现资产、场景、输入或渲染业务。

典型启动骨架在 `zircon_app/src/entry/engine_entry.rs` 中：

```rust
let runtime = CoreRuntime::new();
for module in modules {
    runtime.register_module(module.descriptor())?;
}
runtime.activate_registered_modules()?;
```

### `zircon_runtime`：运行时吸收层

`zircon_runtime` 是所有内建高层运行时功能的物理归属。它既提供核心主干，也包含 foundation、platform、input、asset、scene、operation，以及受 feature 控制的 graphics、text、ui、script、navigation、animation 等领域模块。

这意味着 `zircon_runtime` 不是一个只转发其他 crate 的 facade。它是运行时世界、服务实现、跨模块契约投影和产品功能模块的实际所有者。

### `zircon_editor`：作者态宿主

`zircon_editor` 只在编辑器产品模式中进入组合。它拥有编辑命令、历史、选择、viewport 工具、gizmo、overlay、UI workbench 等作者态数据，并通过运行时公开的 manager/scene 接口修改 World。

编辑器不能成为运行时 Scene/World 的第二权威来源。保存数据、渲染提取或游戏逻辑需要读取的状态，应由 runtime world 或明确的中性数据包拥有。

## 权威数据模型

| 数据 | 权威所有者 | 派生消费者 |
| --- | --- | --- |
| 模块与服务生命周期 | `core::runtime` | app、diagnostics、devtools |
| 运行时 World/ECS | `zircon_runtime::scene` | editor hierarchy、render extract、serialization |
| 编辑器选择和工具状态 | `zircon_editor::scene` | viewport overlay、inspector、command history |
| 跨模块 trait/DTO | `core::framework` | manager、runtime modules、editor、plugins |
| 服务名与稳定句柄 | `core::manager` | app、editor、runtime modules |
| 数学单位与精度 | `core::math` | scene、render、physics、animation |
| 资源身份与状态 | `core::resource` | asset、scene、render、editor asset tooling |

## 架构原则

### 描述先于实例

模块通过 `ModuleDescriptor` 声明自身，服务通过 `DriverDescriptor`、`ManagerDescriptor` 或 `PluginDescriptor` 声明。运行时在注册阶段验证名字和依赖，在激活或首次解析时才执行工厂。调用者不应自行创建一个服务实例后偷偷放进上层对象图。

### 依赖是有方向的

服务种类的依赖等级为 `Driver < Manager < Plugin`：Driver 只能依赖 Driver；Manager 可以依赖 Driver/Manager；Plugin 可以依赖三者。跨模块服务依赖还必须有对应的模块级依赖声明。

### 上层持有稳定入口

一般业务使用 `ManagerServiceHandle<dyn Trait>` 或 `ManagerResolver`，而不是缓存具体 `Arc<Implementation>`。句柄携带服务槽位、generation、规范名称和所属 runtime 身份，因此卸载后旧句柄会被判定为过期。

### World 与 Editor State 分离

运行时 World 负责实体、层级、变换、可渲染对象、相机、灯光、序列化和基础渲染提取。选择、高亮、gizmo、临时相机覆盖、viewport 工具属于 Editor。该边界保证同一个 World 能用于游戏、服务器、编辑器和无头测试。

## 当前实现状态

| 能力 | 状态 | 说明 |
| --- | --- | --- |
| 三包产品形态 | 已实现 | `zircon_app`、`zircon_runtime`、`zircon_editor` 均存在并承担对应职责 |
| runtime 内部 core 主干 | 已实现 | `runtime/framework/manager/math/resource` 均位于 `zircon_runtime::core` |
| 描述符注册与有向依赖 | 已实现 | 包含模块/服务拓扑排序、循环检测和类型规则 |
| Editor/Scene 完全收束 | 部分实现 | 已有边界测试；具体子系统仍需以当前源码逐项核对 |
| VM 插件替换语义 | 部分实现 | 宿主协议与运行时插件体系存在，不能假设所有原生插件路径已消失 |
| 独立 `zircon_core` 等根 crate | 目标架构已被后续收束计划替换 | 当前权威代码形态是 `zircon_runtime::core::*`，不要按早期计划导入不存在的 crate |

## 限制与扩展规则

- 不允许新增第四个承担通用引擎所有权的根产品包。
- `server` 命名仅用于真实网络或服务端语义，不能作为 manager facade 的泛称。
- `core::framework` 只能包含中性契约和 DTO，不得持有具体线程、注册表或业务实现。
- runtime-owned 服务不能强持有 `CoreHandle`；使用 `CoreWeak` 避免 `CoreRuntimeInner -> service -> CoreHandle` 引用环。
- 可选领域由 Cargo feature 和模块组合共同决定。公开类型存在不等于对应产品一定激活该能力。
