---
doc_type: optimization-implementation
status: candidate_static_review_complete_managed_validation_pending
runtime: Runtime62
gate: RSH-G19
related_code:
  - zircon_runtime/src/scene/world/derived_state_clean_world_matrix_profile.rs
  - zircon_runtime/tests/runtime_scene_projected_read_performance.rs
tests:
  - runtime62_clean_active_in_hierarchy_profile
  - runtime62_clean_active_in_hierarchy_allocation_profile
---

# Runtime62 RSH-G19: clean active-hierarchy read evidence

## Scope

The clean `active_in_hierarchy` read already uses the published
`ActiveInHierarchy` row when the active frontier is empty. This successor adds
measurement coverage for that ordinary read path alongside the clean
world-matrix candidate. It does not change production behavior.

The ignored Windows Release unit profile builds committed chains of depths 1,
32, and 1,024. It confirms the leaf's published cache and clean frontier, then
collects 31 raw batches of 256 public `active_in_hierarchy` calls per depth.
It prints batch and per-query latency samples with nearest-rank p50/p95/p99
under `RUNTIME62_CLEAN_ACTIVE_LATENCY_PROFILE_V1`. The existing world-matrix
profile keeps its marker and query label after sharing the sample reporter.

The ignored integration profile reuses the test binary's existing global
allocator. It checks the committed `World::new` active-camera cache before
measuring 31 batches of 256 reads at depth 1, prints raw latency, allocation
count, and requested-byte samples under
`RUNTIME62_CLEAN_ACTIVE_ALLOCATION_PROFILE_V1`, and asserts zero allocation
in every measured batch. Fixture setup and cache assertions are outside the
measurement window. Run it exactly and serially with `--exact
--test-threads=1`; this depth-1 allocation result cannot represent the deeper
latency fixtures or the 100K/1M G24 scale gate.

## Validation and remaining gate

Pinned Rustfmt 1.94.1 and scoped `git diff --check` passed for the two changed
Rust paths. The v4 preimages were
`8c4756964b84a2522d7ace9d209bc230186254c548f35ae6b46067b807693e79`
and `8f740062a2e9ac360fd112e0bd31bbb6cb62cf430cef2f5b20f4aca90942f4f5`;
the successor postimages are
`dd94b7ecf4fdd53e501040137581ebb5c3a7dcc0fc4df8116b18ce635534a10e`
and `df11ca911c79811ebfddfb006cccf38c08cf4b3dabe6d4b8319cb336edc53931`.

No Cargo test or profile has run. No latency ceiling, before/after comparison,
or measured allocation pass is claimed. RSH-G19 and G24 remain open pending
managed behavior, latency, allocation, and scale evidence.
