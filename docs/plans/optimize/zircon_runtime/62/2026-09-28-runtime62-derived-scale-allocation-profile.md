---
doc_type: optimization-implementation
status: candidate_static_review_complete_managed_validation_pending
runtime: Runtime62
gate: RSH-G24
related_code:
  - zircon_runtime/tests/runtime62_derived_scale_allocation_profile.rs
  - zircon_runtime/src/scene/ecs/frame_performance_diagnostics.rs
tests:
  - runtime62_derived_scale_tick_publishes_matrix_and_active_state_for_small_star
  - runtime62_derived_scale_allocation_profile
---

# Runtime62 RSH-G24: public tick scale and allocation evidence

## Scope

The new standalone integration-test binary uses public `CoreRuntime`,
`LevelSystem`, and `World` APIs. It creates star worlds containing 1,000,
100,000, and 1,000,000 nodes, publishes the initial derived state, then takes
31 root-transform-change samples and 31 root-active-change samples at each
size. Fixture construction, setters, frame-time snapshots, post-tick counter
reads, and report formatting are outside each sample window. A non-ignored
1,000-node regression also checks that the changed world matrix and active
state reach a child and that each changed pass reports the expected number of
visited and written rows.

The ignored Windows Release profile times the public `LevelSystem::tick`
envelope. It does not isolate derived propagation: world-driver stages,
derived-state work, node-cache/render preparation, and normal tick bookkeeping
may contribute to the latency and allocation samples. The integration binary
uses its own thread-local `GlobalAlloc` window. It reports allocation request
calls and gross requested bytes from the measured thread, counting `alloc`,
`alloc_zeroed`, and `realloc` requests and not subtracting deallocations.
Allocations on other threads are outside that TLS measurement. Elapsed values
are instrumented tick-envelope samples, not a whole-process or peak-memory
measurement.

## Candidate result and limits

The profile has not run. No latency or allocation values, numeric ceiling, or
G24 pass are claimed. The one-million-node fixture's peak memory and managed
Windows Release behavior remain unverified. G24 still needs retained raw
1K/100K/1M visited, written, allocation-request, gross-byte, and elapsed
samples, plus a product-grounded ceiling and a measured acceptance run. This
full-tick profile supplements the internal propagation-only profile; it does
not replace it or attribute full-tick values to propagation alone.

## Candidate source hash

| Path | SHA-256 |
|---|---|
| `zircon_runtime/tests/runtime62_derived_scale_allocation_profile.rs` | `e06381bda6c34e31a158cb43b9b420efda49ce9ed824f22f28944bc092ece9fd` |

## Verification

Only pinned Rustfmt and direct byte/document-structure checks are authorized
for this slice. Cargo compilation, runtime execution, and managed Release
validation remain pending. The ignored filter for the later serialized
managed run is `runtime62_derived_scale_allocation_profile` with
`--exact --ignored --nocapture --test-threads=1`.
