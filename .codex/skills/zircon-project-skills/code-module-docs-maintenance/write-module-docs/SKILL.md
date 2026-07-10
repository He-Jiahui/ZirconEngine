# Write Module Docs

## Progressive Disclosure Index

- Read this file when creating or expanding documentation for a code module or subsystem.
- Read `../../milestone-first-workflow-policy.md` first for path-mirroring and testing-stage expectations.
- If you still need the required metadata header, go to `../required-doc-header-format.md`.
- If you still need to choose where the document lives, go to `../plan-docs-directory-first.md`.
- If the code changed and existing docs may be stale, also read `../maintain-docs-with-code-changes/SKILL.md`.

## Writing Rules

1. Start from the machine-readable header.
   - Fill `related_code`, `implementation_files`, `plan_sources`, and `tests` before writing the body.
   - Treat missing metadata as an incomplete document.

2. Explain intent before mechanics.
   - State what problem the module solves.
   - State why this design exists in the repository.
   - State which layers depend on it.

3. Describe behavior in concrete terms.
   - Explain input forms, outputs, side effects, failure modes, invariants, and important control-flow or data-flow decisions.
   - When relevant, describe how source syntax maps into parser structures, instructions, runtime state, or external behavior.
   - Mention the key code comments or invariants that future unit-test/debug work should inspect first.

4. Explain design choices and tradeoffs.
   - Record why the current structure was chosen.
   - Record constraints, rejected alternatives, compatibility assumptions, and performance or correctness priorities when they matter.

5. Tie the document back to plan and tests.
   - State which milestone, plan, or request this implementation came from.
   - State which tests verify the behavior and whether they are planned for the milestone testing stage, already run, or deferred.
   - State which behaviors are intentionally out of scope.

## Recommended Body Structure

- `# <Module or Feature Name>`
- `## Purpose`
- `## Related Files`
- `## Behavior Model`
- `## Design and Rationale`
- `## Control Flow or Data Flow`
- `## Edge Cases and Constraints`
- `## Test Coverage`
- `## Plan Sources`
- `## Open Issues or Follow-up`

## Quality Bar

- Write enough detail that another engineer can understand the design without reverse-engineering the whole module from source.
- Prefer explicit descriptions over generic statements such as "handles parsing" or "manages runtime state".
- Keep overview files short and navigational; keep leaf documents detailed and implementation-aware.
