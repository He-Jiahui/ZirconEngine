---
title: Runtime77 Pointer and Navigation Dispatch Handler Hash Lookup
category: zircon_runtime
report_id: Runtime77-dispatch-handler-hash-lookup-2026-09-13
date: 2026-09-13
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime77 Pointer / Navigation Dispatch Handler Hash Lookup

## Scope

Pointer and navigation dispatch only probe handler tables by their complete
`(node, event[, phase])` key. Neither dispatcher exposes handler-map iteration or
uses map order as a delivery rule; registration order inside each handler vector
and the route's candidate order remain authoritative.

## Change

- Replace the private pointer `handlers` and `phase_handlers` `BTreeMap`s with
  `HashMap`s for expected-constant-time exact-key lookup.
- Replace the private navigation handler `BTreeMap` with a `HashMap` for the same
  lookup-only path.
- Derive `Hash` for the pointer/navigation event-kind enums and dispatch phase while
  retaining `PartialOrd`/`Ord` for serialized and diagnostic ordering contracts.
- Keep handler-vector order, phase chaining, route traversal, visited-node policy,
  and all dispatch effects unchanged.

## Complexity boundary

For `H` registered handler keys, each candidate lookup changes from ordered-map
`O(log H)` to expected `O(1)`; handler invocation and route traversal remain
unchanged. This is a lookup/allocation-layout optimization, not a claim that the
remaining route or callback work is bounded by a fixed latency. Release allocation
and pointer/navigation p50/p95/p99 evidence remains a managed gate.

## TDD and local evidence

- The new source contract intentionally failed before implementation because both
  dispatchers still used `BTreeMap` and the key enums were not hashable.
- After the production change, the focused contract passes `4/4`.
- The batched Runtime UI/input contract invocation (including Runtime737/738
  adjacent guards) loaded 113 modules and passed `492/492` tests in `19.899s`.
  Existing Runtime input-route behavior remains the semantic regression set.
- The corresponding Editor asset/hierarchy/reference batch loaded 18 modules and
  passed `89/89` tests in `0.499s`; both are local contract evidence, not a
  managed Cargo receipt.
- Scoped rustfmt, Python compilation, and diff/whitespace checks are recorded with
  the combined batch; no coordinator status was queried.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/dispatch/pointer/dispatcher.rs` | `2E97DF1D32C54B018AA6E712DF7F765444F98A28E8D91F4F7279B26EF9EE602F` |
| `zircon_runtime/src/ui/dispatch/navigation/dispatcher.rs` | `9CF842AD5A705414A8450A34C43864B099F683BC131FF08DBBCEB8E70ADC6220` |
| `zircon_runtime_interface/src/ui/surface/pointer/event_kind.rs` | `FDCB15656F0C2D5B163819F6DF72239E086A48E6371CC66D7835F6E56A3AF9B6` |
| `zircon_runtime_interface/src/ui/surface/navigation/event_kind.rs` | `539E8C960743EC634124D1EC65AF7174B29DC6CAE0011E272F397A8BCBF5CA61` |
| `zircon_runtime_interface/src/ui/dispatch/input/reply.rs` | `9FC2692AE0E8AAAF1919EE0965C050D9C9ABCD8518F72790B936D58FAE994DA5` |
| `tools/tests/test_runtime_ui_dispatch_hash_lookup_performance_contract.py` | `63BABB083E682FE8E7F866D404D7F34E28A9B5139B94E2E4FF2A7027BAE80AD5` |

## Managed gate

This slice joins the existing owner-attributed multi-task Runtime/Editor batch.
Managed Cargo/Release validation is pending because the prior admission was
rejected by the external `E:\\Git\\zr_vm` dirty-worktree and static-overlay
ownership gates. No standalone Cargo process or coordinator polling was started.
