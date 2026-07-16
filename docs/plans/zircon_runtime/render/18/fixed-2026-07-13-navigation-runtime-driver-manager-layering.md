---
handoff_kind: fixed
status: fixed
created_at: 2026-07-13
summary_slug: navigation-runtime-driver-manager-layering
origin_plan: docs/plans/zircon_runtime/render/18-advanced-lighting-features.md
fixing_plan: docs/plans/zircon_plugins/05-navigation.md
origin_child_dir: docs/plans/zircon_runtime/render/18
fixing_child_dir: docs/plans/zircon_plugins/05
related_code:
  - zircon_plugins/navigation/runtime/src/lib.rs
  - zircon_plugins/navigation/runtime/src/tests/registration.rs
  - zircon_runtime/src/navigation/module.rs
  - zircon_runtime/src/core/runtime/handle/registration/validation.rs
  - zircon_app/src/bin/editor.rs
tests:
  - cargo build -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host --locked --jobs 1
  - .codex/tmp/runtime02_validate_editor_startup.ps1 -Executable .codex/tmp/plan18_hybrid_gi_zircon_editor_product_r2.exe -BuiltinView editor.runtime_diagnostics -OutputDirectory .codex/tmp/plan18_hybrid_gi_editor_runtime_diagnostics_startup_r2 -TimeoutSeconds 180
  - cargo test -p zircon_plugin_navigation_runtime --lib tests::registration --locked
resolved_at: 2026-07-13
---


# Navigation 05：runtime driver/manager 层级违反启动合同

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/render/18-advanced-lighting-features.md`
- 来源执行切片：HybridGI M4 / real Editor Runtime Diagnostics actual/fallback product gate
- 修复责任计划：`docs/plans/zircon_plugins/05-navigation.md`
- 交接原因：Render18 的真实 Editor 诊断画面必须经过 `target-editor-host` 产品入口；当前最低失败是 Navigation runtime 插件自己的模块描述符把 Driver 反向依赖到 Manager，属于 Navigation 05 服务注册 owner，Render18 不得跳过插件或放宽 Core 服务层级验证。

## 失败现象与复现证据

2026-07-13 当前源码的受管 Windows 产品构建已经通过：

`cargo build -p zircon_app --bin zircon_editor --no-default-features --features target-editor-host --locked --jobs 1`

随后以隔离 `ZIRCON_CONFIG_PATH`、`LOCALAPPDATA`、`APPDATA` 启动
`--builtin-view editor.runtime_diagnostics`，进程约 8 秒后 exit 1，只出现一个可见但 `0x0` 的辅助窗口，未创建有效主窗口。标准错误为：

```text
failed to create runtime session: error: register runtime module:
invalid service dependency for navigation.runtime.Driver.SceneNavigationRuntime:
Driver cannot depend on navigation.runtime.Manager.DefaultNavigationManager (Manager)
```

完整报告位于 `.codex/tmp/plan18_hybrid_gi_editor_runtime_diagnostics_startup_r2/report.json`；
`valid_main_window_observed=false`、`timed_out=false`、`forced_stop=false`、`stack_overflow_observed=false`。
这证明旧 startup stack overflow 已不再是当前最低失败，当前阻断发生在窗口创建前的 Navigation 模块注册。

## 最低共享层根因

`zircon_plugins/navigation/runtime/src/lib.rs::module_descriptor()` 把 concrete
`DefaultNavigationManager` 注册为 `Manager`，再让 `Driver.SceneNavigationRuntime` 依赖该 Manager 并通过
`resolve_manager` 取得实现。Core 的服务层级合同明确禁止 Driver 依赖 Manager；同仓
`zircon_runtime/src/navigation/module.rs` 已给出正确 owner 形态：具体导航实现是无依赖 implementation Driver，
Scene Driver 与公开 Manager facade 都依赖该 Driver。

## 架构修复验收

- Navigation 插件具体实现必须注册为无依赖 Driver；`SceneNavigationRuntime` Driver 只能依赖 Driver，公开 `NavigationManagerHandle` Manager 可依赖同一 implementation Driver。
- 新增模块描述符层级测试，逐项断言 implementation/scene/public facade 的 kind、依赖名和依赖 kind，并用 `CoreRuntime::register_module` 与三个 typed resolve 验证真实行为。
- `zircon_plugin_navigation_runtime` 聚焦注册测试通过，且不得破坏现有 agent/bake/query owner。
- 重新执行上述 `target-editor-host` 产品构建和隔离 Runtime Diagnostics 启动；模块注册不得再报告 Navigation Driver->Manager 反向依赖，Render18 才能继续真实 actual/fallback 窗口截图。

## 禁止临时方案

- 禁止放宽或删除 Core 的 Driver/Manager 依赖层级验证。
- 禁止在 Editor 启动、profile 或 Render18 测试中跳过 Navigation 插件来制造窗口。
- 禁止用 alias、兼容 manager、字符串特判、test-only bypass 或重复 concrete runtime owner 绕过真实模块描述符修复。

## 修复结果与回传

- 根因：Navigation plugin registered its concrete DefaultNavigationManager as a Manager, forcing the SceneNavigationRuntime Driver to depend on and resolve a Manager in violation of Core service layering.
- 架构修复：Hard-cut the concrete implementation to the unique Driver.DefaultNavigationRuntime service; both the SceneNavigationRuntime Driver and public NavigationManager facade now depend on and resolve that implementation Driver, with no legacy Manager alias.
- 验证：TDD RED reproduced missing implementation Driver; focused layer/typed-resolve test passed 1/1; complete current Navigation runtime binary passed 61/61; current target-editor-host product build passed; isolated Runtime Diagnostics startup exited 0 and observed a 1688x980 Zircon Editor main window with no timeout, force-stop, stack overflow, or Navigation registration error.
- 回传：Navigation module layering is fixed and the Render18 real Editor gate can resume; retained Runtime Diagnostics overlay/text readability remains a separate Render18/Editor UI visual acceptance issue.
