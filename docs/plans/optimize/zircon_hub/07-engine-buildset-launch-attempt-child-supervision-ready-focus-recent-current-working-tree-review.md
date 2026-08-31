---
related_code:
  - zircon_hub/src/build
  - zircon_hub/src/engines
  - zircon_hub/src/process
  - zircon_hub/src/projects
  - zircon_hub/src/settings/hub_config.rs
  - zircon_hub/src/state
  - zircon_hub/src/tauri_app/commands.rs
  - zircon_hub/src/tauri_app/runtime_state.rs
  - zircon_hub/src/tauri_app/runtime_state
tests:
  - zircon_hub/src
  - zircon_hub/tests
plan_sources:
  - docs/plans/optimize/zircon_hub/01-project-engine-build-editor-launch-process-persistence-delivery-review.md
  - docs/plans/optimize/zircon_hub/06-current-source-product-control-plane-project-lifecycle-process-delivery-web-host-test-evidence-review.md
  - docs/plans/optimize/zircon_runtime_interface/16-project-manifest-admission-ready-focus-recent-buildset-correlation-current-working-tree-review.md
  - docs/plans/optimize/zircon_editor/268-editor-project-startup-open-create-activation-session-recent-recovery-current-working-tree-review.md
  - docs/plans/zircon_hub/03/failure-2026-08-27-shared-recent-project-load-import.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Public/Misc/MonitoredProcess.h
  - dev/UnrealEngine/Engine/Source/Runtime/Core/Private/Misc/MonitoredProcess.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Public/ProjectEditorRecords.h
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/ProjectEditorRecords.cpp
  - dev/UnrealEngine/Engine/Source/Developer/DesktopPlatform/Public/InstalledPlatformInfo.h
  - dev/UnrealEngine/Engine/Source/Programs/UnrealBuildTool/System/TargetReceipt.cs
  - dev/UnrealEngine/Engine/Source/Editor/GameProjectGeneration/Private/SProjectBrowser.cpp
  - dev/godot/editor/project_manager/project_list.cpp
  - dev/godot/editor/project_manager/project_manager.cpp
  - dev/Fyrox/project-manager/src/manager.rs
  - dev/Fyrox/fyrox-build-tools/src/export/mod.rs
  - dev/bevy/crates/bevy_app/src/app.rs
  - dev/bevy/crates/bevy_app/src/schedule_runner.rs
  - dev/Graphics/Packages/com.unity.render-pipelines.core/package.json
  - dev/Graphics/.yamato/wrench/package-pack-jobs.yml
  - dev/Graphics/.yamato/wrench/api-validation-jobs.yml
  - dev/Graphics/.yamato/wrench/promotion-jobs.yml
doc_type: current-working-tree-review-and-refactor-plan
review_status: review_complete
implementation_status: pending
supersedes_currentness_of:
  - docs/plans/optimize/zircon_hub/01-project-engine-build-editor-launch-process-persistence-delivery-review.md#4-p0-阻断项
  - docs/plans/optimize/zircon_hub/01-project-engine-build-editor-launch-process-persistence-delivery-review.md#52-background-jobbuild-与-editor-process
  - docs/plans/optimize/zircon_hub/01-project-engine-build-editor-launch-process-persistence-delivery-review.md#53-engine与project-lifecycle
  - docs/plans/optimize/zircon_hub/06-current-source-product-control-plane-project-lifecycle-process-delivery-web-host-test-evidence-review.md
source_recheck_required: true
---

# 07 · Hub Engine / BuildSet / Launch Attempt / Child Supervision / Ready / Focus / Recent 当前工作树复核

> report_id: `Hub07`
> canonical_finding_owner: [Hub01](01-project-engine-build-editor-launch-process-persistence-delivery-review.md)
> full-Hub baseline: [Hub06](06-current-source-product-control-plane-project-lifecycle-process-delivery-web-host-test-evidence-review.md)
> cross-crate contract baseline: [Interface16](../zircon_runtime_interface/16-project-manifest-admission-ready-focus-recent-buildset-correlation-current-working-tree-review.md)
> Editor consumer baseline: [Editor268](../zircon_editor/268-editor-project-startup-open-create-activation-session-recent-recovery-current-working-tree-review.md)
> observed HEAD: `17b92691e1d67c3df09376a2ca599bf2e07a061d`，观察日期 `2026-08-31`；HEAD 在审查期间并发前进，但 Hub source/test 指纹复算未漂移
> status: review-only；不修改 production Rust、Cargo、ABI、tests 或 UI，不运行 Cargo、Hub/Editor 双进程、fault、scale、soak 或 benchmark。Tooling 排除；不查询、轮询、等待或实时跟踪协调器。

## 1. 结论

Hub 当前已经不是完全没有协议：launch intent 是版本化类型，Editor Ready 在 first-present callback 后发布，Ready receipt 带 PID、instance、generation 和固定 milestone 集；existing Editor focus 要等待精确 request ack；recent store 已有 revision、CAS、bounded lease、quarantine 和 durable replace；后台队列也增加了 64 项上限。这些都是真实进展，应当保留。

但端到端工程语义仍然不成立。当前路径实际上是：

```text
recent project path
  -> private metadata.engine_id existence check
  -> mutable global settings source/output
  -> exit-code-only build success
  -> preferred executable, possibly Hub sibling override
  -> spawn Child
  -> pass only child.id() into fixed mailbox polling
  -> drop Child without wait/kill/reap/terminal ownership
  -> validate Ready PID only
  -> persist recent/history
  -> project a final Success string
```

这条链没有冻结 `ProjectIdentity -> engine requirement -> resolved BuildSet -> executable artifact -> launch attempt -> process creation identity -> Ready -> running -> terminal outcome`。因此它无法证明用户打开的是项目要求的引擎，无法证明构建成功产出了可运行且 ABI 相容的 Editor/Runtime，也无法在 timeout、Hub shutdown、Editor crash、persist failure 或 PID reuse 时给出唯一、可恢复的终态。

本轮不新增唯一 finding，继续使用 Hub01 的 canonical 编号。聚焦重判结果是：P0 **2 Open**；P1 **13 Open / 4 Partial / 0 Closed**；P2 **0 Open / 1 Partial / 0 Closed**。在 Hub01-05 的 323 项唯一账本中，保守全局状态由 Hub06 的 `322 Open / 0 Partial / 1 Closed` 更新为 **317 Open / 5 Partial / 1 Closed**。5 个 Partial 仅是 `ZHUB-P1-06`、`ZHUB-P1-09`、`ZHUB-P1-16`、`ZHUB-P1-17`、`ZHUB-P2-08`，不代表对应工作流已完成。

## 2. 审查边界与证据冻结

### 2.1 当前选择集

| 选择集 | files | lines | non-empty | bytes | tests | ignored | fingerprint |
|---|---:|---:|---:|---:|---:|---:|---|
| Hub Rust owner source | **101** | **25,654** | **23,400** | **938,354** | **284** | **8** | `395528bad169f1f8636cce81f5dde641ec105780266939106595a93cee4e4c95` |
| Hub integration tests | **40** | **19,466** | **18,519** | **715,625** | **271** | **1** | `6d8f912a211774136de672e538a1556afba984dc98eaf51fd4208c83c926e550` |
| Unreal/Godot/Fyrox/Bevy/Unity Graphics references | **17** | **11,936** | **10,422** | **416,788** | **19** | **0** | `087c2da4c6e66dbd9c1eb57916d90e6afb45851a23a55d167048d05baf459a34` |

Hub source选择集覆盖 `build`、`engines`、`process`、`projects`、`settings`、`state`、全部 Tauri command/runtime state/view model 及其内联测试；不是只搜索 `Child` 或 `BuildSet`。跨 crate 的 manifest、identity、compatibility、BuildSet、session、Ready、focus、recent 及 App/Editor 消费链沿用 Interface16 已冻结的 232 文件选择集，本篇只记录 Hub owner 结论，避免重复统计。

指纹算法为：workspace 相对路径转 `/` 并小写，逐文件计算 lowercase SHA-256，按路径 ordinal 排序，再对 `path + NUL + hash + LF` 的 UTF-8 字节流计算 SHA-256。当前工作树存在大量并发 staged/unstaged/untracked 修改；这些 bytes 纳入观察，但不等于 clean checkout 或已集成交付。

### 2.2 动态证据边界

本轮没有运行 Cargo。静态证据足以确认所有权、调用顺序、字段消费、阻塞策略、持久化次序和缺失的 production consumer；不能证明编译、真实进程行为、Windows Job Object/Unix process group、崩溃恢复、ACL、跨进程竞争、性能或发布资格。

`docs/plans/zircon_hub/03/failure-2026-08-27-shared-recent-project-load-import.md` 仍记录 recent import/identity 测试的受管验证缺口。本篇只把它作为验证边界，不等待该外部状态，也不把未执行测试写成 Pass。

## 3. 当前实现中必须保留的底座

| 能力 | 当前真实实现 | 保留边界 |
|---|---|---|
| typed launch intent | `ProjectLaunchIntent` 带 operation、source、profile 与 typed target；路径经 JSON 参数传递 | intent 只是请求，不得被误当成 admission、identity 或 BuildSet approval |
| App Runtime preflight | App 的 `LoadedRuntime::preflight_default()` 校验 Runtime artifact sidecar，并把 BuildSet ID 传入 Editor project startup | 该事实目前不受 Hub resolver 约束，不能替代 Hub 的 expected BuildSet |
| first-present Ready | Editor 在 focus binding 后构造 receipt，并在 `ui.on_first_presented` callback 发布 | 保留 first-present fence；改为 request-bound evidence，不保留固定全集 constructor 的过强语义 |
| Ready PID check | Hub 拒绝 receipt PID 与刚 spawn PID 不同的 mailbox | 这是必要但不充分的检查；还需 creation identity、operation、BuildSet、project digest 与 nonce |
| focus acknowledgement | signal/ack 绑定 request UUID、instance、generation、sequence 和 typed rejection | 保留协议字段；重做 namespace owner、bounded read、monotonic wait、cleanup 和 ACL |
| recent projection store | revision/CAS、stale delete protection、250 ms Hub writer timeout、retry、quarantine、sync | 保留 store 原语；修复 startup base、identity、诊断投影和业务 owner |
| bounded item queue | queue 在 clone 前拒绝第 65 个 request，FIFO 行为有单元测试 | 只保留 item admission；补 bytes、lane、fairness、deadline、cancel、durable operation |
| panic containment | background loop 捕获单个 action panic 并投影错误 | 不得用 catch-unwind 代替 owned runtime、join、shutdown、compensation 和 restart reconcile |

## 4. 当前端到端事实

### 4.1 Engine binding 与实际 build command 已经漂移

`build_actions.rs:70-114` 先调用 `selected_or_latest_recent_project_with_engine_for_action()`，后者只确认 project metadata 中有一个仍存在于 `config.engines` 的私有 `engine_id`。随后真正的 `BuildCommand` 却固定读取：

- `settings.python_path`
- `settings.cargo_path`
- `settings.default_source_dir`
- `settings.default_build_output_dir`
- `settings.build_profile`
- `settings.jobs`

它没有从刚确认的 project-bound engine 冻结 source/output/toolchain，也没有把 project requirement 或 manifest digest 放入 pending build。结果是项目可以通过“绑定 Engine A”的前置检查，实际执行却使用 settings 中的 source/output B；完成时又把记录写到当前 active engine。此处是 `ZHUB-P1-14` 与 `ZHUB-P1-22` 的当前最强证据，不应另建重复 finding。

`BuildExecutionReport::succeeded()` 仍只判断 `status_code == Some(0)`。成功分支立即写入 `StagedEditorRuntimePayload`，没有检查 Editor/Runtime 文件是否存在、artifact sidecar、hash、ABI、target、configuration、architecture、toolchain、source revision、smoke load 或原子激活。Hub production source 对 `ZrRuntimeBuildSetId` 的唯一命中仍在 focus probe 测试 fixture；没有 BuildSet resolver、manifest consumer 或 receipt producer。

### 4.2 EngineInstallation 仍是 source-folder bookmark

`SourceEngineInstall` 只持有 `id/display_name/source_dir/output_dir/last_build/build_history`。engine ID 是 normalized source path 的 FNV-1a hash；移动 checkout 会换身份，同一路径内容、revision、toolchain 或 target 改变却不换身份。registry 在 active engine 缺失时选择第一项，并会把项目上指向缺失 engine 的 binding 清空。

`validate_source_engine()` 同步整文件读取 `Cargo.toml`，只检查 directory、workspace members 中出现 `zircon_runtime` 路径字符串，以及 `tools/zircon_build.py` 存在。它不使用 `cargo metadata` 或受控 resolver，不解析 glob/exclude，不冻结 toolchain/host/target/required artifacts，也不生成可缓存的 revision-keyed qualification receipt。

### 4.3 Launch attempt 没有长期 owner

`PendingEditorLaunch::run()` 的 project 分支执行：

```rust
let child = launch_editor(command)?;
wait_for_project_editor_ready(project_path, *handshake_session, child.id())?
```

`wait_for_project_editor_ready` 只接收 PID，未接收 `&mut Child`。函数离开分支时 `Child` 被 drop；Rust `Child::drop` 不会 kill 或 wait。Empty Editor 分支更直接，执行 `Command::spawn()?.id()` 后立即丢 owner。全 Hub source 没有任何 `Child` 字段或 process registry。

因此 10 秒 timeout、malformed mailbox、Ready mismatch、Hub close 和完成阶段 persist failure 都不会触发 `try_wait/kill/wait/reap`。Hub 也无法知道 Editor 后续退出、崩溃、卡死、升级阻塞或产生 orphan process。源码错误信息“not bound to the Hub-supervised child process”强于事实：这里只证明 PID 相等，不存在 supervision。

### 4.4 Ready 是 first-present receipt，但不是 launch transaction receipt

`HubEditorReadyReceiptV1` 强制五项固定 milestone：SessionCommitted、NativeWindowCreated、FirstPresent、FocusInboxBound、Interactive。Editor 确实在 `on_first_presented` 回调中发布，因此 Hub01 中“Ready 未经 first frame”的旧表述应更新。

但 receipt 只有 PID、Editor instance ID、session generation 和固定 milestone set。Hub 只校验 PID，不校验：

- launch operation ID
- process creation identity / nonce
- `ProjectIdentity` / ProjectGuid / manifest digest
- expected BuildSet ID / artifact manifest digest
- admission record digest / principal
- issue time、deadline、receipt sequence

Hub 也不把 receipt 的 instance/generation 写入长期 process record。Ready 后先 `remember_project()`，再写 action history；任何 recent/config persist 失败都会把 action 变成 failure，尽管 Editor 已经运行。正确终态应是 `EditorRunningAwaitingHubProjectionRepair`，而不是允许重试再 spawn。

### 4.5 Handshake mailbox 没有有界、claim 和 lifecycle

handshake 使用 `.zircon/hub/{session}.json`，每 250 ms 轮询一次，固定 10 秒。`read_editor_handshake()` 直接 `fs::read` 后 JSON decode，没有文件 byte cap、opened-file identity、owner/ACL 校验、claim/rename、成功 remove 或 startup scavenger。malformed mailbox 会立即失败并永久留下；Ready mailbox 被消费后也不删除。

wait loop 不同时观察 child exit，不支持 cancel、continue wait 或 deadline extension。timeout 是本地 action failure，却无法决定 child 是仍在启动、已经退出还是之后会写 Ready。

### 4.6 Focus 已从 Open 推进到 Partial，但仍不是完整 owner 协议

当前 focus 的积极变化：

- 只向 Ready 且有 generation 的 session record发布。
- request 带 UUID、target instance、generation、sequence、10 秒 deadline。
- pending item 上限为 32。
- Hub 等待精确 ack；只把 `Focused` 当成功，并能显示 expired/target mismatch/inbox full/stale rejection。
- Editor 在 native focus callback 后发布 owner-confirmed acknowledgement。

剩余风险：

- probe 先检查 OS lease，再单独 `read_to_string` record，存在 lease/record TOCTOU；Windows 只证明同名 mutex 存在。
- Hub 不比较 active admission record 的 BuildSet 与 project expected BuildSet。
- cleanup 对每个 JSON 先无界 `fs::read`；malformed 文件永不清理，却计入 32 项上限。
- `filter_map(Result::ok)` 隐藏目录枚举错误。
- ack 也是 `fs::read` 后才检查 4 KiB；mismatched poison file 不删除。
- ack timeout 依赖 wall clock；系统时钟回拨可能延长等待。
- sequence 是 process-local `AtomicU64`，Hub 重启后归一；没有 BootId/issuer generation 或 durable replay set。
- Hub 可以自行创建 Editor inbox directory，没有 owner-created namespace、ACL 或 root capability 证明。

因此 `ZHUB-P1-16` 可标 Partial，但不得 Closed。

### 4.7 Recent store 原语进步，startup 仍能复活离线删除

`reconcile_shared_recent_projects_snapshot()` 已使用 revision、CAS retry、stale delete rebase 和 250 ms writer timeout；同进程 stale snapshot 测试也覆盖“旧 Hub 状态不能恢复刚删除项目”。

但 `HubRuntimeSession::load_from_paths()` 仍以 `SharedRecentProjectsSnapshot::default()` 作为 previous base，再把 `config.recent_projects` 当作本次 Hub mutation 合并。Hub 离线期间 Editor 删除 A 后，旧 config 中的 A 会在下次启动被视为“Hub 新增”，再次写回 shared registry。已有 startup fixture 仍把 shared 为空、config 有项目后写回项目固化为成功语义。

另外 Hub wrapper 丢弃 `HubRecentProjectsLoadDisposition` 和 quarantine path，corrupt/oversized projection 会作为空投影继续重建，产品层看不到 Degraded/Quarantined/RepairRequired。recent identity 仍是 normalized/lossy path key；虽然 summary 已带 ProjectGuid，Hub 不用它做 identity。

### 4.8 Queue 只限制 item 数，没有 operation runtime

`BACKGROUND_ACTION_QUEUE_CAPACITY = 64` 关闭了“无限 item 数”的一小部分风险，所以 `ZHUB-P1-09` 重判 Partial。请求仍可带任意 JSON payload，队列 clone 没有 byte budget；Build/Package/Install/Open Editor 仍共享单一 FIFO worker，没有 lane、priority、dedup、coalesce、fairness 或 resource key。

worker 由 `thread::spawn` 启动且丢弃 join handle；focus refresh thread同样 detached。没有 stop admission、drain/cancel/detach policy、shutdown deadline、operation journal 或 restart reconcile。queued request 也没有冻结 resolved project identity、engine/BuildSet 和 immutable target；执行时仍会修改并读取当前 selected project/session state。

## 5. Canonical finding 当前重判

下表是 Hub01 finding 的 currentness refresh，不新增唯一计数。

| Canonical ID | 当前状态 | 当前证据与关闭条件 |
|---|---|---|
| `ZHUB-P0-03` | **Open** | startup 仍以 revision 0/default snapshot 合并旧 config；必须持久化 last-seen base/revision/tombstone，并通过 Hub 离线删除跨重启测试 |
| `ZHUB-P0-04` | **Open** | project/empty launch 都丢 `Child`；必须建立 Hub 生命周期内 owned process/process-tree、terminal outcome 与 guaranteed reap |
| `ZHUB-P1-06` | **Partial** | shared recent writer 已有 timeout/cancel policy，Hub 使用 250 ms；仍缺 owner/wait diagnostics、产品 cancel/read-only degradation 和完整 caller 资格 |
| `ZHUB-P1-09` | **Partial** | queue 有 64 item cap；仍缺 byte budget、lane、fairness、priority、dedup/coalesce 和 queue-age SLO |
| `ZHUB-P1-10` | **Open** | worker/focus refresh thread 仍 detached，无 join、shutdown、journal、restart reconcile |
| `ZHUB-P1-11` | **Open** | operation 无 cancel/deadline/pause/retry/compensation，progress 仍是合成百分比 |
| `ZHUB-P1-12` | **Open** | build 仍使用 `Command::output()`，完整 stdout/stderr 在结束前驻留内存 |
| `ZHUB-P1-13` | **Open** | build 无 Job Object/process group、kill escalation、timeout、tree reap |
| `ZHUB-P1-14` | **Open** | exit code 0 仍直接写 staged success；Hub 不消费或生成 BuildSet artifact manifest |
| `ZHUB-P1-15` | **Open** | build history status/profile 仍是自由字符串、8 条截断、6 行日志，无 operation/buildset/stage/duration/artifact |
| `ZHUB-P1-16` | **Partial** | 已有精确 focus ack 和 typed rejection；namespace、clock、bounded read、poison cleanup、ACL 与 session recheck 未闭合 |
| `ZHUB-P1-17` | **Partial** | 已有 first-present typed Ready、PID/instance/generation；仍是 fixed polling，无 child race、heartbeat、claim/scavenge、request/BuildSet correlation |
| `ZHUB-P1-18` | **Open** | open path能 probe existing lease，但 Hub 无 active process registry，delete/engine removal/update/close 不协调 active consumers |
| `ZHUB-P1-19` | **Open** | EngineInstallation 仍是 source/output bookmark，无 signed distribution、components、repair、rollback、uninstall |
| `ZHUB-P1-20` | **Open** | workspace validation 仍是 TOML member 字符串与文件存在检查 |
| `ZHUB-P1-21` | **Open** | engine identity 仍是 source path FNV hash，不代表 publisher/version/channel/platform/BuildSet |
| `ZHUB-P1-22` | **Open** | Hub 不消费 `engine_version_req`/compatibility/ProjectIdentity，缺失 engine binding 仍会被 prune |
| `ZHUB-P1-23` | **Open** | sibling `zircon_editor` 仍隐式优先于 configured staged engine |
| `ZHUB-P1-24` | **Open** | Hub 仍把 summary parse 收敛为 Valid/Invalid，没有 Exact/Upgrade/Future/Feature/Recovery 产品决策 |
| `ZHUB-P2-08` | **Partial** | queue/focus/handshake/recent已有真实单元行为测试；仍无双进程、kill/restart、PID reuse、clock/fault/ACL/scale 资格 |

## 6. 与参考引擎的工程差异

| 参考 | 本轮核对的源码事实 | Zircon 应采用的语义边界 |
|---|---|---|
| Unreal `FMonitoredProcess` | 长期对象持有 process handle、pipes、thread、running state、return code；持续读输出，timeout/cancel 能终止 process tree，析构会处理仍运行实例 | Hub process supervisor 必须拥有 handle、stream、timeout、kill tree 与 terminal receipt；不能只复制 API 名称 |
| Unreal `InstalledPlatformInfo` | installed configuration 同时表达 platform、configuration、target type、architecture、required file 与 Supported/Downloaded | Engine capability 要按 BuildSet/target/component 查询，不再用 source folder presence 代表可用 |
| Unreal `TargetReceipt` | target receipt 记录 platform、configuration、architecture、version、launch executable、build products 与 runtime dependencies，并严格读写字段 | Hub build success 要产出可验证、可激活的 artifact manifest；exit code 只是一个 stage fact |
| Unreal `ProjectEditorRecords` / Project Browser | project-engine mapping 在 system-wide lock 下 load-latest/mutate/save；Browser 打开前检查 status/engine/version 并提供 upgrade/build/copy 路径 | Hub resolver 与 recent/engine mapping 要有跨进程 currentness；preflight 结果必须成为产品状态和可恢复 action |
| Godot Project Manager | missing、future config、older conversion、unsupported features、recovery mode 分别建模；升级可先 backup/duplicate，future version拒绝 | Hub 不得把全部失败压成 InvalidManifest；必须区分不可打开、可升级、可降级、可恢复及用户批准 |
| Fyrox Project Manager | `Mode::CommandExecution` 在 manager 状态中长期保存 `Option<Child>`，逐 tick `try_wait`，串行 command queue 与 UI 生命周期相连 | 即使这个参考并非完整 process-tree supervisor，也已强于 Zircon 的 PID-only/drop；Zircon 需进一步补 cancel/kill/reap/terminal |
| Bevy App | runner 拥有 App 主循环；plugin `finish`、`cleanup` 和 `AppExit` 是显式阶段与终止结果 | Hub worker/process runtime 要有 finish/cleanup/shutdown phase，不能依赖 thread/Child drop 表达生命周期 |
| Unity Graphics | package 固定 version/dependency；Yamato 把 pack artifact、API validation、多平台 validation 与 promotion 依赖串联 | 只借鉴 build artifact/promotion 资格链；Graphics 源码不提供 Hub process supervision 参考，不应越界类比 |

## 7. 目标架构与所有权

### 7.1 Owner 分层

| Owner | 必须唯一拥有 | 不得拥有 |
|---|---|---|
| Runtime Interface | versioned neutral schema/codec：identity、requirement、BuildSet expectation、launch/Ready/focus/recent DTO | filesystem store、process spawn、engine discovery、Hub business policy |
| Hub `EngineInstallationRegistry` | stable installation identity、location、channel/version/components、health、repair/rollback status | project open approval、build process handle |
| Hub `BuildSetResolver` | `ProjectIdentity + requirement + platform/profile -> ResolvedBuildSet` 与 explanation | mutable UI selected state、implicit sibling override |
| Hub `OperationService` | durable operation ID、immutable input digest、phase graph、deadline/cancel/retry/compensation、terminal repair | Tauri view strings作为authority |
| Hub `EditorProcessSupervisor` | `Child`/OS handle、process-tree lease、stdout/stderr、heartbeat、creation identity、kill/reap、terminal outcome | recent/config write |
| Hub `LaunchAttemptService` | resolved request、spawn receipt、mailbox claim、Ready correlation、running/terminal transition | Editor activation internals |
| Hub `FocusService` | active session query、request issuer generation、bounded inbox、ack wait、cleanup/security | 创建或伪造 Editor-owned session lease |
| Hub `RecentProjectService` | persisted base revision、tombstone/mutation journal、CAS/reconcile、diagnostic disposition | 把 display path当唯一项目 identity |
| Hub `HubReadModel` | immutable versioned projection of operations/processes/repairs/degraded stores | 直接执行 filesystem/process side effects |
| Editor | project admission、activation transaction、native window/focus owner、first-present/terminal evidence | 选择 Hub engine installation 或替 Hub 声明 launch success |
| App | executable/runtime loading与 host ABI authentication | Hub engine resolver、recent registry 或 process supervision |

### 7.2 必须新增或收敛的核心记录

```text
ResolvedProjectLaunch
  operation_id
  project_identity { canonical_descriptor, project_guid, manifest_digest }
  requirement_assessment
  resolved_build_set { id, manifest_digest, platform, configuration, arch }
  editor_artifact { absolute_path, digest, signer/source }
  launch_profile + authorization
  request_digest + deadline

LaunchAttemptRecord
  Prepared -> Spawned -> HandshakePending -> Ready -> Running
           -> FailedBeforeSpawn
           -> ExitedBeforeReady
           -> TimedOutStillOwned
           -> CancelledAndReaped
           -> Exited / Crashed / DetachedByPolicy
           -> RunningAwaitingProjectionRepair

SupervisedProcessIdentity
  pid + creation_time/process_nonce + root_handle + process_tree_identity
  stdout/stderr artifact + byte counters + truncation counters
  last_heartbeat + terminal_code/signal + reap_receipt
```

Ready receipt 不应自行重新声明所有 launch facts；它应回传并签认 Hub issue 的 `operation_id/request_digest/expected_build_set/session`，Hub 再与自己冻结的 attempt 对比。PID 只是 correlation 的一部分。

## 8. 重构里程碑

### M0：修正产品真相与回归护栏

- 删除或改写“Hub-supervised child”这类强于事实的错误文案。
- 把 `Spawned`、`Ready`、`Running`、`Exited`、`ProjectionRepairRequired` 分成 typed state。
- 为 project-bound engine 与 global settings 漂移增加行为测试，先证明当前 bug。
- 为 timeout 后 Child 仍归 owner、malformed mailbox、persist-after-ready failure 建立 failing tests。

### M1：EngineInstallation 与 BuildSet resolver

- Hub 消费 Interface 的 `ProjectIdentity`、`engine_version_req` 和 compatibility assessment。
- stable engine/build identity 与 location 分离；不再 prune unresolved binding。
- build command从 resolved engine snapshot 生成，禁止晚取 global settings。
- build完成必须生成并验证 BuildSet artifact manifest，原子激活后才可供 launch。
- sibling executable 只能作为显式 development override，必须经过 compatibility 和用户可见 provenance。

### M2：Owned operation runtime 与 process supervisor

- worker、focus refresh、build、Editor process 都有 owner handle 与 shutdown protocol。
- Windows 使用 Job Object，Unix 使用 process group；实现 cooperative cancel、deadline escalation、kill tree、wait/reap。
- stdout/stderr流式读取，限制单行、队列、内存和总字节，滚动落盘并投影 truncation counter。
- queue 增加 payload byte budget、lane/resource key、priority、dedup/coalesce、fairness 和 queue-age metrics。

### M3：Launch attempt 与 mailbox hardening

- spawn前持久化 immutable `ResolvedProjectLaunch` 和 attempt generation。
- supervisor 同时等待 mailbox、child exit、cancel、deadline、Hub shutdown。
- mailbox 在 open 前检查 size/metadata，使用 claim rename，消费后删除；启动时按 owner/age scavenging。
- Ready 绑定 operation/request digest/BuildSet/project identity/process creation identity。
- timeout 保留 owner，并给出 Continue Wait/Cancel/Kill/Detach policy；不再自动释放责任。

### M4：Focus/session correctness

- lease 与 record 读取收敛为同一可验证 snapshot，防止 lease/record TOCTOU。
- request issuer 加 BootId/generation；ack wait 使用 monotonic local deadline。
- malformed/mismatch/expired request与ack进入 quarantine/cleanup，不得永久占满 inbox。
- namespace 必须由 Editor owner 创建并发布 capability；校验 root ownership、ACL 与 reparse/symlink policy。

### M5：Recent project durable authority

- 持久化 Hub last-seen shared revision/base 或 durable mutation journal。
- startup 使用真实 base 做三方合并，外部 tombstone 在无本地新 mutation 证明时获胜。
- identity 优先使用 ProjectGuid + canonical descriptor generation，不再以 lossy display path 唯一寻址。
- 把 load disposition、quarantine、repair path 投影到 Hub read model。
- 将 filesystem business store 从 Interface 迁回正确 owner，Interface 只保留 DTO/codec。

### M6：Product read model 与 destructive coordination

- UI 展示每个 build/launch/process 的 operation、phase、BuildSet、deadline、PID identity、last event、terminal/repair action。
- delete project、remove/update engine、Hub close 必须查询 active process/session consumer，并走 Save/Close/Cancel/Force/Detach 决策。
- persistence failure 不覆盖已经发生的外部 effect；显式投影 `RunningAwaitingProjectionRepair`。

### M7：资格与性能

- 先通过 correctness/fault/security gates，再建立 launch cold/warm latency、queue wait、log throughput、mailbox load、recent reconcile contention 基线。
- 只有同硬件、同项目、同 workload、同图形/工具链配置的可复现实验，才允许比较 Zircon 与 Unreal 的性能。
- “优于 Unreal”必须绑定具体指标、样本、置信区间、profiling artifact 和回归阈值，不能作为源码存在性结论。

## 9. 工程资格门

| Gate | 状态 | 验收要求 |
|---|---|---|
| G01 ProjectIdentity freeze | Fail | Hub launch冻结 canonical descriptor、GUID、manifest digest |
| G02 engine requirement assessment | Fail | `engine_version_req` 有真实 Hub consumer 与 explanation |
| G03 immutable resolved BuildSet | Fail | queue/attempt持有 resolved BuildSet，不晚取 selected/settings |
| G04 artifact manifest validation | Fail | exit 0 后校验 artifact/hash/ABI/platform/config/arch |
| G05 atomic BuildSet activation | Fail | staging验证后原子激活，失败保持 last-good |
| G06 stable engine identity | Fail | identity 与 location/source path 分离 |
| G07 owned Editor Child | Fail | Hub 生命周期内持有 Child/process handle |
| G08 process-tree ownership | Fail | Job Object/process group覆盖 descendants |
| G09 stream budgets | Fail | stdout/stderr有 line/item/byte/disk retention budgets |
| G10 cancel/deadline/escalation | Fail | cooperative cancel到kill tree有typed receipt |
| G11 guaranteed wait/reap | Fail | success/failure/timeout/shutdown均可证明 reap/explicit detach |
| G12 first-present publication | Pass | Ready 只在真实 first-present callback 后发布 |
| G13 spawned PID validation | Pass | Hub 拒绝不同 PID 的 Ready receipt |
| G14 request/BuildSet correlation | Fail | Ready绑定 operation、request digest、BuildSet、identity |
| G15 mailbox byte bound/claim | Fail | open前cap、claim、consume-remove、owner check |
| G16 mailbox scavenger | Fail | stale/malformed/foreign mailbox有可审计清理 |
| G17 exact focus ack | Pass | exact request且 disposition=Focused 才成功 |
| G18 focus namespace/bounds | Partial | 32项上限已存在；byte、malformed、owner namespace未闭合 |
| G19 monotonic focus deadline | Fail | wall-clock回拨不延长本地等待 |
| G20 recent bounded writer/CAS | Pass | Hub使用bounded lease、revision CAS与retry |
| G21 offline delete tombstone | Fail | Hub离线期间Editor删除不能在重启后复活 |
| G22 recent diagnostic disposition | Fail | corrupt/quarantine/repair进入read model |
| G23 bounded queue item admission | Pass | 第65项在clone前被拒绝 |
| G24 queue byte/lane/fairness | Fail | payload bytes、resource lane、priority/fairness均受控 |
| G25 owned worker shutdown | Fail | join handle、stop admission、drain/cancel、restart reconcile |
| G26 typed launch terminal state | Fail | Running/Exited/Crashed/RepairRequired不是自由字符串 |
| G27 destructive consumer coordination | Fail | delete/remove/update/close先协调 active Editor |
| G28 cross-process/fault/scale evidence | Fail | 双进程、kill、PID reuse、clock、ACL、restart、large log均有资格测试 |

资格门合计：**22 Fail / 1 Partial / 5 Pass**。Pass 只证明窄原语，不代表整个 launch workflow 已通过。

## 10. 必须补齐的测试矩阵

| 维度 | 必须覆盖 |
|---|---|
| engine/build | bound A vs settings B、missing binding不丢失、wrong ABI、missing artifact、hash mismatch、partial staging、activation rollback |
| process | exit-before-ready、timeout-then-ready、malformed mailbox、cancel、kill tree、Hub close、PID reuse、orphan/zombie、persist-after-ready failure |
| Ready | wrong operation、wrong BuildSet、wrong digest、wrong instance/generation、duplicate/replay、stale mailbox、oversized mailbox |
| focus | denied foreground、clock rollback/forward、malformed poison、inbox full、Hub restart sequence、session changes while waiting、ACL/reparse point |
| recent | Hub offline Editor delete、双方离线交错新增/删除、revision conflict、lease timeout、corrupt quarantine、power-loss points、GUID/path move |
| queue/shutdown | 64项边界、大 payload、lane starvation、duplicate action、shutdown drain/cancel、panic后继续、restart reconcile |
| performance | 10 MiB/1 GiB logs、100/1,000 projects、32 focus files、concurrent recent writers、cold/warm Editor launch；记录 p50/p95/p99 与资源峰值 |

现有 inline behavior tests 应保留；40个 integration test 文件仍需逐步从 source-shape assertion 迁移到 production API、child process 和 fault harness。ignored benchmark 只有实际受管执行、原始样本和环境记录后才能成为性能证据。

## 11. 输出状态

- review：完成。
- production implementation：未开始。
- canonical finding：不新增；重判 20 项聚焦旧 finding。
- 全局 Hub ledger：**317 Open / 5 Partial / 1 Closed**，总数仍为 323。
- Cargo / dynamic product validation：未运行，不能宣称 build、tests 或 runtime 通过。
- 首要实现顺序：`M0 truth/tests -> M1 resolver/BuildSet -> M2 supervisor -> M3 launch/Ready -> M4 focus -> M5 recent -> M6 product projection -> M7 qualification`。
