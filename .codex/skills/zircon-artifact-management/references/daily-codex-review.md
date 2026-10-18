# Codex 每日审查任务

这是用户要求的任务配置稿，不是任务注册或启用回执。登记状态须依据 Codex
正式任务接口的结果，或用户通过 Codex 创建后的真实任务记录。只读核对任务
身份、启用状态、重复规则、目标会话和提示词，并区分配置启用、下次触发及
实际运行验收。接口缺失且没有真实任务记录时保持待创建，不另建 Windows
计划任务，不直接修改 Codex 的任务文件或数据库。

| 配置 | 值 |
| --- | --- |
| 名称 | ZirconEngine 每日文件状态与 Jenkins 合并审查 |
| 时间 | 每天北京时间 09:00，时区 Asia/Shanghai |
| 项目 | E:/Git/ZirconEngine |
| 执行位置 | 当前本地项目目录，不创建工作树或源码副本 |
| 上下文 | 延续当前接管会话及持久审查记录 |
| 模型与推理 | 保持当前可用默认值 |
| 提醒 | 仅异常有实质变化、完成、失败或需要用户行动时提醒 |

创建时核对显示的时区与下次运行时间；不能把 America/Los_Angeles 的 09:00
当作北京时间 09:00，也不能用固定夏令时换算代替 Asia/Shanghai 的每日规则。

## 保存到 Codex 任务的提示词

```text
在 E:/Git/ZirconEngine 的实际 checkout 执行 ZirconEngine 每日文件状态与 Jenkins 合并审查。
使用 $zircon-artifact-management 和 $jenkins-coordination；读取当前 AGENTS.md、.codex/config.toml、docs/tools/jenkins-transition-monitor.md 及现有审查和交付记录。

检查 config.toml 授权目录中的产物归属、重复源码候选、遗留构建、活动进程、磁盘余量、共享 Git index、锁、未完成合并、分支与工作树。使用已有 tools/dev/jenkins_transition_monitor.py 执行完整只读审查（在项目根调用 .jenkins/runtime/python/python.exe -B -X utf8 tools/dev/jenkins_transition_monitor.py --repo E:/Git/ZirconEngine --deep）；先核对写入权限和归属清单，只将审查元数据写到 E:/cargo-targets/zircon-local/reports/jenkins-transition-monitor。扫描不完整、超时或失败时保留事实和证据，不能当作无异常。

核验当前 Jenkins selector、不可变实现、运行代次、原操作和真实控制器/agent 身份，再读取实际健康及待处理请求；不能因存在配置、Ready 或历史 fixture 通过就认为版本、构建、复用、合并和清理流程已经接管。

按过渡协议跟进当前人工合并：对账已有原会话交付、精确候选路径、父提交、文件哈希、依赖、验证和已有结果。归属不明、外部暂存、活动 Git 占用、锁身份不明或验收缺失时保留候选并记录阻塞。只有已审查并明确获准的操作批次才能由接管会话集成；不能仅凭 dirty、目录年龄或存在分支自动提交。日常版本与生成流程仍由 Jenkins 负责；本任务不自行创建分支、工作树、源码副本，不运行构建/生成/清理，不推送，不重启服务或恢复旧协调器。

保留未知源码、其他会话活动产物、共享缓存、Git 元数据及历史回执。记录本次实际观察时间、变化、Jenkins 健康、合并阻塞、保留项、实际操作及证据；未清理或归属未知的项目保持待处理，不能宣称关闭。读取原任务结果再重试，避免重复提交或重复接管。

与上次有效记录比较，仅在异常有实质变化、流程完成或失败、需要用户行动时在 Codex 提醒，用中文说明影响、证据和下一步。没有可行动变化时安静保存记录，不重复发送例行状态。只有版本/多工作树集成、生成、编译验证、产物复用、任务结束清理及定时维护均有当前 Jenkins 代次的真实验收与移交记录后，才结束临时合并接管，并通过 Codex 正式任务接口更新临时任务；不要直接编辑任务数据库。
```
