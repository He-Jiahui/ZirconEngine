---
related_code:
  - zircon_hub/src/projects
  - zircon_hub/src/engines
  - zircon_hub/src/build
  - zircon_hub/src/process
  - zircon_hub/src/settings
  - zircon_hub/src/tauri_app/mod.rs
  - zircon_hub/src/tauri_app/commands.rs
  - zircon_hub/src/tauri_app/runtime_state.rs
  - zircon_hub/src/tauri_app/runtime_state/action_tasks.rs
  - zircon_hub/src/tauri_app/runtime_state/build_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/editor_launch_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/new_project_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/project_actions.rs
  - zircon_hub/src/tauri_app/runtime_state/project_delivery_actions.rs
  - zircon_runtime_interface/src/hub_protocol
  - zircon_editor/src/core/hub_link
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/zircon_hub/01-action-dispatch-and-typed-payload.md
  - docs/plans/zircon_hub/02-background-task-framework-and-persistence.md
  - docs/plans/zircon_hub/03-project-lifecycle-robustness.md
  - docs/plans/zircon_hub/04-settings-draft-and-source-engine.md
  - docs/plans/zircon_hub/05-frontend-componentization-and-type-safety.md
  - docs/plans/zircon_hub/06-layout-and-visual-standard.md
  - docs/plans/zircon_hub/07-localization-schema-and-coming-soon.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/ProjectEditorRecords.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/ProjectEditorRecords.cpp
  - dev/UnrealEngine/Engine/Source/Developer/DesktopPlatform/Public/InstalledPlatformInfo.h
  - dev/UnrealEngine/Engine/Source/Editor/GameProjectGeneration/Private/SProjectBrowser.cpp
  - dev/godot/editor/project_manager/project_list.cpp
  - dev/godot/editor/project_manager/project_manager.cpp
  - dev/godot/editor/project_manager/project_dialog.cpp
  - dev/Fyrox/project-manager/src/manager.rs
  - dev/Fyrox/project-manager/src/project.rs
  - dev/Fyrox/project-manager/src/upgrade.rs
  - dev/Fyrox/fyrox-build-tools/src/export/mod.rs
  - dev/Fyrox/fyrox-build-tools/src/export/pc.rs
  - dev/bevy/crates
  - dev/Graphics/Packages
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 01 · Project、Engine、Build、Editor Launch、Process、Persistence 与 Delivery 工程化差距

## 1. 结论

Zircon Hub并非只有静态页面。当前后端已经实现typed action request、单worker FIFO、panic隔离、配置原子替换、Hub/Editor共享recent-project DTO、Editor完整启动门握手、project session lease只读探测、工程创建staging/rename回滚、回收站删除、source checkout构建入口、package/install receipt与SHA-256。这些原语是真实代码，不应在重构时退回同步UI脚本或散乱字符串命令。

但它还不是工程级引擎的安装器、项目管理器、构建协调器或进程监督器。当前所谓Engine Install只是本机source/output路径记录；Build只是调用一次`zircon_build.py --targets editor,runtime`并把整个输出读入内存；Package只是递归复制工程源码；Device Install只是再复制到本地目录；Editor launch在启动后立即丢弃`Child`。项目manifest已经有`engine_version_req`，Hub却不求解它，还会优先启动Hub旁边的任意同名Editor。与Unreal的project-to-engine记录、installed platform/configuration能力以及Godot的版本/feature/recovery开门检查相比，产品authority仍缺一整个层级。

本轮确认4个P0。第一，clean tracked source当前存在确定的Rust调用参数不匹配，`zircon_hub`无法通过类型检查。第二，Delete Project在没有检查活动Editor session的情况下先把工程送入回收站，可删除正在编辑的工程。第三，Hub重启时用空previous snapshot合并本地旧recent记录，会把Editor在Hub关闭期间删除的项目重新写回共享registry。第四，Editor启动后Hub立即丢弃唯一`Child`，固定10秒握手超时后也无法等待、终止、回收或确认真实进程，控制面可把仍在启动/运行的Editor记为失败并成为无owner进程。

本报告记录4个P0、36个P1、8个P2。Windows管理验证器随后实际执行`cargo build -p zircon_hub --locked`并以exit 101失败，Rust编译器在`project_actions.rs:583`复现`E0061`；测试因编译阻断未运行。没有运行Hub窗口、真实Editor启动、回收站故障、进程树故障、磁盘断电、跨进程配置竞争、远程设备、签名、安装/回滚或大工程benchmark。

## 2. 审查边界与可复验证据

### 2.1 物理范围

| 子域 | 文件/行数 | 本轮状态 |
|---|---:|---|
| Project/config/shared registry clean set | 20 / 5,224 | E3：create/import/remove/delete、metadata、recent reconcile、config load/save与startup；fingerprint `43f58a7ebce0d13819f515ab90e8ae10c671c552c3106c4071d1512eae56a8c4` |
| Engine/build/process clean set | 25 / 3,997 | E3：source registry/validation、build runner、job queue、Editor launch/handshake/focus与Tauri composition；fingerprint `385d3044e2874a8c45ed8056c2fb5d4ffda4ee2f362620c7762472bd60b2e844` |
| Delivery clean set | 5 / 1,855 | E3：package、local install、receipt/download manifest与产品action；fingerprint `6cc3da481390962c6da55d9d3551bd6bc4e13c9d6d2f4830e04a88aa81074836` |
| Cross-host Hub protocol clean set | 26 / 1,593 | E3：mailbox/session token/focus/recent DTO与Editor publish/consume；fingerprint `25438ae49f60d16e560a0b5b27706db70580f4c63c250cbb37dc9b3c16b9e3c6` |
| focused Hub contract tests | 7 / 3,228 / 42 test attributes | E2源码审查：其中多数是source-text contract，不等价于行为、故障或性能验证；fingerprint `822cfb9d76091ce91908cab10abd44a3d0d8afc3fe238d786ce7e9068c1907eb` |

fingerprint按相对路径排序，将`path + NUL + per-file SHA-256 + LF`串联后再计算SHA-256。它只标识本轮clean阅读集合，不是schema/build ID，也不能替代编译和运行验证。整个tracked Hub Rust生产树另有98文件、23,642行、258个inline test attributes；`zircon_hub/tests`有39文件、19,261行、270个test attributes。本报告只完成上述后端纵向切片，web/UI catalog、Learn/Team/Plugin页面和其余settings交互将另立报告，不能据此宣称Hub全量审查完成。

### 2.2 工作树与验证隔离

成文时上述Hub production、Hub focused tests、runtime-interface Hub protocol和Editor Hub link均为clean tracked source。工作树中既有的其他文档修改没有回退；本报告只新增/更新`docs/plans/optimize`记录。实施前必须重算指纹并重读重叠owner。

本轮按Windows协调器路径执行：

```powershell
.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -Package zircon_hub -SkipTest -VerboseOutput
```

首次尝试因共享复用池忙未进入Cargo；重试取得managed target后实际运行`cargo build -p zircon_hub --locked`，以exit 101失败。编译器报告`E0061: this method takes 0 arguments but 1 argument was supplied`：`runtime_state.rs:548`只定义`persist_unchecked(&mut self)`，`project_actions.rs:583`却调用`persist_unchecked(None)`。同轮另有`editor_focus/publish.rs:89` unused variable warning，以及runtime-interface UI clip dead-code warnings；它们不是本次build失败原因。

### 2.3 本轮追踪的产品链

1. Tauri composition在窗口创建前执行`HubCommandState::load()`，再加载`hub.toml`、刷新recent manifest、获取共享registry writer lease、执行三方合并、注册source engine、刷新catalog并再次持久化。
2. Create Project验证名字/目标路径与链接祖先，把template写入同父级staging目录，必要时备份空目标，再用rename提交；随后把项目/engine binding写入Hub与共享recent registry。
3. Open Editor选择project与Hub metadata engine，探测project session lease；已有Editor时写focus文件，没有时选择Editor executable、spawn并轮询session mailbox。
4. Build Project进入全局background FIFO，调用Python build script，等待完整输出，按exit code记录source build history，再把staged output视为可启动engine。
5. Package Project递归复制project tree；Install Device先重新package，再把package目录复制到本地device root，最后遍历文件生成receipt与一个`file://` download manifest。
6. Remove/Delete先修改内存或文件系统，再分别尝试同步recent registry和Hub config；本轮逐个检查每个不可逆步骤后的失败终态与重试语义。

## 3. 已有工程基础，重构时必须保留

### 3.1 Project create提交与路径防护

- 工程创建不是直接往最终目录逐文件写入：它使用同父级staging目录，空目标会先改名为backup，commit失败会尝试恢复。
- target验证会拒绝文件、非空目录、symlink和Windows reparse-point祖先；unsafe project name也由共享project contract拒绝。
- “目录已经提交但Hub记录失败”已经有明确的kept-folder recovery文案。需要修复其编译与持久化终态，不能把这项部分成功语义删除。

### 3.2 Cross-host DTO、session lease与完整启动门

- Hub/Editor共享recent registry有严格versioned DTO、unknown-field拒绝、去重/排序/8项上限和跨进程writer lease。
- Hub探测活动Editor时只打开platform lease并读取record，不抢占Editor的`SessionGuard`。残留record不会被误判为active owner。
- Hub launch mailbox按project和随机session token隔离；Editor只在完整startup gate成功后原子发布Ready，失败也best-effort发布Failed。Ready会校验project路径，不是单纯把`spawn()`当成功。
- focus signal按Editor instance ID定向并原子发布；Editor端rename-claim后消费，避免两个consumer同时处理。

### 3.3 Typed action、panic containment与原子文件替换

- action ID/payload、task operation与structured message已经类型化；旧计划中的字符串command分发问题大部分已经修复。
- background worker对每个request做`catch_unwind`，poisoned session mutex也有显式恢复路径；队列按FIFO推进。
- Hub config和共享registry都先写临时文件再替换，Windows使用`ReplaceFileW(..., WRITE_THROUGH)`；原文件在替换失败时通常可保留。

这些原语只解决局部正确性。它们没有提供事务journal、进程owner、job cancellation、build-set identity、package recipe或startup recovery，不能用“已有原子写/已有后台线程”替代完整工程化。

## 4. P0：当前阻断与数据/进程authority破坏

### ZHUB-P0-01 · clean tracked Hub存在确定的函数参数不匹配

`HubRuntimeSession::persist_unchecked`在`runtime_state.rs:548`定义为零参数方法；`record_create_project_kept_folder_failure`在`project_actions.rs:583`却调用`self.persist_unchecked(None)`。Rust不支持按参数数量重载，该调用无法通过类型检查。

这个错误位于工程创建“目录已提交、Hub记录失败”的补偿分支，不是dead documentation。M0必须先恢复可编译基线，并增加编译gate与该分支的行为测试。不能只删掉补偿调用，否则会继续让部分成功终态在重启后不可见。

### ZHUB-P0-02 · Delete Project可直接回收正在被Editor编辑的工程

`prepare_project_editor_launch`会调用`probe_project_editor_session`避免重复打开，但`confirm_project_delete`没有任何session lease、dirty document、save/close或process owner检查。它在确认path后立即执行`recycle_delete(project.path)`，成功后才从Hub registry删除并持久化。

因此用户可在Editor持有project session、autosave和未保存文档时从Hub删除整棵工程目录。Editor之后的save、asset watcher、recovery journal和plugin写入将面对消失或被替换的根路径，存在直接数据损失。删除必须进入project-scoped destructive transaction：探测owner、向Editor请求Save/Discard/Cancel/Close并等待ack，冻结新写入，取得exclusive delete lease，记录durable intent，再执行可恢复移动。任何无法联系的active owner默认必须拒绝删除。

同一链还有第二个不可逆次序错误：回收成功后，`drop_project_from_hub`或`persist()`失败会把整个action记为失败，但文件已经离开原路径。重试“Delete”不能再次执行回收。终态必须是`FilesRecycledAwaitingRegistryRepair`，带recycle item identity和只修复metadata的命令。

### ZHUB-P0-03 · Hub重启会复活Editor在Hub关闭期间移除的recent project

三方合并的设计注释声称“外部Editor变化优先，旧Hub内存不能复活删除项”，但startup调用为：

```text
reconcile_shared_recent_projects(shared_path, [], config.recent_projects)
```

`previous_by_key`为空时，每个Hub config项目都会被判定为`changed_by_hub`并重新`registry.record`。复现链是：Hub关闭前config含A；Editor在Hub关闭期间从共享registry移除A；下次Hub启动读取旧config A，以空previous合并并把A写回。现有“stale snapshot不复活”测试只覆盖同一进程持有previous snapshot的情况；startup测试反而把“shared为空、config有A后写回A”固化为成功语义。

必须给每条recent mutation加入host/generation或operation journal，并保存Hub最后见到的shared revision。startup只能以持久化base revision做三方合并；无法证明本地记录是新mutation时，外部tombstone必须胜出。至少增加“Hub离线期间Editor删除”与“双方离线新增/删除交错”的跨重启测试。

### ZHUB-P0-04 · Editor spawn后唯一Child被丢弃，Hub失去监控、回收和真实终态

Project launch执行`let _child = launch_editor(command)?;`，随后最多轮询mailbox 10秒；函数返回时`Child`被drop。Rust `Child`的drop不会kill或wait。Empty Editor路径更直接，只返回`spawn()?.id()`，完全没有handshake。

后果包括：

1. 10秒超时只把Hub action记为失败，Editor可能仍在冷启动并随后正常运行；Hub无法Continue Wait、Cancel、Kill、Attach或Reap。
2. Unix child退出后可能长期成为zombie；Windows虽然关闭process handle，但进程继续运行且Hub无法获得exit code、crash dump或tree ownership。
3. mailbox Ready里的PID不与spawned child identity校验，也没有creation time/process nonce、heartbeat或terminal event。
4. Ready后`remember_project`或config persist失败会让background completion记录失败，但Editor已经运行；状态重试可能再开一个进程。
5. focus existing路径写文件后立即报告成功，没有Editor focus ack；PID存在不等于窗口已前置。

必须建立`EditorProcessSupervisor`，在Hub生命周期内持有root child/process-tree lease、stdout/stderr transport、startup milestones、last heartbeat和terminal outcome。启动、focus、shutdown、Hub close、engine update必须经过同一session record；超时是可继续/可取消的phase，不是释放owner的终态。

## 5. P1：Authority、工程生命周期、Build 与 Delivery 缺口

### 5.1 Startup、配置与共享状态

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| ZHUB-P1-01 | `HubCommandState::load()`在Tauri window创建前执行；任一config parse、recent manifest refresh、共享registry decode/lock或catalog刷新失败都会阻止UI出现。 | 两阶段bootstrap：最小窗口/diagnostic shell先启动，再后台加载各store；每个store有Healthy/Degraded/Quarantined/Repairing状态和独立恢复动作。 |
| ZHUB-P1-02 | `HubConfig::load`同步读取完整TOML，并对每个存在的recent project刷新manifest；一个损坏manifest即可让整个Hub启动失败。 | manifest加载逐项隔离并有file-size/time budget；损坏项目保留为Invalid/Missing条目，不得拖垮全局配置。 |
| ZHUB-P1-03 | `HubConfig`没有显式format/schema version、migration registry、minimum reader或forward-compat policy，仅靠`serde(default)`吸收字段。 | versioned config envelope、逐版本迁移、dry-run/backup/rollback和unsupported-future quarantine；迁移结果进入可审计journal。 |
| ZHUB-P1-04 | config使用固定`.tmp`名，没有Hub single-instance约束、writer lease、expected generation或CAS；多个Hub进程可互相覆盖/碰撞临时文件。 | 单实例/多实例策略必须明确；store写入使用unique staging、跨进程lease、generation CAS和lost-update重试。 |
| ZHUB-P1-05 | config/shared registry写入没有跨平台一致的file + parent-directory durability、last-good backup或corrupt-file quarantine。 | durable store抽象统一file sync、directory sync、backup rotation、checksum、startup recovery和故障注入语义。 |
| ZHUB-P1-06 | Windows recent writer lease用`WaitForSingleObject(INFINITE)`，Unix用blocking `flock(LOCK_EX)`；startup或每次persist可永久挂死，无deadline/cancel/owner诊断。 | deadline-aware lease，展示owner/等待时长，支持cancel、retry和只读降级；UI线程与startup shell不得无限等待。 |
| ZHUB-P1-07 | `search_projects`每次query都调用`persist()`且忽略错误；persist又会获取共享lease、读写registry和写全量config。快速输入可反复阻塞磁盘和全局mutex。 | ephemeral UI state与durable domain state分离；搜索只更新内存projection，持久化debounce且不得触碰共享recent registry。 |
| ZHUB-P1-08 | session所有字段挂在一个`Arc<Mutex<HubRuntimeSession>>`；focus refresh、manifest I/O、config save和projection refresh可在锁内执行，所有Tauri command共用一把锁。 | store/job/process/catalog分别拥有authority；锁内只做短事务与snapshot swap，I/O在锁外执行并以expected generation提交。 |

### 5.2 Background job、Build 与 Editor process

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| ZHUB-P1-09 | Build/Package/Install/Open Editor共享一个无界`VecDeque`和一个串行worker；长build会阻塞focus或启动，重复点击可无限堆积。 | 有界job coordinator：按resource/project/engine建lane，支持priority、dedup/coalesce、admission、queue age与backpressure。 |
| ZHUB-P1-10 | worker是detached `std::thread`，没有join handle、shutdown protocol、持久化queue或crash resume；Hub退出可在copy/build中间终止。 | owned job runtime，App close执行Cancel/Drain/Detach决策；durable operation journal在重启后reconcile staging与外部进程。 |
| ZHUB-P1-11 | job没有cancel token、deadline、pause/resume、retry policy或per-stage rollback；TaskStatus只用10/35/100的合成进度。 | typed phase graph和真实work units；每阶段声明cancel safety、deadline、retryability、compensation与terminal repair action。 |
| ZHUB-P1-12 | `run_build_command`使用`Command::output()`，把完整stdout/stderr无界缓存在内存，结束前无实时日志和进度。 | process supervisor流式读取有行长/条目/总字节/tick预算的日志，落盘滚动保存并实时投影；超限有计数而非OOM。 |
| ZHUB-P1-13 | build无process-tree owner、cooperative cancel、kill escalation、timeout和reap；工具启动的Cargo/链接器可能逃逸。 | Windows Job Object/Unix process group + cooperative cancel + deadline escalation + guaranteed reap，保留每级终止结果。 |
| ZHUB-P1-14 | build只凭exit code 0记成功；不验证staged Editor/Runtime、manifest、hash、ABI、platform/configuration或smoke launch。 | 发布`EngineBuildSetManifest`：source revision/dirty state、toolchain、target/config/arch、ABI、artifact hash与验证结果；只有原子验证通过的build set可激活。 |
| ZHUB-P1-15 | `SourceBuildRecord.status/profile`是自由字符串，历史只保留8条、日志只取最后6行，没有operation ID、duration、stage、cache stats或incident artifact。 | typed build outcome与profile，关联job/build-set/log artifact；retention按数量+字节+时间配额，失败保留first-error与完整诊断位置。 |
| ZHUB-P1-16 | focus existing只发布mailbox并立即成功，没有ack、window identity、deadline或foreground denial原因。 | focus request/ack协议携session、instance、window、sequence和result；Hub显示Focused/Denied/TimedOut/Stale，不把写文件当用户可见成功。 |
| ZHUB-P1-17 | launch handshake只有Ready/Failed，固定250 ms轮询/10 s timeout；没有phase、heartbeat、terminal event、mailbox TTL或startup scavenger。 | framed/evented local transport或有界watcher，提供Spawned/LockAcquired/ProjectLoaded/WindowReady/FirstFrame/Terminal；陈旧mailbox按owner/age清理。 |
| ZHUB-P1-18 | Hub close、engine removal/update和project remove不了解活跃Editor process；没有attach/restart/crash/hang/output/resource diagnostics。 | process registry按project + engine build set持session；所有破坏性操作先协调active consumers，提供Inspect/Focus/Graceful Exit/Force/Detach/Restart。 |

### 5.3 Engine与Project lifecycle

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| ZHUB-P1-19 | `SourceEngineInstall`只是`id/display/source_dir/output_dir/build_history`，没有发行版version/channel/platform components、download、signature、repair或uninstall。 | 统一`EngineInstallation`模型同时表达source build和signed distribution；安装器支持发现、下载/恢复、校验、原子激活、repair、rollback、uninstall与磁盘配额。 |
| ZHUB-P1-20 | source validation只解析workspace members字符串、检查`zircon_runtime`字样和build script文件；不处理Cargo glob/exclude、metadata、toolchain或完整workspace build set。 | 用`cargo metadata`/受控resolver验证workspace，冻结rust toolchain、host/target、required crates/tools和输出contract；结果可缓存且有revision key。 |
| ZHUB-P1-21 | engine ID是规范化source path的FNV hash；移动checkout会产生新identity，同一路径内容/版本变化又保持同identity。 | engine distribution/build identity基于publisher/product/version/channel/platform/build-set digest；location是可变安装属性，不是身份。 |
| ZHUB-P1-22 | project manifest的`engine_version_req`没有consumer；Hub另存一个私有`engine_id`，缺失engine时还会清掉binding。 | project声明要求是authority，Hub保存resolved build-set + resolver explanation；缺失安装保持`Unresolved`并提供Install/Locate/Change/Upgrade，不能静默丢绑定。 |
| ZHUB-P1-23 | `from_preferred_engine`只要Hub同目录存在`zircon_editor`就优先使用它，可能绕过project绑定的staged engine与ABI/version要求。 | launch必须从resolver冻结的build set取绝对artifact；development override需要显式标识、兼容检查和可见警告，不能隐式抢占。 |
| ZHUB-P1-24 | 打开project只验证manifest可解析，不像Godot/Unreal那样检查future version、unsupported features、required modules、upgrade/copy/recovery选择。 | `ProjectOpenPreflight`输出Exact/Compatible/UpgradeRequired/Future/FeatureMissing/RecoveryRequired，并提供Copy & Upgrade、Open Read-only、Locate Engine、Cancel。 |
| ZHUB-P1-25 | template catalog只有`renderable-empty`可用，其余三项是disabled保留项；没有template version、engine range、dependencies、content hash、signature或migration。 | versioned/signed template registry，支持内置与下载模板、dependency resolver、preview、license、offline cache、update/migration和创建receipt。 |
| ZHUB-P1-26 | create虽有staging/rollback，却没有per-target跨进程lease、file/parent sync、startup transaction scavenger；template与engine binding也不在同一durable transaction。 | project creation journal持request/template/build-set/staging/backup/commit阶段；跨进程锁定目标，sync后发布，重启可Finish/Rollback/Import Kept Folder。 |

### 5.4 Package、Install与不可逆终态

| ID | 当前差距 | 必须重构为 |
|---|---|---|
| ZHUB-P1-27 | Package只跳过`.git`和`target`后复制整棵project；`.zircon` recovery/autosave/play/cache、source-only和本机文件都可能进入产物。 | package recipe使用显式include graph和deny-by-default policy；用户数据、缓存、凭据、source-only文件有测试化排除规则。 |
| ZHUB-P1-28 | 没有cook/build/package阶段、目标platform/config/arch、runtime executable、asset dependency closure、shader pipeline、compression、chunking或signing。 | `CookPlan -> BuildSet -> StagedImage -> SignedPackage`流水线，所有输入/输出content-addressed并带可复现manifest。 |
| ZHUB-P1-29 | copy时没有project snapshot、Editor save barrier、source revision或read lease；工程可在递归复制中变化，生成混合时刻产物。 | package从冻结的document/asset/build revision或snapshot读取；变更后abort/restart，manifest记录完整input roots。 |
| ZHUB-P1-30 | package直接创建最终可见目录，失败时再best-effort递归删除；无staging marker、atomic publish、startup scavenger或quota。 | owner-scoped staging + durable manifest + atomic activate；失败/崩溃后由reconciler按journal清理或继续，所有临时空间受配额控制。 |
| ZHUB-P1-31 | package manifest只有name、绝对source path、timestamp、project dir和file count，没有每文件hash、build identity、schema、platform、entrypoint或signature。 | versioned package manifest含logical paths、sizes/hashes/modes、build/cook/toolchain/template IDs、entrypoints、dependencies、SBOM/license和signature chain。 |
| ZHUB-P1-32 | Install接受任意目录，不校验`zircon-package.toml`、schema、hash或签名；篡改/半成品也会复制。 | 安装入口只接受validated package handle；验证manifest、内容hash、publisher trust、platform capability、space与downgrade policy。 |
| ZHUB-P1-33 | “Device”只是用户配置的本地目录，没有device discovery、transport/auth、capability、free space、版本查询、launch/log、update/uninstall/rollback。 | `DeviceProvider`插件契约，区分local/USB/ADB/SSH/console等transport；install session可续传、验证、激活、健康检查和回滚。 |
| ZHUB-P1-34 | install直接写最终目录，失败再删除；没有slot/staging、atomic activation或existing install update策略。 | inactive slot或versioned staging安装，验证通过后原子切换current；失败保持旧版本并产生repairable receipt。 |
| ZHUB-P1-35 | receipt为每个文件`fs::read`整文件后手写SHA-256；大文件峰值内存无界，也没有文件变化检测或成熟crypto implementation。 | 流式hash、固定buffer、metadata-before/after校验、成熟crypto crate和并发/总内存预算；测试覆盖标准向量、大文件与故障。 |
| ZHUB-P1-36 | `content_download_manifest`指向已经安装好的`file://`文件，却把各文件伪装成一个连续offset流并标记range resume；Install失败后已成功Package也没有独立成功记录。 | download manifest描述真实remote blobs/chunks/mirrors/ETag与resume contract；Package和Install是两个有独立terminal outcome、artifact owner和可重试边界的job。 |

## 6. P2：可维护性、诊断与产品质量

| ID | 当前差距 | 建议收敛方向 |
|---|---|---|
| ZHUB-P2-01 | recent 8项、action history 16项、build history 8项和log excerpt 6行分散为固定常量，缺少字节/时间预算和用户策略。 | 统一retention policy，按items/bytes/age三维限制并公开diagnostic counters。 |
| ZHUB-P2-02 | action/task/build状态仍混合自由字符串与`HubMessage`，日志、history和UI的terminal vocabulary不完全一致。 | 统一typed outcome/stage/recovery ID，localization只在projection末端发生。 |
| ZHUB-P2-03 | package/receipt/build history保存本机绝对路径，降低可移植性、隐私和可复现性。 | manifest使用logical roots/content IDs；机器路径只留在受控local diagnostic中并支持redaction。 |
| ZHUB-P2-04 | package/install目录仅用毫秒timestamp命名，同名同毫秒operation会冲突。 | 使用operation UUID + deterministic artifact digest；人类时间只作display metadata。 |
| ZHUB-P2-05 | folder picker、open folder、recycle等platform capability散落为cfg分支/外部命令，unsupported通常到点击后才发现。 | 启动时发布platform capability matrix；command只在provider可用时enable并提供替代路径。 |
| ZHUB-P2-06 | 设置里保存`rustup_path`，当前build command不消费；python/cargo/rustup也没有版本/来源/信任预检。 | Toolchain inventory显示resolved executable、version、source、target availability与repair命令，移除无owner字段。 |
| ZHUB-P2-07 | 没有统一operation correlation、structured trace、duration/resource指标、crash bundle或可导出support report。 | 每个job/session/store transaction携operation ID，输出有界structured event与可脱敏support bundle。 |
| ZHUB-P2-08 | focused tests中大量断言源码包含某些字符串，能防止表面回退却不能证明跨进程、崩溃、竞争、断电或性能语义。 | 保留轻量contract tests，同时增加process/fs fault harness、property tests、cross-restart scenarios和规模基线。 |

## 7. 参考实现差异

| 参考 | 已核对的具体能力 | Zircon当前差距与采用边界 |
|---|---|---|
| Unreal | `ProjectEditorRecords`把project映射到engine location并用系统锁串行load-latest/mutate/save；`InstalledPlatformInfo`表达configuration/platform/target/architecture/required file与Supported/Downloaded；Project Browser在打开前检查engine/version/status并提供copy/convert/build。 | Zircon需引入project requirement resolver、installed build-set capability和跨进程transaction。不能只复制Unreal UI或把source path hash当engine install。 |
| Godot | Project List把扫描放到可取消thread，保留Missing/Unsupported/Recovery状态；Project Manager在打开前处理future config、unsupported features、version conversion与recovery mode。 | Zircon startup不能因单个坏manifest失败，也不能把missing entry直接抹掉；需要显式open preflight与修复状态。Godot的简单config save不是Zircon多host registry竞争的充分标准。 |
| Fyrox | Project Manager的`Mode::CommandExecution`持有`Child`、piped stdout/stderr、`try_wait`和close confirmation；export至少区分PC/Web/Android、build target、asset copy、binary copy、cancel和run-after-build。 | Fyrox证明即便较小manager也必须保留child ownership和target-aware export。其队列/删除实现仍较简单，不作为Zircon工程级事务和安全删除上限。 |
| Bevy | 本地checkout主要提供runtime/app/asset processor primitives，没有可比的Hub、engine installer或完整project manager owner。 | 只在后续cook/cache/job实现中借鉴asset processing与typed schedule；不从“参考树没有Hub”推导Zircon可以省略控制面。 |
| Unity Graphics | 本地`dev/Graphics/Packages`是渲染package源码，不含Unity Hub、Editor installation或device deployment authority。 | 可在后续shader/package consumer中参考具体包格式；本报告不猜测闭源Unity Hub实现，也不拿缺失源码替Zircon背书。 |

## 8. 目标架构

### 8.1 五个独立authority

1. `HubStoreCoordinator`：versioned config/recent/project binding，generation CAS、migration、backup/quarantine与transaction journal。
2. `EngineInstallationService`：source/distribution catalog、resolver、download/build/verify/activate/repair/rollback/uninstall和build-set manifest。
3. `ProjectLifecycleService`：create/import/open/upgrade/remove/delete的project-scoped lease、preflight、Editor coordination与durable terminal state。
4. `HubJobCoordinator`：有界多lane job、resource admission、progress/cancel/deadline/retry、shutdown和restart reconciliation。
5. `EditorProcessSupervisor`与`DeliveryPipeline`：前者持进程树/session/transport，后者执行snapshot/cook/build/package/sign/install/verify/activate。

UI只消费这些authority的immutable snapshot并提交typed command。`HubRuntimeSession`不能继续同时拥有配置store、长任务队列、process状态、catalog projection和所有页面ephemeral state。

### 8.2 必须显式表达的终态

- `ProjectCreatedAwaitingHubRegistration`
- `FilesRecycledAwaitingRegistryRepair`
- `EditorReadyAwaitingHistoryCommit`
- `EditorStartupTimedOutStillOwned`
- `BuildExitedAwaitingArtifactValidation`
- `PackagePublishedAwaitingHistoryCommit`
- `PackageReadyInstallFailed`
- `InstallVerifiedAwaitingActivation`
- `StoreCorruptQuarantined`
- `MigrationFailedRolledBack`

这些状态必须保留不可逆事实和剩余owner，不能统一压成一个Failed字符串。

## 9. 分阶段重构路线

### M0 · 恢复可编译与建立故障基线

- 修复`persist_unchecked`调用契约并为kept-folder分支加行为测试。
- 给当前P0写最小复现：active Editor delete拒绝、Hub离线recent tombstone、slow Editor launch仍被owner持有。
- 修复后用同一管理验证器重跑`cargo build -p zircon_hub --locked`和focused tests；失败不得绕过共享target协调器。

### M1 · Store与project destructive transaction

- 引入versioned config、unique staging、generation CAS、deadline lease、backup/quarantine和startup diagnostic shell。
- 修复recent跨重启三方合并，引入revision/tombstone。
- Delete/Remove/Create迁移到project-scoped journal与exclusive lease；active Editor必须走close protocol。

### M2 · Job coordinator与process supervisor

- 替换detached singleton FIFO；实现有界lane、cancel/deadline/progress/shutdown/restart reconciliation。
- Editor launch保留Child/process tree，升级typed milestone/heartbeat/focus ack/terminal transport。
- build输出改为有界stream，加入process-tree termination和完整log artifact。

### M3 · Engine resolver与build-set发布

- 定义distribution/source统一identity、semantic requirement resolver和project binding explanation。
- 用metadata/toolchain preflight替代字符串workspace检查。
- build成功后生成并验证build-set manifest，原子激活；Hub sibling Editor只能作为显式development override。

### M4 · Project open/upgrade/template产品链

- Project open加入version/feature/module/recovery preflight和Copy & Upgrade流程。
- template变为versioned signed pack，创建receipt记录template/build-set/input。
- missing/corrupt/unresolved项目保留可修复条目，不让单条失败拖垮Hub。

### M5 · Cook/package/sign与真实device provider

- 建立冻结snapshot、asset dependency closure、shader/cook/build/package/sign流水线。
- package/install全部使用staging + verify + atomic activation + rollback。
- 引入provider式device discovery/transport/capability/log/launch/update/uninstall，移除伪`file://` download manifest。

### M6 · 规模、可靠性与发布gate

- 10k projects、数百engine build sets、百万文件package、100 GiB install的时间/内存/磁盘预算。
- 进程crash/hang、Hub kill、Editor slow start、writer contention、disk full、power-loss point、network resume、signature failure矩阵。
- support bundle、trace/metrics、retention/quota、accessibility和cross-platform发布验收。

## 10. 验收门

1. `zircon_hub` clean checkout通过managed Windows `cargo check`；kept-folder补偿分支可编译并有运行测试。
2. active Editor持project lease时，Hub Delete默认拒绝；完成Save/Discard/Cancel/Close ack前不会移动任何文件。
3. recycle成功后registry写失败进入`FilesRecycledAwaitingRegistryRepair`；Repair不会再次删除文件。
4. Hub关闭期间Editor删除recent A，Hub重启不会复活A；双方新增/删除交错有跨重启property test。
5. config/shared registry lock等待有deadline、cancel与owner diagnostic，Hub窗口可在store降级时启动。
6. 单个坏project manifest、坏recent registry或future config不会让Hub白屏；原文件被隔离且last-good可恢复。
7. 两个Hub writer并发不会lost update、覆盖tmp或产生不可解析文件；generation冲突可重试。
8. 搜索输入不产生config/shared-registry I/O；UI command锁持有时间有p99预算。
9. background queue有entry/byte/age上限、priority与per-resource lane；build不阻塞focus existing Editor。
10. Hub关闭时所有job明确Drain/Cancel/Detach，重启会reconcile staging、child和terminal journal。
11. build日志峰值内存有硬上限，实时可见；1 GiB单行/持续输出不会OOM或卡死UI。
12. cancel build可终止完整process tree并reap；自然退出和终止失败都保留真实terminal outcome与owner。
13. exit code 0但artifact缺失/hash/ABI错误不会激活build set；旧active build set保持可用。
14. project `engine_version_req`得到确定resolver结果；missing/future/incompatible有Install/Locate/Upgrade/Cancel路径。
15. Hub旁边的Editor不能隐式覆盖project resolved engine；development override在history和window中可见。
16. slow Editor启动超过10秒时Hub仍持owner，可Continue Wait或Cancel；PID/creation identity与handshake一致。
17. focus request只有收到目标instance ack后才成功；stale PID/instance和foreground denial有明确状态。
18. package来自冻结revision，不包含`.zircon`或机器私有文件；manifest逐文件hash并关联cook/build-set/toolchain。
19. install先验证package，再写inactive staging；验证/激活失败保持旧版本并可rollback，峰值内存受预算。
20. Windows/Linux至少覆盖create/open/build/package/install/delete与Hub kill/disk full/lock contention；报告包含duration、peak RSS、I/O、遗留进程/文件和可恢复终态。

## 11. 本轮未闭合范围

- Hub React/Tauri页面的loading/empty/error/disabled/accessibility、virtualization和大catalog渲染另立UI报告。
- Learn/Team/Plugin/Assets catalog、cloud预留服务、登录/权限、网络下载/update服务仍未完成纵向审查。
- 本轮没有把旧`docs/plans/zircon_hub/*`的“已实现”声明当产品证据；只有当前clean source和可定位reference source用于结论。
- 修复编译P0后必须重跑managed build与focused tests。任何实施开始前都要重读source、重算fingerprint并重新确认其余P0是否仍存在。
