---
handoff_kind: failure
status: open
created_at: 2026-08-08
summary_slug: project-document-settings-authority-legacy-entry
origin_plan: docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
fixing_plan: docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md
origin_child_dir: docs/plans/zircon_editor/editor/17
fixing_child_dir: docs/plans/zircon_editor/editor/10
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/ui/workbench/project/editor_project_document_load.rs
  - zircon_editor/src/ui/host/project_access.rs
  - zircon_editor/src/tests/workbench/project/document_roundtrip.rs
tests:
  - cargo test -p zircon_editor --lib tests::workbench::project::document_roundtrip --locked --jobs 1 -- --test-threads=1
  - project_document_load_uses_only_the_activated_settings_authority_in_production
---

# Editor10: project document retains a legacy settings-authority entry

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md`
- 来源执行切片：M1 SettingsAuthority current-source second review
- 修复责任计划：`docs/plans/zircon_editor/editor/10-project-and-asset-reference-management.md`
- 交接原因：`EditorProjectDocument` 的 project-open/document loading API 及其 production visibility 属于 Editor10 project/document authority。Editor17 只能提供唯一 settings authority contract，不能在 project document owner 内保留或替换路径。

## 失败现象与复现证据

`zircon_editor/src/ui/workbench/project/editor_project_document_load.rs` 的公开
`EditorProjectDocument::load_from_project(&ProjectManager)` 在函数体内构造
`SettingsAuthority::with_defaults()`，再直接加载 Project layer。当前 production open 路径已改为
`EditorUiHost::open_prepared_project -> EditorProjectDocument::load_from_activated_project(..., &self.settings)`，
而前者的所有当前调用者均在 `zircon_editor/src/tests/`。因此 production 图仍保留一个可重新启用的第二
settings authority API，违反 Editor17 M1 的 authority=1 和 hard-cut 要求，即使它暂未被 production caller 使用。

## 最低共享层根因

项目 document owner 将测试便利入口保留为公开 production API，而不是将测试 fixture 显式限定在
`cfg(test)`。这使 project-settings provenance 可以绕过 active `EditorContext` authority，且未来 call-site
能够在不触发类型错误的情况下恢复第二份 mutable truth。

## 架构修复验收

- production project document loading 只保留 `load_from_activated_project(project, project_info, settings)`，并接收
  active EditorContext 的同一 `SettingsAuthority`；不得在 document module 内构造 `SettingsAuthority`。
- 当前 roundtrip/integration fixtures 如仍需要 direct project loading，必须使用 `#[cfg(test)]`、
  `pub(crate)` 的测试 helper，且 helper 不得成为 host/runtime production 路由的替代入口。
- 增加 focused guard：非测试源中不存在 `EditorProjectDocument::load_from_project` 或
  `SettingsAuthority::with_defaults()` 的 document-load construction；真实 project open 仍从
  `EditorUiHost::open_prepared_project` 将 `self.settings` 传入 activated loader。
- 重跑 Editor10 project/document focused gate，并在 Editor17 M1 union validation 前确认 authority=1。

## 禁止临时方案

- 不得保留 deprecated/public alias、runtime feature fence、silent fallback authority 或双入口 load API。
- 不得把测试改为直接绕过 project/document assembly，或弱化 project-settings startup provenance。
- 不得在 Editor17/retained-host 建立第二个 adapter 来隐藏 document owner 的独立 authority。

## 修复结果与回传

2026-08-10 current-source forward repair:

- production `EditorProjectDocument` 已删除公开 `load_from_project` 入口，只保留 `load_from_activated_project(project, project_info, settings)`；`EditorUiHost::open_prepared_project` 继续注入 active `self.settings`。
- direct project document fixture 改为 `#[cfg(test)] pub(crate) load_from_project_for_tests`。所有 `src/tests` 调用者已显式迁移到该 helper，不保留 alias、deprecated wrapper 或 production feature fence。
- focused source guard 锁定旧 public signature 缺失、test helper cfg gate、activated loader 与 host authority injection；全 `zircon_editor/src` 旧 call-site 扫描为 0。
- `rustfmt --edition 2021 --check`、scoped `git diff --check` 与 source contract 通过；本轮未执行 Cargo 或 retained-host E2E。

Open state: `settings_authority_legacy_entry_source_hard_cut_complete_pending_managed_editor10_editor17_union_validation`; no validation pass is claimed.

## 产出记录与时间

| 日期 | 状态 | 完成项目与证据 |
| --- | --- | --- |
| 2026-08-08 | `open / routed_to_editor10` | Editor17 M1 independent review traced current production open through `EditorUiHost::open_prepared_project` to the injected active authority, then found the separate public `load_from_project` construction path. All current call-sites are test modules, proving a removable production-surface legacy entry rather than a required runtime route. No Editor10 source, tests, or validation state were modified by the originating session. |
| 2026-08-10 | `source_hard_cut_complete / validation_pending` | Removed the production legacy entry, moved direct loading behind the explicit cfg(test) helper, migrated all test callers, and added the authority-route source guard. No Cargo or product validation is claimed. |
