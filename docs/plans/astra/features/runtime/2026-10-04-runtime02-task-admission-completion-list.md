# Runtime02 任务准入修复完成列表

日期：2026-10-04。所属优化计划：Runtime02；本轮状态为 `source_applied_native_validation_pending`。

- [x] 应用 18 个 Runtime 源文件的修复。指定 scope 提交保留传入调度器的真实执行池、生命周期、完成身份和回调派发器；目标 scope 负责自己的准入与取消。
- [x] 场景加载准入失败返回原始 `TaskGraphAdmissionError`，在失败时保留 pending 容器、队列和字节计数；失败原因进入本帧报告。
- [x] 场景读写、VM 包发现使用绑定原任务图且诊断计数独立的调度器。
- [x] 保留原测试；补充执行池归属、关闭原因优先级、外国 scope 关闭取消和加载拒绝等回归。候选格式检查及非作者源码审查通过，18 个文件应用前后哈希匹配，控制配置与 Git 索引未变更。
- [ ] 运行合并的类型检查和原行为回归，修复实际失败。
- [ ] 完成原 count／bytes／join、CPU、RSS、I/O、allocation、P50／P95／P99 与 DLL／取消排空验收；尚无本轮性能达标结论。

证据：[优化记录](../../../optimize/zircon_runtime/02/2026-10-04-task-scheduler-owner-and-loader-refusal.md)。历史完成列表保留在 `_archive`；历史私有候选和 pending 验证不作为本轮通过证据。
