---
handoff_kind: fixed
status: fixed
created_at: 2026-07-15
summary_slug: compound-shader-persisted-reference-contract
origin_plan: docs/plans/zircon_runtime/shader/03-module-imports-and-cross-references.md
fixing_plan: docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
origin_child_dir: docs/plans/zircon_runtime/shader/03
fixing_child_dir: docs/plans/zircon_runtime/runtime/04
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/tests/material_shader_redirect_dependency_contract.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import/sources.rs
  - zircon_runtime/src/asset/project/manager/persisted_reference.rs
  - zircon_runtime/src/asset/reference_resolver.rs
  - zircon_runtime/src/asset/migration/resolver.rs
tests:
  - cargo test -p zircon_runtime --test material_shader_redirect_dependency_contract --no-default-features --features target-server --locked
resolved_at: 2026-07-16
---


# Runtime04: compound shader persisted-reference contract is incomplete

## 来源执行者

- 来源计划：`docs/plans/zircon_runtime/shader/03-module-imports-and-cross-references.md`
- 来源执行者：`shader03-material-redirect-asset-contract-20260715`
- 来源执行切片：SH03 material redirect dependency contract canonical importer/document migration。
- 修复责任计划：`docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md`
- 交接原因：Shader03 已把 material redirect fixture 迁移到生产 `ZMaterialDocument` 构造和 project-reference serializer；生成的 project material 必须引用 compound shader package，因此暴露的是 Runtime04 缺失的 persisted-reference mapping，而不是 Shader03 construction 或 redirect graph 缺陷。

## 失败现象与复现证据

`AssetSourceUnit::Compound` deliberately gives a compound shader package a logical asset URI at its directory root.  For the fixture package this is `res://shaders/redirect_surface`, while `redirect_surface.zmeta` is the physical source metadata file.

The canonical project-reference boundary currently assumes every `AssetReference::locator` names a safe regular file:

- `ProjectManager::persist_runtime_reference` rejects the compound directory through `is_safe_regular_file` before it can emit `PersistedAssetReference::Project`.
- `resolve_project_reference` and `MigrationResolver::project_relative_path` make the same regular-file check when resolving a persisted or retired reference.

Consequently, a material serialized through `ZMaterialDocument::to_project_toml_string` cannot round-trip a canonical reference to a registered compound shader asset.  The Shader03 dependency-contract test correctly avoids raw `{ uuid, url }` data and `from_toml_str`; accepting the material with a test-only path or direct locator comparison would conceal this lower asset-pipeline hole.

## 最低共享层根因

Runtime04 has no single canonical mapping from a compound asset's logical root URI to the physical project-relative persisted reference accepted by its writer, scanner, migration resolver, and runtime resolver.  Treating arbitrary directories as ordinary source files is not an architectural repair: the mapping must preserve the asset registry's compound-source semantics and resolve the same `AssetUuid` through every production entry point.

## 架构修复验收

- Define and implement one canonical persisted-reference mapping for compound asset roots, consistently used by `persist_runtime_reference`, scan/import, migration, and project-reference resolution.
- A `ZMaterialDocument` serialized with `to_project_toml_string` must load through `ProjectManager::scan_and_import` when it points at a compound `.zmeta` shader package; the loaded shader dependency must retain its persisted `AssetUuid` identity.
- The original Shader03 target must then pass without test-only parsing APIs or comparison traits:

  ```powershell
  cargo test -p zircon_runtime --test material_shader_redirect_dependency_contract --no-default-features --features target-server --locked
  ```

- Re-run the affected Runtime04 asset-pipeline checks and report whether the canonical material project-reference flow reaches the original assertions.

## 禁止临时方案

- Do not restore legacy raw `{ uuid, url }` material references, `from_toml_str`, or a test-only resolver.
- Do not serialize the compound directory as if it were a generic regular-file source, or special-case this test's path outside the shared asset registry contract.
- Do not use a synthetic single-file `.zshader` fixture: the registered shader importer consumes the `.zmeta` compound package contract.
- Do not weaken Shader03's missing-dependency or redirect-readiness assertions.

## 修复结果与回传

- 根因：Compound logical directory locators had no canonical mapping to their physical .zmeta persisted source; target-server also omitted real shader-package importer registration.
- 架构修复：Added one validated logical-root to .zmeta mapping shared by persistence, scan/import, runtime reference resolution, and migration; target-server enables the real package importer with Naga only, not GPU graphics.
- 验证：Focused Runtime04 mapping checks: 4/4 passed. Exact Shader03 target: 2/2 passed (managed job a4de04ae9e6a4ba8b2ec7debbc81b39f). Current-source zircon_runtime build passed (managed job c402eb59bbc2438eba07045057994d0e). Final independent review: Critical 0, Important 0.
- 回传：Canonical compound shader persisted references now use assets/shaders/redirect_surface.zmeta while preserving the registered AssetUuid; Shader03 redirect dependency contract passes unchanged.
