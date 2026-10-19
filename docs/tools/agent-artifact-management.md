---
doc_type: workflow-detail
created: 2026-10-11
---

# Jenkins 管理源码、版本流程与生成产物

用户要求 Jenkins 统一负责所有版本管理和文件生成流程，包括源码封存、分支与工作树创建、多工作树合并、验证编译、产物复用、任务结束清理和定时清理。AI 会话负责读取、设计、编辑明确授权的规范源码/文档/技能、准备增量输入，并核对 Jenkins 的实际回执。任务另有明确例外授权时按该授权处理；目录可写本身不授予例外。

使用用户的 [artifact-lifecycle](D:/CodexData/.codex/skills/artifact-lifecycle/SKILL.md) 清单与保留格式。
其通用本地编译/清理步骤在本项目由 Jenkins 的受管流程执行，不能成为会话绕过 Jenkins 的入口。

普通编译遵守[编译存储强制规则](jenkins-build-storage-policy.md)：显式模块与 features、
关闭默认 features、定向测试、dev/test 禁用增量缓存和调试符号，使用兼容共享池。
[原 Codex 每日任务](jenkins-transition-monitor.md)比较新增占用、记录原请求与会话候选，
对已证实的浪费交给 Jenkins 纠正；增长或目录年龄本身不能证明可删除。

## 本机请求自动受理

普通本机补丁、编译和测试请求由当前 Jenkins 入口自动登记，不需要每个会话领取授权、人工发布许可或等待另一个会话代签。会话与请求 ID 用于归属、幂等和结果查询。入口自动记录本机运行身份及请求上下文，继续核对补丁与依赖哈希、路径、冲突和实际验证回执。

原来已明确拒绝且未入队的请求保留失败记录，并按关联尝试接续；交付结果不明的请求先核对原操作，不重复提交。现有工作树的窄 writer 范围保持原归属，提交、推送和通知仍按各自已有动作规则执行。入口能力不匹配时修复部署，不能再把人工会话登记作为普通验证的前置条件。

当前用户已授权 Jenkins 对明确选定的业务和工具文件执行普通本地提交，并要求取消哈希对账和逐会话提交许可。已完成范围审阅的业务改动使用 [普通本地提交入口](jenkins-local-commits.md)：`tools/dev/jenkins_commit.py submit`，复用实际 checkout，不等待整树封存、配置/依赖哈希或旧 formal flow。活动读取、文件写权限和原生 Git 锁仍保留；产品测试与里程碑验收记录如实报告。该本地提交授权不包含推送或外部通知。

在这一已有授权内，完成范围审阅的任务交付明确的文件清单和提交说明，可使用 [本地提交队列](jenkins-commit-queue.md) 或 `submit-jenkins-management.ps1 -Action commit-checkout`。入口会保存原请求，由即时构建或每五分钟的 `zircon-git-followup` 完成；等待锁和回执不明的请求会自动接续。核对回执中的实际提交号后再报告已提交；验证成功或收件成功不能代替 Git 提交，未交付清单的其他会话改动保持原状。

现有 `zircon-scheduled-cleanup` 的半小时触发器还承担[共享 Rust 增量缓存清理](jenkins-incremental-cleanup.md)。已解锁的指定编译缓存可直接按物理路径、原生锁和文件身份清理，无需让未知源码、历史归属对账或缺席 inventory 阻止本轮缓存回收。

## 管理边界

| 事项 | Jenkins 的职责 | AI 会话的职责 |
| --- | --- | --- |
| 分支、工作树、checkout | 创建、绑定任务、切换、释放和删除；记录仓库、base SHA、归属及操作身份 | 使用任务指定的 checkout；请求需要的版本操作，不自行建树或切换 |
| 多工作树合并与冲突 | 固定输入 SHA、作者范围和合并目标，排他合并，在结果树上验证，保留失败回执 | 提供获准的差异与冲突分析；只编辑明确授予的解决路径 |
| 提交、回滚、rebase、推送 | 根据已有动作授权执行，保全共享索引和外来改动，返回精确 SHA/操作回执 | 请求并核对原操作；不因任务完成推定提交、推送或通知授权 |
| 源码基线、候选、封存输入 | 准备、复用、校验来源并管理保留引用 | 提供有范围的补丁、before/after 哈希、依赖及测试范围 |
| 生成代码、资源、导出包、截图、日志及测试中间文件 | 受管 recipe 生成、登记物理位置/哈希/所有者、归档必要证据 | 提供生成参数和交付要求，引用 Jenkins 发出的产物身份 |
| 编译与测试 | 根据验证阶段选择覆盖，准入资源、复用产物、编译、执行并证明进程终结 | 提交必要范围，核对实际执行或复用证据，保持验收状态准确 |
| 热构建池、编译缓存、共享缓存 | 兼容性选择、排他 writer、引用计数、空间准入及回收 | 不创建替代池、私有 target 或临时编译副本 |
| 任务结束与定时清理 | 根据受管清单、保留引用和原生终结证据执行有界 GC，返回实际删除结果 | 核对原请求和清理回执，报告仍被保留的内容 |
| 遗留源码副本、旧缓存和无归属目录 | 核对并接管合法资源；不能证明所有权的内容保持保护 | 只读盘点、指出差异及证据缺口，交给受管维护流程处理 |

AI 不执行版本管理写操作，也不直接运行生成型 recipe、Cargo 构建、`cargo clean`、产物移动/删除、Windows 定时清理任务或自行恢复旧协调器。需要的流程没有已核验入口时，记录能力缺口并保留原数据；继续不依赖该入口的授权工作。不得用本地 wrapper、后台服务、另一个工作树或源码包代替未完成的 Jenkins 请求。

只读的 `git status`、`git diff`、源码/目录盘点与结构检查可以本地执行，其输出只是诊断信息。规范源码、文档和技能的创作按当前任务明确的文件范围进行；新增非规范文件树或生成产物由 Jenkins 发出身份并管理，不能由会话任意创建 `tmp`、`backup`、`final2`。

## 使用现有 Jenkins 流程

- **`zircon-flow`**：补丁/输入、校验、验收与获准的 Git 集成。增量入口及已修改 checkout 的交接规则见 [jenkins_validate.py](../../tools/dev/jenkins_validate.py) 与[源码所有权](../../.codex/skills/jenkins-coordination/references/incremental-patch-validation.md)。
- **`zircon-execution`**：共享执行、受管生成、编译、测试和产物复用。既有稳定池保持完整输入、平台、工具链、profile、features 和覆盖身份；不按会话创建新的 target。
- **`zircon-maintenance`**：清单核对与有界回收；任务结束的缓存评估由 [post_task_cache](../../tools/jenkins/workflow/post_task_cache.py) 契约派生原 execution 的独占 TEMP。候选目录不能由会话伪造为已注册资源。

以当前选中的 `.jenkins/state/deployment/driver.json`、不可变 driver、实际节点/runtime 身份和已验收的接口为准。参考[工程流程规格](../../.jenkins/workflow-spec.json)、[Jenkins 说明](../../.jenkins/README.md)和[托盘](../tooling/jenkins-tray.md)。旧 pilot 的 bundle 提交不能替代正式增量入口；代理不得自己捕获源码包来满足它。

存在代码、job 或设计规格不等于已经启用。检查 selected driver 是否包含相应实现、运行策略是否启用、job 参数/触发器是否绑定当前身份，以及最近的实际回执。定时清理周期来自 Jenkins 的已授权配置；不在 Codex 或 Windows 新建另一套计划任务。本文定义所有者和约束，不自行启用计划任务或改变生产 driver。

## 源码和多工作树约束

遵守[源码所有权与增量补丁工作流](../../.codex/skills/jenkins-coordination/references/incremental-patch-validation.md)：代理不得创建整树或选中文件的源码备份、快照、源码压缩包、验证 checkout 副本，也不把完整前后源码藏在 `.txt`、`.bak` 或证据 JSON 里。补丁和哈希不附带源码备份。源码基线、候选和源码包由 Jenkins 准备并保留其来源身份。

多工作树流程至少绑定 repository、worktree/branch、不可变 base SHA、作者路径范围、依赖输入、目标 branch 和原操作身份。合并前核对 HEAD/输入 hash 与冲突；合并后验证实际结果树。联合树的成功不能接受未验证的子集。提交、推送和回滚保持原动作授权；重试核对原操作，不以换 request ID 隐藏未知结果。删除工作树前由 Jenkins 证明 writer 已终结且独有改动已交付。

现有默认共享 main 策略继续有效。请求多工作树时，由 Jenkins 的已实现、已核验流程承接；规格若禁用 worktree 或缺少合并入口，记录为未满足的能力，AI 不自行绕过。

## 存储、引用与收尾

写入权限以 [config.toml](../../.codex/config.toml) 为准，实际约束需按[激活与实测](../tooling/codex-file-permissions.md)核验。Jenkins 管理权限和沙箱仍在各自边界内运行；Jenkins 所有权不授予未列出的写入权限。所有编译产物及编译器缓存物理位于盘根 `D:\cargo-targets`、`E:\cargo-targets` 或 `F:\cargo-targets` 的获准命名空间。禁止 C 盘、repo-local target、旧 `targets`/`ZirconBuilds`、嵌套同名根和路径别名。保留 35 GiB 空间准入要求。

Jenkins State 中的 request、source、execution、artifact、native_job 与清理回执是生命周期权威。会话交接说明只引用这些身份、精确路径和哈希，不另造一套产物登记库、租约或自动清理服务。`.codex/outbox` 只存获准交接的补丁/输入 metadata；`.codex/sessions` 只存交接与证据链接，不能变成备份树或一次性生成器仓库。

按 `artifact-lifecycle` 在 `reports/<task-id>/artifacts.json` 记录实际 owner、checkout、路径、
依赖和保留条件。该相对布局位于 config 获准的物理输出命名空间，例如
`E:/cargo-targets/zircon-local/reports/<task-id>/artifacts.json`，不能在无写授权的仓库根新增
`reports`。这是 Jenkins 交接与审阅用的普通 metadata；引用 State 的原请求和回执，
不以清单文字授予 GC 或代替其权威。已有产物先登记真实原位置，不为统一目录重复制。

每个任务收尾都要核对：

1. 原 request/attempt 的生成、验证、合并与交付状态，以及产物/源码的完整身份。
2. Jenkins 的进程/Job 终结及 EOF 证据；超时、session ID 或 cancel_requested 不等于终结。
3. 受管引用、仍待验收的输入和历史 holds；待核对请求不按年龄过期。
4. 原任务的维护评估和有界清理回执。报告实际删除、保留原因和仍待处理的路径。

维护失败单独记录，不重写源码验收、提交结果或历史失败。保留[退役协调器资料](../tooling/coordinator-retirement.md)以及共享索引、锁和原生进程身份。`zircon_runtime/build` 是规范源码，不能按名字回收。

## 遗留目录只读盘点

[盘点工具](../../tools/dev/artifact_inventory.py)读取真实白名单，统计显式 write grants 和输出子目录，不追随 reparse point；输出 JSON 到 stdout，文件归档由 Jenkins 的受管执行负责。工具本身不写报告、不移动数据、不删除目录、不创建源码副本。

```powershell
& '.jenkins/runtime/python/python.exe' -B -X utf8 -m tools.dev.artifact_inventory
```

报告列出逻辑大小、读取错误、硬链接、源码树特征、现有清单和进程命令行引用。进程参数只在内存用于匹配，不进入报告。逻辑字节不等于唯一物理占用或可回收字节；源码特征不等于违规副本；缺少 manifest、旧时间和无命令行命中不能证明可以删除。

无归属内容标为 `owner-review-required`。提交维护候选时附精确路径、用途、来源、与规范源码的差异/哈希、保留引用和已知 writer。独有源码差异保持原文并交给所有者；Jenkins 完成归属核验和资源准入后才能清理。盘点报告不是 GC inventoryDigest，也不是 Jenkins 验收回执，不能用它伪造 intake、release、native termination 或 acceptance。
