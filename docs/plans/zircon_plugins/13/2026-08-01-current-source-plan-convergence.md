# Plugins 13 当前源码与计划收敛记录

- 日期：2026-08-01
- 归属计划：[`13-standalone-plugin-build.md`](../13-standalone-plugin-build.md)
- 结论：B1-B8 已标为 2026-06-23 初始基线；当前待办不再重复列出已交付的独立构建架构。
- 代码证据：当前树包含 38 个 `dist/Cargo.toml`；根级结构审计沿用 37 个 plugin root 口径。`zircon_plugins/ai/dist/Cargo.toml` 声明 `cdylib` 且只依赖 native SDK，`ai/dist/src/lib.rs` 使用 ABI v3 helper 导出 registration manifest 并带 focused tests。
- 计划修正：区分 root matrix 与 feature-local/non-root dist carrier，确认 build/validate、依赖 guard、per-plugin package 与开放 plugin id 已有实现和后续记录。
- 未关闭范围：更广 editor/export/full regression 以及 `failure-2026-07-15-virtual-geometry-runtime-support-compute-workload-drift.md` 保持原状态。
- 验证：完成当前源码静态对账与 scoped diff 检查；本记录不声明 Cargo、真实导出或 Hub/editor E2E 通过。
