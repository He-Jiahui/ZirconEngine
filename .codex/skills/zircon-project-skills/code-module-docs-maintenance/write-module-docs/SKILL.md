# Write Module Docs

## Progressive Disclosure Index

- Read this file when creating or expanding documentation for a code module or subsystem.
- Read `../../milestone-first-workflow-policy.md` first for path-mirroring and testing-stage expectations.
- If you still need the required metadata header, go to `../required-doc-header-format.md`.
- If you still need to choose where the document lives, go to `../plan-docs-directory-first.md`.
- If the code changed and existing docs may be stale, also read `../maintain-docs-with-code-changes/SKILL.md`.

## Writing Rules

1. Write only durable facts.
   - Create this document only for a public/cross-module interface, operator workflow, or non-obvious durable decision that source comments and the numbered plan cannot carry.
   - Use a compact machine-readable header naming the owner modules and one relevant validation suite; do not list every implementation or test file.

2. Explain intent before mechanics.
   - State what problem the module solves.
   - State why this design exists in the repository.
   - State which layers depend on it.

3. Describe only what readers cannot safely infer from source.
   - Record the public contract, cross-module ownership, operator action, or invariant that motivated the document.
   - Point to the owning source and tests instead of duplicating inputs, outputs, control flow, or per-slice implementation detail.

4. Explain design choices only when they constrain future changes.
   - Record a tradeoff, rejected alternative, compatibility limit, or performance/correctness boundary only if removing it would invite a wrong implementation.

5. Tie it back minimally.
   - Link one owning plan/milestone and the relevant test suite when needed for navigation.
   - Do not duplicate command logs, granular progress, or a second status record.

## Recommended Body Structure

- `# <Module or Feature Name>`
- `## Contract or Decision`
- `## Ownership and Constraints`
- `## Relevant Source and Validation`

## Quality Bar

- Prefer a short, current fact over a broad explanation that will drift.
- Keep overview files navigational and leaf documents limited to the durable decision they own.
- Delete or tighten a document when it duplicates source rather than making a future change safer.
