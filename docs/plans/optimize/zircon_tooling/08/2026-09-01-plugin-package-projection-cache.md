# Tooling08 Plugin package projection cache

## Problem

`PluginPackage.native_dynamic_crates`, `rlib_static_crates`, and `carriers` are
immutable projections of the frozen package model. They were recomputed on every
access. Native build selection and execution query the same package repeatedly,
so a package with `N` crates paid repeated `O(N)` scans even though neither the
crate tuple nor its distribution policy can change.

## Change

The three projections now use `functools.cached_property`. Their public tuple
results and constructor contract remain unchanged. Each crate-filtering projection
is computed once per package and `carriers` reuses those cached tuples.

The performance contract uses an iteration-counting tuple and proves that eight
rounds of all three queries reduce crate scans from `32` to `2`.

## Validation

```powershell
python -m unittest -v tools.tests.test_tooling08_plugin_package_projection_cache_performance_contract tools.tests.test_zircon_build_plugin_catalog_owner_boundaries tools.tests.test_tooling32_plugin_selection_lazy_index_performance_contract
```

Result: 6/6 tests passed.

An alternating 21-pair benchmark used 512 crates and 64 rounds of all three
projections per sample:

- p50: `11,861,700 ns -> 242,200 ns` (`97.958%` reduction, `48.98x` speedup).
- p95: `14,161,900 ns -> 314,000 ns` (`97.783%` reduction, `45.10x` speedup).
- Crate scans per sample: `256 -> 2` (`99.219%` reduction).
- All legacy and optimized samples produced checksum `32,896`.

## Acceptance

Accepted locally: behavior contracts pass, repeated projection scans are bounded
to two per package, and both p50 and p95 improve by more than 95%. Coordinator
compile validation remains asynchronous and must be batched with other Tooling
candidates before commit and push.
