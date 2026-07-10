---
related_code:
  - zircon_runtime/src/asset/assets/ui.rs
  - zircon_runtime/src/asset/assets/ui/document_loader.rs
  - zircon_runtime/src/asset/assets/ui/resource_references.rs
  - zircon_runtime/src/asset/importer/ingest/asset_importer.rs
  - zircon_runtime/src/asset/importer/ingest/mod.rs
  - zircon_runtime/src/asset/tests/support.rs
  - zircon_runtime/src/asset/tests/assets/importer/typed_toml_ui.rs
  - zircon_runtime/src/asset/tests/assets/ui/importer.rs
  - zircon_runtime/src/asset/tests/assets/ui/project_manager.rs
  - zircon_runtime/src/asset/artifact/cache_payload.rs
  - zircon_runtime/src/asset/artifact/cache_payload/font.rs
  - zircon_runtime/tests/font_artifact_cache_contract.rs
  - zircon_runtime/src/lib.rs
  - zircon_plugins/ui_document_importer/runtime/src/lib.rs
  - zircon_plugins/ui_document_importer/runtime/src/plugin.rs
  - tools/runtime_domain_dependency_audit.py
  - tools/tests/test_frameworks_05_asset_ui_boundary.py
implementation_files:
  - zircon_runtime/src/asset/assets/ui.rs
  - zircon_runtime/src/asset/assets/ui/document_loader.rs
  - zircon_runtime/src/asset/assets/ui/resource_references.rs
  - zircon_runtime/src/asset/importer/ingest/asset_importer.rs
  - zircon_runtime/src/asset/importer/ingest/mod.rs
  - zircon_runtime/src/asset/tests/support.rs
  - zircon_runtime/src/asset/tests/assets/importer/typed_toml_ui.rs
  - zircon_runtime/src/asset/tests/assets/ui/importer.rs
  - zircon_runtime/src/asset/tests/assets/ui/project_manager.rs
  - zircon_runtime/src/asset/artifact/cache_payload.rs
  - zircon_runtime/src/asset/artifact/cache_payload/font.rs
  - zircon_runtime/tests/font_artifact_cache_contract.rs
  - zircon_runtime/src/lib.rs
  - tools/tests/test_frameworks_05_asset_ui_boundary.py
plan_sources:
  - docs/plans/zircon_runtime/frameworks/05-subsystem-decoupling-contracts.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
  - user: 2026-07-10 frameworks 基础架构新版硬切换目标
tests:
  - tools/tests/test_frameworks_05_asset_ui_boundary.py
  - tools/tests/test_runtime_domain_dependency_audit.py
  - zircon_runtime/src/asset/tests/assets/ui.rs
  - zircon_runtime/tests/font_artifact_cache_contract.rs
doc_type: acceptance-evidence
status: package_check_ui_and_font_cache_focused_validation_passed_full_lib_test_pending
---

# Frameworks 05 asset→ui loader 硬切换验收证据

## 完成范围

- asset production 对 `crate::ui` 的直接引用从 3 清零。
- asset 内 built-in `.zui` importer、旧 importer owner 和转换 owner 已删除；不保留 alias 或兼容 shell。
- `.zui` 只由 `ui_document_importer` runtime plugin 通过既有 `RuntimeExtensionRegistry` / `AssetImporterRegistry` 注册。
- asset wrapper 的当前 schema codec 和资源 URI 提取改为 asset 本域 helper，不再借用 UI 实现。
- crate root 的“UI 必须先于 asset 声明”语义注释已删除。

## 验证

- `python -m unittest tools.tests.test_frameworks_05_asset_ui_boundary tools.tests.test_runtime_domain_dependency_audit`：5/5 通过。
- dependency audit：2401 production references / 79 domain edges；asset→ui = 0（M1 为 3）。
- focused Rust 首轮运行：16/18；仅两个 project-manager fixture 未安装 plugin importer。修复后于 2026-07-11 复用当前默认 feature lib-test binary 直接执行 `asset::tests::assets::ui`，18/18 通过、0 failed、7433 filtered、测试体 0.24s。
- 本轮 Cargo 增量构建在 10 分钟工具上限后仍完成了当前 999,329,792-byte lib-test binary；随后直接执行该二进制取得上述精确结果。该 focused 证据不替代计划要求的完整 Runtime package check/lib-test。
- `cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-shader-probes-0710`：通过，5m24s，当前基线 418 warnings，无 compile error。
- 扩大到 `asset::tests` 的当前默认 feature 二进制首轮为 393/395；字体用例稳定失败为 `ArtifactCacheDeserialize(UnexpectedEof)`，另一个 Vampire manifest/shader 失败属于活动 Shader/PBR owner 的期望漂移。根因是 `FontAsset` 的 authoring `skip_serializing_if` 直接参与顺序 bincode wire，已硬切到 `asset/artifact/cache_payload/font.rs` 完整字段 DTO，不保留旧缓存 reader 或兼容分支。
- 新公开边界测试 `font_artifact_cache_roundtrips_fields_omitted_by_authoring_formats` 覆盖默认省略字段、family member、variable instance、composite font、parsed face/metrics/variation/cmap 与 `ZRARTZ01` 磁盘往返；当前 `core-min` rlib 直接编译后 1/1 通过（0.03s）。
- `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:\cargo-targets\zircon-frameworks05-font-cache-0711`：通过，4m59s，52 existing warnings。
- 当前默认 feature lib-test 新构建被活动 text-layout owner 的 rich-text 根导出迁移阻断（6 errors）；`core-min` unit-test 又被仓库级 test cfg 缺口阻断（83 errors），Cargo integration orchestration 被未声明 required-features 的 `zircon_host_reflection_docs` 阻断。以上均未修改绕过，也不声明完整 test pass。

## 当前判定

M2 代码硬切、静态架构门、机器基线、既有完整 Runtime lib check、focused UI Rust 18/18 与字体 artifact cache focused contract 1/1 已完成；计划规定的全量 `cargo test -p zircon_runtime --lib --locked` 和修复后 `asset::tests` 全组复验仍 pending。Frameworks 05 M2、M3–M4 以及用户总目标均未完成。
