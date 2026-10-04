---
title: Runtime Binding Literal Single Buffer
category: zircon_runtime
report_id: Runtime871-binding-literal-single-buffer-2026-09-21
date: 2026-09-21
session_id: root-runtime-editor-async-optimization-20260921
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_target_met
---

# Runtime871 Binding Literal Single Buffer

## Finding

Runtime UI binding-expression serialization created one escaped `String` per
Flags value, collected those strings into a vector, joined the vector, and then
formatted the final constructor. Vec2/Vec3/Vec4 performed the same chain with
temporary float strings. Typed string constructors also created a quoted child
string before producing the required final literal.

## Optimization

- Stream Flags values, separators, quotes, and escapes into one exactly sized
  constructor output.
- Stream vector components into one capacity-bounded output and append each
  finite non-exponent float in place; truncate the attempted component and
  preserve `None` on rejected input.
- Reuse append-oriented string and float owners from the existing owned helpers,
  including exact escaped-length calculation for a one-allocation string path.
- Preserve every escape spelling, Unicode byte sequence, `flags()` emptiness,
  vector punctuation, `.0` normalization, and non-finite/exponent rejection.

## TDD and deterministic evidence

The combined Editor890/Runtime871 source-model batch was observed RED at `2/10`
and GREEN at `10/10`. Lower regressions lock empty/plain/escaped/control/Unicode
strings, typed strings, Flags, Vec4, integral float spelling, and rejected
non-finite vector components against the retired collect/join implementation.

For 4,096 Flags values, the deterministic model changes temporary child strings
and vector slots from `4096/4096` to `0/0`; the required final literal remains.
Ignored marker `RUNTIME871_BINDING_LITERAL_SINGLE_BUFFER_BENCH_V1` emits 101
alternating p50/p95/p99 sample pairs over 128 literals of 64 escaped values and
requires single-buffer p95 to remain within 10% of collect/join.

## Local validation boundary

- Exact-file Rustfmt, Python bytecode compilation, and scoped
  `git diff --check` pass.
- Runtime871 and Editor890 contracts pass `10/10`; the combined batch with
  Animation Editor ZUI, curve projection, and timeline projection contracts
  passes `18/18`.
- Runtime871 received no per-task Cargo run and was submitted with Editor890 in
  asynchronous v17 (PID `10460`) at `2026-09-21T21:47:37.0745928+08:00`.
- No v13-v17 receipt was read or monitored after submission.

## Source snapshot

| File | SHA-256 |
| --- | --- |
| `zircon_runtime/src/ui/template/asset/compiler/binding_param_resolver.rs` | `0B4FD8F4A3460F9B26D4A2138A03ABBDDD622B52442F871DE9837A983716A99F` |
| `zircon_runtime/src/ui/template/asset/compiler/binding_param_resolver/single_buffer_tests.rs` | `AF3E8ACB405EB2C3AB302B1D4CF7062220B3035D6786E9508BEA8AA436268F65` |
| `tools/tests/test_runtime871_binding_literal_single_buffer_performance_contract.py` | `51F3B8DDD61591ABD57DD7792BADE11B802730D45B52227FBD1FB677574AF9EE` |

## Runtime871 compile repair (2026-09-21)

The one-time v21 receipt identified `E0596` in this source's escaped-control
character branch: the `&mut String` parameter was erroneously borrowed again
in `write!(&mut source, ...)`. The exact source hash matched this record before
the repair. A new static shape regression was RED `1/6`, then GREEN `6/6`
after writing through the existing mutable reference with `write!(source, ...)`.
The lower parity case now includes `U+0001` and `U+001F`; it awaits managed
Rust execution. The combined repaired Runtime871/Editor894/Runtime875 static
batch passes `17/17`. The v21 compile failure is real, not a passed gate.

## Acceptance boundary

Keep this record `implementation_complete` / `managed_validation_pending` until
a combined current-source Windows lane compiles Runtime, executes the lower
regression and ignored Release marker, and supplies allocator plus real UI
template-binding compilation p50/p95/p99 evidence. The deterministic allocation
model is not product acceptance.
