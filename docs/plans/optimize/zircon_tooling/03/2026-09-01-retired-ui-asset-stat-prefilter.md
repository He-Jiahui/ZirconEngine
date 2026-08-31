# Tooling03 Retired UI asset stat prefilter

## Problem

Repository-level retired UI validation called `Path.is_file()` for every entry in
the Editor, plugin, and runtime trees before checking whether its name could match
`.ui.toml` or `.v2.ui.toml`. Normal Rust, ZUI, asset, and directory entries therefore
caused filesystem metadata IO despite being lexically irrelevant.

## Change

The retired-suffix predicate now runs on the discovered path before both the file
metadata query and relative-path projection. Only matching candidates call
`is_file()` and `relative_to()`. Candidate ordering, directory rejection, suffix
selection, and diagnostics remain unchanged.

The performance contract creates 2,048 current `.zui` files and two retired files.
It proves the production scan returns the same ordered retired paths while reducing
both file metadata calls and relative-path projections from `2,050` to `2`.

## Validation

```powershell
python -m unittest -q tools.tests.test_tooling03_retired_ui_asset_stat_prefilter_performance_contract tools.zircon_export.tests.test_plugin_validate_retired_ui_assets
python -m unittest discover -s tools/zircon_export/tests -p 'test_plugin_validate*.py' -q
```

Result: 191/191 tests passed.

An alternating 21-pair benchmark used one physical tree with 4,096 current assets
and four retired candidates:

- p50: `250,381,800 ns -> 3,451,500 ns` (`98.622%` reduction, `72.54x` speedup).
- p95: `563,975,400 ns -> 7,909,000 ns` (`98.598%` reduction, `71.31x` speedup).
- File metadata calls: `4,100 -> 4` (`99.902%` reduction).
- Relative-path projections: `4,100 -> 4` (`99.902%` reduction).
- Both paths retained checksum `4`.

## Acceptance

Accepted locally: the full plugin-validation family passes, file metadata and path
projection work are bounded by retired-suffix candidates, and p50/p95 both improve
by more than 98%. Coordinator validation remains asynchronous and should be batched
with other Tooling candidates before commit and push.
