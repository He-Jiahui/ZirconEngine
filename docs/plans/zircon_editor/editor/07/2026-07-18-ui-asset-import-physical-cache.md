---
plan: zircon-editor-07
failure: ui-asset-editor-full-projection-and-import-rehydrate
status: implemented-validation-pending-failure-open
session: editor07-domain-performance-failure-repairs-r3-20260718
related_code:
  - zircon_editor/src/ui/host/asset_editor_sessions/imports/generation.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/imports/traversal.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/hydration.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/refresh/pipeline/job.rs
  - zircon_editor/src/ui/host/asset_editor_sessions/refresh/pipeline/commit.rs
tests:
  - tools/tests/test_editor07_ui_asset_import_physical_cache_contract.py
  - zircon_editor/src/ui/host/asset_editor_sessions/imports/tests.rs
---

# UI Asset Import Physical Cache

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 验证与未完成项 |
|---|---|---|---|
| 2026-07-23 12:41 +08:00 | `implemented-validation-pending-failure-open` | 随 Editor09 watcher generation 硬切，将本记录的实现与测试锚点从已删除的扁平 `imports.rs`/`refresh.rs` 更新为 `imports/{generation,traversal,tests}.rs` 与 `refresh/pipeline/{job,commit}.rs`；不再保留旧架构路径。 | Editor07 physical-cache 静态合同 4/4 GREEN；受管 Cargo 仍受 Coordinator01 validation-copy failure 与外部 Cargo 队列阻塞，因此 parent failure 继续 open。 |
| 2026-07-18 19:15 +08:00 | `implemented-validation-pending-failure-open` | 收紧 physical identity 边界：canonical resolved path 现在同时拥有 generation cache key、实际 `read_to_string` 目标与 `.zui`/legacy parser mode 判定，禁止 alias/symlink 路径用一套 identity 缓存另一套读取内容或解析语义。 | 新静态合同先 RED（1/4 失败）再 GREEN（4/4）；精确 rustfmt 与 scoped diff check 通过。未运行 Cargo；parent failure 仍按原剩余项保持 open。 |
| 2026-07-18 18:35 +08:00 | `implemented-validation-pending-failure-open` | 新增 generation-scoped `UiAssetImportTraversal`，以 canonical resolved source path 缓存成功或失败的 read/parse/v2-project 结果；成功值使用 `Arc` 避免 cache hit 克隆整份 parse tree，并以独立 physical expansion set 终止 diamond/cycle 重复展开。logical `reference#fragment` 仍逐条 materialize，expected-kind 仍逐 edge 校验。strict hydration 与 lossy workspace refresh 均在全部 widget/style roots 间复用同一 traversal，旧 normalized-reference visited 参数已删除。 | TDD RED 为静态合同 3/3 失败，实现后 3/3 通过；精确 rustfmt、scoped diff check、旧 visited 形态与调用点扫描通过。Rust 行为测试已写入 `imports.rs`，覆盖单 generation load 一次、两个 fragment alias 两条 logical rows/一次 physical expansion，以及失败 parse 缓存；因共享 Runtime12 Cargo 与 Editor10/Render02 source-bound 顺序门，本轮未运行 Rust tests。Failure 保持 open：typed dirty-domain projection、typing debounce/后台 revision 安全、1k stress build/read/parse/clone/p95 与行为等价证据尚未完成。 |
