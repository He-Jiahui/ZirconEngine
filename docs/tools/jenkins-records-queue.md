# Jenkins records 收件与待办队列

`tools/jenkins-management/records_queue.py` 是固定本机 Jenkins 的交接入口。
它复用已安装管理入口的完整能力、driver、runtime 和健康校验，不替换管理
capsule、不修改旧协调器，也不执行任意交接命令。使用前执行 `install`，其
固定 job 为 `zircon-records`；已有 job 的脚本不一致时拒绝覆盖。

会话按用户每 30 分钟检查、所有任务入队后清空收件目录的授权，准备普通文件的
绝对路径、大小、mtime 和 SHA256 清单，关联 Markdown 合同、JSON 合同、回应
及审查证据。`submit --inventory-file <清单>` 在 HTTP 前保存一次性提交身份；
未知交付只核对原身份，不再次发送。历史回应作为 evidence，不新增验证任务。

Jenkins worker 在 State 的 `records_queue_task` 保留原任务/session、消息哈希、
原始要求、实际 queue/build URL 和未满足门槛。原始消息字节保存在
`.jenkins/state/records-queue/archive/<inventoryDigest>/`；`records_queue_message`
记录原路径到保留位置的映射。只有归档哈希吻合且 State 已持久受理后，固定
清理入口才通过已验证的 `GuardedFileDelete.cs` 删除同一路径、大小、mtime 和
SHA256 的文件。变化文件、别名、硬链接、缺失证据及未知交付都不能清理。

此授权只改变收件目录的保留位置，不改变原任务的执行和验收门槛。待办在
Jenkins pipeline 中等待，不占用 executor；每 30 分钟沿用原 request/attempt
对账。可表示的 scoped validation 由正式 ordinary entry 执行；不支持的参数、
source-freeze-pending、正式覆盖及提交/通知授权缺口保持开放。入队、收件清理
和原任务验收分别记录，接收入口不能给出 Native 或产品验收成功。

本入口的独立工作只包括交接生命周期。它不授权提交、推送、发布、外部通知、
源码副本或新的 compiler target。后续正式请求仍须带原身份、当前源码哈希、
完整 compiler seal、相应命令/features/覆盖及合法盘根输出。

同一原任务的补充消息使用 `records_followup.py` 的固定
`zircon-records-followup` job；它只附加 metadata，不提交新的 Native 请求。
`install` 不修改正在等待的 `zircon-records` job 或其固定源码。`submit` 要求
原任务、原输入 digest 和原身份完全吻合；未知交付不重复发送。Jenkins 在原
任务的同一排他锁下将归档、回应位置/hash 和补充待办写入 State，再通过精确
路径/hash 清理收件文件。原 `records_queue_task.supplements` 和
`records_queue_followup` 可检索回应；此 job 的 SUCCESS 仅表示补充受理与清理，
不能表示生成、源码审查、验证或提交已经完成。
