# Tooling08 Plugin shader module hash cache

## Problem

A plugin manifest may publish the same shader source through several import paths.
`collect_shader_module_specs` validated each row and then reread and rehashed the
same file for every import path. Work therefore scaled with descriptor count times
source size rather than unique source files.

## Change

Shader module collection now keeps a per-manifest `Path -> SHA-256` cache. Path
validation still runs for every descriptor, preserving diagnostic order and
containment behavior. Only the immutable content digest is reused.

The performance contract creates 64 import paths for one source and proves that
all 64 output rows retain the digest while the source hash function runs once.

## Validation

```powershell
python -m unittest -q tools.tests.test_tooling08_plugin_shader_module_hash_cache_performance_contract tools.tests.test_zircon_build_plugin_shader_descriptor_owner_boundaries tools.tests.test_zircon_build_shader_prewarm tools.tests.test_zircon_build_shader_prewarm_acceptance_contract
```

Result: 53/53 tests passed.

An alternating 21-pair benchmark used a 1 MiB shader source referenced by 64
distinct import paths:

- p50: `129,307,000 ns -> 6,283,200 ns` (`95.141%` reduction, `20.58x` speedup).
- p95: `156,294,200 ns -> 11,316,300 ns` (`92.759%` reduction, `13.81x` speedup).
- File hashes per manifest: `64 -> 1` (`98.438%` reduction).
- Legacy and optimized paths both produced checksum `4,096`.

## Acceptance

Accepted locally: behavior and owner-boundary tests pass, source hashing is bounded
by unique source paths, p50 improves by more than 95%, and p95 improves by more
than 90%. Coordinator compile validation remains asynchronous and should be batched
with other Tooling candidates before commit and push.
