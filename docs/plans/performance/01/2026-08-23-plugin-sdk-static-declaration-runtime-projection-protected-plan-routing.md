---
source_report:
  - docs/plans/performance/01/2026-08-23-plugin-sdk-static-declaration-runtime-projection-current-architecture-review.md
doc_type: protected-plan-routing
status: routing_pending
---

# Plugin SDK静态declaration与runtime projection受保护计划路由（2026-08-23）

- `docs/plans/performance/pending.md`：SDK declaration/runtime projection current 5/5完成，SDK累计8/21；current Cargo、
  constructor/allocator counter、F0/F4 WPR/RSS/power仍open。本Session不直接编辑受保护ledger。
- PERF-MVP-629 + Plan02 M1/M5 + Plugins01/12：保留`PluginDeclaration`和native const bytes，建立唯一
  `PluginProviderSchemaGeneration`；schema validation/projection<=1/provider/process。
- Plugins12 `runtime_plugin_exports!`：metadata manifest/selection façade直接消费schema generation，不调用runtime plugin
  constructor；plugin-specific manifest extension改为schema contribution/factory，不留旧constructor façade兼容层。
- Runtime module/service owner：module descriptor schema不持session manager Arc；manager/module factory只进入selected
  `RuntimePluginInstanceGeneration`，AI/Physics metadata-only manager constructor=0。
- Plugins10/Editor12：`mirrors_runtime_manifest`借用runtime provider schema，不为editor declaration创建runtime manager；mutable
  editor consumer仍按editor session实例化。
- `docs/plans/performance/review.md`：只有SDK 21/21、current Cargo、28-provider metadata/instance counter、AI/Physics
  metadata-only constructor=0、reload隔离及F0/F4 WPR/RSS/power通过后迁入。本轮不迁移、不提交milestone、不发送完成企微。
