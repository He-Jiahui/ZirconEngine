---
related_code:
  - .github/workflows/ci.yml
  - Cargo.toml
  - tools/session_coordinator/server.py
  - tools/session_coordinator/client.py
  - tools/session_coordinator/config.py
  - tools/session_coordinator/database.py
  - tools/session_coordinator/migrations.py
  - tools/session_coordinator/leases.py
  - tools/session_coordinator/validation_tickets.py
  - tools/session_coordinator/validation_ticket_worker.py
  - tools/session_coordinator/validation_copies.py
  - tools/session_coordinator/workspace_copy.py
  - tools/session_coordinator/workspace_copy_terminal.py
  - tools/session_coordinator/git_finalize.py
  - tools/session_coordinator/git_index_lock.py
  - tools/session_coordinator/integration_candidates.py
  - tools/session_coordinator/artifact_governance.py
  - tools/session_coordinator/control_plane/artifact_downloads.py
  - tools/session_coordinator/offline_queue.py
  - tools/session_coordinator/control_plane/http.py
  - tools/session_coordinator/control_plane/router.py
  - tools/session_coordinator/control_plane/actions/service.py
  - tools/session_coordinator/control_plane/actions/executor.py
  - tools/session_coordinator/control_plane/snapshot.py
  - tools/session_coordinator/codex_sync/hook.py
  - tools/session_coordinator/codex_sync/spool.py
  - tools/session_coordinator/codex_sync/worker.py
  - tools/session_coordinator/supervision/runtime_descriptor.py
  - tools/session_coordinator/supervision/service.py
  - tools/session_coordinator/web/package.json
  - tools/session_coordinator/web/src/api/contracts.ts
  - tools/session_coordinator/web/src/api/validation.ts
  - tools/session_tray/Cargo.toml
  - tools/session_tray/src/coordinator_client.rs
  - tools/session_tray/src/runtime_descriptor.rs
  - tools/session_tray/src/recovery.rs
  - tools/session_tray/src/lifecycle.rs
  - tools/install-codex-session-hook.ps1
  - tools/install-session-coordinator-task.ps1
  - tools/zircon-session.ps1
tests:
  - tools/session_coordinator/tests/test_control_http.py
  - tools/session_coordinator/tests/test_control_snapshot.py
  - tools/session_coordinator/tests/test_deferred_action_client.py
  - tools/session_coordinator/tests/test_integration_candidates.py
  - tools/session_coordinator/tests/test_validation_tickets.py
  - tools/session_coordinator/tests/test_workspace_copy.py
  - tools/session_coordinator/tests/test_artifact_governance.py
  - tools/session_coordinator/tests/test_git_index_lock.py
  - tools/session_tray/src/lib.rs
plan_sources:
  - docs/plans/optimize/00-engine-wide-review.md
  - docs/plans/optimize/zircon_tooling/01-workspace-toolchain-ci-validation-and-developer-entrypoints-review.md
  - docs/plans/zircon_tooling/session_coordinator/01-workflow-control-center-and-tray.md
reference_engines:
  - dev/UnrealEngine/Engine/Source/Programs/UnrealBuildTool/Actions/Graph/ActionGraph.cs
  - dev/UnrealEngine/Engine/Source/Programs/UnrealBuildTool/Actions/History/ActionHistory.cs
  - dev/UnrealEngine/Engine/Source/Programs/UnrealBuildTool/System/FileHashCache.cs
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/BuildGraph/BgGraphBuilder.cs
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/BuildGraph/BgNodeExecutor.cs
  - dev/UnrealEngine/Engine/Source/Programs/AutomationTool/BuildGraph/TempStorage.cs
  - dev/UnrealEngine/Engine/Source/Programs/Horde/Docs/Config/Agents.md
  - dev/UnrealEngine/Engine/Source/Programs/Horde/Docs/Config/Artifacts.md
  - dev/bevy/.github/workflows/ci.yml
  - dev/godot/.github/workflows/static_checks.yml
  - dev/godot/.github/workflows/windows_builds.yml
doc_type: review-and-refactor-plan
review_status: review_complete
implementation_status: pending
source_recheck_required: true
---

# 06 · Session Coordinator 控制面、租约、验证、产物、提交与监督工程化差距

## 1. 结论

Session Coordinator 已经不是一组简单脚本，而是一套独立的本机控制系统。本轮覆盖 `tools/session_coordinator` 的 306 个 tracked 文件、106,498 physical lines、5,429,337 bytes，其中 106 个生产 Python 文件为 55,498 行，102 个测试文件为 43,943 行并声明 1,175 个 `unittest` 方法；再加 Session Tray 与安装入口后，审查范围为 333 个 tracked 文件、116,080 text lines、6,008,898 bytes。数据库已到 schema 65，包含 Session、计划 WIP、路径租约、Cargo lane、validation copy/ticket、integration candidate、artifact receipt、workflow、Codex sync、supervision、control action 和审计事件。路径租约的 canonicalization/ancestor conflict、artifact 删除前的 filesystem identity reservation、bounded output tail、benchmark Job Object、integration candidate 的 temporary index/commit-tree 构造，以及 Tray 的恢复断路器都比“临时实现”成熟，应保留并升格为共享基础设施。

- 失败交接（`open / Hub03 修复中`）：[Hub shared recent-project loader test import drift](../../zircon_hub/03/failure-2026-08-27-shared-recent-project-load-import.md)

但当前控制面没有可信安全边界。普通 HTTP handler 的 `_authorized()` 恒为 `True`，control HTTP 又把每个请求标成 `runtime_authorized=True`；router 因而把任意本机请求映射成 `maintainer`。runtime descriptor 明确写空 token，测试还固化了“浏览器无需 bearer/cookie 并得到 maintainer”。同一个无认证 `/command` 接口没有 Host/Origin、body 上限、Content-Type 或线程准入限制，却能提交任意 validation command；ordinary validation 随后直接 `Popen(command_tuple)`。浏览器简单请求、被攻陷的本机低权限进程或 DNS rebinding 链可以越过已经实现的 cookie、CSRF、elevation、role 与 action confirmation 体系，进入本机命令执行。

验证和提交的证明链也未闭合。`validation.record_result` 允许调用方把 queued/materializing/running ticket 直接写成 passed，未要求 worker identity、run ID、process receipt 或 command exit evidence；integration candidate 只检查 ticket 同 Session 且 passed，不证明 ticket 的 source manifest、command 和 candidate blob OID 相同。另一条 milestone finalize 路径先覆盖共享 `.git/index`，再在实时工作树根运行 validation commands，既不能证明被测试的字节就是将提交的 tree，也可能在长验证期间覆盖外部 Git staging，最后用旧 index snapshot 抹掉用户新暂存内容。

进程监督只在 benchmark 路径完整。普通 validation 不进入 Windows Job Object，不持久化 process creation identity；取消只对根进程 `terminate()`。daemon 重启后如果 PID 仍存活，恢复逻辑会继续把任务留在 running，但已经丢失 Popen、管道 reader 和 collector；它无法重新附着、取消或收集终态。无 timeout 的 `process.wait()` 与 reader `join()` 又允许任务及 lane 无限挂起。这里需要统一 managed process tree，而不是继续在每条执行路径补 recovery 分支。

动态验证同样不能支持“已工程化完成”的结论。完整 Python discovery 在 15 分钟后超时；四分组中的一个分组运行 231 项后有 3 个 error。两个离线队列 error 串行复跑后消失，证明测试存在跨组共享/patch 隔离问题；`test_control_snapshot` 串行 19 项仍有 1 个 error，因测试要求同一计划族同时存在两个 executable primary，而生产 WIP 规则拒绝。Web `npm run check` 的 typecheck、70 tests、Vite build 均通过，但命令先覆盖 tracked `dist`，真实重建产生 22 个新 hashed 文件，说明提交的 dist 不是当前 source 的结果；check 只验证刚生成的目录，没有验证 Git currentness。Tray `cargo test --locked` 的 36 项通过。根 CI 对上述 Python/Web/PowerShell/Tray 全部没有 consumer。

本轮记录 8 个 P0、48 个 P1 和 10 个 P2。未修改 Coordinator、Web、Tray、PowerShell、CI、数据库或 Git 状态，只写审查与索引。

## 2. 审查边界与证据

### 2.1 物理范围

| 子域 | 规模 | 本轮深度 |
|---|---:|---|
| Coordinator tracked set | 306 文件 / 106,498 physical lines / 5,429,337 bytes | E2 全量 inventory；所有 owner chain E3 |
| 生产 Python | 106 文件 / 55,498 行 / 2,267,339 bytes | server、DB/migration、lease、validation、Git、artifact、workflow、control、sync、supervision逐文件读取 |
| Python tests | 102 文件 / 43,943 行 / 1,878,724 bytes | 1,175 test methods / 107 TestCase；动态全量与分组验证 |
| Control Web source/scripts | 60 文件 / 3,998 行 / 238,338 bytes | contract、validation、API、build/currentness与测试链 |
| Session Tray 与安装入口 | 25 tracked 文件 / 13,456 physical lines / 558,595 bytes | Rust client/lifecycle/recovery/startup、PowerShell install/query、独立workspace |
| Combined report scope | 333 tracked 文件 / 116,080 text lines / 6,008,898 bytes | index-record fingerprint固定 |

combined tracked scope 的 Git index-record SHA-256 为 `c68a32ca33054189645c32a02db82a87a5122b937ee6b57cb3f39d5d9072a759`。实现前必须重取指纹；`source_recheck_required` 不是形式字段。

生产 Python 最大文件为 `cargo_jobs.py` 3,332 行、`server.py` 3,046 行、`migrations.py` 2,834 行、`git_finalize.py` 2,465 行、`workflows/milestones.py` 2,448 行、`workspace_copy.py` 2,358 行、`cli.py` 2,038 行和 `supervision/service.py` 1,816 行。`server.py` 至少有 98 个唯一直接 command name 分支。生产路径创建 15 个显式线程，全部为 daemon thread；未发现统一 semaphore、`ThreadPoolExecutor` 或 `ProcessPoolExecutor`。

### 2.2 动态验证

| 命令 | 结果 | 可支持结论 |
|---|---|---|
| `python -m unittest discover -s tools/session_coordinator/tests -p "test_*.py"` | 904.040 秒超时，无完整summary | 当前全量门时长不可接受；不能声称全量通过 |
| 四组并行 Python discovery | 已观察分组跑 231 项，3 error、1 skip、435.891 秒 | snapshot 规则漂移；并行测试存在共享/patch隔离问题 |
| `python -m unittest -v ...test_deferred_action_client` | 12 passed / 12.594 秒 | 并行出现的两个 ctypes error 串行不复现，不能误报产品兼容bug |
| `python -m unittest -v ...test_control_snapshot` | 18 passed、1 error / 101.931 秒 | 同计划族 primary 假设与当前 WIP admission 不一致 |
| `npm run check` | exit 0 / 42.652 秒；70 tests passed；954 modules built | source测试通过；不证明checked-in dist current |
| `cargo test --manifest-path tools/session_tray/Cargo.toml --locked` | 36 passed / 61.4 秒 | Tray当前Windows测试基线通过 |
| Hook `Query` | exit 0；`daemonCompatible=false` | hard-coded schema 28 与当前65动态不兼容 |
| Scheduled task `Query -DryRun` | exit 0；task未安装 | 只验证只读查询；没有安装/升级/回滚实机证据 |

Web check 产生的新 dist 已机械恢复为 Git index 内容，`git diff --quiet -- tools/session_coordinator/web/dist` 为 0，且无残留 untracked hashed file。真实仓库有一个本轮开始前已存在的零字节 `.git/index.lock`；本轮没有删除、恢复或绕过它，也没有让 Coordinator 的自动 lock recovery接触它。

### 2.3 正向基线

- `LeaseService` canonicalize 路径、保护 `.git`/`.codex/state`、按分隔符边界判断祖先冲突，并在事务内重查 live owner；这比字符串前缀锁可靠。
- artifact governance 在删除前拒绝 reparse point/越界 root，持久化 filesystem identity reservation，并使 cleanup reservation 可恢复；不能把它误写成任意递归删除。
- validation output tail 每流限制 65,536 characters，避免把完整编译输出永久塞入 SQLite；问题在进程生命周期和证明链，不在这项上限。
- benchmark validation 已使用 atomic kill-on-close process/job 与 creation identity；它是普通 validation 应收敛到的现成实现方向。
- integration candidate 的 sealed blob 和 temporary Git index 可生成精确 commit tree；缺口是 compile ticket 没绑定这些 blob，以及共享 index 对齐仍触碰外部状态。
- control plane 已有 strict Host/Origin helper、cookie、CSRF、one-use elevation、role catalog、preview/confirm、state fingerprint 与审计事件；当前失败是入口把它们全部旁路，而不是从零缺安全组件。
- Tray 对 PID + process creation time 做终止身份校验，恢复有 bounded backoff/open circuit，相关 36 项测试通过。

### 2.4 参考边界

- Unreal UBT `ActionGraph`显式建立 produced-item owner、prerequisite graph、cycle/output-conflict detection与action invalidation；`ActionHistory`和`FileHashCache`持久化action/input history。Zircon需要借鉴“命令、输入、输出、环境、工具链共同形成action identity”，不是照搬C# API。
- Unreal AutomationTool BuildGraph 把 node、agent、dependency、executor和 `TempStorage` complete marker/integrity check作为独立协议。它直接反驳“任意argv + caller写passed就是验证凭据”。
- Unreal Horde 将 agent、lease、job、artifact和ACL作为服务端受认证资源。当前 Zircon仍是单机工具，不要求立即复制分布式规模，但本机 maintainer authority同样必须有不可伪造身份和fencing。
- Bevy CI 提供并发取消、job timeout、矩阵、cache与artifact upload；Godot按平台拆build并独立运行static checks。两者证明大型引擎不会把 1,175 个控制系统测试留在本地且无 merge consumer。
- Fyrox、Godot runtime与 Unity Graphics 在本轮所选源码中没有与 Coordinator 等价的本机控制平面；报告不虚构不存在的参考能力。它们继续作为后续asset/build/editor子域的参考，而非此处认证依据。

## 3. 当前 P0

### TOOL-COORD-P0-001 · 所有本机请求都被授予 maintainer

`server.py:2665-2668` 的 `_authorized()` 恒为 `True`；`control_plane/http.py:45-47` 又无条件设置 `runtime_authorized=True`，`router.py:332-340` 因而返回 `local-runtime/maintainer`。`runtime_descriptor.py:31-34`、server启动结果和 Python client 都把 token 固定为空。`test_control_http.py` 还明确测试 direct browser query 无 bearer/cookie，并断言 actor/role 为 local-runtime/maintainer。

必须新增 control protocol v2：daemon启动生成256-bit capability，只通过 ACL 限制的 descriptor/命名管道交付；CLI/Tray/hook用challenge-bound bearer或本机IPC身份，浏览器只能用短期single-use bootstrap ticket换 HttpOnly session。删除 `runtime_authorized` 布尔旁路；role必须由认证结果产生。v1 只能在显式migration window读，不得继续执行mutation。

### TOOL-COORD-P0-002 · 旧 `/command` 绕过 Host/Origin、body预算和连接准入

`do_POST`先接受恒真授权，随后直接读取调用方 `Content-Length` 并 `json.loads`；没有Host/Origin、Content-Type、最大body、header/read deadline或request thread admission。control v1已有1 MiB上限和loopback校验，但高权限command入口不使用它。浏览器可以用simple `text/plain` POST触发请求，即使同源策略阻止读取响应也不能阻止副作用；恶意Host/DNS rebinding同样没有被旧入口拒绝。

v2必须只保留一个router/decoder：先验证exact Host或本机IPC peer、认证、method/content type、header/body budget、deadline、idempotency key，再进入generated command dispatcher。`ThreadingHTTPServer`前增加有界connection/request executor和per-principal quota；旧 `/command` 在迁移后返回410。

### TOOL-COORD-P0-003 · validation command 是无认证本机任意进程执行入口

validation submit接受调用方 command/toolchain/coverage/source manifest；`workspace_copy.py:1138-1142`只拒绝空argv，普通路径在 `1242-1250` 直接以validation source root为cwd调用 `subprocess.Popen(command_tuple)`。没有可执行文件allowlist、模板ID、resolved toolchain、sandbox policy或签名action descriptor。与P0-001/002组合后，任意可访问loopback的网页或进程可让Coordinator执行任意本机命令。

把 public command 改为 `ValidationActionId + declared parameters`。server从版本化action catalog解析固定executable、argv schema、cwd、environment allowlist、resource class、timeout和output contract；worker只执行server签发且MAC/DB identity匹配的resolved action。需要自由命令的maintenance入口必须单独认证、显式elevation并默认禁用。

### TOOL-COORD-P0-004 · validation passed 状态可由调用方伪造

`server.py:1468-1535` 的 `validation.record_result` 接受任意 status/evidence并直接调用ticket service。状态机允许 queued/materializing/running 转 passed；没有worker lease、attempt/run ID、process creation identity、exit code、action digest、copy identity或receipt signature。CLI暴露同一command，因此“passed”只是调用者声明，不是执行证明。

终态只能由持有attempt fencing token的worker提交。receipt至少绑定 ticket ID、attempt、resolved action digest、input tree/CAS manifest、toolchain fingerprint、environment policy、start/end、root process identity、exit code、bounded log hashes和produced artifact hashes。状态转换用 compare-and-swap 校验running attempt；人工override进入独立`waived`状态并记录审批者，绝不能伪装成passed。

### TOOL-COORD-P0-005 · compile ticket 没有绑定 integration candidate 字节

`integration_candidates.py:102-120`只查询ticket的session_id/status；随后才从实时路径写Git blob。它不比较ticket的source manifest/action/validation copy与 candidate paths/blob OID，也不要求命令确实是compile。任意同Session passed ticket，即使验证完全不同文件或只是成功执行 `cmd /c exit 0`，都能使候选进入integration_ready。

candidate seal时生成candidate tree manifest；compile action必须把该manifest或精确ancestor input tree作为identity，并声明覆盖的target/profile/features。提交前server验证receipt/action/candidate的hash闭包。candidate任何blob变化都会产生新candidate generation并使旧ticket失效，不允许只按Session复用。

### TOOL-COORD-P0-006 · milestone finalize 验证的不是将要提交的tree

`git_finalize.py:808-873`先把目标路径stage进共享index，然后在 `cwd=self.repo_root` 运行 validation commands。命令看到的是实时工作树，包括stage后变化、未stage候选变化和外部脏文件；最终commit使用的却是index tree。验证完成与commit之间也没有重新比较tree identity。因此“validation passed”不能证明commit内容通过。

所有finalize统一到 isolated worktree/CAS materialization：从candidate tree OID创建只读或隔离workspace，在该tree上执行resolved validation，receipt绑定tree OID；commit只能引用同一tree。验证后若HEAD或candidate generation变化，重新rebase/构树/验证，不允许复用旧receipt。

### TOOL-COORD-P0-007 · shared index snapshot/restore 可抹掉外部暂存数据

同一路径读取原始 `.git/index` bytes、覆盖共享index、持锁执行长验证，最后恢复旧snapshot。SQLite `git_mutex`只能约束Coordinator，不能阻止用户、IDE、hook或其他Git进程在期间更新index；这些更新会在restore时被静默删除。`cleanup_shared_index`还会把整个index reset到HEAD。自动index-lock recovery也不应成为这条设计的安全证明。

彻底禁止验证/构树写共享index。所有内部Git操作设置独立 `GIT_INDEX_FILE`，对齐共享index只作为短事务、逐path compare-and-swap且默认不做。外部index状态永远属于用户；Coordinator只发布candidate commit/tree并由显式integration transaction移动ref。

### TOOL-COORD-P0-008 · 普通 validation 在 daemon 重启后成为不可管理活进程

普通路径不创建kill-on-close Job Object，也不持久化creation time；cancel只从内存map取Popen并对根进程 `terminate()`。`recover_interrupted_jobs`遇到仍活的PID会保留running，但Popen、stdout/stderr reader和collector已经丢失，后续cancel返回race，终态永远无人提交。根进程结束而子进程继承pipe时，无timeout `reader.join()`还可永久阻塞。

把benchmark的managed process tree抽成所有validation/Cargo/tool action共用的ProcessSupervisor。启动必须原子创建suspended process + kill-on-close job、记录PID/creation time/job lease/attempt后再resume；daemon重启要么通过durable worker子进程重附着，要么以fencing终止整棵树并重试。取消、超时、shutdown、crash recovery都走同一tree termination与receipt协议。

## 4. 控制协议与服务架构差距

### TOOL-COORD-P1-001 · 安全旁路已被测试固化为产品契约

测试名称和断言把token-free maintainer视为成功语义，未来只改实现会被旧测试阻止。需要先写v2 threat model和协议ADR，再把旧测试改成“无认证拒绝、observer最小权限、maintainer需elevation”，并保留v1拒绝/迁移测试。

### TOOL-COORD-P1-002 · maintenance capability 在未配置环境变量时等于关闭

`_require_maintenance_capability`在 `ZIRCON_COORDINATOR_MAINTENANCE_TOKEN` 未设置时直接return，注释又错误声称runtime descriptor已经认证。destructive maintenance必须始终要求daemon-generated capability/OS identity和独立elevation，环境变量只能覆盖或轮换secret，不能决定是否认证。

### TOOL-COORD-P1-003 · cookie、CSRF、elevation和role系统成为不可达的安全外壳

router只在 `runtime_authorized=False` 时验证cookie与CSRF，但HTTP adapter永远传True。action preview返回confirmation phrase，任意调用方又已是maintainer，所以confirmation只证明同一未认证调用方读写两次。认证改造必须做端到端测试，不能只单测AuthService helper。

### TOOL-COORD-P1-004 · 旧command decoder没有统一的strict schema

98个以上command分支在`server.py`逐项取dict字段，required/optional/unknown-field、numeric budget和enum规则分散。建立IDL生成Python request decoder、TypeScript/Rust client和文档；默认拒绝unknown field，版本迁移由显式adapter处理。

### TOOL-COORD-P1-005 · `ThreadingHTTPServer` 为每个请求创建无界daemon thread

`daemon_threads=True`使关闭不等待handler，且没有最大并发、queue、per-client quota或overload response。改为有界executor/async server，区分read/action/stream lane，达到上限返回429/503，并把active request纳入shutdown drain。

### TOOL-COORD-P1-006 · `/command` 没有读取deadline与半包防护

control HTTP会设置5秒socket timeout，旧command路径没有；客户端可声明大长度后慢速发送，长期占用线程。统一HTTP adapter必须在解析前设置header/body deadline并限制Content-Length，处理缺失、负数、超限和提前EOF。

### TOOL-COORD-P1-007 · SSE 每客户端占一线程并每250ms轮询SQLite

当前最多8个stream虽限制总数，但每条stream仍独占HTTP thread、周期查询数据库并依赖5秒socket timeout。改为单event broker读取append-only cursor，向bounded subscriber queue fan-out；慢消费者丢到last-resumable cursor或断开，不得放大数据库读。

### TOOL-COORD-P1-008 · 后台工作以零散daemon thread而非owned executor运行

deferred action、validation collector、watch/sync等线程缺统一owner、budget、cancellation token、panic/exception supervision和join次序。引入RuntimeSupervisor管理thread/task registry；每项声明owner、criticality、deadline和shutdown phase。

### TOOL-COORD-P1-009 · maintenance loop 串行承载多个独立职责

同一循环顺序执行watch、Cargo reconciliation、cleanup、artifact scan、workspace recovery、ticket worker和scheduled maintenance。一个慢磁盘扫描会推迟lease/ticket recovery。拆成按职责调度的周期任务，保存last_started/completed/error/duration，设置overrun与staleness告警。

### TOOL-COORD-P1-010 · 固定6518端口与repo-scoped daemon身份冲突

config称“single local service”，但state、runtime descriptor和scheduled task name都按repository key分隔；安装脚本却不给daemon传port，所有仓库默认争用6518。必须选择并实现一种模型：单machine broker路由多个repo，或每repo由broker分配端口/命名管道并写descriptor。不能同时声称单服务和每repo独立进程。

### TOOL-COORD-P1-011 · `CoordinatorApplication` 和 command switch 已成为god object

3,046行server同时完成composition root、98+ command dispatch、maintenance和HTTP host。按Session/lease/validation/Cargo/Git/artifact/workflow/supervision拆 typed command handlers；composition root只装配接口，transport不得直接调用service私有helper。

### TOOL-COORD-P1-012 · Python、TypeScript、Rust、PowerShell手工复制协议

Web `contracts.ts`有47个exported type/interface，`validation.ts`有44个parse/validation function和36个throw；Tray另写Rust DTO/HTTP，PowerShell又解析descriptor。用版本化JSON Schema/OpenAPI或等价IDL生成codec和compat tests，并固定schema fingerprint。

### TOOL-COORD-P1-013 · runtime descriptor没有能力协商和滚动升级边界

descriptor有版本数组和instance identity，这是好基础；但token固定空、各consumer仍手工检查字段，installer甚至固定schema 28。定义min/max protocol、feature capability、server build ID、contract hash与expiry；consumer根据能力降级，只在破坏性不兼容时拒绝。

### TOOL-COORD-P1-014 · control action确认没有独立授权主体

preview/state fingerprint可以阻止状态漂移，却不能证明confirm来自另一个可信主体。高风险操作至少绑定authenticated principal、session nonce、短期elevation和reason；真正破坏性维护支持双人/策略审批，而不是把phrase当authentication。

## 5. Validation、Cargo 与证明链差距

### TOOL-COORD-P1-015 · validation没有版本化 action catalog

command、toolchain、coverage requirement和manifest由调用方拼装，无法集中审计允许的编译/测试组合。建立 `ValidationActionDescriptor`，声明owner、target/profile/features、toolchain、argv template、environment、inputs、outputs、resources、timeout、retry与receipt schema。

### TOOL-COORD-P1-016 · validation没有执行时限

`process.wait()`和reader `join()`均无timeout，action descriptor也没有budget。每个action必须有soft/hard deadline、heartbeat与progress policy；soft timeout采集诊断，hard timeout终止整个job并写typed terminal reason。

### TOOL-COORD-P1-017 · 普通与benchmark执行器安全等级分裂

benchmark使用Job Object/creation identity，普通验证使用裸Popen。合并为一个执行器，通过resource policy附加benchmark约束；不能让最常见的compile/test路径保留较弱监督。

### TOOL-COORD-P1-018 · 终端收集在root退出后仍可被继承pipe卡死

bounded tail限制内存，但reader必须等EOF。若未受管子进程继承stdout/stderr，root exit不等于pipe close。Job tree终止/complete marker之后再有限时join；超时记录truncated/collector error，不能无限占lane。

### TOOL-COORD-P1-019 · 恢复只用PID alive，不能抵抗PID复用

普通copy表不保存process creation time，startup recovery把同PID的无关新进程当原任务。所有process evidence必须使用PID + creation identity + supervisor generation；无法验证身份时fail closed并人工隔离，而非附着或终止未知进程。

### TOOL-COORD-P1-020 · validation input closure依赖手写源码解析

`validation_copies.py`识别部分 `include_bytes!`/`include_str!` 与有限 `concat!`/`env!`，无法证明proc macro、custom macro、build script、compiler config和动态生成输入。使用Cargo/rustc dep-info、build plan、compiler trace与action graph作为权威closure；手写扫描只做提前诊断。

### TOOL-COORD-P1-021 · terminal evidence只是可变JSON而非typed receipt

即使真实worker执行成功，ticket evidence也没有schema/version/hash chain。把receipt作为不可变表和artifact存储对象，所有projection只引用receipt ID；数据库约束终态ticket必须存在同attempt receipt。

### TOOL-COORD-P1-022 · Cargo reservation与validation action identity尚未统一

Cargo lane已有command/source manifest/copy绑定和FIFO防抢占，但普通validation ticket仍是另一套自由command语义。统一为ActionGraph node：reservation调度资源，validation copy提供input tree，process supervisor执行，receipt发布结果。

### TOOL-COORD-P1-023 · 缺少UBT式producer/output冲突与action history

当前协调器能调度命令，却没有统一的produced-item owner、prerequisite cycle、output conflict和历史action invalidation。引入persistent ActionGraph；同一output只允许一个producer，cache key覆盖tool binary、argv、env policy、input hashes和platform。

### TOOL-COORD-P1-024 · 测试与编译状态没有统一的waiver语义

人工场景目前可直接写passed。应提供独立 `waived`/`accepted_with_risk` 状态，记录approver、scope、expiry和reason；release policy可明确拒绝waiver，而不污染自动验证统计。

### TOOL-COORD-P1-025 · 失败重试缺attempt级fencing与幂等边界

ticket/job存在状态，但重启/重试的worker所有权没有统一attempt lease。每次执行生成单调attempt与fencing token；旧worker迟到的progress/result必须被DB CAS拒绝，side effect输出写attempt staging后一次publish。

### TOOL-COORD-P1-026 · 分布式worker协议仍不存在

当前只支持本机Popen和本机路径。长期达到Horde级别需要authenticated agent enrollment、capability/resource advertisement、lease heartbeat/fencing、CAS materialization、log streaming和artifact upload；第一阶段先把本机worker走同一协议，避免未来重写证明模型。

## 6. Git、租约、存储与产物差距

### TOOL-COORD-P1-027 · Git finalize存在两套不等价路径

integration candidate已用temporary index构tree，milestone finalize仍快照共享index并验证live worktree。收敛为一个 `CommitTreeTransaction`：sealed inputs、temp index、tree OID、isolated validation、ref CAS、receipt和recovery journal。

### TOOL-COORD-P1-028 · SQLite git mutex不能协调外部Git

数据库锁只覆盖本进程协议参与者。所有不需要共享index的操作必须彻底隔离；确实要更新ref时使用Git原生lock/ref transaction和expected old OID，不把SQLite mutex当跨进程互斥证明。

### TOOL-COORD-P1-029 · 自动清理stale index lock风险边界过宽

process probing与零字节/age heuristic可以辅助诊断，但Coordinator不应主动删除不由它创建的共享 `.git/index.lock`。内部临时index lock可按owner receipt恢复；外部lock只报告path/age/process evidence并要求用户或Git owner处理。

### TOOL-COORD-P1-030 · 路径租约获取是全表扫描式层级冲突检查

正确的separator-aware逻辑应保留，但每次acquire读取并比较所有lease，规模上接近O(existing × requested)。增加canonical path segments/ancestor keys或数据库辅助索引，并用千级Session/十万路径基准验证p95。

### TOOL-COORD-P1-031 · cooperative lease没有写入fencing

租约能阻止遵守协议的Session，但文件系统写入本身不会携lease generation，过期owner或绕过Coordinator的工具仍可写。关键commit/import/generator入口必须提交lease token并在publish前重验；外部写由watcher标记baseline degraded并阻止证明复用。

### TOOL-COORD-P1-032 · SQLite单写者承担全部控制域

每次connect设置WAL，每个默认transaction使用`BEGIN IMMEDIATE`。本机规模可接受，但command、event、SSE、lease、artifact与workflow共享10秒busy timeout，没有domain队列/写入延迟SLO。先建立transaction metrics与短事务规则；分布式阶段再拆authoritative store。

### TOOL-COORD-P1-033 · 65个migration集中在2,834行单文件

迁移逻辑、schema常量、数据修复和VACUUM边界耦合，review与故障定位成本持续上升。按版本文件组织up migration、preflight、postcondition和fixture；启动只执行短兼容migration，大表rewrite走显式maintenance job。

### TOOL-COORD-P1-034 · 缺少数据库备份、恢复和升级失败runbook

WAL与事务不等于可恢复升级。定义升级前online backup、schema/DB integrity check、磁盘预算、失败回滚或forward repair、保留窗口和operator命令；用旧schema fixture到65的fault injection验证每个中断点。

### TOOL-COORD-P1-035 · artifact root发现硬编码D/E/F和三个目录名

`enabled_target_roots`只扫描D/E/F下的`cargo-targets`、`targets`、`ZirconBuilds`，遗漏C盘、挂载卷、网络/容器路径和显式workspace配置。改为配置注册的ArtifactRoot descriptor，包含owner、filesystem identity、quota、retention和allowed artifact classes。

### TOOL-COORD-P1-036 · artifact下载存在resolve/stat/open间TOCTOU窗口

路径containment、range和大小上限是正确基线，但若文件在验证后被替换为reparse target，后续open可能读取不同对象。用handle-first open、no-follow/reparse rejection，再从打开handle验证final path/file identity和size。

### TOOL-COORD-P1-037 · artifact仍是本机路径集合而非内容寻址存储

receipt已有基础，但没有跨worker CAS、dedup、remote upload、checksum-on-read、complete marker和replication policy。建立immutable blob/chunk CAS与manifest，产品目录只作为materialized view；TempStorage式complete marker阻止半成品被消费。

### TOOL-COORD-P1-038 · cleanup与retention缺全局quota/SLO

局部reservation防错删值得保留，但需要按root/session/action/artifact class统计bytes、age和last access，明确high-watermark回收、pinned release evidence、quarantine与deletion receipt。磁盘不足要在启动action前admission，而不是事后清理。

## 7. Codex sync、安装、Tray 与 Web 差距

### TOOL-COORD-P1-039 · Codex spool到1024时静默删除最旧未确认trigger

`_enforce_cap`直接unlink超额pending file，没有drop receipt、metric或控制面告警。满队列时应写durable overflow marker并拒绝/合并可压缩事件；任何丢弃必须在snapshot/health可见。

### TOOL-COORD-P1-040 · spool原子replace后没有目录fsync

文件本身fsync后`os.replace`是好基础，但断电后目录项持久性未被证明。支持的平台对pending/quarantine目录fsync；否则明确durability等级并用journal/sequence恢复缺口。

### TOOL-COORD-P1-041 · Hook为“不阻塞Codex”吞掉错误却没有旁路告警

Hook静默失败是交互可靠性选择，但不能让sync长期不可见地失效。写bounded local health marker/Windows Event Log或独立diagnostic counter；daemon恢复后读取并投影last hook success/error/drop。

### TOOL-COORD-P1-042 · Hook installer把schema 28硬编码为兼容条件

当前daemon schema为65，只读Query动态返回`daemonCompatible=false`。installer必须按descriptor/protocol capability和min/max版本判断，不得绑定内部DB schema；添加current fixture和前后版本兼容测试。

### TOOL-COORD-P1-043 · Hook installer拥有并覆盖整个hooks.json

install/update仅在当前文件与managed object完全相等时视为configured，否则写整份desired JSON；remove又拒绝非完全相等文件。改用Codex支持的结构化hook merge，以stable owner ID只增删自己的entries，保留其他项目hook与字段顺序无关语义。

### TOOL-COORD-P1-044 · TOML feature开关由正则/字符串手术维护

`config.toml`必须通过TOML parser读取和写回，保留未知字段并对duplicate/table/quoted key有正确语义。无法保真时只输出patch建议，不直接覆盖用户配置。

### TOOL-COORD-P1-045 · Scheduled Task运行mutable repo脚本和PATH Python

任务动作指向工作树内 `tools/zircon-session.ps1 start -Automatic`，后续checkout可在未审核升级/部分更新状态执行；Python解析又依赖机器PATH。发布签名/versioned coordinator bundle，任务只启动稳定shim；升级采用staged install、health check和rollback。

### TOOL-COORD-P1-046 · Tray手写HTTP/1.0并无响应大小上限

`coordinator_client.rs`用TcpStream拼request，`read_to_end`到连接关闭，只解析status line和header/body边界，不验证Content-Length/chunking，也发送空Bearer。使用成熟HTTP client或本机IPC SDK，设置header/body上限、deadline、protocol version和authenticated channel。

### TOOL-COORD-P1-047 · Web check会覆盖dist后再验证，不能发现提交产物漂移

本轮check通过，却生成22个新的hashed文件；checked-in dist与source不一致。CI应在临时目录build，比较完整manifest/hash与Git tracked dist，或不提交dist而由release pipeline产出签名bundle。check绝不能通过修改被检查对象来制造currentness。

### TOOL-COORD-P1-048 · Coordinator关键验证完全未进入merge CI

`.github/workflows`没有Coordinator Python、Web、PowerShell smoke或Tray。根`cargo test --workspace`也不包含有独立`[workspace]`的Tray。新增分层门：fast unit、Windows security/process/integration、Web contract/dist、PowerShell Query/install sandbox、Tray `--locked`、migration fixtures与nightly fault/soak；每个job有timeout和artifact evidence。

## 8. P2 与可维护性债务

### TOOL-COORD-P2-001 · `/health` 无认证暴露repo root与内部版本信息

loopback限制降低严重度，但这些信息会帮助恶意本机/浏览器请求定位进程和仓库。匿名health只返回粗粒度alive/protocol；详细health要求observer身份。

### TOOL-COORD-P2-002 · 旧command把内部异常字符串返回调用方

`except Exception`直接返回`str(error)`，可能泄露路径、SQL或进程细节。外部只返回stable code/correlation ID，完整异常进入受限日志。

### TOOL-COORD-P2-003 · 仓库跟踪了零字节 `session_coordinator.db`

真实DB位于`.codex/state/.../coordinator.sqlite3`；源码目录零字节DB没有运行价值且会暗示错误authority。删除tracked占位并以fixture/schema工具生成测试DB。

### TOOL-COORD-P2-004 · descriptor保留永远为空的token字段

字段让consumer误以为有认证，Tray还发送 `Bearer `。v2移除或赋予真实capability语义；compat decoder对v1空token明确标为unauthenticated legacy。

### TOOL-COORD-P2-005 · Hook Query不解释daemonCompatible=false的具体原因

本轮JSON只给布尔值，同时`reviewRequired=false`。返回结构化reason、observed/required versions和repair action，避免operator把“不兼容”误解为未运行。

### TOOL-COORD-P2-006 · current plan与failure状态不再是可信进度投影

主计划仍有37个unchecked与1个checked；5个`status: open` failure record已有“修复结果与回传”。区分origin severity、current lifecycle和verification状态，并由Coordinator projection生成summary，避免文字与状态冲突。

### TOOL-COORD-P2-007 · Web和Tray版本均缺build provenance

0.1.0 package/crate version不足以定位部署字节。UI、Tray、daemon显示Build Set ID、Git tree、schema hash和build timestamp policy；health能诊断mixed version但不泄露给匿名caller。

### TOOL-COORD-P2-008 · Windows symlink安全测试因权限被skip

本轮group记录1项symlink test因WinError 1314跳过。CI需提供Developer Mode/权限lane，或用junction/reparse fixture覆盖等价边界，不能让最关键路径安全测试长期skip。

### TOOL-COORD-P2-009 · 缺少HTTP/JSON/descriptor的fuzz与malformed corpus

当前有很多精确unit cases，但未形成持续fuzz/mutation corpus。对path normalization、Host/Origin、range、JSON schema、runtime descriptor和HTTP framing增加bounded fuzz target。

### TOOL-COORD-P2-010 · 缺少公开的容量与可靠性SLO

现有测试输出部分p95 metric，但没有提交的session/path/event/artifact规模、启动恢复时限、API p95、DB大小、最大停机和数据丢失目标。为每个control domain定义budget，并用nightly benchmark趋势阻断显著回退。

## 9. 目标架构

### 9.1 Control Broker

单machine broker拥有稳定IPC endpoint、OS identity与repo registry；每个repo worker拥有独立state DB和generation。浏览器通过single-use ticket进入observer session，maintainer/elevated role由认证和policy产生。所有transport复用generated codec与一个authorization middleware。

### 9.2 Action Graph 与 Worker

`ActionDescriptor`定义immutable inputs、producer outputs、toolchain/environment、resource class、timeout与receipt。Scheduler发attempt lease/fencing token，Worker在CAS materialized workspace中由ProcessSupervisor执行，terminal receipt一次发布。Cargo、validation、codegen、cook、pack与maintenance以后都共享这条证明链。

### 9.3 Commit Tree Transaction

Session sealed changes形成candidate tree；compile/test receipt绑定tree OID；integration以expected HEAD做ref CAS。任何内部stage使用temporary index，任何验证使用isolated tree。共享worktree/index只作为用户视图，不作为执行沙箱。

### 9.4 Artifact/CAS

Blob、tree manifest、logs、receipts和release artifacts进入content-addressed store；staging通过complete marker提交，下载从打开handle验证identity。local root、remote cache和Horde式worker只是同一接口的backend。

### 9.5 Supervision 与 Delivery

Daemon、worker、Tray、Hook和Web均有build/protocol identity、bounded restart与upgrade rollback。Scheduled Task启动签名shim；mutable checkout只作为输入，不作为已安装控制服务二进制。

## 10. 分阶段重构顺序

### M0 · 立即封锁危险入口

1. 为legacy `/command`加入Host/Origin、1 MiB body、Content-Type、read timeout和bounded concurrency。
2. 生成真实runtime capability，所有mutation默认拒绝空token；maintenance capability强制开启。
3. 暂停自由validation command和caller-written passed，只允许临时hard-coded action allowlist与worker-only result。
4. 禁止shared index snapshot finalize；未迁移路径fail closed。

### M1 · 协议v2与generated contracts

1. 定义principal/role/elevation/threat model和protocol v2。
2. 从IDL生成Python/TS/Rust codec、command catalog和compat fixtures。
3. 迁移CLI、Web、Tray、Hook；v1 mutation返回410，保留只读诊断窗口。
4. 引入single broker或动态repo endpoint，解决6518冲突。

### M2 · Managed Process Supervisor

1. 抽取benchmark Job Object实现，覆盖所有child action。
2. 持久化attempt、PID/creation identity、deadline、job/worker lease和progress。
3. 完成restart reattach或kill-and-retry、tree cancel、shutdown drain与terminal receipt。
4. 增加crash/kill/power-loss/PID-reuse/descendant-pipe fault tests。

### M3 · Action Graph 与验证证明

1. 建立versioned action catalog、input tree、toolchain fingerprint和output owner。
2. validation ticket只引用action/attempt/receipt；人工override使用waived。
3. candidate tree与compile/test receipt hash闭包绑定。
4. 导入Cargo dep-info/build plan，删除手写parser的权威地位。

### M4 · Git/Artifact transaction

1. 所有finalize合并为temporary index + tree OID + isolated validation + ref CAS。
2. 禁止Coordinator恢复外部index lock或重置共享index。
3. 建立CAS、complete marker、artifact manifest、quota/retention和handle-first download。
4. 用并发外部Git、HEAD advance、disk-full和daemon crash测试事务。

### M5 · 数据、服务与交付治理

1. 拆分god object、migration文件和background task owner。
2. 建立DB backup/restore、migration fixture、transaction/SSE/maintenance SLO。
3. 发布签名versioned daemon/Tray/shim，完成install/update/rollback。
4. 把Python/Web/PowerShell/Tray/security/fault/soak纳入CI矩阵。

### M6 · 分布式构建演进

1. 让本机worker先使用agent enrollment/lease/CAS协议。
2. 增加remote worker capability、artifact replication和scheduler fairness。
3. 以BuildGraph/Horde概念验证多agent，但保留Zircon自己的Rust/Cargo/Windows约束。

## 11. 验收门

- 未认证GET只能访问最小alive/bootstrap；任意mutation得到401/403，恶意Host/Origin/simple browser POST均无法产生side effect。
- 每个passed ticket都有唯一worker attempt receipt，数据库约束禁止无receipt passed；旧attempt迟到提交被fencing拒绝。
- candidate seal、validated tree和commit tree OID完全相同；修改任一blob会使receipt失效。
- 所有执行进程均在managed process tree中；cancel/timeout/daemon crash后没有存活后代，也没有永久running row。
- Coordinator从不覆盖、snapshot restore、reset或自动删除用户shared index/lock；外部并发stage内容零丢失。
- 两个不同repo可同时运行并被各自Tray/Hook准确发现，无端口或descriptor串线。
- Web currentness gate在source改变而dist未更新时失败，且check不修改tracked文件。
- Python full suite有分层timeout并全绿；并行lane不共享全局patch/plan state；snapshot当前失败被修复或按新合同重写。
- Tray `cargo test --locked`、PowerShell install/update/remove sandbox、schema 1..current migration、Windows process/security tests成为required CI。
- 10k Session/100k lease path/1M event规模下有已提交p95与恢复SLO；SSE与maintenance不会造成DB写饥饿。

## 12. 本轮非目标

- 本报告不修改production code，不把审查当修复完成。
- 不因Tray 36项通过而推断daemon/control安全；它只证明当前Tray局部合同。
- 不因Web 70项通过而推断dist current；动态重建已证明两者漂移。
- 不把并行出现、串行消失的ctypes错误写成Python 3.14产品bug；它首先是测试隔离证据。
- 不删除本轮前已存在的`.git/index.lock`，也不调用Coordinator lock recovery。
- 不要求第一阶段立刻部署Horde规模；必须先完成本机身份、action receipt、process fencing和tree-exact validation，否则分布式只会放大不可信状态。
