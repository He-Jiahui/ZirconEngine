# Validate Before Promoting

## Progressive Disclosure Index

- Read `../../milestone-first-workflow-policy.md` before selecting validation depth.
- Read this file when a milestone is about to be called complete.
- If you still need to define the milestone itself, go back to `../plan-by-layer/SKILL.md`.
- If the failure may come from a lower shared layer, apply `../../support-first-regression-testing/SKILL.md`.

## Validation Workflow

Only run this workflow in the milestone testing stage unless the user explicitly asks for earlier compile/test validation.

1. Re-state the milestone scope.
   - Name the exact syntax, instructions, runtime helpers, modules, or external behaviors that are supposed to be complete.

2. Build the coverage checklist from source material.
   - Use the relevant spec, implementation files, and existing tests to enumerate all in-scope forms.
   - Treat undocumented code paths and untested branches as uncovered until proven otherwise.

3. Validate from lower to upper layers.
   - Run or add focused tests for the lowest in-scope shared behavior first.
   - Then run parent-layer tests.
   - Then run integration or end-to-end checks that exercise the same path through normal entrypoints.

4. Include edge and failure coverage.
   - Boundary values, invalid syntax, unsupported combinations, null or empty inputs, range edges, and type mismatches all count when relevant to the milestone.
   - If the project accepts multiple source forms that should converge on the same behavior, test all of them.

5. Refuse false completion.
   - A milestone stays open if any in-scope behavior lacks direct evidence.
   - A milestone stays open if tests only prove one representative example.
   - A milestone stays open if the upper layer passes only because of a special-case path that bypasses broken shared behavior.
   - A milestone stays open if the plan lacks a testing-stage record of compile/build commands, unit tests, failures fixed, and remaining accepted risk.

## Instruction-Generation Rule

- For instruction generation, enumerate every supported `zr` syntax form that should lower into instructions.
- Prove each expected instruction form is emitted by at least one targeted test.
- Prove boundary behavior with dedicated cases, not by inference from ordinary programs.
- Re-run the broader execution path after generator-level correctness is established.

## Reporting

- State the milestone being evaluated.
- State the full in-scope inventory that was checked.
- State which lower-layer tests, parent-layer tests, and integration tests were run.
- State any uncovered behavior that keeps the milestone open.
- State explicitly whether the milestone advanced or was forced back down to a lower layer.
