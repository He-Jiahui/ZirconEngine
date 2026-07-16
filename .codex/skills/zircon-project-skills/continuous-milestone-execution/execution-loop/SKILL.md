# Execution Loop

## Progressive Disclosure Index

- Read `../../milestone-first-workflow-policy.md` first if the active work is in `zirconEngine`.
- Read this file when the task should continue through the active milestone without unnecessary interruption.
- If the milestone itself is not yet defined, go to `../../layered-milestone-development/SKILL.md`.
- If you are deciding whether it is acceptable to ask the user, go to `../stop-only-for-real-branch-ambiguity.md`.

## Workflow

1. Rebuild context from the repository and the active plan.
   - Read the touched code and the plan artifacts that define the current milestone.
   - Scan for applicable `failure-*.md` at milestone start. A handoff blocks only slices that depend on its cause.
   - State the current milestone in concrete repository terms.

2. Decide the next unfinished slice inside the milestone.
   - Pick the next step implied by the dependency order, existing tests, and current failures.
   - Do not pick an unrelated convenience task just because it is easier.

3. Generate code, tests, and supporting changes as needed.
   - Continue until the slice is actually integrated, not merely drafted.
   - Add unit-test code, docs, and comments during the slice when they belong to the change.
   - Do not run compile/build/unit-test commands after every slice by default.
   - If implementation exposes missing lower-layer support, go fix that lower layer and then resume upward progress.
   - If another numbered plan owns that lower layer, apply `../../handle-plan-failure-handoffs/SKILL.md`, publish the failure there, and continue every independent slice in the current plan rather than pausing the session.
   - Keep the completed slices in the milestone batch; do not append plan rows while the batch is still in implementation.

4. Use lightweight checks during implementation.
   - Use formatting, `git diff --check`, and source guards by default.
   - Defer Cargo compile/build/unit-test execution to the milestone testing stage unless the user asks for earlier validation or a blocker, ABI change, unsafe change, or persistence risk requires it.
   - If a lightweight check fails, debug and continue within the same milestone rather than stopping at the first obstacle.

5. Enter the milestone testing stage at the boundary.
   - Run the declared compile/build/unit-test commands only when the implementation slices for that milestone are ready.
   - Debug and correct failures, starting from the lowest shared support layer that can explain the failure.
   - Record one accepted milestone outcome in `## 状态与产出记录` before calling the milestone complete.

6. Stop only at a real milestone boundary.
   - End the work only when the milestone's completion gate and concise outcome record are complete, or when a real branch ambiguity requires the user to choose between materially different directions.

## Anti-Patterns

- "I analyzed the files, so I will stop here."
- "I implemented one function, so the user can take over."
- "The first passing test is enough for now."
- "I wrote one slice, so I must immediately build and unit test the whole workspace."
- "There may be more work, but I will wait for confirmation before continuing."
- "I found a failure in a lower shared layer, but I will postpone it and keep patching the upper layer."
- "Another plan owns the failure, so this session must stop instead of publishing a handoff and continuing independent work."
- "I will skip the milestone outcome record because the code and tests are enough."

## Reporting

- State the active milestone.
- State what code and plan artifacts were read before execution.
- State which unfinished slice you advanced next.
- State which lightweight checks were run during implementation and which compile/unit-test validations were deferred to the milestone testing stage.
- State which row was added to `## 状态与产出记录`, including the evidence used.
- State why the work stopped: milestone completed or real branch ambiguity.
