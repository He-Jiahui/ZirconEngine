# Asset Editor API Boundary Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 把 editor asset catalog/details/change/preview 这组 API 从 `zircon_manager` 迁回 `zircon_asset`，让 `zircon_editor` 直接依赖 asset-owned 协议，同时移除 `zircon_manager` 的 editor-only 泄漏。

**Architecture:** 保留 `DefaultEditorAssetManager` 仍由 `zircon_asset` 实现和注册，但把 trait、records、resolver、handle、service-name 常量统一下沉到 `zircon_asset::editor` 子树并在 crate 根 re-export。`zircon_editor` 改成从 `zircon_asset` 获取这组 API；`zircon_manager` 收缩回通用 manager façade，不再承载 editor asset 协议。

**Tech Stack:** Rust 2024, cargo test/check, repo-local docs, Slint editor host, crossbeam-channel

---

## Target File Structure

```text
zircon_asset/src/editor/
  mod.rs
  api.rs
  records.rs
  resolver.rs
  catalog.rs
  preview.rs
  reference_graph.rs
  manager.rs
```

边界目标：

- `zircon_asset` 根级公开 `EditorAssetManager`、`EditorAsset*Record`、`EditorAssetManagerHandle`、`resolve_editor_asset_manager`、`EDITOR_ASSET_MANAGER_NAME`
- `zircon_manager` 仅保留 `AssetManager` / `ResourceManager` / `ConfigManager` / `InputManager` / `EventManager` / `LevelManager` / `RenderingManager`
- `zircon_editor` 的 asset workspace、event runtime、Slint host 只从 `zircon_asset` 获取 editor asset 协议

## Validation Baseline

- `cargo test -p zircon_asset editor_asset --locked`
- `cargo test -p zircon_editor asset --locked`
- `cargo test -p zircon_manager --locked`

预期：在新增边界 red tests 后先失败，再通过实现转绿。

### Task 1: Add Boundary Regression Tests

**Files:**
- Create: `zircon_asset/src/tests/editor/boundary.rs`
- Modify: `zircon_asset/src/tests/editor/mod.rs`
- Create: `zircon_editor/src/tests/host/asset_manager_boundary.rs`
- Modify: `zircon_editor/src/tests/host/mod.rs`
- Modify: `zircon_manager/src/tests.rs`

- [ ] 写 source-based red tests，锁死 `zircon_asset` 必须导出本地 editor asset API
- [ ] 写 source-based red tests，锁死 `zircon_editor` host 不再通过 `zircon_manager` 获取 editor asset API
- [ ] 写 source-based red tests，锁死 `zircon_manager` 不再公开 `EditorAssetManager` / `EDITOR_ASSET_MANAGER_NAME`
- [ ] 运行：
  - `cargo test -p zircon_asset editor_asset_api_boundary_lives_in_zircon_asset --locked`
  - `cargo test -p zircon_editor editor_asset_boundary_lives_in_asset_crate --locked`
  - `cargo test -p zircon_manager manager_public_surface_excludes_editor_asset_api --locked`

### Task 2: Move Editor Asset API Into `zircon_asset`

**Files:**
- Modify: `zircon_asset/src/editor/mod.rs`
- Create: `zircon_asset/src/editor/api.rs`
- Create: `zircon_asset/src/editor/records.rs`
- Create: `zircon_asset/src/editor/resolver.rs`
- Modify: `zircon_asset/src/editor/manager.rs`
- Modify: `zircon_asset/src/lib.rs`
- Modify: `zircon_asset/src/pipeline/manager/module_descriptor.rs`
- Modify: `zircon_asset/src/pipeline/manager/service_names.rs`

- [ ] 在 `zircon_asset::editor` 下创建 trait、records、resolver/handle 模块
- [ ] 让 `DefaultEditorAssetManager` 实现本地 `EditorAssetManager` trait
- [ ] 让 module descriptor 用本地 `EditorAssetManagerHandle` 注册 public manager service
- [ ] 在 `zircon_asset` 根导出这组 API，并保持 `DefaultEditorAssetManager` 与现有 service name 可用
- [ ] 运行：
  - `cargo test -p zircon_asset editor_asset --locked`

### Task 3: Remove Editor Asset Leakage From `zircon_manager`

**Files:**
- Modify: `zircon_manager/src/traits.rs`
- Modify: `zircon_manager/src/records/mod.rs`
- Delete: `zircon_manager/src/records/editor_asset.rs`
- Modify: `zircon_manager/src/resolver.rs`
- Modify: `zircon_manager/src/service_names.rs`
- Modify: `zircon_manager/src/lib.rs`
- Modify: `zircon_manager/src/tests.rs`

- [ ] 删除 `EditorAssetManager` trait、`EditorAsset*Record`、`EditorAssetManagerHandle`、`resolve_editor_asset_manager`、`EDITOR_ASSET_MANAGER_NAME`
- [ ] 保证 `zircon_manager` 其余 manager facade 不受影响
- [ ] 运行：
  - `cargo test -p zircon_manager --locked`

### Task 4: Rewire `zircon_editor` To Asset-Owned API

**Files:**
- Modify: `zircon_editor/src/editing/asset_workspace.rs`
- Modify: `zircon_editor/src/editing/state/editor_state_asset_workspace.rs`
- Modify: `zircon_editor/src/editor_event/runtime/accessors.rs`
- Modify: `zircon_editor/src/host/slint_host/app.rs`
- Modify: `zircon_editor/src/host/slint_host/app/host_lifecycle.rs`
- Modify: `zircon_editor/src/tests/editing/asset_workspace.rs`
- Modify: `zircon_editor/src/tests/host/slint_asset_pointer.rs`
- Modify: `zircon_editor/src/tests/host/slint_asset_refresh.rs`
- Modify: `zircon_editor/src/tests/host/asset_manager_boundary.rs`

- [ ] 把 editor asset records / trait imports 从 `zircon_manager` 切到 `zircon_asset`
- [ ] 把 host lifecycle 的 resolver 从 `ManagerResolver::editor_asset()` 切到 `zircon_asset::resolve_editor_asset_manager(...)`
- [ ] 更新相关测试样本与边界断言
- [ ] 运行：
  - `cargo test -p zircon_editor asset --locked`

### Task 5: Sync Docs And Final Validation

**Files:**
- Modify: `docs/assets-and-rendering/directory-project-asset-rendering.md`
- Modify: `docs/assets-and-rendering/index.md`
- Modify: `docs/ui-and-layout/ui-asset-documents-and-editor-protocol.md`

- [ ] 更新文档头部 `related_code` / `implementation_files` / `tests`
- [ ] 在正文中把 `EditorAssetManager` 从“`zircon_manager` façade”改成“`zircon_asset` owned editor-facing asset workspace API”
- [ ] 运行：
  - `cargo test -p zircon_asset --locked`
  - `cargo test -p zircon_editor --locked`
  - `cargo test -p zircon_manager --locked`
  - `cargo check --workspace --locked`

## Sequencing Rules

- 先加 boundary tests，再改 API 所有权
- 先让 `zircon_asset` 自己能独立导出和测试这组 API，再删 `zircon_manager` 的旧面
- `zircon_editor` 最后切换 imports，避免中间态同时编译两套 public surface
- docs 只在代码和测试稳定后更新

## Completion Checklist

- `zircon_asset` 根级可直接提供 editor asset trait / records / resolver / handle
- `zircon_manager` public surface 不再包含 editor asset API
- `zircon_editor` 不再通过 `zircon_manager` 获取 editor asset records 或 resolver
- 相关 docs 同步更新
- `cargo test -p zircon_asset --locked`、`cargo test -p zircon_editor --locked`、`cargo test -p zircon_manager --locked` 至少通过
