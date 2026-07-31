# Maintain Docs With Code Changes

## Progressive Disclosure Index

- Read this file when code already changed or is about to change and you need to keep documentation synchronized.
- Read `../../milestone-first-workflow-policy.md` first for source-path mirroring and testing-stage rules.
- If you need to write a new detailed document, also read `../write-module-docs/SKILL.md`.
- If you need the required metadata format, go to `../required-doc-header-format.md`.

## Workflow

1. Decide whether documentation is necessary.
   - Start from the changed public contract, cross-module boundary, operator workflow, or durable design decision—not from every changed file.
   - Find an existing document only if it claims ownership of that fact.
   - If source comments, tests, and the numbered plan already communicate the change, do not create a document. A plan status row is sufficient for completion evidence; it is not a reason to create explanatory prose.

2. Keep the smallest truthful owner.
   - Update an existing document when its public fact would otherwise become stale.
   - Add a document only for a missing durable interface/workflow decision.
   - Do not split, create an overview, or add a category document merely to mirror source-file churn.

3. Synchronize only the durable fact.
   - On a retained document, update the compact owner header and the changed contract/decision.
   - Mention one relevant milestone or validation suite only when it helps locate the fact.
   - Remove stale statements instead of piling new notes on top of invalid documentation.

4. Refuse false documentation, not missing prose.
   - Do not call code work complete while a retained document describes an old public fact.
   - Do not require new documents for private implementation detail.
   - Do not maintain per-file test inventories; the source test and milestone evidence remain authoritative.

## Reporting

Report this decision once in the task handoff or terminal response: retained document path (if any) and the durable fact corrected. Do not create a separate documentation report.
