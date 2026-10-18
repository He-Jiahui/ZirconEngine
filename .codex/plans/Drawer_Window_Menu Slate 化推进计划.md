# Drawer/Window/Menu Slate 化推进计划

## Summary
- 先把抽屉、窗口、菜单从“投影层临时状态”收束为 editor 内唯一数据层，再让 `.ui.toml` 只描述布局和 Material 控件，Rust host 只做注册、路由、命中、重绘。
- 当前代码已有 `ActivityWindowLayout`、窗口级 `activity_drawers`、菜单路径和快捷键描述，但仍有旧全局 drawer、单层菜单、窗口关闭无 dirty prompt、菜单/标题栏/抽屉投影分散的问题。
- 本轮目标先修你列出的 GUI 问题：输入框不可输入、任务栏不显示、页签/抽屉/页面不可交互、场景拖拽重影、verbose 和重复重建导致卡顿，并用实际运行截图验收。

## Key Changes
- **M0 基线与回归复现**
  - 运行 editor，截图记录 Welcome、Workbench、Asset Browser、抽屉展开/折叠、菜单弹出、拖拽场景。
  - 打开最小日志节流，先区分输入路由失败、投影缺失、重绘过量、拖拽缓存残留四类问题。
  - 先保证 `zircon_editor` 可 check，再做 GUI 修复。

- **M1 窗口/抽屉单例数据层**
  - 新增 editor 内部 `EditorWindowRegistry` 作为唯一窗口/抽屉注册服务，由 `EditorUiHost`/runtime 持有，不用全局静态变量。
  - 管理三类实例：普通窗口、含抽屉窗口、独立抽屉窗口；提供 `get_window`、`get_drawer_view`、`get_drawer_window` 风格的 typed accessor。
  - 窗口只有注册过 drawer view 才显示抽屉；动态注册和初始化注册都写入同一 registry。
  - 对外默认位置使用 `LeftTop`、`LeftBottom`、`Bottom`、`RightTop`、`RightBottom`；旧 `BottomLeft/BottomRight` 布局在加载时合并/迁移到 `Bottom` 顺序列表。

- **M2 抽屉语义与拖拽**
  - 折叠状态定义为该 drawer list 没有 selected/open view：保留 tabs/order，但 `active_view = None`。
  - 点击 drawer tab 选择并展开；再次关闭/切换到无对应 drawer 的窗口时，当前 host 立即隐藏抽屉，不泄漏上一个窗口的 drawer。
  - 拖出 drawer view 且未吸附时创建独立 drawer window，并保留标题栏；拖入其他 drawer list 时执行 attach + reorder。
  - 所有拖拽结束都清理 drag overlay、capture、hover damage，修复场景拖拽重影。

- **M3 主框架、标题栏与关闭保护**
  - 主框架顺序固定为：任务栏/菜单栏在最上方，窗口标题栏/窗口页签在其下方，active window 内容在下方。
  - 标题栏管理吸附 window、拖出独立窗口、关闭窗口；独立窗口也保留同一标题栏。
  - 关闭窗口、关闭独立 drawer window、关闭主窗口都先查询 dirty registry；存在未保存内容时弹出 `Save / Discard / Cancel`，主窗口聚合所有 dirty 文档。
  - 切换窗口时只显示目标窗口注册的 drawer 和菜单上下文。

- **M4 多级菜单与任务栏修复**
  - 将现有 flat `MenuBarModel -> MenuItemModel` 升级为 tree model，直接消费 `EditorMenuItemDescriptor.path` 和 `EditorOperationDescriptor.menu_path`。
  - 支持二级、三级及更深路径，保留 shortcut、enabled、capability、priority。
  - 菜单栏超宽时默认水平滚动；弹出菜单超出窗口时默认约束回窗口内，过高时滚动，配置为 multi-column 时按列布局。
  - 嵌套 popup 根据可用空间自动向左/向上翻转，不能跑出窗口边界。

- **M5 输入、事件与 Material 控件闭环**
  - 输入框使用 shared focus/text route，鼠标点击获得 focus，键盘/IME commit 进入 `.ui.toml` binding，不再被页面/抽屉命中层吞掉。
  - 页签栏、抽屉栏、页面栏、菜单、Material 控件统一走 `UiSurfaceFrame + hit_grid + UiDispatchReply/effect`。
  - Button、IconButton、CheckBox、Switch、ComboBox、LineEdit/TextEdit、Slider、SpinBox、Menu、Tabs、Scroll/Table 等 Material meta component 必须把 bindings 透传到 expanded root control。
  - Editor host 不新增控件专用坐标表，只做平台事件翻译和 native projection。

- **M6 渲染与性能**
  - 固定 dirty 分层：layout dirty、render dirty、presentation dirty、pointer-only dirty。
  - Hover、press、text cursor、popup、drag overlay、viewport image 更新只产生局部 damage。
  - 100 次同目标 mouse move 不触发 presentation rebuild；日志中 template load/compile 和 verbose hit/debug 输出必须节流或按 debug flag 开启。
  - 截图验证菜单栏可见、输入框可输入、抽屉可展开、页签可拖拽、场景拖拽无残影。

## Public Interfaces / Types
- 新增 editor 数据层合同：`EditorWindowRegistry`、`WindowInstance`、`WindowKind`、`DrawerViewInstance`、`DrawerWindowInstance`、`DrawerBinding`、`DrawerDockPosition`、`MenuOverflowMode`。
- 保留并收敛共享 UI 合同：`UiSurfaceFrame`、`UiArrangedTree`、`UiHitTestGrid`、`UiHitPath`、`UiVisibility`、`UiDispatchReply/effect`。
- `ActivityWindowLayout.activity_drawers` 成为 drawer ownership 真源；旧 `WorkbenchLayout.drawers` 只允许迁移读取，不能再作为运行时投影事实。
- 菜单 path 注册继续兼容 Unity `MenuItem` 风格路径，shortcut 从 operation/menu descriptor 进入 tree model。

## Test Plan
- 编译：`cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never`。
- Runtime：运行 hit-grid、input ownership、event routing、material layout focused tests。
- Editor focused tests：窗口注册、无 drawer 窗口切换自动隐藏、drawer detach/attach/reorder、dirty close prompt、menu tree/shortcut/overflow、text input focus/commit、drag overlay cleanup。
- Native GUI 验收：启动 editor 后截图 Welcome 输入、顶部任务栏、Workbench 抽屉、菜单多级 popup、独立 drawer window、场景拖拽后画面。
- 性能验收：同目标 mouse move 不 rebuild；popup/hover/press/text cursor 只局部 damage；日志不重复刷 template load/compile。

## Assumptions
- “单例管理”按 editor runtime 内单一 registry/service 实现，不使用不可控全局 mutable static。
- “下方”对外按一个 bottom drawer list 处理；旧 `BottomLeft/BottomRight` 数据迁移时保序合并。
- “类 Unreal Slate”复刻 Slate 的 arranged geometry、hit grid、widget path、reply/effect、focus/capture、invalidation、fast paint 语义，不复制 Unreal C++ API。
- 本轮优先 2D editor/runtime UI；world-space UI 只保留进入同一 hit grid 的接口边界。

## Progress Evidence
- 2026-05-07：Bottom drawer 已收束为 canonical `Bottom` 列表；旧 `BottomLeft` / `BottomRight` 输入在加载/投影时迁移合并。`LayoutManager.apply(...)` 在变更命令后同步 legacy drawer projection，避免 host 与 direct manager 使用不同 drawer 真源。
- 抽屉拖出生成独立 drawer window 的 focused gate 已通过：`cargo test -p zircon_editor --lib tab_drop_dispatch_detaches_drawer_tab_to_independent_drawer_window --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never`。
- dirty close UX 进入 Save/Discard/Cancel 闭环：`close_prompt` filter 通过，新增 `dirty_saveable_floating_window_save_prompt_saves_then_closes_window` 覆盖保存后关闭子窗口；视觉 artifact 为 `target/visual-layout/editor-window-20260507-close-prompt-900x620.png`。
- GUI 截图 gate 已补：`capture_m3_gui_acceptance_visual_artifacts` 通过并生成 Welcome 输入、Workbench、Asset Browser、drawer、menu popup + SVG icon、drag release、small/large SVG icon scale artifacts。当时待关闭的菜单多列/滚动溢出、drag/drop lifecycle、editor native shared translation 后续证据已记录在下方 2026-05-07 条目中。
- 2026-05-07：shared drag/drop lifecycle 已进入 runtime surface input state。`UiDispatchEffect::DragDrop` 现在统一 begin/update/accept/reject/complete/cancel，begin 取得 source capture/high precision，complete/cancel 释放 drag overlay 所需的 capture/high precision，stale pointer/session 不会清掉当前有效拖拽。验证：`cargo test -p zircon_runtime --lib drag_drop_ --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m5 --message-format short --color never` 2 passed / 0 failed；`cargo test -p zircon_runtime --lib runtime_input_ownership --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m5 --message-format short --color never` 11 passed / 0 failed。
- 2026-05-07：editor native 事件与 painter broad gate 通过，覆盖输入框 keyboard/IME 转译、document tab drag capture cleanup、viewport toolbar、drawer/header/menu/native host route 和 SVG/preview image paint。验证：`cargo test -p zircon_editor --lib native_input_translation --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m4 --message-format short --color never` 5 passed / 0 failed；`cargo test -p zircon_editor --lib native_ --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m4 --message-format short --color never` 66 passed / 0 failed。
- 2026-05-07：M4 菜单 popup hit geometry 已补 bounds 与 layout-level multi-column opt-in。`HostMenuPointerLayout.menu_overflow_mode` 默认 `Auto`，`MenuOverflowMode::MultiColumn` 下 `popup_layout.rs` 生成 capped `PopupGridLayout`，宽高不会超出 host shell，`host_menu_pointer_bridge_rebuild_surface.rs` 按列/行放置 shared hit nodes 且保留 absolute action index。验证：`cargo test -p zircon_editor --lib shared_menu_pointer_bridge_clamps_popup_hit_frames_to_tiny_shell --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture` 1 passed / 0 failed / 1113 filtered out；`cargo test -p zircon_editor --lib shared_menu_pointer_bridge_routes_multi_column_popup_items_after_right_edge_clamp --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture` 1 passed / 0 failed / 1113 filtered out；`cargo test -p zircon_editor --lib slint_menu_pointer --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never -- --nocapture --test-threads=1` 17 passed / 0 failed / 3 ignored / 1094 filtered out。当时剩余：production builder 固定 `Auto`，因为 registry/chrome snapshot 尚无用户或配置层 overflow preference。
- 2026-05-07：M4 菜单 overflow production preference 已接通。`ActivityWindowLayout.menu_overflow_mode` 作为持久 per-window 设置，默认 `Auto` 兼容旧布局；`WindowInstance` 与 `EditorChromeSnapshot.menu_overflow_mode` 镜像活动窗口偏好；`build_host_menu_pointer_layout(...)` 不再硬编码 `Auto`，会把 chrome 偏好写入 `HostMenuPointerLayout`。验证：`cargo check -p zircon_editor --lib --tests --locked --jobs 1 --target-dir F:\cargo-targets\zircon-m4-menu-overflow --message-format short --color never` 通过；`cargo test -p zircon_editor --lib menu_overflow_preference --locked --jobs 1 --target-dir F:\cargo-targets\zircon-m4-menu-overflow --message-format short --color never -- --nocapture --test-threads=1` 3 passed / 0 failed；`cargo test -p zircon_editor --lib window_registry --locked --jobs 1 --target-dir F:\cargo-targets\zircon-m4-menu-overflow --message-format short --color never -- --nocapture --test-threads=1` 6 passed / 0 failed；`cargo test -p zircon_editor --lib slint_menu_pointer --locked --jobs 1 --target-dir F:\cargo-targets\zircon-m4-menu-overflow --message-format short --color never -- --nocapture --test-threads=1` 18 passed / 0 failed / 3 ignored。
- 2026-05-07：M4 多级菜单 popup 进入 tree route + native painter 闭环。`MenuItemSpec`、`HostMenuChromeItemData` 保留 children，branch hover/click 打开 `open_submenu_path`，shared hit surface 为每层 popup 分配独立 node id；子 popup 右侧溢出时翻左并约束在 shell 内，leaf 仍按 pre-order index 路由到 legacy action 或 `EditorOperationPath`。Native chrome painter 和 native pointer guard 消费同一 children/path 状态，子菜单区域不再落到 pane/viewport 命中。验证：`cargo test -p zircon_editor --lib shared_menu_pointer_bridge_opens_flipped_nested_popup_for_branch_hover --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m4-nested-menu --message-format short --color never -- --nocapture` 1 passed / 0 failed；`cargo test -p zircon_editor --lib shared_menu_pointer_click_dispatches_nested_editor_operation_leaf_from_workbench_model --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m4-nested-menu --message-format short --color never -- --nocapture` 1 passed / 0 failed；`cargo test -p zircon_editor --lib rust_owned_host_painter_draws_open_nested_menu_popup --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m4-nested-menu --message-format short --color never -- --nocapture` 1 passed / 0 failed；`cargo test -p zircon_editor --lib slint_menu_pointer --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m4-nested-menu --message-format short --color never -- --nocapture --test-threads=1` 21 passed / 0 failed / 3 ignored。
- 2026-05-07：M4 nested menu slice 通过 editor `--lib --tests` 编译门槛：`cargo check -p zircon_editor --lib --tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m4-nested-menu --message-format short --color never` exit 0。输出仍有既有 runtime/editor unused warning，但没有 tree route、native chrome children 或 painter 合同错误。
- 2026-05-07：M4 nested menu visual gate 通过：`cargo test -p zircon_editor --lib capture_nested_menu_popup_visual_artifact --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m4-nested-menu --message-format short --color never -- --ignored --nocapture --test-threads=1` 1 passed / 0 failed，并生成 `target/visual-layout/editor-window-20260507-nested-menu-popup-900x620.png`。截图显示顶部 Tools 菜单、Weather branch、右侧子 popup 和 hovered child row 均由 native host contract/painter 输出。
- 2026-05-07：真实 editor 可执行构建与桌面截图通过。`cargo build -p zircon_app --bin zircon_editor --features target-editor-host --no-default-features --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m4-nested-menu --message-format short --color never` exit 0；随后以 `SLINT_BACKEND=software` 启动真实 `zircon_editor.exe`，枚举 `Zircon Editor` 顶层窗口并抓图到 `E:\zircon-build\gui-validation\editor-nested-menu-live-software.png`，画面显示顶部菜单栏、Workbench、抽屉栏、Scene viewport、Inspector 和底部 Console。默认后端的 GDI `CopyFromScreen` 抓图得到白色客户区，软件后端用于本轮桌面截图证据以避开 GPU 后端截图限制。
- 2026-05-07：M4 菜单栏横向溢出与动态槽位已补齐。`HostMenuPointerLayout.button_frames` 从固定 7 槽改为动态槽位并记录 `menu_bar_content_width`；shared pointer surface 根据 `menu_bar_scroll_offset` 裁剪按钮命中并允许横向滚动；`HostMenuStateData.menu_bar_scroll_px` 让 native painter、popup 锚点和 native pointer guard 使用同一偏移。`chrome_template_projection.rs` 会从 `.ui.toml` 的 `MenuSlot0..6` stencil 继续克隆 extension top-level menu 槽位，避免插件菜单被截断。验证：`rustfmt --edition 2021 --check ...` 本次触碰文件通过；`cargo test -p zircon_editor --lib slint_menu_pointer --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m4-menu-overflow-dynamic --message-format short --color never -- --nocapture` 21 passed / 0 failed / 4 ignored；`cargo check -p zircon_editor --lib --tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m4-menu-overflow-dynamic --message-format short --color never` exit 0；`cargo test -p zircon_editor --lib menu_chrome_nodes_project_extension_slots_beyond_authored_stencil --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m4-menu-overflow-dynamic --message-format short --color never -- --nocapture` 1 passed / 0 failed。post-M8 收尾已补：`cargo fmt --all -- --check` 通过，上一轮记录的 5 个既有格式差异文件已按 rustfmt 机械收束。
- 2026-05-07：M4/M6 当前合并状态最终 editor 检查通过：`cargo check -p zircon_editor --lib --tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-m4-menu-overflow-dynamic --message-format short --color never` exit 0。剩余输出仍是既有 unused warning，没有新增编译错误。
- 2026-05-07：M5/M6/M8 关闭证据已归档到 `tests/acceptance/ui-m8-final-cleanup-acceptance.md` 和 `docs/ui-and-layout/editor-host-final-cleanup.md`。真实 editor 运行截图为 `E:\zircon-build\gui-validation\editor-throttled-workbench.png`，日志只保留 `active_filter=log` 级别；隔离 target 验证通过：`cargo test -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-lib-fresh-m8 --message-format short --color never` 1115 passed / 0 failed / 3 ignored，`cargo test --workspace --locked --jobs 1 --target-dir D:\cargo-targets\zircon-root-fresh-m8 --message-format short --color never` exit 0，`cargo test --manifest-path zircon_plugins\Cargo.toml --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugins-m8-final --message-format short --color never` exit 0。旧 `D:\cargo-targets\zircon-m8-current` 曾产生 false-positive `zircon_runtime_interface` type split，已用 clean target 复跑证明不是源码合同分裂。

## Milestone Status
- **M0 基线与回归复现**：已由 GUI 截图 gate、真实 editor 运行截图和日志节流证据覆盖。
- **M1 窗口/抽屉单例数据层**：已由 bottom drawer canonical 化、registry focused tests、window_registry filter 和 production preference registry/chrome mirror 证据覆盖。
- **M2 抽屉语义与拖拽**：已由 drawer detach、shared drag/drop lifecycle、capture/high precision cleanup 和 drag release visual artifact 证据覆盖。
- **M3 主框架、标题栏与关闭保护**：已由 M3 GUI artifact、dirty close prompt Save/Discard/Cancel 证据、native route/painter broad gate 覆盖。
- **M4 多级菜单与任务栏修复**：已由 popup bounds/multi-column、overflow production preference、tree route/native painter、nested visual artifact、动态 top-level slots 和 menu-bar horizontal scroll 证据覆盖。
- **M5 输入、事件与 Material 控件闭环**：已由 runtime input ownership、native input translation、Material/SVG painter 相关 gates 和 M8 acceptance 归档覆盖。
- **M6 渲染与性能**：已由 final cleanup acceptance、真实 editor throttled screenshot/log、fresh editor/root/plugin targets 证据覆盖。
