# Test-Driven Development

Use a failing behavioral test to guide a feature or regression fix when that test gives useful evidence. Honor an explicit user request for TDD.

- State the behavior and choose a test at the responsible layer. Prefer an existing regression or a focused addition that can fail for the reported reason.
- When running the test before the fix is useful, confirm its failure reflects the missing behavior rather than a broken fixture.
- Implement the smallest coherent change, then run the affected checks at the repository's validation boundary.
- Refactor with the relevant evidence still valid; re-run affected checks when behavior, contracts, or dependencies change.

[ZirconEngine validation cadence](../../../../../docs/plans/milestone-validation-policy.md) controls when Cargo runs. Writing a test does not require an immediate build after each slice.

Do not delete useful implementation solely because it preceded its test. Verify that the test can detect the defect through a focused reproduction or a safe isolated comparison when needed. Documentation, mechanical edits, and other reversible low-impact changes may use structural checks without new unit tests or an exception approval.

Keep tests about observable contracts. Do not mirror implementation details or mock away the behavior being tested. Read [testing anti-patterns](testing-anti-patterns.md) only when designing mocks or diagnosing a brittle test.
