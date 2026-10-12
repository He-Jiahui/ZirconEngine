# Runtime03：借用 profiling 父路径并合并遍历

对应[原优化计划](../03-core-runtime-diagnostics-profiling-config-review.md)。生产代码已接入；原生测试及全部原性能验收仍开放。

## 已接入的修复

- 在当前线程 span stack 的不可变借用内找到当前 epoch 最后一个父节点，并同步累计饱和深度。原两次扫描合并为一次遍历；达到 `u16::MAX` 后继续更新父节点。
- 格式化最终 path 时借用父路径，移除完整父路径的临时 clone；最终 token/TLS 所需的拥有型路径继续保留。根 scope 没有这项父路径复制节省。
- 不可变 TLS 借用在 recorder、frame stack 和可变 stack 操作之前结束；保留跨 stream 祖先、当前 epoch、frame identity、非 LIFO 释放及原路径文本语义。
- 原 11 个 case 函数体完整保留，新增公共回归 `nested_scope_paths_keep_current_epoch_depth_after_out_of_order_drop`，覆盖旧 epoch 留在 TLS、Unicode 与斜杠/冒号路径、跨 stream 和非顺序释放。

独立核心审查通过；补丁 SHA256 `ede9cc6cb41ea42bb9c590a76becb4cf8c4676f33819843d9d49ba8cafc277c3`。实际接入与格式/diff 检查证据 `runtime-next-core-v2/actual-canonical-apply.json`；独审 `runtime03-core-review-v1/review.json`，位于 `E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/`。

原 `runtime_profiling_recorder_performance::active_profile_recorder_reports_multithread_scaling_and_allocation_cost` 用非嵌套 scope 测试 1/8/64 worker，保持完整并纳入验证；它不能证明本次嵌套父路径分配收益。新的嵌套真实 allocator 证据补充已通过源码审查并接入，尚未执行；详见下方接入记录。实际收益需要同一 workload 在正确绑定的原生产与优化生产源码上量测，不能用手写分配常数或替代基线证明。

## 异步验证状态

上一 16 命令批次的 `sourcechanged-refusal` 历史记录保留：首条命令在调用 Cargo 前被拒绝，其余命令未调用；该批没有原生测试通过证据。

后继请求 `runtime-editor-source-repair-batch-20261009-02` 的 `batch-native-v2` 已落盘真实失败终态：首条 `three-package-check` Cargo 原生编译以退出码 `101` 失败，结束时间为 `2026-10-09T04:10:34.317492+00:00`；`terminal-receipt.json` 的 `sourceUnchanged=true`。`actual-compile-errors.json` 共 42 条 JSON entries：24 个迁移后缺失 `.zui` 资源的 `include_str!` 诊断与 4 个 E0282 推断级联构成 `zircon_editor` lib test 的 28 个错误，`editor_world_sync_watch_map` integration test 有 12 个原始 source errors，另含 2 条 crate summary，合计 40 条原始 source diagnostics。后续 11 个命令均为 `not-invoked-prerequisite-failed`；`nativeTestsRun=false`，native、性能、里程碑与 Jenkins 验收均保持开放。

证据：`E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/batch-native-v2/terminal-receipt.json`、`actual-compile-errors.json`、`01-three-package-check.log`（log SHA256 `f3c8c06d419a5f7a9a338e49195498d311b80536ccf53a1e84b224d947f26455`）。下一批次待修复再绑定：先交付并审查两个最小 core 修复（Editor assets 迁移引用、`ServiceHandle`/`Result` test 契约），再用保持 `sourceUnchanged` 的回执重新调用原生批次；在此之前不把本失败批次当作原生或性能通过证据。

- [x] 原生产语义、原 case 与性能生产者保留，源码优化及新回归实际接入。
- [x] 独立审查、LF 内容哈希、限定格式与 diff 检查。
- [ ] profiling 启用与未启用配置下原生编译和相关原测试/新回归通过。
- [ ] 真实嵌套工作负载分配与时间的前后量测。
- [ ] 原 Runtime03 的全部 worker、series、history、consumer、CPU/RSS/锁等待和产品帧扰动门槛达标。

## 嵌套 probe 实际接入（2026-10-09）

`runtime03_nested_scope_allocation_profile.rs` 已于 `2026-10-09T06:18:10.778448+00:00` 接入。旧 probe 的进程全局计量窗口存在跨线程晚到写入 P2，原失败审查和补丁完整保留；修正版使用 const TLS `Cell<bool>` 与 `try_with` 回调，只归因 workload thread 的真实成功 alloc/alloc_zeroed/realloc 请求，AtomicU64 原计数、101 个实际原始样本与 current/stale/non-LIFO 矩阵保持。它没有手写分配常数，也没有替代原 producer。

独立源码审查 accepted（`runtime03-tls-core-review-v1/review.json` SHA256 `56cb22455fc42f60c68e2663ebb209a7954cb7e81b885621f68e6da88d5f574c`）；实际 rustfmt 与 scoped diff check 为 0。接入收据 `runtime-next-core-v3-tls-v2/actual-canonical-apply.json` SHA256 `92e918b61a1256e474771b5adbec7a1e096370fa5a2d79877773554eeee79bba`，位于 `E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/`。新测试 LF hash `79903f47fc6c0c92c7cc63106366b3d22cc0e24d685036613248c658f90fbb1a`、raw hash `615ce6e66dbb04cebd8ec16823351bfd4095fa17e1a623247a6dcf92a8e25da9`。

原非嵌套 1/8/64-worker producer `runtime_profiling_recorder_performance.rs` SHA256 `f310558e8ca19162298fdbe45b2887eb2bca8980272af07be44f235f4439e80d` 完整保留。真实嵌套收益须同一修正版 probe 在生产 scope 的 LF before hash `6e370fdcee114cfd67382f29c63424cc2ac4b444e131324bf1baf01cf2205e85` 与 after hash `ed29294beddc5ee411fd89392fde3a376c377836035bd0dcada1a70211285a71` 上量测；after-only 的分配或时间结果无法关闭前后收益。新 native、配对量测和全部原性能/产品验收仍开放，下一批尚未启动。
