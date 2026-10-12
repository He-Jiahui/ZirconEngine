# Editor63 pending history 完成列表

优化记录：[有序合并 pending history](../../../optimize/zircon_editor/63/2026-10-08-ordered-pending-history-merge.md)。

- [x] 最多两个 pending ID 有序插入，跳过重复 ID，省去边界第二次排序/去重。
- [x] 保留 dirty/generation、cursor、Reset/Delta、volatile、取消/切换和 journal visits。
- [x] 原 7 个公共用例与原 Release producer/标记/预算保留；新增真实 pending transition 公共回归。
- [x] 独立核心审查、实际接入、限定格式与 diff 检查。
- [ ] 合并异步批次原生编译、相关原测试和新回归通过。
- [ ] 原 `EDITOR63_DIRTY_JOURNAL_HASH_DEDUP_BENCH_V1` 的 P95 ≤ 60% 门槛实际通过。
- [ ] 原 Editor63 1/10/100 文档、100K 命令、适用的 1GB history、恢复与产品性能验收。

12 命令请求 `runtime-editor-source-repair-batch-20261009-02` 的 `batch-native-v2` 已形成真实失败终态：首条 `three-package-check` Cargo 原生编译退出码 `101`（结束 `2026-10-09T04:10:34.317492+00:00`），`sourceUnchanged=true`；24 个迁移后缺失 `.zui` 的 `include_str!` 诊断与 4 个 E0282 级联构成 `zircon_editor` lib test 的 28 个错误，`editor_world_sync_watch_map` integration test 有 12 个原始 source errors，JSON 共 42 条（含 2 条 crate summary）。其余 11 命令均为 `not-invoked-prerequisite-failed`，`nativeTestsRun=false`。证据：`E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/batch-native-v2/terminal-receipt.json`、`actual-compile-errors.json`、`01-three-package-check.log`（log SHA256 `f3c8c06d419a5f7a9a338e49195498d311b80536ccf53a1e84b224d947f26455`）。上一 16 命令批次的 `sourcechanged-refusal` 历史记录保留；下一批次待两个最小 core 修复（Editor assets 迁移引用、`ServiceHandle`/`Result` test 契约）完成审查后再绑定。原测试、性能、Jenkins 与里程碑验收均未据此关闭。
