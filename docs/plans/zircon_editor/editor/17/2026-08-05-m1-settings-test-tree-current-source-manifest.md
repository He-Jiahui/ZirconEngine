Plan: docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
Milestone: M1
Status: pending
Files: ["docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md", "docs/plans/zircon_editor/editor/17/2026-08-05-m1-settings-test-tree-current-source-manifest.md", "docs/plans/zircon_editor/editor/17/failure-2026-07-30-editor-settings-persistence-and-hot-projection.md", "docs/zircon_editor/core/settings.md", "zircon_editor/assets/i18n/en.toml", "zircon_editor/assets/i18n/zh-CN.toml", "zircon_editor/src/core/jobs/quota_settings.rs", "zircon_editor/src/core/jobs/tests/quota_settings_contract.rs", "zircon_editor/src/core/settings/authority.rs", "zircon_editor/src/core/settings/defaults.rs", "zircon_editor/src/core/settings/definition.rs", "zircon_editor/src/core/settings/mod.rs", "zircon_editor/src/core/settings/registry.rs", "zircon_editor/src/core/settings/snapshot.rs", "zircon_editor/src/core/settings/tests.rs", "zircon_editor/src/core/settings/tests/mod.rs", "zircon_editor/src/core/settings/tests/persistence.rs", "zircon_editor/src/core/settings/tests/registry.rs"]
---

# Editor17 M1 Settings Current-Source Manifest

## Scope Delivered

The settings test owner is folder-backed. Shared fixtures live in `tests/mod.rs`; authority and change-log coverage lives in `tests/registry.rs`; typed payload and store coverage lives in `tests/persistence.rs`. The legacy `tests.rs` owner is deleted without a compatibility module.

This candidate also hard-cuts `SettingDefinition.category_path: String`. Definitions now keep private `SettingsPresentation` identities: label, description, and non-empty category keys are all validated `settings.*` localization keys, never localized display text or slash-separated paths. The seven default settings and four job quota registrations use that one contract, and both embedded bundles declare every required key. The former 865-line settings registry is now three named owners: registry owns definitions/layers/precedence/change log, snapshot owns built-in typed slots and immutable projection, and authority owns publication/subscriber/project-layer lifecycle.

## Current-Source Evidence

- The original test suite retains 30 `#[test]` cases and 35 functions; normalized function-body comparison found no behavioral changes in the 30 test bodies.
- The settings presentation regression registers all eleven built-ins and verifies every label, description, and category key is directly present in every embedded locale bundle, rather than accepting an English fallback. Invalid schemas are rejected through the public `SettingDefinition::new` constructor; test code no longer constructs a private invalid definition.
- Scoped `rustfmt --check`, scoped `git diff --check`, TOML parsing, and the 31-key en/zh-CN coverage guard passed before managed validation submission.
- The subsequent owner hard cut is covered by explicit owner/entry-point guards; the combined Editor17 static suite is 17/17 green, with `py_compile` and scoped `rustfmt --check` also green. This source changed after the earlier validation submission attempt, so the coordinator must create a new immutable M1/M3 union snapshot.
- No local Cargo command was run. This pending manifest binds the coordinator-owned Windows package validation snapshot.
- `en.toml` and `zh-CN.toml` are shared with the pending M3 notification candidate. The coordinator must materialize a fresh M1/M3 union snapshot; this record must not be treated as an independently accepted immutable manifest.

## Review

The earlier independent second review found and then verified the repair for the moved `SettingsPersistenceSubmitError` import. The latest re-review found a stale private-field test, missing source files in this manifest, fallback-only locale assertions, and the old parent-plan contract. Those four findings are forward-fixed in this candidate. The later registry/snapshot/authority hard cut is included in the active independent second review and must not inherit the earlier green verdict.

## 产出记录与时间

No accepted output: M1 remains pending current-source managed validation and the open settings failure return. The manifest now has 18 paths and requires independent review plus a fresh coordinator-bound immutable snapshot.
