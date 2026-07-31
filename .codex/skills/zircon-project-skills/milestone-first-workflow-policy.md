# Milestone-First Workflow Policy

This policy has priority for `zirconEngine` planning, implementation, validation, and documentation. When a generic workflow conflicts with this policy, follow this policy.

## Priority

- Do not let TDD, plan execution, review, validation, or subagent workflows force per-slice write-test-run loops.
- Do not run build or unit-test commands after every small implementation task unless the user explicitly asks.
- Prefer code production, documentation, formatting, diff checks, and source guards during implementation. Cargo is an exception for a concrete blocker, public-contract risk, or an explicit user request.
- Apply `prefer-windows-validation/SKILL.md` before every Cargo build or validation command. All Cargo output must be coordinator-managed below exactly one of the drive-root `cargo-targets`, `targets`, or `ZirconBuilds` directories on `D:`, `E:`, or `F:` (nine Windows roots and their mounted WSL equivalents). No repository, profile, temporary, home, or other output location is allowed.

## Documentation Default

- Treat source, focused tests, and coordinator state as the current truth. No narrative document is the default outcome of an implementation slice.
- Generic skill templates that create `docs/superpowers/specs/` or `docs/superpowers/plans/` do not authorize those files in `zirconEngine`. Reuse the owning numbered plan; create a new plan only when a substantial, multi-milestone change has no usable owner.
- Retain only the minimal numbered-plan scope, one accepted milestone status row, and a cross-plan failure/fixed artifact when applicable. Create or update durable prose only for a missing or otherwise false public contract, operator workflow, architectural boundary, or decision that source and tests cannot carry.

## Planning

- Split substantial work into milestones. A plan can have a few milestones or more than ten when the work genuinely needs that shape.
- Each milestone must separate implementation slices from a final testing stage.
- The plan must name the testing stage, validation scope, debug/correction loop, acceptance evidence, and retained documentation only when an existing owner would otherwise become false. Link the owning source/test suites instead of copying commands or implementation snippets that will drift.
- The plan must include a `## 状态与产出记录` section with a milestone/status/date/evidence table. Leave it empty at planning time except for the header and separator rows.
- Before creating or writing that section, apply `write-plan-output-records/SKILL.md`; concrete records belong to numbered child plans, never `index.md`, `engine-code-*.md`, or session notes.
- Small tasks do not need milestone-scale compile and unit-test gates. They still need enough local Rust syntax/type checking to avoid handing off malformed code.

## Implementation Cadence

- Use `tools/zircon-session.ps1` only for overlap-sensitive work, shared files, or active cross-plan failures. Isolated changes do not require a session record.
- Scan for `failure-*.md` when the active milestone starts, when a related test fails, or when a current dependency is known to be owned elsewhere. Apply `handle-plan-failure-handoffs/SKILL.md` only to an applicable failure.
- An applicable failure blocks its dependent milestone, not independent work. The fixing owner repairs it in the next repair window, immediately when no independent slice remains.
- Claim shared files only when another active Session may be editing them.
- During implementation slices, produce production code and focused tests first. Add source comments for non-obvious invariants; create or update prose only under the documentation exception below.
- Unit-test code may be written during a milestone, but do not immediately compile or run it just because a slice was added.
- Avoid Cargo build artifacts until the milestone testing stage begins, unless a concrete blocker, persistence/ABI risk, or explicit request requires earlier evidence.
- Do not create per-slice plan output rows. Keep a live task note only when the coordinator state cannot convey a material coordination risk, then write one concise progress/evidence record per accepted milestone through `write-plan-output-records/SKILL.md`.
- Ask the coordinator to authorize a `docs/plans` target only when a shared Session is registered. Global plan definitions and indexes are not ordinary business-output targets.

## Cross-Plan Failures

- When another numbered child plan owns the lowest shared cause, create `failure-{date}-{summary}.md` in that fixing child directory and continue every independent slice in the originating plan.
- Do not mark a session blocked solely because a cross-plan handoff is open.
- At the repair window, the fixing-plan Session must set `resolving_failure` and repair the lowest shared architecture before resuming work that depends on it. Do not add a fallback, alias, compatibility shim, test-only bypass, or call-site exception.
- After upward validation, move and rename the canonical artifact into the originating child directory as `fixed-{date}-{summary}.md`; the fixing plan retains a concise status summary and relative link.

## Testing Stage

- At each milestone boundary, enter the testing stage before calling the milestone complete.
- Run the declared compile/build checks and unit tests as one batched scope selected by `docs/plans/milestone-validation-policy.md`, then debug and correct failures.
- If an upper-layer test fails, diagnose the lowest shared support layer first, fix it, and rerun validation upward.
- Record what was tested, what failed, what was fixed, and what remains accepted or open in one concise evidence record per accepted milestone.

## Comments And Documentation

- Add concise comments for key data structures, invariants, non-obvious state transitions, and core logic decisions.
- Do not comment obvious assignments or mechanical forwarding.
- Treat source and tests as the primary facts; the current numbered plan records only scope and completion state. Do not create a narrative document for an ordinary implementation slice, private refactor, test-only change, or a command result.
- Update an existing `docs/` owner only when it would otherwise state a false public contract, operator workflow, architectural boundary, or non-obvious invariant. Keep the correction concise; remove stale claims instead of appending a changelog.
- Create a new module document only for a new public/cross-module interface, operational workflow, or durable design decision that has no truthful owner and cannot be understood from source comments, tests, and the plan. Otherwise prefer a concise code comment and the milestone status row.
- When a documentation file is retained, keep its machine-readable header to the owning modules and relevant validation suites. Do not enumerate every touched file or per-slice test command.
