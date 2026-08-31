---
owner_plan: docs/plans/zircon_editor/editor/11-serialization-and-versioning.md
milestone: M2
slice: workbench-layout-version-shell-hard-cut
status: implementation_complete_isolated_gate_green_full_editor_blocked
related_code:
  - zircon_editor/src/ui/workbench/layout_persistence_document.rs
  - zircon_editor/src/ui/host/layout_persistence.rs
  - zircon_editor/src/ui/workbench/layout_preset.rs
  - zircon_editor/src/ui/workbench/project/editor_workspace_document.rs
  - zircon_editor/src/ui/workbench/project/editor_workspace_persistence.rs
  - zircon_editor/src/ui/workbench/project/layout_preset_asset_document.rs
  - zircon_editor/src/ui/workbench/project/layout_preset_assets.rs
tests:
  - zircon_editor/src/ui/workbench/layout_persistence_document.rs
  - zircon_editor/src/ui/workbench/project/editor_workspace_document.rs
  - zircon_editor/src/ui/workbench/project/layout_preset_asset_document.rs
  - zircon_editor/src/tests/workbench/layout/layout_preset_persistence.rs
---

# Workbench 布局统一版本壳硬切

本切片按 Plan11 M2.1 收口所有当前 workbench 布局 IO，不保留旧 reader、alias、双写字段或
隐式格式判断。实现前复核了 host 配置、工程 workspace、工程 preset asset、按用户/页面 preset
store 的完整调用拓扑，并以 Unreal `ApplicationMode.cpp` 中
`TabManager->PersistLayout()` -> `FLayoutSaveRestore::{SaveToConfig,LoadFromConfig}` 为 owner
边界参考：session 仍是内存布局权威，专用 persistence owner 只在显式 IO 时读写快照。

## 产出记录与时间

| 时间 | 状态 | 完成项目 | 验证与未完成项 |
| --- | --- | --- | --- |
| 2026-08-28 CST | `implementation_complete_isolated_gate_green_full_editor_blocked` | 新增 `layout_persistence_document.rs`，为 global default、named presets、user/page presets 建立三个互不混用的 `$zircon` v1 schema；工程 workspace 与工程 layout preset asset 分别建立独立 v1 schema。五个 schema 均以显式 v0 migration step 拒绝裸 JSON。删除工程 workspace/preset 的私有 `format_version`、`ProjectEditorWorkspace.layout_version`、page preset 逐条 `PersistedLayoutPreset.format_version` 及相关常量/公开类型；无兼容 reader。工程文件改用 borrowed document view 与 Runtime `atomic_write`，preset save 不再为序列化 clone 完整布局。坏的配置型布局告警后回退 built-in/空 store，使新格式仍可覆盖旧配置；坏工程 workspace 保留 typed diagnostic；工程 preset asset 保持 typed fail-closed。模块合同见 `docs/zircon_editor/ui/workbench/layout-persistence.md`。 | TDD RED：D 盘隔离门 3/3 在六个未实现 codec 入口失败。实现后，直接包含 E 盘实际三个 document owner，并使用当前 `zircon_runtime_interface/src/serialization/**` 原样 D 盘最小快照，`cargo test --manifest-path D:\zt\plan11-layout-shell\Cargo.toml --offline` 为 7 passed / 0 failed / 0 ignored，0.10s；覆盖 current shell round-trip、三类 config schema 隔离、cross-schema 拒绝、global/project workspace/project preset 裸 v0 拒绝。限定 rustfmt 通过。产品门 `cargo check -p zircon_editor --lib --offline --target-dir D:\zt\plan11-layout-product-target` 在进入 editor 前被当前共享 `zircon_runtime_interface/src/ui/text/model_update.rs` 4 个 E0609 阻断（`UiTextModelUpdateRequest` 不存在 `status/failure` 字段），不归本切片，故未声明完整 editor GREEN、未提交 commit、未同步协调器或企微。M2.1 仍需 03 journal 和动态全门后才可勾选。 |

## 性能边界

本切片是结构正确性硬切，不是性能优化，因此没有在缺少 profiler capture 的情况下声明耗时或
功耗收益。工程文档 writer 使用借用 payload，避免仅为保存而 clone 整个 workspace/layout；配置
值因 `ConfigManager` 的 `serde_json::Value` 合同需要一次 value/text 转换，但只发生在显式保存或
恢复，不进入 frame、pointer、projection 或 paint 路径。后续若优化该转换，必须先按
`docs/plans/optimize` 与 PERF-MVP-570 完成 reader/canonical writer profiling，再修改算法。
