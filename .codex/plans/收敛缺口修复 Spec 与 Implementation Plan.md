# 收敛缺口修复 Spec 与 Implementation Plan

## Summary
- 产出两份总文档：`docs/superpowers/specs/2026-04-18-convergence-gap-closure-design.md` 与 `docs/superpowers/plans/2026-04-18-convergence-gap-closure-implementation.md`。
- 文档只覆盖当前仍未闭合的三项：`zircon_ui` 的 stub descriptor、`zircon_script` 的 plugin host context/backend family、`zircon_editor` 的 `editing/ui_asset/session.rs` 热点。
- 实施顺序固定为 `zircon_ui -> zircon_script -> zircon_editor`。先去掉 skeleton，再收敛 core/plugin contract，最后做 editor 大体量下沉，避免 editor 先拆完又被底层接口返工。

## Key Changes
- `zircon_ui`
  - `UiModule` 继续作为 owner，`zircon_ui/src/module/ui_module_descriptor.rs` 改成真实 `ModuleDescriptor`，不再调用 `stub_module_descriptor`。
  - 增加一个真实可注册的 `UiRuntimeDriver` 作为 UI runtime anchor；它可以是 no-op runtime service，但必须有明确类型、稳定注册名和真实 `DriverDescriptor`。
  - 现有 `UiEventManager` 升格为真实 `ManagerDescriptor` 服务，模块激活后必须可被 `CoreRuntime` 解析。
  - spec 明确目标状态是 `zircon_ui` 从 `skeleton` 升到 `converged`，不是“仍靠 stub 但换个名字”。

- `zircon_script`
  - `zircon_core::PluginContext` 从 `{ plugin_name, core }` 扩成真正的宿主上下文基类，至少包含插件注册名、`CoreWeak`、package/source/data roots；不把 VM 专属 backend 细节硬塞进 core。
  - `PluginDescriptor` 改为使用 plugin-context-aware factory，而不是继续复用只接收 `&CoreHandle` 的 `ServiceFactory`；`DriverDescriptor` 和 `ManagerDescriptor` 保持现状。
  - `zircon_script` 新增 `VmPluginHostContext`，在 core `PluginContext` 之上补齐宿主 capability、backend family 选择结果、`VmPluginPackageSource`、slot lifecycle façade、`HostRegistry`。
  - `VmBackendRegistry` 收敛成 backend family registry；新增 `VmBackendFamily` 作为真正的可插拔后端族接口，`Unavailable`/`Mock` 变成该家族的内建实现，不再把“两个 backend 实例”当成完整 backend 体系。
  - `VmPluginInstance::activate`、backend load path、`HotReloadCoordinator` 都改成接收 `VmPluginHostContext`，由 runtime 显式传入宿主上下文，而不是 plugin runtime 再通过 `CoreHandle` 回查自己。

- `zircon_editor`
  - `zircon_editor/src/editing/ui_asset/mod.rs` 继续以 `#[path = "session/mod.rs"] mod session;` 为根，`session/mod.rs` 成为唯一真实 session 模块根。
  - 把 `zircon_editor/src/editing/ui_asset/session.rs` 的真实实现迁入 `session/` 子树，目标文件固定为 `ui_asset_editor_session.rs`、`lifecycle.rs`、`command_entry.rs`、`preview_compile.rs`、`style_inspection.rs`、`hierarchy_projection.rs`、`palette_state.rs`、`binding_state.rs`、`session_state.rs`。
  - `ui_asset_editor_session.rs` 只保留类型声明、错误类型、必要构造入口和极窄 façade；palette、binding、style、hierarchy、preview 的具体 impl 迁入各自责任文件。
  - 旧 `session.rs` 不能继续作为大文件实现仓库；最终状态是删除它，或把它降成不会再被审计标红的极薄 shim。
  - editor 计划里必须单列“test-only 编译漂移修复”和“`ui_asset_sessions` 结构测试回绿”，不能把它们混进 session 拆分任务里略过。

## Public Interfaces
- `zircon_core`
  - `PluginContext` 新增 host roots 与稳定 plugin metadata。
  - `PluginDescriptor` 的 plugin factory surface 改成显式接收 `PluginContext`。
- `zircon_script`
  - 新增 `VmPluginHostContext`。
  - 新增 `VmBackendFamily` 与对应 registry。
  - `VmPluginInstance::activate` 和 backend load entry 改为接收 host context。
  - `VmPluginSlotLifecycle` 作为独立 façade 暴露 load/hot-reload/unload/query slot 记录的操作面。
- `zircon_ui`
  - 新增 `UiRuntimeDriver`。
  - `UiEventManager` 进入正式 module descriptor。
- `zircon_editor`
  - `UiAssetEditorSession` 的对外构造与命令入口保持兼容。
  - 内部实现拆分不改变 `UiAssetEditorSession::from_source` 与现有 host/editor 调用面。

## Implementation Tasks
1. 先写 spec 文档，固定三条修复线的审计现状、目标状态、接口变更、依赖边界和验收标准；文档标题用 `Convergence Gap Closure Design`。
2. 实施计划文档把 `zircon_ui` 作为 Task 1：新增 `UiRuntimeDriver`、真实 `ModuleDescriptor`、core resolve 测试与 audit 目标，明确不再允许 `stub_module_descriptor`。
3. Task 2 固定 `zircon_core` plugin contract 收敛：`PluginContext` 扩展、plugin factory 签名分流、`resolve_plugin` 构造路径改为基于 plugin context。
4. Task 3 固定 `zircon_script` 收敛：引入 `VmPluginHostContext`、backend family registry、family-based load/hot reload path、slot lifecycle façade 和 package root/source root 传递。
5. Task 4 固定 `zircon_script` 验证：现有 `vm/tests.rs` 扩展为 context propagation、backend family selection、slot lifecycle、discover/load/hot reload/unload 回归，验收结果是 audit 不再给出 `plugin-runtime-gap`。
6. Task 5 固定 `zircon_editor` 拆分：先把 struct/error 和最窄 façade 迁到 `session/ui_asset_editor_session.rs`，再按 lifecycle、command entry、preview、style、hierarchy、palette、binding 七个责任面逐步搬迁 impl，最后移除或清空旧 `session.rs`。
7. Task 6 单独处理 `zircon_editor` 现存 test-only 编译漂移，再跑 `src/tests/editing/ui_asset/`、`src/tests/editing/ui_asset_palette_drop.rs`、`src/tests/host/manager/` 中 `ui_asset_sessions` 结构测试，保证 session 拆分后仍是绿色。
8. Task 7 做全局审计收尾：运行 `audit_runtime_structure.py --json`，预期 `zircon_ui`、`zircon_script`、`zircon_editor` 都转成 `converged`，且 JSON 中不再出现 `stub-module-descriptor`、`plugin-runtime-gap`、`large-production-file`。

## Test Plan
- `zircon_ui`
  - 模块注册/激活后可解析 `UiRuntimeDriver` 与 `UiEventManager`。
  - 不再存在 `stub_module_descriptor` 使用点。
- `zircon_script`
  - package discovery 得到的 source/package root 会进入 host context。
  - backend selection 通过 backend family 完成，而不是只有 `Unavailable/Mock` 实例切换。
  - slot load/hot reload/unload/query 都通过 lifecycle façade 可验证。
  - core `resolve_plugin::<VmPluginManager>` 走的是新 plugin-context-aware 构造路径。
- `zircon_editor`
  - `UiAssetEditorSession::from_source`、palette drop、binding/style/hierarchy 操作、preview compile 现有测试继续通过。
  - `ui_asset_sessions` 结构测试回绿。
  - `editing/ui_asset/session.rs` 不再是审计热点。
- 全局
  - `python .\\.codex\\skills\\zircon-project-skills\\zr-runtime-interface-convergence\\scripts\\audit_runtime_structure.py --json`
  - 若时间允许，再补 `cargo test -p zircon_ui`、`cargo test -p zircon_script`、`cargo test -p zircon_editor` 作为文档中的推荐验证序列。

## Assumptions
- 这轮只写 1 份总 spec 和 1 份总 implementation plan，不再拆成三份并行文档。
- `real backend` 的定义是“可插拔 backend family + 完整宿主上下文 + slot lifecycle contract 已到位”，不是必须在本轮接入外部真实 VM 执行器。
- `UiRuntimeDriver` 可以是 no-op runtime anchor，但必须是真实 service type，而不是 descriptor helper 返回的假骨架。
- `UiAssetEditorSession` 的外部 API 以兼容为先；本轮重点是内部责任下沉和测试恢复，不顺带改 editor 产品行为。
