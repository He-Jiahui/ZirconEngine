# Editor01：BlendSpace 调用者完成列表

状态：源码阶段完成；原生功能、产品与性能待完成。

- [x] 通过实际 Window / Ability / typed selection 加载物理 BlendSpace 工作区，使用真实父 viewport、Source、frame 和 host paint/hit。
- [x] 保留原 geometry、Grid / timeline / heatmap / action 断言及实际像素 4/8/4 阈值。
- [x] 独立源审、实际三文件采用、98 双次输入核对、95 非修改依赖、三文件实际格式与 scoped diff 检查通过。
- [x] 保留共享索引、旧索引锁内容及元数据；没有生产文件、实体资产或 tooling 修改。
- [x] 原 28 普通用例在当前 V8 无重叠；同批次 V9 #15 加精确名称，最低 146→174，新增测试与 arm 均为零。
- [x] 原 1 ignored capture 名称、body、ignore 状态保持，不激活该截图作为产品验收。
- [x] 写入[优化记录](../../../optimize/zircon_editor/01/2026-10-09-blend-space-real-parent-caller-record.md)和[实际采用回执](../../../../../.codex/outbox/editor-blend-space-real-parent-callers-v1/after-core-actual-v3/actual-canonical-apply.json)。
- [ ] 当前源码批量编译及 28 精确普通用例实际通过，确认非零匹配。
- [ ] 真实 Editor 产品显示、paint/hit/input、生命周期与原完整性能预算达标。
- [ ] Generated / paint-hit fixture / strong gate 及三个旧聚合消费者迁移与实体退役完成。
- [ ] 已激活 Jenkins 协调器提供归属当前源码的异步请求与最终回执。

源码修复与格式检查不能替代测试、性能或产品验收。
