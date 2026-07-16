---
related_code:
  - zircon_runtime/src/text/sdf/offline
  - zircon_runtime/src/text/font_sdf_build_tool
  - zircon_runtime/src/bin/zircon_font_sdf_bake
  - zircon_runtime/src/graphics/scene/scene_renderer/ui/font_asset.rs
  - zircon_runtime/src/text/sdf/font_bake/offline_source.rs
  - tools/zircon_build_font_sdf.py
implementation_files:
  - zircon_runtime/src/text/sdf/offline/artifact.rs
  - zircon_runtime/src/text/sdf/offline/codec.rs
  - zircon_runtime/src/text/sdf/offline/error.rs
  - zircon_runtime/src/text/sdf/offline/identity.rs
  - zircon_runtime/src/text/sdf/offline/path.rs
  - zircon_runtime/src/text/sdf/font_bake/offline_source.rs
  - zircon_runtime/src/text/font_sdf_build_tool/bake.rs
  - zircon_runtime/src/text/font_sdf_build_tool/pack.rs
  - zircon_runtime/src/text/font_sdf_build_tool/request.rs
  - zircon_runtime/src/bin/zircon_font_sdf_bake/args.rs
  - zircon_runtime/src/bin/zircon_font_sdf_bake/write.rs
plan_sources:
  - docs/plans/zircon_runtime/text/05-sdf-msdf-pipeline.md
  - docs/superpowers/specs/2026-07-13-runtime-zsdf-offline-bake-design.md
  - docs/superpowers/plans/2026-07-13-runtime-zsdf-offline-bake.md
tests:
  - text_sdf_variation_hash_is_order_stable_and_instance_sensitive
  - zircon_runtime/src/text/sdf/tests/offline.rs
  - zircon_runtime/src/text/sdf/font_bake/tests/offline.rs
  - zircon_runtime/tests/runtime_text_sdf_offline_artifact.rs
  - tools/tests/test_zircon_build_font_sdf.py
doc_type: module-detail
status: accepted
---

> 当前 owner 已硬切为 [`zircon_runtime::text`](../../text/mod.md)。本文件保留 offline SDF/MSDF/MTSDF 细节；`graphics::text` 旧 namespace 不再存在。

# Runtime offline SDF/MSDF/MTSDF artifacts

## Ownership and boundary

`text/sdf/offline/` owns the versioned `.zsdf` binary contract. It is the only decoder used by runtime prefill and the only encoder used by the feature-gated build tool. The offline path therefore shares `SdfBakeParams`, `SdfMode`, the fdsm glyph generator, and atlas storage rules with dynamic generation; Python and the CLI do not carry a second distance-field implementation.

The public surface is deliberately feature-gated at `zircon_runtime::text::font_sdf_build_tool` behind `font-sdf-build-tool`. Normal runtime builds keep the artifact representation crate-private. The former crate-root build-tool path was removed rather than forwarded, so the offline tool cannot add an unclassified Runtime crate-root seat. `tools/zircon_build.py` only selects the `font-sdf` target, while `tools/zircon_build_font_sdf.py` owns manifest validation and orchestration. The Rust binary owns font decoding, cmap selection, glyph generation, packing, artifact encoding, and atomic file replacement.

## `.zsdf` format contract

Version 1 is one deterministic binary with embedded atlas pages. Its fixed header records:

- magic and format version;
- canonical lowercase font asset UUID;
- face index, variation-instance BLAKE3 hash, and standalone face-source BLAKE3 hash;
- SDF/MSDF/MTSDF mode plus normalized bake em and spread;
- page size, page/glyph counts, section lengths, reserved bytes, and BLAKE3 checksum.

Page records and glyph records are fixed-width and sorted. Glyph records contain actual font glyph id, representative Unicode scalar, page index, packed rectangle, bearings, advance, and ascent. Page bytes use the runtime atlas storage contract: SDF is R8; MSDF and MTSDF are RGBA8. MTSDF alpha remains the true-distance channel.

Decode rejects unknown versions, invalid UUIDs or modes, non-zero reserved bytes, arithmetic overflow, inconsistent section lengths, trailing bytes, checksum mismatch, non-contiguous pages, duplicate glyph ids, invalid scalars, non-finite metrics, missing pages, and out-of-bounds rectangles. Runtime identity validation additionally requires exact UUID, face, variation hash, source hash, mode, bake em, and spread equality.

Runtime variation lookup now hashes the effective sorted design coordinates carried by the selected `InstancedFaceId`, after `fvar` bound/default normalization, OpenType normalized F2DOT14 quantization, and the real `wght` override. Default-valued coordinates preserve the historical empty-coordinate hash. A non-default coordinate set cannot reuse a default artifact and falls through to the coordinate-aware dynamic SDF/MSDF/MTSDF generator. The V1 build request still accepts a variation hash rather than coordinates; generating non-default offline outlines therefore remains open and the runtime does not infer coordinates from a hash.

The deterministic cache path is derived from the full normalized identity under the project library cache:

```text
<cache>/text/sdf/v1/<asset-uuid>/face_<index>/<variation-hash>/<mode>_<bake-em>_<spread-milli>.zsdf
```

The source hash remains in the checked header rather than the filename, so a source update replaces one logical cache entry instead of leaving unreachable historical files beside it.

No generated `.zsdf` file belongs in repository `target/` or `docs/tests/runtime/text/`; those locations respectively hold disposable build outputs and accepted visual framebuffer evidence.

## Build target

The `font-sdf` manifest accepts one or more bake records. Each record selects exactly one glyph source:

- `all_cmap: true` enumerates all Unicode cmap subtables;
- `codepoints` accepts individual `U+XXXX` scalars and inclusive `U+XXXX-U+YYYY` ranges.

Explicit selections expand in scalar order and deduplicate before the Rust CLI receives repeated `--codepoint` values. Rust then deduplicates aliases by glyph id, generates each outline once through the shared fdsm owner, packs deterministic shelf pages, and writes the versioned path beneath `cache_root`. SDF, MSDF, and MTSDF use the same command and differ only by mode and storage channels.

## Runtime precedence and failure behavior

The Text-owned SDF preparation state keeps this precedence:

1. return the existing mode-keyed in-memory glyph cache;
2. for the requested primary project font face, resolve the authoritative manifest `.zmeta` UUID and exact standalone face bytes;
3. load or reuse an identity-matched `.zsdf` artifact and copy the actual shaped glyph id's pixels/metrics;
4. if the artifact is absent, stale, corrupt, or does not cover the glyph, run the existing dynamic generator;
5. retain the existing typed per-glyph failure/native-overlay path if dynamic generation also fails.

Offline pixels are copied by Text into the existing R8/RGBA atlas-page stream. Graphics consumes the resulting pixel/metric report for the existing upload commands, WGPU texture arrays, draw pass, and shader decode mode. There is no offline-only GPU atlas, texture binding, renderer pass, layout cache, public UI DTO, or Graphics-owned font database/cache.

The accepted diagnostics add only `offline_glyph_count` and `dynamic_glyph_count` to the internal atlas bake report. These counts prove path selection without changing layout or exposing a permanent engine API.

## Validation state

SM-M3 passed Windows acceptance on 2026-07-13. Current-source artifact/renderer exact tests pass 6/6; the CLI range test passes 1/1; the feature-gated SDF/MSDF/MTSDF deterministic/decode/checksum integration passes 2/2; the Python font-SDF contracts pass 5/5 and the existing plugin/shader-prewarm build-tool regressions pass 45/45.

The real CLI baked the Fira Sans `A-C,g` subset twice for every mode. Each repeat was byte-identical: SDF 65,956 bytes with SHA256 `DD527C14A742BCFAFFC45D13887ECE61F64CA1C16AAE86D26ACA9BB7A3615902`; MSDF 262,564 bytes with `FEBF36209B76AE1C1F218A05BFADFD0656384406C164F972624A8982CAA8B132`; MTSDF 262,564 bytes with `EF8D1639E55276F39FF971148A7225A8CAEB12EDC5DF99B8D9A25D79131EA55A`. Temporary artifacts were removed after inspection.

Managed Windows target-client check job `1aaddc58899c4d77a23a88005614eb77` exited 0 in 11m44s. Coordinator finish/release removed its ephemeral target. Final hygiene found zero `.zsdf` files in repository `target/`, `docs/tests/runtime/text/`, the released managed target, and the temporary bake root. The Text05 numbered output archive is the canonical acceptance record.
