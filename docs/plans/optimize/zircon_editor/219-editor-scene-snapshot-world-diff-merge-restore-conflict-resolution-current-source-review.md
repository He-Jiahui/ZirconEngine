---
title: Editor Scene Snapshot、World Diff、Merge、Restore 与 Conflict Resolution 当前源码复审
category: zircon_editor
report_id: Editor219
review_date: 2026-08-29
baseline_head: f660cfa9f3f84bff0903e4564ff1af4d065aee73
verification_head: f660cfa9f3f84bff0903e4564ff1af4d065aee73
canonical_owner: Editor42
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/42-scene-snapshot-world-diff-merge-restore-conflict-resolution-authoring-review.md
  - docs/plans/optimize/zircon_editor/116-editor-scene-snapshot-world-diff-merge-restore-conflict-resolution-current-source-review.md
  - docs/plans/optimize/zircon_editor/163-editor-scene-snapshot-world-diff-merge-restore-conflict-resolution-current-source-review.md
related_runtime_owners:
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/40-save-game-checkpoint-slot-participant-capture-serialization-migration-platform-cloud-async-network-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/99u-runtime-dynamic-scene-asset-reload-event-generation-reconciliation-stage-apply-instance-replacement-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99v-runtime-dynamic-scene-session-archive-slot-capture-restore-path-merge-retention-durability-product-integration-current-source-review.md
related_editor_owners:
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/27-project-operations-source-control-changelist-diff-automation-report-submit-gates-health-dashboard-review.md
  - docs/plans/optimize/zircon_editor/218-editor-level-variant-data-layer-level-instance-world-outliner-current-source-review.md
related_code:
  - zircon_editor/assets/ui/editor/components/workbench/shell/workbench_top_toolbar.zui
  - zircon_editor/src/core/commands/defaults.rs
  - zircon_editor/src/core/editing
  - zircon_editor/src/core/play/snapshot
  - zircon_editor/src/core/recovery
  - zircon_editor/src/ui/asset_editor/document_diff.rs
  - zircon_editor/src/ui/asset_editor/undo_stack.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/template_bridge/workbench/module_command_feedback.rs
  - zircon_editor/src/ui/retained_host/workbench_preview_actions.rs
  - zircon_runtime/src/scene/dynamic_scene/patch.rs
  - zircon_runtime/src/scene/dynamic_scene/scene
  - zircon_runtime/src/scene/dynamic_scene/session
  - zircon_runtime/src/scene/inspection
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime_interface/src/reflect
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
doc_type: current-source-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# Editor219 · Scene Snapshot / World Diff / Merge / Restore / Conflict Resolution 当前源码复审

## 1. 结论

Editor163 之后仍没有形成工程级 Scene Snapshot、语义 World Diff、三方 Scene Merge、选择性 Restore 或 Conflict Resolution 产品。对四个 production roots 的 16,908 个已跟踪和 2,368 个未跟踪物理文件执行精确合同扫描，SceneSnapshotAsset、SceneChangeSet、SceneDiffRequest、SceneMergeService、SceneRestoreCoordinator、ConflictPlan、CaptureCoverage、PropertySelectionMap、SnapshotArtifact、ApplySceneChangeSet、SceneMergeRequest、SceneRestoreReceipt 均为零命中。

Workbench 仍暴露 workbench.module.diff 路由，最终只写入 Scene diff prepared 与 Diff: scene preview state compared 固定文字；默认 command registry 没有 Scene Snapshot、Diff、Merge 或 Restore 领域命令。没有 provider、source/target revision、artifact、change set、selection、preflight、commit receipt，却显示 prepared/compared，P0-01 因此继续是虚假成功面。

Runtime 的真实进展仍是通用底座而不是本域产品。DynamicScene capture 只遍历已注册、声明 serializable、具备 adapter 且 contains 为真的 component/resource，再过滤不可序列化 field；所有跳过仍没有 coverage 或 fail-closed 结果。ScenePatch 只是包裹完整 DynamicScene，preview 预览 spawn remap，apply 调用 additive spawn_into。compiled spawn 的 target fence、staged preflight 和 validation 提升了追加生成的事务安全，但没有 object/property change identity、remove/modify/reparent、base/ours/theirs 或选择性 restore 语义。

Session Archive 的 diff 仍是完整 scene equality 加 entity/resource 数量；merge 仍只按 slot_id 执行 Reject/Keep/Replace；apply 仍追加 spawn；restore 仍从空 World 生成后整体替换 Level World，再单独设置 metadata。Archive writer 具备 bounded keyed admission、进程内 path generation/commit authority、staged atomic publish、file/parent sync 与 stale artifact 检查，这是唯一 Partial；该 authority 仍是进程内 OnceLock/Mutex 状态，没有 typed Snapshot store、持久或跨进程 CAS、journal/startup reconciliation 与本域 crash qualification。

Recovery 的 Restore/OpenComparison 仍只把 autosave 安全物化为 .zircon/recovered 下的副本，UiAssetDocumentDiff 仍保存完整 target 并整体替换。它们与 Editor transaction engine 都是可复用基础设施，但不是 Scene semantic comparison 或 selective authoring restore。

Editor42 保持唯一 canonical owner；状态仍为：**P0：5 Open；P1：69 Open / 1 Partial；P2：12 Open；Gates：31 Fail / 1 Partial / 0 Pass**。当前没有同内容完整度、同 durability、同平台、同硬件与同故障语义的动态 benchmark，不能声称本域达到、接近或优于 Unreal。

## 2. 审查范围、统计与 currentness

统计读取当前 working-tree 物理内容，包含相关未跟踪文件。行数为物理行；tests/ignored 只统计精确 Rust test/ignore attribute。fingerprint 按 lowercase repository-relative path 排序，将路径与文件 SHA-256 组成清单后再次取 SHA-256。选择集用于复现本报告，不代表参考仓库全部规模。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | fingerprint |
|---|---:|---|
| Editor authoring / recovery / surface | **144 / 23,970 / 21,949 / 840,937 / 118 / 0** | 85be3edf78e2af8b617efb840216412c2a3c8ad9951e60a95b74b1cd98d1eb49 |
| Runtime Session Archive | **578 / 13,091 / 11,686 / 450,877 / 52 / 0** | afeca7ac5a4f03f34c33418c7e4fef195376ab4498b93ace5c5ee060461b1124 |
| DynamicScene / inspection / reflection support | **54 / 11,014 / 9,960 / 381,425 / 94 / 0** | 386b8f9e4cf403946c46ce4ebcff2803bcb59d616df6e0eeb840d1ad5e68bed0 |
| Zircon selected union | **776 / 48,075 / 43,595 / 1,673,239 / 264 / 0** | ab46169504ef665647762fd031b89ca8b7fd22637f6bb013fd92b3149adf80fd |
| Unreal / Godot / Fyrox / Bevy / Unity Graphics selected | **27 / 17,876 / 15,080 / 639,373 / 1 / 0** | 5e2b41fe1a51fd3fa7e33450c46363c539c1d30a19789913890a2cd209cd259c |
| All selected | **803 / 65,951 / 58,675 / 2,312,612 / 265 / 0** | 78d96ac9a491f4993fda04c26f4c80d78430e129fe1dbda3a6d95146fe6f1483 |

- baseline 与 verification HEAD 均为 f660cfa9f3f84bff0903e4564ff1af4d065aee73；共享工作树包含大量在途修改，本报告读取物理文件，不回退、不覆盖，也不把未完成修改写成已集成能力。
- 四个 production roots 为 zircon_editor、zircon_runtime、zircon_runtime_interface、zircon_plugins，共 19,276 个物理 Rust/TOML/ZUI/Zr 文件；上述十二个精确产品合同全部零命中。
- Editor 集合覆盖 toolbar、default commands、editing、Play snapshot、recovery、document diff/undo、module feedback 与 preview action；Runtime 集合覆盖完整 Session owner、DynamicScene scene/document/patch、inspection、level system 与 reflection。
- 参考集合按 frontmatter 的入口展开为 27 个物理文件；Unreal Level Snapshots 是主产品基线，其余引擎只承担明确的局部架构参照或负边界。
- 按用户要求未查询、轮询、等待或实时跟踪协调器；Tooling 按用户要求排除。
- 本轮只修改 review 与索引，没有修改 production、tests、Cargo、ABI 或参考源码，也没有运行 Cargo、Editor、动态 restore、fault、scale、soak、跨平台或跨引擎 benchmark。

## 3. 当前源码事实与语义边界

### 3.1 DynamicScene capture 仍不是完整 Snapshot

1. DynamicScene::from_world 遍历 World node records，并只捕获注册表中满足 component/resource、serializable、adapter、contains 四层条件的对象。
2. serializable_fields 再按 metadata 过滤字段；未注册 storage、缺 adapter、不可序列化 type/field 以及 adapter 未覆盖的状态均不进入 structured coverage，也不阻止成功。
3. ensure_scene_supported 只验证 schema/version、component descriptor 唯一性和 source entity 唯一性；新增 HashSet 优化与 descriptor validation 不等于 capture completeness。
4. payload header、migration、entity remap、compiled spawn target fence、preflight 与 resource/component write validation 是应保留的底座，但仍没有 authoring stable identity、source revision、capture policy、coverage 或 dependency manifest。
5. EntityId 仍是 u64，没有 project/scene/source/instance namespace；display name/path 不能承担跨保存 identity。
6. Play snapshot 仍是子进程输入与生命周期临时文件，不是持久、可索引、可审计的 SceneSnapshotAsset。

### 3.2 ScenePatch 与 Session Archive 仍不是语义 Diff/Merge

1. ScenePatch 只含一个 DynamicScene；from_world 捕获整场，preview_apply 返回 spawn 数量、descriptor/resource 状态和 remap，apply 直接调用 spawn_into。
2. ScenePatch 没有 operation list、before/after digest、remove/modify/reparent/reorder、typed property address、dependency closure、inverse 或 receipt，因此名称不得被 UI 解释成 authoring change set。
3. compiled spawn 会在 preflight World 中验证 component/resource writes，并在 target generation 变化时拒绝 stale plan；这提高 additive spawn 的失败隔离，不会把它变成 selective restore。
4. RuntimeSessionSlotDiffReport 只有 matches、slot/target entity count 和 resource count；没有 added/removed/modified 分类或稳定匹配。
5. RuntimeSessionArchiveMergePlan 虽绑定 target generation/revision 并支持 preview/commit，冲突域仍只是重复 slot_id；RejectConflicts、KeepExisting、ReplaceExisting 不是 base/ours/theirs Scene conflict policy。
6. apply_to_world 是追加 spawn，restore_into_level 是 whole-world replacement 后另设 metadata；两者都不得成为 Editor selective restore 的默认实现。

### 3.3 Durability、Recovery 与 Editor transaction 的正确归属

1. Archive path authority 通过 ResolvedProjectPathIdentity、write generation、expected commit 和 artifact revision 阻止同进程 stale write，并复用 stage_atomic_write 发布。
2. authority 只存在于本进程 OnceLock/Mutex/Weak map；重启或第二进程看不到 generation/commit，且没有 durable journal、lease、startup repair 或 typed SceneSnapshot store。
3. RestoreExecutor 校验 candidate 位于其 autosave document directory，并以 atomic_write_new 输出 restore/comparison copy，不覆盖 authoritative source；这是 disaster recovery，不是 semantic restore。
4. OpenComparison 当前只生成 comparison copy，没有 comparison provider、semantic result tree、property selection 或 apply plan。
5. UiAssetDocumentDiff 的变化模型仍是 Option<Arc<完整 target document>>；apply 直接整体替换，不能外推为 Scene property diff。
6. Editor transaction engine 的 RAII/nested transaction、apply/revert/finalize、rollback、selection、dirty/history/journal 应成为未来 ApplySceneChangeSet 的唯一提交 authority，本域不得另建第二 undo/dirty stack。

## 4. 参考引擎差异

1. Unreal ULevelSnapshot 是可索引资产，明确分离 SnapshotWorld、DiffWorld、ApplySnapshotToWorld；FPropertySelectionMap 单独记录属性选择、actor respawn/despawn、component add/remove 与 custom subobject restore。
2. Unreal FWorldSnapshotData 保存 snapshot version、class/archetype、actor path、name/reference tables、subobject 与 custom serialization；FActorSnapshotData 保存 label、class、serialized/custom/component/owned-subobject data 与 hash。
3. Unreal HasChangedSinceSnapshotWasTaken 先比较 hash 再比较属性；custom serializer、restoration listener、filter extender、restorability reason 与 Results owner 形成可扩展产品面。Zircon 当前只有 whole equality、无选择模型和固定反馈。
4. Godot SceneState 保存 node path、owner、type、name、instance、editable instance、property/group/connection 与 scene unique identity；pack/instantiate 和 SceneTree UndoRedo 展示 ownership 与可逆编辑边界。
5. Fyrox Command 的 execute/revert/finalize、CommandGroup 的正序执行/逆序撤销，以及 graph Handle/Ticket/SubGraph 的 add/delete/replace/reparent ownership，约束 Zircon restore 必须形成单次可逆 Editor transaction。
6. Bevy ScenePatch 要求 dependency 注册/加载、resolve 后 spawn/apply，并在 ResolvedScene apply 失败时清理中间 entity；它可作为生命周期与失败清理参照，不是三方 authoring merge 产品。
7. Unity Graphics SceneObjectIDMap 只是 scene-scoped rendering utility，把 GameObject 映射到 compact category/object ID；它不能替代 qualified authoring identity，也不能证明 Unity Editor Snapshot 的内部语义。

## 5. 差距清单

### 5.1 P0：5 Open

1. **P0-01 · Open** 禁用或明确标记 toolbar 的固定 Scene Diff 成功文案；无 provider/artifact 时不得显示 native Diff。
2. **P0-02 · Open** 捕获若跳过 required component/resource/field，必须 fail closed 或发布 coverage report；不得把不完整 payload 当成功 snapshot。
3. **P0-03 · Open** 禁止把 whole equality、slot-ID merge、additive ScenePatch/spawn 或 whole-level replacement 接为 Editor semantic diff/merge/restore。
4. **P0-04 · Open** 在 stable identity、source revision、typed change set、preflight、atomic rollback 和 receipt 前，不得修改 authoring World 或共享 asset。
5. **P0-05 · Open** Play snapshot、autosave restore、UI document diff、diagnostic capture 必须与 Scene Snapshot 分离，消除第二 authority 和数据丢失路径。

### 5.2 P1：69 Open / 1 Partial

1. **P1-01 · Open** 定义 project/scene/source/instance/snapshot qualified identity。
2. **P1-02 · Open** 为 object/component/property 分配跨保存稳定 ID。
3. **P1-03 · Open** 定义 source revision、schema revision、plugin catalog fingerprint。
4. **P1-04 · Open** 建立 source-object 与 instance-object provenance map。
5. **P1-05 · Open** 建立 typed property address 与 collection selector。
6. **P1-06 · Open** 建立 property schema fingerprint 和 migration contract。
7. **P1-07 · Open** 建立 typed reference/dependency table 及 dangling 分类。
8. **P1-08 · Open** 定义 capture policy、required state 与 plugin serializer registry。
9. **P1-09 · Open** 定义 owner/generation/request ID 传播规则。
10. **P1-10 · Open** 用 lint/test 分离 display path 与 stable identity。
11. **P1-11 · Open** 实现 immutable SceneSnapshotAsset source/catalog metadata。
12. **P1-12 · Open** 实现 chunked object/component/property payload 与 content digest。
13. **P1-13 · Open** 实现 capture consistency barrier，禁止跨 frame 混合 World。
14. **P1-14 · Open** 输出 captured/skipped/failed coverage report。
15. **P1-15 · Open** required-state 缺失时执行 fail-closed policy。
16. **P1-16 · Open** 为 custom serializer 提供 version、restorability reason 和 hooks。
17. **P1-17 · Open** 建立 reference closure、external asset readiness 和 orphan report。
18. **P1-18 · Open** 建立 schema/plugin migration、unknown field 与 codec fallback policy。
19. **P1-19 · Open** 为 capture job 接入 bounded memory、cancel、progress 和 shutdown drain。
20. **P1-20 · Partial** Session writer 已复用共享 atomic file primitive、同进程 path generation/commit authority 与 file/parent sync；仍需 Scene Snapshot typed store、persistent/cross-process CAS、journal/startup recovery 与本域 fault qualification。
21. **P1-21 · Open** 实现 object/component/property/reference 分层 hash tree。
22. **P1-22 · Open** 定义 SceneDiffRequest source、target、policy 和 freshness。
23. **P1-23 · Open** 实现 hash-first semantic diff，不能只比较完整 equality。
24. **P1-24 · Open** 区分 added/removed/modified/renamed/reparented/reordered object。
25. **P1-25 · Open** 区分 component topology、property、reference 与 asset changes。
26. **P1-26 · Open** 为 nested property、array/map/set element 提供稳定 change ID。
27. **P1-27 · Open** 支持 snapshot-vs-live、snapshot-vs-snapshot、revision-vs-live 三种模式。
28. **P1-28 · Open** stale source/target 必须阻断 Apply 并给出诊断。
29. **P1-29 · Open** 计算 selected change 的 dependency closure 并解释强制项。
30. **P1-30 · Open** 为 change set 提供 deterministic ordering、serialization 和 replay。
31. **P1-31 · Open** 实现 Results model、tree、search、filter、sort 和 hidden dependency 显示。
32. **P1-32 · Open** 同步 Outliner、Inspector、viewport highlight 和 selection revision。
33. **P1-33 · Open** 建立 base/ours/theirs 三方 Merge request/artifact。
34. **P1-34 · Open** 强制绑定三份 input digest、schema/plugin fingerprint。
35. **P1-35 · Open** 分类 delete/edit、add/add、reparent/reparent、same-property 冲突。
36. **P1-36 · Open** schema/plugin/codec 缺失时输出 blocked/unsupported，不自动当删除。
37. **P1-37 · Open** 实现 typed custom conflict resolver extension。
38. **P1-38 · Open** 生成可序列化、可审计、input-qualified resolution plan。
39. **P1-39 · Open** resolution plan 的 input 变化必须拒绝 commit。
40. **P1-40 · Open** 实现 SceneRestoreCoordinator staging-world resolve/apply/validate。
41. **P1-41 · Open** restore 前执行 dependency closure、ownership、permission 和 editability preflight。
42. **P1-42 · Open** 以 current authoring revision 做 CAS。
43. **P1-43 · Open** 选择性 restore 只提交一个可撤销 Editor bulk transaction。
44. **P1-44 · Open** 任一 serializer/listener 失败时保证 world/dirty/history 不变。
45. **P1-45 · Open** 成功 restore 后发布 hierarchy/inspector/viewport/selection resync。
46. **P1-46 · Open** 返回逐项 applied/skipped/blocked/remap/diagnostic receipt。
47. **P1-47 · Open** whole-level disaster restore 与 selective authoring restore 使用不同 command。
48. **P1-48 · Open** 禁止 Restore 复用 additive spawn 作为默认语义。
49. **P1-49 · Open** 将 snapshot catalog 接入 asset index、reference graph、thumbnail 和 source control revision。
50. **P1-50 · Open** 为 snapshot browser 提供 open/duplicate/rename/delete/retention transaction。
51. **P1-51 · Open** 接入 Editor09 job admission、cancel、quota、progress 与 shutdown。
52. **P1-52 · Open** 接入 Editor02 document dirty/save/autosave/recovery，不建立第二 stack。
53. **P1-53 · Open** 接入 Editor27 VCS handoff，但不把 file diff 当 live semantic diff。
54. **P1-54 · Open** 建立 crash recovery、temporary artifact cleanup 和 interrupted commit recovery。
55. **P1-55 · Open** 统一 stable diagnostic code、owner/item/property、related asset、fix action。
56. **P1-56 · Open** 为 plugin serializer/filter/listener 提供 capability、version、thread 和 unload contract。
57. **P1-57 · Open** 为 capture/diff/merge/apply 建立 telemetry，不记录 UI fixture 成功。
58. **P1-58 · Open** 记录 capture coverage、skipped type、hash hit rate 和 dependency closure。
59. **P1-59 · Open** 记录 staging memory、I/O、CPU、rollback 和 receipt latency。
60. **P1-60 · Open** 设计 100k object、1M property、深引用图的性能预算。
61. **P1-61 · Open** 添加 capture roundtrip、stable matching 和 migration golden tests。
62. **P1-62 · Open** 添加 semantic diff object/component/property/reference golden tests。
63. **P1-63 · Open** 添加 three-way merge conflict matrix 与 custom resolver tests。
64. **P1-64 · Open** 添加 selective restore dependency/permission/stale tests。
65. **P1-65 · Open** 添加 fault-injected serializer/listener/rename/durability rollback tests。
66. **P1-66 · Open** 添加 filtered result、selection、Outliner/viewport projection integration tests。
67. **P1-67 · Open** 添加 cancellation、memory cap、quota、shutdown drain tests。
68. **P1-68 · Open** 添加 source/plugin catalog change、unknown field、codec unavailable tests。
69. **P1-69 · Open** 将 Archive slot merge、ScenePatch、Play snapshot、autosave semantics 在 API/UI 中显式标注。
70. **P1-70 · Open** 删除固定 Diff/Restore/Conflict 文案并通过端到端产品资格门。

### 5.3 P2：12 Open

1. **P2-01 · Open** chunked binary/compression/dedup/incremental remote storage。
2. **P2-02 · Open** cross-branch/cross-project relocation 与显式 identity remap package。
3. **P2-03 · Open** team snapshot sharing、review、approval、access control。
4. **P2-04 · Open** viewport before/after ghost、heatmap、change trail。
5. **P2-05 · Open** script/automation API，仍受 provider/revision/transaction gate 约束。
6. **P2-06 · Open** domain-specific conflict rules、batch resolver 与审计。
7. **P2-07 · Open** retention tier、pin、quota、GC、remote archive。
8. **P2-08 · Open** receipt 的 HTML/JSON export 与审计报表。
9. **P2-09 · Open** capture/diff/merge/apply 分阶段 profiling dashboard。
10. **P2-10 · Open** plugin serializer/restorability/resolver certification suite。
11. **P2-11 · Open** partition-aware lazy diff、streaming descriptor、局部大世界 restore。
12. **P2-12 · Open** 以同数据完整度、同 durability 条件建立超过参考引擎的 benchmark。

## 6. 目标架构与 ownership

~~~text
SceneSnapshotSource -> CaptureService -> immutable SnapshotArtifact
SnapshotArtifact + LiveWorld -> DiffService -> SceneChangeSet
base + ours + theirs -> MergeService -> ConflictPlan
selected changes -> staging World -> revision CAS -> Editor bulk transaction -> Receipt
~~~

| Owner | 必须拥有 | 不得冒充 |
|---|---|---|
| Runtime Scene / Reflection | typed capture codec、schema、reference resolve、staging primitive | Editor selection、undo、Results UI |
| Runtime Archive / Resource I/O | immutable artifact、bounded durable store、retention primitive | semantic diff、三方 merge、authoring restore |
| Editor Snapshot domain | provider/catalog、capture/diff/merge request、results、restore plan | 第二 serializer、第二 transaction stack |
| Editor Transaction | single bulk command、revision CAS、dirty/history/journal/rollback | schema migration、conflict policy |
| Editor Recovery | autosave discovery、安全副本、灾难恢复决策 | selective Scene change apply |
| Source Control | repository revision、file diff、changelist handoff | live World semantic merge |
| Plugin SDK | serializer/filter/listener/resolver extension | 绕过 capability、budget、revision、diagnostic gate |

SceneSnapshotAsset 必须携带 qualified identity、source/schema/plugin revision、capture policy、coverage、object/component/property/reference hash tree、reference/dependency manifest 与 immutable payload。UI filter 只能投影 SceneChangeSet，不能成为 authority。Archive slot、ScenePatch、Play file、autosave copy、UI whole-document diff 均不得进入该领域图。

## 7. 分阶段重构路线

| Milestone | 退出条件 |
|---|---|
| M0 | 固定 Diff 成功面封口；slot/ScenePatch/Play/autosave/UI diff 语义标注；required capture 缺失不再静默成功。 |
| M1 | identity、revision、typed address、serializer/capability 合同冻结。 |
| M2 | capture policy、coverage、consistency barrier、immutable artifact 完成。 |
| M3 | reference/dependency table、hash tree、migration、durable typed store 完成。 |
| M4 | semantic object/component/property/reference diff 与 canonical Results model 完成。 |
| M5 | filtered result、Outliner/Inspector/viewport/selection projection 完成。 |
| M6 | base/ours/theirs merge、conflict taxonomy、resolver、stale plan 完成。 |
| M7 | staging restore、dependency closure、CAS、bulk transaction、rollback、receipt 完成。 |
| M8 | plugin serializer/restorability/listener/resolver compatibility suite 完成。 |
| M9 | chunk/dedup/quota、bounded jobs、cancel、durability 和大世界性能完成。 |
| M10 | snapshot browser、Editor09/02/27/41 产品接线完成。 |
| M11 | fixture/API 硬切、32 门资格、文档与 reference recheck 完成。 |

实施顺序必须保持 M0 -> M1 -> M2/M3 -> M4 -> M5 -> M6 -> M7 -> M8 -> M9 -> M10 -> M11。缺少 M2/M3 时不得开放 Restore；缺少 M4 时不得把 equality 摘要展示为 Diff；缺少 M6 时不得把 VCS 或 slot merge 包装成 Conflict Resolution。

## 8. 验收门

| Gate | 状态 | 验收要求 |
|---|---|---|
| G01 | Fail | Scene Snapshot/Diff command 无 provider 时 disabled，不写固定成功文字。 |
| G02 | Fail | artifact 含 project/scene/source/snapshot identity 与 source revision。 |
| G03 | Fail | artifact 绑定 schema/plugin catalog、capture policy 与 coverage。 |
| G04 | Fail | required component/resource/field 缺失 fail closed 并给稳定 reason。 |
| G05 | Fail | object/component/property stable identity 跨保存、重载、实例化仍匹配。 |
| G06 | Fail | schema rename/move/convert/remove 有迁移或明确 blocked 结果。 |
| G07 | Fail | internal/subobject/external/unloaded reference 可无损 roundtrip。 |
| G08 | Fail | dependency closure、dangling、orphan 与 readiness 可审计。 |
| G09 | Fail | hash-first diff 与全量语义比较结果一致。 |
| G10 | Fail | capture consistency barrier 阻止跨 generation 混合。 |
| G11 | Fail | stale comparison 可见，stale apply 被拒绝。 |
| G12 | Fail | object/component/property/reference change 分类与 Results golden 通过。 |
| G13 | Fail | merge 强制 base/ours/theirs 并绑定三份 input digest。 |
| G14 | Fail | delete/edit、add/add、reparent/reparent、same-property 冲突确定分类。 |
| G15 | Fail | schema/plugin/codec 缺失为 blocked/unsupported，不降成删除或默认值。 |
| G16 | Fail | custom resolver 有 version、determinism、timeout、diagnostic 合同。 |
| G17 | Fail | resolution plan 可序列化、审计、重放。 |
| G18 | Fail | 任一 input/revision 变化后 resolution commit 被拒绝。 |
| G19 | Fail | selected changes 自动计算依赖闭包并解释强制项。 |
| G20 | Fail | restore 先在隔离 staging World resolve/apply/validate。 |
| G21 | Fail | restore 以 current authoring revision 做 CAS。 |
| G22 | Fail | apply 只提交一个 Editor bulk transaction，一次 undo 完整撤销。 |
| G23 | Fail | serializer/listener 失败时 World、dirty、history、selection 均不变。 |
| G24 | Fail | 成功 apply 同步 Hierarchy、Inspector、viewport、selection、toolkit。 |
| G25 | Fail | receipt 逐项记录 applied/skipped/blocked/remap/diagnostic/revision。 |
| G26 | Fail | whole-level disaster restore 与 selective restore 是不同 command/确认流。 |
| G27 | Fail | Archive slot、ScenePatch、Play、autosave、UI diff 标签不暗示 Scene semantic 产品。 |
| G28 | Fail | capture/diff/merge/preflight 支持 cancel、bounded memory、quota、shutdown。 |
| G29 | Partial | Session 已有同进程 path authority 与 file/parent-synced atomic primitive；typed snapshot store、persistent CAS、journal/startup recovery 与本域 crash fault 证明仍缺失。 |
| G30 | Fail | 100k object、1M property、深引用图满足预算且结果确定。 |
| G31 | Fail | plugin serializer/filter/listener/resolver 通过版本、失败、线程、卸载矩阵。 |
| G32 | Fail | 默认 UI、command、automation、docs、manifest、telemetry 只报告真实 artifact/receipt 状态。 |

## 9. 验证说明与风险

1. 本轮完成 803 个 selected 物理文件的统计/fingerprint、19,276 个 production 文件的精确合同扫描、当前源码语义复核与参考实现复核；没有执行会改变产品状态的操作。
2. 未运行 Cargo 或动态 Editor 测试，因为本轮为 review-only；静态存在性不能把任何功能门推断为 Pass。
3. 共享 working tree 正在变化，报告冻结的是验证时物理内容；实施前必须重新计算 manifest/fingerprint 并复核接口终态。
4. DynamicScene preflight、Session path authority、Recovery safe copy 与 Editor transaction 是正确底座；把这些底座直接重命名为 Snapshot/Diff/Merge/Restore 会掩盖数据完整性和提交语义缺口。
5. Session Archive facade 已有 578 个选择文件，Editor 只能依赖少量 typed service/store trait，不能从 facade 拼接 Scene Snapshot 产品。
6. “优于 Unreal”必须在功能完整度、捕获覆盖率、durability、故障注入、项目规模、交互结果、硬件和平台一致的条件下由 benchmark 与 profile 证明；当前没有这类证据。

文档终验必须确认：frontmatter 路径存在；P0/P1/P2 ID 唯一连续为 5/70/12，状态为 5 Open、69 Open/1 Partial、12 Open；M0-M11 完整；G01-G32 唯一且状态为 31 Fail/1 Partial；Editor/根索引与 coverage 451 同步；Markdown 无 trailing whitespace 或断链。整体引擎 review 继续进行，本报告不把 review 完成解释为实现完成。
