---
related_code:
  - zircon_editor/assets/ui/editor/keymap/default.keymap.toml
  - zircon_editor/assets/ui/editor/components/workbench/floating/workbench_command_palette.zui
  - zircon_editor/src/core/commands
  - zircon_editor/src/core/commandlet
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/extension/store
  - zircon_editor/src/core/plugin/materializer.rs
  - zircon_editor/src/core/settings
  - zircon_editor/src/ui/binding_dispatch/editor_event_normalization.rs
  - zircon_editor/src/ui/host/command_eval_projection.rs
  - zircon_editor/src/ui/host/editor_event_control_requests.rs
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/ui/host/editor_extension_registration.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/ui/host/editor_operation_dispatch.rs
  - zircon_editor/src/ui/retained_host/app/command_palette_actions.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/workbench/command_palette.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/workbench/menu_action.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/command_palette.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/text_input/keyboard
  - zircon_editor/src/ui/workbench/event
  - zircon_editor/src/ui/workbench/model/menu
  - zircon_runtime_interface/src/editor_contribution.rs
  - zircon_runtime_interface/src/ui/event_ui/control.rs
  - zircon_plugins/plugin_sdk/src/editor_contribution.rs
tests:
  - zircon_editor/src/tests/commands
  - zircon_editor/src/tests/editor_event/runtime/keymap_settings.rs
  - zircon_editor/src/tests/editor_event/runtime/registry.rs
  - zircon_editor/src/tests/editor_event/runtime/extensions_registration
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/command_palette.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/template_bridge/workbench_window_menus
  - zircon_editor/src/tests/ui/control/reflection_projection.rs
  - zircon_editor/src/tests/workbench/reflection
  - zircon_editor/src/ui/retained_host/app/tests/command_palette.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/130-editor-command-registry-keymap-menu-palette-context-routing-remote-automation-current-source-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
  - docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
  - docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Commands/UICommandList.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Commands/UICommandInfo.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Commands/InputBindingManager.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Commands/InputBindingManager.cpp
  - dev/UnrealEngine/Engine/Source/Developer/ToolMenus/Public/ToolMenu.h
  - dev/UnrealEngine/Engine/Source/Developer/ToolMenus/Public/ToolMenus.h
  - dev/godot/core/input/shortcut.h
  - dev/godot/core/input/shortcut.cpp
  - dev/godot/editor/settings/editor_command_palette.h
  - dev/godot/editor/settings/editor_command_palette.cpp
  - dev/Fyrox/editor/src/command/mod.rs
  - dev/Fyrox/editor/src/settings/keys.rs
  - dev/bevy/crates/bevy_ui_widgets/src/menu.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/ContextualMenuDispatcher.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Debugging/DebugWindow.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Editor/Material/Decal/DecalProjectorEditor.cs
doc_type: review-and-refactor-plan
refreshes:
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/130-editor-command-registry-keymap-menu-palette-context-routing-remote-automation-current-source-review.md
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# 178 · Editor Command Registry / Keymap / Menu / Palette / Context Routing / Remote Automation 当前源码复核

## 1. 结论与状态

Editor08/130定义的工程级命令控制面仍未闭合。当前源码不是临时按钮回调的空壳：`EditorOperationPath`、descriptor/factory分离、`WhenClause`、context generation、typed keymap override、keyboard metadata、menu/palette投影、immutable palette catalog、operation transaction和headless metadata都是真实基础。其中palette已有byte posting、rarest-posting候选、bounded top-K、offset/limit窗口和metrics；keymap resolver也会在signature命中后比较完整chord，并在多个enabled候选时fail-close。这些结构必须保留。

但metadata、执行、授权、owner生命周期和各surface呈现仍是分裂authority。三项P0在当前磁盘全部可静态复现：

1. `UiControlRequest::InvokeBinding/InvokeRoute`执行`EditorOperation`时没有等价remote gate，最终以`UiBinding/RetainedHost`来源进入operation与journal；`CallAction`和direct Remote/Cli operation才检查remote policy。
2. Runtime Interface、Plugin SDK和materializer fixtures接受二段command ID，宿主`EditorOperationPath`要求至少三段，公开合法值无法形成宿主合法operation identity。
3. serialized plugin `Command`只有ID/schema/display name，materializer注册operation descriptor却不注册factory；即使统一ID，执行仍确定落入`MissingFactory`。

本轮不增加或重排Editor130的canonical finding，只按当前源码重判其 **3项P0、44项P1、10项P2和24个资格门**：

| 等级 | Open / Fail | Partial | Closed / Pass |
|---|---:|---:|---:|
| P0 | 3 | 0 | 0 |
| P1 | 25 | 19 | 0 |
| P2 | 9 | 1 | 0 |
| Gate | 17 Fail | 7 Partial | 0 Pass |

当前源码相对Editor130的可确认进展主要是：built-in menu已经展示effective shortcut；typed settings authority与manager cache能持久化并刷新keymap override；extension contribution store能按ticket撤销自己的snapshot；menu extension能递归构树；palette/window request具有catalog generation与有界窗口。这些进展只把对应finding降为Partial，尚未形成command definition、factory、menu、keymap和palette同generation安装/撤销的产品闭环。

## 2. 冻结语料与currentness

### 2.1 物理选择集

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 本轮证据 |
|---|---:|---|
| Zircon命令、输入、菜单、palette、control、plugin、settings、commandlet与聚焦测试 | **121 / 28,640 / 26,168 / 1,006,722 / 247 / 8** | 当前磁盘产品路径与词法测试属性；fingerprint `17a84b6c504facf008b3e2623edbf0f4da856ed73a25c31a2e0ef4e30f8d671c` |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics | **16 / 6,073 / 5,094 / 241,740 / 3 / 0** | command mapping、input binding、owner menu、palette lifecycle、undoable command与UI consumer参考；fingerprint `5ecbfda11012b810191835f0e3ce295cf5b9293437e005890aec3594e16c1f5c` |

统计按normalized relative path的ordinal顺序，将每个`path + NUL + raw bytes + NUL`串联后计算SHA-256；行数按CRLF/LF/CR split并保留终止空行。`tests/ignored`是词法计数，不是执行receipt。冻结时Git HEAD为`81b574e44a2b4a64698ffb062cf48b380e3542ea`；共享工作树存在大量在途修改，所以本报告以当前磁盘fingerprint而不是HEAD tree为证据锚点。

### 2.2 产品链真实性矩阵

| 产品链 | 当前事实 | 判定 |
|---|---|---|
| Definition -> registry | descriptor与factory分存，registry有generation和typed error；registry/descriptor/when/chord仍可直接Deserialize，command无owner lease/unregister | 局部基础；不是可恢复、可撤销的definition authority |
| Plugin contribution -> execution | contribution store有ticket/revoke；Host先克隆registry注册command/factory，再独立publish store ticket | 两次publication无共同commit；store revoke不证明command/factory撤销 |
| Keyboard -> command | native事件保留logical/physical/scan/text，text focus吞掉常见编辑键；keymap有signature index、override和冲突计算 | Partial；单chord、Option结果、orphan、scope/priority/layout/IME/repeat合同不足 |
| Menu -> invocation | built-in和extension都能生成menu，built-in已展示effective shortcut | Partial；两种建树算法、薄model、固定top-level、legacy prefix旁路仍在 |
| Palette -> invocation | immutable generation、posting、bounded heap/window、MRU与metrics真实存在 | Partial；Unicode不一致、同步查询、owner metadata和generation-qualified commit仍缺 |
| Control -> operation | direct Remote/Cli operation和CallAction有gate，control failure不会伪成功 | P0；InvokeBinding/Route operation分支绕过gate并洗白provenance |
| Commandlet -> automation | registry有headless metadata、capability检查和structured result局部字段 | Partial；parser/action enum/executor仍硬编码三个commandlet，无definition生成与生命周期协议 |

### 2.3 当前源码新增或修正证据

- `zircon_editor/src/ui/host/editor_event_control_requests.rs:23-29`直接处理`InvokeBinding/InvokeRoute`；`CallAction`在`:131-140`检查remote callable，但operation binding在`:163-183`以`EditorOperationSource::UiBinding`执行。
- `zircon_editor/src/ui/host/editor_operation_dispatch.rs:114-129`只在source被判定为remote时执行gate；`:502-506`只把`Remote/Cli`视为需要remote许可，普通UI event又可归为`RetainedHost`。
- `zircon_editor/src/core/editor_operation.rs:55`固定`MIN_OPERATION_PATH_SEGMENTS = 3`；Runtime Interface、SDK和materializer测试仍分别使用`command.a`、`sample.command`、`fixture.command`。
- `zircon_editor/src/core/plugin/materializer.rs:111-117`只注册`EditorCommandDescriptor::operation`；`editor_operation_dispatch.rs:192-205`对缺失factory返回typed `MissingFactory`。
- `zircon_editor/src/core/commands/registry.rs:19`仍derive `Deserialize`，`:184`仍线性反查event，`:261`用`checked_add(...).expect`推进generation。
- `zircon_editor/src/core/commands/descriptor.rs:28-29,62,294`仍让remote policy在constructor与serde缺省时为true。
- contribution store的ticket revoke会删除自己的command/factory snapshot；但`editor_extension_registration.rs`先克隆并发布product registry，再单独发布contribution ticket，没有找到product command registry的owner revoke路径。
- built-in menu现在从effective keymap投影shortcut，因此Editor130的P1-25不再是“menu与palette都显示default”；palette entry仍读descriptor default，所以该项只降为Partial。
- keymap settings通过typed Settings authority持久化并由manager刷新`Arc` cache；产品Settings UI仍把Keymap Overrides呈现为空字符串，没有浏览、录制、冲突修复或restore workflow。
- materializer的“every supported contribution kind”测试仍以二段`fixture.command`期待成功；这与当前三段parser静态矛盾。本轮没有运行它，不能将该矛盾描述成已取得失败receipt。

### 2.4 目标架构符号缺席

对本轮生产选择集检索`InvocationPrincipal`、`InvocationSurface`、`SourceProvenance`、`InvocationGateway`、`CommandRegistrationLease`、`EditorCommandId`、`CommandDefinition`、`KeyResolutionOutcome`、`EffectiveKeymapSnapshot`、`MenuGraph`、`AutomationService`、`CommandletDefinition`和`OwnerRevoked`，产品定义命中为0。现有相近局部类型不能被当作这些跨surface合同已经实现。

## 3. P0：当前仍可复现的契约断裂

### E-CMD-P0-01 · Open · control route/binding绕过remote policy并丢失来源

`CallAction`明确检查remote许可，证明`UiControlRequest`是需要按remote principal评估的控制边界；`InvokeBinding/InvokeRoute`却直接解析binding。若payload是`EditorOperation`，宿主以`UiBinding` source调用operation，gate因source不是Remote/Cli而不执行，journal再看到`RetainedHost`。普通`EditorCommand`目前会经过Headless -> Remote重映射并复核descriptor，这只关闭了一个payload分支，不能关闭同一control family的operation旁路。

必须在最外层建立唯一`InvocationGateway`：先保留principal、transport、request、route/binding，再解析command/operation，最后按surface-specific policy deny-by-default。来源只允许追加stage，不能被后续`UiBinding`覆盖。回归必须覆盖CallAction、InvokeRoute、InvokeBinding、MenuAction和direct Operation五条入口的allow/deny矩阵及journal/audit provenance。

### E-CMD-P0-02 · Open · public command ID与宿主operation ID语法不一致

public DTO与SDK成功构造二段ID，Editor materializer自己的success fixture也使用二段ID；宿主parser则要求至少三段。这不是局部输入校验遗漏，而是跨crate公共合同互相矛盾。修复必须由Runtime Interface提供共享、版本化、validated `EditorCommandId`，SDK builder、serde DTO、menu reference、registry和operation route全部复用同一parser与golden corpus。若最终选择`owner.domain.action`，二段旧值只能经显式migration/diagnostic处理，禁止宿主静默补段。

### E-CMD-P0-03 · Open · serialized plugin Command可发现但不可执行

serialized `Command`没有executor/callback handle、host route kind、payload codec、owner generation、capability或surface policy。materializer却将其发布成operation descriptor，没有factory registration；调用会稳定得到`MissingFactory`。必须明确二选一：声明versioned host-known route并由宿主factory catalog解析，或声明受稳定ABI/IPC和owner lease约束的plugin callback。admission必须把definition、executor、menu和default keymap作为一个batch安装；端到端验收必须从发现、执行、transaction/result一直覆盖到plugin unload后的同generation撤销。

## 4. P1：当前状态与重构要求

### 4.1 Registry、descriptor与context authority

| ID | 状态 | 当前证据与必须重构为 |
|---|---|---|
| E-CMD-P1-01 | Open | registry仍derive Deserialize，commands/generation可绕过admission，而factory/palette被`serde(skip)`清空。只允许versioned declarative DTO，经validator与factory resolver后原子发布snapshot。 |
| E-CMD-P1-02 | Open | descriptor、chord、WhenClause的derived Deserialize仍绕过排序、规范化和resource limit。改为validated newtype/custom deserialize及统一长度/数量/depth预算。 |
| E-CMD-P1-03 | Partial | contribution store已有ticket revoke，但product command registry没有owner/token/unregister；Host两次publication可留下ghost command/factory。建立`CommandRegistrationLease`与atomic owner revoke。 |
| E-CMD-P1-04 | Open | pending metadata/factory仍分别take，失败可产生分裂状态。definition、factory、menu、key default必须同batch校验和发布。 |
| E-CMD-P1-05 | Open | `descriptor_for_event`线性扫描并取确定顺序首项，同event的command identity不唯一。禁止从event反推command，或在admission建立唯一reverse index。 |
| E-CMD-P1-06 | Partial | keymap/palette已有局部index，但headless name/route和部分duplicate admission仍扫描，bulk register仍可O(N²)。统一构建name/route/event/owner secondary index与规模预算。 |
| E-CMD-P1-07 | Open | validator仍未统一限制label、keyword、menu depth、when、capability/schema/localization/owner/shortcut。建立field/stage typed diagnostics和resource budget。 |
| E-CMD-P1-08 | Open | recursive WhenClause可反序列化任意深度并递归eval，冲突solver还会组合探索。admission编译为有界迭代program/DAG并限制solver工作量。 |
| E-CMD-P1-09 | Partial | context已有project、undo/redo、document kind、scene mode、selection、writable、play、feature capability；仍缺focused control、modal、document instance/revision、tool/input capture、principal/platform。扩成typed generation snapshot。 |
| E-CMD-P1-10 | Open | mutation对overflow `expect`，其他snapshot逻辑并非同一不可回绕合同。使用opaque epoch或明确exhausted终态，所有handle/cache/commit统一验证。 |

### 4.2 Remote、control与operation policy

| ID | 状态 | 当前证据与必须重构为 |
|---|---|---|
| E-CMD-P1-11 | Open | descriptor constructor及serde缺省仍remote allow。迁移为surface-specific deny-by-default，并对每个旧命令显式审计。 |
| E-CMD-P1-12 | Open | CLI与remote继续共用一个bool。policy必须区分local batch、CLI、MCP/remote、test replay、interactive requirement和side-effect class。 |
| E-CMD-P1-13 | Open | enabled feature capabilities仍被当作caller authorization。分离`FeatureCapabilitySet`与principal permission/project grant。 |
| E-CMD-P1-14 | Open | control request/response无version、principal、request/correlation、deadline、idempotency。建立versioned envelope与稳定response correlation。 |
| E-CMD-P1-15 | Open | JSON arguments和operation group没有bytes/depth/node/string/count/time预算。限制必须在factory执行业务前完成并返回typed stage。 |
| E-CMD-P1-16 | Open | ListOperations仍全量collect，无cursor/catalog generation/page/byte limit。改为generation cursor分页并返回disabled reason与stale结果。 |
| E-CMD-P1-17 | Open | history仍是全局固定128项，无project/document/principal/filter/cursor。按stable sequence分页；机械存储仍由Editor02拥有。 |
| E-CMD-P1-18 | Open | Menu/UiBinding可折叠为RetainedHost，control transport被覆盖。改为不可覆盖的principal -> transport -> request -> binding -> command -> operation -> transaction链。 |
| E-CMD-P1-19 | Open | payload schema仍只是语法字符串，没有执行前codec/catalog validation。schema ID必须解析versioned codec并记录validation stage/version。 |
| E-CMD-P1-20 | Open | control/commandlet错误仍大量自由文本。统一typed code、stage、retryability、field path、remediation和受控wire projection。 |

### 4.3 Keymap与keyboard routing

| ID | 状态 | 当前证据与必须重构为 |
|---|---|---|
| E-CMD-P1-21 | Partial | resolver会评估同signature候选并在第二个enabled候选时fail-close，conflict solver也存在；仍无typed owner/priority/UI修复入口。冲突应成为settings transaction的一等结果。 |
| E-CMD-P1-22 | Partial | disabled候选不会遮蔽唯一enabled候选，但`Option<&str>`无法区分unbound、unsupported、all-disabled、ambiguous、orphan。返回`KeyResolutionOutcome`及原因/generation。 |
| E-CMD-P1-23 | Partial | unknown override在dispatch closure中被视为disabled，不会截获已注册候选；但它仍留在effective map且没有隔离、迁移或清理。orphan不得进入active dispatch index。 |
| E-CMD-P1-24 | Partial | typed settings与默认TOML存在，测试只验证descriptor default出现，不验证双向集合和值完全相等。默认键位必须只有一个生成authority。 |
| E-CMD-P1-25 | Partial | built-in menu已展示effective shortcut，dispatch也用effective keymap；palette仍展示descriptor default。四个surface必须订阅同一`EffectiveKeymapSnapshot` generation。 |
| E-CMD-P1-26 | Open | 没有产品级Keymap Editor；Settings UI对override只产空字符串。交付浏览/搜索/录制/冲突对比/恢复/import/export/migration workflow。 |
| E-CMD-P1-27 | Open | chord serde仍可产生空或非规范key，持久化无schema/version。采用validated `KeyBindingDocumentV2`并隔离损坏项。 |
| E-CMD-P1-28 | Open | 单logical key + modifiers无法表达sequence、scope、priority、location/numpad、primary modifier、AltGraph或repeat policy。建立logical/physical event sequence模型。 |
| E-CMD-P1-29 | Partial | native bridge保留physical/scan/logical/text，text focus也有局部优先级；resolver仅消费logical与有限Windows fallback，缺repeat、IME、AltGraph和layout政策。建立统一KeyboardRoutingService。 |

### 4.4 Menu model与路由

| ID | 状态 | 当前证据与必须重构为 |
|---|---|---|
| E-CMD-P1-30 | Partial | extension menu递归构树，built-in仍只`split_once('/')`并压平深层路径。两者必须共用完整path/section/anchor的`MenuGraphBuilder`。 |
| E-CMD-P1-31 | Open | `MenuItemModel`仍只有label/action/operation/shortcut/enabled/children。改为separator/section/check/radio/visibility/icon/tooltip/disabled reason/a11y的typed union。 |
| E-CMD-P1-32 | Partial | extension已有path/priority和store ticket revoke基础；top-level仍固定，缺owner anchor、section、before/after、profile/customization及与command同批撤销。建立owner-aware menu registry。 |
| E-CMD-P1-33 | Open | menu/action/control/operation/legacy alias仍由多处字符串映射。由versioned identity catalog生成codec与UI binding。 |
| E-CMD-P1-34 | Open | `SavePreset.*`/`LoadPreset.*`等prefix仍直接构造LayoutCommand，绕过registry/when/policy/audit。只保留有截止版本和使用量的边界migration adapter。 |
| E-CMD-P1-35 | Open | 无action/operation节点仍会形成空MenuAction binding。entry kind必须在model层确定，非action节点不注册hit target/binding/shortcut。 |
| E-CMD-P1-36 | Partial | palette已有immutable generation结构，extension tree也有排序；built-in menu仍频繁重建/clone，duplicate递归扫描，无统一generation/context cache和预算。 |

### 4.5 Command Palette

| ID | 状态 | 当前证据与必须重构为 |
|---|---|---|
| E-CMD-P1-37 | Open | document用ASCII lowercase，query用Unicode lowercase，byte fuzzy无统一normalization。采用明确的NFKC/case-fold策略并保存原文/highlight映射。 |
| E-CMD-P1-38 | Partial | entry已有ID、label、category、keywords、shortcut；source仍只是category tag，缺localized description/path、owner/package、effective shortcut、deprecation。补齐typed source metadata。 |
| E-CMD-P1-39 | Partial | bounded posting/window和generation真实存在；query仍同步、无最大长度/debounce/cancel/deadline/frame budget。worker结果必须绑定catalog/query generation。 |
| E-CMD-P1-40 | Partial | 100k ignored benchmark存在，但只断言MRU membership改善，不测cold/warm query、rebuild、commit、Unicode、common-byte或plugin churn。建立required p50/p95/p99/allocation/cancel gate。 |
| E-CMD-P1-41 | Partial | window request检查catalog generation，最终commit只携任意ID且MRU在dispatch后记录。commit应携catalog generation与selection token，stale/revoked返回typed refresh。 |
| E-CMD-P1-42 | Partial | menu展示disabled、palette隐藏disabled，二者都复用局部enablement但没有共同reason/remediation。surface决定hide/disable，evaluation提供统一typed reason。 |

### 4.6 Commandlet与headless automation

| ID | 状态 | 当前证据与必须重构为 |
|---|---|---|
| E-CMD-P1-43 | Partial | registry已有headless metadata，但parser、flag、action enum和executor match仍硬编码三个commandlet。由versioned `CommandletDefinition`生成parser/help并按owner lease注册executor。 |
| E-CMD-P1-44 | Partial | 当前有capability检查和局部structured report；envelope仍无schema/request/timing/progress/cancel/deadline，default runner硬编码capability且authoring automation定义不一致。收敛统一automation lifecycle和exit mapping。 |

## 5. P2：长期维护、可发现性与观测

| ID | 状态 | 当前差距与方向 |
|---|---|---|
| E-CMD-P2-01 | Open | category仍是closed enum。改为stable category ID、localized label与owner contribution。 |
| E-CMD-P2-02 | Partial | descriptor现在已有description，但仍缺help URL、examples、side-effect class、deprecation/replacement。由palette/settings/discovery/diagnostics共同消费。 |
| E-CMD-P2-03 | Open | menu label与shortcut仍退化为裸String，无localization key、mnemonic和screen-reader语义。建立typed localized/a11y metadata。 |
| E-CMD-P2-04 | Open | palette MRU固定全局32项，无workspace/profile/pin/privacy policy。迁移到versioned scoped profile。 |
| E-CMD-P2-05 | Open | u8 rank + ID tie-break无feature breakdown和质量corpus。建立versioned rank、golden queries及质量/延迟双gate。 |
| E-CMD-P2-06 | Open | parser接受`F10000`，native fallback仅F1-F12。key vocabulary必须由平台能力表生成并在admission拒绝不可产生token。 |
| E-CMD-P2-07 | Open | 无platform/layout-aware formatter和logical/physical round-trip matrix。补macOS primary符号、非US布局、location/numpad golden。 |
| E-CMD-P2-08 | Open | generation仍暴露裸u64且overflow策略分裂。使用opaque revision/epoch token。 |
| E-CMD-P2-09 | Open | 无按surface/owner聚合的denied/disabled/unknown/conflict/stale/factory failure与latency telemetry。建立有界、默认不记录payload的指标。 |
| E-CMD-P2-10 | Open | 247个词法测试属性仍没有remote InvokeBinding/Route deny、plugin execution/unload、Unicode、typed conflict、keymap UI等required行为与故障测试。source-shape断言不得替代执行gate。 |

## 6. 本地参考源码对照

| 参考 | 直接读取到的工程机制 | 对Zircon的约束 |
|---|---|---|
| Unreal `FUICommandList` / `FUICommandInfo` | action mapping同时携execute、can execute、check、visibility、repeat；command list支持append、unmap和context组合 | command definition不能只存label/ID；执行、状态、context composition和显式撤销必须同一authority |
| Unreal `FInputBindingManager` | context map、active/default/user/project chords、duplicate检查、user chord changed delegate、register/remove command | keymap必须是context/owner/generation-aware产品服务，不能只靠全局override map |
| Unreal ToolMenus | owner、unregister owner、section/insertion、dynamic context、profile/customization/save/remove | plugin menu必须按owner lease原子撤销，并支持anchor/section/profile而非path字符串追加 |
| Godot Shortcut | 一个Shortcut保存多个InputEvent并复用event matching | Zircon单logical chord不足以覆盖sequence、physical/location、platform/layout与多事件替代 |
| Godot Command Palette | command绑定Callable和Shortcut，支持add/remove、execute、history及settings shortcut刷新，并限制展示数量 | Zircon应保留更强的bounded query结构，同时补executor/owner lifecycle和effective shortcut刷新 |
| Fyrox CommandStack / KeyBindings | command明确execute/revert/finalize并有bounded stack；键位覆盖多个Editor工具域 | transaction仍由Editor02拥有，但command service必须把身份与可撤销业务operation稳定相连；scope不能只有全局 |
| Bevy menu widget | popup/focus/navigation/disabled/repeat行为是typed UI状态的一部分 | 只作为presentation/input behavior参考，不足以替代Editor command authority |
| Unity Graphics consumers | `MenuItem`以context/priority/validate组织；Debug/Decal工具展示context-bound和numpad Shortcut用法 | 只证明真实工具会消费context、validation和特定键位；该局部源码不是Unity Editor命令authority参考 |

参考结论不是复制Unreal API形状。Zircon可以保留Rust typed DTO、immutable snapshots和bounded query优势，但owner lifecycle、context、authorization、extension、input vocabulary和automation protocol必须达到同等级完整性。Unity Graphics只按本地可见consumer证据使用，不推断闭源Unity Editor内部实现。

## 7. 目标架构与唯一owner

### 7.1 Definition与registration authority

- `EditorCommandId`：Runtime Interface拥有的versioned validated newtype，统一SDK、wire、registry、menu/keymap reference和operation mapping。
- `CommandDefinition`：包含localized identity、description/help、category、payload codec、side-effect class、surface policy、compiled context predicate、default bindings、menu contributions和executor specification。
- `CommandRegistrationBatch`：owner/module/plugin generation下同时admit definition、executor/factory、menu和default keymap；成功只发布一次generation，失败零可见。
- `CommandRegistrationLease`：revoke后上述所有投影同generation消失，旧handle得到typed `OwnerRevoked/StaleGeneration`。

### 7.2 Invocation authority

- `InvocationGateway`是UI、keyboard、menu、palette、control、CLI与replay的唯一入口。
- `InvocationPrincipal`、`InvocationSurface`、project/session trust、feature capability与principal permission彼此独立；remote/automation默认deny。
- `SourceProvenance`只追加principal -> transport -> request -> binding/route -> command -> operation -> transaction，不允许重写历史stage。
- payload decode、schema validation、context/policy、factory execution和transaction publication返回统一typed stage/error/receipt。

### 7.3 Keymap、Menu与Palette投影

- `EffectiveKeymapSnapshot`同时服务resolver、menu、palette、tooltip和Keymap Editor，含registry/settings/platform/layout/context generation。
- `KeyResolutionOutcome`区分resolved、unbound、unsupported、disabled、ambiguous、orphan和stale，并携候选owner/priority/reason。
- `MenuGraph`按owner、section、anchor、profile和context由同一builder生成built-in/plugin树；动态state与definition generation分离。
- `CommandPaletteCatalog`使用localized normalized document、owner metadata、effective shortcut和generation-qualified selection token；查询可取消且有deadline/预算。

### 7.4 Automation authority

- `CommandletDefinition`声明versioned args/result schema、executor owner、required capability/permission、deadline/cancel/artifact policy。
- `AutomationService`从definition生成parser/help/discovery，统一local CLI与受控remote调用；report包含schema、request、timing、diagnostic、artifact和稳定exit mapping。
- commandlet业务executor由owner lease注册，新增领域commandlet不修改central enum/match。

## 8. 必须硬切的旧路径

1. 删除`EditorCommandRegistry`、descriptor、chord和WhenClause绕过admission的直接Deserialize入口；持久化只允许versioned DTO。
2. 删除各crate独立command ID语法；二段ID只在有截止版本的migration adapter存在。
3. 将remote缺省从allow硬切为deny；禁止CLI借用remote bool，禁止InvokeBinding/Route绕过gateway。
4. serialized plugin Command在executor ABI/host route就绪前fail admission，禁止发布不可执行descriptor。
5. 删除`SavePreset.*`/`LoadPreset.*`等业务prefix直接执行路径，全部解析到canonical command。
6. 删除descriptor default shortcut作为menu/palette展示authority，统一读取effective snapshot。
7. 删除built-in/extension两套menu path算法和空MenuAction推断。
8. 删除commandlet central concrete action enum/match，改为definition + owner executor registry。

## 9. 分层重构里程碑

### M0 · P0封口与golden contract

- 为所有control surface接入统一gateway/provenance，并先补deny回归。
- 引入共享command ID golden corpus，修正DTO/SDK/materializer二段fixtures。
- 在executor模型完成前拒绝serialized plugin Command，避免继续发布不可执行条目。

### M1 · Definition、owner lease与atomic registry

- 建立validated ID/definition/resource budgets、secondary indexes和compiled When program。
- 实现owner batch、single generation publish/revoke、stale/owner-revoked typed outcome。
- 将contribution store与product registry安装合并为同一receipt，而不是两次独立publication。

### M2 · Invocation、authorization与protocol

- 所有UI/keyboard/menu/palette/control/CLI入口汇入gateway。
- 分离feature capability、principal permission、surface policy与project trust。
- 建立versioned envelope、payload schema validation、deadline/idempotency、typed error及audit chain。

### M3 · Keyboard与Keymap产品化

- 交付logical/physical sequence、scope/priority/repeat/location/AltGraph/platform primary modifier模型。
- 建立effective snapshot和typed resolution outcome。
- 交付Keymap Editor、冲突修复、orphan隔离、import/export、恢复默认与版本迁移。

### M4 · Menu与Palette统一投影

- built-in/plugin菜单统一owner-aware graph、section/anchor/dynamic state/profile/customization。
- 删除legacy prefix与重复ID mapping。
- palette补localized owner document、effective shortcut、cancellable query和generation-safe commit。

### M5 · Plugin executor与Automation闭环

- 完成host-known route或稳定plugin callback executor，并绑定owner generation/capability/unload revoke。
- commandlet从definition生成parser/help/discovery，executor按owner注册。
- automation支持progress/cancel/deadline/artifact和稳定report/exit协议。

### M6 · 规模、故障、安全与兼容验收

- 10k/100k command/menu/palette和1k owner churn benchmark。
- corrupted settings、deep when、oversized payload、stale generation、unload race、remote bypass、process cancel故障注入。
- Windows/macOS/Linux的IME、AltGraph、numpad、layout、logical/physical round-trip矩阵。

## 10. 24个资格门当前重判

| Gate | 状态 | 当前判定 |
|---:|---|---|
| 1 | Fail | SDK/wire/host仍不共享ID parser/golden corpus。 |
| 2 | Fail | serialized plugin Command没有可执行factory端到端。 |
| 3 | Fail | 缺executor/owner revoke/generation skew不能保证零部分注册。 |
| 4 | Fail | InvokeBinding/Route operation remote deny旁路仍在。 |
| 5 | Fail | provenance会被UiBinding/RetainedHost覆盖。 |
| 6 | Fail | CLI/local UI/remote/replay没有独立policy matrix。 |
| 7 | Fail | payload byte/depth/node/string/batch预算未定义。 |
| 8 | Fail | discovery/history无generation cursor分页。 |
| 9 | Partial | clone-then-assign提供局部atomicity，contribution store也可revoke；product registry/factory/menu/keymap/palette仍不能同generation撤销。 |
| 10 | Fail | 10k owner churn与callback leak/UAF/recovery无required evidence。 |
| 11 | Fail | When admission/eval/solver无depth/node/instruction/work budget。 |
| 12 | Partial | resolver能context eval并对多个enabled候选fail-close；缺typed priority/owner/outcome及产品修复。 |
| 13 | Fail | resolver Option不能区分五类以上outcome，orphan仍在effective map。 |
| 14 | Partial | menu与dispatch已用effective binding，palette/tooltip未同generation收敛。 |
| 15 | Fail | settings schema migration、损坏隔离、跨平台formatter round-trip未闭合。 |
| 16 | Partial | text focus已有局部按键优先级；IME composing、modal、viewport capture、global shortcut矩阵未证明。 |
| 17 | Partial | extension menu支持多层递归；built-in、typed entry、anchor/profile/dynamic state仍不统一。 |
| 18 | Partial | 100k ignored MRU membership benchmark存在；完整query/rebuild/commit阈值不成立。 |
| 19 | Partial | palette window request检查catalog generation；commit不携generation/selection token，revoke并发仍无typed stale行为。 |
| 20 | Fail | commandlet parser/help仍不由definition生成，新增项需改central match。 |
| 21 | Fail | report/cancel/deadline/partial artifact protocol不完整。 |
| 22 | Fail | 仍有source-shape测试，required行为/fault gate未覆盖关键旁路。 |
| 23 | Fail | 三平台logical/physical/layout/AltGraph/numpad golden矩阵缺失。 |
| 24 | Fail | legacy prefix、二段ID adapter与默认allow尚无migration deadline/absence test。 |

## 11. 跨报告边界

- Editor01拥有retained tree、popup/focus/hit-test、绘制和frame性能；本报告拥有menu/palette进入command authority的identity、state与dispatch。
- Editor02拥有operation transaction、undo/redo、journal持久化与recovery；本报告拥有解析、授权、provenance及调用operation factory之前的合同。
- Editor06/Editor50拥有Plugin Manager、extension contribution store与reload lifecycle；本报告要求command definition/executor/menu/keymap/palette以其owner generation原子安装和撤销。
- Runtime Interface与Plugin SDK报告拥有wire/ABI/version治理；共享`EditorCommandId`、versioned envelope和callback/host-route模型不能成为Editor私有语法。
- Runtime UI拥有通用keyboard/text/IME采集与呈现；本报告只定义Editor focus/modal/capture优先级和command binding消费合同。
- Tooling按用户当前范围继续排除；automation协议只审查Editor commandlet/control产品面，不扩展到独立工具链实现。

## 12. 本轮非结论

- 没有证明`UiControlRequest`当前已暴露到不受信任公网；P0是同一control boundary内部可静态确定的policy/provenance旁路。
- 没有声称palette整体性能差于参考引擎；它已有较好的有界结构，但required阈值、Unicode、owner churn和完整commit benchmark缺失。
- 没有运行Cargo、Editor、GUI、插件DLL、MCP/网络transport、键盘布局、race/fault/scale/soak或跨引擎benchmark，因而不宣称当前基线green。
- 没有把materializer fixture与parser的静态矛盾描述成动态测试失败；取得真实test receipt仍是实现前置验收。
- 没有查询、轮询、等待或实时跟踪协调器；本报告直接沿不受该状态阻塞的审查链完成。
