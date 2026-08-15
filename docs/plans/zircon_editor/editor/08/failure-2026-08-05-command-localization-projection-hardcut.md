---
handoff_kind: failure
status: open
created_at: 2026-08-05
summary_slug: command-localization-projection-hardcut
origin_plan: docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
fixing_plan: docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md
origin_child_dir: docs/plans/zircon_editor/editor/17
fixing_child_dir: docs/plans/zircon_editor/editor/08
plan_link_mode: child_record_only
related_code:
  - zircon_editor/src/core/commands/descriptor.rs
  - zircon_editor/src/core/commands/defaults.rs
  - zircon_editor/src/core/commands/menu.rs
  - zircon_editor/src/core/commands/palette.rs
  - zircon_editor/src/core/commands/registry.rs
  - zircon_editor/src/ui/retained_host/app/command_palette_actions.rs
  - zircon_editor/assets/i18n/en.toml
  - zircon_editor/assets/i18n/zh-CN.toml
tests:
  - cargo test -p zircon_editor --lib core::commands --locked --jobs 1 -- --test-threads=1
  - localized command menu and palette current-source projection regression
---

# Editor08: Command localization projection hard cut is missing

## 来源执行者

- 来源计划：`docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md`
- 来源执行切片：M3.3 i18n catalog, locale change, and first command/settings/notification consumer migration
- 修复责任计划：`docs/plans/zircon_editor/editor/08-tool-orchestration-and-commands.md`
- 交接原因：translation catalog ownership remains Editor17, but command descriptor presentation, menu materialization, palette generation cache, and command UI projection are the single Editor08 command-system boundary. Editor17 must not introduce a parallel localized command registry or retained-host-only adapter.

## 失败现象与复现证据

`core/commands/defaults.rs` constructs built-in commands with English display names, descriptions, and slash-separated English `menu_path` values. `core/commands/menu.rs` derives visible menu labels by splitting those paths, while `core/commands/palette.rs` copies `descriptor.display_name()` into the generation-owned palette catalog. Consequently changing `EditorI18nService` locale cannot change either a command menu or palette row without reconstructing a second presentation truth.

Current repository evidence is direct and reproducible:

- `git grep -n 'EditorCommandDescriptor::operation\|EditorCommandDescriptor::new' -- zircon_editor/src` reports 59 command descriptor construction sites.
- `git grep -n 'command.file.open\|command.file.save' -- zircon_editor/src zircon_editor/assets` finds the two embedded bundle keys only in the i18n bundle and i18n/context tests, not in `core/commands` or its consumers.
- `EditorCommandPaletteEntry::from_descriptor` captures the literal display name before query; `menu_model` and `command_menu_item` receive literal top-level and leaf labels from `menu_path`.

The expected M3.3 behavior is that command labels and menu labels are first-class locale projections from the one `EditorI18nService`, with no English fallback strings acting as a second command presentation authority.

## 最低共享层根因

Editor08 currently mixes canonical executable command metadata with display strings, and caches that mixed representation in `EditorCommandPaletteCatalog`. A translation key cannot be inserted at a retained-host call site: menu and palette would still derive labels from different sources, locale change would leave a stale cache, and plugins would need an incompatible exception. The command registry must own locale-neutral presentation identities and emit locale-bound views only at the presentation boundary.

## 架构修复验收

- Hard-cut built-in `EditorCommandDescriptor` presentation from literal display/menu strings to validated localization keys. Keep executable id, action, `WhenClause`, capability, chord, and headless metadata locale-neutral.
- Replace slash-delimited display `menu_path` parsing with a structured menu presentation path whose segments are localization identities. Menu ordering/deduplication must use stable canonical segment identities, never translated text.
- Make the palette catalog cache locale-neutral command identities, enablement, and canonical search metadata. Materialize a bounded, immutable locale-specific palette/menu projection from the captured locale and the single `EditorI18nService`; locale transition must invalidate/rebuild the presentation projection without cloning or mutating the command registry.
- Define one plugin command presentation contract through the plugin localization bundle boundary. Plugins may not fall back to arbitrary host-English literals, a second registry, or retained-host-specific label overrides.
- Route retained-host command palette/menu consumers through the new registry projection. The active locale must update visible built-in command labels after a locale transition, while command ids, MRU identity, keyboard routing, operation factories, CLI names, and `WhenClause` evaluation remain unchanged.
- Add focused regressions for English and zh-CN menu/palette projections, menu segment ordering independent of translated collation, locale transition cache invalidation, missing translation raw-key fallback, and no full registry clone per query or locale switch.
- Re-run the Editor17 M3.3 i18n consumer acceptance after the Editor08 projection repair, then return this artifact through the coordinator lifecycle key.

## 禁止临时方案

- Do not translate only retained-host strings, parse translated labels back into command paths, or add a second localized command registry/cache.
- Do not retain English `display_name` or `menu_path` as built-in presentation fallback/compatibility fields.
- Do not make command execution, command id, CLI route, MRU identity, keymap conflict resolution, or `WhenClause` depend on the active locale.
- Do not rebuild a mutable all-command row vector on every palette query or mutate the canonical registry on locale change.

## 修复结果与回传

Open state: `Editor17 has a localized catalog and notification consumer, but Editor08 command presentation remains English-literal and locale-insensitive. No localized command behavior, Cargo result, or return is claimed.`

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-08-05 | Editor17 M3.3 -> Editor08 command presentation handoff | `open / forward_repair_required` | Proven command descriptor/menu/palette localization boundary routed to Editor08. Existing M3 catalog and notification code remains integrated; no rollback, retained-host bypass, compatibility field, or unverified localized command claim was added. |
