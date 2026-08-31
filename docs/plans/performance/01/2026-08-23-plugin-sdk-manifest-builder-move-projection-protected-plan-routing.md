---
source_report:
  - docs/plans/performance/01/2026-08-23-plugin-sdk-manifest-builder-move-projection-current-architecture-review.md
doc_type: protected-plan-routing
status: routing_pending
---

# Plugin SDK manifest builder move projection受保护计划路由（2026-08-23）

- `docs/plans/performance/pending.md`：SDK manifest current 7/7完成，SDK累计15/21；importer foreign M0的current Cargo、
  parity test、managed allocator benchmark及F0/F4 trace仍open。本Session不直接编辑受保护ledger。
- PERF-MVP-629 + Plan02 M1/M5 + Plugins01/12：manifest builder保留by-value局部形态，最终manifest只在
  `PluginProviderSchemaGeneration`构建一次；stable metadata query build=0。
- Plugins12/13 importer M0：把ignored test迁入受管release benchmark receipt；用真实clone/allocator counter替代打印常量与
  单一20%机器wall阈值。要求builder-field clones=1、capability clones=0、manifest parity=100%。
- Native ABI边界：distribution/importer继续使用owned versioned DTO，不以borrowed Rust地址或process-global mutable report
  规避String成本。
- `docs/plans/performance/review.md`：只有SDK 21/21、current Cargo、importer C/I矩阵counter+allocator receipt、28-provider
  schema build门及F0/F4 WPR/RSS/power通过后迁入。本轮不迁移、不提交milestone、不发送完成企微。
