# Runtime08c 动画帧需求完成列表

任务：`optimize-20261013-01a13451`。对应 [08C 原计划](../../../optimize/zircon_runtime/08c-animation-runtime-review.md) P1-2。日期：2026-10-15。

- [x] 在 17 个现有文件内建立 epoch/token 帧需求事务与成功发布、失败清除、过期发布拒绝。
- [x] 沿实际采样、嵌套求值和既有 worker 返回路径传播活动标志，保留原采样/事件/join 语义。
- [x] 保留显式需求和未消费事件的独立 OR，补入 5 个 ordinary 回归用例源码；原 ordinary 与 ignored producer 身份保留。
- [x] 冻结原始 before/after，独立完成增量正向/反向及 HEAD/index 证明；保留预先存在的 UNKNOWN 修改与历史失败证据。
- [x] 独立源码复核：gpt-6-luna max，PASS_static_only，未发现可操作 P1/P2；最终只读会话 32305 实际 exit 0 并已关闭，系统 PID 未暴露（UNKNOWN）。。证据为静态结果。
- [ ] 受支持 Jenkins job 对最终实际路径并集执行 scoped guard，修复所有错误并登记警告。
- [ ] 合批原生编译、真实 cfg/name 库存、普通及 Release ignored 测试通过。
- [ ] 实际完整编译输入与环境 seal、原始 producer 样本、Runtime03 绝对 P95 与原 helper 门槛、Runtime44 内存 AND 条件均通过。
- [ ] App/Editor reactive cadence、30/60/120Hz、1k 角色阶段 P50/P95/P99 与一小时 soak 通过。
- [ ] 原计划 P1-2 完整产品目标及 M1/全 08C 验收关闭。

详细优化记录：[2026-10-15-animation-frame-demand-record.md](../../../optimize/zircon_runtime/08c/2026-10-15-animation-frame-demand-record.md)。源码绑定：[runtime08c-animation-frame-demand-v1-source-binding.json](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime08c-animation-frame-demand-v1-source-binding.json)。静态证明：[runtime08c-animation-frame-demand-v1-static.json](../../../../../.codex/outbox/reports/optimize-20261013-01a13451/runtime08c-animation-frame-demand-v1-static.json)。源库存 73 ordinary、plugin 92 ordinary/24 ignored 和 physics_tick 42 仅供后续实际发现核对，不表示测试已运行。

实际释放 0 B；保留本任务的绑定、补丁、静态和失败回执。原生与性能验收 OPEN。
