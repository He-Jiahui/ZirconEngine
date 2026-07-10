# Plan By Layer

## Progressive Disclosure Index

- Read `../../milestone-first-workflow-policy.md` before shaping milestones.
- Read this file when you need to turn a goal into dependency-ordered milestones.
- If you are deciding whether a milestone can advance, return to `../milestone-completion-requires-full-coverage.md`.
- If execution finds an upper-layer regression, apply `../../support-first-regression-testing/SKILL.md`.

## Workflow

1. State the target outcome in repository terms.
   - Name the affected layers such as parser, IR or instruction generation, runtime execution, library support, CLI integration, and tests.

2. Map the lower-to-upper dependency chain.
   - Prefer the order implied by the current tree: shared definitions and common support, parser and semantic construction, core execution or instruction behavior, library and builtins, CLI or end-to-end entrypoints, then broad integration checks.
   - Adjust the exact order to the task, but never let a higher layer become the proof that a lower layer works.

3. Define milestone slices that can be fully validated.
   - Keep each milestone narrow enough that you can enumerate all in-scope behaviors.
   - Write the promotion gate for each milestone before implementation starts.
   - Include implementation slices, unit-test code to write, docs to update, required edge cases, and artifacts in the milestone definition itself.
   - Add a named testing stage for the milestone. This is where compile/build commands, unit tests, debug correction, and acceptance evidence run.
   - Add the standard status/output record section to the plan so execution can write one row as each slice finishes.

4. Protect VM core correctness first.
   - Prefer milestones that establish parser correctness, instruction correctness, execution correctness, and shared runtime correctness before polish, wrappers, or auxiliary tools.
   - Delay convenience features until the core path is proven.

5. Re-plan when evidence changes.
   - If validation shows a lower shared layer is still broken, move the plan back down immediately instead of continuing upward.

## Milestone Template

- Goal: the concrete capability this milestone must provide.
- In-scope behaviors: the complete inventory of syntax, instructions, runtime paths, or APIs covered by the milestone.
- Dependencies: the lower layers that must already be correct.
- Implementation slices: code, unit-test code, comments, and docs to produce before the testing stage.
- Testing stage: focused unit tests, negative tests, boundary tests, integration checks, compile/build commands, debug/correction loop, and acceptance records needed for promotion.
- Lightweight checks: scoped Rust syntax/type checks allowed before the testing stage, especially for small tasks.
- Exit evidence: exact commands, suites, or artifacts that must pass during the testing stage before moving upward.

## Required Status Record

Every milestone plan must include this section. During execution, fill it slice by slice; after one slice completes, update exactly one row immediately. Do not batch backfill records at the end.

```markdown
## 状态与产出记录

执行时逐切片填写；完成一个切片更新一行，不许批量补记。

| 里程碑 | 切片 | 状态 | 完成日期 | 证据（命令输出 / 文件 / 测试名） |
|---|---|---|---|---|
```

Use the local calendar date for `完成日期`. In `证据`, cite concrete files, commands, test names, or docs that prove the slice outcome.
