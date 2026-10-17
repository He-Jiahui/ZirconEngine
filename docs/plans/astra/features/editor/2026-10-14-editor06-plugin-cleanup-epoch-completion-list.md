# Editor06：插件清理进度完成列表

状态：`source_reviewed / native_performance_pending`；详见[优化记录](../../../optimize/zircon_editor/06/2026-10-14-plugin-cleanup-epoch-record.md)。

- [x] CR-EDITOR-SERVICES-0002：按当前实例/激活轮次判断退役进度，历史阶段不再替代本轮清理。
- [x] 保留真实失败阶段重试；普通 Unloaded 通知不会写入或抹除退役 authority。
- [x] same Arc 延续生命周期，candidate metadata 保留；fresh native 重置、旧宿主中断重试保留。
- [x] 新增真实回归，保留既有断言；源码/packet/格式/diff/root guard 与独立静态审查完成。
- [ ] 合批原生回执、全部 required names/计数与 full-input seal 通过。
- [ ] 适用原性能/资源门槛通过，随后关闭历史 BUG 和整体计划验收。

原 Editor06 其他 P0/P1 项保持开放。本条不代表性能已达标；没有本地构建/缓存/夹具或源码副本，实际释放0B。
