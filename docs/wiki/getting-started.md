---
related_code:
  - Cargo.toml
  - zircon_app/src/lib.rs
  - zircon_app/src/entry/mod.rs
  - zircon_runtime/src/lib.rs
  - zircon_runtime/src/core/runtime/runtime.rs
  - zircon_editor/src/lib.rs
  - zircon_editor/src/ui/retained_host/app.rs
implementation_files:
  - zircon_app/src/entry
  - zircon_runtime/src/core/runtime
  - zircon_editor/src/ui/retained_host
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 详细 Wiki 文档集合
  - docs/plans/mvp/index.md
tests:
  - zircon_app/src/tests
  - zircon_runtime/src/core/runtime/tests
  - zircon_editor/src/tests
  - .github/workflows/ci.yml
doc_type: workflow-detail
---

# 快速开始

本页给出从源码工作区进入 ZirconEngine 的最短路径。它适合确认 crate 关系、选择产品 profile 和编写一个最小宿主；完整的项目创建、资产导入和编辑器操作请继续阅读[产品生命周期](product-lifecycle.md)、[场景与资产](scene-assets/index.md)和[编辑器](editor/index.md)。

## 前置条件

- Rust toolchain 能够解析 workspace 的 Rust 2021 crate。
- Windows 是普通 Cargo 验证的首选环境；平台策略和目标目录限制见[测试与平台](testing-platform/index.md)。
- 运行产品时需要与 profile 匹配的窗口、图形后端和插件/资产构建物。`target-server` 使用 headless 平台，不应尝试创建窗口。
- 不要从仓库根目录直接产生 `target/`。验证脚本会把构建物放入受管的 `D:\`、`E:\` 或 `F:\` 目标池。

## 工作区入口

| 包 | 角色 | 典型依赖方向 |
| --- | --- | --- |
| `zircon_app` | 入口、profile 选择、主循环和产品组合 | 依赖 runtime/editor/interface |
| `zircon_runtime` | 引擎实现、World、资源、图形、UI 和模块内核 | 对外提供 facade，内部组合 `core/*` 与 `zr_*` |
| `zircon_editor` | 作者态会话、场景工具、命令/Workbench 宿主 | 通过 gateway/契约消费 runtime |
| `zircon_runtime_interface` | ABI/DTO/句柄/状态契约 | 不持有引擎行为实现 |
| `zircon_runtime_host` | 宿主侧 ABI 输出所有权与解码 | 依赖 interface |
| `zircon_hub` | 桌面启动、项目列表和本地服务 | 启动 app，不回流 editor |
| `zircon_plugins` | 独立插件 workspace、SDK、catalog 和 dist | 通过 SDK/interface 接入 |

## 运行验证命令

先做不产生构建物的元数据检查：

```powershell
cargo metadata --no-deps --format-version 1
```

需要编译或测试时使用仓库验证器，而不是裸 `cargo build`：

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_runtime_interface
```

验证器会选择兼容 target pool、检查磁盘余量、管理缓存并记录退出状态；不要通过自定义 `--target-dir` 绕过这些约束。

## 最小 CoreRuntime

`CoreRuntime::try_new` 创建一个拥有自己任务图的运行时；`CoreRuntime::new` 在初始化失败时 panic，适合已经完成进程配置预检的入口，不适合需要向调用者报告错误的库代码。

```rust
use zircon_runtime::core::CoreRuntime;

fn make_runtime() -> Result<CoreRuntime, Box<dyn std::error::Error>> {
    let runtime = CoreRuntime::try_new()?;
    runtime.activate_registered_modules()?;
    Ok(runtime)
}
```

显式控制线程预算时传入 `EngineTaskGraphOptions`；需要确定性随机序列或手动时钟时，使用 `with_random_seed`、`with_clock_source` 等构造器。模块注册必须通过 `ModuleDescriptor`，不能在调用方自行排序和直接构造 manager。

## 最小编辑器宿主

编辑器 crate 的公开入口是 `run_editor`、`run_editor_with_startup_request` 和 `run_editor_with_config`。完整窗口循环由 retained host 接管；业务状态仍由 `EditorManager` 和 runtime gateway 负责。

```rust
use zircon_editor::{
    run_editor_with_config, EditorHostRunConfig, SharedEditorRuntimeGateway,
};

fn run_editor_host(
    core: zircon_runtime::core::CoreHandle,
    runtime_gateway: SharedEditorRuntimeGateway,
) -> Result<(), Box<dyn std::error::Error>> {
    let config = EditorHostRunConfig::new()
        .with_startup_layout_preset("default")
        .with_exit_after_first_presented_frame(false);
    run_editor_with_config(core, runtime_gateway, config)?;
    Ok(())
}
```

自动化截图或 smoke 测试可使用 `run_retained_host_automation`，并设置首帧退出/捕获路径；不要把测试专用 callback 当成普通产品交互 API。

## 选择产品入口

`zircon_app::entry` 通过 `EntryConfig`、`EntryProfile`、`ProductCompositionRequest` 和 runtime profile 组合入口。常见产品角色是：

- **Client**：窗口、输入、图形、UI、文本和可选插件。
- **Editor host**：在 client 能力上加载作者态 editor。
- **Server**：headless、无窗口，按功能合同裁剪图形/平台依赖。

应用入口应先做 profile/feature 能力校验，再调用 runtime/editor bootstrap。动态插件由 manifest 和导出根扫描进入统一注册链，不能在应用层偷偷链接 plugin 内部模块。

## 下一步

1. 阅读[基础概念](concepts.md)理解 World、资源、模块、gateway 和 ABI 术语。
2. 按[产品生命周期](product-lifecycle.md)走启动、帧、保存和关闭顺序。
3. 需要渲染时阅读[图形与渲染](graphics/index.md)；需要作者态操作时阅读[编辑器命令与事务](editor/commands-transactions.md)。
