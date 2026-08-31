# PlatformBundle template file single metadata probe

## Change

`materialize_platform_bundle_template_files` now obtains file kind from one
`stat()` result instead of calling `exists()` and `is_file()`. Missing files keep
their existing diagnostic; permission and other metadata errors are reported as
inspection failures instead of being swallowed as missing files.

## Performance evidence

Windows local benchmark, 25 rounds of 30,000 valid template-file checks:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 1,211,198,100 ns | 790,839,100 ns | 34.706% lower, 1.53x |
| p95 | 2,303,062,200 ns | 893,589,800 ns | 61.200% lower, 2.58x |
| Metadata probes | 2 | 1 | 50.000% lower |

## Validation

Included in the 68/68 PlatformBundle combined batch. Contract:
`tools/tests/test_tooling03_platform_bundle_template_file_single_probe_performance_contract.py`.
