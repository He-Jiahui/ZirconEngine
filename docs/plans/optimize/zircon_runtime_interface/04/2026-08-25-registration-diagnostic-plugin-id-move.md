---
title: Runtime Interface 04 Registration Diagnostic Plugin ID Move
category: zircon_runtime_interface
report_id: RuntimeInterface04-registration-diagnostic-plugin-id-move-2026-08-25
date: 2026-08-25
session_id: optimize-runtime-interface04-diagnostic-id-move-r1-20260825
implementation_status: implementation_complete
validation_status: managed_validation_passed
---

# Runtime Interface 04 Registration Diagnostic Plugin ID Move

## Scope

This batch advances the plugin registration diagnostic path described by Runtime Interface 04. It
preserves severity, stable code, plugin ID, capability text, and the exact user-facing message. It
does not claim the parent plan's shared diagnostic envelope, stage/artifact identity, native sink,
profiling budgets, event continuity, or generated ABI work is complete.

## Change

`RegistrationDiagnostic::missing_capability` previously converted the caller's plugin ID to an
owned `String`, cloned that complete string into the diagnostic, and retained the original only
long enough to format the message. The implementation now reserves the exact message capacity,
appends static text and both borrowed values directly, then moves the same plugin-ID allocation
into the diagnostic. Plugin-ID clones per diagnostic are `1 -> 0`; the message uses one reserved
buffer and avoids generic formatting dispatch. Severity, code, ID, and message are unchanged.

The ignored release gate uses a 16 KiB plugin ID over 21 alternating sample pairs. Inputs are
prepared outside the measured interval, so the comparison isolates diagnostic construction. The
optimized P95 must be at most 80% of the legacy clone path. Managed Windows-native validation
measured `30.2 us -> 14.6 us`, a `51.656%` P95 reduction.

## Validation

- TDD red state: the source performance contract failed 2/2 against the clone path.
- Source performance contract after implementation and whitespace-tolerant rustfmt guard: 2/2
  passed.
- A focused Rust regression preserves all fields and the exact missing-capability message.
- Python bytecode compilation, `rustfmt --check`, and scoped whitespace validation: passed.
- Managed ticket `75bef115f188423f83976634ec1680fa` passed focused behavior but measured
  `legacy_p95_ns=29,800 -> optimized_p95_ns=24,400`, only an `18.121%` reduction and short of the
  20% gate. The follow-up replaces generic formatting with one exact-capacity message buffer rather
  than weakening the threshold.
- Managed follow-up ticket `620c6a251212434bb83a094fd9f21724` passed focused behavior and the
  ignored release gate. It measured `legacy_p95_ns=30,200 -> optimized_p95_ns=14,600`, a
  `51.656%` reduction, while plugin-ID clones remain `1 -> 0`.
- No local Cargo lane or Cargo dry-run was launched, polled, or terminated.

## Remaining Parent-plan Work

Runtime Interface 04 still requires the three P0 closures, observation identity and clock
qualification, producer-bounded paging, host-owned artifacts, generated native capability tables,
plugin-event continuity, and diagnostic-envelope convergence.
