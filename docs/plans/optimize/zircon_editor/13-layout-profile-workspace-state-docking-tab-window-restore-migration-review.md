---
related_code:
  - zircon_editor/src/ui/workbench/layout
  - zircon_editor/src/ui/workbench/layout_preset.rs
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
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/12-settings-preferences-scope-persistence-locale-i18n-appearance-plugin-extensibility-review.md
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
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 13 · Layout Profile、Workspace State、Dock/Panel/Tab、Window Restore 与 Schema Migration 工程化差距

## 1. 结论

Zircon Editor已经形成可识别的Workbench布局领域：typed view/instance/page/window ID、递归document split tree、activity drawer/window、layout command/diff、view registry、project workspace envelope、page preset、native presenter projection，以及close prompt。这些基础证明当前实现不只是静态界面稿。

但它尚未成为可承受大型项目、插件变更、崩溃、版本升级和多显示器变化的工程级Editor layout系统。最严重的五个断点是：

1. `MoveView`、`AttachView`和`CreateSplit`先从原host移除实例，再验证目标并执行attach；失败不回滚。`open_view`又先创建registry/session记录再attach，失败会留下孤儿。普通非法drop即可损坏当前布局。
2. `ResetToDefault`先`clear_document_toolkits()`并清理asset session，再替换布局；该clear只拦截active save/closing，不检查dirty文档，也不走close decision。用户可因“重置布局”静默丢失未保存的文档会话。
3. `apply_project_workspace_state`先清空当前toolkit、view registry、session和dependency，再逐个恢复；任何descriptor、payload或UI asset失败都会让旧workspace已经消失、新workspace只恢复一部分。启动默认布局也采用先破坏后构建。
4. page user layout不是完整布局快照，而是有损`LayoutPreset`。保存只保留drawer mode/整数size和`SingleDocument`或`Split(axis, panes)`；恢复先把全部document tabs折叠到一个stack，再构造右深空叶树。普通page切换可丢失split ratio、精确树形、tab分配和active tab，而且产品允许激活不存在的page ID。
5. workspace/preset缺少统一schema validator、migration registry、resource budget、unknown-plugin placeholder和last-known-good/quarantine。外层workspace `format_version`只接受1，但内层`layout_version`写1却从不验证；preset asset甚至忽略自身version/name。坏数据可能被静默fallback、部分应用，或长期毒化启动路径。

本报告记录5个P0、58个P1、12个P2，给出M0-M7重构路线与32个验收门。目标不是复制任一参考引擎，而是建立一个可验证的`LayoutAuthority + LayoutTransaction + versioned WorkspaceBundle + monitor-aware WindowPlacement`体系。没有修改生产代码。上一轮同一工作树的`zircon_editor --lib`测试编译在617.2秒后被239个既有test-build错误和122个warning阻断，本轮没有重复相同Cargo lane；113个focused test attributes只做静态inventory，不得表述为动态通过。

## 2. 审查边界与可复验证据

### 2.1 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| layout/view/window registry/preset领域 | 78 / 5,032 / 169,585 | E3：layout model、command/apply/attach/detach/drop/normalize/restore、view policy/registry、window registry和page preset；fingerprint `507ff857...54723c4c` |
| project workspace与host persistence | 37 / 3,506 / 128,503 | E3：project envelope/store/save rollback、preset asset、host apply/capture/restore、builtin repair和window host；fingerprint `1c7b4ba9...114bb0ef` |
| retained drag/drop与native projection | 37 / 3,477 / 118,935 | E3：tab drag、layout callback、floating projection、native target/presenter/close和recompute；fingerprint `1f725ee9...3698c12a` |
| focused tests | 35 / 6,199 / 215,971 | E3静态阅读：113个test attributes、0 ignored；fingerprint `8b9cf8eb...0e47938` |
| selected combined scope | 187 / 18,214 / 632,994 | 当前工作树去重集合；fingerprint `3e3fdd86...7572f5b9` |

fingerprint按相对路径排序，将`path + NUL + per-file SHA-256 + LF`拼接后计算SHA-256。它只标识本轮阅读集合，不是layout schema、migration或兼容性hash。

focused tests仅计入布局command/roundtrip/page preset/workspace restore、registry、dock binding/tab drop和native target/presenter相关文件，排除大量只借用“layout”命名的视觉密度、material painter和模板截图测试。范围内已有测试覆盖happy path、部分typed error、raw serde roundtrip、restore precedence、native target投影和presenter stale-hide；没有覆盖失败不变式、dirty reset、Nth-item restore rollback、unknown plugin、schema migration、恶意输入budget、多进程竞争、monitor hotplug或DPI迁移。

### 2.2 在途文件与验证隔离

成文时scoped source/tests有24个文件显示并行在途修改，包括`layout/manager/restore.rs`、`preset/default_layout.rs`、`project/editor_project_document*.rs`、`project/layout_preset_assets.rs`、`retained_host/floating_window_projection.rs`、tab drag和多份focused tests。本报告没有回退、格式化、暂存或提交这些修改。实施前必须重取source、fingerprint、schema fixture和动态测试结果，因此`source_recheck_required=true`。

证据等级：

- E3：`callback/binding -> LayoutCommand -> LayoutManager::apply -> host registry/session -> recompute`逐分支闭环。
- E3：`project open/save -> workspace envelope -> staged instance restore -> UI asset restore -> layout/session metadata`逐函数闭环。
- E3：`layout frame -> WindowHostManager -> retained projection -> NativeWindowTarget -> presenter`逐函数闭环。
- E3：Unreal/Godot/Fyrox相关layout保存、恢复、未知tab/dock、窗口屏幕映射和延迟保存源码逐段对照。
- E2：Bevy只提供logical/physical size、monitor selection、scale factor的window abstraction；Unity Graphics checkout只提供package-level EditorWindow persistence opt-out和EditorPrefs migration例证，不代表Unity Editor核心布局源码。
- 未覆盖：真实Windows窗口拖移回调、跨显示器拔插、DPI动态变化、断电、磁盘满、网络目录、双Editor并发、插件卸载、十万tab/layout stress和真实升级样本。它们全部进入验收门，不冒充已验证能力。

### 2.3 本轮追踪的生产链

1. retained callback或binding构造`LayoutCommand`，host对Open/Close和Reset做额外session操作，其余直接交给`LayoutManager::apply`。
2. Move/Attach/CreateSplit在manager内先detach；目标workspace/path验证和attach发生在后，错误只返回caller，不恢复旧layout。
3. host `open_view`先让registry创建instance并写session，再执行attach；manager失败不会回滚registry/session。
4. changed command之后的normalize只修正drawer active selection和legacy drawer mirror，不验证树、ID、descriptor、policy、数值或window placement。
5. project workspace捕获raw `WorkbenchLayout`、instances、focus和另一份`active_drawers`，写入项目下`.zircon/editor-workspace.json`。
6. project open读取整个JSON，验证外层format 1，然后先清空当前workspace，再逐实例恢复；UI asset workspace在后续第二阶段继续恢复。
7. project save先写workspace，再写scene；scene失败时尝试恢复旧workspace文件。该补偿对进程崩溃或断电不构成双文件原子提交。
8. page切换把前一page按硬编码user `default`保存为semantic preset，激活新ID并recompute，再加载新page preset或Authoring fallback。
9. project named layout preset将raw workspace写到名称派生路径，随后再import；load忽略asset format/name并直接替换session layout。
10. global/default preset另存于Runtime `ConfigManager`，写入为异步eventual persistence；读取把service不可用、key缺失和parse失败都折叠为None/default。
11. floating layout frame被同步到`WindowHostManager`和native target；presenter将`f32`直接round/cast为physical position/size。未观察到OS move/resize事件把用户实际窗口位置写回`WorkbenchLayout`。
12. native close先保留窗口显示，再走host close decision，这是正确基础；但layout reset绕过此close workflow，floating window多instance关闭也不是原子操作。

## 3. 已有工程基础，重构时必须保留

### 3.1 Typed领域与可组合树

- page、activity window、view descriptor、view instance和window采用typed ID wrapper，减少裸字符串在核心结构内扩散。
- `DocumentNode`支持Tabs/Split递归表达，command、drop target和insertion anchor已经形成可扩展词汇。
- `LayoutDiff`和generation给projection提供增量更新入口，后续可作为transaction commit result而不是废弃。
- view descriptor已经声明kind、dock policy、persistence key policy和capability，缺口是消费与强制执行，不是重新发明metadata。

### 3.2 Persistence与产品骨架

- workspace外层有明确format version，project save会保留旧workspace并在scene save失败时做best-effort compensation。
- 通用workspace store使用同目录临时写、flush与rename，是后续durability层可复用基础。
- restore policy已经表达Project > Global > Default优先级，page preset也有version mismatch fallback result。
- named project layout asset、global default、page user preset和reset command已经形成产品入口骨架。

### 3.3 Native close与projection基础

- `WindowHostManager`集中持有window/surface记录，presenter store能复用surface、隐藏stale window并跳过相同generation。
- native close callback不会直接销毁dirty document host，而是转交host close decision；close prompt具备save/discard/cancel方向。
- autolayout已有floating frame clamp函数，说明work area约束可以上提成统一placement policy。

## 4. 目标架构

### 4.1 权威状态分层

| 层 | 应持有内容 | 不得持有内容 |
|---|---|---|
| `LayoutSchema` | page/window/tree/tab/drawer的versioned、validated结构 | live native handle、document payload |
| `LayoutProfile` | 用户级named layouts、page presets、tool visibility偏好 | project open documents、团队共享scene状态 |
| `ProjectWorkspaceState` | project-relative open documents、active toolkit、project context | 用户显示器坐标、个人panel大小 |
| `WindowPlacementProfile` | monitor stable ID、saved/current work area、logical rect、DPI、state、z/modal关系 | authoritative dock tree |
| `ViewInstanceSnapshot` | descriptor ID、instance ID、owner/version、bounded typed payload | live plugin object或native pointer |
| `LayoutSession` | 当前validated committed generation和live registry bindings | 直接充当磁盘document |

当前`.zircon/editor-workspace.json`把用户布局与project workspace混在一起。目标默认应将用户layout profile放在用户配置域，project只保存可协作的workspace语义；若项目确实需要共享layout template，应使用显式project layout asset而不是隐式覆盖个人状态。

### 4.2 Transaction与恢复协议

```text
Input bytes / command / preset
  -> bounded parse
  -> schema version dispatch
  -> pure migration chain
  -> structural + semantic validation
  -> descriptor/policy resolution
  -> placeholder or explicit rejection
  -> stage layout + registry + documents + native targets
  -> dirty-document decision barrier
  -> atomic authority swap (one generation)
  -> projection diff / diagnostics / durable checkpoint
```

任何阶段失败都必须保持旧`LayoutSession`、registry、document toolkits、native windows和持久化LKG不变。`LayoutTransaction`至少要提供precondition generation、prepare/commit/abort、resource lease、dirty decision和typed diagnostics；不能靠“先clone整个host，失败再猜着恢复”掩盖副作用。

### 4.3 Canonical schema与placeholder

- 每个persisted document必须有format version、producer build/schema、profile/project identity、generation和checksum或内容hash。
- ID必须唯一，tree深度/节点/tab/window/payload bytes必须有budget；所有float必须finite并满足范围。
- unknown/missing plugin view不应删除其布局位置。保存opaque bounded payload、owner/version和placeholder，插件恢复后可rebind。
- canonical model只保留一份activity drawer authority；legacy字段只存在于migration input，不进入当前写出schema。
- validator输出stable diagnostic code、JSON path、severity、repair action和provenance；repair必须确定、幂等且可预览。

### 4.4 Window placement

窗口恢复必须使用logical coordinates作为持久化语义，并同时保存monitor stable identity、保存时usable work area、scale factor和window state。恢复时先选择目标monitor，再把旧usable rect映射到新usable rect，约束最小可见标题区域，处理minimized/maximized/fullscreen，最后由backend转换physical coordinates。OS move/resize/DPI/monitor事件需debounce写回profile，而不是每帧用旧layout frame覆盖真实窗口。

## 5. P0：会造成数据丢失、状态破坏或升级不可恢复的问题

### P0-1 · Layout command错误会先破坏原布局，失败不满足no-op语义

`MoveView`、`AttachView`和`CreateSplit`先调用`detach_instance`，再验证目标workspace/path并attach。`open_view`又先创建registry/session实例。无效path、缺失target、policy拒绝或attach错误均可造成原placement消失、orphan registry/session或generation不一致。修复必须先在immutable candidate上解析和验证，再一次性commit；所有typed error测试要断言layout、registry、focus、generation和projection byte-for-byte不变。

### P0-2 · Reset layout绕过dirty document close decision

host的`ResetToDefault`先`clear_document_toolkits()`、清asset sessions并替换layout；toolkit clear只拦active save/closing，不检查dirty，也不调用Editor02定义的close transaction。布局偏好操作不应拥有丢弃文档内容的权限。Reset/import/switch/restore必须先计算将关闭的document集合，进入统一save/discard/cancel barrier；cancel时整个布局操作不得提交。

### P0-3 · Workspace restore采用destructive incremental apply

`apply_project_workspace_state`在验证完整输入前清空当前toolkit、registry、sessions和dependencies，然后逐实例`?`恢复；UI asset instances又在第二阶段继续。第N个descriptor/payload/asset失败会留下半恢复状态，且旧session已经不可重建。必须引入detached staging world、owner lease和single authority swap；失败只发布diagnostic，不触碰live session。

### P0-4 · Page preset restore有损重写document dock topology

`LayoutPreset::capture_from_layout`不保存ratio、精确tree、leaf tab allocation或active tab；`apply_center_split`先合并全部tabs再重建right-deep empty leaves。页面切换因此不是“恢复该页面布局”，而是破坏性拓扑转换。`ActivateMainPage`还允许不存在的page ID，现有测试甚至固化该行为。必须区分完整layout profile、受约束template和presentation preference，页面切换只能应用与页面声明兼容的完整/增量layout，不得暗中重排文档。

### P0-5 · Versioned restore没有完整validator/migration/LKG边界

外层workspace只接受format 1，内层`layout_version`不消费；preset asset忽略version/name，raw layout直接进入live session。递归树、数量、字符串/payload和float无输入budget，unknown plugin无placeholder。结果是坏文件可能OOM/深递归、进入NaN、静默删除实例、反复fallback或部分应用。必须在任何live mutation前完成bounded parse、migration、validation、LKG/quarantine和明确用户恢复路径。

## 6. P1：工程级完整性差距

### 6.1 Layout model、command与policy

1. `ResizeSplit`对ratio只做`clamp(0.1, 0.9)`；NaN可保留为NaN，并使后续JSON序列化失败或projection持续dirty。所有numeric command必须先`is_finite`再范围校验。
2. `SetDrawerExtent`只有最小值，没有相对窗口/工作区的上限、finite校验或content minimum协调。
3. `ActivateMainPage`不验证page存在、enabled/owner状态或当前capability。
4. core `SavePreset/LoadPreset`分支是no-op，真正副作用散落host；同一command在不同入口可能有不同语义。
5. `DockPolicy`只被descriptor/reflection读取，manager、host和drag/drop均不执行允许host/float/split/close等约束。
6. `PersistenceKeyPolicy`只有声明/default，没有capture/restore consumer，instance identity无法形成稳定恢复合同。
7. `DragPayload.kind`在drop resolution中基本未消费，document/tool/activity类型不能约束drop target。
8. `DropTarget::NewFloatingWindow`存在于领域枚举，但未发现生产创建路径，UI承诺与能力不一致。
9. reflection对Activity Window统一暴露floating支持，不考虑descriptor policy或平台能力。
10. binding可提交raw instance/page/window ID，没有descriptor existence、owner、policy、source host和target host验证。
11. 非multi descriptor可通过restore产生多个instance，single-instance index会被最后一个覆盖。
12. duplicate instance ID恢复时可静默覆盖，缺少collision diagnostic和owner namespace。
13. `DocumentNode::remove_instance`用短路`first.remove || second.remove`，重复placement只移除遍历到的第一处。
14. Tabs节点对path `[0]`与root采用兼容解释，路径不是单义stable address。
15. recursive tree没有depth、node、leaf和tab count budget，restore/normalize可遭受堆栈与内存放大。
16. raw serde可以构造空/重复tab、unknown instance、非法active tab和无效split ratio，领域不变量不在deserialize边界成立。
17. changed command后的normalize只修drawer active/legacy mirror，不处理空split、单子树、重复placement、focus和policy。
18. `LayoutNormalizationReport.placeholders`在生产没有填充，API承诺的unknown view修复能力尚未实现。
19. `collect_instance_hosts`用map insert收集，重复placement由遍历顺序静默覆盖，无法诊断多宿主。
20. session metadata会删除layout中不存在的known instance，却保留layout里的unknown ID；两侧漂移没有统一reconcile策略。
21. `drawers`、`activity_windows[*].activity_drawers`和project workspace `active_drawers`形成三份authority。
22. `activity_windows()`可能返回临时canonical clone而不提交repair，调用者看到的规范状态和persisted状态可不同。
23. `ActivateMainPage`先保存旧page、改active、recompute，再restore新page；任一后段失败会留下部分切换。
24. `close_view`先改layout/session/registry再commit document close；commit失败时close lease与live UI可能分叉，应复用Editor02 transaction。
25. floating window按instance循环close，后一个失败时前面已关闭，window close不是all-or-nothing decision。

### 6.2 Workspace、profile与preset persistence

26. `ProjectEditorWorkspace.layout_version`固定写1却从不校验或迁移，形成无效兼容性字段。
27. 用户window/dock/layout状态默认写入project `.zircon`目录，多个开发者或同机多用户会互相覆盖个人布局。
28. layout变化只有项目save时被捕获，没有独立debounced autosave、dirty generation或shutdown deadline，崩溃会丢近期布局。
29. workspace/preset使用`read_to_string`后raw serde，缺少file bytes、JSON nesting、string和entity budgets。
30. workspace missing rollback/delete使用直接`remove_file`，没有同等directory sync、tombstone或journal语义。
31. workspace与scene两个文件只做运行时compensation，不是crash-atomic generation commit，崩溃可留下混合代际。
32. named preset sanitizer会把多个不同名称映射到同一文件名，且空名归一为`preset`，没有collision detection。
33. project preset用raw `fs::write`，没有temp/create-new/flush/rename/parent sync。
34. preset先提交文件再import，import失败时command报错但磁盘已有 durable side effect，用户无法判断真实结果。
35. preset load忽略`format_version`和`preset_name`，不能阻止错误schema或路径/内容identity错配。
36. loaded project preset直接替换session layout，不经过canonical validation、normalize、builtin repair或placeholder resolution。
37. global/default layout存于Runtime `ConfigManager`，与Editor Settings/Profile authority分裂，scope、错误和shutdown语义不同。
38. ConfigManager写入是eventual async persistence，save command在磁盘durable前返回成功，也没有layout专用ticket/receipt。
39. global default load把service unavailable、missing key和parse failure都折叠为None/default，没有diagnostic、quarantine或repair UX。
40. config preset map内一个malformed value可使整个map deserialize失败，缺少per-entry隔离。
41. page user persistence在产品硬编码user ID `default`，没有真实account/local profile identity和迁移。
42. UI asset workspace在base workspace apply之后二次恢复，失败可造成第二种半应用状态。
43. `bootstrap_default_layout`先清live state再创建replacement，与restore共享destructive pattern。
44. missing/disabled plugin descriptor没有opaque placeholder和owner/version lease，layout意图会丢失或加载失败。
45. restore diagnostics缺少source path、schema generation、JSON path、repair/fallback reason和用户可执行action。
46. layout profile没有明确User/Project/Session/Default layer precedence、锁定策略和“恢复本机/共享模板”的产品语义。

### 6.3 Native window placement与生命周期

47. `WindowHostManager`只存optional handle、bounds和surface，没有monitor ID、saved work area、DPI、maximized/minimized/fullscreen、z-order、always-on-top或parent/modal关系。
48. 未观察到OS move/resize callback把用户实际native bounds写回`WorkbenchLayout`，当前recompute更像把requested frame重复投影给OS。
49. native presentation把logical `f32`直接round/cast到physical position/size，没有scale factor和per-monitor DPI转换。
50. host bounds只检查width/height大于0，不检查x/y/size finite、最小可见标题区或当前monitor work area。
51. autolayout clamp只约束shared-source frame；一旦host已有positive bounds，projection优先使用host值，可绕过clamp。
52. monitor消失、分辨率变化和taskbar/work area变化没有旧rect到新rect的映射策略。
53. minimized/maximized状态不持久化，恢复最小化窗口和普通rect的语义未定义。
54. presenter `sync_targets`增量hide/create/show；第N个backend失败可留下部分window topology，没有prepare/commit或reconcile report。
55. stale target采用hide/remove，但缺少native close acknowledgement和retry/backoff，backend object与authority可能漂移。
56. floating支持没有用户multi-window preference、平台capability或headless/remote session policy gate。
57. window ID与native backend identity没有epoch/incarnation，迟到callback可能命中新创建的同ID窗口。
58. native close、layout close和application shutdown缺少统一window/document decision DAG与deadline，容易出现循环close或不同入口不同结果。

## 7. P2：长期产品化与运维差距

1. 缺少Layout Manager产品页：创建、复制、重命名、删除、导入、导出、覆盖确认和恢复默认。
2. 缺少layout diff预览，用户无法在应用shared/preset layout前看到将关闭、移动或placeholder的panel。
3. 缺少per-page/profile最后使用时间、producer build、兼容状态和owner信息。
4. 缺少可搜索的layout diagnostics/history以及一键导出support bundle。
5. 缺少keyboard-only docking、screen reader语义、焦点恢复和高对比drop target验证。
6. 缺少RTL、长文本和locale切换后tab/window title重排验证。
7. 缺少大规模layout性能预算：万级tabs、深树拒绝、百窗口、多插件placeholder和restore latency。
8. 缺少layout telemetry的本地可控指标：migration、repair、fallback、restore duration和backend failure，不应采集文档内容。
9. 缺少multi-process profile锁、CAS/merge和冲突副本策略。
10. 缺少云同步/漫游profile的明确边界、加密、敏感payload过滤和离线冲突协议。
11. 缺少可重复的golden fixture corpus，覆盖每个历史schema和真实坏文件。
12. 缺少developer inspector显示canonical tree、instance owner、host、policy、generation和native placement来源。

## 8. 参考引擎对照

| 参考 | 可验证做法 | Zircon应吸收 | 不应照搬 |
|---|---|---|---|
| Unreal TabManager/LayoutService | 保存live/collapsed/invalid area；未知tab可保留；open/close/relocate请求延迟持久化；layout name编码version并清理旧version；主area失败fallback | unknown owner preservation、deferred save、version migration、invalid area diagnostics、workflow document state hook | Unreal的INI/JSON双格式与全局singleton历史包袱 |
| Godot EditorDockManager/WindowWrapper | dock自定义save/load hook；保存slot/selection/floating/split offset；按screen和saved usable rect恢复；显示器缺失、分辨率变化、minimized均有回退 | extension-owned bounded state、monitor/work-area映射、multi-window preference、named layout UX | 直接依赖Godot Control树和ConfigFile键布局 |
| Fyrox DockingManager | 递归tile descriptor保存split/tab/floating结构，窗口settings保存position/size/maximized | 精确拓扑schema和简洁typed descriptor | 其restore同样是先清后建且缺少完整事务，不能作为可靠性上限 |
| Bevy Window/Winit | 明确区分logical/physical resolution、monitor selection和scale factor | native backend placement转换与monitor abstraction | Bevy没有完整Editor docking/profile系统，不能补齐产品/迁移结论 |
| Unity Graphics package | 特定EditorWindow可显式`DontSaveToLayout`；package setting展示EditorPrefs migration/change event | descriptor persistence opt-out和显式setting migration | checkout不含Unity Editor核心layout源码，不能据此声称其完整恢复行为 |

目标基线应同时超过这些局部实现：保留Unreal的unknown tab与version discipline、Godot的monitor-aware restore、Fyrox的精确递归描述和Bevy的scale abstraction，再补上当前参考实现普遍不足的transactional staging、resource budgets、fault injection和可观测commit receipt。

## 9. 重构路线

### M0 · 冻结不变量与故障fixture

- 定义command error no-op、restore all-or-nothing、dirty decision、unique identity、finite geometry和unknown owner preservation不变量。
- 保存当前format 1、page preset、project preset、global config和代表性坏文件golden fixtures。
- 给现有113个focused test建立可编译lane；先修阻断该lane的既有test-build错误，不以删除测试绕过。

### M1 · LayoutAuthority与事务核心

- 新建pure candidate model、`LayoutTransaction`、precondition generation、prepare/commit/abort和typed commit result。
- command先validate target/policy，再在candidate上detach/attach；host registry/session以lease加入同一commit。
- Reset/close/import/switch接入Editor02 document close decision，cancel保持所有authority不变。

### M2 · Canonical schema、validator与migration

- 发布当前canonical schema v2，移除legacy drawer duplicate authority，定义唯一ID和tree invariants。
- 建立bounded streaming/read admission、migration registry、deterministic normalize/repair、LKG与quarantine。
- unknown plugin/view使用opaque bounded placeholder，保留owner/version/persistence key和原placement。

### M3 · Persistence authority与durability

- 分离User `LayoutProfileStore`、Project workspace store、shared layout asset和Session overrides。
- 全部写入走atomic temp/flush/rename/parent sync，并返回durable ticket/receipt；增加debounced autosave和shutdown deadline。
- workspace+scene采用manifest/generation journal或bundle commit，启动时只选择完整代际。

### M4 · Safe restore与page/layout产品

- restore先构造detached registry/document/native target staging，验证成功后single swap。
- page layout保存精确拓扑或显式增量patch；禁止semantic template伪装成roundtrip snapshot。
- 补齐named layout管理、preview diff、overwrite/collision、import/export、fallback和repair UX。

### M5 · Descriptor policy与插件扩展

- 强制执行DockPolicy/PersistenceKeyPolicy、single/multi instance、owner lease和payload schema/version。
- 插件提供layout state codec/migrator/validator与lifecycle hook，卸载变placeholder，重载可rebind。
- reflection、binding、drag/drop和command registry从同一effective policy snapshot投影。

### M6 · Monitor-aware native window lifecycle

- 扩展placement schema和platform monitor service，处理DPI、work area、hotplug、min/max/fullscreen、z/modal。
- OS callback更新authority，debounce持久化；recompute只在generation变化时向backend发目标。
- presenter支持prepare/reconcile、backend failure recovery、incarnation过滤和close acknowledgement。

### M7 · 规模、兼容与运维验收

- 建立历史schema corpus、恶意输入fuzz、Nth-operation fault injection、crash/restart和双进程竞争测试。
- 建立多显示器/DPI矩阵、plugin missing/reload、dirty documents和十万级layout输入budget测试。
- 记录restore latency、repair/fallback/backend failures和commit durability，失败可导出support bundle。

依赖顺序必须是M0 -> M1 -> M2 -> M3 -> M4/M5 -> M6 -> M7。不得先做Layout Manager UI或更多preset格式，再把非事务core继续包装进产品。

## 10. 验收门

1. Move/Attach/CreateSplit对每一种typed error都证明layout、registry、session、focus、generation和native target不变。
2. OpenView attach失败不留下instance、single-instance index、session或projection orphan。
3. Reset/import/restore/page switch遇到dirty document时统一出现save/discard/cancel decision，cancel零副作用。
4. 第N个view、UI asset、plugin codec或native target restore失败时旧workspace完整可用，新workspace零提交。
5. page A -> B -> A在无用户编辑时精确恢复split tree、ratio、tab order/allocation、active tab、drawer和window placement。
6. 不存在/disabled page ID被拒绝，不能进入active state或写入preset key。
7. NaN、Infinity、负值、超界ratio/extent/frame在parse和command两条入口均被稳定diagnostic拒绝。
8. duplicate page/window/instance/placement和ambiguous path被validator拒绝或确定repair，绝不静默覆盖。
9. tree depth/node/tab/window/payload/file bytes超过budget时在受控内存和时间内失败。
10. normalize/repair幂等：连续两次结果相同，第二次无diff且diagnostic稳定。
11. legacy drawer输入迁移后只写出一份canonical authority，roundtrip不重新生成legacy字段。
12. missing plugin保留placeholder、opaque bounded payload和placement；安装同版本plugin后可无损rebind。
13. plugin payload version不兼容时只隔离该instance，不阻止其余workspace恢复。
14. DockPolicy和PersistenceKeyPolicy在command、binding、drag/drop、reflection和restore五条入口给出同一结果。
15. non-multi descriptor在所有restore/import路径最多一个instance，collision有明确owner diagnostic。
16. project workspace不再隐式覆盖用户layout profile；两个用户打开同一project保持各自布局。
17. layout变更在debounce窗口后获得durable receipt，进程崩溃后最多丢失已公开上限内的变化。
18. 磁盘满、permission denied、rename失败和parent sync失败不发布“已保存”，live layout继续工作且标记unsaved。
19. scene/workspace任一commit点进程终止后，重启只加载完整同代际bundle，不混合新旧generation。
20. named preset的非法名、空名和sanitizer collision有明确错误/确认，不覆盖其他preset。
21. malformed global preset只隔离该entry并保留LKG，service unavailable与missing/parse error可区分。
22. schema v1 fixture经pure migration到v2，输出golden稳定；未来version不被当前writer覆盖。
23. current/bad/LKG/quarantine来源在诊断和Layout Manager中可见并可恢复。
24. 用户实际拖动/缩放native window后，authority收到新logical placement并在重启后恢复。
25. 100%、125%、150%、200% DPI之间移动窗口，logical content size稳定且physical rect正确。
26. 保存时monitor消失或work area改变后，窗口映射到可用monitor，标题栏至少保持可见。
27. minimized/maximized/fullscreen状态恢复不生成不可见或零尺寸窗口，normal rect仍可找回。
28. presenter第N个create/show/hide失败后能reconcile到authority，无重复窗口、泄漏surface或迟到callback误命中。
29. multi-window关闭含多个dirty document时只提交一个一致decision DAG，cancel恢复全部窗口。
30. keyboard-only docking、focus restore、screen reader role/name/state和高对比drop indicator通过自动化与人工验证。
31. 10,000 tabs/100 windows/100 missing plugin placeholders的合法上限fixture满足明确restore latency和peak-memory门槛。
32. Windows required lane、migration/fuzz/fault/crash/multi-monitor矩阵全部产出可归档报告；不得以静态test inventory代替动态通过。

## 11. 实施边界与交叉计划

- dirty document save/close/recovery authority由Editor02拥有；本计划只接入，不创建第二套dirty判断。
- plugin enable/reload/owner lease由Editor06拥有；本计划定义layout placeholder和codec接缝。
- command权限、context和remote automation由Editor08拥有；布局command只消费统一invocation/policy，不另设旁路。
- settings/profile scope与durable commit基础由Editor12拥有；layout profile使用其持久化合同，但保持独立schema和migration。
- Retained UI diff/presentation性能由Editor01拥有；本计划输出single committed generation和可消费diff。
- platform window/monitor backend若归属`zircon_runtime`或`zircon_app`，必须以公共logical/physical placement DTO接入，Editor不得直接持有平台私有指针。

本轮仅完成review与重构计划，没有修改production layout、workspace、preset、window或tests。开始实施前先复核24个在途scoped文件及已知test-build阻断，再从M0/M1建立失败不变式，禁止在旧格式上继续堆叠新功能。
