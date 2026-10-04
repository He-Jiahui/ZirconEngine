---
status: in_progress
source_recheck_required: true
plan_sources:
  - docs/plans/astra/optimize/01-review-and-repair.md
  - docs/plans/optimize/zircon_hub/03-marketplace-account-auth-organization-cloud-repository-provider-review.md
---

# 本地账号团队商城与云同步服务

## 目标与现状

用户已批准真实本地可部署服务；不要求公网运营或商业支付上线。Hub03 的原 P0-01..05、
P1-01..72、相关本地能力 P2 和 G01..G32 是细化合同来源；涉及公网支付、税务、多区域
运营的内容单列范围，不把本地交付当作这些能力的完成。

当前 `zircon_hub/Cargo.toml` 已拆分 `desktop`、`account-broker`、`local-service`
features，并提供独立 `zircon_hub_service` binary。`src/service/` 已有 axum、SQLite、
OIDC、组织授权、签名目录和 CAS 实现；`src/account/` 已有 PKCE、callback 校验、
Windows 凭据存储及 refresh/logout broker。schema v4 永久撤销已失去授权的邀请者
发出的旧邀请，已有重新授权后旧邀请仍拒绝的回归源码。
schema v5 新增 SQLite/CAS UUID 与密钥指纹绑定，并持有数据库和 CAS 的进程 owner。

本地服务已有受管 Windows 历史证据：2026-09-07 封存输入上的 `local-service` lib 测试
78/78、独立 service binary 构建退出码 0，以及记录为 40/40 的真实 Keycloak、Chromium
和 Rust service 协议探针。探针没有执行原生 account broker 或 Tauri；原证据目录现已不可读取。
2026-09-25 的直接检出工作区作业记录 `local-service` lib 测试 84/84 和 release 构建退出码 0；
coordinator 保留了两项作业的命令与成功退出码，但没有独立保存本轮源码摘要和可复核的测试计数。
详情见[本地服务验证记录](../../optimize/01/2026-09-07-hub-local-service-validation.md)。
真实 Keycloak/Hub service/desktop 三进程、原生凭据 broker、S4 安装、S6 同步、部署恢复及性能
仍按 S0-S7 逐项验收。`tauri_app` 的 state
epoch/revision 由 `01-state-publication-order.md` 负责，不新增第二套版本 authority。

## 责任与落点

独立 Rust 服务进程放在现有 `zircon_hub` package 的独立 binary/feature 中，使用
axum、SQLite 和 CAS 文件存储。`src/service/` 按 config、identity、authorization、
storage、organization、catalog、cloud、http 分域；root 只接线。独立 binary 不启动
Tauri window，也不把服务密码/token 传给 WebView。客户端 broker、provider session
与页面投影仍由 Hub desktop owner 负责。

Keycloak 作为独立 Windows ZIP 进程，标准 OIDC 管理用户、密码、登录和会话撤销。
桌面 client 默认启用 Keycloak 26.7.3 的 `basic` client scope，确保 access token 含有
服务端要求的 `sub` claim；其余授权仍由 issuer、audience、签名和服务端 membership 决策。
使用官方协议库验证 issuer/audience/signature/expiry，桌面 Authorization Code + PKCE
与系统浏览器，限制 loopback callback 生存期并验证 state/nonce。不得实现自制密码库、
把 JWT decode 当 verify、使用 Git author 作为 principal，或将 bearer 写入 config/log。

主参考 Unreal OnlineServices 的 Auth/Commerce/UserFile/TitleFile 合同和 BuildPatch 的
安装事务职责；次参考 Godot asset_library 的分页目录、详情、下载和失败处理。二者均
不能替代 Hub03 CAS conflict 和不可信 native package 隔离的额外约束。

## 依赖与实施切片

| 顺序 | 最低 owner 与具体行为 | 退出条件 |
|---|---|---|
| S0 | 服务 config、版本、生命周期；SQLite migrations、事务、审计和 bounded HTTP；loopback-only 本地部署 policy | 未配置身份/密钥拒绝启动，readiness 仅数据库及身份就绪后发布，shutdown 有界，schema mismatch 不破坏旧库 |
| S1 | OIDC verifier 与 Hub Windows credential broker、account generation、PKCE、refresh/logout | 真实 Keycloak 登录/刷新/撤销；过期、错 issuer/audience、坏签名、重放、并发 callback 拒绝；零 token 泄漏 |
| S2 | qualified org/member/project、role/policy revision、invite、服务端 resource-action authorization | A/B tenant 全部 API 隔离；stale revision 零写入；撤权立即作用于新请求，带审计终态 |
| S3 | signed versioned catalog、publisher/package/release/artifact/offer、entitlement、分页查询和缓存 | 修改、过期、回退及未知/revoked key 拒绝；免费包也需 license/trust；10万项不全量传前端 |
| S4 | 复用 Plugin Package Service 的 lock、依赖闭包、下载、验签、stage/install/rollback 和 Editor activation | 前置信任通过前零执行；任意失败保持旧 inventory/lock；Installed/Enabled/Running 不混用 |
| S5 | ProjectSnapshotManifest、租户授权 CAS blob、revision compare-and-swap、quota/retention | 同 base 并发提交恰好一个成功，另一个 typed conflict；无引用丢失或跨租户 blob 读取 |
| S6 | Hub 扫描/离线 journal、同步状态机、验证后 staging apply、冲突 artifact 和恢复 | pull 故障/崩溃可恢复；不覆写本地修改；VCS/scene merge 有领域 owner，不能 last-writer-wins |
| S7 | 全路由 UI、provider health/operation receipt、CSP/origin、真实 Windows 部署和性能 | 正常 UI 完成本地账号到团队、发现/安装、双客户端冲突/恢复/退出；所有 G 门逐项证据 |

每个切片先写明确文件 ownership、迁移与测试范围，再实施。API 和持久化格式由唯一
服务 owner 版本化。S4 依赖 native trust admission，安全前置不满足时仅允许可信目录
查询，不将包复制进自动扫描/执行目录。S5/S6 使用独立 project snapshot authority，
不混用 Runtime SaveGame、engine release update 或 Editor live collaboration。

## 数据与安全合同

- AccountHandle 使用 provider/environment/subject/generation；display name/email 只是
  属性。Authorization 每次在服务端按 principal/resource/action 和新鲜 membership
  决策；客户端按钮、Git config、任意 header 均不能授予角色。
- SQLite 使用显式版本迁移、foreign keys、transaction、busy timeout 与参数绑定，阻塞
  I/O 在有界 worker 执行；组织变更、CAS publish、审计与 receipt 原子提交。错误不能
  暴露 SQL、bearer 或机器绝对路径。
- CAS 只接受校验过的 digest 名称，限定单 blob、manifest entries、总快照、并发和磁盘
  quota。临时写、回读 digest、flush 后原子发布；未提交内容不能从目录中误显为完整。
  相同 bytes 可物理去重，但下载仍校验 tenant/object 引用授权。
- ProjectSnapshotManifest 使用稳定路径、digest、base/head、engine/package lock 和
  ignore policy；拒绝绝对、父目录逃逸、大小写碰撞、链接与重复路径。secret/cache/
  generated 文件不自动上传；上传先 blobs 后 CAS head，冲突不覆盖任何一方。
- 签名使用现有或成熟密码库，不发明签名序列化算法；明确 key ID、canonical bytes、
  expiry/revocation/trust source。密钥留在 OS credential owner 或服务受控密钥文件，
  不进仓库、命令参数、日志、DTO 或测试 snapshot。
- 本地开发 HTTP 只允许明确配置的 loopback；非本地 endpoint 必须 HTTPS 并遵循 origin
  allowlist。后端请求关闭任意 redirect/SSRF，WebView 不能以远程内容触发特权 IPC。

## 验证与部署

合并一次 compatible package/feature correctness 批次，包含 service Router/SQLite 事务、
fault injection、tenant/permission、signed catalog、CAS conflict 和 client broker 测试；
单独执行 release 规模和实际三进程 Keycloak/Hub service/Hub desktop 验收。验证 lane
封存 Cargo.lock、realm/client policy、服务配置非秘密部分和依赖指纹。回执后立即推进
后续切片，不持续监控编译。

Windows 部署说明必须含可复现构建、官方 Keycloak ZIP/JDK 前置、realm/client bootstrap、
数据库/CAS 目录、启动/停止、health/readiness、备份/恢复和双客户端冲突复现。配置中
不保留演示密码或长效 token。外部下载校验官方摘要，不把下载成功当产品通过。

性能继承 Hub03 有效门槛；100k catalog cursor、并发 tenant 操作、大 project blob/manifest
及长离线恢复测 p50/p95/p99、吞吐、RSS/磁盘、HTTP/SQLite 队列和错误率。新增热点
等价 baseline p95 改善目标 20%，正常路径回退不超过 5%。W8 仍需真实 Hub 帧和交互证据。

## 状态与产出记录

| 里程碑 | 范围 | 状态 | 完成日期 | 证据 |
|---|---|---|---|---|
| S1/S2 desktop candidate | 原生配置加载、epoch/generation 与持久化 operationsRevision 拒绝陈旧响应；分页资源；创建组织/项目、接受邀请；重启恢复、同编号查询/重试、终态确认；日志读取失败停止新写入 | implemented_pending_validation | 未完成 | `zircon_hub/src/tauri_app/account_commands.rs`、`src/account/operations/`、`web/src/account/`；本机 Chrome 顺序执行 `account_state.test.mjs` 与 `account_admin.test.mjs` 共 66/66、零跳过，40 张账号/团队浏览器夹具截图位于 `target/astra-hub-account-browser-20260921/`。这些证据不替代真实 PKCE、DPAPI、凭据和服务验收 |
| S3/S4 catalog UI candidate | 授权目录、许可确认、带回执的安装、未知结果恢复和库存刷新已接入 Hub 服务目录 | implemented_pending_validation | 未完成 | `zircon_hub/web/src/catalog/service/`、`web/tests/catalog_service_browser.test.mjs`；本机 Chrome 18/18、零跳过，覆盖中英文和 360/768/1280/1920，16 张浏览器夹具截图位于 `target/astra-hub-catalog-browser-20260921/`。真实 Keycloak/签名包、服务安装事务、回滚及 Editor 激活尚未由此验收 |
| S0/S5 service candidate | 加密 CAS、当前租户授权、revision CAS、配额、失败原子性、恢复；schema v5 root/key/DB 绑定；取消调用者后仍追踪数据库任务；HTTP/DB 共用 20 秒关闭期限 | implemented_pending_validation | 未完成 | `zircon_hub/src/service/cloud/`、`service/storage/jobs.rs`、`service/lifecycle.rs`；受管封存输入 `local-service` lib 78/78，后续直接检出工作区作业记录 84/84、退出码 0；源码身份限制和未闭合产品门见[验证记录](../../optimize/01/2026-09-07-hub-local-service-validation.md) |
| S6 desktop CloudCommit sync candidate | generation/account/project fenced head; immutable bounded local snapshot; typed target/policy/build-set package-lock provenance and canonical digest; verified staging/apply and explicit durable conflict recovery | partially_implemented | 未完成 | Normal Runtime package service and Hub pinned helper supply `ProjectPackageLockState::Present` from the host policy/index, target and installed generation. New commit/head/stage/apply consumers require Present and a matching digest; legacy optional v1 receipts remain parseable. Provider witnesses do not grant account/cloud authorization. `local-service` explicitly activates the shared runtime-interface DTO dependency. These current-source contracts are uncompiled and their provider-unavailable/generation-mismatch-before-persistence and real helper/commit/head/retry/apply matrices remain unexecuted. The earlier run6 service binary and Desktop R2 source generation predate these S6 changes. Exact-scope Unknown/orphan cleanup, native trust ownership, Windows two-client recovery/cancel/restart, responsive multilingual UI, performance and Jenkins acceptance remain open. |
| S6 bounded blob transfer candidate | Rust broker performs organization/project-scoped reads and uploads in the signed-in generation; each request is limited to 16 MiB and checked against lowercase SHA-256; downloaded bytes stay out of WebView, ambiguous writes remain Unknown, and content-addressed PUT accepts identical retries | partially_implemented | 未完成 | `zircon_hub/src/account/cloud/blob.rs` and `cloud/blob/tests.rs`; seven new native regressions remain unexecuted. Prior service static contracts (12/12) and Rustfmt are source evidence only. The candidate now participates in the S6 sync path above; broker Cargo, actual Tauri/Keycloak integration, restart recovery and two-client acceptance remain pending |
| Windows deployment candidate | 固定本地 Keycloak realm、配置、启动、readiness、停机备份和空目标恢复 | implemented_pending_validation | 未完成 | `zircon_hub/deploy/local/`、`docs/crates/zircon_hub/local-service-windows.md`；真实 Keycloak 与协议探针 15 项通过；2026-10-03 run6 已执行隔离 Keycloak/Hub service API 闭环，但 desktop/broker auth 未观察，原生三进程 UI/S1 仍未验收 |

2026-09-07 续接边界：桌面 mutation 在发送前写入 Windows 当前用户 DPAPI 日志；同一
qualified identity 重启后恢复，未知项不得确认移除。19 项前端状态回归、8 个带恢复项
的中英文宽度布局、取消登录和恢复/重试/确认浏览器流程已实际通过；结果来自模拟 IPC，
不能替代 Rust 和真实服务。该批 29/29、零跳过及生产构建已再次完整执行通过。
retry/ack 跨进程准入与按身份版本已完成源码修复；日志 v1 显式迁移 v2，历史身份版本
受 16 MiB 文件预算约束，不再因固定 1024 个历史身份而永久拒绝新身份。原生测试仍待执行。
角色变更、所有权移交、创建/撤销邀请已有浏览器验证的 UI 实现，见
`03-team-administration.md`；原生闭环、S4 真实安装和 S6 同步仍待验收。
S5 的 schema v6 保留策略与分阶段 GC 正在修复独立复审问题，见
`04-retention-recovery.md`；密钥轮换和掉电验证仍待办。
服务 20 秒期限已覆盖启动 owner、axum 与已登记数据库任务；Ctrl-C 在配置读取前注册，
最后使用 2 秒 Tokio shutdown_timeout，真实进程退出及掉电后的持久化结果仍待执行验收。
Windows DB 在 SQLite 打开与迁移前拒绝已有 hardlink，路径句柄与连接共同存活，取消
调用者不释放仍在运行的 job 所持有的句柄。其他平台尚无同等 DB 路径准入合同。

SQLite 旁路文件复审已补充：打开前同时检查已有 `-wal`、`-shm`、`-journal`，拒绝
链接、非普通文件和公开 ACL；主文件及目录的 owner/allow ACE 只准当前服务用户、
SYSTEM 和 Administrators。目录继承链逐层检查至 protected DACL，使后续 SQLite
创建的旁路文件受同一权限边界保护。检查后的旁路句柄释放，允许 SQLite 正常恢复和
删除日志；主文件与父目录继续随连接存活。拥有同一 Windows 用户或管理员权限的
进程属于受信任 OS 边界，不能据此宣称抵抗同用户任意文件写入。
部署 Configure/Backup/Restore 创建新私有目录，拒绝不符合要求的已有目录且不改其
ACL。`windows-database-access-probe.json` 六项和
`deployment-private-directory-probe.json` 八项已在 Windows 实际通过，含热回滚日志
恢复、WAL 提交/删除、公开目录拒绝、重解析父目录拒绝及已有 ACL 不变；独立静态
复审无新 finding。Rust 回归仍未执行，这些原语和部署脚本证据不升级产品状态。

Windows 系统 API 探针实际证明：仅持有目录句柄不能阻止空目录被改成 junction。
共享 `src/file_io/windows.rs` 因此使用 `NtCreateFile` 的 `OBJ_DONT_REPARSE`，在每次
打开/创建时拒绝整条路径中的重解析点；配置读取、账号日志/凭据锁和 DB 准入均消费它。
探针确认 junction 后创建返回 4395 且不写入目标、普通创建成功、SQLite WAL 可在保护
句柄持有期间工作。证据为当前任务 `hub-account/windows-path-sharing-probe.json`；这是
系统原语验证，不是 Rust 编译或 DPAPI/SQLite 产品验收。对应 Rust 测试已加入待验证批次。

2026-09-07 身份部署实测：校验官方 SHA-256 后解包 Keycloak 26.7.3 与
Temurin 25.0.4.1+1，使用仓库部署脚本在任务私有目录中 bootstrap/import/start。
真实 Chromium 授权码 PKCE、RSA/JWKS 身份与 service audience、Basic 机密客户端
introspection、刷新及注销共 15 项通过；注销后 introspection inactive 且 refresh
返回 invalid_grant。探针清理合成用户，Keycloak 正常停止，8080/9000/8480 端口关闭。
这次运行定位并修复 desktop 缺少 `basic` scope 导致 access token 没有 `sub`，以及
刷新 ID token 省略 nonce 被 broker 错拒的问题。首次登录仍严格要求 nonce，刷新仅在
返回 nonce 时比对原值；issuer/audience/signature/expiry/subject 等校验继续执行。
缺少 basic scope 的部署副本已被实际 ValidateArtifacts 拒绝。

证据目录为当前任务
`C:/Users/HeJiahui/.codex/visualizations/2026/09/06/01a0780b-97a2-7380-9b15-1e90591a6b4b/hub-local-runtime/`：
`oidc-evidence.json` 记录 2026-09-07T05:40:26.969Z 的 15 项结果、官方压缩包摘要与
部署/身份源码 SHA-256；`oidc-probe.mjs` 为可重跑协议探针，
`keycloak-login-1280.png` 为真实登录页截图，`deployment-subject-evidence.json`
记录配置正反例与端口关闭检查。`nativeBrokerExecuted` 明确为 false，不能据此接受
Rust broker、DPAPI、Hub service、Tauri 或完整 S1。对应刷新 Rust 回归源码尚待受管执行。

2026-09-07 外部输入已通过独立 Git index 封存为
`zr_vm` commit `1966223f74da1d7df8ace543ca98588594cca34e`，包含 183 项 dirty 输入；
原 HEAD、共享 index 和工作树保持完整。使用既有 `validation-copy materialize-cargo`
显式 commit 入口，未修改协调器。最初 73 项 Hub snapshot 2885 遗漏其他已变更的
workspace manifests，作业 `3a20670807bb409ba5514855e7955687` 在 `--locked` metadata
阶段失败，执行测试数为零。补入匹配的完整清单、两个新增 package 入口和 Hub
build script 后，84 项 snapshot 2890 对应作业
`307ab5915f03467c9bc19c0ab27f9d1c` 已成功 materialize。执行命令为
`cargo test -p zircon_hub --no-default-features --features local-service --lib --locked`；
执行请求 `6e8be07eed7646258dea37f3ec66241a` 已受理；Cargo 作业
`bda4833f3a164c2eac37a2128e0b7c98` 的 run `d3e1b78d36ea4c83927ad57a0dbbc8c9`
实际执行后报告 70 项编译错误，尚未执行测试。其 stderr 保存在协调器对应
`cargo-runs/<job>/<run>/stderr.log`。

后续输入审查发现共享 `src/file_io/` 的五个新源文件也需封存；已纳入 snapshot 2891。
编译修复启用 rusqlite 0.40.2 已有的 `fallible_uint` feature，其转换保持超限/负值拒绝；
`JobCensus` 的可见性与 service 内消费者一致。OIDC discovery 回调的 future 持有
克隆的配置和 HTTP client，消除 borrowed callback 在 Tokio spawn 的 Send 推断错误。
这些是 2026-09-07 首轮编译失败及当时待重跑的历史过程；后续封存输入的通过结果与
2026-09-25 直接检出工作区作业见[本地服务验证记录](../../optimize/01/2026-09-07-hub-local-service-validation.md)。
`service_authority_contract` 改用 TOML 结构核对 feature 依赖集合，包含现有 Unix libc
依赖，避免整行文本匹配误报。静态检查、浏览器 fixture、受理回执及本地服务 Rust 测试
均不代替尚未执行的原生 account broker、Tauri、S4 安装及 S6 同步产品验收。

## 2026-10-03 本地服务/API证据（产品门开放）

私有 run6 在新的 Keycloak 26.7.3 runtime、SQLite 与 CAS 状态目录上执行了当前普通 Hub
service API。真实 Authorization Code + PKCE S256 登录取得签名 RS256 access token，并对
issuer、`zircon-hub-service` audience、JWT signature、JWKS 和 subject 做了验证。23 个 API
类别均有脱敏回执：health、organizations-empty、organization create/replay、project create、
members、projects、cloud blob upload、signed catalog publish、catalog artifact upload、
licenses-before、license accept、catalog manifest、catalog artifact download、cloud commit、
post-commit blob download、commit replay、typed conflict、cloud head、post-logout old-token
rejection、post-restart cloud head、post-restart cloud blob、post-restart authenticated list。
Refresh 成功，logout 返回 204，旧 access token 返回 401。服务从相同 SQLite/CAS 路径重启后，
cloud head/blob 语义读取成功；SQLite 字节未保持相同（审计/receipt 行会写入），因此这不是
字节相等证据。service 与 Keycloak 都通过自有 dedicated-console Ctrl+C 正常退出，8080、
9000、8787、8480 最终关闭。

这项回执只证明 service/API 与本地身份闭环，不证明 desktop UI、原生 credential broker/
Windows Credential Manager、安装/rollback、原生 S6 双客户端同步、掉电、backup/restore、
performance、Jenkins 或完整 S1。不能把它写成三进程 UI 已通过。证据：
[C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-native-identity-execution-r2/terminal-result.json](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-native-identity-execution-r2/terminal-result.json)，SHA-256
`b50ec983fa9996cec18cdcf0dc7df11bd51262e2cee633f03ff8e8697eebd5ac`。

独立回执复核 `598059ff47c1a151ed6ed5686005f67bf920d155eb224fb050455099d201539e`
确认16个sourceManifest输入与当前字节相等，并将真实已执行业务路径限定为局部证据。
该复核指出的三个负向边界已取得独立的实际HTTP证据：2026-10-03
`03:22:11–03:24:01 UTC` 的 r10 在旧编译二进制 `45dcaaa2346d830d1f764f394f31ae325a818d46a8189503d747dbb130364643`
上执行13项负向断言。无效signature/kid/malformed bearer/audience/issuer/expiry均返回401；
跨租户read/mutation、成员自提权式撤销、撤权后read/mutation及提交前blob GET均返回403；
冲突返回409。拒绝操作前后的成员和head语义hash相等。提交后的payload读取、相同结果
replay和head revision 1另有实际通过断言；r10不重复PKCE wrong-verifier/code-replay，复用
先前实际r6证据。service/Keycloak正常停止，四个端口最终关闭。精确终态
[terminal-result.json](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-native-identity-negative-verification-r10/terminal-result.json)
SHA `72ee2d32697f009d483ca70fda52e42c905d31a75167491fd832c229b21b5a3e`；原观察
SHA `35040df77cf11326ec28a2517ed5980614f6ee43b0c40f44bdb5d730adf13dcf`。
这些证据只覆盖旧二进制的服务HTTP行为，当前S6 typed package-lock及四路径桌面编译修复
尚未由它验证，S0–S7完整产品、desktop/credential broker、性能和Jenkins仍开放。
旧r1 warm-build原始stdout及原运行脚本hash不可恢复，不能以较晚r5回执冒认旧封存。
当前服务业务回执不会提升整项S0–S7、原生桌面、Jenkins或产品/性能状态。

复核输出在首次交回后发生元数据/结构覆盖；首次 `5bba985c…` raw 未由 Root 保存，
目前只绑定 `598059ff…` 的原始字节副本及 Root 观察 `0ab4cf20797bab467efdc6d12e0b61f5fd67c45ddfa1010f4868864eb2557a52`。
旧封存不可凭摘要或hash重构；上文原始run6 service回执 `b50ec983…` 保持不变。


### 2026-10-03 当前 R3 普通库编译与实际回归

在不可变输入 `a21f521e78b988793e99eae65e8226697869befc124978731b6cec6ad2f9c590`
上，`+1.94.1 check --locked -p zircon_hub --lib --bin zircon_hub_service --tests
--no-default-features --features local-service -j2` 实际 exit 0；随后普通 `test --locked
-p zircon_hub --lib --no-default-features --features local-service -j2` 实际通过96项，
0失败、0忽略，耗时49.86秒。完整日志保存于 `D:/cargo-targets/zircon-local/
hub-s6-current-validation-20261003-r3/scratch`，test stdout SHA
`0b39df707eb618ac6e4b2d9650cd25ca6867cca933d3fd3a095193125ab8b0ed`；输入前后
聚合哈希 `495930d1b674966b514256f3da84ab59d898e8c9cf2befc0ab3974b97bda2157` 相同，
封存输入0字节漂移、0 reparse，编译产品留在 drive-root `D:/cargo-targets`。

实际集合是80项 `service::` 与16项 `file_io::`。预期清单中的
`secure_bounded_read_rejects_a_symlink_target` 不存在于这代全部Rust源码；保留81项旧清单，
单独登记这一集合差异，不把96项结果写成81项service均已执行。实际集合包含Windows
父路径替换、junction admission、hardlink及WAL/SHM reparse拒绝、ACL、数据库路径守卫、
OIDC缓存/校验、组织权限/撤销、catalog授权与cloud冲突/保留/恢复的测试。

同批 Runtime Interface 普通库测试已经执行，结果783 passed、7 failed、101 ignored
（共891项）。7项失败涉及template descriptor摘要、manifest依赖边界、ABI owner文件目录、
两个design-token集合、生产来源扫描与批量dirty-node测试。该条命令只保留PTY终态文本，
完整原stdout/stderr未保存；不重构原日志，不为补日志重复执行。6项package-lock实际通过名
保存在原终态回执中。最低责任模块回修正在继续，整批验收开放。

精确终态 [execution-receipt-r3.json](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-s6-current-validation-20261003-r3/execution-receipt-r3.json)
SHA `b58d847282c49e7056977ad99a8a6053d1bc74e5c7d17a89f5f050c38ceec714`；Root完整日志
复核 `dbffdbe72a87c0b4431f3bb5f52983923ac0915dbb935320a73901452f90370d`。
证据只适用于当前 R3 普通库编译/测试。当前服务二进制随后在相同封存输入上由 D 盘正常
Cargo 编译取得 exit 0；最初 runner 的探测路径错误及实际产物分别记录于 [Hub actual binary](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/hub-r3-d-binary-root-terminal-reconciliation-r2/root-terminal-reconciliation.json)。
当前 23 条正常 API 与 13 条负例尚未通过，[Hub native review](C:/Users/HeJiahui/.codex/visualizations/2026/09/30/01a0f033-ce80-77e2-86f0-eed5bfbb5926/B/hub-r3-guarded-native-independent-review-u-r1/review.json) 的路由断言、干净身份服务
输入及信号前进程身份问题正在新代次修复。desktop、credential broker、原生安装/双客户端同步、
UI 全部路由与窗口、backup/restore、性能、Jenkins 和 S0–S7 产品门继续开放。
