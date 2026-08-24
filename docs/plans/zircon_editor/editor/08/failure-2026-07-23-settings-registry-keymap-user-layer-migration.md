---
handoff_kind: failure
status: open
created_at: 2026-07-23
summary_slug: settings-registry-keymap-user-layer-migration
origin_plan: docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
origin_workflow_node: M1.1
fixing_plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
origin_child_dir: docs/plans/zircon_editor/editor/17
fixing_child_dir: docs/plans/zircon_editor/editor/08
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/commands/keymap.rs
  - zircon_editor/src/core/commands/keymap/tests.rs
  - zircon_editor/src/core/settings/defaults.rs
  - zircon_editor/src/core/settings/io.rs
  - zircon_editor/src/core/settings/keymap_overrides.rs
  - zircon_editor/src/core/settings/tests/persistence.rs
  - zircon_editor/src/tests/editor_event/runtime/keymap_settings.rs
  - zircon_editor/src/ui/host/editor_manager.rs
tests:
  - user keymap override resolve through SettingsRegistry
  - SettingsStore current-shell save/load for keymap overrides
  - keymap conflict detection after applying an override
---

# Editor08: keymap 用户层尚未迁入 SettingsRegistry

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md`
- 来源执行切片：Editor17 M1.1 keymap User layer migration
- 修复责任计划：`docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md`
- 交接原因：Editor08 拥有 keymap preset、typed override 合并与冲突检测，Editor17 只提供共享 Settings User 层。

## 失败现象与复现证据

Editor17 M1.1 已建立三层 `SettingsRegistry`、User `SettingsStore` 和当前版本文本壳，并删除旧 appearance preferences。Editor08 的 `EditorKeymap` 仍在 `keymap.rs:21-24` 自有 `user_layer`，且 `keymap/persistence.rs` 仍公开 `apply_user_layer`、`serialize_user_layer`、`save_user_layer_to_path`、`load_user_layer_from_path`，直接维护另一份 versioned 文件和迁移链。

这与 Editor17 计划的“08 keymap 用户覆盖（User）首批迁入”冲突：同一用户偏好不能同时有 keymap 私有落盘和 settings User 层两个权威。

## 最低共享层根因

keymap 私有 versioned 文件与 SettingsRegistry User 层同时持有用户覆盖，形成两份迁移链和持久化权威。

## 架构修复验收

- Editor08 将 keymap delta 以强类型 Settings value/schema 登记为 User 项；不得用 JSON/TOML 字符串作为绕过 schema 的载荷。
- 启动与保存只经 `SettingsRegistry` + `SettingsStore` 的 User 层；`EditorKeymap` 保留内建 preset 和纯合并/冲突检测，不再拥有文件 I/O、VersionedSchema 或迁移链。
- 删除旧 `keymap/persistence.rs` 读写 API 和所有消费点，不保留双写、回退读取或旧文件导入。
- 覆盖 User/Session 覆盖解析、冲突检测、当前版本壳 round-trip 与旧 keymap 文件拒绝。

## 禁止临时方案

- 不得在设置保存后再次写 keymap 私有文件。
- 不得把 keymap TOML 原文塞进 `SettingValue::String`，也不得为旧路径增加 compatibility shim。

## 修复结果与回传

Open state: `keymap SettingsRegistry hard cut 已静态落地；current-source Cargo、独立复核、fixed return 与受管提交尚未完成`。

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-07-23 | Editor17 M1.1 -> Editor08 keymap migration handoff | open | 实查 `EditorKeymap.user_layer` 与 `keymap/persistence.rs` 的独立 current-shell 文件读写仍存在；新 Settings User 层已成为唯一迁移目标，等待 Editor08 硬切消费。 |
| 2026-07-27 | Editor08 M2.2 keymap SettingsRegistry hard cut + PERF-MVP-074 source | source prepared / static green / immutable validation pending | 当前 `EditorKeymap` 只保留内建 preset、typed override 合并、冲突检测与 generation-time `signature_index`；`EditorKeymapOverrides` 已作为 `SettingValue::KeymapOverrides` 的 User schema 值由 `EditorManager` 启动时解析，私有 `apply_user_layer`、`serialize_user_layer`、私有文件读写与旧 document 标识扫描均为 0。新增 User/Session 覆盖、冲突顺序、current shell/legacy 拒绝、10k bindings、dead/unidentified/modifier/F5 与 1,000,000 event storm 回归；`rustfmt --check`、scoped diff check、借用签名静态合同通过。旧 snapshot `1122` 在后续测试补齐后已过期，当前 snapshot/source-copy 创建受 Coordinator01 external-source 与前台超时阻断，未运行 Cargo、未独立复核、未 fixed return 或提交。 |
| 2026-08-02 | stale private-shell receipt cleanup | static owner review passed / managed Cargo pending | 当前源码复核确认 private persistence owner 已删除，`EDITOR_KEYMAP_OVERRIDES_KEY`、typed tombstone 与 strict `zircon.editor.settings` 壳均由 `core/settings` 持有。失效 Python contract 实跑出现 2 个 missing-file error 与 1 个旧测试名 failure，已连同仅被其引用的 v0 fixture 删除；Editor11 child record 与模块文档已改到现行 owner。failure 继续 open：本次未运行 managed Cargo，不据静态证据执行 fixed return。 |
| 2026-08-24 | current owner-path refresh and managed preflight | static contract current / Cargo not started | Current HEAD retains typed User/Session overrides, conflict detection, current settings-shell round-trip, legacy private-document rejection, `EditorManager` consumption and event-runtime coverage. The stale deleted `core/settings/tests.rs` entry was replaced by the current split persistence test, production manager and event consumer paths. Two official `validate-matrix.ps1 -Package zircon_editor -SkipBuild -LibTests -TestFilter keymap` attempts were rejected before job creation with `cargo_reuse_pool_busy`: first by terminal Frameworks01 job `6a7c0627725544fba312d0a39498e233`, then by running job `246fdaf5d6c443f9b71149d744b5675e`. No Cargo result or fixed return is claimed; the failure remains open for the next legal FIFO window. |
