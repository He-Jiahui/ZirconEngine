# Runtime02：事件发布复用无淘汰预检锁

对应[原优化计划](../02-core-runtime-events-tasks-review.md)。生产代码与回归已接入；原生测试、原性能与产品验收仍开放。

## 已接入的修复

- 预检后没有实际淘汰事件时，复用已持有的 admission ledger 和订阅者队列锁，省去一轮全部订阅者队列加锁以及一次 ledger 重新获取。
- 有实际淘汰事件时，保留原锁外释放保留对象的顺序，再重新取得队列与 ledger 锁。`RetainedLease::drop` 重新获取 ledger，不能在持有该锁时执行。
- 保留全局 commit 与 topic delivery guards、全部订阅者原子准入、字节/事件保留计费、关闭与拒绝返回语义。
- 新公共回归 `mixed_fanout_preserves_retention_across_empty_and_nonempty_displacement` 覆盖首次无淘汰、实际淘汰、收到对象的克隆继续占用预算、全扇出拒绝不改队列、最后克隆释放后重试以及共享 payload。

独立核心审查通过；补丁 SHA256 `54096fcd8e7b85888d01172e38541551dd36decfd4b6e2999abb97c644e13df7`。实际接入回执 `runtime-next-core-v1/actual-canonical-apply.json` 记录原始与 LF 内容哈希、格式/diff 检查退出码 0 和索引/控制文件未变。独审证据位于 `runtime02-no-displacement-core-review-v1/review.json`；两目录均在 `E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/`。

减少加锁次数是源码因果说明，尚无实测支持延迟、分配、RSS 或吞吐达标。原 EventBus 性能矩阵、诊断开关、保留压力、并发与 teardown 门槛保持完整。

## 异步验证状态

上一 16 命令批次的 `sourcechanged-refusal` 历史记录保留：首条命令在调用 Cargo 前被拒绝，其余命令未调用；该批没有原生测试通过证据。

后继请求 `runtime-editor-source-repair-batch-20261009-02` 的 `batch-native-v2` 已落盘真实失败终态：首条 `three-package-check` Cargo 原生编译以退出码 `101` 失败，结束时间为 `2026-10-09T04:10:34.317492+00:00`；`terminal-receipt.json` 的 `sourceUnchanged=true`。`actual-compile-errors.json` 共 42 条 JSON entries：24 个迁移后缺失 `.zui` 资源的 `include_str!` 诊断与 4 个 E0282 推断级联构成 `zircon_editor` lib test 的 28 个错误，`editor_world_sync_watch_map` integration test 有 12 个原始 source errors，另含 2 条 crate summary，合计 40 条原始 source diagnostics。后续 11 个命令均为 `not-invoked-prerequisite-failed`；`nativeTestsRun=false`，native、性能、里程碑与 Jenkins 验收均保持开放。

证据：`E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/batch-native-v2/terminal-receipt.json`、`actual-compile-errors.json`、`01-three-package-check.log`（log SHA256 `f3c8c06d419a5f7a9a338e49195498d311b80536ccf53a1e84b224d947f26455`）。下一批次待修复再绑定：先交付并审查两个最小 core 修复（Editor assets 迁移引用、`ServiceHandle`/`Result` test 契约），再用保持 `sourceUnchanged` 的回执重新调用原生批次；在此之前不把本失败批次当作原生或性能通过证据。

- [x] 生产优化、新公共回归、独立审查、限定格式/diff 检查及实际接入。
- [ ] 原生编译、全部相关原用例及新回归通过。
- [ ] 原 Runtime02 的性能规模、CPU/分配/RSS、p50/p95/p99、并发及产品门槛达标。
