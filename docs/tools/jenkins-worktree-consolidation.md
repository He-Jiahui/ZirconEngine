# Jenkins 分支与工作树收束

`tools/jenkins-management/worktree_consolidation.py` 为现有 `zircon-management`
添加有界 `consolidate-worktrees` 操作。源码在正式任务工作树内维护，运行代码、
原生 Git helper、runtime helper 和检查器逐项固定 SHA-256；不从临时报告目录部署执行源码。

先以 `qualify` 执行保护条件回归，再以 `fetch-audit` 获取 origin 并盘点。
`apply` 只处理明确列出的本地 codex ref/OID 和工作树路径/HEAD。
工作树必须有登记的 owner，全部提交已被 origin/main 包含，源码干净，
无未知文件、忽略文件、活动进程引用、Git 操作或受管 reader/writer。
删除前保存逐文件 SHA-256 计划，重新核对来源，并保留 Git 自身的干净检查。
分支使用期望旧 OID 原子比较删除。入口不删除远端分支、不裁剪对象、不清理缓存。

早于 management State 登记的工作树，需在明确选择中绑定原 owner chat 的完整
证据哈希、原分支、HEAD 和原生 `.git` 登记文件，保留归属再执行相同保护检查。
这不会补造 State 登记。回收统计分别记录文件逻辑大小和 Windows 原生 allocation；
存在其他硬链接的文件不计为已释放物理空间。

所有写操作在真实 Jenkins agent 中通过现有原生 Job 与共享 Git 锁执行。
回执位于 `E:/cargo-targets/zircon-local/reports/<taskId>/<operationId>`。
`summary.json` 的自然退出及 EOF 证据与 `receipt.json` 的实际删除结果需一起核对；
未提交源码、不完整 checkout、独有提交和未知生命周期内容继续保留。

`mvp-tooling` 运行 M0.1 的当前入口与存储策略定向 Python 回归；固定套件不调用
Cargo，不能替代 Runtime、Editor 或产品接受门。夹具只在该操作登记的报告目录中
创建，并由 Jenkins 内的测试生命周期回收，失败保留项写入 `test-result.json`。

若原定时集成因临时执行源码丢失而停止，先用 `qualify-integration` 验证当前分支
提交、精确历史回执、队列公平性和既有 PR 发布契约，再执行 `repair-integration`。
修复创建带原计划引用的新运行计划，绑定当前 runtime，使用仓库内 canonical
路径权限检查器和队列恢复模块；旧计划、请求、失败证据和 `H/15` 调度保持可追溯。
同时通过现有公开安装器恢复 canonical 分支提交入口。新计划安装成功仍不等于
实际 PR 集成成功，需核对后续原生结果。
