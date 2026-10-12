# Editor01：Core / Particle 实际 Source 验证入口优化记录

日期：2026-10-09。状态：两文件源码修复已采用；实际测试、产品呈现与性能待验证。

## 已采用的修复

将既有 Core 与 Particle 回归测试中的完整实际驱动提取为 BuiltinTemplateBridge 的直接 cfg(test) 验证入口。调用方持有环境锁；原四个测试名称、正负断言和逐页 Scene 退休检查保留，不增加测试或生产 ABI。每组入口返回实际执行取得的 SourceReceipt，按真实选页、物理源及主题的完整有序五字段结果核对。

既有模块 mounter 的测试 getter 在实际 owner 不存在时返回空结果，在存在时返回实际加载记录。这修复 Scene 阶段对不存在 owner 的错误 expect；生产 getter、工作区发布及生产流程保持原行为。

## 源码验证证据

独立评审及实际应用回执：[当前评审](../../../../../.codex/outbox/editor-real-component-source-witness-core-review-v2/current-getter-rebind-v1/review.json)、[实际采用](../../../../../.codex/outbox/editor-real-component-source-witness-proposal-v2/actual-canonical-apply.json)。

- 实际应用两文件；116 个输入两次核对，114 个非修改输入保持。
- 两文件实际 Rust 2021 格式检查及 scoped diff 检查通过。
- 保留共享索引、既有零字节索引锁、Cargo 和控制配置的内容及元数据。
- 两个外来只读测试依赖使用独立审查的当前输入绑定；历史补丁、评审和拒绝记录保留。overflow 依赖的完整历史差异没有重建，不声明历史全字节等价。
- 实际采用回执 SHA-256：38904cc57a595bc362bb22cbd2100cb7ac49b0a0305fa4760bc243ac8f96f498。

## 待验证

沿用同一批次的现有四个精确测试名称，不增加 Cargo arm 或提高测试计数。当前 V8 共 15 arms，Editor #15 保守最低 146；入口提取本身不计新用例。后续 strong gate 的调用接入仍需自己的当前输入审查及实际采用。

完整 205 组件 offender-empty、三个旧聚合资产的消费者迁移与实体退役、真实产品 paint/hit/input 呈现，以及原 allocation、CPU、RSS、I/O 和延迟预算均待验证。没有提交协调器、运行 native 或性能达标声明。

对应[计划完成列表](../../../astra/features/editor/2026-10-09-editor01-core-particle-source-completion-list.md)。
