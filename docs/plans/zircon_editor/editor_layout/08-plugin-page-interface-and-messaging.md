---
related_code:
  - zircon_editor/src/ui/host/editor_event_runtime_reflection.rs
  - zircon_editor/src/core/editor_event/mod.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/ui/workbench/window_registry/editor_window_registry.rs
  - zircon_plugins/texture/editor/src/plugin.rs
  - zircon_runtime/src/plugin/mod.rs
design_references:
  - docs/ui-and-layout/ai-workbench-style/component-prototype/web-native-handoff-matrix.md
plan_sources:
  - docs/plans/zircon_editor/editor_layout/07-windowing-chrome-tabs-and-dockable-drawers.md
  - docs/zircon_app/plugins.md
status: planned
---
# 08 插件页面接口与编辑器消息交互

## 1. 目标

给插件提供一套**页面接口(Page Interface)**:插件不仅能注册抽屉/视图(现状已能),还能把自己的页面作为一个**可窗口化的目的视图(PurposeView,承接 07)**接入工作台,并通过**编辑器消息交互协议**与编辑器/其它页面通信。接口面向"插件页面 = 一个可注册、可收发消息、可参与布局流转的单元",而非直接操作宿主内部结构。

## 2. 现状(按代码核实)

### 2.1 已存在的设施

| 能力 | 落点 | 证据 |
| --- | --- | --- |
| 插件编辑器扩展注册 | `texture/editor/src/plugin.rs` | `register_authoring_extensions` 注册 drawer/template/surfaces |
| 编辑器扩展注册中心 | `core/editor_extension` | `EditorExtensionRegistry`、`ComponentDrawerDescriptor` |
| 编辑器事件运行时 | `core/editor_event` | `EditorEventRuntime`、`refresh_reflection` |
| 窗口注册中心 | `window_registry`(承接 07) | 视图可窗口化/页签化/停靠 |

### 2.2 真实缺口

- 插件只能注册 drawer/surface,缺**页面级接口**(把页面声明为 PurposeView,参与 07 的页签/抽屉/独立窗口流转)。
- 缺插件页面↔编辑器的**消息交互协议**(请求刷新、订阅状态、跨页面通信)。
- 缺插件页面的**生命周期钩子**(挂载/激活/失活/卸载)。

## 3. 设计

### 3.1 插件页面接口(PageInterface)

插件实现一个页面 trait,声明:页面身份(对应 PurposeView 的 descriptor_id)、内容资产(`.zui`)、默认宿主形态(页签/抽屉/浮窗)、默认 dock 槽(若抽屉)。注册后插件页面成为 07 体系里的一等目的视图,可被合并进 Chrome 页签、可独立成窗口、可吸附转移。

### 3.2 编辑器消息交互协议

页面接口暴露消息端点,所有通信走**编辑器消息总线(承接 09)**,不直接调宿主内部:

| 消息方向 | 用途 | 形态 |
| --- | --- | --- |
| 页面 → 编辑器 | 请求刷新自身、请求布局动作(独立/吸附)、发布脏区 | `EditorPageRequest` |
| 编辑器 → 页面 | 生命周期钩子、激活/失活、选择变更、资产变更通知 | `EditorPageEvent` |
| 页面 ↔ 页面 | 跨页面协作(如选中对象→属性页刷新) | 经总线主题(topic)路由,非点对点硬耦合 |

页面声明它**订阅哪些主题**(如 `selection.changed`、`asset.<id>.changed`),只在订阅主题命中时被通知——这是 09 增量分发的入口。

### 3.3 生命周期钩子

`on_mount`/`on_activate`(成为激活页签或抽屉)/`on_deactivate`/`on_unmount`(关闭/卸载)。钩子让插件页面按需创建/释放重资源,避免常驻全量。

### 3.4 与现有扩展注册的关系

现有 `register_authoring_extensions` 保留;页面接口是其上层:插件可只注册 drawer(轻),也可注册 PurposeView 页面(全功能、参与流转 + 消息)。

## 4. 接口与数据结构草案(Rust)

```rust
// core/editor_page/mod.rs
pub trait EditorPage: Send + Sync {
    fn descriptor(&self) -> &EditorPageDescriptor;        // 身份 + 内容资产 + 默认宿主形态
    fn subscribed_topics(&self) -> &[EditorTopic];        // 订阅哪些消息主题(09 增量入口)
    fn on_event(&mut self, event: &EditorPageEvent, ctx: &mut EditorPageContext);
}
pub struct EditorPageDescriptor {
    pub page_id: EditorPageId,
    pub purpose_view: ViewDescriptorId,                   // 接 07
    pub content: AssetRef,
    pub default_host_form: PageHostForm,                  // Tab | Drawer(dock) | Floating
}
pub enum EditorPageEvent { Mount, Activate, Deactivate, Unmount, Topic(EditorTopic, EditorMessage) }
// 页面发起请求(走总线,非直调宿主)
pub enum EditorPageRequest { RequestRefresh, Detach, Reattach(DrawerDockPosition), Publish(EditorTopic, EditorMessage) }
```

## 5. 模块与文件落点

| 动作 | 文件 | 说明 |
| --- | --- | --- |
| 新增 | `zircon_editor/src/core/editor_page/mod.rs` | 页面 trait + descriptor + 事件/请求 |
| 修改 | `core/editor_extension/mod.rs` | 注册 EditorPage(在 drawer 之上) |
| 修改 | `core/editor_event/mod.rs` | 页面消息进事件运行时 |
| 修改 | `zircon_plugins/texture/editor/src/plugin.rs` | 示例:注册一个 EditorPage |

## 6. 里程碑切片化

| # | 切片 | 涉及文件 | 验证命令 | 硬切换 |
| -- | --- | --- | --- | --- |
| S1 | 页面接口 + 注册 + 生命周期钩子 | editor_page/mod.rs / editor_extension | `cargo test -p zircon_editor --lib --locked` | — |
| S2 | 页面消息进事件运行时 + 插件示例 | editor_event / texture plugin | `cargo check --manifest-path zircon_plugins/Cargo.toml --workspace --locked` | — |

## 7. 测试矩阵

- 插件注册 EditorPage 后,页面作为 PurposeView 出现在 window registry(可页签/抽屉/浮窗)。
- 生命周期钩子按 mount/activate/deactivate/unmount 顺序触发。
- 页面只在订阅主题命中时收到 `Topic` 事件。
- 页面请求(刷新/独立/吸附)经总线转为 07 注册中心动作。

## 8. 风险与对策

- 风险:插件页面直接持宿主可变引用导致耦合。对策:页面只经 `EditorPageContext` + 消息总线交互,不暴露宿主内部。
- 风险:页面订阅过宽主题退化为全量。对策:09 强制主题粒度 + 脏区路由,本计划只定订阅形态。

## 9. 完成定义

插件可注册页面级目的视图,参与 07 布局流转,经消息协议与编辑器/其它页面增量通信,生命周期钩子完整。

## 10. 边界约束

不实现消息总线分发机制本身(属 09);不让插件直调宿主;dock 职责按 index §1.1。

## 11. 参考实现对照(dev/ 源码锚点)

- `dev/UnrealEngine/.../Slate/Public/Framework/Application`:命令/扩展接入参考。
- `dev/Fyrox/editor`:编辑器扩展面板接入参考。

## 12. 状态与产出记录

planned。后续项:S1 页面接口与生命周期。
