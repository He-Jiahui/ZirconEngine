# Jenkins 分支提交与统一定时合并

2026-10-17 用户再次明确：**共享 main 工作区必须纯净；任务必须在 Jenkins worktree 分支提交，合入最新主干内容后验证，再通过 PR 合并主干，最后快进同步本地 main**。任意会话均可提交 Jenkins，无需逐会话审批。所有常规 Git 写操作、候选准备、验证和清理由 Jenkins 执行。本次用户另行要求的本地主干基线提交是过渡例外，不授权今后继续在 main 开发或批量提交。

main 仅用于同步和只读检查。开始交付、准备候选、同步前后均检查主干分支、index.lock、未结束 Git 操作，以及暂存、工作区和未跟踪文件。任何脏内容均保留并交回原 owner；不得用 stash、reset、clean、忽略新增源码或自动提交外来内容来制造纯净状态。任务实现从一开始就在所属 worktree 中进行；权限仍由现有 canonical 路径及工作树 ownedPaths 约束。

## 归属与顺序

1. `zircon-management / commit-branch` 核验当前不可变实现、运行代次、真实 controller/agent、原工作树 ready 身份和精确归属文件。先通过 `git rev-parse --absolute-git-dir` 找到该工作树真实的 `index.lock`，有锁则先对账原 owner、进程身份及原生终止记录。锁的年龄、空内容或一次没有打开句柄均不能单独证明可释放。
2. Jenkins 仅提交本任务逐项列出的文件到原工作树分支。保留其他暂存和未提交内容；不等待编译，不检查或提交整树 dirty。记录原父提交、文件哈希、真实 commit SHA 及原请求；无法确认的初始化删除、外来改动、非 ready 或未完成 merge 保持阻塞。
3. 分支 commit 成功后，在 `.jenkins/state/management/branch-integration/requests/<operationId>.json` 登记不可变提交、owner session、验证配方、依赖和预期缺失路径。此时 `status=committed, integrationStatus=pending`；它不是产品验证通过或 main 已合并。
4. 唯一 `zircon-merge-queue` 每 15 分钟运行（`H/15 * * * *`），禁用并发构建，串行处理最多 8 个待处理任务。使用记录的 commit，而不是分支最新 HEAD。核验该提交相对 main 的功能范围；由 Jenkins 复用一个独立受管候选工作树，合入最新 main，验证实际候选。
5. 验证需要真实 Windows Job 终止、退出码、stdout/stderr EOF、输入和 HEAD 一致。保留该实际候选的验证回执；验证失败不得进入 PR 合并。任务分支发布及 PR 创建、审核、合并走已授权的正常远端分支流程，禁止直接推送 main。当前本地队列不执行远端 push 或 PR 写操作；缺少此远端执行能力或授权时，准确保留待办，不改走本地直接合并。
6. 队列通过 GitHub API 查找同仓库、同任务分支、精确 sourceCommit、目标 main 的 PR，再独立读取其真实 merged 状态。没有匹配 PR 或 PR 尚未合并时，状态为 `awaiting-pr`，本地 main 不动。只有 PR 实际合并提交可从 origin/main 到达、是本地 main 的后代，并且 Git tree 与已验证候选完全一致，才能快进本地 main。PR 合并结果改变、远端观察失败、本地主干尚未同步到远端或任一工作区变脏时保留原请求及证据，重新对账或验证；本地候选验证通过不再直接发布 main。
7. 失败或冲突保留 commit、证据和原请求，并在 `.jenkins/records` 生成带 `ownerThreadId` 的精确修复交接。写入收件箱不等于原聊天已读；可用 Codex 正式接口通知时另记录其回执。修复后提交新的明确变更，沿原合并记录对账，不能以新身份重放未知操作。

原任务工作树不会由该队列自动删除。Jenkins 后续维护必须先核对已交付提交、原生终止、未知源码和其他活动内容，再按精确计划清理。编译产物、临时目录和缓存仅在获准的 `E:/cargo-targets`；保留可复用构建和失败证据。每日 Codex 巡检与此 Jenkins 合并定时流程是两个职责。

## 公开入口

```powershell
python -B -X utf8 tools/dev/jenkins_branch_queue.py submit --input-file '<任务请求.json>'
python -B -X utf8 tools/dev/jenkins_branch_queue.py query --operation-id task-commit-r1
```

```json
{
  "action": "deliver-task",
  "sessionId": "original-worktree-session",
  "ownerThreadId": "original-codex-thread-id",
  "requestId": "task-commit-r1",
  "operationId": "task-commit-r1",
  "sourceRoot": "E:/Git/ZirconEngine/.jenkins/worktrees/existing-task-worktree",
  "message": "Implement the scoped task",
  "paths": ["tools/example.py"],
  "dependencies": [],
  "validation": [{"kind": "python-static", "paths": ["tools/example.py"]}]
}
```

`paths` 必须属于原工作树的 ownedPaths，并匹配 canonical 写权限；目录和通配符不作为提交范围。适当验证仍由任务负责，静态检查不能代替 Rust 编译、测试或产品验收。配方支持 `pytest`、`python-static`、`skills`、`cargo`，沿用已有公开参数约束。

相同输入重复提交只返回原回执。已知 `waiting/failed` 在确认原 Jenkins 构建真正终止后才可 `submit --resume`；分支已经 commit 时不能重复提交或把其后改动混进旧请求。部署前的 `deliveries`、历史 `allowCommit:false`、失败 fixture 和旧 runtime 回执保持原样，不由新队列改写为成功。

PR 仓库固定为 policy 的 `pullRequestRepository`，并须匹配 origin。只读 API 可使用 Jenkins 已有 GH_TOKEN/GITHUB_TOKEN 或非交互 Git credential helper；凭证不写回执、不输出、不弹出登录。队列支持 merge、squash 或 rebase 后的实际 Git 提交，但均须满足已验证 tree 和本地快进条件。一次本地基线提交不代表远端已包含该基线；若两者历史无法快进，先走基线的正常 PR 流程完成对齐，不能重写 main 来跳过检查。

`submit --validate-only` 只供修复协调器时在 Jenkins 检查精确的实现文件；它不提交 Git，也不登记产品合并。`install-entry` 安装管理路由，`install` 另安装统一定时 job；配置存在与 cron 注册不代替该代次真实运行、产品验证和 main 合并验收。
