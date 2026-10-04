---
record_kind: milestone
status: implemented_pending_validation
created_at: 2026-09-12
plan: docs/plans/astra/optimize/01-review-and-repair.md
milestone: AI-P1-015 bounded behavior-tree evaluation admission
session: astra-ai-budget-admission-20260912
---

# AI behavior-tree bounded evaluation admission

This bounded AI runtime slice hardens the existing recursive behavior-tree
executor against unbounded per-tick work. The executor now admits each node
through fixed node-count and nested-depth budgets and returns the existing
typed `AiDecisionStatus::Blocked` result with a deterministic diagnostic when
admission is denied. Valid shallow trees retain their existing semantics.

## Finding status and ownership

| Finding | Status | Lowest owner and current evidence |
|---|---|---|
| `AI-P1-015` recursive evaluate has no node/depth admission | `implemented_pending_validation` | `zircon_plugins/ai/runtime/src/behavior_tree/executor.rs:34-52,210-270,361-461` owns per-agent execution context and node dispatch; `evaluate_node` admits depth and cumulative node work before dispatch. |
| `AI-G06` deterministic budget result | `implemented_pending_validation` | Existing `BehaviorTreeExecution`/`AiDecisionStatus::Blocked` contract is reused; focused tests assert stable status, active node, and diagnostic across repeated ticks. |

The source review also records iterative stack/program-counter execution,
wall-clock budget, continuation receipts, and reentrancy policy as part of the
full AI-P1-015/G06 contract. Those portions remain open and are not silently
claimed by this slice. Existing `BehaviorTreeStack` cycle detection remains in
place for subtree re-entry.

## Implementation and tests

- `executor.rs` keeps the local policy constants
  `DEFAULT_BEHAVIOR_TREE_NODE_EVALUATION_BUDGET = 4_096` and
  `DEFAULT_BEHAVIOR_TREE_EVALUATION_DEPTH_BUDGET = 256` private to the
  executor; no cross-crate contract currently consumes them.
- `BehaviorTreeExecutionContext` tracks cumulative node admissions and active
  nested depth per tick. The context is freshly initialized for each normal
  evaluation; abort-only contexts initialize the same fields but do not
  evaluate nodes.
- `evaluate_node` performs admission before invoking node semantics, emits a
  typed `Blocked` execution with either the node-work or depth diagnostic on
  rejection, and releases depth after normal dispatch. The node budget spans
  nested registered subtrees in the same tick. Parallel evaluation stops at
  the first blocked child so a denied budget cannot devolve into an unbounded
  sibling scan while preserving the first-blocked result.
- `behavior_tree_execution.rs` adds a 300-node sequence regression for depth
  admission and a 4,100-leaf parallel regression for node admission. Both
  assert deterministic repeated-tick status, active node, and diagnostic. A
  2,048-leaf parallel regression ticks twice under the limit to prove the
  counters are reset for each tick while preserving successful wide-tree
  behavior.

## Static evidence and validation boundary

- `rustfmt --edition 2021 --check` passed for the two Rust files.
- Scoped `git diff --check` passed.
- A source-contract gate confirmed both named constants, per-tick context
  initialization, pre-dispatch admission, typed diagnostic reasons, and all
  three regression names.
- TDD RED was recorded before implementation: the regression names were
  present while the executor had no budget diagnostics; the GREEN static gate
  then confirmed the implementation contracts.
- No Cargo, native, DLL, or product command was run. Managed Windows Cargo and
  runtime behavior execution remain pending because the external
  `E:\\Git\\zr_vm` worktree is dirty and the parent task deferred those
  commands.

Current post-edit source fingerprints (SHA-256):

```text
zircon_plugins/ai/runtime/src/behavior_tree/executor.rs A19B129CB23E3F578092D3BEB003AAD9650D898B32C35E80135EEC0F455C159C
zircon_plugins/ai/runtime/src/tests/behavior_tree_execution.rs F5E383B73620C439D4BF8FF64818060E289A9869ACDA8BA29AF64921BCAAEC0D
```

## Residual risks and follow-up

The recursive call graph and selector probe helpers remain; the new admission
guard bounds normal node dispatch but does not yet separately budget probe
walking, provide an iterative continuation, or enforce a wall-clock deadline.
Compile-time descriptor traversal and same-agent execution leasing are
separate AI-P1 owners. A follow-up should define continuation/cancellation
semantics before making budget exhaustion retryable, and should add managed
release, long-chain, wide-tree, subtree, and reentrant concurrency evidence.

## Coordination

Session `astra-ai-budget-admission-20260912` owned exactly the two source/test
paths and this record; all three leases were released after the final static
checks (source/record release receipt `d3cc829abe8c430ca1180cbd396dbf7a`;
final record fingerprint refresh receipt `38815d7b1d6449338488ebf8251bce73`).
The coordinator session is `waiting_validation`, and no commit was created.
This record does not claim broader AI or Windows acceptance.
