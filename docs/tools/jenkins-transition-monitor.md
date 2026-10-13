---
doc_type: operational-standard
created: 2026-10-11
---

# Jenkins 过渡期每日巡检与人工合并

用户在 2026-10-11 明确要求每日审查文件状态、监控 Jenkins 健康并在 Jenkins
落地前临时接管合并。这是针对当前过渡期的例外。长期的版本、工作树、生成、
编译、产物复用和清理归属仍遵循 [Jenkins 管理标准](agent-artifact-management.md)。

## 调度与报告

用户明确要求 **Codex 的任务**，每天北京时间 09:00（Asia/Shanghai），使用
`E:/Git/ZirconEngine` 的实际本地项目目录，不创建额外工作树或源码副本。
任务名称、提示词和创建后的核对项见
[Codex 任务配置稿](../../.codex/skills/zircon-artifact-management/references/daily-codex-review.md)。
使用 Codex 正式任务接口创建，核对任务身份、启用状态、时区及下次运行时间。
该配置稿及一次手动巡检成功均不证明任务已注册。

用户随后通过 Codex 添加了任务「规范产物与源码副本管理」，实际 ID 为
`automation`，状态 `ACTIVE`，类型 `heartbeat`，绑定当前接管会话。已只读核对
真实 `automation.toml` 的规则为一周七天的 09:00，系统时区为北京时间；提示词
与本配置稿一致（末尾多一个 Markdown 结束标记，不改变操作要求）。
任务记录没有独立时区字段，当前会话仍无 `automation_update` 接口；下一次
自动触发时间与定时运行尚未验收，下述手动执行不能替代定时触发证据。
核对回执保存在报告根的 `codex-task-registration-observation.json`。

先前误建的 Windows 计划任务
`ZirconEngine-Daily-Transition-Audit` 已按用户更正撤销；本次新增的 Windows
注册和桌面通知脚本已移除。保留历史运行报告与撤销回执，不能把旧 Windows
运行结果当作 Codex 任务验收。不得用 Windows 调度、后台轮询服务或直接编辑
Codex 数据库、自动化 TOML 来替代用户要求的 Codex 任务。

Codex 审查可使用 `tools/dev/jenkins_transition_monitor.py`。巡检没有编译、服务重启、Git 写入、
源码捕获或目录删除能力。它只写获准的审查元数据目录：

`E:/cargo-targets/zircon-local/reports/jenkins-transition-monitor`

`artifacts.json` 在报告创建前记录归属和保留条件；日期和运行 ID 目录保存实际
报告，`latest.json` 保存最新摘要并指向原报告。报告保存 config 哈希、巡检代码
哈希、改动文件哈希、暂存状态、未完成合并、锁状态、工作树注册、磁盘余量、
Jenkins 原始请求标识及待处理流程。完整扫描记录链接、读取失败、硬链接、
活动进程引用、缺失归属清单和源码树候选。扫描结果是实时观察，不是原子快照；
逻辑字节不能作为可回收空间。旧目录和未知源码只登记复核，不自动清理。

只有异常有实质变化、完成、失败或需要用户行动时在 Codex 提醒；状态不变时
安静保存报告。磁盘报告是核查入口；本次授权不包括外部消息发送。
保留条件 30 天后复核；此期限不授予自动删除权。

## 健康判定与移交条件

先验证当前 `driver.json` 指定的不可变实现及完整文件集合，再用该实现只读
检查。控制器与 agent 需有匹配的 PID、创建时间、可执行文件和命令摘要；端口
需属于该控制器。只有端口归属核对通过，才向本机 API 发送认证 GET。运行代次
需与 selector、host 和 operation 一致，实际健康及运行状态需同时通过。
报告不保存凭据或原始命令行。

恢复启动时，项目 Python 的临时辅助进程仍计入潜在占用者；不得因其来自
Codex 或声称只读就忽略。恢复入口允许最多 60 秒的只读观测等待，占用消失后
重新核验完整的原计划。持续占用、身份不可读、Boot 或生命周期变化仍阻止
启动；这不是命令重试或历史队列重放。辅助审查应使用独立的现有解释器，
避免审查自身占用待恢复的运行时。每日深度巡检继续使用规定的 Jenkins Python，
不在恢复事务内启动。

`tools/dev/prepare_jenkins_runtime_contention_recovery.py` 是针对原请求的 Jenkins
离线部署修复入口。它只安装经过实际针对性回归验证的不可变支撑驱动，复用
已审查计划与请求，保留原证据；不准备产品源码、操作 Git、构建或启动服务。
启动仍由 `tools/dev/start-jenkins-coordinator.ps1` 的严格权限宿主执行。

`Ready`、fixture 成功和历史支持层测试只证明各自范围。它们不证明当前源码、
多工作树合并、版本提交、生成、复用或定时清理已验收。移交须同时具备：

- 当前部署实现与真实服务身份匹配，原请求的源码及覆盖验收成立。
- Git 集成和多工作树流程有真实目标提交、路径、依赖、冲突及结果回执。
- 生成、产物复用、任务结束清理和定时维护均有当前代次的实施及运行证据。
- 保留对象、未知归属、历史锁与其他会话的活动资源完成原状态对账。

移交记录需指向上述证据，然后停用本次临时任务；不能只见到 `Ready` 就自动
切换写入主体，也不能删除旧证据或历史 State 来模拟验收。

## 人工合并操作

每日任务负责巡检和提醒。由临时接管会话处理已审查的候选批次，不让计划任务
根据“dirty”“目录旧”或“有分支”直接提交。当前共享 main 的异会话改动须保留。
每个批次按以下顺序执行，记录到上述报告根的 `integration` 目录：

1. 明确原会话/作者、候选路径、父提交、文件和依赖哈希；收集当前源码所需的
   检查证据。未声明归属或行为验收不成立的候选保持待验收。
2. 检查前一次人工交付、当前 HEAD、共享 index、未完成合并、活动 Git 进程和
   Jenkins 所有权。存在外部暂存、占用或身份不明的锁时停止该批次 Git 写入。
3. 仅暂存批准的路径，检查暂存后的内容、路径集合、哈希与父提交。
   忽略的新技能/测试须单独明确选择，不能使用 `git add -A`。
4. 用正常 Git 提交交付当前 main 上的候选，不复制源码、不创建额外工作树，
   不 reset/stash 别人的改动。真正的分支合并须先确认源 ref、目标 ref、共同
   基线和验收；已经包含于 main 的旧分支不得为了“清积压”制造合并。
5. 对账实际 commit 的 parent、tree、路径和内容；保存结果与仍阻塞项。
   本次授权不包含 push、发布、外部通知、解除 Jenkins 历史保留或清理未知文件。

锁恢复必须有当前占用/身份证据、精确路径及原字节记录；不能仅凭零字节或年龄
删除锁。其他会话恢复操作已经完成时，不恢复其历史锁或重复接管其提交。

## 当前记录

Windows 任务撤销回执位于
`E:/cargo-targets/zircon-local/reports/jenkins-transition-monitor/windows-task-revocation.json`。
当前调度状态在同目录 `handoff.json` 和 `artifacts.json` 登记为 Codex 任务配置
已启用、本会话手动执行已验证、定时触发未验收；历史 Windows 注册与首轮运行
回执仅保留为历史证据。

用户要求立即执行后，本会话于北京时间 2026-10-11 23:25–23:32 执行了保存的
任务提示词，实际深度扫描回执为
`E:/cargo-targets/zircon-local/reports/jenkins-transition-monitor/2026-10-11/20261011T152532-6381af41/report.json`。
汇总见同根 `codex-manual-first-run-summary.md`，原生进程退出证据单独保留。
扫描覆盖 142 个写权限项，其中 119 个存在；903,837 个已观测文件不包含
1,478 处权限拒绝及未跟随链接的内容，不能称为完整可读或无异常。
当前 Jenkins 原操作 `start-1791729289455556300` 仍为 `failed`，host 原因为
`agent_node_migration_unproven`；E 盘余量恢复不是本轮清理成果，本轮删除与回收均为 0。

另一人工提交会话已交付工具与 186 个文件的注释，其
[交付记录](../plans/jenkins-session-coordination/01/2026-10-11-manual-commit-acceptance.md)
指出行为、依赖迁移及 Jenkins 实现仍待验收。2026-10-11 22:00 北京时间的只读
检查确认 Jenkins 实际健康失败，当前原操作
`start-1791708207030401800` 为 `stopped`；新的 `.git/index.lock` 出现于
2026-10-11 21:13:22 北京时间。后续状态以实际巡检回执为准。
