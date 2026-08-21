---
related_code:
  - zircon_runtime/src/core/resource/io/atomic_file/transaction.rs
  - zircon_runtime/src/core/resource/io/transaction/schema.rs
  - zircon_runtime/src/core/resource/io/transaction/engine.rs
  - zircon_runtime/src/core/resource/io/transaction/journal.rs
  - zircon_runtime/src/core/resource/io/transaction/recovery.rs
  - zircon_runtime/src/core/resource/io/transaction/owner_lock.rs
  - zircon_runtime/src/core/resource/io/transaction/commit.rs
  - zircon_runtime/src/core/resource/io/transaction/stage.rs
  - zircon_runtime/src/core/resource/io/transaction/observation.rs
  - zircon_runtime/src/asset/migration/transaction.rs
  - zircon_runtime/src/asset/migration/transaction/journal_owner.rs
  - zircon_runtime/src/asset/migration/transaction/recovery.rs
  - zircon_runtime/src/asset/project/manager/durable_transaction.rs
  - zircon_runtime/src/scene/world/transaction.rs
  - zircon_runtime/src/scene/world/typed_api/bundle_transaction.rs
  - zircon_runtime/src/scene/world/typed_api/bundle_transaction/deferred_bundle_commit.rs
  - zircon_runtime/src/scene/world/typed_api/bundle_transaction/deferred_bundle_removals.rs
  - zircon_runtime/src/scene/world/typed_api/bundle_transaction/deferred_bundle_staging.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/spawn/transaction.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/spawn/preflight_mutation.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/spawn/commit.rs
  - zircon_runtime/src/scene/dynamic_scene/session/io/atomic.rs
  - zircon_runtime/src/scene/ecs/commands/command_queue.rs
  - zircon_runtime/src/scene/world/commands.rs
  - zircon_runtime/src/core/runtime/handle/activation.rs
  - zircon_runtime/src/core/runtime/handle/activation/batch.rs
  - zircon_runtime/src/core/runtime/handle/activation/module_lifecycle.rs
  - zircon_runtime/src/core/runtime/handle/activation/service_lifecycle.rs
  - zircon_runtime/src/core/runtime/handle/runtime_extensions.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/hot_reload.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/lifecycle.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/loading.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator.rs
  - zircon_runtime/src/ui/template/asset/hot_reload_executor.rs
  - zircon_runtime/src/ui/template/asset/hot_reload_plan.rs
  - zircon_runtime/src/core/framework/net/http.rs
  - zircon_runtime/src/core/framework/net/rpc.rs
  - zircon_runtime/src/graphics/pipeline/declarations/compiled_render_pipeline.rs
  - zircon_runtime/src/graphics/pipeline/render_pipeline_asset/compile.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/camera_loop.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_editor/src/core/editing/engine/transaction/lifecycle.rs
  - zircon_editor/src/core/editing/engine/transaction/operation_group.rs
  - zircon_editor/src/core/editing/engine/history.rs
  - zircon_editor/src/core/editing/engine/journal.rs
  - zircon_editor/src/core/project/scene_document.rs
  - zircon_editor/src/core/document/scene_route.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/enablement/capabilities.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/enablement/project.rs
  - zircon_editor/src/ui/host/editor_manager_plugins_export/enablement/native.rs
  - zircon_plugins/physics/runtime/src/skeletal/profile.rs
  - zircon_plugins/sound/runtime/src/service_types/mixer_graph/configuration.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager/dispatch.rs
  - zircon_plugins/net/features/rpc/runtime/src/manager/state.rs
  - zircon_hub/src/projects/create_project.rs
  - zircon_hub/src/process/editor_focus/publish.rs
  - zircon_app/src/entry/runtime_entry_app/frame_capture.rs
  - tools/session_coordinator/database.py
  - tools/session_coordinator/command_requests.py
  - tools/session_coordinator/offline_queue.py
  - tools/session_coordinator/git_finalize.py
  - tools/session_coordinator/workspace_copy.py
  - tools/session_coordinator/workspace_copy_terminal.py
tests:
  - zircon_runtime/src/core/resource/io/transaction/engine/tests.rs
  - zircon_runtime/src/core/resource/io/transaction/recovery/tests.rs
  - zircon_runtime/src/asset/tests/migration/project_commandlet/crash_windows.rs
  - zircon_runtime/src/asset/tests/migration/project_commandlet/transaction_recovery.rs
  - zircon_runtime/src/scene/tests/ecs_typed_api/bundle_transactions.rs
  - zircon_runtime/src/script/vm/runtime/hot_reload_coordinator/tests/unload_atomicity.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/hot_reload_failures.rs
  - zircon_runtime/src/plugin/native_plugin_loader/native_plugin_live_host/tests/hot_reload_publication.rs
  - zircon_editor/src/tests/editing/transaction_engine/operation_group.rs
  - zircon_editor/src/core/document/scene_route_tests.rs
  - tools/session_coordinator/tests/test_command_protocol.py
  - tools/session_coordinator/tests/test_git_finalize.py
plan_sources:
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/coverage.md
  - docs/plans/optimize/zircon_runtime/01-core-runtime-lifecycle-registry-review.md
  - docs/plans/optimize/zircon_runtime/04-core-resource-asset-serialization-review.md
  - docs/plans/optimize/zircon_runtime/05-scene-ecs-world-lifecycle-review.md
  - docs/plans/optimize/zircon_runtime/07-script-plugin-runtime-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
  - docs/plans/optimize/zircon_hub/01-project-engine-build-editor-launch-process-persistence-delivery-review.md
  - docs/plans/optimize/zircon_tooling/06-session-coordinator-control-plane-leases-validation-artifacts-finalize-supervision-review.md
  - docs/plans/optimize/zircon_tooling/09-release-channel-artifact-repository-install-update-rollback-operations-review.md
  - docs/plans/optimize/zircon_tooling/23-failure-contract-panic-unwind-error-propagation-poison-recovery-result-observability-review.md
  - docs/plans/optimize/zircon_tooling/24-concurrency-locking-atomic-ordering-blocking-thread-lifecycle-backpressure-deadlock-review.md
  - docs/plans/optimize/zircon_tooling/27-version-domain-schema-compatibility-support-window-migration-deprecation-upgrade-downgrade-review.md
  - docs/plans/optimize/zircon_tooling/31-declarative-project-asset-ui-scene-manifest-schema-generated-artifact-physical-authority-review.md
  - docs/plans/optimize/zircon_tooling/32-hot-path-catalog-algorithmic-complexity-data-movement-batching-cache-locality-performance-governance-review.md
  - docs/plans/optimize/zircon_tooling/33-reference-engine-source-corpus-snapshot-provenance-citation-applicability-comparison-currentness-review.md
  - docs/plans/optimize/zircon_tooling/35-ownership-graph-shared-weak-borrow-lease-callback-subscription-raii-cycle-detach-leak-isolation-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Classes/Editor/TransBuffer.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Public/ScopedTransaction.h
  - dev/UnrealEngine/Engine/Source/Runtime/CoreUObject/Private/UObject/SavePackage2.cpp
  - dev/bevy/crates/bevy_ecs/src/world/command_queue.rs
  - dev/bevy/crates/bevy_ecs/src/error/command_handling.rs
  - dev/Fyrox/editor/src/command/mod.rs
  - dev/godot/core/object/undo_redo.h
  - dev/godot/editor/editor_undo_redo_manager.h
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/RenderGraph.cs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/Runtime/RenderGraph/Compiler/NativePassCompiler.cs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 37 · Transaction Atomicity、Prepare/Commit/Publish、Rollback/Compensation、Idempotency 与 Crash Recovery 审查

## 1. 结论

Zircon并不缺“事务”代码。Durable resource I/O已经实现版本化intent journal、owner lock、staging与backup证据、逐文件`Committing`状态、持久commit point、目录同步、幂等rollback、重启recovery、deferred cleanup以及覆盖破坏窗口的fault injection；asset migration和project generation也已复用这套内核。动态场景发布、World bundle insertion和部分结构命令又把可失败工作前移到preflight，最终publication尽量压缩为不可失败内存段。Session Coordinator的SQLite transaction、request fingerprint和terminal replay同样是可保留的工程基线。

问题在于这些成熟局部能力尚未成为跨域统一合同。Editor scene route把文件、catalog、authoring world和document lifecycle依次发布，最后activation失败时没有恢复前三者；UI asset hot reload先驱逐compile cache、刷新resolver/theme，再可能因surface mutation失败返回错误；Editor plugin enablement先改capability/runtime state再改manifest，外层native completion和status publication没有统一rollback receipt；runtime module activation把module标成`Running`后才通知observer，observer panic会进入失败分支，但reset只恢复仍为`Initializing`的module；Session archive用进程内`OnceLock<HashMap>`仲裁revision，写入只`flush`不`sync_all`，target先改名为backup后没有持久journal或restart recovery；Hub create project也有同类directory rename crash window。

网络和外部副作用把风险进一步放大。RPC虽然有`NetRequestId`、timeout、queue和quota，却没有明确的idempotency class、dedup store、result replay或at-most-once/at-least-once合同；HTTP公开`max_retry_attempts`，但request没有method/body replay safety或idempotency key。Coordinator能保证数据库内request admission和结果原子化，却不能把Git commit、文件复制、进程启动或通知纳入SQLite事务；崩溃后将accepted request标为interrupted，仍不能证明外部动作未发生、已发生或部分发生。GPU submit同样不可回滚，正确策略必须是提交前完整编译/验证、提交后记录fence与失败域，而不是承诺虚假的frame rollback。

因此本篇不是要求所有函数套统一`Transaction`类型，而是建立可声明的operation taxonomy与跨owner编排层：

`OperationInventory -> OperationId/Intent -> Preflight -> Staging -> CommitPoint -> Publication -> Compensation/Rollback -> Recovery/Reconciliation -> Idempotency/Dedup -> OperationReceipt -> TransactionQualification`

本篇登记 **0 项 P0、48 项 P1、12 项 P2 和40个验收门**。没有新增P0，因为静态证据能证明部分发布窗口、崩溃后未知状态和重复执行风险，却未独立证明shipping BuildSet已造成不可恢复用户内容丢失。Runtime04/05、Editor02/06、Hub01、Tooling06/09分别继续拥有具体文件、ECS、document、plugin、Hub与Coordinator实现；Tooling23拥有错误传播，Tooling24拥有锁与linearization，Tooling27拥有schema migration，Tooling31拥有物理authority，Tooling35拥有lease/teardown。本篇只拥有跨域operation状态机、commit point、部分发布分类、补偿、幂等、重启仲裁与统一receipt。

## 2. 审查边界、口径与限制

### 2.1 词法账本只用于发现候选

| Candidate signal | 命中 | production-like保守文件 | 解释 |
|---|---:|---:|---|
| 扫描输入 | 13,438文件 | Runtime/Editor/Plugins/Hub/App/Runtime Interface/Tools/Examples | 排除常见tests、fixtures、benches、target、node_modules与dist；不是Cargo-resolved BuildSet |
| `transaction` | 900行 | 172文件 | 同时包含DB transaction、undo transaction、render命名和注释，不能按行计缺陷 |
| `rollback` | 163行 | 40文件 | 既有durable rollback正例，也有忽略返回值的手工cleanup |
| `commit` | 1,737行 | 437文件 | 大量Git commit、render command commit及测试命名，不等价于事务提交点 |
| `publish` | 730行 | 240文件 | registry、snapshot、event、UI和文件publication语义各不相同 |
| `preflight` | 239行 | 61文件 | 动态场景/bundle已有强基线，其他domain仍可能只做输入格式校验 |
| `compensat*` | 2行 | 2文件 | 术语极少不代表没有手工补偿，许多代码使用restore/remove/cleanup |
| `idempoten*` | 55行 | 15文件 | 主要集中在Coordinator与少量生命周期，RPC/HTTP公开合同仍缺失 |
| atomic write/replace/rename | 222行 | 65文件 | 单文件rename不自动提供multi-file、directory、cross-device或crash recovery |

### 2.2 必须先声明事务类别

| 类别 | 可以承诺 | 不得隐含承诺 |
|---|---|---|
| In-memory atomic publication | preflight后单锁/单swap可见，失败前live state不变 | 进程崩溃后持久恢复、外部设备撤销 |
| Durable file transaction | journal、fsync、commit point与restart recovery后的明确终态 | 跨文件系统rename原子、远端catalog或GPU副作用 |
| Undo/redo transaction | 当前进程内反向/正向命令与history语义 | crash recovery、任意外部I/O可逆、永久审计日志 |
| Best-effort command batch | 稳定顺序、逐项错误、继续/停止策略与已应用计数 | all-or-nothing或panic前自动回滚 |
| Compensating workflow | 对不可回滚步骤执行显式反向动作并保留失败证据 | 完全恢复原状态、外部服务恰好一次 |
| Commit-only submission | 所有可失败验证前移，提交后以fence/result/next-frame recovery处理 | GPU、audio device、process launch或network send rollback |

### 2.3 Evidence限制

1. 本轮逐项读取resource transaction、asset migration、scene/world publication、Editor history/scene/plugin、module/plugin reload、Hub、RPC、render submit和Coordinator代表实现及测试；没有运行crash harness、断电测试、真实GPU/device/network故障或跨进程重复请求。
2. 当前工作树有其他会话正在修改大量Editor与测试源码，报告记录审查时物理内容与HEAD；所有finding实施前必须source recheck，不能把dirty内容误称为稳定提交基线。
3. 已知Editor、Hub、WOC和plugin metadata动态验证阻断未变化，本篇不重复触发；这些阻断既不证明也不否定事务finding。
4. `rename`只在满足同filesystem、平台替换规则和durability同步时提供有限原子性；本篇不会从API名称推断power-loss safety。
5. 补偿不是rollback的同义词。远端请求、GPU submit、process launch、通知和用户可见publication通常只能对账或补偿，receipt必须保留`PartiallyApplied/Unknown`终态。

## 3. 必须保留的工程基础

### 3.1 Durable resource transaction已形成内部黄金基线

`JOURNAL_VERSION = 4`的immutable intent与append-only transition记录`Intent/Prepared/Committing/Committed/RollingBack`及`Active/AllCommitted/Cleanup`等phase；输入要求绝对路径、规范化identity、无别名target，owner lock拒绝并发owner并在新事务前拒绝pending recovery。staging与backup计算digest并同步，live replace前记录`Committing`，commit marker写入与同步形成明确提交点；失败路径区分rollback、`CommitRecoveryDeferred`和`CleanupDeferred`，恢复器根据journal/evidence仲裁，而不是猜测最后一个rename是否成功。

### 3.2 故障注入覆盖了真实破坏窗口

engine/recovery测试覆盖stage write、staging directory sync、backup copy、target replace后失败/崩溃、retired delete后崩溃、commit point write/sync、rollback transition/restore、rollback-completed、cleanup transition和journal删除等窗口，并分别统计rollback attempt/success与deferred recovery。后续所有durable workflow应复用或达到这一证据等级，不能再自造只测正常rename的“atomic save”。

### 3.3 Asset migration与project generation已经复用通用内核

asset migration把core disposition映射为migration phase，commit marker未同步时明确拒绝宣称durable并要求重跑recovery；project generation在open/restart执行policy-bound recovery并记录rollback、cleanup和deferred metrics。此处的抽象方向正确：domain提供path/recovery policy，通用内核拥有journal和commit algorithm。

### 3.4 动态场景发布把失败前移到preflight

dynamic scene publication先验证entity ownership、descriptor collision、reflection registration、component row storage、allocator exhaustion和resource transfer，随后publication段只导入preflighted descriptor、安装rows/resources、更新generation并派发生命周期。staged World commit通过整体swap保留runtime-only callback/queue容器。这是内存事务应采用的“解析/验证/分配在前，短提交段在后”模式。

### 3.5 Bundle transaction为结构变更建立final-row artifact

bundle insertion持有target、preflighted component、unregistered type、hierarchy/mobility effect和deferred removals；deferred artifact脱离World borrow后，可在一个barrier先验证全部target，再进入publication pass。应保留固定容量与typed component staging，不要退化成逐component插入后手工删除。

### 3.6 Editor transaction engine已处理apply/revert失败

Editor history记录selection before/after、participants、merge mode与command effect；apply部分成功时尝试revert，undo/redo某命令失败时会重新执行已反向的命令恢复原状态，rollback失败会fault engine而不是继续伪装成功。该语义比多数参考编辑器command stack更严格，应作为in-memory authoring command基线。

### 3.7 Core module batch activation已有预检与逆序cleanup

batch activation先取得transition ownership、验证module state与reactivation services，再build所有module、解析startup service、等待ready、finish并publication；失败时逆序cleanup built modules、reset services并聚合rollback errors。缺口位于最终`Running`与observer publication窗口，不应因此推翻已有batch preflight/cleanup结构。

### 3.8 Coordinator数据库与request journal已有幂等基础

SQLite使用WAL、foreign keys、busy timeout和`BEGIN IMMEDIATE`；`CommandRequestJournal`以32位hex request ID绑定canonical payload fingerprint，重复ID不同payload fail closed，terminal response/error可回放，transactional admission使用savepoint，restart会处理悬空accepted request。Offline spool又使用repository-bound envelope、临时文件+`os.replace`和互斥锁。这些是OperationId/DedupStore的本地正例。

## 4. 已确认的结构断点

### 4.1 同名Transaction API没有共同状态机与receipt

resource、World、Editor、plugin、Hub、Coordinator各自定义prepare/commit/rollback，但没有统一taxonomy声明durability、commit point、retry policy、owner、partial outcome和recovery入口。调用方看到`Result<(), Error>`时无法判断失败前是否已改变live state、是否可重试、是否需要restart recovery。

### 4.2 Editor scene route在document activation前已发布三套authority

create路径先hard-link发布scene source、删除staging、import catalog、install authoring world，最后才`activate_scene_while_routed`。若activation失败，函数直接返回Lifecycle error；source、catalog和authoring world都没有恢复。open路径也先install world再activate，失败时没有恢复旧world。现有rollback只覆盖catalog/import/install之前的部分窗口。

### 4.3 PreparedSceneCreation的文件原子性低于resource基线

scene create用hard link发布staging并删除staging；rollback和Drop删除live/staging时忽略Drop错误，也没有journal、file/parent sync或restart recovery。crash可能留下published source但没有catalog/document事实，或者留下staging artifact；这不是durable transaction内核的同等级保证。

### 4.4 Runtime Session archive的“atomic”只在当前进程近似成立

archive写入用`BufWriter::flush`，没有`File::sync_all`或parent directory sync；并发revision由process-local `OnceLock<Mutex<HashMap>>`保存，restart后commit/lineage evidence归零。existing target先rename到backup再rename temp到target，删除backup与restore失败都被忽略，没有journal扫描；进程在两次rename之间退出会让canonical target消失。

### 4.5 Hub create project缺directory transaction恢复协议

Hub会完整render到staging，空target先rename为backup，commit失败时尝试恢复并保留唯一backup，这是正确局部补偿；但transaction ID仅为PID+process-local counter，template entries与目录没有durability sync，commit前后没有journal/owner lock/restart scan，backup cleanup失败被静默忽略。崩溃后用户可能同时看到missing target、staging和backup且Hub无法自动仲裁。

### 4.6 UI asset hot reload在可失败surface步骤前已修改多套cache

executor先evict compile cache，再invalidate resource resolver，再apply theme document，最后`mark_target_surfaces_dirty`才返回`Result`。最后一步失败时前三项已生效但报告整体返回Err，没有partial report、generation rollback或强制full rebuild marker；调用方无法知道下一帧应继续、重建还是恢复旧主题。

### 4.7 Editor plugin enablement跨manifest、capability与runtime publication不原子

project enablement先更新Editor capability/runtime state，之后才写`manifest.plugins`；native-aware路径还基于两套completed manifest投影并在末尾发布status。内部capability apply失败能尝试restore，但外层manifest/status/native completion没有统一reservation、commit point和structured rollback errors，字符串错误也无法表达`ManifestOld/RuntimeNew/StatusOld`等终态。

### 4.8 Runtime module activation先标Running再调用可panic observer

single与batch路径先`finish_module_activation`把entry改为Running，再调用`runtime_module_activated` observer。observer没有Result且panic被外层catch成activation failure；失败清理会cleanup built module和reset services，但`reset_initializing_module`只恢复Initializing entry，已经Running的module可能保留与error矛盾的lifecycle。batch还可能已通知前几个module后在后续observer panic，形成部分外部publication。

### 4.9 Native/VM hot reload有强内存rollback但没有统一跨进程终态

Native loader保留old plugin、runtime snapshot、schema check和rollback diagnostic，但旧library一旦标记Unloaded就明确“rollback unavailable”；VM coordinator能保存state、prepare reflection、按generation恢复interface并把失败slot标Failed。两者都是正向基础，但receipt没有共同operation ID、阶段/副作用清单或process crash恢复；native DLL卸载、host registration、reflection publication与state restore间的未知窗口无法由restart仲裁。

### 4.10 Ragdoll spawn仍使用忽略结果的手工删除

Ragdoll预先验证profile、bone target与transform，随后逐body`spawn_node`并配置transform/body/collider/joint；失败时`rollback_spawn`和当前body删除都忽略`remove_entity_recursive`结果。World已经具备bundle/dynamic publication机制，此处却没有staged entity batch或rollback receipt，可能留下部分physics bodies、joints或generation/event副作用。

### 4.11 Sound mixer补偿只保留第一个rollback error

mixer先apply backend graph update、停止旧source、安装新source并sync；失败时停止新/旧source、sync旧graph、恢复source并重sync，补偿设计方向正确。但`record_first_error`丢弃后续rollback故障，失败后直接deactivate backend并用字符串拼接原因，没有逐step outcome、detached voice清单或可自动恢复的terminal state。

### 4.12 Deferred command queue不是事务但接口附近缺显式误用防线

queue预留spawn tokens、结构命令按batch finish并收集typed errors，panic时丢弃未执行命令并保持queue内存安全；普通命令在panic前已执行的mutation不会回滚。这个行为与Bevy Commands类似，是有序deferred batch而非transaction。Zircon需在API/report中声明partial apply和panic boundary，要求需要all-or-nothing的调用方使用bundle/scene transaction。

### 4.13 RPC/HTTP重试没有业务幂等合同

RPC invocation的optional `NetRequestId`主要用于pending response correlation；state没有bounded dedup result store、payload fingerprint、effect class或duplicate terminal replay。handler是同步调用，timeout只能在返回后判断，已超时handler的副作用仍可能发生。HTTP request允许配置retry次数，却没有method safety、body replayability、idempotency key或retryable status/error policy，未来transport实现极易重复非幂等动作。

### 4.14 Coordinator数据库事务不能证明外部动作终态

request journal把DB mutation与terminal response原子化是正确基础，但callback/after_commit可执行Git、filesystem、process和notification副作用。进程在外部动作后、terminal DB commit前退出，startup只把accepted标为`command_execution_interrupted`；它不会读取Git SHA、filesystem identity、child PID或provider receipt来判定真实结果。相同request被永久failed也避免了盲目重复，却留下需要人工对账的Unknown而非已恢复终态。

### 4.15 Render/GPU publication必须明确为commit-only

render framework已有camera-loop preflight、compiled pipeline和submit后history/stat更新，但一旦command buffer送入GPU便不能通过CPU transaction撤销。多camera/overlay/probe序列若中途backend失败，只能保留已提交fence和partial product事实、隔离本帧并在后续frame重建。报告与API需要禁止使用“rollback frame”措辞，所有可能失败的graph/resource/schema validation必须在首个queue submit前完成。

## 5. 目标架构

### 5.1 OperationDescriptor与OperationId

每个跨owner mutation声明stable operation kind、caller-provided或owner-minted ID、canonical input fingerprint、scope、participants、durability class、effect class、retry policy、timeout semantics和sensitive-data policy。ID既不能只用进程counter，也不能在每次重试重新生成。

### 5.2 Prepare与Preflight Artifact

解析、权限、identity、schema、capacity、dependency、target collision、backend availability和所有可预分配资源在live mutation前完成，输出immutable `PreparedOperation`。artifact绑定source/owner generation并有expiry；commit只消费一次，stale artifact返回typed rejection。

### 5.3 CommitPoint与PublicationPlan

每个operation明确唯一commit point：内存swap、durable journal marker、DB commit、plugin generation publish或GPU queue submit。commit point前失败应保持live不变或完整rollback；commit point后只能finish publication、cleanup、compensate或recover，不能继续用通用Err掩盖已提交事实。

### 5.4 Rollback与Compensation分离

rollback恢复同一authority内尚未提交的状态；compensation对已提交/外部副作用执行新动作。每步声明`Reversible/Compensatable/CommitOnly/ObserveOnly`，补偿按逆依赖执行并收集全部错误，不以first-error丢失其余结果。

### 5.5 Recovery与Reconciliation

durable owner在启动/打开project/加载plugin前扫描非终态intent，按evidence与commit point恢复；跨系统workflow通过participant receipts对账，产生`NotApplied/Applied/PartiallyApplied/Compensated/RecoveryPending/Unknown`。Unknown必须阻止自动重试，直到owner-specific probe给出结论。

### 5.6 Idempotency与DedupStore

operation ID绑定input fingerprint、result/terminal outcome、retention和owner scope。重复相同payload回放结果，不同payload冲突；at-least-once transport的effect handler先查dedup admission。read/query、idempotent set、create-once、append、transfer等effect class采用不同retry规则。

### 5.7 OperationReceipt

receipt至少包含operation/attempt ID、input fingerprint、participants、prepared generation、commit point、每步started/completed/effect、rollback/compensation outcome、recovery owner、terminal disposition、artifact/fence/commit SHA和bounded diagnostics。普通`Result<()>`只适用于已证明失败前无可见mutation的leaf operation。

### 5.8 TransactionQualification

每类operation由fault matrix证明pre-commit atomicity、post-commit completion、crash recovery、duplicate replay、compensation failure、orphan cleanup与performance budget。qualification绑定BuildSet、平台/filesystem/backend和source fingerprint，不能从Windows NTFS测试外推到全部平台。

## 6. P1 重构项

### TX-P1-001 · 建立OperationInventory单一真源

列出所有命名为transaction/commit/publish/atomic/retry的生产operation，并记录真实类别、owner、participants、commit point、durability和当前receipt；词法扫描只作入口。

### TX-P1-002 · 冻结Operation taxonomy

统一`InMemoryAtomic/DurableFile/UndoRedo/BestEffortBatch/CompensatingWorkflow/CommitOnly`语义，禁止同名API隐含不同保证。

### TX-P1-003 · 定义stable OperationId与AttemptId

跨进程或可重试操作使用稳定ID；attempt单独计数，PID/counter只能作临时文件suffix，不能作唯一业务identity。

### TX-P1-004 · 定义canonical Intent与input fingerprint

operation kind、normalized target、payload/schema、caller/owner generation进入canonical hash；same ID/different intent fail closed。

### TX-P1-005 · 统一PreparedOperation envelope

artifact携preflight evidence、participant reservation、source generation、expiry与single-consume token；stale/duplicate消费返回typed outcome。

### TX-P1-006 · 强制每类operation声明commit point

代码、文档和receipt标明首个不可无损撤销步骤；commit point前后使用不同error/disposition，评审禁止模糊“commit附近”。

### TX-P1-007 · 建立PartialPublication taxonomy

统一`NotApplied/Applied/PartiallyApplied/RolledBack/Compensated/RecoveryPending/CleanupPending/Unknown`，禁止把全部终态折叠为Err。

### TX-P1-008 · 定义OperationReceipt schema

receipt记录step、participant、generation、commit证据、rollback/compensation、recovery owner和artifact；敏感payload只留hash或脱敏摘要。

### TX-P1-009 · 抽取durable resource transaction conformance

把现有journal/version/owner-lock/fsync/recovery/fault-injection基线变成可复用测试套件，domain wrapper必须证明等价而非复制rename代码。

### TX-P1-010 · 规范化deferred durable disposition消费

所有调用者显式处理`CommitRecoveryDeferred/CleanupDeferred`，不得把cleanup pending当失败回滚，也不得把commit-marker unresolved当成功。

### TX-P1-011 · 统一journal owner与启动恢复顺序

project open、asset migration、Hub和session archive在新mutation前先取得owner、扫描pending、完成recovery；未知journal版本fail closed。

### TX-P1-012 · 建立atomic file/directory capability matrix

按Windows/Linux/macOS、filesystem、replace-existing、cross-volume、file/directory、sync semantics记录能力；不支持场景走copy+verified publish或显式拒绝。

### TX-P1-013 · 将PreparedSceneCreation接入durable内核

scene source publication使用通用prepared writes或等价journal，包含source/staging/catalog reconciliation evidence与parent sync。

### TX-P1-014 · 事务化SceneDocumentRoute

把source、catalog、authoring world和document lifecycle建成participant plan；activation preflight前不替换world，失败后有完整rollback/compensation receipt。

### TX-P1-015 · 给scene open提供retired world恢复句柄

installer返回可commit/rollback的installation artifact，而不是`Result<()>`；document activation成功后才finalize retired world。

### TX-P1-016 · 收敛document dirty/save/activation终态

由Editor02绑定history save token、source commit、catalog generation和document activation receipt，失败不能出现Clean UI指向未激活或未持久内容。

### TX-P1-017 · 重构Runtime Session archive保存

复用durable file transaction，执行file/parent sync、owner lock、persistent lineage/revision和restart recovery；移除process-local map作为唯一仲裁真源。

### TX-P1-018 · 为archive cleanup/restore保留typed结果

所有temp/backup删除和restore失败进入receipt；恢复前禁止覆盖唯一backup，启动时清点orphan artifact。

### TX-P1-019 · 为Hub create project建立durable directory journal

intent记录target/staging/backup/template hash与commit phase，启动扫描PID-independent artifact并恢复或完成publication。

### TX-P1-020 · 让Hub backup cleanup可观察

成功但cleanup pending返回typed disposition并进入维护队列；不得静默吞掉backup remove失败。

### TX-P1-021 · 保留并标准化World staged publication

把dynamic scene和bundle正例抽成World transaction API，明确preflight artifact、single commit与retired state；避免domain手工spawn/remove。

### TX-P1-022 · 将Ragdoll spawn迁移到staged entity batch

一次preflight所有body/component/joint，单次publish；失败不派发部分lifecycle，rollback error不再被忽略。

### TX-P1-023 · 给deferred command queue声明best-effort语义

API/report暴露applied/failed/discarded范围与panic boundary；文档禁止把`apply_deferred`称为atomic transaction。

### TX-P1-024 · 为需要原子性的deferred mutation提供transaction command

结构性多步命令先构建bundle/final-row artifact再入队；任意closure command只能用于明确best-effort或不可失败操作。

### TX-P1-025 · 事务化UI asset hot reload

先编译theme/resource/surface mutation plan，验证全部target，再按generation swap；失败时旧cache/theme/surface保持一致或强制进入FullRebuildPending。

### TX-P1-026 · 给UI reload返回partial execution receipt

cache eviction、resolver invalidation、theme publish、surface dirty与rebuild逐项记录；调用方能根据terminal disposition恢复。

### TX-P1-027 · 事务化Editor plugin enablement

manifest selection、capability configuration、runtime plugin state、native projection与status publication使用同一operation ID和participant plan。

### TX-P1-028 · 结构化plugin enablement rollback

替换拼接字符串，记录每个capability/runtime/manifest/status的previous/current generation及rollback outcome。

### TX-P1-029 · 修复module activation最终publication窗口

observer notification必须在Running publication前完成且可返回Result，或Running视为commit point并把observer failure降为post-commit delivery failure；不得两种语义混用。

### TX-P1-030 · 给batch activation生成participant receipt

记录每个module build/ready/finish/running/observer、service reset和cleanup结果，部分notification可被reconcile。

### TX-P1-031 · 统一native/VM hot reload operation contract

load、state save/migrate/restore、registration/reflection publish、generation swap、old unload与cleanup共享阶段和receipt，保留各backend专用实现。

### TX-P1-032 · 明确DLL unload后的不可回滚点

在old library真正卸载前完成所有可失败验证；卸载后失败只能进入Failed/RecoveryRequired，禁止诊断仍称rolled back。

### TX-P1-033 · 给hot reload加入crash/restart reconciliation

持久化package/build/schema/generation与state artifact identity；restart清点staging package、old/new generation和registration snapshot。

### TX-P1-034 · 重构Sound mixer compensation receipt

记录graph apply、old/new source stop/start、restore与backend deactivate所有结果，不只保留first error；输出detached playback census。

### TX-P1-035 · 声明external backend effect class

audio/physics/window/device/process操作标为Compensatable或CommitOnly，compensation失败必须进入degraded/fused state并由owner恢复。

### TX-P1-036 · 给RPC descriptor增加IdempotencyPolicy

区分ReadOnly、IdempotentSet、CreateOnce、Append、NonRetryable和CustomDedup；server拒绝与transport retry不兼容的调用。

### TX-P1-037 · 建立RPC DedupStore与result replay

以caller/session/operation ID和payload fingerprint准入，bounded retention内回放terminal response；冲突ID fail closed并有quota。

### TX-P1-038 · 修正RPC timeout语义

同步handler返回后超时只能报告`CompletedAfterDeadline`而非假装未执行；可取消handler必须使用cooperative cancellation并声明effect boundary。

### TX-P1-039 · 限制HTTP自动重试

request声明method safety、body replayability、idempotency key、retryable errors/status和backoff；非幂等请求默认0次自动重试。

### TX-P1-040 · 把Coordinator request journal推广到外部effect receipt

callback在DB内只提交intent/admission；外部worker执行后写participant receipt，再由reconciler提交terminal result，不能只靠try/except包裹。

### TX-P1-041 · 为Git finalize建立commit reconciliation

operation intent绑定base HEAD、index tree、path blob OID和message；restart通过commit SHA/tree/parent验证是否已提交，same operation不重复commit。

### TX-P1-042 · 为workspace copy/process launch建立durable phase

materialization、child start、run、terminal capture、cleanup各有persistent state与filesystem/process identity；unknown PID或copy root先reconcile再重试。

### TX-P1-043 · 统一Offline Queue与command request identity

spool envelope保存caller提供的operation ID与payload fingerprint，replay不得为同一业务操作生成新ID；move-to-terminal也要sync目录并可恢复。

### TX-P1-044 · 为通知/remote provider定义outbox

DB内提交outbox intent，独立sender按operation ID交付并记录provider receipt；通知失败不回滚已提交产品mutation。

### TX-P1-045 · 定义Render/GPU commit-only receipt

首个queue submit为commit point，receipt包含compiled graph hash、resource generation、queue/fence、已提交camera/pass范围和后续frame recovery决定。

### TX-P1-046 · 将所有render可失败验证前移

pipeline executor、resource/imported handle、surface、pass schema与output target在首个submit前验证；提交中失败不再尝试CPU state假rollback。

### TX-P1-047 · 建立transaction cost与batching基线

测量journal/fsync、backup bytes、preflight clone、lock hold、dedup lookup、receipt write和recovery scan；按durability class分预算，不以关闭fsync换取虚假性能。

### TX-P1-048 · 以TransactionQualificationReceipt作为产品准入

shipping mutation必须证明commit point、duplicate replay、crash recovery、compensation failure、orphan cleanup和平台矩阵；缺receipt只能标Prototype/BestEffort。

## 7. P2 完善项

### TX-P2-001 · 建立operation timeline可视化

展示prepare、participant、commit point、publication、compensation与recovery，不显示敏感payload。

### TX-P2-002 · 增加orphan artifact scanner

扫描staging/backup/journal/outbox/temporary project与plugin package，按owner和operation ID分类，不自动删除Unknown证据。

### TX-P2-003 · 增加multi-mutation lint

标记单函数跨多个authority调用且后段可失败的路径，允许以PreparedOperation或结构化waiver消除。

### TX-P2-004 · 增加state-machine property test

随机phase、retry、duplicate、rollback和recovery顺序，验证非法transition fail closed且终态幂等。

### TX-P2-005 · Fuzz durable journal与torn tail

覆盖截断、重复frame、未知version、路径alias、digest mismatch和损坏backup，恢复器不得猜测成功。

### TX-P2-006 · 发布compensating action catalog

记录每个external provider的effect/compensation/reconciliation能力、期限和不可逆点。

### TX-P2-007 · 建立deterministic chaos seed

故障注入记录seed、platform/filesystem/backend与准确phase，可重放而非随机睡眠。

### TX-P2-008 · 细分transaction metrics并限制基数

聚合phase latency、rollback、recovery、duplicate、unknown与orphan；operation ID不直接进入常规metric label。

### TX-P2-009 · 增加recovery operator runbook

说明如何冻结owner、导出receipt、验证backup、完成/回滚/补偿和解除Unknown，禁止手工猜路径删除。

### TX-P2-010 · 建立reference currentness复核

通过Tooling33跟踪Unreal transaction/save、Bevy Commands、Fyrox CommandStack、Godot UndoRedo和Unity RenderGraph漂移。

### TX-P2-011 · 建立transaction debt趋势

统计string-only result、ignored cleanup、process-local operation ID、unjournaled rename、external effect without receipt与unknown recovery。

### TX-P2-012 · 提供operation schema文档生成

从descriptor生成owner、state diagram、retry/timeout、receipt和recovery文档，绑定Tooling28 publication gate。

## 8. 参考引擎差异与适用性

### 8.1 Unreal

Unreal `UTransBuffer`维护transaction context、nested active count、redo removal cache、memory budget、undo barrier及before/after delegate；`FScopedTransaction`以RAII结束并可显式Cancel。SavePackage又是独立的内容publication管线，不能把Editor undo等同durable save。Zircon应吸收context/primary owner、nested scope、barrier、memory budget与publication分域；不应复制全局`GUndo`、UObject snapshot成本或把Unreal存在transaction当性能证明。

### 8.2 Bevy

Bevy `CommandQueue`保证deferred command存储、panic/error handler和remaining queue安全，但Commands并非all-or-nothing事务；panic前的World mutation不会自动恢复。它直接支持本篇对Zircon deferred queue的定性：保留高吞吐batch，原子结构变更另用preflighted bundle artifact，不能给Commands附加虚假rollback承诺。

### 8.3 Fyrox

Fyrox Editor `CommandTrait`提供execute/revert/finalize，`CommandGroup`逆序revert，`CommandStack`维护undo/redo与容量。它适合作为Editor command ergonomics参考，却不提供跨文件、进程或崩溃原子性。Zircon现有command effect与failed-undo recovery更严格，应补持久journal/replay codec和跨document participant，而不是退回无错误返回的简单command stack。

### 8.4 Godot

Godot `UndoRedo`有merge mode、backward undo、history/version和commit_action；`EditorUndoRedoManager`按object/history路由并跟踪saved version。适用结论是事务必须绑定正确history owner、保存版本和merge policy；但Godot undo同样不是durable multi-authority commit，Zircon不能用Editor history替代asset/catalog/world publication coordinator。

### 8.5 Unity Graphics

Unity RenderGraph先record，再由NativePassCompiler执行`ValidatePasses -> Cull -> Merge -> Resource lifetime`，随后BeginExecute和ExecuteNativeRenderGraph；异常路径cleanup资源并清空尚未提交的command buffer。关键不是“GPU可回滚”，而是尽量在execute/submit前完成验证、提交后清理CPU graph并隔离失败。Zircon应吸收compile-before-submit、side-effect pass标记和明确cleanup，仍需自己的fence/partial submit receipt。

## 9. 实施顺序

### M0 · Inventory与taxonomy冻结

- 用Cargo-resolved shipping/editor/tool BuildSet重取operation inventory；
- 为每个operation标注真实类别、owner、commit point与retry policy；
- 冻结新增un-journaled multi-file rename、external effect without ID及Result-only partial publication。

### M1 · Schema、receipt与durable conformance

- 实现OperationId/Intent/PreparedOperation/Disposition/Receipt基础schema；
- 抽取resource transaction conformance和fault matrix；
- 建立platform/filesystem atomic capability与owner recovery入口。

### M2 · 文件与Editor authority收敛

- Scene creation/route、Session archive和Hub create接durable/coordinated transaction；
- document installer返回retired-state rollback handle；
- UI hot reload与plugin enablement改为preflighted generation publication。

### M3 · Runtime lifecycle与domain补偿

- 修复module activation Running/observer commit point；
- native/VM reload共享operation phase与receipt；
- Ragdoll走World staged batch，Sound返回完整compensation census。

### M4 · Network与Coordinator外部副作用

- RPC/HTTP接idempotency policy、dedup和deadline semantics；
- Coordinator采用intent/outbox/participant receipt/reconciler；
- Git、copy、process、offline replay和notification按owner实现对账。

### M5 · Render commit-only与动态资格

- 将render validation前移到首个submit之前；
- 记录queue/fence/partial camera/pass receipt；
- 运行crash、duplicate、rollback-failure、device-loss和restart矩阵。

### M6 · Required gate与文档

- transaction qualification、orphan scan和source fingerprint进入required CI；
- waiver含owner、类别、commit point、reason、expiry和替代证据；
- G01-G40完成前保持implementation pending。

## 10. 验收门

| Gate | 验收内容 |
|---|---|
| G01 | Cargo-resolved BuildSet中的transaction/commit/publish/atomic/retry operation进入inventory |
| G02 | 每个operation声明类别、owner、participants、durability、effect、retry与timeout policy |
| G03 | 跨进程/可重试operation使用stable OperationId，PID/counter不作业务identity |
| G04 | OperationId绑定canonical input fingerprint，same ID/different intent fail closed |
| G05 | PreparedOperation绑定source/owner generation、expiry且只能消费一次 |
| G06 | 每类operation有唯一明确commit point，代码与receipt一致 |
| G07 | pre-commit failure保持live不变或完整rollback，post-commit failure不伪装未应用 |
| G08 | terminal disposition区分NotApplied/Applied/Partial/Compensated/RecoveryPending/Unknown |
| G09 | rollback与compensation分离，commit-only participant不得声明可回滚 |
| G10 | OperationReceipt记录所有participant step与全部rollback/compensation error |
| G11 | durable resource transaction conformance覆盖journal、lock、fsync、commit point与restart recovery |
| G12 | 未知journal version、torn tail、digest mismatch和missing backup均fail closed |
| G13 | `CommitRecoveryDeferred/CleanupDeferred`被调用方正确消费且可在restart收敛 |
| G14 | atomic file/directory语义按平台/filesystem验证，不从rename名称推断durability |
| G15 | Scene create/source publication使用durable journal并清点staging/orphan |
| G16 | SceneDocumentRoute失败不会留下source/catalog/world/document相互矛盾 |
| G17 | scene open activation失败能恢复retired authoring world |
| G18 | Session archive执行file/parent sync，revision evidence跨restart且backup可恢复 |
| G19 | Hub project create在任意rename crash window后自动完成或恢复，backup cleanup可观察 |
| G20 | dynamic scene与bundle publication保持all-failable-work-before-publish基线 |
| G21 | Ragdoll等多entity mutation不再用ignored manual delete模拟rollback |
| G22 | deferred command queue公开best-effort/partial semantics，原子调用方使用typed transaction command |
| G23 | Editor command apply/undo/redo失败恢复原状态；rollback失败fuse且有receipt |
| G24 | transaction journal可重放的command有stable schema/decoder，不可重放命令明确分类 |
| G25 | UI hot reload失败时cache/theme/surface generation一致或进入可恢复FullRebuildPending |
| G26 | plugin enablement的manifest/capability/runtime/status同operation提交或完整补偿 |
| G27 | module activation Running与observer notification只有一个一致commit point |
| G28 | batch module activation能报告每个module/service/observer/cleanup participant终态 |
| G29 | native/VM reload在old unload前完成所有可失败验证，卸载后错误不称rolled back |
| G30 | plugin reload crash/restart能按package/schema/generation/state artifact对账 |
| G31 | Sound/physics/external backend补偿收集全部失败并进入明确degraded/fused终态 |
| G32 | RPC调用声明idempotency class，duplicate ID相同payload回放、不同payload冲突 |
| G33 | RPC timeout报告区分未开始、取消、已完成超期和效果未知 |
| G34 | HTTP非幂等请求默认不自动retry，body replay和idempotency key经过验证 |
| G35 | Coordinator DB request journal为Git/file/process/notification保存participant receipt |
| G36 | restart能通过Git SHA/tree、filesystem identity、PID creation和provider receipt仲裁Unknown |
| G37 | offline spool与online request共享业务operation ID，replay不重复副作用 |
| G38 | Render首个GPU submit前完成graph/resource/surface validation，提交后记录fence与partial scope |
| G39 | crash/duplicate/rollback-failure/device-loss测试和transaction performance预算绑定平台BuildSet |
| G40 | `git diff --check`、frontmatter路径、finding ID/severity、source fingerprint、索引/coverage/总账计数通过 |

## 11. 状态与产出记录

| 里程碑 | 状态 | 日期 | 证据 |
|---|---|---|---|
| transaction/rollback/commit/publish/preflight/idempotency lexical inventory | review_complete | 2026-08-16 | 13,438个production-like候选文件；900/163/1,737/730/239行及55行idempotency，只作candidate |
| representative operation review | review_complete | 2026-08-16 | resource/asset/scene/ECS/Editor/plugin/module/Hub/RPC/render/Coordinator逐commit point核对 |
| source/reference evidence fingerprint | review_complete | 2026-08-16 | HEAD `25e09a23...d404`；85个source/test/reference输入、1,586,278 bytes；SHA-256 `dd44319bedfd6f1f99d36498d8f55c6dbc212e2e9af19e100a3d3e41612539e4` |
| five-engine comparison | review_complete | 2026-08-16 | Unreal transaction/save、Bevy Commands、Fyrox CommandStack、Godot UndoRedo、Unity RenderGraph |
| Operation/CommitPoint/Recovery/Idempotency/Receipt architecture | design_complete | 2026-08-16 | 本篇第5节；未实现schema、coordinator、dedup、reconciler或qualification |
| production refactor与crash/device/network tests | pending | - | 本篇只review，不修改production/tests/Cargo/workflow |

当前结论仍是`review_complete / implementation_pending`。在M0-M6和G01-G40完成前，Zircon不能把“用了transaction命名”“写了temp再rename”“有rollback函数”“返回Err”“有request ID”“SQLite已commit”或“GPU command buffer已清空”当成跨owner原子性、exactly-once、崩溃可恢复或用户内容安全的工程证明。
