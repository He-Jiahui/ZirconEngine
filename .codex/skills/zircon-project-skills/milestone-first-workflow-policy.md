# Milestone-First Workflow Policy

This policy has priority for `zirconEngine` planning, implementation, validation, and documentation. When a generic workflow conflicts with this policy, follow this policy.

## Priority

- Do not let TDD, plan execution, review, validation, or subagent workflows force per-slice write-test-run loops.
- Do not run build or unit-test commands after every small implementation task unless the user explicitly asks.
- Prefer code production, documentation, and lightweight Rust syntax/type checking during implementation.

## Planning

- Split substantial work into milestones. A plan can have a few milestones or more than ten when the work genuinely needs that shape.
- Each milestone must separate implementation slices from a final testing stage.
- The plan must name the testing stage, expected compile/build commands, expected unit-test or integration-test commands, debug/correction loop, acceptance evidence, and docs to update.
- The plan must include a `## 状态与产出记录` section with the standard milestone/slice/status/date/evidence table. Leave it empty at planning time except for the header and separator rows.
- Before creating or writing that section, apply `write-plan-output-records/SKILL.md`; concrete records belong to numbered child plans, never `index.md`, `engine-code-*.md`, or session notes.
- Small tasks do not need milestone-scale compile and unit-test gates. They still need enough local Rust syntax/type checking to avoid handing off malformed code.

## Implementation Cadence

- Before advancing a numbered child plan, scan its `{id}/` directory for `failure-*.md`. Apply `handle-plan-failure-handoffs/SKILL.md` and resolve applicable handoffs before normal feature slices.
- During implementation slices, generate production code, unit-test code, comments, and docs as needed.
- Unit-test code may be written during a milestone, but do not immediately compile or run it just because a slice was added.
- For small tasks or pre-handoff confidence, use a lightweight Rust syntax/type check such as `cargo check` scoped to the affected crate or target when practical.
- Avoid generating debug build artifacts until the milestone testing stage begins, unless a concrete blocker requires earlier debug evidence.
- After each implementation slice is complete, immediately append exactly one row to the owning numbered child plan or its numbered output archive according to `write-plan-output-records/SKILL.md`. Do not batch-fill multiple slice records later.
- Recount the owning child plan before every session write. If the new total exceeds 10 records, move all concrete records into `docs/plans/{plans_path}/{nn}/{date}-{summary}.md` in the same session.

## Cross-Plan Failures

- When another numbered child plan owns the lowest shared cause, create `failure-{date}-{summary}.md` in that fixing child directory and continue every independent slice in the originating plan.
- Do not mark a session blocked solely because a cross-plan handoff is open.
- A fixing-plan session must prioritize its open `failure-*.md` before normal feature progress and repair the lowest shared architecture, not add a fallback, alias, compatibility shim, test-only bypass, or call-site exception.
- After upward validation, move and rename the canonical artifact into the originating child directory as `fixed-{date}-{summary}.md`; the fixing plan retains a concise status summary and relative link.

## Testing Stage

- At each milestone boundary, enter the testing stage before calling the milestone complete.
- Run the declared compile/build checks and unit tests during the testing stage, then debug and correct failures.
- If an upper-layer test fails, diagnose the lowest shared support layer first, fix it, and rerun validation upward.
- Record what was tested, what failed, what was fixed, and what remains accepted or open.
- Before declaring a milestone complete, confirm the plan's status/output table has one evidence row for every completed slice and the testing stage outcome.

## Comments And Documentation

- Add concise comments for key data structures, invariants, non-obvious state transitions, and core logic decisions.
- Do not comment obvious assignments or mechanical forwarding.
- For meaningful code modules that are created, changed, or reorganized, create or update docs under `docs/` using a source-path mirror unless an existing document already owns the behavior: `zircon_runtime/src/foo/bar.rs` maps to `docs/zircon_runtime/foo/bar.md`.
- If an existing `docs/` module document already owns the behavior, update that document and keep its machine-readable `related_code`, `implementation_files`, `plan_sources`, and `tests` header current.
