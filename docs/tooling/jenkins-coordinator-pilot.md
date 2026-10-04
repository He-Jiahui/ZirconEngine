---
related_code:
  - tools/jenkins_pilot/
  - tools/jenkins_coordinator.py
  - tools/jenkins_tray/
  - tools/jenkins/install-coordinator-worker.ps1
  - tools/build-editor.ps1
  - tools/dev/dev-fast-build.ps1
  - tools/zircon_build.py
  - tools/build/zircon_build_cargo_environment.py
  - tools/zircon-session.ps1
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
implementation_files:
  - tools/jenkins_pilot/
plan_sources:
  - "user: 2026-10-01 isolated Jenkins coordinator pilot with retained source acceptance authority"
  - "user: 2026-10-02 keep the old service retired and establish independent Jenkins acceptance"
  - docs/plans/jenkins-coordinator-pilot.md
tests:
  - tools/jenkins_pilot/tests/
  - tools/session_coordinator/tests/
  - docs/plans/milestone-validation-policy.md
doc_type: workflow-detail
---

# Jenkins 独立验证与验收

> **当前方向：旧服务保持退役，Jenkins 独立验收。** 用户已于 2026-10-02 授权本方向。当前独立实现、两次真实 Jenkins Cargo 归档、Python、幂等/拒绝反例和四项故障试点验收均通过；范围与结果以[计划中的 I1–I6](../plans/jenkins-coordinator-pilot.md)和对应证据为准。本文不授予提交、推送、发布、删码或生产切换许可。

Jenkins 负责队列、节点、构建身份与归档；`tools/jenkins_pilot` 独立处理输入封存、物理存储边界、Windows Job、缓存排他写入和回执核验。该路径没有退役协调器的运行时导入、注册、租约、队列或数据库写入。所有编译产物及缓存实际位于驱动器根级 `D/E/F:\cargo-targets`；私有资源保管进程持有本根的物理身份和精确 PID/birth，仅管理资源生命周期。

## 当前独立入口

日常协调器功能测试使用 [Windows 托盘及激活入口](jenkins-tray.md) 和 [jenkins-coordination 技能](../../.codex/skills/jenkins-coordination/SKILL.md)，核验 `active.json` 后调用 `tools/jenkins_coordinator.py`，保留原请求完成提交、对账、取消和接受。下方低层入口用于独立根的部署及受控诊断，不自动获得全工作区、里程碑或生产迁移接受。

从仓库根运行以下顺序，并使用各步骤实际返回的 root、bundle 和 input hash。分配的 state 文件必须唯一；已有 pending 或 attempt 一律核对原请求，不能以换身份掩盖不确定状态。Windows Cargo 执行前在父进程装载本机 MSVC 环境，使保管进程与后续执行器继承相同环境。

```powershell
python -B -m tools.jenkins.pilot --repo-root <repo> allocate --state-file <repo>/.codex/state/jenkins-pilot/<unique>.json --prefix jenkins-pilot-<unique>
python -B -m tools.jenkins.pilot --root <root> --repo-root <repo> prepare --reuse-from <preserved-verified-asset-root>
python -B -m tools.jenkins.pilot --root <root> --repo-root <repo> start
python -B -m tools.jenkins.pilot --root <root> --repo-root <repo> seal --path <declared-input> --allow-new <explicit-new-file>
python -B -m tools.jenkins.pilot --root <root> submit --request-file <request-v2.json> --bundle <sealed.zip>
python -B -m tools.jenkins.pilot --root <root> reconcile --request-file <request-v2.json>
python -B -m tools.jenkins.pilot --root <root> --repo-root <repo> verify --request-file <request-v2.json>
```

`prepare` 只复制已核验 WAR、JDK 和声明插件的字节到新根；创建新的配置与凭据，保留原资产和历史。私有 root 的 namespace 仅由本资源 owner 写入；代码库存与现有字节句柄须在 Java 加载前核验并保留。该约束不声称隔离同一 Windows 账户的恶意并发写入。

管理员登录密码可与自动化凭据分开：私有 `credentials.json` 的 `authenticationMode: "api-token"` 表示 `password` 字段保存管理员 API token，供托盘和任务客户端认证。此模式下启动只核验现有管理员及 token，保留 Jenkins 已保存的登录密码；缺失或无效的 token 会阻止安全引导，不会改写登录密码。未指定模式的新实例仍使用生成的随机密码初始化账号。

启动器先在独立的 180 秒准备期限内核验实际 Java 启动标记，绑定本次 host/child 的原生 PID、出生时间、当前 keeper Job 和固定资产摘要；标记在保留资产句柄且真实创建 Java 后写入。确认标记后才开始 controller 原有的 120 秒 HTTP 就绪期限，agent 同样必须确认真实 Java。准备或就绪失败仍清理本次精确 Job，并保留记录和日志。

request schema 2 固定 `sessionId/requestId/attemptId/generation/inputHash/template`；模板仅有 `python-static`、`managed-cargo-check-v2` 和 `fault-probe`。Cargo 模板直接执行已安装工具链的 `cargo check --locked -p zircon_reflect_derive`，target、Cargo home、scratch、incremental 和可复用 generation 均在本根内，保持单 writer。兼容性绑定工具链、配置、锁文件与发现环境；未知终止状态阻止复用，禁止收养旧缓存或通过清理锁模拟释放。

Windows 的 generation 路径采用完整兼容性摘要的前 20 位，状态保留完整摘要并拒绝碰撞；执行前核查 MSVC build-script 输出长度，禁止以路径别名绕过存储边界。执行环境固定 `CARGO_HTTP_PROXY=""`，避免 Cargo 回落到机器的 Git 代理；不修改全局 Git 配置。该设置属于缓存兼容性合同，具体优先级见 [Cargo 官方配置文档](https://doc.rust-lang.org/cargo/reference/config.html#httpproxy)。

内部 baseline 可从本独立根的已释放 generation 复制 registry index/cache；复制期间持有双方 writer 锁并记录逐文件摘要，不复制编译产物、registry source、锁或旧状态。绑定请求的 `registry-seed.json` 会随随后复用的构建归档，`verify` 要求归档、当前 run 与 generation 的证明字节及摘要一致。该选项不开放给 Jenkins 请求参数，不能用于收养退役协调器缓存。

Cargo 在 `build` 子树中正常生成内部硬链接。专用原生审计必须枚举所有 NTFS 链接名，逐一证明属于本 generation 的 `build`，卷和完整文件身份相同；外部链接、不完整枚举或链接到控制文件均拒绝。源码、registry、状态和回执仍要求普通单链接文件。

若编译已退出而终态审计失败，原回执继续为 failed、缓存不得直接复用。内部 `terminal_reconcile` 只处理确切原请求：重新核验原生终态、输入与原封存 driver、物理身份及独占 writer，再新增不可变释放证明。它不改写原失败记录、不授予验收；后续真实执行与 Jenkins 归档仍须独立通过。

`verify` 必须再次读取确切 Jenkins build 的终态和全部请求参数，并核验归档、封存源码、固定驱动器、真实退出码、原生 Job0、stdout/stderr EOF 以及 Cargo 释放证据。通过结果为 `accepted: true`、`acceptance: jenkins-validation`、`scope: sealed-input`。它只验证固定输入，不自动授予里程碑或生产迁移验收；缺档、旧 attempt、漂移、取消或未知资源状态拒绝通过。故障探针只提供资源收敛证据。

执行交接等待始终计入原请求的观察期限，取得排他锁后重新校验当前 owner、封存 driver 与确切 Jenkins build。准入、冲突扫描或源包核验耗尽期限后，禁止发布新的执行意图；已有 staged 输入及原意图保留。保管进程的历史完成审计不占用交接锁，忙时在后续轮询继续核对。

`cancel` 只操作确切本任务 queue/build；操作后继续对账原请求及独立持久化回执。`stop` 只停止本根 controller/agent；保管进程与历史继续保留。旧服务恢复、旧数据库写入、目录删除及历史抹除均不属于此流程。

独立保管进程异常退出后，正常 `allocate` 禁止收养已有根。专用 `governance_recovery.inspect_recovery`/`recover_storage` 要求显式原所有权文件摘要、确切原生死亡证明、未变化的根与原状态身份、资源摘要、完整历史终止回执和可取得的 writer 锁；未知旧请求拒绝恢复。恢复使用唯一新 state 和不可变历史链，只替换当前所有权指针，原 state/pending、回执、锁和缓存状态保留。多次恢复逐层核验每个原 state 的字节、完整物理身份与封存证明；历史完成记录须在每个中间 checkpoint 中完全一致。PID 被复用时，只以原生出生时间不同证明原身份已结束；查询或等待权限不足不能证明确切原进程已退出。历史已完成意图仅在原证明字节匹配时跳过，不重新投递；新任务使用新的所有权版本。后续查询旧 state 明确拒绝为 superseded，必须使用恢复返回的新 state。启动日志与 fatal 回执保留以便诊断；禁止把无日志的退出推断为某次改动造成。

## 历史试点协议与证据

以下记录保留原时点语义，其中租约、正式协调器及旧验收接口已经退役。历史命令禁止执行，也不能用于授予当前独立验收。

## 历史部署与执行边界（禁止恢复使用）

原隔离试点使用了本机 loopback Jenkins 2.580.1、Java 21。controller 无执行槽；一台 Windows WebSocket inbound agent 只有一个执行槽。试点不迁移现有服务，不停止其他 Session，不复用其他 Session 的执行目录。

所有 controller、agent、workspace、日志、归档、Cargo target 和缓存必须真实位于驱动器根级 `D:\cargo-targets`、`E:\cargo-targets` 或 `F:\cargo-targets` 下，并持有活跃 `artifact.fixture_acquire` 租约。目录名符合策略不等于持有租约。旧自定义根曾被现有 unmanaged GC 部分清理，不能继续使用或作为恢复来源。当前试点根由操作者提供；不要把一次运行的 PID 或 fixture 名当作部署默认值。

历史步骤（当前禁止执行）：原流程先执行 `python -B -m tools.jenkins.pilot --repo-root <repo> allocate --state-file <repo>/.codex/state/jenkins-pilot/<unique>.json --prefix jenkins-pilot-<unique>`，取得现有协调器分配的 root，再把该 root 传给后续命令。独立隐藏 keeper 持有 fixture 租约和 controller/agent 的 kill-on-close Job；keeper 死亡会收敛其 Job，死 owner 的本地 lease 文件不能作为准入依据。`stop` 终止本次运行，keeper 继续保管归档；只有明确获准移除已停止的目录后才释放 fixture。分配处于 reserved/uncertain 时先核实原 keeper 和协调器记录，不盲目重试。

`snapshot.py` 封存固定 base commit、变更/新增文件 payload、删除条目及整个声明 scope 的 digest。执行从封存输入还原，不从继续变化的共享 checkout 读取源文件。新增文件必须显式允许。`journal.py` 绑定 session、request、attempt、generation、input hash 与 bundle hash；重试、取消和失联恢复沿用已有记录，不能生成一个新身份来掩盖未确定状态。

显式外部 sibling 输入使用 schema 2：`external_snapshot.py` 将每个外部仓库的 canonical mount、固定 base commit 和已选择条目纳入同一 input digest；所有未删除外部文件均带完整 payload，包括未修改的 base 文件。重建不依赖执行时的 sibling checkout 或 Git objects。外部输入必须是主仓库的直接 sibling Git 根，不能跨 symlink/reparse point；planner 的逐项 digest 必须匹配，封存前后重读以拒绝漂移。还原到全新的 sealed parent，保持相对布局，并验证所有外部文件及额外/缺失目录。schema 1 继续用于无外部输入的封存。

实际 `cargo-closure-v2.json` 封存主仓库 108 项与 `zr_vm` 三项（含 `Cargo.toml`），bundle/input hash 前缀为 `549f7a`/`7acfd804`，完整 hash 以 manifest/journal 为准。metadata 在 25.8487 秒完成，所属 native Job 活跃进程为零。这是依赖发现与封存证据，不能替代 Cargo check。

`pipeline.groovy`、`runner.py` 只执行固定 Python、managed Cargo 和 fault 模板。Windows Job Object 负责本次任务的进程树边界。回执校验必须覆盖命令 hash、执行前后源码 hash、真实工具链、终止证明、耗时和归档文件大小/hash；归档丢失、源码漂移或旧 generation 都不能成为通过证据。

`lifecycle.py` 以每物理根的 Windows mutex 串行化 prepare/start/stop，启动记录使用原生受管存储的唯一临时文件原子替换。活跃 legacy 进程没有 keeper Job 证明时拒绝收养或 PPID 清理；不能用根 PID 退出推断整个任务树已收敛。

Cargo 输入通过独立 CLI `python -B -m tools.jenkins.pilot.closure --root <root> --repo-root <repo>` 封存，固定声明为 `cargo check --locked -p zircon_reflect_derive`。原 `plan_live_inputs` 继续负责完整 metadata、依赖闭包、配置与源码指纹；试点只在该独立进程内替换私有 metadata transport，使用从创建时归属的原生 Job、完整双管道捕获和原 180 秒窗口，最后恢复绑定。此过渡 seam 不修改运行中的协调器或共享源码。原 `processes.py` 的 descendant 终止只依据 PPID，尚缺 retained-parent/child birth ordering，不能作为可信 fallback；正式抽取薄层前需要公开窄 transport 接口并修复该共享支持层。

## 源码职责与退役依赖

以下是职责 inventory，不是批准删除清单。2026-10-01 静态枚举 `tools/session_coordinator/**/*.py` 得到非 `tests` 路径 263 文件、155,153 行，`tests` 路径 213 文件、107,149 行；计数包含空行与注释，不含 PowerShell、前端或其他工具。这些源码含大量正式验收、恢复与治理逻辑，不能把整个协调器的行数当作可删除量。迁移需要先拆开同一文件中的执行机制与项目策略。

| 职责与真实路径 | 后续候选处理 | 删除前必须解除的依赖 |
|---|---|---|
| `validation_batch_scheduler.py`、`validation_ticket_worker.py`、`local_executor.py`、`local_input_slots.py`、`validation_group_runner.py` | Jenkins 接管排队、槽位和通用执行循环后，退役重复执行机制 | `server.py` 创建 `ValidationTicketWorker`；`coordinator_services.py` 配置加速 scheduler、物理输入池、before/after-run hooks。必须保留 scope 准入、票据归因和 terminal 回调语义，先切这些 composition 点。 |
| `durable_tasks.py`、`durable_task_schema.py`、`validation_priority.py`、`validation_groups.py` | 任务分派、优先级和通用 attempt 调度可迁移；身份与验收数据继续保留 | `coordinator_services.py` 将 `durable_tasks.accepted_validation_attempt` 注入 `validation_tickets`。`durable_tasks.py` 读取票据封存、copy/run-link 事件并校验 remote terminal proof；不能直接删除 service 或历史表。 |
| `remote_transport/`、`remote_worker/`、`worker_protocol/`、`remote_task_output.py`、`install-coordinator-worker.ps1` | Jenkins agent/WebSocket/日志归档替换通用 worker 传输和安装职责 | `server.py` 导入 `RemoteTransport`；同一 listener 同时承载控制面、SSE、worker WebSocket、对象传输。`coordinator_services.py` 连接 WebDAV、artifact relay、log sink，并把 `worker_protocol.accepted_remote_attempt_terminal_proof` 注入 `durable_tasks` 与 `validation_reuse`。先提供可信终止证明替代和控制面拆分。 |
| `web/` 与 `control_plane/tasks/` 的队列/worker展示 | Jenkins 替代重复构建队列、控制台日志和构建历史 UI | `control_plane/` 还包含鉴权、动作权限、Session/租约/Failure/正式验收操作。保留这些业务页面/API或为其提供等价入口；不能把控制面目录整体删除。 |
| `snapshots.py`、`compile_workspaces.py`、`validation_copies.py`、`validation_copy_cargo.py`、`validation_ticket_inputs.py`、`validation_ticket_identity.py` | 保留封存、归因、复制后校验；可抽取成薄层 | `ValidationTicketWorker` 的 copy executor 契约包含 sealed overlay、baseline commit、外部依赖和 copy cleanup。试点 `managed_cargo.py` 直接导入 `compile_workspaces.manifest_digest/tree_manifest`。Jenkins checkout 插件不能替代未提交源码契约。 |
| `validation_tickets.py`、`validation_ticket_policy.py`、`validation_reuse.py`、`artifact_receipts.py`、`integration_candidates.py` | 保留正式验收、复用判定和 integration 证据归因 | 通过必须匹配当前 attempt/generation/input、命令、source、产物和进程终止证明；Jenkins build number 仅是执行历史关联键。 |
| `cargo_jobs.py`、`cargo_runner.py`、`cargo_pipeline.py`、`reserved_starts.py` | 通用启动/排队机制后续可替换；Cargo admission、环境策略和分阶段证据保留 | validator 仍调用 `cargo_pipeline.py`，注册 Session/Cargo job 并写 managed metrics。必须先把状态/receipt 查询切到等价契约，再解除 `server.py` 的 runner、job service composition；不能直接绕过现有 guarded Cargo。 |
| `cargo_storage*.py`、`cargo_target_*.py`、`cache_budget.py`、`build_policy.py`、`storage_ledger.py`、`artifact_governance.py`、`artifact_fixture_liveness.py`、`low_disk_gc.py` | 保留或抽取存储/缓存治理薄层 | Jenkins workspace/归档保留策略不能识别项目活跃 fixture、target compatibility、实际磁盘路径和租约；继续沿用物理路径与存储 owner 校验，活跃对象不能被 GC。 |
| `processes.py`、`windows_job_process.py`、`leases.py`、`command_requests.py`、`record_recovery.py` | 保留身份、锁、幂等回执和恢复职责 | executor 槽空闲、Jenkins ABORTED 或 HTTP 超时均不能证明子进程终止；失联后先恢复同一 request，再释放对应资源。 |

`coordinator_services.py` 是首要切换点：它同时连接执行器、存储 ledger、worker proof reader、正式 ticket accepted-attempt reader 和 terminal hooks。抽取薄层必须按这些真实接线分离，避免让 Jenkins 的状态投影成为新的验收权威。

静态 import 反向扫描还定位到容易遗漏的调用者：`coordinator_hooks/managed_validation_lifecycle.py`、`coordinator_hooks/validation_source_hooks.py` 与 `control_plane/tasks/service.py` 依赖 `durable_tasks`；`managed_probe_snapshot.py` 依赖 scheduler/durable tasks；`validation_group_materializer.py` 依赖 scheduler/local executor；`live_compile_inputs.py` 依赖 Cargo pipeline/validation copies；`pinned_cargo_planner.py` 与 `workspace_copy.py` 依赖 validation copies；`record_recovery.py` 依赖 validation tickets。删码批次必须包含这些调用者的切换与相应回归，而不只修改主 server。该扫描只确认静态引用，不宣称覆盖动态 import、SQL、HTTP 或全部脚本调用。

## 构建入口必须同步切换

| 调用入口 | 当前依赖 | 切换要求 |
|---|---|---|
| `tools/build-editor.ps1` | `.codex/skills/zircon-dev/scripts/validate-matrix.ps1`、`tools/zircon-session.ps1`；按 package 发起 managed build | 保留两次 package 构建各自的归因、storage policy 和失败返回；替换后需要完整入口回归。 |
| `tools/dev/dev-fast-build.ps1` 及对应 `.cmd` 包装 | 正式 `validate-matrix.ps1` 与 compatibility pool/sccache 策略 | 新执行桥必须继续执行同一存储准入与环境策略，不能转成裸 Cargo。 |
| `tools/zircon_build.py` | `zircon_build_cargo_environment.py` 的 root guard 与 Cargo 环境；产品 staging/输出契约 | 即使调度迁移，output/engine/targets/cache 物理根和产品清单语义仍保持。 |
| `validate-matrix.ps1` | Session 注册、`zircon-session.ps1`、managed compile workspace、`development_artifacts.py`、`cargo_pipeline.py`、metrics receipt | 同步切换 job admission、环境 lease、source digest、执行终止与 receipt 查询。保留非批准根、路径别名和失配源码的拒绝。 |
| `tools/jenkins_pilot/managed_cargo.py` | 固定 `zircon_reflect_derive` check；正式 validator；`compile_workspaces`；`cargo list --history` | pilot 不绕过正式入口。executor manifest 固定 validator 脚本、coordinator 非测试 Python、顶层 `.psm1` 与 wrapper；运行期间改动这些文件会拒绝证据。 |

修改共享 guard 或 coordinator 源码后必须生成新封存身份并重跑受影响 gate。不能把旧通过结果绑定到新实现。退休代码关联的测试先分类为执行机制测试、保留契约测试与历史读取测试；保留契约测试在薄层中继续运行，不能为了减少行数删除验收断言。

## Drain、cutover 与 rollback

### G7 前瞻切换设计（待实现，未获正式迁移授权）

下面是下一阶段的可审查设计，不是当前可调用命令。`DispatchRouter`、`WorkerEvidenceAdapter`、`LegacyHistoryAdapter`、拟议 `/control/pilot-dispatch/*` 与 `/control/pilot-history/*` 均不存在；实施、服务切换和删码必须另行获准。当前 263 个非测试 Python 文件仅是 inventory，不是可删除数量。

| 批次 | delete / extract / retain 的具体符号 | 调用点与 replacement 契约 | 独立接收边界 |
|---|---|---|---|
| B1 旧 remote worker client 分发包，首个可独立退役批次 | **delete** `remote_worker/client.py::WorkerClient.connect/run/receive_once/handle_frame/_heartbeat_loop/_log_sender_loop` 及 wire queue/reconnect 机制；切换完成后删除 `WorkerClient`、`WorkerConnectionConfig`、`WorkerStatus` 与旧 client 文件。**extract** `_result_with_final_log_cursors/_final_log_fields/_find_persisted_artifact/_handle_artifact_message/_download_object/_upload_object` 的身份、最终日志、范围/hash/归档完整性校验到拟议 `WorkerEvidenceAdapter`，不保留旧 WebSocket 发送机制。**retain** `remote_worker/execution.py::SealedTask/WorkerExecutor`、`providers.py::register_builtin_providers` 与全部项目策略。 | `remote_worker/cli.py` 直接 import 三个 client 类型；其 `_serve`、`_run` serve/capabilities/status 分支与 `_build_config` 同批切到 Jenkins node/健康查询或移除旧包入口，`remote_worker/__main__.py` 与 `install-coordinator-worker.ps1` 安装/打包入口同批更新。新 dispatch 调用已存在 `JenkinsClient.submit/reconcile/cancel/receipt`；Jenkins agent 取代 connect/heartbeat，日志与 artifacts API 取代旧 object 上传。`WorkerEvidenceAdapter` 接收 `SealedTask.result_identity`、最后日志 cursor、原始 artifact bytes 与 native terminal proof，经现有 acceptance gate 处理。 | 先登记所有旧 worker 安装、CLI、打包与动态 import 消费者；有未迁移消费者则不得删除文件。全部属于获准 B1 的旧节点完成 drain，历史 artifacts 可读取，取消/失联/重放/缺档合同通过后退役该 client 分发包。正式 listener、Cargo executor、providers 和验收服务继续运行；没有解除依赖的远端对象服务不删。 |
| B2 本地 validation 调度与批合并，pending | **delete 候选** `ValidationTicketWorker.tick` 中新 claim/启动循环、`LocalExecutor._run_loop/_dispatch/_start_claimed_attempt`、`ValidationBatchScheduler.pump/_execute_group` 的排队/launch 机制。**extract** `ValidationTicketWorker._manifest_drift/_run_evidence/_record_terminal`、`LocalExecutor._verify_materialization/_verify_manifest_receipt/_validate_provider` 与 scheduler `_validate_candidate/_verify_ticket/_verify_probe_outcome` 策略到保留薄层。**retain** ticket/source/pin、reuse/group 合同和 typed diagnostics。 | `server.py::CoordinatorApplication.advance_validation_queue`、`RunningCoordinator._run_validation_worker_tick`、`CoordinatorUpgradeServices._configure_validation_acceleration`、`coordinator_hooks/local_validation_runtime.py::LocalValidationRuntime._build_local_executor` 必须切到 router 的 claim gate 与保留 evidence adapter；`coordinator_services.install` 继续绑定 `set_accepted_attempt_reader(durable_tasks.accepted_validation_attempt)` 和 terminal hooks。 | 必须补齐所有 local/managed probe、批合并与 continuation 消费者的逐符号清单，验证分离后继续收敛 active attempts。当前未完成此分析，B2 不批准整文件删除。 |
| B3 Cargo 启动、传输服务与 UI，pending | **retain** `CargoJobRunner.toolchain_identity/_managed_cargo_environment/status/reconcile_terminal_runs`、`ValidationCopyCargoExecution.advance`、Cargo storage/environment/source/receipt guard。启动机制、worker server routes 和 UI 是否删除须逐符号再审。 | `server.py` 的 `CargoJobRunner` composition；`coordinator_services.install/bind_transport` 的 accepted proof reader/result publisher；`validate-matrix.ps1` 与 `managed_cargo.py` 的正式 job/metrics 查询不能直接改成 Jenkins SUCCESS。`providers.register_builtin_providers` 同时被主 composition 使用，不能跟 B1 client 一起删。 | 实际 Cargo/缓存/入口验证未完成。后续 map 必须涵盖 `durable_tasks`、`worker_protocol`、控制面鉴权和产品入口；未梳理者保留。 |

B1 的 review 交付物必须逐一列出旧 client exports 的所有 import/动态加载/打包引用及其删除或新 owner；上述 extract 符号只能在新 adapter 的合同测试通过后迁移。对旧 server 回传的 evidence DTO 和 Jenkins archived receipt 的差异做显式版本转换，不通过字段名猜测兼容，不让归档验证器取得正式验收权。

此 G7 设计已由 root 与独立 review 审查，无 P1/P2，证明 `evidence/retirement-design-independent-review.json`（SHA 前缀 `c343927`）。只证明设计可审查；没有 cutover、module 退役或维护量改善，所有拟议入口仍待实现。

`remote_worker/cli.py` 还 import `WorkerClientError`；B1 须把其 error DTO 与 CLI 参数/健康错误转换迁到 adapter 的独立错误类型并更新引用，不能留下指向已删 client 的 import。CLI 的 `_prepare_vsdev_environment` 与 provider 注册属于执行准备，继续保留供新 node runner 使用；`_serve/_run` 不能连同这些职责一起抹掉。现有 worker protocol 服务端的 pairing/drain/terminal proof reader 在 B1 保留供未迁移节点及历史归因，移除其 routes 属于尚未展开的 B3。

### 拟议 dispatch router 和收敛判据

router 是正式准入薄层的一个持久记录，每个获准 task family 独立配置。拟议 schema `dispatch_route_v1` 包含 `repoKey/family/state/epoch/operationId/expectedPreviousEpoch`；拟议认证动作 `POST /control/pilot-dispatch/operations` 使用现有 runtime 身份、权限 preview/confirm 与 `CommandRequestJournal` 幂等边界，`GET /control/pilot-dispatch/status?family=...` 返回 epoch、owner 和在途证据计数。这两个 endpoint 仍待实现，不得发送到现有服务并认为已生效。

状态机为 `old-only -> draining -> jenkins-only -> rollback-drain -> old-only`。`old-only` 仅旧 claim 可新派发；`draining` 同时禁止该 family 的旧与 Jenkins 新 admission，仅继续已有 materialize/run/reap/reconcile；`jenkins-only` 仅 Jenkins 可新派发；`rollback-drain` 同时停止两端新 admission并对账 Jenkins 已绑定请求。state/epoch 变更在同一 writer 内 compare-and-swap，operationId 重放只返回原操作结果，跨 epoch 更新拒绝。

未来 dispatch binding 的唯一键为 `(repoKey, family, originalRequestId)`，保存 input/bundle hash、attempt/generation、epoch、chosenBackend 和旧 job 或 Jenkins queue/build ID。先持久 reserve 后网络 submit；后端不确定、timeout 或 queue 404 只可 reconcile 原 binding，不能重新 original submit、改 originalRequest 或切另一个 backend。执行端和结果端均核对 epoch/attempt/generation；旧 epoch 的迟到 receipt 只能归档为历史，不能关闭当前 generation。JenkinsClient/SubmissionJournal 现有幂等检查可复用，但不等于已实现跨旧/Jenkins 的唯一锁。

**需新增的 drain 控制入口**是 router 的 family claim gate，而不是停止正式 daemon。它必须进入 `ValidationTicketService.claim_next`、`LocalExecutor._dispatch` 与 scheduler `pump` 的新 claim/start 边界，并进入 `CoordinatorApplication.advance_validation_queue` 的手动路径和 `_run_validation_worker_tick` 的后台路径；后台当前 `require_admission=False`，不能只拦手动 API。把 tick 拆为 claim 与 active-reconcile 后，draining 仍执行 `_advance_materializing/_advance_running/_finish_from_run`、`CargoJobRunner.reconcile_terminal_runs` 和结果接收。`LocalExecutor.stop/close/reset`、`CoordinatorUpgradeServices.stop` 会停 polling/关闭资源，不能当作安全 drain 开关使用。

在途集合按 family/epoch 包含 reserved/uncertain/queued、尚未 claim 的旧 eligible tickets、materializing、pending provider futures、running attempt、取消未获 terminal proof、待上传/待验档/待正式对账结果，以及执行关联 slot/target/input/process lease。cutover/rollback 的零判据是：旧端与 Jenkins 相应集合各为零、无未定 submission；所有 bound native PID+birth 的 Job terminal，reader EOF/产物闭合已证实；所有结果 receipt 已校验或明确拒绝并归档；每个旧 request/job 与 Jenkins queue/build 均可在历史 adapter 查询；仅本操作拥有的 execution lease 已正常释放。保留历史/fixture/其他 Session 的长期租约不要求为零，也不能为满足计数清除它们。候选计数与 epoch 在最终 writer 内再次复核后才切换。

取消只针对获准 family 的精确 request/attempt：当前 `TaskControlCommands.task_cancel` 优先 `DurableTaskService.cancel_queued`、fallback `cancel_legacy_validation`，不提供通用 running-task kill；运行取消的新桥仍待实现。Jenkins 侧复用 `JenkinsClient.cancel` 的已核验 queue cancel/build stop，并继续 `reconcile` 和 native proof 对账。ABORTED、executor idle 或 task.cancel 返回都不是零在途证明。未识别进程 owner、失联 agent、无法查到旧回执或超时 drain 一律停留 draining，不擅停旧 validation/Cargo/fixture 服务或杀其他 Session。

在 `jenkins-only` 发现双分派、来源/epoch/command/receipt 不一致、无可信 terminal proof、正式 guard 被绕过、history 查询损坏或持续不可用超过批准的 operation deadline，进入 `rollback-drain` 并禁止新 admission。不得立即恢复 old-only；必须满足上述零判据，再以同一 operation 的新 epoch 恢复旧路由。无法收敛保持阻塞并要求操作 owner 决策。B1 退役前保留旧可部署配置/包和验收 API；删除包后的恢复必须使用已封存旧版本，不能悄悄重建新身份。deadline 与授权范围由未来操作显式提供，当前不设自动执行策略。

### 拟议只读历史 adapter

schema `pilot_history_v1` 的每条记录保留 `repoKey/origin/originalRequestId/legacyCommandRequestId/ticketId/jobId/runId/jenkinsJob/queueId/buildId/sourceCommit/sourceDigest/inputHash/bundleHash/commandHash/attemptId/generation/epoch/result/acceptanceKind/receiptArtifacts/nativeTerminalProof/createdAt/terminalAt`。不适用字段明确 null；缺失原证据标 `unverified`，不得生成虚构 hash、日期或 SUCCESS。`receiptArtifacts` 含 protected object reference、size、SHA-256、保存位置与 missing/pruned 状态，旧 IDs 原样保留，import 主键 `(origin, repoKey, original ID, attempt, generation)` 幂等且遇内容冲突拒绝。

拟议 `GET /control/pilot-history/records?kind=command|validation|cargo|jenkins&id=...` 与有界 cursor list：command 查询接 `CoordinatorClient.command_request_status` / `CommandRequestJournal`；validation 接 `ValidationTicketService.get` 与 `ControlHistoryService.validation`；cargo 接现有 `cargo.list --history` / `CargoJobRunner.status`；Jenkins 接 `SubmissionJournal.get`、`JenkinsClient.reconcile/receipt` 和 archive bytes。旧 `ControlPlaneRouter` 的 validation history 投影保留；新 adapter 不能偷偷接管写入或 ticket acceptance。

历史查询使用现有 repo identity 与 Observer/runtime 鉴权，artifact 下载仍受 owner/权限与路径校验，不暴露 bearer/秘密。旧 success 仅保留原 acceptanceKind，不能升级为当前正式接受或改变 live generation/source pins。retention 沿用现有保护规则与声明期限：只读 import 不收养旧 object、不把 terminal history 变成永久 GC pin，active 独立 retain 仍有效；pruned artifact 可返回元数据与 missing 状态，但不能用于通过证明。

未来 required guards/tests：旧 ID 稳定查询与分页、跨 repo/未认证拒绝、import 重放幂等/冲突失败、archive hash/范围/路径与缺档拒绝、旧成功不可升级、迟到 generation 不接受、现有 retention 活跃保护不变；router 并发 claim/epoch CAS、reserve 后断网 recovery、双 backend 只一 original submit、两个 drain 方向 pending/native/EOF 零判据、queued/running cancel 区分和 history 不可用阻止 cutover。B1 再加 CLI/安装包 import 闭包扫描与全部 transferred evidence 合同回归。仅这些待实现 gate 通过后，才可在另行授权的迁移操作中切换；当前 G7 是设计待审。

这些是未来获准迁移时的操作步骤，试点不会自动执行。

1. **建立清单。** 记录两个系统的运行版本、服务 owner、Session、queue、running attempt、process identity、target/cache/fixture lease 和历史 schema。保存旧配置与数据库可验证备份；秘密不进入构建归档。
2. **Drain 旧执行入口。** 在具体旧 worker 上停止新分派，保留控制面、正式验收、回执与现有执行。当前源支持 `zircon-session.ps1 worker drain <node-id> --draining`，但必须核对实际运行版本支持情况。分别处理本地 ticket worker/scheduler 的 admission；不能以 remote worker drain 冒充全局 drain。
3. **等待已有 attempt 收敛。** 每项都取得 terminal 证据并对账 request/generation/source/leases。未知、失联、materializing 或待收回的 attempt 继续占用资源；不得自动关闭其他 Session。
4. **Cutover 单一入口。** 只为已批准的任务族启用 Jenkins dispatch；旧入口拒绝该任务族新请求。保留 acceptance adapter，避免同一 request 同时在两个 scheduler 执行。完成构建入口、Cargo guard、receipt/history adapter 的同步变更后，再扩大范围。
5. **验收后删码。** 先确认正式门禁、故障恢复和对等性能比较，再逐个解除表中 composition/import/API 依赖。历史 schema 读取和保留业务 API 独立验证；没有解除依赖的文件继续保留。
6. **Rollback。** 停止 Jenkins 新派发，核实其 queue/running attempt 的归属与终止证明；未知执行保留 pin。对账后恢复旧入口 admission，沿用不可变输入和 request 身份。恢复配置不删除 Jenkins 归档、旧数据库或 active target，不重放已经验收的 attempt。

## 历史查询与待验收项

### 当前 compiler cache 与回执边界

当前 `.codex/coordinator-retirement.json` 于07:19:48.379193 UTC标retired，reason为用户授权退役（包含repo integration要求），jenkinsMigration pending。client constructor/from_runtime与zircon-session均拒绝coordinator_retired；正式50504于07:33 UTC native gone。旧恢复/guardian active声明仅是历史。主线程澄清独立退役路径与原试点协调仍待答复，不代表已授权新cutover、恢复或实现；005/baseline未执行，G4/G6 pending。

仅本会话guardian由normal stop flag正常退出（53236/birth134353982987268601、exit0/Job0/EOF），launcher40300 gone；keeper3432/birth134353975568257128仍live，fixture数据/旧lease历史保留。最后旧API renew07:29:45.873、两doc lease07:34:45到期；当前RO未见foreignlease，before-hash与上次一致，不使用旧协调器写API或夺foreign路径。

诊断002为31tests/28pass/2errors/1failure，嵌套writer自锁和未定位async failure不能写全通过；broad manifest受foreign test改变 sourceStable=false。小fixturemetadata反例inode0、fresh真实ID、bulk修改/替换/消失失败且无speedup，不能作正式根因或实施依据。具体proof/hash归单一计划索引；foreign正式操作doc与retirement/gate文档保持原owner内容。

Historical pre-retirement state: 当前试点root为 `D:\cargo-targets\mvp-test-fixtures-3432\jenkins-pilot-01a0f6e1-native-v3-b31b321247fb42559610d67eaef1d66e`，keeper3432/birth `134353975568257128` canonical live，自有29路径由guardian90秒续租。旧6768及其controller/agent/guardian/formal52864 gone原因未定、OS未reboot。旧6178资产/1.16GB逐source before/after/copy hash匹配保留在新root，旧root和lease未迁移修改删除；历史身份继续属于原运行。

Historical pre-retirement state: 正式PID50504/birth `134353971682599459`、instance前缀69c61，06:46 UTC started，read_write/schema80 health OK但baseline degraded；root06:49 canonical start exit0的ready记录早于调用，不能算首次启动证明。ledger两文件优化的37tests/36pass/1skip、12synthetic相等、benchmark最大143→31秒与reviews仅为源码/合成证据。foreign865ee affected测试尚未终态，driver重封存/Jenkins恢复中，005/baseline未submit/execute，G4/G6 pending。

正式操作文档现被foreign owner持有，本任务不再编辑该路径；当前事实只更新自有两份pilot文档，不覆盖其他Session文本或擅自夺claim。

当前快照 publication `8728...` asOf05:20:44.241148 UTC，保守发布延迟293.334–294.554秒，五roots均partial；RO observer28reads/0errors/source与bindings一致/native0Job0EOF只证明观察边界。GC metric32summaries合计5984字/max187、无索引排序、ROquery/decode0.379秒，不支持大JSON根因，也不排除正式连接锁；短record未捕获upgrade frame。新600秒/1Hz idle nonblocking record及publication002仍观察中，不重置timestamp或放宽420/220来通过。005/baseline未提交执行，G4/G6 pending。

Jenkins 已承接调度和真实 Python 任务，仍是过渡试点；正式替换与维护量收益尚未证明。本轮大部分故障发生在共享存储/生命周期支持层，不能据此声称 Jenkins 调度性能改善或整个 workspace 验收。

当前第七正式实例 PID52864/birth `134353876597346275`、instance `b2d95e76cc6e407184430b6aec5b9eac` 已实际 health/intent/action succeeded。scanner 已部署 fresh nofollow stat symlink 判断和仅缺失 category sum；36 ran/35 passed/1 permission skip，review通过，没有420秒保证。当前 driver `6d87d5cea909d3a8b94e4a6fcc1624f6041b1be8e1097fdfa1cf465559bc9e90` / input `2591b74b811fde8e00938ab2eb64ed530e69cd2696085c4dc4811ee523a20b15`，生产288/正式执行285，Rust111/digest8cc不变，binding/pair前缀889124fe/6af49abf。

当前 agent42680 已按实际 `driver-refresh-scanner-final.json` 核对。V4/window004已terminal/no POST；唯一active V5 PID49848/birth `134353905542669929` 于04:56:05.766022 UTC READY，只读watcher005 PID16260/birth `134353903595904226` 于04:52:44.759566 UTC READY。fixture wait3600/native3900只延长观察等待，正式420/trial220/generic8不变。latest `cd639ed...` partial，005未提交，baseline未执行，G4/G6 pending。baseline根dry exit0/Job0/EOF不是实际编译或比较；诊断仅证明footprint预算耗尽，`_observations`耗时根因未证。完整proof/hash由单一计划索引拥有。

正式 `cargo.compiler_cache_prepare` 接受已注册 running supervisor 的 Session/job/PID/native birth，校验当前 active target generation/filesystem 与 exact live reservation，formal service 在短命 validation Job 之外准备共享 daemon。typed schema-1 binding 的 20 字段与 Start -> Workspace -> formal prepare -> Push 顺序见[正式操作契约](local-session-coordinator.md)。Push 仅通过 `verify_prepared_compiler_cache_binding` 只读校验 current claim、native birth、generation、filesystem、runtime/marker；不启 daemon，不获得 CargoAcquire/Home/scratch 权限。amount/FIFO/420 秒 freshness 与 generic 八秒 grace 不变。

Historical cache-binding deployment: 最终源码冻结且独立 review 通过；33 项 Python affected、八项 PowerShell、八项 pilot native 验证通过。daemon/native birth 模型证据不能替代真实 daemon 准备。正式 successor PID 8908/birth `134353809254264621`、instance `d88b8f152d8a425e9232de3e88817107` 已实际恢复；旧 42580 自然退出、没有强杀，285 份 executor manifest before/after 一致。实际空参数 API 调用被 `cargo_compiler_cache_prepare_unproven` guard 拒绝，只证明 guard，不是 daemon 成功。

Historical cache-binding deployment: 当前 driver 前缀 `63fb076f24c404d1`、bundle 尾缀 `a838be`、input 前缀 `988cf309`/尾缀 `2378e`、agent PID 8264；完整身份见 `evidence/driver-refresh-cache-binding-final.json`。Cargo005 与 baseline 仅 prepared，尚未 submit。pilot 异常路径先关闭 QUERY duplicate 避免 last-handle KILL_ON_CLOSE 延迟；晚到 reader 的 receipt snapshot 冻结，failed/unverified 不伪造退出码零，必须保存实际双 EOF 证明。Cargo004 正式编译 exit 0、Jenkins build 10 整体 FAILURE/无 receipt 仍保留。G4/G6 pending；G7 仅 reviewed design，未 cutover 或删除模块。

试点主 CLI 包含 `allocate/prepare/start/stop/install/seal/submit/reconcile/cancel`，Cargo 闭包使用上述独立 `closure` CLI；没有独立 history 命令。操作者从 Jenkins job/build 历史查 queue/build，再用 `submissions.sqlite3` 的 request/attempt journal 和归档 receipt 对账 session、generation、input、bundle。失联请求使用 `reconcile --request-file` 查已有 queue/build，不能直接再次 submit。正式 Cargo 历史仍由 `tools/zircon-session.ps1 cargo list --history --limit 100 -Json` 提供。未来退休旧数据库前须提供按旧 ticket/job/request ID 的只读查询适配和保留期限；当前试点未实现该迁移。

真实 Python 执行、真实 managed Cargo 执行、故障演练和对等性能比较的最终证据由[试点计划](../plans/jenkins-coordinator-pilot.md)统一记录。排入队列、metadata 超时、静态测试或缺少归档回执都不能替代这些 gate。

当前部署根为 `D:\cargo-targets\mvp-test-fixtures-6768\jenkins-pilot-01a0f6e1-native-v2-cba9ebc2924e4b08bd170517b5eec325`：controller 2.580.1、Java 21.0.12.1、29 plugins、零执行槽，单 Windows node 一槽，loopback 53748。这里只记录当前运行，不作为长期默认配置。Python build 2/6 SUCCESS 且归档回执已验证，仍为 `formalAcceptance: false`。build 1–8 均 terminal；Cargo build 4 的 `storage_snapshot_unavailable` 和 build 8 的 `storage_cleanup_reserved` 均是正式准入拒绝，没有编译。共享 root filter 已部署、完整新鲜容量扫描已取得；真实 Cargo 准入成功仍 pending，不能把模型测试当作实际 Cargo 证明。

canonical reuse adapter 不再传 `-TargetDir`；managed 模板清除继承的 `CARGO_TARGET_DIR`，由正式协调器选择 canonical physical target。回执固定 argv、source/hash、实际 supervisor PID/birth、Session、real job、canonical target 与 metrics exact path。七项模型/native tests 通过，独立 review 无 P1/P2；这些是契约验证，尚不构成实际 Cargo 成功。

共享修复涉及 `low_disk_gc.py`、`storage_ledger.py`、`storage_path_index.py`、`cache_budget.py`、`validation_timings.py` 与五份测试：GC busy 后继续存续、ancestor index 加速、cancel 停止预算，以及 timings 只读六种事件类型，避免解码 1,186,653,967 bytes 的无关 dependency events。provider 引用对齐 terminal source/external pin 释放，终态历史不再充当永久 object 引用；实际 Retentions 非 candidate 的 indexed objects 仍包含 active task source/external 保护，active missing 继续 fail closed。历史、索引和 GC 规则没有改动。具体验证与部署状态由试点计划统一记录。

Historical pre-Cargo003 driver: 当前 sealed driver 前缀 `9600e78bee19d474`、input `7ded5db2b57855aee8a5749c51d61fb1b7e6b148664e266434b03cf7f1fad205`，agent PID 33992。正式服务第四次 rollover 已成功，最终 root filter 已部署；完整新鲜容量扫描已取得。Cargo002 queue 16/build 8 因正式 `waiting_disk_space` / `storage_cleanup_reserved` 准入失败，没有 Rust 编译或 cargo job row；baseline001 仍 prepared、未运行。G4、G6 Cargo 与 G5 live cache gate 保持 pending。

历史第二次 rollover 的 successor PID 8084 于 19:26:45 UTC 启动、19:29:11 UTC 健康。快照 `5acd684c3fd44ff2b7b374a618f24dd3` 于 19:37:57 UTC 扫描、约 19:46 UTC 发布，因 coordinator state 内 420 秒文件系统预算耗尽而 partial；聚合约八分钟导致发布时 stale。旧引用不完整/终态未索引诊断消失、unknown JSON 由约 14 MB 降至约 1 KB，不构成完整容量证明。

第四次 rollover action `8f98a5633ced4ad6a59de2de10b50507` 已 succeeded，正式 PID 33204/birth `134353616084209510`、instance `62d15b19bbcf413a93a84c7189952f9a` 于 20:53:31 UTC 启动、约 20:55 UTC ready，已加载最终 trie/hash root filter。历史 21:06 UTC API 返回 available=true、stale=false，但扫描 `4b346d9b5165489d93bcc8ce60863202`（scannedAt 21:04:31 UTC）仍 scanComplete=false，420 秒预算耗尽于 `state/offline-candidates/editor04/project-n100000`。这一失败诊断继续保留；当前扫描状态以下述完整新鲜 scan 为准。

最终 root filter 的源测试和独立审查无 P1/P2；合成 `full_observations` 在 20,000 entries × 5 roots × 2 rounds 下由 16.8198 秒降至 1.8478 秒，全部字段一致。这是支持层合成比较，不是实际 Jenkins/Cargo 性能结论。运维仍需持续维护 fixture/路径租约、封存 driver、服务滚动切换后的身份核验、长扫描预算与发布新鲜度、保留 reservation 归因和两个系统的回执对账；接入 Jenkins 尚未消除这些负担，没有模块退役。

正式 PID 33204 的新扫描 `ee8ff161c9054b47bbd8bbf348f2ae4e` / scannedAt `21:19:23.570563 UTC` 已覆盖五个 roots，21:23:32 UTC 认证 API 返回 available=true、complete=true、stale=false、age 248.195578 秒。Cargo002 的 sourceBefore/sourceAfter 一致，archive receipt/output exact bytes 与 declared command 已核对，Windows Job terminal，失败回执正确拒绝；完整 scan 与可核对失败回执不能替代成功 Cargo 证据。

当前拒绝关联遗留 `D:/cargo-targets/mvp-test-fixtures-6132` reservation：native identity 匹配、无 durable ownership，每约 33 秒的 `artifact.unmanaged_retained` 表示保留，不是删除。其占用必须继续保护。`storage_admission` scope 修复已完成源码回归与审查，已通过第五次 rollover 部署；不清 reservation、不改预算或门槛。`cache-live-002` 零 matching jobs 不构成 live cache proof，Cargo 性能 baseline 仍未运行。具体失败耗时与证明归试点计划。

最新 cleanup scope 源码已冻结：32 项回归全部通过，31.364 秒，独立 review 无 P1/P2；第五次 rollover 已成功，正式 successor 已加载该修复。scope 包含实际 target 与完整 canonical/target-derived `zircon-engine` cache/pool/scratch；未知 scope 仍整根保护，完整 scope overlap 在容量复核与最终 allocator 的同一 writer 内复核。部署证据不构成真实 Cargo 成功，G4、G5 live cache、G6 Cargo 继续 pending。

第五次 action `4e67288e6c994a478c1b406dcf6cfe29` / intent `3144803b83ea4056bdb6c1a5599316a1` 均 actual succeeded。旧 PID 33204/birth `134353616084209510` 自然退出，没有手动 Terminate。新 PID 42580/birth `134353677310677336`、instance `ed6da1b076dd411ca32837883eb381bf` 于 22:35:34 UTC 启动；22:40 UTC 后 read_write health、schema 80 和 native birth 已核对，284 份 executor source before/after 未变，证明为 `evidence/formal-cleanup-scope-successor-restoration.json`。PS5 start 请求仍等完整 status 输出；已有 accepted successor/reconcile 证明，不重复 accepted start。

Historical Cargo003 driver: 当前 driver 为 `9f1d42edc66da2d02ccf161f95eba76394d4745501384b49fa1d986bdc24a396`，input `dfa156c7891dbfc20384a00847d536db44d68f8ddbdc44915eaf589d9a9d3f9d`，driver root `driver-9f1d42edc66da2d0`、agent PID 42692。Cargo003 沿用 Cargo source bundle/input 的 `549...`/`7acfd...` 身份；准备阶段 live watcher PID 43788 READY、容量 stale，曾等待提交。实际后续执行与当前 Cargo004 状态如下，不构成 G4/G5/G6 通过。

后续 Cargo003 已实际执行为 queue 18/build 9 FAILURE，88.210 秒。prepare 32.585604 秒超过实际启动时剩余的 19.294917 秒 freshness 窗口，正式 acquire 随后因 `storage_snapshot_unavailable` 拒绝；没有 Rust 编译或 job row，不能归因为 scope 修复再次失败。source hash 未变、native Job terminal、archive exact bytes 相等，失败回执仍正确拒绝。两个 cache observer 窗口零 captures 不构成 live cache proof。

Historical pre-queue20 observation: Cargo004 自动提交助手正在启动：预检查 driver/source/agent/queue/watcher 后，最后要求完整新鲜快照 age <= 220 秒才一次 original submit；正式 420 秒门槛不变。实际 queue ID 尚未确认，G4/G5/G6 不标通过。PS5 start native PID 17992 已 retained-handle 证明 exit 0，但 caller PID 43272 仍等两 reader EOF，不能声称 stdout 已收集或启动失败。正式 PID 42580 健康，caller/正式服务 external Jobs 尚未识别，保留 caller；实际 retained collector 是运维尾项，未停止正式服务。

最新提交准备已重封存为 driver `f96cc8f191e325096003e0d6bbfcd1538ecd669ff6340234a44153a6580c1b70` / input `af6beff6dc80c094e914f62caa4c82a8e7a9c10548a85eea5d6d9b0c584477da`，agent PID 45032 于 23:52:48 UTC 启动。原因是 foreign 三份源码的实际逻辑变化，未修改 foreign 文件或重启正式服务；`9f...` driver 仅保留为 Cargo003 历史证据。

Historical pre-queue20 observation: Cargo004 v2 助手 PID 39520/birth `134353728722327542` 于 00:01:15 UTC 实际 READY，native keeper 6768；Cargo identity、bundle/source input 不变，仍未 submit。先冻结 286 份实际 executor manifest、核实 idle agent/watcher，候选完整新鲜快照出现后重复 prechecks，最后检查认证 API 与本地 age <= 220 秒才一次 submit，正式 420 秒不变。当前快照 `a26c80ad...` asOf 23:50:04 UTC incomplete/stale，G4/G5/G6 pending。单次 py-spy 采样显示系统忙，不构成吞吐改善证据。

后续 Cargo004 fresh gate 在 00:49:40.016 UTC 核实五 roots complete、local/API age 172.227/171.915 秒、queue idle、watcher exact live 和 wrapper equal，仅提交一次 queue 20/build 10。正式 managed job `ee05800ce7334bbea54a8f8f86017f62` check exit 0/released，但 nested native grace 八秒超时，build 10 FAILURE，没有 pilot receipt/Jenkins artifacts（archive 404）。不能由 formal exit 0 或 autogate zero 推断 nested Job zero、EOF 或整链 accepted。root 正在修 fail-closed 错误归档并诊断 persistent daemon，无新服务 restart/阈值变化；G4 whole chain pending。

livecache004 三 captures 已核实同一 job 的 leased/running writer active=1、完成后 active=0、released/retained physical identity 不变，watcher terminal zero/EOF。它支持单 job live writer 子项，不能证明跨 job reuse 或 G5 全部通过。v3 autogate 已 terminal queue 20，v4 仅 prepared/未启动；baseline f96 仅 prepared/dry validation，G6 Cargo 对比仍未执行。具体 native 身份、耗时和证明见试点计划。

已演练排队/运行重复 submit、排队取消、运行取消、controller 重启和 agent 停止。controller 重启恢复原 build；agent 停止后 native Job 与 probe 已终止、日志已释放，但重启后原 build 仍 running，需要显式 cancel 至 ABORTED。不能以 Jenkins 状态代替进程证明。单样本 Python 比较已记录，OS file cache 未受控；没有速度或删除量结论，Cargo 比较仍 pending。
