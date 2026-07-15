---
related_code:
  - zircon_plugins/navigation/editor/src/plugin.rs
  - zircon_plugins/navigation/editor/src/plugin/registration/mod.rs
  - zircon_plugins/navigation/editor/src/bake_panel.rs
  - zircon_plugins/navigation/editor/src/overlay.rs
  - zircon_plugins/navigation/editor/src/runtime_mirror.rs
  - zircon_plugins/navigation/runtime/src/agent.rs
  - zircon_plugins/navigation/runtime/src/manager/tick.rs
  - zircon_runtime/src/core/framework/navigation/agent.rs
  - zircon_plugins/navigation/editor/bake.zui
  - zircon_plugins/navigation/editor/debug_gizmos.zui
implementation_files:
  - zircon_plugins/navigation/editor/src/plugin/registration/assets.rs
  - zircon_plugins/navigation/editor/src/plugin/registration/components.rs
  - zircon_plugins/navigation/editor/src/plugin/registration/operations.rs
  - zircon_plugins/navigation/editor/src/plugin/registration/templates.rs
  - zircon_plugins/navigation/editor/src/bake_panel.rs
  - zircon_plugins/navigation/editor/src/overlay.rs
  - zircon_plugins/navigation/editor/src/runtime_mirror.rs
  - zircon_plugins/navigation/editor/bake.zui
  - zircon_plugins/navigation/editor/debug_gizmos.zui
plan_sources:
  - docs/plans/zircon_plugins/05-navigation.md
  - docs/plans/zircon_plugins/10-editor-integration.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - user: 2026-07-12 严格按 zircon_plugins 架构计划完成插件功能
tests:
  - zircon_plugins/navigation/editor/src/tests.rs
doc_type: milestone-detail
---

# 2026-07-13 Navigation M6 Editor 产出记录

## 状态与产出记录

| 里程碑 | 切片 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| M6 | T1 烘焙面板 | `本计划修复完成-Editor03 gate open` | 2026-07-13 | command overwrite、framework request/force-full mapping、backend rejection、action-aware monotonic progress 与 stale report 已修；真实 factory/transaction/runtime adapter 归 Editor03 M3.2 failure。 |
| M6 | T2 NavMesh viewport overlay | `本计划修复完成-Editor05 gate open` | 2026-07-13 | provider id、toggle controller/sink、多 area 颜色与 path/vector 投影已落；真实 provider registry/factory 与 render-packet 合并归 Editor05 failure。 |
| M6 | T3 Agent/avoidance PIE 调试 | `本计划修复完成-Plugins12 gate open` | 2026-07-13 | shared serde DTO、runtime producer、只读 session/sequence mirror、capture resource gate 与真实 path query 已落；runtime-client typed consumer 归 Plugins12 D9 failure。 |
| M6 | Testing | `未验收` | 2026-07-13 | 修复后受管 current-source gate 被外部 Render11 lightmap 编译错误阻断；独立复审仍为 Not Ready，三个跨计划产品 wiring failure 未返回前不声明 M6 完成。 |

## 架构与结构收敛

- `plugin.rs` 从混合注册实现收敛为 70 行入口/lifecycle 适配器。
- 扩展注册按 assets/components/operations/templates 拆入 `plugin/registration/`，由插件声明模块拥有，避免 crate root 与入口文件继续承担多域职责。
- Bake、Overlay、PIE mirror 各自拥有独立模块；当前生产 Rust 文件均低于 200 行。
- 编辑器只消费 framework/runtime DTO，不旁路持有 `World`、`NavigationManager` 或 renderer 内部对象。
- `plugin.rs` 已迁至 SDK `authoring_plugin!`，关闭 review D5；debug capture 默认关闭，避免非 PIE 时每 agent 每帧分配，并移除 editor mirror 的二次 agent-vector clone。

## UI 对位

- 参考 `docs/ui-and-layout/ai-workbench-style/ai-navmesh-ai-layout.png` 的导航工具/中心工作区/右侧详情结构。
- `bake.zui` 使用 surface list / settings / diagnostics 三栏和底部进度区。
- `debug_gizmos.zui` 使用 overlay filter / viewport / PIE agent mirror 三栏。
- 控件密度按 STYLE-NOTES 的 28–32 px 命令控件与暗色 editor shell 约定。

## 验证记录

- 测试先行：新增 M6 契约后首次受管验证未进入编译，协调器返回 `cargo_reuse_pool_busy`；占用者为并行 `zircon_editor` 验证任务，因此不将该次资源阻塞计作代码失败。
- 静态 `.zui` 校验：`bake.zui` 19 nodes、`debug_gizmos.zui` 13 nodes，TOML 可解析且 child 引用均存在。
- 独立受管 target 已进入实际编译，但在到达 Navigation Editor 前被并行 IBL/ProceduralSky 迁移阻断：`zircon_runtime` 报告缺失 PMREM 常量/方法和旧 Sky 字段等约 100 个错误；对应文件由 Render/Runtime05 Session 持有且 Runtime05 正在修复最低共享层，Navigation M6 不越权修改。
- 协调器稳定验证副本尝试因全仓 2.5 万文件物化触发服务重启，只留下不完整副本，已清理；仓库 main-only 策略禁止用 worktree 替代。
- 最低共享层修复进入工作区后，受管独立 Windows target 的完整 build 通过；首次 test 到达本包后暴露旧 `extensions.commands()` 测试 API，按当前 `command_ids()` 契约修复。
- 最终 `validate-matrix.ps1 -Package zircon_plugin_navigation_editor -TargetDir E:\cargo-targets\zircon-navigation-m6-editor -SkipBuild` 通过；测试二进制列出 6 tests、0 benchmarks，全部执行成功。插件子工作区由同一验证脚本进程注入 `zircon_plugins/Cargo.toml` manifest，Cargo job 与 target 仍由协调器登记。
- 首轮独立审查：`Not Ready`（0 Critical / 3 Important / 3 Minor）；修复 command overwrite、SDK macro、capability/multi-area、progress 与 shared DTO 后复审仍 `Not Ready`，确认剩余问题属于 Editor03 M3.2、Editor05 viewport host、Plugins12 D9 的共享产品 wiring，而非 Navigation 内可合法旁路的局部实现。
- 当时建立的三个共享 handoff 中，[Editor03 operation factory](fixed-2026-07-15-plugin-operation-factory-runtime-wiring.md) 与 [Plugins12 mirror consumer](fixed-2026-07-15-plugin-editor-runtime-mirror-consumer-wiring.md) 已返回 fixed，[Editor05 overlay provider](../../zircon_editor/editor/05/failure-2026-07-13-plugin-viewport-overlay-provider-runtime-wiring.md) 仍为 open；后续独立复核新增的 [selected-surface 参数投影](failure-2026-07-15-navigation-bake-selection-operation-arguments.md) 由 Plugins05 继续处理。
- 修复后受管复验进入共享 `zircon_runtime` 编译，阻断于并行 Render11 lightmap 当前源：`mesh_pipeline_cache/construct.rs` 使用作用域外 `queue`，`forward_shadow_receiver.rs` 从临时 bindings 借用；Navigation 不越权修改其持有文件。
