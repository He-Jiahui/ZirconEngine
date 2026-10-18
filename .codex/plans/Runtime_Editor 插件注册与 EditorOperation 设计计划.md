# Runtime/Editor 插件注册与 EditorOperation 设计计划

## Summary
- v1 采用你选择的“双层插件”方案：Rust `runtime/editor` crate 负责强类型注册、构建和导出；VM 插件通过稳定 host handle 调用同一套注册和操作接口。
- Runtime 插件可注册 Manager、Component 类型、UI Component 类型和其它功能扩展点；Editor 插件可注册窗口/View/Drawer、菜单路径、组件编辑 UI、`ui.toml` 模板和事件绑定。
- Editor 操作统一抽象为命名的 `EditorOperation`，可撤销操作单独建模为 `UndoableEditorOperation`，所有 UI、菜单、远程和 CLI 触发都走同一操作派发与 journal/undo stack。

## Public APIs / Interfaces
- Runtime 新增 `RuntimeExtensionRegistry`：承接 `ManagerDescriptor`、`ComponentTypeDescriptor`、`UiComponentDescriptor`、系统阶段 hook、资源/导入器等扩展注册；入口接在现有 `CoreRuntime`/`ModuleDescriptor` 生命周期下。
- Scene 新增 `ComponentTypeRegistry` 与动态组件存储：内建组件继续保留现有高性能 HashMap，插件组件以 `PluginId.Component.TypeName` 注册，并可通过同一 Entity 附加、查询、序列化、属性编辑。
- Editor 新增 `EditorExtensionRegistry`：注册 `ViewDescriptor`、`DrawerDescriptor`、`EditorMenuItemDescriptor`、`ComponentDrawerDescriptor`、`EditorUiTemplateDescriptor` 和 `EditorOperationDescriptor`。
- Editor 操作命名统一为 `XXX.YYY.ZZZ`，菜单路径可用 Unity 风格 `Tools/My Plugin/Action`，最终绑定到操作名；参考 Unity `MenuItem` 的路径、优先级和验证函数概念，但不使用 IMGUI。
- `ui.toml` 组件编辑界面通过 `ComponentDrawerDescriptor { component_type, ui_document, controller, bindings }` 注册；事件绑定进入 `EditorOperationInvocation`，控制脚本可以是 Rust editor crate handler 或 VM handler。
- 外部控制新增两条入口：进程内 `EditorOperationControlRequest::InvokeOperation`，以及 `zircon_app` 编辑器命令行 `--operation <id> --args <json> --headless`；返回结构化成功/错误并写入同一 journal。

## Implementation Changes
- 在 [zircon_runtime/src/plugin](E:/Git/ZirconEngine/zircon_runtime/src/plugin) 扩展插件 manifest/contribution 描述，让项目启用的插件声明 runtime/editor crate、能力、组件、UI 组件和导出策略。
- 在 [zircon_runtime/src/scene](E:/Git/ZirconEngine/zircon_runtime/src/scene) 增加组件类型注册层，避免插件组件变成 `World` 的特殊分支；插件卸载时若仍有实体持有该插件组件，默认拒绝卸载，后续热替换可走状态迁移。
- 在 [zircon_editor/src/core/editor_event](E:/Git/ZirconEngine/zircon_editor/src/core/editor_event) 将现有 enum 派发收束为 registry-backed operation 派发；现有创建节点、删除、Inspector、Viewport、Layout、Asset 等内建行为先作为 built-in operations 注册。
- `EditorOperationStack` 替代当前分散的 scene `EditorHistory` 作为统一 undo/redo 栈；拖拽等连续操作按 operation group 合并，journal 保存 source、operation id、display name、effects、revision、result。
- Workbench 菜单构建改为“内建菜单 + 插件菜单贡献”合并，插件菜单项支持路径、排序、快捷键、启用校验、required capabilities，并映射到 `EditorOperation`。
- Editor UI 反射与 `EditorUiControlService` 继续作为远程可调用基础；`CallAction`、`InvokeRoute`、`InvokeBinding` 最终都规范化成 operation invocation。

## Test Plan
- Runtime：插件注册 manager/component/UI component 成功；重复 component id 报错；插件组件能附加到普通 entity 并与内建组件共同序列化、属性读写、查询。
- Lifecycle：插件 manager 依赖声明走 Core；Lazy/Immediate 行为正确；有活跃插件组件时卸载被拒绝；无活跃数据时可卸载。
- Editor：插件注册 View/Drawer/菜单后出现在 workbench；禁用 capability 后不可见且不可打开；重复 view/menu/operation id 有明确诊断。
- Operation：菜单、`ui.toml` 控件、远程 request、CLI 调用同一 operation；undo/redo 栈记录名称；失败操作写 journal 但不污染 undo 栈。
- Component Drawer：插件组件的 `ui.toml` Inspector 能渲染，字段变更提交为 `UndoableEditorOperation`，撤销后 ECS 数据恢复。
- Export：`ExportBuildPlan` 只链接启用的 runtime 插件 crate，editor 插件不进入 runtime 发行包。
- 验证命令：`cargo test -p zircon_runtime`、`cargo test -p zircon_editor`、`cargo test -p zircon_app`，最后跑 workspace check/test。

## Assumptions
- v1 不做纯 Rust 动态库对象共享；native dynamic 仍只是发行/加载后端之一。
- VM 插件先通过 host handle 调用注册和 operation 能力，不要求本轮实现完整真实 VM 后端。
- Unity 参考点：[`MenuItem`](https://docs.unity3d.com/cn/2022.3/ScriptReference/MenuItem.html)、[`UI Toolkit`](https://docs.unity3d.com/2023.2/Documentation/Manual/UIElements.html)、[`Undo`](https://docs.unity3d.com/cn/2017.3/ScriptReference/Undo.html)、[`-executeMethod`](https://docs.unity3d.com/cn/2021.2/Manual/EditorCommandLineArguments.html)。
