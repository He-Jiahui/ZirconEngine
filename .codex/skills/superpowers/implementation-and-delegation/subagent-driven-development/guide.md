# Subagent-Driven Development

Use permitted subagents for bounded implementation or review work when delegation improves an approved plan. If delegation is unavailable or restricted, perform the work locally.

- Give each agent the objective, acceptance criteria, relevant context, exact owned paths, dependencies, and required return evidence. Follow [model selection](../../../zircon-project-skills/cross-session-coordination/references/model-tier-policy.md).
- Run independent tasks concurrently only when their writes and assumptions do not conflict. Serialize shared contracts and integration work.
- Share enough context for correct decisions. Full history or a focused extract can both be appropriate; do not require a particular context transport.
- Resolve routine questions from existing requirements. Escalate a consequential missing user decision while other authorized work continues.
- The primary agent owns integration and may make local fixes. Reuse an agent when continuity helps; create a new one for an independent perspective when risk warrants it.
- Review the actual changed scope. Combine requirements and quality review unless the risk or a required gate calls for separate reviewers. Reuse unchanged review evidence.
- Batch validation after integration under [the validation policy](../../../../../docs/plans/milestone-validation-policy.md). Avoid separate full-suite runs by each agent and the primary agent.

Optional dispatch templates: [implementation](implementer-prompt.md), [requirements review](spec-reviewer-prompt.md), and [quality review](code-quality-reviewer-prompt.md). These are starting points, not mandatory additional agents or review stages.
