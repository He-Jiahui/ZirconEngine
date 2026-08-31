---
related_code:
  - zircon_editor/assets/ui/editor/keymap/default.keymap.toml
  - zircon_editor/assets/ui/editor/components/showcase/showcase_collections_section.zui
  - zircon_editor/assets/ui/editor/components/showcase/showcase_selection_section.zui
  - zircon_editor/src/core/commands/defaults.rs
  - zircon_editor/src/core/editing
  - zircon_editor/src/ui/workbench/state/editor_state_apply_intent.rs
  - zircon_editor/src/ui/workbench/state/editor_state_selection.rs
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/drag_source.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/events/drag.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/menu_items_for_layout.rs
  - zircon_runtime/src/scene/components/scene/node.rs
  - zircon_runtime/src/scene/world/hierarchy.rs
  - zircon_runtime/src/scene/world/records.rs
  - zircon_runtime/src/scene/world/transaction/detached_entity_batch.rs
  - zircon_runtime/src/scene/dynamic_scene/scene
  - zircon_runtime/src/scene/dynamic_scene/entity
  - zircon_runtime/src/scene/dynamic_scene/remap.rs
  - zircon_runtime/src/scene/dynamic_scene/value/remap.rs
  - zircon_runtime/src/ui/surface/input/keyboard_clipboard.rs
  - zircon_runtime/src/ui/surface/input/text_keyboard/clipboard.rs
  - zircon_runtime_interface/src/ui/component/drag.rs
  - zircon_runtime_interface/src/ui/dispatch/input/effect.rs
tests:
  - zircon_editor/src/tests/editing/node_ops.rs
  - zircon_editor/src/ui/retained_host/app/tests/drag_sources/scene_and_object.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/spawn/transaction/tests.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard_clipboard.rs
  - zircon_runtime/src/ui/tests/widget_text_input_keyboard/clipboard_newline.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/05-inspector-reflection-property-authoring-customization-review.md
  - docs/plans/optimize/zircon_editor/08-command-registry-keymap-menu-palette-context-routing-remote-automation-review.md
  - docs/plans/optimize/zircon_editor/23-ui-asset-hud-widget-binding-theme-icon-accessibility-menu-flow-font-atlas-authoring-review.md
  - docs/plans/optimize/zircon_editor/24-data-table-structured-data-schema-import-validation-save-game-slot-migration-platform-cloud-storage-authoring-review.md
  - docs/plans/optimize/zircon_editor/42-scene-snapshot-world-diff-merge-restore-conflict-resolution-authoring-review.md
  - docs/plans/optimize/zircon_runtime/11a-runtime-ui-architecture-tree-layout-input-accessibility-review.md
  - docs/plans/optimize/zircon_runtime/11b-runtime-text-font-shaping-layout-editing-ime-review.md
  - docs/plans/optimize/zircon_runtime/24-stable-identity-handle-generation-owner-epoch-stale-reference-exhaustion-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorActor.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/EditorServer.cpp
  - dev/UnrealEngine/Engine/Source/Editor/LevelEditor/Private/LevelEditorActions.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Classes/Editor/EditorEngine.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/Tests/CopyPasteCrossActorRefTests.cpp
  - dev/godot/editor/docks/scene_tree_dock.cpp
  - dev/godot/editor/docks/scene_tree_dock.h
  - dev/Fyrox/editor/src/scene/clipboard.rs
  - dev/Fyrox/editor/src/ui_scene/clipboard.rs
  - dev/bevy/crates/bevy_ecs/src/entity/clone_entities.rs
  - dev/bevy/crates/bevy_ecs/src/component/clone.rs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Util/CopyPasteGraph.cs
  - dev/Graphics/Packages/com.unity.shadergraph/Editor/Drawing/Views/MaterialGraphView.cs
doc_type: review-and-refactor-plan
refreshes:
  - docs/plans/optimize/zircon_editor/55-editor-structured-clipboard-cut-copy-paste-duplicate-delete-cross-document-remap-drag-payload-product-integration-review.md
  - docs/plans/optimize/zircon_editor/128-editor-structured-clipboard-cut-copy-paste-duplicate-delete-cross-document-remap-drag-payload-current-source-review.md
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# 176 · Editor Structured Clipboard / Cut-Copy-Paste / Duplicate / Delete / Cross-Document Remap / Drag Payload 当前源码复核

## 1. 结论

Editor55的当前P0仍为 **Open**：Zircon Editor没有Scene Copy、Cut、Paste或Duplicate产品路径。`EditorIntent`仍只有Create/Delete/Select/Rename/SetParent/Transform/Undo/Redo，`EditorCommand`仍只有Create/Delete/Update/SetReflectedSceneField；默认command、keymap、menu、provider、portable payload、paste plan、transfer receipt和Editor clipboard host consumer均不存在。showcase中的`Duplicate|Ctrl+D`继续只是静态资产。

旧版`ED55-P0-01`所指的Delete→Undo数据损失仍保持关闭：`DeleteNodeCommand`独占move-only `DetachedEntityBatch`，apply exact detach，revert exact restore，restore失败会把批次返还command以便重试；source guard继续拒绝退回`Vec<NodeRecord>`。但journal仍只保存`root_id`与`fallback_selection`，因此它只支持基线上的forward replay，不是portable inverse，也不能提供重启后的Delete undo。这个边界必须保留，不能把exact ECS storage batch误当跨进程clipboard格式。

Runtime `DynamicScene`新增或强化了schema/duplicate validation、先分配entity remap、preview、compiled preflight、commit和bounded staging，是未来transfer provider可复用的底层Partial；它仍只能`from_world()`捕获整个World，未映射entity reference仍原值保留，不能解决selected roots、external reference、portable identity或跨文档碰撞。当前状态为 **1项P0 Open；64项P1中48 Open / 16 Partial；12项P2全部Open；40门中28 Fail / 12 Partial / 0 Pass**。

## 2. currentness与产品可达性

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 证据 |
|---|---:|---|
| Zircon command/delete/drag/DynamicScene/text clipboard | **106 / 20,349 / 18,580 / 720,016 / 123 / 8** | 当前磁盘产品入口、exact detach、whole-World capture/spawn、drag与host request；fingerprint `43d5af9b1adf8398aecbaaa9c55deab719b87f83711c648fbf38b1dacd07dbeb` |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics | **13 / 29,549 / 25,105 / 1,117,327 / 31 / 0** | capability routing、top-root normalization、deep clone、reference map、dependency closure和single action；fingerprint `788fc52a4d3848a05194d31d5628f770cf8e3aee5ade76ff6583f7744ee9de42` |
| 计划与契约 | **12 / 5,676 / 4,002 / 628,849 / 1 / 0** | transaction、scene、provider、command、UI/text、identity owner边界；fingerprint `7dfc34970e2f1fe6db9a4afb3fb9e600004d86e67056d14a075ccfaba5864498` |
| 全部选择集 | **131 / 55,574 / 47,687 / 2,466,192 / 155 / 8** | normalized path + NUL + raw bytes + NUL；fingerprint `6b7513bdc1b2ec1cd2689a7573a7154d61c597094e45bd1af49eaabdd64f3a79` |

冻结时间`2026-08-27T17:47:04+08:00`，HEAD `ea35974cdf64068f6789010451d20bbf69e0a29d`，共享工作树8,289条status记录。聚焦域有大量在途修改，本报告只审查当前磁盘，不回退共享修改；实施前必须重取fingerprint、产品caller与build receipt。

| 能力 | 当前生产事实 | 判定 |
|---|---|---|
| Delete | command/key/menu/selection路径可达，exact batch本地撤销 | 可保留；durable inverse仍缺失 |
| Copy/Cut/Paste/Duplicate | `CopySelection=0`、`CutSelection=0`、`DuplicateSelection=0`、`StructuredClipboard=0`；Paste命中仅Graph局部vocabulary | Scene产品不存在 |
| hierarchy reparent drag | payload是`scene://node/{id}`，authoritative IDs另存`active_hierarchy_drag_node_ids`，pointer down即武装 | 可用reparent交互，但不是transfer protocol |
| DynamicScene capture | `from_world()`遍历全部`node_records()`和全部reflected resources | whole-World snapshot，不是selected-subtree payload |
| remap | `BTreeMap<EntityId,EntityId>`；unmapped typed/JSON entity保留原数值 | 跨World可碰巧误绑定 |
| Runtime clipboard | `ReadText/WriteText`且Editor对`RequestClipboard`为0 consumer | 文本request基础存在，产品host bridge断开 |

## 3. P0与必须保留边界

| ID | 状态 | 当前证据 | 必须重构 |
|---|---|---|---|
| ED55-P0-01 | **Open** | Scene Copy/Cut/Paste/Duplicate无command、keymap、menu、provider、service或portable payload | 建立`TransferDomainProvider -> CaptureRequest -> PortableAuthoringPayload -> PastePlan -> TransferReceipt`，再接Editor08与multi-format host clipboard |

`DetachedEntityBatch`只用于同World、同进程、move-only exact undo/rollback；`PortableAuthoringPayload`用于versioned、bounded、跨文档/进程的逻辑对象图。两个类型必须在API上隔离。Duplicate复用in-process capture→paste plan，不另造clone算法；Cut必须先取得clipboard write receipt，再revalidate source并在单document transaction删除。

## 4. P1逐项状态

### 4.1 产品入口与capture

| ID | 状态 | 当前差距 |
|---|---|---|
| ED55-P1-01 | Open | 无Copy/Cut/Paste/Duplicate command descriptor |
| ED55-P1-02 | Open | 默认keymap无authoring C/X/V/D chord与text-focus优先级 |
| ED55-P1-03 | Open | Edit/Selection菜单不投影四项transfer能力与disabled reason |
| ED55-P1-04 | Open | intent/command无transfer request/terminal receipt |
| ED55-P1-05 | Open | showcase静态Duplicate仍未绑定真实command |
| ED55-P1-06 | Open | 无focus/domain capability resolver或provider registry |
| ED55-P1-07 | Open | 无typed disabled reason |
| ED55-P1-08 | Open | keyboard/menu/context/remote没有共同operation authority |
| ED55-P1-09 | Open | 无绑定project/document/generation/provider的CaptureRequest |
| ED55-P1-10 | Partial | Delete已有top-level roots折叠，但未提升为所有provider共享的稳定root normalization合同 |
| ED55-P1-11 | Open | 无selected entities/subtree capture API |
| ED55-P1-12 | Open | `DynamicScene::from_world`仍只能whole-World capture |
| ED55-P1-13 | Partial | reflection capture区分component/resource与`serializable`，但无cloneable/portable/transient/forbidden policy registry |
| ED55-P1-14 | Open | unreflected component、observer与plugin state无portable policy/omission |
| ED55-P1-15 | Open | 无domain dependency collector |
| ED55-P1-16 | Open | 无orphan relation validation/cleanup阶段 |
| ED55-P1-17 | Partial | DynamicScene有schema header、exact version与deny-unknown validation；无transfer format/domain/source/limits/digest envelope |
| ED55-P1-18 | Open | 无portable object key和stable entry type/schema identity |
| ED55-P1-19 | Open | 无dependency manifest/link report |
| ED55-P1-20 | Open | 无complete/partial capture coverage和typed omission |

### 4.2 Identity、reference与destination

| ID | 状态 | 当前差距 |
|---|---|---|
| ED55-P1-21 | Open | payload无source document/generation |
| ED55-P1-22 | Partial | `EntityRemap`真实存在但只表达World-local数值ID map |
| ED55-P1-23 | Partial | NodeRecord remap已覆盖parent、joint和部分animation binding，仍无全字段/nested visitor |
| ED55-P1-24 | Open | JSON `{entity: n}`形状猜测仍是reference remap路径 |
| ED55-P1-25 | Open | unmapped entity保持原数值，跨文档仍可误绑定 |
| ED55-P1-26 | Open | 无internal/external/resource/owner reference table与decision receipt |
| ED55-P1-27 | Partial | DynamicScene spawn先建立全entity remap再compile records/components；未覆盖portable subobject/reference graph |
| ED55-P1-28 | Open | nested subobject/component无portable identity |
| ED55-P1-29 | Partial | whole-World spawn有resource staging/write基础，未定义reuse/duplicate/embed/import/owner policy |
| ED55-P1-30 | Open | plugin缺失/版本不兼容无transfer migration/link contract |
| ED55-P1-31 | Open | Duplicate产品不存在，无法证明与Paste共享identity语义 |
| ED55-P1-32 | Partial | DynamicScene会校验schema、duplicate source和plugin descriptor；跨project path/permission/trust仍无destination validation |
| ED55-P1-33 | Open | 无typed PasteDestination/document lease |
| ED55-P1-34 | Open | 无child/sibling/root/replacement paste和cycle policy |
| ED55-P1-35 | Open | 无Scene/Graph placement policy |
| ED55-P1-36 | Open | 无deterministic name conflict policy/receipt |
| ED55-P1-37 | Open | 无stable sibling insertion合同 |
| ED55-P1-38 | Open | 无owner closure prepare/apply validation |
| ED55-P1-39 | Partial | `SceneSelection`已有generation、ordered items和primary；transfer无selection result同代提交 |
| ED55-P1-40 | Open | 无focused document authority/lease，仍依赖全局state |
| ED55-P1-41 | Open | Scene edit仍使用Global history context |
| ED55-P1-42 | Partial | DynamicScene有spawn preview report，但无name/ref/resource/schema/owner conflict decision set |

### 4.3 Transaction、drag、平台与资格

| ID | 状态 | 当前差距 |
|---|---|---|
| ED55-P1-43 | Open | Cut无copy receipt→delete顺序 |
| ED55-P1-44 | Open | 无clipboard generation/source mutation saga和CopiedNotCut状态 |
| ED55-P1-45 | Partial | local Delete exact batch已完成；journal仍只有forward root/fallback，无portable inverse |
| ED55-P1-46 | Partial | transaction engine和command exact revert可作为多root rollback底座，未覆盖第N项失败的World/selection等价 |
| ED55-P1-47 | Partial | DynamicScene compiled preflight/commit有零目标mutation准备方向；无Editor paste单document transaction |
| ED55-P1-48 | Open | 无paste command-owned immutable plan，redo合同不存在 |
| ED55-P1-49 | Open | journal无transfer schema/provider/source/destination/identity摘要 |
| ED55-P1-50 | Open | 无Cut/Paste durable phase/idempotency/recovery |
| ED55-P1-51 | Open | drag payload仍只有kind/reference/source metadata |
| ED55-P1-52 | Open | authoritative NodeIds仍在payload外的Host Vec中 |
| ED55-P1-53 | Open | `scene://node/{id}`没有project/document/generation/owner资格 |
| ED55-P1-54 | Open | 通用`UiDragMetrics`存在但hierarchy不消费，pointer down仍立即武装 |
| ED55-P1-55 | Open | project/world/focus/plugin变化无transfer session retire合同 |
| ED55-P1-56 | Open | reparent/copy/move/link无operation negotiation |
| ED55-P1-57 | Open | host request仍仅ReadText/WriteText，无format inventory/custom MIME |
| ED55-P1-58 | Open | Editor retained host对`RequestClipboard`仍0 consumer |
| ED55-P1-59 | Open | 无OS clipboard ownership/change generation |
| ED55-P1-60 | Partial | DynamicScene compiled preflight有byte limit，但无不可信clipboard对象/edge/depth/string/decompression前置预算 |
| ED55-P1-61 | Open | 无payload digest/source诊断与独立destination validation |
| ED55-P1-62 | Partial | spawn task/preflight可分阶段并限制staging，capture与transfer仍可能在UI线程whole-World clone |
| ED55-P1-63 | Partial | DynamicScene已有局部profile/performance evidence；无transfer phase/provider/terminal metrics taxonomy |
| ED55-P1-64 | Open | 无operation/payload/plan/generation/decision terminal receipt或脱敏export |

## 5. P2状态

| ID | 状态 | 后续能力 |
|---|---|---|
| ED55-P2-01 | Open | Paste Special component/resource/reference policy |
| ED55-P2-02 | Open | 有预算、隐私与project隔离的clipboard history |
| ED55-P2-03 | Open | 同plan ghost preview与placement gizmo |
| ED55-P2-04 | Open | 多目标批量paste |
| ED55-P2-05 | Open | signed local cross-Editor transfer channel |
| ED55-P2-06 | Open | redacted bounded payload diagnostic view |
| ED55-P2-07 | Open | Paste as Replacement diff preview |
| ED55-P2-08 | Open | reference conflict可视化重连 |
| ED55-P2-09 | Open | provider telemetry dashboard |
| ED55-P2-10 | Open | automation payload artifact引用 |
| ED55-P2-11 | Open | Content Browser与Hierarchy typed drag-copy/link |
| ED55-P2-12 | Open | 跨版本migration preview |

## 6. 参考源码约束

| 参考 | 必须吸收的结构 |
|---|---|
| Unreal | 按focus/selection/domain能力路由；Cut在copy成功后单transaction删除；cross-actor/nested引用顺序无关remap |
| Godot | top selected root normalization、child/sibling/replacement、cycle、owner/resource policy与single UndoRedo action |
| Fyrox | clipboard隔离图double clone，返回old→new mapping和root handles；Scene/UI provider分域 |
| Bevy | per-component clone opt-in/out、required/linked relation、EntityMapper和insert policy显式化 |
| Unity Shader Graph | typed graph payload、依赖闭包、不可复制项/orphan edge过滤、target eligibility、undo和selection |

## 7. 目标架构与实施顺序

```text
Command/Keymap/Menu/Context
  -> TransferDomainProvider resolver
  -> CaptureRequest(source identity + generation + normalized roots)
  -> PortableAuthoringPayload(schema + object graph + dependencies + references + digest)
  -> PastePlan(destination lease + preallocated identities + decisions + selection)
  -> one Document Transaction
  -> TransferReceipt(terminal + generations + created/deleted + rollback)
```

1. 固化Delete exact-batch状态机并补table/sparse/dynamic/plugin/observer/tick、多root失败和durable语义测试。
2. Runtime增加subset capture、component clone/serialize/remap registry和external reference policy；未知类型默认fail-closed。
3. Editor落地transfer service与Scene provider，接command/menu/keymap/focus capability，先完成Duplicate与同document Paste。
4. App/Runtime扩展multi-format clipboard、custom MIME、inventory、generation和bounded decode；Cut实现可恢复saga。
5. hierarchy drag迁移到qualified transfer session，删除字符串+旁路IDs双权威，再扩展跨文档/provider。
6. 通过10K root/100K relation、恶意payload、provider unload/document close/crash、三平台clipboard、跨版本与UI latency资格。

## 8. 资格门状态

| Gate | 状态 | 当前判定 |
|---|---|---|
| G01 | Fail | 五类命令未来自同一descriptor/capability snapshot |
| G02 | Fail | text-focus与authoring provider优先级未建立 |
| G03 | Fail | 无selection/read-only/type/destination/payload typed reason |
| G04 | Fail | showcase仍可展示不存在的Duplicate |
| G05 | Fail | Scene Duplicate产品不存在 |
| G06 | Fail | 无四类paste destination/selection result |
| G07 | Fail | 无共同operation/receipt ID |
| G08 | Fail | 无provider lifecycle/Unavailable |
| G09 | Partial | exact batch存在；任意component/observer/tick/order资格未完整取得 |
| G10 | Partial | transaction rollback与exact revert是底座；多root第N项失败未验收 |
| G11 | Fail | 无subset capture |
| G12 | Partial | serializable/plugin descriptor校验存在；未知portable clone policy未默认拒绝并报告 |
| G13 | Partial | entity remap先分配；nested subobject与顺序无关全图测试缺失 |
| G14 | Fail | unmapped数值ID仍可在target碰巧命中 |
| G15 | Partial | resource write与部分built-in remap存在；四类reference policy/receipt缺失 |
| G16 | Fail | unsupported capture会跳过而非typed omission/拒绝 |
| G17 | Fail | 跨文档owner/resource/name/order无Undo/Redo合同 |
| G18 | Partial | exact schema/version/deny-unknown存在；无migration support window/golden roundtrip |
| G19 | Partial | compiled scene spawn有preflight方向；无paste零mutation plan与document transaction |
| G20 | Partial | spawn preflight/commit有失败保护底座；无selection/dirty/history全等价证明 |
| G21 | Fail | 无paste command state，无法证明redo独立于clipboard/source |
| G22 | Fail | Cut write receipt与source precondition不存在 |
| G23 | Fail | CopiedNotCut终态不存在 |
| G24 | Fail | 无document/world/provider generation session fence |
| G25 | Fail | 无Cut/Paste durable phase crash recovery |
| G26 | Partial | ordered/primary SceneSelection存在；无transfer同代transaction提交 |
| G27 | Fail | 无三平台custom MIME真实roundtrip |
| G28 | Fail | 无clipboard generation/change detection |
| G29 | Partial | whole-scene preflight有byte limit；clipboard decode多维预算不存在 |
| G30 | Fail | 无不可信payload path/URI/type/build destination validation |
| G31 | Partial | 有局部large DynamicScene ignored performance evidence；无10K/100K transfer UI/commit P95 |
| G32 | Partial | spawn/staging有局部byte/perf测试；无transfer peak memory/clone/payload基线 |
| G33 | Fail | 无transfer phase/provider/terminal metrics与redaction |
| G34 | Fail | 无可导出可复现receipt |
| G35 | Fail | 无Unreal级cross-reference corpus |
| G36 | Fail | 无Godot级top-root/placement/owner/single-action corpus |
| G37 | Fail | 无Fyrox级Scene/UI double-clone corpus |
| G38 | Fail | 无Bevy级per-component clone policy corpus |
| G39 | Fail | 无Unity Graph级dependency/orphan/target/selection产品测试 |
| G40 | Fail | 真实Editor窗口无法完成Scene Copy/Cut/Paste/Duplicate/跨文档/drag-copy全流程 |

## 9. 本轮边界

本轮只写review、索引与coverage，没有修改实现，也没有运行Cargo、Editor、GUI、OS clipboard、跨文档、crash、scale、soak或性能测试。Tooling实现按用户要求排除、未来迁移Rust；未查询、轮询、等待或实时跟踪协调器状态。实施前必须重新读取当前command/intent/delete/DynamicScene/host request/drag代码并取得Windows build/test receipt。
