---
title: Editor Scene Document、Authoring World、Open/New/Reload/Save/Close、Dirty Transition、Autosave、Recovery 与 Multi-Document Product Integration 当前源码工程化差距
category: zircon_editor
report_id: Editor61
review_date: 2026-08-22
baseline_head: bee4c707b714738346b49bba15c59468b8bd9b39
baseline_epoch: 339
related_code:
  - zircon_editor/src/core/document
  - zircon_editor/src/core/project/scene_document.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/ui/host/editor_manager_project.rs
  - zircon_editor/src/ui/host/project_access.rs
  - zircon_editor/src/ui/workbench
  - zircon_editor/src/ui/host
  - zircon_editor/src/ui/retained_host/app/scene_picker_actions.rs
  - zircon_editor/src/ui/retained_host/app/scene_picker_session.rs
  - zircon_editor/src/ui/retained_host/app/assets/workspace.rs
  - zircon_editor/src/ui/retained_host/app/autosave.rs
  - zircon_editor/src/ui/retained_host/app/native_window_close.rs
  - zircon_editor/src/ui/retained_host/app/startup.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch
  - zircon_runtime/src/scene/world/project_io
  - zircon_runtime/src/scene/world/project_io.rs
tests:
  - zircon_editor/src/core/document/lifecycle/tests.rs
  - zircon_editor/src/core/document/scene_route_tests.rs
  - zircon_editor/src/core/project/tests/scene_document.rs
  - zircon_editor/src/ui/host/editor_manager_project.rs
  - zircon_editor/src/ui/retained_host/app/scene_picker_session_tests.rs
  - zircon_editor/src/tests/editor_event/runtime/integration/project.rs
  - zircon_editor/src/tests/host/retained_asset_refresh/scene_reload.rs
  - zircon_editor/src/ui/retained_host/app/native_window_close.rs
  - zircon_app/tests/editor_mvp_authoring.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/03-scene-prefab-selection-mode-gizmo-picking-review.md
  - docs/plans/optimize/zircon_editor/51-editor-project-startup-open-create-authority-hub-handshake-session-guard-focus-recent-recovery-product-integration-review.md
  - docs/plans/optimize/zircon_editor/57-editor-asset-workspace-content-browser-folder-source-tree-selection-open-create-import-rename-move-delete-history-collection-product-integration-review.md
  - docs/plans/optimize/zircon_runtime/99j-runtime-scene-world-level-registry-lifecycle-project-io-snapshot-clone-serialization-schema-transaction-product-integration-current-source-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Public/FileHelpers.h
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/FileHelpers.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/PackageAutoSaver.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/PackageAutoSaver.h
  - dev/godot/editor/editor_data.h
  - dev/godot/editor/editor_data.cpp
  - dev/godot/editor/editor_node.cpp
  - dev/godot/editor/editor_node.h
  - dev/Fyrox/editor/src/scene/mod.rs
  - dev/Fyrox/editor/src/scene/container.rs
  - dev/Fyrox/editor/src/scene/commands/mod.rs
  - dev/Fyrox/editor/src/menu/file.rs
  - dev/Fyrox/editor/src/message.rs
  - dev/bevy/crates/bevy_asset/src/path.rs
  - dev/bevy/crates/bevy_asset/src/io/source.rs
  - dev/bevy/crates/bevy_asset/src/saver.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/CoreEditorUtils.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Lighting/ProbeVolume/ProbeVolumeLightingTab.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Editor/Volume/VolumeComponentEditor.cs
doc_type: current_source_review
review_status: complete
implementation_status: not_started
source_recheck_required: true
---

# Editor Scene Document、Authoring World、Open/New/Reload/Save/Close、Dirty Transition、Autosave、Recovery 与 Multi-Document Product Integration 当前源码工程化差距

## 1. 结论

当前场景持久化底座并非临时空壳。Runtime保存入口要求显式`World + target scene URI`，先序列化再走atomic write；Editor的场景创建流程具有staging、catalog rollback和导入失败补偿；document lifecycle、history、selection、session guard、autosave store和recovery DTO也都已有可复用实现。问题在于这些能力尚未收敛为一个产品级Scene Document authority：普通启动、picker打开、source创建、菜单保存、asset watcher重载、project关闭、窗口关闭和autosave各自持有不同的“当前文档”定义。

当前源码仍可到达5条已由旧报告拥有的P0路径：打开次要场景后“Save Project”仍写manifest的default scene；Open/New会无dirty preflight地替换World并清空Global history/selection；Close Project绕过dirty gate；窗口关闭的Save分支明确未实现；default scene watcher可直接以磁盘默认场景替换当前World。它们继续分别由Editor03、Editor02和Editor57唯一计数，本报告不另立重复P0。

本轮新增并细化 **28项P1、8项P2与40个资格门**。核心判断是：`World`不能同时充当“运行中对象图”“当前文档身份”“dirty authority”“保存目标”和“恢复单元”。目标必须是`SceneDocumentSessionRegistry`拥有每个场景的source identity、authoring World、history、dirty、selection、viewport state、autosave/recovery与外部revision；所有Open/New/Reload/Close/Project Switch/Exit均经过同一可回滚transition coordinator；保存必须绑定明确源目标和expected revision，而不是从当前World快照反推manifest default scene。

本轮为review-only。未修改production Rust，未运行Cargo、真实Editor、崩溃恢复、外部并发写、百万实体、fault/soak/profile或同负载跨引擎benchmark，因此不能据此宣称性能或表现优于Unreal。tooling按用户要求排除。

## 2. 审查边界、currentness与证据

### 2.1 冻结语料

| 范围 | 文件 / 行 / 非空行 / bytes / test attributes | 本轮证据 | fingerprint |
|---|---:|---|---|
| Core document与project scene authority | **10 / 3,101 / 2,855 / 118,224 / 20** | lifecycle、route、scene document、manager/session、project access与installer | `f8c43f1938e33da2a7be24da847ef25881825b7b9ee1efeeb8143dc65d14ae33` |
| Retained product paths | **24 / 4,884 / 4,501 / 182,537 / 37** | startup、picker、workspace save/reload、autosave、native close与callback dispatch | `debf0a0c13537deb877eedcaa8bf82e8515883d20408a6c08bf1149c73ae5b19` |
| Runtime/support persistence | **7 / 3,017 / 2,724 / 98,936 / 3** | project scene IO、authoring codec、atomic writer与workspace support | `8b2a53e20ca868478218dd041ef300495bff890133b25d9cf93fd8cfcae47c5b` |
| Focused tests | **9 / 3,779 / 3,487 / 133,850 / 74** | lifecycle、picker、create rollback、save/close/autosave与MVP authoring | `26bfac7a07b4c33eba02ae1125181660bb2e3c8afc9a4e2e032159d5707b8a4d` |
| Zircon去重focused set | **50 / 14,781 / 13,567 / 533,547 / 134** | 当前working tree静态冻结；focused文件均无dirty | `75ba0acef0f1c2fc336cbe2a9508de748ebcdb018140789d0f673d7c6155ee2b` |
| 五引擎19个显式参考文件 | **19 / 32,259 / 27,885 / 1,254,980 / 24** | Unreal save/autosave、Godot edited scenes、Fyrox scene container、Bevy asset source、Unity Graphics Editor使用面 | `abe11cf4e0b832ecc50184b533daaf36f29da6910a2063af7f4d9d7127d0564b` |

fingerprint算法沿用Editor58至Editor60：按normalized lowercase relative path排序，把`path + NUL + lowercase per-file SHA-256 + LF`串联后再取SHA-256。它只证明本轮读取集合与working-tree内容，不是ABI、artifact、cache key或动态验收receipt。

冻结Git基线为`bee4c707b714738346b49bba15c59468b8bd9b39`，coordinator baseline epoch为339。Godot、Fyrox、Bevy、Unity Graphics参考revision分别为`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`与`a7e4c051d256a781ab362c64316b125a1e104694`；Unreal镜像随主仓基线冻结。

### 2.2 当前产品链

```text
ordinary project startup
  -> EditorManager commits project/session guard
  -> document_lifecycle.begin_project_session(root)
  -> activate_project_document(root)
  -> retained host later installs startup World
  -> no active scene key is established

picker Open/Create Scene
  -> SceneDocumentRoute loads/imports/clones ProjectSceneDocument
  -> AuthoringSceneInstaller installs Scene into current World
  -> lifecycle activates (project_root, scene_uri)
  -> previous scene session/history is not retained

File > Save Project
  -> snapshot current live World as Scene
  -> EditorProjectDocument::save_to_project
  -> World::save_scene_to_project(manifest.default_scene)
  -> reimport manifest.default_scene
  -> lifecycle emits Saved for whichever document is active

asset watcher sees default scene change
  -> reload_default_scene
  -> load disk default scene
  -> runtime.replace_world
  -> no dirty/current-scene/external-conflict decision

autosave / native close
  -> enumerate dirty document toolkits only
  -> active Scene uses Global history and is not a toolkit document
  -> scene is absent from autosave capture and Save-on-close execution
```

### 2.3 已有基础必须保留

1. 保留Runtime显式目标的scene writer、serialize-before-write和single-file atomic publication。
2. 保留场景创建的staging、catalog transaction rollback与失败补偿，但补上跨进程operation journal。
3. 保留DocumentId、DocumentMessage和session generation的方向，但扩展为qualified scene identity与typed receipt。
4. 保留HistoryContext、selection snapshot和World replacement exclusive transition，迁移为每SceneDocumentSession所有。
5. 保留autosave/recovery store的有界写入、retention和session guard，不建立Scene私有的第二套文件格式。
6. 保留workspace持久化的atomic rollback能力，扩展其schema以保存open scenes和active document。
7. 保留导入artifact读取与source URI解析，增加source/artifact revision和freshness receipt。
8. 保留现有单元测试作为局部回归，不把`include_str!`源码形状断言当作产品生命周期资格。

## 3. 既有P0的current-source裁决与唯一owner

| canonical owner | 当前可达链 | Editor61裁决 |
|---|---|---|
| Editor03 P0-01 | 打开非default scene后，`SaveProject -> save_to_project -> save_scene_to_project(manifest.default_scene)`；selected scene URI在installer/state边界被丢弃 | 保持Open。Editor03唯一计数；本报告负责把active source identity接入save coordinator |
| Editor03 P0-02 | Open/Create成功后立即`replace_world`，清Global history和selection，没有Save/Discard/Cancel | 保持Open。所有Scene/Project/Window切换统一进入transition coordinator |
| Editor02 P0-02 | File > Close Project直接调用`EditorManager::close_project` | 保持Open。Project close不得旁路Scene dirty registry |
| Editor02 P0-03 | native/floating window close的Save分支只返回“Documents could not be saved” | 保持Open。Scene与toolkit文档都必须进入真实save plan并等待terminal receipt |
| Editor57 P0-01 | watcher只认default scene，直接`runtime.replace_world`；非default active scene也会被默认场景覆盖 | 保持Open。外部变更必须由active source identity和conflict reconciler处理 |

Editor51的project activation提交顺序、Editor02的Document/Autosave/Recovery通用合同、Editor03的Scene/Prefab/Selection业务语义、Editor57的asset watcher/catalog，以及Runtime61的World/project IO继续保持父owner。本报告只拥有Scene Document产品集成缺口和current-source验收路线，跨报告汇总时上述5项P0不得再次相加。

## 4. P1：Scene Document工程化差距

### 4.1 文档身份、启动与session authority

1. **ED61-P1-01** 普通项目启动只激活project-root document，随后安装default scene World却不建立`active_scene_key`。第一帧起World内容、lifecycle身份、保存目标和watcher假设就可能指向不同对象。
2. **ED61-P1-02** lifecycle在retained host安装World之前发布`Opened`。若startup World prepare/replace失败，project、session guard和document lifecycle已提交，调用者只显示错误而不rollback已激活session。
3. **ED61-P1-03** `DocumentLifecycleAuthority`只保存一个`active_document`和一个`active_scene_key`；切换时发送Closed/Opened并丢弃旧session状态，无法支持多场景tab、split view或暂存dirty scene。
4. **ED61-P1-04** 没有“取得当前场景源身份”的通用读取合同，只有调用者提供URI后做相等查询。Save、watcher、recovery和workspace无法共享同一个authoritative target。
5. **ED61-P1-05** `AuthoringSceneInstaller`只接受`&Scene`；安装边界丢失DocumentId、source URI/path、artifact revision、base digest、schema version和load diagnostics。
6. **ED61-P1-06** startup/open/create中的World安装与lifecycle激活不是prepare/commit事务，也没有rollback receipt。installer成功、lifecycle失败或反向失败时缺少恢复到旧document session的合同。

### 4.2 Open/New/Reload/Save/Close transition与receipt

7. **ED61-P1-07** route对`AlreadyActive`直接返回，产品没有显式Reload/Revert语义；picker仍笼统显示“Opened scene”，无法区分真正安装与no-op。
8. **ED61-P1-08** 产品菜单没有Save Scene、Save Scene As、Save All Scenes、Reload Scene、Revert Scene或Close Scene动作；Save Project被迫承担错误的scene persistence职责。
9. **ED61-P1-09** `DocumentMessage::Saved`在持久化后根据“当时active document”临时制造，不携带source URI、expected/committed revision、digest、bytes、durability、projection disposition或project/session generation。
10. **ED61-P1-10** save完成后reimport/refresh失败只记日志，调用者仍收到成功；“source已持久化但catalog/artifact projection陈旧”没有独立terminal disposition。
11. **ED61-P1-11** `ProjectSceneDocument`只有URI/path/world，没有source revision、artifact revision、base digest、schema/load receipt，无法证明打开的artifact与source一致，也无法构造安全保存precondition。
12. **ED61-P1-12** Scene使用Global history；切换、保存、dirty判断和selection无法按文档隔离。未来多场景会把撤销记录和clean checkpoint错误应用到另一World。

### 4.3 Dirty、autosave、recovery与workspace恢复

13. **ED61-P1-13** Scene没有注册为document toolkit；production中的toolkit注册只覆盖UI asset与animation。通用dirty/save/close/autosave产品枚举天然看不到Scene。
14. **ED61-P1-14** `dirty_document_toolkits()`驱动autosave和native close prompt，但Scene dirty来自Global history，导致场景修改既不进入autosave capture，也不能在Save-on-close分支执行。
15. **ED61-P1-15** autosave scheduler只为dirty toolkit创建identity/capture；active scene没有autosave key、snapshot generation、source/base revision或恢复元数据。
16. **ED61-P1-16** recovery store、RestoreFlow和SessionGuard takeover虽有定义与测试，但没有产品级executor把恢复候选还原为SceneDocumentSession；普通启动也不会枚举并决策scene recovery。
17. **ED61-P1-17** workspace只围绕project/default scene与layout持久化，不保存open scene list、active scene URI、per-scene history checkpoint、selection、viewport或unsaved recovery link。
18. **ED61-P1-18** close/exit的dirty集合没有冻结generation。prompt展示后到save/discard commit之间仍可能发生编辑、watcher reload或document switch，缺少一致的`ClosePlan`与逐文档terminal receipt。

### 4.4 外部变更、schema与发布一致性

19. **ED61-P1-19** watcher把default-scene文件变化简化为Boolean reload，没有clean auto-reload、dirty conflict、keep local、reload、save-as或merge决策状态。
20. **ED61-P1-20** watcher观察manifest default，而非真实active scene source；打开次要场景后，次要源变化不会正确刷新，默认源变化反而替换当前World。
21. **ED61-P1-21** Runtime writer没有expected revision/digest CAS。即使Editor未来修正保存URI，外部进程在打开后修改同一scene仍会被静默覆盖。
22. **ED61-P1-22** create flow只有进程内补偿；source publication、catalog mutation、import和World install之间崩溃可留下orphan source或catalog/source不一致，没有durable operation journal与startup reconciliation。
23. **ED61-P1-23** scene TOML没有显式format/schema version与migration pipeline；reader可暂存部分`_rest`字段，但转成`SceneAsset`/World再保存时不能保证未来字段或plugin-owned数据无损往返。
24. **ED61-P1-24** 打开使用imported artifact，却没有source/artifact freshness proof；保存后也没有把source revision、artifact generation和installed World generation收敛为同一publication receipt。

### 4.5 多文档、规模、诊断与测试覆盖

25. **ED61-P1-25** 没有`SceneDocumentSessionRegistry`，无法同时持有多个authoring World、per-scene dirty/history/selection/viewports/interactions，也无法定义后台scene的内存与资源预算。
26. **ED61-P1-26** SaveProject先clone/snapshot整个Scene并在菜单调用链同步序列化/写入/reimport；open/install也存在多次clone。大型场景可能造成UI停顿和`O(scene size)`峰值额外内存。
27. **ED61-P1-27** retained route大量把typed IO/import/catalog/install错误压成`String`，无法按retryable/conflict/schema/corruption/cancel/stale generation分类，也不能生成稳定诊断与自动化断言。
28. **ED61-P1-28** 现有测试偏重局部route、atomic write和rollback；没有secondary open-edit-save、dirty Open/New/Close/Exit、startup identity、scene autosave/recovery、watcher+nondefault、external CAS、same-scene reload、多Scene session和large-scene latency的端到端RED矩阵。

## 5. P2：长期工程化与效率差距

1. **ED61-P2-01** lifecycle DocumentId仍依赖path/FNV派生、collision probing和有限retention，没有project/session epoch；通用owner为Editor02 P2-02，本报告只要求Scene key不得继续复用该弱身份。
2. **ED61-P2-02** 没有recent scenes、pinned scenes、missing-source remediation和workspace reopen diagnostics，复杂项目的日常导航效率不足。
3. **ED61-P2-03** 没有background save admission、priority、cancelability、progress、backpressure和quiescence；大场景只能同步阻塞或无界并发两种错误选择。
4. **ED61-P2-04** 没有增量serialization、chunk/partition-aware save、copy-on-write snapshot或dirty component/page tracking，无法给出十万/百万实体编辑保存预算。
5. **ED61-P2-05** 没有场景保存/加载trace span、阶段耗时、clone bytes、peak memory、fsync/reimport耗时和失败stage telemetry。
6. **ED61-P2-06** 没有scene-specific crash breadcrumb、last transition journal和恢复来源说明，故障后难以判定磁盘、autosave、artifact与World谁更新。
7. **ED61-P2-07** 无collaborative/external lease、read-only checkout、source-control checkout或multi-user edit policy；CAS只能作为底线，不能替代团队协作治理。
8. **ED61-P2-08** 没有可访问的dirty/conflict/recovery状态展示、批量Close决策摘要和失败焦点恢复；文档规模扩大后会形成操作风险。

## 6. 五引擎参考裁决

| 参考 | 本轮可证事实 | Zircon应吸收 | 禁止误用 |
|---|---|---|---|
| Unreal | `SaveWorld`显式接收World与目标文件；LoadMap前先请求保存dirty packages并允许取消；autosaver监听dirty/saved、维护restore文件和恢复候选，恢复失败时保留信息 | 显式target、dirty preflight、save/load非重入、autosave journal与可恢复terminal state | 不能只复制菜单名称，也不能把Package概念硬套成单一default scene |
| Godot | `EditedScene`保存root、path、modified time、editor state、selection、selection history、custom state和独立history id；EditorData管理多个edited scenes；reload/close有Save & Reload、Save & Close决策 | per-scene session registry、独立history/selection/state、明确reload/close动作 | 不把单线程UI实现细节当作性能基线 |
| Fyrox | `SceneContainer`保存多个scene entry的id/path/unsaved；文件菜单提供Save/Save As/Save All；dirty确认框具有Yes/No/Cancel并以deferred action执行load/create/close | 多Scene container、延迟transition action、Save All与terminal decision | 不照搬其具体消息枚举而绕过Zircon typed transaction/receipt |
| Bevy | `AssetPath`区分source/path/label；`AssetSource`分离reader/writer/watcher和processed source；typed saver接收asset path | source identity、typed path、reader/writer/watcher职责分离 | Bevy源码明确saver不是通用持久化产品；不能拿AssetSaver冒充Editor document workflow |
| Unity Graphics | 本地开源范围可证明Editor扩展使用Undo、SetDirty、SaveAssets和sceneOpened订阅 | 只作为Editor API消费与dirty集成的局部证据 | 本地Graphics仓不包含Unity专有Scene lifecycle authority，不据此推断其完整产品实现 |

参考选择路由为：Unreal主导save/load/autosave durability与dirty package gating；Godot/Fyrox主导多场景authoring session和交互决策；Bevy只支撑asset source identity；Unity Graphics只作负边界与Editor集成证据。不存在“参考引擎有某个类名，所以Zircon也新增同名类”的浅层映射。

## 7. 目标架构与authority边界

```text
zircon_editor
  ProjectSessionGeneration
    -> SceneDocumentSessionRegistry
         -> SceneDocumentSession
              SceneDocumentKey
              SceneSourceIdentity
              AuthoringWorldSession
              HistoryContext + clean checkpoint
              Selection / viewport / interaction state
              Autosave / recovery state
              ExternalRevisionState

  DocumentTransitionCoordinator
    -> freeze requested generations
    -> build SaveDiscardCancel plan
    -> prepare load/create/reload/close
    -> commit registry + World + lifecycle + workspace atomically
    -> publish typed terminal receipt

  DocumentSaveCoordinator
    -> capture immutable save input outside UI mutation
    -> verify target + expected source revision
    -> call zircon_runtime scene writer
    -> reconcile import/catalog projection
    -> advance clean checkpoint only after classified terminal result

  SceneExternalChangeReconciler
    -> qualify watcher event by SceneSourceIdentity
    -> clean: planned reload
    -> dirty: ConflictDecision
    -> stale/closed: ignore with receipt

zircon_runtime::scene
  explicit scene encode/decode
  atomic target publication
  expected-revision/CAS primitive
  source/artifact revision receipt
  no Editor dirty/history/selection policy

zircon_app
  window/process host only
  requests close and waits for Editor terminal disposition
  never selects save target or replaces authoring World
```

`SceneDocumentKey`至少绑定`ProjectSessionGeneration + canonical source identity + document generation`。`AuthoringWorldSession`是World的owner，不允许watcher、picker或menu callback直接替换裸World。Runtime继续拥有scene encode/decode、atomic publication与World数据结构；Editor拥有authoring session、dirty/history/selection、切换政策和产品消息；App只等待结果。

### 7.1 文档状态机

```text
Preparing
  -> ReadyClean
  -> ReadyDirty
  -> SavePlanning -> Saving
       -> ReadyClean                         PersistedAndProjected
       -> ReadyDirty                         Cancelled/RetryableFailure
       -> PersistedProjectionFailed          source durable, projection stale
       -> Conflict                           expected revision mismatch

ExternalRevisionObserved
  + ReadyClean -> ReloadPlanning -> Preparing -> ReadyClean
  + ReadyDirty -> ConflictDecision
                   -> KeepLocal / ReloadDiscard / SaveAs / MergeUnsupported

TransitionRequested(Open/New/Reload/Close/ProjectSwitch/Exit)
  -> FreezeGeneration
  -> DirtyDecision(Save/Discard/Cancel)
  -> PrepareTarget
  -> CommitWorldAndSession
  -> TerminalReceipt
```

每个terminal receipt必须携带operation id、project/session/document generations、source identity、old/new revision、dirty checkpoint、durability disposition、projection disposition、error category和elapsed stages。任何failure不得通过字符串消息猜测是否已经写盘。

## 8. 分层实施路线

### ED61-M0：固化RED tests并关闭既有P0产品路径

1. 为5条既有P0建立当前产品路径行为测试，不再依赖源码字符串。
2. secondary scene编辑后Save必须只写secondary source，default source bytes保持不变。
3. Open/New/Close Project/window close/watcher reload统一验证Save/Discard/Cancel与Cancel零状态变化。
4. 这一里程碑不引入多文档UI，先消除数据丢失和错误World replacement。

### ED61-M1：Scene identity与session hard cutover

1. 引入`SceneDocumentKey`、`SceneSourceIdentity`、`AuthoringWorldSession`和registry。
2. startup安装default World时同时激活真实default scene session，不再激活project-root伪场景文档。
3. installer改收prepare bundle与返回install receipt；禁止只传`&Scene`。
4. Global scene history迁移到per-document history context，World replacement必须退休旧interaction/selection。

### ED61-M2：保存authority、revision CAS与typed receipt

1. 新增Save Scene/Save As/Save All；Save Project不再隐式决定scene target。
2. Runtime writer增加expected revision/digest precondition与durability receipt。
3. source持久化、reimport/catalog projection和clean checkpoint分阶段提交。
4. `PersistedProjectionFailed`保留dirty/retry信息，禁止伪装完整Saved。

### ED61-M3：统一Open/New/Reload/Close与外部变更

1. 建立prepare/commit/rollback transition coordinator和generation freeze。
2. same-scene Reload/Revert不再被`AlreadyActive`吞掉。
3. watcher按真实active source qualified dispatch，clean可计划重载，dirty进入conflict decision。
4. Project Switch/Exit/Window Close只消费统一ClosePlan，不直接调用manager或replace_world。

### ED61-M4：Scene autosave与crash recovery产品闭环

1. Scene加入统一dirty document enumeration和autosave capture。
2. autosave记录source/base revision、document generation、snapshot schema和operation journal。
3. startup枚举recovery candidates，用户决策后恢复为隔离SceneDocumentSession，再显式保存/丢弃。
4. restore失败保留候选与诊断，不销毁唯一恢复数据。

### ED61-M5：Multi-Scene与workspace恢复

1. 支持多个SceneDocumentSession、active tab、Save All、Close Others和per-scene状态。
2. workspace保存open scene list、active identity、selection/viewports与recovery links。
3. 定义background scene资源预算、LRU/suspension和missing-source remediation。
4. Scene toolkit、animation/UI asset toolkit和未来文档共用统一close/autosave协议。

### ED61-M6：规模、故障与长期工程资格

1. 将snapshot/encode/write/import移出UI临界路径并接入bounded job admission。
2. 建立incremental/chunk-aware候选前先测量clone bytes、peak memory和save阶段耗时。
3. 覆盖disk full、permission、rename/fsync、CAS conflict、import failure、crash point与startup reconciliation。
4. 在相同实体数、组件量、文件大小和存储条件下与参考实现做同语义基准；没有证据不得声称更快。

## 9. 40个资格门

### 9.1 Identity与启动（SD-01至SD-06）

1. **SD-01** 任意installed authoring World都能唯一反查active `SceneDocumentKey`和source identity。
2. **SD-02** 普通startup第一帧即激活manifest default scene，而非project-root伪文档。
3. **SD-03** project/session/document generation不匹配的事件全部fail closed。
4. **SD-04** installer失败时旧World、history、selection、lifecycle和workspace保持字节/语义不变。
5. **SD-05** lifecycle publish只发生在World/session commit成功之后。
6. **SD-06** project切换会退休旧scene interactions、jobs、watchers与recovery writers。

### 9.2 Transition与dirty（SD-07至SD-13）

7. **SD-07** dirty Scene执行Open时必有Save/Discard/Cancel，Cancel零变化。
8. **SD-08** dirty Scene执行New时必有Save/Discard/Cancel，保存失败不得继续New。
9. **SD-09** dirty Scene执行Reload/Revert时必有明确决策，same-scene请求不可变成no-op。
10. **SD-10** Close Scene、Close Project、Project Switch与Exit消费同一ClosePlan。
11. **SD-11** prompt后发生新编辑会使旧plan因generation不匹配失效，而不是误标clean。
12. **SD-12** 多dirty文档批量决策可逐项取消/失败，并给出确定terminal summary。
13. **SD-13** 任意transition重入、重复callback或延迟completion都不能双重commit。

### 9.3 保存与持久化（SD-14至SD-21）

14. **SD-14** secondary scene Save只改变secondary source；default source和manifest保持不变。
15. **SD-15** Save As发布新source identity并原子迁移session，失败保留旧identity。
16. **SD-16** Save All按冻结document generation执行并返回逐文档typed receipt。
17. **SD-17** expected revision不匹配时零覆盖外部文件并进入Conflict。
18. **SD-18** serialize/write失败不推进clean checkpoint或Saved message。
19. **SD-19** source已持久化而reimport失败时返回`PersistedProjectionFailed`，可安全重试projection。
20. **SD-20** Save receipt包含target、revision/digest、bytes、durability和projection disposition。
21. **SD-21** crash在temp write/flush/rename/catalog/import任一注入点后可确定性reconcile。

### 9.4 Autosave与recovery（SD-22至SD-27）

22. **SD-22** Scene dirty进入与toolkit文档相同的bounded autosave scheduler。
23. **SD-23** autosave不覆盖source，且绑定document/source/base revision与schema。
24. **SD-24** clean scene不产生冗余autosave；新编辑generation不会被旧capture标clean。
25. **SD-25** unclean shutdown后startup能枚举、预览、恢复或丢弃Scene候选。
26. **SD-26** restore失败不删除候选，诊断能定位schema/corruption/dependency错误。
27. **SD-27** recovered scene进入隔离dirty session，只有显式Save才覆盖source。

### 9.5 外部变更与schema（SD-28至SD-32）

28. **SD-28** watcher只向匹配source identity与project/session generation的session投递。
29. **SD-29** clean scene外部变化走planned reload；dirty scene进入ConflictDecision。
30. **SD-30** 非default active scene变化可正确检测，default变化不会替换它。
31. **SD-31** scene format具有显式version、migration和unsupported-version诊断。
32. **SD-32** 未知/plugin-owned字段通过load-edit-save往返不丢失，或由明确版本拒绝代替静默删除。

### 9.6 Multi-Scene、规模与诊断（SD-33至SD-40）

33. **SD-33** 两个以上scene同时打开时history、dirty、selection、viewport和save target完全隔离。
34. **SD-34** workspace重启后恢复open scene list、active scene与per-scene UI state。
35. **SD-35** background scene suspend/evict不会丢dirty state或悬挂resource/interaction owner。
36. **SD-36** 100K实体场景save/open不在UI线程执行无界clone、IO或import。
37. **SD-37** job admission具备entry/byte/age预算、cancel、progress和shutdown quiescence。
38. **SD-38** trace能分离capture/encode/write/fsync/import/install各阶段及peak bytes。
39. **SD-39** fault、soak、外部并发写和重复恢复测试在Windows产品路径通过；Linux需求再单独取证。
40. **SD-40** 与Unreal/Godot/Fyrox的比较使用相同场景规模、存储和操作语义，并保存可复核原始证据。

当前40项全部为 **Fail/Not Executed**。静态源码中存在局部单元测试不改变gate状态；只有production路径的行为、故障与规模证据才能关闭资格门。

## 10. 验证矩阵

| 层级 | 必测内容 | 通过条件 |
|---|---|---|
| Unit | SceneDocumentKey、source canonicalization、state machine、ClosePlan、save receipt、CAS | property/transition表全覆盖，stale generation fail closed |
| Component | installer prepare/commit/rollback、save coordinator、external reconciler、autosave capture | 任一注入失败后authority和dirty checkpoint一致 |
| Integration | startup、secondary open-edit-save、New/Reload/Close/Project Switch/Exit | 5条既有P0路径RED转绿，Cancel/失败零越权提交 |
| Recovery | process kill、temp/rename/catalog/import crash points、candidate restore | 可确定恢复或诊断，恢复数据不被失败流程删除 |
| External | 第二进程修改/删除/移动source，watcher乱序/重复事件 | CAS防覆盖，clean/dirty分支正确，非active source不替换World |
| Multi-document | 2/10/100 scenes的open/save-all/close-all/workspace restore | per-scene状态隔离，bounded memory与确定terminal summary |
| Scale | 10K/100K/1M实体、不同组件密度与文件大小 | UI无无界同步阶段，报告P50/P95/P99、peak RSS与写放大 |
| Fault/soak | disk full、permission、fsync/rename/import failure、长时编辑 | 无数据丢失、无双commit、无stuck transition/lease |

## 11. Hard-cut规则与owner约束

1. 禁止保留“如果无active scene则保存manifest default scene”的兼容fallback；identity缺失必须fail closed。
2. 禁止菜单、picker、watcher、window callback直接调用`replace_world`；只能提交transition request。
3. 禁止以`Save Project`继续隐藏Scene save target选择；Project和Scene持久化是两个显式plan participant。
4. 禁止Scene继续依附Global history；迁移完成后旧路径删除，不留双写或re-export shim。
5. 禁止Saved/Opened/Closed只携DocumentId；typed receipt必须携带qualified generations和source disposition。
6. 禁止以字符串错误、toast或日志作为rollback、dirty或durability authority。
7. 禁止把autosave文件当作source覆盖写，也禁止restore失败后自动删除唯一候选。
8. 禁止把Bevy AssetSaver、Unity Graphics Editor调用或Runtime atomic write单独宣称为完整Scene Document产品。
9. 禁止为多Scene UI另建第二套World/selection/history真相；所有pane消费SceneDocumentSession。
10. 禁止在没有同语义benchmark和fault证据时使用“优于Unreal”或“production complete”状态。

## 12. 状态与交付记录

- **Review**：complete，基于2026-08-22 working tree静态源码、focused tests与五引擎显式参考文件。
- **Implementation**：not started。
- **Canonical new P0**：0；current-source确认5项父P0仍Open。
- **Editor61 additions**：28 P1、8 P2、40 qualification gates。
- **Dynamic evidence**：not executed；本轮未运行Cargo或产品程序。
- **Recheck**：实现前必须重取当前source manifest/fingerprint，尤其是Editor02/03/51/57与Runtime61边界。
- **首个实现切片**：ED61-M0，只允许先建立5条P0产品RED tests和统一dirty transition入口；不得从多标签UI或增量序列化开始。
