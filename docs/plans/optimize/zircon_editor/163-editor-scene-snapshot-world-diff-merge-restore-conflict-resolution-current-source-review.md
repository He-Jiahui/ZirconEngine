---
title: Editor Scene Snapshot、World Diff、Merge、Restore 与 Conflict Resolution 当前源码复审
category: zircon_editor
report_id: Editor163
review_date: 2026-08-27
baseline_head: 1c8076ac65faee28290c575356e9fee6cc1fac48
verification_head: 1c8076ac65faee28290c575356e9fee6cc1fac48
canonical_owner: Editor42
supersedes_currentness_of:
  - docs/plans/optimize/zircon_editor/42-scene-snapshot-world-diff-merge-restore-conflict-resolution-authoring-review.md
  - docs/plans/optimize/zircon_editor/116-editor-scene-snapshot-world-diff-merge-restore-conflict-resolution-current-source-review.md
related_runtime_owners:
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/40-save-game-checkpoint-slot-participant-capture-serialization-migration-platform-cloud-async-network-scalability-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/99u-runtime-dynamic-scene-asset-reload-event-generation-reconciliation-stage-apply-instance-replacement-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_runtime/99v-runtime-dynamic-scene-session-archive-slot-capture-restore-path-merge-retention-durability-product-integration-current-source-review.md
related_editor_owners:
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/09-background-jobs-admission-scheduling-cancellation-progress-shutdown-product-integration-review.md
  - docs/plans/optimize/zircon_editor/27-project-operations-source-control-changelist-diff-automation-report-submit-gates-health-dashboard-review.md
  - docs/plans/optimize/zircon_editor/162-editor-level-variant-data-layer-level-instance-world-outliner-current-source-review.md
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

# Editor163 · Scene Snapshot / World Diff / Merge / Restore / Conflict Resolution 当前源码复审

## 1. 结论

Zircon 当前仍没有工程级 `SceneSnapshotAsset`、语义 `SceneChangeSet`、三方 Scene Merge、选择性 Restore 或 Conflict Resolution 产品。对 production roots 和当前未跟踪源码一起精确检索，`SceneSnapshotAsset`、`SceneChangeSet`、`SceneDiffRequest`、`SceneMergeService`、`SceneRestoreCoordinator`、`ConflictPlan`、`CaptureCoverage`、`PropertySelectionMap` 均为 **0 个定义**；默认 Editor command registry 也没有 Scene Snapshot、Diff、Merge 或 Restore 命令。

Workbench 仍暴露 `workbench.module.diff.invoke`，最终只写入 `Scene diff prepared` 与 `Diff: scene preview state compared` 固定文字。它没有 provider、snapshot source、target revision、change set、结果模型、apply transaction 或 receipt，因此 P0-01 仍是明确的虚假成功面。

Runtime DynamicScene 与 Session Archive 是真实但边界不同的底座。`RuntimeSessionSlot::diff_world` 仍重新执行 `DynamicScene::from_world`，只比较完整 scene equality，并返回 matches 与 entity/resource 数量；Archive merge 只处理重复 `slot_id` 的 Reject/Keep/Replace；`apply_to_world` 仍是 additive `spawn_into`；`restore_into_level` 仍先向空 World spawn、替换整个 World，再单独发布 metadata。这些语义不能重命名为对象级 diff、三方 merge 或选择性 restore。

本轮确认 recovery 子系统已有时效进展：`RestoreExecutor` 会校验 autosave 路径，通过共享 `atomic_write_new` 把 Restore/OpenComparison 物化到 `.zircon/recovered/{restore,comparison}`，不会覆盖 authoritative source；`ProjectRecoveryAssessment` 也已组合残留 session lock、activation ledger 终态、autosave catalog 与 RestoreFlow。Editor transaction engine 已有 RAII/nested transaction、apply/revert/finalize、失败 rollback、selection restoration、generation snapshot、journal、dirty/history。这些是应复用的灾难恢复与提交底座，但仍没有 comparison opener/provider、Scene change set、staging world、authoring revision CAS 或 `ApplySceneChangeSet`，不能把 Scene finding 判为 Partial。

Editor42 的 canonical finding 总数保持不变，本轮状态为：**P0：5 Open；P1：69 Open / 1 Partial；P2：12 Open；Gates：31 Fail / 1 Partial / 0 Pass**。唯一 Partial 是 Session writer 已硬切到共享 `stage_atomic_write`，具备 staging file 与父目录同步；Scene Snapshot store adapter、持久/跨进程 path CAS、Session journal/startup reconciliation 和本域 crash qualification 仍缺失。没有同内容完整度、同 durability、同硬件、同平台与同故障语义的动态 benchmark 可以证明本域达到、接近或优于 Unreal；功能缺失、完整 equality 更快或序列化文件更小均不是领先证据。

## 2. 审查范围、统计与 currentness

统计读取共享 working tree 的物理内容，包含相关未跟踪文件。行数为物理行；tests/ignored 只统计精确 Rust `#[test]` / `#[ignore]`。fingerprint 保留 repository-relative path 大小写并排序，对每文件 SHA-256 组成清单后再次计算 SHA-256；选择集用于复现本报告，不代表参考仓库的全部规模。

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored / dirty | fingerprint |
|---|---:|---|
| Editor authoring / recovery / surface | **136 / 22,304 / 20,418 / 782,753 / 112 / 0 / 119** | `2c7d1fad618b225fab854584a7988f967251e8de80f5e589c253be4a5db28581` |
| Runtime Session Archive | **574 / 12,091 / 10,799 / 415,927 / 35 / 0 / 21** | `5234c2196553340e9a47119f18bd8ee626d06fdb79fcc850a797353e9ef75da3` |
| DynamicScene / inspection / reflection support | **50 / 10,524 / 9,517 / 363,417 / 92 / 0 / 23** | `4eb948a437e246414b539e9a7668d46967340541908e42304f85fa698305a77d` |
| Focused Runtime tests | **23 / 6,845 / 6,271 / 249,565 / 125 / 0 / 13** | `98de0e5a8c60acfa2f6f616b02c254e50260cbc400b778dbc3e335357eec8684` |
| Zircon selected union | **783 / 51,764 / 47,005 / 1,811,662 / 364 / 0 / 176** | `d9a1f0fe27034a077831025a01a540b560a8da97baa40abfd1060168cb8ab5ea` |
| Unreal / Godot / Fyrox / Bevy / Unity Graphics selected | **27 / 17,876 / 15,080 / 639,373 / 1 / 0 / 0** | `7cb0fa45e3ca245f1d6bf6c4df6df41175baeb98e1ed04ba68dda8c6c56f74f3` |
| All selected | **810 / 69,640 / 62,085 / 2,451,035 / 365 / 0 / 176** | `62352ba93dc8ee17e04a69288a2958c8bb61f0b403bdf1063aa63760624e8eac` |

- baseline/verification HEAD 为 `1c8076ac65faee28290c575356e9fee6cc1fac48`，commit 时间为 2026-08-27T10:13:20+08:00；176 个 selected 文件有 working-tree 差异，实施前必须重新冻结。终验期间 Session owner 从 570 漂移到 574 个物理文件，本表冻结终验读取到的当前内容，不等待外部工作结束。
- Editor 集合包含 toolbar、default commands、core/editing、play/snapshot、core/recovery、document diff/undo stack、module feedback 与 preview actions；Runtime 集合包含完整 Session owner、DynamicScene scene/document、inspection、level system、reflection 与聚焦测试。
- 参考集合按 frontmatter 的 20 个入口展开为 27 个文件；Unreal Results 目录贡献 8 个文件。Unreal 是产品主基线，Godot/Fyrox/Bevy 只承担明确的局部架构参照，Unity Graphics 只用于划清 render object ID 与 authoring snapshot identity 的边界。
- 按用户要求未查询、轮询、等待或实时跟踪协调器；open plan/handoff 只作为静态 ownership 证据，不阻塞本轮 review。Tooling 按用户要求排除。
- 本轮只写 review 与索引，没有修改 production、tests、Cargo、ABI 或参考源码，也没有运行 Cargo、Editor、动态 restore、fault、scale、soak、跨平台或跨引擎 benchmark。

## 3. 当前源码事实与语义边界

### 3.1 DynamicScene capture 不是完整 Scene Snapshot

1. `DynamicScene::from_world` 遍历 `world.node_records()`；component/resource 只有在 registration 标记相符、声明 serializable、adapter 存在且 `contains` 为真时才进入 payload。
2. field capture 继续过滤到 metadata 中的 serializable 字段。未注册 storage、缺 adapter、不可序列化 type/field 和被跳过原因不会形成 coverage report，也不会阻止“捕获成功”。
3. payload 已有版本、schema migration、entity remap、compiled spawn preflight、schema catalog generation fence 与 bounded preflight；这些是可保留 primitive，不等于 capture completeness。
4. `EntityId` 仍是 `u64`，没有 project/scene/source/instance namespace；display path/name 也不能替代跨保存 stable identity。
5. Play Snapshot 仍把场景写入 `.zircon/play/<id>/play-scene.zrscene.json` 临时文件，作为子进程输入，退出后删除 owned root。它不是持久可浏览的 authoring snapshot asset。

### 3.2 Session Archive 不是 Scene Diff / Merge / Restore

1. Archive 具备 immutable sealed artifact、canonical payload、512 MiB cap、manifest/statistics/index、lineage/revision/cache、bounded keyed writer、path reservation/generation 与 staged rename；这些是应复用的 storage primitive。
2. Session writer 已删除私有 BufWriter/temp/backup 替换路径，改用共享 `stage_atomic_write`，从而复用 staging file sync、committed target sync 与 parent-directory sync；这是本轮唯一改变 finding 状态的真实进展。它仍使用进程内 `OnceLock<Mutex<HashMap<...>>>` 管理 path generation，没有 Scene Snapshot typed store adapter、持久/跨进程 CAS/lease、Session journal/startup reconciliation 或本域 crash fault qualification。
3. diff 仍是 scene equality 与数量摘要，没有 stable matching、typed property address、change classification 或 dependency closure。
4. merge plan 绑定 in-memory generation/revision 并支持 preview/commit，但冲突域仍只是不重复的 Archive slot ID，不是 base/ours/theirs World conflict。
5. apply 是追加 spawn；restore 是 whole-world replacement，metadata 又分步发布。没有 authoring revision CAS、staging validation、Editor bulk transaction、projection resync 或逐项 receipt。

### 3.3 Editor 恢复与事务底座的正确归属

1. `RestoreExecutor` 是 document disaster-recovery executor：Restore/OpenComparison 写出安全副本，Discard 删除受校验的 autosave candidate；它不直接覆盖源文档，这是正确进展。
2. 当前 comparison action 只物化 comparison copy，没有 semantic comparison provider/opener；它不能满足 Scene diff result tree 或 property selection。
3. `UiAssetDocumentDiff` 仍保存完整 target `UiAssetDocument` 并整体替换。undo stack 虽有更丰富 command，也不能把 whole-target replay称为 Scene property-level diff。
4. Editor transaction engine 应成为未来一次性 `ApplySceneChangeSet` 的提交 authority；Snapshot domain 不得另建 dirty/history/journal/rollback stack。
5. autosave disaster restore、Play process snapshot、UI document diff、WorldInspectionDelta、Source Control file diff、Archive slot merge 必须在 API、UI、docs 与 telemetry 中各自标明 owner 和真实语义。

## 4. 参考引擎差异

1. Unreal Level Snapshots 把 `ULevelSnapshot` 作为 asset，分离 `SnapshotWorld`、`DiffWorld`、`ApplySnapshotToWorld`；`FPropertySelectionMap` 明确 selected properties、added/removed components 与 custom subobject restoration。
2. Unreal actor snapshot 保存 label、class index、serialized/custom serializer/component/owned-subobject data 与 hash，并提供 hash-first change detection、custom serializer、restoration listener、filter extender、restorability reason 和完整 Results UI owner。
3. Godot `SceneState` 保存 node/id path、editable instance、base scene、parent/owner/type/name/instance/index、property/group/connection 与 scene-unique ID；pack/instantiate 与 SceneTree UndoRedo 展示的是 ownership 和可逆编辑边界，而非 whole-world replacement。
4. Fyrox command 以 execute/revert/finalize 为合同，CommandGroup 正序执行、逆序撤销；graph command 通过 Handle/Ticket/SubGraph 保持 add/delete/replace/reparent 的可逆 ownership。
5. Bevy `ScenePatch` 显式注册并加载 dependency、resolve 后 spawn/apply；`ResolvedSceneRoot::spawn` 在 apply 失败时回收中间 entity。它提供生命周期参考，但不是三方 authoring merge 产品。
6. Unity Graphics `SceneObjectIDMap` 只是 scene-scoped rendering utility，将 GameObject 映射到 compact category/id；它不能用来推断 Unity Editor 的 Scene Snapshot 内部实现，也不能替代 qualified authoring identity。

## 5. 差距清单

### 5.1 P0：5 Open

1. **P0-01 · Open** 禁用或明确标记 toolbar 的固定 Scene Diff 成功文案；无 provider/artifact 时不得显示 native Diff。
2. **P0-02 · Open** 捕获若跳过 required component/resource/field，必须 fail closed 或发布 coverage report；不得把不完整 payload 当成功 snapshot。
3. **P0-03 · Open** 禁止把 slot equality、slot-ID merge、additive spawn 或 whole-level replacement 接为 Editor semantic diff/merge/restore。
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
11. **P1-11 · Open** 实现 immutable `SceneSnapshotAsset` source/catalog metadata。
12. **P1-12 · Open** 实现 chunked object/component/property payload 与 content digest。
13. **P1-13 · Open** 实现 capture consistency barrier，禁止跨 frame 混合 World。
14. **P1-14 · Open** 输出 captured/skipped/failed coverage report。
15. **P1-15 · Open** required-state 缺失时执行 fail-closed policy。
16. **P1-16 · Open** 为 custom serializer 提供 version、restorability reason 和 hooks。
17. **P1-17 · Open** 建立 reference closure、external asset readiness 和 orphan report。
18. **P1-18 · Open** 建立 schema/plugin migration、unknown field 与 codec fallback policy。
19. **P1-19 · Open** 为 capture job 接入 bounded memory、cancel、progress 和 shutdown drain。
20. **P1-20 · Partial** Session writer 已复用共享 atomic file primitive 并补齐 file/parent sync；仍需 Scene Snapshot typed store adapter、persistent/cross-process CAS、journal/recovery 与本域 fault qualification。
21. **P1-21 · Open** 实现 object/component/property/reference 分层 hash tree。
22. **P1-22 · Open** 定义 `SceneDiffRequest` source、target、policy 和 freshness。
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
33. **P1-33 · Open** 建立 `base/ours/theirs` 三方 Merge request/artifact。
34. **P1-34 · Open** 强制绑定三份 input digest、schema/plugin fingerprint。
35. **P1-35 · Open** 分类 delete/edit、add/add、reparent/reparent、same-property 冲突。
36. **P1-36 · Open** schema/plugin/codec 缺失时输出 blocked/unsupported，不自动当删除。
37. **P1-37 · Open** 实现 typed custom conflict resolver extension。
38. **P1-38 · Open** 生成可序列化、可审计、input-qualified resolution plan。
39. **P1-39 · Open** resolution plan 的 input 变化必须拒绝 commit。
40. **P1-40 · Open** 实现 `SceneRestoreCoordinator` staging-world resolve/apply/validate。
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
69. **P1-69 · Open** 将 Archive slot merge/Play snapshot/autosave semantics 在 API/UI 中显式标注。
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

```text
SceneSnapshotSource -> CaptureService -> immutable SnapshotArtifact
SnapshotArtifact + LiveWorld -> DiffService -> SceneChangeSet
base + ours + theirs -> MergeService -> ConflictPlan
selected changes -> staging World -> revision CAS -> Editor bulk transaction -> Receipt
```

| Owner | 必须拥有 | 不得冒充 |
|---|---|---|
| Runtime Scene / Reflection | typed capture codec、schema、reference resolve、staging primitive | Editor selection、undo、Results UI |
| Runtime Archive / Resource I/O | immutable artifact、bounded durable store、retention primitive | semantic diff、三方 merge、authoring restore |
| Editor Snapshot domain | provider/catalog、capture/diff/merge request、results、restore plan | 第二 serializer、第二 transaction stack |
| Editor Transaction | single bulk command、revision CAS、dirty/history/journal/rollback | schema migration、conflict policy |
| Editor Recovery | autosave discovery、安全副本、灾难恢复决策 | selective Scene change apply |
| Source Control | repository revision、file diff、changelist handoff | live World semantic merge |
| Plugin SDK | serializer/filter/listener/resolver extension | 绕过 capability、budget、revision、diagnostic gate |

`SceneSnapshotAsset` 必须携带 qualified identity、source/schema/plugin revision、capture policy、coverage、object/component/property/reference hash tree、reference/dependency manifest 与 immutable payload。UI filter 只能投影 `SceneChangeSet`，不能成为 authority。Archive slot、Play file、autosave copy、UI whole-document diff 均不得进入上述领域图。

## 7. 分阶段重构路线

| Milestone | 退出条件 |
|---|---|
| M0 | 固定 Diff 成功面封口；slot/Play/autosave/UI diff 语义标注；required capture 缺失不再静默成功。 |
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
| G27 | Fail | Archive slot、Play、autosave、UI diff 标签不暗示 Scene semantic 产品。 |
| G28 | Fail | capture/diff/merge/preflight 支持 cancel、bounded memory、quota、shutdown。 |
| G29 | Partial | Session 已复用 file/parent-synced atomic primitive；typed snapshot store、persistent CAS、journal/startup recovery 与本域 crash fault 证明仍缺失。 |
| G30 | Fail | 100k object、1M property、深引用图满足预算且结果确定。 |
| G31 | Fail | plugin serializer/filter/listener/resolver 通过版本、失败、线程、卸载矩阵。 |
| G32 | Fail | 默认 UI、command、automation、docs、manifest、telemetry 只报告真实 artifact/receipt 状态。 |

## 9. 验证说明与风险

1. 本轮完成静态源码、现有测试 inventory、参考源码和选择集 fingerprint 复核；没有执行会改变项目状态的产品操作。
2. 未运行 Cargo 或动态 Editor 测试，因为本轮授权是 review-only；因此 32 个 Gate 均不能由静态存在性推断为 Pass。
3. 176 个 selected 文件存在共享 working-tree 改动。报告记录的是本轮物理快照，不回退、不覆盖其他修改；实施开始前必须重新计算 manifest/fingerprint 并复核接口终态。
4. autosave executor 与 transaction engine 是正确底座，但 Scene 产品合同仍为零定义；以底座存在把相关 finding 降为 Partial 会掩盖产品缺口。
5. Session Archive facade 规模已经过大，Editor 只能依赖少量 typed service/store trait，不能从数百文件 facade 任意拼接“产品”。
6. “优于 Unreal”必须在功能完整度、数据完整度、durability、故障注入、项目规模、画质/交互结果、硬件和平台一致的条件下由 benchmark 与 profile 证明；当前没有这类证据。

文档收尾验证必须确认：frontmatter 路径存在；P0/P1/P2 ID 分别唯一且连续为 5/70/12，状态为 5 Open、69 Open/1 Partial、12 Open；M0-M11 完整；G01-G32 唯一且状态为 31 Fail/1 Partial；Editor/根索引与 coverage 395 同步；Markdown 无 trailing whitespace 或断链。整体引擎 review 继续进行，本报告不把 review 完成解释为实现完成。
