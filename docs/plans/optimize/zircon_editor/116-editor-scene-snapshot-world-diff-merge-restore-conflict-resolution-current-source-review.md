---
title: Editor Scene Snapshot、World Diff、Merge、Restore 与 Conflict Resolution 当前源码复核
category: zircon_editor
report_id: Editor116
review_date: 2026-08-26
baseline_head: 590376671b8745a0d230304c94432857c669bfbd
baseline_epoch: 524
canonical_owner: Editor42
refreshes:
  - docs/plans/optimize/zircon_editor/42-scene-snapshot-world-diff-merge-restore-conflict-resolution-authoring-review.md
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
tests:
  - zircon_editor/src/core/play/snapshot
  - zircon_editor/src/core/recovery/restore_flow.rs
  - zircon_editor/src/ui/asset_editor/document_diff.rs
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

# 116 · Editor Scene Snapshot / World Diff / Merge / Restore / Conflict Resolution 工程化差距

## 1. 结论

当前 Zircon 没有工程级 Scene Snapshot、语义 World Diff、三方 Merge、选择性 Restore 或 Conflict Resolution 产品。Runtime 已有规模可观的 DynamicScene 与 Session Archive，但真实语义是“把当前可反射世界放入 slot、做完整 equality 摘要、按 slot ID 合并或整世界恢复”。这些 API 名称不能直接成为 Editor 的对象级比较和恢复能力。

`RuntimeSessionSlot::diff_world` 重新捕获目标世界后只比较 `self.scene == target_scene`，报告只有 matches 与 entity/resource 数量。Archive merge 只处理重复 `slot_id` 的 Reject/Keep/Replace；`apply_to_world` 是 additive spawn，`restore_into_level` 是先构造空 World 再整世界替换。直接接入 Editor 会造成重复对象、未捕获状态丢失、绕过 dirty/selection/viewport/document revision，不能作为 Restore 或 Merge。

DynamicScene/Archive 仍是可保留的底座：版本化 payload、reflection adapter、entity remap、spawn preflight、canonical archive、512 MiB 上限、seal/cache、lineage/revision、manifest、retention、bounded writer 和路径原子替换都值得复用。但捕获只遍历已注册、声明 serializable、拥有 adapter 且实际存在的 type/field；storage 未注册、adapter 缺失、不可序列化字段和失败原因会静默消失，成功 snapshot 没有 coverage proof。

顶部 toolbar 已公开 Diff；route 最终只返回 `Scene diff prepared` 与 `Diff: scene preview state compared` 固定文字，没有 snapshot source、target revision、change set、provider 或 executor。Play Snapshot 是进程启动临时文件，autosave RestoreFlow 只有 plan，UI Asset DocumentDiff 整体替换完整 document，三者都不能冒充 Scene Snapshot 产品。

本轮选定 Zircon scope 为 669 files / 30,898 lines / 27,347 non-empty / 1,061,187 bytes / 187 test attributes；参考 scope 为 27 / 17,898 / 15,080 / 639,373 / 2；union 为 696 / 48,796 / 42,427 / 1,700,560 / 189。Zircon fingerprint `38a9cca4900a41fce10e3ffe47252dc0db3579423aec2b3e47b021f5b4910443`，refs `5e2b41fe1a51fd3fa7e33450c46363c539c1d30a19789913890a2cd209cd259c`，union `1d83581809ca62f1a753b19bb3efce067a156d6e941feda0d04f33906b2f6c06`。本报告登记 5 个 P0、70 个 P1、12 个 P2 与 M0-M11；不修改生产代码。

## 2. 证据与边界

### 2.1 当前实现事实

1. DynamicScene `from_world` 依据 registry、component/resource 标记、serializable 字段和 adapter 捕获；无 adapter、未注册 storage 与 skipped field 不进入成功结果。
2. `DynamicEntity` 保存 source entity、NodeRecord 和 component payload；Entity/Node ID 是进程内 u64，没有 project/scene/instance namespace。
3. serialization strategy 已覆盖 None/Value/Json/ResourceHandle/EntityReference，可作为 typed codec registry 起点，但当前没有 property hash tree、reference table、source revision 或 capture policy。
4. validation 检查 schema/version、source entity 唯一性和 plugin descriptor 一致性，却不输出 coverage、skipped reason、restorability 或 required-state fail-closed 结果。
5. Session slot 只含 slot_id、metadata 和完整 DynamicScene；metadata 没有 immutable project/scene identity、base revision、schema/plugin catalog digest。
6. slot diff 是 exact equality；slot merge 的 conflict 是重复 slot ID；apply 是 additive spawn；restore-to-empty/restore-into-level 是 whole-world flow。
7. Archive 的 canonical/seal/cache、retention/selection/prune、bounded writer、temporary/backup rename 和 path guard 是可复用 storage 原语，但不是 semantic diff/merge。
8. Scene Diff toolbar binding/navigation/preview action/feedback 只操作 control-local strings；无结果树、selection、filter、viewport highlight、apply、receipt 或 diagnostic provider。
9. Play snapshot 同步捕获并序列化临时输入，RestoreFlow 只规划 autosave action；生产中未发现 comparison executor。
10. UI Asset DocumentDiff 保存完整 target document 并整体 replay；与 Scene property-level diff 是不同 owner。

### 2.2 语义边界

1. `WorldInspectionDelta` 是同一 live World 的 generation 增量，不是持久化 snapshot diff。
2. `ScenePatch` 是 spawn/remap 描述，不表达对象删除、保留、修改或冲突。
3. `restore_into_level` 的全量 replacement 不能替代可选择、可撤销、revision-CAS 的 authoring restore。
4. `RuntimeSessionArchiveMergePolicy` 只能解决容器 slot 冲突，不应显示为 world conflict resolver。
5. Source Control 文件 diff、Editor document diff、Runtime diagnostic snapshot、Play input snapshot 必须在 UI 和 API 中标明各自 owner。
6. `DynamicScene` 是反射序列化 payload，不自动获得 stable cross-session object identity。

### 2.3 参考引擎差异

1. Unreal Level Snapshot 是持久 asset，分离 SnapshotWorld、DiffWorld、ApplySnapshotToWorld 和 PropertySelectionMap，并提供 actor/object/property/reference hash、filter、custom serializer、restoration listener 与 Results UI。
2. Godot PackedScene 保存 owner/instance/editable-children 和稳定 scene-local ownership，SceneTree 操作经可逆 command；不是简单 whole-world replacement。
3. Bevy Scene/ScenePatch 分离依赖 resolve、spawn、entity remap 与 apply failure；其 patch 仍不能直接当作三方 authoring merge。
4. Fyrox command stack 以可逆 ownership command 为边界；Unity `SceneObjectIDMap` 仅证明 rendering object ID，不是 Editor snapshot 实现。

## 3. 差距清单

### 3.1 P0：实施前必须阻断

1. **P0-01** 禁用或明确标记 toolbar 的固定 Scene Diff 成功文案；无 provider/artifact 时不得显示 native Diff。
2. **P0-02** 捕获若跳过 required component/resource/field，必须 fail closed 或发布 coverage report；不得把不完整 payload 当成功 snapshot。
3. **P0-03** 禁止把 slot equality、slot-ID merge、additive spawn 或 whole-level replacement 接为 Editor semantic diff/merge/restore。
4. **P0-04** 在 stable identity、source revision、typed change set、preflight、atomic rollback 和 receipt 前，不得修改 authoring World 或共享 asset。
5. **P0-05** Play snapshot、autosave restore、UI document diff、diagnostic capture 必须与 Scene Snapshot 分离，消除第二 authority 和数据丢失路径。

### 3.2 P1：70 项重构主线

1. **P1-01** 定义 project/scene/source/instance/snapshot qualified identity。
2. **P1-02** 为 object/component/property 分配跨保存稳定 ID。
3. **P1-03** 定义 source revision、schema revision、plugin catalog fingerprint。
4. **P1-04** 建立 source-object 与 instance-object provenance map。
5. **P1-05** 建立 typed property address 与 collection selector。
6. **P1-06** 建立 property schema fingerprint 和 migration contract。
7. **P1-07** 建立 typed reference/dependency table 及 dangling 分类。
8. **P1-08** 定义 capture policy、required state 与 plugin serializer registry。
9. **P1-09** 定义 owner/generation/request ID 传播规则。
10. **P1-10** 用 lint/test 分离 display path 与 stable identity。
11. **P1-11** 实现 immutable `SceneSnapshotAsset` source/catalog metadata。
12. **P1-12** 实现 chunked object/component/property payload 与 content digest。
13. **P1-13** 实现 capture consistency barrier，禁止跨 frame 混合 World。
14. **P1-14** 输出 captured/skipped/failed coverage report。
15. **P1-15** required-state 缺失时执行 fail-closed policy。
16. **P1-16** 为 custom serializer 提供 version、restorability reason 和 hooks。
17. **P1-17** 建立 reference closure、external asset readiness 和 orphan report。
18. **P1-18** 建立 schema/plugin migration、unknown field 与 codec fallback policy。
19. **P1-19** 为 capture job 接入 bounded memory、cancel、progress 和 shutdown drain。
20. **P1-20** 复用 Archive storage adapter 并补齐 file/parent durability。
21. **P1-21** 实现 object/component/property/reference 分层 hash tree。
22. **P1-22** 定义 `SceneDiffRequest` source、target、policy 和 freshness。
23. **P1-23** 实现 hash-first semantic diff，不能只比较完整 equality。
24. **P1-24** 区分 added/removed/modified/renamed/reparented/reordered object。
25. **P1-25** 区分 component topology、property、reference 与 asset changes。
26. **P1-26** 为 nested property、array/map/set element 提供稳定 change ID。
27. **P1-27** 支持 snapshot-vs-live、snapshot-vs-snapshot、revision-vs-live 三种模式。
28. **P1-28** stale source/target 必须阻断 Apply 并给出诊断。
29. **P1-29** 计算 selected change 的 dependency closure 并解释强制项。
30. **P1-30** 为 change set 提供 deterministic ordering、serialization 和 replay。
31. **P1-31** 实现 Results model、tree、search、filter、sort 和 hidden dependency 显示。
32. **P1-32** 同步 Outliner、Inspector、viewport highlight 和 selection revision。
33. **P1-33** 建立 `base/ours/theirs` 三方 Merge request/artifact。
34. **P1-34** 强制绑定三份 input digest、schema/plugin fingerprint。
35. **P1-35** 分类 delete/edit、add/add、reparent/reparent、same-property 冲突。
36. **P1-36** schema/plugin/codec 缺失时输出 blocked/unsupported，不自动当删除。
37. **P1-37** 实现 typed custom conflict resolver extension。
38. **P1-38** 生成可序列化、可审计、input-qualified resolution plan。
39. **P1-39** resolution plan 的 input 变化必须拒绝 commit。
40. **P1-40** 实现 `SceneRestoreCoordinator` staging-world resolve/apply/validate。
41. **P1-41** restore 前执行 dependency closure、ownership、permission 和 editability preflight。
42. **P1-42** 以 current authoring revision 做 CAS。
43. **P1-43** 选择性 restore 只提交一个可撤销 Editor bulk transaction。
44. **P1-44** 任一 serializer/listener 失败时保证 world/dirty/history 不变。
45. **P1-45** 成功 restore 后发布 hierarchy/inspector/viewport/selection resync。
46. **P1-46** 返回逐项 applied/skipped/blocked/remap/diagnostic receipt。
47. **P1-47** whole-level disaster restore 与 selective authoring restore 使用不同 command。
48. **P1-48** 禁止 Restore 复用 additive spawn 作为默认语义。
49. **P1-49** 将 snapshot catalog 接入 asset index、reference graph、thumbnail 和 source control revision。
50. **P1-50** 为 snapshot browser 提供 open/duplicate/rename/delete/retention transaction。
51. **P1-51** 接入 Editor09 job admission、cancel、quota、progress 与 shutdown。
52. **P1-52** 接入 Editor02 document dirty/save/autosave/recovery，不建立第二 stack。
53. **P1-53** 接入 Editor27 VCS handoff，但不把 file diff 当 live semantic diff。
54. **P1-54** 建立 crash recovery、temporary artifact cleanup 和 interrupted commit recovery。
55. **P1-55** 统一 stable diagnostic code、owner/item/property、related asset、fix action。
56. **P1-56** 为 plugin serializer/filter/listener 提供 capability、version、thread 和 unload contract。
57. **P1-57** 为 capture/diff/merge/apply 建立 telemetry，不记录 UI fixture 成功。
58. **P1-58** 记录 capture coverage、skipped type、hash hit rate 和 dependency closure。
59. **P1-59** 记录 staging memory、I/O、CPU、rollback 和 receipt latency。
60. **P1-60** 设计 100k object、1M property、深引用图的性能预算。
61. **P1-61** 添加 capture roundtrip、stable matching 和 migration golden tests。
62. **P1-62** 添加 semantic diff object/component/property/reference golden tests。
63. **P1-63** 添加 three-way merge conflict matrix 与 custom resolver tests。
64. **P1-64** 添加 selective restore dependency/permission/stale tests。
65. **P1-65** 添加 fault-injected serializer/listener/rename/durability rollback tests。
66. **P1-66** 添加 filtered result、selection、Outliner/viewport projection integration tests。
67. **P1-67** 添加 cancellation、memory cap、quota、shutdown drain tests。
68. **P1-68** 添加 source/plugin catalog change、unknown field、codec unavailable tests。
69. **P1-69** 将 Archive slot merge/Play snapshot/autosave semantics 在 API/UI 中显式标注。
70. **P1-70** 删除固定 Diff/Restore/Conflict 文案并通过端到端产品资格门。

### 3.3 P2：主线完成后扩展

1. **P2-01** chunked binary/compression/dedup/incremental remote storage。
2. **P2-02** cross-branch/cross-project relocation 与显式 identity remap package。
3. **P2-03** team snapshot sharing、review、approval、access control。
4. **P2-04** viewport before/after ghost、heatmap、change trail。
5. **P2-05** script/automation API，仍受 provider/revision/transaction gate 约束。
6. **P2-06** domain-specific conflict rules、batch resolver 与审计。
7. **P2-07** retention tier、pin、quota、GC、remote archive。
8. **P2-08** receipt 的 HTML/JSON export 与审计报表。
9. **P2-09** capture/diff/merge/apply 分阶段 profiling dashboard。
10. **P2-10** plugin serializer/restorability/resolver certification suite。
11. **P2-11** partition-aware lazy diff、streaming descriptor、局部大世界 restore。
12. **P2-12** 以同数据完整度、同 durability 条件建立超过参考引擎的 benchmark。

## 4. 目标架构与里程碑

```text
SceneSnapshotSource -> CaptureService -> immutable SnapshotArtifact
SnapshotArtifact + LiveWorld -> DiffService -> SceneChangeSet
base + ours + theirs -> MergeService -> ConflictPlan
selected changes -> staging World -> revision CAS -> Editor bulk transaction -> Receipt
```

`SceneSnapshotAsset` 应携带 SnapshotIdentity、source/schema/plugin revision、capture policy、coverage、object/component/property hash tree、reference/dependency manifest 与 immutable bulk payload。UI filter 只能投影 `SceneChangeSet`，不能成为 authority。Runtime Scene/Reflection 提供 codec、reference resolve 和 staging primitive；Archive 只负责 artifact/storage；Editor Snapshot domain 负责请求与结果；Editor Transaction 负责一次性提交；Source Control 只提供 repository revision。

| Milestone | 退出条件 |
|---|---|
| M0 | 固定 Diff 成功面封口；slot/Play/autosave 语义标注；required capture 缺失不再静默成功。 |
| M1 | identity、revision、typed address、serializer/capability 合同冻结。 |
| M2 | capture policy、coverage、consistency barrier、immutable artifact 完成。 |
| M3 | reference/dependency table、hash tree、migration、durable storage 完成。 |
| M4 | semantic object/component/property/reference diff 与 Results model 完成。 |
| M5 | filtered result、Outliner/Inspector/viewport/selection projection 完成。 |
| M6 | base/ours/theirs merge、conflict taxonomy、resolver、stale plan 完成。 |
| M7 | staging restore、dependency closure、CAS、bulk transaction、rollback、receipt 完成。 |
| M8 | plugin serializer/restorability/listener 与 compatibility suite 完成。 |
| M9 | chunk/dedup/quota、bounded jobs、cancel、durability 和大世界性能完成。 |
| M10 | snapshot browser、Editor09/10/27/41 产品接线完成。 |
| M11 | fixture/API 硬切、32 门资格、文档和 reference recheck 完成。 |

## 5. 验收门

1. **G01-G06** Scene Snapshot command 无 provider 时 disabled；artifact 有 identity/revision/catalog/policy/coverage；required 缺失 fail closed；stable identity 和 migration 通过。
2. **G07-G12** reference/dependency、hash-first、一致性 barrier、stale diff、object/component/property/reference 分类和 Results projection 通过 golden。
3. **G13-G18** base/ours/theirs digest、冲突分类、unsupported blocked、resolver plan、input freshness 和 deterministic replay 通过。
4. **G19-G25** dependency closure、staging World、revision CAS、single bulk transaction、rollback、projection resync、逐项 receipt 通过。
5. **G26-G30** whole-level 与 selective restore 分离；Archive slot merge、Play snapshot、autosave、UI document diff 不再冒充 Scene semantic 产品；cancel/quota/shutdown/durability 通过 fault tests。
6. **G31-G32** plugin compatibility、100k/1M benchmark、crash recovery、Windows dynamic matrix、UI/docs/manifest/telemetry 状态一致。

## 6. 本轮验证与限制

本轮仅做静态源码、测试 inventory、参考源码与物理范围 fingerprint 复核；没有修改 Runtime、Editor、interface、plugin 或 tests，也没有运行 Cargo、capture/diff/merge/restore 动态验证。frontmatter 列出的 52 个生产/测试路径与 20 个参考根需在实施前重取 manifest；历史报告中读取时存在的 recovery adapter、Workbench action 和 spawn transaction 在途文件必须复核。`git diff --check`、索引链接唯一性、P0/P1/P2=5/70/12、M0-M11 和 32 门是文档收尾门；整体引擎 review 仍保持进行中。
