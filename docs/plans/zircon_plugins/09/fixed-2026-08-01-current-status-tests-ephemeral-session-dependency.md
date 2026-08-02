---
handoff_kind: fixed
status: fixed
created_at: 2026-08-01
resolved_at: 2026-08-01
summary_slug: current-status-tests-ephemeral-session-dependency
origin_plan: docs/plans/zircon_plugins/09-export-publishing.md
fixing_plan: docs/plans/zircon_plugins/09-export-publishing.md
origin_child_dir: docs/plans/zircon_plugins/09
fixing_child_dir: docs/plans/zircon_plugins/09
plan_link_mode: child_record_only
failure_scope: local
related_code:
  - tools/tests/plugin_status_document.py
  - tools/tests/plugin_docs_current_status_source_template_compile_host_support.py
  - tools/tests/test_plugin_docs_current_status_source_template_compile_host_owner_splits.py
  - tools/tests/test_tracked_tests_do_not_depend_on_codex_sessions.py
tests:
  - python -m unittest tools.tests.test_plugin_docs_current_status_source_template_compile_host_owner_splits
  - python -m unittest discover -s tools/tests -p "test_plugin_docs_current_status*.py"
  - python tools/tests/test_tracked_tests_do_not_depend_on_codex_sessions.py
---

# Plugins09: current-status tests depend on an ephemeral session note

## 来源执行者

- 来源计划：`docs/plans/zircon_plugins/09-export-publishing.md`
- 来源执行切片：2026-08-01 current plan/code/test review
- 修复责任计划：`docs/plans/zircon_plugins/09-export-publishing.md`
- 交接原因：Plugins09 owns export/plugin current-status evidence and its Python guard family. This is a local test-authority failure and must not be transferred to session-coordinator runtime ownership.

## 失败现象与复现证据

The tracked test suite references `.codex/sessions/20260628-0317-zui-migration-validation.md`, but that file is not tracked and is absent from the current workspace. The selected SourceTemplate suite therefore raised eight `FileNotFoundError` exceptions before checking a product or documentation contract.

The first repair slices removed that dependency from four SourceTemplate test owners, one shared helper and the Hub owner-split test. A complete inventory then found and migrated all remaining current-status and ZUI guard owners. The tracked test tree now contains zero `.codex/sessions/` inputs.

## 最低共享层根因

Historical closeout tests treated a mutable, ignored session note as canonical repository input and required the same receipt phrases to be duplicated across plans, public docs, review notes and that session. Numbered plan output archives are now the durable evidence owner and `StatusDocumentPath` already expands those tracked archives. The legacy session dependency is therefore both non-reproducible and a second evidence authority.

## 架构修复验收

- Remove every test/helper read of `.codex/sessions/20260628-0317-zui-migration-validation.md`; do not replace it with another session path.
- Keep current contract assertions on tracked source, public docs and the owning numbered plan. Historical acceptance receipts may be read through numbered plan output archives, not copied into every current document.
- Add a repository guard that rejects `.codex/sessions/` dependencies under `tools/tests`.
- Run the full `test_plugin_docs_current_status*.py` family and the focused owner-boundary suites from a checkout that contains no ignored session note.
- Return this record as fixed only after every tracked owner has been migrated and the full tracked suite is green.

## 禁止临时方案

- Do not recreate or commit the missing historical session note merely to satisfy tests.
- Do not make `StatusDocumentPath` return empty text or redirect missing session paths to unrelated documents.
- Do not weaken production/source contract tests or claim archived receipt text is current implementation evidence.
- Do not close this record based only on the repaired SourceTemplate subset.

## 修复结果与回传

- 根因：tracked current-status guards mixed durable plan/public-document contracts with an ignored session-note receipt, making a clean checkout non-reproducible.
- 架构修复：removed every session-note input and every `active session ...` requirement while retaining assertions over the owning plan, public contract docs and numbered output records. Retired the one workspace test that asserted exact 2026-07-04 target-dir/timestamps across unrelated module docs. Added a repository guard against future `.codex/sessions/` dependencies under `tools/tests`.
- 验证：SourceTemplate 12/12; ZUI document guards 12/12; full `test_plugin_docs_current_status*.py` suite 327/327; session-dependency guard 1/1; `git grep -F '.codex/sessions/' -- tools/tests` returned no matches.
- 回传：this local Plugins09 test-authority failure is fixed. Historical execution evidence remains in numbered plan output records rather than ephemeral session files.
