---
doc_type: runtime-evidence
created: 2026-10-16
related_code:
  - tools/jenkins-management/source_preparation.py
  - tools/jenkins-management/install_guard.py
plan_sources:
  - ../01-single-host-jenkins-coordination.md
tests:
  - "zircon-management-tests #62/#64/#65: SUCCESS; 17 commit methods, 9 execution methods and R090 scoped source review"
  - "zircon-management #83/#84: SUCCESS; exact validated files committed locally, no push"
  - "zircon-governance #101/#102: SUCCESS; bounded physical scan and same-inventory audit, global complete false"
  - "zircon-management #60: SUCCESS, 17 tests and final input verification passed; archive API/Home equal, local JSON equal"
  - "zircon-management #70: SUCCESS, preparation receipt verified"
  - "zircon-management-tests #50: historical failure retained"
  - "zircon-management-tests #51: SUCCESS, 67 methods and four native suites"
  - "zircon-management-tests #52: SUCCESS, 69 methods and five native suites"
  - "zircon-management-tests #53: SUCCESS, current 69 methods and five native suites"
  - "zircon-governance #87: SUCCESS, 49 methods and native EOF"
  - "zircon-governance #89/#90: SUCCESS, bounded index scan and audit; physical traversal zero"
  - "zircon-records-followup #96: SUCCESS, exact readiness followup archived and removed"
  - "zircon-management #74: FAILURE, prepared absence unproven; no product acceptance"
  - "zircon-records #35: SUCCESS, two exact stale messages archived and removed"
  - "zircon-governance #78: full 49-method self-test not accepted"
---

# 受管工作树与 Jenkins 运行进展

本记录区分已经取得的回执与仍在运行的步骤。当前状态核对于 **2026-10-17 07:40 UTC**；下方带时间的旧段落保留当时观察，当前状态以本节和实际回执为准。本记录不能作为进程管理凭据。

## 服务与目录

http://127.0.0.1:18080/ 的 controller、agent、插件与请求准入健康检查均通过，quietingDown=false。本机保留固定节点 zircon-windows-agent，四个执行槽；controller 为零执行槽。运行 operation 为 start-1792141433756007100，封存 driver 为 0fc997ca92dd1a6501aa31f3bb0c1c5c9841f4a4d65cb69fb45931330af83565。

Jenkins Home、运行组件、agent 控制工作区、日志与状态留在工程 .jenkins；当前请求的构建输出位于 E:/cargo-targets/zircon-local。这些观察不能替代持久宿主故障恢复或托盘右键菜单的交互验收。没有停止或重启正在处理积压任务的实例。

## 已有运行证据

| 流程 | 实际证据 | 尚未完成的部分 |
| --- | --- | --- |
| Git 管理 #63–#65 | #63 被管理员中止；#64 成功对账原提交 e965c722166cfeb612f4f2d8d8953cb6fc2a5b15；#65 成功记录 Git 原生进程修复提交 89789b74beaef9bf83cb61f8c13236ca730a81c0 | 不重放被中止任务来重复提交 |
| MVP 同一工作树准备 | 同一 m15w-m 保留原 W25、reader 和完整 map；#68／#70 SUCCESS；654c 支持层下的 #76／queue 3543 已实际启动并完成 63,891 文件首次捕获，后续输入核对仍在运行 | 原 #74 的 FAILURE／unknown 原样保留；#76 尚无完整返回，不能提交产品 Flow 或宣称产出新 EXE；视觉、交互、性能和正式包仍待验收 |
| 原 17 项定向测试 | #60 已 SUCCESS；17 项命名测试通过，最终封存输入核对完成，原生退出 0、EOF、Job 活跃进程为 0 | 证据仅属于原请求及受管工作树；当前 checkout 的 r6／#73 源码捕获失败另行保留，不能继承此通过 |
| 管理入口 R12 | 当前 #54 SUCCESS：72 方法、5 原生套件、223 源码 pin 和 32 份三处原字节一致归档；已在线应用 654c 胶囊，installed 指针 37b1 | controller、agent、keeper 未重启；对应产品流程须各自完成实际运行验收，不能继承协调器回归结果 |
| 存储治理 | #98 定向与 #99 完整 49 方法均 SUCCESS；#101 扫描和 #102 审计使用同一新库存，实际遍历 41 目录、92 文件 | 150 个具体未知／历史范围仍被剪枝；globalInventoryComplete=false，cleanupAuthorized=0，缓存实际回收 0 B |
| 验证后本地提交 | #62 的 17 方法通过，#83 提交 ee18dfd3；R090 的 #65 范围审查通过，#84 精确五文件提交 e6235426 | 均未推送；R090 主任务的其余审查保留，各自产品验收独立 |
| 共享执行对账 | #64 的九方法通过，reconciling 适配已在当前 runtime 原子应用 | 独立 Pipeline adapter 不属于封存 driver；产品链仍须实际完成 |

#70 的输入准备身份为 87019138e6deb3f446a2c408e65a7e82f8ef5ab12645e6ec57860b0bdd73e486；准备回执 SHA-256 为 a2893ac708d2df6574c3d12952fd8fcdcaf875e43437e1e414297c98c315b7ff。汇总与三方验证见：

- .jenkins/state/management/reports/mvp-current-input-prepare-20261016-r11/summary.json，SHA-256 0c83d580fa918e2679aca4f4a795b783014fb202f22573f9d0c47c083078ec6f。
- 同目录 03-prepare.verification.json，SHA-256 8aded00302dfc53a72839ea0266492ab3c12165f2537bc256790ff3b43de41e8。
- 原 17 项测试步骤回执为 E:/cargo-targets/zircon-local/zircon-jenkins/management/reports/abe1f85df7d25c60b43a8cbf0018bff3a8d7502c0be90ef408f590a773700062/steps/0001/receipt.json，SHA-256 9fc09b35db53c935774c14b2c110322d5ea10ca4b55daaa04bbedf67c76a39e7。

以上保留原失败与未知请求，不重复提交、不替换历史归档。回归套件清单按实际选择生成的修复已有 R11 #49 成功证据；#50 对新增补丁的验证必须另行完成。

## 产品路由与下一步

现有 prepare-flow → submit-flow 提供 worktree current-source 产品模板，可组合 Interface、Runtime、Editor、App 的检查、构建、实际测试、EXE、DLL、资源、字体、CRT 与同一 BuildSet 打包。针对原 #70 完整 W25 的 prepared-WT 适配已通过 #52，但实际 #74 在两条非自有、冻结依赖的 before/current 均为 null 路径上失败。新修复保留完整 map 并分别证明非自有缺席、自有缺席和真实删除，且在首次 sealed_input 发布前重新核对外部 CAS；#53 当前 69 项已通过。当前 654c 胶囊已正式在线应用；原 #74 UNKNOWN 与失败证据保留。同一 reader、工作树、Flow 请求和覆盖的新 #76／queue 3543 已实际运行，输入准备尚未返回，submit-flow 和产品编译通过仍待完成。

M8 的既有调用方审计仍列出产品 CLI、Export、特性矩阵、Tauri 子执行及 hosted CI 的接入与运行核验缺口。完整部署、复用与并行恢复、五类模板端到端、唯一入口与回退门槛继续开放，按主计划推进。本记录没有创建新源码副本、交付新产品包或宣告正式验收完成。


## 01:39 UTC：固定节点扩容与收件接续

同一 zircon-windows-agent 已通过 Jenkins 公共配置接口在队列锁内按原配置哈希热调为 **4 个执行槽**，controller 仍为零。原 Computer、remoting channel、#60／#72 的 executor／任务对象及原生出生身份保持，controller／agent／keeper 未重启，重步骤预算未增加。回执位于 .jenkins/state/management/reports/fixed-node-four-executor-runtime-20261017-r2/verification.json，SHA-256 4683e399f42c5d3647b894679e401494d4785bd9ed2674180a6435eb9ad87682。

这是已保存到 Jenkins 节点配置的运行期覆盖；部署规格的默认值仍为 2。下次显式 bootstrap 前须按受控部署流程对账并采纳此覆盖，不能用旧规格覆盖正在运行的节点。

R8 诊断补充已由 zircon-records-followup #95 SUCCESS 接入原 MVP task，原父任务身份、71 条待办及产品验收门槛保留。原 3,880 字节诊断文件先归档核对，再由受管清理删除；没有删除 Cargo 缓存或未知对象。最终验证见 .jenkins/state/management/reports/mvp-r8-diagnostic-followup-20261017-r1/terminal-verification.json。


## 2026-10-17 02:00 UTC：在线支持层与原 17 项回归终态

Jenkins 回归 zircon-management-tests #51 已 SUCCESS：67 项方法、4 个原生套件，222 个源码 pin 一致，27 个归档文件本地／API／Home 原字节相等。管理支持层已在线切换至 b41267398378b7f345e4af66632c3a66785f5fbf19339fba270d7f835f36b018；controller、agent、keeper 身份保持，未重启，原 #72 保留启动时版本。

原 zircon-management #60 已自然 SUCCESS，17 项命名测试全部通过，最终封存源码检查一致，原生进程已退出且管道 EOF。证据属于现有受管工作树和原请求链，不改变此前拒绝或中止记录。该任务的自动缓存清理仍返回 protected（缺少作用域登记），实际回收 0 B，构建池和保护中的任务目录保留。这不代表产品、性能或 M8 验收。治理修复的 7 份源码已应用，完整 49 项 Jenkins 自检及有界扫描仍待完成。

证据：[R12 在线切换](../../../../.jenkins/state/management/reports/worktree-samebyte-materialize-live-20261017-r12r3/hot-install-verification.json)、[17 项终态](../../../../.jenkins/state/management/reports/original17-management60-terminal-20261017-r1/verification.json)。


## 2026-10-17：工作树 Flow 兼容修复与过时消息归档

原 17 项测试的初次授权拒绝和 R9 等待观察已被 #60 的实际成功结果取代。zircon-records #35／queue 3379 将这两份消息作为历史证据处理，保留原 chat、request、attempt、源码补丁哈希；没有再提交原生验证。两份原文先按完整哈希归档，再由 Jenkins 删除收件叶文件，共 38,081 B。Cargo 缓存实际回收 0 B。台账只关闭这两份过时消息的传递，不代表当前 checkout、源码提交、产品、性能或 M8 验收。

#60 的归档由 API 与 Home 返回相同的 197,496 B 原字节；本地控制回执为 197,497 B，JSON 内容相同。这里明确保留字节差异，不把它记为三处原字节全等。

prepared-WT Flow 的 5 份精确增量补丁已应用到 canonical 源码：区分经过原封存和 Git 基线核对的两条自有未创建路径与真实删除，保留完整 W25、源码／外部输入和原工作树。静态正反补丁、AST 与真实权限探针通过；新增两项针对 typed sealing 与原 prepared consumer 的主链测试。Jenkins 69 项／5 套件回归和新版本在线切换仍待完成，当前 installed 继续为已验证 b412，未重启服务。原 #72 使用启动时 2f 胶囊继续执行。

治理 #78 已因既有 1500 秒时限 FAILURE：计划 49 项，启动 42 项，完成 41 项，其中 39 通过、2 ERROR；另 1 项未完成、7 项未开始。截止前没有 unittest 最后异常栈。原生 Job 清理、等待器与双管 EOF 已证明，不能将该次终止称为自然退出。两个错误将通过 Jenkins 定向复现，在修复和验收前不提交后续扫描或删除缓存。

证据：[17 项消息归档](../../../../.jenkins/state/management/reports/original17-records-terminal-archive-20261017-r1/terminal-verification.json)、[Flow 增量应用](../../../../.jenkins/state/management/reports/prepared-worktree-current-source-adapter-20261017-r2/source-edits.json)。

## 2026-10-17 04:03 UTC：真实接入与治理修复

#52 的实际回归和随后在线切换分别见 [69 项回归](../../../../.jenkins/state/management/reports/prepared-worktree-flow-live-20261017-r3/live-state/verification.json)与[在线切换回执](../../../../.jenkins/state/management/reports/prepared-worktree-flow-live-20261017-r3/hot-install-verification.json)。controller、agent、keeper 身份保持，服务未重启。#74 使用新 93e0 胶囊，但仍在输入准备阶段失败；其返回与本地、控制台记录一致，不重复投递 UNKNOWN。#72 保留原 2f 胶囊，已核对真实 Cargo PID 45276 及活跃 rustc 子进程，仍无测试终态，不记为通过。

治理 #82 的 foreign preimage 字节完整，原 Git show 因 Windows 长 refspec／路径消歧失败。修复采用受控进程级 core.longpaths 和精确 cat-file blob 读取，保留原断言；#81 同类长路径仍须原用例复跑证明。#78 原 1,500 秒截止不作为通过，新整套预算为 3,600 秒，单项仍为 1,500 秒。两项原用例已在 #85、#86 实际通过；完整 49 项 #87 正在运行。[四文件修改回执](../../../../.jenkins/state/management/reports/governance-git-longpath-compat-20261017-r2/source-edits.json)证明源码已应用，不能替代完整 49 项及治理正式准入回执。

诊断读取意外创建的四个 SQLite 副文件已按精确身份、哈希和独占句柄清理，[清理回执](../../../../.jenkins/state/management/reports/governance-two-error-repro-live-20261017-r1/sidecar-cleanup-20261017-r1/verification.json)记录实际释放 65,536 B；原数据库及 25 个受保护文件字节不变。此前收件归档的 38,081 B 单独计数；Cargo 缓存本轮仍回收 0 B。


## 2026-10-17 05:24 UTC：当前回归、收件接续与有界治理

#53 的 69 项管理入口回归实际 SUCCESS，5 个原生套件自然退出 0、输出 EOF、关闭后 Job 活跃进程为 0；32 份归档在本地、API 与 Home 原字节一致。223 个源码 pin 的当前版本与证据相符。此次修复仅增量修改 source_preparation.py、prepared-WT 测试和历史胶囊测试夹具；历史 guard 生产逻辑未改。新候选 8b515 正式安装尚待完成，未重启 controller、agent 或 keeper。[当前 69 项回执](../../../../.jenkins/state/management/reports/prepared-worktree-flow-operation-proof-live-20261017-r3/live-state/verification.json)。

#72 的原生步骤已得到 8 passed、0 failed、1 ignored，命名测试证据完整，Cargo 自然退出且管道 EOF。最终封存源码一致性守卫仍在执行，进程有读取进展；尚无 input-final 或根终态回执，因此不能把整条 Pipeline 记为通过，也不重复提交。[原 #74 无收件副作用核对](../../../../.jenkins/state/management/reports/mvp-original-prepare-flow-no-intake-20261017-r1/verification.json)证明原失败前未建立 Flow intake，原 UNKNOWN 不重写；续办仍须核对正式安装和单次提交身份。

zircon-records-followup #96 将 8,122 B 的原 MVP readiness 文件按完整哈希归档、接入原父任务并删除该收件叶文件。父任务身份和 pending 验收要求保留；没有新 EXE、界面／交互／性能通过或缓存回收。[接续回执](../../../../.jenkins/state/management/reports/mvp-readiness-followup-20261017-r1/verification.json)。

治理 #87 的 49 项实际全部通过，原生进程自然结束并完成三处归档核对；夹具清理记录逻辑量与分配量，实际卷释放量未知，不能作为缓存回收数字。#89 与 #90 使用同一 ec9ce2e5 库存完成扫描及审计，但旧容器级 buildRoot 归属使 D/E/F 根直接被保护，实际目录和文件遍历均为 0。globalInventoryComplete=false、cleanupAuthorized=0、releasedBytes=0；这不是磁盘缓存完整盘点。两文件容器命名空间候选只保存，等待管理入口激活后另行增量应用及 Jenkins 验证；具体活跃池、源码、未知归属和历史证据继续保护。[49 项回执](../../../../.jenkins/state/management/reports/governance-git-longpath-focused-live-20261017-r2/verification.json)、[扫描与审计回执](../../../../.jenkins/state/management/reports/governance-managed-roots-inventory-live-20261017-r2/verification.json)。

## 2026-10-17 07:06 UTC：实际运行、提交入口与积压接续

当前运行仍为 start-1792141433756007100／driver 0fc997ca，固定 zircon-windows-agent 在线且保留四个执行槽，controller 为零。controller、agent、插件、接单均健康；没有为本轮修复重启服务。

管理入口 #54 的实际 72 方法验证完成后，正式支持层已在线应用 654c88ddde1df20e91b0c6292ecd378a18e091bd0bd4438061e8660b07d68a6c。安装指针 SHA-256 为 37b1eb6d1bd7d490aaeaa54a6b3f850e53a686f96076261181e62437888de403。[当前回归](../../../../.jenkins/state/management/reports/prepared-worktree-flow-operation-proof-live-20261017-r4/live-state/verification.json)与[正式在线应用](../../../../.jenkins/state/management/reports/prepared-worktree-flow-operation-proof-live-20261017-r4/idle-install-verification.json)分别核对，不能继续把旧 8b515 候选写成当前未安装状态。

原 #72 已实际 SUCCESS：8 passed、0 failed、1 ignored，原生 Cargo 自然退出 0、双管 EOF、Job 活跃进程 0，最终四项封存输入守卫已通过。证据属于原封存源码及原请求；旧 legacy inputsUnchanged=false 仍保留，不代表当前共享 checkout 或 MVP 预览程序通过。[作用域终态核对](../../../../.jenkins/state/management/reports/engine-audit-m1-r13-current-20261016-r1/scoped-terminal-finalization-20261017-r1/verification.json)。

S47 的 zircon-management-tests #61 SUCCESS：50 个方法及 65 个 subtests，JUnit 共 115 个 testcases，零失败、零跳过。实际 summary 与 JUnit 在本地、API、Home 原字节一致；其余 native、源码与清理证据保留在本地控制报告。zircon-records-followup #97 已将结果写回原 engine-audit-20261009-01a12097，按哈希先归档再删除唯一 11,031 B 原收件叶文件；原 S42 产品验证仍待完成，没有再提交产品测试。[S47 实际回归](../../../../.jenkins/state/management/reports/standard-management-tests-s47-20261017-r1/verification.json)与[原交接接续](../../../../.jenkins/state/management/reports/s47-capture-diagnostic-integration-followup-20261017-r2/verification.json)。

公开本地提交入口已实际执行 #77、#78、#79、#80，四次 Jenkins 根结果均 SUCCESS。它们是各自任务的提交记录，后续产品验收以各自测试回执为准。可选 expectedFileHashes 增量修复只绑定显式选择的文件，复用原生读取守卫持有至 Git 提交结束；普通提交仍不要求额外会话许可或完整源码摘要。该修复的实际 17 方法回归 #62 正在运行，R090 的精确五文件尚未提交。[增量源码回执](../../../../.jenkins/state/management/reports/commit-validation-binding-20261017-r1/source-edits-r2.json)。

execution #34 的历史根失败包含编译结束后对账状态误判：原生结果与 Pipeline 根结果须分别核对。增量修复把 reconciling 加入非终态轮询和对账分支；两文件九方法与真实 Groovy 适配正在等待串行 Jenkins 验证及单 execution job 激活，尚未据此宣称已在线修复。

收件待办不等于全部没有接单权限。当前七项主要分为：MVP 已实际执行 #76；R090 五文件已冻结，等待轻量语法、格式与 JSON 验证；其余任务仍须按各自交接补齐当前源码归属、原补丁的当前版本适配或完整编译入口。原记录 formalRequests=[] 只反映当时收件快照，不能用它否认后来单独提交的正式任务。unknown 请求继续原身份对账，不以新请求掩盖原结果。

治理 #99 的完整 49 方法实际通过后，后续 scan 与 audit 使用同一新库存，固定 D/E/F 合法根、明确文件和目录预算，保留具体活跃池、历史对象、源码候选及未知归属。物理盘点完成前不执行缓存 GC。本轮明确核对的缓存实际释放仍为 0 B；此前四个自有 SQLite 副文件释放的 65,536 B 与收件归档分别记录，不重复计入缓存回收。M5–M8、唯一入口、跨 Session 恢复、产品包与性能门槛继续开放。

## 2026-10-17 07:40 UTC：已验证文件实际提交与执行适配上线

公开提交入口修复已通过 zircon-management-tests #62：17 方法、0 跳过，四份归档在本地、API、Home 原字节一致，自有夹具已清理。随后 zircon-management #83 成功提交两个精确选择文件，commit ee18dfd3dda74ea8d12b19036d55fab5143bff7b，9 个 Git 原生退出事件均为 0。普通本地提交继续使用已有用户授权，可选 expectedFileHashes 仅绑定显式选择文件；没有增加每个 chat 的许可或全仓摘要要求。[17 项证据](../../../../.jenkins/state/management/reports/commit-validation-binding-20261017-r1/ci-verification.json)、[实际 Git 提交](../../../../.jenkins/state/management/reports/commit-validation-binding-20261017-r1/git-verification.json)。

R090 的原 task source-comments-20261014-01a13a81 经 #65 完成两文件解析、格式检查与三份 JSON 审查，四条真实原生命令均自然退出 0、双管 EOF、Job 活跃进程 0，27 份归档三处字节一致。该 scoped-source-review 没有执行 Cargo 或单测，不作为产品验收。#84 按原 session、已有阶段提交授权和完整 validatedFileHashes 精确提交五文件，commit e62354261849ede15838db4e1df8328a96827e0e，12 个 Git 退出事件均为 0。未推送或发送外部通知；其余来源和暂存保留。[范围审查](../../../../.jenkins/state/management/reports/source-comments-r090-scoped-review-20261017-r1/live/verification.json)、[五文件 Git 提交](../../../../.jenkins/state/management/reports/source-comments-r090-scoped-review-20261017-r1/git-verification.json)。

执行对账九方法 #64 已 SUCCESS。当前 zircon-execution 通过 Queue.withLock 对单 job 原子应用独立 Scripted Pipeline adapter 4274894044176a815c463ce57c1616ccb4512b789dadb91845840d9a82664370，将 reconciling 保留在等待和对账分支；29 参数、原 prelude、0fc CLI 与封存 driver 均保持。该 adapter 不是封存 driver 成员，安装方式与来源分别记录；controller、agent、keeper 未重启。#63 的夹具失败及 #34 的历史失败保留，产品执行不能继承回归结果。[实际九方法](../../../../.jenkins/state/management/reports/execution-reconciling-live-20261017-r2/live-state/verification.json)、[在线应用](../../../../.jenkins/state/management/reports/execution-reconciling-live-20261017-r2/activation/verification.json)。

治理 #101／#102 已实际 SUCCESS，D/E/F 分别遍历 7/15/19 目录与 16/19/57 文件，观察到 201,290,808 逻辑字节，未遇读取错误或链接。库存 digest 117ca0b01eb8f0520136a7dfbc06a9d583f8a4d673a895fe36938c5615441098，两任务使用同一库存；11+6 份归档三处原字节一致。150 个具体范围（140 未知、10 历史）继续剪枝，globalInventoryComplete=false、cleanupAuthorized=0；这是有界观察，不是三盘完整清查。本轮缓存释放 0 B，自有测试夹具与收件原文清理分别记录。[物理扫描与同库存审计](../../../../.jenkins/state/management/reports/governance-managed-roots-inventory-live-20261017-r3/verification.json)。

当前固定单节点四槽在线，十个主要流程均可接单；旧 zircon-pilot 仍停用。MVP #76 使用原工作树和原 reader 继续完整输入准备，源码捕获已经完成，随后仍有读进展；提交产品 Flow、编译、EXE、视觉／交互／性能和 M5–M8 门槛继续开放。其他原请求缺少当前补丁归属或完整 recipe 的部分仍须在原身份上补齐，不用重复请求替换未知结果。
