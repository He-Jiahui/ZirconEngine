# Milestone Completion Requires Full Coverage

- A milestone is complete only when every feature inside its declared scope is implemented and validated across normal, boundary, and failure cases.
- Validation belongs to the milestone testing stage. Do not convert every implementation slice into its own compile/build/unit-test loop unless the user explicitly asks.
- Do not accept a milestone because a single demo program works, a constructor test passes, or one integration smoke goes green.
- Build an explicit inventory for the milestone: syntax forms, instruction forms, runtime behaviors, error paths, data-shape variants, and boundary conditions.
- If the milestone concerns instruction generation, verify every supported `zr` syntax that should emit instructions, every emitted instruction form, and every relevant boundary case. Missing syntax coverage means the milestone is still open.
- If the milestone concerns parsing, type handling, runtime helpers, or library behavior, apply the same rule: all reachable variants in scope need targeted tests, not just representative examples.
- When tests are missing, add them before declaring success.
- Unit-test code may be written during implementation, but the milestone is not complete until the testing stage has run the declared compile/build and unit-test evidence and fixed failures.
- The milestone is not complete until the active plan's `## 状态与产出记录` table has one row per completed slice and testing-stage outcome, written as each slice completed rather than batch-filled at the end.
- When an upper-layer scenario fails, reopen the lower-layer inventory first and prove the shared support path is correct before adjusting the upper layer.
