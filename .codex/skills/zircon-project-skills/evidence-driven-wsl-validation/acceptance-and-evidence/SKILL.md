# Acceptance And Evidence

## Progressive Disclosure Index

- Read this file when preparing the full validation flow or deciding whether a change can be accepted.
- If you still need to choose tools, go to `../wsl-tool-selection/SKILL.md`.
- If you still need test placement rules, go to `../plan-tests-under-tests-directory.md`.

## Validation Workflow

1. Re-state the scope.
   - Name the changed feature, bug, or milestone.
   - Name the affected layers: parser, instruction generation, runtime execution, library support, CLI, test harness, or project fixtures.
   - When WSL is used, state the Linux-specific reason, compatibility key inputs, coordinator job, and granted mounted `/mnt/d`, `/mnt/e`, or `/mnt/f` Cargo target directory.

2. Establish the baseline.
- Record the pre-change failure, baseline passing set, and any known repository-level failures from `zircon-dev`.
   - Do not erase baseline context. Separate "already broken" from "introduced or fixed now" with evidence.

3. Build the complete test inventory.
   - List the direct unit or subsystem tests.
   - List the end-to-end or project-level tests.
   - List boundary cases.
   - List negative or failure-path cases.
   - List tool-assisted runs such as sanitizer, `gdb`, `valgrind`, or `heaptrack`.

4. Execute from lower to upper layers.
   - Run focused lower-layer tests first.
   - Run parent-layer tests next.
   - Run integration or project tests last.
   - If any step fails, keep debugging and updating coverage until the failure is explained and resolved or explicitly documented as a blocker.

5. Refuse incomplete acceptance.
   - A change is not accepted if a relevant boundary case is untested.
   - A change is not accepted if a relevant failure path is untested.
   - A change is not accepted if a test failure is ignored or silently deferred.
   - A change is not accepted if the evidence trail cannot explain why the result is trustworthy.

## Required Acceptance Document

- Create or update `tests/acceptance/<feature-or-milestone>.md`.
- Use this minimum structure:

```markdown
# <Feature or Milestone>

## Scope
- What changed
- Which layers are affected

## Baseline
- Previous failures
- Existing known repository baseline

## Test Inventory
- Unit or focused subsystem cases
- Integration or project cases
- Boundary cases
- Negative cases

## Tooling Evidence
- Tool name and version
- Why that tool was used
- Why WSL was required and which coordinator-granted mounted-drive primary pool was used
- Exact commands
- Key observed outputs

## Results
- Passed checks
- Failed checks
- Fixes made in response

## Acceptance Decision
- Accepted or blocked
- Exact reason
- Remaining risks or baseline blockers
```

## Boundary Coverage Rules

- Enumerate numeric boundaries, empty inputs, null-like cases, malformed syntax, unsupported syntax, duplicate names, type mismatches, large inputs, and repeated execution where relevant.
- When a feature lowers source syntax into instructions or runtime behavior, test both the source-level form and the resulting execution consequences.
- Do not rely on one "representative" case when multiple syntactic or semantic variants exist.

## Reporting

- State the acceptance-document path.
- State every test suite, targeted test, tool run, and boundary matrix that contributed to the conclusion.
- State every failure encountered and how it was resolved, or why it remains a blocker.
