# Material UI + `.ui.toml` 全链路 UI 系统推进计划

## Summary
- 继续采用“分层全链路”路线：先修通当前 editor 编译门槛，再把 Slint Material 元组件通过 `.ui.toml` 组件资产、runtime layout、shared hit grid、editor host projection 和 native painter 串成一个可交互系统。
- 参考主线：Slint Material 负责组件结构与状态语义；Unreal Slate 负责 arranged geometry、visibility、hit grid、widget path、focus/capture、invalidation 和 fast paint 模型。
- 当前状态（2026-05-07）：editor/runtime 编译门槛、Material layout、Material meta asset、Component Showcase、native host 输入/点击、Inspector Material roots、native painter、hit-grid/event routing focused 验收已收束；world-space UI 的 shared hit-grid 接口、投影查询和 host capability gate 已完成 focused 复核；post-M8 runtime math 重复 owner warning 清理已完成源码/文档改动和 focused Cargo 验证；M0-M8 当前里程碑关闭。后续只保留非阻塞增强：长期清理 unused warning、扩大 GUI 真机脚本覆盖和更高层 world-space UI 实机路径。

## Key Changes
- 已恢复 editor 可编译：修正 `load_preview_image` 的模块可见性/落点，并把所有 `ViewTemplateNodeData` 测试构造补齐 `media_source`、`icon_name`、`has_preview_image`、`preview_image` 字段。
- 以 `material_meta_components.ui.toml` 为 Slint Material 复刻入口，保持组件展开为现有 Runtime UI 控件，不引入生成 Slint UI；补齐 Button、IconButton、CheckBox、ComboBox、LineEdit/TextEdit、Slider/SpinBox、Switch、Menu、Scroll/Table、Tab、Progress/Spinner 的 `.ui.toml` 参数、样式和绑定透传。
- 事件路径按下方“Event System”落地：所有 host pointer/keyboard/text/popup 输入先归一化为 shared route，再通过 `UiSurfaceFrame.arranged_tree + hit_grid` 生成 `UiHitPath`，最后翻译成 `UiComponentEventEnvelope` 和 `.ui.toml` binding 调用；不再为 Material 控件补 Rust 坐标表或控件专用点击分支。
- 渲染优化继续沿用 Slate 思路：layout dirty、render dirty、presentation dirty、pointer-only dirty 分层；Material 状态层、hover、press、text cursor、viewport image 和 popup 更新优先走局部 damage，不触发完整 presentation rebuild。
- Editor 接入面优先覆盖 Component Showcase、Asset Browser、Inspector surface controls、Viewport toolbar 和 pane template buttons，确保新增 Material 控件只改 `.ui.toml` 不改 Rust 坐标表。

## Public Interfaces / Types
- 保持并收敛现有共享契约：`UiVisibility`、`UiArrangedTree`、`UiHitTestGrid`、`UiHitPath`、`UiSurfaceFrame` 是 layout/render/hit/input 的共同事实。
- `TemplatePaneNodeData` 的 Material/媒体字段作为 host projection 合同保留：`surface_variant`、`text_tone`、`button_variant`、`font_size`、`corner_radius`、`border_width`、`media_source`、`icon_name`、`has_preview_image`、`preview_image`。
- `.ui.toml` 组件实例上的 bindings 必须继续透传到 expanded root control；这是 Material meta component 成为真实控件而非视觉 wrapper 的关键合同。

## Event System

### Reference Shape
- Unreal Slate 对应语义：`FHittestGrid` 只保存 arranged geometry 派生出的命中索引；`FWidgetPath` 是 root-to-leaf 路由事实；`FSlateApplication` 先根据 hit/focus/capture 生成事件路径，再沿路径 bubble/tunnel，并由 `FReply` 决定 handled、capture、focus、popup 等后续状态。
- Slint 对应语义：`TouchArea` 在 press 时 grab pointer，在 release-inside 时触发 clicked；输入项把 hover、pressed、focus、scroll、text edit、drag/drop 映射成控件属性与回调；focus chain 独立于鼠标命中，但鼠标 press 可以更新键盘 focus。
- Zircon 落点：Unreal 的 widget path 映射到 `UiHitPath`/bubble route，Slint 的控件状态映射到 `UiComponentEvent` 与 host projection 属性，editor native host 只负责把平台输入归一化并提交给 shared surface router。

### Evidence Files
- Zircon shared contracts：`zircon_runtime_interface/src/ui/component/event.rs`、`zircon_runtime_interface/src/ui/component/data_binding/event_envelope.rs`、`zircon_runtime_interface/src/ui/binding/model/event_kind.rs`、`zircon_runtime_interface/src/ui/surface/pointer/route.rs`、`zircon_runtime_interface/src/ui/surface/focus_state.rs`。
- Zircon shared behavior：`zircon_runtime/src/ui/surface/surface.rs`、`zircon_runtime/src/ui/surface/frame_hit_test.rs`。
- Zircon editor host：`zircon_editor/src/ui/slint_host/host_contract/native_pointer.rs`、`zircon_editor/src/ui/slint_host/ui/pane_data_conversion/pane_component_projection/mod.rs`。
- Unreal reference：`dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/HittestGrid.h`、`dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Layout/WidgetPath.h`、`dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Application/SlateApplication.cpp`。
- Slint reference：`dev/slint/internal/core/input.rs`、`dev/slint/internal/core/items/input_items.rs`、`dev/slint/internal/core/item_focus.rs`。

### Ownership And Boundaries
- `zircon_runtime_interface::ui` 拥有事件 DTO 和可序列化契约：pointer route、keyboard/navigation route、focus/capture/hover snapshot、component event envelope、binding event kind、drag payload、drop policy、event dispatch result。
- `zircon_runtime::ui` 拥有纯 shared 行为：根据 `UiSurfaceFrame` 命中、维护 hover/focus/capture、生成 semantic component event、执行 bubble/default-action 规则、返回 dirty/damage 建议。
- `zircon_editor::ui::template_runtime` 负责把 `.ui.toml` binding、component descriptor、public part、component focus/input contract 编译成 runtime surface 节点，不把事件语义硬编码到 editor host。
- `zircon_editor::ui::slint_host` 只做 host adapter：winit/slint native 回调归一化、surface registry 查询、调用 shared router、把 envelope 交给现有 editor adapter/command 层、按 router 返回的 damage 请求 repaint。
- 禁止新增 `Button`、`Slider`、`ComboBox`、`Toolbar` 这类控件名专用 Rust dispatch 分支；如果必须区分行为，差异来自 component descriptor 的 role/capability/default action，而不是 host 坐标或字符串特判。

### Input-To-Binding Pipeline
1. Platform input normalization：native host 把 pointer down/move/up/wheel、keyboard down/up/text、window blur、popup dismiss、drag source/drop payload 归一化成 shared event DTO，并附带 `surface_key`、global point、device/button/modifier、timestamp/repeat/click count。
2. Surface selection：host-level surface registry 按 z-order 查询 root shell、popup/menu、floating window、pane、viewport toolbar、template surface；popup surface 优先于 owner surface，capture path 优先于 pointer hit path。
3. Hit path resolution：router 使用 `hit_test_surface_frame(surface_frame, point)` 生成 `UiHitPath`、leaf target、stacked path、bubble path、rejected reason；disabled、visibility、clip、input policy 都在 shared layer 过滤。
4. State transition：router 比较旧 `UiFocusState` 与新 path，产出 hover entered/left、pressed target、captured target、focused target、active drag source、drop hover target、popup owner 等状态差异。
5. Semantic event synthesis：router 根据 control descriptor 和 binding contract 把低层输入转换为 `UiComponentEvent`：`Press`、`Hover`、`Focus`、`ValueChanged`、`Commit`、`SelectOption`、`ToggleExpanded`、`SetVisibleRange`、`BeginDrag`、`DragDelta`、`DropReference` 等。
6. Binding dispatch：semantic event 携带 `document_id`、`control_id`、`component_id`、`UiComponentBindingTarget`、binding id/route，形成 `UiComponentEventEnvelope`；editor adapter 只处理 envelope，不再根据鼠标坐标猜 action。
7. Dirty/damage result：dispatch result 返回 handled/blocked target、focus/capture/popup mutation、layout/render/input dirty flags、damage rects、diagnostic counters；pointer-only 状态变化优先只 damage old/new path frame。

### Pointer And Activation Semantics
- Hover：`Move` 只在 leaf/path 改变时发 `Hover { hovered }` 和 repaint；100 次同目标 move 必须保持 idle，不触发 presentation rebuild。拖拽或 captured pointer 例外，因为 delta 仍可能更新值。
- Press：primary/secondary/middle down 生成 `Press { pressed: true }`，记录 pressed target，并可按 control focus contract 设置 focus。可 focus 控件在 primary press 后进入 focused path；disabled 或 hit-test-invisible 控件不得更新 focus。
- Release：up 生成 `Press { pressed: false }`/`Release` binding，清理 capture。默认 `onClick` 在 primary press target 与 release-inside target 相同且未被 drag 消费时触发，匹配 Slint `TouchArea` clicked 语义。
- Immediate command：viewport toolbar 和显式 `activation = "primary_press"` 的 command control 继续在 `PrimaryPress` 触发，避免 press/release 双触发；该策略必须来自 `.ui.toml` component contract 或 binding metadata，而不是 toolbar 专用 host 分支。
- Capture：press 后需要连续 move/up 的控件可返回 capture，包括 slider、spin drag、scroll thumb、tab drag、reference drag source。capture active 时 route target 优先 captured node，但 hit path 仍用于 drop hover 和 outside-release 判断。
- Double click：保留 `UiEventKind::DoubleClick`，由 normalized click count 触发；第一阶段只要求不破坏单击，双击用于 asset row/open reference 等已有 editor 交互后续接入。

### Keyboard, Focus, And Text Semantics
- Focus path：keyboard event 默认沿 focused route bubble；无 focused node 时落到 navigation root 或 surface root。`Tab`/`Shift+Tab` 映射 `Next`/`Previous`，方向键映射 `Up`/`Down`/`Left`/`Right`，`Esc` 映射 `Cancel`。
- Activation：`Enter` 和 `Space` 在 focused button、checkbox、switch、combo、menu item 上映射 `Activate`，再由 control descriptor 转换为 `Click`、`Toggle`、`OpenPopup`、`SelectOption` 或 `Commit`。
- Text input：LineEdit/TextEdit 获得 focus 后消费 text/editing keys；普通字符、Backspace、Delete、Left/Right/Home/End 更新 value/cursor 并产生 `ValueChanged`，`Enter` 或 focus lost 产生 `Commit`。首阶段保留 IME composition 接口但不把完整 IME 作为验收主线。
- Blur：window blur、surface removal、focused control hidden/disabled、popup owner closed 都必须发 `Focus { focused: false }`，清理 capture/pressed/text cursor，并只 damage 旧 focus path。
- Focus visible：鼠标 press 设置 focus 但默认不显示 keyboard focus ring；keyboard navigation 设置 `focus_visible = true`，paint path 只 repaint focus old/new frames。

### Value, Scroll, Drag, Drop, And Popup Semantics
- Checkbox/Switch：primary click 或 focused activate 产生 `Toggle`/`ValueChanged { property: "checked" }`；只有 value 实际变化才触发 `Commit` 或 adapter mutation。
- Slider/SpinBox：press 捕获 pointer；move 计算 normalized delta 并产生 live `ValueChanged`；release 产生 `Commit`。Shift/Ctrl 等 modifier 可放大或缩小 step，但必须通过 shared normalized modifier 输入表达。
- ComboBox/Menu：press 或 keyboard activate 打开 popup surface；popup option hit 产生 `SelectOption { property, option_id, selected: true }`，随后关闭 popup 并 damage popup + owner anchor。outside primary press 先关闭 popup，不把同一个 press 继续透传给下层 surface。
- Scroll/Table/List：wheel 先给 hit leaf 的 `onScroll`；未 handled 时 bubble 到最近 scrollable candidate。VirtualList/Table 产生 `SetVisibleRange`，只更新 visible range dirty 和 viewport damage，不重建完整 presentation。
- Drag source：press 后超过 shared threshold 或 descriptor 标记 immediate drag 时产生 `BeginDrag`，随后 captured move 产生 `DragDelta`/`LargeDragDelta`，release/cancel 产生 `EndDrag`。drag payload 来自 `UiDragSourceMetadata`，不能由 host 拼字符串。
- Drop target：drag move hit path 中第一个接受 payload kind 的 target 进入 `DropHover`/`ActiveDragTarget`；release on accepted target 产生 `DropReference` 或 descriptor 指定的 drop event。rejected drop 必须返回 structured diagnostic，不静默吞掉。
- Popup lifetime：popup/menu 是独立 surface frame，注册在 owner 之上，带 owner control id。owner surface rebuild 或 owner hidden 时 popup 自动关闭；popup 关闭只 invalidates popup frame 和 owner anchor state。

### Binding Contract Rules
- `.ui.toml` binding 必须声明 event kind、binding id、route/target；Material meta component 展开后，instance binding 透传到 expanded root control，slot 内部视觉节点不得抢走公开 binding。
- Control descriptor 必须声明支持的 event kinds、focusability、capture behavior、default activation policy、value property、commit policy、drop policy。编译期发现 binding 事件不被 descriptor 支持时给出 `UiBindingDiagnostic`。
- Runtime dispatch 不允许以 `control_id.contains("Button")` 或 component 名字符串决定行为；它只能读取 descriptor capability、node role、binding event kind 和 current interaction state。
- Adapter 层只能收到 typed envelope。Inspector/Asset Browser/Showcase/ViewportToolbar 的差异保留在 adapter target domain 和 command route，不回流到 shared hit router。

### Event Diagnostics And Acceptance Counters
- 每次 routed event 可输出 `surface_key`、normalized event kind、hit leaf、bubble path、focused/captured node、binding id、component event、handled/blocked target、damage rect。
- Debug hit dump 继续包含 rejected reason，并补充事件层 skip reason：missing binding、unsupported event、disabled target、lost capture、popup consumed outside press、drop payload rejected。
- 性能计数至少包括 pointer routed、pointer ignored same target、hover enter/leave、capture begin/end、focus changed、binding envelope dispatched、popup open/close、scroll defaulted、presentation rebuild avoided、region damage emitted。

### Event Milestones
- E1 Shared event contract：补齐 keyboard/text route、pointer dispatch result、activation policy、capture/pressed/drag/drop state、diagnostic reason DTO；不触碰 editor host 坐标逻辑。
- E2 Shared router behavior：在 `zircon_runtime::ui` 中实现 hit path routing、hover diff、focus/capture、click release-inside、scroll fallback、popup outside close、drag/drop accept/reject；用 runtime tests 证明不依赖 editor host。
- E3 `.ui.toml` binding integration：Material component descriptors 声明支持事件与默认 policy；component expander 保证 bindings 透传；template projection 携带 binding ids、event kinds、focus/capture/value metadata。
- E4 Editor native host cutover：host-level surface registry 调用 shared router；pane/template/toolbar/menu/popup 事件都通过 envelope dispatch；移除新增控件需要 host 分支的路径。
- E5 Event validation and docs：集中运行 focused runtime/editor tests，更新 UI/event docs 和 acceptance 记录；不声明 workspace-wide green，只记录 scoped commands。

## Progress Evidence
- 2026-05-07 `cargo check -p zircon_editor --lib --tests --locked --jobs 1 --target-dir D:\cargo-targets\zircon-material-final-check --message-format short --color never` 通过；剩余为既有 unused warning。
- 2026-05-07 `cargo test -p zircon_runtime --lib material_layout --locked --jobs 1 --target-dir D:\cargo-targets\zircon-material-final --message-format short --color never -- --nocapture --test-threads=1` 通过，23 个 Material layout focused tests 覆盖 button/icon/input/list/menu/table/tab 等 Slint-like 尺寸语义。
- 2026-05-07 `cargo test -p zircon_editor --lib material_meta_components_cover_slint_material_exports --locked --jobs 1 --target-dir D:\cargo-targets\zircon-material-final-check --message-format short --color never -- --nocapture --test-threads=1` 通过，确认 `material_meta_components.ui.toml` 覆盖当前 Slint Material 复刻出口。
- 2026-05-07 `cargo test -p zircon_editor --lib component_showcase_uses_material_meta_component_assets --locked --jobs 1 --target-dir D:\cargo-targets\zircon-material-final-check --message-format short --color never -- --nocapture --test-threads=1` 通过，确认 Component Showcase 使用 Material meta component 资产。
- 2026-05-07 `cargo test -p zircon_editor --lib component_showcase_projection_carries_runtime_component_semantics --locked --jobs 1 --target-dir D:\cargo-targets\zircon-material-final-check --message-format short --color never -- --nocapture --test-threads=1` 通过，确认 Material 元组件展开后仍携带 runtime component semantics。
- 2026-05-07 `cargo test -p zircon_editor --lib component_showcase --locked --jobs 1 --target-dir D:\cargo-targets\zircon-material-final-check --message-format short --color never -- --nocapture --test-threads=1` 通过，19 个 Component Showcase focused tests 覆盖分类筛选、下拉/搜索、菜单、复杂绑定、collection/structure/reference projection 和 Rust-owned state。
- 2026-05-07 `cargo test -p zircon_editor --lib native_host_welcome_material --locked --jobs 1 --target-dir D:\cargo-targets\zircon-material-final-check --message-format short --color never -- --nocapture --test-threads=1` 通过，确认 Welcome Material text field 可接收 keyboard input，Material button 可回调 welcome action。
- 2026-05-07 `cargo test -p zircon_editor --lib native_host_pointer_click_routes_projected_material_showcase_button --locked --jobs 1 --target-dir D:\cargo-targets\zircon-material-final-check --message-format short --color never -- --nocapture --test-threads=1` 通过，确认 native host pointer click 能命中 projected Material showcase button。
- 2026-05-07 `cargo test -p zircon_editor --lib editor_ui_host_runtime_projects_inspector_controls_through_material_roots --locked --jobs 1 --target-dir D:\cargo-targets\zircon-material-final-check --message-format short --color never -- --nocapture --test-threads=1` 通过，确认 Inspector controls 通过 Material roots 投影。
- 2026-05-07 `cargo test -p zircon_editor --lib native_template_painter_uses_material_state_palette_for_controls --locked --jobs 1 --target-dir D:\cargo-targets\zircon-material-final-check --message-format short --color never -- --nocapture --test-threads=1` 通过，确认 native painter 消费 Material state palette。
- 2026-05-07 `cargo test -p zircon_runtime --lib hit_grid --locked --jobs 1 --target-dir D:\cargo-targets\zircon-material-final-check --message-format short --color never -- --nocapture --test-threads=1` 通过，12 个 hit-grid focused tests 覆盖 visibility/clip、disabled skip、world query、cached counts、focus/capture/hover snapshot 和 debug stats。
- 2026-05-07 `cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir D:\cargo-targets\zircon-material-final-check --message-format short --color never -- --nocapture --test-threads=1` 通过，23 个 event routing focused tests 覆盖 click envelope、release-inside/outside、focus/blur、capture、same-target hover idle、keyboard/text/IME、scroll fallback 和 repeated mouse move 不 rebuild。
- 2026-05-07 GUI 截图证据沿用 Drawer/Window/Menu Slate 验收：`E:\Git\ZirconEngine\target\visual-layout\editor-window-20260507-nested-menu-popup-900x620.png`、`E:\zircon-build\gui-validation\editor-nested-menu-live-software.png`、`E:\zircon-build\gui-validation\editor-throttled-workbench.png`。其中 software backend 截图用于避开 Windows GDI CopyFromScreen 白屏限制。
- 2026-05-07 post-M8 收尾：`cargo fmt --all -- --check` 通过；上一轮记录的 5 个格式差异文件已按 rustfmt 机械收束，不再阻塞全仓格式门禁。
- 2026-05-07 post-M8 world-space 接口约束复核：在 `F:\cargo-targets\zircon-world-space-ui-interface` 上通过 `ui_hit_metadata_contract_carries_scope_space_and_world_ray`（1 test）、`hit_grid`（12 tests）和 `component_catalog`（39 tests）。本轮确认 `with_projected_world_hit(...)` 必须携带 finite `UiWorldHitRay` + `UiVirtualPointerPosition` 才能进入 shared hit grid；缺失投影或非 finite ray 会以 `WorldHitUnavailable` 诊断拒绝；`WorldSpaceSurface` 只对带 `UiHostCapability::WorldSpaceUi` 的 host capability set 暴露。
- 2026-05-07 post-M8 runtime math warning cleanup：`zircon_runtime::core::math` 已收敛为 `zircon_runtime_interface::math` 的运行时 re-export 入口，删除 runtime-local `precision/*` 与 `transform/*` 重复私有实现；`rustfmt --edition 2021 --check zircon_runtime/src/core/math/mod.rs zircon_runtime_interface/src/math.rs` 通过；`git diff --check` 通过且仅有 LF/CRLF 提示；仍存在的 `mod precision;` 扫描结果属于 editor viewport pointer 模块，非本轮 runtime math owner；`cargo test -p zircon_runtime --test math_transform_helpers --locked --jobs 1 --target-dir F:\cargo-targets\zircon-runtime-math-warning-cleanup --message-format short --color never` 通过，3 passed；编译输出剩余 warning 位于 graphics/ui 等既有区域，不再包含已删除的 runtime-local math owner warning 组。
- 2026-05-07 editor visual density B：用户在视觉辅助页选择 B 方案后，Material meta component token 收紧到 compact/default/prominent 28/32/40，标准图标 18、按钮图标 16；workbench activity rail 收到 44/32，menu action strip 收到 24，Inspector Material controls 收到 28。轻量 TOML asset check 通过；`rustfmt --edition 2021 --check zircon_editor\src\tests\ui\boundary\material_meta_component_contracts.rs` 通过；`cargo test -p zircon_editor --lib material_meta_component_contracts --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m22-dual-host-parity --message-format short --color never` 通过 6 / 0；`cargo test -p zircon_editor --lib global_material_surface_assets --locked --jobs 1 --target-dir D:\cargo-targets\zircon-ui-m22-dual-host-parity --message-format short --color never` 通过 3 / 0；输出仍包含既有 runtime/editor warning noise。

## Milestone Status
- M0 编译门槛：已完成。`zircon_editor` lib/tests check 通过，当前门槛不再被 `view_projection.rs` 可见性或 `ViewTemplateNodeData` 字段漂移阻挡。
- M1 Shared Slate Core 合同：已完成当前 2D 验收。`UiSurfaceFrame`、`UiArrangedTree`、`UiHitTestGrid`、`UiHitPath`、`UiVisibility` 由 hit-grid/event routing focused tests 覆盖。
- M2 Material 元组件覆盖：已完成当前资产与 layout 验收。Button、IconButton、CheckBox/Switch、ComboBox、LineEdit/TextEdit、Slider/SpinBox、Menu、Tabs、Scroll/Table、Progress/Spinner 的出口由 meta asset coverage、layout tests 和 Showcase tests 约束。
- M3 `.ui.toml` 编辑器布局接管：已完成当前 Component Showcase、Inspector 和 native host Material 投影验收；Rust host 只承载投影/路由合同，不新增控件坐标表作为验收路径。
- M4 完整交互闭环：已完成 focused 交互验收。button click、keyboard/text/IME、focus/capture、same-target hover idle、scroll fallback、menu/showcase bindings、Welcome text field 和 projected button click 均有测试覆盖。
- M5 渲染与 Material 状态层：已完成 native painter focused 验收；Material state palette、popup/menu 截图和局部 dirty/rebuild 计数由 painter、event routing、GUI 截图证据约束。
- M6 Text/Edit 收敛：已完成 2D editor 输入路径验收；shared text input 与 IME preedit/commit/cancel focused tests 通过，完整平台 IME 仍按接口边界作为后续增强。
- M7 诊断、性能与 Widget Reflector：已完成当前诊断/性能 focused 验收；surface debug snapshot、hit-grid stats、same-target mouse move 不 rebuild 和 throttled workbench GUI 证据已记录。
- M8 Runtime 独立 UI 验收：已完成当前 2D runtime/editor shared-contract 验收；world-space UI 的 shared hit grid 接口、投影查询拒绝语义和 host capability gate 已用 runtime interface/runtime focused tests 复核，更高层 editor/RHI 实机 world-space 路径保留为后续增强，不阻塞本轮 M8。

## Test Plan
- 编译门槛：`cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never`。
- Runtime 底座：`cargo test -p zircon_runtime --lib hit_grid --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never`。
- Material 覆盖：`material_meta_components_cover_slint_material_exports`、`component_showcase_uses_material_meta_component_assets`、`component_showcase_projection_carries_runtime_component_semantics`。
- 交互验收：focused tests 覆盖 Material button release-inside click、toolbar primary-press command、checkbox/switch change、keyboard activate、focus/blur、text edit live change/commit、slider/spin capture drag、combo/menu popup option select/outside close、scroll/table visible range、drag/drop reference field accepted/rejected。
- 事件路由验收：focused tests 证明 hover same-target move 不派发 binding、不 rebuild presentation；capture target 优先于 hit target；disabled/hidden/clip/input-policy 节点不更新 focus；popup surface 优先命中并可消费 outside press；`.ui.toml` 新增 button 只靠 binding metadata 即可生成 envelope。
- Editor native 验收：viewport toolbar、pane template buttons、Inspector Apply/Delete、Asset Browser row hover/click、Component Showcase 全控件在 native host 下可命中并产生可见 repaint。
- 性能验收：100 次同目标 mouse move 不触发 presentation rebuild；hover/press/text cursor/popup 只产生局部 damage；日志中不再反复 template load/compile。

## Assumptions
- “尽可能复刻 Unreal”解释为复刻 Slate 架构语义和行为边界，不直接复制 C++ API 或源码结构。
- 本轮聚焦 2D editor/runtime UI；world-space UI 当前验收到 shared `UiHitTestGrid` 接口合同，更高层实机渲染/交互路径不作为第一验收主线。
- 继续使用 Rust-owned native host/painter 路线，不恢复 deleted/generated `.slint` UI 源。
- 当前工作区已有大量 UI 改动，实施时必须保留现有改动并在其上修正，不能回滚其他会话的文件。
