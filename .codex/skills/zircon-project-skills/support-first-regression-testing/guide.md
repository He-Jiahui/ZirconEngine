# Support-First Regression Testing

Use when a failing upper-layer scenario may originate in shared lower-layer behavior. Follow [systematic debugging](../../superpowers/review-and-stabilization/systematic-debugging/SKILL.md) for the investigation loop.

- Trace the failing contract through the plausible dependencies indicated by the symptom: parsing, resolution, type inference, runtime helpers, loading, or shared execution.
- Select a discriminating lower-layer check. Do not enumerate and test every possible support layer when evidence already narrows the cause.
- Repair the owning contract before changing upper-layer expectations. Avoid caller-specific fallbacks, test-only bypasses, and weaker assertions.
- Re-run the affected lower-layer check, then the failed caller or integration case. Broaden only when shared contracts or new evidence require it.
- For a cause owned by another numbered plan, use [failure handoffs](../handle-plan-failure-handoffs/guide.md) and continue independent work.

An existing reliable failure and unchanged supporting evidence can be reused. The repository validation policy governs the timing and breadth of compilation.
