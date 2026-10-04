# ZirconEngine size-optimized products

`tools/zircon_build.py` owns the product build path. Development profiles keep
their existing `target-client`/`target-editor-host` behavior; a shipping build
selects a separate, build-only product profile and never changes the serialized
`RuntimeProfileId` contract.

## Supported products

The first platform is Windows x86_64 (`x86_64-pc-windows-msvc`):

| Profile | Product tree | Budget | Separate outputs |
| --- | --- | ---:| --- |
| `runtime-windows` | Host + Runtime DLL + runtime assets/manifests | 200,000,000 bytes (warning 300,000,000) | symbols, report, archive |
| `editor-windows` | Editor + Runtime DLL + manifests | 400,000,000 bytes (warning 500,000,000) | symbols, report, archive |

The budget is measured in decimal bytes. Cooked project packs are supplied with
`--project-pack-root` and reported separately; they are not copied into the
engine baseline. PDB/DBG/dSYM files are copied to `symbols/<profile>` and are
never accepted in the product tree.

## Build commands

```powershell
python tools/zircon_build.py `
  --product-profile runtime-windows `
  --out E:\ZirconBuilds\zircon-runtime-shipping `
  --mode shipping

python tools/zircon_build.py `
  --product-profile editor-windows `
  --out E:\ZirconBuilds\zircon-editor-shipping `
  --mode shipping
```

`--target-triple x86_64-pc-windows-msvc` is accepted explicitly and is checked
against the profile. Shipping uses Cargo's `shipping` profile (`opt-level=z`,
fat LTO, one codegen unit, aborting panics, stripped symbols). Use
`--mode shipping-symbols` with a matching custom profile when a symbol-bearing
diagnostic build is needed.

The Runtime tree is staged under `ZirconEngine`; Editor assets are staged under
`EditorAssets`. Runtime staging excludes editor fonts, UI caches and editor
resources. The development `zircon_runtime` Cargo target remains gated by
`target-client`; shipping uses the same source through the separate
`zircon_runtime_product` target gated by `runtime-product`, and publishes it as
`zircon_runtime.exe`. Hub, Editor and plugin targets cannot be combined with the
Runtime shipping profile.

## Auditing and reproducibility

Each product writes:

- `ZirconEngine/staging_manifest.json`, with file provenance, category, size and
  SHA-256;
- `reports/<profile>/product_size_report.json`, a `ProductSizeReportV1` with
  totals, warnings/hard-gate status, largest contributors and external symbol /
  project-pack references;
- `archives/<profile>.zip`, written with stable order, timestamp and mode bits.

Absolute checkout paths, source/cache paths, symbols and forbidden Hub/Editor
paths fail the publication gate. The static closure audit is available as:

```powershell
python tools/build/zircon_build_feature_closure.py `
  --app-features runtime-product `
  --runtime-features shipping-runtime
```

At this first implementation wave the report intentionally exposes the
remaining `dynamic-api -> text/UI/script/navigation/animation` closure as
`needs-split`; the explicit `graphics-core`, `runtime-api-core` and
`cooked-assets` aliases make that debt visible without silently claiming that
the heavy modules have already been removed. The feature-closure gate must be
`core-only` before the final 200/400 MB release gate is promoted.

Cargo validation remains Windows-native, `--locked`, and under a managed
approved drive-root target directory. Existing MVP/export failures are
reported separately from the product-size gate.
