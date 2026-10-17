# Jenkins 分支提交与统一定时合并

2026-10-17 用户更新了流程顺序：**先在任务自己的工作树分支 commit，再登记合并请求；统一定时流程验证实际合并候选，通过后才推进 main**。任意会话均可提交 Jenkins，无需逐会话审批。所有 Git 写操作、候选准备、验证和清理由 Jenkins 执行；不推送远端。

## 归属与顺序

1. `zircon-management / commit-branch` 核验当前不可变实现、运行代次、真实 controller/agent、原工作树 ready 身份和精确归属文件。先通过 `git rev-parse --absolute-git-dir` 找到该工作树真实的 `index.lock`，有锁则先对账原 owner、进程身份及原生终止记录。锁的年龄、空内容或一次没有打开句柄均不能单独证明可释放。
2. Jenkins 仅提交本任务逐项列出的文件到原工作树分支。保留其他暂存和未提交内容；不等待编译，不检查或提交整树 dirty。记录原父提交、文件哈希、真实 commit SHA 及原请求；无法确认的初始化删除、外来改动、非 ready 或未完成 merge 保持阻塞。
3. 分支 commit 成功后，在 `.jenkins/state/management/branch-integration/requests/<operationId>.json` 登记不可变提交、owner session、验证配方、依赖和预期缺失路径。此时 `status=committed, integrationStatus=pending`；它不是产品验证通过或 main 已合并。
4. 唯一 `zircon-merge-queue` 每 15 分钟运行（`H/15 * * * *`），禁用并发构建，串行处理最多 8 个待处理任务。使用记录的 commit，而不是分支最新 HEAD。核验该提交相对 main 的功能范围；由 Jenkins 复用一个独立受管候选工作树，合入最新 main，验证实际候选。
5. 验证需要真实 Windows Job 终止、退出码、stdout/stderr EOF、输入和 HEAD 一致。main 与候选无锁、原生读取已释放、main 未漂移时才快进 main。共享 checkout 的外来暂存或工作改动阻止更新时保留候选及分支提交，不 stash/reset 或覆盖其他会话。
6. 失败或冲突保留 commit、证据和原请求，并在 `.jenkins/records` 生成带 `ownerThreadId` 的精确修复交接。写入收件箱不等于原聊天已读；可用 Codex 正式接口通知时另记录其回执。修复后提交新的明确变更，沿原合并记录对账，不能以新身份重放未知操作。

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

`submit --validate-only` 只供修复协调器时在 Jenkins 检查精确的实现文件；它不提交 Git，也不登记产品合并。`install-entry` 安装管理路由，`install` 另安装统一定时 job；配置存在与 cron 注册不代替该代次真实运行、产品验证和 main 合并验收。
