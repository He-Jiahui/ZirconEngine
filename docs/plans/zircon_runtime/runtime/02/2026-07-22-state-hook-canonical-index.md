# Runtime02 State Hook Canonical Index

status: implementation_static_reviewed_compile_blocked_foreign_current_source
date: 2026-07-22
base_head: f7627a0d2ba277be67e7b12abf2538b4d79d763c

## Scope

This Runtime02 child slice closes the independently actionable half of
`PERF-MVP-320`: state hook lookup no longer scans all registered enter, exit,
and transition hooks for every transition. It deliberately does not change
transition-history retention because that public consumer/cursor contract is
still undecided.

Exact source and record scope:

- `zircon_runtime/src/core/framework/state/hook_index.rs`
- `zircon_runtime/src/core/framework/state/mod.rs`
- `zircon_runtime/src/core/framework/state/machine.rs`
- `zircon_runtime/src/tests/state.rs`
- `zircon_runtime/src/tests/state/hook_index.rs`
- `docs/plans/zircon_runtime/runtime/02/2026-07-22-state-hook-canonical-index.md`

The scope was clean and had no active lease before Session
`runtime02-state-hook-canonical-index-r1-20260722` claimed its exact six paths.
The slice avoids the active Frameworks01/Runtime11 prerequisite session and
does not edit Runtime02/index/review-findings parent mirrors.

## Design And Implementation

- `StateHookIndex<T>` is the single hook-storage owner.
- Enter and exit hooks use `HashMap<T, Vec<StateHook<T>>>`; transition hooks
  use a two-level `HashMap<T, HashMap<T, Vec<StateHook<T>>>>`, so both state
  values are borrowed during lookup instead of cloned to construct a pair key.
- Registration appends to the matching bucket, preserving registration order.
- Dispatch clones only the three matching buckets, then keeps the established
  `exit -> transition -> enter` run order in `StateTransitionDispatch`.
- `StateMachine<T>` retains current/next/event authority and delegates hook
  registration and lookup to the index. Dispatch remains outside the registry
  lock through the existing CoreRuntime/CoreHandle path.
- No public API, alias, forwarding shim, event-history behavior, or facade
  owner changes are introduced.

## TDD And Validation

The focused structure RED was observed before production implementation:

- exit 1: `RED: canonical hook_index owner is missing`.

The new child tests lock both behavior and ownership:

- matching exit/transition/enter buckets execute in phase and registration
  order while nonmatching buckets do not execute;
- initialization dispatches only the matching enter bucket,
  `set_next_if_neq` suppresses identity hooks, and an explicit identity
  transition preserves exit -> transition -> enter ordering;
- `StateMachine<T>` owns one `StateHookIndex<T>`, the index owns hash buckets,
  and the old three linear `Vec` scans are absent.

Current static evidence:

- `rustfmt +1.94.1 --edition 2021` completed for the exact five Rust paths;
- the focused source assertion is green;
- scoped `git diff --check` passed with repository line-ending warnings only;
- snapshots `900` and `910` froze their respective preceding exact-six record
  revisions while the five Rust hashes shown below remained unchanged;
- the authoritative current exact-six freeze is the latest coordinator
  replacement snapshot for this Session, avoiding a self-referential snapshot
  number in the record whose content that snapshot hashes;
- independent final review completed with `Critical 0 / Important 0 / Minor 0`.

The source-bound focused reservation
`8c8c6082e8aa4a069c45c850c6784b68` was released unconsumed under the
support-first policy after a broader current-source lib-test compile stopped on
37 foreign errors before any target test ran. That compile emitted no error
header for an exact-six path, but it is diagnostic evidence only: this record
makes no compile, test-pass, performance, fixed-return, or commit claim.

## Current Source Manifest

| Path | Lines | SHA256 |
|---|---:|---|
| `zircon_runtime/src/core/framework/state/hook_index.rs` | 90 | `a397025a6110026b0c6d332b7e4b3e040f8c6bfec66620e50d7503753173381d` |
| `zircon_runtime/src/core/framework/state/mod.rs` | 24 | `710d74c337a582266cbbf0f11e45180f5c2ba2d95e19c893f94111a0faba8568` |
| `zircon_runtime/src/core/framework/state/machine.rs` | 107 | `a0894b81fe9a7ad0017ff7f95ec94b9181cbd65333d4b41b8786f9445cbc24dc` |
| `zircon_runtime/src/tests/state.rs` | 170 | `449c62cb9170047e6d0a0feaaccc655479e1078cebf59f1c75327c3a73826c6a` |
| `zircon_runtime/src/tests/state/hook_index.rs` | 140 | `679e3846ac6bb72d0ec278f306c5bf66684241813bb56893f4a4d091186e4ce7` |

## Remaining Gates

- Run a source-bound managed Windows focused state-hook test with raw target
  test-count evidence.
- Run the existing Runtime02 state regression/parity gate after the focused
  test is valid.
- Recheck the exact six-path manifest and repeat independent acceptance review
  after valid managed Cargo evidence, before any managed scoped commit.
- Keep transition history/cursor/retention as a separate architecture slice;
  this hash index must not be cited as closing that requirement.
