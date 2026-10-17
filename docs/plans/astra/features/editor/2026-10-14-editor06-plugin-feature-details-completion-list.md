# Editor06：当前插件全部 feature 详情、限定操作与稳定焦点：计划完成列表

对应 [原计划](../../../optimize/zircon_editor/06-plugin-manager-discovery-enablement-live-reload-settings-diagnostics-review.md)；实现与证据见 [优化记录](../../../optimize/zircon_editor/06/2026-10-14-plugin-feature-details-record.md)。

- [x] 当前 owner 全 feature 分页、稳定身份/焦点和 Back 已实现；Back 后报告移除旧选择时会恢复合法选择或清空。
- [x] qualified 单 feature 操作、stale 撤销、effective Unknown、详情 cache 和双模型投影已实现。
- [x] 11 个新增 ordinary 与20个相邻既有 ordinary 已冻结，独立 source/packet peer 和 scoped guard 实际通过。
- [ ] 合批原生回归、Release/ignored producers 与原全部性能门槛。
- [ ] 真实双语三种宽度的 selected-plugin 输入/截图/滚动/焦点/manifest 矩阵与详情性能。
- [ ] 完整 Editor06 P1-11 及关联原计划总体验收；此列表不关闭 BUG 或全功能。

仅标记本次源码/静态/独立复核完成；native、性能和产品验收分开保留。实际释放0B，保留必要交付/失败证据。


### 2026-10-15 source-only review

- Read-only source check: host_actions.rs:25–46 routes Unload and Hot Reload to the live-host path before manifest load/save. This is existing behavior, not a code change in this review.
- host_actions.rs:47–54 still sends SetEnabled and configuration actions through manifest load, project action, then save; E-PLUGIN-UX-P0-01 remains open.
- The prepare, durable commit, runtime apply, and typed forward-recovery flow remains a design only. Expected-revision checks cover participating writers under the same owner; external-writer CAS remains OPEN.
- CommitRecoveryDeferred stops apply; CleanupDeferred records cleanup still pending. These are design semantics, not implemented behavior in this review. Native/UI/P95/P1-11 and the original acceptance gates remain unchecked.
