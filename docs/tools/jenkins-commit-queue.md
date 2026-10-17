# 普通本地提交的持久队列

用户已授权 Jenkins 对明确选定的任务文件执行本地提交。完成范围审阅的任务可通过 `tools/dev/jenkins_commit.py submit` 或通用入口 `tools/dev/submit-jenkins-management.ps1 -Action commit-checkout` 交付。入口复用实际 checkout，不创建工作树或源码副本，不要求整树哈希、逐会话授权或旧 formal flow 验收。文件写权限、活动读取和原生 Git 锁继续生效；产品验证如实记录。

请求只列本任务已准备好的文件，例如：

```json
{
  "action": "commit-checkout",
  "sessionId": "my-task",
  "requestId": "my-task-commit",
  "operationId": "my-task-commit",
  "paths": ["tools/example.py", "tools/tests/test_example.py"],
  "message": "Fix the task"
}
```

将 JSON 放入已获写权限的任务报告目录或 outbox，然后调用：

```powershell
& tools/dev/submit-jenkins-management.ps1 -Action commit-checkout `
  -SessionId my-task -RequestId my-task-commit -OperationId my-task-commit `
  -InputFile '<任务请求.json>'
```

入口先把精确请求保存到 `.jenkins/state/management/checkout-queue/<操作ID>.json`，再投递 Jenkins。即时构建与 `zircon-git-followup` 共用原操作 ID 和 Git 锁。定时任务每五分钟接续等待锁、活动读取或 HTTP 回执不明的请求，每轮最多十二条。它只处理已交付的文件清单，保留其他文件的暂存内容和工作区改动。

需要下一次轮询提交时，加 `-DeferCommit`，或 Python 入口加 `--defer`。原生 Git 命令超过时限时，先终止并等待本次拥有的进程树，再以 `waiting/native_git_timeout` 自动接续；如果提交已经写入，下一次恢复同一操作的实际提交号。确定的输入错误标为 `failed` 并保持告警，修复原因后显式使用相同请求重试。请求交付不明时查询原操作，不能换 ID 重放。

查询使用同一命令加 `-QueryOnly`，或 Python 入口加 `--query-only`；查询直接读取原回执，不新建 Jenkins 构建。回执位于 `.jenkins/state/management/checkout-commits/<操作ID>.json` 和 Jenkins 构建归档。只有实际 `commit` 才表示已经提交；`queued`、`waiting`、验证成功或收件成功都不是提交完成。

提交前失败且其他任务推进 HEAD 时，重试先查找原操作的实际提交。已提交则恢复原回执，尚未提交则基于当前 HEAD 继续原文件清单并保留之前尝试的记录。再次调用已完成请求返回原提交号，不会把后续的新改动重新提交。

`tools/dev/jenkins_commit.py install` 安装提交动作、回归入口和每五分钟的接续 job；保留现有路由及运行任务，不重启 Jenkins。该入口执行本地提交；推送、发布和外部通知仍需另行授权。
