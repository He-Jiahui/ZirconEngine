# ZirconEngine Session Goal 里程碑收口技能设计

## 目标

新增一个仓库本地 Codex 技能，统一处理当前工程中 Session Goal 的两类完成边界：

1. 单个里程碑完成，但同一 Goal 后续仍有里程碑需要推进；
2. 整个 Goal 完成，Session 可以正式结束。

两类边界都必须形成普通 Git commit。里程碑之间的未完成版本继续由本地 Session 协调服务管理，不创建 checkpoint、Session 标签提交、分支或 worktree。

## 选型

### 方案 A：纯流程文档技能

只在 `SKILL.md` 中描述收口步骤。实现成本最低，但无法确定性检查遗漏的未跟踪文件、文档、测试、脚本或暂存范围，不适合共享 `main` 的多 Session 环境。

### 方案 B：将全部规则固化进协调服务

新增专用服务状态和提交 API。强制力最高，但会把“何时视为里程碑完成”的计划语义耦合进服务内核，也扩大本次技能任务的代码范围。

### 方案 C：仓库技能加只读收口检查器

由技能负责完成语义和操作顺序，由小型只读脚本检查当前 Session、计划证据、文件分类、Git 暂存范围和敏感信息；实际状态、租约、归属仍由协调服务持有，Git 提交保持普通仓库提交。

采用方案 C。它不复制协调服务的状态机，同时把最容易遗漏的机械约束自动化。

## 技能布局

技能放在：

```text
.codex/skills/zircon-project-skills/close-session-goal-milestones/
  SKILL.md
  agents/openai.yaml
  scripts/check-closeout.ps1
  scripts/check-closeout.Tests.ps1
```

技能保持单层、聚焦，不再拆分子技能。完成后刷新：

- `.codex/skills/zircon-project-skills/SKILL.md`
- `.codex/skills/project-skills-index/catalog-existing-skills/current-project-skills.md`

## 触发语义

当出现以下任一条件时使用技能：

- 用户明确表示当前里程碑、阶段或 Goal 已完成；
- 实施计划的 testing stage 已通过，下一步是记录证据并进入下一里程碑；
- 当前 Goal 所有计划项已完成，需要结束 Session；
- Session 准备提交阶段性成果，但必须保留其他并行 Session 的工作区改动。

“实现切片完成”不等于“里程碑完成”。只有计划中该里程碑的所有切片及 testing stage 都有验收证据时，才能触发提交。

## 里程碑完成流程

1. 查询协调服务健康、当前 `main`、Session 枚举状态、Failure 图和当前计划。
2. 确认该里程碑所有实现切片与 testing stage 已完成；证据只写入对应编号子计划目录，不写全局计划索引或定义。
3. 心跳并持有本次提交文件的租约；按当前内容哈希记录归属。
4. 生成精确收口清单，必须分别列出：
   - 生产代码；
   - 文档；
   - 测试；
   - 工具/脚本；
   - 原先未跟踪、现需纳入版本管理的文件。
5. 检查没有 foreign staged path、未申报的当前 Session dirty path、Session 标签提交信息、webhook URL、维护令牌或凭据。
6. 只暂存清单中的文件并复核暂存集合完全相等。
7. 立即创建普通 Conventional Commit。提交信息不得包含 `[zircon-session:*]` 或 checkpoint 语义。
8. 每次成功 commit 后立即推送一次企业微信，不自动重试失败通知。格式固定为四行：
   - `核心内容摘要：...`
   - `提交时间：...`
   - `修改情况统计：...`
   - `提交的commit内容：...`
9. 若 Goal 尚未完成，保持或恢复 Session 为 `active`，释放本里程碑不再需要的文件租约，继续下一里程碑；不得把 Goal 标为 complete。

## 整个 Goal 完成流程

整个 Goal 使用相同的检查、分类、提交和企业微信通知流程，并额外执行：

1. 确认计划中没有剩余 pending/in-progress 项，也没有属于本计划的未解决 Failure；
2. 确认协调器范围在提交后无未暂存差异；
3. 释放该 Session 全部租约并处理其可安全执行的延迟 patch；
4. 将 Session 状态设为 `completed`，写入最终 commit SHA 和完成原因；
5. 只有在全部目标真正完成后，才把 Codex Goal 标为 complete；
6. 最终回复列出 commit、验证证据、仍属于其他 Session 的诊断和企业微信发送结果。

## 共享 main 与异常处理

- 不创建分支、worktree、stash 或隐藏 checkpoint commit。
- 其他 Session 的 dirty/untracked 文件是正常并发状态，不纳入、不回退、不清理。
- 文件租约冲突时不能覆盖；提交延迟 patch 或继续不冲突工作。
- 全局基线因其他 Session 变更而 degraded 时，不得吸收其内容。只有在当前 Session 文件已持有租约、按哈希归属、暂存集合精确且无 foreign staged path 时，才能形成该里程碑的 scoped commit。
- 验证失败时保留工作区内容，修复最低共享层后重新执行 testing stage；不得为了满足“里程碑即提交”而提交失败状态。
- 企业微信发送失败不回滚已经成功的 Git commit，也不自动重试；明确报告失败即可。

## 收口检查器

`check-closeout.ps1` 是只读检查器，不执行暂存、提交、状态迁移或消息发送。输入包含 Session ID、完成类型（`Milestone`/`Goal`）和显式路径清单，输出结构化 JSON，并以非零退出码报告：

- 服务离线或非 `main`；
- Session 不存在、状态不允许收口或 Goal 收口仍有计划项；
- 清单路径不存在、越界或分类缺失；
- 当前 Session 已归属的 dirty path 未全部列入；
- 暂存区包含清单外路径；
- 提交信息包含 Session/checkpoint 标签；
- staged diff 包含 webhook、维护能力或凭据模式。

脚本测试使用临时 Git 仓库和模拟协调器 JSON，不读取或修改真实业务文件。

## 验收

- 技能 frontmatter、名称、`agents/openai.yaml` 通过 `quick_validate.py`；
- PowerShell 测试覆盖 Milestone/Goal、遗漏未跟踪文件、foreign staged path、非法提交信息、敏感信息和合法收口；
- 技能能在共享 dirty `main` 中只生成显式范围清单；
- 项目技能浅层目录和缓存目录已刷新；
- scoped `git diff --check` 和敏感信息扫描通过；
- 独立复审无 Critical/Important；
- 设计、实现按正常工作流提交，每次提交后均发送四行企业微信详情。
