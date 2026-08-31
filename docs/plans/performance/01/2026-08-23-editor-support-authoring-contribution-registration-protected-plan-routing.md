---
source_report:
  - docs/plans/performance/01/2026-08-23-editor-support-authoring-contribution-registration-current-architecture-review.md
doc_type: protected-plan-routing
status: routing_pending
---

# Editor support authoring contribution受保护计划路由（2026-08-23）

- `docs/plans/performance/pending.md`：新增`zircon_plugins/editor_support/src/**` current 1/1文件、334物理行、1 test静态覆盖；
  current Cargo、startup/reload counter、allocator、F4 WPR/RSS/power仍open。本Session不直接编辑受保护ledger。
- PERF-MVP-629 + Plan02 M1/M5 + Plugins01/10/12 + Editor06/12：把authoring descriptor/factory纳入process级immutable
  `EditorProviderSchemaGeneration`，把mutable mode/consumer/mirror纳入per-editor-session `EditorPluginInstanceGeneration`；
  stable projection不得重跑extension registration。
- Editor12 + Plugins10：在generation边界把`EditorExtensionRegistry -> ContributionBatch`收敛为单次
  validate/materialize transaction与owner family ranges；失败publish=0，不缓存完整mutable report，不留双registry compat
  facade。
- Editor06：审计`views/drawers/menu_items/... -> Vec<&T>`及pane-source map复制的真实调用频率；仅在counter确认后改为borrowed
  iterator/range或generation-owned snapshot，不在注册helper局部猜测热度。
- Editor14：本注册路径不应进入TaskGraph；只有plugin discovery/reload中I/O或独立provider build可有界并行，validation后的
  generation publish保持单事务。主线程filesystem/DLL/等待=0由PERF-MVP-631另行验收。
- `docs/plans/performance/review.md`：只有current Cargo、provider/contribution/session矩阵counter、F4 startup/reload WPR、
  allocator/RSS/power、unload隔离通过后迁入；该非渲染切片不要求RenderDoc。本轮不迁移、不提交milestone、不发送完成企微。
