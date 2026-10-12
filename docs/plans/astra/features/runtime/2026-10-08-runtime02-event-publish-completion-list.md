# Runtime02 事件发布完成列表

优化记录：[复用无淘汰预检锁](../../../optimize/zircon_runtime/02/2026-10-08-event-publish-preflight-lock-reuse.md)。

- [x] 无实际淘汰发布复用预检队列/ledger guards，保留有淘汰时锁外释放对象与重新加锁顺序。
- [x] 保留原子准入、共享 payload、事件/字节预算、关闭及拒绝所有权契约。
- [x] 新公共回归覆盖无淘汰/有淘汰/收到克隆占用/拒绝后重试，原测试与原性能生产者保留。
- [x] 独立核心审查、实际源码接入、限定格式与 diff 检查。
- [ ] 合并异步批次当前源码原生编译、原测试与新回归通过。
- [ ] 原 Runtime02 全规模、延迟、分配、CPU、RSS、并发与产品性能验收。

`runtime-editor-source-repair-batch-20261009-02` 的 12 命令 `batch-native-v2` 已形成真实失败终态：首条 `three-package-check` Cargo 原生编译退出码 `101`（结束 `2026-10-09T04:10:34.317492+00:00`），`sourceUnchanged=true`；24 个迁移后缺失 `.zui` 的 `include_str!` 诊断与 4 个 E0282 级联构成 `zircon_editor` lib test 的 28 个错误，`editor_world_sync_watch_map` integration test 有 12 个原始 source errors，JSON 共 42 条（含 2 条 crate summary）。其余 11 命令均为 `not-invoked-prerequisite-failed`，`nativeTestsRun=false`。证据：`E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/batch-native-v2/terminal-receipt.json`、`actual-compile-errors.json`、`01-three-package-check.log`（log SHA256 `f3c8c06d419a5f7a9a338e49195498d311b80536ccf53a1e84b224d947f26455`）。上一 16 命令批次的 `sourcechanged-refusal` 历史记录保留；下一批次待两个最小 core 修复（Editor assets 迁移引用、`ServiceHandle`/`Result` test 契约）完成审查后再绑定。测试、性能、Jenkins 与里程碑验收保持开放。尚未提交或推送代码。
