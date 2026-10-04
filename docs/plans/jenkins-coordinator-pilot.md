---
related_code:
  - tools/jenkins_pilot/
  - tools/jenkins_coordinator.py
  - tools/jenkins_tray/
  - .codex/skills/zircon-dev/scripts/validate-matrix.ps1
implementation_files:
  - tools/jenkins_pilot/
plan_sources:
  - "user: 2026-10-01 implement isolated Jenkins coordinator pilot; retain formal source acceptance"
  - "user: 2026-10-02 keep the old service retired and establish independent Jenkins acceptance"
tests:
  - tools/jenkins_pilot/tests/
  - tools/session_coordinator/tests/
  - docs/plans/milestone-validation-policy.md
doc_type: milestone-detail
---

# Jenkins 协调器试点与独立验收

> **当前方向：旧服务保持退役，建立 Jenkins 独立验收路径。** 2026-10-02 用户已明确授权本方向。禁止恢复旧客户端、服务、数据库写入、租约或队列；下方原协议和旧结果仅保留为历史。独立实现已完成 I1–I6 的固定输入试点验收与独立审查；本计划不授予提交、推送、发布、删码或生产切换许可。

## 独立路径的职责与验收

Windows 托盘、独立协调器启用及功能测试入口见 [操作流程](../cli-and-tooling/jenkins-tray.md)；当前绑定与启用状态由 `.codex/state/jenkins-coordinator/active.json` 和对应新回执拥有。下方固定输入试点及历史 gate 保留原范围，协调器功能启用不倒填全工作区、里程碑或生产迁移接受。

Jenkins 负责排队、节点选择、构建身份及归档。`tools/jenkins_pilot` 独立拥有封存输入、Windows Job、存储边界、缓存写入排他锁和回执核验，不依赖退役协调器的运行时或数据库。私有资源保管进程只维护本次物理根与固定执行器的生命周期；它不决定队列或验证模板。

新请求采用 schema 2 和 `managed-cargo-check-v2`，历史请求、数据库、锁与归档原样保留。每次执行绑定 session/request/attempt/generation、bundle/input/source 摘要、可信 driver、实际 Cargo 命令与工具链、缓存兼容性和原生进程身份。只有确认实际退出、Job 活跃进程为零及两条管道 EOF 后才能释放缓存写权并产生通过证据。未知或缺失证据必须拒绝验收。

Cargo 的物理 generation 使用兼容性摘要前 20 位缩短路径，状态和回执仍保存完整摘要，发生碰撞必须拒绝。执行前检查 MSVC build-script 输出路径预算；注册表的可选复制只允许本独立根内已有完整释放证明的 generation，持有来源和目标的 writer 锁，只复制 index/cache，保存逐文件摘要并随 Jenkins 构建归档。历史长路径 generation 不移动、不收养旧协调器缓存。

| 验收项 | 必须取得的证据 | 状态 |
|---|---|---|
| I1 独立部署 | 无退役命名空间运行依赖；物理输出均在 D/E/F 根 cargo-targets；controller 零槽、agent 一槽 | passed：独立根资产、当前封存 driver 与节点实测通过；两次恢复均保留完整所有权链及原 state，controller 零槽、agent 一槽 |
| I2 封存输入 | dirty/new/deleted 与外部依赖闭包；固定输入还原；漂移与摘要失配拒绝 | passed：受影响 snapshot/planner 18 测试通过；111 项 Cargo 封存输入已实际还原 |
| I3 请求与归档 | 幂等提交、原构建关联、尝试隔离、缺档和旧回执拒绝 | passed：当前 driver 的 Python build13 SUCCESS 且独立验收；真实重复提交仍仅关联 build13，六类坏归档/身份/冲突提交反例拒绝；旧证据保留 |
| I4 Cargo 与缓存 | 实际 locked package check；完整终止回执；同输入跨构建缓存复用；单 writer | passed：baseline005 与 builds8/9 整链通过，归档严格核验 accepted；相同物理 generation 跨构建复用，独立释放证明和真实第二 writer 拒绝；旧失败保留 |
| I5 故障收敛 | 队列取消、运行取消、controller 重启、agent 断线；后代退出、回执持久化、资源释放 | passed：当前 driver 的 queue7 取消无执行，builds10/11/12 运行取消/Controller 重启/Agent 断线对账完成；native Job0/EOF、精确后代退出、持久回执及资源释放齐全 |
| I6 对照与维护成本 | 相同输入与环境的独立 baseline/Jenkins 结果、阶段耗时及实际保留代码范围 | passed：同 driver 对照、逐文件当前源码清单和独立证据审查完成；维护范围与运维限制见下文 |

验证批次：先独立支持层与受影响 Python 合同测试，再跑固定 `zircon_reflect_derive` Cargo 批次、同输入复用及故障场景，最后核对归档与独立审查。命令通过只授予该固定输入的验证证据，不自动授予 MVP、生产迁移或提交权限。历史未完成项继续保留，禁止用新的结果倒填旧 gate。

## 当前接受结果与证据

本次接受的是独立 Jenkins 路径的固定输入试点：Python build13、Cargo builds8/9 均为真实 SUCCESS，`verify` 返回 `accepted: true`、`acceptance: jenkins-validation`、`scope: sealed-input`。故障探针只验证资源收敛。未提交、推送、发布或删码；旧数据库、排队任务、锁、归档与失败回执保留，生产迁移仍 pending。

当前独立运行根为 `D:\cargo-targets\zircon-jenkins\jenkins-pilot-independent-001-abb0b3960a4a29e420358419ea5687a7`。此位置是本次实测实例，不能作为新分配默认值。唯一汇总记录为该根的 `evidence/independent-pilot-acceptance-001.json`，包含以下文件的完整 SHA256、当前源码清单和最后节点/所有权观察：

| 证据文件（相对当前 root） | 证明范围 |
|---|---|
| `evidence/independent-authority-recovery-002-result.json`、`evidence/runtime-recovery-independent-003-after.json` | 原 state/历史链与物理根保留；独立 controller0/agent1，旧服务未恢复 |
| `evidence/independent-final-candidate-009.json` | 当前 59 个 Python 文件摘要、无退役运行时导入；Pipeline 单独绑定 driver |
| `evidence/python-independent-004-acceptance.json`、`evidence/request-and-archive-checks-independent-002.json` | build13 严格验收、真实幂等关联与六类拒绝反例 |
| `evidence/cargo-independent-002-acceptance.json`、`evidence/cargo-independent-003-acceptance.json` | builds8/9 当前封存输入与六项归档严格验收 |
| `evidence/cargo-independent-comparison-001.json`、`evidence/cargo-baseline-independent-005-live-writer-proof.json` | 相同输入/环境/driver/物理 generation、三个独立释放证明、真实第二 writer 拒绝与阶段耗时 |
| `evidence/fault-{queued,running,controller,agent}-independent-002-result.json` | 当前 driver 四个真实故障场景；精确退出、Job0/EOF、原请求对账及资源释放 |
| `evidence/cargo-baseline-independent-003-terminal-reconciliation-001.json` | 原失败文件未改写；仅新增确切资源释放证明，原尝试仍 failed |

受影响支持层验证按失败逐层修复并重跑：最后原生身份/进程、恢复链与交接组合 43 项通过，受限查询句柄 7 项通过；Cargo、快照、归档、资产与启动边界沿用相同相关源码的已通过回归。独立审查核对当前源码、原回执、归档摘要、释放记录和故障证据，无 P1/P2 阻断。通过证据没有改写历史 gate，未授予 main、MVP 或生产迁移验收。

## 当前测量与维护范围

2026-10-02 实测固定 `zircon_reflect_derive` 输入，共 111 项，source digest `8cc8f2b19cdeda7c7c13db419fb409787ea803e5bf265367492ce8b7bf6022f7`；当前 driver input `01767c0fa23060c93563607bf46a896424442e56f13a407eac51e727b28a2af6`。三次执行的源码、实际命令、工具链、配置、环境、完整缓存兼容性摘要和物理 generation 一致。验收范围为 `sealed-input`，不代表当前 main、完整工作区、MVP 或生产迁移通过。

| 执行 | prepare 秒 | run 秒 | Jenkins build 总秒 | 验收核验秒 |
|---|---:|---:|---:|---:|
| 独立 baseline005 | 49.616 | 3.053 | 不适用；本地整链 58.694 | 本地 collector 已计入整链 |
| Jenkins build8 | 31.935 | 7.447 | 88.169 | 19.262 |
| Jenkins build9 | 277.884 | 13.359 | 373.466 | 15.970 |

这是一份 warm baseline 与两次顺序构建的观测，未控制 OS 文件缓存和系统负载；不能据此声称速度改善。build 总时长还包含交接、准备外围、归档和流水线开销，不能把与 prepare/run 的差额全部归因于归档。上传完成后到 build 的间隔分别约 6.678/5.541 秒，只是排队区间的下界。Cargo 输出保留 incremental session 回收时的文件占用警告；长期缓存增长与自动回收仍需生产运维验收。

当前独立实现共 38 个生产 Python 文件、9,484 行，21 个测试 Python 文件、3,539 行，另有 20 行 Pipeline，计数包括空行与注释。运行时导入扫描未发现退役命名空间。保留责任集中于封存/输入闭包、可信执行器、Windows 原生进程与物理文件边界、缓存写权、不可变证据和验收；Jenkins 承接通用队列、节点、构建身份、控制台及归档。

试点实际使用 Jenkins 2.580.1、Microsoft Java 21.0.12.1 与 29 个声明插件。这些组件的升级、备份恢复、节点维护和容量回收仍有运维成本。旧源码只作为历史保留，本次没有删码数量或总体维护成本下降的实测结论；生产删码与入口切换必须按操作文档中的逐路径依赖 inventory另行验收。

## 历史试点

以下记录属于原隔离试点；原正式验收权和服务恢复步骤现已退役。源码职责与历史证据详见[试点操作文档](../cli-and-tooling/jenkins-coordinator-pilot.md)。

## 历史试点范围与顺序（禁止恢复使用）

1. 在有活跃 fixture 租约的批准物理根部署 loopback controller（零执行槽）和 Windows WebSocket inbound agent（一个执行槽）。
2. 封存固定 base commit、changed/new payload、deleted entries 与 scope digest；建立不可变 request/attempt/generation journal。
3. 完成固定 Python、managed Cargo、fault 模板及 Windows Job Object 终止证明；所有结果仅为 `pilot-evidence`。
4. 完成真实运行与故障恢复 gate，验证现有 Cargo guard、正式验收和 active lease 不被绕过。
5. 对同一源码、命令、环境和缓存条件做性能比较，提出可删除源码与保留薄层的具体切换批次。后续服务迁移和删码另行按获准范围执行。

## 历史 gate 状态

此表保留历史试点的验收条件、已有证据和未通过项；旧 workflow 已退役，不能据此继续提交任务，也不是 accepted milestone 输出记录。最终证据必须绑定当前封存输入与 driver/executor manifest；静态通过不能写成运行 gate 通过。

| Gate | 接受条件 | 历史状态 |
|---|---|---|
| G1 存储与部署 | 所有试点资产实际位于批准 cargo-targets 根、有活跃 fixture 租约；controller 零槽、agent 一槽、仅 loopback | 协调器retired、旧client入口拒绝coordinator_retired、formal50504已gone；keeper3432保留fixture。本会话guardian已正常停止，恢复/独立迁移澄清pending，未授权新cutover。 |
| G2 封存与身份 | dirty/new/deleted 输入可复现；源码漂移、bundle 失配、generation 重放被拒绝；journal 支持恢复 | schema 2 `cargo-closure-v2.json` 已封存主 108 项与 `zr_vm` 三项（含 Cargo.toml）；bundle/input hash 前缀 `549f7a`/`7acfd804`。metadata 25.8487 秒成功、native Job 零活跃进程；完整身份见 manifest/journal。 |
| G3 真实 Python 任务 | Jenkins agent 实际执行、取得终止证明和完整归档 receipt，reconcile 校验成功 | build 2、6 SUCCESS，归档回执已验证；属于 pilot evidence，`formalAcceptance: false`。保留旧 build 1 的 sandbox 准入失败诊断。 |
| G4 真实 Cargo 任务 | 固定 package check 经现有 validator/guard；对应正式 job/history、source digest、metrics 与归档一致 | Cargo004 queue 20/build 10 FAILURE；真实正式 managed job check exit 0/released，但 nested native grace 超时，没有 pilot receipt/Jenkins artifacts。正式 check 子项有证据，整链仍未接受；最新 cache API guard/reload已核实，Cargo005仅prepared未submit，G4 pending。 |
| G5 故障恢复 | 取消排队/执行、controller/agent 失联或重启、重复提交、迟到回执、源码漂移、坏归档与 GC/lease 边界可对账 | 既有取消/恢复证据保留；livecache004 已支持单 job writer 子项，不能宣称跨 job reuse 或整 gate 通过。Cargo004 缺少整链 receipt/归档，剩余 gate pending。 |
| G6 对等性能比较 | 相同封存源码/模板/工具链/缓存条件，区分 prepare、queue、run、archive、reconcile 时间及资源使用 | 同 source/driver、Python 3.14.4 的单样本记录已生成；OS file cache 未受控，无性能/删除量结论；Cargo 对比 pending。 |
| G7 退役可执行性 | 列出 composition/import/API/历史读取依赖；确定保留薄层与入口同步切换范围；审查 drain/rollback | 符号级批次、router/drain/history 前瞻设计已独立 review 无 P1/P2。新开关/adapter 待实现，未 cutover/退役模块，没有维护量改善结论；未来操作另行授权。 |

当前运行根为 `D:\cargo-targets\mvp-test-fixtures-6768\jenkins-pilot-01a0f6e1-native-v2-cba9ebc2924e4b08bd170517b5eec325`。schema 2 将外部 sibling 的固定 base、mount 和选择条目纳入 digest；所有未删除外部文件均保存完整 payload，重建不读取活跃外部 checkout。主仓库 base/payload 规则继续保留；路径拒绝与还原布局见操作文档。

G5 已观察场景：queue/running 重复 submit 沿用原请求；排队 cancel 未产生 runDir。运行 build 3 ABORTED，probe 全部终止。controller 在 probe 中途重启后，build 5 FAILURE，恢复查询仍为同一 build，重复提交没有新 build；runner 原因为 disconnected，Job terminal。agent 在 build 7 probe 中途停止后，probe 全死、native Job 活跃进程为零，runtime/output log 已释放；重启后同一 build 仍 running，显式 cancel 后 ABORTED，没有创建新 build。

G6 的 `python-comparison.json` 单样本：baseline prepare 2.46784 秒、run 0.446386 秒；Jenkins prepare 2.81122 秒、run 0.440582 秒、build 总计 9.835 秒、upload 0.277364 秒。两边 source/driver 相同，但 OS file cache 未受控，阶段耗时与 Jenkins 总耗时口径也不同；仅记录观测值，不能推导速度优势或删除量。

## 证据要求与收口

当前协调器退役标记 `.codex/coordinator-retirement.json` 于 `07:19:48.379193 UTC` 为 status retired，reason `User-authorized retirement including repo integration requirements`，jenkinsMigration pending。client constructor/from_runtime 与 zircon-session 入口无条件拒绝 `coordinator_retired`；正式50504/birth `134353971682599459` 于07:33 UTC已 native gone。主线程已请求澄清“保持退役并实现独立路径”或“协调原试点”，答复 pending；这不授予新 cutover、恢复或迁移实现许可。G4/G6仍 pending，005/baseline未提交执行，旧恢复状态仅为历史。

本会话guardian仅经既有normal stop flag停止：child53236/birth `134353982987268601` actual exit0/Job0/EOF，elapsed1737.841秒，native receipt SHA `3657821d71b486b398b9eb766ff27c876fc8ff6489ff8bcaf735a16030e171d2`，launcher40300 exact gone。keeper3432/birth `134353975568257128` live，fixture数据/旧lease历史全保留。旧API29路径最后renew为07:29:45.873 UTC，owned两docs lease07:34:45到期，当前只读未见foreignlease；不再renew/release旧API，不用失效写入口。编辑前两doc hashes与本任务上次结果一致，未覆盖外来变更；foreign正式操作doc与新retirement/gate文档均未改。

诊断002真实31 tests：28pass/2errors/1failure。两个cleanup嵌套BEGIN writer自锁；async failure未定位，需第二tick eligibility，raw数据库已清理不能虚构补证。production/selected modules稳定，但broad manifest因foreign cargo测试变化 sourceStable=false，不得标sealed acceptance。Windows metadata小fixture反例发现1001 DirEntry inode为0、fresh真实ID；bulk改写/替换/消失验证失败，0.380秒对0.049秒无speedup，不证明正式scanner根因、不实施该方案。

Historical pre-retirement state: 当前 managed fixture 已改为 `D:\cargo-targets\mvp-test-fixtures-3432\jenkins-pilot-01a0f6e1-native-v3-b31b321247fb42559610d67eaef1d66e`，keeper3432/birth `134353975568257128` 已 canonical live，29条自有路径由新guardian90秒续租。旧6768/controller/agent/guardian/formal52864均 gone，原因未定，OS未reboot，不能归因某次修复。旧6178资产/1.16GB按逐source before/after与copy hash匹配保存在新root；旧root/租约未迁移、修改或删除，历史proof不失效也不升级当前验收。

Historical pre-retirement state: 当前正式PID50504/birth `134353971682599459`、instance前缀 `69c61` 于06:46 UTC started，read_write/schema80 actual health OK，baseline degraded。root06:49 canonical start实际exit0，但记录ready06:49:01早于调用，不能算root首次启动。两文件ledger优化37tests/36pass/1skip、12轮synthetic字段等价、最大benchmark143→31秒与source/bench reviews通过，只是源码/合成证据，不是live420秒full或Cargo。foreign865ee affected tests session13862仍待终态；driver重封存与Jenkins恢复进行中，005/baseline未submit/execute，G4/G6 pending。

`local-session-coordinator.md` 于06:59:11 UTC被foreign owner `engine-audit-m01-migration-scripts-20261001-01a0f06a` 取得lease（base SHA前缀9dcf）。本任务已排除该路径，不编辑、覆盖或夺取claim；其公共契约建议待owner归属确认。当前运维状态仅由这两份自有pilot文档更新。

当前 publication `8728...` / asOf `05:20:44.241148 UTC`，保守可见区间为 `05:25:37.574967–05:25:38.794969 UTC`（延迟293.334–294.554秒），五 roots 均 partial。只读 observer 28 reads/0 errors，source与bindings一致、native0/Job0/EOF；它证明观察与发布边界，不能归因 `_observations`，也不能重置 timestamp 或放宽420/220使其通过。新600秒/1Hz/idle nonblocking record 与 publication002 仍在观察，不提前记成功。005/baseline未submit/execute，G4/G6 pending。

最新 GC metric 32 summaries 合计5984字、最大187，索引无排序，只读 query/decode 0.379秒。现有证据不支持大JSON解码是根因，也不排除正式连接锁。此前短 record 28 samples/0 errors中10条 watch cleanup.plan→footprint、14条其他DB `_ensure_wal_mode`，没有 upgrade frame；采样不等于根因证明。

Jenkins 已能接调度与真实 Python 任务，目前仍是过渡试点。正式替换与维护量收益未证明；本轮大部分失败位于共享存储/生命周期支持层，不能推为 Jenkins 调度性能结论或整个 workspace 验收。

当前正式第七实例 PID 52864/birth `134353876597346275`、instance `b2d95e76cc6e407184430b6aec5b9eac` 已实际 health，intent/action succeeded。已部署 scanner 微小修复：fresh nofollow stat 判 symlink，仅缺失 category 计算 sum；36 ran/35 passed/1 permission skip，独立 review 通过，但没有 420 秒扫描保证。当前 driver `6d87d5cea909d3a8b94e4a6fcc1624f6041b1be8e1097fdfa1cf465559bc9e90` / input `2591b74b811fde8e00938ab2eb64ed530e69cd2696085c4dc4811ee523a20b15`，生产源码288、正式执行285（manifest前缀 `0a68`），Rust111/digest前缀 `8cc` 不变；single binding/pair 前缀 `889124fe`/`6af49abf`。

第一全量 scan `72f199d269ba48409aeb16fdbf007998` 已完成，但 04:27 UTC 读取 age 425 秒、stale，没有提交；V4/window004 已 terminal/no POST，组合 proof SHA `c5d8cd5f7143551fcbe7469f942b961730c283f4c4337b2d095f49afb163394c`。当前唯一 active V5 PID49848/birth `134353905542669929` 于 04:56:05.766022 UTC READY，与只读 watcher005 PID16260/birth `134353903595904226`（04:52:44.759566 UTC READY）仅观察等待。fixture wait3600/native3900，正式420/trial220/generic8不变；latest `cd639ed...` partial，005未提交、baseline未执行，G4/G6 pending。baseline根dry实际exit0/Job0/EOF/source285一致（native31888/birth `134353894452065115`），不是编译或比较。

Historical cache-binding deployment: 最终 compiler-cache/pilot 源码冻结且独立 review 通过，33 Python affected / 8 PowerShell / 8 pilot native 验证通过；模型 daemon/birth 不代表真实 daemon 准备成功。正式 PID 8908/birth `134353809254264621`、instance `d88b8f152d8a425e9232de3e88817107` 恢复，rollover action/intent 前缀 `18003f`/`f4a6` actual succeeded；旧42580自然退出，无强杀，285executor source before/after一致。实际空参数 API 被 `cargo_compiler_cache_prepare_unproven` 拒绝，只证明 guard。Cargo005/baseline prepared未submit；G4 whole chain/G6 pending。

| managed evidence 索引 | 证据范围 |
|---|---|
| guardian native receipt SHA `3657821d71b486b398b9eb766ff27c876fc8ff6489ff8bcaf735a16030e171d2`；diagnostic002 proof SHA `f2cf82e4b1a4dd41f65258e50e4e1cb09e52c146d06275bbb5a6c4a0a05200f3` | 仅本会话guardian正常停止；31tests非全绿，rawDB已清理、async未定位，broad sourceStable=false。 |
| `diagnostics/windows-metadata-001/final-proof.json` SHA `b693339bccd66622f693646883c0fd333a6a19285511a29f8dd3d3328b7b45f3` | inode0/修改替换消失反例与无speedup；仅小fixture，不接受bulk方案或正式根因结论。 |
| 旧资产逐source/copy hash迁移proof（新fixture历史目录）；ledger source review前缀 `cae9` / benchmark review前缀 `36f1` | 历史6178资产/1.16GB已核copy，旧root/lease保留；37tests与synthetic12/143→31s不是live容量或Cargo通过。 |
| `profiling/snapshot-publication-watch-001-final-proof` SHA前缀 `9d4ccb`；GC metric proof SHA前缀 `c53f` | 实际发布延迟/五roots partial、RO observer与小metric读取；无 `_observations` 或大JSON根因结论，publication002仍观察中。 |
| V5/watcher independent review SHA `27f28399e1e9df0b2985b2f9183b4bb8d0dfb1a4e1cef94fbdee75c1ef55f1d1`；watcher READY proof SHA前缀 `fa1bf8f` | 观察等待配置/真实READY；不代表容量complete/fresh或已submit。 |
| `scanner-snapshot-publication-diagnostic-001` SHA前缀 `ada4a8e` | 确证254b footprint预算耗尽；failed py-spy299/历史capacity raw omitted，`_observations`耗时根因未证，不推断性能改善。 |
| `scanner-formal-successor-restoration.json` SHA `380ce1a5ba62c17ff5003a235d3e6ecbea7b786ffceb4bf17ca7743ebe3d6286`；`scanner-duplicate-metadata-final-proof` SHA前缀 `5a3c6a`；independent review前缀 `1af8` | 第七实例实际恢复与小扫描修复部署，36/35/1skip；不保证420秒内完成或提交时fresh。 |
| `cargo-baseline-6d87-dry-final-proof.json`；runtime-prepared SHA `754b7756b9d1b8f93012d7bfe0bb2023e8f55b83243ba18a2681bb3fec5fe6cc`；baseline/observer review SHA `41cc32779977746778cd20310cf26bb33c06de0745a208a40899367bf2cd9824` | 根dry exit0/Job0/EOF、285source一致；005/baseline实际未执行，不是Cargo性能结果。 |
| `compiler-cache-and-pilot-final-review.json`；`compiler-cache-formal-successor-restoration.json` | 最终源验证/review与正式successor，不是 actual daemon success。 |
| `driver-refresh-cache-binding-final.json` | Historical cache-binding driver63fb/agent8264；旧身份保留，不作为当前 driver。 |
| `driver-refresh-scanner-final.json` SHA `8bbe52286d7101d7f52e9a52c5c5fc40ef219997120aeef12f0f2684235f5dd5` | 当前 driver6d87/input2591，agent42680；actual wrapper manifest `0a68d2c6fdbca5ac04a1ec1a8d07bca5aabdc219c30d16e295527aa18e2d427d`，身份已按实际proof核对。 |
| `cargo004-build10-failure-proof.json`；console/API/rawmetrics SHA `d303905954b6323b63b6f2233a24dc4e5d97c00c2fa8716205ac3b887491e57a` | formal check exit0/released，但nested grace超时/build10 FAILURE/no receipt/archive404，非whole chain accepted。 |
| `cache-live-004-window004.json` SHA前缀3d7325；outcome SHA前缀af7b | 三captures/单job live writer和retained identity；非跨job reuse，非G5全通过。 |
| `cargo-real-003/archive-proof.json`；`cargo-real-003-admission-timeline.json`；`command-timestamps.json` | build9 freshness准入失败/noRust；archive原字节匹配/failed拒绝，不归因scope再失败。 |
| `cargo-real-002/archive-proof.json`；`storage-cleanup-retained-002.json` SHA前缀0d25f2fd | build8 storage_cleanup_reserved/noRust；legacy6132保护，不授delete/adopt。 |
| `formal-cleanup-scope-successor-restoration.json`；`formal-cleanup-scope-start-output-collection.json`；`driver-refresh-cleanup-scope-final.json`；`driver-refresh-foreign-source-final.json` | 历史reload/native identity/retained exit0与输出收集不同边界；foreign源归因不重写。 |
| `profiling/formal-final-full-fresh-inventory-33204.json` SHA前缀2b18e892；`formal-root-filter-current-capacity.json`；`profiling/root-filter-*.json` | 历史fullfresh及合成字段等价；不永久复用旧容量，partial/stale历史保留。 |
| `retirement-design-independent-review.json` SHA前缀c343927 | G7设计reviewed无P1/P2，未cutover/退役/维护量改善。 |

当前验收状态由 I1–I6 表拥有；本节 G1–G7 与下方记录保留原历史事实供审计。公共契约和历史 G7 设计由操作文档拥有，不再向历史段添加当前执行日志。

<details>
<summary>既有失败、恢复与来源归因记录</summary>

G7 root review 与独立 review 无 P1/P2，证明 `evidence/retirement-design-independent-review.json`（SHA 前缀 `c343927`）。仅审查设计，不代表 cutover、模块退役或维护量改善。

Cargo004 fresh gate 于 `00:49:40.016 UTC` 观察 scan `3b8af749...` / asOf `00:46:47.789 UTC`，local age 172.227 秒、API age 171.915 秒，五 roots complete、queue idle、watcher PID 31316 exact live、wrapper equal；随后只提交一次 queue 20/build 10。build 10 实际 FAILURE：正式 managed job `ee05800ce7334bbea54a8f8f86017f62` released/exit 0，check 42.607 秒、source sync 6.718 秒；validator PID 42632/birth `134353758103320796` 已退出。但 runner117 wait -> sealed remote process192 -> Windows Job399 的 native grace 八秒超时，未写 pilot receipt，Jenkins 无 artifacts、archive 404。autogate 自身 zero 不能证明 nested Job zero/reader EOF；正式 job exit 0 不能作为整链 accepted。证明 `evidence/cargo004-build10-failure-proof.json`、console/API/raw metrics（SHA `d303905954b6323b63b6f2233a24dc4e5d97c00c2fa8716205ac3b887491e57a`）。

live cache window004 取得三 captures，其中 leased/running 两样本同 job/同 key、active=1、native PID 42632/birth `134353758103320796`、physical generation=1、identity `5212c32f12c316bf:32b10f0000000a000000000000000000`；第三样本 succeeded/active=0，最终 released/retained，物理 identity 不变。证明 `cache-live-004-window004.json`（SHA 前缀 `3d7325`）与 outcome（SHA 前缀 `af7b`）；native watcher terminal zero/EOF 已核实。只支持单 job writer 子项，不是跨 job reuse 或 G5 全部通过。

v3 autogate 已 terminal、绑定 queue 20；v4 仅 prepared、未启动。baseline f96 launcher 仅 prepared/dry validation，不能声称已执行 Cargo 对比。当前正在修 pilot 的 fail-closed 错误归档并诊断 persistent daemon；没有新服务 restart 或阈值变化。G4 整链与 G6 Cargo 仍 pending。

最新 driver 为 `f96cc8f191e325096003e0d6bbfcd1538ecd669ff6340234a44153a6580c1b70` / input `af6beff6dc80c094e914f62caa4c82a8e7a9c10548a85eea5d6d9b0c584477da`，证明 `evidence/driver-refresh-foreign-source-final.json`；新 agent PID 45032 于 23:52:48 UTC 启动。此次重封存来自 foreign 三份文件的实际逻辑变化，未修改这些 foreign 文件，也未重启正式 PID 42580。`9f...` driver 保留为 Cargo003 历史，不能改写旧回执的 executor 归因。

Historical pre-queue20 observation: Cargo004 仍为同一 identity、`549...` bundle / `7acfd...` source input，未提交。v2 助手实际 READY：PID 39520/birth `134353728722327542`，00:01:15 UTC；native keeper 6768。顺序为冻结 286 份实际 executor manifest，确认 idle agent/watcher，等待 candidate full fresh，重复 prechecks，最后要求认证 API 与本地 age 均 <= 220 秒才一次 original submit；正式 420 秒门槛未变。当前 `a26c80ad...` 快照 asOf 23:50:04 UTC、incomplete 且 stale，不能提交或标 G4/G5/G6 通过。单次 py-spy profile 显示系统忙，不能据此推断吞吐改善。

Cargo003 queue 18/build 9 实际 FAILURE，总计 88.210 秒、prepare 32.585604 秒、run 34.066848 秒；native PID 45296/birth `134353699180889470` terminal，sourceBefore/sourceAfter 均为原固定 `7acfd...` 身份。Jenkins artifact 原字节与本地 archive verification exact 相等，verifier 正确拒绝失败回执；证明为 `evidence/cargo-real-003/archive-proof.json`。003 cache observer 两个窗口均 native zero、EOF、零 errors、零 captures，不能算 live cache proof。

23:10:09 UTC 容量 API 的 d97 快照完整且新鲜，age 336.556852 秒；Jenkins 于 23:11:14 UTC 真正启动时仅剩 19.294917 秒新鲜窗口，prepare 约 32.6 秒。正式 `cargo.acquire` request `be0d3f2d768340088e0f589ebb37453a` 于 23:12:29.495534 UTC received/accepted、23:12:30.401174 UTC complete，reason `storage_snapshot_unavailable`、requested 512 MiB；proposed job 前缀 `1b5` 没有对应 row，没有 Rust 编译。scope 修复尚未被成功 Cargo 验证，不能把这次过期拒绝归因为 scope 再次失败。时间线证明为 `evidence/cargo-real-003-admission-timeline.json` 与 `command-timestamps.json`。

Historical pre-queue20 observation: Cargo004 已准备不可变请求，沿用 `549...` bundle / `7acfd...` input。自动助手先检查 driver/source/agent/queue 与 watcher，最后要求完整新鲜快照 age <= 220 秒才一次 original submit；正式 420 秒 freshness 门槛未放宽。助手正在启动，实际 queue ID 尚未确认；G4/G5/G6 未完成 gate 保持 pending。

PS5 start invocation 的 retained native handle 已证明 PID 17992/birth `134353677263368796` exit 0；root caller PID 43272/birth `134353677255362440` 仍在 communicate 等两个 reader EOF，stdout 尚未收集完成。正式 PID 42580 健康、action/intent actual succeeded，不能把输出 pending 当作启动失败。caller 与正式服务位于尚未识别的 external Jobs，不能安全 terminate，继续保留 caller；actual retained collector 是运维尾项，未停止正式服务。证明 `evidence/formal-cleanup-scope-start-output-collection.json` 正在写，不冒称完成。

最新 cleanup scope 修复源码已冻结，32 项回归全部通过（31.364 秒），独立 review 无 P1/P2。第五次 rollover 已加载该修复；实际 Cargo 成功、live cache 和 Cargo 性能比较尚未验收。该修复按实际 target 与完整 canonical/target-derived `zircon-engine` cache/pool/scratch 声明 union，绑定 exact live reservation，并在容量复核与最终 allocation 的同一 writer 内复核全 scope overlap。未知 scope 整根保护，physical path 先拒 reparse/alias，legacy 声明不授予删除/收养许可，freshness、容量门槛和 FIFO 不变。G4/G5/G6 未完成部分保持 pending。

第五次 rollover action `4e67288e6c994a478c1b406dcf6cfe29` / intent `3144803b83ea4056bdb6c1a5599316a1` 均 actual succeeded。旧 PID 33204/birth `134353616084209510` 自然退出，未手动 Terminate；successor PID 42580/birth `134353677310677336`、instance `ed6da1b076dd411ca32837883eb381bf` 于 22:35:34 UTC 启动。22:40 UTC 后 read_write health、schema 80 与 native birth 匹配，284 份 executor source before/after unchanged；证明为 `evidence/formal-cleanup-scope-successor-restoration.json`。PS5 start 请求仍等待完整 status 输出，accepted successor/reconcile 已成立，不重复 accepted start。

Historical Cargo003 driver: 当前 driver `9f1d42edc66da2d02ccf161f95eba76394d4745501384b49fa1d986bdc24a396` / input `dfa156c7891dbfc20384a00847d536db44d68f8ddbdc44915eaf589d9a9d3f9d` / root `driver-9f1d42edc66da2d0` / agent PID 42692，证明为 `evidence/driver-refresh-cleanup-scope-final.json`。`cargo-request-003.json` 已准备不可变请求与新的 preflight pair proof，沿用 Cargo source bundle/input `549...`/`7acfd...`；live watcher PID 43788 READY。当前 API 容量完整性状态仍 stale，因此尚未 submit。新 driver 准备不重写 Cargo002 的失败归因；baseline 与 G4/G5/G6 未完成 gate 继续 pending。

Historical pre-Cargo003 driver: 当前 canonical reuse adapter 已移除 `-TargetDir`，managed 模板清除继承 `CARGO_TARGET_DIR`。回执核对固定 argv/source/hash、实际 supervisor PID/birth、Session、real job、canonical physical target 与 metrics exact path；七项模型/native tests 通过，review 无 P1/P2。当前 driver 前缀 `9600e78bee19d474`、input `7ded5db2b57855aee8a5749c51d61fb1b7e6b148664e266434b03cf7f1fad205`、agent PID 33992。新实现必须取得新的真实 Cargo 证据，不能采用 build 4 的失败回执或旧 driver 的结果。

共享支持层修复涉及五份生产源码（`low_disk_gc`、`storage_ledger`、`storage_path_index`、`cache_budget`、`validation_timings`）与五份测试。GC busy 后存续、ancestor index、cancel 停止预算、timings 六种事件筛选及 provider terminal source/external pin 释放已按源契约修复。timings 避免解码 1,186,653,967 bytes 的无关 dependency events。终态历史不是永久 object 引用；实际 Retentions 非 candidate 的 indexed objects 仍保护 active task source/external，active missing 继续 fail closed；历史、索引与 GC 规则未改。

相关源验证：GC/admission 20 OK；storage 26 OK、1 symlink skip；path index 3 OK；scanner 新回归、app08 与 index 合计 33 OK；cancel P2 修复影响集 11 OK；timings/claim 11 OK。所有最终源码独立 review 无 P1/P2。这些证明源码修复，不能代替服务 health、完整容量准入或 Cargo 编译。

服务第一次 rollover 从 `7048...` 到 successor `60c...` 恢复监听，但 first snapshot partial 暴露 refs/index 热点。第二次 action `aed7050e1d5c4faa82a5823cd090fb51` / intent `93fc2bcc8af240de9593d64a23c8deca` 已成功；旧 PID 43456/birth `134353522223204528` 已退出。successor PID 8084/birth `134353564021966482`、instance `96123ca42284414dab04e94109f5119e` 于 19:26:45 UTC 启动、19:29:11 UTC 健康；认证查询确认 schema 80 与正确 repoKey。15 条已授权路径由 managed native keeper 90 秒 heartbeat 持续续租。支持层修复已部署，服务健康不等于容量准入或 Cargo 已通过。

历史快照 `5acd684c3fd44ff2b7b374a618f24dd3` 的 scannedAt 为 19:37:57 UTC，约 19:46 UTC 发布；coordinator state 内的文件系统扫描耗尽 420 秒预算，状态 partial。旧 `reference_inventory_incomplete` 与 terminal-unindexed 诊断消失，unknown JSON 从约 14 MB 降至约 1 KB；D/E/F accounted 分别为 51.7 GB、0.65 GB、1.14 GB。这些是部分统计。聚合约八分钟导致发布时 stale，不能证明完整容量。

最终 trie/hash root filter 已独立审查无 P1/P2：`storage_path_index.py` SHA `90253fc19d3c3a7de7d86993abbfb92835bb74b4438520608ae5ee78da2e95b9`、`storage_ledger.py` SHA `cbc57dc1da9b0a7a89267345820f2c6e50e209ab03590f847332ed38534749dc`。root filter 实验 40 run、39 pass、1 skip，15.715 秒；extra protections 六项通过，6.699 秒。合成 `full_observations` 的 20,000 entries × 5 roots × 2 rounds 旧实现 16.8198 秒、新实现 1.8478 秒，全部字段相等；证明文件位于本次 fixture 的 `evidence/profiling/root-filter-*.json`。合成实验不证明完整扫描或真实 Cargo 性能。

第四次 rollover action `8f98a5633ced4ad6a59de2de10b50507` succeeded，正式 PID 33204/birth `134353616084209510`、instance `62d15b19bbcf413a93a84c7189952f9a` 于 20:53:31 UTC 启动，约 20:55 UTC ready，已实际加载最终支持层。最新扫描 `4b346d9b5165489d93bcc8ce60863202` / scannedAt 21:04:31 UTC，在 21:06 UTC API 查询为 available=true、stale=false，但 scanComplete=false；420 秒预算耗尽于 `state/offline-candidates/editor04/project-n100000`。因此完整容量恢复与 Cargo 验收仍未完成。

Historical pre-Cargo003 driver: 当前 driver bundle 为 `9600e78bee19d474bba77783ffa60be2ee1e88c43c9184f95463e375ecda4ae3`、input `7ded5db2b57855aee8a5749c51d61fb1b7e6b148664e266434b03cf7f1fad205`、agent PID 33992。Cargo002 已提交并准入失败；baseline001 仍 prepared、未运行。实际运维负担包括租约续期、driver 重封存、服务多轮 rollover 与身份验证、420 秒扫描预算、快照聚合/新鲜度、保留 reservation 归因与回执对账；这些观察不能直接授权删除模块。G4、G5 live cache、G6 Cargo 继续 pending。

正式 PID 33204 的完整新鲜扫描 `ee8ff161c9054b47bbd8bbf348f2ae4e` / scannedAt `21:19:23.570563 UTC` 已覆盖全部五个 roots（complete=1）。只读证明为本次 fixture 的 `evidence/profiling/formal-final-full-fresh-inventory-33204.json`（SHA 前缀 `2b18e892`）。21:23:32 UTC 认证 API 返回 available=true、complete=true、stale=false，age 248.195578 秒，证明为 `evidence/formal-root-filter-current-capacity.json`。这取代前轮 partial/stale 的当前扫描结论，保留其历史诊断；完整扫描不等于 Cargo 准入已通过。

Cargo002 queue 16/build 8 FAILURE：prepare 11.3607199 秒、run 13.6890991 秒、build 总计 41.596 秒。正式 validator 返回 `waiting_disk_space`，reason `storage_cleanup_reserved`，requested bytes 68,922,325,937；没有 Rust 编译或 cargo job row。native PID 23076/birth `134353635451739304` 的 Windows Job terminal；driver/input/封存源码身份未变，sourceBefore/sourceAfter 一致。实际 Jenkins archive receipt/output 的 exact bytes 已核对，declared command 匹配，verifier 正确拒绝失败回执；证明为 `evidence/cargo-real-002/archive-proof.json`。这些耗时是失败任务观测，不能作为成功 Cargo 性能比较。

`cache-live-002` 的零 matching jobs 不能证明执行中 cache 保护。共享 blocker 是遗留 artifact reservation `D:/cargo-targets/mvp-test-fixtures-6132`：native identity 匹配、无 durable ownership，维护每约 33 秒记录 `artifact.unmanaged_retained`，表示保留而非正在删除，必须继续保护其占用。证明为 `evidence/storage-cleanup-retained-002.json`（SHA 前缀 `0d25f2fd`）。独立 review 支持 `storage_admission` root-wide check 误用候选的 P2；完整 Cargo 写入 scope 的最小修复已通过回归与 review，覆盖 target、canonical/target-derived root 的 `zircon-engine` shared cache/pool/scratch。修复已在第五次 rollover reload；不会清 reservation 或改变 budget/准入门槛。

历史 `cargo-real-002-final-driver` 在前轮尚未提交；当前已取得下述 queue 16/build 8 的准入失败证据。build 4 与 build 8 均未编译，G4、G5 live cache、G6 Cargo 保持 pending，既有真实失败与历史回执继续保留。

最终证明必须列出 request identity、base/source/input/bundle/command hashes、Jenkins queue/build、真实工具链、进程身份/终止证明、产物 hashes 和 artifact lease owner。Cargo 额外包含现有 coordinator job ID、source digest 和 managed metrics receipt。性能比较不能把冷缓存与热缓存、不同 package 或仅 metadata 与编译作对比。

未完成 gate 保持 pending；不存在“Jenkins 成功即正式验收”转换。需要回滚时按操作文档 drain Jenkins 新分派、核实本次进程树、对账历史与租约，再恢复旧 admission。旧服务、其他 Session 与共享源码变更不属于试点自动清理范围。

2026-10-01 16:09 UTC 补充证据：正式协调器已恢复为 schema 80、新 instance `201e5da3fce642a8b08c5ee5b9b41d86`，身份验证通过；旧 fixture owner 6132 已退出，拒绝继续准入。新 fixture `cba9ebc2924e4b08bd170517b5eec325` 的 owner 为 6768/birth `134353440704803594`。原 3 项 native lifetime 检查通过；新增两次并发 start 回归先复现启动两个 Job，修复后 4 项 bootstrap guards 通过（0.612 秒）。9 项 closure transport 检查通过（14.765 秒），包括真实子孙进程 timeout、成功根退出后的后代收敛、完整双管道输出和失配失败。上述检查不能替代 G3–G6 实际运行验收。

Historical observation before the currently authorized shared-support repair: 共享支持层风险：`processes.py` SHA `b2513bac480da6637d3a90f51deefd4abb1e591916869bf7c20eab0f375a36ce` 的 Windows descendant 清理缺少子/父 birth ordering，PPID 重用可能归入无关进程。试点 runtime 拒绝 legacy fallback，metadata 使用独立原生 Job transport；共享 coordinator 源码未修改。没有证据证明此前任何服务退出由该风险造成。 Current shared-support source checks and independent review passed; the deployed service is healthy, while full capacity admission and actual Cargo remain pending. The earlier unchanged-source statement describes the historical observation only.

</details>
