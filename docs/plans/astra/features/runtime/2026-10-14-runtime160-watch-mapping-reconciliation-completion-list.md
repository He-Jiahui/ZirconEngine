# Runtime160：监听映射不确定性进入原批次重扫：计划完成列表

对应 [原计划](../../../optimize/zircon_runtime/160-runtime-filesystem-path-uri-vfs-mount-watch-sandbox-atomic-io-current-source-review.md)；实现与证据见 [优化记录](../../../optimize/zircon_runtime/2026-10-14-watch-mapping-reconciliation-record.md)。

- [x] 私有映射结果显式携带 reconciliation，并区分已知忽略与相关映射不确定性。
- [x] 沿原 loop batch 时钟/flush/reset/stop/disconnect 发出重扫 token，原下游 owner 保留。
- [x] 5 个新增 ordinary、14个既有 ordinary、1个原 ignored已冻结；独立 source/packet peer 与 scoped guard 实际通过。
- [ ] 合批原生 ordinary、原ignored producers及全部原性能/resource/Runtime03验收。
- [ ] 跨平台路径、rename/replay与完整 watcher storm/FS-G11 qualification。
- [ ] 完整原 Runtime160 计划及 BUG 总体验收；本次 ingress-loop-callback 不是全下游 E2E。

仅标记本次源码/静态/独立复核完成；native、性能和产品验收分开保留。实际释放0B，保留必要交付/失败证据。
