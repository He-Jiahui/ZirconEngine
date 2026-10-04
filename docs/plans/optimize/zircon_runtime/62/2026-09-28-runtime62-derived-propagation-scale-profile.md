---
doc_type: optimization-implementation
status: candidate_static_review_complete_managed_validation_pending
runtime: Runtime62
gate: RSH-G24
related_code:
  - zircon_runtime/src/scene/world/derived_state.rs
  - zircon_runtime/src/scene/world/derived_state_scale_profile.rs
tests:
  - runtime62_derived_propagation_scale_profile
---

# Runtime62 RSH-G24: isolated propagation scale samples

## Scope

An ignored Windows Release unit profile now builds star hierarchies of 1,
1,000, 100,000, and 1,000,000 nodes. After fixture construction and initial
publication, it changes the root transform or active state outside the timed
region and runs only the corresponding internal propagation system inside the
timed region. Each operation collects 31 elapsed-time, visited-row, and
written-row raw samples, prints nearest-rank p50/p95/p99 latency, and asserts
that a root change visits and rewrites the affected hierarchy. The fixture and
normal scene stage preparation are not included in propagation latency. After
each sample, the pending render journal and node cache are published outside
the timer, so one sample cannot carry another sample's accumulated dirty IDs.
The same filter also samples 1/32/1,024-node single-child chains to catch a
depth regression from the new branching-ancestor stack; raw lines identify
`shape=star` or `shape=chain`.

This is a measurement harness, not a passing performance result. The paired
depth-bounded propagation implementation is recorded separately in
`2026-09-28-runtime62-depth-bounded-propagation-stack.md`. The profile has not
run. It does not measure allocation count/bytes,
does not set a numeric ceiling, and does not close RSH-G24. Peak memory for
the million-node fixture and managed Release runtime remain unverified.

## Candidate source hashes

| Path | SHA-256 |
|---|---|
| `zircon_runtime/src/scene/world/derived_state.rs` | `fe82df8f26f412cf51d728e152486aff5441e907f54ebcbed9929d8b1af8b045` |
| `zircon_runtime/src/scene/world/derived_state_scale_profile.rs` | `fbc55d04ac16e51c4aaf7d5448bbd3f5c0131f9b7e2df11746ca3dfec495f896` |

## Verification and open gate

Pinned `rustfmt +1.94.1 --edition 2021 --check --config skip_children=true`
passed for both Rust files, and scoped `git diff --check` passed for the
tracked module binding. No Cargo, compiler, or Release profile was run.
The successor grouped validation needs the exact ignored profile filter and
must retain raw output. Allocation-byte sampling, a defensible numeric
ceiling, and a passing measured run are still required for G24.
