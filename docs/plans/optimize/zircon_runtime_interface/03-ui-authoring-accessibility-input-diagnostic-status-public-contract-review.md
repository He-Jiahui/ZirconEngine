---
related_code:
  - zircon_runtime_interface/src/ui
  - zircon_runtime_interface/src/runtime_api/session/requests.rs
  - zircon_runtime_interface/src/runtime_api/session/events.rs
  - zircon_runtime_interface/src/runtime_api/session/operation.rs
  - zircon_runtime_interface/src/runtime_api/abi/api_table.rs
  - zircon_runtime/src/ui
  - zircon_runtime/src/asset/assets/ui
  - zircon_runtime/src/dynamic_api/frame.rs
  - zircon_runtime/src/dynamic_api/session/runtime_ui.rs
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/operation
  - zircon_runtime_host/src/foreign_output
  - zircon_editor/src/ui/asset_editor
  - zircon_editor/src/ui/control
  - zircon_editor/src/ui/reflection
  - zircon_editor/src/ui/workbench/debug_reflector
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime_interface/01-runtime-dll-abi-ffi-version-handle-foreign-ownership-review.md
  - docs/plans/optimize/zircon_runtime_interface/02-serialization-reflection-resource-project-world-sync-public-dto-contract-review.md
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11b-runtime-text-font-shaping-layout-editing-ime-review.md
  - docs/plans/optimize/zircon_runtime/11c-gpu-ui-renderer-atlas-sdf-batch-clip-submit-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
  - docs/plans/optimize/zircon_editor/23-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Widgets/Accessibility/SlateCoreAccessibleWidgets.h
  - dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Widgets/Accessibility/SlateAccessibleWidgets.h
  - dev/UnrealEngine/Engine/Source/Runtime/SlateCore/Public/Input/Reply.h
  - dev/UnrealEngine/Engine/Source/Runtime/UMG/Public/Blueprint/WidgetBlueprintGeneratedClass.h
  - dev/UnrealEngine/Engine/Source/Runtime/UMG/Public/Blueprint/WidgetTree.h
  - dev/godot/core/input/input_event.h
  - dev/godot/scene/gui/control.h
  - dev/godot/core/io/resource_loader.h
  - dev/godot/core/error/error_list.h
  - dev/godot/core/error/error_macros.h
  - dev/bevy/crates/bevy_a11y/src/lib.rs
  - dev/bevy/crates/bevy_ui/src/accessibility.rs
  - dev/bevy/crates/bevy_ui/src/ui_node.rs
  - dev/bevy/crates/bevy_diagnostic/src/diagnostic.rs
  - dev/Fyrox/fyrox-ui/src/message.rs
  - dev/Fyrox/fyrox-ui/src/lib.rs
  - dev/Fyrox/fyrox-core/src/pool/handle.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugDisplaySerializer.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugManager.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Lighting/ProbeVolume/ProbeVolume.Migration.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 03 · UI Authoring、Accessibility、Input、Diagnostic、Status 公共合同工程化差距

## 1. 结论

`zircon_runtime_interface::ui` 已经不是空壳：它包含 retained tree、layout、dispatch、focus、binding、component contract、authoring document、compiled package、accessibility snapshot、reflection/control 和 debug snapshot 等大量类型；Runtime 也确实能构建 UI surface、处理输入、导出 accessibility JSON。问题恰恰在于这些能力尚未被组织成可发布的工程合同。

当前最危险的第一条链是 UI 作者数据。仓内同时存在递归 `UiTemplateDocument`、v1 `UiAssetDocument` 与 flat-map v2 `UiV2AssetDocument` 三套活跃格式。Editor 打开 `.zui` 时会把 v2 投影到 legacy v1，保存时再重建 v2；该投影明确把 `repeat`、`slots`、focus、navigation、picking、accessibility、widget 等字段置空，并用 `toml::to_string_pretty` 重写全文。Editor 23 已登记这一具体数据损失行为；本文拥有其公共合同根因：缺少唯一 canonical authoring IR、lossless syntax tree、unknown-field preservation 与显式 migration registry。

第二条 P0 是生产端资源耗尽。v1/v2 loader 使用 `read_to_string` 和 `toml::from_str` 读取任意大小、任意嵌套输入；公共 DTO 广泛暴露无界 `Vec`、`String`、`BTreeMap`、递归节点和 `toml::Value`。Accessibility capture 又忽略请求中的 `generation_hint`，总是构建并 JSON 序列化完整多 surface 快照，再注册 foreign allocation。host consumer budget 只能在分配完成后拒绝，不能保护 producer 的内存、CPU、栈和暂停时间。

第三条 P0 是 accessibility action 身份。快照没有 generation/epoch，动作请求也不携带 tree/snapshot generation；Runtime 把 surface index 填入 node ID 高 16 bit，把本地 ID 截断为低 48 bit，随后只靠这个整数反解目标。过期快照、节点复用或超出位宽的本地 ID 都可能把动作送到另一个当前节点。`UiAccessibilityActionResult` 虽声明 StaleTarget 等状态，却没有生产 consumer，动态事件只返回 `ZrStatus`，因此协议无法向调用方证明动作作用于它观察到的对象。

更广泛的结构问题是：一个自称稳定 ABI/DTO 的 crate 在 UI 子树中同时承载 source syntax、编译制品、mutable runtime tree、layout engine、ECS projection、input reducer、debug visualizer 和跨边界 JSON。UI 子树已有 642 个公开 struct/enum/type/trait，80 个文件使用 `#[serde(default)]`，没有一个 `deny_unknown_fields`；同时存在 45 个以 Diagnostic/Status/Report/Result 命名的公共类型，却没有统一 code、severity、source span、correlation、budget、truncation 或 receipt。

本轮登记 3 项 P0、72 项 P1、12 项 P2，均为 `pending`。整改顺序应是先阻断无损保存、producer budget 与 generation-qualified accessibility，再拆开 Authoring/Compiled/Runtime/Transport/Diagnostics 五类合同；之后才能收敛 remote control、input side-effect receipt、operation state machine 和 host output。继续在现有公开 struct 上追加可选字段，会扩大而不是解决兼容面。

## 2. 审查边界与证据

### 2.1 物理范围

| 集合 | 文件 / 物理行 / bytes | 证据等级与边界 |
|---|---:|---|
| `zircon_runtime_interface/src/ui` | 232 / 25,568 / 804,699 | E3：逐模块、公开类型、serde 面、authoring/compiled/runtime/debug/control/accessibility 纵向链 |
| entire interface Rust source | 442 / 55,571 / 1,815,527 | E2/E3：用于确认 UI 占比、公共类型和 runtime API/operation 交叉合同 |
| UI public declarations | 642 | E3 source scan：`pub struct/enum/type/trait`；不是稳定 ABI 数量，而是变更面规模 |
| UI serde pressure | 431 个 `#[serde(default)]` 命中 / 80 文件 | E3；0 个 `deny_unknown_fields`，13 文件含 `toml::Value`，43 文件含 `usize` |
| UI diagnostic/status/result families | 45 | E3：只计名称匹配的公开类型，尚不含所有裸 `String` 错误和顶层 `ZrStatus` |
| cross-crate consumers | Runtime、Runtime Host、Editor | E3：追踪 authoring load/save、runtime surface、dynamic export、control/reflection 和 foreign output |

UI source 指纹为 `7eecef1b729bf604f503ad656535ddcd8a48c6f8a02827458e2a924ee0cc2707`。算法是相对路径排序、逐文件 SHA-256，再对 `path<TAB>hash<LF>` 清单取 SHA-256。它固定的是本轮观察点，不是实施 baseline。

成文时 `zircon_runtime_interface` 有 15 个在途 source/test 文件，包含 `requests.rs`、`operation.rs`、API table、owned-result tests 与 accessibility tests；本文不修改、不回退，也不对未提交状态背书。`zircon_runtime_host::foreign_output` 已取代 Interface 02 成文时未跟踪的 `host_output` 草案，并由 App/Editor 共享七类 consumer budget，这是重要正向收敛；但 accessibility 尚未纳入该 kind/policy，且 consumer-side gate 仍不能代替 producer admission。因此 `source_recheck_required: true`。

### 2.2 Owner 边界

1. Interface 01继续拥有 C ABI table、foreign buffer ownership、build identity、FFI carrier、通用 handle/session/thread/callback 安全；本文不重复设计 allocator。
2. Interface 02继续拥有通用 Schema Catalog、resource/reflection/world-sync 和 generic diagnostic envelope；本文定义 UI profile 如何采用它们。
3. Runtime 18/19/20拥有 tree/layout/input/accessibility/text/GPU UI 的执行算法与性能；本文只审公共 source/runtime/transport 合同。
4. Editor 23拥有当前 `.zui` 打开、编辑、保存、undo 与 UI authoring 产品行为；本文拥有三套公共 authoring model 的收敛与迁移规则。
5. Editor 08拥有 remote automation 的 principal/capability gateway；本文拥有 `UiControlRequest/Response` 作为 domain protocol 的 revision、identity 和 receipt。
6. Editor 11拥有日志产品、保留、搜索、导出和用户呈现；本文拥有跨 runtime DLL/UI/host 的结构化 diagnostic/status 载体。

### 2.3 纵向调用链

本轮不是只读 struct，实际追踪了以下链：

1. legacy/v1/v2 TOML -> Runtime loaders/validators -> Editor v2-to-legacy projection -> edit -> legacy-to-v2 rebuild -> pretty TOML save；
2. `UiV2AssetDocument` -> compile/package manifest -> runtime asset loader/cache -> surface instance；
3. retained `UiTree` -> layout/dispatch/ECS/render extract -> surface/debug/reflection snapshot；
4. dynamic API accessibility request -> full snapshot build -> global node-ID packing -> JSON -> owned allocation -> action JSON decode -> target dispatch；
5. `UiControlRequest` -> Runtime/Editor control service -> property/action mutation -> reflection diff/subscription；
6. input event -> dispatch reply/effect/host request -> Runtime side effects与debug report；
7. operation submit -> admission/worker/apply -> poll phase/detail -> harvest outcome -> Runtime Host foreign-output budget；
8. focused interface tests、Runtime/Editor consumers、reference engines 的 authoring/runtime/accessibility/input/diagnostic ownership。

### 2.4 动态证据边界

本轮没有重复执行当前不可达的 Cargo lane。此前 `zircon_editor --lib` 测试编译在 617.2 秒后被 239 个既有错误和 122 个 warning 阻断；这些错误不是本文产生，也不能证明 UI contract tests 通过。当前 Interface/Runtime API 文件仍在并发修改，重复相同命令只会消费共享构建资源而不增加产品行为证据。

因此本文结论主要为 E3 静态纵向证据。实施时必须重新取 source fingerprint，并在编译基线恢复后执行本文第 10 节的兼容、恶意输入、故障和跨进程资格门。

### 2.5 参考源码给出的最低基线

- Unreal Slate 的 `FSlateAccessibleWidget` 持有独立 accessible ID、parent/children 与更新通知，并按 activatable/property/text/window capability 拆接口；不是每次导出一个无世代 JSON 树。`FReply` 把 capture/focus/navigation 请求交回 `SlateApplication` 统一执行，而不是让任意 handler 直接伪造系统状态。UMG 又把 Widget Blueprint generated class、WidgetTree 与 runtime widget instance 分层。
- Fyrox `UiMessage` 显式携带 destination、direction、handled 与 flags，`Handle<T>` 保存 index + generation；这直接说明路由目标和对象代际不能压成一个可复用裸整数。
- Godot 使用 typed `InputEvent` 层次和 `Control` 的事件传播，把 resource format recognition、dependency 与 UID 查询交给 `ResourceLoader/ResourceFormatLoader`；稳定 `Error` 与 error handler 又保留文件、函数、行等来源上下文。
- Bevy accessibility 以 AccessKit node component 与 ECS entity/world 生命周期集成，UI node 也是内部 ECS component；它适合参考内部投影，不可被误用为稳定 DLL wire。其 diagnostic 使用 `DiagnosticPath`、measurement/history/suffix 等明确身份和保留策略。
- Unity Graphics 不是通用 UI 或远程控制参考，只用于两点：`DebugManager` 对 debug data/panel 有 register/unregister/reset 生命周期，`ProbeVolume.Migration.cs` 以持久 version 和逐步 migration 升级序列化数据。本文不外推闭源 UI Toolkit 或服务能力。

## 3. 已有可保留基础

1. v2 source 已有明确 `UI_V2_ASSET_SCHEMA_VERSION = 2`，compiled header 已记录 source/compiler/package version、descriptor registry revision、component fingerprint 与 source/cache fingerprint。
2. Runtime UI 已形成 retained tree、layout、hit grid、focus/capture、dispatch、render extract 和 accessibility projection，不需要推倒算法底座。
3. `UiComponentEventEnvelope` 会校验 envelope event kind 与 payload kind 一致；部分 component/schema/action-policy diagnostic 已经采用 typed code。
4. `UiDispatchReply` 已接近集中 side-effect intent 的方向，能表达 focus/capture/navigation/host request 等效果；应收敛 receipt/lifecycle，而不是把效果重新散回 handler。
5. Runtime operation 已有 admission count/byte limit、deadline、worker panic、terminal TTL 与 harvest 概念，poll status 使用 allocation-free fixed layout。
6. `zircon_runtime_host::foreign_output` 已由 App/Editor 共享，具备分 kind encoded byte、item、nesting、decode-time检查、metrics 与 session fuse；这是 Interface 01/02 在途问题的实质进展。
7. 多数 UI 集合使用 `BTreeMap`，为 deterministic ordering 提供了起点；现有 focused tests 可转化为 migration/golden/property tests。

## 4. P0：必须先阻断的数据损失、资源耗尽与错目标动作

### P0-01 · 三套活跃 authoring model 与有损 v2/legacy 往返会静默删除用户数据

`UiTemplateDocument`、`UiAssetDocument`、`UiV2AssetDocument` 同时有生产 consumer。`v2_projection.rs` 把 v2 转 legacy 时丢弃 focus/navigation/picking/accessibility/widget，重建时固定 `repeat: None`、空 slots，并只遍历可达 root/component root；保存还重写 TOML trivia。必须先禁止不支持字段的保存或启用 lossless document owner，再建立单一 canonical IR 与显式迁移。Editor 23 P0-01拥有当前保存行为，本文拥有公共模型收敛，两个 gate 必须一起通过。

### P0-02 · authoring parse、编译与 accessibility capture 缺 producer budget

两个 v2 loader 和 v1 loader均先读取完整字符串，再无预算解析任意 TOML；递归/集合/字符串/unknown value 无 node/depth/byte 限制。Accessibility capture 忽略 `generation_hint`，构建全 surface 快照后一次性 JSON 编码和注册 allocation。必须在读取、解析、验证、编译、快照构建和编码每阶段设置一致预算、deadline/cancel 与 typed `LimitExceeded`，并支持 page/delta；consumer budget 不算修复。

### P0-03 · accessibility action 没有 snapshot generation，裸 node ID packing 可向错误当前节点派发

`UiAccessibilityTreeSnapshot` 没有 generation/epoch；action request没有 tree/snapshot token。Runtime 用高 16 bit surface、低 48 bit local ID打包，未验证 local ID 位宽且会截断，再按当前 surface vector反解。必须改为 `{session_epoch, tree_id, snapshot_generation, stable_node_key, instance_generation}`，动作采用 compare-to-snapshot admission；stale/recycled/overflow目标必须返回结构化 `StaleTarget`，不能继续以 `ZrStatus` 丢失动作结果。

## 5. P1：发布前必须闭合的公共合同

### 5.1 公共层职责与版本面

### P1-01 · interface crate 把稳定 DTO 与可变执行算法混为一个发布单元

`ui` 子树包含 layout engine、ECS projection、mutable tree、event adapter、text shaping helper 和 debug visualizer。应拆成 `ui_contract`、`ui_authoring`、`ui_runtime_model`、`ui_transport` 与实际 Runtime 实现，只有语言无关 contract进入稳定 ABI 发布矩阵。

### P1-02 · 642 个 UI public declarations 没有 family-level stability policy

公开不等于稳定，但当前 crate定位会让调用方自然依赖它们。每个 family必须标注 owner、profile、visibility、compatibility window、deprecation与test lane；内部 runtime helper降为 crate-private或迁入 Runtime。

### P1-03 · source、compiled artifact、runtime state、debug snapshot 都复用 serde public shape

同一 Rust derive 被当作持久格式、进程内快照和潜在 JSON wire。必须为每个 profile建立独立 envelope/codec/limits，禁止因内部字段重构无意改变磁盘或跨进程协议。

### P1-04 · UI 类型没有统一 schema ID、fingerprint 与 capability set

v1/v2只有局部整数 version，control/debug/accessibility/input DTO甚至没有 envelope。应接入 Interface 02 Schema Catalog，发布 reader/writer range、schema fingerprint、required capabilities与unknown-field policy。

### P1-05 · 80 个文件上的 `serde(default)` 会把缺字段和损坏字段降为合法零值

默认值对进程内构造方便，却会让 malformed/old/future payload静默变成空 ID、空图或 false。decode必须先进入版本化 raw representation，再由 validator/migrator显式填 default 并记录 migration diagnostic。

### P1-06 · UI 中没有任何 `deny_unknown_fields`，也没有 unknown-field preservation

直接 serde round-trip 会静默删除 future字段。authoring profile应保存 unknown syntax/field，closed transport profile应拒绝未知，telemetry/debug profile可跳过但必须报告；不能统一选择“静默忽略”。

### P1-07 · closed serde enum 依赖 Rust variant spelling

role、event、widget kind、visibility、phase等没有显式 wire tag/unknown variant。应生成稳定 numeric/string ID与 `Unknown(raw)`/capability negotiation，Rust rename不能成为格式迁移。

### P1-08 · `usize` 出现在 43 个 UI 文件中并进入可序列化类型

repeat count、text selection、indices和统计值随目标字宽变化。持久/transport使用有界 `u32/u64` 或 domain newtype，转换到 `usize` 只能发生在validated runtime admission之后。

### P1-09 · public fields允许绕过 constructor 和跨字段 invariant

`UiInvocationResult` 可同时有 value/error或两者皆无，多个 report/adapter/result也存在布尔和 optional字段矛盾。wire model使用tagged enum，domain model私有字段 + checked constructor，deserialize后必须validate。

### P1-10 · UI ID 的命名空间、grammar、owner 与 lifetime 不统一

`UiTreeId(String)`、`UiNodePath(String)`、action/property/control/component IDs 和裸 u64各自定义。应建立 source ID、compiled stable ID、runtime generational handle、transport qualified ID四层，禁止互换。

### P1-11 · default/zero ID 被普遍当作可构造合法值

derive `Default` 生成空字符串或 0，而运行代码未总是区分 invalid。跨边界 ID必须 nonzero/nonempty、带 owner/epoch，并在 decode时拒绝 placeholder。

### P1-12 · 没有 UI contract manifest 与发布兼容矩阵

需要生成当前/最低 reader-writer、schema digest、public symbols、capabilities、budgets和golden corpus清单，并在 Runtime/Editor/App build set中锁定；不能靠测试里手写字段偏移和 JSON样例维持。

### 5.2 Authoring、编译与迁移

### P1-13 · legacy `UiTemplateDocument` 没有退休策略

它仍被 Runtime template loader/instance/pipeline/validate消费。应冻结为 read-only legacy adapter，导出迁移工具和sunset telemetry，禁止新资产/新字段继续写入。

### P1-14 · v1 `UiAssetDocument` 与 v2 `UiV2AssetDocument` 没有权威关系

两者不是清晰的旧新版本，而是递归树与 flat graph、不同字段集合的平行模型。需要一个 canonical semantic IR，所有版本只作为 syntax adapter，compiler只接收validated IR。

### P1-15 · v2 loader只接受恰好 version 2，没有 migration graph

future拒绝是对的，但旧版本升级不能靠 Editor重建。应注册连续 migration、source/target fingerprint、loss report、idempotence与golden fixtures，migration失败保持原文件不变。

### P1-16 · Runtime 中存在两份 v2 parse/profile validation实现

`ui/v2/loader.rs` 与 `asset/assets/ui/document_loader.rs` 重复 version/profile规则，后续必然漂移。应由唯一 compiler/admission service拥有，Runtime asset system和Editor调用同一版本化入口。

### P1-17 · 作者语义大量藏在任意 `toml::Value`

props/state/layout/slots/params/style/action payload缺typed schema、单位、范围和资源依赖。必须通过 descriptor registry解析成typed value，未知扩展放进namespaced extension bag并保留原syntax。

### P1-18 · raw String ID/selector/path不支持安全重命名

component、node、slot、class、binding、resource和action引用靠字符串相等。需要 stable element ID + display name/path hint，rename产生redirect或transaction patch，并验证重复/悬空引用。

### P1-19 · authoring source没有 syntax span、comment/trivia 与原始 token owner

因此诊断不能精确定位，保存不能最小化diff，也无法保留未知字段。应采用 lossless TOML CST或等价语法树，semantic IR节点保留 source span/source file与syntax anchor。

### P1-20 · flat v2 graph没有统一 orphan、cycle、multi-parent policy

loader profile只验证局部形状，Editor投影只遍历可达root。compiler必须定义所有节点恰好一个owner、slot cardinality、cycle检测、orphan policy与deterministic traversal，并输出全量diagnostic。

### P1-21 · recursive legacy tree没有深度和节点上限

malicious或生成错误的输入可造成深递归、栈溢出和指数工作。parse后先做 iterative bounded admission，再允许任何recursive helper运行。

### P1-22 · authoring collection/string/value没有统一 byte/item budget

单个属性、节点、class、slot、component、diagnostic和resource列表均无上限。预算应来自schema profile并在read/parse/validate/compile/package各阶段累计，不得每层各自重置。

### P1-23 · `toml::to_string_pretty` 被当成保存器

它会重排/重写全文且无法形成字段级transaction。保存需要syntax patch、expected source revision、temp+journal+atomic publish、validated reread与receipt，复用Interface 02 Persistence Service。

### P1-24 · v2-to-legacy projection没有可机器读取的 loss report

当前调用方不知道哪些字段会丢。所有migration/projection必须返回 retained/dropped/defaulted/renamed 字段清单，P0字段损失时默认拒绝保存，而不是warning后继续。

### P1-25 · compiled artifact与source没有完整 dependency closure

header已有fingerprint基础，但还需每个resource/component/font/icon/style/plugin依赖的stable ID、version/digest、optional policy与build-set identity，缺依赖时fail-close。

### P1-26 · compiled package没有明确的 deterministic wire specification

`Vec<u8>` artifact与manifest仍由Rust内部类型驱动。应定义语言无关layout、endianness、alignment、section table、checksums、unknown section与reader range，并保存golden bytes。

### P1-27 · compiler report和artifact可同时存在不一致状态

公开 `report + bytes` 没有“只有无 error 才可发布”的类型约束。使用 `Rejected{diagnostics}` / `Compiled{artifact, warnings, receipt}` tagged outcome，publisher只接受后者。

### P1-28 · authoring与compiled cache没有事务性 source revision/CAS

Editor save、background compile和Runtime hot reload可基于不同source版本。request/result必须携带 document revision、source digest、compiler build、attempt和expected current artifact，过期结果不得发布。

### 5.3 Runtime tree、reflection 与 control

### P1-29 · `UiTree` 可通过 serde 构造违反运行时不变量的图

duplicate root、missing parent、cycle、slot不一致、重复ID等不能靠普通derive保护。反序列化只生成raw snapshot，随后由bounded validator构造不可变/受控mutable runtime tree。

### P1-30 · `UiTree::insert_root` 对重复 root 静默返回

同一错误在 `insert_child` 返回 Result，行为不一致会隐藏编译器或hot-reload bug。所有结构变更返回typed mutation outcome和before/after revision。

### P1-31 · pending mutation state被 serde skip，反序列化后观察语义改变

`UiTreeNodes` 的 pending set不是普通持久字段。应禁止序列化live mutable tree，或把snapshot与mutation journal明确分离，并在restore时重建/验证dirty state。

### P1-32 · `UiSurfaceFrame` 是包含 layout/render/hit/focus/debug/ECS 的 mega snapshot

它把多个生命周期、成本和consumer绑在一起。应拆成 generation-bound core frame +按需 debug/render/hit pages，使用共享 snapshot token和独立budget。

### P1-33 · `UiReflectionSnapshot` 与 `UiReflectorSnapshot` 重复描述同一树

两套 node/property/action模型字段和语义不同，容易让Editor和Runtime漂移。建立单一 reflection schema和view profile；轻量 remote query与重型debug view通过projection生成。

### P1-34 · reflection snapshot/diff没有 generation、sequence 或 base revision

`changed_nodes/removed_nodes` 无法证明相对哪一版，丢通知后consumer仍可能继续应用。加入 tree epoch、base/new revision、sequence、overflow/resync和digest。

### P1-35 · `UiReflectionNodePatch` 使用 path + arbitrary JSON，没有 CAS

路径可重命名，值无schema，多个writer会last-write-wins。patch应使用qualified node/property IDs、expected revision/old value digest、typed value、principal和transaction ID。

### P1-36 · property/action descriptor没有 stable type/schema identity

`UiValueType` 粗粒度且 reflected value直接 JSON；参数只含name/type/optional。应引用registry type/field/action schema，表达range/unit/enum/resource/capability/default与compatibility。

### P1-37 · `callable_from_remote: bool` 不能表达权限

一个布尔值无法区分principal、role、project、session、read/write/side-effect scope、rate limit与consent。remote admission必须由Editor 08 gateway检查capability token和policy，descriptor只声明required capability。

### P1-38 · `UiControlRequest` 没有 request/correlation ID、deadline 或 cancellation

调用方无法去重、关联回复、撤销昂贵query或判断迟到结果。建立 request envelope，并对read/write/action定义不同timeout、idempotency和retry规则。

### P1-39 · `SetProperty`/`CallAction` 没有 tree/document/instance generation

node path可能在另一个tree或新instance中重新出现。请求必须携带 target scope与expected revision，服务端返回 conflict/stale而不是对当前对象盲写。

### P1-40 · `UiControlResponse::Ack` 不包含 mutation receipt

调用方不知道何对象、何版本、何副作用被提交。write/action返回transaction ID、before/new revision、changed IDs、side-effect receipts与diagnostics；no-op也必须明确。

### P1-41 · query响应使用 `Option` 表达 not-found

它无法区分unknown、unauthorized、stale、filtered、budget exceeded和internal error。统一 typed outcome，不让安全策略与不存在对象产生相同观察结果。

### P1-42 · `UiInvocationResult` 可表达矛盾成功/失败状态

公开optional route/binding/value/error允许 both/neither。改为 `Succeeded{route,binding,value,receipt}` 与 `Failed{context,error,diagnostics}`，反序列化拒绝非法组合。

### P1-43 · subscription只有裸 u64，无lease、ack或overflow

`UiSubscriptionId` 不绑定session/epoch，diff没有backpressure。采用generation-qualified subscription handle、bounded queue、sequence/ack、lease/TTL、overflow marker与snapshot resync。

### 5.4 Accessibility、input 与 side effects

### P1-44 · `generation_hint` 是装饰字段

请求声明该字段，但 Runtime capture从不读取它。要么硬切删除并升API table，要么实现 conditional no-change/delta语义；绝不能保留一个让host误以为增量有效的字段。

### P1-45 · accessibility capture没有 App/Editor host consumer

production search只有Runtime export、tests和ABI inventory，App/Editor未调用它。应先定义 platform accessibility bridge、lifecycle、threading和budget owner，再把API标为可用；没有consumer不能算产品完成。

### P1-46 · accessibility未纳入 shared foreign-output policy

当前七个 `RuntimeForeignOutputKind` 不含 Accessibility。加入专用kind、encoded/decoded/node/string/depth预算与metrics，同时更重要地把限制前移到Runtime producer。

### P1-47 · accessibility snapshot没有 page/delta/truncation contract

它总是 roots +全nodes + diagnostics。应支持 snapshot token、page cursor、complete/truncated、usage、next cursor与变更序列；platform bridge可按需取子树或delta。

### P1-48 · accessibility node关系和状态不是 extensible capability model

closed role/action/state集合无法无损覆盖平台与未来widget能力。采用核心stable role/state/action IDs + namespaced extension，能力缺失返回unsupported而非默认按钮/文本。

### P1-49 · accessibility text selection使用 `usize`

跨架构不稳定，也未声明UTF-8 bytes、Unicode scalar、grapheme还是UTF-16 code units。选择与range必须声明坐标单位、验证边界，并与platform adapter显式转换。

### P1-50 · accessibility action result类型是死合同

它只在interface tests构造，Runtime dispatch不返回该结果。应将 action变成request/response operation或事件ack，返回 accepted/rejected/unsupported/stale、target generation和side-effect receipt。

### P1-51 · accessibility diagnostics嵌在每次全树中

重复字符串会放大payload，且无source span/correlation。diagnostic应有stable code、node/source location、snapshot generation、dedup key和separate bounded page。

### P1-52 · global node ID 的 16/48 bit packing是未发布的隐式协议

位宽、overflow和surface reorder没有文档或校验。删除packing，使用显式qualified struct；若ABI需要定长，分配registry handle并保留generation/owner验证。

### P1-53 · input dispatch result复制原事件和多个无界副产物

`UiInputDispatchResult` 同时携带 event、reply、diagnostics、effects、rejected effects、host requests、events、binding reports和damage，易产生clone与日志放大。内部热路径改为bounded arena/borrowed record，跨界只输出摘要或按需trace page。

### P1-54 · pointer/navigation dispatch拥有平行 result/diagnostic模型

重复字段会形成不同的handled、target、reason和effect语义。建立统一 dispatch outcome，再以typed phase detail表达pointer/key/navigation/text差异。

### P1-55 · focus/capture/pointer ID没有 owner epoch 与设备/user identity

多window、多用户、设备重连和surface重建时裸ID可能复用。所有输入身份绑定 seat/device/session epoch，capture/focus变更返回由中央router确认的receipt。

### P1-56 · host side effects没有统一执行回执和安全策略

clipboard、open link、focus/capture/navigation/IME 等不能只在reply中声明。Runtime Host应按capability、安全来源和user gesture执行，回传accepted/denied/failed与平台错误，handler不得假定已生效。

### 5.5 Diagnostic、operation 与 host output

### P1-57 · 45 套 Diagnostic/Status/Report/Result 家族没有公共 envelope

应复用 Interface 02的diagnostic核心：stable code、severity、message key/args、source span、related location、correlation/attempt、fix-it、privacy、budget/truncation和cause chain。

### P1-58 · diagnostic code有 typed enum、raw String 与完全缺失三种形态

binding/accessibility/component部分typed，invalidation/localization/resource/schema等又使用字符串。建立namespaced code registry，插件可扩展但必须声明owner/version；显示文案不能充当code。

### P1-59 · severity enum重复且取值不一致

多个模块各自定义 Info/Warning/Error 等集合，debug event甚至 `Option<String>`。统一核心severity和domain facets，未知值保留raw，不以字符串比较控制行为。

### P1-60 · diagnostic没有统一 source span 与 build/source identity

UI作者错误无法定位文件/行列/节点/property，运行时错误又不能绑定compiled artifact。每条diagnostic携带 source URI/span、semantic address、source/artifact digest与producer build。

### P1-61 · report列表普遍无预算、dedup和truncation marker

恶意文档可产生每节点/每字段错误并放大内存。collector必须限制总数/bytes/per-code，保留first/representative和suppressed count，结果显式 `truncated`。

### P1-62 · diagnostic缺少 correlation、attempt 与 causal chain

parse/compile/load/instantiate/dispatch/host side-effect之间无法串联。统一 trace/correlation/attempt/parent IDs，并让Editor journal按这些字段聚合而非解析message。

### P1-63 · debug event用 raw name/detail/severity 字符串

它不适合稳定分析、过滤、隐私或本地化。定义typed event schema、category、fields、clock domain、thread/task、privacy class和payload budget。

### P1-64 · debug snapshot没有 envelope、page、filter 或 redaction

`UiSurfaceDebugSnapshot` 可携带完整tree/render/hit/overdraw/events/overlay，可能暴露文本和资源路径。加入schema/generation、capture request filter、principal policy、redaction、page和immutable receipt。

### P1-65 · debug timestamp没有 clock domain

毫秒整数不能判断wall/monotonic/frame/virtual time，也不能跨进程关联。使用typed timestamp `{clock_id, epoch, ticks, frequency}` 或统一trace timebase。

### P1-66 · operation submit使用任意 `operation_id: String` + JSON payload

没有registry schema、required capability、producer budget或compatibility。operation ID应解析到typed descriptor，payload按schema/budget admission并记录source/principal/idempotency key。

### P1-67 · operation handle只有 nonzero u64

它不绑定session、service epoch或generation；Runtime restart/reuse会产生歧义。采用Interface 01 qualified handle规则，并让poll/harvest验证owner与epoch。

### P1-68 · operation phase声明 Cancelled/Expired，但API table没有 cancel函数

内部可能因deadline取消，caller却无法主动请求取消或查询cancellation acceptance。增加cancel request/ack，定义queued/preparing/ready/apply各阶段的cooperative/too-late语义。

### P1-69 · harvest outcome无法表达 Cancelled/Expired

`ZrRuntimeOperationOutcomeV1` 只有 Succeeded/Failed，poll的终态会在harvest丢语义。结果必须覆盖所有终态并携带typed reason、partial artifact policy、retryability和cleanup receipt。

### P1-70 · operation progress没有单位、时间和真实性声明

`completed_work/total_work/detail_value` 无 unit、stage、estimated/measured、started/updated/deadline。发布typed progress snapshot，允许indeterminate，保证monotonic规则并报告stall/heartbeat。

### P1-71 · operation result的 arbitrary JSON output没有schema与producer receipt

host只能在消费端限制结构，不能判断字段或来源。结果携带 output schema ID/version、operation descriptor revision、producer build、source digest、attempt与encoded digest。

### P1-72 · host output metrics被压成 ad-hoc `diagnostic_line()`

共享 consumer state已经有有价值计数，但字符串阻止机器聚合、版本化和告警。输出typed metrics snapshot/event并绑定session/build；保留文本只作为presentation adapter。decode-time check也应明确它不能抢占已阻塞decode。

## 6. P2：主链稳定后收敛

### P2-01 · legacy/v1/v2/module命名没有生命周期词汇

改用明确 `AuthoringSourceVn`、`CompiledPackageVn`、`RuntimeSnapshotVn`，并在module/doc中声明read/write状态和sunset。

### P2-02 · `UiTreeId(pub String)` 和 `UiNodePath(pub String)` 显示实现细节

收敛为opaque validated ID/path，Display只用于presentation，协议不依赖其格式化文本。

### P2-03 · `insert_root` 与其他mutation错误策略不一致

在P1-30重构时统一typed error、no-op与duplicate语义。

### P2-04 · accessibility snapshot node lookup仍有线性扫描路径

在bounded snapshot/index设计后提供ID index；不要以此替代generation修复。

### P2-05 · surface index被格式化进 tree/path字符串

这会把容器顺序变成用户可见identity。使用stable surface ID，index只作为局部排序信息。

### P2-06 · report中的 bool bundle命名模糊

`changed/dirty/refresh_projection` 等改成typed invalidation mask与明确receipt，避免调用方猜组合。

### P2-07 · `UiValueType::Any` 过度宽松

在schema registry完成后限制到显式extension/opaque类型，不让Any成为跳过validation的默认入口。

### P2-08 · debug schema version放在局部 context 而非顶层 envelope

迁移到统一snapshot envelope，局部context只表达capture配置。

### P2-09 · closed enum 的 Default常选择第一个业务状态

对wire decode使用Unknown/Unspecified，不让缺字段自动变成Declared、Visible或Accepted。

### P2-10 · UI module中文件和职责颗粒不均

拆分时按contract owner而非仅按行数整理，避免再次形成几百行的“types”聚合文件。

### P2-11 · public注释对“stable”“snapshot”“generation”承诺过强

只有通过兼容矩阵、identity和consistency gates后才使用这些词；当前应注明in-process/experimental边界。

### P2-12 · 测试名称大量验证shape而非产品不变量

保留ABI/layout smoke test，但增加migration、stale target、budget、fault、unknown field与cross-version corpus，避免“可serde”被误认为工程完成。

## 7. 目标架构

### 7.1 五类合同与三个执行 owner

```text
Lossless Authoring Source
  syntax tree + semantic IR + stable element IDs + source revision
             |
             v
UI Compiler / Migration Service
  bounded admission + schema registry + diagnostics + dependency closure
             |
             v
Immutable Compiled Package
  versioned sections + build/source fingerprints + dependency manifest
             |
             v
Runtime UI Owner
  generational tree/surface/input/accessibility state
             |
             +--> bounded Snapshot/Page/Delta Transport
             +--> typed Diagnostic/Trace Transport
             +--> Host Side-effect Request/Receipt

Owners:
  Interface Schema Authority -> IDs、wire/profile、compatibility、budgets
  Runtime UI Service        -> tree/layout/input/a11y execution与producer limits
  Editor UI Authoring       -> document transaction、lossless save、compile UX
```

### 7.2 Identity 分层

Authoring element使用持久stable ID并绑定document；compiled element使用source ID + artifact generation；runtime node使用surface owner + slot index + generation；transport target再加session epoch、snapshot generation与capability scope。路径、显示名和surface vector index都只能作为hint，不能作为权限或mutation identity。

### 7.3 Snapshot 与 action protocol

capture request声明tree/surface filter、base generation、max nodes/bytes/depth、deadline和capabilities。响应返回snapshot ID、tree epoch、generation、page、usage、truncated/complete与next cursor。action必须引用该snapshot和qualified node；Runtime在当前generation验证后执行，返回action outcome、new generation和host side-effect receipts。gap/stale永远fail-close并要求resync。

### 7.4 Diagnostic 与 operation protocol

所有parse/compile/load/dispatch/host/operation输出共享结构化 diagnostic envelope。operation descriptor注册input/output schema、capability、budget与cancellation semantics；submit/poll/cancel/harvest围绕一个generation-qualified handle和attempt receipt，所有terminal phase都有一一对应result。

## 8. 现有实现处置

| 当前实现 | 处置 |
|---|---|
| `UiTemplateDocument` | 冻结read-only，迁移后sunset；禁止新作者功能 |
| `UiAssetDocument` | 作为legacy syntax adapter；不再作为Editor内存authority |
| `UiV2AssetDocument` | 保留字段语义，迁入lossless source + semantic IR；不直接作为runtime state |
| 两份Runtime v2 loader | 合并到唯一bounded compiler/admission service |
| `UiTree`/layout/dispatch/ECS | 迁入Runtime implementation；公开snapshot与mutation command另建contract |
| reflection/control DTO | 加schema、qualified ID、revision/CAS、principal与receipt |
| accessibility full JSON | 替换为generation-bound bounded page/delta；纳入host policy |
| `UiAccessibilityActionResult` | 接入真实action response，不再保留dead type |
| UI diagnostic families | 迁移到共享envelope，domain code保留namespaced extension |
| operation v1/v2 | 发布完整cancel/terminal/output schema的新版本，旧版只作compat adapter |
| host `diagnostic_line()` | 退为presentation，authority改为typed metrics snapshot |

## 9. 分阶段重构

### M0 · 冻结三项 P0

1. `.zui` 含 v2-only/unknown字段时禁止legacy保存，导出loss report和备份；建立round-trip corpus。
2. 为read/parse/validate/compile/capture/encode加入共享 producer budget、deadline和cancel。
3. 禁止无snapshot generation的accessibility action；检测48-bit overflow并fail-close。

### M1 · Contract inventory 与 crate boundary

1. 生成642个公开类型的owner/profile/consumer/serde/ABI清单。
2. 把implementation-only tree/layout/dispatch/debug helper迁出stable contract面。
3. 定义 Authoring/Compiled/Runtime/Transport/Diagnostic profiles与Schema Catalog entries。

### M2 · Lossless authoring 与 migration

1. 引入lossless TOML syntax owner、canonical semantic IR和stable element IDs。
2. 把legacy/v1/v2实现为bounded adapter与连续migration graph。
3. 保存使用revision/CAS、journal、atomic publish、validated reread和receipt。

### M3 · Deterministic compiler/package

1. 合并loader/validator/compiler authority，解析typed descriptor和依赖闭包。
2. 定义language-neutral package sections、limits、digest与reader matrix。
3. artifact publish绑定source revision、compiler build和expected cache generation。

### M4 · Runtime model 与 generational identity

1. 由validated package构造runtime tree，禁止serde直接恢复live state。
2. 所有surface/node/focus/capture/pointer identity绑定owner/epoch/generation。
3. mutation输出before/new revision、invalidations和receipt。

### M5 · Reflection/control gateway

1. 合并reflection snapshot模型，建立page/delta/sequence/resync。
2. query/mutation使用typed schema、qualified IDs、CAS和transaction。
3. remote path接入Editor 08 principal/capability/rate/deadline gateway。

### M6 · Accessibility product bridge

1. 建立App/Editor platform bridge owner和host consumer kind。
2. 实现bounded snapshot/page/delta和stale-safe action response。
3. 建立role/state/action/text range的platform compatibility matrix。

### M7 · Input/side-effect receipts

1. 统一pointer/key/navigation/text dispatch outcome与trace预算。
2. Host集中执行clipboard/link/focus/capture/IME等side effect并回执。
3. 覆盖multi-window/multi-seat/device reconnect/surface rebuild。

### M8 · Diagnostic convergence

1. 迁移45套family到共享envelope和namespaced code registry。
2. 加入span/build/correlation/privacy/fix-it/budget/truncation。
3. Editor journal/console只消费typed事件，文本由presentation adapter生成。

### M9 · Operation protocol hard cutover

1. 注册typed operation descriptor和input/output schema。
2. 发布submit/poll/cancel/harvest完整状态机与generation-qualified handle。
3. 所有terminal phase返回可审计result、cleanup与retry receipt。

### M10 · Host output producer/consumer 对称化

1. Accessibility纳入shared host policy，所有family补producer admission。
2. metrics改为typed snapshot，fuse scope、reset和recovery策略显式化。
3. 验证decode deadline不是伪抢占，并对大JSON改为stream/page codec。

### M11 · 兼容、故障、安全与规模资格

1. 运行old/new reader-writer、Windows/Linux、debug/release与build-set矩阵。
2. fuzz所有source/transport parser和migration；child process验证OOM/stack/deadline。
3. 运行真实Runtime DLL + App/Editor accessibility、remote control、operation与host fault lanes。

## 10. 产品资格门

1. 含每个v2-only字段、unknown field、comment和orphan fixture的打开-无修改-保存必须byte或semantic-lossless；任何loss产生blocking outcome。
2. legacy/v1/v2每条migration都有golden before/after、loss report、idempotence与downgrade policy。
3. 100 MiB source、百万节点、10,000层嵌套、超长字符串和巨大map在固定RSS/CPU/stack预算内被拒绝。
4. parse/compile/capture每阶段deadline/cancel在规定时间内生效，并报告实际usage。
5. compiled package相同输入/build set产生相同bytes和digest，依赖变化必使artifact失效。
6. unknown package section按capability规则保留或拒绝，不发生静默字段删除。
7. live runtime tree不能通过普通serde绕过graph/invariant validation。
8. duplicate/orphan/cycle/multi-parent/slot cardinality错误均有stable diagnostic code和source span。
9. surface reorder、node delete/recreate和Runtime restart后，旧accessibility action全部返回stale，绝不命中新节点。
10. local node ID超过48 bit时当前API立即拒绝；新协议不再依赖bit packing。
11. accessibility snapshot按page/delta消费，producer在达到node/byte/depth预算前停止并标记truncated。
12. App与Editor真实platform bridge可连续接收create/update/remove/focus/action，线程与shutdown无悬挂callback。
13. UTF-8/UTF-16/grapheme text range在Windows和Linux adapter有golden conversion并拒绝非法边界。
14. reflection diff故意丢包、乱序、重复后consumer检测gap并通过snapshot resync。
15. concurrent SetProperty基于同一revision时恰好一个提交，另一个返回conflict而非覆盖。
16. remote query/write/action分别验证principal、capability、project/session scope、rate与deadline。
17. `Ack` 被mutation receipt替换，receipt可证明before/new revision、changed IDs和side-effect outcome。
18. subscription queue溢出输出明确overflow/resync marker，内存保持bounded。
19. focus/capture在multi-window、multi-seat、device reconnect和surface rebuild中不串目标。
20. clipboard/open-link等敏感side effect没有user gesture或capability时fail-close并回执。
21. input hot path在10K nodes/high-rate pointer move下不复制完整debug payload，trace budget可关闭/采样。
22. 所有45个现有diagnostic family有迁移映射，重复code/owner在build时失败。
23. 每条authoring diagnostic具有文件/行列/semantic address；runtime diagnostic具有artifact/build/generation。
24. 百万重复错误被dedup/truncate并报告suppressed count，日志和UI不OOM。
25. privacy-marked文本、路径和clipboard内容不会进入未授权debug export。
26. operation caller可cancel，too-late/accepted/completed race有确定结果。
27. Completed/Failed/Cancelled/Expired/Harvested的poll与harvest一一对应，不丢终态原因。
28. operation output按schema和producer budget生成，host能验证descriptor/build/source/attempt receipt。
29. foreign output所有kind都有producer与consumer对称限制；恶意payload不能先分配后才被拒绝。
30. typed host metrics可按session/build/kind聚合，文本变化不影响监控。
31. Runtime DLL断开、host crash、allocation release失败、decode失败和session fuse均有恢复/终止策略与fault tests。
32. 全部资格结果绑定source fingerprint、build set、command、duration、exit code和artifact，编译未到test binary不得标记通过。

## 11. 验证说明

本文是 review/refactor plan，不是实现完成。没有修改 production Rust、manifest、测试或参考源码，也没有运行新的动态测试。此前 Editor test编译阻断仍有效；当前 interface/API在途修改又要求实施前重取指纹。静态报告应执行frontmatter、编号、计数、链接、UTF-8/LF与索引一致性检查，但这些只验证文档，不替代产品资格门。

## 12. 审查决策

当前 `zircon_runtime_interface` 的 UI 公共面不能被视为工程级稳定authoring/runtime/remote/accessibility协议。保留已有tree/layout/dispatch/compiler fingerprint/host budget底座，但停止扩展三套作者格式、无世代action、任意JSON remote mutation和碎片化diagnostic。

后续 interface 队列应转向剩余非UI public DTO与最终全crate ownership consolidation；UI实施必须先完成 M0，并由 Interface、Runtime UI、Editor UI三方共同签署schema/identity/transaction gates。本文所有3项P0、72项P1、12项P2均为 `pending`。

Open Failure（source repaired / managed return pending）：generic UI host bridge 的 `ActivateLink`
mixed-era `href: String` 投影已由本计划硬切为 `link_target: UiRichLinkTarget`，wire key 仍为 `href`；
lower/upward managed gates 未完成，详见
[runtime-interface-ui-activate-link-field-mismatch](03/failure-2026-08-31-runtime-interface-ui-activate-link-field-mismatch.md)。
