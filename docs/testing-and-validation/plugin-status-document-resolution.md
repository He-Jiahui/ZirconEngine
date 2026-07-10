---
related_code:
  - tools/tests/plugin_status_document.py
  - tools/tests/test_plugin_status_document.py
  - tools/tests/test_plugin_docs_current_status*.py
  - docs/plans/zircon_plugins/07-net.md
  - docs/plans/zircon_plugins/09-export-publishing.md
  - docs/plans/zircon_plugins/10-editor-integration.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
implementation_files:
  - tools/tests/plugin_status_document.py
  - tools/tests/test_plugin_docs_current_status*.py
plan_sources:
  - user: 2026-07-10 implement the zircon_plugins plans and prioritize structure/review findings
  - docs/plans/zircon_plugins/12-plugin-dx-and-structure-framework.md
  - docs/plans/engine-code-structure-convention.md
  - docs/plans/engine-code-review-findings-2026-06.md
tests:
  - tools/tests/test_plugin_status_document.py
  - tools/tests/test_plugin_docs_current_status_convergence.py
  - python -m unittest discover -s tools/tests -p 'test_plugin_docs_current_status*.py'
doc_type: testing-guide
---

# Plugin Status Document Resolution

## Purpose

Plugin current-status guards must validate the canonical numbered output records without forcing concrete evidence back into overview files. `StatusDocumentPath` provides that test-only view: plan and authority documents stay concise and compliant, while status tests see the evidence owned by linked numbered archives.

## Behavior Model

`StatusDocumentPath.read_text()` behaves like `pathlib.Path.read_text()` for non-Markdown files. For Markdown, it recognizes only links whose parent directory is a two-digit child-plan number and whose filename starts with a date, such as `09/2026-07-09-export-publishing-output-records.md` or `runtime/15/2026-07-09-engine-code-structure-output-records.md`.

The archive text is inserted immediately after its link. This preserves section slicing: a guard that reads `## 状态与产出记录` up to the following milestone heading sees the canonical archive evidence before the end marker. Normal architecture, module, and reference links are not expanded.

## Current State Versus Historical Evidence

Required-evidence assertions use the expanded text. Assertions that forbid stale wording use `strip_resolved_output_archives()` so historical snapshots such as an early “rollout not complete” record do not masquerade as the current overview state. The archive remains intact and searchable; only the current-wording assertion excludes the injected historical block.

## Caching and Invalidation

Expanded documents are cached because hundreds of status guards repeatedly read the same large archives. The cache key includes the source document timestamp and size plus every resolved archive timestamp and size. Repeated reads reuse the same expanded snapshot, while a changed plan or archive automatically produces a fresh result.

## Plan Integrity

Archive migration moves concrete completion and validation records only. Milestone task tables, acceptance commands, risks, and reference-source maps remain in their numbered child plans. Plugins 07, 09, and 10 were restored to that shape after an earlier migration had removed their non-record plan sections together with the evidence rows.

## Test Coverage

- `test_plugin_status_document.py` covers in-place archive expansion, ordinary-link exclusion, cache reuse/invalidation, and historical-block stripping.
- The complete `test_plugin_docs_current_status*.py` suite covers 326 plugin status contracts and passes through the archive-aware path.
- Plan output layout remains independently checked by `audit_plan_output_records.py`; archive-aware reads do not weaken placement rules.

## Constraints

The resolver is test-only. Production tooling and documentation renderers continue to consume normal Markdown links. A status archive must use the numbered/date naming convention to be expanded; malformed or missing links remain visible as ordinary text and should be reported by the plan-output audit.
