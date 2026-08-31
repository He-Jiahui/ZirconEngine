---
handoff_kind: failure
status: open
created_at: 2026-08-30
summary_slug: runtime136-composition-test-validation-materialization
origin_plan: docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md
fixing_plan: docs/plans/optimize/zircon_runtime/99zk-runtime-builtin-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-current-source-review.md
origin_child_dir: docs/plans/optimize/zircon_runtime/02
fixing_child_dir: docs/plans/optimize/zircon_runtime/136
plan_link_mode: child_record_only
related_code:
  - zircon_runtime/src/builtin/runtime_modules/tests/registration/structure.rs
  - zircon_runtime/src/builtin/runtime_modules/tests/registration/composition.rs
tests:
  - validation ticket cc62406c73b5435ab3c8dada05132535
  - validation copy job e3fb1b77eea14cde851f6f585222acd9
---

# Runtime136: composition regression is outside validation materialization

## Source executor

- Origin plan: `docs/plans/optimize/zircon_runtime/02-core-runtime-events-tasks-review.md`
- Origin slice: aggregate Runtime/Editor Release validation for optimization batches 501-504 and
  506-512
- Fixing plan:
  `docs/plans/optimize/zircon_runtime/99zk-runtime-builtin-module-catalog-profile-target-feature-selection-extension-registration-capability-load-report-product-integration-current-source-review.md`
- Handoff reason: the missing compile-time resource contains Runtime136 composition compiler
  regressions and `runtime136.*` fixtures, below Runtime02 optimization ownership.

## Failure evidence

Validation ticket `cc62406c73b5435ab3c8dada05132535`, copy job
`e3fb1b77eea14cde851f6f585222acd9`, failed during closure planning with
`validation_copy_compile_time_resource_missing`. The source guard
`zircon_runtime/src/builtin/runtime_modules/tests/registration/structure.rs` includes
`composition.rs`, but that target is not materializable from the ticket source closure.

Current hashes are:

- `structure.rs`:
  `4e653d23e8b74f09fecbf0afa56bec9257a9b04827f82e04a87b05655004cf09`
- `composition.rs`:
  `37ac596b5a2d4e832c0ad76830545f74ffbf006dc1e6cb9cb3004b0ec672e248`

`composition.rs` exists as an untracked, unattributed file. `structure.rs` retains archived
Frameworks02 attribution. Runtime02 did not edit, claim, attribute, or add either file to its
candidate.

## Lowest shared-layer root cause

Runtime136 split its composition regressions into a child source and made the structural guard
require that child, but the child has no durable coordinator attribution or integrated source
owner. Any validation copy that discovers `structure.rs` therefore fails before Rust compilation.

## Architecture acceptance

- Runtime136 legally claims and attributes the exact composition regression source together with
  its registration module/structure closure, or reconciles it through the owning Runtime136
  session without absorbing unrelated files.
- The split test owner remains canonical; the tests are not copied back into `structure.rs`.
- Managed Runtime validation advances beyond closure planning and compiles the Runtime136
  composition tests.
- Runtime02 reruns one aggregate validation for the affected optimization batches after the source
  is integrated or legally transferred.

## Forbidden workarounds

- Do not delete or weaken the `include_str!("composition.rs")` structural assertion.
- Do not inline duplicate composition tests into `structure.rs` or add a fallback include path.
- Do not claim the unattributed source through maintenance override or include it in a Runtime02
  commit candidate.

## Return contract

Return the exact ownership/integration request ID, final hashes for the registration test closure,
and managed validation evidence that `composition.rs` is materializable. Runtime02 will then resume
the aggregate validation without polling the fixing session.
