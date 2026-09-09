---
related_code:
  - zircon_editor/src/ui/workbench/mod.rs
  - zircon_editor/src/ui/workbench/layout/mod.rs
  - zircon_editor/src/ui/workbench/view/mod.rs
  - zircon_editor/src/ui/workbench/window_registry/mod.rs
  - zircon_editor/src/ui/workbench/project/mod.rs
  - zircon_editor/src/ui/binding/mod.rs
  - zircon_editor/src/ui/binding_dispatch/mod.rs
  - zircon_editor/src/ui/retained_host/mod.rs
implementation_files:
  - zircon_editor/src/ui/workbench/layout/workbench_layout.rs
  - zircon_editor/src/ui/workbench/view/view_registry.rs
  - zircon_editor/src/ui/workbench/window_registry/editor_window_registry.rs
  - zircon_editor/src/ui/workbench/state/editor_state.rs
  - zircon_editor/src/ui/retained_host/ui.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine 编辑器详细 Wiki
  - .codex/plans/布局系统.md
  - .codex/plans/Zircon Editor Workbench Shell V1.md
  - docs/editor-and-tooling/editor-workbench-shell.md
tests:
  - zircon_editor/src/ui/workbench
  - zircon_editor/src/ui/retained_host/app/tests
  - zircon_editor/tests/integration_contracts.rs
doc_type: module-detail
---

# Workbench、面板与 UI 绑定

## 概览

Workbench 是编辑器窗口和面板的模型层，不是 native window toolkit。它描述页面、drawer、document split、pane、tab、浮动窗口和布局操作；`retained_host` 把这些 snapshot 投影为原生窗口与共享 `UiSurface`。

## 工作台组成

```text
Workbench window
├─ menu / main host strip
├─ main page tabs
├─ activity rail
├─ left / right / bottom drawers
├─ document workspace
│  └─ split tree -> tab stacks -> view instances
├─ status bar
└─ floating windows / detached panes
```

固定 chrome 与可扩展内容分离。扩展通常贡献 view/activity/command，不直接替换原生窗口根。

## View descriptor 与 instance

`ViewDescriptor` 描述一种可打开内容，`ViewInstance` 表示一次具体打开。两者分别由 `ViewDescriptorId` 和 `ViewInstanceId` 标识。

Descriptor 包含：

- `ViewKind` 与 `ViewHost`。
- `DockPolicy`：允许的停靠方式。
- `PaneInteractionMode`。
- `PanePayloadKind`/route namespace。
- `PaneTemplateSpec` 或 activity window template。

`ViewRegistry` 是 descriptor 和 instance 的权威注册表。一个 descriptor 可以是 singleton，也可以允许多个实例；restore 时必须先验证 descriptor 仍存在，并根据策略恢复或降级。

插件应通过扩展贡献注册 descriptor。不要直接把任意 pane 塞进 layout，因为缺失 descriptor 会破坏恢复、菜单和反射。

## 窗口与 Drawer registry

`EditorWindowRegistry` 统一管理：

- 普通 `WindowInstance`。
- `DrawerViewInstance` 与 `DrawerBinding`。
- 独立 `DrawerWindowInstance`。
- `DrawerDockPosition` 和菜单 overflow 策略。

`ActivityWindowLayout` 自己拥有 left/right/bottom drawer 状态。切换到不支持 drawer 的窗口时，不能继承前一个 workbench window 的 drawer。Drawer view 与普通 document view 是不同宿主合同。

## 布局模型

`WorkbenchLayout` 聚合主 frame、activity windows、document tree 和 floating windows。Document workspace 使用 `DocumentNode` 树：

- `DocumentLeafLayout`：一个 tab stack leaf。
- split node：按 `SplitAxis` 分割两个子树。
- `DocumentNodeId`：稳定节点身份，拖放和 normalization 使用。

常见操作由 `LayoutCommand` 表达：激活页面、切换 drawer、移动/插入 tab、split、float、dock、resize 等。`LayoutManager` 应用命令并返回 `LayoutDiff`；非法目标返回 `LayoutCommandError`。

拖放由 `DragPayload` 与 `DropTarget` 表示。`WorkspaceTarget`、`SplitPlacement`、`TabInsertionAnchor/Side` 使“插入 tab”和“建立 split”不依赖像素坐标字符串。

## 布局规范化

每次 restore 或结构操作后需要 normalization：

- 删除空 tab stack/split。
- 合并不再需要的单子节点 split。
- 过滤缺失 view instance。
- 修复 active tab/page。
- 保证 builtin shell 必需视图存在。
- 对超出窗口范围的 floating placement 做约束。

`LayoutNormalizationReport` 记录修复，而不是静默把损坏数据当正常布局。Restore policy 决定严格拒绝还是回退默认 preset。

## 布局持久化

布局分用户默认、项目 workspace、preset 等 scope。`LayoutPreset`/`LayoutPresetPersistenceStore` 保存逻辑布局和 size override；项目目录下的 editor workspace sidecar 保存项目相关的最近文档和布局。

持久化对象应使用带版本的 Zircon document envelope，并执行原子写入。恢复时先解析和验证，再装入 registry；旧格式按硬切策略迁移或回退，不能长期维护双格式 runtime 分支。

## AutoLayout 与呈现

`workbench::autolayout` 从 shell region、chrome metrics、resolution context 和 preferred extents 计算稳定几何。它输出 pane/drawer/tab/status 等 frame，`retained_host` 再将 snapshot 交给 shared UI layout/render/hit framework。

规则：

- 同一 frame 的绘制与 hit-test 使用同一几何快照。
- Viewport 使用 pane content frame，不能使用整个 document leaf。
- Drag/hotzone 基于 layout target，而不是 UI callback 临时推导业务结构。
- Native child window 有自己的 surface、capture 和 source-window identity。

## EditorState 与 snapshot

`EditorState` 是 workbench 的作者态 UI 状态聚合，包含 selection projection、play mode、field drafts、console history、scene document binding 等。它不是 runtime world。

Snapshot 层把状态转换为无锁或短锁读取的 presentation DTO：

- `EditorDataSnapshot`：Inspector、Scene entry、console、diagnostics、任务进度。
- `WorkbenchSnapshot`：页面、pane、tab、drawer、floating window。
- `AssetWorkspaceSnapshot`：资产文件夹、item、selection、operation。

Retained host 只消费 snapshot 和 effect；不要在 paint 回调中查询/修改 manager。

## Typed UI binding

`EditorUiBinding` 是模板控件与编辑器行为之间的 typed 协议。`EditorUiBindingPayload` 当前覆盖：

| 命令类型 | 使用场景 |
| --- | --- |
| `DockCommand` | drawer、pane、tab 和窗口操作 |
| `SelectionCommand` | Hierarchy/Viewport 选择 |
| `AssetCommand` | 打开资产、导入模型等 |
| `DraftCommand` | Inspector/路径文本草稿 |
| `ViewportCommand` | pointer、scroll、resize、toolbar |
| `AnimationCommand` | 动画轨道、关键帧、scrub/playback |
| `WelcomeCommand` | 新建/打开/最近项目 |

处理链：

```text
template nativeBinding
 -> EditorUiRouter
 -> dispatch_*_binding
 -> typed host event or normalized EditorEvent
 -> apply_*_binding / host effect
```

`binding_dispatch` 提供不同域的 dispatch/apply 函数。欢迎页保留为 typed host event，因为项目选择和 session lifecycle 属于宿主；资产文件导入也先产生 host request effect，由宿主执行文件系统副作用。

## 反射与远控

`EditorUiReflectionAdapter` 将菜单、页面、drawer、floating window 和 activity 投影为可检查模型。反射 action 使用与 nativeBinding 相同的 typed payload，并回到正式 event runtime 执行；`CallAction` 不应运行一套独立 preview 逻辑。

Widget Reflector 消费 shared runtime 的 `UiReflectorSnapshot`，只持有本地选中行。它不重建 layout 或第二套 hit index。

## Retained host

`retained_host` 负责：

- 原生窗口/event loop 生命周期。
- keyboard/pointer/capture/popup dismiss。
- shared UI surface 绘制、文本和模板节点转换。
- callback 到 typed binding 的适配。
- root 与 child/floating window presentation。
- viewport surface 嵌入和 redraw 调度。

这些大多是内部 API。外部集成应使用 `run_editor*`、view/command/plugin 描述符和自动化入口。

## 状态与限制

- **已实现**：多页面、drawer、document split/tabs、浮动窗口、持久化、typed binding、reflection、native retained host。
- **可扩展基础**：更多 asset/activity command catalog、远程控制 transport。
- **内部实现**：具体模板节点、pointer bridge、paint/hit index、native presenter。
- 浮动窗口部分 public 类型仅在 `test` 或 `integration-contracts` feature 可见。
- 新面板必须有 descriptor、payload kind 和 restore 策略；仅添加绘制模板不足以成为完整 view。
