---
title: Editor51 Project Session Transition Authority
category: zircon_editor
report_id: Editor51-project-session-transition-authority-2026-08-31
date: 2026-08-31
session_id: root-editor51-project-session-transition-authority-20260831
parent_plan: docs/plans/optimize/zircon_editor/51-editor-project-startup-open-create-authority-hub-handshake-session-guard-focus-recent-recovery-product-integration-review.md
implementation_status: project_session_effect_ledger_close_coordinator_review_pass
validation_status: static_pass_independent_review_pass_managed_compile_queued
---

# Editor51 Project Session Transition Authority

## 目标与边界

本子计划修复同一 `EditorManager` 内项目激活、恢复接管和关闭事务可并发交错的结构性缺陷。
它不改变 `SessionGuard` 的跨进程文件锁协议、不重写 autosave policy、不声称完成 Editor51 M4-M7，
也不吸收当前工作树中其它未归属的 recovery、Hub 或 retained-host blob。

## 结构复审与算法结论

旧实现只在激活开始和结束时短暂锁住 `project_session_guard` 槽。两个不同工程可以同时通过空槽检查、
分别完成 runtime/document/UI effects；后完成者发现槽已占用后只释放自身 OS guard，不会回滚已经打开的
runtime project。关闭也可以在激活 effect chain 中途进入。Recovery profile 还在 Ready 返回后、恢复决策
安装前留下第二个关闭窗口。

Unreal 当前参考路径将 project switch 收敛到 `GameProjectUtils::OpenProject ->
FUnrealEdMisc::SwitchProject`，主框架命令以串行 UI action 发起，并在复杂切换中采用 editor restart。
Zircon 保留当前单进程可逆 activation ledger，但必须先补齐等价的单一 lifecycle transition authority，
不能继续依赖 guard 槽的两个短临界区推断完整事务互斥。

选择 Manager-local `Mutex<()>` 的原因：

- project activation/close 是低频控制面事务，串行化语义优先于并行吞吐；
- 一个 manager 对应一个 editor host/runtime owner，不把独立 manager 或独立 runtime 全局串行化；
- gate 只覆盖 4 个激活入口和 1 个关闭入口，heartbeat、focus snapshot、asset query 和 frame tick 不经过；
- 每次 transition 增加 1 次 O(1) lock/unlock，不增加文件 I/O、堆分配、轮询或 per-frame 工作；
- gate poison 不做 `into_inner` 恢复；任一 transition panic 后 Manager 生命周期进入 fail-closed quarantine，
  后续 open/create/recover/close 都返回显式错误，避免继续使用可能只完成一半的 runtime/UI 状态。

## 已完成实现

- 新增单一声明 owner `ui/host/project_session_transition.rs`，只包含 gate、不变量和线程互斥回归测试。
- `EditorManager` 持有一个 `ProjectSessionTransitionGate`，不暴露公共兼容 facade。
- open document、open-and-remember、recovery takeover、create-and-open 四条激活入口在 preflight 后一直持锁到
  Ready、recent projection和返回完成。
- `commit_project_close` 从 recovery-settled 检查一直持锁到 runtime close、projection cleanup、document close
  publication 和 SessionGuard 最终释放。
- Recovery takeover 在持有 residual writer lease 时重新生成 assessment，并与同一 residual record 做 exact
  identity compare；只有 terminal ledger assessment 才可接管，删除启动页 assessment 与 takeover 间的 TOCTOU。
- lease-protected assessment 按值移入 serialized recovery activation，恢复决策在释放 gate 前安装，删除
  Ready 与 recovery coordinator 之间的关闭窗口。
- Recovery coordinator 安装失败时将 Ready guard 转为 `RecoveryRequired` 并保留 exclusive fence；状态机
  禁止 `RecoveryRequired -> Closing`，避免错误返回后清除尚未处理的 residual recovery。
- session claim/takeover 后的 `PreflightApproved` 与 `Activating` 持久化统一进入 activation compensation
  边界；写盘失败先尝试显式释放 guard，释放失败则把 exact guard 留在 Manager 槽并进入 quarantine，
  不再因 early `?` 丢失 `session.lock` 的清理责任。
- gate 为独立 66 行 owner；强化测试后 session owner 一度达到 954 行，已将纯测试拆到独立 178 行 owner，
  生产 orchestration 回落到 773 行；`host/mod.rs` 只增加一条私有 module declaration。

## 验证与性能证据

- `rustfmt --edition 2021 --check`：7 个 exact-owned Rust 文件通过。
- `git diff --check`：7 个 exact-owned Rust 文件通过，仅有工作树既有 LF/CRLF 提示。
- RED 票据 `7405f72be0f4442ca029a839f6cddb65`：测试先提交，但 worker 运行前源码继续演进，终态为
  `snapshot_stale`；不计为 RED 动态通过证据。
- 初次独立 review 为 C0/I4/M0；其中 recovery failure quarantine、lease-protected assessment refresh、
  poison fail-closed 已修复。二次 review 为 C0/I1/M0，唯一问题是 2 个 source-contract test 使用了过时或
  自匹配的文本；生产正确性复核通过，实际 Manager effect-chain fault injection 被分级为测试缺口。
- 最终独立 review 为 C0/I0/M0；7 个源码哈希、模块可见性、入口顺序、reverse lock order、同步回调重入
  和高频路径污染均完成复核。实际 Manager effect-chain barrier/fault injection 保留为非阻塞增强测试。
- GREEN 托管 test 与 crate compile：ticket `6840999aeee9420a9e8f0b818266a19e` 与
  `1bb0c29b9cd24cc5a871708ceedb0bb7` 均在 materialization 被 stale attribution 拒绝，没有进入 Rust 编译。
  current-hash attribution request `0c88f15e87ea43a88816b6439877445a` 后，ticket
  `00f398963675498d8cd5679f5a4636d1`、copy `ebe38c43f00d40858ebcbedfd74299a2` 已完成 exact
  materialization 并进入 Cargo；Cargo job `e907bc47c1054d308dd47e7737a295a8` 在进入 editor 前以外部
  Runtime72 `E0599` 截断：base `CoreHandle` 缺少 `active_module_shutdown_order`。7 个 Editor51 owner 没有诊断。
- admission lifecycle compensation 的 fresh 托管验证 request `45822328ee074f64be467de64b7c9248`、copy
  `b4de7775fb9d4a3084dd4fef5bdb5001`、run `bbc1af9b15fd4a03bb762e9b6c8687fc` 以 exit 101
  终止；仍是同一 Runtime72 `active_module_shutdown_order` E0599，发生在进入 `zircon_editor` 前，因此不计为
  新回归测试通过。独立复审为 C0/I0/M1；M1 仅指出 source-contract 尚未替代 lifecycle write/release 的
  四象限 fault injection，不阻塞本次生产控制流修复。
- 本里程碑是正确性结构修复，不宣称启动/关闭耗时优于 Unreal；尚无同硬件 p50/p95/p99、CPU、RSS、I/O
  或功耗样本。性能结论仅限静态复杂度：5 个低频入口、0 个高频入口、每事务 1 个 O(1) gate。

## Exact Ownership 与当前哈希

| 路径 | SHA-256 |
|---|---|
| `zircon_editor/src/ui/host/project_session_transition.rs` | `10e21d86776531feec38c36724def2fe7240b5d78d0f2b26741d2cc81469f861` |
| `zircon_editor/src/ui/host/mod.rs` | `03e85492762e5b89eb18af4df9f69ae4fa89720e7b8518dc7250c94556b7529d` |
| `zircon_editor/src/ui/host/editor_manager.rs` | `e76f8346cebcbdce61ec74686a7f2397e1348bee456d8408b1c76cb5595893de` |
| `zircon_editor/src/ui/host/editor_manager_project_session.rs` | `1d41933033756e25fd8b457d8a9312908889c7ad04333c37bcbfd21d2978a6e9` |
| `zircon_editor/src/ui/host/editor_manager_project_session/tests.rs` | `f07802abeef051ecb5e2ed4789d5f75c6cae24448ee7c6fbae5f0427e687c94e` |
| `zircon_editor/src/ui/host/editor_manager_project.rs` | `f3acb8ba7d08335f6ec70f014d54425f9725a728b0a9e4ebbd992640dbbc7020` |
| `zircon_editor/src/ui/host/editor_manager_startup.rs` | `f1f20f44e545665c8c6cec1350f8996d16c3d09b72f2a886de7f48f21f6053c6` |

## Runtime72 外部阻塞复审

canonical Failure 为 `docs/plans/optimize/zircon_runtime/72/failure-2026-08-22-active-ledger-owner-wiring.md`，
保持 open，不重复建立 Failure。共享工作树 current `CoreHandle` 已出现 accessor，但 `runtime.rs`、
`core_handle.rs`、`core_runtime_state.rs` 同时混入 task graph、time、random 等多个未集成架构域；直接 fresh
transfer 会吸收非 Runtime72 blob，因此本轮拒绝接管。

Unreal `FModuleManager::UnloadModulesAtShutdown` 只收集已加载模块，按 `LoadOrder` 逆序排序，先统一
`PreUnloadCallback` 再卸载。Zircon current ledger 保留等价的“成功激活完成顺序逆序关闭”语义，并将稀疏
shutdown 从 declared `O(n)` 收敛到 active `O(k)`；但 `Vec<String>` 的 membership/retain 使单次 mutation
为 `O(k)`，全量反复激活/卸载可能达到 `O(k^2)`。现有 16,384 declared / 8 active / 21 pair gate 只覆盖
稀疏 shutdown，不覆盖 churn。后续 owner 必须补 active-count/churn profile 后再选择 Vec、ordered index 或
tombstone/index map，不能只为当前编译加 facade 或恢复 declared-graph scan。

## Close / Switch 当前拓扑与目标架构复审

当前 retained close 已正确覆盖 dirty document、排队 Save All、模型导入、资产移动/删除、命令面板、Play
session、Play gateway、runtime event consumers 与 deferred Play edit decision；Manager 也会先把 admission
record 持久化为 `Closing`，并在所有当前步骤结束后最后释放 exclusive guard。这里不再缺“有没有调用 Play
teardown”这一类局部步骤。

结构性断点在 Manager commit 内部：`host.close_project()` 先不可逆地 detach runtime project，随后
`clear_project_registration_reports()` 才对项目插件发送 `Disabled -> Unloaded`。插件 cleanup 失败时，
runtime 已关闭；如果只交换两行，runtime close 失败时又会留下“插件已卸载、runtime 仍打开”的相反半状态。
现有 `SessionGuard` 只能持久化 `Closing/RecoveryRequired`，不能保存每个 close effect 的 terminal
disposition；activation ledger 又会在 recent projection 后删除，且类型、文件名和 effect 集均只表达 activation。
因此禁止用调用重排、字符串错误拼接或兼容 facade 处理 E-PROJ-P1-30/P1-47/P1-48。

Unreal current `GameProjectUtils::OpenProject -> FUnrealEdMisc::SwitchProject` 把 project switch 明确定义为
editor restart：先进入 main-frame close gate，拒绝 save/load/GC/slow task 与活动 Interchange/Lightmass，允许
tab/plugin veto，完成 dirty package decision；shutdown 再终止 PIE/本地 Play/Launcher，最后在 `OnExit` 尽可能晚
spawn 目标工程进程，spawn 失败则撤销 pending project 并中止旧进程退出。Fyrox Project Manager 同样保存并
监督 editor child process，关闭 manager 时不会把仍运行的 child 当作已结束。

据此确定下一硬切：

- 新建单一 retained `ProjectCloseCoordinator`，状态固定为
  `Decision -> Quiescing -> Committing -> Closed | RecoveryRequired`，terminal 前禁止新 project writer/focus。
- 把 activation-only durable record 收敛为 `ProjectSessionEffectLedger`，按 session operation 记录 dirty/save、
  asset jobs、Play、project plugins、runtime project、documents、focus binding 与 workspace projection；每项只有
  `Prepared/Committed/RolledBack/RecoveryRequired`，错误必须携 exact effect inventory。
- plugin 与 runtime owner 必须返回 typed terminal receipt；close coordinator 只消费 receipt，不复制各子系统
  的 unload、stop 或 cancel 实现。exclusive guard 仍是最后释放项。
- project switch 默认走进程重启，不再把 `close old + open new` 当同进程热切换。只有完整 close receipt 和目标
  process spawn receipt 都成立才提交 switch；热切换在所有 global/project-derived owner 可证明 generation-safe 前
  不提供兼容路径。
- effect 数量为固定小常数，单次状态推进和恢复扫描均为 `O(E)`、空间 `O(E)`，不进入 frame tick。正确性与
  kill-point 证据完成前不做微观性能优化，也不宣称接近 Unreal 的耗时或功耗。

## Session Effect Ledger 与 Close Coordinator 实现终态

上述硬切已经落地，不再保留 activation-only 类型或目录兼容层：

- 删除 `core/recovery/activation_ledger/` 的 9 个旧 owner，新增
  `core/recovery/project_session_effect_ledger/` 的 10 个单一职责 owner；schema 固定为 1，持久化路径为
  `.zircon/session-effects/<operation>.json`，读取在 JSON decode 前受 8 KiB 上界约束。
- ledger 固定 5 个 phase（`Activating/Ready/Closing/Closed/RecoveryRequired`）、12 个 effect；activation
  inventory 为 6 项，close inventory 为 11 项。`begin_closing` 在一次原子写中清除 activation inventory 并预登记
  全部 close effect 为 `Prepared`，正常关闭为每个 effect 一次 terminal 写，最后一次 `Closed` 写。
- 新增 retained `ProjectCloseCoordinator`，状态固定为
  `Decision -> Quiescing -> Committing -> Closed | RecoveryRequired`；关闭 operation 必须与 project root 和
  operation id exact match，exclusive guard 仍是最后释放的 owner。
- project plugin 与 runtime close 分别返回 typed terminal receipt。coordinator 只有在插件 inventory 已清空、
  runtime 返回 exact root、asset projection 已失活且 watcher transition 成功时才写入对应 `Committed`；失败只把
  exact owner 标记为 `RecoveryRequired`，不再把未知半提交展平为 effect 名列表。
- persisted decode 校验 phase、inventory 和 disposition 的可达状态组合；测试覆盖所有内部可达 phase，另覆盖
  `Closed + Runtime=Prepared` 非法记录。activation abort 先完成 terminal ledger，Manager 释放 guard 后才清理
  `Closed` 记录，删除了旧 cleanup-before-fence-release 顺序。
- `editor_manager_project_session.rs` 从 1029 行降至约 713 行；旧 666 行 `project_session_close.rs` 已硬切为
  folder-backed 的 11 个 owner，最大 `persistence.rs` 为 314 行，coordinator 为 105 行，不保留同名旧文件或 facade。
- 第二轮复审发现 normal close 合同错误复用于 activation compensation、ledger mutation 可写出 decode 不接受状态、
  plugin receipt 跨锁拼接三个结构问题，均已修复：compensation 明确接受 `AlreadyAbsent/AlreadyEmpty`，normal close 仍
  要求 exact root；Closing 禁止 rollback，Ready+Recent recovery 可达，所有 encode 前运行同一 invariant；plugin 的
  before/after/terminal generation 在单一 lifecycle mutation guard 内生成，两个调用方都验证 terminal receipt。

算法复杂度保持为低频控制面 `O(E)` 时间和 `O(E)` 空间，当前 `E=11`；无 frame tick、轮询或按 catalog 规模扫描。
这只是结构与静态复杂度结论。尚未取得同硬件 p50/p95/p99、CPU、RSS、I/O 与功耗数据，因此不声称已达到
Unreal/Fyrox 的耗时或能效经验值。

## 剩余工作

- exact current-hash attribution 已刷新，fresh compile manifest 已删除旧单文件 tombstone，并纳入 folder-backed
  close owner；ticket `f256cff656544039b72c3bb84cdae6df` 当前为 queued，尚未提供 Rust GREEN。
- 只有在 compile/test terminal evidence 和独立 C0/I0 复审同时完成后，才进入 coordinator integration、atomic
  commit 和 WeCom。
- 下一实现接 project switch restart；禁止恢复同进程 `close old + open new` 兼容路径，也禁止退回 plugin/runtime
  两行重排。
- Editor51 后续仍需 focus ack、first-present 和多进程 fault/scale qualification。

## 产出记录与时间

| 时间（Asia/Shanghai） | 项目 | 状态 | 量化证据 |
|---|---|---|---|
| 2026-08-31 06:03 | 当前 topology、Unreal project switch 与回调重入复审 | completed | 发现 2 个生命周期竞态窗口；拒绝在 autosave policy 上做无证据微调 |
| 2026-08-31 06:06 | Exact ownership transfer 与 lease | completed | 初始 6 个 Rust 路径 + 1 个子计划路径；无 maintenance override |
| 2026-08-31 06:18 | Project session transition authority 初版 | completed | 5 个低频入口、1 个 Manager-local gate、0 个 per-frame 入口 |
| 2026-08-31 06:39 | 初次独立 code review | needs_changes | C0/I4/M0；确认无 reverse lock order/reentrancy 和高频路径污染 |
| 2026-08-31 06:45 | Review 修复：recovery quarantine、TOCTOU、poison | completed | 3 个 correctness finding 已落实；assessment 在 residual lease 内刷新并 exact compare |
| 2026-08-31 06:49 | 二次独立 code review | needs_changes | C0/I1/M0；生产修复通过，定位 2 个确定失败的 source-contract test |
| 2026-08-31 06:54 | 测试契约修复与模块拆分 | completed | 7 个源码 owner；production 954 -> 773 行，tests 178 行；静态 gate 计数 4+1 |
| 2026-08-31 06:57 | 最终独立 code review | completed | C0/I0/M0；7/7 哈希一致；fault injection 仅为后续增强测试 |
| 2026-08-31 07:04 | 首次 final materialization | infrastructure_stale | ticket `6840999aeee9420a9e8f0b818266a19e`；`validation_copy_attribution_stale`，0 条 Rust 诊断 |
| 2026-08-31 07:08 | Exact lease/snapshot refresh | completed | 8 路 lease；snapshot `2416`；7 个源码哈希保持不变 |
| 2026-08-31 07:16 | Current-hash attribution refresh | completed | request `0c88f15e87ea43a88816b6439877445a`；7 个 source attribution 与 manifest 完全一致 |
| 2026-08-31 07:30 | Final managed compile | external_blocked | ticket `00f398963675498d8cd5679f5a4636d1`、copy `ebe38c43f00d40858ebcbedfd74299a2`、Cargo `e907bc47c1054d308dd47e7737a295a8`；进入 editor 前仅 1 条 Runtime72 E0599，Editor51 诊断 0 |
| 2026-08-31 07:38 | Runtime72 whole-module/Unreal/complexity review | completed | canonical Failure 复用；拒绝吸收 task graph/time/random mixed blob；shutdown O(k)，mutation O(k)，churn worst-case O(k²) 待 profile |
| 2026-08-31 09:18 | Close/switch 全链与 Unreal/Fyrox 结构复审 | completed | 复核 retained 8 类前置 owner、Manager 6 类 commit effect、Unreal restart 与 Fyrox child supervision；确认 1 个双向半提交结构缺口，确定 `O(E)` durable effect ledger + restart hard cut |
| 2026-08-31 12:30 | Admission lifecycle 持久化失败补偿 | implementation_complete_external_blocked | 2 个 post-claim 写盘点进入统一 compensation；独立复审 C0/I0/M1；托管 run `bbc1af9b15fd4a03bb762e9b6c8687fc` 在进入 Editor 前被同一 Runtime72 E0599 截断，Editor51 动态通过证据仍为 0 |
| 2026-08-31 14:18 | Activation ledger -> Project session effect ledger 硬切 | implementation_complete | 删除 9 个旧 activation-only owner；新增 10 个 ledger owner；5 phase、12 effect、11 close owner；schema 1、read cap 8 KiB |
| 2026-08-31 14:31 | 首轮 close/ledger 独立复审 | needs_changes | C0/I4/M1；定位 owner receipt、exact disposition、guard cleanup 顺序、decode invariant 与 test cfg 5 类问题 |
| 2026-08-31 14:43 | 复审问题修复与 Manager 模块拆分 | completed | 5/5 类问题已落实；session owner 1029 -> 约 713 行，close owner 约 607 行；plugin/runtime 使用 typed terminal receipt |
| 2026-08-31 14:49 | Exact source 静态验证 | passed | 30 个 current Rust owner 通过 rustfmt check 与 scoped diff-check；旧 activation symbol 命中 0 |
| 2026-08-31 14:51 | Managed compile snapshot R3 | infrastructure_stale | ticket `645260d69513455a93e2d6f31fce5aaf`、manifest `2e70f16610ad2709215cc21128d3fb78366358803e7a6babb7865c7ef9779451`；终态 `snapshot_stale`，Rust 编译与诊断均未开始 |
| 2026-08-31 15:06 | 第二轮 close/ledger 独立复审 | needs_changes | C0/I3/M1；定位 compensation terminal、ledger write/decode invariant、plugin atomic receipt 与 666 行 umbrella owner |
| 2026-08-31 15:24 | 第二轮结构修复 | implementation_complete_review_pending | normal close/compensation typed 合同分离；每次 encode 前 invariant；Closing rollback 拒绝；plugin receipt 单锁生成并双调用方验 terminal；旧 666 行文件 -> 11 owner，最大 314 行 |
| 2026-08-31 15:27 | 第二轮修复静态检查 | passed | 20 个变更/新增 Rust owner 通过 rustfmt check 与 scoped diff-check；旧 close 单文件工作树存在性为 false；动态 Rust GREEN 仍为 0 |
| 2026-08-31 21:39 | Close/ledger 最终独立复审与 fresh managed compile 入队 | static_pass_compile_queued | 最终复审 C0/I0/M0；normal close 与 activation compensation 均在 destructive runtime close 前 exact preflight，receipt 区分 `ClosedActive/AlreadyEmpty/AlreadyAbsent`；direct/source-order 回归已补。manifest `bd8ef75b4edfe1fd6d780378e9de38889ee0b78c426413b6754b91192abc53cf` 覆盖 40 个现存 Rust owner且不含删除 tombstone，ticket `f256cff656544039b72c3bb84cdae6df` 为 queued；动态 GREEN 仍为 0。 |
