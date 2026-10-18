# Bevy/Fyrox 取向的 UI 原型缓存重构方案

## Summary
- 当前问题不是简单缓存不足，而是架构仍在反复把 `.ui.toml` 反序列化成递归树，再递归展开组件。`project_overview.ui.toml` 这类深层 `[root.children.node...]` 结构已经能触发栈溢出。
- 参考 Bevy 的方向：组件行为无样式、状态外置、系统按脏域更新。依据：[bevy_ui_widgets/lib.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_ui_widgets/src/lib.rs:1)、[bevy_ui_widgets/button.rs](E:/Git/ZirconEngine/dev/bevy/crates/bevy_ui_widgets/src/button.rs:1)。
- 参考 Fyrox 的方向：持久 UI 图、稳定 Handle、BuildContext 只负责构建，运行时通过消息/脏标记更新。依据：[fyrox-ui/lib.rs](E:/Git/ZirconEngine/dev/Fyrox/fyrox-ui/src/lib.rs:1)、[fyrox-ui/build.rs](E:/Git/ZirconEngine/dev/Fyrox/fyrox-ui/src/build.rs:1)。
- 目标方案：`.ui.toml` 只在加载阶段解析一次，组件按需编译成堆上的 `Arc<UiComponentPrototype>`，运行时只通过 handle/arena 取用，不再递归解析或递归展开组件。

## Key Changes
- 把现有嵌套 tree schema 硬切到 flat prototype schema：`[root] node = "..."`、`[nodes.<id>]`、`[components.<name>] root = "..."`。核心编辑器和运行时 UI 资产全部迁移，编辑器宿主不再接受深层 `children.node` 作为热路径输入。
- 新增 runtime 原型层：`UiPrototypeStore`、`UiRawAssetPrototype`、`UiDocumentPrototype`、`UiComponentPrototype`、`UiNodePrototype`、`UiStylePrototype`。Store 按 asset fingerprint/import graph 管理生命周期，组件第一次被引用时才 materialize 到堆缓存。
- 替换 `UiAssetDocument -> UiDocumentCompiler::expand_node/expand_component_instance` 这条递归路径。新的实例化器使用显式工作栈：遇到 component/reference 时取 `UiComponentPrototypeHandle`，叠加 params/slot overlay，再把原型节点切片压回工作栈。
- 按 Bevy 思路拆分节点数据：layout/style/text/input/a11y/widget contract 分开存储和标脏；组件定义只表达行为与结构，不携带一次性展开后的完整 UI 树。
- 按 Fyrox 思路保留长生命周期 `UiInstanceGraph`/`UiSurface`：`UiBuildContext` 只构建或补节点，运行时更新走 `UiMessage`/patch/dirty flags，不再每帧或每次 pane projection 重建整棵树。

## Implementation Changes
- `zircon_runtime::ui::template` 成为原型加载、组件缓存、实例化的所有者；`zircon_runtime_interface` 只暴露稳定 ID、handle、prototype DTO 和错误类型。
- `zircon_editor::ui::template_runtime` 持有单个共享 `Arc<UiPrototypeStore>`；`EditorUiHostRuntime::default()` 和 `load_builtin_host_templates()` 不再在多个 bridge/pane 内重复创建。
- `build_view_template_nodes_with_imports` 改为从共享 store 请求 projection surface。`project_overview_pane_data` 只提交 size 和 text overrides，不再读文件、不再反序列化、不再编译 import map。
- 现有 `flat_nodes::migrate_flat_toml_str` 不能继续 materialize 成递归 `UiAssetDocument`；它要改成直接生成 flat prototype arena。旧 tree migrator 仅用于离线迁移或测试，不进入编辑器运行路径。
- Fyrox 风格 UI 视觉调整放在原型缓存稳定之后统一做：深色面板、细边框、密集 dock/pane 工具条、低对比状态色、清晰焦点/hover/pressed 状态，全部通过共享 style prototype 驱动。

## Test Plan
- 栈安全：用 `project_overview.ui.toml`、`asset_browser.ui.toml`、`ui_asset_editor.ui.toml` 和一个 10k 节点 flat fixture 验证加载/实例化不递归溢出。
- 缓存行为：同一 pane 连续 projection 1000 次，断言 `toml::from_str`/`UiAssetLoader::load_toml_file` 不在热路径出现，组件 prototype 只 materialize 一次。
- 组件语义：params 默认值、slot required/multiple、reference import、style token、text override 都通过 prototype handle 正确应用。
- 失效更新：修改 style/widget/layout asset 时只 evict 受影响 prototype，已存在 surface 通过 dirty flags/patch 更新，不整树重建。
- 编辑器回归：保留并扩展 retained host、template bridge、project overview、component showcase、runtime UI golden 测试；启动日志中 builtin template 注册应从多次重复降为一次共享加载。

## Assumptions
- 这是硬切方案：不保留递归组件展开作为兼容热路径。
- 不引入 Bevy/Fyrox 依赖，只吸收它们的数据流和 retained UI 架构思想。
- 旧嵌套 `.ui.toml` 可以批量迁移；资产编辑器若需要打开旧格式，也必须先走离线迁移到 flat prototype。
