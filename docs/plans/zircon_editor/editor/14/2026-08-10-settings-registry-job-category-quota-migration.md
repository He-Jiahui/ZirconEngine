Plan: docs/plans/zircon_editor/editor/14-threading-and-job-scheduling.md
Milestone: quota-startup-migration
Status: implementation_complete_second_review_clean_managed_validation_pending
Files: ["docs/plans/zircon_editor/editor/14/2026-08-10-settings-registry-job-category-quota-migration.md", "docs/plans/zircon_editor/editor/14/failure-2026-07-23-settings-registry-job-category-quota-migration.md", "docs/zircon_editor/core/settings.md", "tools/tests/test_editor17_settings_owner_modules_contract.py", "zircon_editor/assets/i18n/en.toml", "zircon_editor/assets/i18n/zh-CN.toml", "zircon_editor/src/core/context/builder.rs", "zircon_editor/src/core/context/builder/quota_startup_tests.rs", "zircon_editor/src/core/jobs/limits.rs", "zircon_editor/src/core/jobs/quota_settings.rs", "zircon_editor/src/core/jobs/system/mod.rs", "zircon_editor/src/core/jobs/tests/quota_settings_contract.rs", "zircon_editor/src/core/settings/authority.rs", "zircon_editor/src/core/settings/defaults.rs", "zircon_editor/src/core/settings/definition.rs", "zircon_editor/src/core/settings/mod.rs", "zircon_editor/src/core/settings/registry.rs", "zircon_editor/src/core/settings/snapshot.rs", "zircon_editor/src/core/settings/startup.rs", "zircon_editor/src/core/settings/tests.rs", "zircon_editor/src/core/settings/tests/mod.rs", "zircon_editor/src/core/settings/tests/persistence.rs", "zircon_editor/src/core/settings/tests/registry.rs"]
Depends-On-Snapshots: ["1572"]
---

# Editor14 Job Quota Startup Migration

## Scope Delivered

The context composition root now owns the cross-module startup order: it creates one settings registry, registers Editor14 quota definitions, loads the User layer through generic `SettingsStartup`, resolves a complete immutable `EditorJobLimits` from the loaded registry and scheduler parallelism, moves that same registry into `SettingsAuthority`, and only then constructs `EditorJobSystem`.

The old implicit paths are hard-cut. `SettingsAuthority::at_startup` no longer hides load failures, `EditorJobSystem` no longer reapplies runtime defaults, and Play no longer reaches Export's fallback branch. User quota changes remain restart-only: the current admission owner is immutable while a successor context consumes the persisted value.

## Validation And Review

- Rust behavior coverage includes production registry wiring, typed `Loaded/Missing/Invalid`, invalid zero/negative/over-64/wrong-kind persisted values, runtime-derived categories, and Context A versus Context B admission behavior.
- `python -B -m unittest tools.tests.test_editor17_settings_owner_modules_contract -v` passes 5/5 after replacing its retired `SettingsAuthority::at_startup` assertion with the generic startup owner contract.
- First independent review: `Critical/Important/Minor = 0/2/0`. The ownership-cycle finding was repaired by moving quota registration/resolution out of settings and into the context composition root.
- First re-review: `Critical/Important/Minor = 0/1/2`. The manifest closure now explicitly lists the two localization sources, definition/registry changes, all seven split owner files, and the deleted monolithic test tombstone. The settings-owned presentation test no longer imports or duplicates the jobs-owned quota contract; the complete closure is being reformatted and rechecked.
- Closure-repair static evidence: settings owner Python contract passes 5/5; exact `rustfmt --check` passes for all 16 live Rust files; exact 23-path `git diff --check` passes with only line-ending notices.
- Final independent re-review: `Critical/Important/Minor = 0/0/0`. It confirms the 23-path candidate is self-contained, the generic settings owner has zero jobs dependency, persisted Invalid input cannot partially mutate the registry, and the context composition root is the only quota registration/resolution owner.
- Source-bound managed Cargo validation remains required before fixed return. No direct Cargo command was run.

## 产出记录与时间

| 日期 | 切片 | 状态 | 完成项目与验证证据 |
| --- | --- | --- | --- |
| 2026-08-10 | Editor14 quota startup migration | implementation_complete_second_review_clean_managed_validation_pending | One-registry startup, typed User load provenance, single resolved limits owner, restart-only admission, hard-cut legacy defaults, and focused behavior tests are complete. First review 0/2/0 closed the production ownership cycle; re-review 0/1/2 closed the exact snapshot closure and duplicate test ownership. The final 23-path candidate passes Python 5/5, exact rustfmt, exact diff-check, and independent review 0/0/0; only source-bound managed Cargo acceptance remains before fixed return. |
