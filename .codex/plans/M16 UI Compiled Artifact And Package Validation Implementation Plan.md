# M16 UI Compiled Artifact And Package Validation Implementation Plan

## Summary
完成 M16 的第一批产品化缺口：把 M7/M10/M12/M13 已落地的 UI Asset 文档、组件契约、描述符权威、失效与编译缓存能力，收束成可验证的“编译产物元数据 + 依赖清单 + package validation report”。

2026-05-01 back-half reconciliation note：本文件最初只定义 V1 metadata/report surface；后续 runtime/package slice 已在同一 owner 下补齐 binary artifact envelope、cache record、package manifest writer/importer，并把 M15 compiled resource dependencies 写入 dependency manifest。历史假设“V1 不设计真实磁盘 binary artifact 格式，也不实现 persistent compiled cache”只适用于初版计划，不再描述当前源码状态。

目标 plan 文件路径：`E:\Git\ZirconEngine\docs\superpowers\plans\2026-05-01-ui-compiled-artifact-package-validation.md`

## Key Changes
- 在 `zircon_runtime` 的 UI asset/compiler 归属下新增 M16 package validation owner，提供：
  - `UiCompiledAssetHeader`
  - `UiCompiledAssetDependencyManifest`
  - `UiCompiledAssetPackageValidationReport`
  - `UiCompiledAssetPackageProfile::{Runtime, Editor}`
- 新增 `UiDocumentCompiler::validate_package(...)` 入口，复用现有文档解析、组件契约、描述符 registry revision、compile cache key 与 invalidation fingerprint，不另起并行规则。
- Header 记录 source schema version、compiler/package schema version、descriptor registry revision、component contract revision、root document fingerprint、compile cache key summary。
- Dependency manifest 以确定性顺序记录 widget/style imports 及其 fingerprint，并消费 M15 compiled resource dependencies，作为 M12 cache 与资源 package report 的同一 manifest surface。
- Package report 输出 runtime/editor profile 下的 retained/stripped section 信息；back-half 另行持久化 binary artifact envelope、cache record 和 TOML package manifest writer/importer。
- 更新 UI asset 文档与未完成归档，把 M16 从 “mostly missing” 调整为“compiled artifact metadata/package validation landed”，并写入验证证据。

## Test Plan
- 新增 focused runtime tests：
  - header 会包含 schema version、registry revision、contract revision、document fingerprint 与 cache key 信息。
  - widget/style dependency manifest 顺序稳定，fingerprint 与 M12 invalidation/cache 输入一致。
  - package validation 失败时沿用现有 asset/compiler 错误路径，不吞掉 M10/M13 契约错误。
  - runtime/editor profile report 稳定区分 retained/stripped section。
  - report 生成不会改变 source document、compiler registry 或 cache 内容。
  - back-half artifact envelope roundtrip、cache record、package manifest writer/importer、compiled resource dependency rows。
- 回归验证：
  - `asset_component_contract`
  - `asset_compile_cache`
  - `asset_invalidation`
  - `component_catalog`
  - `ui::tests::asset`
  - workspace-level `cargo check` 或现有 zircon validation wrapper

## Assumptions
- M16 V1 只落 metadata/report surface；back-half 已补 binary artifact envelope、cache record、package manifest writer/importer，但仍不实现跨进程 cache store。
- M14 localization、M21 action safety/host policy 不在本 plan 内实现。M15 media/font/resource refs 的 runtime/package collector 与 manifest hookup 已接入；resolver/file existence checks、runtime loader backend、editor resource UX 和 graphics/RHI consumption 仍在计划外。
- 避开当前活跃的 plugin renderer hard-cutover、runtime UI showcase/schema panel、runtime UI data-source 相关会话范围。
- 代码变更完成时同步维护 `docs/ui-and-layout` 下的模块文档与 `.codex/plans/UI Asset Editor 与共享 Layout 未完成内容归档.md`。

## Back-Half Evidence

- `cargo test -p zircon_runtime --lib asset_package_validation --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m15-m16-resource-package --message-format short --color never` passed with `9 passed; 0 failed`, including the contract-precondition-before-resource-fingerprint regression.
- `cargo test -p zircon_runtime --lib asset_compile_cache --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m15-m16-resource-package --message-format short --color never` passed with `9 passed; 0 failed`, including the shape-precondition-before-resource-fingerprint regression.
- `cargo test -p zircon_runtime --lib asset_invalidation --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m15-m16-resource-package --message-format short --color never` passed with `6 passed; 0 failed`.
- `cargo test -p zircon_runtime --lib asset_resource_refs --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m15-m16-resource-package --message-format short --color never` passed with `9 passed; 0 failed`.
- `cargo test -p zircon_runtime --lib ui::tests::asset --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m15-m16-resource-package --message-format short --color never` passed with `86 passed; 0 failed`.
- `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ui-m15-m16-resource-package --message-format short --color never` passed with unrelated graphics/plugin warnings.
