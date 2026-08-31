# Tooling03 Plugin build artifact single probe

## Problem

Plugin package and asset-pack materialization checked `exists()` and then
`is_file()` for their generated artifacts. Existing artifacts therefore paid two
filesystem metadata probes immediately after the build process had completed.

## Change

Both gates now call only `Path.is_file()`. Missing paths and directories remain
rejected, while regular files remain accepted. Diagnostics and stage behavior are
unchanged.

The performance contract drives both production entrypoints with an existing
directory in place of the expected file. It proves each gate rejects the input with
one `is_file()` call and no preceding `exists()` call.

## Validation

```powershell
python -m unittest -q tools.tests.test_tooling03_plugin_build_artifact_single_probe_performance_contract tools.zircon_export.tests.test_plugin_build
python -m unittest discover -s tools/zircon_export/tests -p 'test_plugin_build*.py' -q
```

Result: 18/18 tests passed.

An alternating 21-pair benchmark modeled a stable 250 microsecond metadata probe
for the native artifact and asset pack gates:

- p50: `1,010,500 ns -> 507,600 ns` (`49.767%` reduction, `1.99x` speedup).
- p95: `1,064,800 ns -> 523,200 ns` (`50.865%` reduction, `2.04x` speedup).
- Metadata operations: `4 -> 2` (`50.000%` reduction).
- Both paths retained checksum `2`.

## Acceptance

Accepted locally: both production paths retain their negative-file behavior, all
plugin build tests pass, metadata operations are halved, and p50/p95 improve by
approximately 50%. Coordinator validation remains asynchronous and should be
batched with other Tooling candidates before commit and push.
