# Runtime DDS Cubemap Protocol Naming Acceptance

## Scope

- Owner: `zircon_runtime::asset::assets::texture::external_source_cubemap`.
- Plan: Runtime 15 naming and code-review priority.
- Change: replace generic legacy labels with the actual DDS caps2 protocol vocabulary.

## Invariants

1. `DDSCAPS2_CUBEMAP` and DX10 `DDS_RESOURCE_MISC_TEXTURECUBE` detection behavior is unchanged.
2. Duplicate caps2-plus-DX10 declarations remain rejected.
3. Caps2 cubemaps still require all six face flags.
4. No `legacy_cubemap`, `legacy caps2`, or `legacy cubemap` label remains in the owner.

## Evidence

| Check | Result |
|---|---|
| Test-first naming guard | passed, 1/1 |
| Scoped rustfmt and diff health | passed |
| Runtime naming asset-schema debt | reduced 10 to 2 |
| Hard-cutover runtime asset debt | reduced 4 to 2 |
| Runtime core-min library check | passed with existing warnings |

The slice is accepted. The two remaining asset debt locations are in the active Shader/PBR owner's shader-package importer and are intentionally not modified by this slice. Runtime 15 remains `in_progress`.
