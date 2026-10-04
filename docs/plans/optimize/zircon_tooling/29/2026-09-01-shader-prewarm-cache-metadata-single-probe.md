---
status: local_candidate
implementation_files:
  - tools/build/zircon_build_shader_prewarm_cache_artifacts.py
  - tools/tests/test_shader_prewarm_cache_metadata_single_probe_performance_contract.py
---

# Shader prewarm cache metadata single probe

## Problem

The shader prewarm cache scan called `exists()` and then `is_dir()` for the
cache root, and `exists()` followed by `is_file()` for every metadata sidecar.
Each successful path therefore paid two metadata syscalls. The per-sidecar cost
scaled linearly with the number of cached shader variants.

## Change

The scanner now calls `Path.stat()` once per root or metadata path and classifies
the returned mode with `stat.S_ISDIR` or `stat.S_ISREG`. Missing paths retain
the existing empty-cache or missing-metadata behavior. Other filesystem errors
still propagate instead of being treated as absence.

The static performance contract locks the scan to exactly one root and one
per-sidecar `stat()` call, with no `exists/is_file/is_dir` reprobes.

## Validation

```text
python -m unittest tools.tests.test_shader_prewarm_cache_metadata_single_probe_performance_contract tools.tests.test_zircon_build_shader_prewarm_cache_contract
26/26 passed in 2.601s
```

## Performance

Windows local benchmark over 4,096 existing metadata sidecars, two warm-ups and
41 alternating-order iterations; p95 is the 39th ordered sample.

| Metric | Baseline | Candidate | Improvement |
| --- | ---: | ---: | ---: |
| Metadata probes | 8,192 | 4,096 | -50.000% |
| p50 | 177,315,900 ns | 100,742,600 ns | -43.185% (1.76x) |
| p95 | 323,506,400 ns | 199,629,300 ns | -38.292% (1.62x) |
