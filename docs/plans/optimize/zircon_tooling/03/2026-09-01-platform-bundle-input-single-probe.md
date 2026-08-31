# PlatformBundle input single metadata probe

## Scope

- Owner: Tooling03 export pipeline.
- Production: `tools/zircon_export/platform_bundle_materialize.py`.
- Contract: `tools/tests/test_tooling03_platform_bundle_input_single_probe_performance_contract.py`.

## Change

`platform_bundle_file_input_diagnostic` previously called `exists()`, `is_file()`,
and `stat()` for each valid host, pack, and delta input. It now performs one
`stat()` and derives file kind and size from the returned metadata. A missing
path still reports `does not exist`; other metadata errors now correctly report
`could not be inspected` instead of being swallowed by `Path.exists()`.

## Performance evidence

Windows local benchmark, Python 3, 25 rounds of 20,000 valid-input checks:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 1,181,834,600 ns | 478,474,500 ns | 59.514% lower, 2.47x |
| p95 | 1,465,583,500 ns | 559,583,000 ns | 61.818% lower, 2.62x |
| Metadata probes per valid input | 3 | 1 | 66.667% lower |

## Validation

The combined PlatformBundle batch passed 35/35 tests, covering the new
performance contract, input behavior, cleanup failures, template resolution,
and owner boundaries. Final repository-wide validation remains assigned to the
coordinator's longer combined lane.
