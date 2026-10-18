# Zircon Editor Workbench Shell V1

## Summary

- 将当前固定三栏 `iced` editor 重构为“混合固定壳 + 受控 docking 树”。
- 顶部采用 `MainHostTabStrip`，用于切换主工作台页和占据主编辑器的 `ActivityWindow`。
- 边缘固定 6 个 `ActivityDrawer` 槽位：`left_top / left_bottom / right_top / right_bottom / bottom_left / bottom_right`。
- 只有中心 `DocumentWorkspace` 和原生浮动窗口允许递归分栏；`ActivityDrawer` 只允许 tab 化、展开收起和尺寸调整，不参与递归 split。
- `ActivityView` 与 `ActivityWindow` 都可进入中心文档区，也都支持分离成真实原生窗口；`ActivityWindow` 额外支持“占据主编辑器”的独占页模式。
- 布局持久化采用“双层”：全局默认/预设走用户级配置，项目最近布局嵌入项目文件；恢复优先级固定为 `preset > project workspace > global default > builtin fallback`。

## Key Changes

### 1. Workbench Layout Model

- 新增 `WorkbenchLayout` 作为 editor 壳层权威布局对象，包含：
  - `active_main_page`
  - `main_pages`
  - `drawers`
  - `floating_windows`
- `main_pages` 固定分两类：
  - `WorkbenchPage`
    - 默认主工作台页
    - 持有中心 `DocumentWorkspaceLayout`
    - 激活时显示边缘 drawers
  - `ExclusiveActivityWindowPage`
    - 对应占据主编辑器的 `ActivityWindow`
    - 激活时保留菜单栏和顶部 `MainHostTabStrip`
    - 边缘 drawers 隐藏或冻结但不销毁状态
- `DocumentWorkspaceLayout` 只允许两种节点：
  - `SplitNode { axis, ratio, first, second }`
  - `TabStack { tabs, active_tab }`
- `ActivityDrawerLayout` 固定字段：
  - `slot`
  - `tab_stack`
  - `active_view`
  - `mode: Pinned | AutoHide | Collapsed`
  - `extent`
  - `visible`
- `FloatingWindowLayout` 承载真实 OS window，内部持有一个 `DocumentWorkspaceLayout`，允许一个或多个 tabs。

### 2. View Model And Registry

- `View` 是统一抽象，运行时必须拆成：
  - `ViewDescriptor`
    - 静态注册元数据
    - 字段固定为 `descriptor_id / kind / default_title / icon_key / multi_instance / dock_policy / preferred_drawer_slot / preferred_host / persistence_key_policy`
  - `ViewInstance`
    - 运行时实例
    - 字段固定为 `instance_id / descriptor_id / title / serializable_payload / dirty / host`
- `ActivityView`
  - 默认归属某个 `ActivityDrawer`
  - 可拖到中心 `DocumentWorkspace`
  - 例子：`Hierarchy / Inspector / Scene / Game / Assets`
- `ActivityWindow`
  - 可作为普通中心 tab
  - 可作为 `ExclusiveActivityWindowPage`
  - 可分离为原生窗口
  - 例子：`Prefab Editor / Asset Browser`
- 新增 `ViewRegistry`，唯一负责：
  - descriptor 注册
  - 实例创建
  - 实例恢复
  - 内建 view 列表暴露
- `ViewRegistry` 不保存布局，不负责拖拽。

### 3. EditorManager / LayoutManager / WindowHostManager Responsibilities

- `EditorManager` 从空壳升级为 editor 域总协调者，持有：
  - `ViewRegistry`
  - `LayoutManager`
  - `WindowHostManager`
  - `EditorSessionState`
- `EditorManager` 公开接口固定为：
  - `register_builtin_views()`
  - `open_project(path)`
  - `save_project(path)`
  - `current_layout() -> WorkbenchLayout`
  - `apply_layout_command(cmd: LayoutCommand) -> Result<bool, EditorError>`
  - `open_view(descriptor_id, target_host)`
  - `close_view(instance_id)`
  - `focus_view(instance_id)`
  - `detach_view_to_window(instance_id)`
  - `attach_view_to_target(instance_id, drop_target)`
  - `restore_workspace(policy: RestorePolicy)`
- `LayoutManager` 是布局树和恢复规则的唯一权威，公开接口固定为：
  - `default_layout() -> WorkbenchLayout`
  - `load_global_default(config) -> Option<WorkbenchLayout>`
  - `load_project_workspace(doc) -> Option<ProjectEditorWorkspace>`
  - `save_global_default(config, layout)`
  - `save_project_workspace(project_doc, workspace)`
  - `apply(cmd: LayoutCommand, layout: &mut WorkbenchLayout) -> Result<LayoutDiff, LayoutError>`
  - `resolve_drop(payload, hit_target) -> DropTarget`
  - `normalize(layout, registry) -> LayoutNormalizationReport`
- `WindowHostManager` 只负责真实原生窗口生命周期：
  - `open_native_window(window_layout)`
  - `close_native_window(window_id)`
  - `sync_window_bounds(window_id, bounds)`
  - `reattach_window(window_id, drop_target)`
- `LayoutCommand` 在 V1 固定包含：
  - `OpenView`
  - `CloseView`
  - `FocusView`
  - `MoveView`
  - `AttachView`
  - `DetachViewToWindow`
  - `CreateSplit`
  - `ResizeSplit`
  - `SetDrawerMode`
  - `SetDrawerExtent`
  - `ActivateDrawerTab`
  - `ActivateMainPage`
  - `SavePreset`
  - `LoadPreset`
  - `ResetToDefault`

### 4. Dragging Rules

- 拖到边缘活动栏图标区：
  - 注册或移动到对应 `ActivityDrawer`
- 拖到某个 drawer 内容区：
  - 并入该 drawer 的 `TabStack`
- 拖到中心 tab strip：
  - 并入当前 `TabStack`
- 拖到中心内容区四边热区：
  - 对目标 `TabStack` 执行一次 split
- 拖出主壳释放：
  - 生成 `FloatingWindowLayout`
- `ActivityDrawer` 永远不能被递归拆分成 `SplitNode`
- `ExclusiveActivityWindowPage` 不参与 drawer 停靠；它是主壳顶层页
- `ActivityView` 分离成原生窗口后，宿主变为 `FloatingWindowLayout`，不是特殊类型树。

### 5. Persistence And Restore

- 全局布局持久化：
  - 使用 `ConfigServer`
  - key 固定为：
    - `editor.workbench.default_layout`
    - `editor.workbench.presets`
- 为保证跨重启有效，V1 同步让配置存储具备文件落盘能力：
  - 由配置层持久化到用户级 editor config 文件
  - editor 不自己再维护第二套全局 JSON 存储
- 项目级布局持久化：
  - 嵌入项目文件
  - 新增 `editor_workspace: Option<ProjectEditorWorkspace>`
- 项目文档 ownership 调整：
  - editor 项目保存/加载不再直接把 `World` 当完整项目文件
  - 新项目文档应包含 `world + editor_workspace`
  - 缺失 `editor_workspace` 的旧项目视为“无最近布局”，按恢复优先级回退
- `ProjectEditorWorkspace` 字段固定为：
  - `layout_version`
  - `workbench`
  - `open_view_instances`
  - `active_center_tab`
  - `active_drawers`
- 恢复优先级固定：
  1. 显式加载的 preset
  2. 项目文件中的 `editor_workspace`
  3. 全局默认工作台
  4. 内建 fallback
- 恢复策略固定为 best-effort：
  - descriptor 缺失时保留 placeholder tab
  - 实例 payload 恢复失败时丢弃该实例并记 warning
  - 原生窗口无法恢复时并回主壳中心
  - 非法 split ratio、空 active tab、坏 host 引用都由 `normalize` 修正，不整体失败

## Public Interfaces And Types

- `zircon_editor`
  - `EditorManager`
  - `LayoutManager`
  - `WindowHostManager`
  - `ViewRegistry`
  - `ViewDescriptor`
  - `ViewInstance`
  - `WorkbenchLayout`
  - `DocumentWorkspaceLayout`
  - `ActivityDrawerLayout`
  - `FloatingWindowLayout`
  - `MainHostPageLayout`
  - `ProjectEditorWorkspace`
  - `LayoutCommand`
  - `DropTarget`
  - `RestorePolicy`
- `zircon_server` / config facade
  - 继续使用 `ConfigServer`
  - 增加 editor workbench 所需的持久化调用路径，不新增 editor 专用全局配置服务
- editor presenter / host
  - 顶部菜单栏
  - `MainHostTabStrip`
  - 6 个 `ActivityDrawer`
  - 中心 `DocumentWorkspace`
  - 原生窗口宿主桥接
- built-in registered descriptors
  - `Hierarchy`
  - `Inspector`
  - `Scene`
  - `Game`
  - `Assets`
  - `PrefabEditor`
  - `AssetBrowser`

## Test Plan

- 单元测试
  - `ViewRegistry` 的注册、重复注册、单实例与多实例行为
  - `WorkbenchLayout`、`DocumentWorkspaceLayout`、`ActivityDrawerLayout`、`FloatingWindowLayout` 的序列化 roundtrip
  - `LayoutManager::apply` 对 open/move/split/detach/attach/set_drawer_mode 的行为
  - `LayoutManager::normalize` 对坏 ratio、坏 active tab、坏 host 引用、缺失 descriptor 的修正
  - 恢复优先级 `preset > project > global > fallback`
- 集成测试
  - editor 启动后默认出现混合固定壳
  - `Hierarchy/Inspector/Scene/Game/Assets` 能以 `ActivityView` 注册并停靠
  - `Prefab Editor/Asset Browser` 能以 `ActivityWindow` 打开
  - `ActivityView` 能从 drawer 拖到中心，再拖回 drawer
  - `ActivityWindow` 能进入中心、切为 `ExclusiveActivityWindowPage`、再分离为原生窗口、再附回
  - `ActivityDrawer` 在 `Pinned / AutoHide / Collapsed` 间切换后重启可恢复
  - 保存项目后重新打开，项目级最近布局恢复成功
  - 缺失某个 view descriptor 后打开旧项目，布局仍能恢复为 placeholder 而不是崩溃
- 人工验收
  - 顶部菜单 + 顶部 window tabs + 边缘活动栏 + drawers + 中心文档区符合 VSCode/JetBrains 混合目标
  - 拖拽反馈清楚，落点与结果一致
  - resize 后 viewport、hierarchy、inspector 宿主区域稳定重排
  - 切到 `ExclusiveActivityWindowPage` 时只保留菜单和顶部 tabs，drawers 隐藏或冻结
  - 应用 preset 能覆盖当前项目最近布局

## Assumptions And Defaults

- V1 固定采用“混合固定壳 + 受控 docking 树”，不做全壳 universal docking。
- 只有中心 `DocumentWorkspace` 与原生浮动窗口允许递归 split。
- `ActivityView` 与 `ActivityWindow` 都可进入中心区、都可分离成原生窗口。
- `ActivityWindow` 额外支持主壳独占页模式；`ActivityView` 不支持该模式。
- `ActivityDrawer` 默认同时支持 `Pinned / AutoHide / Collapsed` 三态。
- 顶部 `MainHostTabStrip` 是主壳页切换层，不等同于中心 `TabStack`。
- 项目级最近布局嵌入项目文件；全局默认与 preset 走用户级配置。
- 缺失 `editor_workspace` 的旧项目默认按“无最近布局”处理，并回退到全局或内建默认布局。
