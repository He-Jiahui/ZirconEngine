# 普通本地提交

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
