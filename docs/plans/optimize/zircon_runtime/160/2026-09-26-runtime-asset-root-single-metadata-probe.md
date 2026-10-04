---
title: Runtime Asset Root Single Metadata Probe
category: zircon_runtime
report_id: Runtime160-asset-root-single-metadata-probe-2026-09-26
date: 2026-09-26
implementation_status: implementation_complete
validation_status: managed_validation_pending
performance_status: deterministic_work_reduction_met_dynamic_pending
---

# Runtime asset root single metadata probe

Both non-authoritative runtime asset-root selection paths checked `exists()` and `is_dir()` for
each candidate. An existing root therefore incurred two metadata lookups before the asset path
could be selected. Diagnostic output also repeated those checks. One `root_status` lookup now
supplies the existing `exists` and `is_dir` flags and the admission decision. It follows directory
symlinks like the prior `Path` methods and preserves missing-root, regular-file, candidate-order,
and authoritative-root behavior. The explicit-root and fallback contracts are unchanged.

| Evidence | Before | After / acceptance target |
| --- | ---: | ---: |
| Metadata lookups for each existing non-authoritative root | 2, or 4 with verbose diagnostics | 1, or 2 with verbose path-list diagnostics |
| Missing/file/directory candidate behavior | Existing root selection | New regression plus existing asset-path tests |
| Release P95, existing-root probe | pending | at most 95% of legacy |

The ignored `RUNTIME_ASSET_ROOT_SINGLE_METADATA_PROBE_BENCH_V1` benchmark compares the original
`exists() && is_dir()` admission with the new single metadata lookup on one existing directory,
using 21 alternating sample pairs and 4,096 probes per sample. It runs with the grouped Runtime
Release batch. This slice does not close the wider Runtime160 path, VFS, or security review. Cargo
and dynamic timing evidence remain pending.
