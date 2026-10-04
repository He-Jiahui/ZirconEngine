# Dispatching Parallel Agents

When the current environment permits delegation, split genuinely independent investigations or implementation tasks that can proceed alongside useful primary-agent work.

1. Identify independent ownership and dependencies. Shared mutable files, a common changing contract, or an unresolved prerequisite require sequencing.
2. Dispatch a bounded objective with relevant evidence, exact write scope, acceptance criteria, and expected return format. Follow [model selection](../../../zircon-project-skills/cross-session-coordination/references/model-tier-policy.md).
3. Continue the primary task while agents work. Use completion notifications or bounded event waits; avoid repeated unchanged status polling.
4. Inspect every result, reconcile assumptions and edits, and run one appropriate integration batch. Expand only for affected contracts or observed failures.

Do not delegate solely to satisfy a quota, dispatch the same failing build to several agents, or create user-visible tasks without a user request. If the environment prohibits subagents, execute the same bounded work locally.

For implementation and review ownership, use [subagent-driven development](../subagent-driven-development/guide.md).
