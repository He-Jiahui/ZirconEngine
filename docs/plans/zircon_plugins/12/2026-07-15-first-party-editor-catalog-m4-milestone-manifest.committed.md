Plan: docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md
Committed-Milestone: M4
Status: completed
Files: ["docs/plans/zircon_plugins/12/2026-07-15-first-party-editor-catalog-m4-milestone-manifest.md","zircon_plugins/Cargo.toml","zircon_plugins/Cargo.lock","zircon_plugins/first_party_editor_catalog/Cargo.toml","zircon_plugins/first_party_editor_catalog/src/catalog.rs","zircon_plugins/first_party_editor_catalog/src/lib.rs","zircon_plugins/first_party_editor_catalog/src/tests.rs"]

# Plugins12 M4 first-party editor catalog 里程碑清单

## Scope Delivered

M4 建立 first-party editor plugin catalog 的单一装配入口，并将该 support catalog 纳入插件 workspace。catalog 只汇总已有 editor plugin 声明，不伪装为可分发插件根；workspace 同时保留 Render18 已加入的 light cookies 与 irradiance volumes runtime/editor 成员。

## Fresh Testing Evidence

- catalog 源码采用薄 `lib.rs` facade，目录级行为 owner 与测试分离。
- 独立复核结果为 0 Critical / 0 Important。
- 本里程碑通过协调器 validation copy 执行隔离校验；结果由对应 workflow run 记录。

## Review

独立 reviewer 已核对 catalog 汇总、workspace 成员保留和结构约束，结论为 0 Critical / 0 Important。

## Acceptance Boundary

本清单只声明自身与上述六个 Cargo/catalog 文件。Plugins12 其他 runtime mirror、动态 ABI、Editor 生命周期与计划状态改动继续由主会话持有，不进入本次 catalog handoff 提交。
