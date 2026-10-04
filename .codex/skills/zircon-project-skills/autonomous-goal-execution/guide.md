# Autonomous Goal Execution

Apply when the user explicitly authorizes a longer-running ZirconEngine Goal and asks the agent to choose recommended technical branches. This extends the execution horizon; ordinary authorized task completion already uses [the shared execution loop](../continuous-milestone-execution/execution-loop/index.md).

- Establish the requested Goal outcome, acceptance boundaries, and scope. Create or update a tool-managed Goal only when the user or higher-priority instructions request it.
- Select dependency-ready work and make routine technical decisions without repeating permission requests.
- For complex engine decisions, use UnrealEngine as the primary reference where it fits, with [reference routing](../zr-reference-engine-routing/guide.md) selecting additional evidence only when it resolves a question.
- Keep implementation, verification, and correction moving under the repository's validation cadence. Establish exact file ownership and preserve foreign changes where overlap requires it; no coordinator registration is required.
- Honor explicit approval checkpoints and action-specific authorization for commits, publishing, and messages. Goal authorization does not waive those boundaries.
- Keep pending acceptance open under [receipt lifecycle](../cross-session-coordination/references/receipt-lifecycle.md). Finish after all authorized Goal requirements are satisfied.

See [decision and execution notes](decision-and-execution.md) for the same scope boundary.
