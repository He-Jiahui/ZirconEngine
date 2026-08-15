---
handoff_kind: failure
status: open
created_at: 2026-08-05
summary_slug: settings-page-localization-contract-hardcut
origin_plan: docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
fixing_plan: docs/plans/zircon_editor/editor/06-ui-extension-framework.md
origin_child_dir: docs/plans/zircon_editor/editor/17
fixing_child_dir: docs/plans/zircon_editor/editor/06
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/settings/page.rs
  - zircon_editor/src/core/editor_extension.rs
  - zircon_editor/src/core/extension/store/batch.rs
  - zircon_editor/src/core/plugin/materializer.rs
  - zircon_runtime_interface/src/plugin/editor_contribution.rs
  - zircon_plugins/plugin_sdk/src/editor_contribution.rs
tests:
  - cargo test -p zircon_editor --lib settings_page --locked --jobs 1 -- --test-threads=1
  - plugin SettingsPage localized presentation materialization and revoke regression
---

# Editor06: Settings page localization contract requires a hard cut

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md`
- 来源执行切片：M3.3 i18n first settings consumer migration
- 修复责任计划：`docs/plans/zircon_editor/editor/06-ui-extension-framework.md`
- 交接原因：the `SettingsPageDescriptor` contribution contract and extension registry are Editor06's boundary. Editor12's serialized plugin materializer must follow that contract, but it cannot define a competing page presentation schema.

## 失败现象与复现证据

Editor17 hard-cut built-in `SettingDefinition` presentation to localization label, description, and category keys. The plugin settings-page path remains separate and incompatible: `core/settings/page.rs` stores literal `display_name: String` and slash-separated `category_path: String`; `core/editor_extension.rs` and `core/extension/store/batch.rs` validate that literal path; `core/plugin/materializer.rs` constructs it directly from `SerializedEditorContribution::SettingsPage { display_name, category_path }`.

This means a plugin page cannot project through the active locale without a retained-host string override, and the extension registry would sort or identify categories by translated presentation strings. The current DTO has no locale-neutral page label key, description key, structured category key path, or plugin translation-bundle identity.

## 最低共享层根因

The Editor06 contribution descriptor predates the Editor17 localization boundary and carries display data as authority. The string DTO is consumed by both in-process extension registration and Editor12 cdylib materialization, so changing only Editor17 settings definitions would retain a second settings-page presentation contract.

## 架构修复验收

- Replace `SettingsPageDescriptor` literal display/category fields with validated locale-neutral label, description, and structured category localization keys. Category ordering, deduplication, and contribution identity must use canonical keys, never rendered text.
- Upgrade the serialized Editor12 `SettingsPage` contribution DTO, materializer, plugin SDK builder, and plugin bundle contract to provide those keys from the plugin's registered localization bundle.
- Provide one locale-bound settings-page projection at the Editor06 settings UI boundary. It captures a locale once, resolves all page texts from `EditorI18nService`, and invalidates after a locale transition without mutating contribution state or cloning an unbounded page registry.
- Preserve ticket/revoke semantics, plugin identity, page id, and contribution validation. A missing plugin translation must display its raw key through the canonical i18n fallback, not an English literal or a second cache.
- Add in-process and serialized plugin regressions for en/zh-CN projection, locale change, category order independent of translated collation, duplicate validation, and revoke after a localized projection is cached.
- Re-run Editor17 M3.3 settings-consumer acceptance, then return this artifact through the coordinator lifecycle key.

## 禁止临时方案

- Do not add optional legacy `display_name`/slash `category_path` fields, retained-host translation overrides, or per-plugin presentation registries.
- Do not translate category strings before registry ordering/deduplication or use locale text as a contribution id.
- Do not modify Editor17 built-in settings definitions to accept plugin page literals.
- Do not alter runtime setting persistence or plugin revoke behavior to mask a page-presentation migration failure.

## 修复结果与回传

Open state: `Built-in setting presentation is locale-neutral, but plugin SettingsPage contributions remain literal-string DTOs owned by Editor06/Editor12. No localized plugin page behavior or validation result is claimed.`

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-08-05 | Editor17 M3.3 -> Editor06 settings-page contract handoff | `open / forward_repair_required` | Routed the remaining plugin settings-page localization boundary to its extension contract owner. Built-in settings hard cut remains forward-only; no retained-host fallback, DTO compatibility field, or plugin source mutation was introduced. |
