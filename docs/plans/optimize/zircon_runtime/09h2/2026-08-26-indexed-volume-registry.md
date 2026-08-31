---
title: Runtime09H2 Indexed Volume Registry
category: zircon_runtime
report_id: Runtime09H2-indexed-volume-registry-2026-08-26
date: 2026-08-26
session_id: root-runtime09h2-indexed-volume-registry-20260826
implementation_status: implementation_complete
validation_status: managed_validation_pending
---

# Runtime09H2 indexed Volume registry

## Scope

- Parent scope: the Runtime09H2 Volume evaluation CPU path, specifically component descriptor resolution for builtin and plugin overrides.
- Baseline: `8e56165c4c789416c328898d3d8937d934b52efa`, epoch `443`.
- Owners: `VolumeComponentRegistry` registration and lookup, its source/performance contract, focused Rust tests, and this record.
- This slice removes linear descriptor scans only. It does not close Volume persistence, unsupported shapes, overlay ownership, resource readiness, GPU effects, color correctness, or the remaining Runtime09H2 acceptance gates.

## Change

- `VolumeComponentRegistry` retains its descriptor `Vec` as the authoritative registration and iteration order.
- A `HashMap<&'static str, usize>` side index maps component ids to vector positions without owning or copying ids.
- Registration uses the side index for duplicate detection and records the descriptor position once.
- `contains` and `get` now perform indexed lookup; `get` projects the stored position back into the ordered descriptor vector.
- Focused Rust coverage verifies stable registration order, indexed resolution, duplicate rejection, and an unchanged registry after a rejected duplicate.

The built-in registry currently contains 15 descriptors, while the same public registry also accepts plugin descriptors. The side index prevents per-override evaluation cost from growing linearly with the combined registry size.

## TDD and local evidence

- RED: `python -m unittest tools.tests.test_runtime09h2_indexed_volume_registry_performance_contract -v` initially failed 4/4 because the registry had no side index and used vector `any/find` scans.
- GREEN: the source contract passes 4/4 after indexed registration and lookup are implemented.
- Two intermediate false negatives caused by `rustfmt` method-chain line breaks were corrected by normalizing whitespace in the source contract; the guarded operations and exclusions were not weakened.
- `rustfmt +1.94.1 --edition 2021 --config skip_children=true` and `git diff --check` pass for the owned files.
- The standalone model is compiled with `rustc +1.94.1 -O`; it does not use Cargo or the shared build lane.

The plugin-scale model measures 31 alternating linear/indexed sample pairs over 2,048 descriptors and 16,384 deterministic successful lookups. Registry construction is excluded, and every pair must produce the same complete lookup checksum. Four local runs passed the plugin-scale acceptance thresholds; the table records the latest run.

| Metric | Linear descriptor scan | Static-id side index | Change |
|---|---:|---:|---:|
| P50 | 179.7066 ms | 0.9213 ms | -99.487% |
| P95 | 269.7692 ms | 1.6198 ms | -99.400% |

The other three plugin-scale runs produced P50 reductions of 99.539%, 99.481%, and 99.536%, with P95 reductions of 97.905%, 96.109%, and 98.910%.

A separate 15-descriptor built-in workload kept P50 lower in all measured runs. With 262,144 lookups per sample, two runs reduced P50 by 52.221% and 57.853%; their P95 changes were -9.388% and +49.331% on the busy host. Therefore the acceptance gate is deliberately scoped to plugin-scale lookup behavior and does not claim a stable built-in-only P95 win. These timings isolate CPU registry lookup; they do not claim complete Volume evaluation or frame time.

## Async validation

One coordinator batch must run the four Python source contracts, all six focused Volume registry Rust tests in the real `zircon_runtime` crate, Rust formatting checks, scoped diff checks, exact plugin-scale model parity, and the same performance workload.

Acceptance requires 4/4 source contracts and 6/6 Rust tests to pass, exact lookup checksum parity, and plugin-scale P50/P95 reductions of at least 90%. Integration and automatic WeCom publication remain coordinator-owned after managed validation succeeds. The WeCom message must include managed P50/P95 reductions and label them as 2,048-descriptor CPU registry-lookup evidence, with the 15-builtin P95 caveat retained.
