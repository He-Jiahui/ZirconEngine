# Tooling08 Plugin workspace manifest single probe

## Problem

Plugin workspace discovery called `Path.exists()` before opening every member
`Cargo.toml`. The current plugin workspace contains 139 members, so all existing
manifests paid a redundant metadata query before the required open operation.

## Change

Discovery now opens each member manifest directly and skips only
`FileNotFoundError`. This preserves the prior missing-member behavior while keeping
permission, parsing, and other IO failures visible. Package ordering and projection
remain unchanged.

The performance contract proves that 64 existing manifests use 65 total TOML reads
(workspace plus members) and zero `exists()` calls. Separate cases lock missing-file
skip and permission-error propagation.

## Validation

```powershell
python -m unittest -q tools.tests.test_tooling08_plugin_workspace_manifest_single_probe_performance_contract tools.tests.test_zircon_build_plugin_catalog_owner_boundaries
python -m unittest discover -s tools/tests -p 'test_zircon_build_plugin*.py' -q
```

Result: 26/26 tests passed.

An alternating 21-pair benchmark modeled the current 139-member workspace with a
stable 250 microsecond metadata operation:

- p50: `71,302,200 ns -> 35,487,800 ns` (`50.229%` reduction, `2.01x` speedup).
- p95: `77,376,500 ns -> 38,794,700 ns` (`49.864%` reduction, `1.99x` speedup).
- Metadata operations: `278 -> 139` (`50.000%` reduction).
- Both paths retained checksum `139`.

## Acceptance

Accepted locally: behavior and owner-boundary tests pass, error visibility remains
explicit, metadata operations are halved, and p50/p95 improve by approximately
50%. Coordinator compile validation remains asynchronous and should be batched
with the other Tooling08 candidates before commit and push.
