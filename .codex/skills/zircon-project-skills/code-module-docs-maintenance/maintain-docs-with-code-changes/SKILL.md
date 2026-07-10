# Maintain Docs With Code Changes

## Progressive Disclosure Index

- Read this file when code already changed or is about to change and you need to keep documentation synchronized.
- Read `../../milestone-first-workflow-policy.md` first for source-path mirroring and testing-stage rules.
- If you need to write a new detailed document, also read `../write-module-docs/SKILL.md`.
- If you need the required metadata format, go to `../required-doc-header-format.md`.

## Workflow

1. Identify impacted docs from code files.
   - Start from the changed code files.
   - Find existing docs whose `related_code` or `implementation_files` headers mention those paths.
   - If no document exists for an important module, create one under the source-path mirror in `docs/` unless an existing functional category document is the stronger owner.

2. Decide whether to update, split, or add docs.
   - Update an existing leaf document when behavior changed but the ownership is the same.
   - Split a document when it has become too broad for one topic.
   - Update the category overview when a new child document appears or the category scope changes.

3. Synchronize all affected sections.
   - Update the machine-readable header.
   - Update behavior descriptions, design rationale, test coverage, and plan-source references.
   - Mark tests as planned for the milestone testing stage, run, failed/fixed, or deferred. Do not imply full unit-test evidence before the testing stage ran.
   - Remove stale statements instead of piling new notes on top of invalid documentation.

4. Refuse incomplete completion.
   - Do not call the code work complete if the docs still describe old behavior.
   - Do not leave new code files undocumented when they introduce meaningful new behavior.
   - Do not leave the test section stale after adding, deleting, or renaming tests.

## Reporting

- State which code files triggered the doc update.
- State which `docs/` files were updated or created.
- State which plan source and test sections changed.
- State whether any existing document had to be split or re-categorized.
