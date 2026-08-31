# Tooling08 Plugin asset-root single probe

## Problem

Plugin asset-root discovery called `Path.exists()` and then `Path.is_dir()` for
every candidate. Existing directories therefore paid two metadata probes even
though `is_dir()` already returns false for missing paths and regular files.
Remote, antivirus-filtered, and cold filesystems amplify this redundant IO.

## Change

Asset-root filtering now calls only `Path.is_dir()`. The accepted set is unchanged:
directories are retained, while missing paths and non-directories are omitted.

The performance contract supplies 64 roots and counts the production metadata
calls, proving `128 -> 64` probes with identical ordered output.

## Validation

```powershell
python -m unittest -q tools.tests.test_tooling08_plugin_asset_root_single_probe_performance_contract tools.tests.test_zircon_build_plugin_asset_owner_boundaries
python -m unittest discover -s tools/tests -p 'test_zircon_build_plugin*.py' -q
```

Result: 24/24 tests passed.

An alternating 21-pair benchmark modeled a stable 250 microsecond metadata probe
across 64 asset roots:

- p50: `37,161,400 ns -> 18,591,800 ns` (`49.970%` reduction, `2.00x` speedup).
- p95: `79,664,900 ns -> 35,847,600 ns` (`54.999%` reduction, `2.22x` speedup).
- Metadata probes: `128 -> 64` (`50.000%` reduction).
- Both paths retained checksum `64`.

## Acceptance

Accepted locally: behavior and owner-boundary tests pass, metadata probes are
halved, and p50/p95 both improve by approximately 50% or more. Coordinator compile
validation remains asynchronous and should be batched with other Tooling candidates
before commit and push.
