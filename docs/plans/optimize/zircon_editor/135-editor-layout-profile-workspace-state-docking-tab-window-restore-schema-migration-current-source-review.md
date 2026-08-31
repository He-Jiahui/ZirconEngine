---
related_code:
  - zircon_editor/src/ui/workbench/layout
  - zircon_editor/src/ui/workbench/layout_preset.rs
  - zircon_editor/src/ui/workbench/layout_preset
  - zircon_editor/src/ui/workbench/view
  - zircon_editor/src/ui/workbench/window_registry
  - zircon_editor/src/ui/workbench/preset
  - zircon_editor/src/ui/workbench/project
  - zircon_editor/src/ui/host/layout_commands.rs
  - zircon_editor/src/ui/host/layout_persistence.rs
  - zircon_editor/src/ui/host/workspace_state.rs
  - zircon_editor/src/ui/host/window_host_manager.rs
  - zircon_editor/src/ui/host/layout_hosts
  - zircon_editor/src/ui/retained_host/tab_drag
  - zircon_editor/src/ui/retained_host/callback_dispatch/layout
  - zircon_editor/src/ui/retained_host/floating_window_projection.rs
  - zircon_editor/src/ui/retained_host/app/native_windows
  - zircon_editor/src/ui/retained_host/app/native_window_close
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/native_window_presenters
  - zircon_runtime/src/core/framework/foundation/config_manager.rs
  - zircon_runtime/src/foundation/runtime/config_manager.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/12-settings-preferences-scope-persistence-locale-i18n-appearance-plugin-extensibility-review.md
  - docs/plans/optimize/zircon_editor/13-layout-profile-workspace-state-docking-tab-window-restore-migration-review.md
  - docs/plans/optimize/zircon_editor/126-editor-builtin-view-window-descriptor-catalog-content-provider-capability-template-localization-current-source-review.md
  - docs/plans/optimize/zircon_editor/127-editor-workbench-shell-autolayout-constraint-language-responsive-region-binding-geometry-current-source-review.md
  - docs/plans/optimize/zircon_editor/134-editor-settings-preferences-scope-persistence-locale-i18n-appearance-plugin-extensibility-current-source-review.md
  - docs/plans/optimize/zircon_runtime/45-preference-settings-scope-storage-overlay-bounded-io-generation-fence-durability-migration-multi-process-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/55/failure-2026-08-24-config-manager-domain-error-consumer.md
  - docs/plans/zircon_editor/editor_ui/08/failure-2026-07-17-layout-metadata-full-recompute-and-sync-io.md
  - docs/plans/zircon_runtime/runtime/02/failure-2026-07-18-config-manager-synchronous-full-file-rewrite.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/TabManager.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Docking/LayoutService.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/WorkflowOrientedApp/WorkflowTabManager.cpp
  - dev/godot/editor/docks/editor_dock_manager.cpp
  - dev/godot/editor/gui/window_wrapper.cpp
  - dev/godot/editor/editor_node.cpp
  - dev/godot/editor/settings/editor_layouts_dialog.cpp
  - dev/Fyrox/fyrox-ui/src/dock/config.rs
  - dev/Fyrox/fyrox-ui/src/dock/mod.rs
  - dev/Fyrox/editor/src/settings/windows.rs
  - dev/Fyrox/editor/src/lib.rs
  - dev/bevy/crates/bevy_window/src/window.rs
  - dev/bevy/crates/bevy_winit/src/winit_windows.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Properties/AdvancedProperties.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Debugging/DebugWindow.cs
refreshes:
  - docs/plans/optimize/zircon_editor/13-layout-profile-workspace-state-docking-tab-window-restore-migration-review.md
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
evidence_captured_at: 2026-08-26T13:51:45+08:00
---

# 135 · Editor Layout Profile、Workspace State、Dock/Tab/Window Restore 与 Schema Migration 当前源码复核

## 1. 结论

Editor13指出的五个高危断点在当前源码中仍然全部存在：layout command会先detach再验证目标；Reset会先清toolkit而不进入dirty document decision；project workspace restore先破坏live state再逐项恢复；page preset继续以有损semantic结构重写精确dock topology；workspace/preset仍没有统一的bounded parse、schema migration、validator、placeholder、LKG与quarantine边界。这不是“还缺一些布局功能”，而是layout authority尚未具备失败原子性、升级兼容性和用户数据保护资格。

当前源码也出现了真实进展，必须在重构中保留：drawer region extent会先验证全部slot再成组修改；geometry-only command减少normalize/metadata重算；Runtime `ConfigManager`已有debounce、atomic writer、`flush(timeout)`和persistence report；native dirty-close prompt增加异步save、generation recheck与discard fencing；workspace诊断至少携带path/message并能进入startup status。这些改进降低了局部风险，但没有把layout、registry、document toolkit、plugin payload与native presenter纳入同一事务。

本轮保留Editor13的finding ID并重新判定当前状态：5个P0全部Open；P1为56 Open、4 Partial、0 Closed；P2为12 Open。新增`E-LAYOUT-P1-59`与`E-LAYOUT-P1-60`，分别追踪native child window在配置前先显示，以及公共`restore_workspace`只替换raw layout、未同步staging registry/session/payload/native target。32个验收门继续有效。本文是currentness刷新，不重复增加canonical finding总数，也不把静态test inventory写成动态通过。

## 2. 审查边界与可复验证据

### 2.1 物理范围

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 本轮证据 |
|---|---:|---|
| Layout model | **80 / 6,286 / 5,695 / 215,112 / 42 / 7** | layout/view/window registry/preset完整递归集合；fingerprint `b90da5b4f3d014561af8004df9c46d32034919f447fe7c27485a2ed0f9c027c8` |
| Persistence与host | **37 / 4,332 / 3,904 / 152,035 / 31 / 2** | workspace/project/preset/host及Runtime ConfigManager selected closure；fingerprint `a4b9739fbd5a3a2792e14c1842e05a7f91fd4add1c2b36b5095966817137e950` |
| Retained drag与native | **36 / 3,834 / 3,542 / 132,716 / 25 / 0** | tab drag、layout callback、floating projection、native target/presenter/close；fingerprint `d324748bca2df0665e2b66b8f03f9db8ce33b484a18a2a15d295622290fbbd10` |
| Focused tests | **30 / 5,001 / 4,550 / 178,167 / 103 / 1** | layout command/roundtrip/preset/workspace/registry/native presenter selected tests；fingerprint `32ec9253f8b3049f370b347f7c2de8e54b4a15b0cb0a7b0baa51028b03570187` |
| Zircon selected union | **183 / 19,453 / 17,691 / 678,030 / 201 / 10** | 上述四组去重源码集合；fingerprint `58bde2a5e7fbbc27a0ee938204c4ee945d6cae6e5ea97dff3c079f5e2e36c8d3` |
| Reference engines | **15 / 23,784 / 20,603 / 894,831 / 3 / 0** | Unreal/Godot/Fyrox/Bevy/Unity Graphics selected evidence；fingerprint `ee4129b0ee266561885d9da0da8b27e11455b25b196b4db33e05d17b070c380a` |
| Plan/docs evidence | **14 / 4,219 / 3,314 / 529,348 / 0 / 0** | engine/editor/runtime owner reports及3份failure handoff；fingerprint `3a6233821de8979e4a32ee14085edf2b7569cf01adda790fd842802434c9f849` |
| Zircon/reference/docs union | **212 / 47,456 / 41,608 / 2,102,209 / 204 / 10** | `2026-08-26T13:51:45+08:00`捕获的去重集合；fingerprint `b156cd6fc27e87e0195b8006c4b4bfee270328230886daaa64c9bcba274ac6e9` |

fingerprint算法与Editor134一致：相对路径排序，对每个文件计算SHA-256，再对`path + NUL + per-file hash + LF`的UTF-8 manifest计算SHA-256。它标识本轮实际阅读集合，不是layout schema hash或兼容性承诺。

### 2.2 Test inventory与在途隔离

selected Zircon union包含201个`#[test]`属性和10个ignored。9个ignored属于release/Windows-native performance evidence，另1个是managed release benchmark；它们不是默认测试门。现有测试覆盖happy path、typed error的一部分、serde roundtrip、preset precedence、native target投影和presenter stale hide，但没有证明command error no-op、dirty reset、Nth restore rollback、unknown plugin placeholder、恶意输入budget、crash consistency、双进程竞争、monitor hotplug或DPI迁移。

成文时相关范围有28条tracked/untracked在途路径，包括layout command/apply/normalize、layout preset、view/window registry、project preset assets、native close、native presenter store、floating projection和tab drag。本报告不回退、不格式化、不暂存这些更改。实施前必须重取所有source、fingerprint、schema fixture与动态测试结果，因此`source_recheck_required=true`。

证据等级：

- E3：`binding/callback -> LayoutCommand -> host side effect -> LayoutManager::apply -> normalize/recompute -> retained/native projection`逐分支阅读。
- E3：`project open/save -> workspace envelope -> instance restore -> UI asset second phase -> session/window metadata`逐函数阅读。
- E3：Unreal/Godot/Fyrox的dock schema、unknown tab/dock、延迟保存、named layout和monitor-aware restore逐段对照。
- E2：Bevy只提供logical/physical、monitor、scale factor和window mode abstraction；Unity Graphics checkout不含Unity Editor核心layout authority，只提供package preference migration与EditorWindow局部材料，不能外推完整Unity布局行为。
- 未执行：Cargo、真实Editor、磁盘/崩溃/双进程、插件缺失/重载、多显示器/DPI、键盘/读屏/UIA和规模性能动态验证。

## 3. 当前生产链与必须保留的基础

### 3.1 Layout command不是事务

`OpenView`/`MoveView`/`AttachView`共用“先detach、后attach”路径，`CreateSplit`也先detach再解析workspace path。host `open_view`先在registry创建descriptor instance并插入session，再调用attach；任何后续错误都没有undo record。新的drawer region extent实现会在写入前验证所有sibling slot，这是正确的局部prepare模式，但尚未扩展到view、registry、focus、toolkit或native target。

`LayoutDiff`目前只有`changed: bool`，不能表达受影响authority、precondition generation、prepared resources、commit receipt或rollback。geometry-only命令跳过normalize/metadata重算降低了热路径开销，但正确性仍依赖调用者自己知道哪些side effect必须补做。

### 3.2 Workspace与preset有多套authority

project workspace捕获raw `WorkbenchLayout`、open instances、focus和另一份`active_drawers`。读取外层document会验证`format_version == 1`，但内层`layout_version`固定写1且从不消费。`apply_project_workspace_state`先clear toolkits/registry/animation/UI asset/dependency state，再逐实例restore；UI asset session还在返回后执行第二阶段恢复。

global default、config preset map和page user layout存于Runtime `ConfigManager`；project named preset则写asset文件。Runtime manager已有异步I/O lane、debounce、atomic writer、flush与persistence report，但layout保存命令只调用`set_value`并立即返回，没有等待durable receipt。project preset继续raw write、先写文件后import；load忽略asset version/name并直接替换session layout。

page切换仍使用硬编码user `default`：先保存旧page有损preset，激活新page并recompute，再恢复新page preset。preset仅保留drawer mode/整数extent和`SingleDocument`/`Split(axis, panes)`语义，应用时把tabs先折叠再构造right-deep empty leaf tree，无法roundtrip真实split ratio、leaf tab allocation、tab order与active tab。

### 3.3 Native窗口是投影，不是可恢复authority

`WindowHostManager`只保存handle、bounds和surface。recompute会把layout/shared frame同步为requested bounds，未观察到OS move/resize/DPI/monitor callback反向提交logical placement。native presentation直接round/cast `f32`到physical坐标，没有per-monitor scale转换。

`NativeWindowPresenterStore`先移除stale map记录再`hide()`；创建时`UiHostWindow::new -> on_close -> on_create -> show -> insert`，随后才调用`apply(window, target)`。这会让新窗口在完整presentation与geometry配置前可见，也让第N个hide/show/backend失败留下部分topology。现有dirty close prompt具备异步save、generation recheck、discard fencing和project/main/floating target，是正确基础，但floating window仍按instance循环close，尚未形成all-or-nothing window decision。

### 3.4 必须保留的工程基础

- typed page/window/view/instance ID、recursive `DocumentNode`、typed command/drop target和descriptor metadata应继续作为领域词汇。
- `DockPolicy`、`PersistenceKeyPolicy`、single/multi descriptor和owner metadata已有声明，重构重点是统一执行而不是再建第二套描述系统。
- `LayoutDiff`、generation、retained projection cache、native target/presenter reuse可升级为transaction commit result的consumer。
- workspace atomic store、Runtime ConfigManager async I/O/flush/report和close decision fencing应被复用，不应降级回同步整文件热路径。
- builtin shell repair与normalize可以成为validator后的deterministic repair pass，但不得继续承担raw input admission或事务回滚职责。

## 4. Currentness状态总表

| 等级 | Open | Partial | Closed | 结论 |
|---|---:|---:|---:|---|
| P0 | 5 | 0 | 0 | 原Editor13五个破坏性断点均未关闭 |
| P1 | 56 | 4 | 0 | P1-25/38/45/58有局部基础，新增P1-59/60 |
| P2 | 12 | 0 | 0 | 产品管理、兼容运维与规模资格仍为空白 |

`Partial`只表示当前源码出现可复用基础，不表示finding完成。`Closed`要求对应动态门、故障注入和跨authority不变量全部通过；本轮没有任何finding达到该标准。

## 5. P0：数据保护、状态原子性与升级资格

### E-LAYOUT-P0-01 · Open · Command失败会先破坏原布局

Move/Open/Attach/CreateSplit在目标存在性、path、policy和registry一致性完成验证前detach。host Open又提前登记instance/session。无效drop即可丢失原placement或制造orphan。需要pure candidate、precondition generation、prepare/commit/abort与逐authority no-op断言。

### E-LAYOUT-P0-02 · Open · Reset layout绕过dirty document decision

Reset先`clear_document_toolkits()`再替换default layout；该clear不等价于Editor02的save/discard/cancel transaction。布局偏好操作不应拥有静默丢弃document会话的权限。Reset/import/page switch/restore必须先计算closing set并进入同一decision barrier，cancel时零提交。

### E-LAYOUT-P0-03 · Open · Workspace restore采用destructive incremental apply

project restore与bootstrap都先清live registry/session/toolkit/dependency，再逐实例/asset恢复。第N个descriptor、payload、plugin codec或UI asset失败时旧workspace已消失，新workspace只完成一部分。必须在detached staging world完成全部resolve/lease/validate，再单generation swap。

### E-LAYOUT-P0-04 · Open · Page preset有损重写dock topology

semantic preset不是完整layout snapshot，却被page switch当作roundtrip恢复使用。split ratio、精确树、leaf tab allocation、tab order和active tab都会丢失；不存在page ID仍可被激活。必须区分完整profile、受约束template与presentation preference，并为每种类型定义不可混用的schema/command。

### E-LAYOUT-P0-05 · Open · Restore没有完整validator/migration/LKG边界

workspace、project preset与ConfigManager layout以raw serde进入live state；内层version未迁移，preset version/name未验证，递归/数量/payload/float无budget，unknown plugin无placeholder。需要bounded admission、pure migration registry、structural/semantic validator、deterministic repair、LKG、quarantine与用户可执行诊断。

## 6. P1：工程级完整性差距

### 6.1 Layout model、command与policy

| ID | 状态 | 当前差距与所需重构 |
|---|---|---|
| E-LAYOUT-P1-01 | Open | `ResizeSplit`用`clamp`但不拒绝NaN；command与deserialize入口都必须先验证finite，再验证范围。 |
| E-LAYOUT-P1-02 | Open | drawer extent只有最小值，没有finite、window-relative上限、content minimum和region budget。新的region原子slot验证只解决部分写入顺序。 |
| E-LAYOUT-P1-03 | Open | `ActivateMainPage`不验证page存在、enabled/owner/capability，非法ID可进入authority。 |
| E-LAYOUT-P1-04 | Open | core `SavePreset/LoadPreset`仍返回unchanged，副作用在host执行；同一command跨入口语义不统一。 |
| E-LAYOUT-P1-05 | Open | `DockPolicy`未由manager、host、binding和drag/drop统一执行，descriptor声明不是安全边界。 |
| E-LAYOUT-P1-06 | Open | `PersistenceKeyPolicy`仍无capture/restore consumer，stable instance identity合同缺失。 |
| E-LAYOUT-P1-07 | Open | `DragPayload.kind`未参与完整target eligibility，document/tool/activity不能形成一致drop约束。 |
| E-LAYOUT-P1-08 | Open | `DropTarget::NewFloatingWindow`没有完整生产创建route；领域能力、hit target和backend能力不一致。 |
| E-LAYOUT-P1-09 | Open | reflection对floating支持未结合descriptor、用户偏好、平台与headless capability。 |
| E-LAYOUT-P1-10 | Open | binding接受raw IDs，缺descriptor existence、owner、source/target host、policy与generation验证。 |
| E-LAYOUT-P1-11 | Open | restore可为non-multi descriptor产生多个instance，single index最后写入者覆盖前者。 |
| E-LAYOUT-P1-12 | Open | duplicate instance ID restore可静默覆盖，缺collision diagnostic与owner namespace。 |
| E-LAYOUT-P1-13 | Open | `DocumentNode::remove_instance`短路递归，重复placement只移除第一处，无法恢复唯一性。 |
| E-LAYOUT-P1-14 | Open | Tabs root与`[0]`存在兼容解释，path不是单义stable address，不适合作为持久化patch identity。 |
| E-LAYOUT-P1-15 | Open | recursive tree无depth/node/leaf/tab budget，parse/normalize/restore存在栈与内存放大。 |
| E-LAYOUT-P1-16 | Open | raw serde可构造空/重复tab、unknown instance、非法active tab与无效ratio，领域不变量未在admission成立。 |
| E-LAYOUT-P1-17 | Open | normalize只修drawer active/legacy mirror，不处理空split、单子树、重复placement、focus、numeric与policy。 |
| E-LAYOUT-P1-18 | Open | `LayoutNormalizationReport.placeholders`仍没有production填充，unknown view preservation只是API形状。 |
| E-LAYOUT-P1-19 | Open | `collect_instance_hosts`用map insert静默覆盖重复宿主，recompute无法诊断多placement。 |
| E-LAYOUT-P1-20 | Open | metadata删除known-unplaced instance，却保留layout中的unknown ID；没有统一reconcile policy与report。 |
| E-LAYOUT-P1-21 | Open | legacy `drawers`、activity-window drawers与workspace `active_drawers`仍是三份authority。 |
| E-LAYOUT-P1-22 | Open | `activity_windows()`可返回临时canonical clone而不提交repair，读取结果与persisted state可能不同。 |
| E-LAYOUT-P1-23 | Open | page switch按save old -> activate -> recompute -> restore执行，后段失败会留下部分切换。 |
| E-LAYOUT-P1-24 | Open | `close_view`先改变layout/session/registry再commit document close；commit失败可造成lease与UI分叉。 |
| E-LAYOUT-P1-25 | Partial | native dirty prompt已有async save、generation recheck与discard fence；但floating window仍逐instance close，后一个失败时前面已关闭。 |

### 6.2 Workspace、profile与preset persistence

| ID | 状态 | 当前差距与所需重构 |
|---|---|---|
| E-LAYOUT-P1-26 | Open | `ProjectEditorWorkspace.layout_version`固定写1但不校验、不迁移，是无效兼容字段。 |
| E-LAYOUT-P1-27 | Open | 用户window/dock状态默认进入project `.zircon`，个人profile与团队project scope混合。 |
| E-LAYOUT-P1-28 | Open | layout只随project save捕获，无独立dirty generation、debounced autosave和shutdown deadline。 |
| E-LAYOUT-P1-29 | Open | workspace/preset仍`read_to_string`后raw serde，无file bytes、nesting、string、entity与payload budgets。 |
| E-LAYOUT-P1-30 | Open | missing rollback/delete使用直接`remove_file`，没有directory sync、tombstone或journal等价语义。 |
| E-LAYOUT-P1-31 | Open | workspace与scene仅运行时compensation，不是crash-atomic generation bundle。 |
| E-LAYOUT-P1-32 | Open | preset sanitizer会把不同名称映射到同一文件，空名归一为`preset`，无collision admission。 |
| E-LAYOUT-P1-33 | Open | project preset继续raw `fs::write`，没有temp/create-new/flush/rename/parent sync。 |
| E-LAYOUT-P1-34 | Open | preset先写文件再import；import失败时命令报错但磁盘已有side effect，缺transaction receipt/recovery。 |
| E-LAYOUT-P1-35 | Open | preset load忽略`format_version`与`preset_name`，无法阻止future schema和path/content identity错配。 |
| E-LAYOUT-P1-36 | Open | loaded project/config preset直接替换session layout，未经过validator、repair、placeholder与staged resources。 |
| E-LAYOUT-P1-37 | Open | global/default存于Runtime ConfigManager，与Editor settings/profile scope、diagnostic和migration authority分裂。 |
| E-LAYOUT-P1-38 | Partial | Runtime ConfigManager已有debounce、atomic writer、`flush(timeout)`和report；layout caller仍在`set_value`后立即报告成功，没有layout ticket/durable receipt。 |
| E-LAYOUT-P1-39 | Open | global default load把service unavailable、missing与parse error都折叠为None，缺LKG/quarantine/repair UX。 |
| E-LAYOUT-P1-40 | Open | config preset map中一个malformed entry可使整张map失败，缺per-entry version、checksum与隔离。 |
| E-LAYOUT-P1-41 | Open | page profile硬编码user `default`，没有真实local/account profile identity、scope migration与logout语义。 |
| E-LAYOUT-P1-42 | Open | UI asset workspace在base restore后第二阶段apply，失败仍形成半恢复状态。 |
| E-LAYOUT-P1-43 | Open | `bootstrap_default_layout`先清live state再构建replacement，与project restore共享destructive模式。 |
| E-LAYOUT-P1-44 | Open | missing/disabled plugin descriptor无opaque bounded placeholder、owner/version lease与rebind路径。 |
| E-LAYOUT-P1-45 | Partial | workspace错误已有source path/message并可投影startup status；仍缺schema generation、JSON path、stable code、repair/fallback reason、quarantine与action。 |
| E-LAYOUT-P1-46 | Open | User/Project/Session/Default profile precedence、锁定、共享模板与本机恢复语义没有统一产品合同。 |

### 6.3 Native window placement、presentation与生命周期

| ID | 状态 | 当前差距与所需重构 |
|---|---|---|
| E-LAYOUT-P1-47 | Open | `WindowHostManager`只有handle/bounds/surface，无monitor stable ID、saved work area、DPI、state、z、topmost、parent/modal。 |
| E-LAYOUT-P1-48 | Open | 未观察到OS move/resize反向写回layout；recompute会再次投影requested frame，用户移动不是authoritative event。 |
| E-LAYOUT-P1-49 | Open | logical `f32`直接round/cast physical，未使用per-monitor scale factor与logical/physical DTO。 |
| E-LAYOUT-P1-50 | Open | bounds只要求width/height为正，不检查x/y/size finite、最小可见标题区和current work area。 |
| E-LAYOUT-P1-51 | Open | autolayout clamp只约束shared frame；已有positive host bounds可绕过统一placement policy。 |
| E-LAYOUT-P1-52 | Open | monitor消失、分辨率/DPI/taskbar work area变化没有old usable rect到new usable rect映射。 |
| E-LAYOUT-P1-53 | Open | minimized/maximized/fullscreen与normal rect未持久化，恢复state machine未定义。 |
| E-LAYOUT-P1-54 | Open | presenter第N个hide/create/show/apply失败会留下部分topology，无prepare、receipt与reconcile report。 |
| E-LAYOUT-P1-55 | Open | stale presenter先从maps移除再hide，hide失败后authority已忘记backend object，缺ack/retry/backoff。 |
| E-LAYOUT-P1-56 | Open | multi-window没有用户偏好、platform capability、headless/remote session与安全策略gate。 |
| E-LAYOUT-P1-57 | Open | window ID无backend epoch/incarnation，迟到close/move/scale callback可能命中新同ID窗口。 |
| E-LAYOUT-P1-58 | Partial | close prompt已有project/main/floating target与异步decision基础；native/layout/app shutdown仍无统一DAG、deadline和single commit。 |
| E-LAYOUT-P1-59 | Open | 新建`UiHostWindow`在`apply(window,target)`前先`show()`；窗口可能以默认geometry/空presentation闪现，失败也会留下可见半配置对象。 |
| E-LAYOUT-P1-60 | Open | 公共`restore_workspace`只交换raw `WorkbenchLayout`并recompute metadata，不staging/reconcile registry、instances、payload、toolkit或native target；缺失instance会被裁剪，unknown layout ID仍可残留。 |

## 7. P2：产品化、兼容运维与长期质量

| ID | 状态 | 差距 |
|---|---|---|
| E-LAYOUT-P2-01 | Open | 缺Layout Manager：创建、复制、重命名、删除、导入、导出、覆盖确认、恢复默认。现有Window菜单列表不等价。 |
| E-LAYOUT-P2-02 | Open | 缺apply前layout diff预览，用户看不到将关闭、移动、repair或placeholder的panel。 |
| E-LAYOUT-P2-03 | Open | 缺profile/page metadata：last used、producer build、schema、compatibility、owner与scope。 |
| E-LAYOUT-P2-04 | Open | 缺可搜索layout diagnostics/history、LKG/quarantine浏览与support bundle导出。 |
| E-LAYOUT-P2-05 | Open | 缺keyboard-only docking、screen reader role/state、focus restore与高对比drop target门。 |
| E-LAYOUT-P2-06 | Open | 缺RTL、长文本、locale切换后tab/window title重排和geometry稳定性验证。 |
| E-LAYOUT-P2-07 | Open | 缺深树拒绝、10k tabs、100 windows、100 placeholders与restore latency/peak memory预算。 |
| E-LAYOUT-P2-08 | Open | 缺本地可控的migration/repair/fallback/restore/backend failure telemetry，且需禁止文档内容进入指标。 |
| E-LAYOUT-P2-09 | Open | 缺multi-process profile lock、CAS/merge、conflict copy与writer identity。 |
| E-LAYOUT-P2-10 | Open | 缺cloud roaming边界、加密、敏感payload过滤、离线冲突与退出账户语义。 |
| E-LAYOUT-P2-11 | Open | 缺覆盖所有历史schema、future version、坏文件和真实升级样本的golden corpus。 |
| E-LAYOUT-P2-12 | Open | 缺developer inspector显示canonical tree、owner、host、policy、generation、placement source与pending receipt。 |

## 8. 参考引擎对照

| 参考 | 当前源码证据 | Zircon应吸收 | 边界 |
|---|---|---|---|
| Unreal `TabManager`/`LayoutService`/`WorkflowTabManager` | 精确recursive area/stack/tab结构；保存window placement/size/maximized；保留collapsed/invalid area和unknown tab；deferred save ticker；layout extension、legacy tab mapping与versioned layout name | exact topology、unknown owner preservation、deferred persistence、version discipline、workflow hook | 其singleton、INI/JSON历史与隐式全局状态不能作为Zircon事务上限 |
| Godot `EditorDockManager`/`WindowWrapper`/layouts dialog | per-dock owner save/load；slot/selection/split offset/floating restore；delayed save；multi-window capability gate；按saved/current usable rect与screen fallback恢复 | extension-owned bounded state、named layout UX、monitor/work-area remap、minimized fallback | 直接Control tree mutation与ConfigFile key布局不能替代candidate transaction |
| Fyrox docking/editor settings | typed recursive split/tab/floating descriptor；window position/size/maximized persistence | 精确递归schema、简单typed descriptor与window state | restore仍偏clear/rebuild，不足以证明失败原子性 |
| Bevy window/winit | 明确logical/physical resolution、scale factor override、monitor selection、window mode | backend DTO、monitor abstraction、scale conversion | 没有Editor docking/profile/migration产品链 |
| Unity Graphics package | `AdvancedProperties`展示EditorPrefs migration flag；`DebugWindow`是package EditorWindow材料 | 显式setting migration与package-owned lifecycle的局部例证 | checkout不含Unity Editor核心layout源码；本轮也未在selected core package找到可验证的完整dock/layout authority，禁止外推 |

参考源码是设计输入，不是合规上限。目标应组合Unreal的unknown-tab/version discipline、Godot的monitor-aware restore、Fyrox的精确递归描述和Bevy的scale abstraction，再补上这些局部实现普遍未完整证明的staged transaction、resource budget、fault injection、durable receipt和跨authority reconciliation。

## 9. 目标架构

### 9.1 Authority分层

| Authority | 持有内容 | 禁止混入 |
|---|---|---|
| `LayoutSchema` | versioned page/window/tree/tab/drawer与structural invariants | native handle、live plugin object、document payload |
| `UserLayoutProfile` | named layouts、page layouts、tool visibility、window placement | project共享scene状态与open document内容 |
| `ProjectWorkspaceState` | project-relative open documents/toolkits、project context | 用户显示器坐标和个人panel尺寸 |
| `WindowPlacementProfile` | monitor stable ID、saved work area、logical rect、scale、normal/min/max/fullscreen、parent/z | authoritative dock tree |
| `ViewInstanceSnapshot` | descriptor/instance/owner/version/persistence key、bounded opaque payload | live pointer与unbounded plugin data |
| `LayoutSession` | 单一validated committed generation、registry/toolkit/native bindings | 直接充当磁盘document |

### 9.2 Prepare/commit协议

```text
bytes / command / preset / OS placement event
  -> bounded admission
  -> schema dispatch + pure migration
  -> structural and semantic validation
  -> descriptor/policy/owner resolution
  -> placeholder or typed rejection
  -> stage layout + registry + toolkit + payload + native targets
  -> dirty-document decision barrier
  -> one authority swap with precondition generation
  -> projection delta + persistence ticket + diagnostic receipt
```

失败必须保持旧layout、registry、toolkit、focus、generation、native topology与LKG不变。transaction result至少包含changed domains、old/new generation、repair report、pending/durable persistence状态和projection delta。不能继续以“先改live state，再由caller补偿”作为内部协议。

### 9.3 Schema、migration与placeholder

- canonical writer只输出当前schema；legacy drawer/activity fields只允许存在于migration input。
- ID唯一，tree/node/tab/window/string/payload/file bytes有明确budget；所有geometry必须finite并满足policy。
- unknown/missing plugin保存owner/version/persistence key、opaque bounded payload与原placement；插件恢复后通过codec rebind。
- migration必须pure、ordered、幂等、可golden测试；future version只读拒绝，不能被当前writer覆盖。
- validator输出stable code、JSON path、severity、provenance、repair action；repair确定、幂等、可预览。

### 9.4 Monitor-aware placement

持久化logical rect、monitor stable identity、saved usable work area、scale factor、normal rect与window state。恢复先选monitor，再把旧usable rect映射到新usable rect，保证最小可见标题区域，最后由backend转换physical。OS move/resize/DPI/monitor事件带incarnation与generation回写authority并debounce保存；recompute不得每帧用旧frame覆盖用户真实位置。

## 10. 重构路线

### M0 · 冻结不变量与fixture

- 定义command error no-op、restore all-or-nothing、dirty decision、unique identity、finite geometry、unknown owner preservation不变量。
- 保存format 1、page preset、project preset、ConfigManager layout、monitor placement与坏文件golden fixtures。
- 建立focused Windows test lane，明确10个ignored performance test的独立release资格，不删除或伪装执行结果。

### M1 · `LayoutAuthority`与transaction core

- 建立pure candidate、precondition generation、prepare/commit/abort、resource lease和typed commit receipt。
- command先validate target/policy再candidate detach/attach；registry/session/focus/toolkit进入同一commit。
- Reset/close/import/switch/restore接入Editor02统一document decision。

### M2 · Canonical schema、validator与migration

- 发布canonical schema v2，移除写出端legacy duplicate authority，定义唯一ID/tree/geometry invariants。
- 建立bounded reader、pure migration registry、deterministic normalize/repair、LKG/quarantine。
- 为unknown plugin/view实现opaque placeholder与rebind codec。

### M3 · Persistence authority与durability

- 分离User profile、Project workspace、shared layout asset和Session override。
- 全部writer返回ticket/receipt，复用atomic temp/flush/rename/parent sync并增加debounce/shutdown deadline。
- workspace+scene使用manifest/generation journal或bundle commit，启动只选择完整代际。

### M4 · Safe restore与Layout Manager

- restore在detached registry/toolkit/native staging中完成，成功后single swap。
- page layout保存精确topology或显式patch，semantic template不得冒充snapshot。
- 实现named layout CRUD、preview diff、collision/overwrite、import/export、fallback与repair UX。

### M5 · Descriptor policy与插件生命周期

- 在command/binding/drag/reflection/restore统一执行Dock/Persistence/single-multi/owner policy。
- 插件提供state codec/migrator/validator；disable/unload变placeholder，reload可rebind。
- policy从immutable effective snapshot投影，所有入口使用同一generation。

### M6 · Native window transaction

- 配置geometry/presentation/callback后再show；presenter create/hide/apply返回backend receipt并可reconcile。
- 扩展monitor/DPI/work-area/state/incarnation模型，OS callback反向提交logical placement。
- native close、layout close与app shutdown共享window/document decision DAG与deadline。

### M7 · 规模、兼容与运维资格

- 历史schema corpus、future version、恶意输入fuzz、Nth-operation fault、crash/restart与双进程竞争。
- multi-monitor/DPI/hotplug、plugin missing/reload、dirty documents与上限fixture矩阵。
- restore latency、peak memory、repair/fallback/backend failure和durability receipt可归档、可导出。

依赖顺序是M0 -> M1 -> M2 -> M3 -> M4/M5 -> M6 -> M7。不得先扩展更多preset UI或格式，再继续包装非事务core。

## 11. 验收门

1. Move/Attach/CreateSplit每一种typed error都证明layout、registry、session、focus、generation和native target byte-for-byte不变。
2. OpenView attach失败不留下instance、single-instance index、session、payload lease或projection orphan。
3. Reset/import/restore/page switch遇到dirty document统一进入save/discard/cancel，cancel零副作用。
4. 第N个view、UI asset、plugin codec或native target restore失败时旧workspace完整可用，新workspace零提交。
5. page A -> B -> A无用户编辑时精确恢复split tree、ratio、tab order/allocation、active tab、drawer与placement。
6. 不存在/disabled page ID被拒绝，不能进入active state或生成preset key。
7. NaN、Infinity、负值和超界ratio/extent/frame在parse与command入口均被stable diagnostic拒绝。
8. duplicate page/window/instance/placement和ambiguous path被拒绝或确定repair，不静默覆盖。
9. depth/node/tab/window/payload/file bytes超过budget时在受控时间和内存内失败。
10. normalize/repair幂等：连续两次结果相同，第二次无diff且diagnostic稳定。
11. legacy input迁移后只写一份canonical authority，roundtrip不再生成legacy字段。
12. missing plugin保留placeholder、opaque bounded payload与placement；安装兼容plugin后无损rebind。
13. plugin payload不兼容只隔离该instance，不阻断其余workspace恢复。
14. Dock/Persistence policy在command、binding、drag/drop、reflection和restore入口结果一致。
15. non-multi descriptor所有restore/import路径最多一个instance，collision包含owner diagnostic。
16. project workspace不再覆盖用户profile；两个用户打开同project保持各自布局。
17. layout change在debounce后获得durable receipt，crash最多丢失公开上限内的变化。
18. disk full、permission、rename、flush与parent sync失败不发布“已保存”，live state标记unsaved。
19. scene/workspace任一commit点终止后，重启只加载完整同代际bundle。
20. preset非法名、空名和sanitizer collision明确拒绝/确认，不覆盖其他preset。
21. malformed ConfigManager preset只隔离该entry并保留LKG，service unavailable/missing/parse可区分。
22. schema v1经pure migration到v2，golden输出稳定；future version不被当前writer覆盖。
23. current/bad/LKG/quarantine来源在诊断与Layout Manager可见、可恢复。
24. 用户拖动/缩放native window后authority收到新logical placement并在重启恢复。
25. 100%、125%、150%、200% DPI移动时logical content size稳定、physical rect正确。
26. saved monitor消失或work area改变时映射到可用monitor，标题栏至少保持可见。
27. minimized/maximized/fullscreen恢复不产生不可见/零尺寸窗口，normal rect可找回。
28. presenter第N个create/configure/show/hide失败后可reconcile，无闪现、重复window、surface leak或迟到callback误命中。
29. multi-window关闭多个dirty document只提交一个一致decision DAG，cancel恢复全部窗口。
30. keyboard-only docking、focus restore、screen reader role/name/state和高对比drop indicator通过验证。
31. 10,000 tabs、100 windows、100 placeholders的合法上限满足restore latency与peak-memory预算。
32. Windows required lane、migration/fuzz/fault/crash/multi-monitor矩阵产出可归档报告；静态inventory不得替代动态通过。

## 12. 实施边界与currentness要求

- Editor02拥有dirty document save/close/recovery authority；本计划接入统一decision，不创建第二套dirty判断。
- Editor06拥有plugin enable/reload/owner lease；本计划定义layout placeholder、state codec与rebind接缝。
- Editor08拥有command invocation/context/remote automation；layout只消费统一policy snapshot。
- Editor12/Runtime45拥有settings/profile scope和durable persistence基础；layout profile复用ticket/flush/report，但保留独立schema/migration。
- Editor01/127拥有retained projection与shell geometry；本计划输出single committed generation和typed placement delta。
- platform monitor/window backend若位于Runtime/App，必须通过logical/physical DTO、incarnation和receipt接入，Editor不持有平台私有指针。
- `failure-2026-07-17-layout-metadata-full-recompute-and-sync-io.md`仍为Open。geometry-only局部优化不能关闭真实focus/dock/page switch的typed delta、debounce、transaction generation与sync I/O问题。

本轮仅完成review、currentness重判和重构计划，没有修改production layout/workspace/preset/window/tests，也没有运行Cargo或宣称动态资格。开始实施前必须重取28条在途路径、全部manifest指纹、schema fixture和Windows验证结果；任何source drift都优先更新finding状态与门禁，不以本报告的捕获时间替代当前事实。
