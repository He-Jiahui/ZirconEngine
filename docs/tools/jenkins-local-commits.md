# Jenkins 验证、提交与主干集成

2026-10-17 用户进一步明确：**main 工作区保持纯净；任意会话先由 Jenkins 在自己的工作树分支提交，合入最新 main 内容并验证，通过真实 PR 合并后才快进同步本地主干**。不再逐会话、逐提交或逐本地同步申请许可。当前流程、状态含义与入口见 [Jenkins 分支提交与统一定时合并](jenkins-branch-integration.md)。缺少已合并 PR 证据时状态为 `awaiting-pr`，禁止直接把本地候选合入 main。本页下方保留早期接口的对账说明；新任务不再直接在共享 main commit 或即时合并。

新请求使用 `tools/dev/jenkins_branch_queue.py`，必须包含原 ready 工作树的 `sourceRoot`、`sessionId` 和用于失败交接的 `ownerThreadId`。下面的旧请求字段仅用于理解历史回执，实际新请求使用上方流程文档的完整示例：

```json
{
  "action": "deliver-task",
  "sessionId": "my-task",
  "requestId": "my-task-delivery",
  "operationId": "my-task-delivery",
  "message": "Implement the task",
  "paths": ["tools/example.py", "tools/tests/test_example.py"],
  "dependencies": [],
  "validation": [
    {"kind": "pytest", "paths": ["tools/tests/test_example.py"]}
  ]
}
```

将请求保存在已获写权限的任务 outbox 或报告目录，然后调用：

```powershell
& '.jenkins/runtime/python/python.exe' -B -X utf8 tools/dev/jenkins_delivery.py submit --input-file '<任务请求.json>'
& '.jenkins/runtime/python/python.exe' -B -X utf8 tools/dev/jenkins_delivery.py query --operation-id my-task-delivery
```

入口核验当前 Jenkins driver、不可变管理实现、运行代次和真实 controller/agent。它在现有 `zircon-management` 上扩展路由，保留已有动作与运行任务，不重启服务。验证在 Jenkins agent 的保留 Windows Job 中执行；需要进程树终止、真实 stdout/stderr EOF、退出码为零，以及任务输入未变。任务文件及声明依赖按范围核验，不要求整树哈希。验证失败不会提交。

`validation` 可选 `pytest`、`python-static`、`skills`、`cargo`。前三种使用逐项列出的 Python 文件或 `SKILL.md`；`cargo` 使用 `arguments` 数组，例如 `["test", "-p", "zircon_runtime_interface", "--lib", "--offline"]`，按公开 Cargo 参数规则执行，并使用现有 MSVC 环境入口。编译产物、临时目录和缓存只写 `E:/cargo-targets`；保留同一操作的构建以便复用，Jenkins 在原生终止后按清单清理本次临时夹具。测试范围必须适合本次修改，静态解析不能替代产品编译和定向测试。

旧流程曾直接验证并提交 main，或在工作树中即时验证和合并；该顺序已被用户替换。现在先精确提交任务工作树分支，保留其他改动，再排入唯一 `zircon-merge-queue`。定时流程由 Jenkins 准备和验证实际候选；只有它可以发布 main。冲突保留分支 commit 和证据，不 stash/reset 其他会话的内容。

结果和原生证据保存在 `.jenkins/state/management/deliveries/<操作ID>.json`、`E:/cargo-targets/zircon-local/reports/jenkins-deliveries/<操作ID>` 和 Jenkins 构建归档。重复调用同一输入返回原提交结果。需要继续 `waiting`、`validation-failed` 或 `merge-conflict` 时先读原结果，修复原因，并对同一请求使用 `submit --resume`；只有原构建实际终止后才允许继续。提交响应不明或原生终止证据不完整时先对账，不更换操作 ID 重放。

工作树归属、任务文件边界、活动 Git 锁、外部暂存内容和冲突是执行条件，不是再次向用户申请提交权限的理由。旧请求中的 `allowCommit: false` 不原地改写；通过本入口发起相应任务的已授权交付。推送、发布和外部通知仍需另行授权。

## 历史独立提交接口的对账

任务清单内的 `.codex/skills/` 规范技能文件可由 Jenkins 精确加入 Git，保留其余忽略内容。

历史 `commit-checkout` 请求仍可查询原结果。新流程启用后，共享 main 的独立 commit 入口应返回“先提交任务工作树、由统一队列合并”，不能绕过队列。无需新的人工授权。

```powershell
& '.jenkins/runtime/python/python.exe' -B tools/dev/jenkins_commit.py submit `
  --session-id my-task --request-id my-task-commit --operation-id my-task-commit `
  --message 'Fix the scoped task' `
  --path tools/example.py --path docs/tools/example.md
```

入口复用 `zircon-management` 的 `commit-checkout` 动作，使用当前实际 checkout，不创建工作树或源码副本。只提交逐项列出的文件，支持新增、修改与删除；保留其他文件的未提交改动和暂存内容。列表需属于任务范围，不能用目录、通配符或整个仓库代替。文件写权限、原生 Git 锁和活动源码读取仍然生效；它们不要求用户重复授权。

请求 ID 和操作 ID 用于查询与避免重复提交。查询同一输入时加 `--query-only`；中断后继续使用同一个操作 ID，不换 ID 重放。实际结果保存在 Jenkins 构建归档及 `.jenkins/state/management/checkout-commits/<操作ID>.json`。

历史空锁隔离回执保留在 `.jenkins/state/management/recovered-index-locks`。新流程先检查每棵工作树真实 Git 目录的锁；须对账原 owner、进程和操作结果后再由 Jenkins 释放。空锁、目录年龄或暂时没有句柄不能单独作为释放依据。

该入口返回实际本地提交号。按需执行的定向测试保持原有职责，普通提交不再依赖旧 formal flow 的验收记录。产品、里程碑和迁移验收仍以各自实际测试为准。推送、发布和外部通知需要另外授权。

安装管理 job 时若覆盖了扩展，此入口会在下一次提交时恢复同一动作；也可运行 `tools/dev/jenkins_commit.py install`。安装只扩展当前管理 job，保留其已有路由和正在执行的 Pipeline，不重启 Jenkins。
