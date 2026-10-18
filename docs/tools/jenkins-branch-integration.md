# Jenkins 分支提交与统一定时合并

2026-10-17 用户最新要求：**降低 Jenkins 验证要求，让所有会话自由提交；先提交，错误交回原会话自行修正。** 普通提交、定时集成的验证统一采用 `validationMode=advisory`：配方可省略，检查失败如实记录但不阻断分支提交或 PR 集成。不要求逐文件哈希、完整编译、formal acceptance 或旧管理包验收。Jenkins 在线且原任务文件归属明确即可提交，不需逐会话审批。

任务仍在所属 Jenkins worktree 分支提交，主干通过真实 PR 快进同步。提交只包含明确列出的文件；保留其他会话的改动、活动 Git 锁和真实合并冲突。这些 Git 执行条件用于防止覆盖源码。普通分支提交不要求共享 main 纯净，只有主干同步需要 main 纯净。

main 仅用于同步和只读检查。开始交付、准备候选、同步前后均检查主干分支、index.lock、未结束 Git 操作，以及暂存、工作区和未跟踪文件。任何脏内容均保留并交回原 owner；不得用 stash、reset、clean、忽略新增源码或自动提交外来内容来制造纯净状态。任务实现从一开始就在所属 worktree 中进行；权限仍由现有 canonical 路径及工作树 ownedPaths 约束。

## 归属与顺序

1. `zircon-management / commit-branch` 使用当前在线 Jenkins 实例、原任务工作树和精确归属文件。普通交付不扫描完整驱动哈希，不要求旧 capsule 与当前运行代次一致，原工作树不因驱动更新而失去提交资格。通过 `git rev-parse --absolute-git-dir` 找到实际 `index.lock`；保留仍需原 owner 处理的锁。
2. Jenkins 使用原生 Git 仅提交本任务逐项列出的文件到原分支，保留其他暂存和未提交内容。提交前不执行验证、逐文件哈希或整树检查，也不等待 main 的其他会话完成工作。记录父提交、实际 commit SHA 和原请求；操作中断后按 Git 提交及操作标记恢复原结果。
3. 分支 commit 成功后，在 `.jenkins/state/management/branch-integration/requests/<operationId>.json` 登记不可变提交、owner session、验证配方、依赖和预期缺失路径。此时 `status=committed, integrationStatus=pending`；它不是产品验证通过或 main 已合并。
4. 唯一 `zircon-merge-queue` 每 15 分钟运行（`H/15 * * * *`），禁用并发构建，串行处理最多 8 个待处理任务。使用记录的 commit，而不是分支最新 HEAD。核验该提交相对 main 的功能范围；由 Jenkins 复用一个独立受管候选工作树，合入最新 main，验证实际候选。
5. `validation` 可省略或为空，回执记为 `skipped`，不宣称测试通过。提供检查时保留真实结果；失败设置 `repairRequired=true` 并交回原会话，同时继续 PR 集成。等待真实检查进程退出后才能复用该候选目录。相同失败结果在等待 PR 时可以复用，避免每轮重跑。任务分支发布、PR 创建和合并继续使用已有远端授权；队列本身不增加远端写权限。
6. 队列查找同仓库、同任务分支、精确 sourceCommit、目标 main 的真实 PR。PR 尚未合并时保留 `awaiting-pr`；合并后从 origin/main 取得真实合并提交，在本地 main 可快进且纯净时同步。建议性验证不要求 PR 结果 tree 与本地候选一致，也不要求检查全部通过。直接推送或重写 main 仍不属于此入口。
7. 检查失败保留 commit、日志和原请求，并在 `.jenkins/records` 生成带 `ownerThreadId`、`repairRequired`、`repairOf` 的修复待办。原会话读取后自行修正并立即提交，无需再次请用户批准或等待检查转绿。可在新请求携带 `repairsOperationId` 说明修复来源。写入收件箱不等于原聊天被唤醒或已经读取；没有正式聊天投递回执时不得宣称自动唤醒成功。交付不明的旧操作仍按原操作查询，不能重复提交。

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

`paths` 必须属于原工作树的 ownedPaths，并匹配 canonical 写权限；目录和通配符不作为提交范围。`dependencies` 和 `validation` 可省略。可选配方支持 `pytest`、`python-static`、`skills`、`cargo`，沿用已有公开参数约束。提交成功只说明 Git 已保存变更；测试失败仍为失败，产品和发布验收按实际证据表述。

相同输入重复提交只返回原回执。已知 `waiting/failed` 在确认原 Jenkins 构建真正终止后才可 `submit --resume`；分支已经 commit 时不能重复提交或把其后改动混进旧请求。部署前的 `deliveries`、历史 `allowCommit:false`、失败 fixture 和旧 runtime 回执保持原样，不由新队列改写为成功。

PR 仓库固定为 policy 的 `pullRequestRepository`，并须匹配 origin。只读 API 可使用 Jenkins 已有 GH_TOKEN/GITHUB_TOKEN 或非交互 Git credential helper；凭证不写回执、不输出、不弹出登录。队列支持 merge、squash 或 rebase 后的实际 Git 提交，但均须满足已验证 tree 和本地快进条件。一次本地基线提交不代表远端已包含该基线；若两者历史无法快进，先走基线的正常 PR 流程完成对齐，不能重写 main 来跳过检查。

`submit --validate-only` 只供修复协调器时在 Jenkins 检查精确的实现文件；它不提交 Git，也不登记产品合并。`install-entry` 安装管理路由，`install` 另安装统一定时 job；配置存在与 cron 注册不代替该代次真实运行、产品验证和 main 合并验收。
