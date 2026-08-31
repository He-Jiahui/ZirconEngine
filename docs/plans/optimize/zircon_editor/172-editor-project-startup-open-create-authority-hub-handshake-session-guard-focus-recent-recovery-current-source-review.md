---
title: Editor Project Startup、Open/Create、Authority、Hub Handshake、Session Guard、Focus、Recent 与 Recovery 当前源码复核
category: zircon_editor
report_id: Editor172
review_date: 2026-08-27
baseline_head: ea35974cdf64068f6789010451d20bbf69e0a29d
production_baseline: 982baa1ba87bc8c25fe44312507a4af15027e058
canonical_owner: Editor51
refreshes:
  - docs/plans/optimize/zircon_editor/51-editor-project-startup-open-create-authority-hub-handshake-session-guard-focus-recent-recovery-product-integration-review.md
  - docs/plans/optimize/zircon_editor/124-editor-project-startup-open-create-authority-hub-handshake-session-guard-focus-recent-recovery-current-source-review.md
related_code:
  - zircon_editor/src/core/gui_startup_request.rs
  - zircon_editor/src/core/project
  - zircon_editor/src/core/hub_link
  - zircon_editor/src/core/recovery
  - zircon_editor/src/ui/host/editor_host_startup.rs
  - zircon_editor/src/ui/host/editor_manager.rs
  - zircon_editor/src/ui/host/editor_manager_project.rs
  - zircon_editor/src/ui/host/editor_manager_project_session.rs
  - zircon_editor/src/ui/host/editor_manager_project_activation_effects.rs
  - zircon_editor/src/ui/host/editor_manager_startup.rs
  - zircon_editor/src/ui/host/project_access.rs
  - zircon_editor/src/ui/host/startup
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/host_lifecycle/startup
  - zircon_editor/src/ui/retained_host/app/welcome_session/actions
  - zircon_editor/src/ui/retained_host/host_contract/window
  - zircon_editor/src/ui/retained_host/run_config.rs
  - zircon_editor/src/ui/workbench/project
  - zircon_editor/src/ui/workbench/startup
  - zircon_app/src/entry/cli/launch_args.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_app/src/entry/entry_runner/editor
  - zircon_app/src/entry/entry_runner/bootstrap.rs
  - zircon_app/src/entry/first_party_editor_plugins.rs
  - zircon_app/src/entry/first_party_runtime_plugins.rs
  - zircon_hub/src/process/editor_focus
  - zircon_hub/src/process/editor_handshake
  - zircon_hub/src/process/editor_launch.rs
  - zircon_hub/src/projects/recent_project.rs
  - zircon_hub/src/projects/shared_recent_projects.rs
  - zircon_hub/src/tauri_app/runtime_state/editor_launch_actions.rs
  - zircon_runtime_interface/src/hub_protocol
  - zircon_runtime_interface/src/project/session_lock
  - zircon_runtime_interface/src/project/manifest_summary
  - zircon_runtime_interface/src/project/project_identity.rs
  - zircon_runtime_interface/src/project/canonical_descriptor_identity.rs
  - zircon_runtime_interface/src/project/project_launch_intent.rs
  - zircon_runtime_interface/src/project/engine_compatibility
  - zircon_runtime/src/asset/project/manifest
  - zircon_runtime/src/asset/project/manager/open.rs
  - zircon_runtime/src/asset/project/manager/durable_transaction.rs
  - zircon_runtime/src/asset/project/paths.rs
tests:
  - zircon_editor/src/core/project/tests
  - zircon_editor/src/core/recovery/tests.rs
  - zircon_editor/src/tests/host/manager/bootstrap_and_startup/session_startup.rs
  - zircon_editor/src/tests/host/retained_callback_dispatch/welcome/open_recent.rs
  - zircon_app/src/entry/entry_runner/editor/tests
  - zircon_hub/src/process/editor_handshake/tests.rs
  - zircon_runtime_interface/src/hub_protocol/tests.rs
  - zircon_runtime_interface/src/project/session_lock/tests.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/01-cross-report-owner-schema-abi-p0-consolidation-review.md
  - docs/plans/optimize/zircon_editor/02-document-transaction-save-autosave-recovery-review.md
  - docs/plans/optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md
  - docs/plans/optimize/zircon_editor/07-play-session-process-pie-game-view-live-edit-recovery-review.md
  - docs/plans/optimize/zircon_editor/50-editor-extension-contribution-store-registry-toolkit-provider-snapshot-reload-lifecycle-product-integration-review.md
  - docs/plans/optimize/zircon_app/07-renderable-empty-project-template-create-import-render-export-evidence-product-integration-review.md
  - docs/plans/optimize/zircon_hub/01-project-engine-build-editor-launch-process-persistence-delivery-review.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Editor/GameProjectGeneration/Private/GameProjectUtils.cpp
  - dev/UnrealEngine/Engine/Source/Editor/GameProjectGeneration/Private/SProjectBrowser.cpp
  - dev/UnrealEngine/Engine/Source/Editor/UnrealEd/Private/UnrealEdMisc.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Projects/Private/ProjectDescriptor.cpp
  - dev/UnrealEngine/Engine/Source/Runtime/Launch/Private/LaunchEngineLoop.cpp
  - dev/godot/editor/project_manager/project_manager.cpp
  - dev/godot/editor/project_manager/project_list.cpp
  - dev/godot/main/main.cpp
  - dev/Fyrox/project-manager/src/manager.rs
  - dev/Fyrox/project-manager/src/settings.rs
  - dev/Fyrox/project-manager/src/project.rs
  - dev/bevy/crates/bevy_app/src/app.rs
  - dev/bevy/crates/bevy_app/src/schedule_runner.rs
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 172 · Editor 项目启动、打开/创建、Authority、Hub、Session、Focus、Recent 与 Recovery 工程化复核

## 1. 最终结论

当前项目启动链已经越过“临时拼接即可运行”的早期阶段，但仍没有达到工程级、可证明的跨进程项目生命周期。可以保留的底座包括：versioned `ProjectLaunchIntent` 与 operation id；data-only preflight；由 canonical descriptor、project GUID 和 manifest digest 组成的 `ProjectIdentity`；directional engine compatibility；Normal/Safe/Recovery composition；OS ownership lease 与持久 `session.lock` 分离；`Claimed -> PreflightApproved -> Activating -> Ready -> Closing/RecoveryRequired` 生命周期；逐 effect activation ledger；first-present 后 Hub Ready；generation-qualified focus request/owner ack；以及有界、revisioned、CAS、corruption-tolerant 的 recent projection。

主事务仍在五个 P0 上断裂。`editor_manager_project_session.rs` 先持久化 `guard.commit_ready()`，再提交 ledger 的 `Session` effect；guard、ledger、runtime、plugin、document、window、first-present 和 Hub mailbox 没有共同 activation receipt。Normal profile仍直接批准manifest-derived scripts/native extensions/scene restore，没有trust decision。operation id进入intent/session/ledger，却没有durable dedup/replay authority。`RetainedEditorHost::new`成功后的插件、模板、scene、layout、focus和退出检查任一步提前返回，都不会经过显式project close；`Drop`只停止autosave并释放本地hierarchy watcher。

本轮刷新不重复登记finding，继续由Editor51拥有5个P0、60个P1和15个P2。当前裁决为：P0 **5 Open / 0 Partial / 0 Closed**；P1 **39 Open / 21 Partial / 0 Closed**；P2 **15 Open**；40个资格门为 **18 Fail / 18 Partial / 4 Pass**。新增Pass只覆盖Safe/Recovery的pre-materialization减权、data-only preflight，以及recent projection的corruption隔离和monotonic revision/CAS；它们不代表整条启动事务可发布。

## 2. 审查边界与 currentness

### 2.1 Owner 与去重

1. Editor172只刷新Editor51/124，不新增canonical编号；document/autosave归Editor02，Play归Editor07，extension/plugin lifecycle归Editor06/50，模板创建事务归App07，Hub child supervision归Hub01。
2. 本报告拥有从launch intent、preflight、admission、activation到Ready/first-present/focus/close/recovery的跨模块合同，以及recent作为post-commit projection的边界。
3. Runtime manifest与ProjectManager拥有descriptor解析和runtime attach/detach；本报告只裁决Editor产品何时允许materialize、如何提交和如何恢复。
4. Tooling按用户要求排除；本轮没有查询、轮询、等待或实时跟踪协调器状态。

### 2.2 冻结点

| 项目 | 当前值 |
|---|---|
| 当前磁盘冻结时间 | `2026-08-27T16:13:40.7378301+08:00` |
| Git HEAD | `ea35974cdf64068f6789010451d20bbf69e0a29d` |
| production baseline | `982baa1ba87bc8c25fe44312507a4af15027e058` |
| working tree | 冻结时`git status --short --untracked-files=all`为9,810条；结论绑定下列fingerprint，不假装等同干净HEAD |
| 动态证据 | 未运行Cargo、Editor、双进程、fault injection、crash-point、真实窗口、network filesystem、scale、soak或benchmark lane |

### 2.3 可复算 selected set

| 范围 | 文件 / 行 / 非空行 / bytes / tests / ignored | Fingerprint |
|---|---:|---|
| Zircon startup/project/Hub/session/focus/recent/tests | **269 / 37,830 / 34,452 / 1,366,805 / 381 / 0** | `656f8084d6c46c04583bd9e10d1353cd47212949eb653fb72f84675cb5dda53a` |
| Unreal/Godot/Fyrox/Bevy reference | **13 / 29,487 / 25,337 / 1,065,419 / 19 / 0** | `dd7527a76114bbb91a51ecd3e03ae5b37043ec5d488ffdf9b2b00058eff611ac` |
| 全部选择集 | **282 / 67,317 / 59,789 / 2,432,224 / 400 / 0** | `77cfc2578f3f5daf5403fbe90eeae44175a082420bbd04e9de6d733cc54666c1` |

Fingerprint使用workspace-relative小写`/`路径和逐文件SHA-256组成`path + NUL + hash + LF`清单，再对清单取SHA-256。Zircon集合递归展开frontmatter中的related/test roots并按物理路径去重；reference集合为13个明确文件。共享工作树继续变化时必须重新计算，不能沿用本冻结值。

## 3. 当前源码事实

### 3.1 Intent与preflight已经形成真实边界，但不是幂等事务

App、CLI、Hub、Welcome和Recent都能形成或传递`ProjectLaunchIntent`。intent包含schema、operation id、source、profile和open/create target；CLI还拒绝Hub session搭配legacy project参数。App完成模板创建后把同一operation retarget到新root，再交给Editor admission，而不会把临时`ProjectManager`跨边界传入Editor。

`ProjectAuthority`只做canonical root、bounded manifest读取、summary、migration assessment、composition和digest。`ProjectIdentity`现已实际组合canonical descriptor、project GUID与manifest digest；current manifest缺GUID会拒绝激活。existing-project路径在取得writer lease后重新preflight并比较manifest digest，确认未变化才构造`ProjectManager`。这修正了Editor51最早的pre-admission materialization问题。

仍缺durable operation journal。`execute_project_launch_intent()`每次直接重新preflight和admit；`retarget_open_existing_project()`允许同一operation改变target；相同operation/payload不能返回原terminal receipt，不同payload也没有deterministic conflict。create与open没有父子receipt，transport request/attempt也没有独立identity。

### 3.2 Compatibility、migration和Safe/Recovery是局部策略，不是完整trust authority

engine compatibility已有`Compatible`、`ProjectRequiresNewerEngine`、`ProjectRequiresOlderEngine`和`Incompatible`，Editor会在激活前拒绝不兼容版本。legacy manifest产生`OpenCopy/ConvertInPlace/Cancel`静态计划并阻断激活，直到owner执行动作并重新preflight。相比旧报告，这些不应再写成完全缺失。

策略仍只检查semantic version requirement；provider、feature、toolchain、plugin catalog和BuildSet compatibility没有进入同一receipt。migration只有可选动作类型，没有operator choice、backup/copy artifact、converter identity或replayable result。

Safe/Recovery plan在materialize前清空project plugin/script输入并禁止scripts、native extensions和scene restore，且激活只消费composition批准后的plugin/native字段。Normal plan则原样复制manifest-derived plugin/script并把三项能力设为true；没有签名、trust principal、approval version、revocation或capability lattice。因此Safe/Recovery可以成为安全底座，但Normal仍不能作为工程级不可信项目入口。

### 3.3 Admission与activation有durable effect ledger，但Ready顺序错误

`SessionGuard`通过Windows named mutex或Unix `flock`持有OS lease，进程退出只释放lease而保留`session.lock`供显式恢复。record持有process、instance、source-derived principal、BuildSet、operation、checked epoch、lifecycle、heartbeat和Ready generation。activation ledger对Runtime、Diagnostics、ProjectPlugins、Document、UserInterface、Session和RecentProjection记录Prepared/Committed/RolledBack/RecoveryRequired，读取限制为4 KiB。

Runtime、diagnostics、project plugins、document和UI effect均走`prepare -> execute -> commit`；rollback失败会把active effects标为RecoveryRequired并保留guard。recent在Ready后作为best-effort projection，失败只记录deferred diagnostic，不回滚已激活project。这些都是可保留的正确方向。

致命窗口仍清楚存在：Session effect先`prepare`，随后`guard.commit_ready()`写入可聚焦generation，最后才`ledger.commit(Session)`。第二次写失败或两次写之间崩溃，会得到`session.lock=Ready`而ledger Session仍Prepared/RecoveryRequired。Hub probe只检查live OS lease和Ready record，所以会把未完成共同commit的会话当作focus target。

### 3.4 Recovery保留未知状态，但没有可重放的共同receipt

激活失败若runtime close或project registration cleanup不完整，会保留exclusive guard并写RecoveryRequired。Recovery profile只允许接管residual session，并要求对应operation的ledger为terminal；Incomplete或RecoveryRequired effect inventory会阻止takeover。这比旧的无条件release安全。

恢复检查仍只拼接session record和operation ledger。它不交叉验证`ProjectIdentity`、manifest/preflight digest、BuildSet、plugin catalog generation、document session、window/first-present或Hub Ready。ledger effect为固定enum，closure不能表达“函数返回错误但已产生部分副作用”，effect inventory也没有每项operator action。terminal ledger随后可被清理，正常close又删除session lock，没有bounded Closed/Failed terminal index。

### 3.5 Focus request/ack已是真实协议，错误健康与长期治理未闭合

Focus request包含request id、target instance、target session generation、sequence和deadline；request/ack各有4 KiB上限，Hub发布前清理过期request并限制32个pending。Editor按sequence原子rename claim，旧generation和过期request写typed rejection；retained bridge最多保留32个等待native focus的request。只有真实`Focused(true)`回调才发布`Focused` ack，Hub后台worker等待完全匹配的ack后才报告`FocusedExisting`。

project switch/close同步binding时会先retire旧ack bridge，使in-flight request得到`RejectedStale`，再启动新generation watcher。Welcome later-open也调用同一sync入口。这修正了旧报告中“入队即Focused”和“启动后open没有watcher”的主要问题。

未闭合点包括：sequence仅进程内AtomicU64，request/ack没有短期audit index；malformed/oversized/target mismatch在claimed文件上返回错误后只靠`eprintln!`，Hub可能等到deadline；目录ACL、owner principal和capability handshake缺失；ack/handshake没有统一retention/quarantine/replay owner；OS拒绝foreground只能超时，不能返回`ForegroundDenied`与taskbar fallback。binding retire与session close也不是同一durable receipt。

### 3.6 Hub Ready晚于first-present，但不能证明activation整体一致

`HubEditorReadyReceiptV1`要求SessionCommitted、NativeWindowCreated、FirstPresent、FocusInboxBound和Interactive milestone，retained host只在first-present callback中发布Ready。Hub会校验session token和child process id，GPU/window/startup失败可发布typed startup failure code。这已修正早期“窗口创建前Ready”。

receipt本身仅携process、editor instance、session generation和预置milestone set，不含operation、ProjectIdentity、manifest/BuildSet、ledger digest、plugin/document generation或actual present artifact。first-present之后的presenter/GPU loss没有Degraded/Revoked/Closed事件；handshake仍以250 ms轮询、10秒固定deadline，无phase stream、negotiation或cancel，并且read成功后没有统一ack/retention cleanup。

### 3.7 Recent registry已工程化为有界projection，但仍不是qualified identity

共享recent store限制8个project、64个tombstone和256 KiB编码；读取损坏/超限时返回empty rebuildable projection，下一次有界mutation会隔离原文件并原子重建。Windows/Unix writer lease支持try-now、timeout和cancellation；Hub reconciliation使用revision CAS、最多4次重试；record/remove推进checked revision和逻辑时间，删除tombstone防止stale Hub snapshot复活条目。Recent写发生在Ready之后，失败不会回滚project session。

P1-42和Gate 25/26的核心存储要求因此已有实质完成。仍不能把recent当ProjectIdentity：key由`to_string_lossy()`的display path规范化得到，entry混合summary、path和wall-derived初始时间；没有project GUID/digest identity、display alias/relocation、pagination/age policy，也没有把上次crash、trust change或BuildSet change纳入auto-open决策。

### 3.8 Close有真实veto与Closing门，但异常退出仍绕过

retained close会先阻断active dirty save、queued Save All、model import、Play teardown和pending Play edit decision；manager随后持久化`Closing`、关闭runtime、清document journal/settings/log/plugin registrations、发布document close，最后释放guard。focus binding在manager close后同步退休。正常应用退出还要求runtime shutdown、最终autosave、settings flush、event-loop/fatal/capture检查全部成功后才调用`commit_project_close()`。

问题正来自这组正常路径条件。`RetainedEditorHost::new`成功后，editor plugin registration、template sync、startup scene、layout、native focus callback、focus binding和first-present callback都使用`?`提前返回；event loop之后settings/autosave/fatal/capture任一失败也会在close前返回。`RetainedEditorHost::Drop`只调用autosave `begin_shutdown()`并丢弃hierarchy watcher；`SessionGuard`没有业务Drop，字段析构只释放OS lease，留下当时状态的record。这样保留了恢复证据，但没有写Closing/RecoveryRequired、逆序关闭effect或生成shutdown receipt。

## 4. 本地参考源码对照

| 参考 | 可验证事实 | Zircon应吸收 | 不应误抄 |
|---|---|---|---|
| Unreal `SProjectBrowser` / `GameProjectUtils` | Open前拒绝newer engine，检查code/compiler/plugin status，并提供Open Copy、Convert In-place、Skip或Cancel。 | data-only compatibility/provider matrix、可恢复migration choice与artifact receipt。 | 不复制Slate、阻塞dialog循环或大型继承结构。 |
| Unreal `UnrealEdMisc` | project switch采用close + restart，并先触发save/cancel流程。 | 把restart作为无法证明热切换完整退休时的正式策略。 | 不要求所有项目切换永久依赖进程重启。 |
| Unreal `LaunchEngineLoop` | recent auto-load写`.InProgress`标记；上次未达到Editor init会禁用下一次auto-load。 | auto-open必须消费上次startup terminal evidence和risk reason。 | 单个sentinel不能替代Zircon的operation/session receipt。 |
| Godot Project Manager / Main | 在open前处理config version、unsupported feature、backup/conversion；recovery mode禁用tool script/editor plugin/GDExtension等高风险能力。 | 在composition前完成版本、feature和Safe/Recovery减权。 | 不把一个recovery bool当完整trust/capability系统。 |
| Fyrox Project Manager | 持有child并用`try_wait`推进build queue。 | Hub应拥有child terminal state和后台监督。 | settings/recent与串行UI实现不是可靠事务上限。 |
| Bevy App / ScheduleRunner | plugin有Adding/Ready/Finished/Cleaned phase，runner显式finish和cleanup。 | project composition与shutdown phase必须显式、可观察并有terminal disposition。 | Bevy没有Editor Project Manager，不能替代admission/Hub/recovery设计。 |
| Unity Graphics | 本地`dev/Graphics`没有Unity Editor/Hub项目启动源码。 | 记录0 applicable，等待可验证源码。 | 不根据闭源产品表现猜测内部lock/handshake。 |

## 5. Editor51 finding重判

### 5.1 汇总

| 级别 | Open | Partial | Closed | 合计 |
|---|---:|---:|---:|---:|
| P0 | 5 | 0 | 0 | 5 |
| P1 | 39 | 21 | 0 | 60 |
| P2 | 15 | 0 | 0 | 15 |

### 5.2 P0

| Finding | 状态 | 当前证据与退出条件 |
|---|---|---|
| **P0-01** Ready早于ledger Session commit | Open | `guard.commit_ready()`仍在`ledger.commit(Session)`之前。必须以ledger/receipt digest完成单一commit fence，并做每个写入间隙的crash injection。 |
| **P0-02** 没有共同durable activation receipt | Open | guard、ledger、runtime、plugin、document、window、first-present和Hub仍为独立authority。必须建立绑定operation、ProjectIdentity、BuildSet、effect digest和generation的不可变receipt。 |
| **P0-03** Normal profile没有trust/authorization boundary | Open | Normal直接批准manifest scripts/plugins/native/scene restore；无signature、principal approval、revocation或policy digest。未信任项目必须只能进入data preview/Safe。 |
| **P0-04** operation id没有durable dedup/replay authority | Open | execute入口无journal lookup，重复请求会重新执行或撞上ledger create。必须持久化Pending/Committed/Failed/RecoveryRequired并校验payload digest。 |
| **P0-05** host post-construction early return无显式close | Open | 多个`?`和退出检查位于host构造后、`commit_project_close()`前；Drop不关闭project。所有出口必须经过bounded shutdown coordinator，失败写RecoveryRequired receipt。 |

### 5.3 P1：Intent、Identity、Preflight、Trust、Admission（01-15）

| Finding | 状态 | 当前证据与退出条件 |
|---|---|---|
| **P1-01** operation与transport request/attempt未分离 | Open | intent只有operation id；新增request id、attempt、deadline和retry lineage。 |
| **P1-02** source principal不具认证与审计 | Partial | source已映射到session provenance principal，但类型明确声明不是authentication claim；补authenticated principal、permission和audit correlation。 |
| **P1-03** retarget保留operation却能改变target | Open | create后retarget复用operation；必须冻结payload digest或生成child operation。 |
| **P1-04** qualified `ProjectIdentity`未贯穿全链 | Partial | canonical descriptor + GUID + manifest digest类型已进入preflight；session/focus/recent/document仍主要使用root/string/generation。 |
| **P1-05** create与open没有父子receipt | Open | App创建后复用同一operation retarget；需要create artifact receipt和activation child receipt。 |
| **P1-06** compatibility缺provider/feature矩阵 | Partial | directional semver disposition已实现；required provider、feature、toolchain和BuildSet decision缺失。 |
| **P1-07** migration缺执行与artifact receipt | Partial | OpenCopy/ConvertInPlace/Cancel计划会阻断激活；operator choice、backup digest、converter和重放缺失。 |
| **P1-08** composition只有粗粒度profile | Partial | Safe/Recovery真实减权；没有script/native/network/write等capability lattice。 |
| **P1-09** trust store与revocation不存在 | Open | 无TrustDecision/TrustReceipt、key rotation、offline policy或operator override。 |
| **P1-10** revalidation只绑定manifest digest | Partial | lease内会重读manifest；BuildSet、plugin catalog、toolchain和policy digest不参与。 |
| **P1-11** preflight无TTL和stale reason | Open | 只有即时digest比较；增加monotonic expiry与invalidated-by。 |
| **P1-12** admission角色/access mode不完整 | Partial | principal provenance和BuildSet存在；没有read-only/headless/migration等组合规则。 |
| **P1-13** checked epoch不是跨存储fence | Open | effect commit不携同一fencing token。 |
| **P1-14** network/removable filesystem无支持矩阵 | Open | local mutex/flock存在，但未检测远程锁/rename/fsync语义。 |
| **P1-15** process instance抗PID复用不足 | Open | instance仍为PID+wall millis+进程内sequence；缺boot/OS creation token和随机nonce。 |

### 5.4 P1：Activation、Commit、Rollback、Close、Recovery（16-30）

| Finding | 状态 | 当前证据与退出条件 |
|---|---|---|
| **P1-16** ledger与session record无跨文件commit protocol | Open | 两份atomic file仍有Ready split-brain窗口。 |
| **P1-17** effect registry固定且unknown effect无演进策略 | Partial | seven-effect versioned ledger与strict decode存在；缺扩展schema和unknown-effect quarantine。 |
| **P1-18** effect closure不能报告partial side effects | Open | `FnOnce() -> Result<T, EditorError>`没有resource/effect receipt。 |
| **P1-19** plugin mount不绑定session/catalog generation | Open | ledger只有ProjectPlugins枚举，没有mount set digest和owner generation。 |
| **P1-20** document journal与message publish不原子 | Open | begin session后立即publish messages，缺generation-stamped durable commit。 |
| **P1-21** close/rollback缺每effect terminal disposition | Partial | ledger能分类activation effect，close有逆序清理；runtime/plugin/document/focus没有共同close receipt。 |
| **P1-22** guard不是coordinator-owned session lease | Open | manager保存`Option<SessionGuard>`，没有包含effect handles与close receipt的统一lease。 |
| **P1-23** Drop没有unresolved shutdown诊断通道 | Open | Drop只begin autosave shutdown和take watcher。 |
| **P1-24** RecoveryRequired缺operator action | Partial | recovery能列effect inventory；没有per-effect retry/rollback/restore/manual action。 |
| **P1-25** takeover缺ready/identity/digest交叉校验 | Partial | terminal ledger是强制条件；ProjectIdentity、manifest、BuildSet和Ready digest未共同验证。 |
| **P1-26** heartbeat只有wall-clock时间 | Open | 无monotonic sequence、elapsed或clock-skew policy。 |
| **P1-27** takeover超时与权限policy分散 | Open | profile分支存在，但无central lease policy和operator audit。 |
| **P1-28** Closing拒绝语义不完整 | Partial | lifecycle与Hub probe能阻止focus；没有typed Closing/RetryAfter ack。 |
| **P1-29** release删除证据且无Closed receipt | Open | session.lock正常关闭后被删除。 |
| **P1-30** terminal ledger cleanup仍无maintenance owner | Partial | cleanup失败已降为deferred diagnostic且不回滚Ready；没有bounded retry/index owner。 |

### 5.5 P1：Hub、Focus、Ready、Recent（31-45）

| Finding | 状态 | 当前证据与退出条件 |
|---|---|---|
| **P1-31** Ready receipt缺activation/manifest digest | Open | receipt只有process/instance/generation/milestones。 |
| **P1-32** first-present后无Degraded/Revoked/Closed | Open | callback只发布一次Ready。 |
| **P1-33** focus callback错误只写stderr | Open | consume/ack失败`eprintln!`，Hub可等到deadline；需owner health和terminal ack。 |
| **P1-34** focus disposition无短期audit record | Partial | typed request/ack/rejection已实现；完成后文件被删除且无queryable history。 |
| **P1-35** mailbox无ACL/capability handshake | Open | 仍依赖project-local filesystem目录。 |
| **P1-36** handshake无phase/deadline/cancel schema | Open | 固定250 ms poll与10秒timeout。 |
| **P1-37** public failure与详细diagnostic未分离 | Open | 多处`error.to_string()`和绝对path进入Hub/Editor消息。 |
| **P1-38** handshake cleanup/retention/replay owner缺失 | Open | token校验存在，terminal mailbox没有统一ack/expiry/quarantine回收。 |
| **P1-39** Queued与Focused边界仍非完整状态类型 | Partial | Hub必须等待owner `Focused` ack才报成功；没有Delivered/ForegroundDenied/Unavailable公共outcome。 |
| **P1-40** foreground denied无协议化fallback | Open | native focus事件不来时只能timeout。 |
| **P1-41** watcher rebind不在session receipt内 | Partial | retire-old/stale-ack/start-new顺序存在；与close/switch generation commit仍非原子。 |
| **P1-42** recent storage缺完整长期journal策略 | Partial | bounded lease、revision/CAS、logical clock、tombstone、quarantine和atomic publish已实现；无增量journal/age maintenance和qualified identity。 |
| **P1-43** recent identity仍是lossy display path key | Open | `to_string_lossy()`规范化参与去重；需ProjectIdentity与独立display alias/relocation。 |
| **P1-44** auto-open未消费crash/trust/BuildSet risk | Open | recent只是projection，没有risk policy。 |
| **P1-45** single/multi-project/window topology未定义 | Open | 当前行为由one-guard/one-binding偶然决定。 |

### 5.6 P1：产品入口、测试与性能（46-60）

| Finding | 状态 | 当前证据与退出条件 |
|---|---|---|
| **P1-46** 入口缺统一pending/terminal UI model | Partial | 用户入口已形成intent；没有operation timeline或共享command receipt。 |
| **P1-47** CLI/Hub/Welcome错误、cancel、retry不统一 | Open | 各层仍拼自由字符串和独立timeout。 |
| **P1-48** startup无per-phase timing | Open | manifest/lease/plugin/document/present没有统一trace。 |
| **P1-49** switch/close veto未形成共同receipt | Partial | dirty save、Save All、model import和Play teardown有真实阻断；asset/extension/document仍无共同drain receipt。 |
| **P1-50** 多项测试仍是源码shape断言 | Open | `.find/.contains/include_str!`不能证明写入与并发语义。 |
| **P1-51** 无Ready-before-ledger crash regression | Open | 当前测试反而固定断言该危险顺序。 |
| **P1-52** 无host early-return guard leak regression | Open | plugin/template/scene/layout/focus/finalization均无fault matrix。 |
| **P1-53** 无duplicate operation并发测试 | Open | 因为dedup authority尚不存在。 |
| **P1-54** trust/profile执行隔离覆盖不足 | Partial | Safe/Recovery composition与revalidation有focused tests；无真实DLL/script materializer隔离E2E。 |
| **P1-55** 无跨存储split-brain recovery矩阵 | Open | 只按session operation读取单ledger。 |
| **P1-56** 无真实Hub+Editor双进程E2E | Open | 当前focus/handshake多为单元或source-shape。 |
| **P1-57** 无heartbeat pause/PID reuse/clock rollback测试 | Open | liveness没有controlled clock/OS process harness。 |
| **P1-58** bounded parser覆盖不完整 | Partial | recent、ledger、focus request/ack有byte gate；lock/handshake统一quarantine和depth/retention矩阵缺失。 |
| **P1-59** 无network/removable lock qualification | Open | 没有平台存储实验或fail-close策略。 |
| **P1-60** 无startup/focus/close统计性能基线 | Open | 没有固定大项目p50/p95/p99、CPU/RSS/I/O与失败成本。 |

### 5.7 P2

| Finding | 状态 | 当前差距 |
|---|---|---|
| **P2-01** | Open | Project Session Inspector不存在。 |
| **P2-02** | Open | compatibility/migration/trust decision history不存在。 |
| **P2-03** | Open | activation/first-present/focus/close distributed timeline不存在。 |
| **P2-04** | Open | signed project catalog与key rotation不存在。 |
| **P2-05** | Open | per-project capability trust policy不存在。 |
| **P2-06** | Open | launch provenance UI与retry lineage不存在。 |
| **P2-07** | Open | effect-aware Recovery Assistant不存在。 |
| **P2-08** | Open | bounded terminal receipt index不存在。 |
| **P2-09** | Open | focus topology/foreground capability inspector不存在。 |
| **P2-10** | Open | recent workspace/tag/pin/relocation model不存在。 |
| **P2-11** | Open | multi-window/read-only viewer policy不存在。 |
| **P2-12** | Open | privacy-aware support bundle不存在。 |
| **P2-13** | Open | deterministic crash-point simulator不存在。 |
| **P2-14** | Open | startup performance budget和CI regression lane不存在。 |
| **P2-15** | Open | multi-BuildSet/legacy/revoked-signature compatibility lab不存在。 |

## 6. Canonical资格门

| Gate | 状态 | 当前裁决 |
|---|---|---|
| `PROJ-GATE-01` duplicate operation | Fail | 无durable dedup journal或terminal lookup。 |
| `PROJ-GATE-02` payload conflict | Fail | operation复用不校验target/policy digest。 |
| `PROJ-GATE-03` Ready commit fence | Fail | Ready仍早于ledger Session commit且不带digest。 |
| `PROJ-GATE-04` crash replay | Fail | 无覆盖所有effect的replay coordinator。 |
| `PROJ-GATE-05` Hub Ready after commit/present | Partial | Hub等待first-present和Ready record，但Ready record本身可能split-brain。 |
| `PROJ-GATE-06` Ready revocation | Fail | 无Degraded/Revoked/Closed milestone。 |
| `PROJ-GATE-07` focus outcome taxonomy | Partial | request/ack/rejectiontyped，缺Delivered/ForegroundDenied/UI状态。 |
| `PROJ-GATE-08` switch watcher retirement | Partial | old bridge先retire并拒绝stale；非durable原子receipt。 |
| `PROJ-GATE-09` Closing retry contract | Partial | Closing不可focus，但没有Busy/RetryAfter typed outcome。 |
| `PROJ-GATE-10` early-return shutdown receipt | Fail | host构造后多条路径绕过close。 |
| `PROJ-GATE-11` bounded Drop/quarantine | Partial | Drop不无限等待且残留record，但不显式写RecoveryRequired或诊断receipt。 |
| `PROJ-GATE-12` reverse dependency close | Partial | normal close有明确顺序，缺全部effect terminal receipt和异常出口。 |
| `PROJ-GATE-13` Normal trust approval | Fail | 无signature/trust/approval。 |
| `PROJ-GATE-14` Safe/Recovery pre-materialization block | Pass | composition在materialize前移除project scripts/plugins/native/scene restore，activation只消费approved plan。 |
| `PROJ-GATE-15` approval digest invalidation | Partial | manifest变化会拒绝；BuildSet/trust/catalog/toolchain变化不会共同失效。 |
| `PROJ-GATE-16` data-only preflight | Pass | preflight不构造ProjectManager、不加载project code、不取得writer或写最终project状态。 |
| `PROJ-GATE-17` migration decision receipt | Partial | typed action plan与activation block存在；选择、artifact与重放receipt缺失。 |
| `PROJ-GATE-18` unsupported filesystem fail-close | Fail | 无network/removable语义检测。 |
| `PROJ-GATE-19` PID/clock reuse safety | Fail | instance与heartbeat仍依赖PID/wall clock。 |
| `PROJ-GATE-20` residual takeover qualification | Partial | explicit Recovery + terminal ledger存在；identity/BuildSet/operator permission交叉校验不足。 |
| `PROJ-GATE-21` bounded corrupt input | Partial | recent和ledger/focus局部有界；全协议统一quarantine/diagnostics未完成。 |
| `PROJ-GATE-22` public error redaction | Fail |多个公共错误含display path和原始`to_string()`。 |
| `PROJ-GATE-23` mailbox nonce/expiry/ack/retention | Partial | focus具request id、expiry、ack和消费删除；handshake/replay/retention未统一。 |
| `PROJ-GATE-24` mailbox ACL/principal | Fail | 无目录ACL与principal capability handshake。 |
| `PROJ-GATE-25` recent corruption isolation | Pass | corrupt/oversize变empty projection并在有界mutation隔离重建，不回滚activation。 |
| `PROJ-GATE-26` recent monotonic merge | Pass | checked revision、logical clock、tombstone与CAS避免wall-clock rollback/stale overwrite。 |
| `PROJ-GATE-27` unified launch intent | Partial |产品入口普遍使用versioned intent；仍有manager内部direct open API和create retarget语义。 |
| `PROJ-GATE-28` retry/double-click integration | Fail |无真实concurrent duplicate/重启覆盖。 |
| `PROJ-GATE-29` split-brain deterministic action | Partial |ledger inventory和RecoveryRequired分类存在；跨manifest/plugin/document/Ready组合不完整。 |
| `PROJ-GATE-30` close veto/drain | Partial |dirty/Save All/import/Play有gate；缺跨asset/extension/document共同receipt。 |
| `PROJ-GATE-31` Windows two-process E2E | Fail |未执行也未发现完整产品harness。 |
| `PROJ-GATE-32` crash-point harness | Fail |不存在每个atomic replace/callback kill matrix。 |
| `PROJ-GATE-33` unknown-state guard fault proof | Fail |有局部unit/source assertions，没有所需fault injection。 |
| `PROJ-GATE-34` statistical performance evidence | Fail |无大型项目p50/p95/p99 CI artifact。 |
| `PROJ-GATE-35` bounded growth/wait | Partial |focus/recent/ledger有局部边界；manifest/plugin/startup整体无统一budget。 |
| `PROJ-GATE-36` phase cancellation/deadline | Fail |多数phase无cancel、deadline、retry-after。 |
| `PROJ-GATE-37` shared session generation | Partial |session/focus/Hub Ready一致；plugin/document/window未统一携带。 |
| `PROJ-GATE-38` shutdown diagnostics/support bundle | Partial |未知状态保留record/ledger；无脱敏support bundle和terminal close receipt。 |
| `PROJ-GATE-39` terminal receipt replay safety | Fail |terminal receipt store不存在。 |
| `PROJ-GATE-40` reproducible evidence | Partial |本轮重算282-file fingerprint并静态复核；未做独立动态review或qualification。 |

## 7. 目标架构与Hard Cutover

```text
ProjectLaunchIntent(OperationId, RequestId, Attempt, Principal, Deadline, PayloadDigest)
  -> LaunchJournal.compare_or_begin(OperationId, PayloadDigest)
  -> DataOnlyPreflight
       ProjectIdentity + Manifest/BuildSet/Policy/Trust/Provider digests
       Compatibility + Migration/Backup/Copy decision
       Approved capability plan
  -> AdmissionLease(ProjectIdentity, fence epoch, owner instance)
  -> ActivationCoordinator.prepare
       runtime -> diagnostics -> plugins -> document -> UI/window/focus inbox
       every effect returns lease + digest + compensation contract
  -> durable commit fence
       committed effect set + ActivationReceipt digest
       publish Ready(session generation, receipt digest)
  -> native first-present evidence
       HubReadyReceipt references the same ActivationReceipt

Close / switch / failure
  -> Closing(fence epoch), reject new open/focus
  -> dirty/play/asset/extension drain receipts
  -> reverse-order effect retirement
  -> Closed receipt, or RecoveryRequired with exact effect actions
  -> release OS ownership lease last
```

Hard cutover要求：旧的裸root/string focus target、无journal operation执行、Normal隐式trust、Ready-before-Session顺序和host early-return直接`?`必须删除，而不是与新路径长期并存。Recent、Welcome文本和Hub history只能投影terminal receipt，永远不能参与project commit权威。

## 8. 分层重构计划

### M0 · RED contracts与fault points

- 为五项P0建立真实fake filesystem/atomic writer/barrier，不再用源码shape证明调用顺序。
- 覆盖每个ledger/session write gap、host post-construction错误点、duplicate operation和rollback failure。
- 固定public error redaction、bytes/depth和filesystem support matrix。

### M1 · Launch journal、identity、trust与compatibility

- 引入bounded launch journal、payload conflict和terminal replay。
- 把ProjectIdentity、BuildSet、provider/toolchain、policy和trust digest编入preflight receipt。
- Normal必须获得显式approval；Safe/Recovery保留当前pre-materialization减权。

### M2 · Activation receipt与单一commit fence

- effect closure返回typed resource lease、digest和compensation disposition。
- ledger先形成完整Committed set，再以digest发布Ready generation。
- Hub、focus、document和plugin只接受同一receipt/session generation。

### M3 · Focus/handshake产品协议

- 保留当前request/sequence/deadline/native-focus ack底座。
- 增加ACL/capability handshake、audit retention、foreground denied、phase/cancel和Ready revocation。
- rebind/close与generation retirement进入同一receipt。

### M4 · Close、recovery与异常出口

- 用shutdown coordinator收敛所有`run_editor_with_config`出口。
- dirty/Play/asset/extension/document产生typed drain/veto receipt，逆依赖退休。
- Closed与RecoveryRequired进入bounded terminal index；Drop只报告未收敛状态。

### M5 · Recent与产品入口

- 保留bounded store、revision/CAS、tombstone和quarantine。
- recent key改用ProjectIdentity，display alias独立；auto-open消费crash/trust/BuildSet risk。
- Welcome/CLI/Hub统一显示operation timeline和localized terminal reason。

### M6 · Qualification与性能

- Windows真实Hub+Editor双进程覆盖launch/focus/close/recovery/PID reuse/foreground denied。
- network/removable filesystem做lock/rename/fsync支持矩阵，unsupported配置fail-close。
- 固定大型项目、BuildSet、plugin/manifest规模，记录startup/first-present/focus/close的p50/p95/p99、CPU、RSS、alloc、I/O和失败成本。

## 9. 本轮closeout与限制

| 项目 | 结果 |
|---|---|
| Canonical owner | Editor51，Editor172只刷新Editor51/124 |
| finding状态 | P0 5 Open；P1 39 Open / 21 Partial；P2 15 Open |
| gate状态 | 18 Fail / 18 Partial / 4 Pass |
| 生产修改 | 0；本轮只写review与索引 |
| 动态验证 | 未运行；所有状态均为current-disk静态证据裁决 |
| 性能宣称 | 无；没有同项目、同硬件、同质量、统计分布证据，不得声称优于Unreal |
| 后续实施前置 | 重新计算fingerprint和HEAD，先做M0 RED/fault harness，再做Hard Cutover |

