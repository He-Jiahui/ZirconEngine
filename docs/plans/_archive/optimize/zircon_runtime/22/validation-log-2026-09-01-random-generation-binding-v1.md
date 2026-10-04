---
record_kind: managed_validation_attempt
status: admission_rejected
date: 2026-09-01
session_id: root-runtime22-checkpoint-atomicity-20260829
request_id: runtime22-random-generation-binding-aggregate-20260901-v1
lease_request_id: 871e3b0d8df442da92b8ada0fe2448b4
attribution_request_id: 82a62e211f2c4c43a36536bfd5617bd6
log_lease_request_id: 2d094a09696d421f9cfd3d66af1e1aa6
---

# Runtime22 random generation-binding validation attempt

The Runtime22 owner session re-claimed and attributed the exact random checkpoint contract and
restore/eviction paths before this attempt. The source manifest contained:

- `zr_contracts`: `stream_checkpoint.rs`, `service_checkpoint.rs`, `checkpoint_error.rs`, and
  `tests/checkpoint.rs`;
- Runtime random: `registry.rs`, `service.rs`, `tests/service.rs`, `tests/retention.rs`, and
  `registry/evict_matching_tests.rs`.

Current source hashes at submission:

- `stream_checkpoint.rs` `a353e505a5b674c81f864ab13130acd10cd4f18e37f042048c592e06c1bde44c`;
- `service_checkpoint.rs` `34a8c276f4e2c969e2069d30dfd368d31009f7ee7b91ce09055f0c4fd0edd739`;
- `checkpoint_error.rs` `4c306bc801928cafdcf9ee934f3da1aa68316aef38c36d881eb27e75465d3480`;
- `tests/checkpoint.rs` `b6d364aa58267619e393c3fc2c307dfe6c77449c13c20a0dc9b5769e271aee45`;
- Runtime registry `c274d3402ad8615740255d2220b22db673012ed7fa244ced601245d18e8bab3c`;
- Runtime service `ed28b21c90b9276c04caee73c112a1d0a415cdc0a0d48a2f238685da93132516`;
- Runtime service tests `3eeb0cdf0d28025cf48a23795385766595222d76b441f5db93086a7545e8de1c`;
- Runtime retention tests `e5010dac741f105c61a509fb802345330da18f81651a6d6a85de6634e37e068b`;
- eviction tests `a27ea6c09da8a3dc4eee2ec1160d9f92df141abd414f8b28db609222bf65acdb`.

The requested managed command was `cargo test --locked --release -p zr_contracts -p zircon_runtime
random -- --include-ignored --nocapture --test-threads=1` on Rust 1.94.1 /
`x86_64-pc-windows-msvc`.

Coordinator rejected the request before immutable validation admission with
`validation_ticket_external_worktree_dirty` for external repository `E:\Git\zr_vm`. No Cargo
ticket, compile, test, restore/replay check, eviction check, benchmark, measured performance,
commit, push, or WeCom success is claimed. This session did not clean, reset, modify, wait for,
poll, or monitor the external worktree.

## Retry v2 (2026-09-01)

The owner re-claimed the nine exact source, regression, eviction, and Failure-record paths with
lease request `c0d843572084480c9b1d14ecc79e0a36` and attributed the current snapshots with
request `2d67a522021f4bbdb7ba03be4ffda775`. The checkpoint error dependency was separately
re-claimed with `f660af8fcf1a414684cf3657a4a19993` and attributed with
`a259546f521749dbafea62253b0006ee`.

Aggregate request `runtime22-random-generation-binding-aggregate-20260901-v2` was rejected before
immutable validation with `validation_ticket_external_worktree_dirty` for external repository
`E:\\Git\\zr_vm`. No Cargo ticket, compile, test, restore/replay check, eviction check, benchmark,
measured performance, commit, push, or WeCom success is claimed. The external worktree was not
cleaned, reset, modified, waited on, polled, or monitored.

## Retry v3 (2026-09-02)

Aggregate request `runtime22-random-generation-binding-aggregate-20260902-v3` covered the current
`zr_contracts` generation-binding constructor/serde regressions and Runtime atomic capture,
restore/replay, and eviction paths. The coordinator again rejected admission with
`validation_ticket_external_worktree_dirty` for `E:\Git\zr_vm`; the read-only request audit found
no `validation_ticket_requests` row, so no Cargo job was queued or started. No compile, test,
benchmark, commit, push, WeCom, or failure-return success is claimed.
