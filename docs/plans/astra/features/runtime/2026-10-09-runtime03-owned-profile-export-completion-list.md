# Runtime03 拥有型 profile 导出完成列表（2026-10-09）

对应[优化记录](../../../optimize/zircon_runtime/03/2026-10-09-owned-frozen-profile-export.md)与[原计划](../../../optimize/zircon_runtime/03-core-runtime-diagnostics-profiling-config-review.md)。本条仅记录成功导出路径的一次冗余深克隆消除，原 Runtime03 验收开放。

- [x] `export_report()` 的冻结 snapshot 移入成功 report，公开借用导出保留原 clone 点。
- [x] 保留 session owner、写文件/错误顺序、capture 身份和截断合同。
- [x] 新 public 冻结/worker/epoch/ring/stop/reset 回归接入，独立审查 accepted；实际 rustfmt/scoped diff 为 0。
- [ ] profiling 及 profiling-chrome 实际匹配并通过新 case，feature-off 编译保持。
- [ ] 原导出、owner、streaming 回归与原 Release producer 通过。
- [ ] 绑定生产 before/after 真实 workload 的 allocation、时间与 RSS 达标。
- [ ] 全部原 Runtime03 产品、Jenkins 与里程碑验收完成。

## 异步验证状态

现有 batch-v2 首条 Cargo101 的失败收据完整保留。本次随后接入的源码尚未被 native 验证；13 命令 successor 草案尚未绑定或启动。后续统一补入新 public test、chrome 分支及原导出 producer；提交后继续其他功能候选，在实质检查点一次核对终态。

接入收据 `E:/cargo-targets/zircon-local/source-repairs/current-main-20261008/runtime-next-core-v4/actual-canonical-apply.json` SHA256 `118d2c94cab77cd2de1aba02a83a90825370efc6268596fd5b2f958a52ff5bd5`。这份源码证据表明一次冗余 clone 已消除，未报告原生测试或性能数值通过。
