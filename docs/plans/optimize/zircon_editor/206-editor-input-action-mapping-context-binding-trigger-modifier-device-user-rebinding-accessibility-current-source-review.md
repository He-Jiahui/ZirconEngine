---
title: Editor Input Action、Mapping Context、Binding、Trigger、Modifier、Device、User、Rebinding 与 Accessibility 当前源码复核
category: zircon_editor
report_id: Editor206
review_date: 2026-08-28
baseline_head: 11cac2d08a891ee92dcc206fd84a2d15f9e1a3f4
verification_head: 11cac2d08a891ee92dcc206fd84a2d15f9e1a3f4
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
canonical_owner: Editor29
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/29-input-action-mapping-context-binding-trigger-modifier-device-user-rebinding-accessibility-authoring-review.md
  - docs/plans/optimize/zircon_editor/87-editor-input-action-mapping-context-binding-trigger-modifier-device-user-rebinding-accessibility-authoring-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/103-editor-input-action-mapping-context-rebinding-accessibility-current-source-review.md
  - docs/plans/optimize/zircon_editor/150-editor-input-action-mapping-context-binding-trigger-modifier-device-user-rebinding-accessibility-current-source-review.md
related_code:
  - zircon_runtime/src/core/framework/input
  - zircon_runtime/src/input
  - zircon_runtime/src/dynamic_api/session/events.rs
  - zircon_runtime/src/dynamic_api/session/events
  - zircon_runtime/src/dynamic_api/session/input_events.rs
  - zircon_runtime/src/dynamic_api/session/construction.rs
  - zircon_runtime/src/script/vm/gameplay_host.rs
  - zircon_runtime/src/script/vm/gameplay_host/input.rs
  - zircon_runtime/src/asset/project/manifest/project_manifest.rs
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_app/src/entry/builtin_modules.rs
  - zircon_editor/src/core/asset/type_registry
  - zircon_editor/src/core/asset/dirty
  - zircon_editor/src/core/commands/key_chord.rs
  - zircon_editor/src/core/commands/keymap.rs
  - zircon_editor/src/core/settings/keymap_overrides.rs
  - zircon_plugins/first_party_editor_catalog/src/catalog.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/99r-runtime-input-device-event-frame-state-action-map-focus-gamepad-recording-replay-host-product-integration-current-source-review.md
  - docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
  - docs/plans/zircon_runtime/runtime/12/failure-2026-07-19-app-entry-input-and-gamepad-storm-budget.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/12-settings-preferences-scope-persistence-locale-i18n-appearance-plugin-extensibility-review.md
  - docs/plans/optimize/zircon_editor/145-editor-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-current-source-review.md
  - docs/plans/optimize/zircon_editor/148-editor-multiplayer-lobby-matchmaking-online-services-replication-network-emulation-pie-authoring-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/EnhancedInput/Public/InputAction.h
  - dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/EnhancedInput/Public/InputMappingContext.h
  - dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/EnhancedInput/Public/EnhancedInputSubsystemInterface.h
  - dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/EnhancedInput/Public/PlayerMappableInputConfig.h
  - dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/EnhancedInput/Public/PlayerMappableKeySettings.h
  - dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/InputEditor/Private/ActionMappingDetails.cpp
  - dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/InputEditor/Private/AssetDefinition_InputAction.cpp
  - dev/UnrealEngine/Engine/Plugins/EnhancedInput/Source/EnhancedInputTestSuite/Private/InputPlayerMappableKeysTests.cpp
  - dev/godot/core/input/input_map.h
  - dev/godot/core/input/input_map.cpp
  - dev/godot/editor/settings/action_map_editor.cpp
  - dev/godot/editor/settings/input_event_configuration_dialog.cpp
  - dev/bevy/crates/bevy_input/src/lib.rs
  - dev/bevy/crates/bevy_input/src/button_input.rs
  - dev/bevy/crates/bevy_input/src/keyboard.rs
  - dev/bevy/crates/bevy_input/src/gamepad.rs
  - dev/Fyrox/fyrox-impl/src/engine/input.rs
  - dev/Fyrox/fyrox-ui/src/key.rs
  - dev/Fyrox/editor/src/settings/keys.rs
  - dev/Graphics/Templates/com.unity.template-hd/Assets/InputSystem_Actions.inputactions
  - dev/Graphics/Templates/com.unity.template-hd/Assets/SampleSceneAssets/Controllers/Common/InputSystem/StarterAssets.inputactions
  - dev/Graphics/Templates/com.unity.template-hd/Assets/SampleSceneAssets/Controllers/Common/InputSystem/StarterAssets.inputsettings.asset
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/Debugging/DebugManager.Input.cs
finding_status:
  p0_open: 5
  p1_open: 46
  p1_partial: 14
  p1_closed: 0
  p2_open: 11
  p2_partial: 1
  p2_closed: 0
gate_status:
  fail: 32
  partial: 0
  pass: 0
---

# Editor206 · Input Action / Mapping / Rebinding / Accessibility 当前源码复核

## 1. 结论

Zircon当前仍没有可交付的Input Action/Mapping Context创作产品。`ResourceKind`、builtin asset registry、first-party Editor catalog、project manifest与App首帧安装链均未声明Input Action或Mapping Context。对`InputActionDocument`、`InputMappingContextDocument`、`CompiledInputMapArtifact`、`InputMapInstallRequest/Receipt`、`InputUser`、`PlayerBindingProfile`、`RebindCaptureRequest/Receipt`、`StableInputControlId`和`InputActionFrameSnapshot`的精确搜索，在索引内production源码及当前2,293个未跟踪production文件中均为零。

Runtime Input的局部性能底座继续进步：context改为增量有序插入，binding排序减少scratch分配，evaluator复用action state、axis/consumed索引与workspace，focus/disconnect清理减少临时分配，recorder以批量front retirement降低逐项pop成本，Dynamic Session又把keyboard/IME与gamepad事件拆成子模块。这些改进没有补出source asset、semantic compiler、immutable artifact、frame-barrier install、per-user context stack、typed trigger/modifier、profile/rebind或accessibility authority。

当前公共schema仍是调试级字符串结构。Action只有`id/context/display_name`；Context只有`id/priority/enabled`；Binding保存action string、button chord和携runtime `GamepadId`的scalar axis；ActionState仍为string集合与`f32`。Context虽然按priority存储，evaluation仍没有完整的priority/block/reserve arbitration；duplicate、missing context和unknown action仍没有fail-close semantic diagnostics。

本轮还纠正Editor150的过宽表述：Dynamic Session并非所有事件都“physical-first”。键盘非文本、指针、鼠标与gamepad通常先写physical manager再投递Runtime UI；text keyboard与IME则先给UI消费，被消费后不再写physical manager。两套顺序并存且没有统一的sequence-qualified ownership合同，因此P1-56/57只能保持Partial，不能把当前顺序视为已完成的输入所有权模型。

因此Editor29/87/103/150状态不变：5项P0 Open；P1为46 Open/14 Partial/0 Closed；P2为11 Open/1 Partial/0 Closed；32门全部Fail。没有同语义、同硬件、同规模的Unreal对照receipt，不能声称性能或表现优于Unreal。

## 2. 冻结范围与方法

本报告读取当前共享工作树，以`11cac2d08a891ee92dcc206fd84a2d15f9e1a3f4`标记提交基线；其他会话的在途改动不回退、不覆盖、不暂存。物理行按文件读取；tests统计Rust `#[test]`，ignored统计`#[ignore...]`；fingerprint由排序相对路径与逐文件SHA-256聚合。Dynamic集成集同时包含父模块`events.rs`与拆分目录`events/`，避免因当前模块拆分漏算。

| 选择集 | 文件 / 行 / 非空行 / bytes / tests / ignored | fingerprint |
|---|---:|---:|
| Runtime Input两棵源码树 | **66 / 8,494 / 7,559 / 286,663 / 90 / 12** | `7497bca16a856eff0739291151bf9ffd61d52a10bfbeb95e3b5e93a4021179d9` |
| Dynamic/App/Script/Manifest集成边界 | **10 / 2,738 / 2,555 / 111,576 / 14 / 0** | `f120f94cb81525a180589ec74815aaa51c13ad90ac09222b57a441342eb4cd37` |
| Editor authoring primitive | **31 / 6,937 / 6,247 / 230,755 / 56 / 8** | `754eca327fbaf272c28d56153ae0704dec11e6fa7994af397c4de2a85e2d39bd` |
| Zircon selected union | **107 / 18,169 / 16,361 / 628,994 / 160 / 20** | `8d08c078d624445e6b5d56a01af9e8238310cef1765f7924ba9cca4d153ef757` |
| 参考选择集 | **23 / 12,911 / 11,558 / 502,917 / 50 conventional markers / 0** | `bd49e4deb067d9aa864e7558305d71fdff0eac18701f503ae173ad1af3fbc53f` |

参考marker由45个Bevy Rust `#[test]`与5个Unreal automation declaration组成，仅表示静态存在。Godot revision为`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`，Bevy为`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`，Fyrox为`8d815db36494f1badb347547dfc7094bf4fbbdf8`，Unity Graphics为`a7e4c051d256a781ab362c64316b125a1e104694`；Unreal以vendored文件fingerprint冻结。

本轮没有运行Cargo、Editor、真实设备、cook/install、PIE多玩家、rebind、accessibility、fault、scale、soak或跨引擎benchmark。Runtime12仍为`in_progress`，其input/gamepad storm failure仍为Open；Tooling按用户要求排除。

## 3. 当前产品事实

### 3.1 产品装配断路未变化

- `ResourceKind`、builtin registry和first-party catalog没有Input Action/Context contribution、factory、toolkit、thumbnail、reference analyzer或open route。
- `ProjectManifest`没有input source、compiled artifact或binding profile引用；App module manifest可以选择Input module，但`module_descriptor()`仍调用`module_descriptor_with_config(InputConfig::default())`。
- `InputConfig::default()`仍为`enabled=false`及空`InputActionMap`；configured descriptor、`set_action_map`和`evaluate_actions*`除实现与tests外没有production caller。
- `InputDriver`仍无真实platform ingress所有权合同；Input manager、Action manager与driver的装配没有首帧artifact identity、health、generation或teardown receipt。
- gameplay script仍可通过`gameplay.key_pressed("A/D/W/S")`读取raw global manager，绕过context、consume、profile、device assignment与typed action phase。

### 3.2 evaluator优化没有形成compiler/artifact

`InputActionMap::add_context`的增量有序插入、binding的scratch-free排序、generation对binding ranges的预编译，以及workspace、frame-axis index、consumed-input index和action-state原位归一化，都是应保留的性能底座。但generation仍直接接收任意serde map，没有schema/source/dependency digest、diagnostic span、capability manifest、compiler version或install receipt。

Priority当前只影响context容器顺序，generation只保留slot与enabled；没有完整的priority仲裁、block lower-priority、reserve all mappings、trigger phase或modifier pipeline。duplicate ID继续静默忽略，binding引用missing context时会创建并启用，unknown action没有source-qualified diagnostic，空active-context仍可隐式表示全部启用。这是运行时便利行为，不是可审计的产品编译语义。

### 3.3 事件顺序是分裂策略

- 键盘非文本键先`submit_input_event`，再dispatch Runtime UI；gamepad button/axis也是physical manager先行。
- cursor/mouse motion/wheel等路径通常先更新physical/camera状态，再发UI事件；UI consume不能撤销已经发生的physical truth。
- text keyboard先dispatch UI，UI consumed后直接返回；IME同样先dispatch，只有未消费事件才在末尾submit。
- 当前没有统一的event sequence、surface/window/user/device qualification、capture lease、ownership decision或“physical truth与semantic consume如何并存”的规范。

该分裂可以是未来政策的一部分，但必须由显式router合同表达。目前依赖event kind手写顺序，新增事件很容易落到错误分支，且无法证明Editor command、Runtime UI、gameplay Action和raw observer之间的确定性关系。

### 3.4 Recorder与Replay仍不是产品资产

底层`InputEventRecorder`具capacity、discarded status与批量front retirement，capture frame也能标记recording disabled或数据不完整。这解决了部分无界原始记录问题，但frame transient queue仍只有相邻motion/cursor coalescing，没有capacity、age或drop policy合同。

`InputRecording`自身仍持有无上限`Vec<InputRecordingFrame>`，`push_frame`直接追加。每帧只存frame index、records、enabled与discard count，没有schema/build/artifact/profile/user/device/clock/checksum；`from_events`把timestamp固定为0，replay只clone event重新submit，丢弃原timestamp/sequence和Action结果，也没有过滤可能产生host effect的事件。因此它不能作为deterministic replay、跨版本迁移或性能对照的完成证据。

### 3.5 Editor primitive不能代替Gameplay authoring

Editor asset type registry、dirty/save batch、creation template、toolkit primitive、key chord、keymap override和conflict基础均可复用；当前selected集还增加了针对这些基础设施的性能测试。但没有任何Input document adapter、domain transaction、compiler projection或product contribution。Editor command shortcut属于host operation authority，Gameplay mapping属于shipping Runtime authority；二者只能共享physical token、capture和conflict primitive，不能共享source identity、存储或active context。

### 3.6 Camera controller仍是下游算法DTO

Free/Orbit/Pan controller消费调用者组装的movement/look/pan/zoom/focus DTO，并进行确定性变换。它没有从Action artifact、InputUser、context stack或PlayerBindingProfile读取，也不返回Action evaluation/install receipt；所以它是可复用的输入消费算法，不是Input authoring产品consumer。

## 4. Owner边界与目标合同

| 领域 | 唯一owner | Editor206职责 |
|---|---|---|
| raw device/window/focus/event/frame reducer | Runtime Input / Platform | 只消费qualified physical snapshot、device registry和ownership decision |
| Action/Context source、compiler、artifact、installer | Runtime Input产品owner | transactional authoring、compile orchestration、diagnostic与projection |
| InputUser/LocalPlayer/device assignment | Runtime Gameplay/Platform | 选择user/profile/context，不复制player或device registry |
| Editor command shortcuts | Editor08 | 只复用token/capture/conflict primitive，保持authority隔离 |
| asset/document/job/settings/save | Editor02/04/09/12与Save owner | source revision、transaction、profile persistence与migration |
| Runtime UI consume/capture | Runtime UI owner | 提供sequence-qualified ownership，不删除physical truth |
| Script/gameplay facade | Runtime Script/Gameplay | per-user typed Action query，raw key降为低层受限capability |
| PIE/network | Editor148 + Runtime Net | 提供server/client/local-player/device topology与artifact identity |

```text
InputActionDocument + InputMappingContextDocument
  -> InputSemanticCompiler
  -> CompiledInputMapArtifact
  -> InputMapInstaller at frame barrier
  -> per-InputUser ActiveInputMapGeneration
  -> typed InputActionFrameSnapshot

PlayerBindingProfile
  -> RebindCaptureRequest + ConflictQuery
  -> atomic profile transaction
  -> InputMapInstallReceipt
  -> Action snapshot / Editor debugger / gameplay consumers
```

最低合同必须含stable Action/Context/Binding/Trigger/Modifier identity、Bool/Axis1D/2D/3D value、typed physical selector、schema/source/compiler/artifact/profile revisions、diagnostic span、capability requirement、InputUser/device lease、context priority/consume policy、frame tick/generation、single terminal install/rebind receipt和bounded observation。

## 5. P0 currentness重判

| ID | 状态 | 当前证据 | 硬切目标 |
|---|---|---|---|
| P0-1 | **Open** | 无Input资产、factory、toolkit、document或Editor surface。 | 建两类source asset、共享compiler与真实toolkit；无provider明确Unavailable。 |
| P0-2 | **Open** | shipping Action Manager默认disabled+空map，production install/evaluate caller为零。 | project引用artifact，首帧前frame-barrier install并返回typed receipt。 |
| P0-3 | **Open** | serialized button/axis仍保存runtime `GamepadId`。 | source保存stable selector，install按InputUser/device assignment解析。 |
| P0-4 | **Open** | duplicate/missing/unknown source仍被静默接受，priority未完整执行。 | semantic compiler fail-close并输出source-qualified diagnostics。 |
| P0-5 | **Open** | raw `key_pressed`仍是玩法路径，无InputUser/profile/rebind。 | per-user Action facade、原子profile/rebind并限制raw capability。 |

## 6. P1 currentness状态

| IDs | 状态 | 当前判定 |
|---|---|---|
| P1-1..P1-16 | **Open** | Action/Context资产、stable IDs、typed value/metadata、selector、trigger/modifier、schema/compiler/diagnostic/conflict/artifact均缺失。 |
| P1-17 | **Partial** | internal evaluator generation存在；无artifact identity、frame barrier、expected generation或receipt。 |
| P1-18..P1-21 | **Open** | source provenance、phase、vector aggregation和per-user context stack缺失。 |
| P1-22 | **Partial** | consumed button/axis index可复用；priority/block/reserve与产品producer缺失。 |
| P1-23 | **Open** | context owner lease与teardown回收缺失。 |
| P1-24 | **Partial** | map-change generation可重建lookup；preserve/flush/held policy与receipt缺失。 |
| P1-25 | **Open** | held-input rebind策略缺失。 |
| P1-26 | **Partial** | raw focus/disconnect会清理物理状态；没有per-user Action cancellation与phase证据。 |
| P1-27 | **Open** | multi-window/viewport/user qualified route缺失。 |
| P1-28 | **Partial** | raw submit、bounded recorder、discard status与replay helper可复用；缺artifact/profile/user-qualified deterministic injection。 |
| P1-29..P1-33 | **Open** | typed gameplay facade、bounded Action observation、InputUser、selector/runtime identity分离与assignment缺失。 |
| P1-34..P1-35 | **Partial** | disconnect cleanup及部分logical/physical key原语存在；稳定匹配、layout迁移与Action reconcile缺失。 |
| P1-36 | **Open** | gamepad semantic layout/glyph family缺失。 |
| P1-37 | **Partial** | raw touch存在；gesture/motion/VR selector、capability与artifact扩展缺失。 |
| P1-38..P1-45 | **Open** | mappable metadata、profile delta、capture/conflict/atomic receipt、persistence、accessibility与privacy缺失。 |
| P1-46..P1-47 | **Partial** | document/dirty/asset/toolkit primitive可复用；Input contribution与adapter为零。 |
| P1-48..P1-55 | **Open** | Action/Context Editor、capture、inspector、conflict UI、debugger、PIE与layout/accessibility preview缺失。 |
| P1-56..P1-57 | **Partial** | command/gameplay authority保持分离，physical与UI consume已有事件级顺序；统一token、sequence ownership、capture lease与per-user schedule缺失。 |
| P1-58 | **Open** | settings/save/network/profile contributor闭环缺失。 |
| P1-59..P1-60 | **Partial** | 160个selected Rust tests及排序/index/reset/recorder benchmark可复用；产品矩阵、同语义性能、动态receipt和raw API零引用门缺失。 |

汇总：**46 Open / 14 Partial / 0 Closed**；Partial仅为**17、22、24、26、28、34、35、37、46、47、56、57、59、60**。

## 7. P2 currentness状态

| IDs | 状态 | 后续专项 |
|---|---|---|
| P2-1 | **Open** | advanced combo/sequence graph与compiled automata。 |
| P2-2 | **Partial** | raw bounded recorder/replay helper存在；缺Action artifact/profile/user/device/frame identity与确定性流。 |
| P2-3..P2-12 | **Open** | 自动scheme、平台认证、adaptive提示、校准、本地多人、semantic merge、extension SDK、latency lab、accessibility共享与simulation farm。 |

## 8. 参考引擎差异

| 参考 | 已验证合同 | Zircon应吸收 | 适用限制 |
|---|---|---|---|
| Unreal InputAction | 独立asset、Bool/Axis1D/2D/3D、aggregation、trigger/modifier、consume/reserve与mappable metadata。 | typed source/value/pipeline、stable metadata与消费政策。 | 不照搬UObject层次或TODO语义。 |
| Unreal Mapping/Subsystem | DataAsset mappings/profile override/input-mode filter/tracked registration；per-player rebuild/flush、held-key policy、query、injection和user settings。 | owner lease、per-user context、frame install、conflict query、profile/rebind与injection receipt。 | Zircon仍需自己的immutable artifact与fail-close compiler。 |
| Unreal Input Editor | property transaction、group/reorder、asset rename观察和5个mappable automation tests。 | transactional mapping UI、stable binding定位与真实asset lifecycle。 | Editor细节不应成为Runtime authority。 |
| Godot InputMap/Editor | project-persistent action/event/deadzone和可操作的add/edit/reorder配置UI。 | 产品可用性下限、typed event capture与明确持久化入口。 | 不足以覆盖complex trigger、artifact与multi-user。 |
| Bevy | typed raw events、ButtonInput边沿/复杂度、focus与gamepad设备语义。 | raw reducer、edge/reset与device-specific focus policy。 | 不是Action authoring方案。 |
| Fyrox | physical keyboard/mouse frame state与Editor key binding settings。 | 低层physical key和工具快捷键隔离参考。 | 明确不追踪多设备，不能作为InputUser产品。 |
| Unity Graphics模板 | JSON action map、expected control type、binding path、interaction/processor和control scheme样例。 | canonical serialized shape与device group测试语料。 | 只使用本地模板，不推测闭源Unity Input Editor。 |

## 9. Currentness资格门

| Gates | 状态 | 当前依据 |
|---|---|---|
| G01-G05 | **Fail** | asset/document/compiler/artifact/create-save-reopen-reference链缺失。 |
| G06-G10 | **Fail** | install/currentness/LKG/held policy与source-qualified diagnostics缺失。 |
| G11-G16 | **Fail** | typed value/phase/priority/consume/trigger/modifier/composite缺失。 |
| G17-G22 | **Fail** | InputUser/device assignment/reconnect/layout/profile与rebind atomicity缺失。 |
| G23-G27 | **Fail** | accessibility/privacy/script/PIE/network/save闭环缺失。 |
| G28-G32 | **Fail** | deterministic replay、fault/platform/scale/performance/soak与旧旁路清理缺失。 |

汇总：**32 Fail / 0 Partial / 0 Pass**。raw unit tests、ignored benchmark或disabled module不能提升产品gate。

## 10. 分层重构路线

1. **M0 Truthfulness**：Input资产/Editor入口保持Unavailable；撤销空manager、raw helper或局部测试暗示的产品可用性，冻结旧API与fixture inventory。
2. **M1 Stable Source**：建立Action/Context/Binding/Trigger/Modifier stable ID、typed selector/value/schema/migration，先写round-trip/rename/reorder/delete RED tests。
3. **M2 Compiler / Artifact**：shared semantic compiler产canonical IR/digest/dependency/capability/diagnostic，Editor、PIE、cook、shipping只消费同artifact。
4. **M3 Runtime Install / Evaluate**：per-user context owner lease、priority/consume、frame-barrier install、held policy、typed phase/value/snapshot与single terminal receipt。
5. **M4 Device / User**：stable device/control registry、seat/assignment/hotplug/reconnect/layout/glyph与LocalPlayer topology。
6. **M5 Profile / Rebind / Accessibility**：capture lease、conflict query、atomic profile transaction、migration/save/cloud/privacy和composable accessibility transform。
7. **M6 Transactional Editor**：Action/Context toolkit、graph/table/inspector/capture/conflict/debugger、LKG/currentness、undo/save/recovery。
8. **M7 Integration**：UI ownership sequence、typed script facade、PIE server/client/local players、network/save/replay artifact linkage。
9. **M8 Hard Cutover / Qualification**：删除raw gameplay旁路和旧map安装，完成fault、platform、10K/100K、latency、24h soak与同语义Unreal对照。

## 11. 性能与规模资格

必须绑定source/artifact/compiler/profile/device/user/topology/hardware digest，报告compile/install/rebind P50/P95/P99、physical-to-action latency、frame CPU/allocation/RSS、context churn、queue/record drop、debug observation成本和soak retention。矩阵至少覆盖1/1K/10K/100K bindings、1/4/16 users、keyboard/gamepad/touch、focus/hotplug/reconnect、held-map-swap与record/replay。当前局部sort/index/reset/retirement benchmark只证明数据结构优化，不能替代产品资格。

## 12. 禁止的临时修补

- 禁止继续给String map追加字段，或把runtime `GamepadId`、注册顺序、Debug key spelling持久化成stable identity。
- 禁止把Editor command keymap、camera controller DTO、raw recorder、Workbench binding或`key_pressed`改名成Gameplay Action产品。
- 禁止在`set_action_map`外加bool/lock就声称generation-safe install；必须有artifact、expected generation、frame barrier、LKG与receipt。
- 禁止按event kind继续散落手写UI/physical先后顺序；必须收敛到sequence-qualified routing与capture/consume policy。
- 禁止Editor/PIE/cook/shipping各写compiler，或保留old/new双轨、silent duplicate/missing/unknown与无界frame/recording。
- 禁止用单用户、默认disabled、无trigger/profile/rebind/accessibility的低延迟声称优于Unreal。

## 13. 本轮完成定义

本轮完成Editor29/87/103/150 current-source刷新：冻结107个Zircon selected文件、18,169行、628,994 bytes、160个Rust test attributes与20个ignored declarations；冻结23个参考文件、12,911行、502,917 bytes和50个conventional test markers。5项P0保持Open；P1保持46 Open/14 Partial/0 Closed；P2保持11 Open/1 Partial/0 Closed；32门全部Fail；canonical finding delta为零。

本轮只修改review与导航索引，不修改Runtime、Editor、App、plugin、ABI、tests或产品资源；没有运行Cargo或动态产品资格，也没有查询、轮询、等待或实时跟踪协调器。实现状态仍为pending，后续修正从M0 truthfulness和M1 stable source RED evidence开始。
