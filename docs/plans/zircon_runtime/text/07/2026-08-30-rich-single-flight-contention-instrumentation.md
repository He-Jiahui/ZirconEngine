# Rich single-flight contention instrumentation

Date: 2026-08-30  
Scope: `RRT-P1-009` current-source correction and `RRT-P1-014` measurement foundation  
Status: `RRT-P1-014_contention_measurement_static_complete / bounded_worker_cancellation_and_managed_profile_pending`

## Current-source review

`DecoratorRegistry::apply` already wraps a decorator with `catch_unwind`, maps panic to
`RichTextParseError::DecoratorPanicked`, checks a per-call metadata quota, and later parsing checks the
aggregate retained-run metadata budget. The open RRT-P1-009 problem is narrower and harder: a custom
decorator is synchronous and non-cooperative, so a deadline cannot preempt it safely.

`CompiledRichTextCacheOwner::compile` obtains one key-qualified `RichTextArtifactCell` and calls
`OnceLock::get_or_init`. This provides one parse and one shared terminal result, but a hung initializer blocks
all same-key waiters. Existing hit/miss/parse counters could not measure that contention.

## Reference and rejected shortcuts

Local Unreal `FShapedTextCache` is instance-owned and performs synchronous `FindShapedText` followed by
`AddShapedText`; it does not establish a cross-thread single-flight job. Relevant sources are:

- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Public/Framework/Text/ShapedTextCache.h`
- `dev/UnrealEngine/Engine/Source/Runtime/Slate/Private/Framework/Text/ShapedTextCache.cpp`

Zircon's shareable parser cannot copy that calling-model assumption. Two shortcuts are rejected:

- removing single-flight and parsing once per concurrent caller, which can multiply CPU and still hang;
- returning after an arbitrary timeout while the provider continues without a bounded worker/cancel owner.

No cache concurrency algorithm changes before matched contention and fault evidence.

## Measurement implementation

- `compile_requests_in_flight` is a point-in-time gauge for calls that passed the ready-artifact fast path and
  are entering or waiting on `get_or_init`.
- A call-local `Cell<bool>` is set only inside the actual initializer closure. Calls that return from
  `get_or_init` without setting it are completed single-flight waiters.
- Waiters add interval `single_flight_wait_count`, `single_flight_wait_nanos`, and
  `single_flight_wait_max_nanos`.
- `CompiledRichTextCompileRequest` is an RAII guard, so the in-flight gauge is decremented on ordinary return
  and unwind. Counter conversion/addition uses checked saturation and the existing saturation receipt.
- Atomic take/reset preserves the in-flight gauge and clears waiter interval values.
- Surface profile emission remains fixed-cardinality: 16 compiled-cache names, no markup, pointer, resource id,
  or dynamic parser/provider/project label.

The ready artifact path returns before `Instant::now()`. Pending/contended calls add O(1) work and no heap
allocation. Parse/cache asymptotics and single-flight semantics are unchanged.

## Required profile matrix before redesign

Use the managed E/D/F validation path and retain raw samples outside `target`:

| Workload | Caller count | Source size | Provider |
|---|---:|---:|---|
| warm exact hit | 1 | 1/4/16 KiB | built-in |
| unique miss | 1 | 1/4/16 KiB | built-in |
| same-key contention | 2/4/8 | 1/4/16 KiB | built-in |
| same-key contention | 2/4/8 | 1/4/16 KiB | bounded custom provider |
| blocked-provider fault | 2/4/8 | 1 KiB | deliberately blocked custom provider |

Collect wait count/total/max, in-flight gauge, parse count, hit/miss, wall time, CPU, allocations/RSS, and
power. The fault case must prove bounded execution capacity and identical typed terminal receipts before a
deadline/cancel implementation is accepted. A timeout value cannot be selected from source size or a magic
constant.

## Evidence boundary

- failing-first static contract reproduced the missing contention fields;
- a deterministic Rust concurrency regression pins the initializer, observes two in-flight calls, then checks
  one parse, one waiter, shared artifact identity, and gauge cleanup; it is written but not run;
- current infrastructure static suite passes 36/36 in the final 0.206 s rerun;
- focused Rust files pass `rustfmt --edition 2024 --check` and scoped `git diff --check`;
- `rich_cache.rs` is 541 production lines, folder-backed tests are 340, and profile is 739.

Managed Cargo/rustc and the profile matrix did not run because the managed acquisition path remains
unavailable. No worker/cancellation design, deadline, latency improvement, RSS/power result, WGPU/PNG,
commit, or WeCom claim is made. No screenshot is produced because this is nonvisual instrumentation; a
source or strategy screenshot would violate Text07 evidence policy.
