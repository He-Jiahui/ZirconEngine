---
status: local_candidate
implementation_files:
  - tools/analysis/profiling/shader_pbr/zircon_shader_pbr_evidence_identity.py
  - tools/tests/test_shader_pbr_identity_manifest_single_read_performance_contract.py
---

# Shader PBR identity manifest single read

## Problem

Ready-frame identity validation first streamed the identity manifest to verify
its SHA-256, separately probed its byte length, and then reopened it as text for
JSON parsing. Large identity manifests therefore paid two content reads and one
extra metadata syscall before their fields were validated.

## Change

`_load_verified_identity` now reads the identity manifest once as bytes. The
same payload supplies byte length, SHA-256, UTF-8-sig decoding, and JSON parsing.
Screenshot, viewer binary, HDRI, and build-provenance fingerprint validation
remain unchanged and streaming. Existing unavailable, fingerprint mismatch,
malformed JSON, and schema errors remain fail-closed.

The static performance contract prevents the identity path from returning to a
separate fingerprint helper or text reopen.

## Validation

```text
python -m unittest tools.tests.test_shader_pbr_identity_manifest_single_read_performance_contract tools.tests.test_zircon_validate_shader_pbr_viewer_evidence
20/20 passed in 7.237s
```

The combined viewer + full profile summary batch exceeded the 184-second local
window without failure output. It is not reported green or failed and belongs
in the asynchronous coordinator lane.

## Performance

Windows local benchmark with one 8 MiB identity JSON file, two warm-ups and 41
alternating-order iterations; p95 is the 39th ordered sample.

| Metric | Baseline | Candidate | Improvement |
| --- | ---: | ---: | ---: |
| Content reads | 2 | 1 | -50.000% |
| Bytes per validation | 16,777,162 | 8,388,581 | -50.000% |
| Metadata probes | 1 | 0 | -100.000% |
| p50 | 38,901,900 ns | 30,025,000 ns | -22.819% (1.30x) |
| p95 | 65,196,900 ns | 43,040,800 ns | -33.983% (1.52x) |
