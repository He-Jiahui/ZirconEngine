---
handoff_kind: failure
status: open
created_at: 2026-08-31
summary_slug: runtime-interface-input-route-clone-contract
origin_plan: docs/plans/zircon_runtime/frameworks/01-runtime-crate-decomposition.md
fixing_plan: docs/plans/optimize/zircon_runtime_interface/03-ui-authoring-accessibility-input-diagnostic-status-public-contract-review.md
origin_child_dir: docs/plans/zircon_runtime/frameworks/01
fixing_child_dir: docs/plans/optimize/zircon_runtime_interface/03
plan_link_mode: child_record_only
related_code:
  - zircon_runtime_interface/src/ui/dispatch/input/result.rs
tests:
  - validate-matrix.ps1 -Package zr_resource -LibTests -TargetDir E:\cargo-targets\frameworks01-resource-identity-i1-green -VerboseOutput
---

# Frameworks01: Runtime Interface input route promises an unproven Clone iterator

## Failure receipt

Frameworks01 reran the full managed Windows `zr_resource` library validation from shared current
source after the Resource generation, event-order and durable-artifact hard cuts:

```text
validate-matrix.ps1 -Package zr_resource -LibTests
  -TargetDir E:\cargo-targets\frameworks01-resource-identity-i1-green -VerboseOutput
```

Coordinator evidence:

- job `8af64b6fdf4a4d928cc31fb92ea934ae`;
- validation Session
  `validate-matrix:019ffe2b-296a-7023-9433-8654b9ea8f18:successor:fd746df3825b4f80b933d46218f4cf69`;
- start `2026-08-31T08:47:50.502965+08:00`;
- finish `2026-08-31T08:53:16.982510+08:00`;
- release `2026-08-31T08:53:26.393098+08:00`, exit `1`;
- target and managed scratch remained on `E:`; no artifact was written to `C:`.

Cargo stopped while compiling `zircon_runtime_interface`, before it compiled the changed
`zr_resource` crate. Rustc 1.94.1 reported E0277 at
`zircon_runtime_interface/src/ui/dispatch/input/result.rs:116`: `physical_bubble_route()` promises
`Clone`, but the opaque iterator returned by `UiHitPath::bubble_route()` does not expose that trait
bound. The same fingerprint contains six unrelated non-blocking unused-import warnings.

## Current-source diagnosis and ownership

Current `result.rs` SHA-256 is
`f420794ef68e44e3a1ed37288fede517b96813101c06aa8d894d7f6dfe576bf2`. The error is an API
contract mismatch between two Runtime Interface UI route layers, not a Resource implementation
failure.

Coordinator ownership matrix request `10d393e3fe7b4c559a3dc867b4d210e1` identifies owner Session
`root-runtime-interface03-activate-link-failure-20260831`, status `waiting_validation`. Session-show
request `3102cd55692f4c88afadbff14740ed37` confirms the exact file is already in that Session's
immutable write scope under RuntimeInterface03. Frameworks01 did not claim or edit the source.

## Acceptance

- Preserve one coherent public iterator contract across `UiHitPath::bubble_route()` and
  `UiPointerRoutingReceipt::physical_bubble_route()`; either prove and expose `Clone` from the
  lower layer or remove the unsupported upper-layer promise based on actual consumers.
- Add or retain owner-focused tests that compile the public route iterator and cover forward,
  reverse and repeated traversal semantics required by callers.
- Compile and test `zircon_runtime_interface` in the owner Session and return the final file hash
  plus validation/integration receipt.
- Frameworks01 then reruns the exact managed `zr_resource` command above.

## Owner fix receipt

RuntimeInterface03 reconciled the public contract by restoring the `Clone` bound on
`UiPointerRoutingReceipt::physical_bubble_route`. The lower authoritative
`UiHitPath::bubble_route()` already returns a borrowed double-ended, exact-size, cloneable iterator,
so the upper receipt now exposes the same proven bound without allocating or collecting a route.
The focused Rust test proves forward, reverse, and repeat traversal through independent borrowed
iterators.

- Owner Session: `root-runtime-interface03-activate-link-failure-20260831`.
- Final `result.rs` SHA-256: `B85724D756C6140A3D3113D052D163B6B9CDEAC5C2A3525D5BA66805F4BE4A15`.
- Local static contract: `tools.tests.test_runtime_ui_input_routing_receipt_contract`, 9/9 passed.
- Scoped `rustfmt +1.94.1 --edition 2021 --check --config skip_children=true`: passed.
- Scoped `git diff --check`: passed.
- Shared-current managed Cargo acquire request: `114e5f73abc548cc9f73bedbcc923308` (older
  no-Clone source).
- Shared-current managed Cargo job: `722b72d364924d23af0e179af27229e4` (older source receipt).
- A fresh current-source managed Cargo ticket is submitted below; its terminal receipt is required
  before this failure can close.

Current-source batch validation ticket:

- ticket `cd1d6cff000c42d18bf7d4a3abb4be84`;
- submit request `0ffcd0e158c34e8b91dcf6e2b918bff7`;
- command `cargo +1.94.1 test -p zircon_runtime_interface --locked --release --jobs 1`;
- source manifest includes this failure, the pointer-route serde failure, the ActivateLink failure,
  `result.rs`, `effect.rs`, `route.rs`, and `ui_host_request.rs` at their current hashes.

The ticket is queued in the managed Windows lane and is intentionally not polled here.

## 2026-08-31 Current-Source Copy-Complete Reconciliation

The lower authoritative `UiHitPath::bubble_route()` implementation is part of the same current
Runtime Interface UI migration and explicitly exposes the borrowed
`DoubleEndedIterator + ExactSizeIterator + Clone` bound over `root_to_leaf.iter().rev().copied()`.
The upper `UiPointerRoutingReceipt::physical_bubble_route()` promise therefore remains valid and
must not be weakened to manufacture a compile fix.

- Current `zircon_runtime_interface/src/ui/surface/hit.rs` SHA-256:
  `D22539FE5A6167AC235D0B814062D6861B810524DB7BD57555012CC4407539AC`.
- Current `zircon_runtime_interface/src/ui/dispatch/input/result.rs` SHA-256:
  `B85724D756C6140A3D3113D052D163B6B9CDEAC5C2A3525D5BA66805F4BE4A15`.
- The owner Session now leases both exact paths and the input-routing contract test; the complete
  source closure must include `hit.rs` so the managed copy can prove the lower bound.
- Focused static routing contract remains `9/9` GREEN. A copy-complete snapshot/managed ticket is
  pending coordinator capacity; no source bytes were changed in this reconciliation.

The record remains open until that managed current-source interface compile/test returns terminal
evidence; Frameworks01 may then rerun its exact `zr_resource` command.

## Constraints

- Frameworks01 must not claim, rewrite or commit this foreign Runtime Interface input blob.
- Do not allocate or collect the route merely to manufacture `Clone`; the route remains a borrowed
  exact-size iterator over the existing hit path.
- This failure blocks managed compile/test evidence only. Frameworks01 continues durable Resource
  profile and plan-record work while the owner closes it.
