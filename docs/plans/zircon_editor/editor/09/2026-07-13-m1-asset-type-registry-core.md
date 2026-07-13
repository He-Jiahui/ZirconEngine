---
status: completed
owner_plan: docs/plans/zircon_editor/editor/09-editor-asset-management.md
recorded_at: 2026-07-13
milestone: M1
slice: 1.1
related_code:
  - zircon_editor/src/core/asset/mod.rs
  - zircon_editor/src/core/asset/type_registry
  - zircon_editor/src/core/mod.rs
  - zircon_editor/src/tests/editor_asset_type_registry
  - zircon_editor/src/tests/mod.rs
related_docs:
  - docs/zircon_editor/core/asset.md
  - docs/plans/zircon_editor/editor/09/2026-07-13-m1-approved-asset-type-registry-design.md
---

# Editor09 M1.1 AssetTypeRegistry 核心产出

## 完成范围

- 新增 folder-backed `core/asset/type_registry/` owner；根 `mod.rs` 仅负责精选导出，不承载行为。
- 新增开放、严格校验并自定义 serde 的 `AssetTypeId`；合法语法为 lowercase dotted segments，
  大写、空白、路径分隔、控制字符、空/连续 segment 均返回 `AssetTypeIdError`。
- 为 26 个 `ResourceKind` 建立唯一 canonical mapping 与完整内建 definition；首方 authoring key
  对齐 `material.graph`、`animation.state_machine`、`terrain.heightfield`、
  `tilemap_2d.tilemap`，不保留旧 uppercase alias。
- 新增可序列化 `AssetTypeContribution`、materialized `AssetTypeDefinition`、presentation、toolkit、
  thumbnail descriptor 与 typed registry error。
- registry merge 采用候选副本后提交；不完整 custom base、空必要 presentation 字段和重复 scalar/toolkit
  owner 均失败且不污染 live registry。
- 生产模块无 `unwrap/expect`，无 `core -> ui` 反向依赖；最大新增源文件 210 行，未继续膨胀
  已接近结构上限的 `editor_extension.rs`。

## 验证证据

- Windows 受管 Cargo job `9bc49f2eccff4b74bdd95774a3dff666`：
  `cargo test -p zircon_editor --lib editor_asset_type_registry --locked --no-run --jobs 1`，退出码 0；
  `zircon_editor` lib-test 二进制生成成功。
- 从当前源码生成的测试二进制执行 `editor_asset_type_registry --nocapture --test-threads=1`：
  9 passed、0 failed、3125 filtered out，耗时 0.07s；日志：
  `.codex/tmp/editor09-asset-type-registry-focused-tests-20260713.log`。
- scoped `rustfmt --edition 2021 --check`：通过。
- 结构扫描：`core/asset` 生产 Rust 文件无 `unwrap/expect`、`crate::ui` 或 `zircon_editor::ui` 命中。

该完成声明仅覆盖切片 1.1。旧 `asset_editors` / `asset_creation_templates` API、裸 authoring-type
string、Asset Browser/preview 硬编码与 `.editor.meta.toml` 仍由 M1.2-M1.4 继续硬切，因此 Editor09 M1
和线程 Goal 均未完成。

## 产出记录与时间

| 里程碑 | 切片 | 状态 | 完成日期 | 完成项目与证据 |
|---|---|---|---|---|
| M1 | 1.1 AssetTypeId 与 materialized registry 核心 | `COMPLETED` | 2026-07-13 | folder-backed 10 文件 owner；开放 typed/serde `AssetTypeId`；26 个 runtime kind 唯一 builtin definitions；atomic contribution merge 与 typed conflict/incomplete errors；Windows lib-test 编译成功；focused 9/9 通过；scoped rustfmt、无生产 unwrap/expect、无 core→ui 反向依赖。 |
