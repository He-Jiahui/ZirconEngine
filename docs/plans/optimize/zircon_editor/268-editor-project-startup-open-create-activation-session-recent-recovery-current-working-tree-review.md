---
title: Editor Project Startup、Open/Create、Activation、Session、Recent 与 Recovery 当前工作树复审
category: zircon_editor
report_id: Editor268
review_date: 2026-08-31
baseline_head: working-tree
observed_head: ca3ac3cc6ad218d04a5cd469447cea2452441321
canonical_owner: Editor51
refreshes:
  - docs/plans/optimize/zircon_editor/51-editor-project-startup-open-create-authority-hub-handshake-session-guard-focus-recent-recovery-product-integration-review.md
  - docs/plans/optimize/zircon_editor/124-editor-project-startup-open-create-authority-hub-handshake-session-guard-focus-recent-recovery-current-source-review.md
  - docs/plans/optimize/zircon_editor/172-editor-project-startup-open-create-authority-hub-handshake-session-guard-focus-recent-recovery-current-source-review.md
related_code:
  - zircon_editor/src/core/project
  - zircon_editor/src/core/recovery/activation_ledger
  - zircon_editor/src/core/recovery/session_guard
  - zircon_editor/src/core/recovery/project_recovery_assessment.rs
  - zircon_editor/src/core/hub_link
  - zircon_editor/src/ui/host/editor_manager_project_activation_effects.rs
  - zircon_editor/src/ui/host/editor_manager_project_session.rs
  - zircon_editor/src/ui/host/editor_manager_project.rs
  - zircon_editor/src/ui/host/editor_manager_startup.rs
  - zircon_editor/src/ui/host/project_access.rs
  - zircon_editor/src/ui/host/project_session_transition.rs
  - zircon_editor/src/ui/host/startup
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/project_close.rs
  - zircon_app/src/entry/entry_runner/editor.rs
  - zircon_app/src/entry/entry_runner/editor
  - zircon_runtime_interface/src/project
  - zircon_runtime_interface/src/hub_protocol
plan_sources:
  - docs/plans/optimize/zircon_editor/172-editor-project-startup-open-create-authority-hub-handshake-session-guard-focus-recent-recovery-current-source-review.md
  - docs/plans/optimize/zircon_editor/266-editor-filesystem-project-scene-autosave-journal-session-io-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/204-runtime-filesystem-resource-io-path-atomic-transaction-recovery-security-current-working-tree-review.md
  - docs/plans/optimize/zircon_runtime/206-runtime-asset-registry-project-catalog-index-persistence-rebuild-incremental-query-watch-generation-current-working-tree-review.md
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
doc_type: current_source_review
review_status: review_complete
implementation_status: pending
source_recheck_required: true
tooling_scope: excluded_by_user_request
coordination_tracking: skipped_by_user_request
---

# Editor Project Startup、Open/Create、Activation、Session、Recent 与 Recovery 当前工作树复审

## 1. 结论

当前项目启动链已经有一批可保留的工程底座：versioned `ProjectLaunchIntent` 和带 nonce 的 operation id；不物化 `ProjectManager` 的 data-only preflight；由 canonical descriptor、project GUID、manifest digest 组成的 `ProjectIdentity`；lease 内 manifest revalidation；Normal/Safe/Recovery composition；Windows named mutex / Unix `flock` 与持久 `session.lock` 分离；逐 effect activation ledger；first-present 后 Hub Ready；generation-qualified focus；以及有界、revisioned、CAS、tombstone、corruption-tolerant 的 recent store。

但它仍不是一个可以证明正确的工程级项目生命周期。五项 P0 在当前工作树全部保持 Open：

1. `editor_manager_project_session.rs` 仍先执行 `guard.commit_ready()`，再执行 `ledger.commit(ProjectActivationEffect::Session)`。两份独立文件之间存在 Ready 已公开、Session effect 未提交的崩溃窗口；源码 shape 测试甚至明确固定了 `session_prepared < ready < session_committed`。
2. session guard、activation ledger、runtime project/catalog、plugin mount、document session、window、first-present、focus inbox 和 Hub mailbox 没有共同的 durable activation receipt，也没有共同 digest / fencing token。
3. Normal composition 仍直接批准 manifest-derived scripts、project plugins、native extensions 和 scene restore；来源 principal 明确只是 provenance，不是认证或授权，系统没有 trust store、approval、revocation 或 policy digest。
4. operation id 进入 intent、session record 和 ledger 文件名，但入口没有 durable compare-or-begin、payload conflict、terminal lookup 或 replay；重复请求只会重新执行或撞上 `atomic_write_new`。
5. `RetainedEditorHost::new` 成功后，plugin registration、template sync、startup scene、layout、focus callback/binding、first-present wiring、settings/autosave/fatal/capture 任一步都可在 `commit_project_close()` 前提前返回；`Drop` 只调用 autosave `begin_shutdown()` 并丢弃 hierarchy watch。

因此 Editor51/172 的账目本轮重判仍为：P0 **5 Open / 0 Partial / 0 Closed**，P1 **39 Open / 21 Partial / 0 Closed**，P2 **15 Open / 0 Partial / 0 Closed**；40 个资格门为 **18 Fail / 18 Partial / 4 Pass**。Partial 只代表局部机制存在，不能作为里程碑通过。Editor266 继续拥有 filesystem/autosave/document journal I/O，Runtime204/206 继续拥有通用 I/O 和 asset registry/catalog；本报告只拥有项目启动到终止的跨模块事务边界。

## 2. 审查边界与 currentness

### 2.1 当前冻结清单

本轮冻结时间为 `2026-08-31T12:42:45.7746362+08:00`。工作树有并发修改，因此 `observed_head` 只记录 Git 提交基线，指纹才是本报告所见源码集合的 currentness 证据。

| 范围 | files | lines | non-empty | bytes | tests | ignored | fingerprint |
|---|---:|---:|---:|---:|---:|---:|---|
| Zircon startup/project/session/Hub/recent focused production set | **163** | **16,973** | **15,510** | **620,315** | **100 markers** | **0** | `b24c648ed9b5cb5144acf508356e18673d7c3240998678f57ddcfba25d1501f6` |
| Unreal/Godot/Fyrox/Bevy references | **13** | **29,487** | **25,337** | **1,065,419** | **19 markers** | **0** | `6f1e93f7269f3c0e148522a2c078e5ce8cef549231649cf80942ee32062f2f34` |

Zircon 集合递归包含 `core/project`、activation ledger、session guard、Hub link、host startup、App editor entry、runtime-interface project/hub protocol，并加入 retained host/close/recovery owner；排除独立 test 目录、`*_tests.rs`、`tests.rs` 以及由 Editor Scene 报告拥有的 scene document/load job。指纹使用 workspace-relative 小写 `/` 路径和逐文件 SHA-256 组成 `path + NUL + hash + LF` 清单后再取 SHA-256。

### 2.2 证据等级与限制

- **E3**：逐文件读取当前 intent、preflight、create/open、admission、activation、ledger、guard、Ready、focus、recent、close 与 recovery 生产链路。
- **E2**：读取相邻 source-shape/unit tests，并核对本地 Unreal、Godot、Fyrox、Bevy 源码中的对应机制。
- **E1**：本轮没有运行 Cargo、Editor、Hub 双进程、故障注入、跨文件系统、恢复或 UI automation；测试声明数量不等于动态通过。
- **E0**：没有同项目、同硬件、同质量下的启动/first-present/focus/close p50/p95/p99、CPU、RSS、allocation 与 I/O 数据，不得声称性能或可靠性优于 Unreal。

## 3. 当前实现事实

### 3.1 Intent、身份与 data-only preflight 是真实底座

`ProjectActivationOperationId` 绑定 origin instance、非零 sequence 和 UUID nonce；`ProjectLaunchIntent` 把 source/profile/target 带到 Editor。existing project 在 admission 前只 canonicalize root、bounded-read manifest、生成 summary/composition/migration/digest；`ProjectIdentity` 包含 canonical descriptor、project GUID 和 manifest digest。当前 manifest 缺 GUID 会 fail-close。

existing-project materialization 已经移动到 writer lease 之后：`activate_project_from_preflight` 在 lease 内重新 preflight，manifest digest 不同则拒绝，随后才调用 `open_resolved_project`。这是相对早期临时实现的实质改进，应保留。

边界仍不完整：preflight receipt 不包含 BuildSet compatibility、provider/toolchain/plugin catalog/trust/policy digest；session record 不保存 `ProjectIdentity`。revalidation 只比较 manifest digest，不能证明前次批准的运行环境、签名、provider 或权限仍然有效。

### 3.2 Create 不是与 activation 原子一致的一个事务

`ProjectAuthority::create_project` 使用 staging directory、empty-target backup、rename commit 和失败 rollback，并在 target 上重新打开 `ProjectManager`。局部目录事务方向正确，但 transaction id 只是 PID + 进程内 counter，staging 内容没有 journal/receipt、目录树 digest、fsync 证明或 restart recovery owner。

Editor create 路径先把项目发布到最终目录并物化 `ProjectManager`，再 preflight、claim session 和执行 activation。后续 activation 失败时，新建项目目录仍是已提交 artifact；同一 operation 没有 `CreateArtifactReceipt -> ActivationChildReceipt` 关系，用户重试也没有 terminal replay。创建成功与打开失败因此会表现为一个半成功操作。

### 3.3 Compatibility/Profile 不等于 Trust Authority

engine semantic-version compatibility 已有方向性 disposition；legacy manifest 会产生 migration decision 并阻断 activation；Safe/Recovery 在 materialization 前清空 project plugins/scripts，禁止 native extension 和 scene restore。这些局部策略不应写成“完全不存在”。

但 Normal profile 的 `compile` 分支直接 clone manifest plugins/scripts 并设置全部 allow flag。`ProjectSessionPrincipalV1` 的注释明确说明它只是本地来源 provenance。当前没有签名链、project trust identity、operator approval、capability lattice、revocation、key rotation、offline policy或 policy digest。因此“App 已认证 BuildSet”不能推出“项目代码已授权执行”。

### 3.4 Admission/Ledger 有持久状态，但 commit fence 错位

`SessionGuard` 持有 OS ownership lease；持久 record 包含 PID、instance、source principal、BuildSet、operation、lifecycle、checked epoch、Ready generation 和 wall-clock heartbeat。activation ledger 使用 schema v1 和严格 decode，限制 4 KiB，对 Runtime、Diagnostics、ProjectPlugins、Document、UserInterface、Session、RecentProjection 记录 Prepared/Committed/RolledBack/RecoveryRequired。

Runtime、diagnostics、plugin、document 和 UI 均走 `prepare -> execute -> commit`，失败时尝试逆向补偿；补偿不确定会把 active effects 标为 RecoveryRequired 并保留 guard。这一框架可保留。

核心错误发生在 terminal publication：ledger 先 prepare Session，随后 guard 原子写成 Ready 并分配 process-local generation，最后才提交 Session effect。session record 和 ledger 是两个独立 atomic file，checked epoch 也没有进入 effect write。Hub/focus 只检查 Ready record，就可能在 ledger 仍 Prepared 时接受一个 session。需要以单一 activation receipt digest 作为 commit fence，Ready 只能引用已 durable 的完整 committed effect set。

### 3.5 Recovery 保留未知状态，但不能重放完整事务

Recovery profile 只允许接管 residual record，并在持有 OS lease 时重新 assessment；selected residual 变化会拒绝，非 terminal ledger 会阻止 takeover。这比自动删除 stale lock 或无条件接管安全。

但 recovery assessment 只拼接一个 session record 与该 operation 的 ledger。它不交叉验证 ProjectIdentity、preflight digest、BuildSet、plugin mount set、document generation、Ready receipt 或 Hub mailbox，也没有每 effect 的 retry/rollback/restore action。正常 release 删除 `session.lock`，terminal ledger 又可能被立即清理，系统没有 bounded terminal index；恢复成功、关闭成功和清理失败都缺统一可查询收据。

### 3.6 Hub Ready 与 Focus 已有 generation，但不是完整协议

Hub Ready 只在 native window、first-present、focus inbox 和 interactive milestones 都成立后发布；receipt 持有 process、instance、session generation 和 milestone set。focus request/ack 已绑定 generation，active Ready session可接收 Hub focus signal。这是正确的跨进程起点。

但 Ready receipt 不含 operation、ProjectIdentity、BuildSet、activation digest 或 window generation；发布后没有 Degraded/Revoked/Closed。native focus acknowledgement 失败仍只 `eprintln!`。mailbox 缺 ACL/capability handshake、统一 phase/cancel/retry schema、foreground-denied outcome、retention/audit 和 public/private diagnostic separation。

### 3.7 Recent store 工程化程度较高，但身份降级

shared recent store 有 256 KiB read/write gate、writer lease、try-now/timeout/cancellation、revision CAS、logical clock、tombstone、corrupt/oversize quarantine 和 atomic replace。recent 是 Ready 后的 best-effort projection，写失败只产生 deferred diagnostic，不回滚已激活 session。该存储基础应保留。

问题在 projection model：authoritative entry 是 `PathBuf`，但 Editor `RecentProjectEntry` 和 validation 立即通过 `to_string_lossy()` 转为 `String`；key 仍围绕 display path，而非 `ProjectIdentity`。relocation、alias、同 GUID 冲突、manifest digest/BuildSet/trust change、上次 crash 与 auto-open 风险都没有成为 recent identity/policy。

### 3.8 Normal close 有真实 gate，异常出口仍绕过

正常 retained close 会阻断 active save、queued Save All、asset delete/relocate、model import、Play teardown 和 pending Play decision；manager 持久化 Closing，关闭 runtime，清 document/plugin/settings/log projection，发布 close，再 release guard。此路径比简单析构可靠。

但 `run_editor_with_config` 在 host 构造后到最终 `commit_project_close()` 之间存在大量 `?` 与显式 `return Err`。event loop 后 settings flush、autosave、fatal/capture/job 检查失败也会提前返回。`RetainedEditorHost::Drop` 不调用 manager close、guard RecoveryRequired 或 activation ledger settle。OS lease 虽会随字段析构释放并留下 residual record，但没有明确 shutdown phase、逆依赖 effect receipt、bounded drain 结果或 operator-facing terminal reason。

## 4. 本地参考源码对照

| 参考 | 可验证事实 | Zircon 应吸收 | 不应照搬 |
|---|---|---|---|
| Unreal `SProjectBrowser` / `GameProjectUtils` / `ProjectDescriptor` | 打开前处理 newer engine、code/compiler、plugin 与 conversion/open-copy 选择。 | 把 engine、provider、feature、toolchain、plugin、BuildSet、migration artifact 收敛为 data-only qualification receipt。 | 不复制 Slate、同步 modal loop 和大型继承体系。 |
| Unreal `UnrealEdMisc` | project switch 倾向 save/cancel 后 close + restart。 | 当热切换无法证明所有 generation 已退休时，把 restart 作为正式安全策略。 | 不把永久重启当作缺少生命周期设计的借口。 |
| Unreal `LaunchEngineLoop` | recent auto-load 用 InProgress 标记识别上次启动未完成并抑制再次 auto-load。 | auto-open 必须消费上次 terminal evidence、crash/trust/BuildSet 风险。 | 单 sentinel 不能替代 operation/session/effect receipt。 |
| Godot Project Manager / Main | open 前处理 config version、unsupported feature、backup/conversion；Recovery Mode 禁用 tool script、editor plugin、GDExtension 等。 | 保留 pre-materialization qualification，并把 Safe/Recovery 扩展为细粒度 capability policy。 | 单 recovery bool 不是 trust authority。 |
| Fyrox Project Manager | manager 持有 child，通过 `try_wait` 推进 build command queue 和 terminal state。 | Hub/App 必须拥有 child supervision、exit reason、timeout/cancel/reap。 | 其 UI settings/recent 模型不是 durable transaction 上限。 |
| Bevy `App` / `ScheduleRunner` | plugin 有 Adding/Ready/Finished/Cleaned，runner 显式 `finish()` 和 `cleanup()`。 | project composition、plugin mount 和 shutdown 应有显式 phase/terminal disposition。 | Bevy 没有 Editor Project Manager，不能替代 admission/recovery 设计。 |
| Unity Graphics | `dev/Graphics` 不包含 Unity Editor/Hub 项目启动实现。 | 本领域记录 **0 applicable**，等待可验证本地源码。 | 不根据闭源产品表象推测内部协议。 |

## 5. Finding 重判

### 5.1 汇总

| 级别 | Open | Partial | Closed | 合计 |
|---|---:|---:|---:|---:|
| P0 | 5 | 0 | 0 | 5 |
| P1 | 39 | 21 | 0 | 60 |
| P2 | 15 | 0 | 0 | 15 |

### 5.2 P0

| Finding | 状态 | 当前证据与退出条件 |
|---|---|---|
| **P0-01** Ready 早于 ledger Session commit | Open | `guard.commit_ready()`仍在`ledger.commit(Session)`之前；以已 durable 的完整 receipt digest 发布 Ready，并覆盖每个写入间隙 crash。 |
| **P0-02** 无共同 durable activation receipt | Open | guard/ledger/runtime/plugin/document/window/present/Hub 各自提交；建立绑定 operation、ProjectIdentity、BuildSet、effect set 与 generation 的不可变收据。 |
| **P0-03** Normal 无 trust/authorization boundary | Open | manifest scripts/plugins/native 默认批准，principal 只是 provenance；未批准项目只能 data preview/Safe。 |
| **P0-04** operation 无 durable dedup/replay authority | Open | 入口无 compare-or-begin/terminal lookup；建立 payload digest conflict、Pending/Committed/Failed/RecoveryRequired 和 terminal replay。 |
| **P0-05** host post-construction early return 无显式 close | Open | 多个错误出口位于 host 构造后、close 前，Drop 不关闭项目；所有出口必须汇入 bounded shutdown coordinator。 |

### 5.3 P1：Intent、Identity、Preflight、Trust、Admission（01-15）

| Finding | 状态 | 当前证据与退出条件 |
|---|---|---|
| **P1-01** operation 与 transport request/attempt 未分离 | Open | 增加 request id、attempt、deadline、retry lineage。 |
| **P1-02** source principal 不具认证与审计 | Partial | 已持久化 provenance principal；补 authenticated principal、permission、audit correlation。 |
| **P1-03** retarget 可在同 operation 下改变 target | Open | 冻结 payload digest，create/open 使用父子 operation。 |
| **P1-04** qualified `ProjectIdentity` 未贯穿全链 | Partial | preflight 已有强类型 identity；session/focus/recent/document 仍以 root/string/generation 为主。 |
| **P1-05** create 与 activation 无父子 receipt | Open | final directory 已发布后 activation 可失败；创建 artifact 与启动 child 必须分别可重放。 |
| **P1-06** compatibility 缺 provider/feature 矩阵 | Partial | semver disposition 已有；provider/toolchain/plugin/BuildSet decision 缺失。 |
| **P1-07** migration 缺执行与 artifact receipt | Partial | action plan 会阻断激活；choice、backup/copy digest、converter、replay 缺失。 |
| **P1-08** composition 只有粗粒度 profile | Partial | Safe/Recovery 可减权；缺 script/native/network/write 等 capability lattice。 |
| **P1-09** trust store 与 revocation 不存在 | Open | 无 TrustDecision/TrustReceipt、key rotation、offline policy、operator override。 |
| **P1-10** revalidation 只绑定 manifest digest | Partial | lease 内重读 manifest；BuildSet/trust/catalog/toolchain/policy 不参与。 |
| **P1-11** preflight 无 TTL/stale reason | Open | 只有 digest 不同；补 monotonic expiry 与 invalidated-by。 |
| **P1-12** admission role/access mode 不完整 | Partial | principal 与 BuildSet 已有；read-only/headless/migration 组合规则缺失。 |
| **P1-13** checked epoch 不是跨存储 fence | Open | effect commit、Ready、Hub receipt 未携共同 fencing token。 |
| **P1-14** network/removable filesystem 无支持矩阵 | Open | local mutex/flock 不证明远程 lock/rename/fsync 语义。 |
| **P1-15** process instance 抗 PID 复用不足 | Open | instance 为 PID + wall millis + process sequence；缺 boot/OS creation token 和随机 nonce。 |

### 5.4 P1：Activation、Commit、Rollback、Close、Recovery（16-30）

| Finding | 状态 | 当前证据与退出条件 |
|---|---|---|
| **P1-16** ledger 与 session record 无跨文件 commit protocol | Open | 两份 atomic file 仍有 split-brain。 |
| **P1-17** fixed effect registry 无 unknown evolution | Partial | seven-effect strict v1 ledger 已有；缺 extension schema 与 unknown quarantine。 |
| **P1-18** effect closure 不能报告 partial side effects | Open | `FnOnce() -> Result<T, EditorError>`不返回 resource/effect lease/receipt。 |
| **P1-19** plugin mount 不绑定 session/catalog generation | Open | 只有枚举状态，无 mount set digest 与 owner generation。 |
| **P1-20** document journal/message publish 不原子 | Open | begin session 后直接发布消息；缺 generation-stamped durable commit。 |
| **P1-21** close/rollback 缺每 effect terminal disposition | Partial | activation ledger 和 normal close 顺序存在；无共同 close receipt。 |
| **P1-22** guard 不是 coordinator-owned session lease | Open | manager 只保存 `Option<SessionGuard>`，不拥有完整 effect handles。 |
| **P1-23** Drop 无 unresolved shutdown diagnostic | Open | Drop 只处理 autosave 与 hierarchy watch。 |
| **P1-24** RecoveryRequired 缺 operator action | Partial | 可列 effect inventory；无 per-effect retry/rollback/restore/manual action。 |
| **P1-25** takeover 缺 identity/digest 交叉校验 | Partial | terminal ledger 为强制条件；ProjectIdentity/BuildSet/Ready digest 未共同验证。 |
| **P1-26** heartbeat 只有 wall-clock | Open | 无 monotonic sequence、elapsed 或 clock-skew policy。 |
| **P1-27** takeover timeout/permission policy 分散 | Open | profile branch 存在；无 central lease policy 和 operator audit。 |
| **P1-28** Closing ack 不完整 | Partial | Closing 会阻止 Ready-only focus；缺 Busy/RetryAfter typed outcome。 |
| **P1-29** release 删除证据且无 Closed receipt | Open | 正常关闭删除 `session.lock`。 |
| **P1-30** terminal ledger cleanup 无 maintenance owner | Partial | cleanup failure 可 deferred；无 bounded retry/index owner。 |

### 5.5 P1：Hub、Focus、Ready、Recent（31-45）

| Finding | 状态 | 当前证据与退出条件 |
|---|---|---|
| **P1-31** Ready receipt 缺 activation/manifest digest | Open | receipt 只有 process/instance/generation/milestones。 |
| **P1-32** first-present 后无 Degraded/Revoked/Closed | Open | callback 只发布一次 Ready。 |
| **P1-33** focus callback 错误只写 stderr | Open | ack 失败 `eprintln!`；需 owner health 与 terminal ack。 |
| **P1-34** focus disposition 无 audit history | Partial | typed request/ack/rejection 已有；完成文件删除后不可查询。 |
| **P1-35** mailbox 无 ACL/capability handshake | Open | 仍依赖 project-local filesystem mailbox。 |
| **P1-36** handshake 无统一 phase/deadline/cancel schema | Open | 仍是局部 poll/timeout policy。 |
| **P1-37** public failure 与 private diagnostics 未分离 | Open | 多处 `error.to_string()`/display path 进入跨进程消息。 |
| **P1-38** mailbox cleanup/retention/replay owner 缺失 | Open | token 校验存在，terminal mailbox 无统一 ack/expiry/quarantine。 |
| **P1-39** focus outcome taxonomy 不完整 | Partial | owner Focused ack 已有；缺 Delivered/ForegroundDenied/Unavailable。 |
| **P1-40** foreground denied 无协议化 fallback | Open | native focus event 不来只能 timeout。 |
| **P1-41** watcher rebind 不在 session receipt | Partial | retire/stale-ack/start 顺序存在；与 switch/close generation 非原子。 |
| **P1-42** recent 缺长期 journal/maintenance/qualified identity | Partial | bounded lease、revision/CAS、logical clock、tombstone、quarantine、atomic publish 已有。 |
| **P1-43** recent identity 是 lossy display path | Open | `to_string_lossy()`进入 validation/UI model；改用 ProjectIdentity + display alias。 |
| **P1-44** auto-open 不消费 crash/trust/BuildSet risk | Open | recent 只是 projection。 |
| **P1-45** single/multi-project/window topology 未定义 | Open | 当前 one-guard/one-binding 是实现偶然，不是产品 contract。 |

### 5.6 P1：入口、测试与性能（46-60）

| Finding | 状态 | 当前证据与退出条件 |
|---|---|---|
| **P1-46** 入口无统一 pending/terminal UI model | Partial | 已统一到 intent；无 operation timeline/command receipt。 |
| **P1-47** CLI/Hub/Welcome error/cancel/retry 不统一 | Open | 各层仍拼自由字符串和独立 timeout。 |
| **P1-48** startup 无 per-phase timing | Open | manifest/lease/runtime/plugin/document/present 无统一 trace。 |
| **P1-49** switch/close veto 无共同 receipt | Partial | save/import/Play 有阻断；asset/extension/document 无共同 drain receipt。 |
| **P1-50** 多项测试仍是源码 shape 断言 | Open | `include_str!/.find/.contains`不能证明写入、crash、并发。 |
| **P1-51** 无 Ready-before-ledger crash regression | Open | 当前测试反而要求危险顺序。 |
| **P1-52** 无 host early-return guard leak matrix | Open | plugin/template/scene/layout/focus/finalization 无 fault points。 |
| **P1-53** 无 duplicate operation 并发测试 | Open | dedup authority 尚不存在。 |
| **P1-54** trust/profile 隔离覆盖不足 | Partial | Safe/Recovery focused tests 存在；无真实 DLL/script materializer E2E。 |
| **P1-55** 无跨存储 split-brain recovery matrix | Open | recovery 只按 session operation 读取单 ledger。 |
| **P1-56** 无真实 Hub+Editor 双进程 E2E 证据 | Open | 本轮未发现并执行完整产品 harness。 |
| **P1-57** 无 heartbeat pause/PID reuse/clock rollback 测试 | Open | 无 controlled clock/OS process harness。 |
| **P1-58** bounded parser 覆盖不完整 | Partial | recent/ledger/focus 局部有界；session lock `read_to_string`仍无 byte gate。 |
| **P1-59** 无 network/removable lock qualification | Open | 无 storage semantics lab 与 fail-close policy。 |
| **P1-60** 无 startup/focus/close 统计性能基线 | Open | 无固定 corpus 的 p50/p95/p99、CPU/RSS/I/O。 |

### 5.7 P2

| Finding | 状态 | 当前差距 |
|---|---|---|
| **P2-01** | Open | Project Session Inspector 不存在。 |
| **P2-02** | Open | compatibility/migration/trust decision history 不存在。 |
| **P2-03** | Open | activation/present/focus/close distributed timeline 不存在。 |
| **P2-04** | Open | signed project catalog 与 key rotation 不存在。 |
| **P2-05** | Open | per-project capability trust policy 不存在。 |
| **P2-06** | Open | launch provenance UI 与 retry lineage 不存在。 |
| **P2-07** | Open | effect-aware Recovery Assistant 不存在。 |
| **P2-08** | Open | bounded terminal receipt index 不存在。 |
| **P2-09** | Open | focus topology/foreground capability inspector 不存在。 |
| **P2-10** | Open | recent workspace/tag/pin/relocation model 不存在。 |
| **P2-11** | Open | multi-window/read-only viewer policy 不存在。 |
| **P2-12** | Open | privacy-aware support bundle 不存在。 |
| **P2-13** | Open | deterministic crash-point simulator 不存在。 |
| **P2-14** | Open | startup performance budget/CI regression lane 不存在。 |
| **P2-15** | Open | multi-BuildSet/legacy/revoked-signature compatibility lab 不存在。 |

## 6. Canonical 资格门

| Gate | 状态 | 当前裁决 |
|---|---|---|
| `PROJ-GATE-01` duplicate operation | Fail | 无 durable dedup journal/terminal lookup。 |
| `PROJ-GATE-02` payload conflict | Fail | operation 复用不校验 target/policy digest。 |
| `PROJ-GATE-03` Ready commit fence | Fail | Ready 早于 Session commit 且不带 digest。 |
| `PROJ-GATE-04` crash replay | Fail | 无覆盖所有 effect 的 replay coordinator。 |
| `PROJ-GATE-05` Hub Ready after commit/present | Partial | Hub 等 first-present，但底层 Ready 可 split-brain。 |
| `PROJ-GATE-06` Ready revocation | Fail | 无 Degraded/Revoked/Closed。 |
| `PROJ-GATE-07` focus outcome taxonomy | Partial | request/ack/rejection 有类型，终态不全。 |
| `PROJ-GATE-08` switch watcher retirement | Partial | 有 retire/stale gate，无 durable receipt。 |
| `PROJ-GATE-09` Closing retry contract | Partial | Closing 不可 focus，缺 Busy/RetryAfter。 |
| `PROJ-GATE-10` early-return shutdown receipt | Fail | host 构造后错误出口绕过 close。 |
| `PROJ-GATE-11` bounded Drop/quarantine | Partial | Drop 不等待并保留 residual，但不显式写 terminal reason。 |
| `PROJ-GATE-12` reverse dependency close | Partial | normal close 有顺序，异常出口和全 effect receipt 缺失。 |
| `PROJ-GATE-13` Normal trust approval | Fail | 无 signature/trust/approval。 |
| `PROJ-GATE-14` Safe/Recovery pre-materialization block | Pass | project-derived capabilities 在 materialize 前被移除。 |
| `PROJ-GATE-15` approval digest invalidation | Partial | manifest 变化拒绝，其他 qualification 不共同失效。 |
| `PROJ-GATE-16` data-only preflight | Pass | preflight 不构造 runtime project 或执行 project code。 |
| `PROJ-GATE-17` migration decision receipt | Partial | typed plan/block 有，choice/artifact/replay 无。 |
| `PROJ-GATE-18` unsupported filesystem fail-close | Fail | 无 network/removable semantics detection。 |
| `PROJ-GATE-19` PID/clock reuse safety | Fail | instance/heartbeat 依赖 PID/wall clock。 |
| `PROJ-GATE-20` residual takeover qualification | Partial | explicit Recovery + terminal ledger 有，identity/permission 不足。 |
| `PROJ-GATE-21` bounded corrupt input | Partial | recent/ledger/focus 局部有界，lock/协议统一 quarantine 缺失。 |
| `PROJ-GATE-22` public error redaction | Fail | 公共错误可含 path/raw error。 |
| `PROJ-GATE-23` mailbox nonce/expiry/ack/retention | Partial | focus 局部具备，handshake/replay/retention 未统一。 |
| `PROJ-GATE-24` mailbox ACL/principal | Fail | 无 ACL 与 capability handshake。 |
| `PROJ-GATE-25` recent corruption isolation | Pass | corrupt/oversize 可隔离重建且不回滚 activation。 |
| `PROJ-GATE-26` recent monotonic merge | Pass | revision/logical clock/tombstone/CAS 已存在。 |
| `PROJ-GATE-27` unified launch intent | Partial | 主要入口使用 intent，仍有内部 direct API/create 单 operation。 |
| `PROJ-GATE-28` retry/double-click integration | Fail | 无 concurrent duplicate/restart coverage。 |
| `PROJ-GATE-29` split-brain deterministic action | Partial | ledger inventory 有，跨 identity/plugin/document/Ready 不全。 |
| `PROJ-GATE-30` close veto/drain | Partial | 局部 veto 有，无共同 drain receipt。 |
| `PROJ-GATE-31` Windows two-process E2E | Fail | 未发现并执行完整产品 harness。 |
| `PROJ-GATE-32` crash-point harness | Fail | 无每个 atomic replace/callback kill matrix。 |
| `PROJ-GATE-33` unknown-state guard fault proof | Fail | 只有局部 unit/source assertions。 |
| `PROJ-GATE-34` statistical performance evidence | Fail | 无大型项目统计 artifact。 |
| `PROJ-GATE-35` bounded growth/wait | Partial | focus/recent/ledger 局部有界，整体 phase 无统一预算。 |
| `PROJ-GATE-36` phase cancellation/deadline | Fail | 多数 phase 无 cancel/deadline/retry-after。 |
| `PROJ-GATE-37` shared session generation | Partial | session/focus/Hub 一致，plugin/document/window 未携带。 |
| `PROJ-GATE-38` shutdown diagnostics/support bundle | Partial | residual evidence 有，无脱敏 support bundle/close receipt。 |
| `PROJ-GATE-39` terminal receipt replay safety | Fail | terminal receipt store 不存在。 |
| `PROJ-GATE-40` reproducible evidence | Partial | 本轮有 163-file 指纹与静态复核，无动态 qualification。 |

## 7. 目标架构

```text
ProjectLaunchRequest(RequestId, Attempt, Deadline)
  -> ProjectLaunchIntent(OperationId, ParentOperationId, PayloadDigest, Principal)
  -> LaunchJournal.compare_or_begin(OperationId, PayloadDigest)
  -> ProjectQualificationAuthority
       ProjectIdentity + BuildSet + provider/toolchain/plugin catalog
       migration + TrustDecision + CapabilityPolicy + expiry
  -> ProjectSessionCoordinator.acquire(QualificationDigest, Fence)
  -> ActivationTransaction
       prepare effects -> execute with typed leases -> durable effect receipts
       -> commit ActivationReceipt(OperationId, Identity, BuildSet, EffectDigest, Generation)
  -> publish Ready(ActivationReceiptDigest)
  -> bind document/plugin/window/focus/Hub to the same generation and digest
  -> RecentProjection(best effort, ProjectIdentity keyed)

ShutdownCoordinator
  -> stop admission / publish Closing
  -> drain UI, focus, document, plugin, runtime in reverse dependency order
  -> commit ClosedReceipt or RecoveryRequiredReceipt
  -> release OS lease
```

关键不变量：

1. 相同 `OperationId + PayloadDigest` 只能执行一次；相同 operation 的不同 payload 必须返回 conflict，terminal 请求必须 replay receipt。
2. project-derived code 在 qualification/trust approval 前不可 materialize；Safe/Recovery 是 capability policy，不是 UI flag。
3. Ready 只能引用已 durable 的 `ActivationReceiptDigest`；任何 consumer 必须同时验证 session generation、fence 与 digest。
4. create artifact 与 activation 是父子事务；activation 失败不能伪装成 create 全失败，也不能丢失已创建 artifact 的 terminal 状态。
5. 所有 host 出口都经过一个 shutdown coordinator；Drop 只报告未收敛状态，不能承担业务提交。
6. recent 只能是 rebuildable projection，key 为 ProjectIdentity；display path/alias 不参与权威身份。

## 8. 分层重构计划

### M0：RED contracts 与 fault harness

- 为五项 P0 建立可控制 atomic writer、filesystem、clock、process、mailbox 和 callback fault points。
- 覆盖 ledger/session 每个写入间隙、duplicate operation、create 后 activation 失败、host post-construction 每个早退点。
- 先固定 public error redaction、record byte/depth/retention 和 supported filesystem matrix。

### M1：LaunchJournal、Qualification 与 Trust

- 新建唯一 `ProjectLaunchJournal`，实现 compare-or-begin、payload conflict、attempt lineage、terminal replay 和 bounded maintenance。
- 将 ProjectIdentity、BuildSet、provider/toolchain/plugin catalog、migration、trust、capability 与 expiry 编入 qualification receipt。
- Normal 必须取得显式 approval；Safe/Recovery 保留当前 pre-materialization 减权并扩展细粒度能力。

### M2：ActivationReceipt 与单一 commit fence

- effect closure 返回 typed resource lease、effect digest、compensation 和 terminal disposition。
- 先持久化完整 Committed effect set 和 ActivationReceipt，再发布引用 digest 的 Ready generation。
- hard cut 删除“Ready 后再 commit Session”的顺序和没有 receipt 的 direct activation API。

### M3：Project/Create/Recent identity 收束

- create transaction 产出带目录树/manifest digest、durability 与 recovery artifact 的 `CreateArtifactReceipt`；activation 使用 child operation。
- session/ledger/plugin/document/window/focus/Hub/recent 全部携带 ProjectIdentity、generation、fence/receipt digest。
- recent 保留现有 bounded store/CAS/tombstone/quarantine，实现 qualified identity、alias/relocation、risk policy 与 maintenance。

### M4：Hub/Focus 产品协议

- 统一 launch/focus mailbox 的 ACL、capability、nonce、phase、deadline、cancel、ack、retention 和 redacted public outcome。
- 增加 Degraded/Revoked/Closing/Closed、ForegroundDenied、RetryAfter 和 owner-health terminal state。
- Hub/App 成为 child process supervision owner，负责 timeout/cancel/reap 与 terminal reason。

### M5：Shutdown、Close 与 Recovery

- 用 `ShutdownCoordinator` 收敛 `run_editor_with_config` 全部出口，逆依赖 drain 所有 effect。
- Closed/RecoveryRequired 进入 bounded terminal index；release guard 只能发生在 terminal receipt durable 后。
- Recovery Assistant 消费 receipt inventory，执行 typed retry/rollback/restore/manual action，并保持 identity/digest/fence 校验。

### M6：Qualification 与性能

- Windows 真实 Hub+Editor 双进程覆盖 double-click、focus、foreground denied、close、crash、PID reuse、clock rollback 和 restart replay。
- local/network/removable filesystem 建 lock/rename/fsync/ACL 支持矩阵，不满足要求的写 session fail-close。
- 固定 10/100/1K plugin、10K/100K asset、不同 BuildSet 的启动 corpus，记录 p50/p95/p99、CPU、RSS、allocation、I/O 与失败成本。

## 9. 验收顺序与退出条件

1. **F0**：P0-04 launch dedup 和 P0-03 trust qualification 先关闭，否则后续 activation 仍可重复执行不可信 payload。
2. **F1**：P0-01/P0-02 以单一 ActivationReceipt commit fence 关闭；每个 durable write gap 的 kill/restart 都产生唯一 deterministic action。
3. **F2**：P0-05 关闭；host 构造后的每个错误出口都得到 Closed 或 RecoveryRequired receipt，OS lease 不会无解释地释放。
4. **F3**：create/open/recent/Hub/focus/recovery 全链只接受 qualified ProjectIdentity + generation + receipt digest，旧 root/string authority hard cut 删除。
5. **F4**：真实双进程、跨文件系统、故障、scale、soak 与统计性能门通过后，才可关闭 P1-56/59/60 与性能声明限制。

本轮仅完成 review、index 和 coverage 记录。未修改 production Rust、Cargo、ABI、测试或 UI；未运行 Cargo、Editor/Hub 产品动态、fault、multi-process、filesystem qualification、scale、soak 或 benchmark；Tooling 按用户要求排除，也未查询、轮询、等待或实时跟踪协调器。
