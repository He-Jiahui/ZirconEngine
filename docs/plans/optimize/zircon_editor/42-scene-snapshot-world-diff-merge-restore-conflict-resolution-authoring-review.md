---
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_top_toolbar.zui
  - zircon_editor/src/core/commands/defaults.rs
  - zircon_editor/src/core/editing
  - zircon_editor/src/core/play/snapshot
  - zircon_editor/src/core/recovery/restore_flow.rs
  - zircon_editor/src/ui/asset_editor/document_diff.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_command_feedback.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions.rs
  - zircon_runtime/src/scene/dynamic_scene/scene
  - zircon_runtime/src/scene/dynamic_scene/session
  - zircon_runtime/src/scene/inspection
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime_interface/src/reflect/type_registration.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/11-logging-diagnostic-journal-output-console-status-routing-retention-export-review.md
  - docs/plans/optimize/zircon_editor/24-data-table-structured-data-schema-import-validation-save-game-slot-migration-platform-cloud-storage-authoring-review.md
  - docs/plans/optimize/zircon_editor/27-project-operations-source-control-changelist-diff-automation-report-submit-gates-health-dashboard-review.md
  - docs/plans/optimize/zircon_editor/41-level-variant-data-layer-level-instance-world-outliner-authoring-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Plugins/VirtualProduction/LevelSnapshots/Source/LevelSnapshots/Public/Data/LevelSnapshot.h
  - dev/UnrealEngine/Engine/Plugins/VirtualProduction/LevelSnapshots/Source/LevelSnapshots/Public/Data/WorldSnapshotData.h
  - dev/UnrealEngine/Engine/Plugins/VirtualProduction/LevelSnapshots/Source/LevelSnapshots/Public/Data/ActorSnapshotData.h
  - dev/UnrealEngine/Engine/Plugins/VirtualProduction/LevelSnapshots/Source/LevelSnapshots/Public/Data/Hashing/ActorSnapshotHash.h
  - dev/UnrealEngine/Engine/Plugins/VirtualProduction/LevelSnapshots/Source/LevelSnapshots/Public/Filtering/PropertySelectionMap.h
  - dev/UnrealEngine/Engine/Plugins/VirtualProduction/LevelSnapshots/Source/LevelSnapshots/Public/Restorability/Interfaces/ICustomObjectSnapshotSerializer.h
  - dev/UnrealEngine/Engine/Plugins/VirtualProduction/LevelSnapshots/Source/LevelSnapshots/Public/Restorability/Interfaces/IRestorationListener.h
  - dev/UnrealEngine/Engine/Plugins/VirtualProduction/LevelSnapshots/Source/LevelSnapshots/Public/Restorability/Interfaces/ISnapshotFilterExtender.h
  - dev/UnrealEngine/Engine/Plugins/VirtualProduction/LevelSnapshots/Source/LevelSnapshots/Public/Restorability/SnapshotRestorability.h
  - dev/UnrealEngine/Engine/Plugins/VirtualProduction/LevelSnapshots/Source/LevelSnapshotsEditor/Private/Views/Results
  - dev/godot/scene/resources/packed_scene.h
  - dev/godot/scene/resources/packed_scene.cpp
  - dev/godot/editor/docks/scene_tree_dock.cpp
  - dev/Fyrox/editor/src/command/mod.rs
  - dev/Fyrox/editor/src/scene/commands/graph.rs
  - dev/bevy/crates/bevy_scene/src/scene.rs
  - dev/bevy/crates/bevy_scene/src/scene_patch.rs
  - dev/bevy/crates/bevy_scene/src/resolved_scene.rs
  - dev/bevy/crates/bevy_scene/src/spawn.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.high-definition/Runtime/Utilities/SceneObjectIDMap.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 42 · Scene Snapshot / World Diff / Merge / Restore / Conflict Resolution Authoring 工程化差距

## 1. 结论

Zircon当前没有工程级Scene Snapshot、语义World Diff、三方Merge、选择性Restore或Conflict Resolution产品。仓内存在一套规模很大的Runtime Session Archive和可工作的DynamicScene捕获/重放底座，但其真实语义是“把可反射序列化的当前World装进slot，再做归档持久化、整槽选择或重放”。它不能因函数名包含`diff`、`merge`、`restore`就被直接暴露为Editor场景比较产品。

当前`RuntimeSessionSlot::diff_world`只重新捕获目标世界并计算`self.scene == target_scene`，报告仅含是否完全相等及entity/resource数量。Archive merge只处理重复`slot_id`，策略是Reject/Keep/Replace；这不是对象、component、property、reference或三方冲突合并。`apply_to_world`是additive spawn，`restore_into_level`则先构造空World再调用`replace_world_and_reset_runtime_state`整世界替换。把后三者直接接到Editor会分别产生重复对象、丢失未捕获状态，或绕过事务、dirty、selection、打开文档、viewport和source revision保护。

DynamicScene本身不是mock。它拥有版本化document、reflection adapter、component/resource value、entity remap、插件component描述、spawn preflight和事务式发布基础；Session Archive还拥有canonical payload、512 MiB上限、seal/cache、lineage/revision、manifest/statistics/index、retention/selection、preview/commit、bounded writer及带路径CAS的原子文件替换。这些应被复用为底层序列化和存储能力，而不是复制一套Editor私有格式。

然而捕获是“best effort且静默不完整”的：只遍历已注册、声明为component/resource、标为serializable、拥有adapter并实际存在的类型，再只保存标为serializable的字段。未注册storage、无adapter类型、不可序列化字段和失败原因不会进入覆盖清单。一个看似成功的snapshot因而可能缺少决定性authoring状态，随后整世界restore会把缺失解释为不存在，形成数据丢失风险。

Workbench顶部已经公开“Diff”命令，但Scene分支只返回`Scene diff prepared`和`Diff: scene preview state compared`固定文字，没有snapshot、comparison source、change set、provider或apply executor。这是P0虚假能力面。Play Snapshot只是启动子进程的临时`.zircon/play/<instance>/play-scene.zrscene.json`输入；autosave `RestoreFlow`只生成计划且没有执行产品；UI Asset `DocumentDiff`只保存完整目标document并整体替换。三者均不能冒充Scene Snapshot产品。

目标架构必须建立五个明确owner：不可变`SceneSnapshotAsset`、产生typed `SceneChangeSet`的`SceneDiffService`、显式base/ours/theirs的`SceneMergeService`、事务化`SceneRestoreCoordinator`和Editor `SceneSnapshotProvider`。身份必须由project/scene/source/instance/object稳定ID组成；每份snapshot携带source revision、schema/plugin catalog指纹、capture policy与coverage report；restore在隔离staging world中预检，按用户选择及依赖闭包应用为一个可撤销bulk transaction，并返回不可变receipt。

本报告登记5个P0、70个P1、12个P2、M0-M11重构路线和32个验收门。它只做review，不修改Runtime、Editor、interface或reference生产代码和tests。

## 2. 审查边界与证据

### 2.1 当前工作树物理范围

| 子域 | 文件 / 行数 / bytes | test attributes / ignored / 在途 | 审查方式 |
|---|---:|---:|---|
| Editor authoring、Play、recovery与false Diff surface | 62 / 14,280 / 509,814 | 72 / 0 / 2 | E3逐command、transaction、Play snapshot/store、recovery plan、UI document replay、Workbench route/binding/feedback |
| Runtime Session Archive | 565 / 10,510 / 360,657 | 7 / 0 / 0 | E3逐slot/capture/diff/apply/restore、archive/manifest/index/retention/selection/merge、artifact/path/writer/atomic commit |
| Runtime DynamicScene、inspection与identity | 57 / 9,686 / 336,520 | 76 / 0 / 0 | E3逐capture eligibility、entity/resource payload、validation/remap/spawn transaction、live inspection delta及reflection schema |
| Runtime focused tests | 23 / 6,659 / 245,815 | 133 / 0 / 1 | E3逐DynamicScene/Session Archive roundtrip、path atomicity、retention、slot merge、whole-level restore、exact-match diff |
| Unreal Level Snapshots参考 | 106 / 15,941 / 599,400 | 0 / 0 / 0 | E2/E3逐snapshot asset、actor/object/property/reference数据、hash/diff、selection/filter、custom serializer、restore listener与Results UI |
| Godot参考 | 3 / 8,274 / 285,876 | 0 / 0 / 0 | E2/E3逐PackedScene/SceneState node identity、owner/instance/edit state、property/group/connection及SceneTreeDock操作 |
| Fyrox参考 | 5 / 3,149 / 103,923 | 0 / 0 / 0 | E2/E3逐Editor command stack、scene graph command与reversible ownership |
| Bevy参考 | 9 / 5,979 / 212,300 | 51 / 0 / 0 | E2/E3逐Scene/ScenePatch dependency、resolve、spawn/apply lifecycle；计数含1个注释测试标记命中 |
| Unity Graphics参考 | 1 / 339 / 11,746 | 0 / 0 / 0 | E2确认`SceneObjectIDMap`是HDRP rendering object-ID utility，不是Unity Editor snapshot实现 |
| selected combined scope | 831 / 74,817 / 2,666,051 | 339 / 0 / 3 | 当前工作树fingerprint `93bb5e91acdb78f3bd0b68405121f1015ab8d5f4486424411c02365f862a5dab` |

指纹算法为：对831个选择路径按PowerShell `Sort-Object`排序，逐文件计算小写SHA-256，形成`forward/slash/path|file_sha256`行，以单个LF连接且末尾不追加LF，再对UTF-8无BOM payload计算SHA-256。选择规则包括完整Runtime Session Archive、DynamicScene scene/entity/document、inspection、Editor editing/recovery/Play snapshot、Bevy Scene源码和指定参考目录；缺失路径0，重复路径0。

读取时3个在途文件为`zircon_editor/src/core/recovery/tests/autosave_adapter.rs`、`zircon_editor/src/ui/retained_host/workbench_preview_actions.rs`和`zircon_runtime/src/scene/dynamic_scene/scene/spawn/transaction/tests.rs`，均非本报告产生。它们会影响recovery adapter、Workbench route集合和spawn transaction测试证据，实施前必须重导831文件manifest、重算指纹并复核终态。

339个test attributes主要证明DynamicScene roundtrip/spawn、Archive slot/path/retention/selection、Editor transaction/recovery基础与Bevy局部行为。它们不证明capture completeness、stable cross-session object identity、typed property diff、base/ours/theirs merge、选择性restore、Editor rollback、UI筛选或大世界性能，不能替代本报告验收门。

### 2.2 名称命中与产品边界

1. `DynamicScene`是反射驱动的可移植scene payload，不等同Editor snapshot asset。
2. `RuntimeSessionArchive`是slot容器与持久化设施，不等同Scene Snapshot library。
3. `RuntimeSessionSlotDiffReport`是exact equality与数量摘要，不等同semantic change set。
4. `RuntimeSessionArchiveMergePolicy`处理重复slot ID，不等同三方world merge policy。
5. `restore_into_level`是整World replacement，不等同可选择、可撤销的authoring restore。
6. `ScenePatch`描述spawn规模和entity remap，不描述当前World应删除、保留或修改的对象。
7. `WorldInspectionDelta`是同一live world的generation增量投影，不是可持久化snapshot diff。
8. `WorldInspectionFieldDelta`只比较单entity的component type path与field name，不是schema-stable property address。
9. `PlaySceneSource::Snapshot`是进程启动输入，不是用户管理的snapshot asset或恢复点。
10. autosave `OpenComparison`是计划枚举值，当前没有comparison executor或产品UI。
11. `UiAssetDocumentDiff`持有完整target document并整体替换，不是Scene property diff算法。
12. Source Control Diff由Editor27拥有，文件/版本库差异不能与live world snapshot diff共用虚假结果。
13. Runtime Diagnostics的“Capture Snapshot”是观测会话概念，也不能借名声明Scene Snapshot完成。
14. Godot PackedScene与Bevy ScenePatch提供scene序列化/实例化边界，不提供Zircon需要的三方authoring conflict产品。
15. Unity Graphics选集只允许验证render object-ID边界，不能推断未开源Unity Editor内部能力。

### 2.3 DynamicScene捕获的真实执行路径

1. `DynamicScene::from_world`先读取`world.node_records()`，再为每个entity遍历完整type registry。
2. component只有同时满足`is_component`、`serializable`、adapter存在且adapter报告entity持有时才捕获。
3. component payload只保留registration中逐字段标记为serializable的字段。
4. resource使用相同的`is_resource`、`serializable`、adapter与contains门。
5. plugin-owned component descriptor只为实际捕获的plugin component输出。
6. `DynamicEntity`保存`source_entity: EntityId`、`NodeRecord`和component列表。
7. `EntityId`/`NodeId`为进程内`u64`分配值；Scene lineage中可重放，但没有project/scene/instance namespace。
8. payload header、component type descriptor、entity和resource形成版本化document基础。
9. serialization strategy已有None/Value/Json/ResourceHandle/EntityReference，适合作为codec registry起点。
10. validation会检查schema/version、source entity唯一性及plugin descriptor一致性。
11. validation不会报告哪些registration、storage、field或plugin数据被跳过。
12. 无adapter、未注册storage和`serializable = false`在成功结果中不可见。
13. 捕获复杂度近似entities乘registered types，当前没有changed-set、chunk或并行capture证明。
14. snapshot没有source revision、project/scene ID、plugin catalog digest或capture policy。
15. snapshot没有object/component/property hash层级，无法hash-first缩小diff范围。
16. snapshot没有外部/内部reference table的完整性、依赖或dangling分类报告。
17. snapshot没有custom serializer版本、pre/post capture hook或restorability理由。
18. snapshot没有“必须捕获但失败即拒绝”的authoring fail-closed模式。

### 2.4 Session Archive的真实能力与误用风险

1. `RuntimeSessionSlot`只含`slot_id`、metadata和完整DynamicScene。
2. metadata含project root、asset URI、display name、created/updated时间和tags。
3. metadata没有不可变project/scene identity、base revision、schema/plugin fingerprint或coverage digest。
4. capture直接调用`DynamicScene::from_world`，继承所有静默跳过行为。
5. diff重新捕获target world，再比较完整scene equality。
6. diff report只有slot ID、matches、两侧entity/resource数量。
7. apply调用`scene.spawn_into(world)`，已有对象可能触发ID remap并形成并存副本。
8. restore-to-empty先建空World再spawn，适合灾难恢复底层，不等于增量恢复。
9. restore-into-level会替换完整World并重置runtime state，再覆盖metadata。
10. restore report只含slot ID、metadata与entity数量，没有逐项change或rollback receipt。
11. archive merge的conflict只表示slot ID重复。
12. KeepExisting/ReplaceExisting只决定整个slot clone、skip或replace。
13. merge plan会检查archive generation/revision stale，值得复用于真正merge plan的CAS边界。
14. archive拥有canonical/versioned payload与512 MiB artifact上限。
15. seal结果被缓存，manifest/statistics/index避免重复构造。
16. retention、selection、prune和preview/commit具备清晰生命周期。
17. writer有bounded keyed I/O lane，适合承载snapshot后台保存。
18. path commit有temporary/backup rename、lineage/revision/path guards。
19. 当前commit路径可见`BufWriter::flush`，但未形成文件`sync_all`与父目录fsync的完整durability证明。
20. 565文件的笛卡尔式单函数facade膨胀已由Runtime05负责硬切；Editor42只定义typed adapter，不复制wrapper树。
21. Editor/App/Hub/plugin production未发现Session Archive产品consumer。
22. 公共API名称中的`diff`、`merge`、`restore`需要限定为slot/container/whole-world语义，防止误接线。

### 2.5 Editor当前可见面与执行断点

1. 顶部Workbench toolbar公开`workbench.module.diff`。
2. template binding将其转换为`workbench.module.diff.invoke`。
3. navigation只选择`WorkbenchModuleDiff`控制状态。
4. preview action表把该route列为可调用动作。
5. module feedback对Scene返回`Scene diff prepared`。
6.第二行固定返回`Diff: scene preview state compared`。
7. 没有snapshot source selector、target world revision、provider request或artifact ID。
8. 没有Added/Modified/Removed object model或property row。
9. 没有结果选择、筛选、搜索、viewport高亮或Apply入口。
10. 默认command registry没有Scene Snapshot capture/compare/restore/merge command。
11. `EditorIntent`没有snapshot、diff、bulk restore或conflict resolution intent。
12. `EditorCommand`只覆盖Create/Delete/Update/SetReflectedSceneField等基础编辑。
13. `EditorAuthoringWorld::replace`只替换gateway并标记loaded，不提供restore编排。
14. Play Snapshot同步捕获并pretty-serialize DynamicScene，可能阻塞Editor主路径。
15. Play store通过temp、`sync_all`与rename materialize临时进程输入，并在owned root清理。
16. Play输入没有snapshot library、retention、user metadata、comparison或restore contract。
17. `RestoreFlow`能规划RestoreAutosave/DiscardAutosave/OpenComparison。
18. production中未发现执行该plan或呈现comparison的consumer。
19. autosave freshness主要依赖mtime，没有source digest/base generation/schema catalog。
20. UI Asset undo stack可用完整document target做replay，但不输出semantic change set。
21. Runtime Diagnostics固定“Capture Snapshot Session_Player_01 Actors 420 Events 1.2K”属于另一份静态观测面。
22. 这些同名surface必须分别标注owner，不允许一个固定成功文案覆盖多个缺失产品。

### 2.6 可复用的事务与live delta基础

1. Editor transaction scope具备RAII commit/rollback边界。
2. transaction保存selection snapshot并更新dirty/history/journal。
3. play mode会阻止authoring world mutation。
4. create/delete/rename/reparent/transform与Inspector field change已走统一事务。
5. 这些能力适合承载一个bulk `ApplySceneChangeSet` command，而不是逐field散发命令。
6. `WorldInspectionArtifact`按generation发布immutable snapshot。
7. hierarchy delta能表达added/changed/removed anchor并拒绝generation错配。
8. field delta能为同一entity产生稀疏UI刷新。
9. live delta只应用于结果刷新和viewport同步，不能被持久化为跨版本change authority。
10. DynamicScene spawn transaction已有preflight/current-registry generation与无失败publication阶段。
11. 真正restore应在clone/staging world复用preflight思想，并增加当前authoring revision CAS。
12. whole-level灾难恢复与选择性authoring restore必须保留不同command、权限和确认级别。

### 2.7 focused tests与缺失资格

1. DynamicScene tests覆盖component/resource roundtrip、entity remap、插件descriptor和spawn transaction。
2. Session Archive tests覆盖slot roundtrip、metadata、artifact canonical form、path save/load和stale path write。
3. retention/selection tests覆盖prune、selected protection与path mutation。
4. merge tests只覆盖重复slot ID的Reject/Keep/Replace。
5. restore tests覆盖空World和整Level替换。
6. diff tests只断言exact-match与数量摘要。
7. 没有捕获覆盖率为100%或可解释跳过的测试。
8. 没有跨保存/重载/实例化后的stable object matching测试。
9. 没有property chain、container element、component topology或reference diff测试。
10. 没有base/ours/theirs conflict matrix测试。
11. 没有选择性apply、依赖闭包、rollback、dirty/selection/viewport同步测试。
12. 没有百万对象、深引用图、取消、内存上限或增量hash性能基线。

## 3. 参考实现差异与吸收边界

### 3.1 Unreal Level Snapshots

1. `ULevelSnapshot`是持久UObject asset，保存map path、capture time、name、description和serialized world data。
2. SnapshotWorld与ApplySnapshotToWorld明确分开，apply要求`FPropertySelectionMap`。
3. DiffWorld区分matched、removed和added actors，而不是只返回整体equal。
4. actor用`FSoftObjectPath`关联原world位置，并可查询snapshot label/class。
5. `FWorldSnapshotData`保存version、class/archetype、actor map、name table、object reference table、subobject及custom serialization数据。
6. `FActorSnapshotData`保存label、class index、actor payload、component、owned subobject和actor hash。
7. hash先行比较可避免所有对象都做昂贵property deserialization/diff。
8. `FPropertySelectionMap`同时表达property选择、待respawn deleted actors、待despawn new actors和added/removed components。
9. property selection使用嵌套property chain，不依赖易漂移的leaf字符串。
10. `ICustomObjectSnapshotSerializer`允许自定义metadata/subobject、查找/重建及pre/post apply。
11. `IRestorationListener`提供whole snapshot、object properties、actor/component recreate/remove前后事件。
12. `ISnapshotFilterExtender`允许过滤前后扩展并强制加入依赖property。
13. SnapshotRestorability对actor/component/subobject/property给出可恢复policy与排除原因。
14. Results UI按Modified/Added/Removed分组，提供property row、搜索、选择/取消和过滤状态。
15. EditorData拥有active snapshot、user filter、results refresh及apply lock。
16. Zircon应吸收职责、选择集、hook与结果模型，不复制UObject、Slate或历史兼容层。
17. Unreal Level Snapshots本身不是通用三方scene merge，Zircon仍需独立定义base/ours/theirs与conflict artifact。

### 3.2 Godot PackedScene / SceneState

1. PackedScene保存SceneState并区分disabled/instance/main/main-inherited edit state。
2. SceneState保存names、variants、node paths、ID paths、editable instances和base scene state。
3. NodeData包含parent、owner、type、name、instance、index、properties和groups。
4. connections、editable instance和resource remap使scene实例化不只是一组平坦entity。
5. unique IDs与ID paths提供比裸运行时整数更强的序列化identity线索。
6. local-to-scene resource和inherited/base scene需要进入Zircon snapshot provenance及merge分类。
7. PackedScene主要解决scene保存/实例化/继承，不应被误称选择性snapshot restore产品。

### 3.3 Fyrox Editor command

1. Fyrox Editor把scene graph操作封装为可execute/revert的command。
2. command group承担多操作原子语义，适合参考Zircon bulk restore transaction。
3. graph command显式保存撤销所需旧状态，而不是apply后重新猜测。
4. Zircon已有更完整的transaction底座，应吸收command ownership与revert discipline，不新建第二undo stack。
5. 选集没有Level Snapshots等价产品，不能用command存在反推snapshot完成。

### 3.4 Bevy Scene / ScenePatch

1. Bevy Scene被定义为可组合、可patch的template definition。
2. ScenePatch显式携带source、dependencies和resolved cache。
3. dependency必须在resolve前注册/加载，spawn/apply只能消费resolved状态。
4. resolve与apply失败生命周期对Zircon restore preflight和dependency closure有直接参考价值。
5. Bevy场景ID和dynamic reflection模型仍不等同Editor stable object identity或三方conflict UI。

### 3.5 Unity Graphics边界

1. `SceneObjectIDMap`为HDRP渲染对象建立ID映射，服务GPU/rendering流程。
2. render object ID不表达scene source revision、authoring property、component ownership或merge conflict。
3. 本轮不把闭源Unity Editor能力写成已验证事实，也不以该文件作为snapshot完成度标准。

### 3.6 必须吸收与禁止照搬

必须吸收：不可变snapshot asset、qualified object identity、version/schema/reference table、actor/object/component/property层级hash、typed property selection、custom serializer/restorability/listener/filter extender、Modified/Added/Removed结果树、resolved dependency preflight、一个bulk reversible transaction及apply receipt。

禁止照搬：Unreal UObject/Slate宏层、历史deprecated payload、Godot NodePath作为唯一identity、Fyrox另一套undo栈、Bevy ECS内部ID、Unity render object ID，以及Runtime Session Archive的565文件wrapper形状。性能目标要求Zircon在数据布局、增量hash、并行capture/diff和bounded staging方面建立自己的量化基线。

## 4. 差距登记

### 4.1 P0 阻断项

1. **P0-01** 在真实`SceneDiffProvider`、source/target revision和change artifact接通前，禁用Workbench Scene Diff route及固定成功反馈；不可继续向用户宣称比较已经完成。
2. **P0-02** 禁止Editor直接调用`RuntimeSessionSlot::restore_into_level`或等价整World replacement；未捕获状态、未保存编辑、selection、dirty、打开文档和viewport必须受事务与恢复策略保护。
3. **P0-03** 禁止把DynamicScene整体equality/数量摘要或Archive重复slot-ID merge标注为Scene Diff、World Merge或Conflict Resolution；公开API和UI必须限定真实语义。
4. **P0-04** 在`CaptureCoverageReport`、required-state fail-closed policy、schema/plugin catalog fingerprint和unsupported diagnostics完成前，snapshot不得取得可恢复产品资格。
5. **P0-05** 在qualified stable identity、source revision CAS、staging preflight、单一bulk transaction、rollback和immutable receipt完成前，选择性Restore/Merge Apply入口保持不可用。

### 4.2 P1 工程化必做

#### 4.2.1 Snapshot asset、capture policy与coverage

1. **P1-01** 建立版本化`SceneSnapshotAsset`与独立snapshot ID，不再直接把archive slot当authoring asset。
2. **P1-02** 保存project ID、scene asset ID、world/source revision、branch/source descriptor和capture generation。
3. **P1-03** 保存schema catalog fingerprint、plugin set digest、engine build/version和custom codec versions。
4. **P1-04** 建立`SceneSnapshotCapturePolicy`，明确scope、include/exclude、transient、editor-only、runtime-only及external dependency规则。
5. **P1-05** 为component/resource/field输出Captured/Skipped/Unsupported/Failed状态与reason code。
6. **P1-06** 支持required authoring state清单，任何required项缺失时capture fail closed。
7. **P1-07** 建立custom snapshot serializer registry及其version、capability、threading和failure contract。
8. **P1-08** 为capture listener提供pre/post world、object和custom subobject hook。
9. **P1-09** 将capture变为Editor09拥有的可取消background job，具备progress、memory和shutdown drain。
10. **P1-10** 建立immutable capture receipt，记录policy、coverage、diagnostics、duration、bytes及source revision。
11. **P1-11** 实现一致性屏障，避免capture读取一半旧generation和一半新generation。
12. **P1-12** 按changed generation、archetype/type和chunk并行capture，消除entities乘types的无界扫描。
13. **P1-13** 对大value、bulk data和external asset使用content-addressed引用而非重复内联。
14. **P1-14** 明确snapshot retention、pin、delete、GC、orphan artifact和project move策略。

#### 4.2.2 Stable identity、schema、reference与provenance

15. **P1-15** 建立`QualifiedSceneObjectId { project, scene, source, instance, local_object }`。
16. **P1-16** 为component instance建立stable ID，区分同类型多component、replace和recreate。
17. **P1-17** 以stable schema field ID与nested property segment替换leaf field-name authority。
18. **P1-18** 支持array/map/set element identity、index move和key change，不把容器整体降格成JSON blob。
19. **P1-19** 建立内部对象、subobject、外部asset和soft/unloaded reference table。
20. **P1-20** 捕获parent、owner、order、folder、level/data-layer/instance provenance，而非只保存runtime parent。
21. **P1-21** 与Editor41的Level Instance/Prefab stable source/instance/local object ID合同共用身份层。
22. **P1-22** 建立schema migration registry，区分renamed/moved/converted/removed/incompatible field。
23. **P1-23** 为missing type/plugin/codec提供可诊断deferred resolution，不静默丢弃payload。
24. **P1-24** 建立object/component/property/reference分层hash及算法/version字段。
25. **P1-25** 明确identity exact match为authority；名称、类型、空间接近等heuristic只能生成待用户确认的remap suggestion。

#### 4.2.3 Semantic Scene Diff

26. **P1-26** 建立不可变`SceneChangeSet`及source/target snapshot/world revision。
27. **P1-27** 表达object add/remove、rename、reparent、reorder和owner/provenance变化。
28. **P1-28** 表达component add/remove/replace/reorder及schema/type变化。
29. **P1-29** 表达typed property old/new value、nested path、codec和comparison policy。
30. **P1-30** 表达internal/external reference retarget、dangling、dependency add/remove和unresolved。
31. **P1-31** 实现hash-first、object-second、property-third的分层diff pipeline。
32. **P1-32** 为float、transform、color、asset reference、collection和custom type注册comparison policy。
33. **P1-33** 区分authoring semantic change、derived/transient noise、schema migration和unsupported comparison。
34. **P1-34** 为deleted/added对象保留label、type、owner path和可恢复性，即使对象当前未加载。
35. **P1-35** 输出stable change ID，支持选择、评论、缓存、重算和receipt引用。
36. **P1-36** 为filter/search/grouping建立独立query projection，不修改canonical change set。
37. **P1-37** 支持snapshot-vs-live、snapshot-vs-snapshot和revision-vs-live三种明确模式。
38. **P1-38** 对same-source stale world执行revision检查并标记comparison freshness。

#### 4.2.4 Three-way Merge与conflict model

39. **P1-39** 建立显式`SceneMergeInput { base, ours, theirs }`，禁止隐式猜测base。
40. **P1-40** 定义object add/add、delete/edit、rename/rename和reparent/reparent冲突。
41. **P1-41** 定义component topology、same-property、container edit和reference retarget冲突。
42. **P1-42** 定义schema/plugin/codec/external asset unavailable冲突。
43. **P1-43** 区分auto-merged、requires-choice、blocked、unsupported和stale结果。
44. **P1-44** 为每类conflict保存base/ours/theirs typed value及来源revision。
45. **P1-45** 支持Use Base/Use Ours/Use Theirs/Manual/Custom Resolver选择，并验证类型与依赖。
46. **P1-46** 建立custom merge resolver registry及determinism、version、diagnostic、timeout合同。
47. **P1-47** resolution plan必须可序列化、可审阅、可重放并绑定input digests。
48. **P1-48** merge commit前重新验证base、ours、theirs及当前world revision，stale即拒绝。
49. **P1-49** 让Scene merge与Editor27 VCS file merge互相链接但保持不同artifact和provider authority。
50. **P1-50** 将Archive merge API限定或重命名为slot-container merge，消除同名误用。

#### 4.2.5 Restore planning、transaction与rollback

51. **P1-51** 建立`SceneRestorePlan`，输入snapshot/change selection/current authoring revision与policy。
52. **P1-52** 根据selected changes计算dependency closure、required object/component及reference repair。
53. **P1-53** 在隔离clone/staging world执行resolve、apply、validation和invariant check。
54. **P1-54** 输出would-add/remove/change、unsupported、dangling、memory/time estimate和destructive warning。
55. **P1-55** 用单一`ApplySceneChangeSet` bulk command接入现有Editor transaction engine。
56. **P1-56** transaction保存足够逆操作或pre-apply snapshot，任一失败全量rollback。
57. **P1-57** apply以expected authoring revision做CAS，拒绝计划生成后的并发修改。
58. **P1-58** 恢复后统一更新dirty、undo/redo、journal、selection、focus和notification。
59. **P1-59** 同步Hierarchy/Inspector/viewport/toolkit/open document projection及generation。
60. **P1-60** 为actor/component create/remove及custom serializer调用pre/post restoration listener。
61. **P1-61** 输出immutable restore receipt，列出每个change结果、remap、diagnostic、revision和rollback状态。
62. **P1-62** 将whole-level disaster restore定义为独立高风险command，要求未保存状态处理、明确确认和reopen/resync流程。

#### 4.2.6 Editor product、jobs与集成

63. **P1-63** 建立`SceneSnapshotProvider`与capability descriptor；无provider时命令必须disabled并解释原因。
64. **P1-64** 实现snapshot browser、capture metadata、source revision、coverage和retention状态。
65. **P1-65** 实现Added/Modified/Removed/Conflict结果树、property row、搜索、筛选、排序和展开状态。
66. **P1-66** 提供change checkbox/tri-state selection、依赖自动选择与不可恢复理由。
67. **P1-67** 将结果选择映射到World Outliner与viewport highlight，卸载对象显示descriptor而非消失。
68. **P1-68** capture/diff/merge/preflight走Editor09 job admission、cancel、progress、quota、result retention和shutdown drain。
69. **P1-69** 复用Runtime Archive storage时只通过typed adapter和bounded writer，禁止Editor调用565文件facade的任意组合函数。
70. **P1-70** 删除固定Scene Diff与diagnostic snapshot成功文案，增加command/provider/artifact/receipt全链telemetry和错误路由。

### 4.3 P2 纵深能力

1. **P2-01** 为snapshot提供chunked binary、compression、content dedup和增量artifact storage。
2. **P2-02** 支持跨分支、跨项目重定位与显式identity remap package。
3. **P2-03** 支持团队共享snapshot、review comment、approval与访问控制。
4. **P2-04** 提供viewport before/after ghost、heatmap和变化轨迹可视化。
5. **P2-05** 提供脚本/automation API，但仍受provider、revision和transaction gate约束。
6. **P2-06** 支持领域custom conflict resolver、批量rule与可审计自动resolution。
7. **P2-07** 建立snapshot retention tier、pin、quota、GC和remote archive策略。
8. **P2-08** 提供change-set/merge/restore receipt的HTML/JSON导出与审计报告。
9. **P2-09** 建立capture/diff/merge/apply分阶段CPU、内存、I/O与cache telemetry。
10. **P2-10** 建立plugin serializer/restorability/merge resolver兼容矩阵与认证套件。
11. **P2-11** 支持大世界partition-aware lazy diff、streaming descriptor和局部restore。
12. **P2-12** 在真实项目规模建立并维护优于参考引擎的snapshot/diff/apply性能基线。

## 5. 目标架构

### 5.1 Snapshot artifact

```text
SceneSnapshotAsset
  identity: SnapshotIdentity
  source: SceneSourceRevision
  schema_catalog: SchemaCatalogFingerprint
  plugin_catalog: PluginCatalogFingerprint
  capture_policy: SceneSnapshotCapturePolicy
  coverage: CaptureCoverageReport
  objects: ChunkIndex<SnapshotObjectRecord>
  references: SnapshotReferenceTable
  dependencies: SnapshotDependencyManifest
  hashes: SnapshotHashTree
  diagnostics: DiagnosticSet
```

`SnapshotIdentity`必须包含snapshot/project/scene/source/instance namespace；`SnapshotObjectRecord`包含stable object/component identity、parent/owner/order/provenance和typed property payload。artifact immutable，用户metadata修改通过独立catalog记录，不回写内容digest。

### 5.2 Diff与merge artifacts

```text
SceneDiffRequest -> SceneDiffService -> SceneChangeSet
                                         |
base + ours + theirs -> SceneMergeService -> SceneMergeResult
                                              |
                                      ConflictResolutionPlan
```

`SceneChangeSet`只保存语义变化和稳定change ID；UI filter是projection。`SceneMergeResult`必须携带三份input digest、auto-merge change、conflict、unsupported项和freshness。Archive slot merge不进入此图。

### 5.3 Restore transaction

```text
selected changes
  -> dependency closure
  -> staging-world resolve/apply/validate
  -> revision CAS
  -> one Editor bulk transaction
  -> projection resync
  -> immutable receipt
```

任何阶段失败都不能修改authoring world。custom serializer和listener在plan与commit阶段有明确调用次数和线程契约。灾难恢复走独立whole-world flow，不能复用选择性Apply按钮。

### 5.4 Ownership

| Owner | 责任 | 禁止承担 |
|---|---|---|
| Runtime Scene/Reflection | typed capture codec、stable schema、reference resolve、staging apply primitive | Editor UI、用户selection、undo history |
| Runtime Archive/Storage | canonical artifact、bounded writer、path CAS、retention primitive | semantic diff、三方merge、Editor whole-world replacement |
| Editor Snapshot domain | catalog/provider、capture/diff/merge request、results projection、restore transaction | 复制Runtime serializer或另建undo stack |
| Editor Transaction | atomic bulk command、dirty/history/journal/selection/rollback | 决定schema migration或custom merge语义 |
| Source Control | repository revision/file diff/changelist integration | 冒充live world semantic merge |
| Plugin SDK | serializer/restorability/filter/listener/merge resolver extension | 绕过capability、revision、budget和diagnostic gate |

## 6. 分阶段重构路线

### M0 - 真实性封口

禁用Workbench Scene Diff固定成功面；给Archive diff/merge/restore API增加真实语义限定；禁止Editor接入whole-level restore。

### M1 - 领域所有权与合同

冻结Snapshot/Diff/Merge/Restore/Conflict五域术语、owner、stable identity、revision、capability和error taxonomy。

### M2 - Capture policy与coverage

实现required-state fail-closed、coverage report、custom serializer、listener、consistency barrier和可取消capture job。

### M3 - Artifact、reference、hash与migration

完成qualified identity、schema/plugin fingerprint、reference/dependency table、分层hash、migration与content-addressed bulk data。

### M4 - Semantic diff

实现object/component/property/reference change set、hash-first pipeline、comparison policy、freshness和stable change ID。

### M5 - Results projection

实现Modified/Added/Removed tree、query/filter/search、selection dependency、Outliner/viewport同步和unloaded descriptor。

### M6 - Three-way merge

实现base/ours/theirs、conflict taxonomy、custom resolver、serializable resolution plan、input digest和stale rejection。

### M7 - Staged restore transaction

实现dependency closure、staging world、preflight report、revision CAS、single bulk command、rollback、projection resync和receipt。

### M8 - Extension与restorability

完成custom subobject recreate、property dependency、restorability reason、pre/post listener和plugin compatibility suite。

### M9 - Storage与规模化

通过typed adapter复用Archive artifact/writer，补齐durability、chunk/dedup、quota/retention，并建立大世界性能基线。

### M10 - 产品集成

接通snapshot browser、capture/compare/merge/restore UI、Editor09 jobs、Editor10 notifications、Editor27 VCS handoff和Editor41 Outliner identity。

### M11 - 资格与硬切

通过32项门，删除固定feedback、ambiguous Archive API和任何第二序列化/undo authority；发布迁移说明与运行手册。

## 7. 验收门

- [ ] 1. Scene Diff command在无provider时disabled，且不再显示固定成功文字。
- [ ] 2. Snapshot asset含project/scene/source revision、schema/plugin fingerprint、policy和coverage。
- [ ] 3. required component/resource/field缺失会使capture失败并给出稳定reason code。
- [ ] 4. 未注册storage、无adapter和不可序列化字段均出现在coverage报告。
- [ ] 5. 同一scene保存、重载、实例化后stable object/component identity仍能匹配。
- [ ] 6. 名称或transform heuristic不会自动取得identity authority。
- [ ] 7. 内部、subobject、外部asset和unloaded reference可无损roundtrip。
- [ ] 8. schema rename/move/convert/remove均有迁移或明确blocked结果。
- [ ] 9. diff区分object add/remove/rename/reparent/reorder。
- [ ] 10. diff区分component add/remove/replace和property/reference change。
- [ ] 11. nested property及array/map/set element具有稳定typed address。
- [ ] 12. hash-first diff与全量语义比较结果一致。
- [ ] 13. snapshot-vs-live、snapshot-vs-snapshot和revision-vs-live模式来源清晰。
- [ ] 14. stale comparison被标记，stale apply被拒绝。
- [ ] 15. merge强制提供base/ours/theirs并绑定三份digest。
- [ ] 16. delete/edit、add/add、reparent/reparent和same-property冲突有确定分类。
- [ ] 17. schema/plugin/codec缺失不会被自动解析为删除或默认值。
- [ ] 18. resolution plan可序列化重放，input变化后拒绝commit。
- [ ] 19. selected changes会自动计算依赖闭包并解释强制选择项。
- [ ] 20. restore先在隔离staging world完成resolve/apply/validation。
- [ ] 21. restore以current authoring revision做CAS。
- [ ] 22. apply只提交一个Editor bulk transaction，undo一次即可完整恢复。
- [ ] 23. 任一custom serializer/listener失败时world、dirty和history均不变。
- [ ] 24. 成功apply同步Hierarchy、Inspector、viewport、selection和打开toolkit。
- [ ] 25. receipt逐项记录applied/skipped/blocked/remap/diagnostic及前后revision。
- [ ] 26. whole-level disaster restore与选择性restore是不同command和确认流程。
- [ ] 27. Archive slot merge的UI/API明确标注container语义，不出现scene conflict暗示。
- [ ] 28. capture/diff/merge/preflight支持cancel、bounded memory、quota和shutdown drain。
- [ ] 29. snapshot artifact写入具备文件及父目录durability证明和crash fault test。
- [ ] 30. 10万对象、100万property、深引用图的capture/diff/apply满足预算且结果确定。
- [ ] 31. plugin serializer/filter/listener/resolver通过版本、失败、线程和卸载兼容矩阵。
- [ ] 32. 默认产品UI、command、automation、docs和telemetry只报告真实provider artifact/receipt状态。

## 8. 风险、依赖与实施顺序

1. Editor02拥有transaction/save/autosave/recovery；本轮不得建立第二套dirty、undo或recovery authority。
2. Runtime04/Runtime05拥有reflection/DynamicScene/Session Archive结构治理；snapshot产品通过typed接口提出底层需求。
3. Editor09拥有background job admission、progress、cancel、quota和shutdown；长任务不可自建线程池。
4. Editor27拥有VCS revision/file diff；Scene semantic change set只做artifact handoff。
5. Editor41拥有Outliner、Level Instance与stable source/instance identity；两轮必须共享ID和unloaded descriptor。
6. Play Snapshot和autosave recovery先保持原语义，待新artifact稳定后通过adapter迁移，不能直接改名收编。
7. DynamicScene capture与spawn transaction测试文件均有在途变化，实施前必须复核其generation和rollback终态。
8. 565文件Session Archive facade已显著超出合理导航成本；Editor产品只能依赖少量稳定service trait。
9. “优于Unreal”的性能结论只能由同规模、同数据完整度、同durability条件下的benchmark与profile证明。

推荐依赖顺序为：M0真实性封口 -> M1身份/合同 -> M2/M3完整snapshot artifact -> M4 semantic diff -> M5结果UI -> M6三方merge -> M7选择性restore -> M8扩展 -> M9规模化 -> M10产品接线 -> M11硬切。缺少M2/M3时不得并行开放restore；缺少M4时不得把equality摘要展示为diff；缺少M6时不得把VCS或slot merge包装成conflict resolution。

## 9. 验证说明

本轮只修改review文档和索引，没有修改production Runtime、Editor、interface、plugin、App、Hub代码或tests，也没有运行动态测试。此前相同工作树的`zircon_editor --lib`测试编译在617.2秒后被239个既有错误和122个warning阻断，本轮没有重复无法抵达Scene Snapshot产品行为的同一lane。

静态验证必须确认：frontmatter路径存在；P0/P1/P2数量分别为5/70/12；M0-M11完整；验收门为32；831文件指纹与3个在途文件准确；Markdown无trailing whitespace、占位标记或断链；根索引、Editor索引和coverage队列同步。动态资格在实施阶段至少需要capture completeness、roundtrip/migration、semantic diff golden、three-way conflict matrix、fault-injected rollback/durability、Editor transaction/selection/projection集成和大规模benchmark。
