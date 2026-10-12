# Runtime02 预检计数栈存储完成列表

状态：源码阶段完成；原生功能与性能待完成。

- [x] 在既有 `EventBusState::admit` 内使用 8 项 Copy 计数栈数组，较大扇出保留完整逻辑长度的堆路径。
- [x] 保留所有 global/topic/subscriber 容量数学、拒绝、mixed-lossy、锁、提交、退役释放和通知逻辑。
- [x] 独立核心源审通过；实际 1 文件应用、35 fresh guards、34 未修改依赖、Rust 格式与 scoped diff 检查通过。
- [x] 控制配置、Cargo、共享索引及既有索引锁内容和元数据保持原样；未新增测试或 dependency。
- [x] 写入 [优化记录](../../../optimize/zircon_runtime/02/2026-10-09-inline-removal-preflight-record.md) 和 [实际采用证据](../../../../../.codex/outbox/runtime02-inline-removal-preflight-v1/actual-canonical-apply.json)。
- [ ] 当前源码通过批量包级编译及既有 Runtime EventBus 功能用例，真实匹配非零。
- [ ] 原 Release fanout/payload/batch/diagnostics/10K/100K/1M 全矩阵通过，验证 actual allocation、CPU、延迟、retained bytes 与 RSS 原预算。
- [ ] 已激活协调器提供 immutable baseline 的增量补丁入口，批量异步 request ID 与终态 receipt 可归属当前源码。
- [ ] 功能、产品和性能完成标准均满足；不得用源码检查替代验收。
