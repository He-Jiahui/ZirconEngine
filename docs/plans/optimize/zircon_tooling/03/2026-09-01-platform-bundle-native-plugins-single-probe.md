# PlatformBundle native plugins single metadata probe

## Change

`materialize_platform_bundle_native_plugins` now derives directory kind from one
`stat()` result instead of calling `exists()` and `is_dir()`. Missing and invalid
roots preserve the public failure behavior; metadata errors now retain their
actual inspection diagnostic.

## Performance evidence

Windows local benchmark, 25 rounds of 30,000 valid native-plugin-root checks:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| p50 | 1,288,500,500 ns | 733,338,100 ns | 43.086% lower, 1.76x |
| p95 | 1,719,472,700 ns | 1,079,784,100 ns | 37.203% lower, 1.59x |
| Metadata probes | 2 | 1 | 50.000% lower |

## Validation

Included in the 68/68 PlatformBundle combined batch. Contract:
`tools/tests/test_tooling03_platform_bundle_native_plugins_single_probe_performance_contract.py`.
