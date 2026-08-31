---
title: Editor Input Action、Mapping Context、Binding、Trigger、Modifier、Device、User、Rebinding 与 Accessibility 当前源码复核
category: zircon_editor
report_id: Editor150
review_date: 2026-08-26
baseline_head: 166720dcb59c57fb4b33c34b859dc1a3f572b222
verification_head: 166720dcb59c57fb4b33c34b859dc1a3f572b222
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
canonical_owner: Editor29
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/29-input-action-mapping-context-binding-trigger-modifier-device-user-rebinding-accessibility-authoring-review.md
  - docs/plans/optimize/zircon_editor/87-editor-input-action-mapping-context-binding-trigger-modifier-device-user-rebinding-accessibility-authoring-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/103-editor-input-action-mapping-context-rebinding-accessibility-current-source-review.md
related_code:
  - zircon_runtime/src/core/framework/input
  - zircon_runtime/src/input
  - zircon_runtime/src/dynamic_api/session/events.rs
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

# Editor150 · Input Action / Mapping / Rebinding / Accessibility 当前源码复核

## 1. 结论

Zircon当前仍没有可交付的Input Action/Mapping Context authoring产品。资产类型、builtin registry、first-party Editor catalog、project manifest和App install链都没有Input Action或Mapping Context；production对`InputActionDocument`、`InputMappingContextDocument`、`CompiledInputMapArtifact`、`InputMapInstallRequest/Receipt`、`InputUser`、`PlayerBindingProfile`、`RebindCaptureRequest/Receipt`、`StableInputControlId`和`InputActionFrameSnapshot`的精确搜索为零。默认Input module仍用`enabled=false`和空map注册Action Manager，`evaluate_actions*`与configured descriptor没有production caller。

Runtime局部底座继续增长。Action evaluator具immutable generation、binding range、复用workspace、axis与consumed索引；physical input现在先提交再dispatch Runtime UI；frame buffer会合并相邻cursor/motion；focus/disconnect清理、gamepad阈值、host request跨帧保留以及有capacity/discarded status的raw recorder都是真实现。新增Free/Orbit/Pan camera controller也消费normalized controller DTO。这些改进不包含source asset、semantic compiler、artifact/install receipt、per-user context stack、trigger/modifier、profile/rebind或accessibility authority。

公共schema仍是调试级字符串结构。Action只有id/context/display name，Context只有id/priority/enabled，Binding仍保存action string、button chord和携runtime `GamepadId`的scalar axis，ActionState仍是string sets与`f32`。context虽然在map容器中按priority排序，evaluator generation只保存slot/enabled且不执行priority仲裁；duplicate继续静默忽略，missing context继续自动创建并enabled，unknown-action binding没有diagnostic，空active context仍表示全部启用。

因此Editor29/87/103状态不变：5项P0 Open；P1为46 Open/14 Partial/0 Closed；P2为11 Open/1 Partial/0 Closed；32门全部Fail。raw recorder的新边界加强P1-28/P2-2的Partial证据，但`InputRecording.frames`仍无总上界，record使用`SystemTime`且sequence饱和，replay丢原timestamp/sequence并可回放host-effect事件，不能升级为deterministic product。没有同语义benchmark receipt，不能声称优于Unreal。

## 2. 冻结范围与方法

本报告读取当前共享工作树，以`166720dcb59c57fb4b33c34b859dc1a3f572b222`标记提交基线。范围内存在其他会话在途修改，本轮不回退、不覆盖、不暂存。物理行按文件读取；tests统计Rust `#[test]`，ignored统计`#[ignore...]`；fingerprint由排序相对路径与逐文件SHA-256聚合。

| 选择集 | 文件 / 行 / 非空行 / bytes / tests / ignored | fingerprint |
|---|---:|---:|
| Runtime Input两棵源码树 | **66 / 8,494 / 7,559 / 286,663 / 90 / 12** | `e21531f1ffdfb25166c2dc7c81c1478b34472d8526771c5ec0a0e9a0ac129b9e` |
| Dynamic/App/Script/Manifest集成边界 | **8 / 2,795 / 2,609 / 112,475 / 14 / 0** | `fd22939c3fd1deac30da7d340d0377ce27a5e8fb845097385daa2b65db2c6de6` |
| Editor authoring primitive | **30 / 6,775 / 6,105 / 224,931 / 53 / 7** | `8b8208d41b07c6dae067216f99fc2238ed6d08a4585bd801ea539f016fe8fc00` |
| Zircon selected union | **104 / 18,064 / 16,273 / 624,069 / 157 / 19** | `a3374d84e968fc2839fcc2458a3ed88dab52d9ace5c6839e01142204fba0d131` |
| 参考选择集 | **23 / 12,911 / 11,558 / 502,917 / 50 conventional markers / 0** | `28bdc6c8f946f4a2b1b0544cd55de6c1751608dccfe068de9ab2cd8462c8a6a2` |

参考marker由45个Bevy Rust `#[test]`与5个Unreal automation declaration组成，只表示静态存在。Godot revision为`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`，Bevy为`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`，Fyrox为`8d815db36494f1badb347547dfc7094bf4fbbdf8`，Unity Graphics为`a7e4c051d256a781ab362c64316b125a1e104694`；Unreal以vendored文件fingerprint冻结。

本轮没有运行Cargo、Editor、真实设备、cook/install、PIE多玩家、rebind、accessibility、fault、scale、soak或跨引擎benchmark。Runtime12的input/gamepad storm failure仍为Open；Tooling按用户要求排除。

## 3. 当前产品事实

### 3.1 产品装配仍为零

- `ResourceKind`、builtin asset registry与first-party catalog无Input Action/Mapping Context contribution、factory、toolkit、thumbnail、reference analyzer或open route。
- `ProjectManifest`不引用input source/artifact/profile；App只按profile选择Input module，没有首帧前artifact install。
- `InputDriver`仍是零字段ZST；descriptor把driver、raw manager和Action manager都标为Immediate，却没有ingress、player tick、health或teardown dependency证明。
- `module_descriptor_with_config`除自身与tests外无caller；所有`evaluate_actions*` production caller为零。
- Vampire仍以`gameplay.key_pressed("A/D/W/S")`读取raw全局manager，绕过context、consume、profile、device assignment与action phase。

### 3.2 evaluator是底座，不是compiler/artifact

`InputActionMap::add_context`的新sorted insertion和binding的scratch-free sort是合理局部优化；generation把Action映射到binding ranges，workspace、axis index与consumed index降低每帧扫描和分配。但generation仍从任意serde map直接构建，没有schema/compiler/source/dependency digest、diagnostic、capability manifest或install receipt。priority排序没有进入evaluation决策，内部generation也没有公开identity、expected generation、frame barrier、LKG、rollback或held-input policy。

### 3.3 raw retention进步仍不足以成为Replay产品

`InputEventRecorder`默认disabled，启用后以`VecDeque`和capacity覆盖最老record并报告discarded count；capture frame能标记recording disabled或discarded，因此比旧无界record buffer更真实。但frame transient queue仍只有coalescing、没有capacity/age/drop contract，`begin_frame`会清除未drain事件。`InputRecording`自身可无限push frame；record只含wall-clock毫秒、饱和sequence和克隆event，没有schema/build/artifact/profile/user/device/clock/checksum。Replay重新submit event，不恢复原时序或Action stream。

### 3.4 Editor primitive不能冒充Gameplay authoring

Editor asset registry、dirty/save batch、creation template、toolkit primitive和command keymap具真实可复用能力；keymap还有typed chord、override与conflict基础。但没有任何Input document adapter或product contribution。Editor command shortcuts属于host operation authority，Gameplay mapping属于shipping Runtime authority；二者只能共享physical token/capture/conflict primitive，不能共享source identity或storage。

### 3.5 Camera controller不是Action产品consumer

新增Free/Orbit/Pan controller对movement/look/pan/zoom/focus DTO做确定性变换，可作为viewport/gameplay controller算法复用。其输入已经是调用者组装的normalized DTO，没有从Action artifact、InputUser、context stack或binding profile读取；它既不关闭production consumer断路，也不能作为Input authoring完成证据。

## 4. Owner边界与目标合同

| 领域 | 唯一owner | Editor150职责 |
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

最低合同必须包含stable Action/Context/Binding/Trigger/Modifier identity、Bool/Axis1D/2D/3D value、typed physical selector、schema/source/compiler/artifact/profile revisions、diagnostic span、capability requirement、InputUser/device lease、context priority/consume policy、frame tick/generation、single terminal install/rebind receipt和bounded observation。

## 5. P0 currentness重判

| ID | 状态 | 当前证据 | 硬切目标 |
|---|---|---|---|
| P0-1 | **Open** | 无Input资产、factory、toolkit、document或Editor surface。 | 建两类source asset、共享compiler与真实toolkit；无provider明确Unavailable。 |
| P0-2 | **Open** | shipping Action Manager默认disabled+空map，production install/evaluate caller为零。 | project引用artifact，首帧前frame-barrier install并返回typed receipt。 |
| P0-3 | **Open** | serialized button/axis仍保存runtime `GamepadId`。 | source保存stable selector，install按InputUser/device assignment解析。 |
| P0-4 | **Open** | duplicate/missing/unknown source仍被静默接受，priority未执行。 | semantic compiler fail-close并输出source-qualified diagnostics。 |
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
| P1-56..P1-57 | **Partial** | command/gameplay authority当前分离，physical-first与consumed index是真进展；共享token/sequence ownership/per-user schedule缺失。 |
| P1-58 | **Open** | settings/save/network/profile contributor闭环缺失。 |
| P1-59..P1-60 | **Partial** | 157个selected Rust tests与局部排序/index/reset benchmark可复用；产品矩阵、同语义性能和raw API零引用门缺失。 |

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
| Unreal Mapping/Subsystem | DataAsset mappings/profile override/input-mode filter/tracked registration；per-player rebuild/flush、held-key policy、query、injection和user settings。 | owner lease、per-user context、frame install、conflict query、profile/rebind与injection receipt。 | 仍需Zircon自己的immutable artifact和fail-close compiler。 |
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

汇总：**32 Fail / 0 Partial / 0 Pass**。raw unit tests或disabled module不能提升产品gate。

## 10. 分层重构路线

1. **M0 Truthfulness**：Input资产/Editor入口保持Unavailable；撤销空manager、raw helper或局部测试暗示的产品可用性，冻结旧API/fixture inventory。
2. **M1 Stable Source**：建立Action/Context/Binding/Trigger/Modifier stable ID、typed selector/value/schema/migration，先写round-trip/rename/reorder/delete RED tests。
3. **M2 Compiler / Artifact**：shared semantic compiler产canonical IR/digest/dependency/capability/diagnostic，Editor、PIE、cook、shipping只消费同artifact。
4. **M3 Runtime Install / Evaluate**：per-user context owner lease、priority/consume、frame-barrier install、held policy、typed phase/value/snapshot与single terminal receipt。
5. **M4 Device / User**：stable device/control registry、seat/assignment/hotplug/reconnect/layout/glyph与LocalPlayer topology。
6. **M5 Profile / Rebind / Accessibility**：capture lease、conflict query、atomic profile transaction、migration/save/cloud/privacy和composable accessibility transform。
7. **M6 Transactional Editor**：Action/Context toolkit、graph/table/inspector/capture/conflict/debugger、LKG/currentness、undo/save/recovery。
8. **M7 Integration**：UI ownership sequence、typed script facade、PIE server/client/local players、network/save/replay artifact linkage。
9. **M8 Hard Cutover / Qualification**：删除raw gameplay旁路和旧map安装，完成fault、platform、10K/100K、latency、24h soak与同语义Unreal对照。

## 11. 性能与规模资格

必须绑定source/artifact/compiler/profile/device/user/topology/hardware digest，报告compile/install/rebind P50/P95/P99、physical-to-action latency、frame CPU/allocation/RSS、context churn、queue/record drop、debug observation成本和soak retention。矩阵至少覆盖1/1K/10K/100K bindings、1/4/16 users、keyboard/gamepad/touch、focus/hotplug/reconnect、held-map-swap与record/replay。局部sort/index benchmark不能替代产品资格。

## 12. 禁止的临时修补

- 禁止继续给String map追加字段，或把runtime `GamepadId`、注册顺序、Debug key spelling持久化成stable identity。
- 禁止把Editor command keymap、camera controller DTO、raw recorder、Workbench binding或`key_pressed`改名成Gameplay Action产品。
- 禁止在`set_action_map`外加bool/lock就声称generation-safe install；必须有artifact、expected generation、frame barrier、LKG与receipt。
- 禁止Editor/PIE/cook/shipping各写compiler，或保留old/new双轨、silent duplicate/missing/unknown与无界frame/recording。
- 禁止用单用户、默认disabled、无trigger/profile/rebind/accessibility的低延迟声称优于Unreal。

## 13. 本轮完成定义

本轮完成Editor29/87/103 current-source刷新：冻结104个Zircon selected文件、18,064行、624,069 bytes、157个Rust test attributes与19个ignored declarations；冻结23个参考文件、12,911行、502,917 bytes和50个conventional test markers。5项P0保持Open；P1保持46 Open/14 Partial/0 Closed；P2保持11 Open/1 Partial/0 Closed；32门全部Fail；canonical finding delta为零。

本轮只修改review与导航索引，不修改Runtime、Editor、App、plugin、ABI、tests或产品资源；没有运行Cargo或动态产品资格，也没有查询、轮询、等待或实时跟踪协调器。实现状态仍为pending，后续修正从M0 truthfulness和M1 stable source RED evidence开始。
