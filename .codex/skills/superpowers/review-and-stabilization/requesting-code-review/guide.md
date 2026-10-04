# Requesting Code Review

Use independent review for substantial behavioral changes, risky boundaries, or an explicit repository acceptance gate. Small reversible edits can use a focused self-review.

- Define the intended behavior, exact owned diff or snapshot, relevant constraints, and validation evidence.
- In a dirty shared checkout, identify owned paths and current content hashes or a sealed manifest. Do not assume HEAD~1 contains this task's work or include another task's changes.
- Ask the reviewer to assess correctness, requirements, affected contracts, and maintainability together. Separate review stages only when they have distinct required purposes.
- Fix actionable findings, then revisit the affected portions. Reuse review of unchanged portions; do not repeat the whole review per slice, batch, and final message.
- When the applicable milestone requires a distinct reviewer, obtain that review and retain its evidence. The coordinator is retired; review does not require coordinator registration or an API receipt. This skill does not authorize creating user-visible tasks or posting external messages.
- Report unresolved findings and their practical effect. A minor suggestion need not block unrelated authorized work.

Use the [review brief](code-reviewer.md) when a structured prompt helps.
