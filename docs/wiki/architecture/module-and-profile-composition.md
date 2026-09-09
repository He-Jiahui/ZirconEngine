---
related_code:
  - zircon_runtime/src/builtin/runtime_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/assembly.rs
  - zircon_runtime/src/builtin/runtime_modules/core_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/manifest.rs
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/core/framework/asset.rs
  - zircon_app/src/entry/engine_entry.rs
implementation_files:
  - zircon_runtime/src/builtin/runtime_modules/assembly.rs
  - zircon_runtime/src/builtin/runtime_modules/core_modules.rs
  - zircon_runtime/src/builtin/runtime_modules/manifest.rs
  - zircon_app/src/entry/engine_entry.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
tests:
  - zircon_runtime/src/builtin/runtime_modules/tests
  - zircon_app/src/tests
doc_type: workflow-detail
status: current
---

# 模块与产品组合

## 概览

ZirconEngine 不把“编译进二进制”和“在当前产品中启用”视为同一件事。最终模块集合由四类输入共同决定：Cargo feature、`RuntimeTargetMode`、`RuntimeProfileId`、项目插件清单及插件注册报告。

## 基础模块集合

`runtime_core_module_candidates_for_target_with_render_features` 构造候选模块。无论图形 feature 是否开启，基础集合包括：

| 模块 | 初始化职责 | Target 约束 |
| --- | --- | --- |
| `FoundationModule` | 配置管理与运行时基础服务 | 全部 |
| `LogModule` | 核心日志能力 | 全部 |
| `TasksModule` | 任务与调度能力 | 全部 |
| `TimeModule` | 时间策略与帧时间诊断 | 全部 |
| `FrameCountModule` | 帧计数诊断 | 全部 |
| `DiagnosticsCoreModule` | 核心诊断聚合 | 全部 |
| `PlatformModule` | 平台与窗口/设备能力 | 全部，具体后端依产品 |
| `InputModule` | 输入帧与 action 服务 | 全部 |
| `AssetModule` | 资源管理与导入器注册 | 全部 |
| `SceneModule` | Level/World 运行时 | 全部 |
| `TextModule` | 文本能力 | feature 开启且非 ServerRuntime |
| `GraphicsModule` | 图形、render framework 与扩展 provider | graphics feature 开启且非 ServerRuntime |
| `ScriptModule` | 脚本/插件运行时 | script feature 开启且非 ServerRuntime |

候选集在返回前按 `ModuleDescriptor` 的 `InitLevel` 和显式依赖拓扑排序。不要依赖源码中 `vec![]` 的书写顺序表达语义。

## Target 模式

`RuntimeTargetMode` 当前包含：

- `ClientRuntime`：客户端运行时，默认允许 UI/graphics/text/script 等产品能力。
- `ServerRuntime`：服务端/无头运行时，默认不加入 UI 插件，且基础候选集排除 graphics/text/script。
- `EditorHost`：编辑器宿主，运行时模块之外还允许 editor 专用插件和作者工具。

ServerRuntime 的“排除”发生在组合层，因此调用端不能只检查一个类型是否编译存在来判断功能是否可用。

## 项目插件清单

`default_manifest_for_target` 为 ClientRuntime 和 EditorHost 生成默认 UI 与 UI 文档导入器选择；ServerRuntime 默认清单为空。`manifest_with_mode_baseline` 先建立 target 基线，再应用项目 override。

清单覆盖的原则是：

1. Target 决定产品允许的基线。
2. 项目 manifest 表达启用/禁用意图。
3. 链接或动态发现的插件注册报告提供实际可用性。
4. feature 注册报告提供插件贡献的细粒度能力。
5. composition compiler 输出模块与诊断，而不是静默忽略不可满足选择。

## Profile 入口

调用方通常优先使用 Profile API：

```rust
use zircon_runtime::prelude::{
    runtime_modules_for_runtime_profile,
    RuntimeProfileId,
};

let composition = runtime_modules_for_runtime_profile(RuntimeProfileId::Editor);
```

实际 `RuntimeProfileId` 变体应以 `core::framework::project` 当前定义为准；这里的重点是入口形状。若已经编译了不可变 `CompiledProjectPluginPlan`，使用对应的 `runtime_modules_for_*_compiled_project_plugin_plan` 可保证组合来自同一 plan generation。

## 宿主启动流程

```text
产品配置 / Profile / Target
          |
          v
项目插件清单 + 注册报告 + feature 报告
          |
          v
RuntimeModuleCompositionCompiler
          |
          +--> modules（拓扑有序）
          +--> availability / diagnostics
          |
          v
zircon_app: register_module -> activate_registered_modules
```

## 限制

- `runtime_core_modules()` 使用 ClientRuntime 基线，适合简单调用，不适合需要精确产品裁剪的宿主。
- `runtime_modules_for_target*` 中部分入口保留为 legacy report 包装；新代码若已有 compiled plugin plan，应优先使用 compiled-plan 入口。
- 模块存在于组合结果并不代表其服务已经实例化。`Immediate` 在模块激活时创建，`Lazy` 在首次解析时创建。
- 插件清单不能突破编译 feature 和 Target 的硬可用性边界。

## 实现状态

基础模块组合、Target 基线、Profile、插件清单与编译计划入口均已实现，并有 registration、availability、manifest、profile 和 composition 测试。插件生态中各插件的功能完整度不由本页保证。
