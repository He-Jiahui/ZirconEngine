---
related_code:
  - zircon_runtime/src/asset/pack
  - zircon_runtime/src/core/framework/net/download.rs
  - zircon_runtime/src/core/framework/net/mod.rs
  - zircon_runtime/src/bin/zircon_export_pack
implementation_files:
  - zircon_runtime/src/asset/pack/manifest.rs
  - zircon_runtime/src/asset/pack/mod.rs
  - zircon_runtime/src/asset/pack/delta.rs
  - zircon_runtime/src/asset/pack/reader.rs
  - zircon_runtime/src/asset/pack/writer.rs
  - zircon_runtime/src/asset/tests/pack.rs
  - zircon_runtime/src/asset/tests/pack/basic.rs
  - zircon_runtime/src/core/framework/net/download.rs
  - zircon_runtime/src/core/framework/net/mod.rs
  - zircon_runtime/src/core/framework/net/tests.rs
  - zircon_runtime/src/bin/zircon_export_pack/main.rs
  - zircon_runtime/src/bin/zircon_export_pack/pack.rs
  - tools/tests/test_frameworks_03_contract_feature_boundary.py
plan_sources:
  - docs/plans/zircon_runtime/frameworks/03-optional-features-and-profile-matrix.md
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - python -m unittest tools.tests.test_frameworks_03_contract_feature_boundary.Frameworks03ContractFeatureBoundaryTests.test_zrpack_protocol_types_have_one_asset_pack_owner
  - cargo +nightly check -p zircon_runtime --bin zircon_export_pack --no-default-features --features core-min --locked --offline --jobs 1
  - cargo +nightly test -p zircon_runtime --bin zircon_export_pack --no-default-features --features core-min --locked --offline --jobs 1 -- --nocapture --test-threads=1
doc_type: acceptance-evidence
status: passed-with-follow-up
---

# Frameworks 03 ZRPack asset owner 硬切验收证据

## 范围

本记录覆盖 `net-contracts` 门控前置的 ZRPack DTO 所有权硬迁移。它不声明 Net feature、Net plugin matrix 或 Runtime 全局 lib-test 已完成。

## 硬切结果

- `asset::pack::manifest` 是 `ZrPackManifest` 与 `ZrChunkEntry` 的唯一生产定义 owner，并从 `asset::pack` 公开。
- pack manifest/reader/writer/delta 与 asset tests 只使用同域类型，不再依赖 `core::framework::net`。
- Net download contract 删除 ZRPack 定义和 re-export，只保留网络下载描述、重试、镜像与进度 DTO。
- `zircon_export_pack` 继续通过 `#[path]` 复用 asset pack 源文件，但删除 fake `core::framework::net` 模块及其重复 DTO；工具根从共享 manifest re-export 类型。
- 没有兼容 re-export、type alias 或双 owner 过渡期。

## 验证

- 唯一 owner 静态守卫通过；仓库扫描在 asset manifest 之外找不到第二份 ZRPack/Chunk 结构定义，也找不到 asset pack→Net 路径。
- WSL nightly locked/offline export-pack check 通过，3m17s。
- WSL nightly locked/offline export-pack test target 通过，3/3，9m31s；三项测试实际执行 pack preflight/trim/source failure 路径并确认不写非法包。
- scoped nightly rustfmt 与 `git diff --check` 通过。

## 保留失败

`cargo +nightly test -p zircon_runtime --lib --no-default-features --features core-min ... asset::tests::pack::` 在用例执行前失败。84 个错误均来自全局 Runtime 测试模块仍无条件引用 graphics/UI/script/diagnostic-log 及其可选依赖，这是 Frameworks 03 M1 测试树门控债务；本次迁移没有 pack/net 类型错误。该命令不计为通过，也不为 pack 增加特判，后续 M1 testing stage 必须统一修复测试声明边界后重跑默认 lib suite。

## 当前判定

ZRPack asset owner 硬迁移完成，Net 契约已具备独立门控前提。新增 asset pack serde/helper 测试已经归位，但其 library test execution 与完整 M1 test gate 一起 pending。
