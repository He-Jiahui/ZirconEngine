# Editor09：Runtime / Settings 关闭查询源码完成列表

2026-10-17。对应优化记录：docs/plans/optimize/zircon_editor/09/2026-10-17-runtime-settings-shutdown-poll-source-record.md。

- [x] Settings 关闭凭证增加可重复借用的结果查询，Pending / 失败时仍由调用方持有。
- [x] 区分真实工作线程失败与 fence 失败，只有两者均成功才返回成功。
- [x] 导出类型化失败接口，保留旧 finish 实现。
- [x] 写入两项真实 fence 回归测试源码，并保留现有持久化用例。

- [ ] 新旧测试原生验证。
- [ ] 主窗口接入并保留 Session / 关闭凭证直至真实终结。
- [ ] 完整正常关闭、失败恢复、GUI 和性能验收。

源码已实际落地并完成限定范围审查；这些勾选仅代表源码实现，测试尚未运行。详见报告目录 editor09-runtime-settings-shutdown-poll-v1-source-write-receipt.json。编译验证并入下一批 Jenkins 请求；本记录不构成队列接收或验收通过。


补充：Runtime 三项测试接入真实句柄时，增加终态通知到关闭条件变量；断言展开释放测试门闩后，关闭凭证仍能获知真实任务终结。该辅助逻辑仅通知、不回收句柄，保留显式 wait 的 panic 回归条件。源码修订证据：.jenkins/records/runtime-editor-optimize-01a13451-20261017-editor09-shutdown-poll-test-wakeup-correction.json。测试仍未运行，并入同一批 Jenkins 验证。

终态通知补充已按独立复审改为持有 lane 锁后唤醒条件变量，避免检查条件与进入等待之间丢失通知；仍不回收测试句柄。对应 mutex-correction 交接记录保留前次修订证据，原生验证仍待合批。
