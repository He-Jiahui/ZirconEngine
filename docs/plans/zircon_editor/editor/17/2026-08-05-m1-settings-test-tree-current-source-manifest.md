Plan: docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md
Milestone: M1
Status: pending
Files: ["docs/plans/zircon_editor/editor/17-editor-services-and-recovery.md", "docs/plans/zircon_editor/editor/17/2026-08-05-m1-settings-test-tree-current-source-manifest.md", "docs/plans/zircon_editor/editor/17/failure-2026-07-30-editor-settings-persistence-and-hot-projection.md", "docs/crates/zircon_editor/core/settings.md", "zircon_editor/assets/i18n/en.toml", "zircon_editor/assets/i18n/zh-CN.toml", "zircon_editor/src/core/jobs/quota_settings.rs", "zircon_editor/src/core/jobs/tests/quota_settings_contract.rs", "zircon_editor/src/core/settings/authority.rs", "zircon_editor/src/core/settings/defaults.rs", "zircon_editor/src/core/settings/definition.rs", "zircon_editor/src/core/settings/mod.rs", "zircon_editor/src/core/settings/registry.rs", "zircon_editor/src/core/settings/snapshot.rs", "zircon_editor/src/core/settings/tests.rs", "zircon_editor/src/core/settings/tests/mod.rs", "zircon_editor/src/core/settings/tests/persistence.rs", "zircon_editor/src/core/settings/tests/registry.rs", "zircon_editor/src/core/settings/tests/registry/persistence_service.rs"]
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

No accepted output: M1 remains pending current-source managed validation and the open settings failure return. The manifest now has 19 paths and requires independent review plus a fresh coordinator-bound immutable snapshot.

## 2026-09-27 bounded persistence-service test owner

The current `tests/registry.rs` had grown to 1,015 lines. Seven contiguous persistence-service regressions now live in `tests/registry/persistence_service.rs` (266 lines), while the existing registry module remains at 752 lines with 21 tests. The existing `tests/mod.rs` mount and imports were preserved, including foreign changes. Both `include_str!` source guards remain in their original registry file, and the one previously ignored performance test remains ignored.

The 28 current test entries and all their bodies were preserved. Unmoved source bytes match the pinned `bc02eefafead65dbf5050482110e8175250a5e77` preimage; the moved block differs only by its module import and removal of the final blank line required by rustfmt. `rustfmt +1.94.1 --check` passed for both owned Rust files, and `python -X utf8 -m unittest tools.tests.test_editor17_settings_owner_modules_contract -v` passed 5/5 existing source-contract guards. These are structural results; the Rust behavior tests have not executed for this split.

The bounded continuation owns these two Rust paths, the existing shared test parent and this lifecycle's two records. The historical 30-test/35-function evidence above is retained. The wider M1/M3 candidate remains pending a fresh union snapshot and its required Cargo, scale, F0 and F4 acceptance. No accepted milestone, failure return or closeout is claimed here.

The frozen baseline's shared `tests/mod.rs` lacked the imports of `SettingValueSource`, `SettingColorChannel` and `SettingNumericStepDirection`, although its registry/value-batch/persistence children already used these existing types. Because Cargo compiles all lib-test modules before applying the run filter, the original four-path snapshot 4595 did not include a complete compile-support closure. The existing working-tree repair adds only those three imports: parent SHA256 `db9d9b2c4e547e3bd1cb2f02ece9d0565c3d3984f79094626b5076ca723f7348`, baseline SHA256 `2baa88e4bd7f8d94d00587fda7547d1446c21233e3285c2fa34db5103bd1b182`; all fixtures and test bodies remain unchanged. The coordinator found its prior owner cancelled with no live lease or active scope overlap, then transferred this exact file to the same Editor17 Session under fingerprint `5d7e9fce8769dff1165cccaa580c18a8daab6d4edec9daadd8a391bb1be2d2d8` (apply request `7086dfbd6622479c8b18ba6099f5f77a`). No additional shared-parent source edit was performed. The next managed source manifest must include all three Rust files; snapshot 4595 alone is not compile acceptance.

### Snapshot 4597 managed admission result

The five owned paths were frozen as snapshot 4597 at baseline epoch 628 under stable Session `failure-roll-01a0df1a-editor17-settings-test-budget-r1`; the three Rust files were the validation source manifest. Independent review found Critical/Important/Moderate = 0/0/0 for those exact bytes and this bounded support closure.

The single custom request `failure-roll-01a0df1a-editor17-settings-4597-20260927-r1` (coordinator journal UUID `e3f31ee4f2854ed8922a0d247206f537`, terminal `failed` at `2026-09-27T14:20:41.188169+00:00`) submitted `cargo +1.94.1 test -p zircon_editor --lib --locked -- core::settings:: --nocapture --test-threads=1`, expecting 52 tests including one existing ignored case. Admission was rejected with `validation_ticket_external_worktree_changed` at `file_verify` for external `zr_vm` commit `4a52715c676d6434d4364a25f911bbd8edddea54`. Its `docs/code-review/comment-standard.md` read 6,976 bytes; expected SHA256 `945e314be710f63edcd02486d55b8ed0f6b2e028afc77dd0d7c41299d1e53c65`, actual SHA256 `13e167aeee70da68671937a225f73636e09282a2e274bb350429ae908f593288`.

No validation ticket or Rust execution was produced. The unchanged three-source snapshot remains held for a fresh managed capture after the external owner reconciles the actual drift. This rejection does not establish compile, behavioral, scale or product acceptance, and no return or closeout was performed.
