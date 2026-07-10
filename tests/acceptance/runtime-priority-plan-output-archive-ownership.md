# Runtime priority-plan output-archive ownership acceptance

## Scope

This slice covers only the output-record ownership of `engine-code-structure-convention.md` and `engine-code-review-findings-2026-06.md`, plus the Runtime 15 static guard that reads their numbered archives. It does not change runtime, editor, render, plugin, UI, text, or asset production behavior.

## Baseline problem

- Both priority overview documents still duplicated five Runtime 15 M3 review-guard cross-document evidence paragraphs.
- The repository output-record audit therefore reported ten forbidden concrete signatures across those duplicated blocks.
- The Runtime 15 structure-convention guard read those anchors directly from the overview documents, preventing a clean hard cut to numbered archives.

## Invariants

- Priority overview documents retain rules, findings, current summaries, and routing links only.
- Concrete Runtime 15 review-guard evidence belongs to the matching numbered archive under `docs/plans/zircon_runtime/runtime/15/`.
- Static guards read each overview together with its matching archive; the review archive cannot satisfy a structure-document check, and vice versa.
- The separate Runtime text status paragraph remains untouched while its active owner session is in progress.

## Evidence

- Repository output-record audit for the two priority documents: reduced from eleven violations to zero. This session removed ten duplicated Runtime 15 records; the active Runtime text owner converged the final independent paragraph before final validation.
- Standalone structure-convention harness compiled successfully; existing broad warning noise remains non-blocking.
- Focused `runtime_15_review_guard_row_data` suite: passed 73/73 after the archive-reader cutover.
- Broad `runtime_15_priority_plan_docs` suite: moved from 6/25 to 25/25 after current-path reconciliation, real frontmatter test inventory restoration, and dedicated current-owner archive routing.
- Both priority document frontmatters now resolve every `related_code`/`implementation_files` path and list all fourteen folder-backed priority-plan guard functions under `tests:`.
- The removed overview anchors were copied verbatim into the matching numbered output archives before deletion.
- Package/workspace Cargo was not used as completion evidence because active out-of-scope work still owns the shared lib-test baseline.

## Decision

The priority-plan concrete-record migration is accepted as a Runtime 15 static architecture-maintenance slice. Runtime 15 remains `in_progress`; no production or package gate is promoted by this acceptance.
