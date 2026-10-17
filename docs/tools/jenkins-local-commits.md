# Jenkins 验证、提交与主干集成

2026-10-17 用户明确授权：**任意会话都可提交 Jenkins，验证通过后由 Jenkins 自动执行任务的本地 Git commit，并将功能合入 `main`**。不再逐会话、逐提交或逐合并申请许可，不以整个工作区或旧 formal flow 验收作为普通任务的前置条件。持续授权保存在 `tools/jenkins-management/local-commit-policy.json`，适用于后续会话。

优先使用 `tools/dev/jenkins_delivery.py` 的 `deliver-task` 路由。请求只列本任务的文件、必要依赖和实际验证步骤，例如：

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

默认直接在真实 `main` checkout 验证和提交；该提交已经位于主干。已有 Jenkins 工作树可传入 `sourceRoot`，使用它原来的 `sessionId` 表明交付归属。Jenkins 只提交列出的任务文件；分叉时先合入当前 main 并验证实际候选，再将 main 快进到该候选。冲突只保留证据和已验证的功能提交，不制造未完成合并、不 stash/reset 其他会话的内容。此入口不创建源码副本或新工作树。

结果和原生证据保存在 `.jenkins/state/management/deliveries/<操作ID>.json`、`E:/cargo-targets/zircon-local/reports/jenkins-deliveries/<操作ID>` 和 Jenkins 构建归档。重复调用同一输入返回原提交结果。需要继续 `waiting`、`validation-failed` 或 `merge-conflict` 时先读原结果，修复原因，并对同一请求使用 `submit --resume`；只有原构建实际终止后才允许继续。提交响应不明或原生终止证据不完整时先对账，不更换操作 ID 重放。

工作树归属、任务文件边界、活动 Git 锁、外部暂存内容和冲突是执行条件，不是再次向用户申请提交权限的理由。旧请求中的 `allowCommit: false` 不原地改写；通过本入口发起相应任务的已授权交付。推送、发布和外部通知仍需另行授权。

## 已有的独立提交入口

任务清单内的 `.codex/skills/` 规范技能文件可由 Jenkins 精确加入 Git，保留其余忽略内容。

用户已授权 Jenkins 对当前 checkout 的任务文件执行本地提交。此授权不再按会话领取，也不要求整树、配置或依赖的 SHA256 对账。实际策略由 `tools/jenkins-management/local-commit-policy.json` 保存并由提交入口读取。

```powershell
& '.jenkins/runtime/python/python.exe' -B tools/dev/jenkins_commit.py submit `
  --session-id my-task --request-id my-task-commit --operation-id my-task-commit `
  --message 'Fix the scoped task' `
  --path tools/example.py --path docs/tools/example.md
```

入口复用 `zircon-management` 的 `commit-checkout` 动作，使用当前实际 checkout，不创建工作树或源码副本。只提交逐项列出的文件，支持新增、修改与删除；保留其他文件的未提交改动和暂存内容。列表需属于任务范围，不能用目录、通配符或整个仓库代替。文件写权限、原生 Git 锁和活动源码读取仍然生效；它们不要求用户重复授权。

请求 ID 和操作 ID 用于查询与避免重复提交。查询同一输入时加 `--query-only`；中断后继续使用同一个操作 ID，不换 ID 重放。实际结果保存在 Jenkins 构建归档及 `.jenkins/state/management/checkout-commits/<操作ID>.json`。

Windows 上，超过十分钟且内容为空的 `index.lock` 只有在原生独占打开成功、文件身份再次吻合时才会恢复；原文件移动保留在 `.jenkins/state/management/recovered-index-locks` 并记录回执。仍有句柄、较新或含待写索引内容的锁继续等待，不删除锁来掩盖活动操作。

该入口返回实际本地提交号。按需执行的定向测试保持原有职责，普通提交不再依赖旧 formal flow 的验收记录。产品、里程碑和迁移验收仍以各自实际测试为准。推送、发布和外部通知需要另外授权。

安装管理 job 时若覆盖了扩展，此入口会在下一次提交时恢复同一动作；也可运行 `tools/dev/jenkins_commit.py install`。安装只扩展当前管理 job，保留其已有路由和正在执行的 Pipeline，不重启 Jenkins。
