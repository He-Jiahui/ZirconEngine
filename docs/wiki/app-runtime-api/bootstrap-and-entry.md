---
related_code:
  - zircon_app/src/lib.rs
  - zircon_app/src/entry/entry_profile.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/entry_runner/bootstrap.rs
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_app/src/entry/entry_runner/headless.rs
  - zircon_app/src/entry/export_bootstrap.rs
  - zircon_app/src/entry/product_composition/request.rs
  - zircon_app/src/entry/product_composition/composition.rs
implementation_files:
  - zircon_app/src/entry/entry_profile.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/entry_runner/bootstrap.rs
  - zircon_app/src/entry/export_bootstrap.rs
  - zircon_app/src/entry/product_composition/request.rs
  - zircon_app/src/entry/product_composition/composition.rs
plan_sources:
  - user: 2026-09-09 为 ZirconEngine 构建引擎说明书级 Wiki
  - docs/zircon_app/editor-host-entry.md
  - docs/zircon_app/export-bootstrap.md
tests:
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - zircon_app/src/entry/tests/export_bootstrap.rs
  - zircon_app/src/entry/tests/builtin_engine_entry.rs
doc_type: workflow-detail
---

# 应用入口、Profile 与导出启动

## 模块定位

`zircon_app` 是产品宿主层。它决定“启动哪一种产品、采用什么运行策略、选择哪些模块和插件、由谁持有生命周期”，然后把运行时业务交给 `zircon_runtime`。这个分工可防止编辑器、独立游戏和服务器各自复制一套启动顺序。

`EntryProfile` 是最简入口分类：

```rust
pub enum EntryProfile {
    Editor,
    Runtime,
    Headless,
}
```

它会映射到同名的 `EntryRunMode`。更细的产品语义由 `ProductRoleRequest`、`RuntimeProfileId`、`RuntimeTargetMode`、平台配置、插件 manifest 和窗口描述共同解析，不能只靠 `EntryProfile` 推断全部能力。

## 启动对象

| 类型 | 用途 | 生命周期规则 |
| --- | --- | --- |
| `EntryConfig` | 未解析的产品启动请求 | 可由产品角色、导出 profile、插件 manifest 等构建 |
| `ResolvedProductHostConfig` | 合并后的单一配置事实 | 模块选择只能消费解析结果，不得再次解释原始参数 |
| `EntryModuleSelectionReport` | 模块、插件和配置来源的诊断快照 | 适合日志和自动化检查，不持有运行时资源 |
| `ProductCompositionRequest` | 组合请求的唯一汇合点 | 接收链接/原生插件注册报告后执行 `compose()` |
| `ProductComposition` | 完整产品所有者 | 必须活到产品退出；销毁它才释放 Core、插件 owners 和原生 host |
| `EntryRunner` | Runtime、Editor、Headless 的进程运行器 | 负责将参数、动态会话和宿主事件循环串起来 |

## Runtime 可执行程序流程

`zircon_runtime` 可执行入口最终调用 `EntryRunner` 的 runtime 路由。当前可用的会话 profile 为 `runtime`、`runtime-pipelined`、`editor`、`dev`、`minimal` 和 `headless`，默认是 `runtime`。

常用启动形式：

```powershell
zircon_runtime.exe --project E:\Projects\MyGame --runtime-session-profile runtime
```

Play-in-Editor 子进程形式：

```powershell
zircon_runtime.exe `
  --project E:\Projects\MyGame `
  --play-scene .zircon/play/42/play-scene.zrscene.json `
  --play-report-pipe zircon-play-report-42
```

入口首先验证参数唯一性，再把项目目录或 `zircon-project.toml` 解析为一个物理项目根。符号链接、junction 和别名在这一层归一化，后续服务不应各自重新解析项目身份。`--play-scene` 必须是无盘符、无绝对根、无 `.`/`..` 的项目相对 `RelPath`，而且 Play 参数要求同时提供 `--project`。

当前 `--play-report-pipe` 是逻辑出口名，报告实际以单行机器可读记录写入 stdout，由编辑器拥有子进程输出泵。将它提升为原生命名管道属于后续传输实现，不改变会话 ABI 字段。

## Editor 和 Headless

Editor 路由解析诊断和启动意图，创建 `ProductComposition`，再把动态运行时客户端交给 retained editor host。窗口、工作台和 authoring UI 仍属于 `zircon_editor`；`zircon_app` 只拥有进程级组合与交接。

Headless 路由采用显式控制器和 schedule，避免创建原生窗口。是否使用 headless 产品入口和动态会话的 `headless` profile 是两个相关但不同的决定：前者决定宿主形态，后者决定运行时会话策略。

## 导出启动 API

生成的产品脚手架只能准备数据，启动决策必须进入手写 facade。链接式导出使用：

```rust
use zircon_app::{
    bootstrap_export_runtime,
    ExportRuntimeBootstrapConfig,
};
use zircon_runtime::core::framework::project::{
    ExportProfile,
    ProjectPluginManifest,
};

fn start_export(
    plugins: ProjectPluginManifest,
    profile: ExportProfile,
) -> Result<zircon_app::ProductComposition, zircon_runtime::core::CoreError> {
    let config = ExportRuntimeBootstrapConfig::new(plugins, profile);
    bootstrap_export_runtime(config)
}
```

携带动态原生插件的导出使用：

```rust
use zircon_app::{
    bootstrap_export_runtime_with_native_plugins_from_export_root,
    discover_export_root,
    ExportRuntimeBootstrapConfig,
};

let export_root = discover_export_root()?;
let composition = bootstrap_export_runtime_with_native_plugins_from_export_root(
    ExportRuntimeBootstrapConfig::new(project_plugins, export_profile),
    export_root,
)?;
// composition 必须保持存活，直到平台主循环和所有回调结束。
```

`ExportRuntimeBootstrapConfig` 可以附加已生成的注册报告，也可以附加延迟 provider。provider 在手写边界执行，生成代码不直接调用插件注册函数。默认的 `NativePluginArtifactAuthority` 是 deny-all；允许原生插件制品必须显式提供 authority。

## 导出角色映射

桌面目标映射为 `DesktopClient`，无窗口服务器为 `Server`，Android 为 `AndroidClient`，浏览器/Wasm 为 `WebClient`，iOS embedded library 为 `Embedded`。若某角色没有专用宿主 owner，启动必须以 `UnsupportedProductRole` 失败，不能静默降级成桌面客户端。

## 错误处理

启动错误文本采用 `component / requested / cause / recovery` 结构。调用者应完整记录错误，而不是只匹配英文文本。语义进程退出策略由 `ProductExitClass` 和 `ProductProcessExitCode` 负责：成功为 0，产品失败通常为 1，显式命令结果可保留非零 `u8` code。

平台 C/JNI 导出必须用 `catch_unwind` 包围 Rust 入口，禁止 panic 穿过外部 ABI。移动和浏览器模板使用带 `Vacant / Starting / Running / Stopping` 状态的进程级所有者，防止重复启动或销毁期间重入。

## 当前状态

- 已实现：三类入口 profile、统一配置解析、完整 composition 所有权、链接/原生插件导出 facade、导出根发现。
- 部分实现：不同移动/浏览器宿主仍依赖各平台生成模板；能力覆盖由目标和插件包决定。
- 规划约束：生成代码继续保持“数据适配器”角色，不允许重新拥有启动行为。
