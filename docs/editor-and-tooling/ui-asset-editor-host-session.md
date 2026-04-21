---
related_code:
  - zircon_editor/src/lib.rs
  - zircon_editor/src/ui/mod.rs
  - zircon_editor/src/ui/host/mod.rs
  - zircon_editor/src/ui/host/module.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/ui/host/editor_ui_host.rs
  - zircon_editor/src/ui/host/editor_asset_manager/mod.rs
  - zircon_editor/src/ui/host/editor_session_state.rs
  - zircon_editor/src/ui/host/layout_commands.rs
  - zircon_editor/src/ui/host/resource_access.rs
  - zircon_editor/src/ui/host/window_host_manager.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/mod.rs
  - zircon_editor/src/ui/host/ui_asset_promotion.rs
  - zircon_editor/src/ui/asset_editor/mod.rs
  - zircon_editor/src/ui/asset_editor/preview/preview_mock.rs
  - zircon_editor/src/ui/asset_editor/session/mod.rs
  - zircon_editor/src/ui/asset_editor/source/mod.rs
  - zircon_editor/src/ui/slint_host/app/ui_asset_editor.rs
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/host_scaffold.slint
  - zircon_editor/ui/workbench/host_surface.slint
  - zircon_editor/ui/workbench/host_surface_contract.slint
  - zircon_editor/ui/workbench/host_root.slint
  - zircon_editor/src/tests/editing/ui_asset_preview_binding_authoring.rs
  - zircon_editor/src/tests/host/manager/mod.rs
  - zircon_editor/src/tests/ui/boundary/mod.rs
  - zircon_editor/src/tests/ui/boundary/asset_editor_structure.rs
  - zircon_editor/src/tests/ui/boundary/host_cutover.rs
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/mod.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/reflection.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/bootstrap_assets.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/editor_layouts.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/runtime_previews.rs
  - zircon_editor/tests/workbench_slint_shell.rs
implementation_files:
  - zircon_editor/src/ui/host/mod.rs
  - zircon_editor/src/ui/host/module.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/ui/host/editor_ui_host.rs
  - zircon_editor/src/ui/host/editor_asset_manager/mod.rs
  - zircon_editor/src/ui/host/editor_session_state.rs
  - zircon_editor/src/ui/host/layout_commands.rs
  - zircon_editor/src/ui/host/resource_access.rs
  - zircon_editor/src/ui/host/window_host_manager.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/mod.rs
  - zircon_editor/src/ui/host/ui_asset_promotion.rs
  - zircon_editor/src/ui/asset_editor/mod.rs
  - zircon_editor/src/ui/asset_editor/preview/preview_mock.rs
  - zircon_editor/src/ui/asset_editor/session/mod.rs
  - zircon_editor/src/ui/asset_editor/source/mod.rs
  - zircon_editor/src/ui/slint_host/app/ui_asset_editor.rs
  - zircon_editor/ui/workbench.slint
  - zircon_editor/ui/workbench/host_scaffold.slint
  - zircon_editor/ui/workbench/host_surface.slint
  - zircon_editor/ui/workbench/host_surface_contract.slint
  - zircon_editor/ui/workbench/host_root.slint
plan_sources:
  - user: 2026-04-20 目前zircon_editor有两套ui相关代码 一套在core里面需要迁移回ui
  - user: 2026-04-20 要求加载入口不允许放入src
  - user: 2026-04-20 PLEASE IMPLEMENT THIS PLAN
  - user: 2026-04-20 不要re-export 直接清理core里ui部分
  - .codex/plans/Runtime Core Fold-In And Compile Recovery.md
  - .codex/plans/Zircon UI 资产化 Widget Editor 与共享 Layout.md
  - .codex/plans/编辑器 .slint 去真源 Runtime UI 可用 Cutover 路线图.md
tests:
  - zircon_editor/src/tests/editing/ui_asset_preview_binding_authoring.rs
  - zircon_editor/src/tests/host/manager/mod.rs
  - zircon_editor/src/tests/host/asset_manager_boundary/mod.rs
  - zircon_editor/src/tests/ui/boundary/asset_editor_structure.rs
  - zircon_editor/src/tests/ui/boundary/host_cutover.rs
  - zircon_editor/src/tests/ui/boundary/template_assets.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/bootstrap_assets.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/editor_layouts.rs
  - zircon_editor/src/tests/ui/ui_asset_editor/runtime_previews.rs
  - zircon_editor/tests/workbench_slint_shell.rs
  - cargo test -p zircon_editor --lib editor_asset_boundary_lives_in_editor_crate --locked
  - cargo test -p zircon_editor --lib editor_manager_becomes_thin_facade_over_editor_ui_host --locked
  - cargo test -p zircon_editor --lib editor_module_owner_moves_under_ui_host --locked
  - cargo test -p zircon_editor --lib ui_asset_editor_moves_into_a_folder_backed_ui_subsystem --locked
  - cargo test -p zircon_editor --lib editor_manager_ui_asset_sessions_are_split_by_host_orchestration_behaviors --locked
  - cargo test -p zircon_editor --lib ui_asset_editor_bootstrap_assets_parse_and_compile_with_imports --locked
  - cargo test -p zircon_editor --lib tests::ui::ui_asset_editor --locked --offline --message-format short
  - cargo test -p zircon_editor --lib workbench_slint_entry_stays_on_generic_host_bootstrap_files --locked
  - cargo test -p zircon_editor --locked --offline --test workbench_slint_shell
doc_type: module-detail
---

# UI Asset Editor Host Session

## Purpose

这份文档记录 `zircon_editor` 在本轮 cutover 后的当前真相：

- editor UI 宿主实现统一收口到 `zircon_editor/src/ui`
- `core` 不再拥有 workbench/layout/window/session 的真实实现
- `UI Asset Editor` 相关代码不再分裂在 `core/editing/ui_asset` 和 `core/host/manager/ui_asset_sessions`

本篇重点说明 ownership、会话边界和 Slint 宿主入口约束，而不是重复 shared `.ui.toml` 资产格式本身。资产格式见 [`UI Asset Documents And Editor Protocol`](../ui-and-layout/ui-asset-documents-and-editor-protocol.md)。

## Ownership After Cutover

### `core` 还保留什么

`zircon_editor::core` 现在只保留 editor 内核而不是 UI 实现本体：

- 编辑状态机、intent、history 和 editor-event runtime
- 非 UI 的 editor 资产状态与 command/runtime contract

本轮明确删除了旧的 [`zircon_editor/src/core/host/manager.rs`](../../zircon_editor/src/core/host/manager.rs) owner 角色，而且 `core::host` 整个子树都已经退场。`core` 里不再存在兼容性的 host façade 或模块 owner。

### `ui::host` 现在拥有什么

`zircon_editor/src/ui/host/` 现在是 editor UI 宿主编排的唯一 owner，覆盖：

- `EditorUiHost` 作为统一宿主 owner，真实持有 `CoreHandle`、`ViewRegistry`、`LayoutManager`、`WindowHostManager`、`EditorSessionState` 和 UI asset session 账本
- `EditorManager` 退化为薄 façade，只暴露 editor-facing API 并把状态访问委托给 `EditorUiHost`
- `module.rs` 作为 `EditorModule`、service-name 常量和 `module_descriptor()` 的唯一 owner
- `editor_asset_manager/` 与 `resource_access.rs` 也已经并入 `ui::host`，负责 asset workspace catalog/details/reference/preview sidecar 与宿主资源句柄解析
- builtin view 和 builtin layout 注册
- layout command / layout host / layout persistence
- startup、welcome、recent project、workspace session bookkeeping
- native floating window host 账本
- UI asset session orchestration 与 promotion/workspace sync

这意味着 view registry、layout manager、window host manager、startup/welcome/workspace 持久化都已经从旧 `core::host::manager` 目录下迁回 `ui::host`，而 `EditorManager` 本身不再继续成为这些对象的直接 owner。

### `ui::asset_editor` 现在拥有什么

`zircon_editor/src/ui/asset_editor/` 现在承接 UI Asset Editor 自身的领域实现，覆盖：

- route / reflection / window descriptor contract
- source buffer 与 canonical save
- session state、preview compile、presentation
- binding/style/tree edit authoring
- undo、document replay、external effect replay
- promotion draft 与 preview host

原先分散在 `core/editing/ui_asset/*` 的逻辑现在按 `binding/preview/session/source/style/tree` 子树收进同一 UI 域，避免“编辑器 UI 领域逻辑在 core 里继续长大”。

## No Core Re-export Shim

本轮刻意没有采用“`core -> ui` 兼容 re-export”做过渡。

当前链路是直接改 owner：

- [`zircon_editor/src/ui/host/module.rs`](../../zircon_editor/src/ui/host/module.rs) 直接实例化 `crate::ui::host::EditorManager` 并持有 `EditorModule` wiring
- [`zircon_editor/src/lib.rs`](../../zircon_editor/src/lib.rs) 的公开 editor host 类型直接从 `ui::host::module` 导出
- [`zircon_editor/src/core/mod.rs`](../../zircon_editor/src/core/mod.rs) 已不再声明 `host` 子树，`core::host` 目录也已删除

这样做的效果是，后续再清理 `core` 时不会被一层历史兼容命名绑住，也不会让调用方误以为 `core` 仍然是 UI owner。

## Session And Host Split

当前 UI asset editor 的职责边界固定成两层：

- `ui::host::asset_editor_sessions`
  - 负责打开/保存、asset hydration、project/workspace 同步、host-level orchestration
  - 负责把 Slint callback 或 workbench action 路由成稳定 session 调用
- `ui::asset_editor::UiAssetEditorSession`
  - 负责 source/document/preview 三角同步
  - 负责 selection、tree edit、binding/style authoring、undo/replay
  - 负责 canonical TOML 输出和 last-good preview 语义

也就是说，host 层只保留“会话编排”和“工作台整合”；真正的 UI asset authoring 行为已经回到 `ui::asset_editor` 域内，而不是继续夹在 `core` 和 `ui` 之间。

## Slint Entry Boundary

`.slint` cutover 在 editor 侧先冻结了入口边界：

- [`workbench.slint`](../../zircon_editor/ui/workbench.slint)
- [`host_scaffold.slint`](../../zircon_editor/ui/workbench/host_scaffold.slint)
- [`host_surface.slint`](../../zircon_editor/ui/workbench/host_surface.slint)
- [`host_surface_contract.slint`](../../zircon_editor/ui/workbench/host_surface_contract.slint)
- [`host_root.slint`](../../zircon_editor/ui/workbench/host_root.slint)

这些 bootstrap 文件现在只能保留 generic host window / scaffold / surface 职责。边界测试明确禁止它们重新 import `assets.slint`、`panes.slint`、`welcome.slint` 这类业务壳文件。

当前这层 generic bootstrap 还有一个固定合同：

- `host_root.slint` 里的 `HostWindowPresentationData` 统一分组 `host_shell`、`host_layout`、`workbench_scene_data`、`native_floating_surface_data`
- `host_scaffold.slint` 只接收整份 `host_presentation`，不再把 surface/layout/native payload 扇出成一组松散属性
- `host_surface_contract.slint` 只负责从 `host_presentation` 投影出 `workbench_scene_data` 和 `native_floating_surface_data`
- `host_surface.slint` 只消费 contract 输出，再把 scene/native floating surface 分流到真正的 host scene surface

这一步的目标不是一次性删除所有业务 `.slint` 文件，而是先钉死“入口层不能再成为业务真源”。更深层的 pane catalog 和残余业务 Slint 仍是后续 slice 的清理对象。

## Tree-Native Session Helpers

当前 `UI Asset Editor` 的生产代码已经不再把 `document.nodes` 当成作者态真源。

这一轮实际落地的是：

- `UiAssetEditorSession`、tree edit、undo/replay、style inspection、source sync、promotion 和 preview projection 都直接走 `UiAssetDocument` 的递归 helper
- 典型访问路径已经统一成 `contains_node`、`node`、`node_mut`、`iter_nodes`、`parent_of`、`child_index_in_parent`、`replace_node`、`remove_node`、`insert_child`、`push_child`、`swap_children`
- component root 也不再通过旧的根节点字符串索引消费，而是直接把内嵌树根当成正式节点数据处理
- preview mock subject 的默认回退现在按 UI 实际展示顺序选首项，不再因为树遍历顺序和 subject 列表排序不同而出现“初始选中项错位”

剩余 legacy 兼容只留在 runtime 模板层的 `#[cfg(test)]` 迁移 helper，以及 editor 自己的 `src/tests/support.rs` 夹具迁移 helper；production editor authoring path 只接受 tree authority，它已经不再是 editor authoring session 的内部工作模型。

## Acceptance Evidence

本轮与 ownership 收口直接对应的验证有几条：

- `cargo test -p zircon_editor --lib ui_asset_editor_moves_into_a_folder_backed_ui_subsystem --locked`
  - 证明 UI asset editor 已经物理迁入 `src/ui/asset_editor`
- `cargo test -p zircon_editor --lib editor_manager_becomes_thin_facade_over_editor_ui_host --locked`
  - 证明 `EditorManager` 已退化为统一 `EditorUiHost` 的薄 façade，不再直接持有 host/layout/view/window/session 状态
- `cargo test -p zircon_editor --lib editor_module_owner_moves_under_ui_host --locked`
  - 证明 `EditorModule` / `module_descriptor()` owner 已迁入 `ui::host::module`，crate root 不再从 `core::host::module` 导出
- `cargo test -p zircon_editor --lib editor_asset_boundary_lives_in_editor_crate --locked`
  - 证明 editor asset manager 与 resource access 宿主服务已经迁入 `ui::host`，`core::host` 子树已删除
- `cargo test -p zircon_editor --lib editor_manager_ui_asset_sessions_are_split_by_host_orchestration_behaviors --locked`
  - 证明 `EditorManager` 和 `ui::host::asset_editor_sessions` 的职责边界已经稳定
- `cargo test -p zircon_editor --lib ui_asset_editor_bootstrap_assets_parse_and_compile_with_imports --locked`
  - 证明 editor bootstrap 资产仍能经 shared loader/compiler 打开
- `cargo test -p zircon_editor --lib workbench_slint_entry_stays_on_generic_host_bootstrap_files --locked`
  - 证明 `workbench.slint` 入口不再倒回业务壳 import
- `cargo test -p zircon_editor --locked --offline --test workbench_slint_shell`
  - 证明 bootstrap Slint 合同已经稳定收敛到 `HostWindowPresentationData -> HostWorkbenchWindowSurfaceContract -> scene/native split`，不会回退到旧的散装 surface passthrough

这四条测试组合起来，覆盖了“代码物理位置”“owner 边界”“shared 资产链路”和“Slint 入口约束”四个最关键的验收面。
