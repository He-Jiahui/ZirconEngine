---
title: Editor Scene Document、Authoring World、Open/New/Reload/Save/Close、Dirty Transition、Autosave、Recovery 与 Multi-Document Product Integration 当前源码复核
category: zircon_editor
report_id: Editor182
review_date: 2026-08-27
baseline_head: e6bfb5c0240fb62434c4ba86a1dc2525c0434d96
related_code:
  - zircon_editor/src/core/document
  - zircon_editor/src/core/project/scene_document.rs
  - zircon_editor/src/core/recovery
  - zircon_editor/src/ui/workbench/project
  - zircon_editor/src/ui/workbench/startup/editor_state_project.rs
  - zircon_editor/src/ui/workbench/state/scene_document_binding.rs
  - zircon_editor/src/ui/host/editor_manager_project.rs
  - zircon_editor/src/ui/host/editor_scene_document_submission.rs
  - zircon_editor/src/ui/host/project_access.rs
  - zircon_editor/src/ui/host/editor_document_autosave.rs
  - zircon_editor/src/ui/host/editor_save_batch.rs
  - zircon_editor/src/ui/host/project_recovery_decision
  - zircon_editor/src/ui/retained_host/app/autosave.rs
  - zircon_editor/src/ui/retained_host/app/project_close.rs
  - zircon_editor/src/ui/retained_host/app/native_window_close
  - zircon_editor/src/ui/retained_host/app/close_prompt
  - zircon_editor/src/ui/retained_host/app/assets
  - zircon_editor/src/ui/retained_host/app/scene_picker_actions.rs
  - zircon_editor/src/ui/retained_host/app/startup.rs
  - zircon_runtime/src/scene/world/project_io
  - zircon_runtime/src/asset/assets/scene
  - zircon_runtime/src/core/resource/io/atomic_file
tests:
  - zircon_editor/src/core/document/lifecycle/tests.rs
  - zircon_editor/src/core/document/scene_route_tests.rs
  - zircon_editor/src/core/project/tests/scene_document.rs
  - zircon_editor/src/tests/workbench/project/document_roundtrip.rs
  - zircon_editor/src/tests/editor_event/runtime/integration/project.rs
  - zircon_editor/src/tests/host/retained_asset_refresh/scene_reload.rs
  - zircon_editor/src/ui/retained_host/app/tests/close_prompt.rs
  - zircon_editor/src/tests/host/raw_scene_visibility.rs
  - zircon_editor/src/tests/ui/boundary/host_cutover.rs
  - zircon_app/tests/editor_mvp_authoring.rs
plan_sources:
  - docs/plans/optimize/zircon_editor/61-editor-scene-document-authoring-world-open-new-reload-save-close-dirty-transition-autosave-recovery-multi-document-product-integration-current-source-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/172-editor-project-startup-open-create-authority-hub-handshake-session-guard-focus-recent-recovery-current-source-review.md
  - docs/plans/optimize/zircon_editor/181-editor-scene-hierarchy-outliner-tree-projection-expansion-selection-rename-reparent-drag-drop-visibility-lock-multi-world-product-integration-current-source-review.md
  - docs/plans/zircon_editor/editor/03/failure-2026-07-22-transaction-selection-history-wide-snapshot.md
  - docs/plans/zircon_editor/editor/05/failure-2026-07-22-world-inspection-generation-projection.md
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
doc_type: review-and-refactor-plan
refreshes:
  - docs/plans/optimize/zircon_editor/61-editor-scene-document-authoring-world-open-new-reload-save-close-dirty-transition-autosave-recovery-multi-document-product-integration-current-source-review.md
review_status: current_source_refresh_complete
implementation_status: pending
source_recheck_required: true
---

# 182 · Editor Scene Document / Save / Reload / Autosave / Recovery / Multi-Document 当前源码复核

## 1. 结论与状态

Editor61记录的五条父P0数据损失路径在当前源码中均已有实质修复。startup会激活manifest default scene的真实lifecycle identity；Save Project从`ActiveSceneDocumentIdentity`解析精确URI，不再回退default scene；Open/Create在任何写入、catalog变更或World替换前执行dirty admission；Project Close和native window close共用generation-frozen的`PendingClosePrompt`，Save分支真实保存Scene并等待toolkit批处理；watcher按当前scene URI、project generation和activation identity执行后台加载、提交前重验及dirty冲突决策。旧Editor61对当前产品链的描述已经明显过时，不能继续把这五条路径登记为Open。

这些修复还没有把Scene提升为工程级Document。Scene仍不是document toolkit participant，Save All和autosave只枚举UI/Animation等toolkit；recovery虽然已经有逐候选决策、后台executor、恢复副本和失败重试，但产品没有Scene autosave producer，也不会把恢复副本安装为隔离的dirty `SceneDocumentSession`。lifecycle和Workbench仍只持有一个active scene；workspace不保存open scene list或per-scene状态；Scene source没有schema migration、unknown-field无损往返、source/artifact revision和typed save receipt；大场景保存继续在菜单调用链同步clone、encode、write与reimport。

本轮把旧`ED61-P1-21`提升为唯一P0，而不是新增重复缺陷：外部scene变化进入冲突提示后，默认“Save”调用无expected revision/digest的普通保存。提示发布后若另一个进程再次写入，或普通Save与外部writer竞争，Zircon会以atomic replace无条件覆盖较新的外部内容。atomic write只防止半文件，不提供compare-and-swap。这是当前产品可达的数据丢失竞态，必须先于多Scene UI和性能优化关闭。

本轮不新增、不删除其余Editor61 canonical finding，只重判状态：

| 等级 | Open / Fail | Partial | Closed / Pass | Escalated |
|---|---:|---:|---:|---:|
| P0 | 1 | 0 | 0 | 0 |
| P1 | 13 | 8 | 6 | 1（`ED61-P1-21 -> ED182-P0-01`） |
| P2 | 4 | 4 | 0 | 0 |
| Gate | 21 Fail | 12 Partial | 7 Pass | 0 |

本轮为review-only。未修改production Rust，未运行Cargo、Editor、GUI、真实进程崩溃、双进程并发写、100K/1M实体、fault/soak/profile或同负载跨引擎benchmark，因此不能据此宣称性能或表现优于Unreal。Tooling按用户要求排除；本轮也未查询、轮询、等待或实时跟踪协调器。

## 2. 冻结语料与currentness

### 2.1 物理选择集

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | 本轮证据 |
|---|---:|---|
| Editor identity、transition、save与workspace | **35 / 7,409 / 6,732 / 266,781 / 72 / 3** | lifecycle、route、scene document、exact save、startup binding与workspace；fingerprint `dccfe93ffa558c982599a673228f5e9cbcfedd6179eef9cdcb110fbe210cb5c4` |
| Editor close、autosave与recovery | **96 / 15,974 / 14,649 / 562,238 / 123 / 2** | close plan、save batch、autosave catalog/adapter、recovery decision/executor与watcher conflict；fingerprint `9a6040d9fde32f1d9edc26cd6ec5a6aa9d6d1bbaedc7690102792749dd11ad47` |
| Runtime scene persistence与publication support | **38 / 6,544 / 6,039 / 234,575 / 21 / 1** | scene codec、project IO、atomic file、targeted import与generation support；fingerprint `f06cfb50b0446016f45335cc3b4df586e7d9ea6d4aebd7b6f14b84a1c49e1764` |
| 聚焦测试 | **10 / 3,463 / 3,173 / 124,515 / 64 / 0** | lifecycle/route、explicit target、close、watcher plan、MVP save/reopen与boundary guards；fingerprint `6e3901593dccddfab1ea0ce014d1f17c49171c93ffb5433c0bfd204af53f191c` |
| Zircon去重选择集 | **177 / 32,459 / 29,762 / 1,155,294 / 256 / 6** | 当前磁盘静态依赖闭包；fingerprint `2d59ebcf5e7593f7552be8ed4f8f3300ae45059c82a6e55c2f2e8aa47fb4cd88` |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics | **19 / 30,889 / 26,519 / 1,202,936 / 22 / 0** | save/load/autosave、多scene session、asset identity与Editor dirty集成；fingerprint `473482bb873f26ca3c86217f80cd17fa3a444e161abaa898178250fef8c88a2c` |

fingerprint按normalized lowercase relative path的ordinal顺序，将`path + NUL + raw bytes + NUL`串联后计算SHA-256。tests/ignored为词法属性计数，不是执行receipt。冻结时主仓HEAD为`e6bfb5c0240fb62434c4ba86a1dc2525c0434d96`；Godot、Fyrox、Bevy与Unity Graphics参考revision分别为`8c7e6c5877a78e8e61ea4fd42673219a9091dca7`、`8d815db36494f1badb347547dfc7094bf4fbbdf8`、`fb89a8649d9b359e53ffb6e5492ebb7c059ac8af`与`a7e4c051d256a781ab362c64316b125a1e104694`。共享working tree存在其他在途修改，本报告以当前磁盘为事实源，实施前必须重算选择集。

### 2.2 当前产品链

```text
project activation
  -> project/session guard + activation ledger
  -> load default Scene into prepared authoring World
  -> activate_startup_scene_document(default URI)
  -> bind DocumentId to HistoryContextId::Document

picker Open/Create
  -> validate ScenePickerTicket(project session)
  -> prepare_scene_transition() rejects dirty Scene
  -> load/create + staging/catalog compensation
  -> reserve scene lifecycle identity
  -> bind document journal
  -> install authoring World
  -> infallible lifecycle commit + document binding

Save Project
  -> capture active per-document history token
  -> snapshot live Scene
  -> resolve exact ActiveSceneDocumentIdentity.scene_uri
  -> save workspace, encode Scene, atomic replace source
  -> best-effort reimport/catalog/watcher refresh
  -> mark history clean only if token remains current

asset watcher active-source change
  -> qualify active scene identity + project generation
  -> bounded background load
  -> commit-time generation/source/identity recheck
  -> clean: reload; dirty: Save / Discard / Keep Editing decision

autosave / Save All
  -> dirty_document_toolkits()
  -> bounded save/autosave jobs
  -> active Scene is absent

Project/Window Close
  -> dirty toolkits + separate dirty Scene generation
  -> common PendingClosePrompt
  -> Scene synchronous save + toolkit background save
  -> recheck generations before terminal close
```

### 2.3 已有基础必须保留

1. 保留`ActiveSceneDocumentIdentity { document, project_root, scene_uri, activation_revision }`与project-session-qualified picker ticket；任何修复不得恢复manifest default fallback。
2. 保留scene route的dirty-before-side-effect、lifecycle reservation、journal binding、create staging与catalog/source compensation。
3. 保留per-document `HistoryContextId::Document`、save token和`mark_saved_if_unchanged`；Scene不能退回Global history。
4. 保留`PendingClosePrompt`的dirty generation冻结、owner-exclusive dirty save batch和完成后重检。
5. 保留active-source-qualified watcher、project generation token、后台load、commit revalidation、conflict coalescing与bounded admission retry。
6. 保留autosave catalog/source digest/provenance、recovery逐候选显式决策、恢复副本不覆盖source、失败候选保留和安全子集重试。
7. 保留Runtime显式scene URI、serialize-before-publication和single-file atomic replace，但增加CAS与typed durability receipt。
8. 保留现有行为测试；`include_str!`形状断言只能作boundary guard，不能替代真实产品、并发和故障测试。

## 3. 旧父P0的current-source校正

| canonical owner | 旧风险 | 当前源码证据 | Editor182裁决 |
|---|---|---|---|
| Editor03 P0-01 | secondary scene Save Project写manifest default | `EditorManager::save_active_scene`只读取`active_scene_identity`并把其URI传给host；测试验证explicit target不改变default bytes | **Closed in current source**；父报告更新时应吸收此证据 |
| Editor03 P0-02 | dirty Open/Create直接替换World并清history | `SceneDocumentRoute::open/create`在load/create前调用`prepare_scene_transition`；dirty route测试证明install/catalog/source均未发生 | **Closed as data-loss path**；产品Save/Discard/Cancel仍由P1-07/08约束 |
| Editor02 P0-02 | Close Project绕过dirty gate | `request_project_close`收集dirty toolkits和Scene generation，只有全clean才进入commit | **Closed in current source** |
| Editor02 P0-03 | native close Save未实现 | main/native close共用`PendingClosePrompt`；Save先保存Scene，再调度toolkit batch，完成后重检 | **Closed in current source** |
| Editor57 P0-01 | default watcher覆盖非default current World | refresh以`active_scene_identity_for_session().scene_uri()`规划；reload捕获identity/generation并在提交前重验 | **Closed in current source** |

这里的Closed是Editor182对当前磁盘的源码裁决，不替代父owner自己的报告刷新。五项在跨报告汇总中仍只能由Editor02/03/57各自计数一次。

## 4. P0：当前可达

### ED182-P0-01 · Open · Scene保存缺少expected revision，外部更新可被无条件覆盖

该条是`ED61-P1-21`的严重性升级，不是新建重复finding。

`ActiveSceneReloadConflict`保存的是active scene identity与`ProjectAssetGenerationToken`。冲突提示默认选项为Save；选择后`save_active_scene_reload_conflict()`直接调用`save_project_scene()`。该调用最终到`World::save_scene_to_project(project, uri)`，只解析当前路径、序列化并`atomic_write`。整条链没有captured source digest、mtime/file identity、expected revision、compare-and-swap、write lease或冲突terminal result。

因此以下产品路径均可覆盖外部新内容：

1. watcher观察外部版本B并发布冲突提示；提示等待期间外部writer发布版本C；用户选择Save，Zircon以本地版本A覆盖C。
2. 普通Save在snapshot后、atomic replace前遇到外部writer，无任何precondition阻止last-writer-wins。
3. Save完成后的“reload latest source”不能恢复被覆盖的版本，因为被重新加载的已经是Zircon刚写入的内容。

必须把`SceneSourceRevision`作为open/reload/save session状态，而不是临时日志信息。Runtime publication primitive至少接收`ExpectedSourceRevision::{Missing, Digest, FileIdentityAndDigest}`并返回`SceneSaveReceipt::{Persisted, Conflict, PersistedProjectionFailed}`。冲突时source零变化、clean checkpoint不推进、Saved message不发布；用户若明确选择覆盖，也必须基于重新读取的最新revision生成一次性`OverwriteLease`，不能复用旧提示的裸Save分支。

最小RED矩阵必须覆盖：提示后第二次外部写、普通Save与外部writer交错、same-size/same-mtime内容变化、source delete/recreate、symlink/reparse point替换、atomic rename前后故障，以及Conflict后重试。没有这些动态证据，不得把atomic write描述为并发安全保存。

## 5. P1：状态与重构要求

| Finding | 状态 | 当前源码事实 | 必须重构为 |
|---|---|---|---|
| ED61-P1-01 startup scene identity | **Closed** | `build_startup_state`从manifest default URI调用`activate_startup_scene_document`并绑定真实DocumentId | 保留exact identity，补startup整体事务 |
| ED61-P1-02 startup publish/rollback顺序 | **Partial** | startup World先prepare、再scene activate；project session/activation ledger已存在，但retained state构造与scene binding失败仍没有共同rollback receipt | activation ledger覆盖World/session/document/workbench first-present的共同terminal |
| ED61-P1-03 单active lifecycle | **Open** | state仍只有`active_document`、`active_scene_key`和单World | `SceneDocumentSessionRegistry` + active tab/view binding |
| ED61-P1-04 authoritative active source getter | **Closed** | lifecycle提供`active_scene_identity(_for_session)`，Save与watcher共同使用 | 保留getter并纳入source revision |
| ED61-P1-05 installer丢document metadata | **Partial** | installer已接`ProjectSceneDocument`而非裸Scene，但document仍只有URI/path/world | prepared install bundle携revision、schema、artifact freshness和diagnostics |
| ED61-P1-06 prepare/commit/rollback transaction | **Partial** | route gate、reservation和post-install infallible lifecycle commit成立；`replace_world`及其后settings/state步骤仍可能部分提交且无旧World rollback | one-shot transition plan + old session restore receipt |
| ED61-P1-07 same-scene Reload/Revert | **Open** | Open命中active URI直接`AlreadyActive`；只有watcher reload，没有用户Reload/Revert命令 | 显式Reload/Revert transition，dirty decision后执行 |
| ED61-P1-08 Scene save product commands | **Partial** | Save Project能保存exact active Scene，Save All Documents存在但只覆盖toolkits；Save As/Save All Scenes/Close Scene仍缺 | Scene作为统一participant，提供Save/Save As/Save All/Close/Reload |
| ED61-P1-09 typed document save message | **Open** | `DocumentMessage::{Opened,Saved,Closed}`仍只携DocumentId | qualified operation/session/source/revision/durability receipt |
| ED61-P1-10 persisted-vs-projected terminal | **Partial** | durable source后reimport/refresh失败不再伪装整个保存失败，而是写diagnostic并返回成功 | 显式`PersistedProjectionFailed`，可单独重试projection |
| ED61-P1-11 source/artifact/base revision | **Open** | `ProjectSceneDocument`仍只有scene_uri/source_path/world | `SceneSourceIdentity + SourceRevision + ArtifactRevision + BaseDigest` |
| ED61-P1-12 per-document history | **Closed** | active Scene绑定`HistoryContextId::Document(document)`；replace/reload按该history清理 | 多session后仍保持逐文档隔离 |
| ED61-P1-13 Scene toolkit/participant registration | **Open** | Scene仍不在`dirty_document_toolkits()`；close path以独立字段特判 | 通用`DocumentParticipant`协议，不再双路径 |
| ED61-P1-14 close与autosave可见性 | **Partial** | Project/main-window close已显式包含Scene generation；autosave和Save All仍看不到Scene | 所有产品操作消费同一participant snapshot |
| ED61-P1-15 Scene autosave identity/capture | **Open** | autosave request只由toolkit的source path/capture生成 | Scene immutable snapshot + source/base revision + bounded request |
| ED61-P1-16 recovery product executor | **Partial** | 产品已有逐候选Decision、background executor、recovered/comparison copy和retry；但Scene没有producer，恢复结果也不安装Scene session | recovered copy验证后打开为隔离dirty Scene session |
| ED61-P1-17 workspace scene restore | **Open** | `ProjectEditorWorkspace`只存layout、view instances、focus与drawers | open scenes、active scene、per-scene selection/viewport/recovery link |
| ED61-P1-18 generation-frozen close plan | **Closed** | `PendingClosePrompt`捕获toolkit与Scene generation，discard/save completion都重检，new dirty刷新决策 | 将同一合同扩展到Close Scene/Project Switch |
| ED61-P1-19 watcher dirty conflict state | **Closed** | clean reload与dirty Save/Discard/Keep Editing、coalescing、stale identity处理均存在 | 保存选择必须接CAS；补Save As/merge语义 |
| ED61-P1-20 watcher真实active source | **Closed** | refresh按active scene URI匹配，reload捕获identity与generation | 保留source-qualified dispatch |
| ED61-P1-21 expected revision/digest CAS | **Escalated** | 仍完全缺失，并已形成P0可达覆盖路径 | 由`ED182-P0-01`唯一计数 |
| ED61-P1-22 create crash journal | **Open** | staging、hard-link publish和进程内rollback存在；无durable create operation journal/startup reconciliation | durable prepare/publish/catalog/install journal与幂等恢复 |
| ED61-P1-23 scene schema/migration/unknown fields | **Open** | Scene TOML无显式version；reference mapper暂存`_rest`，但decode为SceneAsset/World再encode会丢unknown/plugin-owned字段 | versioned envelope、migration、extension payload preservation或fail closed |
| ED61-P1-24 source/artifact/installed generation proof | **Open** | load通过artifact，watcher只有project generation；没有source digest与artifact revision一致性receipt | freshness proof贯穿open/reload/save/install |
| ED61-P1-25 multi-scene registry | **Open** | lifecycle、EditorState、World和scene binding均为单active | 多Scene session registry、tab/split/background policy |
| ED61-P1-26 save/open规模与UI阻塞 | **Open** | menu同步snapshot/encode/write/reimport；open installer再次clone完整Scene | immutable snapshot handoff、bounded job、UI terminal polling |
| ED61-P1-27 typed errors | **Open** | core route有typed enum，但host/retained边界广泛压成String；save结果只有Ok/Err | stable error category、stage、retryability与commit disposition |
| ED61-P1-28 product RED matrix | **Partial** | explicit target、dirty route rejection、generation close、watcher active-source planning及MVP save/reopen已有测试 | 补完整产品、CAS、recovery、multi-scene、fault与scale矩阵 |

## 6. P2：长期工程化与效率差距

| Finding | 状态 | 当前源码事实 | 长期目标 |
|---|---|---|---|
| ED61-P2-01 weak DocumentId derivation/retention | **Open** | path/FNV、collision probing和1,024 retention仍在，identity不含project/session epoch | opaque monotonic/session-qualified document key |
| ED61-P2-02 recent/pinned/missing scenes | **Open** | 有recent projects和scene picker，没有recent scenes、pin或missing-source remediation | workspace级scene navigation与repair |
| ED61-P2-03 background save control | **Partial** | toolkit Save All有bounded job、owner和source mutex；Scene保存仍同步且不可取消 | Scene也进入priority/cancel/progress/quiescence调度 |
| ED61-P2-04 incremental/chunk save | **Open** | full World -> full SceneAsset -> full TOML | dirty page/component、partition/chunk与copy-on-write snapshot |
| ED61-P2-05 save/load telemetry | **Partial** | 有save generation日志、watcher counters与部分profile scope；无encode/write/fsync/import/peak分段 | typed stage spans、bytes、P50/P95/P99与peak RSS |
| ED61-P2-06 crash breadcrumb | **Partial** | project activation ledger和per-document transaction journal存在；scene save/transition没有完整operation breadcrumb | unified transition/save journal与startup explanation |
| ED61-P2-07 collaborative/source-control policy | **Open** | 无checkout、read-only lease、external writer protocol或multi-user authority | CAS之上的SCM/multi-user ownership policy |
| ED61-P2-08 accessible dirty/conflict/recovery UX | **Partial** | close prompt、reload conflict Decision和逐候选recovery Decision已存在 | multi-document summary、failure focus、diff/preview与accessible status |

## 7. 五引擎参考裁决

| 参考 | 本轮直接证据 | Zircon应吸收 | 禁止误用 |
|---|---|---|---|
| Unreal | `SaveMap/SaveWorld`显式接收World与target；LoadMap前调用dirty package保存并允许中止；SaveWorld有非重入保护；PackageAutoSaver维护dirty/saved与restore状态 | explicit target、non-reentrant save、dirty transition gate、autosave/restore journal、分阶段save output | 不能把atomic rename或一个Save菜单冒充Package级durability |
| Godot | `EditorData::EditedScene`持有root/path/modified time/editor state/selection/history/custom state/history id；`Vector<EditedScene>`支持多scene与切换/恢复 | per-scene session、history/selection/state、external modified time与多scene container | 不照搬global singleton和单线程实现细节 |
| Fyrox | `EditorSceneEntry`持有id/path/unsaved；`SceneContainer`统计多个unsaved scene；File菜单有Save/Save As/Save All/Close，dirty close通过deferred confirmation action | Scene participant、Save All、Close Scene和延迟执行的dirty decision | 不以消息枚举代替typed generation/receipt |
| Bevy | `AssetPath`明确source/path/label；`AssetSource`分离reader/writer/watcher/processed source；typed saver围绕asset path | typed source identity与读写/watch职责隔离 | Bevy AssetSaver不是完整Editor document/session/recovery产品 |
| Unity Graphics | 本地公开范围能证明Undo、SetDirty、MarkSceneDirty、SceneManager多scene枚举和scene-open consumer | 只吸收Editor mutation必须接dirty/undo与scene identity的消费边界 | Graphics仓不含Unity专有完整Scene lifecycle，不能外推其内部实现 |

参考路由保持：Unreal负责save/load/autosave durability基线；Godot/Fyrox负责多Scene authoring与交互；Bevy只用于source identity；Unity Graphics只作Editor API消费和多scene观察的局部证据。

## 8. 目标authority与状态机

```text
ProjectSessionGeneration
  -> SceneDocumentSessionRegistry
       -> SceneDocumentSession
            SceneDocumentKey
            SceneSourceIdentity + ExpectedSourceRevision
            AuthoringWorldLease
            HistoryContext + clean checkpoint
            Selection / viewport / interaction state
            Autosave / recovery state
            ExternalRevisionState

DocumentTransitionCoordinator
  -> capture participant generations
  -> Save / Discard / Cancel plan
  -> prepare load/create/reload/close
  -> commit World + lifecycle + workspace
  -> typed terminal receipt

DocumentSaveCoordinator
  -> immutable Scene snapshot
  -> compare expected source revision
  -> encode + durable publication
  -> project catalog/artifact
  -> advance clean checkpoint only for matching generation
  -> return persisted / conflict / persisted-projection-failed
```

```text
ReadyClean -> edit -> ReadyDirty

ReadyDirty -> SavePlanning(expected revision)
  -> Conflict                         source unchanged
  -> PersistedProjected               clean checkpoint advances
  -> PersistedProjectionFailed        source durable, projection retryable
  -> RetryableFailure / Cancelled     remains dirty

ExternalRevisionObserved
  + ReadyClean -> ReloadPlanning -> ReadyClean
  + ReadyDirty -> ConflictDecision
                   -> KeepLocal
                   -> ReloadDiscard
                   -> SaveAs
                   -> ExplicitOverwrite(new one-shot lease)

Open/New/Reload/Close/Switch/Exit
  -> Freeze participant generations
  -> DirtyDecision
  -> Prepare
  -> Commit or rollback
  -> TerminalReceipt
```

Runtime只拥有scene codec、source revision observation、CAS publication和artifact projection primitive。Editor拥有document session、dirty/history/selection、transition policy、autosave/recovery与用户决策。App只请求关闭并等待terminal disposition，不能选择scene URI或直接替换World。

## 9. 分层重构路线

### ED182-M0：先关闭P0 CAS数据丢失

1. 定义`SceneSourceRevision`和跨进程可比较的digest/file identity。
2. open/reload把observed revision写入`ProjectSceneDocument`和active session。
3. save必须提交expected revision；不匹配返回Conflict且零写入。
4. watcher冲突的Save改为Save As或重新读取后的一次性explicit overwrite lease。
5. 建立双进程/第二writer/fault RED tests后才允许进入下一层。

### ED182-M1：Scene成为统一DocumentParticipant

1. Scene注册到dirty/save/close/autosave统一participant snapshot。
2. 删除`dirty_project_scene_generation`等Scene特判和同步save分支。
3. Save Project拆为project workspace plan与Scene participant save；Save All覆盖Scene。
4. 引入typed per-document save receipt与`PersistedProjectionFailed`。

### ED182-M2：统一transition与recovery

1. Open/New/Reload/Revert/Close/Project Switch/Exit消费同一generation-frozen plan。
2. startup、route与World replacement接共同prepare/commit/rollback receipt。
3. Scene进入bounded autosave capture；snapshot携source/base revision与schema。
4. recovered copy经schema/reference验证后安装为隔离dirty Scene session。

### ED182-M3：Multi-Scene与workspace

1. 引入`SceneDocumentSessionRegistry`，支持tab、split、Save All与Close Others。
2. 每session拥有World/history/selection/viewport/interaction/autosave状态。
3. workspace保存open scene list、active identity和恢复链接。
4. 定义后台Scene suspend/evict/resource budget与missing-source remediation。

### ED182-M4：schema、规模与长期资格

1. Scene source增加显式version/migration和plugin-owned payload preservation。
2. snapshot/encode/write/import移出UI临界路径并支持cancel/progress/quiescence。
3. 先测量clone bytes、peak RSS与阶段耗时，再决定incremental/chunk/partition设计。
4. 执行Windows fault/soak/100K/1M与同语义参考benchmark；Linux特有需求再单独取证。

## 10. 40个资格门current-source裁决

| Gate | 状态 | 当前裁决 |
|---|---|---|
| SD-01 World可反查qualified Scene key/source | **Partial** | manager有active identity，state有DocumentId；两者与gateway World不是一个原子session快照 |
| SD-02 startup首帧激活default Scene | **Pass** | startup明确activate并bind default scene identity |
| SD-03 stale project/session/document generation fail closed | **Partial** | picker/watcher/save身份已qualified；DocumentMessage及多条host event仍裸DocumentId |
| SD-04 installer失败零状态变化 | **Fail** | World/settings/state后半段仍缺完整rollback |
| SD-05 lifecycle publish只在World/session commit后 | **Partial** | route满足；startup/project activation仍无共同terminal receipt |
| SD-06 project switch退休scene jobs/watchers/interactions/recovery | **Partial** | close会cancel reload/stop watcher；全部interaction与writer未统一清点 |
| SD-07 dirty Open有Save/Discard/Cancel | **Fail** | 当前只fail closed并显示错误 |
| SD-08 dirty New有Save/Discard/Cancel | **Fail** | 当前只fail closed并显示错误 |
| SD-09 dirty Reload/Revert明确决策且same-scene非no-op | **Partial** | watcher conflict成立；用户Reload/Revert缺失，Open same scene仍no-op |
| SD-10 Close Scene/Project/Switch/Exit共用ClosePlan | **Partial** | Project/main-window共用prompt；Close Scene与Switch未接入 |
| SD-11 prompt后新编辑使旧plan失效 | **Pass** | Scene/toolkit generation均在terminal前重检 |
| SD-12 多dirty文档逐项失败/取消terminal summary | **Partial** | toolkit batch有逐文档结果；Scene仍同步特判且无统一summary |
| SD-13 transition重入/迟到completion不双commit | **Partial** | route gate、save owner和reload generation有保护；统一operation id/receipt缺失 |
| SD-14 secondary Save不改变default source | **Pass** | exact active URI链与explicit-target测试成立 |
| SD-15 Save As原子迁移identity | **Fail** | 无Scene Save As |
| SD-16 Save All冻结generation并返回逐文档receipt | **Partial** | toolkit满足部分合同，Scene被排除 |
| SD-17 revision mismatch零覆盖并进入Conflict | **Fail** | 无CAS，形成ED182-P0-01 |
| SD-18 serialize/write失败不推进clean/Saved | **Pass** | save成功后才mark token与publish Saved |
| SD-19 projection失败返回typed persisted disposition | **Fail** | 仅日志后返回普通成功 |
| SD-20 save receipt含target/revision/bytes/durability/projection | **Fail** | 无此receipt |
| SD-21 crash point可确定reconcile | **Fail** | scene save/create缺完整operation journal |
| SD-22 Scene进入bounded autosave scheduler | **Fail** | scheduler只枚举toolkit |
| SD-23 Scene autosave绑定source/base revision/schema | **Fail** | 无Scene autosave request |
| SD-24 clean Scene不冗余autosave且旧capture不标clean | **Fail** | Scene尚未接入 |
| SD-25 startup枚举/预览/恢复Scene候选 | **Fail** | 通用候选UI存在，但Scene没有producer |
| SD-26 restore失败保留候选和typed诊断 | **Partial** | 通用executor/service满足；未覆盖真实Scene candidate产品链 |
| SD-27 recovered Scene进入隔离dirty session | **Fail** | 只materialize recovered copy |
| SD-28 watcher按source identity/session generation投递 | **Pass** | active identity + project generation + commit recheck成立 |
| SD-29 clean reload、dirty conflict | **Pass** | 两分支及Decision均已实现 |
| SD-30 nondefault active变化正确，default不误替换 | **Pass** | plan使用active URI而非manifest default |
| SD-31 scene format version/migration | **Fail** | TOML Scene authoring document无version/migration |
| SD-32 unknown/plugin fields无损或明确拒绝 | **Fail** | `_rest`在decode到SceneAsset/World后丢失 |
| SD-33 多Scene history/dirty/selection/target隔离 | **Fail** | 单active World/session |
| SD-34 workspace恢复open scenes与per-scene state | **Fail** | workspace schema无scene session字段 |
| SD-35 background Scene suspend/evict不丢状态 | **Fail** | 无background Scene |
| SD-36 100K Scene无UI线程无界clone/IO/import | **Fail** | 当前同步全量链 |
| SD-37 job admission/byte/age/cancel/progress/quiescence | **Partial** | reload/toolkit job有底座；Scene save没有 |
| SD-38 trace分离capture/encode/write/fsync/import/peak | **Partial** | 有局部日志/scope/counter，缺完整阶段和peak bytes |
| SD-39 Windows fault/soak/external race测试 | **Fail** | 本轮未执行，现有测试也不覆盖完整矩阵 |
| SD-40 同语义跨引擎benchmark | **Fail** | 无可复核证据 |

## 11. 验证矩阵与Hard-cut规则

| 层级 | 必测内容 | 通过条件 |
|---|---|---|
| Unit | source revision、CAS、save receipt、participant generation、transition table | stale/overflow/duplicate均typed fail closed |
| Component | save coordinator、World install rollback、projection retry、Scene autosave capture | 每个注入失败后的source/session/dirty一致 |
| Integration | startup、secondary open-edit-save、Open/New/Reload/Close/Switch/Exit | exact target，Cancel零变化，失败无越权commit |
| External writer | 第二进程write/delete/recreate/rename，提示前后连续两次变化 | 任何非显式新lease都不能覆盖外部revision |
| Recovery | kill at capture/write/rename/import/install，candidate restore/compare/discard | source不被自动覆盖，失败候选保留，恢复session可继续保存 |
| Multi-document | 2/10/100 Scenes，Save All/Close All/workspace restart | per-session状态隔离且terminal summary确定 |
| Scale | 10K/100K/1M实体，不同组件密度与文件大小 | UI无无界同步阶段，报告P50/P95/P99与peak RSS |
| Fault/soak | disk full、permission、fsync/rename/import失败、长时编辑 | 无数据丢失、双commit、stuck lease或遗失recovery |

Hard-cut约束：

1. 禁止把atomic write称为CAS；expected revision不匹配必须在publication前终止。
2. 禁止保留manifest default save fallback；active identity缺失必须fail closed。
3. 禁止继续以Scene特判字段拼接toolkit save/close/autosave；统一participant后删除旧双路径。
4. 禁止picker、watcher、menu或window callback直接替换裸World；只能提交qualified transition plan。
5. 禁止恢复副本自动覆盖source；恢复必须先进入隔离dirty session。
6. 禁止`DocumentMessage::Saved`继续作为durability authority；它必须引用typed terminal receipt。
7. 禁止以String/toast/log推断是否已经写盘、是否可重试或是否冲突。
8. 禁止为Multi-Scene另建第二套history/selection/World truth；session registry是唯一owner。
9. 禁止先做增量serialization再测量真实瓶颈；先建立阶段与内存证据。
10. 禁止在没有fault、scale和同语义benchmark时声明“优于Unreal”或“production complete”。

## 12. 状态与交付记录

- **Review**：complete，基于2026-08-27当前磁盘、177个Zircon文件与19个显式参考文件的静态复核。
- **Implementation**：pending；本轮没有修改production Rust。
- **Current parent P0 correction**：Editor61所列5条父P0路径均在当前源码关闭，父owner后续应吸收证据。
- **Current canonical P0**：1 Open；`ED61-P1-21`升级为`ED182-P0-01`，不重复计数。
- **Editor61 P1 refresh**：13 Open / 8 Partial / 6 Closed / 1 Escalated。
- **Editor61 P2 refresh**：4 Open / 4 Partial / 0 Closed。
- **Qualification gates**：21 Fail / 12 Partial / 7 Pass。
- **Dynamic evidence**：not executed；未运行Cargo、Editor、并发writer、crash/fault、scale/soak/profile或benchmark。
- **第一实现切片**：ED182-M0，仅关闭source revision/CAS与冲突保存数据丢失；不得从多标签UI、增量serialization或视觉包装开始。
