# Zircon Editor UI 回迁 + 树形 TOML Cutover 实施计划

## Summary
- 第一阶段先把 `zircon_editor` 里分裂在 `core` 与 `ui` 的 UI 子系统合并回 [zircon_editor/src/ui](/E:/Git/ZirconEngine/zircon_editor/src/ui)。`core` 只保留编辑状态机、intent/history、editor-event runtime 和薄委托，不再承载 UI 实现。
- 第二阶段把共享 UI 资产真源从当前平面 `root + nodes` TOML 改成递归树形 TOML，并一次性迁移所有受管 `.ui.toml` 资源与加载器。
- 硬约束新增一条：所有生产用“加载入口资源文件”不得放在 `src/` 下。它们只能放在 crate 级 `assets/` 或项目 `assets/`。当前违规入口 [zircon_runtime/src/ui/runtime_ui/fixtures](/E:/Git/ZirconEngine/zircon_runtime/src/ui/runtime_ui/fixtures) 必须在 cutover 前迁出。

## Key Changes
- **UI ownership 收口到 `ui/`。** 在 [zircon_editor/src/ui](/E:/Git/ZirconEngine/zircon_editor/src/ui) 下新增统一 `host/` 子树，承接当前 `EditorManager` 直接持有的 `ViewRegistry`、`LayoutManager`、`WindowHostManager`、UI asset session bookkeeping、builtin view/layout 注册、startup/welcome/workspace/layout persistence。`EditorManager` 改成仅持有一个统一 `EditorUiHost` 并向下薄委托。
- **UI Asset Editor 整体回迁。** 把 [zircon_editor/src/core/editing/ui_asset](/E:/Git/ZirconEngine/zircon_editor/src/core/editing/ui_asset) 与 [zircon_editor/src/core/host/manager/ui_asset_sessions](/E:/Git/ZirconEngine/zircon_editor/src/core/host/manager/ui_asset_sessions) 一起收进新的 `ui/asset_editor/` 子树；`route`、`window descriptor`、`reflection DTO`、`session`、`preview`、`source`、`undo`、`style`、`tree edit`、`promotion/workspace sync` 同域放置，不再跨 `core/ui` 双持有。
- **Workbench/UI 宿主编排回迁。** 当前 `core/host/manager` 里的 `builtin_views`、`builtin_layout`、`layout_*`、`window_host_manager`、`editor_session_state`、`startup` 等 UI 宿主编排逻辑迁入 `ui/host/` 或 `ui/workbench/` 对应子树；`core` 不再成为 workbench/window/layout 的真实 owner。
- **树形 TOML 成为唯一权威。** 重写 [zircon_runtime/src/ui/template/asset/document.rs](/E:/Git/ZirconEngine/zircon_runtime/src/ui/template/asset/document.rs) 的资产模型：
  - 删除顶层 `UiAssetRoot + nodes: BTreeMap<String, UiNodeDefinition>` 权威形态。
  - 改为 `root: Option<UiNodeDefinition>` 的递归树。
  - 每个节点显式带稳定 `node_id`。
  - 子节点通过递归 `children` 内嵌，`mount/slot` 元数据跟随子边保存。
  - `UiComponentDefinition.root` 改成内嵌子树，不再通过根节点 id 引用。
- **一次性资产迁移，不保留长期双解析。** 新增临时迁移器读取旧平面资产并输出新树形 TOML，覆盖当前受管资产：
  - [zircon_editor/assets/ui](/E:/Git/ZirconEngine/zircon_editor/assets/ui) 下 editor/runtime/theme 入口资源。
  - [zircon_runtime/src/ui/runtime_ui/fixtures](/E:/Git/ZirconEngine/zircon_runtime/src/ui/runtime_ui/fixtures) 下 runtime fixture 资源。
  迁移器只用于本次分支改写；落地后删除旧平面 authority 解析路径。
- **加载入口资源文件迁出 `src/`。** 运行时 builtin fixture 入口资源从 [zircon_runtime/src/ui/runtime_ui/fixtures](/E:/Git/ZirconEngine/zircon_runtime/src/ui/runtime_ui/fixtures) 迁到新的 [zircon_runtime/assets/ui/runtime](/E:/Git/ZirconEngine/zircon_runtime) 子树，默认目录定为 `zircon_runtime/assets/ui/runtime/fixtures/`。后续所有生产入口 `.ui.toml` 一律遵守同规则。
- **runtime fixture 加载方式同步改造。** [runtime_ui_fixture.rs](/E:/Git/ZirconEngine/zircon_runtime/src/ui/runtime_ui/runtime_ui_fixture.rs) 删除 `source()` 这种从 `src/fixtures` 取字符串的入口，保留 `asset_id()`，新增固定相对资源路径解析；[runtime_ui_manager.rs](/E:/Git/ZirconEngine/zircon_runtime/src/ui/runtime_ui/runtime_ui_manager.rs) 改为从 crate `assets/` 路径读取树形 `.ui.toml` 文件，再走共享 `UiAssetLoader` / `UiDocumentCompiler`。
- **Slint cutover 收口。** [zircon_editor/ui/workbench](/E:/Git/ZirconEngine/zircon_editor/ui/workbench) 下 `.slint` 文件最终只保留 generic `UiHostWindow` bootstrap 与宿主 catalog，不再承载 workbench/menu/drawer/document/floating 业务树与布局公式；业务结构全部来自树形 TOML -> compiler -> projection。

## Public APIs / Interfaces
- `zircon_editor::core::host::manager::EditorManager` 变为薄 facade，只委托统一 `ui::host::EditorUiHost`。
- `zircon_editor::ui` 新增明确的 `host/` 与 `asset_editor/` 目录边界，原单文件 `ui_asset_editor.rs` 退化为结构性入口或被目录化替代。
- `zircon_runtime::ui::template::asset::UiAssetDocument`、`UiNodeDefinition`、`UiChildMount`、`UiComponentDefinition` 全部改为递归树模型。
- `UiAssetLoader` 最终只接受树形 `.ui.toml` 权威格式；旧平面格式只存在于一次性迁移器中，不保留正式入口。
- `RuntimeUiFixture` 删除 `source()`；builtin runtime fixture 的正式入口变成 crate `assets/ui/runtime/fixtures/*.ui.toml` 文件路径，而不是 `src` 下 `include_str!` 资源。

## Test Plan
- **边界测试：** 证明 `core` 不再直接拥有 UI host/layout/view/window/session 实现；现有针对 `ui_asset_sessions`、view/layout host orchestration 的测试改到新 `ui/host` / `ui/asset_editor` 边界。
- **树形资产测试：** 覆盖 parse/serialize/roundtrip、稳定 `node_id`、子顺序、mount/slot、component root、stylesheet、reflection/undo/replay 所需路径稳定性。
- **迁移测试：** 用旧平面 `.ui.toml` 样本验证迁移后语义不变，特别是 editor widgets、UI asset editor、自带 runtime HUD/pause/settings/inventory fixture。
- **资源位置测试：** 新增守卫，禁止生产 `.ui.toml` 入口资源继续出现在任意 crate 的 `src/` 下；运行时 fixture 必须从 crate `assets/` 成功加载。
- **Slint cutover 测试：** 继续保留 parity/snapshot/route 测试，但新增源码守卫，确保 `.slint` 不再持有 workbench 业务真源。

## Assumptions
- “按现在目录结构标准实施”解释为：生产资源文件走 crate 根 `assets/` 目录，代码走 `src/`，两者不混放。
- 当前 editor 资产目录 [zircon_editor/assets/ui](/E:/Git/ZirconEngine/zircon_editor/assets/ui) 已符合标准；runtime 需要补齐同类 `assets/ui/...` 目录，而不是继续把入口资源留在 `src/ui/.../fixtures`。
- 测试中的内联 TOML 字符串可以继续存在于测试代码；被正式加载的入口资源文件不可以。
- 树形 TOML 仍保留显式 `node_id`，不会改成纯位置路径身份模型。
