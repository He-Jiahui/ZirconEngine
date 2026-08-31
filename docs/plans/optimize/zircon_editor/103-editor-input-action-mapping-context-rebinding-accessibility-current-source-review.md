---
title: Editor Input Action、Mapping Context、Binding、Trigger、Modifier、Device、User、Rebinding 与 Accessibility 当前源码复核
category: zircon_editor
report_id: Editor103
review_date: 2026-08-26
baseline_head: 3282dfad2a3a0dce246dfa8f300d7d30d70ed9a9
baseline_epoch: 524
canonical_owner: Editor29
refreshes:
  - docs/plans/optimize/zircon_editor/29-input-action-mapping-context-binding-trigger-modifier-device-user-rebinding-accessibility-authoring-review.md
  - docs/plans/optimize/zircon_editor/87-editor-input-action-mapping-context-binding-trigger-modifier-device-user-rebinding-accessibility-authoring-product-integration-current-source-review.md
related_code:
  - zircon_runtime/src/core/framework/input
  - zircon_runtime/src/input
  - zircon_runtime/src/dynamic_api/session/events.rs
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
tests:
  - zircon_runtime/src/input/tests/action_mapping.rs
  - zircon_runtime/src/input/tests/action_axis_transitions.rs
  - zircon_runtime/src/input/tests/recording_replay.rs
  - zircon_runtime/src/input/tests/input_manager
  - zircon_runtime/src/dynamic_api/session/tests/physical_input_ownership.rs
  - zircon_editor/src/core/commands/keymap/tests.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/06-platform-input-process-review.md
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/04-asset-index-import-reimport-catalog-thumbnail-reference-workflow-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/12-settings-preferences-scope-persistence-locale-i18n-appearance-plugin-extensibility-review.md
  - docs/plans/optimize/zircon_editor/53-editor-interactive-tool-scheduler-resource-lease-input-capture-mode-modal-extension-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_editor/59-editor-scene-viewport-interaction-controller-input-picking-selection-highlight-gizmo-transaction-cancel-generation-product-integration-current-source-review.md
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
doc_type: current_source_refresh
review_status: complete
implementation_status: pending
source_recheck_required: true
finding_status:
  p0: 5 open
  p1: 46 open
  p1_partial: 14
  p2: 11 open
  p2_partial: 1
gate_status:
  fail: 32
  partial: 0
  pass: 0
---

# Editor29/103 · Input Action、Mapping Context、Binding、Trigger、Modifier、Device、User、Rebinding 与 Accessibility 当前源码复核

## 1. 结论

Zircon 已有可保留的 raw input 与 scalar Action evaluator 底座：`InputActionEvaluator` 在 map 变化时生成 immutable `ActionEvaluationGeneration`，按 action 建立 binding range；`ActionEvaluationWorkspace`、frame axis index、consumed input index 可复用，测试覆盖 chord、context、consumed input、gamepad axis、transition、map replacement 与 10K binding 局部规模。`dynamic_api/session/events.rs` 当前还把 physical input 提交置于 UI dispatch 之前，并保留 physical ownership 的 release 语义。这些是 Runtime06 输入 owner 的真实基础。

但不存在可交付的 Input Action / Mapping Context 产品。`ResourceKind`/builtin asset registry 没有 Input Action 或 Mapping Context，first-party catalog 没有 Input factory/toolkit/surface，`ProjectManifest` 没有 input source/artifact/profile，`zircon_app` 只选择 Input module 而不安装项目 map。默认 `InputConfig` 仍是 `enabled = false`、空 `InputActionMap`；对 `module_descriptor_with_config` 和 `evaluate_actions*` 没有 production asset/cook 调用链。

公共 schema 仍是调试级形状：Action/Context/Binding identity 是裸 `String`；Action 只有 id/context/display_name，Context 只有 id/priority/enabled，Binding 只有 button chord 和 scalar gamepad axis，state 只有 string set 与 `f32`。没有 Bool/Axis1D/2D/3D typed value、phase、trigger、modifier/processor、composite、stable binding/source ID、diagnostic、artifact identity、InputUser、LocalPlayer、device assignment、profile revision 或 rebind receipt。`InputAxisBinding` 与 `InputButton::Gamepad` 直接序列化 runtime `GamepadId`。

非法 source 会静默改变语义：duplicate action/context 被 helper 忽略；missing context 在 generation 中自动插入且默认 enabled；unknown action binding 没有诊断；空 `active_contexts` 表示所有 context active；priority 只保存但不形成确定执行顺序；`set_action_map` 在 mutex 内整图替换，没有 expected generation、frame barrier、held-input policy、LKG 或 install receipt。Gameplay script 继续通过 `gameplay.key_pressed(string)` 读取 raw snapshot，绕过 context/consume/rebind/device/user/action phase。

因此 Editor87 的原账本只做 current-source 重判：5 项 P0 全 Open；60 项 P1 中 46 Open、14 Partial、0 Closed；12 项 P2 中 11 Open、1 Partial；32 Gate 全 Fail。Partial 只归因于 evaluator generation/workspace、physical ownership、raw recording、Editor keymap/dirty/settings 和局部性能测试，不能算作 authoring、cook/install、per-user evaluation、rebind/profile 或 accessibility 完成。没有同语义 benchmark receipt，不能声称优于 Unreal。

## 2. 物理范围与当前事实

| 范围 | 文件 | 行 | 非空行 | bytes | tests | fingerprint |
|---|---:|---:|---:|---:|---:|---|
| Zircon Runtime/Editor/App selected | 101 | 17,750 | 15,978 | 610,938 | 233 | `943c46d563a3cf8499148832d0ba14d06b77023fc4c54d4dc822e13f5b83b4cc` |
| Unreal/Godot/Bevy/Fyrox/Unity reference | 21 | 13,141 | 11,730 | 513,517 | 54 | `4e433cc11994f89823a59c49059ce21c1ba1fc0cd9af1a8a02e9eea6e47a1441` |

当前 selected union 去重统计为 122 文件、30,891 行、1,124,455 bytes、287 个静态 test declarations；fingerprint `6c610fb87df39010b558de36865a3d68aaaa8b56732e56842bc8cbe285023001`。本轮只静态扫描，不运行 Cargo、runtime input focused tests、Editor authoring、device reconnect、cook/install、rebind、multiplayer 或 accessibility lane。

逐文件事实：

1. `InputAction` 是 `{ id: String, context: Option<String>, display_name: Option<String> }`；`InputActionContext` 是 `{ id: String, priority: i32, enabled: bool }`。
2. `InputBinding` 是 action string + button list + `InputAxisBinding { gamepad: GamepadId, axis, direction }`；没有 stable binding ID、device selector、scale/deadzone/invert/composite。
3. `InputActionState` 没有 map generation、frame/tick、InputUser、phase、typed vector value 或 source binding。
4. `InputConfig::default()` 明确关闭 action map；`module_descriptor()` 调用它；没有 project asset/cook install bridge。
5. `DefaultInputActionManager::set_action_map` 立即替换 evaluator；所有 evaluate API 仍接收 string context 和 physical consumed arrays。
6. `gameplay_host/input.rs` 的 raw key helper 只查 InputManager；不存在 Action snapshot、LocalPlayer 或 player profile consumer。
7. builtin asset registry 当前包含 Data/Model/Mesh/Material/Texture/Shader/Scene/Sound/Font/Physics/Nav/World/Animation/UI 等，但没有 Input Action/Mapping Context；first-party catalog 不能提供 factory/toolkit。
8. Editor command keymap 反而有 typed operation path、settings override、signature index、conflict enumeration 和 missing-command fail-safe；这些应复用为 primitive，但 command keymap 与 shipping gameplay map 必须保持两套 authority。
9. physical-first session event 顺序改善了 UI capture 后的 raw release 传播，但尚未形成 UI consume -> per-user context -> action evaluation 的正式 frame schedule。

## 3. 参考引擎对照

- Unreal Enhanced Input 把 `InputAction` 作为独立 asset，包含 Bool/Axis1D/2D/3D value、trigger、modifier、consume/paused policy 和 player-mappable metadata；`InputMappingContext` 独立保存 mappings、priority、profile override 与 activation。
- Unreal subsystem 按 player 管理 mapping context、priority、add/remove/rebuild/flush、mapping query issue、user settings/profile 和 frame-end rebuilt event；Input Editor 提供 asset definition/customization，不把 authoring 藏在 module constructor。
- Godot `InputMap` 提供 project persistence、action 增删、deadzone、InputEvent 增删和 editor configuration dialog，是可用产品下限；Zircon 还需 stable artifact、多用户和复杂 trigger graph。
- Bevy raw keyboard/gamepad/ButtonInput/focus 适合 Runtime 物理事件参考，不等于 Action authoring。Fyrox 提供低层 engine/UI key 语义；Unity Graphics 模板只作为 serialized inputaction 与 debug input consumer 的结构参考，不推测闭源 Editor 行为。

## 4. Owner 边界与目标链

| 领域 | owner | Editor29 边界 |
|---|---|---|
| raw device/window/focus/frame reducer/physical ownership | Runtime06/117 | 消费 qualified physical snapshot，不复制 device registry |
| LocalPlayer/Controller/possess/gameplay tick | Runtime Gameplay99zb | 提供 InputUser consumption contract |
| Action/Mapping source/compiler/artifact/install | Runtime06A + Editor29 | shared schema/compiler；Editor 负责 document/diagnostics/profile UX |
| Editor command shortcuts | Editor08 | 复用 capture/conflict primitives，不共享 gameplay storage |
| document/asset/job/notification/journal | Editor02/04/09/10/11 | source revision、factory/toolkit、compile/rebind receipt |
| settings/profile/persistence | Editor12 + platform/save | versioned profile delta 与 migration |
| interactive capture | Editor53 | rebind lease/state machine，不能劫持 viewport tool capture |
| viewport/UI input | Editor59 + Runtime UI | 消费 command/action，不拥有 shipping schema |
| script gameplay | Runtime script/Gameplay | raw key 降为低层 capability，玩法消费 typed action snapshot |

目标闭环：

```text
InputActionDocument + InputMappingContextDocument
  -> InputSemanticCompiler -> CompiledInputMapArtifact
  -> InputMapInstaller -> per-InputUser ActiveInputMapGeneration
  -> InputActionFrameSnapshot -> Gameplay/UI/Script consumers

PlayerBindingProfile -> RebindCaptureRequest + ConflictQuery
  -> atomic RebindReceipt/ProfileRevision -> same installer/generation
```

最小合同：`InputActionDocument { action_id, schema_version, source_revision, value_type, trigger_graph, modifiers, consumption_policy, player_mapping_metadata }`；`InputMappingContextDocument { context_id, schema_version, mappings, activation_policy, priority_policy, profile_overrides, dependencies }`；`PhysicalInputPattern` 只存 device class/layout/control/logical policy，绝不序列化 runtime GamepadId；`CompiledInputMapArtifact` 携 compiler/version/source/dependency digest、diagnostics 与 bounded payload；`InputUserId/LocalPlayerId`、`ActiveMapGeneration`、`ActionFrameSnapshot`、`RebindCaptureRequest/Receipt` 均为 stable typed IDs。

## 5. P0：不可交付与错误边界

| ID | 当前证据 | 必须重构 |
|---|---|---|
| P0-1 | 无 Input Action/Mapping Context asset、factory、toolkit、Editor surface | 建立 source document、asset registry、toolkit、thumbnail/reference/analyzer |
| P0-2 | shipping Input 默认 disabled+空 map，无 project/cook install | project manifest 只引用 artifact；App 在 frame barrier 安装并返回 receipt |
| P0-3 | serialized binding 保存 runtime GamepadId | 改为 stable physical token/device layout，重连/第二玩家不串线 |
| P0-4 | duplicate/missing/unknown action/context 静默改语义 | compiler diagnostics、deny-unknown、collision admission、fail-closed artifact |
| P0-5 | gameplay.key_pressed 绕过 Action Manager且无 InputUser/profile | 拆 raw capability，玩法改消费 per-user typed action snapshot，rebind 原子提交 |

## 6. P1：Source、Schema、Compiler、Runtime 与 Profile

| ID | 差异与重构要求 | ID | 差异与重构要求 |
|---|---|---|---|
| P1-1 | Input Action asset kind 缺失 | P1-2 | Mapping Context asset kind 缺失 |
| P1-3 | action/context 无 stable identity | P1-4 | schema/version/header/unknown policy 缺失 |
| P1-5 | Bool/Axis1D/2D/3D value type 缺失 | P1-6 | device-independent physical token 缺失 |
| P1-7 | binding stable ID/source span 缺失 | P1-8 | context priority/activation/lease 缺失 |
| P1-9 | duplicate/collision admission 缺失 | P1-10 | reference integrity/dependency graph 缺失 |
| P1-11 | trigger graph/phase/hold/tap/chord 缺失 | P1-12 | modifier/processor/deadzone/scale/invert 缺失 |
| P1-13 | composite/vector/2D/3D binding 缺失 | P1-14 | consume/paused/reserve policy 缺失 |
| P1-15 | localization/display metadata contract 缺失 | P1-16 | compiler diagnostics/span/recovery 缺失 |
| P1-17 | immutable compiled artifact/digest 缺失 | P1-18 | dependency/cook/platform capability manifest 缺失 |
| P1-19 | generation install/frame barrier 缺失 | P1-20 | expected generation/LKG/rollback 缺失 |
| P1-21 | held input/release policy on map swap 缺失 | P1-22 | action phase/value/source binding in state 缺失 |
| P1-23 | InputUser/LocalPlayer identity 缺失 | P1-24 | device assignment/ownership/seat policy 缺失 |
| P1-25 | reconnect/hotplug/remap migration 缺失 | P1-26 | player-mappable profile schema/revision 缺失 |
| P1-27 | rebind capture lease/deadline/cancel 缺失 | P1-28 | conflict query/priority/override resolution 缺失 |
| P1-29 | atomic rebind receipt/audit/journal 缺失 | P1-30 | profile import/export/redaction/migration 缺失 |

## 7. P1：Product Integration、Debug、Accessibility 与资格

| ID | 差异与重构要求 | ID | 差异与重构要求 |
|---|---|---|---|
| P1-31 | Editor document/history/undo/save/recovery 缺失 | P1-32 | catalog/factory/toolkit/thumbnail/reference closure 缺失 |
| P1-33 | compile/install job/progress/cancel/diagnostic 缺失 | P1-34 | Action debugger/trace/trigger visualization 缺失 |
| P1-35 | per-user context stack/activation observation 缺失 | P1-36 | UI consume/capture/cancel/release integration 缺失 |
| P1-37 | gameplay/script action snapshot API 缺失 | P1-38 | Play/PIE/multi-user device topology 缺失 |
| P1-39 | network/local-player/replication input profile bridge 缺失 | P1-40 | save/cloud/platform profile participant 缺失 |
| P1-41 | accessibility alternate binding/hold/toggle policy 缺失 | P1-42 | assistive device/token/locale metadata 缺失 |
| P1-43 | keyboard/gamepad/touch/mouse layout qualification 缺失 | P1-44 | unknown device/fallback/permission UX 缺失 |
| P1-45 | raw/action event ordering and frame schedule contract 缺失 | P1-46 | action value transition/edge/consumed semantics completeness 缺失 |
| P1-47 | large map compile/index budget 缺失 | P1-48 | context churn/install tail latency budget 缺失 |
| P1-49 | allocation/input-to-action latency telemetry 缺失 | P1-50 | 10K/100K binding and multi-user scale qualification 缺失 |
| P1-51 | device reconnect/seat swap/focus fault matrix 缺失 | P1-52 | deterministic recording/replay with artifact identity 缺失 |
| P1-53 | platform-specific keycode/layout migration 缺失 | P1-54 | schema compatibility/plugin contributor SDK 缺失 |
| P1-55 | security/permission/user profile boundary 缺失 | P1-56 | redaction/secret/device identifier policy 缺失 |
| P1-57 | command keymap/gameplay map authority separation gate 缺失 | P1-58 | obsolete raw key API migration/deprecation gate 缺失 |
| P1-59 | complete integration/test matrix 缺失 | P1-60 | benchmark/soak/legacy fixture zero-reference gate 缺失 |

## 8. P2：扩展能力

| ID | 当前缺口 | 方向 |
|---|---|---|
| P2-1 | multi-device chord arbitration | deterministic device/user policy |
| P2-2 | accessibility gesture/assistive remap | typed alternate action profiles |
| P2-3 | haptics/rumble action output | output mapping artifact |
| P2-4 | context prediction/rollback | network-aware action generation |
| P2-5 | cloud profile sync/merge | revision/etag conflict resolver |
| P2-6 | visual input debugger/replay diff | artifact-bound trace comparison |
| P2-7 | mod/plugin input contributor | versioned extension SDK |
| P2-8 | localization/layout auto adaptation | locale/device capability resolver |
| P2-9 | remote control/automation input safety | scoped lease and redaction |
| P2-10 | distributed input latency lab | same-device/network test receipt |
| P2-11 | cross-engine import/export | versioned migration and denial policy |

## 9. 32 个 Gate 与重构顺序

32 个 gate 覆盖 asset/schema/compiler、artifact/install、per-user/device/profile、raw/action schedule、Editor authoring/debug/accessibility、PIE/network/save、security、scale/latency、record/replay、migration 与 fixture hard-cutover，当前全部 Fail。分层顺序如下：

1. **Truthfulness**：没有 asset/catalog/provider 时从产品入口移除 Input Action/Mapping Context 假入口，raw key 仅标记低层/debug，不再宣传 gameplay action 已可创作。
2. **Source/artifact**：建立两个 versioned document、stable IDs、typed value/trigger/modifier/composite、compiler diagnostics、immutable artifact 和 platform dependency manifest。
3. **Runtime install**：以 expected generation + frame barrier 安装 per-InputUser map，保留 LKG、held-input/release policy、rollback 和 terminal receipt；复用现有 generation/workspace，不复制 evaluator 全表扫描。
4. **Profile/rebind**：建立 device-independent token、InputUser/LocalPlayer、capture lease、conflict query、atomic profile revision、permission/redaction 与 cloud/platform adapter。
5. **Product integration**：Editor04 factory/toolkit、Editor02 transaction、Editor09 compile/rebind job、Editor10 notification、Editor11 journal、Editor53 capture 与 Editor59 viewport input 映射统一闭环。
6. **Gameplay migration**：将 script raw key 降级为受限 capability，玩法读取 typed action snapshot；PIE/network/save 使用同一 artifact/profile，禁止另建 test map。
7. **Qualification**：以相同硬件、设备、帧率、map complexity 和 user topology 记录 compile/install latency、input-to-action latency P95/P99、frame CPU/allocation、RSS、map churn、reconnect、replay、soak 和 accessibility receipts；没有同语义数据不宣称优于 Unreal。

## 10. 禁止的临时修补与验证边界

- 不得把更多字段追加到 `InputAction`/`InputBinding` 的字符串结构来假装 schema 完整，也不得将 runtime `GamepadId` 映射成持久化 user identity。
- 不得把 Editor command keymap TOML、viewport key handling、Workbench control binding 或 raw `gameplay.key_pressed` 当作 Gameplay Action authoring authority。
- 不得在 `set_action_map` 周围加一个 bool 或锁就声称 generation-safe install；必须有 artifact digest、expected generation、frame barrier、LKG、receipt 和 late event policy。
- 不得用 10K scalar evaluator 测试、未启用的默认模块或无 user/device 的低延迟来宣称工程级性能。

已完成当前工作树递归枚举、Input schema/evaluator/module/registry/script/keymap 逐层阅读、reference path 检查和 fingerprint 冻结；未运行 Cargo 或动态 Input lane。`source_recheck_required: true` 反映共享 dirty worktree，后续实现前必须重算 selected manifest。Editor103 只刷新 Editor29/87 currentness，不实施生产代码。
