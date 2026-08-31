# Runtime75 Preallocated Child Mount Set

- Date: 2026-08-31
- Parent record: `docs/plans/optimize/zircon_runtime/75/2026-08-22-allocation-free-native-slot-admission.md`
- Status: implementation_complete; managed_validation_pending
- Owner Session: `root-runtime-interface03-activate-link-failure-20260831`
- Scope: `zircon_editor/src/ui/asset_editor/palette/instantiate.rs`

## Problem

Reference conversion validation counted every child mount in a `BTreeMap<&str, usize>` even though
the result only needed to know whether a slot was occupied. The map allocated tree nodes and the
counter values for each child, then was scanned again for required slots.

## Optimization

- Use one `HashSet<&str>` preallocated to `children.len()`.
- Insert borrowed `child.mount.as_deref().unwrap_or_default()` values once.
- The `insert` result rejects duplicate single slots while allowing repeated multiple slots.
- Required-slot validation reuses the same set; unknown slots still fail closed through the schema
  lookup.
- No mount string is cloned and no behavior policy changed.

## TDD And Verification

- RED: the source contract failed against the previous `BTreeMap<&str, usize>` counter path.
- GREEN: current palette occupancy and mount-set contracts pass `4/4`.
- Rust behavior coverage exercises required single, duplicate single, repeated multiple, unknown
  slot, and missing required slot outcomes.
- The ignored release benchmark validates 32 declared slots and 64 child mounts over 10,000 checks
  per sample, with 21 alternating legacy/optimized pairs. It emits raw samples and nearest-rank
  P50/P95 and requires optimized P95 to be at least 20% lower.
- Python bytecode compilation and scoped `git diff --check` pass.

## Deterministic Performance Evidence

For `C` child mounts and `S` declared slots:

| Metric | Before | After |
| --- | --- | --- |
| temporary occupancy structure | `BTreeMap<&str, usize>` | one `HashSet<&str>` |
| capacity policy | node growth per distinct mount | `children.len()` upfront |
| mount-name ownership | borrowed keys | borrowed keys |
| asymptotic validation | `O(C log C + S log C)` | average `O(C + S)` |

The existing Runtime75 release benchmark remains the authoritative measured P50/P95 gate; this
record's benchmark joins that managed release batch and does not claim a wall-clock speedup before
execution.

## Remaining Scope

Native descriptor authority, palette generation caching, component schema admission, and the wider
Editor palette workload matrix remain open in the Runtime75 review.
