# PlatformBundle native plugins streaming directory traversal

## Change

`copy_platform_bundle_native_plugins_dir` now consumes `Path.iterdir()` directly
instead of first materializing every child into a list. Recursive copy behavior
and diagnostics are preserved, including errors raised while the iterator is
being consumed.

## Performance evidence

Windows local Python benchmark, 15 rounds over 100,000 lazy directory entries:

| Metric | Before | After | Improvement |
| --- | ---: | ---: | ---: |
| Peak Python allocation p50 | 3,993,072 bytes | 480 bytes | 99.988% lower, 8318.90x |
| p50 | 274,615,800 ns | 119,716,000 ns | 56.406% lower |
| p95 | 508,020,000 ns | 152,401,200 ns | 70.001% lower |

## Validation

The streaming contract passed 2/2. The expanded PlatformBundle batch passed
70/70 and covers input diagnostics, template/native-plugin materialization,
copy failures, native-dynamic integration, schema, and owner boundaries.
