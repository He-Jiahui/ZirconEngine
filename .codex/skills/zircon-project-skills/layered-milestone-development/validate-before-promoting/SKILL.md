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

2. Build a risk-based coverage checklist from source material.
   - Use the relevant spec, implementation files, and existing tests to identify changed behavior, public contracts, and failure boundaries.
   - Cover every changed contract and high-risk boundary; do not expand a focused milestone into an exhaustive unrelated regression sweep.

3. Validate from lower to upper layers.
   - Run or add focused tests for the lowest in-scope shared behavior first.
   - Then run parent-layer tests.
   - Then run integration or end-to-end checks that exercise the same path through normal entrypoints.

4. Include edge and failure coverage where the change reaches them.
   - Cover boundary values, invalid inputs, unsupported combinations, null or empty inputs, range edges, and type mismatches only when they are part of the changed contract or a known regression boundary.
   - When several source forms converge on the changed behavior, use representative coverage plus targeted variants where the implementation differs.

5. Refuse false completion.
   - A milestone stays open if any changed contract or high-risk boundary lacks direct evidence.
   - A milestone stays open if tests only prove a happy path while changed failure behavior remains untested.
   - A milestone stays open if the upper layer passes only because of a special-case path that bypasses broken shared behavior.
   - A milestone stays open if the plan lacks a testing-stage record of compile/build commands, unit tests, failures fixed, and remaining accepted risk.

## Instruction-Generation Rule

- For instruction generation, enumerate the `zr` syntax forms whose lowering paths changed in this milestone. Reuse existing coverage for unchanged equivalent forms.
- Prove each changed instruction form is emitted by at least one targeted test.
- Prove changed boundary behavior with dedicated cases, not by inference from ordinary programs.
- Re-run the broader execution path after generator-level correctness is established.

## Reporting

- State the milestone being evaluated.
- State the full in-scope inventory that was checked.
- State which lower-layer tests, parent-layer tests, and integration tests were run.
- State any uncovered behavior that keeps the milestone open.
- State explicitly whether the milestone advanced or was forced back down to a lower layer.
