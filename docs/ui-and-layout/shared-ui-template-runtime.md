---
related_code:
  - zircon_ui/src/lib.rs
  - zircon_ui/src/layout/constraints.rs
  - zircon_ui/src/layout/geometry.rs
  - zircon_ui/src/layout/pass/mod.rs
  - zircon_ui/src/layout/scroll.rs
  - zircon_ui/src/template/mod.rs
  - zircon_ui/src/template/bridge/mod.rs
  - zircon_ui/src/template/document.rs
  - zircon_ui/src/template/loader.rs
  - zircon_ui/src/template/validate.rs
  - zircon_ui/src/template/instance.rs
  - zircon_ui/src/tree/mod.rs
  - zircon_ui/src/tree/node/mod.rs
  - zircon_ui/src/surface/mod.rs
  - zircon_ui/src/tests/template.rs
  - zircon_ui/src/tests/asset.rs
  - zircon_editor/src/ui/template/mod.rs
  - zircon_editor/src/ui/template/registry.rs
  - zircon_editor/src/ui/template_runtime/mod.rs
  - zircon_editor/src/ui/template_runtime/builtin/mod.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/src/ui/template_runtime/builtin/component_descriptors.rs
  - zircon_editor/src/ui/template_runtime/runtime/build_session.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
  - zircon_editor/ui/templates/workbench_shell.toml
implementation_files:
  - zircon_ui/src/lib.rs
  - zircon_ui/src/layout/constraints.rs
  - zircon_ui/src/layout/geometry.rs
  - zircon_ui/src/layout/pass/mod.rs
  - zircon_ui/src/layout/scroll.rs
  - zircon_ui/src/template/mod.rs
  - zircon_ui/src/template/bridge/mod.rs
  - zircon_ui/src/template/document.rs
  - zircon_ui/src/template/loader.rs
  - zircon_ui/src/template/validate.rs
  - zircon_ui/src/template/instance.rs
  - zircon_ui/src/tree/mod.rs
  - zircon_ui/src/tree/node/mod.rs
  - zircon_ui/src/surface/mod.rs
  - zircon_editor/src/ui/template_runtime/mod.rs
  - zircon_editor/src/ui/template_runtime/builtin/mod.rs
  - zircon_editor/src/ui/template_runtime/builtin/template_documents.rs
  - zircon_editor/src/ui/template_runtime/builtin/component_descriptors.rs
  - zircon_editor/src/ui/template_runtime/runtime/build_session.rs
  - zircon_editor/src/ui/template_runtime/runtime/runtime_host.rs
plan_sources:
  - user: 2026-04-15 按自定义 TOML 描述文件运行时构建 Slint 树并严格服从 Shared Layout 契约
  - user: 2026-04-15 PLEASE IMPLEMENT THIS PLAN
  - user: 2026-04-18 继续下一步，推进 Runtime visual contract
  - .codex/plans/布局系统.md
  - .codex/plans/Zircon 运行时编辑器共享 UI 布局与事件系统架构计划.md
tests:
  - zircon_ui/src/tests/template.rs
  - zircon_ui/src/tests/asset.rs
  - cargo test -p zircon_ui template -- --nocapture
  - cargo test -p zircon_ui --lib --locked ui_document_compiler_expands_imported_widget_references_and_applies_stylesheets -- --nocapture
  - cargo test -p zircon_ui --offline --verbose
doc_type: module-detail
---

# Shared UI Template Runtime

## Purpose

这一层把“自定义描述文件 -> shared UI 权威模型”的第一段落地在 `zircon_ui`，但刻意只做到模板文档、校验和运行时实例展开，不在这里混入 editor docking、Slint callback 或宿主专属行为。

在这轮模块边界重构后，`template/bridge/*` 已经拆成 builder、layout contract、parser 和轻量推断几个行为族；`template/bridge/mod.rs` 现在只做导出层，不再长期容纳所有桥接逻辑。

当前权威链路已经固定成：

1. TOML 文档解析成 `UiTemplateDocument`
2. `zircon_ui::template::UiTemplateValidator` 约束模板节点、slot 和组件引用
3. `zircon_ui::template::UiTemplateInstance` 展开 composite/template 调用和 slot 内容
4. `zircon_ui::template::UiTemplateTreeBuilder` 把模板里的显式 layout 契约映射到 shared `UiTree` 节点
5. `zircon_ui::template::UiTemplateSurfaceBuilder` 把实例树投影到 shared `UiSurface`
6. `UiSurface::compute_layout(...)` 按 shared measure/arrange 契约求 frame / clip / scroll window
7. editor/runtime adapter 再继续把 shared tree、shared surface 或宿主节点模型投影到宿主层

也就是说，这一层现在已经负责“模板语义真源 + shared tree 首段落点 + 显式 layout 合同落点”，但仍然不负责文本测量服务、样式表达式求值或 Slint 渲染绑定。真正的布局、命中、焦点和 route 权威仍然在 `UiTree` / `UiSurface` / shared layout contract。

## Public Model

### `UiTemplateDocument`

- `version`
- `components: BTreeMap<String, zircon_ui::template::UiComponentTemplate>`
- `root: zircon_ui::template::UiTemplateNode`

文档拥有一个真正的入口 root，以及一组可被重复装配的命名 component template。

### `zircon_ui::template::UiComponentTemplate`

- `root: zircon_ui::template::UiTemplateNode`
- `slots: BTreeMap<String, zircon_ui::template::UiSlotTemplate>`

component template 是“复合组件装配层”的最小权威单元。它不直接描述最终像素 frame，只描述宿主树和 shared tree 应该如何拼装。

### `zircon_ui::template::UiTemplateNode`

当前节点固定只有三种互斥形态：

- `component`
  - 表示一个真实宿主/共享组件节点
- `template`
  - 表示对命名 `zircon_ui::template::UiComponentTemplate` 的调用
- `slot`
  - 表示 template 内部的插槽占位点

额外携带的通用字段包括：

- `control_id`
- `bindings`
- `children`
- `slots`
- `attributes`
- `style_tokens`

这里的 `bindings` 目前不是宿主 callback 名称，而是稳定的 `zircon_ui::template::UiBindingRef`：

- `id`
- `event`
- `route`

`id` 用来承载诸如 `WorkbenchMenuBar/SaveProject` 这类稳定命名空间；`route` 只是稳定 route key，不是桌面宿主私有函数名。

`UiComponentTemplate` / `UiSlotTemplate` / `UiBindingRef` / `UiActionRef` 现在都统一经 `zircon_ui::template::*` 暴露，`zircon_ui` crate root 不再继续平铺这组 template document model。

## TOML Shape

当前实现支持的最小 TOML 形态如下：

```toml
version = 1

[root]
template = "WorkbenchShell"
slots = { menu_bar = [{ template = "MenuBar" }] }

[components.WorkbenchShell]
slots = { menu_bar = { required = true } }
root = { component = "WorkbenchShell", children = [{ slot = "menu_bar" }] }

[components.MenuBar]
root = { component = "UiHostToolbar", children = [
  { component = "UiHostIconButton", control_id = "SaveProject", bindings = [
    { id = "WorkbenchMenuBar/SaveProject", event = "Click", route = "MenuAction.SaveProject" }
  ] }
] }
```

这个结构已经满足第一阶段目标：

- component template 可以嵌套 component template
- slot 内容由调用点提供
- binding 引用保留稳定命名空间
- 运行时实例展开后不会丢掉这些 binding ref

仓库里第一份真实模板资产现在已经放在 [workbench_shell.toml](/E:/Git/ZirconEngine/zircon_editor/ui/templates/workbench_shell.toml)。它先覆盖 workbench shell 的复合装配骨架：

- `WorkbenchShell`
- `MenuBar`
- `ActivityRail`
- `DocumentHost`
- `StatusBar`

## Layout Contract By Attribute

当前模板文档不会额外引入第二套 layout 节点类型，而是固定通过 `attributes.layout` 把 shared contract 写进模板节点。

已落地的字段包括：

- `width` / `height`
  - 对应 shared `AxisConstraint { min, max, preferred, priority, weight, stretch }`
- `anchor` / `pivot` / `position`
  - 直接映射到 shared `Anchor` / `Pivot` / `Position`
- `boundary`
  - 对应 `LayoutBoundary::{ContentDriven, ParentDirected, Fixed}`
- `clip` / `clip_to_bounds`
  - 控制节点 clip 链入口
- `z_index`
  - 控制 shared draw order 的层级偏置
- `input_policy`
  - 支持 `Inherit` / `Receive` / `Ignore`
- `container`
  - 显式声明 shared 容器语义，而不是强依赖 component 名字

`container` 目前支持：

- `Container`
- `Overlay`
- `Space`
- `HorizontalBox { gap }`
- `VerticalBox { gap }`
- `ScrollableBox { axis, gap, scrollbar_visibility, virtualization }`

这一步的关键点是：editor-only component 名字不再需要和 shared primitive 名字一一重合。像 `WorkbenchShell`、`DocumentHost`、`ActivityRail` 这样的 composite，可以继续保留自己的宿主身份，但它们的 shared layout 行为已经由 `attributes.layout.container` 显式给出。

## Validation Rules

`UiTemplateValidator` 当前已经把以下约束钉死：

- 每个节点必须且只能声明 `component` / `template` / `slot` 其中一种
- `template` 调用必须引用已注册的 component template
- required slot 必须由调用点提供
- 不允许给单值 slot 塞多个子节点
- template 内部出现的 slot placeholder 必须先在 `slots` 中声明
- slot placeholder 不能再额外携带 bindings、children、slot fills 或 control id
- template 调用不允许直接再挂 `children`，slot 才是唯一的复合内容注入口

这一步的意义是避免 editor host 或后续 Slint projection 再去容忍一堆“能跑但不清晰”的隐式模板结构。

## Instance Expansion

`zircon_ui::template::UiTemplateInstance::from_document(...)` 当前会：

- 先跑完整 `UiTemplateValidator`
- 再把 `template` 调用展开成真实 component 子树
- 再把 slot placeholder 替换成调用点提供的内容
- 最终得到一个已经没有 `template`/`slot` 占位歧义的运行时模板实例树

目前实例层还提供 `binding_refs()`，按树遍历顺序收集稳定 binding 引用。这正是后续 editor/runtime adapter 把模板树映射成 typed command/binding、再投影给 Slint host 的入口。

## Shared Tree Bridge

这一轮新增了 shared-core 桥接器：

- `UiTemplateTreeBuilder`
- `UiTemplateSurfaceBuilder`
- `UiTemplateBuildError`
- `zircon_ui::tree::UiTemplateNodeMetadata`

### `zircon_ui::tree::UiTemplateNodeMetadata`

`UiTreeNode` 现在可以携带模板元数据快照，用来保留后续 shared core 和宿主投影都会需要的稳定信息：

- `component`
- `control_id`
- `attributes`
- `style_tokens`
- `bindings`

这一步很关键，因为之前如果直接把模板实例铺进 `UiTree`，会丢掉：

- 稳定 binding id
- 宿主 icon / label 等属性
- style token
- component/control identity

那样后续再想从 shared tree 做 route 或宿主投影，就必须重新回头读模板实例，等于 shared tree 不是真正的中继层。

### `UiTemplateTreeBuilder`

当前 builder 会把 `zircon_ui::template::UiTemplateInstance` 转成 `UiTree`，并做两类 shared-core 推断：

- 节点按 preorder 分配稳定 `UiNodeId`
- `UiNodePath` 使用 control/component 名称加顺序索引生成可读路径
- 模板元数据挂入每个 `UiTreeNode`
- 显式 `attributes.layout` 映射到 shared `BoxConstraints` / `Anchor` / `Pivot` / `Position` / `LayoutBoundary` / `UiInputPolicy` / `z_index`
- `attributes.layout.container` 优先映射到 `UiContainerKind`
- 当模板没有显式 `container` 时，再退回到已知共享容器名映射
- 可交互节点根据 bindings / 已知交互 primitive 推断 `clickable` / `hoverable` / `focusable`
- 带 bindings 的节点默认设置 `UiInputPolicy::Receive`
- `ScrollableBox` 自动初始化 `UiScrollState::default()` 并开启 `clip_to_bounds`

当前 layout contract 采用“显式字段优先、组件名仅作回退”的规则。也就是说：

- 如果模板写了 `attributes.layout.container.kind = "VerticalBox"`，shared tree 就直接按 `VerticalBox` 处理
- 如果模板没写 layout 容器，但 component 名字本身就是 `HorizontalBox` / `ScrollableBox` 这类 shared primitive，builder 仍然会做兼容映射
- 如果两者都没有，节点保持 `UiContainerKind::Free`

对于 layout 字段值，builder 现在会在 bridge 阶段做基本结构校验；不支持的 enum 值或错误的 table 形态会直接返回 `UiTemplateBuildError::InvalidLayoutContract`，避免把畸形模板延后到 layout pass 或宿主投影时才暴露。

当前已知容器映射只覆盖 shared primitive 名称：

- `Container`
- `Overlay`
- `Space`
- `HorizontalBox`
- `VerticalBox`
- `ScrollableBox`

未知 component 目前不会被强行解释成布局容器，而是保留 `UiContainerKind::Free`。

### `UiTemplateSurfaceBuilder`

`UiTemplateSurfaceBuilder` 只是 `UiTemplateTreeBuilder` 的轻封装：

- 先构建 `UiTree`
- 再放入 `UiSurface`
- 最后调用 `rebuild()` 生成 hit-test index 和初始 `UiRenderExtract`

这让 shared template runtime 现在已经具备了“模板实例 -> shared retained surface -> shared layout 求 frame -> shared visual draw list”的最低闭环，而不是停留在纯文档/纯实例层。

当前 `rebuild()` 输出的 `UiRenderExtract` 会直接把模板属性里已经 resolved 的视觉字段带出来，而不再只保留几何：

- `background` / `foreground` / `border` 会落到 `UiResolvedStyle`
- `text` / `label` 会落到 render command 的 `text`
- `icon` / `image` 会落到 `UiVisualAssetRef`
- `opacity` 会落到 render command 的 `opacity`

这意味着 style asset 和 inline override 在 template compiler 里完成归并以后，shared surface 已经能把这些视觉结果继续传给 preview/runtime consumer；后续还没做的部分是文本测量、字体 atlas、图片资源装载和真实 GPU pass，而不是再回头重建另一套 visual payload 模型。

## Current Scope And Deliberate Gaps

这一轮刻意没有把以下能力塞进 `zircon_ui::template`：

- Slint host tree 自动投影
- repeat/tree data projection
- 样式 token 继承/覆盖求值
- 模板参数求值和表达式系统
- 文本/图片测量服务
- 基于样式 token 或表达式的动态 layout 合同求值
- runtime widget 级 visual primitive 的完整模板化

原因很直接：这里先锁住模板装配契约，并把显式 layout 合同落进 shared tree，但不在 token、表达式、测量服务都还没定型之前就发明第二层隐式布局公式。

## Why This Boundary Matters

如果没有这层共享模板语义，后续 editor 迁移很容易退回两条错误路线：

- 让 Slint `.slint` 业务树继续做真正的模板权威
- 或者在 `zircon_editor` 里直接把 WorkbenchLayout/ViewModel 拼成另一套 host-only 树

现在 `zircon_ui::template` 已经先把文档、slot、binding 命名、运行时实例展开，以及 shared tree 的第一段桥接统一下来。后续无论是 runtime UI 还是 editor shell，都必须从同一份模板真源继续向 shared layout 求解和宿主投影层推进。

## Builtin Root Document Identity

`zircon_editor` 这一轮又把 shared template runtime 的 builtin root 文档身份往 generic host 边界推进了一步：

- builtin root host 模板现在以 `ui.host_window` 作为首选 `document_id`
- 旧的 `workbench.shell` 仍作为兼容 alias 同时注册到同一份 [`workbench_shell.toml`](/E:/Git/ZirconEngine/zircon_editor/ui/templates/workbench_shell.toml)
- `UiHostWindow` 相关 component descriptor 也同步改成指向 `ui.host_window`
- `EditorUiHostRuntime` 新增 generic `load_builtin_host_templates()`，把“加载一组 builtin host template”与“加载 workbench shell”两个概念拆开
- [`zircon_editor/ui/workbench.slint`](/E:/Git/ZirconEngine/zircon_editor/ui/workbench.slint) 的导出 root 现在也已经跟着这个 identity 收口：`UiHostWindow` 只剩 window/bootstrap wrapper，自身不再直接拥有 menu/drawer/document/floating 业务树；真正的 workbench 结构落在内部 `WorkbenchHostScaffold`
- 这层 wrapper 目前仍通过属性别名和 callback forwarding 暂时保留旧的宿主 ABI，因此 shared template/runtime 的 generic root identity 可以先稳定下来，而不需要一次性重写所有 host/slint 业务接线

这样 shared template runtime 对外暴露的默认 root 入口已经不再是 workbench 业务名；workbench 只剩兼容标签，而不是 builtin host root 的唯一 canonical identity。后续继续做 `Generic host boundary` 时，就可以在不改 shared runtime 主入口命名的前提下，逐步削掉 `workbench.slint` 和 builtin projection 里的业务壳结构。
