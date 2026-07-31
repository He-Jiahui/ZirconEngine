---
related_code:
  - tools/session_coordinator/sessions.py
  - tools/session_coordinator/governance.py
  - tools/session_coordinator/cargo_runner.py
  - tools/session_coordinator/cargo_jobs.py
  - tools/session_coordinator/cargo_reservations.py
  - tools/session_coordinator/failures.py
  - tools/session_coordinator/work_continuations.py
  - tools/session_coordinator/validation_tickets.py
  - tools/session_coordinator/integration_candidates.py
  - tools/session_coordinator/offline_queue.py
  - tools/session_coordinator/git_finalize.py
  - tools/session_coordinator/workflows/milestones.py
  - tools/session_coordinator/baselines.py
  - tools/session_coordinator/validation_copies.py
  - tools/session_coordinator/manifest_retention.py
  - tools/session_coordinator/ownership_transfers.py
  - tools/session_coordinator/database.py
  - tools/session_coordinator/migrations.py
  - tools/session_coordinator/server.py
implementation_files:
  - tools/session_coordinator/sessions.py
  - tools/session_coordinator/governance.py
  - tools/session_coordinator/cargo_runner.py
  - tools/session_coordinator/cargo_jobs.py
  - tools/session_coordinator/cargo_reservations.py
  - tools/session_coordinator/failures.py
  - tools/session_coordinator/work_continuations.py
  - tools/session_coordinator/validation_tickets.py
  - tools/session_coordinator/integration_candidates.py
  - tools/session_coordinator/offline_queue.py
  - tools/session_coordinator/git_finalize.py
  - tools/session_coordinator/workflows/milestones.py
  - tools/session_coordinator/baselines.py
  - tools/session_coordinator/validation_copies.py
  - tools/session_coordinator/manifest_retention.py
  - tools/session_coordinator/ownership_transfers.py
  - tools/session_coordinator/database.py
  - tools/session_coordinator/migrations.py
plan_sources:
  - user: 2026-07-31 收敛会话状态、数据库 manifest、Plan WIP 与共享 main 集成
  - docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
tests:
  - tools/session_coordinator/tests/test_sessions.py
  - tools/session_coordinator/tests/test_governance.py
  - tools/session_coordinator/tests/test_cargo_jobs.py
  - tools/session_coordinator/tests/test_failures.py
  - tools/session_coordinator/tests/test_failure_closeout.py
  - tools/session_coordinator/tests/test_validation_tickets.py
  - tools/session_coordinator/tests/test_integration_candidates.py
  - tools/session_coordinator/tests/test_manifest_retention.py
  - tools/session_coordinator/tests/test_plan_wip.py
  - tools/session_coordinator/tests/test_ownership_transfers.py
  - tools/session_coordinator/tests/test_deferred_action_client.py
  - tools/session_coordinator/tests/test_offline_command_spool.py
  - tools/session_coordinator/tests/test_workflow_commit.py
  - tools/session_coordinator/tests/test_git_finalize_tracked_ignored.py
  - tools/session_coordinator/tests/test_baselines.py
  - tools/session_coordinator/tests/test_workspace_copy.py
  - tools/session_coordinator/tests/test_server.py
doc_type: milestone-detail
---

# Coordinator 状态收敛、Manifest 保留与 Plan WIP 治理设计

## 决策状态

- 设计方案：服务内受审计收敛，完整 manifest 保留 7 天，历史负载压缩归档到 Git 外。
- 用户批准：2026-07-31，方案 A。
- 实施状态：进行中；源码级约束和聚焦验证按里程碑分批落地，生产应用与 `main` 集成仍须保留独立证据。

## 目标与非目标

本治理里程碑同时解决六个相互关联的问题：确保协调器和验证队列绝不让整个 Session 等待，让编译通过且所有权明确的代码尽早进入 `main`，收敛失真的 Session/Cargo 投影，停止重复 manifest 使 SQLite 无界增长，以编号 Plan 为单位限制并发主会话，以及把共享工作树按所有权切成可审查提交。

当前容量审计基线约为：`baseline_epochs.manifest_json` 占 930 MB，`validation_copies.manifest_json` 占 234 MB。它们是 retention/compact 的生产前后对照基线，不是允许误删引用记录的目标值；验收必须同时证明受保护 manifest 可读、归档哈希一致和 SQLite 物理体积实际下降。

本里程碑不终止仍存活的 Cargo 进程，不用额外 Cargo 测试补偿协调器故障，不把活动会话的修改归给治理会话，也不通过直接 SQL、删库、删除审计事件或绕过 coordinator lease 达成表面收敛。验证排队、验证副本物化、数据库维护、跨 Plan Failure 或单个 lease 冲突都只能挂起其直接依赖切片，禁止把整个 Session 留在轮询、睡眠或等待状态。

## 核心不变量

1. 只有新鲜 heartbeat、活动 lease、非终态 patch、活动 Cargo job 或活动 reservation 才能阻止 Session 进入 stale；历史文本状态本身不是活跃证明。
2. 任何 Cargo run 只有在父 job 已有可证明终态且 PID 树为空后才能被终结；真实运行进程永不因状态清理被停止。
3. 未绑定 job 的 pending reservation 可因绝对 TTL 或 owner 非 executable 而过期；已绑定或 running reservation 不由普通状态清理回收。
4. 最新 baseline、任何非终态 Session 引用的 baseline，以及所有非终态 validation copy 始终保留完整 manifest。
5. 历史 manifest 只有在压缩归档成功、逐项哈希复核成功且数据库备份存在后才能从 SQLite 主表退休。
6. 每个规范化编号 Plan 同时最多一个可写主会话；只读审查会话必须显式关联主会话，且最多一个。
7. Git 提交只能包含会话可证明拥有的精确 blob；早期集成还必须有匹配候选 manifest 的最小编译证据，完整验证只决定能否从 `integrated_validation_pending` 晋升为 `accepted`。全局脏工作树不是任何单一提交的授权范围。
8. 验证提交在有界时间内返回持久票据；排队、延后、合并、运行和结果等待属于 validation ticket，不得把业务 Session 变为不可执行状态。
9. 待验证只阻止对应源码快照被标为最终 `accepted`，不阻止 `integrated_validation_pending` 早期提交，也不阻止同一主 Session 继续实现任何无依赖切片、后续已批准工作、代码审查或静态检查。
10. Session 不执行 Cargo 槽位轮询。协调器在结果终态时主动更新 evidence 并唤醒需要处理结果的 Session；没有结果时不消耗 Session token。
11. 所有权明确的源码快照只要通过声明的最小编译门槛即可进入 `main`；聚焦测试、集成测试或产品验证仍可排队，不再作为早期集成前置条件。
12. `integrated_validation_pending` 与 `accepted` 是不同状态。前者允许其他任务立即消费主干提交，后者才表示完整里程碑验证完成。
13. 已集成快照的后续验证失败必须由 coordinator 生成幂等 `failure-*.md` 修复机会并绑定原 commit；除非发现所有权错误、编译证据失真或确定的不可逆仓库/数据破坏，默认不回滚已进入主干的提交。

## 组件边界

### 非阻塞开发与异步验证

验证生命周期从 Session enum 中拆出，成为绑定不可变 source manifest 的 durable validation ticket。`waiting_validation` 只允许作为旧数据兼容输入或 validation node 投影，不能作为会禁止 Session heartbeat、源码编辑、lease 请求、Failure 修复或独立里程碑工作的 Session 状态。迁移时，旧 `waiting_validation` Session 按开放 Failure 恢复为 `resolving_failure`，否则恢复为 `active`；其待处理验证保留在 ticket 中。

验证请求必须在健康服务下于两秒客户端 deadline 内持久化并返回 `validationTicketId`；该 deadline 只约束请求准入，不要求同步完成 source-copy 物化。Ticket 记录请求时的 owner scope、文件指纹、规范化命令、工具链、feature/config 与声明覆盖集，后台 snapshot worker 随后密封不可变 source manifest。若文件在密封前变化，ticket 进入 `snapshot_stale` 并主动回传，禁止静默改用新内容。服务繁忙、数据库 compact 或短时离线时，客户端把同一幂等请求写入仓库本地 offline queue 并立即返回 queued receipt；后台重放负责形成正式 ticket。调用方不得通过循环调用 status、反复 acquire reservation 或维持对话等待确认提交。

调度器只消费已密封 source manifest 的 ticket，并在后台处理 FIFO。完全相同的 source manifest、命令、工具链、环境兼容键与覆盖集共享一个执行，结果扇出给所有 ticket。只有覆盖契约能证明一个已声明批次完整包含多个请求时才允许合并；不同 source manifest、feature、toolchain 或无法证明覆盖关系的测试不得为节省槽位而混跑。相同 Plan 在短暂 batching window 内提交的兼容请求可以形成一个里程碑验证批次，但 batching window 不延迟 Session 返回。

验证票据的 `queued/materializing/running` 不改变主 Session 的可执行性。只有依赖该票据的 milestone acceptance 保持 `validation_pending`；compile-first integration finalizer 不等待完整测试。主 Session 立即重建行动队列，优先继续无依赖实现、已批准下游切片、静态守卫或完成代码审查。验证失败后只把直接依赖切片转入修复，跨 Plan 最低根因按 Failure handoff 路由，其余工作继续。

结果通过 coordinator event、Codex wakeup 和 work continuation 一次性回传；Session 不保持等待 turn。若 Session 已继续修改相同工作树文件，结果仍只证明 ticket 固定的旧 manifest。受管 finalizer 使用已验收 manifest/blob 形成精确提交，不要求回滚当前工作树；后续 manifest 需要独立 ticket，但相同的构建输入可复用 coordinator 已证明兼容的 target。

lease 冲突仍然禁止两个 Session 同时写同一路径，但只能挂起该路径对应的 patch。协调器应返回 delayed-patch receipt，主 Session 转向其他可执行工作；禁止把一个局部 lease 冲突升级为全 Session 等待。

### 编译通过即早期集成

交付状态拆成 `integration_ready`、`integrated_validation_pending` 与 `accepted`。Session 在形成一个连贯、可审查的所有权批次后，提交绑定同一不可变 manifest 的 validation ticket 与 integration candidate；候选包含精确 owned blobs、Conventional Commit 元数据、最小编译门槛和仍待运行的验证票据。编译门槛是覆盖受影响 package/target/feature profile 的最小 `cargo check` 或对应语言的等价编译检查，不扩大为 workspace 测试，也不要求聚焦测试先完成；禁止在每个微小编辑后创建编译候选。

编译票据仍在后台队列运行，Session 立即继续开发。编译通过后，coordinator finalizer 不等待 Session 新 turn，直接验证 owner、lease、blob hash、base/HEAD 与 commit manifest，并把精确快照提交到当前 `main`。工作树已经包含后续编辑时，finalizer 只提交候选中的旧 blob，不回滚或暂存后续内容。若 HEAD 已前进但候选路径未冲突，finalizer 在新 HEAD 上重建同一 owned tree；同一路径存在外部变化时只把该 integration candidate 转为 delayed merge，不阻止 Session 的其他工作。

早期提交记录 `integrated_validation_pending`、commit SHA、compile ticket、未完成 validation tickets 和已知 Failure 链。聚焦测试尚未运行，或者已经发现非编译类问题，都不阻止该提交供其他 Plan 使用；它们只阻止把里程碑标为 `accepted`。若后续失败，coordinator 把 Failure 绑定到原 commit 和最低修复 owner，优先生成向前修复候选。默认不自动 revert，也不把依赖无关的下游 Session 标 stale/blocked。

硬拒绝早期集成仅限：声明编译门槛失败或证据不匹配、owned blob/lease 归属不明确、当前 HEAD 同路径产生无法自动证明的冲突，或者已有证据表明提交会造成不可逆仓库/持久数据破坏。普通单测失败、验证尚未排到、可能存在行为问题或非破坏性 Failure 不属于早期集成拒绝条件。

进入 `main` 后立即刷新 baseline，并让其他 Plan 的 source snapshot 使用新 commit。原主 Session 在完整验证结束前仍占用该 Plan 的唯一 primary WIP 槽，但可在同一 Goal 内继续后续工作；早期集成不能被用于创建一串新的后继微会话。

### 验证失败的 Failure 回写

任一 validation ticket 进入失败终态后，coordinator 先持久化 Failure graph 事件，再以 `commit SHA + source manifest hash + validation ticket + 规范化失败边界 + fixing Plan` 作为生命周期幂等键，生成或更新 `failure-{YYYY-MM-DD}-{summary}.md`。重复执行、重复回传或同一合并验证结果的扇出只能补充现有 evidence，禁止产生多个等价 Failure 文件。完整日志保留在 validation evidence；Failure 文件只记录来源/修复 Plan、原提交与 manifest、失败命令和摘要、最低已知原因、架构修复验收以及禁止的临时绕过。

最低原因由原 Plan 自己拥有时，Failure 写入同一 numbered child-plan 目录，frontmatter 必须显式标记 `failure_scope: local`，并令 `origin_plan` 与 `fixing_plan` 相同。FailureGraph 和 handoff validator 只对该显式 local 形式省略依赖 self-edge，禁止把普通跨 Plan handoff 静默降格；修复完成后在同一目录原子改名为 `fixed-*`。最低共享原因属于其他 Plan 时，沿用既有 handoff：Failure 写入 fixing child-plan，修复通过 `failure return` 移回 origin child-plan 成为 `fixed-*`。原 Session 继续所有无依赖切片；fixing primary 在下一个 repair window 进入 `resolving_failure`，执行修复而不是等待测试或另建微会话。

Failure 文件必须通过 coordinator 的 plan authorization、lease 与受管文件操作写入。目标文件暂被占用时，Failure graph 和 repair continuation 仍立即持久化，文件写入转为 delayed patch；这不会让验证 Session 或修复 Session 原地等待。存在可执行 primary 时 coordinator 主动 wakeup；不存在时 Failure 保持 open，并成为该 Plan 下一次 primary 注册后的首个 repair item，而不是为每个失败自动创建新 Session。

### 状态收敛服务

新增独立的治理服务，而不是继续扩大 `server.py` 或把跨域规则塞入 `SessionService`。服务生成带稳定指纹的 preview，列出：Session 状态迁移、孤儿 run 修复、pending reservation 过期以及待退役的活动说明。Apply 必须携带 preview 指纹，并对每一行使用原状态、原 heartbeat/终态时间和资源存在性作为 compare-and-swap 条件；状态在 preview 后发生变化时跳过该对象并报告冲突。

Session 收敛顺序为：先刷新真实资源投影，再关闭可证明的孤儿 run 和 reservation，随后把超时的 `registered/active/waiting_lease/resolving_failure/waiting_validation` 变为 `stale`，最后把已 stale 至少 24 小时且没有 lease、patch、Cargo 或 reservation 的 Session 归档。已有 `mark_stale` 与 `archive_stale` 继续负责单行状态语义，治理服务只负责一致的候选选择、预览和批处理审计。

`CargoRunner.reconcile_terminal_runs` 扩展为三条明确路径：父 job 为 `succeeded/failed/released` 且有 exit code 时投影为 `completed`；父 job 为 `orphaned`、PID 树已确认退出时投影为 `completed`，保留已知 exit code，否则使用专用 `cargo_run_reconciled_from_orphaned_job` 错误码而不伪造成功；父 job 为 `released`、PID 树为空但历史 exit code 缺失时也允许投影为 `completed`，保留空 exit code 并记录 `cargo_run_reconciled_from_released_job_missing_exit_code`。父 job 仍为 `running` 或 PID 树非空时必须拒绝收敛。

根目录 `.codex/sessions/*.md` 只保留仍对应新鲜非终态 Session 或仍携带材料化协调警告的说明。退役操作移动到 `.codex/sessions/archive/` 并改为 `status: completed` 或 `status: archived`；无法可靠关联 owner 的文件只进入预览，不自动移动。

### Manifest 保留服务

Schema 为 `baseline_epochs` 与 `validation_copies` 增加 manifest 摘要、归档位置和归档时间字段，并增加一次 retention batch 的审计记录。每个摘要至少包含 SHA-256、条目数、原始 UTF-8 字节数和规范化归档键。现有 `manifest_json` 在保留期内仍是运行时读取入口，避免把普通基线扫描变成归档 I/O。

保留集合包括：

- 最新 baseline epoch；
- 所有非终态 Session 的 `baseline_epoch`；
- 最近 7 天创建的 baseline，以及最近 7 天创建或进入终态的 validation-copy；
- 所有非终态 validation-copy；
- 尚未被完整压缩归档和校验的任意记录。

退休流程先通过 SQLite backup API 生成带时间戳的本地备份，再把候选记录写入同目录临时 gzip JSONL。归档项包含表名、主键、完整 manifest、摘要和 batch id。文件关闭后重新读取并逐项核对，随后原子改名；只有此后，单个事务才把对应主表 manifest 收敛为空对象并写入摘要/归档引用。任一步失败都不修改主表 manifest。

逻辑退休完成后，显式数据库 compact action 在后台维护窗口执行 SQLite `VACUUM`。Compact 期间 Cargo 子进程继续运行，Session 过期时钟暂停，内存 health 端点保持可用；客户端 mutation 在本地 offline queue 持久化后立即返回 receipt，而不是等待数据库。依赖 SQLite 的只读查询返回带 retry hint 的 `database_maintenance_active`，调用方不得忙轮询。维护完成后后台按原 request id 幂等重放。Compact 前后执行 `quick_check`、schema version、关键表计数和 retention 摘要复核；失败时保留原备份并报告恢复步骤，禁止静默换库。

### Plan WIP 门禁

“Plan 族群”的规范化 key 使用 coordinator 已有的 numbered-plan owner 解析结果，而不是 Session 名称前缀或顶层 family 目录。一个 numbered Plan 及其 child output、Failure 修复和后继里程碑 Session 共享同一 WIP 族群；不同编号 Plan 不因同属 `zircon_runtime`、`zircon_editor` 等顶层目录而互相占用名额。Session 注册增加显式角色：`primary` 或 `reviewer`；reviewer 必须有 `parent_session_id`、空 write scope，且父 Session 属于同一 Plan 族群。

在同一数据库事务内，新建主会话前统计该 Plan 的 executable 主会话；已有一个时返回 `plan_wip_limit_reached`，并返回当前主会话、状态及最后 heartbeat。新建 reviewer 时最多允许一个，并拒绝任何写范围。现有 Session 的幂等重新注册不消耗新名额。

Failure 不再通过创建修复微会话绕开上限：当前主会话切换为 `resolving_failure`，完成 failure return 后恢复原里程碑。WIP 门禁只拒绝新建后继微会话，绝不禁止现有主会话继续其 Goal 内的无依赖开发。上线前已经超限的 Plan 不强杀会话；有真实资源的会话被 grandfather 到终态，无真实资源且 heartbeat 过期的会话由状态收敛服务处理。在数量回到上限前拒绝新的后继会话，并把工作路由回现有主会话。

### 所有权切片与提交

共享 `main` 的整理使用现有 session、lease、snapshot/commit manifest 与当前 blob 哈希生成所有权矩阵。早期集成切片必须同时满足：owner 唯一、候选 blob 与 owner 证据一致、绑定 manifest 的最小编译门槛通过、没有其他活动 lease 冲突。完整里程碑验证可以保持 pending；证据不足的路径保持未暂存并返回 owner 队列，不能因文件邻近或相同 Plan 家族被猜测归属。

历史归属缺失或 source Session 已非 executable 时，只允许维护者执行 `ownership transfer-preview` 和以同一指纹显式确认的 `ownership transfer-apply`。每个精确路径在 preview 与 apply 之间必须保持同一 baseline、当前 blob hash、source attribution 和无外部有效 lease；apply 在一个事务中扩展目标 primary Session 的 write scope、获取 lease、替换 attribution 并保留不可变审计记录。任何前置条件变化都拒绝整次应用，不推测归属，也不自动执行交接。

提交顺序为：先提交本治理里程碑中编译通过、所有权明确的 coordinator 代码和对应测试源码，再按其他 Plan 的 integration-ready 所有权切片分别执行受管 finalize；完整测试结果在后台补充 acceptance 或 Failure。会话说明退役作为独立、可审查的协调卫生切片，不与业务源码混合。任何提交均不得使用共享暂存区中来源不明的内容。

## 操作接口与审计

治理动作采用 preview/apply 两阶段：

- `governance converge preview/apply`：Session、run、reservation、说明退役；
- `governance retention preview/apply`：备份、压缩归档、主表 manifest 退休；
- `governance compact preview/apply`：数据库物理压缩和完整性复核；
- `ownership transfer-preview/apply`：恢复已放弃精确路径的 scope、lease 与 attribution，apply 需要维护能力和相同 fingerprint 的显式确认；
- `validation submit`：持久化请求后返回 durable ticket 或 offline queued receipt，source manifest 由后台密封；
- `integration candidate submit`：绑定 owned manifest、最小编译票据、提交元数据和待验证票据，编译通过后后台 finalize；
- `failure materialize/return`：把失败验证幂等写成 canonical `failure-*`、回写修复 continuation，并在修复验收后返回为 `fixed-*`；
- Session register：返回 Plan WIP 的允许结果或结构化拒绝原因。

每次 preview 记录候选数量、稳定指纹和拒绝原因；每次 apply 记录应用、跳过、冲突和失败数量。审计事件不得包含完整 manifest，只保存 batch id、摘要和归档相对路径。归档与备份位于 `.codex/state/session-coordinator/` 下，不进入 Git。

## 错误处理

- coordinator preflight 超时由客户端把原 request id 转入 offline queue，后台负责查询、去重和重放并主动回传；调用 Session 不自行轮询确认，也不盲目重复 mutation。
- validation submit、普通状态 mutation 和 compact 期间的请求必须有幂等 request id；deadline 内无法在线提交时写 offline queue 并返回，禁止让 Session 原地等待服务恢复。
- preview 指纹失效时整个 apply 返回 stale-plan，不部分使用旧候选集合；逐行 CAS 冲突作为新状态重新预览。
- 归档写入、gzip 复读、摘要校验、备份或 `quick_check` 任一失败时保留原始 manifest。
- 磁盘空间不足时在备份/归档前拒绝，预估空间必须覆盖数据库备份、归档临时文件和 VACUUM 临时空间。
- WIP 拒绝必须提供当前主会话，便于继续原里程碑或显式完成/取消，而不是诱导创建新名称重试。
- 编译通过后的后台 finalize 若遇到 HEAD 同路径冲突，必须返回 delayed integration receipt 并保留候选 blob；禁止静默改写、丢弃或要求 Session 停止其他开发等待人工提交。
- 后续测试失败必须把 Failure/validation evidence 关联到已经集成的 commit，由 coordinator 幂等物化 canonical `failure-*` 并回写原 owner 的修复 continuation；默认产生 forward-fix 工作而不是自动 revert。

## 验证与验收

实现阶段只运行协调器 Python 测试和复制数据库上的冒烟测试，不增加 Cargo 验证：

1. 临时数据库覆盖 active/resolving/stale 收敛、资源保护、CAS 冲突和 24 小时归档门槛。
2. Cargo fixture 覆盖两个当前孤儿形态（`orphaned` 无 exit、`released` 无 exit）以及一个真实 running PID 树不得被收敛。
3. Reservation fixture 覆盖 pending 无 job 的 TTL/owner 过期和 running reservation 保留。
4. Retention fixture 覆盖引用保留、7 天边界、gzip 原子归档、哈希失败回滚、重复 apply 幂等和 compact 前后完整性。
5. WIP fixture 覆盖每 Plan 一个 primary、一个关联 reviewer、Failure 状态复用主会话、幂等注册和既有超限迁移。
6. Validation fixture 覆盖两秒内 durable/offline receipt、无轮询继续执行、相同输入合并、不同 manifest 禁止合并、结果 wakeup，以及只有 milestone acceptance 被挂起而早期集成与后续无依赖开发不被挂起。
7. Compact fixture 覆盖 mutation 离线排队立即返回、Session stale 时钟暂停、后台幂等重放和真实 Cargo job 不受影响。
8. Integration/Failure fixture 覆盖 compile-pass/test-pending 自动提交、已知非编译 Failure 仍集成、后续 worktree 编辑不被回滚、HEAD 无冲突前进、同路径冲突延后、显式 `failure_scope: local` 的同 Plan 与跨 Plan Failure 路由、重复结果去重、lease 冲突 delayed patch、owner wakeup，以及测试失败 forward-fix。
9. 生产应用前先在 SQLite backup 副本执行完整 preview/apply/compact；生产应用后复核数据库大小、两个已知孤儿 run 的终态投影、一个真实 running run 的保留、reservation 队列、Session 计数和协调器响应延迟。
10. `git diff --check`、Plan 输出审计和精确 Python 测试批次通过后，才允许把里程碑标为 `accepted`；早期 `integrated_validation_pending` 提交只要求其 owned manifest 的编译门槛和 finalizer 完整性证据。

## 实施里程碑边界

- M1（必须最先闭环）：刷新真实资源投影，收敛 `active/resolving_failure/stale`，清理两个可证明的孤儿 run、失效 reservation 和过期会话说明；不触碰真实 running Cargo。
- M2：非阻塞 validation ticket/continuation、Failure 回写与 compile-first early integration。M2 完成后，后续各里程碑的所有权批次一旦最小编译通过即可持续进入 `main`，不等待完整测试。
- M3：Manifest 摘要、压缩归档、7 天保留和数据库 compact，以 930 MB baseline manifest 与 234 MB validation-copy manifest 为生产对照基线。
- M4：Plan 族群 WIP 注册门禁与现有超限收敛。
- M5：清点共享 `main` 的既有修改并按所有权矩阵形成 `integrated_validation_pending` 或完整 `accepted` 的可审查提交；这不阻止 M2 之后已就绪的 M3/M4 批次提前集成。

每个里程碑先集中完成代码与聚焦测试源码，再进入一次批量 Python 测试阶段；不为单个实现切片启动 Cargo。达到连贯、可审查的所有权批次时才创建一次最小编译候选；该后台编译通过即可先集成，批量 Python 测试可继续排队并在之后决定 `accepted` 或触发 forward-fix。M1、M3 和 M4 的数据迁移/门禁行为均须先在临时或备份数据库验证，M5 只提交所有权和 blob 证据明确的路径。

## 可执行实施计划

### 交叉会话与共享工作树约束

- 当前治理任务沿用 `5.6-sol / High`：它决定跨 Plan 所有权、主干提交和数据库恢复边界。每次触碰共享源码前，先以 coordinator lease 认领精确路径；现有脏改动不因为本计划而被覆盖、暂存或归属。
- 先创建 `governance.py`、`validation_tickets.py`、`integration_candidates.py` 和 `manifest_retention.py` 等隔离模块；仅在接口接线阶段修改当前已脏的 `sessions.py`、`cargo_runner.py`、`baselines.py`、`failures.py`、`git_finalize.py`、`migrations.py`、`work_continuations.py` 与 `server.py`。若 lease 不可得，生成 delayed patch 并继续未冲突切片。
- coordinator 的状态查询、validation 提交、Cargo 排队和数据库维护均只允许单次有界请求或 durable/offline receipt；实现和测试不得通过 sleep、轮询或额外 Cargo 批次等待它们完成。

## M1：状态收敛、孤儿与会话卫生

**目标：** 在不停止真实 Cargo、不中断其他可执行 Session 的前提下，先让 `active`、`resolving_failure`、`waiting_validation` 和 `stale` 投影重新反映真实资源，并修复当前 Coordinator01 的 lifecycle orphan-recovery Failure。

**依赖：** 现有 Session、Cargo job/run、reservation、supervision lifecycle 和 FailureGraph；M1 是所有后续数据库维护和 WIP 门禁的前置条件。

### 实施切片

- [ ] **M1.1 先关闭 lifecycle orphan-recovery 的最低共享 Failure。** 在 `tools/session_coordinator/supervision/lifecycle.py`、`tools/session_coordinator/supervision/service.py`、`tools/session_coordinator/server.py` 与 `tools/session_coordinator/sessions.py` 中实现类型化、幂等的启动期 orphan-lifecycle recovery；maintenance hold 下仅允许该恢复事务，普通 mutation window 仍关闭。确保 stale Session 注册失败时整行回滚，显式 resume 与 mutation admission 一致，普通 drain 不创建持久 hold。更新 `test_supervision_service.py`、`test_supervision_actions.py`、`test_sessions.py`，并在上游复现通过后以 `failure return` 回传 `failure-2026-07-16-lifecycle-orphan-recovery-maintenance-hold-integrity-deadlock.md`。
- [ ] **M1.2 建立受审计的状态收敛 preview/apply 服务。** 新建 `tools/session_coordinator/governance.py` 和 `test_governance.py`；在 `migrations.py` 新增单调 schema migration，持久化 preview 指纹、候选摘要、apply/skip/conflict 计数与操作者。服务在读阶段观察 Session、lease、patch、Cargo、reservation 和说明文件，在短事务阶段按原状态/heartbeat/终态作 CAS；禁止以直连 SQL 或文件删除替代服务接口。通过 `server.py` 组合并在 `cli.py` 增加 `governance converge preview/apply`。
- [ ] **M1.3 收紧 Session stale/archive 的真实资源保护。** 在 `sessions.py` 中保留现有单行 `mark_stale`/`archive_stale` 语义，但把活动 lease、非终态 patch、`pending/leased/running` reservation、活跃 Cargo 和非终态 source-copy 都纳入候选和 CAS 防护；旧 `waiting_validation` 按开放 Failure 恢复为 `resolving_failure`，否则恢复为 `active`，验证工作移交给 M2 ticket。扩展 `test_sessions.py` 覆盖 active/resolving/stale 迁移、CAS 竞争、24 小时归档和资源保留。
- [ ] **M1.4 统一 orphan run 与 reservation 的安全终态。** 在 `cargo_runner.py` 扩展 `reconcile_terminal_runs`：当父 job 为 `orphaned` 且 `CargoJobService` 最新 PID-tree 观察为空时，关闭 run 并记录专用 orphan-reconciled 错误码；当父 job 已 `released`、PID tree 为空但历史 exit code 缺失时，关闭 run 并记录 `cargo_run_reconciled_from_released_job_missing_exit_code`，仍不伪造成功。`cargo_jobs.py` 与 `cargo_reservations.py` 复用现有 FIFO/TTL reconciliation，处理两个已知形态和无 job 的失效 reservation，同时保留真实 running tree。扩展 `test_cargo_runner.py`、`test_cargo_jobs.py`、`test_cargo_reservations.py`。
- [ ] **M1.5 受管退役过期会话说明。** 由 `governance.py` 在 preview 中关联根目录 `.codex/sessions/*.md` 与新鲜非终态 Session 或材料化协调警告；apply 仅把已验证无 owner 的说明移动到 `.codex/sessions/archive/` 并写 `completed/archived`，归属不明的说明只报告。把路径授权、delayed patch 和移动失败回滚覆盖写入 `test_governance.py`。

### 测试阶段 M1-T

在临时 SQLite 数据库和临时说明目录运行 `test_governance`、`test_sessions`、`test_cargo_runner`、`test_cargo_jobs`、`test_cargo_reservations`、`test_supervision_service` 与 `test_supervision_actions`；再在生产数据库 backup 副本执行一次 converge preview/apply。调试顺序固定为 lifecycle recovery → 真实 PID tree → reservation → Session CAS → 说明退役。不得启动 Cargo。

**退出证据：** 两个指定 orphan run 已安全终态化；一个真实 running tree 不变；无 job 失效 reservation 已过期；过期说明已受管归档；生产服务能启动并保持普通 mutation 准入；M1 的回归 Failure 已按 canonical `fixed-*` 返回。

## M2：非阻塞验证、Failure 回写与编译优先集成

**目标：** 把等待从 Session 状态中拆出，令已拥有的连贯源码快照通过最小编译门槛后自动进入 `main`，完整测试与 Failure 修复在后台追认。

**依赖：** M1 的真实状态投影；既有 `workspace_copy.py` 的异步物化、`offline_queue.py`、FailureGraph、work continuations、lease/attribution 和 Git finalizer。

### 实施切片

- [ ] **M2.1 增加不可变 validation ticket 与 integration candidate 账本。** 在 `migrations.py` 增加 validation ticket、候选快照、compile 证据、pending validation link、事件和状态索引；新建 `validation_tickets.py` 与 `integration_candidates.py`。ticket 明确区分 `queued/materializing/running/passed/failed/snapshot_stale`，candidate 明确区分 `integration_ready/integrated_validation_pending/accepted/delayed_merge`。请求时固定 owner scope、blob/source manifest、命令、工具链、feature 和覆盖契约，密封前变更必须变为 `snapshot_stale` 而非静默换用新文件。
- [ ] **M2.2 让提交和物化在 deadline 内返回 receipt。** 在 `workspace_copy.py`、`offline_queue.py`、`client.py`、`cli.py`、`server.py` 与新 ticket 服务中接入 idempotent request id：健康服务持久化 ticket/candidate 后立即返回，繁忙/维护/短时离线时写本地 queue 并返回 queued receipt。后台 worker 复用相同 manifest/command/toolchain/coverage 的执行，拒绝不相同或覆盖无法证明的混合；任何 caller 不轮询 reservation、status 或 worker。
- [ ] **M2.3 把验证结果回写为可执行 continuation。** 在 `work_continuations.py` 移除 `waiting_validation` 对业务 Session 可执行性的依赖，结果事件以一次性 wakeup/continuation 回送。扩展 `failures.py`、`failure_fixture.py` 和 `.codex/skills/zircon-project-skills/handle-plan-failure-handoffs/scripts/validate_plan_failure_handoffs.py`：仅带 `failure_scope: local` 的同 Plan `failure-*` 可省略 self-edge；跨 Plan 仍严格使用现有 handoff。coordinator 对 ticket 失败以 commit/manifest/ticket/failure boundary/fixing Plan 幂等物化或更新文件；lease 冲突先持久 Failure graph/continuation，再排 delayed patch。扩展 `test_failures.py`、`test_failure_closeout.py`、`test_milestone_failure_scope.py`、`test_workflow_projections.py` 与 `test_validation_tickets.py`。
- [ ] **M2.4 实现精确快照的 compile-first finalizer。** 在 `integration_candidates.py` 和 `git_finalize.py` 以 candidate blob、base SHA、owner/lease 证据和最小 compile ticket 构建隔离 index/tree；通过后自动提交旧 blob，不触碰候选之后的工作树编辑。HEAD 前进且不同路径时重建候选；同路径外部变更转为 delayed merge；普通测试失败仅阻止 `accepted`，不阻止 `integrated_validation_pending`。后续失败绑定原 SHA 并默认 forward-fix，除所有权/编译证据失真或不可逆破坏外不自动 revert。扩展 `test_git_finalize.py`、`test_git_finalize_tracked_ignored.py`、`test_workflow_commit.py` 与 `test_integration_candidates.py`。

### 测试阶段 M2-T

用临时数据库、fake materialization/compile runner 与临时 Git 仓库运行 `test_validation_tickets`、`test_workspace_copy`、`test_offline_command_spool`、`test_deferred_action_client`、`test_failures`、`test_failure_closeout`、`test_workflow_projections`、`test_git_finalize`、`test_git_finalize_tracked_ignored`、`test_integration_candidates`。覆盖两秒 receipt、离线重放、去重/拒绝合并、无轮询 continuation、local/cross-plan Failure、worktree 后续编辑保留、HEAD 前进/冲突和 forward-fix。所有 compile 结果由 fake runner 证明控制流；不新增 Cargo 测试。

**退出证据：** 同一 primary 在 validation pending 时仍可继续工作；compile-pass/test-pending 候选在临时 Git 仓库自动精确提交；测试失败产生一次 canonical Failure 与 owner continuation；未知/冲突路径不进入提交。

## M3：Manifest 保留、归档与数据库 compact

**目标：** 将 930 MB baseline manifest 与 234 MB validation-copy manifest 的历史完整负载移出 SQLite，同时保留运行时引用、审计和可恢复性。

**依赖：** M1 的稳定资源投影、M2 的不可变 manifest/ticket 引用；现有 `baselines.py`、`validation_copies.py`、`workspace_copy.py` 和数据库连接。

### 实施切片

- [ ] **M3.1 增加 manifest 归档元数据和候选服务。** 在 `migrations.py` 为 `baseline_epochs`、`validation_copies` 和 retention batch 增加摘要、归档路径、归档时间、状态与索引；新建 `manifest_retention.py`。候选选择必须保留最新 baseline、所有非终态 Session 的 baseline、七天窗口、所有非终态 validation copy 和未完整验证归档的记录。现有运行期读取继续使用完整 manifest，直到退休事务完成。
- [ ] **M3.2 实现可恢复的 backup/archive/retire 事务。** 在 `database.py` 提供 SQLite backup API 封装，在 `manifest_retention.py` 写入临时 gzip JSONL、逐项复读 SHA-256/条目数/字节数、原子改名，再在单事务内把主表 manifest 缩为安全空对象并填摘要/归档引用。磁盘空间、backup、写入、复读或 CAS 任一失败均保留原 manifest。用 `baselines.py`、`validation_copies.py` 与 `workspace_copy.py` 接入只读恢复/summary 路径，禁止普通扫描触发归档 I/O。
- [ ] **M3.3 实现非阻塞 compact 维护窗口。** 在 `manifest_retention.py`、`server.py`、`cli.py` 与 `offline_queue.py` 增加 governed compact action：先备份和 `quick_check`，后台执行 `VACUUM`，期间保持 health/Cargo，暂停 stale 时钟，mutation 写 offline queue 即返 receipt，只读返回 `database_maintenance_active` retry hint。完成后复核 schema、关键计数、摘要与大小并幂等重放；失败保留 backup 和恢复证据。

### 测试阶段 M3-T

在复制数据库运行 `test_manifest_retention`、`test_baselines`、`test_validation_copies`、`test_workspace_copy`、`test_database`、`test_server` 和 `test_offline_command_spool`。覆盖七天边界、活跃引用、gzip/hash 失败、幂等 apply、compact 前后完整性、维护期间 mutation receipt、Cargo 不受影响和实际 SQLite 体积下降。不得用 Cargo 补偿。

**退出证据：** backup 副本内所有受保护 manifest 可读；已退休负载可由归档哈希复核；生产 preview/apply 后 baseline/validation-copy 主表负载显著低于 930/234 MB 基线，且 compact 前后 `quick_check` 与计数一致。

## M4：Plan 族群 WIP 与后继会话门禁

**目标：** 每个 normalized numbered Plan 族群最多一个可写 primary 和一个关联只读 reviewer，直到里程碑 acceptance 后才允许后继微会话。

**依赖：** M1 的 executable 状态语义、现有 `plans.py` owner 解析、Session 注册与 Failure routing。

### 实施切片

- [ ] **M4.1 将 Plan 族群和角色持久化。** 在 `migrations.py` 为 Session 增加 normalized plan-family key、role、parent primary、grandfather 标记和必要索引；在 `plans.py` 复用 numbered owner 解析而非名称前缀；在 `sessions.py` 的 register 事务中验证 reviewer 无 write scope 且属于同一族群。
- [ ] **M4.2 强制新建 primary/reviewer 上限并保留现有工作。** 在 `sessions.py`、`server.py` 与 `cli.py` 让新的 primary 在同族群存在 executable primary 时返回 `plan_wip_limit_reached` 和其 heartbeat；同一注册幂等不占名额，reviewer 仅一个。Failure 把现有 primary 转为 `resolving_failure`，不借新 Session 绕过门禁；老的超限 Session 只要有真实资源就 grandfather 到终态，无资源则由 M1 收敛。
- [ ] **M4.3 让 continuation、Failure 和集成状态服从 WIP。** 在 `work_continuations.py`、`failures.py`、`validation_tickets.py` 与 `integration_candidates.py` 中，validation pending、delayed patch、Failure repair 和 `integrated_validation_pending` 都回写现有 primary；完整 acceptance 才释放后继微会话资格。扩展 `test_plan_wip.py`、`test_sessions.py`、`test_workflow_attempts.py`、`test_milestone_failure_scope.py`。

### 测试阶段 M4-T

在临时数据库运行 `test_plan_wip`、`test_sessions`、`test_plans`、`test_failures`、`test_milestone_failure_scope`、`test_workflow_attempts` 与 `test_workflow_topology_testing_stages`。覆盖不同顶层 family 下独立编号 Plan、同族群 primary/reviewer、重复注册、Failure reuse、grandfather 和 acceptance 前后的后继 Session 拒绝/允许。

**退出证据：** 任何新会话均无法通过改名或 Failure micro-session 绕过同 Plan WIP；现有 primary 从 validation/Failure 继续实际工作；只有 acceptance 才打开后继主会话名额。

## M5：所有权矩阵与持续早期主干集成

**目标：** 从共享脏工作树中只挑选已有证据支持的 owned blob，持续形成小而可审查的 `main` 提交；未知、冲突或无 lease 路径保持原状。

**依赖：** M2 candidate/finalizer，M4 Plan 族群，现有 `baselines.py` attribution、`leases.py` 与 `git_finalize.py`。

### 实施切片

- [ ] **M5.1 生成审计式所有权矩阵和提交 preview。** 在 `integration_candidates.py`、`baselines.py`、`git_finalize.py`、`server.py` 与 `cli.py` 聚合 Session attribution、有效 lease、snapshot/commit manifest、当前 blob 与 base SHA，输出每一路径唯一 owner、缺失证据、冲突和可集成批次。输出只记录 hash/owner/原因；不读取共享暂存区作为所有权来源。
- [ ] **M5.2 队列化最小编译候选并自动 finalize。** 对每个连贯、唯一 owner 批次创建一次最小受影响 package/target/feature compile candidate；编译在 coordinator lane 后台执行，成功即按 M2 精确 finalizer 提交到当前 `main`。不为每个编辑创建候选，不运行 workspace Cargo test，不等候验证 ticket；存在同路径 HEAD 冲突时保留 delayed integration receipt。
- [ ] **M5.3 将验证失败和提交后状态回写。** 通过 M2 Failure materialize 将失败与原 commit、最低 fixing owner 和 repair continuation 关联；成功验证把 candidate 晋升为 `accepted` 并释放 M4 后继资格。会话说明退役作为独立协调卫生提交，绝不与业务源码混合。扩展 `test_integration_candidates.py`、`test_git_finalize.py`、`test_baselines.py`、`test_workflow_commit.py`。

### 测试阶段 M5-T

在临时 Git 仓库运行 M2 的 finalizer/ownership 测试批次，再对生产工作树只执行 preview。实际 `main` 集成由每个 candidate 自己的最小 compile ticket 决定；聚焦 Python 测试与完整验证保持异步，不增加任何 broad Cargo 任务。

**退出证据：** 每个生产提交都有唯一 owner、精确 blob、compile SHA、pending/accepted validation 和可审查 Conventional Commit 元数据；其它 Session 的脏文件、未知路径、共享 index 内容和 lease 冲突均未进入提交。

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
