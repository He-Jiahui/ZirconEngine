# Runtime / Settings 关闭查询修复记录

2026-10-17，Editor09 关闭链的底层源码小批，涉及 7 个现有文件。

- [x] 增加借用的关闭结果查询：真实任务尚未结束时返回 Pending，调用方保留关闭凭证。
- [x] 两处任务句柄回收统一保留 Failed / Cancelled；Failed 优先，后续成功不清除历史失败。
- [x] Settings 成功条件同时要求 Runtime 工作线程成功和关闭 fence 成功。
- [x] 关闭凭证析构仍等待真实静止，避免再次抛出任务 panic；原显式 wait 和 finish 实现保留。
- [x] 增加 5 项真实任务 / fence 回归测试源码。

已写入源码并双次读取核对，独立审查通过本批修改。候选、源码写入回执和限定范围审查位于报告目录的 editor09-runtime-settings-shutdown-poll-v1-* 文件。源码写入进程：PowerShell 16288 / Python 39396，退出 0；回执 SHA-256：703b914c9b3468b81f38534c0e6e940bae568db13aa759a45cc6654cccdefb1a。

待验收：

- [ ] 新增测试和相关旧测试的原生运行。
- [ ] 全部改动的公共变更检查、完整 MainWindow 关闭流程和 GUI 验收。
- [ ] 原有性能门槛及 Runtime03 绝对 P95 矩阵。

本批不会放宽静止条件。真实泵崩溃后仍有未释放条目时，恢复策略继续待完成；运行中的关闭凭证析构仍可能等待，正常 UI 需保留其所有权。Runtime 三项用例把真实调度任务句柄加入测试状态并调用生产回收逻辑，不能作为完整泵崩溃恢复的端到端证据。性能尚未实测。

Jenkins 接收本批作为下一次合批验证的追加项，不单独逐项构建。未启动编译，未创建源码副本或编译产物，实际释放空间 0 字节。


补充：Runtime 三项测试接入真实句柄时，增加终态通知到关闭条件变量；断言展开释放测试门闩后，关闭凭证仍能获知真实任务终结。该辅助逻辑仅通知、不回收句柄，保留显式 wait 的 panic 回归条件。源码修订证据：.jenkins/records/runtime-editor-optimize-01a13451-20261017-editor09-shutdown-poll-test-wakeup-correction.json。测试仍未运行，并入同一批 Jenkins 验证。

终态通知补充已按独立复审改为持有 lane 锁后唤醒条件变量，避免检查条件与进入等待之间丢失通知；仍不回收测试句柄。对应 mutex-correction 交接记录保留前次修订证据，原生验证仍待合批。
