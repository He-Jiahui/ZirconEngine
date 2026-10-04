status: implemented_pending_validation
plan_sources:
  - docs/plans/optimize/zircon_editor

# Editor test contract repair

## Current source and repair

The `EditContext` contract now routes transaction work through an explicit
world-domain route. Two UI test fixtures still implemented the removed
`runtime_gateway` method and therefore failed to compile. They now use the
same logical route fixture as the other editor transaction tests, with
no-op activation and retirement because these tests do not own a runtime
session. The dirty-save toolkit fixture also implements the required
reference-validation hook as an explicit successful admission.

## Plan completion list

| Batch | Work | Status | Validation evidence |
|---|---|---|---|
| M27 | Restore `EditContext` and `DocumentToolkit` test fixture contracts | implemented_pending_validation | Static source review complete; combined Runtime/Editor Cargo batch pending |

This change repairs test ownership at the shared contract boundary. It does
not claim that the broader Editor batch is green; the prior batch exposed
independent UI feature, gateway, and pointer-routing failures that remain in
the combined validation queue.
