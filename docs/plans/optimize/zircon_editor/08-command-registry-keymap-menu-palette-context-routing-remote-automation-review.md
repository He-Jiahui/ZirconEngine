---
related_code:
  - zircon_editor/src/core/commands
  - zircon_editor/src/core/commandlet
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/core/plugin/materializer.rs
  - zircon_editor/src/core/settings
  - zircon_editor/src/ui/host/command_eval_projection.rs
  - zircon_editor/src/ui/host/editor_event_control_requests.rs
  - zircon_editor/src/ui/host/editor_event_dispatch.rs
  - zircon_editor/src/ui/host/editor_operation_dispatch.rs
  - zircon_editor/src/ui/binding_dispatch/editor_event_normalization.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/workbench
  - zircon_editor/src/ui/workbench/event
  - zircon_editor/src/ui/workbench/model/menu
  - zircon_plugins/plugin_sdk/src/editor_contribution.rs
  - zircon_runtime_interface/src/editor_contribution.rs
  - zircon_runtime_interface/src/ui/event_ui/control.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_editor/01-retained-ui-architecture-performance-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
  - docs/plans/optimize/zircon_plugins/01-plugin-sdk-package-catalog-distribution-native-abi-review.md
  - docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
  - docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md
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
  - dev/Graphics/Packages
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 08 · Command Registry、Keymap、Menu、Palette、Context Routing 与 Remote Automation 工程化差距

## 1. 结论

Zircon Editor的命令系统并非空壳。它已经有统一的`EditorOperationPath`、descriptor registry、operation factory、`WhenClause`、generation snapshot、键位override、menu/palette投影和headless commandlet；palette采用immutable catalog、byte posting、bounded heap/window和精确enablement复核，键位hash命中后还会做完整chord比较。这些基础应保留，不能退回到每个按钮直接写业务回调。

但当前“统一命令系统”只统一了部分metadata，没有统一身份、执行、授权、生命周期与呈现。最严重的三个断点都是确定的P0：

1. `UiControlRequest::InvokeBinding/InvokeRoute`绕过`callable_from_remote`，operation binding还把来源改写成`UiBinding/RetainedHost`，远程策略与审计来源同时失真。
2. public DTO、plugin SDK及materializer自身测试都接受`command.a`、`sample.command`、`fixture.command`这类二段ID，而宿主`EditorOperationPath`强制至少三段；公开合法值无法进入宿主。
3. 序列化插件`Command`只有ID和显示名，materializer却把它注册成`Operation` descriptor而不注册factory；即使修复ID，调用也必然落入`MissingFactory`。

这意味着继续增加菜单、快捷键或远程自动化会放大一组彼此不一致的旁路，而不是形成Unreal级别的可扩展Editor控制面。本报告记录3个P0、44个P1、10个P2。没有修改生产代码，也没有运行Cargo、真实Editor、插件DLL、MCP/网络transport、键盘布局矩阵或万级命令交互benchmark；确定性P0来自公开DTO、解析规则、路由分支和现有行为测试的静态闭环，不把尚未审查的网络暴露面推断成已存在的外网漏洞。

## 2. 审查边界与可复验证据

### 2.1 物理范围

| 子域 | 文件/行数/bytes | 本轮状态 |
|---|---:|---|
| command core与commandlet | 21 / 5,078 / 183,126 | E3：descriptor、registry、factory、when、key chord/keymap、menu、palette、commandlet parser/executor；fingerprint `74d04199...c1c7ffb` |
| product routing与presentation bridge | 24 / 3,730 / 158,889 | E3：settings snapshot、eval projection、control/event/operation dispatch、retained menu/palette、workbench menu identity与默认keymap资产；fingerprint `1caac178...005f6ad3` |
| public plugin/control boundary | 4 / 779 / 28,144 | E3：runtime interface contribution/control DTO、plugin SDK builder与Editor materializer；fingerprint `b86316bf...fb18653` |
| focused tests | 10 / 1,358 / 52,417 | E2：command、keymap、control binding、palette/menu routing测试源码；连同core inline tests共79个test attributes、0 ignored；fingerprint `ddf79850...4c701a` |
| selected combined scope | 59 / 10,945 / 422,576 | 当前工作树fingerprint `97c797b8...40dd6c6` |

fingerprint按相对路径排序，将`path + NUL + per-file SHA-256 + LF`拼接后计算SHA-256。它标识本轮阅读集合，不是schema或兼容性ID。

### 2.2 在途文件隔离

成文时`core/commandlet/{mod,runner,tests}.rs`、`core/commands/registry.rs`、`ui/binding_dispatch/editor_event_normalization.rs`及retained workbench的`command_palette.rs/menu_action.rs`处于其他Session或用户修改中。已核对的diff主要是rustfmt导入/断言布局，但它们仍标记`source_recheck_required`；实施前必须复取完整内容和fingerprint，不能把本报告当作对未来版本的自动证明。

工作树中还有大量Editor UI、gateway、asset与test在途修改。本轮没有回退、格式化、暂存或提交它们，也没有用无关dirty测试证明命令系统正确。

### 2.3 本轮追踪的产品链

1. built-in/plugin descriptor -> registry admission -> generation/palette catalog -> menu/palette/keymap projection。
2. keyboard input -> typed chord -> effective keymap -> command lookup -> context enablement -> event/operation execution。
3. menu/palette retained binding -> `EditorUiBinding` normalization -> `EditorEvent`或`EditorOperationInvocation` -> transaction/operation factory。
4. `UiControlRequest::{CallAction, InvokeRoute, InvokeBinding}` -> reflection/control boundary -> command/operation dispatch -> journal source。
5. serialized plugin contribution -> shared DTO/SDK -> native plugin batch -> Editor materializer -> command registry/factory lookup。
6. `--run` CLI -> hardcoded argument parser -> registry descriptor lookup -> capability check -> migration/plugin/automation executor -> JSON report。

UI树、绘制、popup geometry和layout persistence分别由Editor 01/02拥有；本报告只审查它们进入command authority的边界。

## 3. 已有工程基础，重构时必须保留

### 3.1 Canonical ID、registry与factory分离

- `EditorOperationPath`已经把业务命令从任意UI control ID中抽离，并由BTreeMap提供确定顺序。
- operation descriptor与`OperationCommandFactoryRegistration`分开保存，执行前会重新查descriptor、factory和enablement；缺factory会返回typed `MissingFactory`，不会静默成功。
- `CommandEvalSnapshotHandle`用`Arc<RwLock<...>>`发布generation snapshot，菜单、palette与dispatch可以共享只读上下文，而不必每次重新扫描整个Editor state。

### 3.2 Keymap正确性与输入边界的局部基础

- chord解析会规范modifier顺序，keymap使用signature bucket后仍比较完整chord，因此FNV碰撞不会直接误派发。
- override支持替换和显式解除绑定，设置snapshot更新后产品manager会重建effective keymap。
- native keyboard桥保留physical key、scan code、logical key与text；当前命令层没有充分使用这些信息，但无需重写底层事件采集。

### 3.3 Palette的有界查询结构

- catalog按registry generation缓存为immutable `Arc`，结果持轻量index handle，不复制整份descriptor。
- 查询以rarest byte posting缩小候选，用bounded heap保留窗口，MRU有32项上限；metrics记录visited entries、enablement eval和document byte visits。
- command commit后还会经registry重新解析和enablement检查，伪造或陈旧ID不会直接绕过业务权限。

### 3.4 控制失败与operation事务

- `CallAction`确实检查reflected action的`callable_from_remote`；operation dispatch的Remote source也检查descriptor gate。
- `ControlFailure`不是“失败却返回成功”：它执行为`Err`，control response保留failure，现有测试覆盖该合同。
- operation factory进入统一editing engine和journal，命令系统不应另造第二套undo/transaction实现。

## 4. P0：远程策略旁路与插件命令公开契约断裂

### E-CMD-P0-01 · `InvokeBinding/InvokeRoute`绕过remote-callable并洗白调用来源

确定调用链：

1. `UiControlRequest::CallAction`在`editor_event_control_requests.rs`检查`action.callable_from_remote`，证明该request family具有远程控制语义。
2. 同一入口的`InvokeRoute`和`InvokeBinding`没有等价gate。
3. `InvokeBinding`若携`EditorOperation` payload，会直接调用`invoke_operation_with_binding_path(EditorOperationSource::UiBinding, ...)`；`operation_source_requires_remote_callable`只对Remote返回true，因此descriptor gate被绕过。
4. 若携普通`EditorCommand`，入口以`EditorEventSource::Headless`调用`dispatch_binding`，但binding normalization/command resolution只复核enablement，不复核remote-callable；`MenuAction`还能直接归一化成event。
5. 现有`workbench_reflection_operation_binding_preserves_native_binding_provenance_and_transaction`测试通过`UiControlRequest::InvokeBinding`执行`scene.node.create_cube`并改变scene，最后断言journal source是`RetainedHost`。远程/direct control来源因此被记录成宿主UI来源。

修复必须在最外层建立统一`InvocationPrincipal + InvocationSurface + SourceProvenance`，所有CallAction/Route/Binding/Command/Operation在解析后、执行前走同一deny-by-default policy；来源只能追加转换stage，不能被`UiBinding`覆盖。需要回归证明remote principal无法通过任一binding/route执行deny命令，且journal、audit、telemetry保留原始principal/request ID。

### E-CMD-P0-02 · 插件SDK接受的命令ID被Editor宿主必然拒绝

public contract现状互相矛盾：

- `zircon_runtime_interface`测试构造`command.a`并认为batch有效。
- `zircon_plugins/plugin_sdk`测试通过builder构造`sample.command`并认为成功。
- Editor materializer的成功测试使用`fixture.command`。
- `EditorOperationPath::parse`却要求`segment_count >= 3`，materializer对Command及MenuItem的`command_id`都调用该parser。

因此“SDK验证通过 -> native batch加载 -> Editor materialize成功”不是可达合同；materializer自己的fixture也违反宿主语法。必须建立一个共享、版本化的`EditorCommandId`类型，DTO反序列化、SDK builder、menu reference、registry与operation route全部复用同一parser和golden vectors。若采用至少三段的`owner.domain.action`，二段旧值只能通过显式migration/diagnostic处理，禁止每层各自猜测补段。

### E-CMD-P0-03 · 序列化插件Command只能被发现，不能被执行

`SerializedEditorContribution::Command`只有`id/schema/display_name`。materializer把它转换为`EditorCommandDescriptor::operation(...)`并注册进cloned registry，却没有同步注册`OperationCommandFactoryRegistration`；DTO也没有callback ABI、route kind、payload schema、capability、owner或remote policy可供宿主生成factory。修复P0-02后，执行会稳定进入`OperationCommandFactoryError::MissingFactory`。

这不是补一个测试mock即可解决。序列化contribution必须选择清晰模型：要么声明受版本控制的host-known route并由宿主factory catalog解析，要么使用稳定plugin callback/IPC handle并受owner lease、capability和lifecycle约束；不能把只有显示信息的声明伪装成可执行operation。materialization验收必须实际从palette/menu/remote允许路径执行插件命令、观察transaction/result，再卸载owner并证明命令和factory一起消失。

## 5. P1：命令authority、输入、呈现与自动化缺口

### 5.1 Registry、descriptor与context authority

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| E-CMD-P1-01 | `EditorCommandRegistry`直接derive Deserialize；反序列化绕过`validate_descriptor`，可注入任意generation，operation factories和palette又因`serde(skip)`被清空。 | registry不是wire/persistence DTO；只序列化versioned declarative definitions，load后统一admission、owner绑定和factory resolution，再原子发布snapshot。 |
| E-CMD-P1-02 | descriptor、`EditorKeyChord`和`WhenClause`也derive Deserialize；constructor中的keyword/capability排序、chord规范化和其他不变量可被绕过。 | 对外字段使用validated newtype/custom Deserialize；所有字符串、集合、递归结构均有长度、数量和规范化上限。 |
| E-CMD-P1-03 | registry只有register，没有owner、registration token、unregister或批量撤销；插件卸载无法证明command/factory/menu/palette同时消失。 | `CommandRegistrationLease`绑定plugin/module generation，支持atomic register/revoke；旧handle调用返回OwnerRevoked而不是命中残留对象。 |
| E-CMD-P1-04 | `EditorCommandContributionSet`分别`take_pending`和`take_pending_factories`，已见ID永久保留；metadata与factory不是一次提交。 | 一个batch同时校验definitions、factories、menus和key defaults，成功才发布新generation；失败无部分可见状态。 |
| E-CMD-P1-05 | `descriptor_for_event`线性扫描并返回BTreeMap顺序的首个相同event；多个command emit同一event时逆向身份不唯一。 | 禁止用event反推command身份；event携resolved command/operation ID，或维护admission时验证的一对一reverse index。 |
| E-CMD-P1-06 | headless name/route查找与duplicate admission多处线性扫描，bulk register可退化为O(N²)。 | admission构建name、route、event和owner secondary indexes，给出命令数量、batch大小和构建时延budget。 |
| E-CMD-P1-07 | validation只覆盖少量menu/schema/headless/asset参数，不限制label、keyword、menu depth、when nodes、capabilities、schema长度，也不校验localization/owner/shortcut policy。 | `CommandDefinitionValidator`输出typed field/stage diagnostics并执行统一resource budget、identity、localization、authorization与collision规则。 |
| E-CMD-P1-08 | `WhenClause::{All,Any,Not}`可经serde构造任意深度/节点数，eval递归执行；恶意或损坏definition可造成热路径过载甚至栈耗尽。 | admission时编译成有最大节点/深度的迭代bytecode或DAG，拒绝超预算表达式；eval有固定instruction budget。 |
| E-CMD-P1-09 | context只描述project、undo/redo、document kind、scene mode、selection、asset writable、play和feature capabilities；没有focused control/text edit、modal/window/document instance、tool stack、principal和platform。 | typed `CommandContextSnapshot`包含focus scope、active document identity/revision、modal stack、input capture、tool/mode、caller principal及read-only reason；按generation增量发布。 |
| E-CMD-P1-10 | registry mutation用`checked_add(...).expect`，eval snapshot却允许wrapping；generation overflow与缓存失效合同不一致。 | 使用不可回绕的epoch token或typed overflow终态；所有cache、handle、commit在同一generation语义下验证。 |

### 5.2 Remote、control与operation policy

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| E-CMD-P1-11 | `EditorCommandDescriptor::new/operation`默认`callable_from_remote=true`，serde缺字段也回填true；大多数built-in未显式声明。 | remote/automation默认deny，只有经过review的surface-specific policy显式allow；migration报告列出每个旧命令的最终决策。 |
| E-CMD-P1-12 | CLI和remote共用单一`callable_from_remote`位；本机受控batch、MCP、测试replay和潜在网络client的风险模型被合并。 | policy按surface、principal、project trust、interaction requirement和side-effect class评估，CLI不借用remote许可位。 |
| E-CMD-P1-13 | `required_capabilities`表示当前feature是否可用；remote context拿到所有enabled capabilities，因此它不是caller authorization。 | 分离`FeatureCapabilitySet`与`PrincipalPermissionSet`，先验证产品能力，再验证caller scope/role/project grant。 |
| E-CMD-P1-14 | `EditorOperationControlRequest/Response`是无version的externally tagged serde DTO，没有request/correlation/idempotency key。 | versioned envelope包含request ID、session/project identity、deadline、idempotency key、protocol feature和response correlation。 |
| E-CMD-P1-15 | invocation arguments与operation group是无边界`serde_json::Value/String`；control入口未定义bytes/depth/node/count/timeout上限。 | decode前执行frame/byte限制，decode中执行depth/node/string/array限制；operation group、batch与结果也有独立budget。 |
| E-CMD-P1-16 | `ListOperations`一次物化全部enabled remote descriptor为JSON，不分页、不带catalog generation，也不返回disabled reason。 | cursor + catalog generation + page/byte limit；可选择列出disabled命令及typed reason，stale cursor明确失败。 |
| E-CMD-P1-17 | `QueryOperationHistory`固定返回全局最近128项，没有cursor、project/document/principal/filter或snapshot token。 | 历史API按project/document/principal/time/command过滤并分页，返回stable sequence/cursor；机械存储仍由Editor 02拥有。 |
| E-CMD-P1-18 | Menu和UiBinding普通事件都折叠为`RetainedHost`；headless/control旁路还可进入同一标签，审计无法还原入口。 | provenance为不可覆盖链：principal -> transport -> request -> route/binding -> resolved command -> operation -> transaction。 |
| E-CMD-P1-19 | `payload_schema_id`只检查字符串语法；没有caller用它验证arguments，也未接Runtime Interface的schema catalog。 | schema ID解析到versioned typed codec/validator，执行前验证payload，响应记录schema/version和validation stage。 |
| E-CMD-P1-20 | control/commandlet大量错误退化成自由文本，缺typed code、stage、retryability、field path和remediation。 | 统一`CommandInvocationError`及稳定wire projection；内部cause chain保留，外部字段受版本和敏感信息策略约束。 |

### 5.3 Keymap与keyboard routing

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| E-CMD-P1-21 | 同一signature bucket的完全相同chord按command ID排序，`resolve`静默返回字典序首项；`conflicts()`只报告不参与admission/dispatch。 | conflict是设置事务的一等结果；按scope/context/priority解析，无法唯一决定时禁用并向用户显示双方owner和修复入口。 |
| E-CMD-P1-22 | keyboard先选一个command再检查enablement；若冲突首项disabled，不会继续选择同chord下另一个enabled命令。 | resolver在同一context snapshot内评估全部候选，要求唯一最高优先级；disabled原因和shadowed结果可诊断。 |
| E-CMD-P1-23 | override可引用未知command；未知binding能截获按键后得到UnknownCommand，阻断原有效命令。 | 设置写入与registry generation联检；orphan binding隔离显示，不能进入active dispatch index。 |
| E-CMD-P1-24 | default TOML与descriptor的`default_chord`是两套authority；测试只检查“有默认的descriptor在文件里出现”，不验证chord相等或多余/未知项。 | 默认键位只有一个typed source，build时生成descriptor展示/默认文件；golden test验证双向集合、值与平台变体完全一致。 |
| E-CMD-P1-25 | menu和palette展示descriptor default，而dispatch使用effective keymap；用户override/tombstone后UI继续显示旧快捷键。 | 所有surface订阅`EffectiveKeymapSnapshot`，展示、冲突、tooltip和dispatch共享同一binding generation。 |
| E-CMD-P1-26 | 全量限定检索只找到settings、host重建和测试，没有产品级keymap浏览/搜索/录制/冲突解决/恢复默认入口。 | 提供按command/context/owner搜索的Keymap Editor，支持录制、平台预览、冲突对比、单项/分组恢复、import/export和migration diagnostic。 |
| E-CMD-P1-27 | chord serde可绕过`new/FromStr`形成空key或非规范化key；持久化数据没有schema/version。 | `KeyBindingDocumentV2`用validated event sequence、platform/layout policy和migration版本，损坏项隔离而非污染active map。 |
| E-CMD-P1-28 | chord只能表达一个logical key和五个modifier，没有sequence、scope、priority、left/right、numpad/location、primary modifier或AltGraph策略。 | 输入模型区分logical/physical binding、platform primary modifier、sequence timeout、scope/context和repeat policy。 |
| E-CMD-P1-29 | native event已有physical key/scan code/text，resolver却只用logical key及有限Windows fallback；也没有repeat字段和IME/text-editor优先级。 | `KeyboardRoutingService`先处理IME/text/modal/input capture，再按binding policy选择logical或physical token；repeat与focus转移有确定合同。 |

### 5.4 Menu model与路由

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| E-CMD-P1-30 | built-in `menu.rs`只取`top/item`，更深路径被压成一个label；extension menu却递归建树，同一metadata有两种层级语义。 | 一个`MenuGraphBuilder`解析完整path/section/anchor，built-in和extension走同一算法与collision规则。 |
| E-CMD-P1-31 | `MenuItemModel`只有label/action/operation/shortcut/enabled/children，无法表达separator、section、check/radio、visibility、icon、tooltip、disabled reason和accessibility。 | 使用typed entry union及动态state provider；呈现层不从空字段组合推断entry kind。 |
| E-CMD-P1-32 | 顶层菜单固定File/Edit/Selection/Play/View/Window/Help；extension只靠path/priority追加，没有owner-scoped anchor、section、before/after、profile或定制。 | owner-aware hierarchical menu registry支持anchor/section/insertion order、dynamic context、profile/customization及owner revoke。 |
| E-CMD-P1-33 | menu path、`MenuAction` canonical ID、control ID、operation path和legacy aliases散落在`menu_action_id/from_id/item_binding/defaults`。 | 一个versioned command/menu identity catalog生成codec与UI binding；禁止手写平行字符串映射。 |
| E-CMD-P1-34 | retained menu仍识别`SavePreset.*`/`LoadPreset.*`等字符串前缀并直接构造`LayoutCommand`，绕过registry、when、capability和统一审计。 | legacy ID只在边界migration adapter解析为canonical command；所有业务执行回到command service，迁移期记录使用量后删除adapter。 |
| E-CMD-P1-35 | 没有action/operation的item仍会生成空`MenuAction` binding，而不是不可交互separator/header。 | entry kind在model层确定；非action节点不注册hit target、binding或shortcut。 |
| E-CMD-P1-36 | built-in menu每次投影克隆字符串/重建树，extension duplicate检查递归扫描；没有generation/context cache或规模budget。 | immutable menu graph按definition generation构建，动态state增量刷新；定义万级entry构建、打开菜单和submenu延迟budget。 |

### 5.5 Command Palette

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| E-CMD-P1-37 | catalog document只做ASCII lowercase，query用Unicode lowercase；非ASCII大写label/keyword无法可靠匹配小写查询，byte fuzzy也不做Unicode normalization。 | 统一locale-aware或明确locale-independent的NFKC/case-fold索引；保存原文用于高亮，按Unicode scalar/grapheme计算匹配位置。 |
| E-CMD-P1-38 | 搜索document只有display name、ID、category和keywords，缺description/menu path；`source`只是category，不是builtin/plugin owner。 | catalog entry包含localized label/description/path、owner/package、effective shortcut、aliases、deprecation和typed source。 |
| E-CMD-P1-39 | query无最大长度、debounce、cancel或frame budget；每次编辑同步构造normalized string/prefix并扫描候选。 | 输入和查询都有byte/scalar上限；大catalog在worker执行可取消query，UI只接受匹配catalog/query generation的窗口。 |
| E-CMD-P1-40 | 性能测试只打印1,000命令的p95，不断言阈值，也不覆盖Unicode、冲突、插件 churn和worst-case common byte。 | required benchmark定义10k/100k catalog、冷/热query、增删owner、Unicode和worst-case分布的p50/p95/p99、allocation与frame预算。 |
| E-CMD-P1-41 | UI commit只携任意command ID，不携catalog generation或选中handle；registry会复核安全性，但用户可点击旧结果后收到无上下文错误。 | commit携catalog generation、command ID和selection token；stale/revoked结果返回typed refresh outcome并保持可理解的UI状态。 |
| E-CMD-P1-42 | menu显示disabled项，palette直接过滤disabled；两者都没有统一disabled reason，发现性和可解释性不一致。 | surface policy决定hide/disable，但enablement由同一evaluation result提供`enabled + reason + remediation`。 |

### 5.6 Commandlet与headless automation

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| E-CMD-P1-43 | registry虽存headless metadata，parser/executor仍硬编码三个commandlet、全部flag和三个concrete action variant；注册descriptor不会产生可运行CLI。 | versioned `CommandletDefinition`声明typed args/schema/executor factory，通用parser从catalog生成；domain executor由owner lease注册。 |
| E-CMD-P1-44 | report无schema version、request/correlation ID、timestamps、duration、progress/cancel和typed diagnostic；default runner又硬编码两项capability，authoring automation没有对应required capability。 | 统一automation envelope、policy context和lifecycle；支持progress/cancel/deadline、structured artifacts/diagnostics及稳定exit mapping，capability来自definition而非runner分支。 |

## 6. P2：长期维护、可发现性与观测缺口

| ID | 差距 | 后续方向 |
|---|---|---|
| E-CMD-P2-01 | `EditorCommandCategory`是closed enum，插件只能挤入内置分类。 | stable category ID + localized label + owner贡献，内置分类只是预注册项。 |
| E-CMD-P2-02 | command缺description/help URL/example/side-effect class/deprecation/replacement metadata。 | 扩展definition并由palette、设置、automation discovery和诊断共同消费。 |
| E-CMD-P2-03 | menu label和shortcut最终退化成裸String，没有localization key、mnemonic或screen-reader语义。 | locale资源键、mnemonic冲突解析和accessible description作为typed metadata。 |
| E-CMD-P2-04 | palette MRU固定32项，只有全局列表，没有workspace/profile、pin或privacy policy。 | MRU作为versioned user profile，支持workspace scope、pin、清除和敏感命令排除。 |
| E-CMD-P2-05 | palette用u8 score并以ID稳定打平，无可解释rank breakdown或质量回归corpus。 | rank feature记录与golden query corpus，算法升级有版本和质量/延迟双gate。 |
| E-CMD-P2-06 | chord parser允许`F10000`等任意函数键名，native fallback只覆盖F1-F12。 | key vocabulary与平台输入能力表共同生成；不可产生的token在admission时报错。 |
| E-CMD-P2-07 | logical/physical显示名称、macOS符号、非US布局与键盘位置没有统一formatter。 | platform/layout-aware formatter与round-trip golden matrix。 |
| E-CMD-P2-08 | registry/eval generation overflow策略不一致且依赖`u64`数字暴露。 | opaque epoch/revision token，序列化和日志仅输出诊断表示。 |
| E-CMD-P2-09 | 没有按source/owner统计dispatch latency、denied/disabled/unknown、key conflict、stale palette commit和factory failure。 | bounded command telemetry，默认不记录敏感payload，支持owner与surface维度聚合。 |
| E-CMD-P2-10 | 79项聚焦测试缺InvokeRoute remote gate、InvokeBinding deny、plugin command端到端执行/卸载、Unicode搜索、冲突fallback和keymap UI合同。 | 把这些加入required behavior/fault/compat lanes，source-shape断言不能替代执行测试。 |

## 7. 参考源码对照结论

| 能力 | Zircon现状 | 参考源码提供的可借鉴机制 | 结论 |
|---|---|---|---|
| Command action/context | descriptor有when与factory，但上下文和lifecycle不完整 | Unreal `FUICommandList`映射execute/can-execute/check/visibility/repeat，支持append与unmap | 借鉴action mapping、context composition和显式unmap，不照搬宏/全局单例。 |
| Input binding | 单chord、全局override、静默冲突首选 | Unreal InputBindingManager维护context、active/default/user chord与冲突；Godot Shortcut可持多个InputEvent并复用事件match | Zircon需先统一typed event sequence与scope，再构建用户编辑器。 |
| Menu extension | built-in/extension两种树算法，无owner revoke | Unreal ToolMenus具有owner、unregister、section、dynamic context、profile/customization | owner lease、anchor/section和profile是工程级插件菜单的必要控制面。 |
| Palette | immutable catalog与有界heap较好，但Unicode、owner和execution lifecycle不足 | Godot palette支持add/remove callable、history、shortcut设置变更刷新和300项展示上限 | 保留Zircon查询结构，补生命周期、effective shortcut、Unicode及端到端执行。 |
| Editing command | operation factory进入transaction，但plugin serialized command不闭环 | Fyrox CommandStack明确execute/revert/finalize并做容量淘汰；其key settings覆盖多工具域 | 事务仍由Editor 02/operation engine拥有；command service只负责解析、授权与路由。 |
| UI菜单示例 | retained surface复杂，但不代表command authority完整 | Bevy checkout只有UI/menu示例，没有完整Editor command registry | Bevy只作为typed UI composition参考，不能用来降低Editor command标准。 |
| Unity Graphics | 本轮没有找到Editor command shell权威实现 | `dev/Graphics`是graphics packages/test/tooling局部源码 | 不推断闭源Unity Editor命令系统，也不以缺失参考为Zircon旁路辩护。 |

参考实现不是复制清单。Unreal的owner/context/action/tool menu分层最接近目标控制面；Godot证明shortcut/palette需要真实add/remove和设置刷新；Fyrox证明编辑命令生命周期与快捷键域都必须显式。Zircon应保持Rust typed ownership、immutable snapshot和有界查询优势，同时补上这些工程边界。

## 8. 目标架构

```text
Plugin/Builtin/Tool Owner
        |
        v
CommandRegistrationBatch --validate--> CommandDefinitionCatalog
        |                                  |
        | atomic owner lease               +--> EffectiveKeymapSnapshot
        |                                  +--> MenuGraphSnapshot
        |                                  +--> PaletteCatalogSnapshot
        v
CommandExecutorCatalog
        ^
        |
InvocationGateway
  - UI / Keyboard / Menu / Palette
  - CLI / MCP / Remote / Replay
        |
        v
Resolve -> Authorize -> Evaluate Context -> Validate Payload -> Execute
        |                                             |
        +--> immutable provenance/audit               +--> Operation/Transaction Engine
```

核心类型与owner：

1. `EditorCommandId`：共享到runtime interface和plugin SDK的versioned newtype，唯一语法和golden vectors。
2. `CommandDefinition`：纯声明metadata、context program、payload/result schema、side-effect class和surface policy；不可直接携Rust event副本。
3. `CommandRegistrationLease`：owner/module/plugin generation的唯一撤销凭证，批量管理definition、executor、menu和default binding。
4. `CommandInvocationGateway`：所有surface唯一入口，生成`InvocationContext`并按Resolve/Authorize/Evaluate/Validate/Execute阶段返回typed outcome。
5. `CommandContextSnapshot`：按document/window/focus/principal发布immutable generation，menu/palette/keymap共享evaluation cache。
6. `KeymapService`：typed sequence、scope、conflict resolver、user document migration和effective snapshot唯一owner。
7. `MenuService`：owner-aware graph、section/anchor/profile/dynamic state，presentation只消费snapshot。
8. `PaletteService`：localized immutable index、cancellable query、generation-safe commit和MRU profile。
9. `AutomationService`：versioned discovery/invoke/history/progress/cancel envelope，策略与UI共享definition但不共享默认授权。

## 9. Hard Cutover 规则

1. 先冻结新增直接`MenuAction`/binding/route旁路；新命令只能通过registration batch进入。
2. 引入共享`EditorCommandIdV2`并让DTO、SDK、registry、menu、operation和tests一次切换；不长期保留二段/三段双parser。
3. `callable_from_remote`默认立即改deny，并为每个已有命令建立显式migration manifest；未知命令不得自动allow。
4. `InvokeRoute/InvokeBinding/CallAction`先统一进入InvocationGateway，再删除内部source改写和重复gate。
5. serialized plugin command在新executor contract可用前必须拒绝materialize并给typed diagnostic，禁止继续注册不可执行descriptor。
6. registry serialization删除；需要持久化的只有versioned user keymap/MRU/menu customization，不保存runtime factory和generation。
7. effective keymap成为展示与dispatch唯一authority，删除descriptor/default TOML的双向手工维护。
8. built-in与extension menu统一成MenuGraph后，删除legacy prefix execution和旧平行ID codec。
9. commandlet切到definition-driven parser/executor后删除三个hardcoded action/parser分支，不长期双跑。
10. 任何兼容adapter必须有使用telemetry、截止版本和删除测试；不能以“插件可能依赖”为由永久保留旁路。

## 10. 分层实施里程碑

### M0 · P0封堵与契约冻结

- 为InvokeBinding/InvokeRoute增加统一deny test和provenance test，临时封堵所有未显式允许的control调用。
- public DTO/SDK/materializer共享同一ID验证；修正现有二段fixtures。
- 在executor ABI完成前拒绝serialized Command，避免继续产生可发现但不可执行条目。

### M1 · Definition、owner lease与atomic registry

- 建立`EditorCommandIdV2`、`CommandDefinitionV2`、typed validation和resource budgets。
- 实现owner registration batch、secondary indexes、atomic publish/revoke及stale handle结果。
- 去除registry/descriptor不安全serde入口，WhenClause编译为有界context program。

### M2 · Invocation、authorization与provenance

- 所有UI/keyboard/menu/palette/CLI/control入口汇入InvocationGateway。
- 分离feature capability、principal permission和surface policy；remote默认deny。
- 建立versioned request/response、payload schema validation、deadline/idempotency、typed error与不可覆盖audit chain。

### M3 · Keymap与keyboard routing

- 建立logical/physical sequence、scope/priority/repeat和platform primary modifier模型。
- effective keymap snapshot统一dispatch、menu、palette和tooltip。
- 交付Keymap Editor、冲突诊断、orphan隔离、import/export和版本迁移。

### M4 · Menu与Palette统一投影

- built-in/plugin菜单统一owner-aware graph、section/anchor/dynamic state/profile。
- 删除legacy menu prefix和重复ID codec。
- palette补localized index、owner/description/effective shortcut、cancellable generation-safe query/commit。

### M5 · Plugin executor与commandlet闭环

- 选择并实现host-known route或稳定plugin callback executor；绑定plugin generation、capability和卸载撤销。
- commandlet由definition生成parser/help/schema，executor按owner注册。
- automation支持discovery/invoke/progress/cancel/artifact及稳定结果协议。

### M6 · 规模、故障、兼容与安全验收

- 10k/100k command/menu/palette与1k plugin owner churn benchmark。
- 损坏settings、深when、超大payload、stale generation、卸载竞态、remote旁路和进程取消故障注入。
- Windows/macOS/Linux键盘布局、IME、AltGraph、numpad和logical/physical round-trip矩阵。

## 11. 必须通过的验收门

1. SDK、wire DTO和Editor host共享同一command ID golden corpus；任一层接受的值其余层必须接受。
2. serialized plugin Command从加载、菜单/palette发现、执行、transaction/result到卸载撤销形成真实端到端测试。
3. plugin缺executor、重复ID、owner撤销和generation skew均返回typed diagnostic且不部分注册。
4. Remote/MCP principal对deny command通过CallAction、InvokeRoute、InvokeBinding、MenuAction和Operation五条路径全部被拒绝。
5. 允许的remote调用在journal/audit中保留principal、transport、request、route、command和operation完整provenance。
6. CLI、本地UI、remote和replay分别有policy matrix；不存在一个bool同时授权全部surface。
7. payload在byte/depth/node/string/batch上限前后均有测试，拒绝发生在业务factory执行前。
8. control discovery/history按generation cursor分页，stale cursor、超限与取消有稳定结果。
9. registry batch失败零可见；owner revoke后definition、factory、menu、keymap active binding和palette entry同generation消失。
10. 10k owner churn下无悬挂callback、UAF、重复ID泄漏或不可回收catalog。
11. When program超过depth/node/instruction budget在admission被拒绝；最大合法表达式eval满足frame budget。
12. 相同chord多候选在不同context下确定解析；同优先级歧义不静默选择字典序首项。
13. disabled首候选不会遮蔽唯一enabled候选；orphan override不截获active command。
14. 修改/解除快捷键后menu、palette、tooltip和dispatch在同一generation更新。
15. Keymap settings跨schema迁移、损坏项隔离、恢复默认和跨平台formatter round-trip通过。
16. IME composing、text field、modal、viewport input capture和global shortcut优先级有产品行为测试。
17. 三层以上submenu、separator、check/radio、dynamic visibility、owner anchor与profile customization行为一致。
18. 100k palette catalog的cold/warm common-byte/Unicode查询满足明确p95/p99与allocation预算，测试必须断言阈值。
19. palette query取消与owner revoke并发时旧结果不能执行，UI收到typed stale outcome。
20. commandlet help/parser完全由definition生成；新增fixture commandlet无需修改central match。
21. commandlet report带schema/request/timing/diagnostic，cancel/deadline退出码与partial artifact规则稳定。
22. registry、keymap、menu、palette和automation没有source-shape测试替代行为测试的required gate。
23. Windows/macOS/Linux至少各一套logical/physical/layout/AltGraph/numpad golden矩阵通过。
24. 旧menu prefix、二段ID adapter和默认allow policy达到迁移截止版本后有absence test，确保硬切完成。

## 12. 跨报告边界

- Editor 01拥有retained tree、hit test、popup geometry、绘制和frame性能；本报告拥有menu/palette进入command authority的identity、state与dispatch。
- Editor 02拥有operation transaction、undo/redo、journal持久化和recovery；本报告拥有命令解析、授权、provenance及调用operation factory之前的合同。
- Editor 06拥有Plugin Manager lifecycle、enablement、reload和contribution store；本报告拥有plugin command definition/executor/materialization与owner撤销语义。
- Plugins 01与Runtime Interface 01/02拥有SDK/ABI/DTO版本治理；本报告要求它们共享`EditorCommandIdV2`和automation envelope，不另造Editor私有wire语法。
- Runtime UI 11A-C拥有通用UI输入/文本/GPU呈现；本报告只定义Editor keyboard routing、IME/focus优先级和command surface消费合同。

## 13. 本轮非结论

- 没有证明当前`UiControlRequest`已暴露到不受信任公网；P0结论是同一公开control boundary内部存在确定的policy/provenance旁路。
- 没有声称Zircon palette整体慢于Unreal/Godot；当前算法有有界结构，但缺required阈值、Unicode正确性和大规模产品验证。
- 没有把Bevy或Unity Graphics仓中缺失的完整Editor command shell解释成“不需要该能力”。
- 没有要求复制Unreal API形状；目标是达到同等级的owner、context、authorization、extension和lifecycle完整性，同时保留Zircon的typed Rust与immutable snapshot优势。
