# Runtime03 profiling 父路径完成列表

优化记录：[借用父路径并合并遍历](../../../optimize/zircon_runtime/03/2026-10-08-borrowed-profiling-parent-path.md)。

- [x] 借用当前 epoch 父路径，移除临时整路径复制并合并父节点/深度遍历。
- [x] 保留 TLS/recorder/frame 借用边界、跨 stream 语义、饱和深度与非 LIFO 行为。
- [x] 保留原 11 个 case 与原非嵌套性能生产者；新增旧 epoch/路径/深度/释放顺序公共回归。
- [x] 独立审查、源码实际接入、限定格式及 diff 检查。
- [ ] 合并批次原生编译、profiling 启用/未启用的相关原测试与新回归通过。
- [ ] 同一真实嵌套 workload 在绑定的前后生产源码上证明分配与时间收益。
- [ ] 原 Runtime03 全规模与产品指标达标。

12 命令请求 `runtime-editor-source-repair-batch-20261009-02` 的 `batch-native-v2` 已形成真实失败终态：首条 `three-package-check` Cargo 原生编译退出码 `101`（结束 `2026-10-09T04:10:34.317492+00:00`），`sourceUnchanged=true`；24 个迁移后缺失 `.zui` 的 `include_str!` 诊断与 4 个 E0282 级联构成 `zircon_editor` lib test 的 28 个错误，`editor_world_sync_watch_map` integration test 有 12 个原始 source errors，JSON 共 42 条（含 2 条 crate summary）。其余 11 命令均为 `not-invoked-prerequisite-failed`，`nativeTestsRun=false`。证据：`E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/batch-native-v2/terminal-receipt.json`、`actual-compile-errors.json`、`01-three-package-check.log`（log SHA256 `f3c8c06d419a5f7a9a338e49195498d311b80536ccf53a1e84b224d947f26455`）。上一 16 命令批次的 `sourcechanged-refusal` 历史记录保留；下一批次待两个最小 core 修复（Editor assets 迁移引用、`ServiceHandle`/`Result` test 契约）完成审查后再绑定。非嵌套基准不能关闭嵌套收益门槛；修正后的嵌套证据补丁已接入、native 待执行，详见下方记录。Jenkins、里程碑及原性能验收仍开放。

## 嵌套 probe 实际接入（2026-10-09）

`runtime03_nested_scope_allocation_profile.rs` 已于 `2026-10-09T06:18:10.778448+00:00` 接入。旧 probe 的进程全局计量窗口存在跨线程晚到写入 P2，原失败审查和补丁完整保留；修正版使用 const TLS `Cell<bool>` 与 `try_with` 回调，只归因 workload thread 的真实成功 alloc/alloc_zeroed/realloc 请求，AtomicU64 原计数、101 个实际原始样本与 current/stale/non-LIFO 矩阵保持。它没有手写分配常数，也没有替代原 producer。

独立源码审查 accepted（`runtime03-tls-core-review-v1/review.json` SHA256 `56cb22455fc42f60c68e2663ebb209a7954cb7e81b885621f68e6da88d5f574c`）；实际 rustfmt 与 scoped diff check 为 0。接入收据 `runtime-next-core-v3-tls-v2/actual-canonical-apply.json` SHA256 `92e918b61a1256e474771b5adbec7a1e096370fa5a2d79877773554eeee79bba`，位于 `E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/`。新测试 LF hash `79903f47fc6c0c92c7cc63106366b3d22cc0e24d685036613248c658f90fbb1a`、raw hash `615ce6e66dbb04cebd8ec16823351bfd4095fa17e1a623247a6dcf92a8e25da9`。

原非嵌套 1/8/64-worker producer `runtime_profiling_recorder_performance.rs` SHA256 `f310558e8ca19162298fdbe45b2887eb2bca8980272af07be44f235f4439e80d` 完整保留。真实嵌套收益须同一修正版 probe 在生产 scope 的 LF before hash `6e370fdcee114cfd67382f29c63424cc2ac4b444e131324bf1baf01cf2205e85` 与 after hash `ed29294beddc5ee411fd89392fde3a376c377836035bd0dcada1a70211285a71` 上量测；after-only 的分配或时间结果无法关闭前后收益。新 native、配对量测和全部原性能/产品验收仍开放，下一批尚未启动。
