# Milestone Completion Requires Risk Coverage

- A milestone is complete when every changed feature, public contract, and known-risk boundary inside its declared scope is implemented and validated.
- Validation belongs to the milestone testing stage. Do not convert every implementation slice into its own compile/build/unit-test loop unless the user explicitly asks.
- Do not accept a milestone because a single demo program works, a constructor test passes, or one integration smoke goes green.
- Build a compact risk inventory for the milestone: changed syntax or instruction forms, runtime behaviors, public error paths, data-shape variants, and boundary conditions.
- If the milestone concerns instruction generation, verify each changed lowering path and any syntax form with distinct code generation. Reuse existing coverage for unchanged equivalent forms.
- If the milestone concerns parsing, type handling, runtime helpers, or library behavior, target changed variants and known regression boundaries instead of duplicating stable equivalent coverage.
- When tests are missing, add them before declaring success.
- Unit-test code may be written during implementation, but the milestone is not complete until the testing stage has run the declared compile/build and unit-test evidence and fixed failures.
- The milestone is not complete until the active plan's `## 状态与产出记录` table has one concise accepted outcome with the testing-stage evidence and accepted residual risk.
- When an upper-layer scenario fails, reopen the lower-layer inventory first and prove the shared support path is correct before adjusting the upper layer.
